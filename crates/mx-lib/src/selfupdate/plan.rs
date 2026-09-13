//! Update plan derivation (pure).
//!
//! Given what kind of install this is and which versions exist, decide what
//! `mx self-update` should do. No IO: the CLI resolves the inputs, prints
//! the plan for `--dry-run`, and executes it.

use std::path::PathBuf;

use semver::Version;

use crate::selfupdate::kind::InstallKind;
use crate::selfupdate::target::{asset_name, checksum_name, Triple};
use crate::selfupdate::version::is_newer;

/// The command run for Homebrew installs.
pub const BREW_UPGRADE: &str = "brew upgrade mx";

/// What an update would do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdatePlan {
    /// Nothing to do.
    UpToDate { current: Version },
    /// Fetch `asset` (verified against `checksum`), install it as
    /// `version`, and — for bare installs — re-point the old executable
    /// path at the new layout.
    Download {
        version: Version,
        asset: String,
        checksum: String,
        repoint: Option<PathBuf>,
    },
    /// Homebrew owns the install; run its upgrade instead.
    DelegateBrew { command: &'static str },
    /// A checkout owns the install; rebuild it in place.
    RebuildSource { repo: PathBuf },
}

/// Decide the plan.
///
/// - Homebrew and source installs ignore versions and pins: the package
///   manager or the checkout decides what "latest" means for them.
/// - Release and bare installs update only to a strictly newer `latest`,
///   never downgrading on their own; an explicit `pin` is honored in either
///   direction unless it equals `current`.
pub fn plan(
    kind: &InstallKind,
    current: &Version,
    latest: &Version,
    pin: Option<&Version>,
    triple: Triple,
) -> UpdatePlan {
    match kind {
        InstallKind::Homebrew { .. } => UpdatePlan::DelegateBrew {
            command: BREW_UPGRADE,
        },
        InstallKind::Source { repo } => UpdatePlan::RebuildSource { repo: repo.clone() },
        InstallKind::Release { .. } | InstallKind::Bare { .. } => {
            let (target, wanted) = match pin {
                Some(pinned) => (pinned, pinned != current),
                None => (latest, is_newer(latest, current)),
            };
            if !wanted {
                return UpdatePlan::UpToDate {
                    current: current.clone(),
                };
            }
            let repoint = match kind {
                InstallKind::Bare { exe } => Some(exe.clone()),
                _ => None,
            };
            UpdatePlan::Download {
                version: target.clone(),
                asset: asset_name(target, triple),
                checksum: checksum_name(target, triple),
                repoint,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selfupdate::version::parse;

    const T: Triple = Triple::UniversalAppleDarwin;

    fn v(s: &str) -> semver::Version {
        parse(s).unwrap()
    }

    fn release() -> InstallKind {
        InstallKind::Release {
            home: PathBuf::from("/Users/me/.mech-crate"),
            version: v("0.1.1"),
        }
    }

    #[test]
    fn a_release_install_on_the_latest_version_is_up_to_date() {
        assert_eq!(
            plan(&release(), &v("0.1.1"), &v("0.1.1"), None, T),
            UpdatePlan::UpToDate {
                current: v("0.1.1")
            }
        );
    }

    #[test]
    fn a_release_install_downloads_the_newer_latest() {
        assert_eq!(
            plan(&release(), &v("0.1.1"), &v("0.1.2"), None, T),
            UpdatePlan::Download {
                version: v("0.1.2"),
                asset: "mx-v0.1.2-universal-apple-darwin.tar.gz".into(),
                checksum: "mx-v0.1.2-universal-apple-darwin.tar.gz.sha256".into(),
                repoint: None,
            }
        );
    }

    #[test]
    fn a_release_install_never_auto_downgrades_when_latest_is_older() {
        // e.g. a locally built 0.2.0-dev with 0.1.9 published
        assert_eq!(
            plan(&release(), &v("0.2.0"), &v("0.1.9"), None, T),
            UpdatePlan::UpToDate {
                current: v("0.2.0")
            }
        );
    }

    #[test]
    fn a_pin_to_an_older_version_is_an_explicit_downgrade() {
        match plan(&release(), &v("0.1.2"), &v("0.1.2"), Some(&v("0.1.1")), T) {
            UpdatePlan::Download {
                version, repoint, ..
            } => {
                assert_eq!(version, v("0.1.1"));
                assert_eq!(repoint, None);
            }
            other => panic!("expected Download, got {other:?}"),
        }
    }

    #[test]
    fn a_pin_to_the_current_version_is_up_to_date() {
        assert_eq!(
            plan(&release(), &v("0.1.2"), &v("0.1.3"), Some(&v("0.1.2")), T),
            UpdatePlan::UpToDate {
                current: v("0.1.2")
            }
        );
    }

    #[test]
    fn a_bare_install_downloads_and_repoints_its_old_path() {
        let bare = InstallKind::Bare {
            exe: PathBuf::from("/usr/local/bin/mx"),
        };
        match plan(&bare, &v("0.1.1"), &v("0.1.2"), None, T) {
            UpdatePlan::Download { repoint, .. } => {
                assert_eq!(repoint, Some(PathBuf::from("/usr/local/bin/mx")));
            }
            other => panic!("expected Download, got {other:?}"),
        }
    }

    #[test]
    fn a_homebrew_install_always_delegates_to_brew() {
        let brew = InstallKind::Homebrew {
            cellar: PathBuf::from("/opt/homebrew/Cellar/mx/0.1.1"),
        };
        let expected = UpdatePlan::DelegateBrew {
            command: "brew upgrade mx",
        };
        assert_eq!(plan(&brew, &v("0.1.1"), &v("0.1.2"), None, T), expected);
        assert_eq!(plan(&brew, &v("0.1.1"), &v("0.1.1"), None, T), expected);
        assert_eq!(
            plan(&brew, &v("0.1.1"), &v("0.1.2"), Some(&v("0.1.0")), T),
            expected
        );
    }

    #[test]
    fn a_source_install_always_rebuilds_from_its_checkout() {
        let repo = PathBuf::from("/Users/me/dev/mech-crate");
        let src = InstallKind::Source { repo: repo.clone() };
        let expected = UpdatePlan::RebuildSource { repo };
        assert_eq!(plan(&src, &v("0.1.1"), &v("0.1.2"), None, T), expected);
        assert_eq!(plan(&src, &v("0.1.1"), &v("0.1.1"), None, T), expected);
    }
}
