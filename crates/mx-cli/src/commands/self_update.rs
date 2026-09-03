//! `mx self-update` — check for, apply, and roll back updates of the mx client.
//!
//! The command is a thin shell over `mx_lib::selfupdate`: detect how this
//! binary was installed, derive a plan, show it, and execute it. Which
//! strategy runs depends on the install kind (spec §3.3):
//!
//! - release / bare: download the tarball from the release channel, verify
//!   its sha256, extract it under `~/.mech-crate/releases`, verify the new
//!   binary, then flip `~/.mech-crate/current` and refresh templates, the
//!   version file, the MCP wrapper and the `~/.local/bin` shims;
//! - homebrew: run `brew upgrade mx`;
//! - source: rebuild the checkout (the historical behaviour, kept).
//!
//! Hidden test seams: `MX_SELFUPDATE_EXE` replaces `current_exe()` for kind
//! detection (the test binary lives under a checkout's `target/`), and
//! `HOMEBREW_PREFIX` is honoured before `brew --prefix` is consulted.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use clap::Args;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use mx_lib::selfupdate::fetch;
use mx_lib::selfupdate::index::{Release, ReleaseIndex};
use mx_lib::selfupdate::layout::Layout;
use mx_lib::selfupdate::notify::{self, Cache};
use mx_lib::selfupdate::refresh;
use mx_lib::selfupdate::verify::{check_binary_version, check_codesign, CodesignStatus};
use mx_lib::selfupdate::{
    self, detect, plan, InstallKind, Triple, UpdatePlan, Version, BREW_UPGRADE,
};

/// Exit status of `--check` when a newer release exists.
pub const UPDATE_AVAILABLE_EXIT: i32 = 10;

/// Releases kept on disk: the current one plus one to roll back to.
const KEEP_RELEASES: usize = 2;

/// Hidden env: path used for install-kind detection instead of `current_exe()`.
const EXE_OVERRIDE_ENV: &str = "MX_SELFUPDATE_EXE";

/// Whole-operation budget for the notifier's background check. It runs
/// detached from a user's shell, so it must never linger.
const REFRESH_TIMEOUT: Duration = Duration::from_secs(5);

/// Update the mx CLI itself
#[derive(Args, Debug, Default)]
pub struct SelfUpdateCommand {
    /// Only report whether a newer release exists (exit 10 when it does)
    #[arg(long, conflicts_with_all = ["dry_run", "rollback", "from_dir", "refresh_cache"])]
    check: bool,

    /// Show what would be done without making changes
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Skip interactive prompts
    #[arg(short, long)]
    yes: bool,

    /// Install this exact version instead of the latest release
    #[arg(long, value_name = "VERSION", conflicts_with_all = ["rollback", "from_dir"])]
    to: Option<String>,

    /// Switch back to the previously installed release
    #[arg(long, conflicts_with = "from_dir")]
    rollback: bool,

    /// Adopt an already-extracted release bundle (used by the installer)
    #[arg(long, value_name = "DIR")]
    from_dir: Option<PathBuf>,

    /// Source checkouts only: git pull --rebase before rebuilding
    #[arg(long)]
    pull: bool,

    /// Refresh the update-check cache and exit (used by the background check)
    #[arg(long, hide = true)]
    refresh_cache: bool,
}

impl SelfUpdateCommand {
    pub async fn run(&self) -> Result<()> {
        if self.refresh_cache {
            return refresh_update_cache().await;
        }

        let home = mx_lib::home_dir()?;
        let layout = Layout::new(&home);

        if let Some(dir) = &self.from_dir {
            return self.adopt_bundle(&layout, dir);
        }
        if self.rollback {
            return self.roll_back(&layout);
        }
        if self.check {
            return self.check_only().await;
        }

        banner("MechCrate Self-Update");

        let kind = detect_kind(&home);
        let current = selfupdate::current();
        let pin = match &self.to {
            Some(raw) => Some(selfupdate::parse(raw)?),
            None => None,
        };

        // Source and Homebrew installs never consult the release channel.
        let release = match kind {
            InstallKind::Release { .. } | InstallKind::Bare { .. } => {
                Some(self.resolve_release(pin.as_ref()).await?)
            }
            InstallKind::Homebrew { .. } | InstallKind::Source { .. } => None,
        };
        let latest = release
            .as_ref()
            .map(|r| r.version.clone())
            .unwrap_or_else(|| current.clone());
        let triple = Triple::host().unwrap_or(Triple::UniversalAppleDarwin);
        let plan = plan(&kind, &current, &latest, pin.as_ref(), triple);

        print_kind(&kind);
        println!("{} Running:      {}", arrow(), current);
        if release.is_some() {
            println!("{} Latest:       {}", arrow(), latest);
        }
        println!("{} Plan:         {}", arrow(), describe(&plan));
        println!();

        if self.dry_run {
            println!("{}", style("[DRY RUN] Nothing changed.").blue());
            return Ok(());
        }
        if matches!(plan, UpdatePlan::UpToDate { .. }) {
            println!("{} mx is up to date.", ok());
            return Ok(());
        }
        if !self.confirm(&format!("Proceed: {}?", describe(&plan)))? {
            println!("{} Cancelled.", style("ℹ").blue());
            return Ok(());
        }
        println!();

        match plan {
            UpdatePlan::UpToDate { .. } => Ok(()),
            UpdatePlan::DelegateBrew { command } => delegate_to_brew(command),
            UpdatePlan::RebuildSource { repo } => self.rebuild_from_source(&repo),
            UpdatePlan::Download {
                version,
                asset,
                checksum,
                repoint,
            } => {
                let release = release.expect("a download plan implies a resolved release");
                self.download_and_install(&layout, &release, &version, &asset, &checksum)
                    .await?;
                finish_install(&layout, &version, repoint.as_deref())
            }
        }
    }

    // ── --check ─────────────────────────────────────────────────────────

    async fn check_only(&self) -> Result<()> {
        let current = selfupdate::current();
        let latest = ReleaseIndex::from_env().latest().await?.version;
        if selfupdate::is_newer(&latest, &current) {
            println!(
                "mx {latest} is available (you have {current}). Run: {}",
                style("mx self-update").cyan()
            );
            std::io::stdout().flush()?;
            std::process::exit(UPDATE_AVAILABLE_EXIT);
        }
        println!("mx {current} is up to date (latest is {latest}).");
        Ok(())
    }

    // ── --from-dir ──────────────────────────────────────────────────────

    fn adopt_bundle(&self, layout: &Layout, dir: &Path) -> Result<()> {
        banner("MechCrate Self-Update");
        let dir = dir
            .canonicalize()
            .with_context(|| format!("bundle directory {} not found", dir.display()))?;
        let version = bundle_version(&dir)?;
        println!("{} Bundle:  {}", arrow(), dir.display());
        println!("{} Version: {}", arrow(), version);
        println!();
        if self.dry_run {
            println!("{}", style("[DRY RUN] Nothing changed.").blue());
            return Ok(());
        }
        if !self.confirm(&format!("Install {version} from this bundle?"))? {
            println!("{} Cancelled.", style("ℹ").blue());
            return Ok(());
        }

        let release_dir = layout.adopt(&dir, &version)?;
        println!("{} Adopted into {}", ok(), release_dir.display());
        verify_release(layout, &release_dir, &version)?;
        finish_install(layout, &version, None)
    }

    // ── --rollback ──────────────────────────────────────────────────────

    fn roll_back(&self, layout: &Layout) -> Result<()> {
        banner("MechCrate Self-Update");
        let current = layout.current()?;
        let previous = layout
            .previous()?
            .ok_or_else(|| anyhow!("nothing to roll back to: no other release is installed"))?;
        match &current {
            Some(c) => println!("{} Current:  {}", arrow(), c),
            None => println!("{} Current:  (none)", arrow()),
        }
        println!("{} Previous: {}", arrow(), previous);
        println!();
        if self.dry_run {
            println!("{}", style("[DRY RUN] Nothing changed.").blue());
            return Ok(());
        }
        if !self.confirm(&format!("Roll back to {previous}?"))? {
            println!("{} Cancelled.", style("ℹ").blue());
            return Ok(());
        }
        flip_and_refresh(layout, &previous)?;
        println!();
        println!("{} Rolled back to mx {previous}.", ok());
        Ok(())
    }

    // ── release channel ─────────────────────────────────────────────────

    async fn resolve_release(&self, pin: Option<&Version>) -> Result<Release> {
        let index = ReleaseIndex::from_env();
        Ok(match pin {
            Some(version) => index
                .get(version)
                .await
                .with_context(|| format!("release v{version} not found on the release channel"))?,
            None => index.latest().await?,
        })
    }

    async fn download_and_install(
        &self,
        layout: &Layout,
        release: &Release,
        version: &Version,
        asset_name: &str,
        checksum_name: &str,
    ) -> Result<()> {
        let asset = release
            .asset(asset_name)
            .ok_or_else(|| anyhow!("release v{} has no asset {asset_name}", release.version))?;
        let checksum_asset = release
            .asset(checksum_name)
            .ok_or_else(|| anyhow!("release v{} has no asset {checksum_name}", release.version))?;

        let scratch = layout.tmp_dir().join(uuid::Uuid::new_v4().to_string());
        let result = self
            .download_verify_extract(layout, asset, checksum_asset, version, &scratch)
            .await;
        let _ = std::fs::remove_dir_all(&scratch);
        result
    }

    async fn download_verify_extract(
        &self,
        layout: &Layout,
        asset: &mx_lib::selfupdate::index::Asset,
        checksum_asset: &mx_lib::selfupdate::index::Asset,
        version: &Version,
        scratch: &Path,
    ) -> Result<()> {
        let client = ReleaseIndex::from_env().client().clone();

        println!("{} Downloading {}...", arrow(), asset.name);
        let bar = ProgressBar::new(asset.size.max(1));
        bar.set_style(
            ProgressStyle::default_bar()
                .template("  [{bar:40.cyan/blue}] {bytes}/{total_bytes}")
                .expect("static template"),
        );
        let on_progress = |received: u64| bar.set_position(received);
        let tarball = fetch::download(&client, asset, scratch, Some(&on_progress)).await?;
        bar.finish_and_clear();

        let expected = fetch::fetch_checksum(&client, checksum_asset).await?;
        fetch::verify(&tarball, &expected)?;
        println!("{} sha256 verified", ok());

        let release_dir = layout.extract(&tarball, version)?;
        println!("{} Extracted to {}", ok(), release_dir.display());
        verify_release(layout, &release_dir, version)
    }

    // ── source checkout ─────────────────────────────────────────────────

    fn rebuild_from_source(&self, repo: &Path) -> Result<()> {
        let repo = if self.pull || repo.join("crates/mx-cli").exists() {
            repo.to_path_buf()
        } else {
            self.find_source_dir()?
        };
        let bin_dir = repo.join("bin");
        println!("{} Source: {}", arrow(), repo.display());
        println!("{} Target: {}", arrow(), bin_dir.display());
        println!();

        if self.pull {
            println!("{} Pulling latest changes...", arrow());
            let status = Command::new("git")
                .args(["pull", "--rebase"])
                .current_dir(&repo)
                .status()?;
            if !status.success() {
                bail!("git pull failed");
            }
            println!("  {} Git pull complete", ok());
            println!();
        }

        println!("{} Building release binaries...", arrow());
        let status = Command::new("cargo")
            .args(["build", "--release", "-p", "mx-cli", "-p", "mx-mcp-server"])
            .current_dir(&repo)
            .status()
            .context("cargo not found; install Rust from https://rustup.rs")?;
        if !status.success() {
            bail!("cargo build failed");
        }
        println!("  {} Build complete", ok());
        println!();

        println!("{} Installing binaries...", arrow());
        let release_dir = repo.join("target/release");
        std::fs::create_dir_all(&bin_dir)?;
        let binaries = [("mx", true), ("mx-mcp", false)];
        for (name, required) in binaries {
            let src = release_dir.join(name);
            if !src.exists() {
                if required {
                    bail!("Binary not found: {}", src.display());
                }
                continue;
            }
            let dst = bin_dir.join(name);
            std::fs::copy(&src, &dst)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&dst)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&dst, perms)?;
            }
            println!("  {} {} -> bin/{}", ok(), name, name);
        }
        println!();

        ensure_system_symlinks(&bin_dir, &binaries)?;

        println!("{} Refreshing templates...", arrow());
        let status = Command::new(bin_dir.join("mx"))
            .args(["init", "--force"])
            .env("MECH_CRATE_ROOT", &repo)
            .status()?;
        if !status.success() {
            println!(
                "  {} Template refresh failed (non-fatal)",
                style("⚠").yellow()
            );
        }

        println!();
        println!("{} Verifying...", arrow());
        let output = Command::new(bin_dir.join("mx"))
            .args(["--version"])
            .output()?;
        if output.status.success() {
            println!(
                "  {} {}",
                ok(),
                String::from_utf8_lossy(&output.stdout).trim()
            );
        }
        println!();
        println!("{} Update complete!", ok());
        Ok(())
    }

    /// Find a mech-crate checkout when the running binary is not inside one:
    /// `MECH_CRATE_ROOT`, then the root `mx init` recorded, then the
    /// executable's ancestors, then a few conventional locations.
    fn find_source_dir(&self) -> Result<PathBuf> {
        if let Ok(root) = std::env::var("MECH_CRATE_ROOT") {
            let path = PathBuf::from(&root);
            if is_mech_crate_root(&path) {
                return Ok(path);
            }
        }

        if let Some(root) = mx_lib::recorded_source_root() {
            if is_mech_crate_root(&root) {
                return Ok(root);
            }
        }

        if let Ok(exe) = std::env::current_exe() {
            let exe = exe.canonicalize().unwrap_or(exe);
            if let Some(repo) = exe.ancestors().skip(1).find(|p| is_mech_crate_root(p)) {
                return Ok(repo.to_path_buf());
            }
        }

        if let Some(home) = dirs::home_dir() {
            for rel in [
                "dev/dev916/mech-crate",
                "dev/mech-crate",
                "code/mech-crate",
                "projects/mech-crate",
            ] {
                let path = home.join(rel);
                if is_mech_crate_root(&path) {
                    return Ok(path);
                }
            }
        }

        bail!(
            "Could not find a mech-crate checkout to rebuild from.\n\n\
             Set MECH_CRATE_ROOT=/path/to/mech-crate, or run 'mx init' from inside \
             the checkout so its location is recorded."
        )
    }

    fn confirm(&self, prompt: &str) -> Result<bool> {
        if self.yes {
            return Ok(true);
        }
        Ok(dialoguer::Confirm::new()
            .with_prompt(prompt)
            .default(true)
            .interact()?)
    }
}

// ── helpers ─────────────────────────────────────────────────────────────

fn is_mech_crate_root(path: &Path) -> bool {
    path.join("Cargo.toml").exists() && path.join("crates/mx-cli").exists()
}

/// `--refresh-cache`: the notifier's background half (spec §3.6).
///
/// Ask the release channel what the latest version is and record the answer
/// at `<home>/cache/update-check.json`. A failure — offline, rate limited,
/// slow — is not an error: the cache keeps whatever the last good check
/// found and is dated forward by [`notify::OFFLINE_BACKOFF`] so an offline
/// laptop forks at most one check an hour. Prints nothing either way; the
/// process runs detached with its stdio on `/dev/null`.
async fn refresh_update_cache() -> Result<()> {
    let home = mx_lib::home_dir()?;
    let previous = notify::read_cache(&home);
    let now = chrono::Utc::now();

    let latest = ReleaseIndex::from_env()
        .with_timeout(REFRESH_TIMEOUT)
        .latest()
        .await
        .ok()
        .map(|r| r.version);

    let (latest, next_check_at) = match latest {
        Some(v) => (Some(v), now + notify::CHECK_TTL),
        // Keep the last known answer so a hint survives a flaky network.
        None => (
            previous.as_ref().and_then(|c| c.latest.clone()),
            now + notify::OFFLINE_BACKOFF,
        ),
    };

    let cache = Cache {
        checked_at: now,
        next_check_at,
        latest,
        current_at_check: selfupdate::current(),
        // Hint bookkeeping belongs to the foreground; carry it forward so a
        // refresh does not make an already-printed hint print again.
        hinted_at: previous.as_ref().and_then(|c| c.hinted_at),
        hinted_version: previous.and_then(|c| c.hinted_version),
    };
    // Even the write is silent: a read-only home is not worth a word here.
    let _ = notify::write_cache(&home, &cache);
    Ok(())
}

/// Classify this binary's install. See the module docs for the test seams.
pub(crate) fn detect_kind(home: &Path) -> InstallKind {
    let exe = std::env::var_os(EXE_OVERRIDE_ENV)
        .map(PathBuf::from)
        .or_else(|| std::env::current_exe().ok())
        .map(|p| p.canonicalize().unwrap_or(p))
        .unwrap_or_else(|| PathBuf::from("mx"));
    let brew_prefix = brew_prefix();
    detect(&exe, home, brew_prefix.as_deref(), is_mech_crate_root)
}

/// `HOMEBREW_PREFIX` if set, else `brew --prefix` when brew is on PATH.
fn brew_prefix() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("HOMEBREW_PREFIX") {
        return Some(PathBuf::from(p));
    }
    let brew = which::which("brew").ok()?;
    let out = Command::new(brew).arg("--prefix").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let prefix = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!prefix.is_empty()).then(|| PathBuf::from(prefix))
}

/// The version an extracted bundle declares in its `VERSION` file.
fn bundle_version(dir: &Path) -> Result<Version> {
    let raw = std::fs::read_to_string(dir.join("VERSION"))
        .with_context(|| format!("{} has no VERSION file", dir.display()))?;
    Ok(selfupdate::parse(&raw)?)
}

/// The new binary must answer `--version` correctly; its signature is
/// checked when `codesign` is available. A failure removes the release dir.
fn verify_release(layout: &Layout, release_dir: &Path, version: &Version) -> Result<()> {
    let exe = release_dir.join("bin").join("mx");
    if let Err(e) = check_binary_version(&exe, version) {
        let _ = std::fs::remove_dir_all(release_dir);
        return Err(e.into());
    }
    println!("{} {} reports {}", ok(), exe.display(), version);
    let path_env = std::env::var("PATH").ok();
    match check_codesign(&exe, path_env.as_deref()) {
        CodesignStatus::Verified => println!("{} code signature verified", ok()),
        CodesignStatus::Skipped(why) => {
            println!("  {} signature check skipped: {why}", style("·").dim())
        }
        CodesignStatus::Failed(why) => {
            let _ = std::fs::remove_dir_all(release_dir);
            let _ = layout;
            bail!("code signature check failed: {why}");
        }
    }
    Ok(())
}

/// Point `current` at `version` and refresh everything derived from it.
fn flip_and_refresh(layout: &Layout, version: &Version) -> Result<()> {
    layout.flip_current(version)?;
    println!("{} current -> mx-v{version}", ok());
    let copied = refresh::refresh_templates(layout.home())?;
    println!("{} templates refreshed ({copied} files)", ok());
    refresh::write_version_file(layout.home())?;
    if refresh::regenerate_mcp_wrapper(layout.home())? {
        println!("{} MCP wrapper re-pointed at current/", ok());
    }
    let shims = layout.ensure_shims(&shim_dir()?, std::env::var("PATH").ok().as_deref())?;
    for link in shims.created.iter().chain(shims.repaired.iter()) {
        println!("{} {}", ok(), link.display());
    }
    if !shims.on_path {
        println!();
        println!(
            "  {} {} is not on your PATH. Add to your shell profile:",
            style("⚠").yellow(),
            shim_dir()?.display()
        );
        println!("    export PATH=\"$HOME/.local/bin:$PATH\"");
    }
    Ok(())
}

/// Everything after a release is verified on disk.
fn finish_install(layout: &Layout, version: &Version, repoint: Option<&Path>) -> Result<()> {
    flip_and_refresh(layout, version)?;
    let removed = layout.prune(KEEP_RELEASES)?;
    for old in removed {
        println!("  {} removed mx-v{old}", style("·").dim());
    }
    if let Some(old_exe) = repoint {
        repoint_bare_exe(layout, old_exe);
    }
    println!();
    println!("{} mx {version} installed.", ok());
    Ok(())
}

/// A bare install's old executable path becomes a symlink into the layout
/// when its directory is writable; otherwise the user gets the two lines.
fn repoint_bare_exe(layout: &Layout, old_exe: &Path) {
    let target = layout.current_link().join("bin").join("mx");
    let attempt = old_exe.parent().ok_or(()).and_then(|dir| {
        let staging = dir.join(".mx.new");
        let _ = std::fs::remove_file(&staging);
        std::os::unix::fs::symlink(&target, &staging)
            .and_then(|_| std::fs::rename(&staging, old_exe))
            .map_err(|_| {
                let _ = std::fs::remove_file(&staging);
            })
    });
    match attempt {
        Ok(()) => println!("{} {} -> {}", ok(), old_exe.display(), target.display()),
        Err(()) => {
            println!();
            println!(
                "  {} could not replace {} (not writable). Either:",
                style("⚠").yellow(),
                old_exe.display()
            );
            println!("    sudo ln -sf {} {}", target.display(), old_exe.display());
            println!("  or remove it and use the ~/.local/bin/mx shim.");
        }
    }
}

fn shim_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|h| h.join(".local").join("bin"))
        .ok_or_else(|| anyhow!("could not determine the home directory"))
}

fn delegate_to_brew(command: &str) -> Result<()> {
    if which::which("brew").is_err() {
        bail!("Homebrew owns this install but 'brew' is not on PATH; run: {command}");
    }
    println!("{} Running: {command}", arrow());
    let status = Command::new("brew").args(["upgrade", "mx"]).status()?;
    if !status.success() {
        bail!("'{command}' failed with {status}");
    }
    println!("{} Homebrew upgrade complete.", ok());
    Ok(())
}

/// The historical `/usr/local/bin` symlinks for source installs.
fn ensure_system_symlinks(bin_dir: &Path, binaries: &[(&str, bool)]) -> Result<()> {
    println!("{} Checking /usr/local/bin symlinks...", arrow());
    let system_bin = PathBuf::from("/usr/local/bin");
    let mut stale = Vec::new();
    for (name, _) in binaries {
        let bin_path = bin_dir.join(name);
        if !bin_path.exists() {
            continue;
        }
        let system_path = system_bin.join(name);
        let correct = system_path.is_symlink()
            && std::fs::read_link(&system_path)
                .map(|t| t == bin_path)
                .unwrap_or(false);
        if correct {
            println!("  {} {} (symlink ok)", ok(), name);
        } else {
            println!("  {} {} needs symlink update", style("→").yellow(), name);
            stale.push((bin_path, system_path, *name));
        }
    }
    if stale.is_empty() {
        println!();
        return Ok(());
    }
    println!();
    println!("  {} Creating symlinks (requires sudo)...", arrow());
    for (bin_path, system_path, name) in stale {
        let _ = Command::new("sudo")
            .args(["rm", "-f"])
            .arg(&system_path)
            .status();
        let status = Command::new("sudo")
            .args(["ln", "-sf"])
            .arg(&bin_path)
            .arg(&system_path)
            .status()?;
        if status.success() {
            println!("  {} /usr/local/bin/{name} -> bin/{name}", ok());
        } else {
            println!(
                "  {} Failed to symlink /usr/local/bin/{name}",
                style("✗").red()
            );
        }
    }
    println!();
    Ok(())
}

fn describe(plan: &UpdatePlan) -> String {
    match plan {
        UpdatePlan::UpToDate { current } => format!("up to date ({current})"),
        UpdatePlan::Download {
            version,
            asset,
            repoint,
            ..
        } => {
            let mut s = format!("download {asset}, verify sha256, install {version}");
            if let Some(exe) = repoint {
                s.push_str(&format!(", re-point {}", exe.display()));
            }
            s
        }
        UpdatePlan::DelegateBrew { command } => format!("run: {command}"),
        UpdatePlan::RebuildSource { repo } => format!("rebuild from {}", repo.display()),
    }
}

fn print_kind(kind: &InstallKind) {
    let detail = match kind {
        InstallKind::Release { home, version } => {
            format!("{} (mx-v{version})", home.join("releases").display())
        }
        InstallKind::Homebrew { cellar } => cellar.display().to_string(),
        InstallKind::Source { repo } => repo.display().to_string(),
        InstallKind::Bare { exe } => exe.display().to_string(),
    };
    println!("{} Install kind: {} ({detail})", arrow(), kind.name());
    if let InstallKind::Homebrew { .. } = kind {
        println!(
            "  {} Homebrew owns this install: {}",
            style("·").dim(),
            BREW_UPGRADE
        );
    }
}

fn banner(title: &str) {
    println!();
    println!(
        "{}{}{}",
        style("  ").on_cyan(),
        style(format!(" 🦝 {title} ")).bold().on_cyan(),
        style("  ").on_cyan()
    );
    println!();
}

fn arrow() -> console::StyledObject<&'static str> {
    style("→").cyan().bold()
}

fn ok() -> console::StyledObject<&'static str> {
    style("✓").green()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    /// `mx init` records the source repo root through
    /// [`mx_lib::paths::save_source_root`] (`~/.mech-crate/config/source-root`).
    /// The resolver used for rebuilds must find that recording (bd
    /// mech-crate-gjl: it used to read a different marker file).
    #[test]
    fn self_update_finds_the_source_root_recorded_by_init() {
        let home = tempfile::tempdir().expect("setup: home tempdir");
        let repo = tempfile::tempdir().expect("setup: repo tempdir");
        std::fs::write(repo.path().join("Cargo.toml"), "[workspace]\n")
            .expect("setup: write fake Cargo.toml");
        std::fs::create_dir_all(repo.path().join("crates/mx-cli"))
            .expect("setup: create fake crates/mx-cli");

        // Isolate from the developer's real environment: no MECH_CRATE_ROOT
        // short-circuit, and a home that holds nothing but what init records.
        // nextest runs one process per test, so these mutations are contained.
        env::set_var("HOME", home.path());
        env::remove_var("MECH_CRATE_ROOT");

        mx_lib::paths::save_source_root(repo.path()).expect("setup: record source root");
        assert!(
            mx_lib::paths::recorded_source_root().is_some(),
            "setup: mx init's recording did not round-trip"
        );

        let cmd = SelfUpdateCommand {
            dry_run: true,
            ..Default::default()
        };

        let found = cmd
            .find_source_dir()
            .expect("self-update must find the source root that mx init recorded");
        assert_eq!(
            found.canonicalize().unwrap(),
            repo.path().canonicalize().unwrap(),
            "self-update resolved a different source root than mx init recorded"
        );
    }
}
