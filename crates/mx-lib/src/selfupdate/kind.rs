//! Install-kind detection (pure).
//!
//! Where the running `mx` lives decides how it can be updated (spec §3.3).
//! `detect` takes the canonicalized executable path plus the facts it needs
//! (MechCrate home, Homebrew prefix, a repo-root predicate) so it never
//! touches the filesystem itself and is testable against literal paths.

use std::path::{Path, PathBuf};

use semver::Version;

use crate::selfupdate::version::parse;

/// How the running binary was installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallKind {
    /// `<home>/releases/mx-v<version>/bin/mx` — the layout self-update owns.
    Release { home: PathBuf, version: Version },
    /// `<brew prefix>/Cellar/mx/<version>/...` — Homebrew owns it.
    Homebrew { cellar: PathBuf },
    /// Inside a mech-crate checkout (`target/release/mx` or `bin/mx`).
    Source { repo: PathBuf },
    /// A plain copied binary anywhere else.
    Bare { exe: PathBuf },
}

impl InstallKind {
    /// Short lowercase name for output and tests.
    pub fn name(&self) -> &'static str {
        match self {
            InstallKind::Release { .. } => "release",
            InstallKind::Homebrew { .. } => "homebrew",
            InstallKind::Source { .. } => "source",
            InstallKind::Bare { .. } => "bare",
        }
    }
}

/// Classify `exe` (already canonicalized by the caller).
///
/// `is_repo_root` answers "is this directory a mech-crate checkout?" —
/// the CLI passes a filesystem probe, tests pass a set.
pub fn detect(
    exe: &Path,
    home: &Path,
    brew_prefix: Option<&Path>,
    is_repo_root: impl Fn(&Path) -> bool,
) -> InstallKind {
    if let Some(kind) = release_kind(exe, home) {
        return kind;
    }
    if let Some(kind) = brew_prefix.and_then(|prefix| homebrew_kind(exe, prefix)) {
        return kind;
    }
    if let Some(repo) = exe.ancestors().skip(1).find(|dir| is_repo_root(dir)) {
        return InstallKind::Source {
            repo: repo.to_path_buf(),
        };
    }
    InstallKind::Bare {
        exe: exe.to_path_buf(),
    }
}

fn release_kind(exe: &Path, home: &Path) -> Option<InstallKind> {
    let rel = exe.strip_prefix(home.join("releases")).ok()?;
    let mut parts = rel.components();
    let bundle = parts.next()?.as_os_str().to_str()?;
    if parts.next()?.as_os_str() != "bin" {
        return None;
    }
    let version = parse(bundle.strip_prefix("mx-")?).ok()?;
    Some(InstallKind::Release {
        home: home.to_path_buf(),
        version,
    })
}

fn homebrew_kind(exe: &Path, prefix: &Path) -> Option<InstallKind> {
    let cellar_root = prefix.join("Cellar").join("mx");
    let rel = exe.strip_prefix(&cellar_root).ok()?;
    let version_dir = rel.components().next()?;
    Some(InstallKind::Homebrew {
        cellar: cellar_root.join(version_dir),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn home() -> PathBuf {
        PathBuf::from("/Users/me/.mech-crate")
    }

    fn no_repo(_: &Path) -> bool {
        false
    }

    #[test]
    fn an_exe_under_home_releases_is_a_release_install() {
        let exe = home().join("releases/mx-v0.1.2/bin/mx");
        match detect(&exe, &home(), None, no_repo) {
            InstallKind::Release { home: h, version } => {
                assert_eq!(h, home());
                assert_eq!(version.to_string(), "0.1.2");
            }
            other => panic!("expected Release, got {other:?}"),
        }
    }

    #[test]
    fn a_release_dir_with_an_unparseable_version_is_not_a_release_install() {
        let exe = home().join("releases/garbage/bin/mx");
        assert!(matches!(
            detect(&exe, &home(), None, no_repo),
            InstallKind::Bare { .. }
        ));
    }

    #[test]
    fn an_exe_in_the_homebrew_cellar_is_a_homebrew_install() {
        let prefix = PathBuf::from("/opt/homebrew");
        let exe = prefix.join("Cellar/mx/0.1.2/libexec/bin/mx");
        match detect(&exe, &home(), Some(&prefix), no_repo) {
            InstallKind::Homebrew { cellar } => {
                assert_eq!(cellar, prefix.join("Cellar/mx/0.1.2"));
            }
            other => panic!("expected Homebrew, got {other:?}"),
        }
    }

    #[test]
    fn a_cellar_path_without_a_brew_prefix_is_not_homebrew() {
        let exe = PathBuf::from("/opt/homebrew/Cellar/mx/0.1.2/libexec/bin/mx");
        assert!(matches!(
            detect(&exe, &home(), None, no_repo),
            InstallKind::Bare { .. }
        ));
    }

    #[test]
    fn an_exe_under_a_checkouts_target_release_is_a_source_install() {
        let repo = PathBuf::from("/Users/me/dev/mech-crate");
        let roots: HashSet<PathBuf> = [repo.clone()].into_iter().collect();
        let exe = repo.join("target/release/mx");
        match detect(&exe, &home(), None, |p| roots.contains(p)) {
            InstallKind::Source { repo: r } => assert_eq!(r, repo),
            other => panic!("expected Source, got {other:?}"),
        }
    }

    #[test]
    fn an_exe_under_a_checkouts_bin_dir_is_a_source_install() {
        let repo = PathBuf::from("/Users/me/dev/mech-crate");
        let roots: HashSet<PathBuf> = [repo.clone()].into_iter().collect();
        let exe = repo.join("bin/mx");
        assert!(matches!(
            detect(&exe, &home(), None, |p| roots.contains(p)),
            InstallKind::Source { .. }
        ));
    }

    #[test]
    fn a_release_install_wins_over_a_repo_root_further_up() {
        // ~/.mech-crate itself being (bizarrely) a repo root must not hijack
        // a release install nested under it.
        let roots: HashSet<PathBuf> = [home()].into_iter().collect();
        let exe = home().join("releases/mx-v0.1.2/bin/mx");
        assert!(matches!(
            detect(&exe, &home(), None, |p| roots.contains(p)),
            InstallKind::Release { .. }
        ));
    }

    #[test]
    fn anything_else_is_bare() {
        let exe = PathBuf::from("/usr/local/bin/mx");
        match detect(&exe, &home(), Some(Path::new("/opt/homebrew")), no_repo) {
            InstallKind::Bare { exe: e } => assert_eq!(e, PathBuf::from("/usr/local/bin/mx")),
            other => panic!("expected Bare, got {other:?}"),
        }
    }

    #[test]
    fn kind_names_are_stable_for_output() {
        assert_eq!(
            detect(
                &home().join("releases/mx-v0.1.2/bin/mx"),
                &home(),
                None,
                no_repo
            )
            .name(),
            "release"
        );
        assert_eq!(
            detect(Path::new("/usr/local/bin/mx"), &home(), None, no_repo).name(),
            "bare"
        );
    }
}
