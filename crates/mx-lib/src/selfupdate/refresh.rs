//! Post-update refresh of everything outside `releases/` (spec §3.4).
//!
//! Each step is idempotent and safe to re-run: they read from
//! `<home>/current` and rewrite derived state (`templates/`, `version`, the
//! MCP wrapper) to match it. `mx init` shares [`copy_dir_recursive`] so the
//! recursive copy exists in exactly one place.

use std::fs;
use std::io;
use std::path::Path;

use walkdir::WalkDir;

use crate::error::{Error, Result};
use crate::mcp::{write_wrapper, WRAPPER_NAME};

/// Copy the tree at `from` into `to`, creating `to` and its parents.
///
/// `on_file` is called with the *source* path of every regular file copied
/// (callers use it to tick a progress bar). Symlinks are skipped; the
/// executable bit is preserved. Returns the number of files copied.
pub fn copy_dir_recursive(from: &Path, to: &Path, on_file: &mut dyn FnMut(&Path)) -> Result<usize> {
    let mut copied = 0usize;
    for entry in WalkDir::new(from) {
        let entry = entry.map_err(walk_error)?;
        let relative = entry.path().strip_prefix(from).map_err(|e| {
            Error::Other(format!(
                "{} is not under {}: {e}",
                entry.path().display(),
                from.display()
            ))
        })?;
        let dest = to.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &dest)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = fs::metadata(entry.path())?.permissions().mode();
                if mode & 0o111 != 0 {
                    fs::set_permissions(&dest, fs::Permissions::from_mode(mode))?;
                }
            }

            on_file(entry.path());
            copied += 1;
        }
    }
    Ok(copied)
}

/// Replace `<home>/templates` with a copy of `<home>/current/templates`.
///
/// The copy is built in `templates.new` and swapped in with renames, so an
/// interrupted refresh never leaves an empty or half-populated `templates/`.
/// Returns the number of files copied.
pub fn refresh_templates(home: &Path) -> Result<usize> {
    let source = home.join("current").join("templates");
    if !source.is_dir() {
        return Err(Error::NotFound(format!(
            "templates directory {}",
            source.display()
        )));
    }

    let staging = home.join("templates.new");
    let retired = home.join("templates.old");
    let live = home.join("templates");

    remove_dir_if_present(&staging)?;
    remove_dir_if_present(&retired)?;
    let copied = copy_dir_recursive(&source, &staging, &mut |_| {})?;

    if fs::symlink_metadata(&live).is_ok() {
        fs::rename(&live, &retired)?;
    }
    fs::rename(&staging, &live)?;
    remove_dir_if_present(&retired)?;
    Ok(copied)
}

/// Mirror `<home>/current/VERSION` into `<home>/version`, trimmed.
pub fn write_version_file(home: &Path) -> Result<()> {
    let source = home.join("current").join("VERSION");
    let version = fs::read_to_string(&source)
        .map_err(|e| Error::NotFound(format!("{} ({e})", source.display())))?;
    fs::write(home.join("version"), version.trim())?;
    Ok(())
}

/// Repoint an existing MCP wrapper script at the release layout.
///
/// Rewrites `<home>/mcp/mx-mcp-wrapper.sh` to export
/// `MECH_CRATE_ROOT=<home>/current` and exec `<home>/current/bin/mx-mcp`, so
/// MCP clients keep working across updates without editing their config.
/// Does nothing (and returns `false`) when no wrapper exists — this never
/// configures MCP for someone who has not asked for it.
pub fn regenerate_mcp_wrapper(home: &Path) -> Result<bool> {
    let wrapper = home.join("mcp").join(WRAPPER_NAME);
    if !wrapper.exists() {
        return Ok(false);
    }
    let root = home.join("current");
    write_wrapper(&wrapper, &root, &root.join("bin").join("mx-mcp"))?;
    Ok(true)
}

/// `remove_dir_all` that tolerates the directory not being there.
fn remove_dir_if_present(dir: &Path) -> Result<()> {
    match fs::remove_dir_all(dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::Io(e)),
    }
}

/// Turn a walk failure into the IO error underneath it where there is one.
fn walk_error(e: walkdir::Error) -> Error {
    let path = e.path().map(|p| p.display().to_string());
    match e.into_io_error() {
        Some(io) => Error::Io(io),
        None => Error::Other(format!("could not walk {}", path.unwrap_or_default())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// A home whose `current` symlink resolves to a release with `templates/`.
    fn home_with_current() -> tempfile::TempDir {
        let home = tempfile::tempdir().unwrap();
        let release = home.path().join("releases/mx-v9.9.9");
        fs::create_dir_all(release.join("templates/sub")).unwrap();
        fs::write(release.join("templates/a.txt"), "a\n").unwrap();
        fs::write(release.join("templates/sub/b.txt"), "b\n").unwrap();
        fs::write(release.join("VERSION"), "9.9.9\n").unwrap();
        std::os::unix::fs::symlink("releases/mx-v9.9.9", home.path().join("current")).unwrap();
        home
    }

    #[test]
    fn copy_dir_recursive_copies_the_tree_and_reports_each_file() {
        let tmp = tempfile::tempdir().unwrap();
        let from = tmp.path().join("from");
        fs::create_dir_all(from.join("nested/deeper")).unwrap();
        fs::write(from.join("top.txt"), "top").unwrap();
        fs::write(from.join("nested/deeper/leaf.txt"), "leaf").unwrap();
        let to = tmp.path().join("to");

        let mut seen: Vec<String> = Vec::new();
        let copied = copy_dir_recursive(&from, &to, &mut |p: &Path| {
            seen.push(p.file_name().unwrap().to_string_lossy().into_owned())
        })
        .unwrap();

        assert_eq!(copied, 2);
        seen.sort();
        assert_eq!(seen, vec!["leaf.txt", "top.txt"]);
        assert_eq!(fs::read_to_string(to.join("top.txt")).unwrap(), "top");
        assert_eq!(
            fs::read_to_string(to.join("nested/deeper/leaf.txt")).unwrap(),
            "leaf"
        );
    }

    #[test]
    fn copy_dir_recursive_preserves_the_executable_bit() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let from = tmp.path().join("from");
        fs::create_dir_all(&from).unwrap();
        fs::write(from.join("run.sh"), "#!/bin/sh\n").unwrap();
        fs::set_permissions(from.join("run.sh"), fs::Permissions::from_mode(0o755)).unwrap();
        let to = tmp.path().join("to");

        copy_dir_recursive(&from, &to, &mut |_| {}).unwrap();

        let mode = fs::metadata(to.join("run.sh"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o111, 0o111, "executable bit must survive the copy");
    }

    #[test]
    fn refresh_templates_replaces_the_contents_and_leaves_no_scratch() {
        let home = home_with_current();
        fs::create_dir_all(home.path().join("templates")).unwrap();
        fs::write(home.path().join("templates/stale.txt"), "stale").unwrap();

        let copied = refresh_templates(home.path()).unwrap();

        assert_eq!(copied, 2);
        assert!(home.path().join("templates/a.txt").is_file());
        assert!(home.path().join("templates/sub/b.txt").is_file());
        assert!(
            !home.path().join("templates/stale.txt").exists(),
            "the old contents must be replaced, not merged"
        );
        assert!(!home.path().join("templates.new").exists());
        assert!(!home.path().join("templates.old").exists());
    }

    #[test]
    fn refresh_templates_errors_when_current_has_no_templates() {
        let home = tempfile::tempdir().unwrap();
        assert!(refresh_templates(home.path()).is_err());
    }

    #[test]
    fn write_version_file_writes_the_trimmed_version() {
        let home = home_with_current();

        write_version_file(home.path()).unwrap();

        assert_eq!(
            fs::read_to_string(home.path().join("version")).unwrap(),
            "9.9.9"
        );
    }

    #[test]
    fn regenerate_mcp_wrapper_rewrites_an_existing_wrapper() {
        use std::os::unix::fs::PermissionsExt;
        let home = home_with_current();
        let wrapper = home.path().join("mcp/mx-mcp-wrapper.sh");
        fs::create_dir_all(wrapper.parent().unwrap()).unwrap();
        fs::write(
            &wrapper,
            "#!/bin/bash\nexport MECH_CRATE_ROOT=\"/old/checkout\"\n",
        )
        .unwrap();

        assert!(regenerate_mcp_wrapper(home.path()).unwrap());

        let body = fs::read_to_string(&wrapper).unwrap();
        let root = home.path().join("current");
        assert!(
            body.contains(&format!("export MECH_CRATE_ROOT=\"{}\"", root.display())),
            "got: {body}"
        );
        assert!(
            body.contains(&format!("exec \"{}/bin/mx-mcp\" \"$@\"", root.display())),
            "got: {body}"
        );
        let mode = fs::metadata(&wrapper).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o755);
    }

    #[test]
    fn regenerate_mcp_wrapper_does_nothing_when_absent() {
        let home = home_with_current();

        assert!(!regenerate_mcp_wrapper(home.path()).unwrap());
        assert!(!home.path().join("mcp/mx-mcp-wrapper.sh").exists());
    }
}
