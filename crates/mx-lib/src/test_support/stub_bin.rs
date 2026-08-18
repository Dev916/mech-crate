//! Stub-bin PATH fixture.
//!
//! Creates a temp directory of scripted fake executables and hands back a
//! `PATH` value with that directory FIRST, so code under test that shells out
//! to `docker`/`make`/`mx` hits the stub instead of the real tool. Each stub
//! appends its argv to a per-command log the test can read back.
//!
//! This is the adapter fake for every process-execution port in the crate:
//! no daemon, no network, no global state — each `StubBin` owns its own
//! tempdir, so tests stay independent even under nextest's process-per-test
//! model.

use std::io::Write;
use std::path::Path;

use tempfile::TempDir;

/// A temp directory of scripted fake executables, first on `PATH`.
///
/// The directory (and every stub in it) is removed when the `StubBin` drops,
/// so hold it for the lifetime of the test.
pub struct StubBin {
    dir: TempDir,
}

impl StubBin {
    /// Create an empty stub directory.
    pub fn new() -> Self {
        Self {
            dir: tempfile::tempdir().expect("create stub-bin tempdir"),
        }
    }

    /// The stub directory itself.
    pub fn dir(&self) -> &Path {
        self.dir.path()
    }

    /// Script a fake executable named `name` that records its argv, prints
    /// `stdout`, and exits with `exit_code`.
    ///
    /// Calling `stub` again for the same name replaces the script but keeps
    /// any invocations already recorded.
    pub fn stub(&self, name: &str, exit_code: i32, stdout: &str) -> &Self {
        let script_path = self.dir.path().join(name);
        let calls_path = self.calls_path(name);

        // The stub is /bin/sh: `echo "$@"` writes one space-joined argv line
        // per invocation. Both interpolated paths and the canned stdout are
        // single-quoted with embedded quotes escaped, so spaces and quotes in
        // either are inert to the shell.
        let mut script = String::from("#!/bin/sh\n");
        script.push_str(&format!(
            "echo \"$@\" >> {}\n",
            sh_quote(&calls_path.to_string_lossy())
        ));
        if !stdout.is_empty() {
            script.push_str(&format!("printf '%s\\n' {}\n", sh_quote(stdout)));
        }
        script.push_str(&format!("exit {}\n", exit_code));

        let mut f = std::fs::File::create(&script_path).expect("create stub script");
        f.write_all(script.as_bytes()).expect("write stub script");
        drop(f);
        make_executable(&script_path);

        self
    }

    /// `"<stubdir>:<original PATH>"` — assign to the child's `PATH` so the
    /// stubs win over anything installed on the machine.
    pub fn path_env(&self) -> String {
        format!(
            "{}:{}",
            self.dir.path().display(),
            std::env::var("PATH").unwrap_or_default()
        )
    }

    /// Recorded argv lines for `name`, oldest first. Empty when the stub was
    /// never invoked.
    pub fn invocations(&self, name: &str) -> Vec<String> {
        match std::fs::read_to_string(self.calls_path(name)) {
            Ok(s) => s.lines().map(|l| l.to_string()).collect(),
            Err(_) => Vec::new(),
        }
    }

    fn calls_path(&self, name: &str) -> std::path::PathBuf {
        self.dir.path().join(format!(".calls-{}", name))
    }
}

impl Default for StubBin {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrap `s` in single quotes, escaping any embedded single quote.
fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .expect("chmod stub script");
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {
    // Non-unix hosts are out of scope for the test suite (see the spec's
    // "Out of scope: Windows/other-OS CI matrices").
}
