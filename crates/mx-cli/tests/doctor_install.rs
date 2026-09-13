//! `mx doctor`'s Install block, hermetically (spec §3.3 last paragraph, §3.6).
//!
//! House style, same as `self_update.rs`: `assert_cmd` against the real
//! binary, `HOME` redirected to a tempdir, and a stub-bin `PATH` so doctor's
//! `docker`/`make` probes never reach the machine. The install kind is pinned
//! through `MX_SELFUPDATE_EXE` (the path the detector classifies instead of
//! `current_exe()`), because the test binary itself lives under the
//! workspace's `target/` and would otherwise always read as a source install.
//!
//! There is no wiremock here on purpose: doctor never talks to the network.
//! Everything it knows about the latest release comes from the notifier
//! cache at `<home>/cache/update-check.json`.

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use chrono::{Duration, Utc};
use predicates::prelude::*;
use predicates::str::contains;

use mx_lib::selfupdate::notify::{self, Cache};
use mx_lib::selfupdate::{parse, Version};
use mx_lib::test_support::StubBin;

const CURRENT: &str = env!("CARGO_PKG_VERSION");

/// A hermetic home plus the env every `mx doctor` invocation needs.
struct Sandbox {
    home: tempfile::TempDir,
    cwd: tempfile::TempDir,
    stub: StubBin,
}

impl Sandbox {
    fn new() -> Self {
        let stub = StubBin::new();
        // doctor probes `docker --version`, `docker info` and `make`; the
        // stubs keep it off the real daemon and off this machine's tools.
        stub.stub("docker", 0, "Docker version 99.0.0, build stub");
        stub.stub("make", 0, "");
        Self {
            home: tempfile::tempdir().expect("home tempdir"),
            cwd: tempfile::tempdir().expect("cwd tempdir"),
            stub,
        }
    }

    fn home(&self) -> &Path {
        self.home.path()
    }

    fn mech_crate(&self) -> PathBuf {
        self.home().join(".mech-crate")
    }

    /// `mx doctor` with the hermetic environment applied, run from a
    /// directory that is not a MechCrate project.
    ///
    /// `HOMEBREW_PREFIX` points at a directory that does not exist so kind
    /// detection never shells out to a real `brew`.
    fn doctor(&self) -> Command {
        let mut cmd = Command::cargo_bin("mx").expect("mx binary built");
        cmd.env_clear()
            .env("HOME", self.home())
            .env("PATH", self.stub.path_env())
            .env("HOMEBREW_PREFIX", self.home().join("no-brew"))
            .env("MX_NO_UPDATE_CHECK", "1")
            .env("TERM", "dumb")
            .current_dir(self.cwd.path())
            .arg("doctor");
        cmd
    }

    /// `mx doctor` pretending to be a release install at
    /// `<home>/.mech-crate/releases/mx-v<version>/bin/mx`.
    ///
    /// The file is really created: the detector canonicalizes both the exe
    /// and the home before comparing them, and on macOS a tempdir under
    /// `/var/folders` resolves to `/private/var/folders`, so an exe that does
    /// not exist (and therefore cannot be canonicalized) would no longer sit
    /// under the home and would read as a bare install.
    fn doctor_as_release(&self, version: &str) -> Command {
        let exe = self
            .mech_crate()
            .join(format!("releases/mx-v{version}/bin/mx"));
        fs::create_dir_all(exe.parent().expect("release bin dir")).expect("fake release bin dir");
        fs::write(&exe, "#!/bin/sh\n").expect("fake release binary");
        let mut cmd = self.doctor();
        cmd.env("MX_SELFUPDATE_EXE", exe);
        cmd
    }

    /// `mx doctor` pretending to be a Homebrew install in a fake Cellar.
    ///
    /// The exe is created for the same canonicalization reason as
    /// [`Sandbox::doctor_as_release`], and `HOMEBREW_PREFIX` is the seam that
    /// keeps kind detection off a real `brew`.
    fn doctor_as_homebrew(&self, version: &str) -> Command {
        let prefix = self.home().join("brew");
        let exe = prefix.join(format!("Cellar/mx/{version}/libexec/bin/mx"));
        fs::create_dir_all(exe.parent().expect("cellar bin dir")).expect("fake cellar bin dir");
        fs::write(&exe, "#!/bin/sh\n").expect("fake cellar binary");
        let mut cmd = self.doctor();
        cmd.env("HOMEBREW_PREFIX", prefix)
            .env("MX_SELFUPDATE_EXE", exe);
        cmd
    }

    /// A fake mech-crate checkout with a built binary under `target/release`,
    /// returning the canonicalized repo root (what doctor prints).
    fn fake_repo(&self) -> PathBuf {
        let repo = self.home().join("dev/mech-crate");
        fs::create_dir_all(repo.join("crates/mx-cli")).expect("fake crates/mx-cli");
        fs::create_dir_all(repo.join("target/release")).expect("fake target/release");
        fs::write(repo.join("Cargo.toml"), "[workspace]\n").expect("fake Cargo.toml");
        fs::write(repo.join("target/release/mx"), "#!/bin/sh\n").expect("fake mx binary");
        repo.canonicalize().expect("canonicalize fake repo")
    }

    /// Write `<home>/.mech-crate/version`.
    fn record_version(&self, version: &str) {
        let mech = self.mech_crate();
        fs::create_dir_all(&mech).expect("create fake mx home");
        fs::write(mech.join("version"), format!("{version}\n")).expect("write version file");
    }

    /// Seed the notifier cache as a check that just succeeded would.
    fn seed_cache(&self, latest: &str) {
        let now = Utc::now();
        let cache = Cache {
            checked_at: now,
            next_check_at: now + Duration::hours(24),
            latest: Some(v(latest)),
            current_at_check: v(CURRENT),
            hinted_at: None,
            hinted_version: None,
        };
        notify::write_cache(&self.mech_crate(), &cache).expect("seed the cache");
    }

    /// `PATH` with the release shim directory prepended.
    fn path_with_shim_dir(&self) -> String {
        format!(
            "{}:{}",
            self.home().join(".local/bin").display(),
            self.stub.path_env()
        )
    }
}

fn v(s: &str) -> Version {
    parse(s).expect("test version parses")
}

/// A version that is always newer than whatever the workspace is at.
const NEWER: &str = "9.9.9";

#[test]
fn a_source_checkout_reports_kind_source_with_the_repo_path() {
    let sb = Sandbox::new();
    let repo = sb.fake_repo();

    sb.doctor()
        .env("MX_SELFUPDATE_EXE", repo.join("target/release/mx"))
        .assert()
        .success()
        .stdout(
            contains("Install")
                .and(contains("Kind:"))
                .and(contains("source"))
                .and(contains(repo.display().to_string())),
        );
}

#[test]
fn a_release_layout_reports_kind_release_and_the_recorded_version() {
    let sb = Sandbox::new();
    sb.record_version(CURRENT);
    sb.seed_cache(CURRENT);

    sb.doctor_as_release(CURRENT).assert().success().stdout(
        contains("Kind:")
            .and(contains("release"))
            .and(contains(format!("mx-v{CURRENT}")))
            .and(contains(format!("Running:  {CURRENT}")))
            .and(contains(format!("Recorded: {CURRENT}")))
            .and(contains("up to date")),
    );
}

#[test]
fn a_seeded_cache_with_a_newer_release_reports_update_available() {
    let sb = Sandbox::new();
    sb.record_version(CURRENT);
    sb.seed_cache(NEWER);

    sb.doctor_as_release(CURRENT).assert().success().stdout(
        contains(format!("Latest:   {NEWER}"))
            .and(contains("update available"))
            .and(contains("mx self-update")),
    );
}

#[test]
fn a_recorded_version_that_differs_from_the_running_build_warns() {
    let sb = Sandbox::new();
    sb.record_version("0.0.1");

    sb.doctor_as_release(CURRENT).assert().success().stdout(
        contains("Recorded: 0.0.1")
            .and(contains(format!("differs from running {CURRENT}")))
            .and(contains("run: mx self-update")),
    );
}

#[test]
fn a_missing_version_file_points_at_mx_init() {
    let sb = Sandbox::new();

    sb.doctor_as_release(CURRENT)
        .assert()
        .success()
        .stdout(contains("Recorded: (none)").and(contains("run: mx init")));
}

#[test]
fn without_a_cache_the_latest_release_is_not_checked_yet() {
    let sb = Sandbox::new();
    sb.record_version(CURRENT);

    sb.doctor_as_release(CURRENT)
        .assert()
        .success()
        .stdout(contains("Latest:   not checked yet"));
}

#[test]
fn a_release_install_warns_when_the_shim_dir_is_off_path() {
    let sb = Sandbox::new();
    sb.record_version(CURRENT);

    sb.doctor_as_release(CURRENT).assert().success().stdout(
        contains(format!(
            "{} is not on PATH",
            sb.home().join(".local/bin").display()
        ))
        .and(contains("export PATH=")),
    );
}

#[test]
fn a_release_install_is_quiet_when_the_shim_dir_is_on_path() {
    let sb = Sandbox::new();
    sb.record_version(CURRENT);

    sb.doctor_as_release(CURRENT)
        .env("PATH", sb.path_with_shim_dir())
        .assert()
        .success()
        .stdout(contains("is not on PATH").not());
}

/// The PATH warning belongs to the release layout only — a source checkout
/// is run from its own `target/release`, and `~/.local/bin` is irrelevant.
#[test]
fn a_source_checkout_never_gets_the_shim_path_warning() {
    let sb = Sandbox::new();
    let repo = sb.fake_repo();

    sb.doctor()
        .env("MX_SELFUPDATE_EXE", repo.join("target/release/mx"))
        .assert()
        .success()
        .stdout(contains("is not on PATH").not());
}

/// Homebrew owns its own installs: both the "you are out of step" and the
/// "there is a newer one" nudges must name brew's commands, not mx's.
#[test]
fn a_homebrew_install_points_at_brew_and_mx_init() {
    let sb = Sandbox::new();
    sb.record_version("0.0.1");
    sb.seed_cache(NEWER);

    sb.doctor_as_homebrew(CURRENT).assert().success().stdout(
        contains("Kind:")
            .and(contains("homebrew"))
            .and(contains("run: mx init --update"))
            .and(contains("update available: brew upgrade mx"))
            .and(contains("is not on PATH").not()),
    );
}
