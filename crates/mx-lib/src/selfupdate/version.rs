//! Version parsing and comparison for self-update (pure).

pub use semver::Version;

use crate::error::{Error, Result};

/// Parse a version string as emitted by git tags (`v0.1.2`), `VERSION` files
/// (`0.1.2`) and GitHub release `tag_name`s. Surrounding whitespace and a
/// leading `v` are tolerated.
pub fn parse(s: &str) -> Result<Version> {
    let trimmed = s.trim();
    let bare = trimmed.strip_prefix('v').unwrap_or(trimmed);
    Version::parse(bare).map_err(|e| Error::Other(format!("invalid version '{trimmed}': {e}")))
}

/// True when `candidate` is strictly newer than `current` under semver
/// ordering (a pre-release sorts below its release).
pub fn is_newer(candidate: &Version, current: &Version) -> bool {
    candidate > current
}

/// The version of the running build (the workspace version).
pub fn current() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("CARGO_PKG_VERSION is valid semver")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_a_leading_v() {
        assert_eq!(parse("v0.1.2").unwrap(), Version::new(0, 1, 2));
    }

    #[test]
    fn parse_accepts_a_bare_semver() {
        assert_eq!(parse("0.1.2").unwrap(), Version::new(0, 1, 2));
    }

    #[test]
    fn parse_trims_surrounding_whitespace() {
        assert_eq!(parse(" v0.1.2\n").unwrap(), Version::new(0, 1, 2));
    }

    #[test]
    fn parse_rejects_garbage() {
        assert!(parse("latest").is_err());
        assert!(parse("").is_err());
    }

    #[test]
    fn a_higher_patch_is_newer() {
        assert!(is_newer(&parse("0.1.2").unwrap(), &parse("0.1.1").unwrap()));
    }

    #[test]
    fn the_same_version_is_not_newer() {
        assert!(!is_newer(
            &parse("0.1.1").unwrap(),
            &parse("0.1.1").unwrap()
        ));
    }

    #[test]
    fn a_pre_release_is_older_than_its_release() {
        let rc = parse("0.1.2-rc.1").unwrap();
        let rel = parse("0.1.2").unwrap();
        assert!(!is_newer(&rc, &rel));
        assert!(is_newer(&rel, &rc));
    }

    #[test]
    fn current_is_the_crate_version() {
        assert_eq!(current(), parse(env!("CARGO_PKG_VERSION")).unwrap());
    }
}
