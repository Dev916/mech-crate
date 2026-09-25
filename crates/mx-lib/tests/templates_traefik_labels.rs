//! Traefik label-name conformance for the shipped compose files
//! (bd:mech-crate-298).
//!
//! # The defect this nets
//!
//! Traefik's routers, services and middlewares live in ONE global table per
//! provider, shared by every container on the machine. The shipped recipes named
//! their entries after the compose service alone — `traefik.http.routers.api.*`
//! — so two mx projects that each scaffolded a service called `api` wrote the
//! same key into that one table. Wave 1 gave each project its own compose
//! namespace and wave 3's sibling net freed the host ports, but the router table
//! stayed shared, and it is the last thing between two stacks and coexistence.
//!
//! What actually happens divides on whether the two projects' labels agree,
//! and both outcomes were reproduced live against the running mx-router
//! (traefik v3.6, two `traefik/whoami` containers, one per compose project):
//!
//!   * **Labels differ in any way** (a different `--domain`, a different
//!     loadbalancer port, zola's extra middleware) and Traefik drops the router
//!     outright:
//!
//!     ```text
//!     ERR Router defined multiple times with different configurations
//!         configuration=["api-probe-a-f92d409…","api-probe-b-be0cca2…"]
//!         providerName=docker routerName=api
//!     ```
//!
//!     Both hostnames then answer `404`. Neither project is reachable, and
//!     nothing in either project says why.
//!
//!   * **Labels agree** and it is quieter and worse: Traefik merges the two
//!     containers into one load-balancer pool and round-robins between them.
//!     `GET api.localhost` alternated between project A's app and project B's
//!     app, and the API confirmed one service holding both backends:
//!
//!     ```text
//!     api@docker → [{"url":"http://192.168.107.6:80"},
//!                   {"url":"http://192.168.107.8:8000"}]
//!     ```
//!
//!     No error is logged. One project's traffic silently lands in another
//!     project's process.
//!
//! # Design call 1: the NAME carries the project
//!
//! Every name in the three shared namespaces is prefixed
//! `${COMPOSE_PROJECT_NAME}-`, so project `blog` and project `shop` write
//! `blog-api` and `shop-api` into Traefik's table and never meet. The prefix is
//! resolved by the compose CLI, not by the installer, because the value is a
//! property of the invocation rather than of the scaffold: the generated project
//! is copied, renamed and cloned, and its own directory name is what
//! `scripts/.bashrc` derives the project name from.
//!
//! Compose supplies `COMPOSE_PROJECT_NAME` to interpolation itself, from the
//! project name it resolved — verified live: `docker compose -p probeproj -f …
//! config` renders `traefik.http.routers.probeproj-api.rule` with the variable
//! UNSET in the environment, and emits no `variable is not set` warning. So the
//! qualification holds for a hand-rolled `docker compose -f …` too, which is the
//! one path `scripts/.bashrc` cannot reach.
//!
//! # Design call 2: the HOSTNAME stays put, and gains a knob
//!
//! Unique names stop the drop and stop the merge. They do NOT stop two projects
//! claiming one hostname: both `blog-api` and `shop-api` still carry
//! ``Host(`api.localhost`)``. Traefik accepts both routers (no error), and with
//! equal rules it serves exactly one of them — the other is unreachable, which
//! fails the whole point of the fix.
//!
//! Changing the default hostname to embed the project would fix that and break
//! every URL, bookmark, OAuth callback and README in every existing project, for
//! the sake of the second stack. So the default is unchanged — `{{DOMAIN}}`,
//! itself defaulting to `{{SERVICE_NAME}}.localhost` — and the rule reads
//! through the environment instead:
//!
//! ```text
//! traefik.http.routers.${COMPOSE_PROJECT_NAME}-api.rule=Host(`${API_ROUTER_HOST:-api.localhost}`)
//! ```
//!
//! A lone stack is untouched. The second stack moves with
//! `API_ROUTER_HOST=api-two.localhost make dev` — an exported variable, not an
//! edit to a shipped file, the same shape as wave 3's `DB_HOST_PORT`. Verified
//! live: with the override on stack B, `api.localhost` answered 200 from A and
//! `api-two.localhost` answered 200 from B, concurrently.
//!
//! The variable is `<SERVICE_UPPER>_ROUTER_HOST` rather than `<SERVICE>_HOST`
//! because `DB_HOST`, `REDIS_HOST` and `API_HOST` are conventional
//! service-address variables that a developer may well already export, and
//! quietly repointing a router at a database host is not a failure anyone would
//! debug. `_ROUTER_HOST` says which host it means.
//!
//! # The singleton exception
//!
//! mx-router itself is deliberately ONE per machine (the wave-2 design call), so
//! its routing is not project-scoped. It also needs no exception here: the router
//! template configures Traefik by file, not by label, and the middlewares in
//! `templates/router/config/dynamic/` are `@file` singletons on purpose. The last
//! test below pins that, so label-based routing appearing on the router template
//! is a deliberate decision rather than an accident that slips past this net.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Directories swept: the shipped templates, plus the repo's own site infra,
/// which was scaffolded from the astro recipe and drifts with it — the same pair
/// of roots the host-port and env sweeps walk, for the same reason.
fn swept_roots() -> Vec<PathBuf> {
    vec![
        repo_root().join("templates"),
        repo_root().join("site/docker/compose"),
    ]
}

/// Template fragments carry `{{PLACEHOLDER}}` tokens, including where YAML wants
/// a mapping key (`{{SERVICE_NAME}}:`), so a shipped file is not YAML until the
/// installer has expanded it. Swap each token for a plain scalar — delimited on
/// both sides so [`restore_placeholders`] can put the braces back and an offender
/// line can quote the file as shipped.
const PH_FENCE: &str = "MXPH";

fn neutralize_placeholders(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(start) = rest.find("{{") {
        let Some(end) = rest[start + 2..].find("}}") else {
            break;
        };
        let name = &rest[start + 2..start + 2 + end];
        out.push_str(&rest[..start]);
        out.push_str(PH_FENCE);
        out.push_str(name.trim());
        out.push_str(PH_FENCE);
        rest = &rest[start + 2 + end + 2..];
    }
    out.push_str(rest);
    out
}

/// Inverse of [`neutralize_placeholders`], for readable failure messages.
fn restore_placeholders(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(PH_FENCE) {
        let after = start + PH_FENCE.len();
        let Some(end) = rest[after..].find(PH_FENCE) else {
            break;
        };
        out.push_str(&rest[..start]);
        out.push_str("{{");
        out.push_str(&rest[after..after + end]);
        out.push_str("}}");
        rest = &rest[after + end + PH_FENCE.len()..];
    }
    out.push_str(rest);
    out
}

/// One `traefik.http.*` label on one shipped compose service.
#[derive(Debug)]
struct Label {
    /// Repo-relative path of the compose file.
    rel: String,
    /// Compose service the label belongs to, as the shipped file spells it.
    service: String,
    /// Label key, as the shipped file spells it.
    key: String,
    /// Label value, as the shipped file spells it.
    value: String,
}

impl Label {
    /// `traefik.http.<namespace>.<name>.<rest>` → `(namespace, name)`.
    ///
    /// Only the three namespaces whose names are keys in Traefik's global tables
    /// are reported; `traefik.enable` and `traefik.docker.network` are per
    /// container and cannot collide.
    fn namespaced(&self) -> Option<(&str, &str)> {
        let rest = self.key.strip_prefix("traefik.http.")?;
        let (namespace, rest) = rest.split_once('.')?;
        if !SHARED_NAMESPACES.contains(&namespace) {
            return None;
        }
        let name = rest.split('.').next()?;
        Some((namespace, name))
    }

    fn at(&self) -> String {
        format!("{}: service `{}` → `{}`", self.rel, self.service, self.key)
    }
}

/// Traefik namespaces that are one flat table per provider, machine-wide.
const SHARED_NAMESPACES: &[&str] = &["routers", "services", "middlewares"];

/// The prefix every name in a shared namespace must carry.
const PROJECT_PREFIX: &str = "${COMPOSE_PROJECT_NAME}-";

fn yml_files(roots: &[PathBuf]) -> Vec<PathBuf> {
    fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("setup: read {}: {e}", dir.display()))
            .flatten()
        {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, files);
            } else if path.extension().is_some_and(|e| e == "yml" || e == "yaml") {
                files.push(path);
            }
        }
    }

    let mut files = Vec::new();
    for dir in roots {
        assert!(
            dir.is_dir(),
            "setup: swept root {} does not exist",
            dir.display()
        );
        walk(dir, &mut files);
    }
    files.sort();
    files
}

/// Every `traefik.http.*` label across the given compose files.
fn traefik_labels_in(files: &[PathBuf]) -> Vec<Label> {
    let root = repo_root();
    let mut out: Vec<Label> = Vec::new();

    for path in files {
        let body = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("setup: read {}: {e}", path.display()));
        let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(&neutralize_placeholders(&body))
        else {
            continue;
        };
        let Some(services) = doc.get("services").and_then(|v| v.as_mapping()) else {
            continue;
        };
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        for (name, service_body) in services {
            let service = restore_placeholders(name.as_str().unwrap_or("<non-string key>"));
            let Some(labels) = service_body.get("labels") else {
                continue;
            };

            // `labels:` is either a sequence of `key=value` strings or a mapping.
            let pairs: Vec<(String, String)> = match labels {
                serde_yaml::Value::Sequence(items) => items
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|entry| match entry.split_once('=') {
                        Some((k, v)) => (k.to_string(), v.to_string()),
                        None => (entry.to_string(), String::new()),
                    })
                    .collect(),
                serde_yaml::Value::Mapping(m) => m
                    .iter()
                    .filter_map(|(k, v)| {
                        let key = k.as_str()?.to_string();
                        let value = match v {
                            serde_yaml::Value::String(s) => s.clone(),
                            other => serde_yaml::to_string(other)
                                .unwrap_or_default()
                                .trim()
                                .to_string(),
                        };
                        Some((key, value))
                    })
                    .collect(),
                _ => panic!("{rel}: service `{service}` has a `labels:` that is neither a sequence nor a mapping"),
            };

            for (key, value) in pairs {
                if !key.starts_with("traefik.") {
                    continue;
                }
                out.push(Label {
                    rel: rel.clone(),
                    service: service.clone(),
                    key: restore_placeholders(&key),
                    value: restore_placeholders(&value),
                });
            }
        }
    }

    out
}

/// Every `traefik.http.*` label the recipes and the repo's own site ship.
fn shipped_traefik_labels() -> Vec<Label> {
    let labels = traefik_labels_in(&yml_files(&swept_roots()));
    let shared = labels.iter().filter(|l| l.namespaced().is_some()).count();

    assert!(
        shared >= 20,
        "setup: the shipped compose files carry ~26 router/service/middleware \
         labels across 8 files; the walker found {shared} — it has stopped \
         reading `labels:` blocks and every check below would pass vacuously"
    );
    labels
}

/// `${NAME:-<default>}` → `(NAME, <default>)`. `None` for any other shape.
fn env_with_default(raw: &str) -> Option<(&str, &str)> {
    let inner = raw.strip_prefix("${")?.strip_suffix('}')?;
    let (name, default) = inner.split_once(":-")?;
    Some((name, default))
}

/// The arguments of every ``Host(`…`)`` in a Traefik rule expression.
fn host_args(rule: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = rule;
    while let Some(at) = rest.find("Host(`") {
        rest = &rest[at + "Host(`".len()..];
        match rest.find('`') {
            Some(end) => {
                out.push(rest[..end].to_string());
                rest = &rest[end + 1..];
            }
            None => break,
        }
    }
    out
}

/// bd:mech-crate-298 — the core net. Traefik keeps one router table, one service
/// table and one middleware table per provider for the whole machine, so a name
/// derived from the compose service alone is a claim on every project at once.
#[test]
fn every_traefik_name_is_project_qualified() {
    let mut offenders: Vec<String> = Vec::new();

    for label in shipped_traefik_labels() {
        let Some((namespace, name)) = label.namespaced() else {
            continue;
        };
        let qualified = name
            .strip_prefix(PROJECT_PREFIX)
            .is_some_and(|tail| !tail.is_empty());
        if !qualified {
            offenders.push(format!(
                "{}: {namespace} name `{name}` is not `{PROJECT_PREFIX}…`",
                label.at()
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "Traefik's router/service/middleware tables are machine-wide, so an \
         unqualified name is shared with every other mx project: two stacks that \
         each scaffolded the same service name either lose the router entirely \
         (`Router defined multiple times with different configurations`, 404 for \
         BOTH) or, when the labels happen to agree, get silently merged into one \
         load-balancer pool that round-robins one project's traffic into the \
         other's process. Prefix the name with `{PROJECT_PREFIX}` (the compose CLI \
         resolves it from the project name it already knows):\n{}",
        offenders
            .iter()
            .map(|o| format!("  {o}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// A qualified middleware DEFINITION whose reference was left unqualified is
/// worse than neither: the router asks for a middleware that does not exist, so
/// Traefik rejects the router and the service disappears. Definition and
/// reference have to move together.
#[test]
fn every_middleware_reference_names_a_project_qualified_middleware() {
    let mut offenders: Vec<String> = Vec::new();

    for label in shipped_traefik_labels() {
        let Some((namespace, _)) = label.namespaced() else {
            continue;
        };
        if namespace != "routers" || !label.key.ends_with(".middlewares") {
            continue;
        }
        for reference in label
            .value
            .split(',')
            .map(str::trim)
            .filter(|r| !r.is_empty())
        {
            // `name@provider` reaches outside this compose file on purpose:
            // the router's own `default-headers@file` singletons, for instance.
            if reference.contains('@') {
                continue;
            }
            if !reference.starts_with(PROJECT_PREFIX) {
                offenders.push(format!(
                    "{}: references middleware `{reference}`, which is not \
                     `{PROJECT_PREFIX}…`",
                    label.at()
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "a router referencing an unqualified middleware either picks up another \
         project's definition or none at all:\n{}",
        offenders
            .iter()
            .map(|o| format!("  {o}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Design call 2. Unique names stop the collision in Traefik's tables; they do
/// not stop two projects claiming one hostname, and Traefik serves exactly one
/// of two routers whose rules are equal. So every hostname must be reachable
/// through the environment, letting the SECOND stack move without touching a
/// shipped file — while the default keeps the first stack's documented URL.
#[test]
fn every_router_hostname_is_env_overridable() {
    let mut offenders: Vec<String> = Vec::new();
    let mut rules = 0usize;

    for label in shipped_traefik_labels() {
        let Some((namespace, _)) = label.namespaced() else {
            continue;
        };
        if namespace != "routers" || !label.key.ends_with(".rule") {
            continue;
        }
        rules += 1;
        let hosts = host_args(&label.value);
        if hosts.is_empty() {
            offenders.push(format!(
                "{}: rule `{}` names no Host(`…`) — a rule this net cannot read \
                 is a rule it cannot keep honest",
                label.at(),
                label.value
            ));
            continue;
        }
        for host in hosts {
            match env_with_default(&host) {
                Some((name, default)) => {
                    if default.is_empty() {
                        offenders.push(format!(
                            "{}: Host(`{host}`) has an empty default — a lone stack \
                             would boot with no hostname at all",
                            label.at()
                        ));
                    }
                    if !name.ends_with("_ROUTER_HOST") {
                        offenders.push(format!(
                            "{}: Host(`{host}`) reads `{name}`; the convention is \
                             `<SERVICE_UPPER>_ROUTER_HOST`, because `API_HOST` and \
                             friends are conventional service-address variables a \
                             developer may already export",
                            label.at()
                        ));
                    }
                }
                None => offenders.push(format!(
                    "{}: Host(`{host}`) is a fixed hostname — the second stack to \
                     want it has nowhere to move to",
                    label.at()
                )),
            }
        }
    }

    assert!(
        rules >= 8,
        "setup: the shipped compose files carry one router rule per web-facing \
         service (~8); the walker found {rules}"
    );
    assert!(
        offenders.is_empty(),
        "two uniquely-named routers with equal rules are both accepted by Traefik \
         and only one of them is ever served, so the hostname needs a knob too. \
         Keep today's hostname as the DEFAULT and read it through the \
         environment: Host(`${{<SERVICE_UPPER>_ROUTER_HOST:-{{{{DOMAIN}}}}}}`):\n{}",
        offenders
            .iter()
            .map(|o| format!("  {o}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The singleton exception, pinned rather than assumed. mx-router is one per
/// machine, so nothing about it is project-scoped — and it needs no carve-out
/// from the checks above only because it routes by FILE, not by label. If
/// label-based routing ever lands on the router template, that is a design
/// decision about the machine-wide singleton, and it should be made here
/// deliberately instead of silently failing the sweep above.
#[test]
fn the_router_template_still_routes_by_file_not_by_label() {
    let router = repo_root().join("templates/router");
    assert!(
        router.is_dir(),
        "setup: templates/router does not exist — the machine-wide router \
         singleton this exception describes has moved"
    );

    let labels = traefik_labels_in(&yml_files(&[router]));
    let shared: Vec<String> = labels
        .iter()
        .filter(|l| l.namespaced().is_some())
        .map(|l| l.at())
        .collect();

    assert!(
        shared.is_empty(),
        "templates/router now carries label-based routing. mx-router is a \
         deliberate machine-wide singleton, so `${{COMPOSE_PROJECT_NAME}}` means \
         nothing there; decide what these names should be and record the call \
         beside this test:\n{}",
        shared
            .iter()
            .map(|o| format!("  {o}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
