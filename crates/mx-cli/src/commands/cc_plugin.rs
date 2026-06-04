//! `mx cc-plugin` — the Unyform Claude Code local plugin (see
//! `docs/unyform/CC_LOCAL_PLUGIN_DESIGN.md`).
//!
//! `install`/`uninstall`/`status` manage hook entries in the user's
//! `~/.claude/settings.json` so CC calls back into `mx cc-plugin <handler>` at
//! the right lifecycle moments, plus a `cc-plugin.json` auth config holding the
//! gateway API key.
//!
//! Hook handlers (called by CC, not humans):
//!   * **SessionStart → `session`** (Phase 2): resolves blueprints from the
//!     Unyform SaaS `/v1/cc/session`, prints a `<system-reminder>` block to
//!     stdout for CC to inject, and records the injected blueprint IDs to a
//!     per-session state file.
//!   * **Stop → `stop`** (Phase 3b): reads CC's `session_id` from stdin, loads
//!     that state file, and POSTs the blueprint-injection audit event to
//!     `/v1/cc/audit` (recorded as a `gateway_usage` row), then removes the
//!     state file. Token usage from CC's transcript is a later phase.
//!
//! Every handler fails soft — on any error it exits 0 and changes nothing, so
//! a misconfigured or offline gateway never breaks the user's `claude`.
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
    /// Install Unyform CC hooks into ~/.claude/settings.json (and optionally
    /// write auth config if --api-key is provided).
    Install {
        /// Per-gateway API key (`uny_gw_...`). When provided, written to the
        /// cc-plugin config file so the SessionStart hook can authenticate
        /// against `/api/v1/cc/session`. Omitting it leaves hooks installed
        /// but the session handler will fail-soft until config exists.
        #[arg(long, env = "UNYFORM_API_KEY")]
        api_key: Option<String>,
        /// Override the Unyform SaaS base URL. Defaults to
        /// `https://gateway.unyform.ai`; primarily for staging/dev use.
        #[arg(long, env = "UNYFORM_BASE_URL", default_value = DEFAULT_BASE_URL)]
        base_url: String,
        /// Overwrite existing Unyform hook entries instead of skipping them.
        #[arg(short, long)]
        force: bool,
        /// Path to the settings.json to edit (default: ~/.claude/settings.json).
        /// Primarily for tests; users rarely need this.
        #[arg(long)]
        settings: Option<PathBuf>,
        /// Path to the cc-plugin config file (default platform XDG path).
        /// Primarily for tests.
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Remove Unyform CC hooks from ~/.claude/settings.json (and the cc-plugin
    /// config file unless --keep-config is passed).
    Uninstall {
        #[arg(long)]
        settings: Option<PathBuf>,
        #[arg(long)]
        config: Option<PathBuf>,
        /// Preserve the auth config file. Useful if you'll re-install soon
        /// and don't want to re-enter the API key.
        #[arg(long)]
        keep_config: bool,
    },
    /// Report which Unyform hooks are installed and whether the auth config
    /// is present.
    Status {
        #[arg(long)]
        settings: Option<PathBuf>,
        #[arg(long)]
        config: Option<PathBuf>,
    },

    // ── Hook handler subcommands (called by CC, not by humans) ─────────────
    /// SessionStart hook handler — resolves blueprints from the Unyform SaaS
    /// and prints a `<system-reminder>` block to stdout for CC to inject
    /// into the session preamble. Fails soft on any error so a misconfigured
    /// or offline gateway never breaks CC itself.
    #[command(hide = true)]
    Session {
        /// Override the cc-plugin config file path (primarily for tests).
        #[arg(long)]
        config: Option<PathBuf>,
        /// Override the SaaS base URL (primarily for tests; takes precedence
        /// over the value in the config file).
        #[arg(long)]
        base_url: Option<String>,
    },
    /// Stop hook handler — reports the finished session's blueprint-injection
    /// audit event to the Unyform SaaS. Fails soft on any error so it never
    /// blocks CC shutdown.
    #[command(hide = true)]
    Stop {
        /// Override the cc-plugin config file path (primarily for tests).
        #[arg(long)]
        config: Option<PathBuf>,
        /// Override the SaaS base URL (primarily for tests).
        #[arg(long)]
        base_url: Option<String>,
    },
}

impl CcPluginCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            CcPluginSubcommand::Install {
                api_key,
                base_url,
                force,
                settings,
                config,
            } => install(
                api_key.as_deref(),
                base_url,
                *force,
                settings.as_deref(),
                config.as_deref(),
            ),
            CcPluginSubcommand::Uninstall {
                settings,
                config,
                keep_config,
            } => uninstall(settings.as_deref(), config.as_deref(), *keep_config),
            CcPluginSubcommand::Status { settings, config } => {
                status(settings.as_deref(), config.as_deref())
            }
            CcPluginSubcommand::Session { config, base_url } => {
                session_handler(config.as_deref(), base_url.as_deref()).await
            }
            CcPluginSubcommand::Stop { config, base_url } => {
                stop_handler(config.as_deref(), base_url.as_deref()).await
            }
        }
    }
}

// ── cc-plugin auth config ───────────────────────────────────────────────────

/// Default Unyform SaaS base URL. The `/v1/cc/session` endpoint lives here.
pub(crate) const DEFAULT_BASE_URL: &str = "https://gateway.unyform.ai";

/// Persistent config for the cc-plugin hook handlers. Stored as JSON at
/// `~/.config/unyform/cc-plugin.json` (Linux) / `~/Library/Application Support/
/// unyform/cc-plugin.json` (macOS) — the platform's `dirs::config_dir()`.
/// File mode is 0600 on Unix so the API key isn't world-readable.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct CcPluginConfig {
    /// Per-gateway API key (`uny_gw_...`). Sent as `Authorization: Bearer`.
    pub(crate) api_key: String,
    /// Unyform SaaS base URL — the host serving `/v1/cc/session`.
    pub(crate) base_url: String,
}

fn default_config_path() -> Result<PathBuf> {
    let cfg = dirs::config_dir().context("could not resolve config directory")?;
    Ok(cfg.join("unyform").join("cc-plugin.json"))
}

fn resolve_config_path(override_path: Option<&Path>) -> Result<PathBuf> {
    match override_path {
        Some(p) => Ok(p.to_path_buf()),
        None => default_config_path(),
    }
}

fn load_cc_plugin_config(path: &Path) -> Result<CcPluginConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read cc-plugin config: {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("parse cc-plugin config: {}", path.display()))
}

fn save_cc_plugin_config(path: &Path, config: &CcPluginConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create config dir: {}", parent.display()))?;
    }
    let raw = serde_json::to_string_pretty(config)?;
    fs::write(path, raw + "\n")
        .with_context(|| format!("write cc-plugin config: {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("chmod 0600 cc-plugin config: {}", path.display()))?;
    }
    Ok(())
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

fn install(
    api_key: Option<&str>,
    base_url: &str,
    force: bool,
    settings_path: Option<&Path>,
    config_path: Option<&Path>,
) -> Result<()> {
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

    // Auth config — only write when the caller supplied an API key.
    // Hooks-without-config is a valid intermediate state; the session handler
    // logs a one-line warning and exits 0 in that case so CC never breaks.
    let config_full_path = resolve_config_path(config_path)?;
    if let Some(key) = api_key {
        let cfg = CcPluginConfig {
            api_key: key.to_string(),
            base_url: base_url.to_string(),
        };
        save_cc_plugin_config(&config_full_path, &cfg)?;
        println!(
            "{} wrote cc-plugin auth config to {} (mode 0600)",
            style("✓").green(),
            config_full_path.display()
        );
    } else if !config_full_path.exists() {
        println!(
            "{} no API key provided and no existing config at {} — session handler will skip blueprint injection until configured. Re-run with --api-key uny_gw_... to enable.",
            style("ℹ").cyan(),
            config_full_path.display()
        );
    }
    Ok(())
}

fn uninstall(
    settings_path: Option<&Path>,
    config_path: Option<&Path>,
    keep_config: bool,
) -> Result<()> {
    let path = resolve_settings_path(settings_path)?;
    if path.exists() {
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
        } else {
            save_settings(&path, &settings)?;
            println!(
                "{} removed {} Unyform CC hook(s) from {}",
                style("✓").green(),
                removed,
                path.display()
            );
        }
    } else {
        println!(
            "{} no settings.json at {}; nothing to remove",
            style("ℹ").cyan(),
            path.display()
        );
    }

    if !keep_config {
        let cfg_path = resolve_config_path(config_path)?;
        if cfg_path.exists() {
            fs::remove_file(&cfg_path)
                .with_context(|| format!("remove cc-plugin config: {}", cfg_path.display()))?;
            println!(
                "{} removed cc-plugin auth config at {}",
                style("✓").green(),
                cfg_path.display()
            );
        }
    }
    Ok(())
}

fn status(settings_path: Option<&Path>, config_path: Option<&Path>) -> Result<()> {
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

    let cfg_path = resolve_config_path(config_path)?;
    println!("\nAuth config ({}):", cfg_path.display());
    match load_cc_plugin_config(&cfg_path) {
        Ok(cfg) => {
            println!(
                "  {} configured — base_url = {}, api_key = {}",
                style("✓").green(),
                cfg.base_url,
                redact_api_key(&cfg.api_key)
            );
        }
        Err(_) => {
            println!(
                "  {} {}",
                style("·").dim(),
                style("not configured — run `mx cc-plugin install --api-key uny_gw_...`").dim()
            );
        }
    }
    Ok(())
}

// ── Hook handlers ───────────────────────────────────────────────────────────

// ── Per-session state (bridges SessionStart → Stop) ─────────────────────────
//
// SessionStart and Stop are SEPARATE process invocations, so the blueprint
// IDs resolved at SessionStart must be persisted somewhere Stop can find them
// to build the audit event. We write a small JSON file keyed by CC's
// `session_id` into a `sessions/` dir beside the cc-plugin config. A local
// file write adds no network latency to the latency-budgeted SessionStart hook
// (the audit POST itself happens at Stop, which is not latency-sensitive).

/// What SessionStart records so Stop can report it. Kept minimal and reliable
/// — the blueprint-injection signal (which blueprints reached this session)
/// plus a start timestamp for duration. Token usage is NOT captured here; it
/// lives in CC's transcript and is deferred to a later phase.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SessionState {
    blueprint_ids: Vec<String>,
    blueprint_tokens: u64,
    base_url: String,
    started_at_unix: u64,
}

/// What `fetch_session` extracts from a `/v1/cc/session` response: the block to
/// print AND the structured signal needed for the later audit event.
#[derive(Debug, Default)]
pub(crate) struct SessionResult {
    pub(crate) block: String,
    pub(crate) blueprint_ids: Vec<String>,
    pub(crate) blueprint_tokens: u64,
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Read CC's hook payload from stdin. CC writes `{session_id, transcript_path,
/// cwd, ...}` as one JSON line then closes stdin. Returns None when stdin is a
/// TTY (manual invocation — don't block waiting for a human to type) or the
/// payload is absent/unparseable. Always best-effort: a missing payload just
/// means no session correlation, never an error.
fn read_hook_stdin() -> Option<Value> {
    use std::io::{IsTerminal, Read};
    if std::io::stdin().is_terminal() {
        return None;
    }
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).ok()?;
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        return None;
    }
    serde_json::from_str(trimmed).ok()
}

/// Pull `session_id` from a hook payload, sanitized to a filesystem-safe slug
/// so a malformed/hostile value can't escape the sessions dir via path
/// traversal. CC session ids are UUIDs (alphanumeric + dash); anything else is
/// dropped to `_`.
fn session_id_from_hook(payload: &Value) -> Option<String> {
    let raw = payload.get("session_id").and_then(|v| v.as_str())?;
    if raw.is_empty() {
        return None;
    }
    let safe: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .take(128)
        .collect();
    Some(safe)
}

/// Path to the per-session state file: a `sessions/` dir beside the cc-plugin
/// config (so it inherits the same parent and cleanup story).
fn session_state_path(config_path: &Path, session_id: &str) -> PathBuf {
    let dir = config_path
        .parent()
        .map(|p| p.join("sessions"))
        .unwrap_or_else(|| PathBuf::from("sessions"));
    dir.join(format!("{session_id}.json"))
}

fn write_session_state(path: &Path, state: &SessionState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create sessions dir: {}", parent.display()))?;
    }
    let raw = serde_json::to_string(state)?;
    fs::write(path, raw).with_context(|| format!("write session state: {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// SessionStart hook handler. Reads the cc-plugin auth config, calls
/// `/v1/cc/session` on the configured SaaS host, prints the returned
/// `system_reminder_block` to stdout for CC to inject, AND (when CC supplied a
/// session_id on stdin) records the injected blueprint IDs to a per-session
/// state file so the Stop hook can report them. Fails soft on every error
/// path — CC's hook contract is that anything the SessionStart command writes
/// to stdout is injected verbatim, so on failure we write nothing and exit 0.
async fn session_handler(
    config_path_override: Option<&Path>,
    base_url_override: Option<&str>,
) -> Result<()> {
    // Read CC's hook payload first (cheap, local) so a session_id is available
    // to correlate the later audit even if the network fetch is slow.
    let session_id = read_hook_stdin().as_ref().and_then(session_id_from_hook);

    let cfg_path = match resolve_config_path(config_path_override) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("mx cc-plugin session: cannot resolve config path: {e}");
            return Ok(());
        }
    };
    let cfg = match load_cc_plugin_config(&cfg_path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!(
                "mx cc-plugin session: no auth config at {} — skipping blueprint injection. Run `mx cc-plugin install --api-key uny_gw_...` to configure.",
                cfg_path.display()
            );
            return Ok(());
        }
    };
    let base_url = base_url_override
        .map(str::to_string)
        .unwrap_or(cfg.base_url);

    match fetch_session(&base_url, &cfg.api_key).await {
        Ok(result) => {
            if !result.block.is_empty() {
                // CC injects SessionStart hook stdout into the session preamble
                // verbatim. `print!` (not `println!`) honors the SaaS framing.
                print!("{}", result.block);
            }
            // Record the injection signal so Stop can audit it. Only when CC
            // gave us a session_id AND something was actually injected.
            if let (Some(sid), false) = (&session_id, result.blueprint_ids.is_empty()) {
                let state = SessionState {
                    blueprint_ids: result.blueprint_ids,
                    blueprint_tokens: result.blueprint_tokens,
                    base_url,
                    started_at_unix: unix_now(),
                };
                let state_path = session_state_path(&cfg_path, sid);
                if let Err(e) = write_session_state(&state_path, &state) {
                    eprintln!("mx cc-plugin session: could not record session state: {e}");
                }
            }
        }
        Err(e) => {
            eprintln!("mx cc-plugin session: {e} — skipping blueprint injection");
        }
    }
    Ok(())
}

/// POST `/v1/cc/session` and parse the block + blueprint signal. Kept separate
/// from `session_handler` so tests can drive it against a wiremock stand-in.
pub(crate) async fn fetch_session(
    base_url: &str,
    api_key: &str,
) -> std::result::Result<SessionResult, String> {
    let url = format!("{}/v1/cc/session", base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        // Tight timeout because CC blocks on the SessionStart hook — a slow
        // SaaS shouldn't translate into a slow `claude` startup. 3s is plenty
        // for a same-region request; users on bad networks will see CC start
        // without blueprint context, which is the correct fail-soft surface.
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| format!("build HTTP client: {e}"))?;
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await
        .map_err(|e| format!("POST {url}: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let preview = body.chars().take(200).collect::<String>();
        return Err(format!("HTTP {status} from {url}: {preview}"));
    }
    #[derive(serde::Deserialize)]
    struct Bp {
        id: String,
    }
    #[derive(serde::Deserialize)]
    struct R {
        system_reminder_block: String,
        #[serde(default)]
        blueprints: Vec<Bp>,
        #[serde(default)]
        estimated_tokens_total: u64,
    }
    let parsed: R = resp.json().await.map_err(|e| format!("parse JSON: {e}"))?;
    Ok(SessionResult {
        block: parsed.system_reminder_block,
        blueprint_ids: parsed.blueprints.into_iter().map(|b| b.id).collect(),
        blueprint_tokens: parsed.estimated_tokens_total,
    })
}

/// Truncate an API key for display in `status` output. Shows the prefix and
/// last 4 chars so a user can confirm which key is configured without
/// leaking the secret to a shoulder-surfer or screencast viewer.
fn redact_api_key(key: &str) -> String {
    if key.len() <= 12 {
        return "***".to_string();
    }
    let prefix: String = key.chars().take(10).collect();
    let suffix: String = key
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{prefix}…{suffix}")
}

fn read_session_state(path: &Path) -> Option<SessionState> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Token usage + model parsed from CC's session transcript.
#[derive(Debug, Default)]
struct TranscriptUsage {
    /// The session's model (last assistant turn — a session uses one primary
    /// model). None when no assistant turn carried a model.
    model: Option<String>,
    /// Sum over all assistant turns of `input_tokens + cache_creation_input_tokens`
    /// — the NEW input tokens the model processed this session. `cache_read` is
    /// deliberately EXCLUDED: it re-reads the same cached prefix every turn, so
    /// summing it multiplies one context by the turn count (verified on a real
    /// 627-turn transcript: including cache_read inflates ~12M → ~294M).
    /// Excluding it gives a sane "new input" figure comparable in scale to
    /// completion_tokens.
    prompt_tokens: u64,
    /// Sum of output_tokens across all assistant turns.
    completion_tokens: u64,
}

/// Parse CC's session transcript (JSONL) for token usage + model. Best-effort:
/// an unreadable / missing / garbled transcript yields zeroes + no model, which
/// the audit endpoint backfills with its `cc-plugin` / 0 defaults. Each
/// assistant line looks like `{type:"assistant", message:{model, usage:{
/// input_tokens, cache_creation_input_tokens, cache_read_input_tokens,
/// output_tokens}}}`. prompt_tokens sums input + cache_creation (the new
/// tokens), EXCLUDING cache_read (re-reads of already-counted context — see
/// the `TranscriptUsage::prompt_tokens` note); the model is the last turn's.
fn parse_transcript_usage(path: &Path) -> TranscriptUsage {
    let mut out = TranscriptUsage::default();
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
    for line in content.lines() {
        let Ok(rec) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if rec.get("type").and_then(|v| v.as_str()) != Some("assistant") {
            continue;
        }
        let Some(msg) = rec.get("message") else {
            continue;
        };
        if let Some(m) = msg.get("model").and_then(|v| v.as_str()) {
            if !m.is_empty() {
                out.model = Some(m.to_string());
            }
        }
        if let Some(u) = msg.get("usage") {
            let g = |k: &str| u.get(k).and_then(|v| v.as_u64()).unwrap_or(0);
            // NEW input tokens only — cache_read excluded (see field doc).
            out.prompt_tokens = out
                .prompt_tokens
                .saturating_add(g("input_tokens").saturating_add(g("cache_creation_input_tokens")));
            out.completion_tokens = out.completion_tokens.saturating_add(g("output_tokens"));
        }
    }
    out
}

/// One audit event for the `/v1/cc/audit` POST. `model` + token counts are
/// `None`/0 when the transcript couldn't be parsed (the endpoint backfills the
/// `cc-plugin` model and leaves counts at 0).
#[derive(Debug, Default)]
pub(crate) struct AuditEvent {
    pub(crate) model: Option<String>,
    pub(crate) prompt_tokens: u64,
    pub(crate) completion_tokens: u64,
    pub(crate) blueprint_ids: Vec<String>,
    pub(crate) blueprint_tokens: u64,
    pub(crate) duration_ms: u64,
}

/// Stop hook handler. Reads CC's `{session_id, transcript_path}` from stdin,
/// loads the per-session state SessionStart wrote, parses the transcript for
/// real token usage + model, and POSTs the audit event to `/v1/cc/audit`. The
/// state file is the only link between the two hook invocations — without it
/// (manual run, no injection, or already-audited) there's nothing to report and
/// we exit quietly. Fails soft on every path so it never blocks CC shutdown.
/// The state file is always removed afterward to keep the sessions dir from
/// growing unbounded (audit is best-effort telemetry; there is no retry).
async fn stop_handler(
    config_path_override: Option<&Path>,
    base_url_override: Option<&str>,
) -> Result<()> {
    let payload = read_hook_stdin();
    let session_id = match payload.as_ref().and_then(session_id_from_hook) {
        Some(s) => s,
        None => {
            // Manual invocation or no payload — nothing to correlate.
            return Ok(());
        }
    };

    let cfg_path = match resolve_config_path(config_path_override) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("mx cc-plugin stop: cannot resolve config path: {e}");
            return Ok(());
        }
    };
    let cfg = match load_cc_plugin_config(&cfg_path) {
        Ok(c) => c,
        Err(_) => {
            // No auth config — the session handler would also have skipped, so
            // there's no state file either. Nothing to do.
            return Ok(());
        }
    };

    let state_path = session_state_path(&cfg_path, &session_id);
    let state = match read_session_state(&state_path) {
        Some(s) => s,
        None => {
            // No state → SessionStart injected nothing (or already audited).
            return Ok(());
        }
    };

    let base_url = base_url_override.map(str::to_string).unwrap_or_else(|| {
        if state.base_url.is_empty() {
            cfg.base_url.clone()
        } else {
            state.base_url.clone()
        }
    });
    let duration_ms = unix_now()
        .saturating_sub(state.started_at_unix)
        .saturating_mul(1000);

    // Best-effort token usage + model from CC's transcript (Phase 3c).
    let usage = payload
        .as_ref()
        .and_then(|p| p.get("transcript_path"))
        .and_then(|v| v.as_str())
        .map(|p| parse_transcript_usage(Path::new(p)))
        .unwrap_or_default();

    let event = AuditEvent {
        model: usage.model,
        prompt_tokens: usage.prompt_tokens,
        completion_tokens: usage.completion_tokens,
        blueprint_ids: state.blueprint_ids,
        blueprint_tokens: state.blueprint_tokens,
        duration_ms,
    };

    if let Err(e) = post_audit(&base_url, &cfg.api_key, &event).await {
        eprintln!("mx cc-plugin stop: audit POST failed: {e}");
    }

    // Consume the state file regardless of POST outcome (no retry mechanism;
    // leaving it would leak one file per session).
    let _ = fs::remove_file(&state_path);
    Ok(())
}

/// POST an audit event to `/v1/cc/audit`. Runs at Stop, which is not
/// latency-sensitive, so a slightly looser 5s timeout than the SessionStart
/// fetch is fine. `model` is sent as null when unknown; the endpoint fills its
/// `cc-plugin` fallback.
pub(crate) async fn post_audit(
    base_url: &str,
    api_key: &str,
    event: &AuditEvent,
) -> std::result::Result<(), String> {
    let url = format!("{}/v1/cc/audit", base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("build HTTP client: {e}"))?;
    let body = serde_json::json!({
        "model": event.model,
        "prompt_tokens": event.prompt_tokens,
        "completion_tokens": event.completion_tokens,
        "blueprint_ids": event.blueprint_ids,
        "blueprint_tokens": event.blueprint_tokens,
        "duration_ms": event.duration_ms,
    });
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("POST {url}: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        let preview = text.chars().take(200).collect::<String>();
        return Err(format!("HTTP {status} from {url}: {preview}"));
    }
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

    /// Pair the settings-temp path with a sibling config-temp path so the
    /// updated install/uninstall signatures get a non-default config target
    /// in each test. Using `temp_settings_path()` to derive both keeps each
    /// test isolated and self-cleaning.
    fn temp_config_path() -> PathBuf {
        let mut p = temp_settings_path();
        let stem = p.file_stem().unwrap().to_string_lossy().to_string();
        p.set_file_name(format!("{stem}.cc-plugin.json"));
        p
    }

    #[test]
    fn install_creates_settings_file_when_missing() {
        let path = temp_settings_path();
        let cfg_path = temp_config_path();
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&cfg_path);
        install(None, DEFAULT_BASE_URL, false, Some(&path), Some(&cfg_path))
            .expect("install must succeed");

        let raw = fs::read_to_string(&path).expect("file must have been created");
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert!(v["hooks"]["SessionStart"].is_array());
        assert!(v["hooks"]["Stop"].is_array());
        // No API key was provided, so no config file should have been written.
        assert!(
            !cfg_path.exists(),
            "config file must NOT be written when --api-key is omitted"
        );

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn install_with_api_key_writes_config_with_restrictive_perms() {
        let path = temp_settings_path();
        let cfg_path = temp_config_path();
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&cfg_path);

        install(
            Some("uny_gw_secret_test_token"),
            DEFAULT_BASE_URL,
            false,
            Some(&path),
            Some(&cfg_path),
        )
        .unwrap();

        // Config file exists with the key + base URL we passed.
        let raw = fs::read_to_string(&cfg_path).expect("config file written");
        let parsed: CcPluginConfig = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed.api_key, "uny_gw_secret_test_token");
        assert_eq!(parsed.base_url, DEFAULT_BASE_URL);

        // Mode 0600 enforced on Unix so the secret isn't world-readable.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&cfg_path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600, "cc-plugin config must be chmod 0600");
        }

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&cfg_path);
    }

    #[test]
    fn uninstall_removes_config_file_by_default() {
        let path = temp_settings_path();
        let cfg_path = temp_config_path();
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&cfg_path);

        install(
            Some("uny_gw_test"),
            DEFAULT_BASE_URL,
            false,
            Some(&path),
            Some(&cfg_path),
        )
        .unwrap();
        assert!(cfg_path.exists(), "precondition: config was written");

        uninstall(Some(&path), Some(&cfg_path), /* keep_config */ false).unwrap();
        assert!(
            !cfg_path.exists(),
            "uninstall (without --keep-config) must remove the config file"
        );

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn uninstall_with_keep_config_leaves_config_intact() {
        let path = temp_settings_path();
        let cfg_path = temp_config_path();
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&cfg_path);

        install(
            Some("uny_gw_test"),
            DEFAULT_BASE_URL,
            false,
            Some(&path),
            Some(&cfg_path),
        )
        .unwrap();
        uninstall(Some(&path), Some(&cfg_path), /* keep_config */ true).unwrap();
        assert!(
            cfg_path.exists(),
            "uninstall --keep-config must preserve the config file for re-install"
        );

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&cfg_path);
    }

    #[test]
    fn redact_api_key_shows_prefix_and_suffix_only() {
        // Long key: 10 chars prefix, ellipsis, last 4. Enough to identify
        // without leaking the secret on screen-shares / asciinema recordings.
        let red = redact_api_key("uny_gw_abcdefghijklmnopqrstuvwxyz1234");
        assert_eq!(red, "uny_gw_abc…1234");
        // Very short keys (shouldn't happen in practice) get fully redacted.
        assert_eq!(redact_api_key("short"), "***");
    }

    #[test]
    fn install_then_uninstall_round_trips_to_a_clean_file() {
        let path = temp_settings_path();
        let cfg_path = temp_config_path();
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&cfg_path);
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

        install(None, DEFAULT_BASE_URL, false, Some(&path), Some(&cfg_path)).unwrap();
        uninstall(Some(&path), Some(&cfg_path), /* keep_config */ false).unwrap();

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

    // ── fetch_session integration (wiremock) ──────────────────────────────

    use wiremock::matchers::{body_partial_json, header, method, path as wpath};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn fetch_session_pulls_block_and_blueprint_signal_from_200() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(wpath("/v1/cc/session"))
            .and(header("Authorization", "Bearer uny_gw_test_key"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "blueprints": [
                    {"id": "00000000-0000-0000-0000-000000000001","name":"x","text":"y","estimated_tokens":1},
                    {"id": "00000000-0000-0000-0000-000000000002","name":"z","text":"w","estimated_tokens":2}
                ],
                "estimated_tokens_total": 3,
                "system_reminder_block": "<system-reminder>\nhello world\n</system-reminder>\n"
            })))
            .mount(&mock)
            .await;

        let result = fetch_session(&mock.uri(), "uny_gw_test_key")
            .await
            .expect("fetch_session must succeed against the mock");
        assert_eq!(
            result.block, "<system-reminder>\nhello world\n</system-reminder>\n",
            "block returned verbatim — formatting is server-owned, the CLI is a dumb pipe"
        );
        assert_eq!(
            result.blueprint_ids,
            vec![
                "00000000-0000-0000-0000-000000000001".to_string(),
                "00000000-0000-0000-0000-000000000002".to_string()
            ],
            "blueprint ids parsed for the later audit event"
        );
        assert_eq!(result.blueprint_tokens, 3);
    }

    #[tokio::test]
    async fn fetch_session_returns_err_on_non_2xx() {
        // 401 (auth fail) is the canonical case — SessionStart handler will
        // surface this on stderr and exit 0, leaving CC's session unaffected.
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(wpath("/v1/cc/session"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                "error": {"message": "Invalid API key", "type": "authentication_error"}
            })))
            .mount(&mock)
            .await;

        let res = fetch_session(&mock.uri(), "uny_gw_bad").await;
        assert!(res.is_err(), "401 must surface as Err");
        assert!(res.unwrap_err().contains("401"));
    }

    #[tokio::test]
    async fn fetch_session_strips_trailing_slash_from_base_url() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(wpath("/v1/cc/session"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "blueprints": [],
                "estimated_tokens_total": 0,
                "system_reminder_block": ""
            })))
            .mount(&mock)
            .await;

        let result = fetch_session(&format!("{}/", mock.uri()), "uny_gw_test")
            .await
            .expect("trailing slash must be normalised");
        assert_eq!(result.block, "");
        assert!(result.blueprint_ids.is_empty());
    }

    #[tokio::test]
    async fn session_handler_writes_nothing_to_stdout_when_config_missing() {
        // Fail-soft contract: missing config → exit 0, no stdout output, just
        // a stderr breadcrumb. We exercise the path and assert it returns Ok.
        let nonexistent = std::env::temp_dir().join(format!(
            "mx-cc-plugin-nonexistent-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_file(&nonexistent);
        let res = session_handler(Some(&nonexistent), None).await;
        assert!(res.is_ok(), "missing config must NOT propagate as an error");
    }

    // ── Phase 3b: session state + audit ───────────────────────────────────

    #[test]
    fn session_id_from_hook_sanitizes_path_traversal() {
        // A hostile session_id must not let the state file escape the sessions
        // dir. Non-[alnum-] chars become `_`.
        let payload = json!({ "session_id": "../../etc/passwd" });
        let sid = session_id_from_hook(&payload).unwrap();
        assert!(!sid.contains('/'), "slashes stripped: {sid}");
        assert!(!sid.contains('.'), "dots stripped: {sid}");
        // "../../etc/passwd" → the 6 leading `./` chars + the inner `/` become `_`.
        assert_eq!(sid, "______etc_passwd");
    }

    #[test]
    fn session_id_from_hook_passes_uuid_through_and_rejects_empty() {
        let uuid = "07eb4652-c7f7-4b1a-82d0-e17ab627ef6d";
        assert_eq!(
            session_id_from_hook(&json!({ "session_id": uuid })).as_deref(),
            Some(uuid)
        );
        assert_eq!(session_id_from_hook(&json!({ "session_id": "" })), None);
        assert_eq!(session_id_from_hook(&json!({})), None);
    }

    #[test]
    fn session_state_path_is_under_a_sessions_dir_beside_config() {
        let cfg = PathBuf::from("/home/u/.config/unyform/cc-plugin.json");
        let p = session_state_path(&cfg, "abc-123");
        assert_eq!(
            p,
            PathBuf::from("/home/u/.config/unyform/sessions/abc-123.json")
        );
    }

    #[test]
    fn write_then_read_session_state_round_trips() {
        let path = temp_config_path();
        let _ = fs::remove_file(&path);
        let state = SessionState {
            blueprint_ids: vec!["a".into(), "b".into()],
            blueprint_tokens: 1514,
            base_url: "https://gateway.unyform.ai".into(),
            started_at_unix: 1_700_000_000,
        };
        write_session_state(&path, &state).unwrap();
        let read = read_session_state(&path).expect("state reads back");
        assert_eq!(read.blueprint_ids, state.blueprint_ids);
        assert_eq!(read.blueprint_tokens, 1514);
        assert_eq!(read.started_at_unix, 1_700_000_000);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn parse_transcript_usage_sums_tokens_and_takes_last_model() {
        // Mirror CC's real transcript JSONL: assistant lines carry
        // message.{model, usage}; user/other lines are skipped. Two assistant
        // turns + one user line + one garbage line (must be tolerated).
        let path = temp_config_path();
        let _ = fs::remove_file(&path);
        let jsonl = [
            json!({"type":"user","message":{"role":"user","content":"hi"}}).to_string(),
            json!({"type":"assistant","message":{"model":"claude-sonnet-4-5","usage":{
                "input_tokens":10,"cache_creation_input_tokens":100,"cache_read_input_tokens":1000,"output_tokens":50
            }}}).to_string(),
            "not json at all".to_string(),
            json!({"type":"assistant","message":{"model":"claude-opus-4-7","usage":{
                "input_tokens":5,"cache_creation_input_tokens":0,"cache_read_input_tokens":2000,"output_tokens":300
            }}}).to_string(),
        ]
        .join("\n");
        fs::write(&path, jsonl).unwrap();

        let usage = parse_transcript_usage(&path);
        // prompt = (10+100) + (5+0) = 115 — input + cache_creation; cache_read
        // (1000, 2000) EXCLUDED to avoid the multiplicative re-read inflation.
        assert_eq!(
            usage.prompt_tokens, 115,
            "sums input + cache_creation across turns, excluding cache_read"
        );
        // completion = 50 + 300
        assert_eq!(usage.completion_tokens, 350);
        // last assistant turn's model wins
        assert_eq!(usage.model.as_deref(), Some("claude-opus-4-7"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn parse_transcript_usage_is_zero_for_missing_or_empty_transcript() {
        // Missing file → all-zero, no model (the audit endpoint backfills).
        let missing = std::env::temp_dir().join(format!(
            "mx-cc-no-transcript-{}-{}.jsonl",
            std::process::id(),
            unix_now()
        ));
        let _ = fs::remove_file(&missing);
        let usage = parse_transcript_usage(&missing);
        assert_eq!(usage.prompt_tokens, 0);
        assert_eq!(usage.completion_tokens, 0);
        assert!(usage.model.is_none());
    }

    #[tokio::test]
    async fn post_audit_sends_blueprint_signal_and_succeeds_on_200() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(wpath("/v1/cc/audit"))
            .and(header("Authorization", "Bearer uny_gw_k"))
            .and(body_partial_json(json!({
                "model": "claude-opus-4-7",
                "prompt_tokens": 5000,
                "completion_tokens": 800,
                "blueprint_ids": ["a", "b"],
                "blueprint_tokens": 1514,
                "duration_ms": 4200
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "recorded": true })))
            .mount(&mock)
            .await;

        let event = AuditEvent {
            model: Some("claude-opus-4-7".into()),
            prompt_tokens: 5000,
            completion_tokens: 800,
            blueprint_ids: vec!["a".into(), "b".into()],
            blueprint_tokens: 1514,
            duration_ms: 4200,
        };
        post_audit(&mock.uri(), "uny_gw_k", &event)
            .await
            .expect("post_audit must succeed when the body matches and server 200s");
    }

    #[tokio::test]
    async fn post_audit_returns_err_on_non_2xx() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(wpath("/v1/cc/audit"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock)
            .await;
        let res = post_audit(&mock.uri(), "uny_gw_bad", &AuditEvent::default()).await;
        assert!(res.is_err(), "401 must surface as Err");
        assert!(res.unwrap_err().contains("401"));
    }

    #[tokio::test]
    async fn stop_handler_posts_audit_from_state_then_removes_it() {
        // End-to-end of the Stop path WITHOUT real stdin: we seed config +
        // state file, point a wiremock at /v1/cc/audit, and call the
        // post-audit + cleanup logic the handler runs once it has a session.
        let cfg_path = temp_config_path();
        let _ = fs::remove_file(&cfg_path);
        save_cc_plugin_config(
            &cfg_path,
            &CcPluginConfig {
                api_key: "uny_gw_stop_key".into(),
                base_url: "http://unused".into(),
            },
        )
        .unwrap();

        let session_id = "sess-stop-test";
        let state_path = session_state_path(&cfg_path, session_id);
        write_session_state(
            &state_path,
            &SessionState {
                blueprint_ids: vec!["bp1".into()],
                blueprint_tokens: 42,
                base_url: "http://will-be-overridden".into(),
                started_at_unix: unix_now().saturating_sub(3),
            },
        )
        .unwrap();
        assert!(state_path.exists(), "precondition: state file written");

        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(wpath("/v1/cc/audit"))
            .and(header("Authorization", "Bearer uny_gw_stop_key"))
            .and(body_partial_json(json!({
                "blueprint_ids": ["bp1"],
                "blueprint_tokens": 42
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "recorded": true })))
            .mount(&mock)
            .await;

        // Drive the same steps stop_handler runs after resolving session_id:
        // load state, POST, remove state file.
        let state = read_session_state(&state_path).unwrap();
        let duration_ms = unix_now()
            .saturating_sub(state.started_at_unix)
            .saturating_mul(1000);
        let event = AuditEvent {
            blueprint_ids: state.blueprint_ids,
            blueprint_tokens: state.blueprint_tokens,
            duration_ms,
            ..Default::default()
        };
        post_audit(&mock.uri(), "uny_gw_stop_key", &event)
            .await
            .expect("audit POST succeeds");
        let _ = fs::remove_file(&state_path);
        assert!(
            !state_path.exists(),
            "state file consumed after audit so the sessions dir doesn't grow"
        );

        let _ = fs::remove_file(&cfg_path);
    }

    #[tokio::test]
    async fn stop_handler_is_noop_without_session_or_state() {
        // No stdin payload (test harness stdin isn't a hook payload) → handler
        // must return Ok without touching the network. Config exists but there
        // is no session_id, so it exits early.
        let cfg_path = temp_config_path();
        let _ = fs::remove_file(&cfg_path);
        save_cc_plugin_config(
            &cfg_path,
            &CcPluginConfig {
                api_key: "uny_gw_k".into(),
                base_url: "http://unused".into(),
            },
        )
        .unwrap();
        let res = stop_handler(Some(&cfg_path), None).await;
        assert!(
            res.is_ok(),
            "stop must fail soft when there's nothing to audit"
        );
        let _ = fs::remove_file(&cfg_path);
    }
}
