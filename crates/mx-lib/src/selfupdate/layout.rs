//! The release install layout: `releases/`, `current`, `tmp/`, shims.
//!
//! This is the only writer of `<home>/{releases,current,tmp}` (spec §3.2).
//! Every entry point takes the MechCrate home explicitly so the whole module
//! is testable under a tempdir, and every operation is idempotent and safe to
//! re-run after a crash: extraction stages under `tmp/` and lands with a
//! rename, and `current` is repointed by renaming a fresh symlink over it —
//! never by removing and recreating it.
//!
//! Unix only: the layout is built out of symlinks, and mx publishes only
//! macOS and Linux targets.

use std::collections::HashSet;
use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use semver::Version;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::selfupdate::refresh::copy_dir_recursive;
use crate::selfupdate::target::bundle_dir_name;
use crate::selfupdate::version::parse;

/// The executables shimmed into the user's bin directory.
pub const SHIMS: [&str; 3] = ["mx", "mx-mcp", "mx-ingest"];

/// What [`Layout::ensure_shims`] did, and whether the shims are reachable.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShimReport {
    /// Shims that did not exist and were created.
    pub created: Vec<PathBuf>,
    /// Shims that pointed somewhere else and were repointed.
    pub repaired: Vec<PathBuf>,
    /// Whether the shim directory appears in the PATH that was passed in.
    pub on_path: bool,
}

/// The release layout rooted at a MechCrate home (`~/.mech-crate`).
#[derive(Debug, Clone)]
pub struct Layout {
    home: PathBuf,
}

impl Layout {
    /// A layout rooted at `home`.
    pub fn new(home: &Path) -> Self {
        Self {
            home: home.to_path_buf(),
        }
    }

    /// The MechCrate home this layout writes into.
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// `<home>/releases` — one directory per installed release.
    pub fn releases_dir(&self) -> PathBuf {
        self.home.join("releases")
    }

    /// `<home>/current` — the symlink an update flips.
    pub fn current_link(&self) -> PathBuf {
        self.home.join("current")
    }

    /// `<home>/tmp` — scratch space; nothing here survives a call.
    pub fn tmp_dir(&self) -> PathBuf {
        self.home.join("tmp")
    }

    /// Where release `version` lives once installed.
    pub fn release_dir(&self, version: &Version) -> PathBuf {
        self.releases_dir().join(bundle_dir_name(version))
    }

    /// Extract a release tarball into `releases/mx-v<version>`.
    ///
    /// The tarball is unpacked into a fresh `tmp/<uuid>/`, checked to contain
    /// exactly one top-level `mx-v<version>` directory, then renamed into
    /// place. On any failure the scratch directory is removed and `releases/`
    /// is left exactly as it was.
    pub fn extract(&self, tarball: &Path, version: &Version) -> Result<PathBuf> {
        let scratch = self.scratch()?;
        let result = self.unpack_into(tarball, version, &scratch);
        let _ = fs::remove_dir_all(&scratch);
        result
    }

    /// Install an already-extracted bundle directory as `version` (`--from-dir`).
    ///
    /// The bundle is copied — never moved — so the directory the user pointed
    /// at survives; the copy is staged under `tmp/` and renamed into
    /// `releases/`, so a partial copy is never visible there.
    pub fn adopt(&self, extracted_bundle: &Path, version: &Version) -> Result<PathBuf> {
        if !extracted_bundle.is_dir() {
            return Err(Error::NotFound(format!(
                "bundle directory {}",
                extracted_bundle.display()
            )));
        }
        let scratch = self.scratch()?;
        let staged = scratch.join(bundle_dir_name(version));
        let result = copy_dir_recursive(extracted_bundle, &staged, &mut |_| {})
            .and_then(|_| self.land(&staged, version));
        let _ = fs::remove_dir_all(&scratch);
        result
    }

    /// Point `current` at `version` by renaming a fresh symlink over it.
    ///
    /// The rename is atomic: readers see either the old release or the new
    /// one, and a failure leaves the previous `current` intact.
    pub fn flip_current(&self, version: &Version) -> Result<()> {
        let release = self.release_dir(version);
        if !release.is_dir() {
            return Err(Error::NotFound(format!(
                "release {} (looked in {})",
                bundle_dir_name(version),
                self.releases_dir().display()
            )));
        }
        let staging = self.home.join("current.new");
        if fs::symlink_metadata(&staging).is_ok() {
            fs::remove_file(&staging)?;
        }
        // Relative, so the home directory can be moved wholesale.
        let target = Path::new("releases").join(bundle_dir_name(version));
        symlink(&target, &staging)?;
        fs::rename(&staging, self.current_link()).map_err(|e| {
            let _ = fs::remove_file(&staging);
            Error::SelfUpdate(format!(
                "could not point {} at {}: {e}",
                self.current_link().display(),
                target.display()
            ))
        })
    }

    /// The version `current` points at, or `None` when it does not exist.
    pub fn current(&self) -> Result<Option<Version>> {
        let link = self.current_link();
        match fs::symlink_metadata(&link) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(Error::Io(e)),
            Ok(_) => {}
        }
        let target = fs::read_link(&link)
            .map_err(|e| Error::SelfUpdate(format!("{} is not a symlink: {e}", link.display())))?;
        let name = target.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
            Error::SelfUpdate(format!("{} points nowhere useful", link.display()))
        })?;
        let bare = name.strip_prefix("mx-").ok_or_else(|| {
            Error::SelfUpdate(format!(
                "{} points at '{name}', which is not a release directory",
                link.display()
            ))
        })?;
        Ok(Some(parse(bare)?))
    }

    /// Every installed release, oldest first.
    pub fn installed(&self) -> Result<Vec<Version>> {
        let entries = match fs::read_dir(self.releases_dir()) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(Error::Io(e)),
        };
        let mut versions = Vec::new();
        for entry in entries {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let name = entry.file_name();
            let Some(bare) = name.to_str().and_then(|n| n.strip_prefix("mx-")) else {
                continue;
            };
            if let Ok(version) = parse(bare) {
                versions.push(version);
            }
        }
        versions.sort();
        Ok(versions)
    }

    /// The newest installed release that is not the current one — what
    /// `--rollback` flips back to.
    pub fn previous(&self) -> Result<Option<Version>> {
        let current = self.current()?;
        Ok(self
            .installed()?
            .into_iter()
            .rev()
            .find(|v| Some(v) != current.as_ref()))
    }

    /// Remove installed releases beyond the `keep` newest.
    ///
    /// The current release is always kept, even when it is older than the
    /// ones retained by `keep`, so a rollback is never pruned out from under
    /// the running binary. Returns what was removed, oldest first.
    pub fn prune(&self, keep: usize) -> Result<Vec<Version>> {
        let current = self.current()?;
        let installed = self.installed()?;
        let retained: HashSet<&Version> = installed
            .iter()
            .rev()
            .take(keep)
            .chain(current.iter())
            .collect();

        let mut removed = Vec::new();
        for version in &installed {
            if retained.contains(version) {
                continue;
            }
            fs::remove_dir_all(self.release_dir(version))?;
            removed.push(version.clone());
        }
        Ok(removed)
    }

    /// Create or repair the `mx`, `mx-mcp` and `mx-ingest` symlinks in
    /// `bin_dir`, all pointing through `<home>/current/bin/`.
    ///
    /// A shim that is already correct is left alone; one pointing elsewhere is
    /// repointed. A regular file in the way is an error — it may be a real
    /// binary the user installed, and this function never deletes one.
    /// `path_env` is the PATH to test `bin_dir` against (callers pass
    /// `std::env::var("PATH")`); nothing is read from the environment here.
    pub fn ensure_shims(&self, bin_dir: &Path, path_env: Option<&str>) -> Result<ShimReport> {
        fs::create_dir_all(bin_dir)?;
        let mut report = ShimReport {
            on_path: on_path(bin_dir, path_env),
            ..Default::default()
        };
        for name in SHIMS {
            let target = self.current_link().join("bin").join(name);
            let link = bin_dir.join(name);
            match fs::symlink_metadata(&link) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    symlink(&target, &link)?;
                    report.created.push(link);
                }
                Err(e) => return Err(Error::Io(e)),
                Ok(meta) if meta.file_type().is_symlink() => {
                    if fs::read_link(&link)? == target {
                        continue;
                    }
                    fs::remove_file(&link)?;
                    symlink(&target, &link)?;
                    report.repaired.push(link);
                }
                Ok(_) => {
                    return Err(Error::SelfUpdate(format!(
                        "{} already exists and is not a symlink; move it aside and re-run",
                        link.display()
                    )))
                }
            }
        }
        Ok(report)
    }

    /// A fresh, empty `tmp/<uuid>` directory.
    fn scratch(&self) -> Result<PathBuf> {
        let dir = self.tmp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    /// Unpack `tarball` into `scratch` and land the bundle it contains.
    fn unpack_into(&self, tarball: &Path, version: &Version, scratch: &Path) -> Result<PathBuf> {
        let file = fs::File::open(tarball)
            .map_err(|e| Error::SelfUpdate(format!("cannot read {}: {e}", tarball.display())))?;
        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(file));
        archive.unpack(scratch).map_err(|e| {
            Error::SelfUpdate(format!("extracting {} failed: {e}", tarball.display()))
        })?;

        let wanted = bundle_dir_name(version);
        let mut tops: Vec<String> = Vec::new();
        for entry in fs::read_dir(scratch)? {
            tops.push(entry?.file_name().to_string_lossy().into_owned());
        }
        let staged = scratch.join(&wanted);
        if tops.len() != 1 || !staged.is_dir() {
            tops.sort();
            return Err(Error::SelfUpdate(format!(
                "{} should contain exactly one top-level directory '{wanted}', found [{}]",
                tarball.display(),
                tops.join(", ")
            )));
        }
        self.land(&staged, version)
    }

    /// Move a staged bundle from `tmp/` into `releases/`, replacing any
    /// half-installed directory of the same version.
    fn land(&self, staged: &Path, version: &Version) -> Result<PathBuf> {
        let dest = self.release_dir(version);
        fs::create_dir_all(self.releases_dir())?;
        if fs::symlink_metadata(&dest).is_ok() {
            fs::remove_dir_all(&dest)?;
        }
        fs::rename(staged, &dest).map_err(|e| {
            Error::SelfUpdate(format!(
                "could not install {} into {}: {e}",
                bundle_dir_name(version),
                self.releases_dir().display()
            ))
        })?;
        Ok(dest)
    }
}

/// Whether `dir` is one of the colon-separated entries of `path_env`.
fn on_path(dir: &Path, path_env: Option<&str>) -> bool {
    path_env.is_some_and(|path| {
        path.split(':')
            .filter(|entry| !entry.is_empty())
            .any(|entry| Path::new(entry) == dir)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selfupdate::version::parse;
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Write a tiny but structurally faithful release bundle at `dir`.
    fn fake_bundle(dir: &Path, version: &str) {
        fs::create_dir_all(dir.join("bin")).unwrap();
        fs::create_dir_all(dir.join("templates")).unwrap();
        fs::create_dir_all(dir.join("scripts")).unwrap();
        fs::write(
            dir.join("bin/mx"),
            format!("#!/bin/sh\necho \"mx {version}\"\n"),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(dir.join("bin/mx"), fs::Permissions::from_mode(0o755)).unwrap();
        }
        fs::write(dir.join("templates/x.txt"), "x\n").unwrap();
        fs::write(dir.join("scripts/.keep"), "").unwrap();
        fs::write(dir.join("VERSION"), format!("{version}\n")).unwrap();
    }

    /// Pack `bundle` into a gzipped tar whose single top-level dir is `root`.
    fn tarball(bundle: &Path, root: &str, out: &Path) {
        let file = fs::File::create(out).unwrap();
        let enc = flate2::write::GzEncoder::new(file, flate2::Compression::fast());
        let mut builder = tar::Builder::new(enc);
        builder.append_dir_all(root, bundle).unwrap();
        builder.into_inner().unwrap().finish().unwrap();
    }

    /// A tempdir home plus a packaged 9.9.9 tarball inside it.
    fn home_with_tarball() -> (tempfile::TempDir, PathBuf) {
        let home = tempfile::tempdir().unwrap();
        let src = home.path().join("src/mx-v9.9.9");
        fake_bundle(&src, "9.9.9");
        let tar = home.path().join("mx-v9.9.9.tar.gz");
        tarball(&src, "mx-v9.9.9", &tar);
        (home, tar)
    }

    fn tmp_entries(home: &Path) -> usize {
        match fs::read_dir(home.join("tmp")) {
            Ok(entries) => entries.count(),
            Err(_) => 0,
        }
    }

    /// Adopt `versions` into `layout` from freshly built bundles.
    fn adopt_all(layout: &Layout, scratch: &Path, versions: &[&str]) {
        for v in versions {
            let src = scratch.join(format!("bundle-{v}"));
            fake_bundle(&src, v);
            layout.adopt(&src, &parse(v).unwrap()).unwrap();
        }
    }

    #[test]
    fn extract_lands_the_bundle_in_releases_and_leaves_tmp_empty() {
        let (home, tar) = home_with_tarball();
        let layout = Layout::new(home.path());
        let dest = layout.extract(&tar, &parse("9.9.9").unwrap()).unwrap();

        assert_eq!(dest, home.path().join("releases/mx-v9.9.9"));
        assert!(dest.join("bin/mx").is_file());
        assert!(dest.join("templates/x.txt").is_file());
        assert_eq!(
            fs::read_to_string(dest.join("VERSION")).unwrap().trim(),
            "9.9.9"
        );
        assert_eq!(
            tmp_entries(home.path()),
            0,
            "tmp/ must be empty after extract"
        );
    }

    #[test]
    fn extract_of_a_corrupt_tarball_leaves_releases_untouched() {
        let home = tempfile::tempdir().unwrap();
        let tar = home.path().join("corrupt.tar.gz");
        fs::write(&tar, b"this is definitely not a gzip stream").unwrap();
        let layout = Layout::new(home.path());

        let err = layout.extract(&tar, &parse("9.9.9").unwrap()).unwrap_err();
        assert!(err.to_string().contains("9.9.9") || err.to_string().contains("extract"));
        assert!(!home.path().join("releases/mx-v9.9.9").exists());
        assert_eq!(
            tmp_entries(home.path()),
            0,
            "tmp/ must be empty after a failure"
        );
    }

    #[test]
    fn extract_rejects_a_tarball_whose_top_dir_is_not_the_bundle() {
        let home = tempfile::tempdir().unwrap();
        let src = home.path().join("src");
        fake_bundle(&src, "9.9.9");
        let tar = home.path().join("wrong.tar.gz");
        tarball(&src, "not-the-bundle", &tar);
        let layout = Layout::new(home.path());

        let err = layout.extract(&tar, &parse("9.9.9").unwrap()).unwrap_err();
        assert!(err.to_string().contains("mx-v9.9.9"), "got: {err}");
        assert!(!home.path().join("releases/mx-v9.9.9").exists());
        assert_eq!(tmp_entries(home.path()), 0);
    }

    #[test]
    fn an_extracted_bundle_passes_the_binary_version_check() {
        let (home, tar) = home_with_tarball();
        let layout = Layout::new(home.path());
        let version = parse("9.9.9").unwrap();
        let dest = layout.extract(&tar, &version).unwrap();

        crate::selfupdate::verify::check_binary_version(&dest.join("bin/mx"), &version).unwrap();
    }

    #[test]
    fn adopt_copies_a_bundle_into_releases_and_keeps_the_source() {
        let home = tempfile::tempdir().unwrap();
        let src = home.path().join("elsewhere/mx-v9.9.9");
        fake_bundle(&src, "9.9.9");
        let layout = Layout::new(home.path());

        let dest = layout.adopt(&src, &parse("9.9.9").unwrap()).unwrap();
        assert_eq!(dest, home.path().join("releases/mx-v9.9.9"));
        assert!(dest.join("bin/mx").is_file());
        assert!(
            src.join("bin/mx").is_file(),
            "--from-dir must not consume the source"
        );
    }

    #[test]
    fn current_is_none_before_the_first_flip() {
        let home = tempfile::tempdir().unwrap();
        assert_eq!(Layout::new(home.path()).current().unwrap(), None);
    }

    #[test]
    fn installed_lists_release_dirs_sorted_ascending() {
        let home = tempfile::tempdir().unwrap();
        let layout = Layout::new(home.path());
        adopt_all(&layout, home.path(), &["0.2.0", "0.1.0", "0.10.0"]);
        fs::create_dir_all(home.path().join("releases/not-a-release")).unwrap();

        let versions: Vec<String> = layout
            .installed()
            .unwrap()
            .iter()
            .map(|v| v.to_string())
            .collect();
        assert_eq!(versions, vec!["0.1.0", "0.2.0", "0.10.0"]);
    }

    #[test]
    fn flip_current_points_current_at_the_release() {
        let home = tempfile::tempdir().unwrap();
        let layout = Layout::new(home.path());
        adopt_all(&layout, home.path(), &["9.9.9"]);

        layout.flip_current(&parse("9.9.9").unwrap()).unwrap();
        assert_eq!(
            fs::read_link(home.path().join("current")).unwrap(),
            PathBuf::from("releases/mx-v9.9.9")
        );
        assert_eq!(
            layout.current().unwrap().map(|v| v.to_string()),
            Some("9.9.9".to_string())
        );
        assert!(
            home.path().join("current/bin/mx").exists(),
            "current must resolve"
        );
        assert!(!home.path().join("current.new").exists());
    }

    #[test]
    fn previous_is_the_release_before_the_current_one() {
        let home = tempfile::tempdir().unwrap();
        let layout = Layout::new(home.path());
        adopt_all(&layout, home.path(), &["0.1.0", "0.2.0"]);

        layout.flip_current(&parse("0.1.0").unwrap()).unwrap();
        layout.flip_current(&parse("0.2.0").unwrap()).unwrap();

        assert_eq!(
            layout.previous().unwrap().map(|v| v.to_string()),
            Some("0.1.0".to_string())
        );
    }

    #[test]
    fn flipping_back_to_previous_rolls_back() {
        let home = tempfile::tempdir().unwrap();
        let layout = Layout::new(home.path());
        adopt_all(&layout, home.path(), &["0.1.0", "0.2.0"]);
        layout.flip_current(&parse("0.1.0").unwrap()).unwrap();
        layout.flip_current(&parse("0.2.0").unwrap()).unwrap();

        let previous = layout.previous().unwrap().unwrap();
        layout.flip_current(&previous).unwrap();

        assert_eq!(
            layout.current().unwrap().map(|v| v.to_string()),
            Some("0.1.0".to_string())
        );
        assert_eq!(
            layout.previous().unwrap().map(|v| v.to_string()),
            Some("0.2.0".to_string())
        );
    }

    #[test]
    fn prune_removes_the_oldest_beyond_keep() {
        let home = tempfile::tempdir().unwrap();
        let layout = Layout::new(home.path());
        adopt_all(&layout, home.path(), &["0.1.0", "0.2.0", "0.3.0"]);
        layout.flip_current(&parse("0.3.0").unwrap()).unwrap();

        let removed: Vec<String> = layout
            .prune(2)
            .unwrap()
            .iter()
            .map(|v| v.to_string())
            .collect();
        assert_eq!(removed, vec!["0.1.0"]);

        let left: Vec<String> = layout
            .installed()
            .unwrap()
            .iter()
            .map(|v| v.to_string())
            .collect();
        assert_eq!(left, vec!["0.2.0", "0.3.0"]);
        assert!(!home.path().join("releases/mx-v0.1.0").exists());
    }

    #[test]
    fn prune_never_removes_the_current_release() {
        let home = tempfile::tempdir().unwrap();
        let layout = Layout::new(home.path());
        adopt_all(&layout, home.path(), &["0.1.0", "0.2.0", "0.3.0"]);
        layout.flip_current(&parse("0.1.0").unwrap()).unwrap();

        let removed: Vec<String> = layout
            .prune(1)
            .unwrap()
            .iter()
            .map(|v| v.to_string())
            .collect();
        assert_eq!(
            removed,
            vec!["0.2.0"],
            "the oldest kept only because it is current"
        );

        let left: Vec<String> = layout
            .installed()
            .unwrap()
            .iter()
            .map(|v| v.to_string())
            .collect();
        assert_eq!(left, vec!["0.1.0", "0.3.0"]);
        assert!(home.path().join("current/bin/mx").exists());
    }

    #[test]
    fn ensure_shims_creates_the_three_symlinks() {
        let home = tempfile::tempdir().unwrap();
        let layout = Layout::new(home.path());
        adopt_all(&layout, home.path(), &["9.9.9"]);
        layout.flip_current(&parse("9.9.9").unwrap()).unwrap();
        let bin = home.path().join(".local/bin");

        let report = layout.ensure_shims(&bin, None).unwrap();
        assert_eq!(report.created.len(), 3, "{report:?}");
        assert!(report.repaired.is_empty());
        for name in ["mx", "mx-mcp", "mx-ingest"] {
            assert_eq!(
                fs::read_link(bin.join(name)).unwrap(),
                home.path().join("current/bin").join(name)
            );
        }

        // Idempotent: a second run changes nothing.
        let again = layout.ensure_shims(&bin, None).unwrap();
        assert!(again.created.is_empty() && again.repaired.is_empty());
    }

    #[test]
    fn ensure_shims_repairs_a_wrong_symlink() {
        let home = tempfile::tempdir().unwrap();
        let layout = Layout::new(home.path());
        let bin = home.path().join(".local/bin");
        fs::create_dir_all(&bin).unwrap();
        std::os::unix::fs::symlink("/somewhere/else/mx", bin.join("mx")).unwrap();

        let report = layout.ensure_shims(&bin, None).unwrap();
        assert_eq!(report.repaired, vec![bin.join("mx")]);
        assert_eq!(report.created.len(), 2);
        assert_eq!(
            fs::read_link(bin.join("mx")).unwrap(),
            home.path().join("current/bin/mx")
        );
    }

    #[test]
    fn ensure_shims_refuses_a_regular_file() {
        let home = tempfile::tempdir().unwrap();
        let layout = Layout::new(home.path());
        let bin = home.path().join(".local/bin");
        fs::create_dir_all(&bin).unwrap();
        fs::write(bin.join("mx"), "a real binary").unwrap();

        let err = layout.ensure_shims(&bin, None).unwrap_err();
        assert!(err.to_string().contains("mx"), "got: {err}");
        assert!(bin.join("mx").is_file(), "the real file must be left alone");
    }

    #[test]
    fn ensure_shims_reports_whether_bin_dir_is_on_path() {
        let home = tempfile::tempdir().unwrap();
        let layout = Layout::new(home.path());
        let bin = home.path().join(".local/bin");
        let on = format!("/usr/bin:{}:/bin", bin.display());

        assert!(layout.ensure_shims(&bin, Some(&on)).unwrap().on_path);
        assert!(
            !layout
                .ensure_shims(&bin, Some("/usr/bin:/bin"))
                .unwrap()
                .on_path
        );
        assert!(!layout.ensure_shims(&bin, None).unwrap().on_path);
    }
}
