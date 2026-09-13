//! Fake release bundles for self-update tests.
//!
//! [`write_fake_bundle`] lays out a directory shaped exactly like an
//! extracted release tarball (`mx-v<ver>/{bin,templates,scripts,VERSION}`)
//! whose `bin/mx` is a shell script that answers `--version`, and
//! [`pack_bundle`] turns one into the `.tar.gz` + sha256 pair the release
//! pipeline publishes, so the download path can be exercised against
//! wiremock without a real build.

use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// Write `mx-v<version>/` under `parent` and return its path.
///
/// `bin/mx` and `bin/mx-mcp` are executable `sh` scripts
/// printing `<name> <version>`; `templates/marker-<version>.txt` lets a test
/// prove which bundle's templates were copied; `scripts/` exists so
/// `mech_crate_root()` recognises the directory.
pub fn write_fake_bundle(parent: &Path, version: &str) -> PathBuf {
    let root = parent.join(format!("mx-v{version}"));
    fs::create_dir_all(root.join("bin/lib")).unwrap();
    fs::create_dir_all(root.join("templates")).unwrap();
    fs::create_dir_all(root.join("scripts")).unwrap();

    for name in ["mx", "mx-mcp"] {
        let path = root.join("bin").join(name);
        fs::write(&path, format!("#!/bin/sh\necho \"{name} {version}\"\n")).unwrap();
        set_executable(&path);
    }
    fs::write(root.join("bin/lib/.keep"), "").unwrap();
    fs::write(
        root.join("templates").join(format!("marker-{version}.txt")),
        version,
    )
    .unwrap();
    fs::write(root.join("scripts/.keep"), "").unwrap();
    fs::write(root.join("VERSION"), format!("{version}\n")).unwrap();
    root
}

/// Pack `bundle_dir` (a `mx-v<ver>` directory) as `<out_dir>/<file_name>`
/// and return the tarball path with its lowercase sha256 hex.
///
/// The archive's entries are rooted at the bundle's directory name, matching
/// `scripts/package.sh`.
pub fn pack_bundle(bundle_dir: &Path, out_dir: &Path, file_name: &str) -> (PathBuf, String) {
    fs::create_dir_all(out_dir).unwrap();
    let tar_path = out_dir.join(file_name);
    let file = fs::File::create(&tar_path).unwrap();
    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::fast());
    let mut archive = tar::Builder::new(encoder);
    archive.follow_symlinks(false);
    let root_name = bundle_dir.file_name().unwrap();
    archive.append_dir_all(root_name, bundle_dir).unwrap();
    archive.into_inner().unwrap().finish().unwrap();

    let bytes = fs::read(&tar_path).unwrap();
    let digest = hex::encode(Sha256::digest(&bytes));
    (tar_path, digest)
}

/// The `.sha256` sidecar body `shasum -a 256` would write for `file_name`.
pub fn sha256_sidecar(digest: &str, file_name: &str) -> String {
    format!("{digest}  {file_name}\n")
}

#[cfg(unix)]
fn set_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap();
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn fake_bundle_answers_version() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = write_fake_bundle(dir.path(), "9.9.9");
        let out = Command::new(bundle.join("bin/mx"))
            .arg("--version")
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "mx 9.9.9");
        assert_eq!(
            fs::read_to_string(bundle.join("VERSION")).unwrap().trim(),
            "9.9.9"
        );
        assert!(bundle.join("scripts").is_dir());
    }

    #[test]
    fn packed_bundle_round_trips_and_digest_matches() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = write_fake_bundle(dir.path(), "9.9.9");
        let (tar, digest) = pack_bundle(&bundle, &dir.path().join("out"), "b.tar.gz");
        assert_eq!(digest.len(), 64);

        let file = fs::File::open(&tar).unwrap();
        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(file));
        let names: Vec<String> = archive
            .entries()
            .unwrap()
            .map(|e| e.unwrap().path().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(names.iter().any(|n| n == "mx-v9.9.9/bin/mx"));
        assert!(names.iter().any(|n| n == "mx-v9.9.9/VERSION"));
        assert!(sha256_sidecar(&digest, "b.tar.gz").ends_with("  b.tar.gz\n"));
    }
}
