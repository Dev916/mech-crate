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

/// Marker the stub scaffolder leaves behind, standing in for the framework
/// markers (`package.json`, `config.toml`) the real tools write.
const SCAFFOLD_MARKER: &str = ".mx-scaffolder-ran";

/// Offline stand-in for a recipe's network scaffolder.
///
/// Every recipe that declares an `init_app` routes its command through the
/// `init_cmd` option, so the suite swaps in a shell one-liner that does what the
/// real scaffolders do — create `<service>/` and drop a marker in it — with no
/// npm, npx, zola or network. The installer still walks its real code path: the
/// skip decision, cwd creation, `sh -c`, outcome reporting.
fn stub_init_cmd() -> String {
    format!("mkdir -p {{{{SERVICE_NAME}}}} && printf 'stub scaffolder\\n' > {{{{SERVICE_NAME}}}}/{SCAFFOLD_MARKER}")
}

/// Install `recipe_name` into a fresh tempdir project and return the project root.
///
/// Installs into `project`, which lets a caller run two `mx add`s over the same
/// tree. The recipe's scaffolder is replaced by [`stub_init_cmd`].
fn install_into(
    project: &Path,
    recipe_name: &str,
    service: &str,
    options: &HashMap<String, String>,
) -> mx_lib::recipe::InstallResult {
    let mut installer = mx_lib::recipe::RecipeInstaller::new(templates_root())
        .expect("setup: build a recipe installer");
    let recipe = installer
        .load_recipe(recipe_name)
        .unwrap_or_else(|e| panic!("setup: load recipe {recipe_name}: {e}"));

    let mut options = options.clone();
    if let Some(init_app) = &recipe.init_app {
        // The override only bites if the recipe takes its command from the
        // option. A recipe that hardcodes `npm create ...` in `init_app.command`
        // would make this suite shell out to the network — fail loudly instead.
        assert_eq!(
            init_app.command.trim(),
            "{{INIT_CMD}}",
            "setup: {recipe_name} hardcodes its init_app command instead of taking it \
             from the `init_cmd` option, so the suite cannot stub it out and would \
             shell out to the network"
        );
        assert!(
            init_app.target_dir.is_some(),
            "setup: {recipe_name} declares an init_app without a target_dir, so a \
             re-run would re-scaffold over an existing app"
        );
        options
            .entry("init_cmd".to_string())
            .or_insert_with(stub_init_cmd);
    }

    installer
        .install(&recipe, project, service, &options)
        .unwrap_or_else(|e| panic!("{recipe_name}: install failed: {e}"))
}

/// Install `recipe_name` into a fresh tempdir project and return the project root.
fn install_into_tempdir(
    recipe_name: &str,
    service: &str,
    options: &HashMap<String, String>,
) -> (tempfile::TempDir, mx_lib::recipe::InstallResult) {
    let project = tempfile::tempdir().expect("setup: tempdir");
    let result = install_into(project.path(), recipe_name, service, options);
    (project, result)
}

// ── The scaffolder actually runs (bd:mech-crate-0uq) ─────────────────────────

/// Recipes shipping an `init_app`, by name.
fn recipes_with_a_scaffolder() -> Vec<String> {
    shipped_recipe_names()
        .into_iter()
        .filter(|name| {
            let rj = recipes_root().join(name).join("recipe.json");
            mx_lib::recipe::Recipe::load(&rj)
                .map(|r| r.init_app.is_some())
                .unwrap_or(false)
        })
        .collect()
}

/// bd:mech-crate-0uq — mx created the recipe's `directories` (which start with
/// `apps/<svc>/…`) *before* `init_app`, so the `skip_if_exists` guard tripped on a
/// directory mx had just made and the framework scaffolder never ran: every astro
/// / nuxt / zola app landed with nothing but the recipe's health endpoint, and the
/// Docker build then failed on a missing `package.json`.
///
/// Asserts the fixed contract: the scaffolder runs, and its output survives the
/// directory and template passes that follow it.
#[test]
fn every_recipe_with_a_scaffolder_runs_it_and_keeps_its_output() {
    let names = recipes_with_a_scaffolder();
    assert!(
        names.len() >= 3,
        "setup: expected the astro/nuxt/zola recipes to declare an init_app, found {names:?}"
    );

    for name in &names {
        let (project, result) = install_into_tempdir(name, "svc", &HashMap::new());

        match result.init_app {
            Some(mx_lib::recipe::InitAppOutcome::Ran { ref command }) => {
                assert!(
                    command.contains(SCAFFOLD_MARKER) && command.contains("svc"),
                    "{name}: init_cmd reached the shell unexpanded: {command}"
                );
            }
            other => panic!(
                "{name}: the app scaffolder did not run on a fresh install: {other:?} \
                 (bd:mech-crate-0uq)"
            ),
        }

        let marker = project.path().join("apps/svc").join(SCAFFOLD_MARKER);
        assert!(
            marker.is_file(),
            "{name}: scaffolder output was wiped by the rest of the install \
             (expected {})",
            marker.display()
        );
    }
}

/// The recipe's own payload must land *on top of* the scaffolded app — that is
/// what makes "scaffolder first" safe: mx-specific wiring wins collisions.
#[test]
fn recipe_files_layer_on_top_of_the_scaffolded_app() {
    for (recipe, service, mx_file) in [
        ("astro", "docs", "apps/docs/src/pages/api/health.ts"),
        ("nuxt", "site", "apps/site/server/api/health.get.ts"),
        ("zola", "blog", "apps/blog/config.toml"),
    ] {
        let (project, _) = install_into_tempdir(recipe, service, &HashMap::new());
        assert!(
            project.path().join(mx_file).is_file(),
            "{recipe}: {mx_file} missing — recipe payload did not land over the scaffold"
        );
        assert!(
            project
                .path()
                .join(format!("apps/{service}"))
                .join(SCAFFOLD_MARKER)
                .is_file(),
            "{recipe}: scaffold marker missing — the payload pass clobbered the app"
        );
    }
}

/// A second `mx add` over a scaffolded app must not re-run the scaffolder: the
/// tools refuse a populated target, and the user's app is not ours to overwrite.
#[test]
fn a_second_add_over_a_scaffolded_app_skips_the_scaffolder() {
    for name in &recipes_with_a_scaffolder() {
        let project = tempfile::tempdir().expect("setup: tempdir");
        install_into(project.path(), name, "svc", &HashMap::new());
        let again = install_into(project.path(), name, "svc", &HashMap::new());

        assert_eq!(
            again.init_app,
            Some(mx_lib::recipe::InitAppOutcome::SkippedExisting {
                target_dir: "apps/svc".to_string()
            }),
            "{name}: re-running `mx add` re-scaffolded over an existing app"
        );
    }
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

// ── Composability of the installed project (bd:mech-crate-eic) ───────────────

/// What `mx new` + `scripts/init.sh` put in `docker/.config/` before any recipe
/// runs. Everything *else* a recipe's compose files reference is the recipe's own
/// job to ship — which is exactly what the astro recipe got wrong.
const SCAFFOLD_PROVIDED_ENV_FILES: &[&str] = &[
    "docker/.config/.env.shared",
    "docker/.config/.env.secrets.template",
    // init.sh copies the template to .env.secrets on first run.
    "docker/.config/.env.secrets",
];

/// Finish an installed recipe into something a human would actually run: add the
/// env files `mx new`/`make init` supply, and the dirs the shipped compose files
/// bind-mount.
fn complete_project_like_mx_new(root: &Path) {
    std::fs::create_dir_all(root.join("docker/.config")).unwrap();
    for rel in SCAFFOLD_PROVIDED_ENV_FILES {
        let path = root.join(rel);
        if !path.exists() {
            std::fs::write(&path, "# test fixture\n").unwrap();
        }
    }
}

/// Base (non-dev) then dev compose files, in the order `compose_context_files`
/// from `templates/scripts/.bashrc` assembles them for `make dev` with no
/// service argument.
fn dev_compose_context(root: &Path) -> Vec<PathBuf> {
    let dir = root.join("docker/compose");
    let mut all: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("setup: read {}: {e}", dir.display()))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "yml"))
        .collect();
    all.sort();

    let (dev, base): (Vec<PathBuf>, Vec<PathBuf>) = all
        .into_iter()
        .partition(|p| p.to_string_lossy().ends_with(".dev.yml"));
    base.into_iter().chain(dev).collect()
}

/// `include:` targets declared by one compose file, resolved against its own
/// directory. Handles both the short (`- ./db.yml`) and long (`- path: ./db.yml`)
/// forms; `path` may itself be a list.
fn include_targets(compose_file: &Path) -> Vec<(String, PathBuf)> {
    let body = std::fs::read_to_string(compose_file)
        .unwrap_or_else(|e| panic!("setup: read {}: {e}", compose_file.display()));
    let doc: serde_yaml::Value = serde_yaml::from_str(&body)
        .unwrap_or_else(|e| panic!("{}: not valid YAML: {e}", compose_file.display()));
    let base = compose_file.parent().unwrap();

    let Some(entries) = doc.get("include").and_then(|v| v.as_sequence()) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    let mut push = |raw: &serde_yaml::Value| {
        if let Some(p) = raw.as_str() {
            out.push((p.to_string(), base.join(p)));
        }
    };
    for entry in entries {
        match entry {
            serde_yaml::Value::String(_) => push(entry),
            serde_yaml::Value::Mapping(map) => {
                assert!(
                    !map.contains_key(serde_yaml::Value::from("optional")),
                    "{}: `include` has no `optional` field — Compose's long syntax accepts \
                     only path / project_directory / env_file, so the key is silently ignored \
                     and a missing sibling is a hard error (bd:mech-crate-eic)",
                    compose_file.display()
                );
                match map.get(serde_yaml::Value::from("path")) {
                    Some(serde_yaml::Value::Sequence(paths)) => paths.iter().for_each(&mut push),
                    Some(v) => push(v),
                    None => panic!(
                        "{}: `include` entry without a `path`: {entry:?}",
                        compose_file.display()
                    ),
                }
            }
            other => panic!(
                "{}: unexpected `include` entry {other:?}",
                compose_file.display()
            ),
        }
    }
    out
}

/// bd:mech-crate-eic — a recipe that `include:`s a sibling compose file must
/// ship it. The astro recipe included `db.yml`/`redis.yml` with `optional: true`
/// and shipped neither, and because `optional` is not a Compose field the key was
/// ignored: every astro service scaffolded without db+redis siblings failed
/// `make dev` outright with "open …/docker/compose/db.yml: no such file".
#[test]
fn every_recipe_include_resolves_to_a_file_the_recipe_ships() {
    let mut missing: Vec<String> = Vec::new();
    let mut checked = 0;

    for name in &shipped_recipe_names() {
        let (project, _) = install_into_tempdir(name, "svc", &HashMap::new());
        for compose in dev_compose_context(project.path()) {
            for (declared, resolved) in include_targets(&compose) {
                checked += 1;
                if !resolved.is_file() {
                    missing.push(format!(
                        "{name}: docker/compose/{} includes `{declared}` which the recipe does not ship",
                        compose.file_name().unwrap().to_string_lossy(),
                    ));
                }
            }
        }
    }

    assert!(
        checked >= 2,
        "setup: expected the shipped recipes to declare compose includes, saw {checked}"
    );
    assert!(
        missing.is_empty(),
        "unresolvable includes:\n{}",
        missing.join("\n")
    );
}

/// bd:mech-crate-pos — project-side env config lives in `docker/.config/` (with
/// the dot). The rust-api and rust-worker recipes mapped their service env file
/// to `docker/config/.env.<svc>` and declared the dotless directory, so every
/// `mx add` left a stray `docker/config/` tree that nothing reads.
#[test]
fn no_recipe_writes_into_a_dotless_docker_config_dir() {
    let mut offenders: Vec<String> = Vec::new();

    for name in &shipped_recipe_names() {
        let rj = recipes_root().join(name).join("recipe.json");
        let raw: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&rj).unwrap()).unwrap();

        let dirs = raw["directories"].as_array().cloned().unwrap_or_default();
        for d in dirs.iter().filter_map(|d| d.as_str()) {
            if d == "docker/config" || d.starts_with("docker/config/") {
                offenders.push(format!("{name}: directories[] declares `{d}`"));
            }
        }
        let templates = raw["templates"].as_array().cloned().unwrap_or_default();
        for to in templates.iter().filter_map(|t| t["to"].as_str()) {
            if to.starts_with("docker/config/") {
                offenders.push(format!("{name}: templates[].to writes `{to}`"));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "project-side env config lives in `docker/.config/` (dotted); \
         these write the dotless path instead:\n{}",
        offenders.join("\n")
    );
}

/// Is a usable `docker compose` on PATH? The config check below is a real
/// compose invocation, so it self-skips where there is none (CI containers,
/// sandboxes) rather than failing for the wrong reason.
fn compose_available() -> bool {
    std::process::Command::new("docker")
        .args(["compose", "version"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// The regression net for the whole class: straight after `mx add`, the compose
/// context `make dev` builds must parse for real. `docker compose config` is
/// client-side — it resolves `include:`s, `env_file:`s and the merge of the dev
/// overrides without touching the daemon — so it catches a missing include target
/// or env file that the installer round-trip (which only asserts files appear)
/// cannot see.
#[test]
fn every_recipe_yields_a_dev_compose_context_that_validates() {
    if !compose_available() {
        eprintln!("skipping: no usable `docker compose` on PATH");
        return;
    }

    let mut failures: Vec<String> = Vec::new();

    for name in &shipped_recipe_names() {
        let (project, _) = install_into_tempdir(name, "svc", &HashMap::new());
        complete_project_like_mx_new(project.path());

        let files = dev_compose_context(project.path());
        assert!(!files.is_empty(), "{name}: no compose files to validate");

        let mut cmd = std::process::Command::new("docker");
        cmd.current_dir(project.path())
            // Never adopt another stack: this only parses, but pin the name anyway.
            .args(["compose", "-p", "mx-conformance"]);
        for f in &files {
            cmd.arg("-f").arg(f);
        }
        let out = cmd
            .arg("config")
            .arg("--quiet")
            .output()
            .expect("setup: run docker compose config");

        if !out.status.success() {
            failures.push(format!(
                "{name}: `docker compose config` failed ({}):\n{}",
                out.status,
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "the compose context `make dev` assembles does not validate:\n{}",
        failures.join("\n\n")
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
