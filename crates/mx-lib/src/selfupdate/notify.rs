//! Passive update notification (spec §3.6).
//!
//! The half of self-update nobody has to remember to run. Every `mx`
//! invocation pays one small file read: the update-check cache at
//! `<home>/cache/update-check.json`. [`decide`] turns that cache plus the
//! ambient facts ([`Context`]) into an [`Action`] — nothing here talks to the
//! network, and nothing here blocks. The caller spawns
//! `mx self-update --refresh-cache` detached when told to, and prints
//! [`hint_line`] on stderr when told to.
//!
//! Everything except [`read_cache`], [`write_cache`] and [`config_disables`]
//! is pure, so the whole policy is unit-testable against literal clocks.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::selfupdate::version::{is_newer, Version};

/// How long a successful check stays good before a refresh is spawned.
pub const CHECK_TTL: Duration = Duration::hours(24);

/// How long a *failed* check (offline, rate limited, anything) is honoured,
/// so an offline laptop forks at most one refresh an hour.
pub const OFFLINE_BACKOFF: Duration = Duration::hours(1);

/// How long the same "x is available" line stays suppressed once printed.
pub const HINT_TTL: Duration = Duration::hours(24);

/// Environment variable that disables the check when set to anything
/// non-empty.
pub const DISABLE_ENV: &str = "MX_NO_UPDATE_CHECK";

/// Config file, relative to the MechCrate home, whose `check = false`
/// disables the notifier.
pub const CONFIG_FILE: &str = "config/update.toml";

/// Cache file, relative to the MechCrate home.
pub const CACHE_FILE: &str = "cache/update-check.json";

/// The one file on the hot path: `<home>/cache/update-check.json`.
pub fn cache_path(home: &Path) -> PathBuf {
    home.join(CACHE_FILE)
}

/// What the last release-channel check found, and what has been said about it.
///
/// Versions are stored as strings so the JSON stays readable and stable;
/// timestamps are RFC 3339 UTC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cache {
    /// When the last check ran (successfully or not).
    pub checked_at: DateTime<Utc>,
    /// When the next check becomes due: `checked_at + CHECK_TTL` after a
    /// success, `checked_at + OFFLINE_BACKOFF` after a failure.
    pub next_check_at: DateTime<Utc>,
    /// Newest published release, when a check has ever succeeded.
    #[serde(default, with = "opt_version_string")]
    pub latest: Option<Version>,
    /// The running version at the time of the check, for debugging a cache
    /// written by a different binary.
    #[serde(with = "version_string")]
    pub current_at_check: Version,
    /// When the hint was last printed, if ever.
    #[serde(default)]
    pub hinted_at: Option<DateTime<Utc>>,
    /// The version the last printed hint was about.
    #[serde(default, with = "opt_version_string")]
    pub hinted_version: Option<Version>,
}

/// Read the cache, treating "missing" and "corrupt" alike: [`None`].
///
/// Never an error — a broken cache must not break the command the user
/// actually ran; the next refresh overwrites it.
pub fn read_cache(home: &Path) -> Option<Cache> {
    let raw = std::fs::read_to_string(cache_path(home)).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Write the cache, creating `<home>/cache` first.
///
/// The write goes to a per-process temp file and is renamed over the target,
/// so a reader on the hot path never sees a half-written file and two
/// concurrent writers never interleave into one.
pub fn write_cache(home: &Path, cache: &Cache) -> Result<()> {
    let path = cache_path(home);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    std::fs::write(&tmp, serde_json::to_vec_pretty(cache)?)?;
    match std::fs::rename(&tmp, &path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(e.into())
        }
    }
}

/// True when `<home>/config/update.toml` says `check = false`.
///
/// Anything else — missing file, unreadable file, invalid TOML, a `check` of
/// another type — leaves the notifier enabled.
pub fn config_disables(home: &Path) -> bool {
    let Ok(raw) = std::fs::read_to_string(home.join(CONFIG_FILE)) else {
        return false;
    };
    let Ok(value) = raw.parse::<toml::Value>() else {
        return false;
    };
    value.get("check").and_then(toml::Value::as_bool) == Some(false)
}

/// The ambient facts [`decide`] needs, gathered by the caller so the decision
/// itself stays pure.
#[derive(Debug, Clone)]
pub struct Context<'a> {
    /// Now, as the caller sees it.
    pub now: DateTime<Utc>,
    /// The running version.
    pub current: &'a Version,
    /// Whether stderr is a terminal — a hint is only ever printed to a human.
    pub stderr_is_tty: bool,
    /// [`DISABLE_ENV`] is set to something non-empty.
    pub disabled_by_env: bool,
    /// `CI` is set.
    pub disabled_by_ci: bool,
    /// [`config_disables`] said so.
    pub disabled_by_config: bool,
}

impl Context<'_> {
    /// Any opt-out, or no terminal to print to.
    fn silenced(&self) -> bool {
        !self.stderr_is_tty
            || self.disabled_by_env
            || self.disabled_by_ci
            || self.disabled_by_config
    }
}

/// What the caller should do after the command it ran has finished.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Do nothing at all.
    Silent,
    /// Spawn a detached `mx self-update --refresh-cache`.
    Spawn,
    /// Print [`hint_line`] for this version on stderr.
    Hint(Version),
    /// Both: the cache is stale *and* what it already holds is newer.
    SpawnAndHint(Version),
}

/// Turn the cache plus the ambient facts into an [`Action`]. Pure.
///
/// A refresh is due when there is no cache at all or `now` has reached
/// `next_check_at`. A hint is due when the cache knows of a version newer
/// than the running one and that exact version has not been hinted within
/// [`HINT_TTL`].
pub fn decide(cache: Option<&Cache>, ctx: &Context) -> Action {
    if ctx.silenced() {
        return Action::Silent;
    }
    let spawn = match cache {
        None => true,
        Some(c) => ctx.now >= c.next_check_at,
    };
    let hint = cache.and_then(|c| hint_due(c, ctx));
    match (spawn, hint) {
        (true, Some(v)) => Action::SpawnAndHint(v),
        (true, None) => Action::Spawn,
        (false, Some(v)) => Action::Hint(v),
        (false, None) => Action::Silent,
    }
}

/// The version to hint about, if the cache holds a newer one that has not
/// already been announced within [`HINT_TTL`].
fn hint_due(cache: &Cache, ctx: &Context) -> Option<Version> {
    let latest = cache.latest.as_ref()?;
    if !is_newer(latest, ctx.current) {
        return None;
    }
    let suppressed = match (&cache.hinted_at, &cache.hinted_version) {
        (Some(at), Some(v)) => v == latest && ctx.now - *at < HINT_TTL,
        _ => false,
    };
    (!suppressed).then(|| latest.clone())
}

/// The single stderr line the notifier prints.
///
/// Homebrew installs are told to use `brew`, because `mx self-update` would
/// only redirect them there anyway.
pub fn hint_line(latest: &Version, current: &Version, homebrew: bool) -> String {
    let command = if homebrew {
        "brew upgrade mx"
    } else {
        "mx self-update"
    };
    format!("mx {latest} is available (you have {current}). Run: {command}")
}

/// serde for a [`Version`] stored as a string.
mod version_string {
    use super::Version;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Version, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&v.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Version, D::Error> {
        let raw = String::deserialize(d)?;
        crate::selfupdate::version::parse(&raw).map_err(serde::de::Error::custom)
    }
}

/// serde for an optional [`Version`] stored as a string or `null`.
mod opt_version_string {
    use super::Version;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Option<Version>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            Some(v) => s.serialize_str(&v.to_string()),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Version>, D::Error> {
        match Option::<String>::deserialize(d)? {
            Some(raw) => crate::selfupdate::version::parse(&raw)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn v(s: &str) -> Version {
        crate::selfupdate::version::parse(s).expect("test version parses")
    }

    fn t0() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 3, 12, 0, 0).unwrap()
    }

    /// A cache whose `next_check_at` is `CHECK_TTL` after `t0` (still fresh).
    fn fresh(latest: Option<&str>) -> Cache {
        Cache {
            checked_at: t0(),
            next_check_at: t0() + CHECK_TTL,
            latest: latest.map(v),
            current_at_check: v("0.1.1"),
            hinted_at: None,
            hinted_version: None,
        }
    }

    /// A cache whose `next_check_at` has already passed at `t0`.
    fn stale(latest: Option<&str>) -> Cache {
        Cache {
            checked_at: t0() - CHECK_TTL - CHECK_TTL,
            next_check_at: t0() - CHECK_TTL,
            latest: latest.map(v),
            current_at_check: v("0.1.1"),
            hinted_at: None,
            hinted_version: None,
        }
    }

    fn ctx(current: &Version, tty: bool) -> Context<'_> {
        Context {
            now: t0(),
            current,
            stderr_is_tty: tty,
            disabled_by_env: false,
            disabled_by_ci: false,
            disabled_by_config: false,
        }
    }

    // ── decide: cache freshness × latest × tty ─────────────────────────────

    #[test]
    fn no_cache_on_a_tty_spawns_a_refresh() {
        let cur = v("0.1.1");
        assert_eq!(decide(None, &ctx(&cur, true)), Action::Spawn);
    }

    #[test]
    fn no_cache_without_a_tty_is_silent() {
        let cur = v("0.1.1");
        assert_eq!(decide(None, &ctx(&cur, false)), Action::Silent);
    }

    #[test]
    fn a_fresh_cache_with_a_newer_latest_hints_without_spawning() {
        let cur = v("0.1.1");
        assert_eq!(
            decide(Some(&fresh(Some("0.1.3"))), &ctx(&cur, true)),
            Action::Hint(v("0.1.3"))
        );
    }

    #[test]
    fn a_fresh_cache_with_a_newer_latest_is_silent_without_a_tty() {
        let cur = v("0.1.1");
        assert_eq!(
            decide(Some(&fresh(Some("0.1.3"))), &ctx(&cur, false)),
            Action::Silent
        );
    }

    #[test]
    fn a_fresh_cache_with_the_same_version_is_silent() {
        let cur = v("0.1.1");
        assert_eq!(
            decide(Some(&fresh(Some("0.1.1"))), &ctx(&cur, true)),
            Action::Silent
        );
    }

    #[test]
    fn a_fresh_cache_with_an_older_latest_is_silent() {
        let cur = v("0.1.1");
        assert_eq!(
            decide(Some(&fresh(Some("0.1.0"))), &ctx(&cur, true)),
            Action::Silent
        );
    }

    #[test]
    fn a_fresh_cache_with_no_latest_is_silent() {
        let cur = v("0.1.1");
        assert_eq!(decide(Some(&fresh(None)), &ctx(&cur, true)), Action::Silent);
    }

    #[test]
    fn a_stale_cache_with_a_newer_latest_spawns_and_hints() {
        let cur = v("0.1.1");
        assert_eq!(
            decide(Some(&stale(Some("0.1.3"))), &ctx(&cur, true)),
            Action::SpawnAndHint(v("0.1.3"))
        );
    }

    #[test]
    fn a_stale_cache_with_a_newer_latest_is_silent_without_a_tty() {
        let cur = v("0.1.1");
        assert_eq!(
            decide(Some(&stale(Some("0.1.3"))), &ctx(&cur, false)),
            Action::Silent
        );
    }

    #[test]
    fn a_stale_cache_with_the_same_version_only_spawns() {
        let cur = v("0.1.1");
        assert_eq!(
            decide(Some(&stale(Some("0.1.1"))), &ctx(&cur, true)),
            Action::Spawn
        );
    }

    #[test]
    fn a_stale_cache_with_no_latest_only_spawns() {
        let cur = v("0.1.1");
        assert_eq!(decide(Some(&stale(None)), &ctx(&cur, true)), Action::Spawn);
    }

    #[test]
    fn a_stale_cache_with_no_latest_is_silent_without_a_tty() {
        let cur = v("0.1.1");
        assert_eq!(
            decide(Some(&stale(None)), &ctx(&cur, false)),
            Action::Silent
        );
    }

    #[test]
    fn a_cache_due_exactly_now_is_stale() {
        let cur = v("0.1.1");
        let mut c = fresh(Some("0.1.1"));
        c.next_check_at = t0();
        assert_eq!(decide(Some(&c), &ctx(&cur, true)), Action::Spawn);
    }

    #[test]
    fn a_pre_release_latest_does_not_beat_its_release() {
        let cur = v("0.1.1");
        assert_eq!(
            decide(Some(&fresh(Some("0.1.1-rc.1"))), &ctx(&cur, true)),
            Action::Silent
        );
    }

    // ── decide: opt-outs ───────────────────────────────────────────────────

    #[test]
    fn the_env_opt_out_silences_everything() {
        let cur = v("0.1.1");
        let mut c = ctx(&cur, true);
        c.disabled_by_env = true;
        assert_eq!(decide(Some(&stale(Some("0.1.3"))), &c), Action::Silent);
        assert_eq!(decide(None, &c), Action::Silent);
    }

    #[test]
    fn ci_silences_everything() {
        let cur = v("0.1.1");
        let mut c = ctx(&cur, true);
        c.disabled_by_ci = true;
        assert_eq!(decide(Some(&stale(Some("0.1.3"))), &c), Action::Silent);
        assert_eq!(decide(None, &c), Action::Silent);
    }

    #[test]
    fn the_config_opt_out_silences_everything() {
        let cur = v("0.1.1");
        let mut c = ctx(&cur, true);
        c.disabled_by_config = true;
        assert_eq!(decide(Some(&stale(Some("0.1.3"))), &c), Action::Silent);
        assert_eq!(decide(None, &c), Action::Silent);
    }

    // ── decide: hint suppression ───────────────────────────────────────────

    #[test]
    fn the_same_version_hinted_within_the_ttl_is_not_hinted_again() {
        let cur = v("0.1.1");
        let mut c = fresh(Some("0.1.3"));
        c.hinted_at = Some(t0() - CHECK_TTL / 2);
        c.hinted_version = Some(v("0.1.3"));
        assert_eq!(decide(Some(&c), &ctx(&cur, true)), Action::Silent);
    }

    #[test]
    fn a_suppressed_hint_still_lets_a_stale_cache_spawn() {
        let cur = v("0.1.1");
        let mut c = stale(Some("0.1.3"));
        c.hinted_at = Some(t0() - HINT_TTL / 2);
        c.hinted_version = Some(v("0.1.3"));
        assert_eq!(decide(Some(&c), &ctx(&cur, true)), Action::Spawn);
    }

    #[test]
    fn a_different_version_hinted_recently_is_hinted_again() {
        let cur = v("0.1.1");
        let mut c = fresh(Some("0.1.4"));
        c.hinted_at = Some(t0() - HINT_TTL / 2);
        c.hinted_version = Some(v("0.1.3"));
        assert_eq!(decide(Some(&c), &ctx(&cur, true)), Action::Hint(v("0.1.4")));
    }

    #[test]
    fn a_hint_older_than_the_ttl_is_repeated() {
        let cur = v("0.1.1");
        let mut c = fresh(Some("0.1.3"));
        c.hinted_at = Some(t0() - HINT_TTL);
        c.hinted_version = Some(v("0.1.3"));
        assert_eq!(decide(Some(&c), &ctx(&cur, true)), Action::Hint(v("0.1.3")));
    }

    #[test]
    fn a_hinted_at_without_a_hinted_version_does_not_suppress() {
        let cur = v("0.1.1");
        let mut c = fresh(Some("0.1.3"));
        c.hinted_at = Some(t0());
        c.hinted_version = None;
        assert_eq!(decide(Some(&c), &ctx(&cur, true)), Action::Hint(v("0.1.3")));
    }

    // ── hint_line ──────────────────────────────────────────────────────────

    #[test]
    fn the_hint_line_names_both_versions_and_the_self_update_command() {
        assert_eq!(
            hint_line(&v("0.1.3"), &v("0.1.1"), false),
            "mx 0.1.3 is available (you have 0.1.1). Run: mx self-update"
        );
    }

    #[test]
    fn the_hint_line_substitutes_brew_for_a_homebrew_install() {
        assert_eq!(
            hint_line(&v("0.1.3"), &v("0.1.1"), true),
            "mx 0.1.3 is available (you have 0.1.1). Run: brew upgrade mx"
        );
    }

    // ── cache IO ───────────────────────────────────────────────────────────

    #[test]
    fn the_cache_round_trips_through_disk() {
        let home = tempfile::tempdir().unwrap();
        let mut c = fresh(Some("0.1.3"));
        c.hinted_at = Some(t0());
        c.hinted_version = Some(v("0.1.3"));
        write_cache(home.path(), &c).unwrap();
        let back = read_cache(home.path()).expect("cache reads back");
        assert_eq!(back.checked_at, c.checked_at);
        assert_eq!(back.next_check_at, c.next_check_at);
        assert_eq!(back.latest, c.latest);
        assert_eq!(back.current_at_check, c.current_at_check);
        assert_eq!(back.hinted_at, c.hinted_at);
        assert_eq!(back.hinted_version, c.hinted_version);
    }

    #[test]
    fn writing_the_cache_creates_the_cache_directory() {
        let home = tempfile::tempdir().unwrap();
        write_cache(home.path(), &fresh(None)).unwrap();
        assert!(cache_path(home.path()).is_file());
    }

    #[test]
    fn writing_the_cache_leaves_no_temp_file_behind() {
        let home = tempfile::tempdir().unwrap();
        write_cache(home.path(), &fresh(None)).unwrap();
        write_cache(home.path(), &fresh(Some("0.1.3"))).unwrap();
        let names: Vec<String> = std::fs::read_dir(home.path().join("cache"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["update-check.json".to_string()]);
        assert_eq!(read_cache(home.path()).unwrap().latest, Some(v("0.1.3")));
    }

    #[test]
    fn a_missing_cache_reads_as_none() {
        let home = tempfile::tempdir().unwrap();
        assert!(read_cache(home.path()).is_none());
    }

    #[test]
    fn an_unparsable_cache_reads_as_none() {
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(home.path().join("cache")).unwrap();
        std::fs::write(cache_path(home.path()), "{not json").unwrap();
        assert!(read_cache(home.path()).is_none());
    }

    #[test]
    fn a_cache_with_a_bad_version_reads_as_none() {
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(home.path().join("cache")).unwrap();
        std::fs::write(
            cache_path(home.path()),
            r#"{"checked_at":"2026-09-03T12:00:00Z","next_check_at":"2026-09-04T12:00:00Z","latest":"nope","current_at_check":"0.1.1"}"#,
        )
        .unwrap();
        assert!(read_cache(home.path()).is_none());
    }

    #[test]
    fn a_cache_without_the_hint_fields_still_reads() {
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(home.path().join("cache")).unwrap();
        std::fs::write(
            cache_path(home.path()),
            r#"{"checked_at":"2026-09-03T12:00:00Z","next_check_at":"2026-09-04T12:00:00Z","latest":"0.1.3","current_at_check":"0.1.1"}"#,
        )
        .unwrap();
        let c = read_cache(home.path()).expect("cache reads back");
        assert_eq!(c.latest, Some(v("0.1.3")));
        assert!(c.hinted_at.is_none());
        assert!(c.hinted_version.is_none());
    }

    // ── config_disables ────────────────────────────────────────────────────

    #[test]
    fn a_missing_config_does_not_disable() {
        let home = tempfile::tempdir().unwrap();
        assert!(!config_disables(home.path()));
    }

    fn write_config(home: &Path, body: &str) {
        let path = home.join(CONFIG_FILE);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    #[test]
    fn check_false_disables() {
        let home = tempfile::tempdir().unwrap();
        write_config(home.path(), "check = false\n");
        assert!(config_disables(home.path()));
    }

    #[test]
    fn check_true_does_not_disable() {
        let home = tempfile::tempdir().unwrap();
        write_config(home.path(), "check = true\n");
        assert!(!config_disables(home.path()));
    }

    #[test]
    fn an_unrelated_or_broken_config_does_not_disable() {
        let home = tempfile::tempdir().unwrap();
        write_config(home.path(), "");
        assert!(!config_disables(home.path()));
        write_config(home.path(), "other = 1\n");
        assert!(!config_disables(home.path()));
        write_config(home.path(), "check = = false\n");
        assert!(!config_disables(home.path()));
        write_config(home.path(), "check = \"false\"\n");
        assert!(!config_disables(home.path()));
    }
}
