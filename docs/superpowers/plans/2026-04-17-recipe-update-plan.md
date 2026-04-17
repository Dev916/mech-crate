# Recipe Update Feature — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `mx update` command that pulls latest recipe updates into already-scaffolded apps using manifest-based 3-way merge.

**Architecture:** At scaffold time, `mx add` writes `.mx/manifest.json` (recipe name, git SHA, placeholders, per-file hashes + source) and `.mx/baselines/<path>` (original file content). At update time, `mx update` re-renders the latest recipe with stored placeholders, then runs a 3-way merge per file using baseline as common ancestor. Conflicts get git-style markers by default; an `--interactive` flag provides a per-file picker. User-added files are ignored; recipe deletions prompt; binary conflicts save as `<file>.new`.

**Tech Stack:** Rust (mx-lib + mx-cli crates), `diffy` crate for 3-way merge, `sha2` for hashing, `tera` for template interpolation (existing), `clap` for CLI (existing), `serde_json` for manifest (existing).

**Spec:** `docs/superpowers/specs/2026-04-17-recipe-update-design.md`

---

## Implementation decision (one deviation from spec)

The spec records `recipe_sha` as "the precise baseline for 3-way merge". In practice, retrieving the template tree at an arbitrary git SHA requires the templates directory to be a git repo at update time, which isn't guaranteed (`mx init` currently copies templates). To keep the MVP self-contained and robust, we store **baseline content in `.mx/baselines/<path>/`** alongside the manifest. `recipe_sha` is still recorded for audit/display. A future enhancement can switch to git-based retrieval when the install layout supports it.

Disk cost is negligible — a typical scaffold produces a few dozen small text files.

---

## File Structure

**New files:**
- `crates/mx-lib/src/recipe/manifest.rs` — Manifest types, JSON serialization, baseline read/write, sha256 helper.
- `crates/mx-lib/src/recipe/merge.rs` — 3-way merge wrapper (text via `diffy`, binary fallback).
- `crates/mx-lib/src/recipe/updater.rs` — Update planner (classify each file) and applier.
- `crates/mx-cli/src/commands/update.rs` — `mx update` CLI command.
- `crates/mx-lib/tests/recipe_update_integration.rs` — End-to-end integration test.

**Modified files:**
- `crates/mx-lib/Cargo.toml` — Add `diffy`, `sha2` deps.
- `crates/mx-lib/src/recipe/mod.rs` — Register new modules, re-export public types.
- `crates/mx-lib/src/recipe/installer.rs` — Write manifest + baselines after install.
- `crates/mx-lib/src/error.rs` — Add `ManifestNotFound`, `PlaceholderMissing` variants.
- `crates/mx-cli/src/commands/mod.rs` — Register `update` module.
- `crates/mx-cli/src/main.rs` — Wire `Update` subcommand.

---

## Task 1: Add dependencies and SHA256 helper

**Files:**
- Modify: `crates/mx-lib/Cargo.toml`
- Create: `crates/mx-lib/src/recipe/manifest.rs` (sha256 helper only in this task)

- [ ] **Step 1: Add deps to mx-lib/Cargo.toml**

Add under `[dependencies]`:

```toml
# 3-way merge
diffy = "0.4"

# SHA-256 hashing
sha2 = "0.10"

# Hex encoding
hex = "0.4"
```

- [ ] **Step 2: Write the failing test**

Create `crates/mx-lib/src/recipe/manifest.rs`:

```rust
//! Manifest for recipe-updateable projects.
//!
//! Stores the state of a scaffolded app so `mx update` can perform
//! 3-way merges against the latest recipe.

use sha2::{Digest, Sha256};

/// Compute SHA-256 hex digest of a byte slice.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hex_known_vector() {
        // SHA-256("abc") = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        let hash = sha256_hex(b"abc");
        assert_eq!(
            hash,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_sha256_hex_empty() {
        let hash = sha256_hex(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
```

Register the module. Edit `crates/mx-lib/src/recipe/mod.rs`:

```rust
//! Recipe management
//!
//! Handles parsing, installation, and caching of MechCrate recipes.

mod parser;
mod installer;
mod manifest;

pub use parser::{
    Recipe, RecipeOption, RecipeService, FileMapping, PostInstall, PostInstallAction,
    PlaceholderDef, InitApp, CreateFile, RenameAction, ChmodAction, RunAction,
};
pub use installer::{RecipeInstaller, InstallResult};
pub use manifest::sha256_hex;
```

- [ ] **Step 3: Run tests to verify they fail then pass**

Run: `cargo test -p mx-lib --lib recipe::manifest`
Expected: build succeeds (after `cargo fetch`), both tests PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/mx-lib/Cargo.toml crates/mx-lib/src/recipe/manifest.rs crates/mx-lib/src/recipe/mod.rs
git commit -m "feat(mx-lib): add diffy+sha2 deps and sha256 helper for recipe updates"
```

---

## Task 2: Manifest types and serialization

**Files:**
- Modify: `crates/mx-lib/src/recipe/manifest.rs`
- Modify: `crates/mx-lib/src/error.rs`

- [ ] **Step 1: Add error variant**

Edit `crates/mx-lib/src/error.rs`, add inside the `Error` enum before `Other`:

```rust
    #[error("Manifest not found at {0} — this app may predate the update feature; run `mx update init` to adopt it")]
    ManifestNotFound(String),

    #[error("Placeholder `{0}` is required by the updated recipe but not in the manifest. Re-run with `--set {0}=<value>`.")]
    PlaceholderMissing(String),
```

- [ ] **Step 2: Write the failing test**

Append to `crates/mx-lib/src/recipe/manifest.rs`:

```rust
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub const MANIFEST_FILENAME: &str = "manifest.json";
pub const MX_DIR: &str = ".mx";
pub const BASELINES_DIR: &str = "baselines";
pub const MANIFEST_VERSION: u32 = 1;

/// Source of a tracked file — either the main recipe or the `common` shared tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileSource {
    Recipe,
    Common,
}

/// Per-file entry in the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    /// SHA-256 of the file content as scaffolded (baseline).
    pub sha256: String,
    /// Which template tree produced this file.
    pub source: FileSource,
}

/// Manifest tracking a scaffolded recipe for later updates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Manifest schema version (for future migrations).
    pub manifest_version: u32,
    /// Recipe name as originally scaffolded.
    pub recipe: String,
    /// Git commit SHA of the templates tree at scaffold time (for audit).
    pub recipe_sha: Option<String>,
    /// ISO 8601 timestamp of scaffold.
    pub scaffolded_at: String,
    /// mx CLI version that performed the scaffold.
    pub mx_version: String,
    /// Placeholder values used during scaffolding (needed to re-render latest).
    pub placeholders: BTreeMap<String, String>,
    /// Map of relative file path → file entry.
    pub files: BTreeMap<String, FileEntry>,
}

impl Manifest {
    /// Directory where manifest and baselines live (`<project>/.mx/`).
    pub fn dir(project_root: &Path) -> PathBuf {
        project_root.join(MX_DIR)
    }

    /// Path to `<project>/.mx/manifest.json`.
    pub fn path(project_root: &Path) -> PathBuf {
        Self::dir(project_root).join(MANIFEST_FILENAME)
    }

    /// Path to `<project>/.mx/baselines/<relative_path>`.
    pub fn baseline_path(project_root: &Path, relative: &str) -> PathBuf {
        Self::dir(project_root).join(BASELINES_DIR).join(relative)
    }

    /// Load manifest from disk. Returns ManifestNotFound if missing.
    pub fn load(project_root: &Path) -> Result<Self> {
        let path = Self::path(project_root);
        if !path.exists() {
            return Err(Error::ManifestNotFound(path.display().to_string()));
        }
        let bytes = std::fs::read(&path)?;
        let manifest: Self = serde_json::from_slice(&bytes)?;
        Ok(manifest)
    }

    /// Save manifest to disk (creates `.mx/` if needed).
    pub fn save(&self, project_root: &Path) -> Result<()> {
        let dir = Self::dir(project_root);
        std::fs::create_dir_all(&dir)?;
        let path = Self::path(project_root);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Write a baseline file (creates parent dirs).
    pub fn write_baseline(project_root: &Path, relative: &str, content: &[u8]) -> Result<()> {
        let path = Self::baseline_path(project_root, relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Read a baseline file.
    pub fn read_baseline(project_root: &Path, relative: &str) -> Result<Vec<u8>> {
        let path = Self::baseline_path(project_root, relative);
        Ok(std::fs::read(path)?)
    }
}

#[cfg(test)]
mod manifest_tests {
    use super::*;
    use tempfile::TempDir;

    fn sample() -> Manifest {
        let mut files = BTreeMap::new();
        files.insert(
            "docker/compose/app.yml".to_string(),
            FileEntry { sha256: "abc".into(), source: FileSource::Recipe },
        );
        files.insert(
            "docker/compose/redis.yml".to_string(),
            FileEntry { sha256: "def".into(), source: FileSource::Common },
        );

        let mut placeholders = BTreeMap::new();
        placeholders.insert("APP_NAME".to_string(), "ai_share".to_string());

        Manifest {
            manifest_version: MANIFEST_VERSION,
            recipe: "rust-api".into(),
            recipe_sha: Some("a7c2f30abc123".into()),
            scaffolded_at: "2026-04-17T12:00:00Z".into(),
            mx_version: "0.1.0".into(),
            placeholders,
            files,
        }
    }

    #[test]
    fn test_roundtrip_save_load() {
        let temp = TempDir::new().unwrap();
        let manifest = sample();
        manifest.save(temp.path()).unwrap();
        let loaded = Manifest::load(temp.path()).unwrap();
        assert_eq!(loaded, manifest);
    }

    #[test]
    fn test_load_missing_returns_manifest_not_found() {
        let temp = TempDir::new().unwrap();
        match Manifest::load(temp.path()) {
            Err(Error::ManifestNotFound(_)) => {}
            other => panic!("expected ManifestNotFound, got {:?}", other),
        }
    }

    #[test]
    fn test_baseline_roundtrip() {
        let temp = TempDir::new().unwrap();
        Manifest::write_baseline(temp.path(), "src/main.rs", b"fn main() {}").unwrap();
        let read = Manifest::read_baseline(temp.path(), "src/main.rs").unwrap();
        assert_eq!(read, b"fn main() {}");
    }

    #[test]
    fn test_path_layout() {
        let root = Path::new("/tmp/foo");
        assert_eq!(Manifest::path(root), Path::new("/tmp/foo/.mx/manifest.json"));
        assert_eq!(
            Manifest::baseline_path(root, "a/b.txt"),
            Path::new("/tmp/foo/.mx/baselines/a/b.txt")
        );
    }
}
```

Export the new types. Edit `crates/mx-lib/src/recipe/mod.rs`:

```rust
pub use manifest::{
    sha256_hex, Manifest, FileEntry, FileSource,
    MANIFEST_FILENAME, MX_DIR, BASELINES_DIR, MANIFEST_VERSION,
};
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p mx-lib --lib recipe::manifest`
Expected: all 6 tests PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/mx-lib/src/recipe/manifest.rs crates/mx-lib/src/recipe/mod.rs crates/mx-lib/src/error.rs
git commit -m "feat(mx-lib): manifest types, load/save, and baseline storage"
```

---

## Task 3: Write manifest and baselines during installation

**Files:**
- Modify: `crates/mx-lib/src/recipe/installer.rs`

Track per-file source and hash during install; at end, write `.mx/manifest.json` + `.mx/baselines/<path>` for every file produced by the recipe.

- [ ] **Step 1: Write the failing test**

Append to the `tests` module in `crates/mx-lib/src/recipe/installer.rs`:

```rust
    #[test]
    fn test_install_writes_manifest_and_baselines() {
        use crate::recipe::{Manifest, FileSource};
        let temp = TempDir::new().unwrap();

        // Fake templates_root with a minimal recipe.
        let recipes = temp.path().join("recipes").join("sample");
        std::fs::create_dir_all(&recipes).unwrap();
        std::fs::write(
            recipes.join("recipe.json"),
            r#"{
                "name": "sample",
                "directories": ["apps/{{ APP_NAME }}"],
                "templates": [
                    { "from": "files/hello.txt", "to": "apps/{{ APP_NAME }}/hello.txt" }
                ]
            }"#,
        ).unwrap();
        std::fs::create_dir_all(recipes.join("files")).unwrap();
        std::fs::write(recipes.join("files/hello.txt"), "hello {{ APP_NAME }}!").unwrap();

        // Install into a separate project dir.
        let project = TempDir::new().unwrap();
        let mut installer = RecipeInstaller::new(temp.path()).unwrap();
        let recipe = installer.load_recipe("sample").unwrap();

        let mut opts = HashMap::new();
        opts.insert("APP_NAME".to_string(), "myapp".to_string());
        installer.install(&recipe, project.path(), "myapp", &opts).unwrap();

        // Manifest exists.
        let manifest = Manifest::load(project.path()).unwrap();
        assert_eq!(manifest.recipe, "sample");
        assert!(manifest.files.contains_key("apps/myapp/hello.txt"));
        assert_eq!(
            manifest.files["apps/myapp/hello.txt"].source,
            FileSource::Recipe
        );

        // Baseline exists and matches scaffolded content.
        let baseline = Manifest::read_baseline(project.path(), "apps/myapp/hello.txt").unwrap();
        assert_eq!(baseline, b"hello myapp!");

        // On-disk file matches baseline (fresh install = no drift).
        let on_disk = std::fs::read(project.path().join("apps/myapp/hello.txt")).unwrap();
        assert_eq!(on_disk, baseline);

        // Hash matches baseline.
        let expected_hash = crate::recipe::sha256_hex(&baseline);
        assert_eq!(manifest.files["apps/myapp/hello.txt"].sha256, expected_hash);

        // Placeholders saved.
        assert_eq!(manifest.placeholders.get("APP_NAME").map(|s| s.as_str()), Some("myapp"));
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p mx-lib --lib recipe::installer::tests::test_install_writes_manifest_and_baselines`
Expected: FAIL (manifest not written yet).

- [ ] **Step 3: Implement manifest writing in installer**

At the top of `crates/mx-lib/src/recipe/installer.rs` add imports:

```rust
use chrono::Utc;

use super::manifest::{FileEntry, FileSource, Manifest, MANIFEST_VERSION};
```

Add a helper to compute the git SHA of the templates subtree (best-effort; may return `None`). Add this method inside `impl RecipeInstaller`:

```rust
    /// Best-effort: get the git SHA of the templates directory's HEAD.
    /// Returns None if templates_root isn't inside a git repo.
    fn templates_git_sha(&self) -> Option<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.templates_root)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let sha = String::from_utf8(output.stdout).ok()?.trim().to_string();
        if sha.is_empty() { None } else { Some(sha) }
    }
```

Change `process_template` to track file source. Add a `source: FileSource` parameter:

```rust
    /// Process a single template mapping
    fn process_template(
        &mut self,
        recipe_dir: &Path,
        project_root: &Path,
        mapping: &FileMapping,
        placeholders: &HashMap<String, String>,
        result: &mut InstallResult,
        tracked: &mut Vec<(String, FileSource)>,
    ) -> Result<()> {
        let from_path = self.resolve_template_source(recipe_dir, &mapping.from)?;
        let to_template = self.interpolate(&mapping.to, placeholders)?;
        let to_path = project_root.join(&to_template);

        // Determine source: "common://" prefix → Common, else Recipe.
        let source = if mapping.from.starts_with("common://") {
            FileSource::Common
        } else {
            FileSource::Recipe
        };

        if from_path.is_dir() {
            self.copy_directory(&from_path, &to_path, placeholders, result, source.clone(), tracked)?;
        } else if from_path.is_file() {
            self.copy_file(&from_path, &to_path, placeholders)?;
            result.files_created.push(to_template.clone());
            tracked.push((to_template, source));
        } else {
            tracing::warn!("Template source not found: {}", from_path.display());
        }

        Ok(())
    }
```

Update `copy_directory` signature to accept `source` + `tracked`. Replace its body:

```rust
    fn copy_directory(
        &mut self,
        from: &Path,
        to: &Path,
        placeholders: &HashMap<String, String>,
        result: &mut InstallResult,
        source: FileSource,
        tracked: &mut Vec<(String, FileSource)>,
    ) -> Result<()> {
        std::fs::create_dir_all(to)?;

        for entry in walkdir::WalkDir::new(from) {
            let entry = entry.map_err(|e| Error::Io(e.into()))?;
            let relative = entry.path().strip_prefix(from).unwrap();

            let relative_str = relative.to_string_lossy();
            let interpolated_relative = self.interpolate(&relative_str, placeholders)?;
            let dest = to.join(&interpolated_relative);

            if entry.file_type().is_dir() {
                std::fs::create_dir_all(&dest)?;
            } else if entry.file_type().is_file() {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                self.copy_file(entry.path(), &dest, placeholders)?;
                let dest_rel = dest.to_string_lossy().to_string();
                result.files_created.push(dest_rel.clone());
                tracked.push((dest_rel, source.clone()));
            }
        }

        Ok(())
    }
```

Modify `install` to collect tracked files and write manifest at the end. Replace the body of `install`:

```rust
    pub fn install(
        &mut self,
        recipe: &Recipe,
        project_root: &Path,
        service_name: &str,
        option_values: &HashMap<String, String>,
    ) -> Result<InstallResult> {
        let mut result = InstallResult::default();
        let mut tracked: Vec<(String, FileSource)> = Vec::new();

        let placeholders = recipe.build_placeholders(service_name, option_values);

        for dir_template in &recipe.directories {
            let dir = self.interpolate(dir_template, &placeholders)?;
            let full_path = project_root.join(&dir);
            if !full_path.exists() {
                std::fs::create_dir_all(&full_path)?;
                result.directories_created.push(dir);
            }
        }

        if let Some(init_app) = &recipe.init_app {
            self.run_init_app(init_app, project_root, &placeholders)?;
        }

        let recipe_dir = self.recipe_dir(&recipe.name);
        for mapping in &recipe.templates {
            self.process_template(
                &recipe_dir,
                project_root,
                mapping,
                &placeholders,
                &mut result,
                &mut tracked,
            )?;
        }

        if let Some(post_install) = &recipe.post_install {
            self.run_post_install(post_install, project_root, &placeholders)?;
        }

        for step in &recipe.next_steps {
            let interpolated = self.interpolate(step, &placeholders)?;
            result.next_steps.push(interpolated);
        }

        // Write manifest + baselines.
        self.write_manifest(recipe, project_root, &placeholders, &tracked)?;

        Ok(result)
    }

    /// Write `.mx/manifest.json` + `.mx/baselines/<path>` for all tracked files.
    fn write_manifest(
        &self,
        recipe: &Recipe,
        project_root: &Path,
        placeholders: &HashMap<String, String>,
        tracked: &[(String, FileSource)],
    ) -> Result<()> {
        use std::collections::BTreeMap;

        let mut files = BTreeMap::new();
        for (rel, source) in tracked {
            let abs = project_root.join(rel);
            if !abs.is_file() {
                continue;
            }
            let bytes = std::fs::read(&abs)?;
            let sha = super::manifest::sha256_hex(&bytes);
            Manifest::write_baseline(project_root, rel, &bytes)?;
            files.insert(
                rel.clone(),
                FileEntry { sha256: sha, source: source.clone() },
            );
        }

        let mut placeholders_map = BTreeMap::new();
        for (k, v) in placeholders {
            placeholders_map.insert(k.clone(), v.clone());
        }

        let manifest = Manifest {
            manifest_version: MANIFEST_VERSION,
            recipe: recipe.name.clone(),
            recipe_sha: self.templates_git_sha(),
            scaffolded_at: Utc::now().to_rfc3339(),
            mx_version: env!("CARGO_PKG_VERSION").to_string(),
            placeholders: placeholders_map,
            files,
        };

        manifest.save(project_root)?;
        Ok(())
    }
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p mx-lib --lib recipe`
Expected: all recipe tests PASS (including new manifest test and existing interpolation test).

- [ ] **Step 5: Commit**

```bash
git add crates/mx-lib/src/recipe/installer.rs
git commit -m "feat(mx-lib): write manifest + baselines during recipe install"
```

---

## Task 4: 3-way merge wrapper with conflict detection

**Files:**
- Create: `crates/mx-lib/src/recipe/merge.rs`
- Modify: `crates/mx-lib/src/recipe/mod.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/mx-lib/src/recipe/merge.rs`:

```rust
//! 3-way merge for recipe updates.
//!
//! Wraps `diffy` for text files and provides a binary-safe fallback.

use crate::error::Result;

/// Result of a 3-way merge attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeResult {
    /// Final content to write. For text conflicts, contains `<<<<<<<`/`=======`/`>>>>>>>` markers.
    pub content: Vec<u8>,
    /// True if the merge had conflicts (for text) or was a binary-collision (can't auto-merge).
    pub has_conflict: bool,
    /// True if this is a binary file (merge was not attempted).
    pub is_binary: bool,
}

/// Heuristic: does the byte slice look like text? (No NUL bytes in first 8 KB.)
pub fn looks_like_text(bytes: &[u8]) -> bool {
    let sample = &bytes[..bytes.len().min(8192)];
    !sample.contains(&0)
}

/// 3-way merge. `base` = common ancestor, `ours` = local edits, `theirs` = incoming.
///
/// For text files: uses `diffy::merge` and preserves standard conflict markers.
/// For binary files: if ours==theirs or ours==base returns the winner; otherwise
/// returns `theirs` with `has_conflict=true` and the caller is expected to save
/// the original `ours` side-by-side as `<file>.new`.
pub fn three_way_merge(base: &[u8], ours: &[u8], theirs: &[u8]) -> Result<MergeResult> {
    let binary = !looks_like_text(base) || !looks_like_text(ours) || !looks_like_text(theirs);

    if binary {
        return Ok(merge_binary(base, ours, theirs));
    }

    // Safe to treat as UTF-8 text for diffy. If UTF-8 conversion fails, fall back to binary.
    let (Ok(base_s), Ok(ours_s), Ok(theirs_s)) = (
        std::str::from_utf8(base),
        std::str::from_utf8(ours),
        std::str::from_utf8(theirs),
    ) else {
        return Ok(merge_binary(base, ours, theirs));
    };

    match diffy::merge(base_s, ours_s, theirs_s) {
        Ok(merged) => Ok(MergeResult {
            content: merged.into_bytes(),
            has_conflict: false,
            is_binary: false,
        }),
        Err(merged_with_conflicts) => Ok(MergeResult {
            content: merged_with_conflicts.into_bytes(),
            has_conflict: true,
            is_binary: false,
        }),
    }
}

fn merge_binary(base: &[u8], ours: &[u8], theirs: &[u8]) -> MergeResult {
    if ours == theirs {
        return MergeResult { content: theirs.to_vec(), has_conflict: false, is_binary: true };
    }
    if ours == base {
        // User didn't touch → take theirs.
        return MergeResult { content: theirs.to_vec(), has_conflict: false, is_binary: true };
    }
    if theirs == base {
        // Recipe didn't change → keep ours.
        return MergeResult { content: ours.to_vec(), has_conflict: false, is_binary: true };
    }
    // Both diverged — can't merge binaries; caller saves theirs as `.new`.
    MergeResult { content: theirs.to_vec(), has_conflict: true, is_binary: true }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_no_changes() {
        let r = three_way_merge(b"hello\n", b"hello\n", b"hello\n").unwrap();
        assert!(!r.has_conflict);
        assert!(!r.is_binary);
        assert_eq!(r.content, b"hello\n");
    }

    #[test]
    fn test_text_user_only_change_kept() {
        let base = "a\nb\nc\n";
        let ours = "a\nB\nc\n"; // user changed line 2
        let theirs = "a\nb\nc\n";
        let r = three_way_merge(base.as_bytes(), ours.as_bytes(), theirs.as_bytes()).unwrap();
        assert!(!r.has_conflict);
        assert_eq!(r.content, b"a\nB\nc\n");
    }

    #[test]
    fn test_text_recipe_only_change_taken() {
        let base = "a\nb\nc\n";
        let ours = "a\nb\nc\n";
        let theirs = "a\nb\nC\n"; // recipe changed line 3
        let r = three_way_merge(base.as_bytes(), ours.as_bytes(), theirs.as_bytes()).unwrap();
        assert!(!r.has_conflict);
        assert_eq!(r.content, b"a\nb\nC\n");
    }

    #[test]
    fn test_text_non_overlapping_auto_merges() {
        let base = "a\nb\nc\nd\ne\n";
        let ours = "A\nb\nc\nd\ne\n"; // user changed line 1
        let theirs = "a\nb\nc\nd\nE\n"; // recipe changed line 5
        let r = three_way_merge(base.as_bytes(), ours.as_bytes(), theirs.as_bytes()).unwrap();
        assert!(!r.has_conflict);
        assert_eq!(r.content, b"A\nb\nc\nd\nE\n");
    }

    #[test]
    fn test_text_overlapping_conflict_has_markers() {
        let base = "line\n";
        let ours = "yours\n";
        let theirs = "recipe\n";
        let r = three_way_merge(base.as_bytes(), ours.as_bytes(), theirs.as_bytes()).unwrap();
        assert!(r.has_conflict);
        let s = String::from_utf8(r.content).unwrap();
        assert!(s.contains("<<<<<<<"));
        assert!(s.contains("======="));
        assert!(s.contains(">>>>>>>"));
    }

    #[test]
    fn test_binary_ours_equals_base_takes_theirs() {
        let base = &[0u8, 1, 2][..];
        let ours = &[0u8, 1, 2][..];
        let theirs = &[0u8, 1, 9][..];
        let r = three_way_merge(base, ours, theirs).unwrap();
        assert!(!r.has_conflict);
        assert!(r.is_binary);
        assert_eq!(r.content, theirs);
    }

    #[test]
    fn test_binary_both_diverged_is_conflict() {
        let base = &[0u8, 1, 2][..];
        let ours = &[0u8, 9, 2][..];
        let theirs = &[0u8, 1, 9][..];
        let r = three_way_merge(base, ours, theirs).unwrap();
        assert!(r.has_conflict);
        assert!(r.is_binary);
    }
}
```

Edit `crates/mx-lib/src/recipe/mod.rs`:

```rust
mod merge;

pub use merge::{three_way_merge, MergeResult, looks_like_text};
```

- [ ] **Step 2: Run test to verify failures, then pass**

Run: `cargo test -p mx-lib --lib recipe::merge`
Expected: all 7 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/mx-lib/src/recipe/merge.rs crates/mx-lib/src/recipe/mod.rs
git commit -m "feat(mx-lib): 3-way merge wrapper over diffy with binary fallback"
```

---

## Task 5: Update planner — classify changes per file

**Files:**
- Create: `crates/mx-lib/src/recipe/updater.rs`
- Modify: `crates/mx-lib/src/recipe/mod.rs`

Pure classification logic: given manifest + current disk state + re-rendered "latest" recipe output, produce a list of `PlannedAction`s. No filesystem writes.

- [ ] **Step 1: Write the failing test**

Create `crates/mx-lib/src/recipe/updater.rs`:

```rust
//! Recipe update planner and applier.

use std::collections::{BTreeMap, BTreeSet};

use super::manifest::{FileEntry, Manifest, FileSource};
use super::merge::{three_way_merge, MergeResult};
use super::{sha256_hex};
use crate::error::Result;

/// The action planned for a single file during an update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannedAction {
    /// No differences anywhere — skip.
    Unchanged,
    /// Recipe changed, user didn't. Apply recipe content.
    TakeTheirs { new_content: Vec<u8> },
    /// User changed, recipe didn't. Keep as-is (no-op on disk, but noted for summary).
    KeepOurs,
    /// Both changed; auto-merge succeeded.
    AutoMerged { new_content: Vec<u8> },
    /// Both changed; overlapping. Content has conflict markers.
    Conflict { new_content: Vec<u8>, is_binary: bool },
    /// Recipe added a new file not previously tracked.
    Added { new_content: Vec<u8>, source: FileSource },
    /// Recipe removed a tracked file. Decision (keep/delete) deferred to applier.
    RecipeRemoved { user_modified: bool },
    /// File was tracked but is missing from disk (user renamed/deleted).
    LocallyMissing,
}

/// Per-file classified entry in an update plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanEntry {
    pub path: String,
    pub action: PlannedAction,
}

/// Full update plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatePlan {
    pub entries: Vec<PlanEntry>,
}

/// Inputs for classification of a single file.
pub struct FileInputs<'a> {
    pub relative_path: &'a str,
    /// Previous manifest entry for this path (None if recipe just added it).
    pub manifest_entry: Option<&'a FileEntry>,
    /// Baseline content from `.mx/baselines/<path>` (None if not tracked previously).
    pub baseline: Option<&'a [u8]>,
    /// Current on-disk content (None if missing on disk).
    pub current: Option<&'a [u8]>,
    /// Content from the re-rendered latest recipe (None if recipe removed it).
    pub latest: Option<&'a [u8]>,
    /// Source of the latest content (Recipe or Common); for Added case.
    pub latest_source: Option<FileSource>,
}

/// Classify a single file.
pub fn classify(inputs: FileInputs<'_>) -> Result<PlannedAction> {
    match (inputs.baseline, inputs.current, inputs.latest) {
        // Recipe added a new file.
        (None, _, Some(latest)) => Ok(PlannedAction::Added {
            new_content: latest.to_vec(),
            source: inputs.latest_source.unwrap_or(FileSource::Recipe),
        }),
        // Was tracked; still tracked.
        (Some(base), Some(curr), Some(latest)) => {
            if base == latest && curr == base {
                return Ok(PlannedAction::Unchanged);
            }
            if base == latest && curr != base {
                return Ok(PlannedAction::KeepOurs);
            }
            if curr == base && latest != base {
                return Ok(PlannedAction::TakeTheirs { new_content: latest.to_vec() });
            }
            // Both sides diverged.
            let merged = three_way_merge(base, curr, latest)?;
            if merged.has_conflict {
                Ok(PlannedAction::Conflict {
                    new_content: merged.content,
                    is_binary: merged.is_binary,
                })
            } else {
                Ok(PlannedAction::AutoMerged { new_content: merged.content })
            }
        }
        // Was tracked; recipe removed.
        (Some(base), Some(curr), None) => Ok(PlannedAction::RecipeRemoved {
            user_modified: curr != base,
        }),
        // Was tracked; locally missing.
        (Some(_), None, _) => Ok(PlannedAction::LocallyMissing),
        // Untracked, nothing on either side — shouldn't happen, treat as unchanged.
        (None, _, None) => Ok(PlannedAction::Unchanged),
    }
}

/// Build the full update plan by iterating union(manifest paths, latest paths).
pub fn build_plan(
    manifest: &Manifest,
    project_root: &std::path::Path,
    latest: &BTreeMap<String, (Vec<u8>, FileSource)>,
) -> Result<UpdatePlan> {
    let mut paths: BTreeSet<&str> = BTreeSet::new();
    for k in manifest.files.keys() { paths.insert(k.as_str()); }
    for k in latest.keys() { paths.insert(k.as_str()); }

    let mut entries = Vec::new();
    for path in paths {
        let manifest_entry = manifest.files.get(path);
        let baseline = if manifest_entry.is_some() {
            Some(Manifest::read_baseline(project_root, path)?)
        } else { None };
        let on_disk = std::fs::read(project_root.join(path)).ok();
        let latest_entry = latest.get(path);

        let action = classify(FileInputs {
            relative_path: path,
            manifest_entry,
            baseline: baseline.as_deref(),
            current: on_disk.as_deref(),
            latest: latest_entry.map(|(b, _)| b.as_slice()),
            latest_source: latest_entry.map(|(_, s)| s.clone()),
        })?;

        // Skip Unchanged entries from the plan to keep summaries concise.
        if !matches!(action, PlannedAction::Unchanged) {
            entries.push(PlanEntry { path: path.to_string(), action });
        }
    }

    Ok(UpdatePlan { entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs<'a>(
        baseline: Option<&'a [u8]>,
        current: Option<&'a [u8]>,
        latest: Option<&'a [u8]>,
    ) -> FileInputs<'a> {
        FileInputs {
            relative_path: "f",
            manifest_entry: None,
            baseline,
            current,
            latest,
            latest_source: Some(FileSource::Recipe),
        }
    }

    #[test]
    fn test_classify_unchanged() {
        let r = classify(inputs(Some(b"x"), Some(b"x"), Some(b"x"))).unwrap();
        assert_eq!(r, PlannedAction::Unchanged);
    }

    #[test]
    fn test_classify_take_theirs() {
        let r = classify(inputs(Some(b"a"), Some(b"a"), Some(b"b"))).unwrap();
        assert_eq!(r, PlannedAction::TakeTheirs { new_content: b"b".to_vec() });
    }

    #[test]
    fn test_classify_keep_ours() {
        let r = classify(inputs(Some(b"a"), Some(b"a-mine"), Some(b"a"))).unwrap();
        assert_eq!(r, PlannedAction::KeepOurs);
    }

    #[test]
    fn test_classify_recipe_removed_user_untouched() {
        let r = classify(inputs(Some(b"x"), Some(b"x"), None)).unwrap();
        assert_eq!(r, PlannedAction::RecipeRemoved { user_modified: false });
    }

    #[test]
    fn test_classify_recipe_removed_user_modified() {
        let r = classify(inputs(Some(b"x"), Some(b"y"), None)).unwrap();
        assert_eq!(r, PlannedAction::RecipeRemoved { user_modified: true });
    }

    #[test]
    fn test_classify_locally_missing() {
        let r = classify(inputs(Some(b"x"), None, Some(b"x"))).unwrap();
        assert_eq!(r, PlannedAction::LocallyMissing);
    }

    #[test]
    fn test_classify_added() {
        let r = classify(inputs(None, None, Some(b"new"))).unwrap();
        assert_eq!(
            r,
            PlannedAction::Added { new_content: b"new".to_vec(), source: FileSource::Recipe }
        );
    }

    #[test]
    fn test_classify_auto_merge() {
        // Base, user edits line 1, recipe edits line 5 → auto merges.
        let base = b"1\n2\n3\n4\n5\n".to_vec();
        let ours = b"ONE\n2\n3\n4\n5\n".to_vec();
        let theirs = b"1\n2\n3\n4\nFIVE\n".to_vec();
        let r = classify(inputs(Some(&base), Some(&ours), Some(&theirs))).unwrap();
        match r {
            PlannedAction::AutoMerged { new_content } => {
                assert_eq!(new_content, b"ONE\n2\n3\n4\nFIVE\n".to_vec());
            }
            other => panic!("expected AutoMerged, got {:?}", other),
        }
    }

    #[test]
    fn test_classify_conflict() {
        let r = classify(inputs(Some(b"a\n"), Some(b"x\n"), Some(b"y\n"))).unwrap();
        match r {
            PlannedAction::Conflict { new_content, .. } => {
                let s = String::from_utf8(new_content).unwrap();
                assert!(s.contains("<<<<<<<"));
            }
            other => panic!("expected Conflict, got {:?}", other),
        }
    }
}
```

Edit `crates/mx-lib/src/recipe/mod.rs`:

```rust
mod updater;

pub use updater::{
    classify, build_plan, PlannedAction, PlanEntry, UpdatePlan, FileInputs,
};
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p mx-lib --lib recipe::updater`
Expected: all 9 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/mx-lib/src/recipe/updater.rs crates/mx-lib/src/recipe/mod.rs
git commit -m "feat(mx-lib): update planner — classify per-file changes"
```

---

## Task 6: Render latest recipe with stored placeholders

Load the latest recipe from templates, render each template file through Tera using manifest's stored placeholders, and return a map `<relative_path> → (bytes, FileSource)`. Fail fast if a required placeholder is missing.

**Files:**
- Modify: `crates/mx-lib/src/recipe/updater.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/mx-lib/src/recipe/updater.rs`:

```rust
use std::collections::HashMap;
use std::path::Path;

use crate::template::TemplateEngine;
use super::installer::RecipeInstaller;
use super::parser::{Recipe, FileMapping};
use crate::error::Error;

/// Render the latest recipe into an in-memory map of `<relative_path> → (content, source)`.
/// Uses `placeholders` from the manifest; errors if the recipe's template references an
/// unknown variable.
pub fn render_latest_recipe(
    templates_root: &Path,
    recipe_name: &str,
    placeholders: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, (Vec<u8>, FileSource)>> {
    let installer = RecipeInstaller::new(templates_root)?;
    let recipe = installer.load_recipe(recipe_name)?;

    // Convert placeholders to the HashMap shape the engine expects.
    let ph: HashMap<String, String> = placeholders.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

    // Merge in recipe-defined placeholders where possible (service name etc.) —
    // but only those the recipe doesn't explicitly override via required inputs.
    // Simpler: trust the manifest's placeholders; if recipe requires a new one we surface it.

    let mut engine = TemplateEngine::new()?;
    let recipe_dir = installer.recipe_dir(recipe_name);
    let mut out = BTreeMap::new();

    for mapping in &recipe.templates {
        render_mapping(
            &mut engine,
            templates_root,
            &recipe_dir,
            mapping,
            &ph,
            &mut out,
        )?;
    }

    Ok(out)
}

fn render_mapping(
    engine: &mut TemplateEngine,
    templates_root: &Path,
    recipe_dir: &Path,
    mapping: &FileMapping,
    placeholders: &HashMap<String, String>,
    out: &mut BTreeMap<String, (Vec<u8>, FileSource)>,
) -> Result<()> {
    // Resolve source (common:// or relative to recipe dir).
    let (from_path, source) = if let Some(rest) = mapping.from.strip_prefix("common://") {
        (
            templates_root.join("recipes").join("common").join(rest),
            FileSource::Common,
        )
    } else {
        (recipe_dir.join(&mapping.from), FileSource::Recipe)
    };

    let to_rel = engine
        .render_string(&mapping.to, placeholders)
        .map_err(map_tera_err)?;

    if from_path.is_file() {
        let content = render_one_file(engine, &from_path, placeholders)?;
        out.insert(to_rel, (content, source));
    } else if from_path.is_dir() {
        for entry in walkdir::WalkDir::new(&from_path) {
            let entry = entry.map_err(|e| Error::Io(e.into()))?;
            if !entry.file_type().is_file() { continue; }
            let rel = entry.path().strip_prefix(&from_path).unwrap();
            let rel_str = rel.to_string_lossy().to_string();
            let rendered_rel = engine
                .render_string(&rel_str, placeholders)
                .map_err(map_tera_err)?;
            let dest = format!("{}/{}", to_rel.trim_end_matches('/'), rendered_rel);
            let content = render_one_file(engine, entry.path(), placeholders)?;
            out.insert(dest, (content, source.clone()));
        }
    }
    Ok(())
}

fn render_one_file(
    engine: &mut TemplateEngine,
    path: &Path,
    placeholders: &HashMap<String, String>,
) -> Result<Vec<u8>> {
    if TemplateEngine::is_binary_file(path) {
        return Ok(std::fs::read(path)?);
    }
    let bytes = std::fs::read(path)?;
    match std::str::from_utf8(&bytes) {
        Ok(s) => Ok(engine.render_string(s, placeholders).map_err(map_tera_err)?.into_bytes()),
        Err(_) => Ok(bytes),
    }
}

fn map_tera_err(e: Error) -> Error {
    // Surface missing placeholder names (Tera includes variable name in message).
    let msg = e.to_string();
    if let Some(var) = extract_missing_var(&msg) {
        return Error::PlaceholderMissing(var);
    }
    e
}

fn extract_missing_var(msg: &str) -> Option<String> {
    // Tera: "Variable `FOO` not found in context..."
    let marker = "Variable `";
    let start = msg.find(marker)? + marker.len();
    let rest = &msg[start..];
    let end = rest.find('`')?;
    Some(rest[..end].to_string())
}

#[cfg(test)]
mod render_tests {
    use super::*;
    use tempfile::TempDir;

    fn write(path: &Path, content: &str) {
        if let Some(p) = path.parent() { std::fs::create_dir_all(p).unwrap(); }
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn test_render_latest_recipe_interpolates_placeholders() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write(
            &root.join("recipes/sample/recipe.json"),
            r#"{
                "name": "sample",
                "templates": [
                    { "from": "files/a.txt", "to": "out/{{ APP_NAME }}.txt" }
                ]
            }"#,
        );
        write(&root.join("recipes/sample/files/a.txt"), "name is {{ APP_NAME }}");

        let mut ph = BTreeMap::new();
        ph.insert("APP_NAME".to_string(), "hello".to_string());

        let rendered = render_latest_recipe(root, "sample", &ph).unwrap();
        assert_eq!(rendered.len(), 1);
        let (bytes, source) = rendered.get("out/hello.txt").unwrap();
        assert_eq!(bytes, b"name is hello");
        assert_eq!(*source, FileSource::Recipe);
    }

    #[test]
    fn test_render_latest_recipe_missing_placeholder_errors() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write(
            &root.join("recipes/sample/recipe.json"),
            r#"{
                "name": "sample",
                "templates": [
                    { "from": "files/a.txt", "to": "out.txt" }
                ]
            }"#,
        );
        write(&root.join("recipes/sample/files/a.txt"), "hello {{ NEW_VAR }}");

        let ph = BTreeMap::new(); // empty — NEW_VAR absent
        let err = render_latest_recipe(root, "sample", &ph).unwrap_err();
        match err {
            Error::PlaceholderMissing(v) => assert_eq!(v, "NEW_VAR"),
            other => panic!("expected PlaceholderMissing, got {:?}", other),
        }
    }

    #[test]
    fn test_render_latest_recipe_resolves_common_namespace() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write(&root.join("recipes/common/snippets/shared.yml"), "shared: {{ VAL }}");
        write(
            &root.join("recipes/sample/recipe.json"),
            r#"{
                "name": "sample",
                "templates": [
                    { "from": "common://snippets/shared.yml", "to": "out/shared.yml" }
                ]
            }"#,
        );

        let mut ph = BTreeMap::new();
        ph.insert("VAL".into(), "x".into());
        let rendered = render_latest_recipe(root, "sample", &ph).unwrap();
        let (bytes, source) = rendered.get("out/shared.yml").unwrap();
        assert_eq!(bytes, b"shared: x");
        assert_eq!(*source, FileSource::Common);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p mx-lib --lib recipe::updater::render_tests`
Expected: 3 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/mx-lib/src/recipe/updater.rs
git commit -m "feat(mx-lib): render latest recipe with stored placeholders for update"
```

---

## Task 7: Update summary printer

Given an `UpdatePlan`, produce a human-readable summary string (the `git status` equivalent for a recipe update).

**Files:**
- Modify: `crates/mx-lib/src/recipe/updater.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/mx-lib/src/recipe/updater.rs`:

```rust
/// Status letter for summary output (git-style).
pub fn action_letter(action: &PlannedAction) -> &'static str {
    match action {
        PlannedAction::Unchanged => " ",
        PlannedAction::TakeTheirs { .. } => "M",
        PlannedAction::KeepOurs => "K",
        PlannedAction::AutoMerged { .. } => "M",
        PlannedAction::Conflict { .. } => "C",
        PlannedAction::Added { .. } => "A",
        PlannedAction::RecipeRemoved { .. } => "D",
        PlannedAction::LocallyMissing => "?",
    }
}

/// Short human description of the action (fits in the summary line).
pub fn action_description(action: &PlannedAction) -> String {
    match action {
        PlannedAction::Unchanged => "no change".into(),
        PlannedAction::TakeTheirs { .. } => "recipe updated".into(),
        PlannedAction::KeepOurs => "kept local edits".into(),
        PlannedAction::AutoMerged { .. } => "auto-merged".into(),
        PlannedAction::Conflict { is_binary: true, .. } => "conflict (binary)".into(),
        PlannedAction::Conflict { .. } => "conflict".into(),
        PlannedAction::Added { .. } => "new".into(),
        PlannedAction::RecipeRemoved { user_modified: true } => "recipe removed (modified locally)".into(),
        PlannedAction::RecipeRemoved { user_modified: false } => "recipe removed".into(),
        PlannedAction::LocallyMissing => "missing on disk (skipped)".into(),
    }
}

/// Render a plan as a multi-line summary string.
pub fn render_summary(plan: &UpdatePlan) -> String {
    if plan.entries.is_empty() {
        return "Already up to date.\n".to_string();
    }
    let mut out = String::from("Changes:\n");
    for entry in &plan.entries {
        out.push_str(&format!(
            "  {}  {:<50} ({})\n",
            action_letter(&entry.action),
            entry.path,
            action_description(&entry.action),
        ));
    }
    out
}

#[cfg(test)]
mod summary_tests {
    use super::*;

    #[test]
    fn test_empty_plan_says_up_to_date() {
        let plan = UpdatePlan { entries: vec![] };
        assert_eq!(render_summary(&plan), "Already up to date.\n");
    }

    #[test]
    fn test_summary_shows_letters_and_paths() {
        let plan = UpdatePlan {
            entries: vec![
                PlanEntry {
                    path: "docker/compose/app.yml".into(),
                    action: PlannedAction::TakeTheirs { new_content: b"x".to_vec() },
                },
                PlanEntry {
                    path: "new.txt".into(),
                    action: PlannedAction::Added {
                        new_content: b"x".to_vec(),
                        source: FileSource::Recipe,
                    },
                },
                PlanEntry {
                    path: "old.txt".into(),
                    action: PlannedAction::RecipeRemoved { user_modified: false },
                },
                PlanEntry {
                    path: "cargo.toml".into(),
                    action: PlannedAction::Conflict {
                        new_content: b"x".to_vec(),
                        is_binary: false,
                    },
                },
            ],
        };
        let s = render_summary(&plan);
        assert!(s.contains("M  docker/compose/app.yml"));
        assert!(s.contains("A  new.txt"));
        assert!(s.contains("D  old.txt"));
        assert!(s.contains("C  cargo.toml"));
    }
}
```

Expose in `crates/mx-lib/src/recipe/mod.rs`:

```rust
pub use updater::{render_latest_recipe, render_summary, action_letter, action_description};
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p mx-lib --lib recipe::updater::summary_tests`
Expected: 2 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/mx-lib/src/recipe/updater.rs crates/mx-lib/src/recipe/mod.rs
git commit -m "feat(mx-lib): human-readable summary for update plans"
```

---

## Task 8: Apply an update plan (non-interactive path)

Takes an `UpdatePlan` + project_root + updated manifest data, writes all changes to disk, and saves a new manifest. Deletions are handled via a `delete_recipe_removed` boolean parameter (the interactive prompt layer lives in the CLI).

**Files:**
- Modify: `crates/mx-lib/src/recipe/updater.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/mx-lib/src/recipe/updater.rs`:

```rust
/// Outcome of applying an update plan.
#[derive(Debug, Default)]
pub struct ApplyResult {
    pub written: Vec<String>,
    pub deleted: Vec<String>,
    pub kept_with_conflicts: Vec<String>,
    pub binary_conflicts_saved_as_new: Vec<String>,
    pub skipped: Vec<String>,
}

/// Decisions for recipe-removed files. Key = relative path.
pub type DeletionDecisions = BTreeMap<String, bool>; // true = delete, false = keep

/// Apply an update plan. `delete_decisions` tells the applier what to do for
/// `RecipeRemoved` entries. Entries not present in the map are kept (safe default).
pub fn apply_plan(
    plan: &UpdatePlan,
    project_root: &Path,
    latest: &BTreeMap<String, (Vec<u8>, FileSource)>,
    manifest: &Manifest,
    delete_decisions: &DeletionDecisions,
) -> Result<ApplyResult> {
    let mut result = ApplyResult::default();
    let mut new_files: BTreeMap<String, FileEntry> = manifest.files.clone();

    for entry in &plan.entries {
        let abs = project_root.join(&entry.path);
        match &entry.action {
            PlannedAction::Unchanged | PlannedAction::KeepOurs => {
                // No disk change. Keep manifest entry as-is.
            }
            PlannedAction::TakeTheirs { new_content }
            | PlannedAction::AutoMerged { new_content } => {
                write_file_with_parents(&abs, new_content)?;
                // Update baseline + hash to the new content (it's the new "original").
                Manifest::write_baseline(project_root, &entry.path, new_content)?;
                let source = latest
                    .get(&entry.path)
                    .map(|(_, s)| s.clone())
                    .or_else(|| manifest.files.get(&entry.path).map(|e| e.source.clone()))
                    .unwrap_or(FileSource::Recipe);
                new_files.insert(
                    entry.path.clone(),
                    FileEntry { sha256: sha256_hex(new_content), source },
                );
                result.written.push(entry.path.clone());
            }
            PlannedAction::Conflict { new_content, is_binary } => {
                if *is_binary {
                    // Save latest as `<path>.new` instead of clobbering.
                    let new_path = format!("{}.new", entry.path);
                    let abs_new = project_root.join(&new_path);
                    write_file_with_parents(&abs_new, new_content)?;
                    result.binary_conflicts_saved_as_new.push(new_path);
                    result.kept_with_conflicts.push(entry.path.clone());
                    // Do NOT update manifest/baseline — user hasn't accepted the change yet.
                } else {
                    write_file_with_parents(&abs, new_content)?;
                    // Do NOT update baseline — user must resolve conflict markers manually.
                    result.kept_with_conflicts.push(entry.path.clone());
                }
            }
            PlannedAction::Added { new_content, source } => {
                write_file_with_parents(&abs, new_content)?;
                Manifest::write_baseline(project_root, &entry.path, new_content)?;
                new_files.insert(
                    entry.path.clone(),
                    FileEntry { sha256: sha256_hex(new_content), source: source.clone() },
                );
                result.written.push(entry.path.clone());
            }
            PlannedAction::RecipeRemoved { .. } => {
                let delete = delete_decisions.get(&entry.path).copied().unwrap_or(false);
                if delete && abs.exists() {
                    std::fs::remove_file(&abs).ok();
                    let baseline = Manifest::baseline_path(project_root, &entry.path);
                    let _ = std::fs::remove_file(&baseline);
                    new_files.remove(&entry.path);
                    result.deleted.push(entry.path.clone());
                } else {
                    result.skipped.push(entry.path.clone());
                }
            }
            PlannedAction::LocallyMissing => {
                result.skipped.push(entry.path.clone());
            }
        }
    }

    // Write updated manifest with new recipe_sha from latest render (caller supplies).
    let mut updated = manifest.clone();
    updated.files = new_files;
    updated.save(project_root)?;

    Ok(result)
}

fn write_file_with_parents(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    std::fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod apply_tests {
    use super::*;
    use tempfile::TempDir;

    fn seed_manifest(root: &Path) -> Manifest {
        let mut files = BTreeMap::new();
        files.insert(
            "a.txt".to_string(),
            FileEntry { sha256: sha256_hex(b"hello\n"), source: FileSource::Recipe },
        );
        std::fs::write(root.join("a.txt"), "hello\n").unwrap();
        Manifest::write_baseline(root, "a.txt", b"hello\n").unwrap();

        let manifest = Manifest {
            manifest_version: MANIFEST_VERSION,
            recipe: "sample".into(),
            recipe_sha: None,
            scaffolded_at: "2026-04-17T00:00:00Z".into(),
            mx_version: "test".into(),
            placeholders: BTreeMap::new(),
            files,
        };
        manifest.save(root).unwrap();
        manifest
    }

    #[test]
    fn test_apply_take_theirs_updates_disk_and_manifest() {
        let temp = TempDir::new().unwrap();
        let manifest = seed_manifest(temp.path());

        let mut latest = BTreeMap::new();
        latest.insert("a.txt".to_string(), (b"goodbye\n".to_vec(), FileSource::Recipe));

        let plan = UpdatePlan {
            entries: vec![PlanEntry {
                path: "a.txt".into(),
                action: PlannedAction::TakeTheirs { new_content: b"goodbye\n".to_vec() },
            }],
        };

        apply_plan(&plan, temp.path(), &latest, &manifest, &BTreeMap::new()).unwrap();

        assert_eq!(std::fs::read(temp.path().join("a.txt")).unwrap(), b"goodbye\n");
        let reloaded = Manifest::load(temp.path()).unwrap();
        assert_eq!(reloaded.files["a.txt"].sha256, sha256_hex(b"goodbye\n"));
        let baseline = Manifest::read_baseline(temp.path(), "a.txt").unwrap();
        assert_eq!(baseline, b"goodbye\n");
    }

    #[test]
    fn test_apply_recipe_removed_default_keeps_file() {
        let temp = TempDir::new().unwrap();
        let manifest = seed_manifest(temp.path());

        let plan = UpdatePlan {
            entries: vec![PlanEntry {
                path: "a.txt".into(),
                action: PlannedAction::RecipeRemoved { user_modified: false },
            }],
        };

        apply_plan(&plan, temp.path(), &BTreeMap::new(), &manifest, &BTreeMap::new()).unwrap();
        assert!(temp.path().join("a.txt").exists());
    }

    #[test]
    fn test_apply_recipe_removed_with_explicit_delete() {
        let temp = TempDir::new().unwrap();
        let manifest = seed_manifest(temp.path());

        let plan = UpdatePlan {
            entries: vec![PlanEntry {
                path: "a.txt".into(),
                action: PlannedAction::RecipeRemoved { user_modified: false },
            }],
        };

        let mut decisions = BTreeMap::new();
        decisions.insert("a.txt".to_string(), true);
        apply_plan(&plan, temp.path(), &BTreeMap::new(), &manifest, &decisions).unwrap();

        assert!(!temp.path().join("a.txt").exists());
        let reloaded = Manifest::load(temp.path()).unwrap();
        assert!(!reloaded.files.contains_key("a.txt"));
    }

    #[test]
    fn test_apply_conflict_preserves_markers_and_skips_baseline_update() {
        let temp = TempDir::new().unwrap();
        let manifest = seed_manifest(temp.path());
        std::fs::write(temp.path().join("a.txt"), "mine\n").unwrap();

        let plan = UpdatePlan {
            entries: vec![PlanEntry {
                path: "a.txt".into(),
                action: PlannedAction::Conflict {
                    new_content: b"<<<<<<<\nmine\n=======\ntheirs\n>>>>>>>\n".to_vec(),
                    is_binary: false,
                },
            }],
        };

        apply_plan(&plan, temp.path(), &BTreeMap::new(), &manifest, &BTreeMap::new()).unwrap();

        let on_disk = std::fs::read_to_string(temp.path().join("a.txt")).unwrap();
        assert!(on_disk.contains("<<<<<<<"));

        // Baseline unchanged — user must resolve first.
        let baseline = Manifest::read_baseline(temp.path(), "a.txt").unwrap();
        assert_eq!(baseline, b"hello\n");
    }

    #[test]
    fn test_apply_added_file() {
        let temp = TempDir::new().unwrap();
        let manifest = seed_manifest(temp.path());

        let mut latest = BTreeMap::new();
        latest.insert("new.txt".to_string(), (b"new\n".to_vec(), FileSource::Common));

        let plan = UpdatePlan {
            entries: vec![PlanEntry {
                path: "new.txt".into(),
                action: PlannedAction::Added {
                    new_content: b"new\n".to_vec(),
                    source: FileSource::Common,
                },
            }],
        };

        apply_plan(&plan, temp.path(), &latest, &manifest, &BTreeMap::new()).unwrap();
        assert_eq!(std::fs::read(temp.path().join("new.txt")).unwrap(), b"new\n");
        let reloaded = Manifest::load(temp.path()).unwrap();
        assert_eq!(reloaded.files["new.txt"].source, FileSource::Common);
    }
}
```

Expose:

```rust
// in recipe/mod.rs
pub use updater::{apply_plan, ApplyResult, DeletionDecisions};
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p mx-lib --lib recipe::updater::apply_tests`
Expected: 5 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/mx-lib/src/recipe/updater.rs crates/mx-lib/src/recipe/mod.rs
git commit -m "feat(mx-lib): apply_plan writes updates and rewrites manifest"
```

---

## Task 9: `mx update` CLI command (default flow, non-interactive)

**Files:**
- Create: `crates/mx-cli/src/commands/update.rs`
- Modify: `crates/mx-cli/src/commands/mod.rs`
- Modify: `crates/mx-cli/src/main.rs`

- [ ] **Step 1: Wire the new module**

Edit `crates/mx-cli/src/commands/mod.rs`:

```rust
pub mod update;
```

Edit `crates/mx-cli/src/main.rs`, add to imports:

```rust
    update::UpdateCommand,
```

Add to the `Commands` enum:

```rust
    /// Pull latest recipe updates into this scaffolded app
    Update(UpdateCommand),
```

Add to the match in `main()`:

```rust
        Commands::Update(cmd) => cmd.run().await,
```

- [ ] **Step 2: Create the command**

Create `crates/mx-cli/src/commands/update.rs`:

```rust
//! `mx update` command — pull latest recipe updates into this app.

use anyhow::{Context, Result};
use clap::Args;
use console::style;
use dialoguer::Confirm;

use mx_lib::project::ProjectDetector;
use mx_lib::recipe::{
    apply_plan, build_plan, render_latest_recipe, render_summary,
    DeletionDecisions, Manifest, PlannedAction,
};
use mx_lib::templates_dir;

/// Pull latest recipe updates into the current project.
#[derive(Args, Debug)]
pub struct UpdateCommand {
    /// Show the summary without applying any changes.
    #[arg(long)]
    pub dry_run: bool,

    /// Restrict updates to matching paths (repeatable).
    #[arg(long = "only", value_name = "PATH")]
    pub only: Vec<String>,

    /// Add/override a placeholder (repeatable, key=value).
    #[arg(long = "set", value_name = "KEY=VALUE")]
    pub set: Vec<String>,
}

impl UpdateCommand {
    pub async fn run(&self) -> Result<()> {
        let detector = ProjectDetector::new();
        let project_root = detector.find_root_from_cwd()?;

        let mut manifest = Manifest::load(&project_root)
            .context("no manifest found — run `mx update init` to adopt this app")?;

        // Merge in --set overrides before rendering.
        for kv in &self.set {
            if let Some((k, v)) = kv.split_once('=') {
                manifest.placeholders.insert(k.to_string(), v.to_string());
            }
        }

        let templates_root = templates_dir()?;
        let latest = render_latest_recipe(&templates_root, &manifest.recipe, &manifest.placeholders)?;

        let mut plan = build_plan(&manifest, &project_root, &latest)?;

        if !self.only.is_empty() {
            plan.entries.retain(|e| {
                self.only.iter().any(|p| e.path == *p || e.path.starts_with(&format!("{}/", p.trim_end_matches('/'))))
            });
        }

        println!("{}", render_summary(&plan));

        if plan.entries.is_empty() || self.dry_run {
            return Ok(());
        }

        // Gather delete decisions for RecipeRemoved entries.
        let mut decisions: DeletionDecisions = Default::default();
        for entry in &plan.entries {
            if let PlannedAction::RecipeRemoved { user_modified } = &entry.action {
                let prompt = if *user_modified {
                    format!("Recipe removed `{}`, but you've modified it — delete?", entry.path)
                } else {
                    format!("Recipe removed `{}` — delete?", entry.path)
                };
                let yes = Confirm::new().with_prompt(prompt).default(false).interact()?;
                decisions.insert(entry.path.clone(), yes);
            }
        }

        if !Confirm::new().with_prompt("Apply these changes?").default(false).interact()? {
            println!("{} aborted.", style("✗").red());
            return Ok(());
        }

        let result = apply_plan(&plan, &project_root, &latest, &manifest, &decisions)?;

        println!();
        println!(
            "{} Updated {} file(s). {} with conflicts. {} deleted.",
            style("✓").green().bold(),
            result.written.len(),
            result.kept_with_conflicts.len(),
            result.deleted.len(),
        );
        if !result.kept_with_conflicts.is_empty() {
            println!();
            println!("Conflicts in:");
            for path in &result.kept_with_conflicts {
                println!("  {}", style(path).yellow());
            }
            println!("Resolve the conflict markers, then commit.");
        }
        if !result.binary_conflicts_saved_as_new.is_empty() {
            println!();
            println!("Binary conflicts — latest saved as `.new`:");
            for path in &result.binary_conflicts_saved_as_new {
                println!("  {}", style(path).yellow());
            }
        }

        Ok(())
    }
}
```

- [ ] **Step 3: Build + check it compiles and the help renders**

Run: `cargo build -p mx-cli`
Expected: clean build.

Run: `cargo run -p mx-cli -- update --help`
Expected: help text lists `--dry-run`, `--only`, `--set`.

- [ ] **Step 4: Commit**

```bash
git add crates/mx-cli/src/commands/update.rs crates/mx-cli/src/commands/mod.rs crates/mx-cli/src/main.rs
git commit -m "feat(mx-cli): mx update command (dry-run, only, set flags)"
```

---

## Task 10: `--interactive` conflict resolution

Add interactive per-file picker for conflicts (keep mine / take theirs / edit / skip). Fires only when `--interactive` is set.

**Files:**
- Modify: `crates/mx-cli/src/commands/update.rs`

- [ ] **Step 1: Add the flag and resolution loop**

Edit `UpdateCommand` — add an `interactive` field:

```rust
    /// Prompt per conflict: [k]eep mine / [t]ake theirs / [e]dit / [s]kip.
    #[arg(long)]
    pub interactive: bool,
```

Add near the top of `commands/update.rs`:

```rust
use std::io::Write;
use dialoguer::Select;
```

Add a helper inside the module:

```rust
/// Resolve a conflict interactively. Returns the content to write, or None to skip.
fn resolve_conflict_interactive(
    path: &str,
    ours: &[u8],
    theirs: &[u8],
    merged_with_markers: &[u8],
) -> Result<Option<Vec<u8>>> {
    let choices = &["Keep mine", "Take theirs", "Open editor on merged", "Skip this file"];
    let pick = Select::new()
        .with_prompt(format!("Conflict in {}", path))
        .items(choices)
        .default(0)
        .interact()?;
    match pick {
        0 => Ok(Some(ours.to_vec())),
        1 => Ok(Some(theirs.to_vec())),
        2 => {
            // Write merged to a temp file and let the user edit it.
            let dir = tempfile::tempdir()?;
            let tmp = dir.path().join("conflict");
            std::fs::write(&tmp, merged_with_markers)?;
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".into());
            let status = std::process::Command::new(&editor).arg(&tmp).status()?;
            if !status.success() {
                anyhow::bail!("editor exited non-zero");
            }
            Ok(Some(std::fs::read(&tmp)?))
        }
        _ => Ok(None),
    }
}
```

Walk the plan entries *before* `apply_plan`, and when `--interactive` is set and a conflict is present, replace the action with `TakeTheirs { ... }` / `KeepOurs` / skip:

```rust
        // Gather delete decisions and (optionally) resolve conflicts interactively.
        let mut decisions: DeletionDecisions = Default::default();
        if self.interactive {
            let mut replacements: Vec<(usize, Option<PlannedAction>)> = Vec::new();
            for (i, entry) in plan.entries.iter().enumerate() {
                if let PlannedAction::Conflict { is_binary: false, .. } = &entry.action {
                    let ours = std::fs::read(project_root.join(&entry.path))?;
                    let theirs = latest
                        .get(&entry.path)
                        .map(|(b, _)| b.clone())
                        .unwrap_or_default();
                    let markers = match &entry.action {
                        PlannedAction::Conflict { new_content, .. } => new_content.clone(),
                        _ => unreachable!(),
                    };
                    match resolve_conflict_interactive(&entry.path, &ours, &theirs, &markers)? {
                        Some(resolved) => replacements.push((
                            i,
                            Some(PlannedAction::TakeTheirs { new_content: resolved }),
                        )),
                        None => replacements.push((i, None)),
                    }
                }
            }
            // Apply replacements in reverse to preserve indices.
            for (i, new_action) in replacements.into_iter().rev() {
                match new_action {
                    Some(a) => plan.entries[i].action = a,
                    None => { plan.entries.remove(i); }
                }
            }
        }
```

Add `tempfile` to `mx-cli` dev/runtime deps if not present:

In `crates/mx-cli/Cargo.toml` under `[dependencies]`:

```toml
tempfile = "3.10"
```

- [ ] **Step 2: Compile check**

Run: `cargo build -p mx-cli`
Expected: clean build.

- [ ] **Step 3: Commit**

```bash
git add crates/mx-cli/src/commands/update.rs crates/mx-cli/Cargo.toml
git commit -m "feat(mx-cli): --interactive conflict picker for mx update"
```

---

## Task 11: `mx update init` adoption path

For apps scaffolded before manifest support existed, allow adopting them by capturing current state as the baseline.

**Files:**
- Modify: `crates/mx-cli/src/commands/update.rs`
- Modify: `crates/mx-lib/src/recipe/updater.rs`

- [ ] **Step 1: Add adoption helper in mx-lib**

Append to `crates/mx-lib/src/recipe/updater.rs`:

```rust
use chrono::Utc;

/// Adopt a pre-manifest app: hash every file the named recipe would produce,
/// snapshot them as baselines, and write a fresh manifest.
///
/// Use only when no `.mx/manifest.json` exists. Placeholder values must be
/// supplied via `placeholders`; if the recipe references one not provided,
/// errors with `PlaceholderMissing`.
pub fn adopt_existing_app(
    templates_root: &Path,
    recipe_name: &str,
    project_root: &Path,
    placeholders: &BTreeMap<String, String>,
    recipe_sha: Option<String>,
) -> Result<Manifest> {
    let latest = render_latest_recipe(templates_root, recipe_name, placeholders)?;

    let mut files: BTreeMap<String, FileEntry> = BTreeMap::new();
    for (rel, (content, source)) in &latest {
        let abs = project_root.join(rel);
        if !abs.is_file() {
            // Recipe declared a file the adopted app doesn't have — skip.
            continue;
        }
        // Baseline = current on-disk content (best-effort — we don't know the true original).
        let on_disk = std::fs::read(&abs)?;
        Manifest::write_baseline(project_root, rel, &on_disk)?;
        let _ = content; // unused in adoption
        files.insert(
            rel.clone(),
            FileEntry { sha256: sha256_hex(&on_disk), source: source.clone() },
        );
    }

    let manifest = Manifest {
        manifest_version: MANIFEST_VERSION,
        recipe: recipe_name.to_string(),
        recipe_sha,
        scaffolded_at: Utc::now().to_rfc3339(),
        mx_version: env!("CARGO_PKG_VERSION").to_string(),
        placeholders: placeholders.clone(),
        files,
    };
    manifest.save(project_root)?;
    Ok(manifest)
}

#[cfg(test)]
mod adopt_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_adopt_creates_manifest_from_current_files() {
        let templates = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();

        std::fs::create_dir_all(templates.path().join("recipes/sample/files")).unwrap();
        std::fs::write(
            templates.path().join("recipes/sample/recipe.json"),
            r#"{
                "name": "sample",
                "templates": [{ "from": "files/a.txt", "to": "a.txt" }]
            }"#,
        ).unwrap();
        std::fs::write(templates.path().join("recipes/sample/files/a.txt"), "{{ V }}").unwrap();

        // Pretend the app was scaffolded previously, with user edits.
        std::fs::write(project.path().join("a.txt"), "user version").unwrap();

        let mut ph = BTreeMap::new();
        ph.insert("V".to_string(), "x".to_string());

        let manifest = adopt_existing_app(
            templates.path(),
            "sample",
            project.path(),
            &ph,
            None,
        ).unwrap();

        assert_eq!(manifest.recipe, "sample");
        assert_eq!(
            manifest.files["a.txt"].sha256,
            sha256_hex(b"user version")
        );
        let baseline = Manifest::read_baseline(project.path(), "a.txt").unwrap();
        assert_eq!(baseline, b"user version");
    }
}
```

Export:

```rust
pub use updater::adopt_existing_app;
```

- [ ] **Step 2: Add the CLI subcommand**

Edit `crates/mx-cli/src/commands/update.rs`. Change `UpdateCommand` to support a subcommand:

```rust
use clap::Subcommand;
use mx_lib::recipe::adopt_existing_app;
use std::collections::BTreeMap;

#[derive(Subcommand, Debug)]
pub enum UpdateSubcommand {
    /// Adopt an existing app by creating a manifest from current file state.
    Init {
        /// Recipe name this app was scaffolded from.
        #[arg(long)]
        recipe: String,
        /// Set a placeholder (repeatable, key=value).
        #[arg(long = "set", value_name = "KEY=VALUE")]
        set: Vec<String>,
    },
}
```

Add the subcommand to `UpdateCommand`:

```rust
    #[command(subcommand)]
    pub subcommand: Option<UpdateSubcommand>,
```

Dispatch at the top of `UpdateCommand::run`:

```rust
    pub async fn run(&self) -> Result<()> {
        if let Some(sub) = &self.subcommand {
            return self.run_subcommand(sub).await;
        }
        // ... existing body
```

Add:

```rust
    async fn run_subcommand(&self, sub: &UpdateSubcommand) -> Result<()> {
        match sub {
            UpdateSubcommand::Init { recipe, set } => {
                let detector = ProjectDetector::new();
                let project_root = detector.find_root_from_cwd()?;
                if Manifest::path(&project_root).exists() {
                    anyhow::bail!(".mx/manifest.json already exists");
                }
                let templates_root = templates_dir()?;
                let mut placeholders = BTreeMap::new();
                for kv in set {
                    if let Some((k, v)) = kv.split_once('=') {
                        placeholders.insert(k.to_string(), v.to_string());
                    }
                }
                let manifest = adopt_existing_app(
                    &templates_root,
                    recipe,
                    &project_root,
                    &placeholders,
                    None,
                )?;
                println!(
                    "{} Adopted {} file(s) under recipe `{}`.",
                    console::style("✓").green().bold(),
                    manifest.files.len(),
                    recipe,
                );
                Ok(())
            }
        }
    }
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p mx-lib --lib recipe::updater::adopt_tests`
Expected: 1 test PASSES.

Run: `cargo build -p mx-cli`
Expected: clean build.

- [ ] **Step 4: Commit**

```bash
git add crates/mx-lib/src/recipe/updater.rs crates/mx-lib/src/recipe/mod.rs crates/mx-cli/src/commands/update.rs
git commit -m "feat: mx update init — adopt pre-manifest apps"
```

---

## Task 12: End-to-end integration test

Full scaffold → modify → change the "recipe" → run update workflow in an isolated tempdir, verifying auto-merge, conflict markers, added files, and deletion prompts.

**Files:**
- Create: `crates/mx-lib/tests/recipe_update_integration.rs`

- [ ] **Step 1: Write the integration test**

Create `crates/mx-lib/tests/recipe_update_integration.rs`:

```rust
//! End-to-end test for the recipe update flow.
//!
//! Simulates: scaffold app → user edits files → update the recipe templates
//! → run the plan+apply pipeline → verify expected outcomes.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use mx_lib::recipe::{
    apply_plan, build_plan, render_latest_recipe, Manifest, PlannedAction, RecipeInstaller,
    DeletionDecisions,
};

fn write(path: &Path, content: &str) {
    if let Some(p) = path.parent() { std::fs::create_dir_all(p).unwrap(); }
    std::fs::write(path, content).unwrap();
}

fn seed_recipe_v1(templates: &Path) {
    write(
        &templates.join("recipes/demo/recipe.json"),
        r#"{
            "name": "demo",
            "directories": ["apps/{{ APP_NAME }}"],
            "templates": [
                { "from": "files/readme.md", "to": "apps/{{ APP_NAME }}/README.md" },
                { "from": "files/dockerfile", "to": "apps/{{ APP_NAME }}/Dockerfile" },
                { "from": "files/old.txt", "to": "apps/{{ APP_NAME }}/old.txt" }
            ]
        }"#,
    );
    write(&templates.join("recipes/demo/files/readme.md"),
        "# {{ APP_NAME }}\n\nline two\nline three\n");
    write(&templates.join("recipes/demo/files/dockerfile"),
        "FROM rust:1.70\nWORKDIR /app\nCOPY . .\nRUN cargo build\n");
    write(&templates.join("recipes/demo/files/old.txt"), "stays the same\n");
}

fn update_recipe_to_v2(templates: &Path) {
    // Recipe drops old.txt, edits Dockerfile line 1, adds a new file.
    write(
        &templates.join("recipes/demo/recipe.json"),
        r#"{
            "name": "demo",
            "directories": ["apps/{{ APP_NAME }}"],
            "templates": [
                { "from": "files/readme.md", "to": "apps/{{ APP_NAME }}/README.md" },
                { "from": "files/dockerfile", "to": "apps/{{ APP_NAME }}/Dockerfile" },
                { "from": "files/ci.yml", "to": "apps/{{ APP_NAME }}/ci.yml" }
            ]
        }"#,
    );
    write(&templates.join("recipes/demo/files/dockerfile"),
        "FROM rust:1.80\nWORKDIR /app\nCOPY . .\nRUN cargo build --release\n");
    // readme unchanged
    write(&templates.join("recipes/demo/files/ci.yml"), "name: ci\n");
    // old.txt removed from recipe dir
    let _ = std::fs::remove_file(templates.join("recipes/demo/files/old.txt"));
}

#[test]
fn full_update_workflow() {
    let templates = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();

    // 1. Scaffold with recipe v1.
    seed_recipe_v1(templates.path());
    let mut installer = RecipeInstaller::new(templates.path()).unwrap();
    let recipe = installer.load_recipe("demo").unwrap();
    let mut opts = HashMap::new();
    opts.insert("APP_NAME".to_string(), "myapp".to_string());
    installer.install(&recipe, project.path(), "myapp", &opts).unwrap();

    // Sanity: manifest exists.
    let manifest = Manifest::load(project.path()).unwrap();
    assert_eq!(manifest.recipe, "demo");
    assert!(manifest.files.contains_key("apps/myapp/README.md"));
    assert!(manifest.files.contains_key("apps/myapp/old.txt"));

    // 2. User edits README (non-conflicting line) and Dockerfile line 4 (non-conflicting).
    write(
        &project.path().join("apps/myapp/README.md"),
        "# myapp\n\nline two edited by user\nline three\n",
    );
    write(
        &project.path().join("apps/myapp/Dockerfile"),
        "FROM rust:1.70\nWORKDIR /app\nCOPY . .\nRUN cargo build --user-flag\n",
    );

    // 3. Recipe author ships v2.
    update_recipe_to_v2(templates.path());

    // 4. Build + inspect the plan.
    let latest = render_latest_recipe(templates.path(), "demo", &manifest.placeholders).unwrap();
    let plan = build_plan(&manifest, project.path(), &latest).unwrap();

    // Expect entries for:
    //   README.md (KeepOurs — recipe didn't change it)
    //   Dockerfile (Conflict — both changed, overlapping line 1 and 4? actually line 1 vs line 4 is non-overlap so AutoMerged)
    //   old.txt (RecipeRemoved)
    //   ci.yml (Added)
    // Note: diffy auto-merges non-overlapping hunks.

    let actions: BTreeMap<String, PlannedAction> = plan
        .entries
        .iter()
        .map(|e| (e.path.clone(), e.action.clone()))
        .collect();

    assert!(matches!(actions.get("apps/myapp/README.md"), Some(PlannedAction::KeepOurs)));
    assert!(matches!(actions.get("apps/myapp/Dockerfile"), Some(PlannedAction::AutoMerged { .. })));
    assert!(matches!(actions.get("apps/myapp/old.txt"), Some(PlannedAction::RecipeRemoved { .. })));
    assert!(matches!(actions.get("apps/myapp/ci.yml"), Some(PlannedAction::Added { .. })));

    // 5. Apply — accept the deletion of old.txt.
    let mut decisions: DeletionDecisions = Default::default();
    decisions.insert("apps/myapp/old.txt".to_string(), true);

    let result = apply_plan(&plan, project.path(), &latest, &manifest, &decisions).unwrap();

    assert!(!project.path().join("apps/myapp/old.txt").exists(), "old.txt should be deleted");
    assert!(project.path().join("apps/myapp/ci.yml").exists(), "ci.yml should be added");
    let dockerfile = std::fs::read_to_string(project.path().join("apps/myapp/Dockerfile")).unwrap();
    assert!(dockerfile.contains("FROM rust:1.80"), "recipe upgrade to rust:1.80 applied");
    assert!(dockerfile.contains("--user-flag"), "user's Dockerfile edit preserved");
    let readme = std::fs::read_to_string(project.path().join("apps/myapp/README.md")).unwrap();
    assert!(readme.contains("line two edited by user"), "user's README edit preserved");

    // Manifest updated.
    let reloaded = Manifest::load(project.path()).unwrap();
    assert!(!reloaded.files.contains_key("apps/myapp/old.txt"));
    assert!(reloaded.files.contains_key("apps/myapp/ci.yml"));

    assert_eq!(result.deleted, vec!["apps/myapp/old.txt"]);
    assert!(result.written.iter().any(|p| p == "apps/myapp/ci.yml"));
    assert!(result.written.iter().any(|p| p == "apps/myapp/Dockerfile"));
}

#[test]
fn conflict_preserves_markers_and_skips_baseline_update() {
    let templates = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();

    // Recipe v1 with a single file.
    write(
        &templates.path().join("recipes/c/recipe.json"),
        r#"{ "name": "c", "templates": [{ "from": "files/f.txt", "to": "f.txt" }] }"#,
    );
    write(&templates.path().join("recipes/c/files/f.txt"), "line\n");

    let mut installer = RecipeInstaller::new(templates.path()).unwrap();
    let recipe = installer.load_recipe("c").unwrap();
    installer.install(&recipe, project.path(), "c", &HashMap::new()).unwrap();

    // User and recipe both change the same line.
    write(&project.path().join("f.txt"), "user-line\n");
    write(&templates.path().join("recipes/c/files/f.txt"), "recipe-line\n");

    let manifest = Manifest::load(project.path()).unwrap();
    let latest = render_latest_recipe(templates.path(), "c", &manifest.placeholders).unwrap();
    let plan = build_plan(&manifest, project.path(), &latest).unwrap();
    apply_plan(&plan, project.path(), &latest, &manifest, &DeletionDecisions::default()).unwrap();

    let on_disk = std::fs::read_to_string(project.path().join("f.txt")).unwrap();
    assert!(on_disk.contains("<<<<<<<"), "conflict markers written");

    // Baseline NOT updated — manifest hash still matches original scaffold.
    let reloaded = Manifest::load(project.path()).unwrap();
    let baseline = Manifest::read_baseline(project.path(), "f.txt").unwrap();
    assert_eq!(baseline, b"line\n");
    assert_eq!(reloaded.files["f.txt"].sha256, mx_lib::recipe::sha256_hex(b"line\n"));
}
```

- [ ] **Step 2: Run the integration test**

Run: `cargo test -p mx-lib --test recipe_update_integration`
Expected: both tests PASS.

- [ ] **Step 3: Run the full test suite to make sure nothing else broke**

Run: `cargo test --workspace`
Expected: all tests PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/mx-lib/tests/recipe_update_integration.rs
git commit -m "test(mx-lib): end-to-end integration test for recipe update workflow"
```

---

## Post-implementation sanity checks (not tasks)

Before marking the feature complete, manually validate:

1. `cargo clippy --workspace -- -D warnings` — no lints regress.
2. `cargo fmt --check` — formatting consistent.
3. Scaffold a new app end-to-end:
   ```
   mx add myservice --recipe rust-api
   ls .mx/manifest.json .mx/baselines/
   mx update --dry-run    # should say "Already up to date."
   ```
4. Simulate an update (in the mech-crate repo: change a `templates/recipes/rust-api/*` file, then in the scaffolded app run `mx update` and verify the summary + confirm flow behaves).
5. Scaffold, delete `.mx/`, run `mx update` — expect the `ManifestNotFound` error with the hint pointing at `mx update init`.

---

## Self-review against spec

- ✅ Manifest shape (recipe, recipe_sha, scaffolded_at, mx_version, placeholders, files with sha256+source) — Task 2.
- ✅ Manifest written at scaffold time — Task 3.
- ✅ Git SHA captured (best-effort) — Task 3 (`templates_git_sha`).
- ✅ 3-way merge via baselines — Task 4.
- ✅ Auto-merge, KeepOurs, TakeTheirs, Conflict, Added, RecipeRemoved — Task 5.
- ✅ Re-render latest with stored placeholders — Task 6.
- ✅ Placeholder drift surfaces as a clear error — Task 6 (`PlaceholderMissing`).
- ✅ `--set KEY=VALUE` — Task 9.
- ✅ common:// files tracked separately — Task 3 (FileSource) + Task 6 (render).
- ✅ Binary file handling (save as `.new` on conflict) — Task 4 + Task 8.
- ✅ Dry-run default preview, explicit confirm — Task 9.
- ✅ `--only <path>` filter — Task 9.
- ✅ Conflict markers by default — Task 4 (diffy).
- ✅ `--interactive` conflict picker — Task 10.
- ✅ Recipe deletion prompts — Task 9.
- ✅ User-added files ignored — Task 5 (plan iterates `union(manifest, latest)` only; disk-only files never appear).
- ✅ Locally-missing files treated as user-deleted — Task 5 (`LocallyMissing`).
- ✅ `mx update init` adoption — Task 11.
- ✅ End-to-end validation — Task 12.
