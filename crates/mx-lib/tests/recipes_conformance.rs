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
///
/// Since bd:mech-crate-298 the rule reads the hostname through the environment
/// so a second stack can move off a shared hostname, which puts the expanded
/// domain in the DEFAULT position: ``Host(`${API_ROUTER_HOST:-api.localhost}`)``.
/// The invariant is unchanged (the option's value reaches the Host rule fully
/// expanded); the assertion pins the whole shape, including the override, so
/// neither half can regress.
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

        let rule = format!(
            "Host(`${{{}_ROUTER_HOST:-{service}.localhost}}`)",
            service.to_uppercase()
        );
        assert!(
            text.contains(&rule),
            "{recipe}: expected {rule} in {}:\n{text}",
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
/// pass `--domain '{{SERVICE_NAME}}.example.com'` and expect it resolved, still
/// in the default position of the `_ROUTER_HOST` override (bd:mech-crate-298).
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
        text.contains("Host(`${API_ROUTER_HOST:-api.example.com}`)"),
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

// ── Build targets resolve to real Dockerfile stages (bd:mech-crate-47j) ──────

/// Stage names declared by a Dockerfile: the `AS <name>` of every `FROM`.
fn dockerfile_stages(path: &Path) -> Vec<String> {
    let body = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("setup: read {}: {e}", path.display()));
    body.lines()
        .filter_map(|l| {
            let mut words = l.split_whitespace();
            if !words.next()?.eq_ignore_ascii_case("from") {
                return None;
            }
            // `FROM <image> AS <stage>` — find the AS keyword, take what follows.
            while let Some(w) = words.next() {
                if w.eq_ignore_ascii_case("as") {
                    return words.next().map(str::to_string);
                }
            }
            None
        })
        .collect()
}

/// One `build:` declaration seen in a compose file.
#[derive(Debug, Default, Clone)]
struct BuildDecl {
    /// Absolute path to the referenced Dockerfile, once both context and
    /// dockerfile have been seen (they may arrive from different files).
    dockerfile: Option<PathBuf>,
    /// Every `target:` declared for this service across the compose context,
    /// with the file that declared it.
    targets: Vec<(String, String)>,
}

/// Merge the `build:` blocks the whole compose context declares, per service.
///
/// Compose merges overrides onto the base file, so a dev override may carry a
/// bare `target:` while the base file owns `context:`/`dockerfile:` — both have
/// to be collected before any target can be resolved.
fn build_decls(files: &[PathBuf]) -> HashMap<String, BuildDecl> {
    let mut decls: HashMap<String, BuildDecl> = HashMap::new();

    for file in files {
        let body = std::fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("setup: read {}: {e}", file.display()));
        let doc: serde_yaml::Value = serde_yaml::from_str(&body)
            .unwrap_or_else(|e| panic!("{}: not valid YAML: {e}", file.display()));
        let dir = file.parent().unwrap();
        let label = file.file_name().unwrap().to_string_lossy().to_string();

        let Some(services) = doc.get("services").and_then(|v| v.as_mapping()) else {
            continue;
        };
        for (name, spec) in services {
            let Some(name) = name.as_str() else { continue };
            let Some(build) = spec.get("build") else {
                continue;
            };
            let entry = decls.entry(name.to_string()).or_default();

            // `dockerfile` resolves relative to `context`, which itself resolves
            // relative to the compose file's directory.
            if let Some(dockerfile) = build.get("dockerfile").and_then(|v| v.as_str()) {
                let context = build.get("context").and_then(|v| v.as_str()).unwrap_or(".");
                entry.dockerfile = Some(dir.join(context).join(dockerfile));
            }
            if let Some(target) = build.get("target").and_then(|v| v.as_str()) {
                entry.targets.push((target.to_string(), label.clone()));
            }
        }
    }

    decls
}

/// bd:mech-crate-47j — the astro recipe's base compose asked for `target: runner`
/// and the Dockerfile it builds defines base/deps/builder/development/production.
/// Nothing caught it: `docker compose config` resolves the merge without reading
/// the Dockerfile, and `make dev`'s override swaps in `development`, which exists
/// — so every non-dev path (`make up`, `make build`, `make release`) failed on a
/// build target that named no stage, and only astro's dev path was ever exercised.
///
/// The net for the class: every `target:` any shipped compose file declares must
/// name a stage its Dockerfile actually defines.
#[test]
fn every_compose_build_target_names_a_real_dockerfile_stage() {
    let mut failures: Vec<String> = Vec::new();
    let mut checked = 0;

    for name in &shipped_recipe_names() {
        let (project, _) = install_into_tempdir(name, "svc", &HashMap::new());
        let files = dev_compose_context(project.path());

        for (service, decl) in build_decls(&files) {
            if decl.targets.is_empty() {
                continue;
            }
            let Some(dockerfile) = &decl.dockerfile else {
                failures.push(format!(
                    "{name}: service `{service}` declares build targets {:?} but no \
                     compose file in the context declares a dockerfile",
                    decl.targets
                ));
                continue;
            };
            if !dockerfile.is_file() {
                failures.push(format!(
                    "{name}: service `{service}` builds {} which the recipe does not ship",
                    dockerfile.display()
                ));
                continue;
            }
            let stages = dockerfile_stages(dockerfile);
            for (target, from) in &decl.targets {
                checked += 1;
                if !stages.contains(target) {
                    failures.push(format!(
                        "{name}: docker/compose/{from} builds service `{service}` with \
                         `target: {target}`, but its Dockerfile defines only {stages:?}"
                    ));
                }
            }
        }
    }

    assert!(
        checked >= 7,
        "setup: expected every recipe to declare a build target, saw {checked}"
    );
    assert!(
        failures.is_empty(),
        "build targets that name no Dockerfile stage:\n{}",
        failures.join("\n")
    );
}

// ── No dead payload (bd:mech-crate-874) ──────────────────────────────────────

/// Recipe-relative files that `recipe.json` never maps, and that we have
/// deliberately not deleted yet — each one is dead weight that lands nowhere.
///
/// The astro recipe used to account for 40 of these: a whole second app (Vue
/// components, Pinia stores, drizzle schema, its own `package.json`) that
/// `recipe.json` mapped not one line of, while the single file it *did* map
/// imported `@/lib/db` from it. The design call (bd:mech-crate-874) is that the
/// framework scaffolder owns `package.json` and the app payload, and the recipe
/// owns infra plus a dependency-free health endpoint — so astro's payload is
/// gone and astro must never appear in this list again.
///
/// nuxt carried the same shape (11 files: a Nuxt 3 starter app with its own
/// `package.json`, lockfile, Tailwind config and pages) until bd:mech-crate-whj
/// applied the same call: `nuxi init` owns the app, the recipe owns infra plus
/// `server/api/health.get.ts`. The list is now empty.
///
/// This list may only shrink. A new entry means a recipe grew dead payload; a
/// stale entry (listed but no longer present) fails too, so deleting the files
/// forces the allowance out with them.
const UNMAPPED_PAYLOAD_ALLOWED: &[&str] = &[];

/// Every file a recipe ships must either be mapped by `recipe.json` (directly or
/// under a mapped directory) or be `recipe.json` itself. Anything else never
/// reaches a project: it is payload the author believes ships and does not.
#[test]
fn no_recipe_ships_payload_files_that_nothing_maps() {
    let mut unmapped: Vec<String> = Vec::new();

    for name in &shipped_recipe_names() {
        let dir = recipes_root().join(name);
        let raw: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("recipe.json")).unwrap())
                .unwrap();

        // `common://` sources live outside the recipe directory.
        let mapped: Vec<String> = raw["templates"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|t| t["from"].as_str())
            .filter(|from| !from.contains("://"))
            .map(|from| from.trim_end_matches('/').to_string())
            .collect();

        let mut files: Vec<PathBuf> = walk_files(&dir);
        files.sort();
        for file in files {
            let rel = file
                .strip_prefix(&dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if rel == "recipe.json" {
                continue;
            }
            let accounted = mapped
                .iter()
                .any(|m| rel == *m || rel.starts_with(&format!("{m}/")));
            if !accounted {
                unmapped.push(format!("{name}/{rel}"));
            }
        }
    }

    let unexpected: Vec<&String> = unmapped
        .iter()
        .filter(|u| !UNMAPPED_PAYLOAD_ALLOWED.contains(&u.as_str()))
        .collect();
    assert!(
        unexpected.is_empty(),
        "these recipe files are mapped by no `recipe.json` template, so they land \
         nowhere — map them or delete them:\n{}",
        unexpected
            .iter()
            .map(|u| format!("  {u}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let stale: Vec<&&str> = UNMAPPED_PAYLOAD_ALLOWED
        .iter()
        .filter(|a| !unmapped.iter().any(|u| u == *a))
        .collect();
    assert!(
        stale.is_empty(),
        "UNMAPPED_PAYLOAD_ALLOWED lists files that are no longer unmapped — drop \
         them from the list:\n{}",
        stale
            .iter()
            .map(|s| format!("  {s}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// A recipe whose app comes from a framework scaffolder ships no lockfile: the
/// scaffolder owns `package.json`, and every shipped `init_cmd` runs with
/// `--no-install`, so a fresh app has no `package-lock.json` until someone runs
/// `npm install`. A bare `RUN npm ci` refuses to run without one, which made
/// nuxt's non-dev build fail on every freshly scaffolded app (bd:mech-crate-whj).
/// `npm ci` is fine behind a lockfile check (astro's
/// `elif [ -f package-lock.json ]; then npm ci; else npm install`).
#[test]
fn no_scaffolder_recipe_runs_a_bare_npm_ci() {
    let names = recipes_with_a_scaffolder();
    assert!(
        names.iter().any(|n| n == "nuxt"),
        "setup: expected nuxt among the scaffolder recipes, found {names:?}"
    );

    let mut offenders: Vec<String> = Vec::new();
    for name in &names {
        let dir = recipes_root().join(name).join("docker/dockerfiles");
        if !dir.is_dir() {
            continue;
        }
        for file in walk_files(&dir) {
            let body = std::fs::read_to_string(&file)
                .unwrap_or_else(|e| panic!("setup: read {}: {e}", file.display()));
            for (i, line) in body.lines().enumerate() {
                let t = line.trim();
                if t.starts_with("RUN ")
                    && t.split_whitespace().nth(1) == Some("npm")
                    && t.split_whitespace().nth(2) == Some("ci")
                {
                    offenders.push(format!(
                        "  {name}/{}:{}: {t}",
                        file.strip_prefix(recipes_root().join(name))
                            .unwrap()
                            .display(),
                        i + 1
                    ));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "these scaffolder recipes run `npm ci` unconditionally, but a fresh \
         scaffold has no package-lock.json, so the build fails:\n{}",
        offenders.join("\n")
    );
}

/// Every file under `dir`, recursively.
fn walk_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d)
            .unwrap_or_else(|e| panic!("setup: read {}: {e}", d.display()))
            .flatten()
        {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    out
}

// ── No recipe ships a {{PLACEHOLDER}} nothing substitutes ────────────────────

/// The `{{NAME}}` tokens the installer owns are written in SCREAMING_SNAKE_CASE,
/// which is exactly what separates them from the foreign `{{ }}` syntax recipe
/// app sources carry (`{{ page.title }}`, `{{ config.base_url | safe }}`,
/// `{{SERVICE_NAME.charAt(0)}}`). A brace-wrapped bare identifier in that case is
/// a promise that `recipe.json` declares the name; anything else is another
/// renderer's business and must reach the project verbatim.
///
/// Mirrors [`expand_placeholders`]'s tag grammar: `{{`, optional `-`, optional
/// whitespace, identifier, optional whitespace, optional `-`, `}}`.
fn placeholder_tokens(body: &str) -> Vec<String> {
    let bytes = body.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 1 < bytes.len() {
        if bytes[i] != b'{' || bytes[i + 1] != b'{' {
            i += 1;
            continue;
        }
        let mut j = i + 2;
        if bytes.get(j) == Some(&b'-') {
            j += 1;
        }
        while bytes.get(j).is_some_and(u8::is_ascii_whitespace) {
            j += 1;
        }
        let name_start = j;
        while bytes
            .get(j)
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
        {
            j += 1;
        }
        let name = &body[name_start..j];
        while bytes.get(j).is_some_and(u8::is_ascii_whitespace) {
            j += 1;
        }
        if bytes.get(j) == Some(&b'-') {
            j += 1;
        }
        let closed = bytes.get(j) == Some(&b'}') && bytes.get(j + 1) == Some(&b'}');
        let screaming = !name.is_empty()
            && name.starts_with(|c: char| c.is_ascii_uppercase())
            && name
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
        if closed && screaming {
            out.push(name.to_string());
        }
        i += 2;
    }
    out
}

/// The placeholder names a recipe's `recipe.json` declares.
fn declared_placeholders(recipe_name: &str) -> Vec<String> {
    let rj = recipes_root().join(recipe_name).join("recipe.json");
    let raw: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&rj)
            .unwrap_or_else(|e| panic!("setup: read {}: {e}", rj.display())),
    )
    .unwrap_or_else(|e| panic!("{}: invalid JSON: {e}", rj.display()));
    let mut names: Vec<String> = raw["placeholders"]
        .as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();
    // The installer seeds SERVICE_NAME whether or not the recipe names it.
    names.push("SERVICE_NAME".to_string());
    names.sort();
    names.dedup();
    names
}

/// bd:mech-crate-fq3 — rust-worker's Dockerfiles said `FROM rust:{{RUST_VERSION}}`
/// while its `recipe.json` declared no such placeholder, so the token shipped
/// through to the project and `docker build` died on
/// `failed to parse stage name "rust:{{RUST_VERSION}}-alpine": invalid reference
/// format`. The recipe's build path had never been exercised.
///
/// The net for the whole class, in two halves: statically, every SCREAMING_SNAKE
/// token a recipe ships must be declared by that recipe (and one under `common/`
/// by every recipe, since any of them may map it); behaviorally, a default
/// install must leave none of them behind in the generated project.
#[test]
fn no_shipped_recipe_leaves_a_placeholder_token_unsubstituted() {
    let names = shipped_recipe_names();
    assert!(
        names.len() >= 7,
        "expected >=7 recipes, found {}",
        names.len()
    );

    // Static half: the token is declared where it is written.
    let mut offenders: Vec<String> = Vec::new();
    let mut seen = 0usize;
    for name in &names {
        let declared = declared_placeholders(name);
        let dir = recipes_root().join(name);
        let mut files = walk_files(&dir);
        files.sort();
        for file in files {
            let Ok(body) = std::fs::read_to_string(&file) else {
                continue; // binary payload: the installer copies it verbatim
            };
            let rel = file.strip_prefix(&dir).unwrap().display().to_string();
            let mut missing: Vec<String> = placeholder_tokens(&body)
                .into_iter()
                .inspect(|_| seen += 1)
                .filter(|t| !declared.contains(t))
                .collect();
            missing.sort();
            missing.dedup();
            for token in missing {
                offenders.push(format!(
                    "  {name}/{rel}: {{{{{token}}}}} is not declared in \
                     {name}/recipe.json placeholders"
                ));
            }
        }
    }

    // `common/` has no recipe.json of its own; whichever recipe maps one of its
    // files supplies the values, so every recipe has to declare the name.
    let common = recipes_root().join("common");
    let mut common_files = walk_files(&common);
    common_files.sort();
    for file in common_files {
        let Ok(body) = std::fs::read_to_string(&file) else {
            continue;
        };
        let rel = file.strip_prefix(&common).unwrap().display().to_string();
        let mut tokens = placeholder_tokens(&body);
        tokens.sort();
        tokens.dedup();
        for token in tokens {
            seen += 1;
            let undeclared: Vec<&String> = names
                .iter()
                .filter(|n| !declared_placeholders(n).contains(&token))
                .collect();
            if !undeclared.is_empty() {
                offenders.push(format!(
                    "  common/{rel}: {{{{{token}}}}} is undeclared by {}",
                    undeclared
                        .iter()
                        .map(|n| n.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
    }

    assert!(
        seen > 0,
        "setup: expected the recipes to still use {{{{PLACEHOLDER}}}} tokens"
    );
    assert!(
        offenders.is_empty(),
        "these shipped recipe files carry a {{{{PLACEHOLDER}}}} token that nothing \
         substitutes, so it reaches the generated project literally:\n{}",
        offenders.join("\n")
    );

    // Behavioral half: a default install leaves no token behind.
    let mut leftovers: Vec<String> = Vec::new();
    for name in &names {
        let (project, _) = install_into_tempdir(name, "api", &HashMap::new());
        let root = project.path();
        let mut files = walk_files(root);
        files.sort();
        for file in files {
            let Ok(body) = std::fs::read_to_string(&file) else {
                continue;
            };
            let mut tokens = placeholder_tokens(&body);
            tokens.sort();
            tokens.dedup();
            for token in tokens {
                leftovers.push(format!(
                    "  {name}: {} still has {{{{{token}}}}} after install",
                    file.strip_prefix(root).unwrap().display()
                ));
            }
        }
    }
    assert!(
        leftovers.is_empty(),
        "a default `mx add` left placeholder tokens in the generated project:\n{}",
        leftovers.join("\n")
    );
}

// ── No infra template ships a {{PLACEHOLDER}} nothing substitutes ────────────

/// The placeholder names `mx new --infra <provider>` substitutes into the copied
/// `templates/infra/<provider>/` tree. Mirrors `setup_cloudflare_infra` in
/// `crates/mx-cli/src/commands/new.rs`, which expands exactly one name.
const INFRA_SUBSTITUTED_PLACEHOLDERS: &[&str] = &["PROJECT_NAME"];

/// File extensions that substitution pass rewrites. A token in any other file
/// is copied through literally.
const INFRA_SUBSTITUTED_EXTENSIONS: &[&str] = &["ts", "toml", "json", "md"];

/// The `templates/infra/<provider>/` trees `mx new --infra` actually installs.
const INFRA_INSTALLED_PROVIDERS: &[&str] = &["cloudflare"];

/// bd:mech-crate-pdr: `templates/infra/` sits outside `templates/recipes/`, so
/// the recipe net above never looked at its `{{PROJECT_NAME}}` tokens. This is
/// the static half of the same net for infra templates: every SCREAMING_SNAKE
/// token must be one `mx new` substitutes, in a file the substitution pass
/// rewrites, under a provider `mx new` installs. The behavioral half
/// (`mx new --no-prompt --infra cloudflare` leaves zero tokens) lives in
/// `crates/mx-cli/tests/new_infra_placeholders.rs`, since it needs the binary.
#[test]
fn no_shipped_infra_template_leaves_a_placeholder_token_unsubstituted() {
    let infra = templates_root().join("infra");
    let mut files = walk_files(&infra);
    files.sort();

    let mut offenders: Vec<String> = Vec::new();
    let mut seen = 0usize;
    for file in files {
        let Ok(body) = std::fs::read_to_string(&file) else {
            continue;
        };
        let mut tokens = placeholder_tokens(&body);
        tokens.sort();
        tokens.dedup();
        if tokens.is_empty() {
            continue;
        }
        let rel = file.strip_prefix(&infra).unwrap();
        let rel_s = rel.display().to_string();
        let provider = rel
            .components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .unwrap_or_default();
        let ext = file
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        for token in tokens {
            seen += 1;
            if !INFRA_INSTALLED_PROVIDERS.contains(&provider.as_str()) {
                offenders.push(format!(
                    "  infra/{rel_s}: {{{{{token}}}}} sits under provider `{provider}`, \
                     which `mx new --infra` does not install or substitute"
                ));
            } else if !INFRA_SUBSTITUTED_PLACEHOLDERS.contains(&token.as_str()) {
                offenders.push(format!(
                    "  infra/{rel_s}: {{{{{token}}}}} is not a name `mx new --infra` \
                     substitutes (only {INFRA_SUBSTITUTED_PLACEHOLDERS:?})"
                ));
            } else if !INFRA_SUBSTITUTED_EXTENSIONS.contains(&ext.as_str()) {
                offenders.push(format!(
                    "  infra/{rel_s}: {{{{{token}}}}} is in a file the substitution \
                     pass skips (extension `{ext}`, rewritten: \
                     {INFRA_SUBSTITUTED_EXTENSIONS:?})"
                ));
            }
        }
    }

    assert!(
        seen > 0,
        "setup: expected templates/infra to still use {{{{PLACEHOLDER}}}} tokens"
    );
    assert!(
        offenders.is_empty(),
        "these shipped infra template files carry a {{{{PLACEHOLDER}}}} token that \
         `mx new --infra` does not substitute, so it reaches the project literally:\n{}",
        offenders.join("\n")
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

    // Since bd:mech-crate-whj the recipe ships no package.json at all: `nuxi init`
    // owns it (the bd:mech-crate-874 call). A missing file declares no scripts,
    // so this stays red for the same reason until bd:mech-crate-ten decides
    // where release scripts come from.
    let pkg_path = repo_root.join("templates/recipes/nuxt/app/package.json");
    let pkg: serde_json::Value = match std::fs::read_to_string(&pkg_path) {
        Ok(body) => serde_json::from_str(&body).expect("setup: parse nuxt app package.json"),
        Err(_) => serde_json::json!({ "scripts": {} }),
    };
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

// ── zola builds on every host arch (bd:mech-crate-b15) ───────────────────────

/// `FROM` base and body of each Dockerfile stage, in file order.
fn dockerfile_stage_bodies(body: &str) -> Vec<(String, String)> {
    let mut stages: Vec<(String, String)> = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.to_ascii_uppercase().starts_with("FROM ") {
            let base = trimmed[5..]
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
            stages.push((base, String::new()));
        } else if let Some((_, stage_body)) = stages.last_mut() {
            stage_body.push_str(line);
            stage_body.push('\n');
        }
    }
    stages
}

/// `(major, minor, patch)` of a plain `x.y.z` version string.
fn semver(v: &str) -> (u64, u64, u64) {
    let parts: Vec<u64> = v
        .split('.')
        .map(|p| {
            p.parse()
                .unwrap_or_else(|e| panic!("setup: `{v}` is not x.y.z: {e}"))
        })
        .collect();
    assert_eq!(parts.len(), 3, "setup: `{v}` is not x.y.z");
    (parts[0], parts[1], parts[2])
}

/// Zola publishes Linux binaries only as glibc (`*-unknown-linux-gnu`) builds,
/// and none for aarch64 before v0.20.0. The recipe used to fetch the gnu
/// tarball into Alpine (musl, where a glibc binary cannot run) at v0.19.2 (no
/// aarch64 asset at all), so `docker compose build` died at `tar xz` on Apple
/// Silicon. Every stage that downloads zola must be glibc-based, map both host
/// arches to a published triple, fail loudly on a missing asset, and the default
/// version must be one that ships an aarch64 Linux build.
#[test]
fn zola_dockerfile_fetches_a_zola_build_that_exists_and_runs_on_both_arches() {
    let (project, _) = install_into_tempdir("zola", "blog", &HashMap::new());
    let dockerfile = std::fs::read_to_string(project.path().join("docker/dockerfiles/blog/app"))
        .expect("setup: zola installs docker/dockerfiles/blog/app");

    let downloads: Vec<(String, String)> = dockerfile_stage_bodies(&dockerfile)
        .into_iter()
        .filter(|(_, body)| body.contains("getzola/zola/releases/download"))
        .collect();
    assert!(
        !downloads.is_empty(),
        "setup: the zola Dockerfile downloads zola in no stage:\n{dockerfile}"
    );
    for (base, body) in &downloads {
        assert!(
            !base.starts_with("alpine"),
            "zola: stage FROM {base} installs a glibc zola build on musl, where it cannot run"
        );
        for triple in ["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"] {
            assert!(
                body.contains(triple),
                "zola: stage FROM {base} never maps a host arch to `{triple}`:\n{body}"
            );
        }
        assert!(
            !body.contains("linux-musl"),
            "zola: stage FROM {base} names a musl triple zola does not publish for every arch"
        );
        assert!(
            body.contains("curl -fsSL"),
            "zola: stage FROM {base} lets a missing release asset reach `tar` as an HTML body"
        );
    }

    let raw: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(recipes_root().join("zola/recipe.json")).unwrap(),
    )
    .unwrap();
    let default = raw["options"]["zola_version"]["default"]
        .as_str()
        .expect("setup: zola declares a zola_version default");
    assert!(
        semver(default) >= (0, 20, 0),
        "zola: default zola_version {default} has no aarch64 Linux release asset (first in 0.20.0)"
    );

    // The build context is the project root: the site is apps/<svc>, and
    // `zola build` over the whole project finds no config.toml.
    assert!(
        dockerfile.contains("COPY apps/blog/ ."),
        "zola: the builder stage does not copy the site from apps/blog:\n{dockerfile}"
    );
}

/// The site payload itself must be one `zola build` accepts: a `taxonomies`
/// key written after `[slugify]` lands inside that table (zola rejects the
/// duplicate key), a selector list that runs into `@media` is a Sass syntax
/// error, and `current_path` is undefined while 404.html renders.
#[test]
fn zola_site_payload_is_one_zola_build_accepts() {
    let (project, _) = install_into_tempdir("zola", "blog", &HashMap::new());
    let site = project.path().join("apps/blog");

    let config: toml::Value = toml::from_str(
        &std::fs::read_to_string(site.join("config.toml")).expect("setup: read config.toml"),
    )
    .unwrap_or_else(|e| panic!("zola: config.toml is not valid TOML: {e}"));
    assert!(
        config.get("taxonomies").is_some_and(|t| t.is_array()),
        "zola: config.toml declares no top-level `taxonomies` array"
    );

    let components = std::fs::read_to_string(site.join("sass/_components.scss"))
        .expect("setup: read sass/_components.scss");
    let squashed: String = components.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        !squashed.contains(", @media"),
        "zola: _components.scss continues a selector list into an @media rule"
    );

    let header = std::fs::read_to_string(site.join("templates/partials/header.html"))
        .expect("setup: read templates/partials/header.html");
    for (i, line) in header.lines().enumerate() {
        if line.contains("current_path") && !line.trim_start().starts_with("{#") {
            assert!(
                line.contains("current_path | default("),
                "zola: header.html:{} reads current_path with no default, which 404.html \
                 does not define:\n{line}",
                i + 1
            );
        }
    }
}

// ── Declared post_install steps take effect (bd:mech-crate-99b) ──────────────

/// zola declared `post_install.rename` of `apps/<svc>/gitignore.template` to
/// `.gitignore`, but no template mapped the file, so the rename found nothing
/// to move and every scaffolded zola site shipped without a `.gitignore`.
#[test]
fn zola_scaffold_ships_its_gitignore() {
    let (project, _) = install_into_tempdir("zola", "blog", &HashMap::new());
    let app = project.path().join("apps/blog");

    let shipped = std::fs::read_to_string(recipes_root().join("zola/app/gitignore.template"))
        .expect("setup: zola ships app/gitignore.template");
    let installed = std::fs::read_to_string(app.join(".gitignore"))
        .unwrap_or_else(|e| panic!("zola: apps/blog/.gitignore was not installed: {e}"));
    assert_eq!(
        installed, shipped,
        "zola: apps/blog/.gitignore does not carry the recipe's gitignore.template"
    );
    assert!(
        installed.lines().any(|l| l.trim() == "public/"),
        "zola: the installed .gitignore does not ignore zola's build output"
    );
    assert!(
        !app.join("gitignore.template").exists(),
        "zola: gitignore.template was left behind instead of renamed"
    );
}

/// The class net: the installer skips a `rename` or `chmod` whose path does not
/// exist, so a step whose source nothing installs is a silent no-op. Every
/// declared step of every shipped recipe must take effect on a clean install.
#[test]
fn every_declared_post_install_step_takes_effect() {
    let mut failures: Vec<String> = Vec::new();
    let mut checked = 0;

    for name in &shipped_recipe_names() {
        let raw: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(recipes_root().join(name).join("recipe.json")).unwrap(),
        )
        .unwrap();
        let post = &raw["post_install"];
        let renames = post["rename"].as_array().cloned().unwrap_or_default();
        let chmods = post["chmod"].as_array().cloned().unwrap_or_default();
        if renames.is_empty() && chmods.is_empty() {
            continue;
        }

        let (project, _) = install_into_tempdir(name, "svc", &HashMap::new());
        let at = |p: &str| project.path().join(p);

        for r in &renames {
            checked += 1;
            let (from, to) = (r["from"].as_str().unwrap(), r["to"].as_str().unwrap());
            let (from, to) = (
                &from.replace("{{SERVICE_NAME}}", "svc"),
                &to.replace("{{SERVICE_NAME}}", "svc"),
            );
            if !at(to).is_file() {
                failures.push(format!("{name}: rename {from} -> {to} left no {to}"));
            }
            if at(from).exists() {
                failures.push(format!("{name}: rename {from} -> {to} left {from} behind"));
            }
        }
        for c in &chmods {
            checked += 1;
            let path = &c["path"]
                .as_str()
                .unwrap()
                .replace("{{SERVICE_NAME}}", "svc");
            match std::fs::metadata(at(path)) {
                Err(_) => failures.push(format!("{name}: chmod of {path}, which nothing installs")),
                #[cfg(unix)]
                Ok(meta) => {
                    use std::os::unix::fs::PermissionsExt;
                    if meta.permissions().mode() & 0o111 == 0 {
                        failures.push(format!("{name}: chmod left {path} non-executable"));
                    }
                }
                #[cfg(not(unix))]
                Ok(_) => {}
            }
        }
    }

    assert!(
        checked >= 4,
        "setup: expected the laravel and zola rename/chmod steps, checked {checked}"
    );
    assert!(
        failures.is_empty(),
        "declared post_install steps that silently did nothing:\n{}",
        failures.join("\n")
    );
}
