//! Conformance sweep for env layering and credential delivery
//! (bd:mech-crate-lwe + bd:mech-crate-q1w).
//!
//! Two halves of one contract, because both failures have the same root: what
//! Compose reads from an `env_file` layer and what the Compose **CLI** can see
//! are different things.
//!
//! **Layering order.** From `docs/development/mx-app-playbook.md`: container env
//! is layered `.env.shared` → `.env.secrets` → `.env.<service>`, and Compose
//! applies an `env_file:` list in order with **last one winning**. So the list
//! order *is* the precedence: a file that lists `.env.secrets` before
//! `.env.shared` silently inverts it and a project-wide default beats the secret
//! meant to override it.
//!
//! **Credential delivery.** The Compose CLI interpolates `${VAR}` from the
//! compose PROJECT DIRECTORY's `.env` only — never from an `env_file` layer,
//! which it hands to the daemon unread. No mx scaffold has a project-dir `.env`,
//! so `${DB_PASSWORD}` in a compose file renders EMPTY with a
//! `variable is not set` warning. Worse, an `environment:` block OUTRANKS every
//! `env_file` layer, so such a reference does not merely fail to resolve: it
//! overwrites the literal `scripts/generate-secrets.sh` already wrote into
//! `.env.<service>`. The rust-api recipe is the model — credentials and
//! everything derived from them (`DATABASE_URL`) live in the env files, where the
//! generator resolves `${…}` into literals, and `environment:` carries only
//! literal, non-credential app settings.
//!
//! This sweep is the regression net for both classes. It walks **every** compose
//! file under `templates/` (recipes included, dev overrides included) *and* under
//! the repo's own `site/docker/compose/`, which ships the same shape and is
//! otherwise outside every templates-only net, and asserts
//!
//!   1. every `env_file:` list is in the documented relative order,
//!   2. any list carrying the shared layer also carries the secrets layer,
//!   3. db-bearing recipes reach their services with the secrets layer,
//!   4. the walker actually sees every `env_file:` key shipped in the swept roots
//!      (so a parser that silently skips a file can't make 1-3 vacuous),
//!   5. no `environment:` value interpolates a variable that lives in an env file,
//!      and
//!   6. no shipped env config file hardcodes a database password, which is how
//!      `:secret@` outlived the credentials it was standing in for.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const SHARED: &str = ".env.shared";
const SECRETS: &str = ".env.secrets";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn templates_root() -> PathBuf {
    repo_root().join("templates")
}

/// Directories this sweep walks: the shipped templates, plus the repo's own site
/// infra. The site's compose files were scaffolded from the astro recipe and drift
/// with it, so a net that stops at `templates/` lets the dogfood copy rot — which
/// is exactly what happened (bd:mech-crate-q1w).
fn swept_roots() -> Vec<PathBuf> {
    vec![templates_root(), repo_root().join("site/docker/compose")]
}

/// Every `*.yml` / `*.yaml` under the swept roots, recursively, sorted.
fn yaml_files() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let entries = std::fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("setup: read {}: {e}", dir.display()))
            .flatten();
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "yml" || e == "yaml") {
                out.push(path);
            }
        }
    }

    let mut out = Vec::new();
    for root in swept_roots() {
        assert!(
            root.is_dir(),
            "setup: swept root {} does not exist",
            root.display()
        );
        walk(&root, &mut out);
    }
    out.sort();
    assert!(
        out.len() >= 30,
        "setup: expected the swept roots to carry dozens of YAML files, saw {}",
        out.len()
    );
    out
}

/// Compose files: a YAML doc in a swept root with a top-level `services`
/// mapping. Excludes the router's static Traefik config and the recipes' app
/// CI workflows, which are YAML but not Compose.
fn compose_files() -> Vec<PathBuf> {
    yaml_files()
        .into_iter()
        .filter(|p| {
            parse_compose(p)
                .and_then(|doc| doc.get("services").cloned())
                .is_some_and(|s| s.is_mapping())
        })
        .collect()
}

/// Template fragments carry `{{PLACEHOLDER}}` tokens, including in mapping keys
/// (`{{SERVICE_NAME}}:`), which is not valid YAML. Swap each token for an inert
/// scalar so the shipped file can be read with real YAML semantics rather than a
/// line-sniffing approximation. Distinct keys stay distinct (`{{SERVICE_NAME}}`
/// vs `{{SERVICE_NAME}}-worker`) because only the braces are replaced.
fn neutralize_placeholders(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(start) = rest.find("{{") {
        let Some(end) = rest[start..].find("}}") else {
            break;
        };
        out.push_str(&rest[..start]);
        out.push_str("MX_PLACEHOLDER");
        rest = &rest[start + end + 2..];
    }
    out.push_str(rest);
    out
}

fn parse_compose(path: &Path) -> Option<serde_yaml::Value> {
    let body = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("setup: read {}: {e}", path.display()));
    serde_yaml::from_str(&neutralize_placeholders(&body)).ok()
}

/// Every `env_file:` list declared by a compose file, as
/// `(service name, entries in declaration order)`. Handles Compose's three
/// shapes: a bare string, a list of strings, and the long form
/// (`- path: …` with an optional `required:`).
fn env_file_lists(compose: &Path) -> Vec<(String, Vec<String>)> {
    let Some(doc) = parse_compose(compose) else {
        panic!("{}: not valid YAML", compose.display());
    };
    let Some(services) = doc.get("services").and_then(|v| v.as_mapping()) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for (name, body) in services {
        let service = name
            .as_str()
            .unwrap_or("<non-string service key>")
            .to_string();
        let Some(raw) = body.get("env_file") else {
            continue;
        };

        let mut entries: Vec<String> = Vec::new();
        let mut push = |v: &serde_yaml::Value, compose: &Path| match v {
            serde_yaml::Value::String(s) => entries.push(s.clone()),
            serde_yaml::Value::Mapping(m) => match m.get(serde_yaml::Value::from("path")) {
                Some(serde_yaml::Value::String(s)) => entries.push(s.clone()),
                other => panic!(
                    "{}: `env_file` long-form entry without a string `path`: {other:?}",
                    compose.display()
                ),
            },
            other => panic!(
                "{}: unexpected `env_file` entry {other:?}",
                compose.display()
            ),
        };

        match raw {
            serde_yaml::Value::Sequence(seq) => {
                for v in seq {
                    push(v, compose);
                }
            }
            single => push(single, compose),
        }

        out.push((service, entries));
    }
    out
}

/// Documented layering position of one `env_file` entry: shared (0) before
/// secrets (1) before service-specific (2).
fn layer_rank(entry: &str) -> u8 {
    let file = entry.rsplit('/').next().unwrap_or(entry);
    if file == SHARED {
        0
    } else if file == SECRETS {
        1
    } else {
        2
    }
}

fn rel(path: &Path) -> String {
    // Both roots are spelled relative to the crate manifest, so strip the repo
    // root rather than `templates/` — otherwise a site path prints as `../../…`.
    let repo = repo_root();
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let base = repo.canonicalize().unwrap_or(repo);
    canonical
        .strip_prefix(&base)
        .unwrap_or(&canonical)
        .to_string_lossy()
        .to_string()
}

/// bd:mech-crate-lwe — Compose applies `env_file:` in order, last one winning,
/// so the list order is the precedence. Documented order is
/// `.env.shared` → `.env.secrets` → `.env.<service>`; anything else means a
/// project-wide default quietly beats the secret written to override it.
#[test]
fn every_env_file_list_follows_the_documented_precedence_order() {
    let mut offenders: Vec<String> = Vec::new();
    let mut checked = 0;

    for compose in compose_files() {
        for (service, entries) in env_file_lists(&compose) {
            checked += 1;
            let ranks: Vec<u8> = entries.iter().map(|e| layer_rank(e)).collect();
            if ranks.windows(2).any(|w| w[0] > w[1]) {
                offenders.push(format!(
                    "{} [{service}]: {:?} — documented order is {SHARED} → {SECRETS} → .env.<service> (last wins)",
                    rel(&compose),
                    entries
                ));
            }
        }
    }

    assert!(
        checked >= 15,
        "setup: expected the shipped templates to declare many env_file lists, saw {checked}"
    );
    assert!(
        offenders.is_empty(),
        "{} env_file list(s) invert the documented env precedence:\n{}",
        offenders.len(),
        offenders.join("\n")
    );
}

/// bd:mech-crate-lwe (zola variant) — omitting `.env.secrets` is the same defect
/// seen from the other side: the layer that is supposed to win is not consulted
/// at all, so a secret can never override a shared default. Any service that
/// layers env at all must carry the secrets layer. `scripts/init.sh` guarantees
/// `docker/.config/.env.secrets` exists in every scaffolded project (it copies
/// the template, or touches an empty file), so listing it is always safe.
#[test]
fn every_env_file_list_carrying_shared_also_carries_secrets() {
    let mut offenders: Vec<String> = Vec::new();
    let mut checked = 0;

    for compose in compose_files() {
        for (service, entries) in env_file_lists(&compose) {
            if !entries.iter().any(|e| layer_rank(e) == 0) {
                continue;
            }
            checked += 1;
            if !entries.iter().any(|e| layer_rank(e) == 1) {
                offenders.push(format!(
                    "{} [{service}]: {:?} — layers {SHARED} but never {SECRETS}",
                    rel(&compose),
                    entries
                ));
            }
        }
    }

    assert!(
        checked >= 15,
        "setup: expected many env_file lists to carry the shared layer, saw {checked}"
    );
    assert!(
        offenders.is_empty(),
        "{} env_file list(s) skip the secrets layer:\n{}",
        offenders.len(),
        offenders.join("\n")
    );
}

/// A recipe whose service set includes a database ships credentials in
/// `.env.secrets`; if no compose file in the recipe lists it, those credentials
/// never reach a container.
#[test]
fn db_bearing_recipes_layer_the_secrets_file() {
    let recipes = templates_root().join("recipes");
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&recipes)
        .expect("setup: templates/recipes must exist")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join("recipe.json").exists())
        .collect();
    dirs.sort();

    let mut db_bearing = 0;
    let mut offenders: Vec<String> = Vec::new();

    for dir in dirs {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        let raw: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("recipe.json")).unwrap())
                .unwrap_or_else(|e| panic!("{name}: invalid recipe.json: {e}"));
        let has_db = raw["services"]
            .as_array()
            .map(|s| {
                s.iter()
                    .filter_map(|svc| svc["name"].as_str())
                    .any(|n| n == "db")
            })
            .unwrap_or(false);
        if !has_db {
            continue;
        }
        db_bearing += 1;

        let compose_dir = dir.join("docker/compose");
        let lists: Vec<(PathBuf, Vec<String>)> = compose_files()
            .into_iter()
            .filter(|p| p.starts_with(&compose_dir))
            .flat_map(|p| {
                env_file_lists(&p)
                    .into_iter()
                    .map(move |(_, entries)| (p.clone(), entries))
            })
            .collect();

        if !lists
            .iter()
            .any(|(_, entries)| entries.iter().any(|e| layer_rank(e) == 1))
        {
            offenders.push(format!(
                "{name}: db-bearing but no compose file layers {SECRETS} (lists seen: {:?})",
                lists
                    .iter()
                    .map(|(p, e)| format!("{}={e:?}", rel(p)))
                    .collect::<Vec<_>>()
            ));
        }
    }

    assert!(
        db_bearing >= 4,
        "setup: expected several db-bearing recipes, saw {db_bearing}"
    );
    assert!(
        offenders.is_empty(),
        "db-bearing recipe(s) never layer the secrets file:\n{}",
        offenders.join("\n")
    );
}

/// Coverage guard: the ordering assertions above are only as good as the walker
/// feeding them. Every `env_file:` key shipped in a swept YAML file must be one
/// the sweep actually read — otherwise a file Compose honors (a new recipe, a dev
/// override, a file whose placeholders break the parse) could invert precedence
/// and every test above would still pass.
#[test]
fn the_sweep_reads_every_env_file_key_shipped_in_the_swept_roots() {
    let mut unread: Vec<String> = Vec::new();
    let mut total_keys = 0;

    for path in yaml_files() {
        let body = std::fs::read_to_string(&path).unwrap();
        let declared = body
            .lines()
            .filter(|l| l.trim_start().starts_with("env_file:"))
            .count();
        if declared == 0 {
            continue;
        }
        total_keys += declared;

        let parsed = env_file_lists(&path).len();
        if parsed != declared {
            unread.push(format!(
                "{}: {declared} `env_file:` key(s) in the file, {parsed} read by the sweep",
                rel(&path)
            ));
        }
    }

    assert!(
        total_keys >= 15,
        "setup: expected many shipped `env_file:` keys, saw {total_keys}"
    );
    assert!(
        unread.is_empty(),
        "the env precedence sweep does not read every shipped env_file list:\n{}",
        unread.join("\n")
    );
}

// ── Credential delivery (bd:mech-crate-q1w) ─────────────────────────────────

/// Every shipped env config file: the `env.*` sources inside a `config/`
/// directory, which are the ones the installer lays down as
/// `docker/.config/.env.*`. `env.secrets.template` counts — it is the seed
/// `scripts/init.sh` copies to `.env.secrets`. An app-level `.env` template
/// (`recipes/laravel/app/env.template`) does not: it lands in `apps/<name>/` and
/// is the framework's own config, not a compose `env_file` layer.
fn env_config_files() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.parent().and_then(Path::file_name) == Some("config".as_ref())
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("env."))
            {
                out.push(path);
            }
        }
    }

    let mut out = Vec::new();
    walk(&templates_root(), &mut out);
    out.sort();
    assert!(
        out.len() >= 10,
        "setup: expected the shipped templates to carry many env config files, saw {}",
        out.len()
    );
    out
}

/// True when a value is the generator's *input* rather than a credential: empty,
/// or still one of the placeholder conventions `scripts/.bashrc` recognizes. Pass
/// 1 of `generate-secrets.sh` fills these in; they are correct as shipped.
fn is_generator_input(value: &str) -> bool {
    value.is_empty()
        || value.starts_with("__GENERATE_")
        || value.starts_with("CHANGE_ME")
        || value == "changeme"
}

/// `KEY=VALUE` pairs of one env config file, comments and blanks skipped.
fn env_config_pairs(path: &Path) -> Vec<(String, String)> {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("setup: read {}: {e}", path.display()))
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect()
}

/// Every variable name any shipped env config file defines. These are exactly the
/// names that reach a container through an `env_file` layer — and therefore
/// exactly the names the Compose CLI cannot interpolate.
fn env_file_variable_names() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for file in env_config_files() {
        for (key, _) in env_config_pairs(&file) {
            // Recipe sources carry `{{SERVICE_UPPER}}_DB_PASSWORD`; the
            // neutralized stem is still the shape a compose file would reference.
            names.insert(neutralize_placeholders(&key));
        }
    }
    names
}

/// The `${NAME}` / `$NAME` references in one value, in order. `$$` is Compose's
/// escape for a literal `$` (deferring expansion to the container), so a `$$VAR`
/// is not a CLI interpolation and is skipped.
fn interpolated_names(value: &str) -> Vec<String> {
    let bytes: Vec<char> = value.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != '$' {
            i += 1;
            continue;
        }
        if bytes.get(i + 1) == Some(&'$') {
            i += 2; // escaped literal `$`, resolved in the container
            continue;
        }
        if bytes.get(i + 1) == Some(&'{') {
            let Some(close) = bytes[i + 2..].iter().position(|c| *c == '}') else {
                break;
            };
            let inner: String = bytes[i + 2..i + 2 + close].iter().collect();
            // Strip `:-default` / `-default` / `:?err` modifiers.
            let name = inner
                .split([':', '-', '?', '+'])
                .next()
                .unwrap_or(&inner)
                .to_string();
            if !name.is_empty() {
                out.push(name);
            }
            i += 2 + close + 1;
            continue;
        }
        let mut j = i + 1;
        while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == '_') {
            j += 1;
        }
        if j > i + 1 {
            out.push(bytes[i + 1..j].iter().collect());
        }
        i = j.max(i + 1);
    }
    out
}

/// Every `environment:` entry of one compose file, as
/// `(service, key, value)`. Handles both Compose shapes: a mapping
/// (`KEY: value`) and a list (`- KEY=value`). A list entry with no `=` inherits
/// from the host environment and carries no value to check.
fn environment_entries(compose: &Path) -> Vec<(String, String, String)> {
    let Some(doc) = parse_compose(compose) else {
        panic!("{}: not valid YAML", compose.display());
    };
    let Some(services) = doc.get("services").and_then(|v| v.as_mapping()) else {
        return Vec::new();
    };

    fn scalar(v: &serde_yaml::Value) -> Option<String> {
        match v {
            serde_yaml::Value::String(s) => Some(s.clone()),
            serde_yaml::Value::Bool(b) => Some(b.to_string()),
            serde_yaml::Value::Number(n) => Some(n.to_string()),
            serde_yaml::Value::Null => Some(String::new()),
            _ => None,
        }
    }

    let mut out = Vec::new();
    for (name, body) in services {
        let service = name
            .as_str()
            .unwrap_or("<non-string service key>")
            .to_string();
        let Some(raw) = body.get("environment") else {
            continue;
        };

        match raw {
            serde_yaml::Value::Mapping(map) => {
                for (k, v) in map {
                    let key = k.as_str().unwrap_or("<non-string key>").to_string();
                    if let Some(value) = scalar(v) {
                        out.push((service.clone(), key, value));
                    }
                }
            }
            serde_yaml::Value::Sequence(seq) => {
                for v in seq {
                    let Some(entry) = scalar(v) else { continue };
                    match entry.split_once('=') {
                        Some((k, value)) => {
                            out.push((service.clone(), k.to_string(), value.to_string()))
                        }
                        None => continue, // pass-through from the host env
                    }
                }
            }
            other => panic!(
                "{}: unexpected `environment` shape {other:?}",
                compose.display()
            ),
        }
    }
    out
}

/// bd:mech-crate-q1w — the non-dev path rendered `DATABASE_URL` empty. Compose
/// interpolates `${VAR}` in a compose file from the project directory's `.env`
/// only; an `env_file` layer is opaque to it. No mx scaffold has a project-dir
/// `.env`, so every such reference resolved to nothing and `docker compose
/// config` warned `variable is not set`.
///
/// It is worse than an empty render. An `environment:` block beats every
/// `env_file` layer, so `DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@…` in
/// compose **replaced** the working literal that `generate-secrets.sh` had
/// already resolved into `.env.<service>`. The recipe with no such line
/// (rust-api) was the only one whose non-dev boot could reach its database.
///
/// So: an `environment:` value may not interpolate any variable that an env
/// config file defines. Those names belong to the `env_file` layers, where the
/// generator has already turned them into literals.
#[test]
fn no_compose_environment_value_interpolates_an_env_file_variable() {
    let from_env_files = env_file_variable_names();
    for root in ["DB_USER", "DB_PASSWORD", "DB_NAME"] {
        assert!(
            from_env_files.contains(root),
            "setup: {root} must be defined by some shipped env config file, or this net is vacuous"
        );
    }

    let mut offenders: Vec<String> = Vec::new();
    let mut checked = 0;

    for compose in compose_files() {
        for (service, key, value) in environment_entries(&compose) {
            checked += 1;
            let mut hits: Vec<String> = interpolated_names(&value)
                .into_iter()
                .filter(|n| from_env_files.contains(n))
                .collect();
            hits.dedup();
            if !hits.is_empty() {
                offenders.push(format!(
                    "{} [{service}] {key}={value} — interpolates {hits:?}, which live in env_file layers the Compose CLI never reads",
                    rel(&compose)
                ));
            }
        }
    }

    assert!(
        checked >= 40,
        "setup: expected the swept compose files to declare many environment entries, saw {checked}"
    );
    assert!(
        offenders.is_empty(),
        "{} compose `environment:` value(s) interpolate an env_file variable:\n{}",
        offenders.len(),
        offenders.join("\n")
    );
}

/// bd:mech-crate-q1w (the other half) — a hardcoded `:secret@` in a shipped env
/// file is the same defect wearing a disguise. It renders non-empty, so nothing
/// warns, and it was right for exactly as long as every project shared one
/// password. Once `generate-secrets.sh` made credentials per-project, `secret`
/// became a value that authenticates against nothing. Derive from the generated
/// roots instead: the generator resolves `${DB_PASSWORD}` into the real literal.
#[test]
fn no_shipped_env_config_hardcodes_a_database_password() {
    // Keys whose value is a database password, or a URL that embeds one.
    const PASSWORD_KEYS: [&str; 3] = ["DB_PASSWORD", "POSTGRES_PASSWORD", "DATABASE_URL"];

    let mut offenders: Vec<String> = Vec::new();
    let mut checked = 0;

    for file in env_config_files() {
        for (key, value) in env_config_pairs(&file) {
            let stem = neutralize_placeholders(&key);
            let is_password_key = PASSWORD_KEYS
                .iter()
                .any(|k| stem == *k || stem.ends_with(&format!("_{k}")));
            if !is_password_key || is_generator_input(&value) {
                continue;
            }
            checked += 1;
            // A derived value references the generated root; a literal does not.
            if !value.contains("${") {
                offenders.push(format!(
                    "{} {key}={value} — a literal credential. Derive it from the generated roots (`${{DB_PASSWORD}}` etc.) so generate-secrets.sh resolves it per project",
                    rel(&file)
                ));
            }
        }
    }

    assert!(
        checked >= 5,
        "setup: expected the shipped env config files to define several database password keys, saw {checked}"
    );
    assert!(
        offenders.is_empty(),
        "{} shipped env value(s) hardcode a database credential:\n{}",
        offenders.len(),
        offenders.join("\n")
    );
}
