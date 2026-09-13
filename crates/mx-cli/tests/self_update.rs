//! `mx self-update` end to end, hermetically.
//!
//! Same house style as `cli_surface.rs`: `assert_cmd` against the real
//! binary, `HOME` redirected to a tempdir, a stub-bin `PATH` (so `brew` and
//! `codesign` are absent unless a test says otherwise), and wiremock standing
//! in for GitHub through `MX_RELEASES_API`.
//!
//! The test binary lives under the workspace's `target/`, which the kind
//! detector would classify as a source checkout, so every test pins the
//! install kind through `MX_SELFUPDATE_EXE` (the path used for detection
//! instead of `current_exe()`) and, for Homebrew, `HOMEBREW_PREFIX`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use mx_lib::selfupdate::Triple;
use mx_lib::test_support::{pack_bundle, sha256_sidecar, write_fake_bundle, StubBin};

const CURRENT: &str = env!("CARGO_PKG_VERSION");

/// A hermetic home plus the env every invocation needs.
struct Sandbox {
    home: tempfile::TempDir,
    stub: StubBin,
}

impl Sandbox {
    fn new() -> Self {
        Self {
            home: tempfile::tempdir().expect("home tempdir"),
            stub: StubBin::new(),
        }
    }

    fn home(&self) -> &Path {
        self.home.path()
    }

    fn mech_crate(&self) -> PathBuf {
        self.home().join(".mech-crate")
    }

    fn shim(&self, name: &str) -> PathBuf {
        self.home().join(".local/bin").join(name)
    }

    /// `mx` with the hermetic environment applied and no install kind chosen.
    fn mx(&self) -> Command {
        let mut cmd = Command::cargo_bin("mx").expect("mx binary built");
        cmd.env_clear()
            .env("HOME", self.home())
            .env("PATH", self.stub.path_env())
            .env("MX_NO_UPDATE_CHECK", "1")
            .env("TERM", "dumb");
        cmd
    }

    /// `mx` pretending to be a release install at `releases/mx-v<ver>/bin/mx`.
    fn mx_as_release(&self, installed: &str) -> Command {
        let exe = self
            .mech_crate()
            .join(format!("releases/mx-v{installed}/bin/mx"));
        let mut cmd = self.mx();
        cmd.env("MX_SELFUPDATE_EXE", exe);
        cmd
    }

    /// Snapshot of every path under home, for "changes nothing" assertions.
    fn tree(&self) -> Vec<String> {
        let mut out = Vec::new();
        for entry in walkdir::WalkDir::new(self.home()).sort_by_file_name() {
            let entry = entry.unwrap();
            let rel = entry.path().strip_prefix(self.home()).unwrap();
            let kind = if entry.path_is_symlink() {
                format!("-> {}", fs::read_link(entry.path()).unwrap().display())
            } else if entry.file_type().is_dir() {
                "dir".to_string()
            } else {
                format!("{} bytes", entry.metadata().unwrap().len())
            };
            out.push(format!("{} {}", rel.display(), kind));
        }
        out
    }

    fn current_target(&self) -> Option<PathBuf> {
        fs::read_link(self.mech_crate().join("current")).ok()
    }

    fn shim_version(&self) -> String {
        let out = StdCommand::new(self.shim("mx"))
            .arg("--version")
            .output()
            .expect("shim runs");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn version_file(&self) -> String {
        fs::read_to_string(self.mech_crate().join("version"))
            .unwrap_or_default()
            .trim()
            .to_string()
    }
}

/// A fake GitHub releases API whose `latest` is `version`, with assets that
/// resolve to `tarball` (served with `digest_body` as the sidecar).
async fn releases_api(version: &str, tarball: Option<(&Path, String)>) -> MockServer {
    let server = MockServer::start().await;
    let triple = Triple::host().unwrap().as_str();
    let asset = format!("mx-v{version}-{triple}.tar.gz");
    let body = serde_json::json!({
        "tag_name": format!("v{version}"),
        "draft": false,
        "prerelease": false,
        "assets": [
            {"name": asset, "browser_download_url": format!("{}/dl/{asset}", server.uri()), "size": 1},
            {"name": format!("{asset}.sha256"), "browser_download_url": format!("{}/dl/{asset}.sha256", server.uri()), "size": 1}
        ]
    });
    Mock::given(method("GET"))
        .and(path("/releases/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    if let Some((tar, sidecar)) = tarball {
        Mock::given(method("GET"))
            .and(path(format!("/dl/{asset}")))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(fs::read(tar).unwrap()))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path(format!("/dl/{asset}.sha256")))
            .respond_with(ResponseTemplate::new(200).set_body_string(sidecar))
            .mount(&server)
            .await;
    }
    server
}

fn host_asset(version: &str) -> String {
    format!("mx-v{version}-{}.tar.gz", Triple::host().unwrap().as_str())
}

// ── dry-run per install kind ────────────────────────────────────────────────

#[test]
fn dry_run_reports_a_source_install_and_changes_nothing() {
    let sb = Sandbox::new();
    let repo = sb.home().join("dev/mech-crate");
    fs::create_dir_all(repo.join("crates/mx-cli")).unwrap();
    fs::write(repo.join("Cargo.toml"), "[workspace]\n").unwrap();
    let before = sb.tree();

    sb.mx()
        .env("MX_SELFUPDATE_EXE", repo.join("target/release/mx"))
        .args(["self-update", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("source").and(contains(repo.to_str().unwrap())));

    assert_eq!(sb.tree(), before, "--dry-run must not touch the filesystem");
}

#[tokio::test(flavor = "multi_thread")]
async fn dry_run_reports_a_release_install_with_the_planned_download() {
    let sb = Sandbox::new();
    let api = releases_api("9.9.9", None).await;
    let before = sb.tree();

    sb.mx_as_release("0.1.0")
        .env("MX_RELEASES_API", api.uri())
        .args(["self-update", "--dry-run"])
        .assert()
        .success()
        .stdout(
            contains("release")
                .and(contains("9.9.9"))
                .and(contains(host_asset("9.9.9"))),
        );

    assert_eq!(sb.tree(), before);
}

#[test]
fn dry_run_reports_a_homebrew_install_and_the_brew_command() {
    let sb = Sandbox::new();
    let prefix = sb.home().join("brew");
    let exe = prefix.join("Cellar/mx/0.1.0/libexec/bin/mx");

    sb.mx()
        .env("HOMEBREW_PREFIX", &prefix)
        .env("MX_SELFUPDATE_EXE", &exe)
        .args(["self-update", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("homebrew").and(contains("brew upgrade mx")));
}

#[tokio::test(flavor = "multi_thread")]
async fn dry_run_reports_a_bare_install() {
    let sb = Sandbox::new();
    let api = releases_api("9.9.9", None).await;
    let exe = sb.home().join("usr/local/bin/mx");

    sb.mx()
        .env("MX_SELFUPDATE_EXE", &exe)
        .env("MX_RELEASES_API", api.uri())
        .args(["self-update", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("bare").and(contains("usr/local/bin/mx")));
}

// ── --check ─────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn check_exits_10_and_names_both_versions_when_newer_exists() {
    let sb = Sandbox::new();
    let api = releases_api("9.9.9", None).await;

    sb.mx_as_release("0.1.0")
        .env("MX_RELEASES_API", api.uri())
        .args(["self-update", "--check"])
        .assert()
        .code(10)
        .stdout(contains(CURRENT).and(contains("9.9.9")));
}

#[tokio::test(flavor = "multi_thread")]
async fn check_exits_0_when_up_to_date() {
    let sb = Sandbox::new();
    let api = releases_api(CURRENT, None).await;

    sb.mx_as_release(CURRENT)
        .env("MX_RELEASES_API", api.uri())
        .args(["self-update", "--check"])
        .assert()
        .success()
        .stdout(contains("up to date"));
}

// ── --from-dir and --rollback ───────────────────────────────────────────────

#[test]
fn from_dir_adopts_a_bundle_flips_current_and_installs_shims() {
    let sb = Sandbox::new();
    let staging = sb.home().join("staging");
    let bundle = write_fake_bundle(&staging, "9.9.9");

    sb.mx()
        .args(["self-update", "--from-dir"])
        .arg(&bundle)
        .arg("--yes")
        .assert()
        .success();

    assert_eq!(
        sb.current_target()
            .map(|p| p.file_name().unwrap().to_os_string()),
        Some("mx-v9.9.9".into()),
        "current must point at the adopted release"
    );
    assert_eq!(
        sb.shim_version(),
        "mx 9.9.9",
        "the ~/.local/bin shim runs the new mx"
    );
    assert!(sb.shim("mx-mcp").is_symlink());
    assert_eq!(sb.version_file(), "9.9.9");
    assert!(
        sb.mech_crate().join("templates/marker-9.9.9.txt").is_file(),
        "templates must be refreshed from the adopted bundle"
    );
    assert!(sb
        .mech_crate()
        .join("tmp")
        .read_dir()
        .map(|d| d.count() == 0)
        .unwrap_or(true));
}

#[test]
fn rollback_restores_the_previous_release() {
    let sb = Sandbox::new();
    let staging = sb.home().join("staging");
    let old = write_fake_bundle(&staging, "9.9.8");
    let new = write_fake_bundle(&staging, "9.9.9");

    for bundle in [&old, &new] {
        sb.mx()
            .args(["self-update", "--from-dir"])
            .arg(bundle)
            .arg("--yes")
            .assert()
            .success();
    }
    assert_eq!(sb.shim_version(), "mx 9.9.9");

    sb.mx_as_release("9.9.9")
        .args(["self-update", "--rollback", "--yes"])
        .assert()
        .success()
        .stdout(contains("9.9.8"));

    assert_eq!(sb.shim_version(), "mx 9.9.8");
    assert_eq!(sb.version_file(), "9.9.8");
    assert!(sb.mech_crate().join("templates/marker-9.9.8.txt").is_file());
}

// ── strategies that refuse ──────────────────────────────────────────────────

#[test]
fn homebrew_install_without_brew_on_path_fails_naming_the_command() {
    let sb = Sandbox::new();
    let prefix = sb.home().join("brew");
    let exe = prefix.join("Cellar/mx/0.1.0/libexec/bin/mx");

    sb.mx()
        .env("HOMEBREW_PREFIX", &prefix)
        .env("MX_SELFUPDATE_EXE", &exe)
        .args(["self-update", "--yes"])
        .assert()
        .code(1)
        .stderr(contains("brew upgrade mx"));
}

// ── the download path ───────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn a_bad_checksum_aborts_and_leaves_current_untouched() {
    let sb = Sandbox::new();
    let staging = sb.home().join("staging");
    let installed = write_fake_bundle(&staging, "9.9.8");
    sb.mx()
        .args(["self-update", "--from-dir"])
        .arg(&installed)
        .arg("--yes")
        .assert()
        .success();

    let newer = write_fake_bundle(&staging, "9.9.9");
    let (tar, _digest) = pack_bundle(&newer, &staging.join("out"), &host_asset("9.9.9"));
    let wrong = sha256_sidecar(&"0".repeat(64), &host_asset("9.9.9"));
    let api = releases_api("9.9.9", Some((&tar, wrong))).await;

    sb.mx_as_release("9.9.8")
        .env("MX_RELEASES_API", api.uri())
        .args(["self-update", "--yes"])
        .assert()
        .code(1)
        .stderr(contains("checksum"));

    assert_eq!(
        sb.current_target()
            .map(|p| p.file_name().unwrap().to_os_string()),
        Some("mx-v9.9.8".into())
    );
    assert!(!sb.mech_crate().join("releases/mx-v9.9.9").exists());
    assert_eq!(sb.shim_version(), "mx 9.9.8");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_verified_download_installs_and_flips_current() {
    let sb = Sandbox::new();
    let staging = sb.home().join("staging");
    let installed = write_fake_bundle(&staging, "9.9.8");
    sb.mx()
        .args(["self-update", "--from-dir"])
        .arg(&installed)
        .arg("--yes")
        .assert()
        .success();

    let newer = write_fake_bundle(&staging, "9.9.9");
    let (tar, digest) = pack_bundle(&newer, &staging.join("out"), &host_asset("9.9.9"));
    let sidecar = sha256_sidecar(&digest, &host_asset("9.9.9"));
    let api = releases_api("9.9.9", Some((&tar, sidecar))).await;

    sb.mx_as_release("9.9.8")
        .env("MX_RELEASES_API", api.uri())
        .args(["self-update", "--yes"])
        .assert()
        .success()
        .stdout(contains("9.9.9"));

    assert_eq!(sb.shim_version(), "mx 9.9.9");
    assert_eq!(sb.version_file(), "9.9.9");
    assert!(
        sb.mech_crate().join("releases/mx-v9.9.8").is_dir(),
        "previous kept for rollback"
    );
    assert!(sb.mech_crate().join("templates/marker-9.9.9.txt").is_file());
}

// ── reserved flag for the notifier ──────────────────────────────────────────

#[test]
fn refresh_cache_is_accepted_and_silent() {
    let sb = Sandbox::new();
    sb.mx()
        .args(["self-update", "--refresh-cache"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}
