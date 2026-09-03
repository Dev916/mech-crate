//! The passive update notifier, end to end (spec §3.6).
//!
//! House style, same as `self_update.rs`: `assert_cmd` against the real
//! binary, `HOME` in a tempdir, a stub-bin `PATH`, wiremock behind
//! `MX_RELEASES_API`. The one difference is deliberate: these tests do **not**
//! set `MX_NO_UPDATE_CHECK`, because the notifier is what is under test.
//!
//! stderr is a pipe under `assert_cmd`, so the TTY branch is reached through
//! the hidden `MX_UPDATE_CHECK_TTY` seam rather than by allocating a pty; the
//! decision itself is unit-tested in `mx_lib::selfupdate::notify`.

use std::path::PathBuf;
use std::time::{Duration as StdDuration, Instant};

use assert_cmd::Command;
use chrono::{DateTime, Duration, Utc};
use predicates::prelude::*;
use predicates::str::contains;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use mx_lib::selfupdate::notify::{self, Cache};
use mx_lib::selfupdate::Version;
use mx_lib::test_support::StubBin;

const CURRENT: &str = env!("CARGO_PKG_VERSION");

/// A port nothing listens on: the refresh fails immediately, no timeout wait.
const UNREACHABLE_API: &str = "http://127.0.0.1:9";

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

    fn mech_crate(&self) -> PathBuf {
        self.home.path().join(".mech-crate")
    }

    fn cache_path(&self) -> PathBuf {
        notify::cache_path(&self.mech_crate())
    }

    /// `mx` with the hermetic environment applied — and no update opt-out.
    fn mx(&self) -> Command {
        let mut cmd = Command::cargo_bin("mx").expect("mx binary built");
        cmd.env_clear()
            .env("HOME", self.home.path())
            .env("PATH", self.stub.path_env())
            .env("TERM", "dumb");
        cmd
    }

    /// `mx` believing stderr is a terminal.
    fn mx_tty(&self) -> Command {
        let mut cmd = self.mx();
        cmd.env("MX_UPDATE_CHECK_TTY", "1");
        cmd
    }

    fn seed(&self, cache: &Cache) {
        notify::write_cache(&self.mech_crate(), cache).expect("seed the cache");
    }

    fn cache(&self) -> Option<Cache> {
        notify::read_cache(&self.mech_crate())
    }

    fn raw_cache(&self) -> Option<Vec<u8>> {
        std::fs::read(self.cache_path()).ok()
    }

    fn write_config(&self, body: &str) {
        let path = self.mech_crate().join(notify::CONFIG_FILE);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }
}

fn v(s: &str) -> Version {
    mx_lib::selfupdate::parse(s).expect("test version parses")
}

/// A cache whose next check is a day overdue and whose `latest` is newer
/// than the running build.
fn stale_cache(latest: &str) -> Cache {
    let now = Utc::now();
    Cache {
        checked_at: now - Duration::hours(48),
        next_check_at: now - Duration::hours(24),
        latest: Some(v(latest)),
        current_at_check: v(CURRENT),
        hinted_at: None,
        hinted_version: None,
    }
}

/// Assert `at` is within five minutes of `Utc::now() + offset`.
fn assert_near(at: DateTime<Utc>, offset: Duration, what: &str) {
    let expected = Utc::now() + offset;
    let slack = (at - expected).num_seconds().abs();
    assert!(slack < 300, "{what}: {at} is not near {expected}");
}

/// Block until `done` or the deadline, polling the cache file.
fn wait_for(timeout: StdDuration, done: impl Fn() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if done() {
            return true;
        }
        std::thread::sleep(StdDuration::from_millis(25));
    }
    done()
}

/// A release channel that is up but broken: every request is answered with
/// a 500 at once. The refresh takes the failure path deterministically, with
/// none of the timing a closed port has on some CI runners (a connect that
/// hangs until the 5 s budget expires instead of being refused).
async fn failing_api() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/releases/latest"))
        .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
        .mount(&server)
        .await;
    server
}

/// A fake GitHub releases API whose `latest` is `version`.
async fn releases_api(version: &str) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/releases/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tag_name": format!("v{version}"),
            "draft": false,
            "prerelease": false,
            "assets": []
        })))
        .mount(&server)
        .await;
    server
}

// ── the hook on the hot path ────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn a_stale_cache_on_a_tty_hints_once_and_refreshes_in_the_background() {
    let sb = Sandbox::new();
    let api = failing_api().await;
    sb.seed(&stale_cache("9.9.9"));

    sb.mx_tty()
        .env("MX_RELEASES_API", api.uri())
        .arg("doctor")
        .assert()
        .success()
        .stderr(contains(format!(
            "mx 9.9.9 is available (you have {CURRENT}). Run: mx self-update"
        )));

    // The hint is recorded so it is not repeated for a day.
    let after_hint = sb.cache().expect("cache still readable");
    assert_eq!(after_hint.hinted_version, Some(v("9.9.9")));
    assert_near(
        after_hint.hinted_at.expect("hinted_at recorded"),
        Duration::zero(),
        "hinted_at",
    );

    // The detached refresh ran: the channel answered 500, so it backed off
    // by an hour rather than a day. The window is generous on purpose: the
    // refresh is an instrumented, freshly started process on a loaded CI
    // runner; a working refresh lands in well under a second.
    let refreshed = wait_for(StdDuration::from_secs(30), || {
        sb.cache()
            .is_some_and(|c| c.checked_at > after_hint.checked_at)
    });
    assert!(refreshed, "the detached refresh never rewrote the cache");
    let c = sb.cache().unwrap();
    assert_near(c.next_check_at, notify::OFFLINE_BACKOFF, "next_check_at");
    assert_eq!(c.latest, Some(v("9.9.9")), "a failed refresh keeps latest");
    assert_eq!(
        c.hinted_version,
        Some(v("9.9.9")),
        "a refresh keeps the recorded hint"
    );
}

#[test]
fn a_second_run_within_the_hint_ttl_stays_quiet() {
    let sb = Sandbox::new();
    let mut cache = stale_cache("9.9.9");
    cache.next_check_at = Utc::now() + Duration::hours(24);
    cache.hinted_at = Some(Utc::now());
    cache.hinted_version = Some(v("9.9.9"));
    sb.seed(&cache);
    let before = sb.raw_cache();

    sb.mx_tty()
        .env("MX_RELEASES_API", UNREACHABLE_API)
        .arg("doctor")
        .assert()
        .success()
        .stderr(contains("is available").not());

    assert_eq!(sb.raw_cache(), before, "nothing to say, nothing to write");
}

#[test]
fn without_a_tty_nothing_is_printed_and_nothing_is_spawned() {
    let sb = Sandbox::new();
    sb.seed(&stale_cache("9.9.9"));
    let before = sb.raw_cache();

    sb.mx()
        .env("MX_RELEASES_API", UNREACHABLE_API)
        .arg("doctor")
        .assert()
        .success()
        .stderr(contains("is available").not());

    assert_unchanged(&sb, before);
}

#[test]
fn the_env_opt_out_silences_the_notifier() {
    let sb = Sandbox::new();
    sb.seed(&stale_cache("9.9.9"));
    let before = sb.raw_cache();

    sb.mx_tty()
        .env("MX_NO_UPDATE_CHECK", "1")
        .env("MX_RELEASES_API", UNREACHABLE_API)
        .arg("doctor")
        .assert()
        .success()
        .stderr(contains("is available").not());

    assert_unchanged(&sb, before);
}

#[test]
fn ci_silences_the_notifier() {
    let sb = Sandbox::new();
    sb.seed(&stale_cache("9.9.9"));
    let before = sb.raw_cache();

    sb.mx_tty()
        .env("CI", "1")
        .env("MX_RELEASES_API", UNREACHABLE_API)
        .arg("doctor")
        .assert()
        .success()
        .stderr(contains("is available").not());

    assert_unchanged(&sb, before);
}

#[test]
fn the_config_opt_out_silences_the_notifier() {
    let sb = Sandbox::new();
    sb.seed(&stale_cache("9.9.9"));
    sb.write_config("check = false\n");
    let before = sb.raw_cache();

    sb.mx_tty()
        .env("MX_RELEASES_API", UNREACHABLE_API)
        .arg("doctor")
        .assert()
        .success()
        .stderr(contains("is available").not());

    assert_unchanged(&sb, before);
}

#[tokio::test(flavor = "multi_thread")]
async fn self_update_subcommands_never_run_the_notifier() {
    let sb = Sandbox::new();
    sb.seed(&stale_cache("9.9.9"));
    let before = sb.raw_cache();
    let api = releases_api("9.9.9").await;

    sb.mx_tty()
        .env("MX_RELEASES_API", api.uri())
        .env("MX_SELFUPDATE_EXE", sb.home.path().join("usr/local/bin/mx"))
        .args(["self-update", "--check"])
        .assert()
        .code(10)
        .stderr(contains("is available").not());

    assert_unchanged(&sb, before);
}

#[test]
fn mcp_subcommands_never_run_the_notifier() {
    let sb = Sandbox::new();
    sb.seed(&stale_cache("9.9.9"));
    let before = sb.raw_cache();

    sb.mx_tty()
        .env("MX_RELEASES_API", UNREACHABLE_API)
        .args(["mcp", "info"])
        .assert()
        .success()
        .stderr(contains("is available").not());

    assert_unchanged(&sb, before);
}

/// The cache is byte-identical after a settling window — proof that neither
/// the foreground hook nor a detached child touched it. A refresh against an
/// unreachable port finishes in milliseconds, so a second is generous.
fn assert_unchanged(sb: &Sandbox, before: Option<Vec<u8>>) {
    std::thread::sleep(StdDuration::from_millis(1000));
    assert_eq!(sb.raw_cache(), before, "the cache must not change");
}

// ── mx self-update --refresh-cache ──────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn refresh_cache_records_the_latest_release_silently() {
    let sb = Sandbox::new();
    let api = releases_api("9.9.9").await;

    sb.mx()
        .env("MX_RELEASES_API", api.uri())
        .args(["self-update", "--refresh-cache"])
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let c = sb.cache().expect("the refresh wrote a cache");
    assert_eq!(c.latest, Some(v("9.9.9")));
    assert_eq!(c.current_at_check, v(CURRENT));
    assert_near(c.checked_at, Duration::zero(), "checked_at");
    assert_near(c.next_check_at, notify::CHECK_TTL, "next_check_at");
}

#[test]
fn refresh_cache_backs_off_an_hour_when_the_channel_is_unreachable() {
    let sb = Sandbox::new();

    sb.mx()
        .env("MX_RELEASES_API", UNREACHABLE_API)
        .args(["self-update", "--refresh-cache"])
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let c = sb.cache().expect("a failed refresh still writes a cache");
    assert_eq!(c.latest, None);
    assert_near(c.next_check_at, notify::OFFLINE_BACKOFF, "next_check_at");
}

#[test]
fn a_failed_refresh_keeps_what_the_last_good_check_found() {
    let sb = Sandbox::new();
    let mut seeded = stale_cache("9.9.9");
    seeded.hinted_at = Some(Utc::now() - Duration::hours(2));
    seeded.hinted_version = Some(v("9.9.9"));
    sb.seed(&seeded);

    sb.mx()
        .env("MX_RELEASES_API", UNREACHABLE_API)
        .args(["self-update", "--refresh-cache"])
        .assert()
        .success();

    let c = sb.cache().expect("cache still there");
    assert_eq!(c.latest, Some(v("9.9.9")));
    assert_eq!(c.hinted_version, Some(v("9.9.9")));
    assert_eq!(c.hinted_at, seeded.hinted_at);
    assert_near(c.next_check_at, notify::OFFLINE_BACKOFF, "next_check_at");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_successful_refresh_carries_the_recorded_hint_forward() {
    let sb = Sandbox::new();
    let mut seeded = stale_cache("9.9.8");
    seeded.hinted_at = Some(Utc::now() - Duration::hours(2));
    seeded.hinted_version = Some(v("9.9.8"));
    sb.seed(&seeded);
    let api = releases_api("9.9.9").await;

    sb.mx()
        .env("MX_RELEASES_API", api.uri())
        .args(["self-update", "--refresh-cache"])
        .assert()
        .success();

    let c = sb.cache().expect("cache still there");
    assert_eq!(c.latest, Some(v("9.9.9")));
    assert_eq!(c.hinted_version, Some(v("9.9.8")), "hint history survives");
    assert_eq!(c.hinted_at, seeded.hinted_at);
}
