//! Conformance sweep for the documented env layering order (bd:mech-crate-lwe).
//!
//! The contract, from `docs/development/mx-app-playbook.md`: container env is
//! layered `.env.shared` → `.env.secrets` → `.env.<service>`, and Compose applies
//! an `env_file:` list in order with **last one winning**. So the list order *is*
//! the precedence: a file that lists `.env.secrets` before `.env.shared` silently
//! inverts it and a project-wide default beats the secret meant to override it.
//!
//! Eight recipe fragments shipped that inversion and the zola recipe omitted the
//! secrets layer outright. This sweep is the regression net for the whole class,
//! not those nine files: it walks **every** compose file under `templates/`
//! (recipes included, dev overrides included) and asserts
//!
//!   1. every `env_file:` list is in the documented relative order,
//!   2. any list carrying the shared layer also carries the secrets layer,
//!   3. db-bearing recipes reach their services with the secrets layer, and
//!   4. the walker actually sees every `env_file:` key shipped in `templates/`
//!      (so a parser that silently skips a file can't make 1-3 vacuous).

use std::path::{Path, PathBuf};

const SHARED: &str = ".env.shared";
const SECRETS: &str = ".env.secrets";

fn templates_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates")
}

/// Every `*.yml` / `*.yaml` under `templates/`, recursively, sorted.
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
    walk(&templates_root(), &mut out);
    out.sort();
    assert!(
        out.len() >= 30,
        "setup: expected the shipped templates to carry dozens of YAML files, saw {}",
        out.len()
    );
    out
}

/// Compose files: a YAML doc under `templates/` with a top-level `services`
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
    path.strip_prefix(templates_root())
        .unwrap_or(path)
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
/// feeding them. Every `env_file:` key shipped in a `templates/` YAML file must
/// be one the sweep actually read — otherwise a file Compose honors (a new
/// recipe, a dev override, a file whose placeholders break the parse) could
/// invert precedence and every test above would still pass.
#[test]
fn the_sweep_reads_every_env_file_key_shipped_in_templates() {
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
