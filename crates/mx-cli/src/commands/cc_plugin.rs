//! `mx cc-plugin` — install / uninstall / status of Unyform's Claude Code hooks.
//!
//! Phase 1 of the CC local-plugin (see `docs/unyform/CC_LOCAL_PLUGIN_DESIGN.md`).
//! This subcommand writes hook entries into the user's `~/.claude/settings.json`
//! so CC will call back into `mx cc-plugin <handler>` at the right lifecycle
//! moments (SessionStart, Stop). The hook handlers themselves are stubbed in
//! Phase 1 — they exit successfully and print a TODO note. Phase 2 wires
//! blueprint resolution against the unyform.ai SaaS; Phase 3 wires audit
//! ingest.
//!
//! Design goals for the installer:
//!   * **Idempotent.** Running `install` twice does not duplicate entries.
//!   * **Additive.** Existing user hooks (e.g. codegraph mark-dirty) are
//!     preserved. We only touch entries whose command starts with the marker
//!     `mx cc-plugin `.
//!   * **Reversible.** `uninstall` removes our entries and tidies up the JSON
//!     so an unaware reader can't tell anything was ever written.
//!   * **No global side effects in tests.** Every command takes an optional
//!     `--settings <path>` so test cases can run against a temp file.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use console::style;
use serde_json::{json, Value};

/// Command-string marker that identifies hooks installed by `mx cc-plugin
/// install`. Install + uninstall use this to find their own entries
/// idempotently without disturbing user-authored hooks.
const OUR_HOOK_COMMAND_PREFIX: &str = "mx cc-plugin ";

/// Phase-1 hooks the installer writes into `settings.json`. Each entry is
/// `(CC hook event name, mx subcommand the hook invokes)`. Phase 1 covers the
/// blueprint-injection moment (SessionStart) and the audit-flush moment
/// (Stop); UserPromptSubmit / PostToolUse / SessionEnd are deliberately
/// deferred until Phase 4 so the surface area stays small and reviewable.
const HOOKS_TO_INSTALL: &[(&str, &str)] = &[
    ("SessionStart", "mx cc-plugin session"),
    ("Stop", "mx cc-plugin stop"),
];

#[derive(Args, Debug)]
pub struct CcPluginCommand {
    #[command(subcommand)]
    command: CcPluginSubcommand,
}

#[derive(Subcommand, Debug)]
enum CcPluginSubcommand {
    /// Install Unyform CC hooks into ~/.claude/settings.json
    Install {
        /// Overwrite existing Unyform hook entries instead of skipping them.
        #[arg(short, long)]
        force: bool,
        /// Path to the settings.json to edit (default: ~/.claude/settings.json).
        /// Primarily for tests; users rarely need this.
        #[arg(long)]
        settings: Option<PathBuf>,
    },
    /// Remove Unyform CC hooks from ~/.claude/settings.json
    Uninstall {
        #[arg(long)]
        settings: Option<PathBuf>,
    },
    /// Report which Unyform hooks are currently installed
    Status {
        #[arg(long)]
        settings: Option<PathBuf>,
    },

    // ── Hook handler subcommands (called by CC, not by humans) ─────────────
    /// SessionStart hook handler — emits blueprint context (stubbed in Phase 1).
    #[command(hide = true)]
    Session,
    /// Stop hook handler — flushes the turn's audit event (stubbed in Phase 1).
    #[command(hide = true)]
    Stop,
}

impl CcPluginCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            CcPluginSubcommand::Install { force, settings } => install(*force, settings.as_deref()),
            CcPluginSubcommand::Uninstall { settings } => uninstall(settings.as_deref()),
            CcPluginSubcommand::Status { settings } => status(settings.as_deref()),
            CcPluginSubcommand::Session => session_handler().await,
            CcPluginSubcommand::Stop => stop_handler().await,
        }
    }
}

// ── Path resolution ──────────────────────────────────────────────────────────

fn default_settings_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not resolve home directory")?;
    Ok(home.join(".claude").join("settings.json"))
}

fn resolve_settings_path(override_path: Option<&Path>) -> Result<PathBuf> {
    match override_path {
        Some(p) => Ok(p.to_path_buf()),
        None => default_settings_path(),
    }
}

// ── settings.json I/O ────────────────────────────────────────────────────────

/// Load the settings JSON, returning a fresh empty object when the file is
/// missing. Treats an empty file the same as a missing one so a user who has
/// never customized CC isn't punished for the lack of file.
fn load_settings(path: &Path) -> Result<Value> {
    if !path.exists() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    let raw =
        fs::read_to_string(path).with_context(|| format!("read settings: {}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    serde_json::from_str(&raw).with_context(|| format!("parse settings: {}", path.display()))
}

/// Write the settings JSON, creating the parent directory if needed. Pretty-
/// prints with a trailing newline so the diff stays sane when the user opens
/// the file in an editor.
fn save_settings(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create parent dir: {}", parent.display()))?;
    }
    let pretty = serde_json::to_string_pretty(value)?;
    fs::write(path, pretty + "\n").with_context(|| format!("write settings: {}", path.display()))
}

// ── Hook entry shape ─────────────────────────────────────────────────────────

/// Build a matcher-block in the shape CC expects:
///   `{ "hooks": [ { "type": "command", "command": "<cmd>" } ] }`
/// SessionStart and Stop don't take per-tool matchers, so we omit the matcher
/// field (matches how the user's existing codegraph Stop entry is written).
fn make_hook_entry(command: &str) -> Value {
    json!({
        "hooks": [
            {
                "type": "command",
                "command": command,
            }
        ]
    })
}

/// Search a hook-event array for the first matcher-block that contains one of
/// OUR hook commands (identified by the `mx cc-plugin ` prefix). Returns the
/// index if found, so callers can either skip or overwrite.
fn find_our_hook_index(event_arr: &[Value]) -> Option<usize> {
    for (i, entry) in event_arr.iter().enumerate() {
        let Some(inner_hooks) = entry.get("hooks").and_then(|v| v.as_array()) else {
            continue;
        };
        for h in inner_hooks {
            if let Some(cmd) = h.get("command").and_then(|v| v.as_str()) {
                if cmd.starts_with(OUR_HOOK_COMMAND_PREFIX) {
                    return Some(i);
                }
            }
        }
    }
    None
}

// ── install / uninstall / status (pure-function cores for tests) ─────────────

/// Mutate `settings` in-place to install the Phase-1 hooks. Returns
/// `(added, skipped)` — `added` includes both freshly-inserted entries and
/// force-overwritten ones; `skipped` counts entries left alone because they
/// already existed and `force` was false.
fn install_hooks_in_value(settings: &mut Value, force: bool) -> (usize, usize) {
    let root = match settings.as_object_mut() {
        Some(o) => o,
        None => unreachable!("caller verifies settings is an object"),
    };
    let hooks_root = root
        .entry("hooks")
        .or_insert_with(|| Value::Object(serde_json::Map::new()))
        .as_object_mut()
        .expect("hooks field must be a JSON object");

    let mut added = 0usize;
    let mut skipped = 0usize;
    for (event, command) in HOOKS_TO_INSTALL {
        let event_arr = hooks_root
            .entry((*event).to_string())
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .expect("each hook event maps to an array of matcher-blocks");
        match find_our_hook_index(event_arr) {
            Some(idx) if force => {
                event_arr[idx] = make_hook_entry(command);
                added += 1;
            }
            Some(_) => {
                skipped += 1;
            }
            None => {
                event_arr.push(make_hook_entry(command));
                added += 1;
            }
        }
    }
    (added, skipped)
}

/// Mutate `settings` in-place to remove every Phase-1 hook we own. Tidies up
/// emptied arrays / objects so the file looks untouched if we were the only
/// thing in `hooks`.
fn uninstall_hooks_in_value(settings: &mut Value) -> usize {
    let Some(root) = settings.as_object_mut() else {
        return 0;
    };
    let Some(hooks_root) = root.get_mut("hooks").and_then(|v| v.as_object_mut()) else {
        return 0;
    };
    let mut removed = 0usize;
    for (event, _) in HOOKS_TO_INSTALL {
        if let Some(arr) = hooks_root.get_mut(*event).and_then(|v| v.as_array_mut()) {
            let before = arr.len();
            arr.retain(|entry| {
                // Keep if the entry has any non-ours commands; drop the whole
                // matcher-block if the only thing in it is ours. (Phase 1 only
                // creates blocks that hold a single command, so this is fine.)
                let inner = entry.get("hooks").and_then(|v| v.as_array());
                let only_ours = inner
                    .map(|hooks| {
                        !hooks.is_empty()
                            && hooks.iter().all(|h| {
                                h.get("command")
                                    .and_then(|v| v.as_str())
                                    .is_some_and(|s| s.starts_with(OUR_HOOK_COMMAND_PREFIX))
                            })
                    })
                    .unwrap_or(false);
                !only_ours
            });
            removed += before - arr.len();
        }
    }
    // Drop emptied hook-event arrays so the file isn't littered.
    let empty_keys: Vec<String> = hooks_root
        .iter()
        .filter_map(|(k, v)| match v.as_array() {
            Some(a) if a.is_empty() => Some(k.clone()),
            _ => None,
        })
        .collect();
    for k in empty_keys {
        hooks_root.remove(&k);
    }
    // And if `hooks` itself is now empty, remove the whole key.
    if hooks_root.is_empty() {
        root.remove("hooks");
    }
    removed
}

// ── User-facing entry points ─────────────────────────────────────────────────

fn install(force: bool, settings_path: Option<&Path>) -> Result<()> {
    let path = resolve_settings_path(settings_path)?;
    let mut settings = load_settings(&path)?;
    if !settings.is_object() {
        anyhow::bail!(
            "settings.json root must be a JSON object, got {}",
            type_name_of(&settings)
        );
    }
    let (added, skipped) = install_hooks_in_value(&mut settings, force);
    save_settings(&path, &settings)?;
    let suffix = if skipped > 0 {
        format!(
            " ({} already present, skipped — pass --force to overwrite)",
            skipped
        )
    } else {
        String::new()
    };
    println!(
        "{} installed {} Unyform CC hook(s) in {}{}",
        style("✓").green(),
        added,
        path.display(),
        suffix
    );
    Ok(())
}

fn uninstall(settings_path: Option<&Path>) -> Result<()> {
    let path = resolve_settings_path(settings_path)?;
    if !path.exists() {
        println!(
            "{} no settings.json at {}; nothing to remove",
            style("ℹ").cyan(),
            path.display()
        );
        return Ok(());
    }
    let mut settings = load_settings(&path)?;
    if !settings.is_object() {
        anyhow::bail!("settings.json root must be a JSON object");
    }
    let removed = uninstall_hooks_in_value(&mut settings);
    if removed == 0 {
        println!(
            "{} no Unyform CC hooks installed in {}",
            style("ℹ").cyan(),
            path.display()
        );
        return Ok(());
    }
    save_settings(&path, &settings)?;
    println!(
        "{} removed {} Unyform CC hook(s) from {}",
        style("✓").green(),
        removed,
        path.display()
    );
    Ok(())
}

fn status(settings_path: Option<&Path>) -> Result<()> {
    let path = resolve_settings_path(settings_path)?;
    println!("Unyform CC hook status (settings: {})", path.display());
    let settings = if path.exists() {
        load_settings(&path)?
    } else {
        Value::Object(serde_json::Map::new())
    };
    let hooks_root = settings
        .get("hooks")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    for (event, command) in HOOKS_TO_INSTALL {
        let installed = hooks_root
            .get(*event)
            .and_then(|v| v.as_array())
            .map(|arr| find_our_hook_index(arr).is_some())
            .unwrap_or(false);
        if installed {
            println!(
                "  {} {} → {}",
                style("✓").green(),
                event,
                style(command).cyan()
            );
        } else {
            println!(
                "  {} {} → {}",
                style("·").dim(),
                event,
                style("not installed").dim()
            );
        }
    }
    Ok(())
}

// ── Hook handlers (stubbed in Phase 1) ──────────────────────────────────────

async fn session_handler() -> Result<()> {
    // Phase 1: stubbed. Phase 2 resolves blueprints for the current workspace
    // against the unyform.ai SaaS and emits a `<system-reminder>` block to
    // stdout for CC to inject into the session preamble.
    eprintln!("mx cc-plugin session: stubbed in Phase 1 (no blueprint resolution yet)");
    Ok(())
}

async fn stop_handler() -> Result<()> {
    // Phase 1: stubbed. Phase 3 will POST the turn's audit event (model,
    // input/output tokens, blueprint IDs, policy outcomes) to the
    // /api/v1/cc/audit endpoint and rollup usage to /api/v1/cc/usage.
    eprintln!("mx cc-plugin stop: stubbed in Phase 1 (no audit flush yet)");
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn type_name_of(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    //! Phase-1 unit tests for `mx cc-plugin`. Pure-function tests on
    //! `serde_json::Value` cover the install / uninstall / find-our-hook
    //! logic; a handful of file-backed tests exercise the I/O wrapping
    //! using unique paths under `std::env::temp_dir()` so test runs are
    //! isolated without needing a tempfile crate.
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Unique temp path generator — avoids needing the `tempfile` crate for
    /// Phase 1. Each call returns a path that's guaranteed not to collide
    /// with another within the same process.
    fn temp_settings_path() -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        std::env::temp_dir().join(format!("mx-cc-plugin-test-{pid}-{n}.json"))
    }

    #[test]
    fn install_into_empty_settings_adds_both_phase1_hooks() {
        let mut settings = json!({});
        let (added, skipped) = install_hooks_in_value(&mut settings, false);
        assert_eq!(added, 2);
        assert_eq!(skipped, 0);

        let hooks = &settings["hooks"];
        for (event, command) in HOOKS_TO_INSTALL {
            let arr = hooks[event].as_array().expect("event entry is array");
            assert_eq!(arr.len(), 1, "{event} got exactly one matcher-block");
            assert_eq!(
                arr[0]["hooks"][0]["command"].as_str(),
                Some(*command),
                "{event} carries the right command"
            );
        }
    }

    #[test]
    fn install_is_idempotent_without_force() {
        let mut settings = json!({});
        let _ = install_hooks_in_value(&mut settings, false);
        let snapshot_after_first = settings.clone();

        let (added, skipped) = install_hooks_in_value(&mut settings, false);
        assert_eq!(added, 0, "second install must add nothing");
        assert_eq!(skipped, HOOKS_TO_INSTALL.len());
        assert_eq!(
            settings, snapshot_after_first,
            "settings JSON must be byte-identical after the no-op second install"
        );
    }

    #[test]
    fn install_with_force_overwrites_existing_entry_in_place() {
        let mut settings = json!({
            "hooks": {
                "SessionStart": [
                    {
                        "hooks": [
                            {
                                "type": "command",
                                "command": "mx cc-plugin session --old-flag"
                            }
                        ]
                    }
                ]
            }
        });
        let (added, skipped) = install_hooks_in_value(&mut settings, true);
        assert_eq!(skipped, 0);
        assert_eq!(added, HOOKS_TO_INSTALL.len());
        // Old `--old-flag` form is gone, replaced with the canonical command.
        let session_arr = settings["hooks"]["SessionStart"].as_array().unwrap();
        assert_eq!(session_arr.len(), 1, "in-place overwrite, not duplicated");
        assert_eq!(
            session_arr[0]["hooks"][0]["command"].as_str(),
            Some("mx cc-plugin session")
        );
    }

    #[test]
    fn install_preserves_user_owned_hooks_for_same_event() {
        // The user's real settings.json (e.g. their codegraph Stop hook) must
        // survive `install`. Our entry is added alongside, not in place of.
        let mut settings = json!({
            "hooks": {
                "Stop": [
                    {
                        "hooks": [
                            {"type": "command", "command": "codegraph sync-if-dirty"}
                        ]
                    }
                ]
            }
        });
        install_hooks_in_value(&mut settings, false);
        let stop_arr = settings["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop_arr.len(), 2, "user hook + our hook");
        // User's first, ours appended.
        assert_eq!(
            stop_arr[0]["hooks"][0]["command"].as_str(),
            Some("codegraph sync-if-dirty")
        );
        assert_eq!(
            stop_arr[1]["hooks"][0]["command"].as_str(),
            Some("mx cc-plugin stop")
        );
    }

    #[test]
    fn uninstall_removes_only_our_hooks_and_tidies_empty_arrays() {
        // Start with both a user hook (codegraph) and ours.
        let mut settings = json!({
            "hooks": {
                "Stop": [
                    {"hooks": [{"type": "command", "command": "codegraph sync-if-dirty"}]},
                    {"hooks": [{"type": "command", "command": "mx cc-plugin stop"}]}
                ],
                "SessionStart": [
                    {"hooks": [{"type": "command", "command": "mx cc-plugin session"}]}
                ]
            }
        });
        let removed = uninstall_hooks_in_value(&mut settings);
        assert_eq!(removed, 2);

        // User's Stop hook survives intact.
        let stop = settings["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 1);
        assert_eq!(
            stop[0]["hooks"][0]["command"].as_str(),
            Some("codegraph sync-if-dirty")
        );

        // SessionStart was only ours, so the empty array is dropped from the JSON.
        assert!(
            settings["hooks"].get("SessionStart").is_none(),
            "emptied event-array must be tidied away"
        );
    }

    #[test]
    fn uninstall_drops_hooks_key_entirely_when_only_ours_was_there() {
        let mut settings = json!({
            "hooks": {
                "SessionStart": [{"hooks": [{"type":"command","command":"mx cc-plugin session"}]}],
                "Stop":         [{"hooks": [{"type":"command","command":"mx cc-plugin stop"}]}]
            },
            "permissions": {"allow": []}
        });
        uninstall_hooks_in_value(&mut settings);
        assert!(
            settings.get("hooks").is_none(),
            "emptied hooks object must be removed so the file looks untouched"
        );
        assert!(
            settings.get("permissions").is_some(),
            "unrelated top-level keys must be preserved"
        );
    }

    #[test]
    fn uninstall_on_settings_with_no_hooks_is_noop() {
        let mut settings = json!({"permissions": {"allow": []}});
        let removed = uninstall_hooks_in_value(&mut settings);
        assert_eq!(removed, 0);
        assert_eq!(settings, json!({"permissions": {"allow": []}}));
    }

    #[test]
    fn find_our_hook_index_ignores_user_commands_with_matching_prefix_inside_args() {
        // A user command that contains 'mx cc-plugin' as an argument (not as
        // its literal prefix) must NOT be claimed as ours.
        let arr = vec![json!({
            "hooks": [
                {"type": "command", "command": "echo 'mx cc-plugin foo'"}
            ]
        })];
        assert_eq!(find_our_hook_index(&arr), None);
    }

    #[test]
    fn find_our_hook_index_matches_when_prefix_is_at_start() {
        let arr = vec![json!({
            "hooks": [
                {"type": "command", "command": "mx cc-plugin session"}
            ]
        })];
        assert_eq!(find_our_hook_index(&arr), Some(0));
    }

    // ── File-backed tests ────────────────────────────────────────────────

    #[test]
    fn install_creates_settings_file_when_missing() {
        let path = temp_settings_path();
        // Ensure we start from a clean slate.
        let _ = fs::remove_file(&path);
        install(false, Some(&path)).expect("install must succeed");

        let raw = fs::read_to_string(&path).expect("file must have been created");
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert!(v["hooks"]["SessionStart"].is_array());
        assert!(v["hooks"]["Stop"].is_array());

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn install_then_uninstall_round_trips_to_a_clean_file() {
        let path = temp_settings_path();
        let _ = fs::remove_file(&path);
        // Pre-seed with a user hook so we have something the round-trip must
        // preserve.
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            serde_json::to_string_pretty(&json!({
                "permissions": {"allow": ["mcp__some__tool"]},
                "hooks": {
                    "Stop": [
                        {"hooks": [{"type": "command", "command": "codegraph sync-if-dirty"}]}
                    ]
                }
            }))
            .unwrap(),
        )
        .unwrap();

        install(false, Some(&path)).unwrap();
        uninstall(Some(&path)).unwrap();

        let raw = fs::read_to_string(&path).unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(
            v,
            json!({
                "permissions": {"allow": ["mcp__some__tool"]},
                "hooks": {
                    "Stop": [
                        {"hooks": [{"type": "command", "command": "codegraph sync-if-dirty"}]}
                    ]
                }
            }),
            "round-trip must restore the user's original settings exactly"
        );

        let _ = fs::remove_file(&path);
    }
}
