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
