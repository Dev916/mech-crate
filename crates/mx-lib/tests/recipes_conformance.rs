//! Conformance sweep: every recipe shipped in `templates/recipes/` must validate clean
//! against the strict validator. This is what makes silent recipe drift (the astro
//! `npm_install` / `"kebab"` class) loud.

use std::path::{Path, PathBuf};

fn recipes_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/recipes")
}

#[test]
fn every_shipped_recipe_validates_clean() {
    let root = recipes_root();
    let mut checked = 0;
    let mut failures: Vec<String> = Vec::new();

    let mut entries: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("templates/recipes must exist")
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();

    for dir in entries {
        let rj = dir.join("recipe.json");
        if !rj.exists() {
            continue;
        }
        let text = std::fs::read_to_string(&rj).unwrap();
        let raw: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("{}: invalid JSON: {e}", rj.display()));

        let findings = mx_lib::recipe::validate::validate_recipe_json_with_root(&raw, Some(&root));
        if !findings.is_empty() {
            for f in &findings {
                failures.push(format!("{}: {} :: {}", rj.display(), f.path, f.message));
            }
        }
        checked += 1;
    }

    assert!(
        failures.is_empty(),
        "recipe conformance findings:\n{}",
        failures.join("\n")
    );
    assert!(checked >= 7, "expected >=7 recipes, found {checked}");
}

#[test]
fn every_shipped_recipe_parses_into_the_typed_struct() {
    let root = recipes_root();
    for entry in std::fs::read_dir(&root).unwrap().flatten() {
        let rj = entry.path().join("recipe.json");
        if !rj.exists() {
            continue;
        }
        let recipe =
            mx_lib::recipe::Recipe::load(&rj).unwrap_or_else(|e| panic!("{}: {e}", rj.display()));
        assert!(!recipe.name.is_empty(), "{}: empty name", rj.display());
    }
}

// ── Known-broken lane (bd:mech-crate-ten) ────────────────────────────────────

/// `make release app=<app>` shells into `apps/<app>` and runs `yarn release*`,
/// but recipe-generated apps ship no such scripts, so the conventional release
/// path fails on every fresh app until someone bootstraps it by hand.
///
/// Asserts the FIXED behavior: every `yarn release*` script the release module
/// invokes is declared by the recipe's app `package.json`. Expected RED until
/// bd:mech-crate-ten lands. (If that issue is instead closed by making
/// `make release` degrade gracefully, retire this test with it.)
#[test]
#[ignore = "bd:mech-crate-ten recipe apps ship no release scripts; make release fails"]
fn kb_nuxt_recipe_ships_the_release_scripts_make_invokes() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/mx-lib should sit two levels below the repo root")
        .to_path_buf();

    let release_mk = std::fs::read_to_string(repo_root.join("templates/make/release.mk"))
        .expect("setup: read templates/make/release.mk");
    let mut required: Vec<String> = release_mk
        .lines()
        .filter_map(|l| l.split("yarn ").nth(1))
        .map(|rest| rest.split_whitespace().next().unwrap_or("").to_string())
        .filter(|s| s.starts_with("release"))
        .collect();
    required.sort();
    required.dedup();
    assert!(
        !required.is_empty(),
        "setup: release.mk invokes no `yarn release*` scripts at all"
    );

    let pkg_path = repo_root.join("templates/recipes/nuxt/app/package.json");
    let pkg: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&pkg_path).expect("setup: read nuxt app package.json"),
    )
    .expect("setup: parse nuxt app package.json");
    let scripts = pkg["scripts"]
        .as_object()
        .expect("setup: nuxt app package.json must declare a scripts object");

    let missing: Vec<&String> = required
        .iter()
        .filter(|s| !scripts.contains_key(*s))
        .collect();
    assert!(
        missing.is_empty(),
        "{} must declare the release scripts `make release` invokes; missing {:?}",
        pkg_path.display(),
        missing
    );
}
