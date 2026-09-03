//! Release target selection and asset naming (pure).
//!
//! The names here are a contract with `scripts/package.sh` and
//! `.github/workflows/release.yml`: `mx-v<version>-<triple>.tar.gz` plus a
//! `.sha256` sibling, extracting to `mx-v<version>/`.

use semver::Version;

use crate::error::{Error, Result};

/// The build targets the release pipeline publishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Triple {
    /// One fat Mach-O for both Apple architectures.
    UniversalAppleDarwin,
    /// Statically linked Linux, x86_64.
    X86_64UnknownLinuxMusl,
    /// Statically linked Linux, aarch64.
    Aarch64UnknownLinuxMusl,
}

impl Triple {
    /// The triple string exactly as `package.sh` spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Triple::UniversalAppleDarwin => "universal-apple-darwin",
            Triple::X86_64UnknownLinuxMusl => "x86_64-unknown-linux-musl",
            Triple::Aarch64UnknownLinuxMusl => "aarch64-unknown-linux-musl",
        }
    }

    /// Map an OS/arch pair (in `std::env::consts` spelling) to a published
    /// triple, or `None` when the pipeline ships nothing for it.
    pub fn for_platform(os: &str, arch: &str) -> Option<Triple> {
        match (os, arch) {
            ("macos", _) => Some(Triple::UniversalAppleDarwin),
            ("linux", "x86_64") => Some(Triple::X86_64UnknownLinuxMusl),
            ("linux", "aarch64") => Some(Triple::Aarch64UnknownLinuxMusl),
            _ => None,
        }
    }

    /// The triple for the machine this binary is running on.
    pub fn host() -> Result<Triple> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        Triple::for_platform(os, arch).ok_or_else(|| {
            Error::Other(format!(
                "no mx release is published for {os}/{arch}; build from source instead"
            ))
        })
    }
}

/// Directory name at the root of a release tarball, e.g. `mx-v0.1.2`.
pub fn bundle_dir_name(version: &Version) -> String {
    format!("mx-v{version}")
}

/// Tarball asset name, e.g. `mx-v0.1.2-universal-apple-darwin.tar.gz`.
pub fn asset_name(version: &Version, triple: Triple) -> String {
    format!("{}-{}.tar.gz", bundle_dir_name(version), triple.as_str())
}

/// Checksum asset name: the tarball name plus `.sha256`.
pub fn checksum_name(version: &Version, triple: Triple) -> String {
    format!("{}.sha256", asset_name(version, triple))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selfupdate::version::parse;

    #[test]
    fn macos_maps_to_the_universal_triple_on_any_arch() {
        assert_eq!(
            Triple::for_platform("macos", "aarch64"),
            Some(Triple::UniversalAppleDarwin)
        );
        assert_eq!(
            Triple::for_platform("macos", "x86_64"),
            Some(Triple::UniversalAppleDarwin)
        );
    }

    #[test]
    fn linux_maps_to_the_musl_triple_for_its_arch() {
        assert_eq!(
            Triple::for_platform("linux", "x86_64"),
            Some(Triple::X86_64UnknownLinuxMusl)
        );
        assert_eq!(
            Triple::for_platform("linux", "aarch64"),
            Some(Triple::Aarch64UnknownLinuxMusl)
        );
    }

    #[test]
    fn unsupported_platforms_have_no_triple() {
        assert_eq!(Triple::for_platform("windows", "x86_64"), None);
        assert_eq!(Triple::for_platform("linux", "riscv64"), None);
    }

    #[test]
    fn host_matches_this_build() {
        let host = Triple::host().expect("this test only runs on supported hosts");
        if cfg!(target_os = "macos") {
            assert_eq!(host, Triple::UniversalAppleDarwin);
        } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            assert_eq!(host, Triple::X86_64UnknownLinuxMusl);
        }
    }

    #[test]
    fn triple_strings_match_package_sh() {
        assert_eq!(
            Triple::UniversalAppleDarwin.as_str(),
            "universal-apple-darwin"
        );
        assert_eq!(
            Triple::X86_64UnknownLinuxMusl.as_str(),
            "x86_64-unknown-linux-musl"
        );
        assert_eq!(
            Triple::Aarch64UnknownLinuxMusl.as_str(),
            "aarch64-unknown-linux-musl"
        );
    }

    #[test]
    fn asset_and_checksum_names_match_package_sh() {
        let v = parse("0.1.2").unwrap();
        assert_eq!(
            asset_name(&v, Triple::UniversalAppleDarwin),
            "mx-v0.1.2-universal-apple-darwin.tar.gz"
        );
        assert_eq!(
            checksum_name(&v, Triple::UniversalAppleDarwin),
            "mx-v0.1.2-universal-apple-darwin.tar.gz.sha256"
        );
    }

    #[test]
    fn bundle_dir_name_is_the_tarball_root() {
        assert_eq!(bundle_dir_name(&parse("0.1.2").unwrap()), "mx-v0.1.2");
    }
}
