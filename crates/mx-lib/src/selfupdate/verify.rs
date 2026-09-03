//! Post-extract verification of a freshly installed release (spec §3.4).
//!
//! Two independent checks, both run against the *new* binary before
//! `current` is flipped: it reports the version we think we installed, and —
//! on macOS, when `codesign` is available — its signature still verifies.
//! Neither reads the process environment: the caller passes the PATH to
//! search, so the hermetic test suite stays hermetic.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use semver::Version;

use crate::error::{Error, Result};
use crate::selfupdate::version::parse;

/// Outcome of the codesign check. Absence of `codesign` is not a failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodesignStatus {
    /// `codesign --verify --strict` accepted the binary.
    Verified,
    /// The check did not run; the string says why.
    Skipped(String),
    /// `codesign` rejected the binary; the string is its complaint.
    Failed(String),
}

/// Run `<exe> --version` and require it to report `expected`.
///
/// The banner is `mx <version>` (clap's default for the crate), so the
/// leading word is dropped before parsing.
pub fn check_binary_version(exe: &Path, expected: &Version) -> Result<()> {
    let output = Command::new(exe).arg("--version").output().map_err(|e| {
        Error::SelfUpdate(format!("could not run {} --version: {e}", exe.display()))
    })?;
    if !output.status.success() {
        return Err(Error::SelfUpdate(format!(
            "{} --version exited with {}: {}",
            exe.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let banner = stdout.trim();
    let reported = banner
        .split_whitespace()
        .nth(1)
        .ok_or(())
        .and_then(|word| parse(word).map_err(|_| ()))
        .map_err(|()| {
            Error::SelfUpdate(format!(
                "{} --version printed '{banner}', expected 'mx <version>'",
                exe.display()
            ))
        })?;

    if &reported != expected {
        return Err(Error::SelfUpdate(format!(
            "{} reports version {reported}, expected {expected}",
            exe.display()
        )));
    }
    Ok(())
}

/// Verify the code signature of `exe` using `codesign` from `path_env`.
///
/// Returns [`CodesignStatus::Skipped`] when `codesign` is not on the PATH
/// that was passed in (Linux, or a trimmed-down macOS): an unsigned platform
/// is not an update failure. Callers pass `std::env::var("PATH")`.
pub fn check_codesign(exe: &Path, path_env: Option<&str>) -> CodesignStatus {
    let Some(codesign) = find_on_path("codesign", path_env) else {
        return CodesignStatus::Skipped("codesign is not on PATH".to_string());
    };

    let output = Command::new(codesign)
        .args(["--verify", "--strict", "--verbose=2"])
        .arg(exe)
        .output();

    match output {
        Err(e) => CodesignStatus::Skipped(format!("could not run codesign: {e}")),
        Ok(out) if out.status.success() => CodesignStatus::Verified,
        Ok(out) => {
            let complaint = String::from_utf8_lossy(&out.stderr).trim().to_string();
            CodesignStatus::Failed(if complaint.is_empty() {
                format!("codesign exited with {}", out.status)
            } else {
                complaint
            })
        }
    }
}

/// First executable named `name` in the colon-separated `path_env`.
fn find_on_path(name: &str, path_env: Option<&str>) -> Option<PathBuf> {
    path_env?
        .split(':')
        .filter(|entry| !entry.is_empty())
        .map(|dir| Path::new(dir).join(name))
        .find(|candidate| is_executable(candidate))
}

/// Whether `path` is a regular file with an execute bit set.
fn is_executable(path: &Path) -> bool {
    fs::metadata(path)
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selfupdate::version::parse;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    /// An executable shell script at `<tmp>/bin/mx` printing `line`.
    fn fake_exe(tmp: &tempfile::TempDir, line: &str) -> PathBuf {
        let exe = tmp.path().join("bin/mx");
        fs::create_dir_all(exe.parent().unwrap()).unwrap();
        fs::write(&exe, format!("#!/bin/sh\necho \"{line}\"\n")).unwrap();
        fs::set_permissions(&exe, fs::Permissions::from_mode(0o755)).unwrap();
        exe
    }

    #[test]
    fn check_binary_version_accepts_the_expected_version() {
        let tmp = tempfile::tempdir().unwrap();
        let exe = fake_exe(&tmp, "mx 9.9.9");

        check_binary_version(&exe, &parse("9.9.9").unwrap()).unwrap();
    }

    #[test]
    fn check_binary_version_rejects_a_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let exe = fake_exe(&tmp, "mx 9.9.9");

        let err = check_binary_version(&exe, &parse("1.2.3").unwrap()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("9.9.9") && msg.contains("1.2.3"), "got: {msg}");
    }

    #[test]
    fn check_binary_version_rejects_output_it_cannot_parse() {
        let tmp = tempfile::tempdir().unwrap();
        let exe = fake_exe(&tmp, "not a version banner at all");

        assert!(check_binary_version(&exe, &parse("9.9.9").unwrap()).is_err());
    }

    #[test]
    fn check_binary_version_errors_when_the_binary_cannot_run() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("bin/mx");

        let err = check_binary_version(&missing, &parse("9.9.9").unwrap()).unwrap_err();
        assert!(err.to_string().contains("mx"), "got: {err}");
    }

    #[test]
    fn check_codesign_is_skipped_when_codesign_is_not_on_path() {
        let tmp = tempfile::tempdir().unwrap();
        let exe = fake_exe(&tmp, "mx 9.9.9");

        assert!(matches!(
            check_codesign(&exe, Some("")),
            CodesignStatus::Skipped(_)
        ));
        assert!(matches!(
            check_codesign(&exe, None),
            CodesignStatus::Skipped(_)
        ));
    }
}
