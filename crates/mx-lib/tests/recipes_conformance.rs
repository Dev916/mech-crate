//! Conformance sweep: every recipe shipped in `templates/recipes/` must validate clean
//! against the strict validator, and must *install* cleanly into an empty project.
//! This is what makes silent recipe drift (the astro `npm_install` / `"kebab"` class)
//! and install-time regressions (the Tera-eats-app-sources class) loud.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn templates_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates")
}

fn recipes_root() -> PathBuf {
    templates_root().join("recipes")
}

/// Every recipe directory shipped under `templates/recipes/`, `common/` excluded.
fn shipped_recipe_names() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(recipes_root())
        .expect("setup: templates/recipes must exist")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join("recipe.json").exists())
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect();
    names.sort();
    names
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

// ── Installer round-trip ─────────────────────────────────────────────────────

/// Install `recipe_name` into a fresh tempdir project and return the project root.
///
/// `init_app` is neutralised by pre-creating its `target_dir`: every recipe that
/// declares one sets `skip_if_exists`, so the network scaffolder (`npm create
/// astro`, `nuxi init`, `zola init`) never runs from the test suite.
fn install_into_tempdir(
    recipe_name: &str,
    service: &str,
    options: &HashMap<String, String>,
) -> (tempfile::TempDir, mx_lib::recipe::InstallResult) {
    let project = tempfile::tempdir().expect("setup: tempdir");
    let mut installer = mx_lib::recipe::RecipeInstaller::new(templates_root())
        .expect("setup: build a recipe installer");
    let recipe = installer
        .load_recipe(recipe_name)
        .unwrap_or_else(|e| panic!("setup: load recipe {recipe_name}: {e}"));

    if let Some(init_app) = &recipe.init_app {
        assert!(
            init_app.skip_if_exists && init_app.target_dir.is_some(),
            "setup: {recipe_name} declares an init_app the test cannot neutralise \
             (needs skip_if_exists + target_dir); it would shell out to the network"
        );
        let target = init_app
            .target_dir
            .as_ref()
            .unwrap()
            .replace("{{SERVICE_NAME}}", service);
        std::fs::create_dir_all(project.path().join(target)).expect("setup: pre-create target_dir");
    }

    let result = installer
        .install(&recipe, project.path(), service, options)
        .unwrap_or_else(|e| panic!("{recipe_name}: install failed: {e}"));
    (project, result)
}

/// The regression net for the whole "installer chokes on a recipe payload" class:
/// every shipped recipe must install into an empty project without erroring.
///
/// Historically Tera one-off rendering evaluated app sources, so recipes carrying
/// Blade/Vue/Zola templates (`{{ page.title }}`, `{% extends "base.html" %}`)
/// blew up at `mx add` — laravel, rust-worker and zola all failed here.
#[test]
fn every_shipped_recipe_installs_into_a_clean_project() {
    let names = shipped_recipe_names();
    assert!(
        names.len() >= 7,
        "setup: expected >=7 recipes, found {}",
        names.len()
    );

    for name in &names {
        let (project, result) = install_into_tempdir(name, "svc", &HashMap::new());
        assert!(
            !result.files_created.is_empty(),
            "{name}: install created no files at all"
        );
        assert!(
            project.path().join("docker/compose").is_dir(),
            "{name}: install produced no docker/compose directory"
        );
    }
}

/// Placeholder expansion must still happen — in file contents, in destination
/// paths, and in `next_steps`. Byte-level check on the pieces the four
/// historically-working recipes depend on.
#[test]
fn install_expands_placeholders_in_paths_and_content() {
    let (project, result) = install_into_tempdir("rust-api", "api", &HashMap::new());
    let root = project.path();

    assert!(
        root.join("docker/compose/api.yml").is_file(),
        "rust-api: destination path placeholder not expanded (docker/compose/api.yml missing); got {:?}",
        result.files_created
    );

    let cargo = std::fs::read_to_string(root.join("apps/api/Cargo.toml"))
        .expect("setup: rust-api ships apps/<svc>/Cargo.toml");
    assert!(
        cargo.contains("name = \"api\""),
        "rust-api: Cargo.toml placeholder not expanded:\n{cargo}"
    );
    assert!(
        !cargo.contains("{{"),
        "rust-api: Cargo.toml still carries an unexpanded token:\n{cargo}"
    );

    // `{{- SERVICE_UPPER }}` uses Tera's whitespace-trim marker; the shipped
    // env.shared writes `${ {{- SERVICE_UPPER }}_DB_PASSWORD}` and must render
    // as `${API_DB_PASSWORD}` — no stray space.
    let shared = std::fs::read_to_string(root.join("docker/.config/.env.shared"))
        .expect("setup: rust-api ships docker/.config/.env.shared");
    assert!(
        shared.contains("DB_PASSWORD=${API_DB_PASSWORD}"),
        "rust-api: whitespace-trim placeholder mis-rendered:\n{shared}"
    );
}

/// App-source payloads must survive installation byte-for-byte: their `{{ }}`
/// and `{% %}` belong to Blade / Vue / Zola, not to the installer.
#[test]
fn install_leaves_app_source_template_syntax_untouched() {
    // Zola theme: inheritance + expressions the site renderer owns.
    let (project, _) = install_into_tempdir("zola", "site", &HashMap::new());
    let base = std::fs::read_to_string(project.path().join("apps/site/templates/base.html"))
        .expect("setup: zola ships app/templates/base.html");
    assert!(
        base.contains("{% block content %}") && base.contains("{{ config.title }}"),
        "zola: theme template syntax was consumed by the installer:\n{base}"
    );

    let index = std::fs::read_to_string(project.path().join("apps/site/templates/index.html"))
        .expect("setup: zola ships app/templates/index.html");
    assert!(
        index.contains(r#"{% extends "base.html" %}"#),
        "zola: `extends` was consumed by the installer:\n{index}"
    );

    // Laravel: Blade + Vue expressions in the copied app tree.
    let (project, _) = install_into_tempdir("laravel", "web", &HashMap::new());
    let blade = std::fs::read_to_string(
        project
            .path()
            .join("apps/web/resources/views/app.blade.php"),
    )
    .expect("setup: laravel ships resources/views/app.blade.php");
    assert!(
        blade.contains("{{") && blade.contains("}}"),
        "laravel: Blade expressions were consumed by the installer:\n{blade}"
    );
}

/// bd:mech-crate-290 — every shipped recipe defaults `domain` to
/// `{{SERVICE_NAME}}.localhost`. That default is itself a placeholder, so unless
/// option values are expanded against the placeholder map the generated Traefik
/// rule ships the literal token and the service is unroutable.
#[test]
fn omitting_domain_yields_a_real_host_rule_not_a_placeholder() {
    for (recipe, service) in [
        ("rust-api", "api"),
        ("rust-leptos", "ui"),
        ("nuxt", "site"),
        ("astro", "docs"),
        ("laravel", "web"),
        ("zola", "blog"),
    ] {
        let (project, _) = install_into_tempdir(recipe, service, &HashMap::new());
        let compose = project.path().join(format!("docker/compose/{service}.yml"));
        let text = std::fs::read_to_string(&compose)
            .unwrap_or_else(|e| panic!("setup: {recipe} ships {}: {e}", compose.display()));

        assert!(
            text.contains(&format!("Host(`{service}.localhost`)")),
            "{recipe}: expected Host(`{service}.localhost`) in {}:\n{text}",
            compose.display()
        );
        assert!(
            !text.contains("{{"),
            "{recipe}: unexpanded placeholder survived into {}:\n{text}",
            compose.display()
        );
    }
}

/// An explicitly supplied option value gets the same treatment — a caller may
/// pass `--domain '{{SERVICE_NAME}}.example.com'` and expect it resolved.
#[test]
fn explicit_domain_option_is_expanded_too() {
    let options = HashMap::from([(
        "domain".to_string(),
        "{{SERVICE_NAME}}.example.com".to_string(),
    )]);
    let (project, _) = install_into_tempdir("rust-api", "api", &options);
    let text = std::fs::read_to_string(project.path().join("docker/compose/api.yml"))
        .expect("setup: rust-api ships docker/compose/<svc>.yml");
    assert!(
        text.contains("Host(`api.example.com`)"),
        "explicit --domain not expanded:\n{text}"
    );
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
