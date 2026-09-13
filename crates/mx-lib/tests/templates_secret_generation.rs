//! Generated dev credentials (bd:mech-crate-rqc).
//!
//! A fresh `mx new` + `mx add <svc> --recipe <db-bearing>` could not `make dev`:
//! `scripts/init.sh` copied `.env.secrets.template` verbatim (empty
//! `DB_USER`/`DB_PASSWORD`/`DB_NAME`) and the recipes shipped
//! `__GENERATE_DB_PASSWORD__` placeholders that nothing replaced, so postgres
//! refused to initialize with an empty `POSTGRES_PASSWORD`.
//!
//! Two separate defects hide in that one symptom, and both are held here:
//!
//!   1. Nothing generated the values. `templates/scripts/generate-secrets.sh` is
//!      now the single shared generator, invoked by `scripts/init.sh` (which
//!      `make dev` runs on every start), so a recipe added *after* `mx new`
//!      still gets real credentials.
//!   2. The env files referenced each other with `${VAR}`. The compose CLI
//!      interpolates `${…}` inside `env_file` values from the compose
//!      *project directory* `.env` — never from a sibling env file — so every
//!      such reference arrived empty in the container. The generator resolves
//!      them to literals.
//!
//! The contract, asserted below for every db-bearing recipe: after the
//! generator runs, no file under `docker/.config/` carries a `__GENERATE_*__`
//! placeholder, an unresolved `${…}` reference, or an empty credential.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn templates_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates")
}

fn scripts_root() -> PathBuf {
    templates_root().join("scripts")
}

fn recipes_root() -> PathBuf {
    templates_root().join("recipes")
}

/// Credentials the shipped stack deliberately leaves blank: the bundled redis
/// runs `redis-server` without `--requirepass`, so a generated password would
/// only break every client that builds `redis://:$REDIS_PASSWORD@…` against a
/// server that wants none.
const BLANK_BY_DESIGN: &[&str] = &["REDIS_PASSWORD"];

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

// ── Fixtures ─────────────────────────────────────────────────────────────────

/// Lay down what `mx new` puts on disk: the shipped top-level scripts and the
/// baseline `docker/.config/` env files. Deliberately *not* `.env.secrets` —
/// creating that from the template is `init.sh`'s job, and this suite exercises
/// it.
fn scaffold_like_mx_new(root: &Path) {
    for dir in ["scripts", "docker/compose", "docker/.config", "tmp/up"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    for entry in std::fs::read_dir(scripts_root()).unwrap().flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let dest = root.join("scripts").join(path.file_name().unwrap());
        std::fs::copy(&path, &dest).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
    let config = templates_root().join("docker/config");
    std::fs::copy(
        config.join("env.shared"),
        root.join("docker/.config/.env.shared"),
    )
    .unwrap();
    std::fs::copy(
        config.join("env.secrets.template"),
        root.join("docker/.config/.env.secrets.template"),
    )
    .unwrap();
}

/// `mx add <service> --recipe <name>` over an existing project tree, with the
/// framework scaffolder stubbed out so the suite never shells out to the network.
fn install_recipe(project: &Path, recipe_name: &str, service: &str) {
    let mut installer = mx_lib::recipe::RecipeInstaller::new(templates_root())
        .expect("setup: build a recipe installer");
    let recipe = installer
        .load_recipe(recipe_name)
        .unwrap_or_else(|e| panic!("setup: load recipe {recipe_name}: {e}"));

    let mut options: HashMap<String, String> = HashMap::new();
    if recipe.init_app.is_some() {
        // Runs in the parent of the scaffold target and must leave it non-empty,
        // the same contract the real framework scaffolders meet.
        options.insert(
            "init_cmd".to_string(),
            "mkdir -p {{SERVICE_NAME}} && printf 'stub scaffolder\\n' > {{SERVICE_NAME}}/.mx-stub"
                .to_string(),
        );
    }

    installer
        .install(&recipe, project, service, &options)
        .unwrap_or_else(|e| panic!("{recipe_name}: install failed: {e}"));
}

/// Run a bash snippet in `cwd`, returning `(exit ok, stdout+stderr)`.
fn bash_in(cwd: &Path, script: &str) -> (bool, String) {
    let out = std::process::Command::new("bash")
        .arg("-c")
        .arg(script)
        .current_dir(cwd)
        .env_remove("COMPOSE_PROJECT_NAME")
        .output()
        .expect("run bash");
    let mut combined = String::from_utf8_lossy(&out.stdout).to_string();
    combined.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), combined)
}

/// Run the shipped generator, panicking with its output if it fails.
fn run_generator(root: &Path) -> String {
    let (ok, output) = bash_in(root, "./scripts/generate-secrets.sh");
    assert!(ok, "scripts/generate-secrets.sh failed:\n{output}");
    output
}

/// The env files compose actually loads: `docker/.config/.env.*`, templates excluded.
fn installed_env_files(root: &Path) -> Vec<PathBuf> {
    let dir = root.join("docker/.config");
    let mut out: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("setup: read {}: {e}", dir.display()))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| {
            let n = p.file_name().unwrap().to_string_lossy();
            n.starts_with(".env.") && !n.ends_with(".template")
        })
        .collect();
    out.sort();
    out
}

/// `KEY=VALUE` pairs from one env file, comments and blanks dropped.
fn parse_env(path: &Path) -> Vec<(String, String)> {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("setup: read {}: {e}", path.display()))
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect()
}

/// Keys in `file` that the shipped rules call a credential but whose value is
/// still empty. Asks `scripts/.bashrc` for the classification rather than
/// restating it, so the test cannot drift from the generator.
fn unfilled_credential_keys(root: &Path, file: &Path) -> Vec<String> {
    let rel = file.strip_prefix(root).unwrap().display().to_string();
    let script = format!(
        r#"set -u
           source ./scripts/.bashrc
           while IFS= read -r line || [ -n "$line" ]; do
               case "$line" in ''|'#'*) continue ;; *=*) ;; *) continue ;; esac
               key="${{line%%=*}}"
               value="${{line#*=}}"
               mech_secret_is_blank_by_design "$key" && continue
               [ -n "$value" ] && continue
               [ "$(mech_secret_kind "$key")" = "none" ] && continue
               printf '%s\n' "$key"
           done < '{rel}'"#
    );
    let (ok, output) = bash_in(root, &script);
    assert!(ok, "classification probe failed:\n{output}");
    output
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

/// Recipes that install a `db` service — the ones whose boot depends on real
/// postgres credentials.
fn db_bearing_recipes() -> Vec<String> {
    shipped_recipe_names()
        .into_iter()
        .filter(|name| {
            let tmp = tempfile::tempdir().unwrap();
            install_recipe(tmp.path(), name, "api");
            tmp.path().join("docker/compose/db.yml").exists()
        })
        .collect()
}

// ── The generator exists and is wired into the path `make dev` takes ─────────

/// A single shared generation point. `make dev` → `scripts/dev.sh` →
/// `scripts/init.sh`, so putting the generator behind `init.sh` is what makes an
/// `mx add` that lands *after* `mx new` still come up with real credentials.
#[test]
fn the_shared_generator_is_shipped_and_init_invokes_it() {
    let generator = scripts_root().join("generate-secrets.sh");
    assert!(
        generator.exists(),
        "templates/scripts/generate-secrets.sh must ship: it is the single point that \
         turns placeholder/empty credentials into real dev values for EVERY recipe"
    );

    let init = std::fs::read_to_string(scripts_root().join("init.sh"))
        .expect("setup: templates/scripts/init.sh must exist");
    assert!(
        init.contains("generate-secrets.sh"),
        "scripts/init.sh must invoke the generator — `make dev` runs init.sh on \
         every start, which is what makes generation re-runnable after `mx add`:\n{init}"
    );

    let dev = std::fs::read_to_string(scripts_root().join("dev.sh"))
        .expect("setup: templates/scripts/dev.sh must exist");
    assert!(
        dev.contains("init.sh"),
        "scripts/dev.sh must run init.sh, otherwise `make dev` never reaches the generator"
    );
}

// ── Every db-bearing recipe initializes unaided ──────────────────────────────

/// The acceptance criterion itself, minus docker: scaffold, add the recipe, run
/// the generator with ZERO hand edits, and assert the env files compose will
/// load carry real literal values.
#[test]
fn every_db_bearing_recipe_gets_real_credentials_unaided() {
    let recipes = db_bearing_recipes();
    assert!(
        recipes.len() >= 4,
        "setup: expected at least 4 db-bearing recipes, found {recipes:?}"
    );

    for recipe in &recipes {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        scaffold_like_mx_new(root);
        install_recipe(root, recipe, "api");
        run_generator(root);

        let secrets = root.join("docker/.config/.env.secrets");
        assert!(
            secrets.exists(),
            "{recipe}: the generator must create docker/.config/.env.secrets from the template"
        );

        // Nothing anywhere under docker/.config/ may still be a placeholder, an
        // unresolved reference, or empty.
        let mut resolved: HashMap<String, String> = HashMap::new();
        for file in installed_env_files(root) {
            let rel = file.strip_prefix(root).unwrap().display().to_string();
            for (key, value) in parse_env(&file) {
                assert!(
                    !value.contains("__GENERATE_"),
                    "{recipe}: {rel}: {key} still carries a placeholder ({value:?}) — \
                     the generator must consume it"
                );
                assert!(
                    !value.contains("${"),
                    "{recipe}: {rel}: {key}={value:?} is a compose-unresolvable reference. \
                     Compose interpolates env_file values from the compose project \
                     directory's .env, never from a sibling env file, so this arrives \
                     EMPTY in the container — the generator must write a literal"
                );
                resolved.insert(key, value);
            }

            // Which empty keys actually matter is the shipped rules' call, not a
            // copy of them here: an optional setting like PUBLIC_SENTRY_DSN is
            // meant to be blank, a credential is not.
            let still_empty = unfilled_credential_keys(root, &file);
            assert!(
                still_empty.is_empty(),
                "{recipe}: {rel}: credentials still empty after generation: {still_empty:?}"
            );
        }

        // The value whose emptiness is what actually stopped postgres booting.
        for key in ["POSTGRES_USER", "POSTGRES_PASSWORD", "POSTGRES_DB"] {
            let value = resolved.get(key).unwrap_or_else(|| {
                panic!(
                    "{recipe}: no {key} anywhere in docker/.config/ — the db service \
                     cannot initialize without it"
                )
            });
            assert!(
                !value.is_empty(),
                "{recipe}: {key} is empty; postgres refuses to initialize"
            );
        }
    }
}

/// Dev credentials must be per-project randoms, never a fixed default shipped in
/// the templates: two scratch projects of the same recipe get different passwords.
#[test]
fn generated_passwords_are_random_per_project() {
    let mut seen: Vec<String> = Vec::new();
    for _ in 0..2 {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        scaffold_like_mx_new(root);
        install_recipe(root, "rust-api", "api");
        run_generator(root);

        let password = parse_env(&root.join("docker/.config/.env.secrets"))
            .into_iter()
            .find(|(k, _)| k == "API_DB_PASSWORD")
            .map(|(_, v)| v)
            .expect("rust-api ships API_DB_PASSWORD in .env.secrets");
        assert!(
            password.len() >= 16,
            "generated password is too short to be a credible dev secret: {password:?}"
        );
        seen.push(password);
    }
    assert_ne!(
        seen[0], seen[1],
        "two projects got the SAME generated password — that is a shipped default, not a generated secret"
    );
}

/// The bundled redis takes no password, so "generate one for every blank" would
/// be a regression dressed as a fix: every client builds
/// `redis://:$REDIS_PASSWORD@redis:6379` and would start failing auth against a
/// server that wants none.
#[test]
fn keys_that_are_blank_by_design_stay_blank() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    scaffold_like_mx_new(root);
    install_recipe(root, "astro", "site");
    run_generator(root);

    let secrets = parse_env(&root.join("docker/.config/.env.secrets"));
    for key in BLANK_BY_DESIGN {
        if let Some((_, value)) = secrets.iter().find(|(k, _)| k == key) {
            assert!(
                value.is_empty(),
                "{key} was generated as {value:?}, but the shipped stack wants it blank"
            );
        }
    }

    // And the rule is documented where both scripts read it from.
    let bashrc = std::fs::read_to_string(scripts_root().join(".bashrc")).unwrap();
    assert!(
        bashrc.contains("MECH_BLANK_BY_DESIGN_KEYS"),
        ".bashrc must own the blank-by-design list so generate-secrets.sh and \
         doctor.sh cannot disagree about whether a project is healthy"
    );
}

/// Re-runnable: `make dev` runs `init.sh` on every start, so the generator must
/// fill only what is still empty or a placeholder and never touch a value a
/// human (or an earlier run) already set.
#[test]
fn generation_is_idempotent_and_never_overwrites_a_real_secret() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    scaffold_like_mx_new(root);
    install_recipe(root, "rust-api", "api");
    run_generator(root);

    let secrets = root.join("docker/.config/.env.secrets");
    let first = std::fs::read_to_string(&secrets).unwrap();

    run_generator(root);
    let second = std::fs::read_to_string(&secrets).unwrap();
    assert_eq!(
        first, second,
        "a second generator run changed .env.secrets — `make dev` would rotate \
         credentials out from under a running database on every start"
    );

    // A hand-set value is the user's, not the generator's.
    let edited = second.replace(
        &format!(
            "API_DB_PASSWORD={}",
            parse_env(&secrets)
                .into_iter()
                .find(|(k, _)| k == "API_DB_PASSWORD")
                .map(|(_, v)| v)
                .unwrap()
        ),
        "API_DB_PASSWORD=chosen-by-a-human",
    );
    std::fs::write(&secrets, &edited).unwrap();
    run_generator(root);
    let after = std::fs::read_to_string(&secrets).unwrap();
    assert!(
        after.contains("API_DB_PASSWORD=chosen-by-a-human"),
        "the generator overwrote a user-set secret:\n{after}"
    );
}

// ── No recipe ships a placeholder nothing consumes ───────────────────────────

/// The conformance net for the class. A `__GENERATE_*__` token is a promise that
/// something will replace it; every one shipped must land in a file the shared
/// generator actually reads (`docker/.config/.env.*`) and must be gone once it
/// has run.
#[test]
fn no_shipped_recipe_leaves_a_generate_placeholder_unconsumed() {
    // Static half: where placeholders are allowed to live at all.
    let mut offenders: Vec<String> = Vec::new();
    let mut found = 0;
    for entry in walk(&templates_root()) {
        let rel = entry
            .strip_prefix(templates_root())
            .unwrap()
            .display()
            .to_string();
        let Ok(body) = std::fs::read_to_string(&entry) else {
            continue;
        };
        if !body.contains("__GENERATE_") {
            continue;
        }
        found += 1;
        // Two kinds of file may name the placeholder: an env template that
        // installs into docker/.config/ (where the generator looks), and the
        // shared consumer itself.
        let is_env_config = rel.contains("/config/env.") || rel.ends_with("config/env.secrets");
        let is_the_consumer = rel == "scripts/generate-secrets.sh" || rel == "scripts/.bashrc";
        if !is_env_config && !is_the_consumer {
            offenders.push(rel);
        }
    }
    assert!(
        found > 0,
        "setup: expected the templates to still use __GENERATE_* placeholders"
    );
    assert!(
        offenders.is_empty(),
        "these shipped files carry a __GENERATE_* placeholder outside the env config \
         files the shared generator reads, so nothing consumes them:\n{}",
        offenders.join("\n")
    );

    // Behavioral half: for every recipe, the generator leaves none behind.
    for recipe in shipped_recipe_names() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        scaffold_like_mx_new(root);
        install_recipe(root, &recipe, "api");
        run_generator(root);

        for file in installed_env_files(root) {
            let body = std::fs::read_to_string(&file).unwrap();
            assert!(
                !body.contains("__GENERATE_"),
                "{recipe}: {} still has an unconsumed placeholder after generation:\n{body}",
                file.strip_prefix(root).unwrap().display()
            );
        }
    }
}

/// Every regular file under `dir`, recursively.
fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&next) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

// ── doctor flags what is still unset ─────────────────────────────────────────

/// A `docker` stub on `PATH` so `doctor.sh` runs without a daemon.
fn stub_docker(dir: &Path) -> PathBuf {
    let bin = dir.join("stub-bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::write(bin.join("docker"), "#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(bin.join("docker"), std::fs::Permissions::from_mode(0o755))
            .unwrap();
    }
    bin
}

fn run_doctor(root: &Path) -> String {
    let bin = stub_docker(root);
    let out = std::process::Command::new("bash")
        .arg("-c")
        .arg("./scripts/doctor.sh")
        .current_dir(root)
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env_remove("COMPOSE_PROJECT_NAME")
        .output()
        .expect("run doctor.sh");
    let mut combined = String::from_utf8_lossy(&out.stdout).to_string();
    combined.push_str(&String::from_utf8_lossy(&out.stderr));
    combined
}

/// Cheap string check, no docker: `make doctor` names the keys that are still
/// empty or still a placeholder, so a project that was hand-edited into a broken
/// state says so before `make dev` hangs on an unhealthy database.
#[test]
fn doctor_names_empty_and_placeholder_secrets() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    scaffold_like_mx_new(root);
    install_recipe(root, "rust-api", "api");

    // Pre-generation: the shipped placeholders are exactly the broken state.
    std::fs::copy(
        root.join("docker/.config/.env.secrets.template"),
        root.join("docker/.config/.env.secrets"),
    )
    .unwrap();
    std::fs::write(
        root.join("docker/.config/.env.secrets"),
        "DB_USER=\nDB_PASSWORD=__GENERATE_DB_PASSWORD__\nDB_NAME=app\n",
    )
    .unwrap();

    let flagged = run_doctor(root);
    assert!(
        flagged.contains("- DB_USER"),
        "doctor must name the empty key DB_USER:\n{flagged}"
    );
    assert!(
        flagged.contains("- DB_PASSWORD"),
        "doctor must name the placeholder key DB_PASSWORD:\n{flagged}"
    );
    assert!(
        !flagged.contains("- DB_NAME"),
        "doctor listed DB_NAME as unset, but it has a real value:\n{flagged}"
    );

    // Post-generation: silence. A warning that fires on a healthy project is noise.
    run_generator(root);
    let clean = run_doctor(root);
    assert!(
        !clean.contains("__GENERATE_"),
        "doctor still reports placeholders after generation:\n{clean}"
    );
    for key in ["DB_USER", "DB_PASSWORD"] {
        assert!(
            !clean.contains(&format!("- {key}")),
            "doctor still flags {key} on a freshly generated project:\n{clean}"
        );
    }
}
