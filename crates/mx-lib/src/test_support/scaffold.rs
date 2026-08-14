//! Tempdir project scaffolder.
//!
//! Builds the minimal on-disk skeleton that [`crate::project::ProjectDetector::strict`]
//! accepts, so CLI/upgrade/doctor tests get a real project root without
//! running `mx new` (which copies the whole template tree and needs an
//! initialized `~/.mech-crate`).

use std::path::Path;

/// Directories every mx project carries — mirrors
/// [`crate::upgrade::UpgradeEngine::required_directories`].
const REQUIRED_DIRS: &[&str] = &[
    "make",
    "scripts",
    "docker/.config",
    "docker/compose",
    "docker/system",
    "docker/dockerfiles",
    "tmp/up",
];

/// Scaffold a valid mx project skeleton at `root`.
///
/// Creates `Makefile` (with a single `help:` target), the four `docker/`
/// subdirectories, `make/`, `scripts/` and `tmp/up`. `root` itself is created
/// if missing. Panics on I/O failure — this is fixture code, and a fixture
/// that half-built a project would only produce a confusing downstream
/// assertion failure.
pub fn scaffold_project(root: &Path) {
    for dir in REQUIRED_DIRS {
        std::fs::create_dir_all(root.join(dir))
            .unwrap_or_else(|e| panic!("scaffold {}: {}", dir, e));
    }

    std::fs::write(
        root.join("Makefile"),
        "help:\n\t@echo \"scaffolded test project\"\n",
    )
    .unwrap_or_else(|e| panic!("scaffold Makefile: {}", e));
}
