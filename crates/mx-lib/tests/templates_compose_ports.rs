//! Host-port publishing conformance for the shipped compose files
//! (bd:mech-crate-1a0, with the legacy-template overlap of bd:mech-crate-qpy).
//!
//! # The defect this nets
//!
//! A host port is a workstation-wide singleton. Wave 1 isolated compose
//! namespaces (`COMPOSE_PROJECT_NAME`) and wave 2 stopped the shipped files
//! pinning `container_name`, but every dev override still published a FIXED host
//! port: `5432:5432`, `6379:6379`, `24678:24678`, and a dozen more. So two mx
//! projects could be started one after the other and the second died on
//! `Bind for 0.0.0.0:5432 failed: port is already allocated` — with nothing in
//! the project to edit short of hand-patching a shipped template.
//!
//! `docker compose config` cannot see this: a pinned publish is perfectly valid
//! compose and renders clean. It only explodes at `up` time, and only once a
//! SECOND stack wants the same number. So the check reads the shipped source.
//!
//! # The design call, per port class
//!
//! Every published host port must be env-parameterized — `${NAME:-<default>}` —
//! so the number is a default rather than a law. What the default should be
//! divides on one question: *who dials this port?*
//!
//! **INFRA ports are dialed by tooling, so they default EPHEMERAL (`:-0`).**
//! Host port `0` tells Docker to pick a free one, which can never collide.
//! Nothing a developer types embeds the number, because nothing could: `psql`,
//! `redis-cli`, a debugger attach and a `curl` of a metrics endpoint are all
//! ad-hoc, and the number is one command away:
//!
//! ```text
//! docker compose -p <project> port db 5432     # → 0.0.0.0:54317
//! ```
//!
//! The ports in this class, and why each is tooling-only:
//!
//!   * **5432** (postgres) and **6379** (redis) — the publish exists for GUI
//!     clients (pgAdmin, DBeaver, RedisInsight). Every service that talks to
//!     them in anger does so over the compose network, by service name.
//!   * **9090** (rust-worker metrics) — scraped by a `curl` or a Prometheus you
//!     point at it; the container's own healthcheck uses `localhost:9090`
//!     INSIDE the container and is unaffected by the host side.
//!   * **9229** (Node inspector) — a debugger attach, chosen in the IDE.
//!   * **13714** (laravel Inertia SSR) — never left the container in the first
//!     place: `config/inertia.php` points PHP at `127.0.0.1:13714`. The publish
//!     is for inspecting the SSR process, nothing more.
//!   * **5173** (laravel Vite) — the browser reaches Vite SAME-ORIGIN: the
//!     recipe's own nginx proxies `/build/` and `/__vite_hmr` to
//!     `127.0.0.1:5173` (docker/system/app/etc/nginx/http.d/app.conf), and the
//!     page itself arrives through the mx router. The host publish is an
//!     inspection convenience, not the HMR path.
//!
//! **BROWSER-FACING ports are dialed by a URL, so they default PINNED.**
//! A live-reload websocket whose port moves every boot is a live-reload
//! websocket the page cannot find: the client embeds the number at build time
//! (`reload-port = 3001` in leptos's Cargo.toml) or has it compiled in (zola's
//! 1024), and no amount of discovery helps a browser that has already been
//! handed the URL. These keep today's number as the DEFAULT so a lone stack's
//! documented UX is untouched, and gain an override so a second stack moves
//! instead of dying:
//!
//!   * **3001** (cargo-leptos reload) — `LEPTOS_RELOAD_HOST_PORT`, which drives
//!     the container side and the service's own `LEPTOS_RELOAD_PORT` as well,
//!     because the number the server binds is the number it injects into the
//!     page. cargo-leptos runs the reload channel as its OWN server on that port,
//!     not through the app, so the browser really does dial it directly.
//!   * **1024** (zola live reload) — `ZOLA_LIVERELOAD_PORT`. `zola serve` injects
//!     `livereload.js?port=1024` into the page and has no flag to change the
//!     number, so the container side stays 1024 and only the host side moves; an
//!     overriding second stack boots with its live reload degraded, which is
//!     strictly better than not booting.
//!   * **3000** / **80** / **443** / **8080** — an app's or a proxy's front door.
//!     Already parameterized for the router (`MX_ROUTER_DASHBOARD_PORT`) and the
//!     legacy `app.yml` (`APP_PORT`); the rest of the legacy reverse-proxy
//!     samples (`templates/docker/compose/{nginx,traefik}.yml`, bd:mech-crate-qpy)
//!     join them so a machine already running the mx router on 80 can still
//!     render them.
//!
//! **And three publishes were neither class: they were squatting.** astro, nuxt
//! and the repo's own site all published 24678 labelled "HMR websocket". Nothing
//! was behind it. `/proc/net/tcp` inside a live astro dev container lists 4321
//! and nothing else; inside nuxt, 3000 and nothing else — before and after
//! serving a request. Both frameworks carry HMR over the app's own port, and the
//! router upgrades it same-origin: a `vite-hmr` websocket handshake through
//! mx-router answers `HTTP/1.1 101 Switching Protocols` for each.
//!
//! The cost was real even so, because Docker binds a published host port whether
//! or not anything answers behind it: an astro stack holding 24678 killed the
//! next stack's boot outright (`Bind for 0.0.0.0:24678 failed: port is already
//! allocated`, reproduced live). Parameterizing a publish that exists for nothing
//! would have been the wrong repair. Those three are gone, along with the
//! `EXPOSE 24678` in each dev Dockerfile that made them look justified.
//!
//! # Why the classification is asserted, not just the parameterization
//!
//! `${DB_HOST_PORT:-5432}` is env-parameterized and still collides out of the
//! box, which is the whole bug. So the net checks BOTH halves: the shape, and
//! that an infra port's default is `0` while a browser-facing port's default is
//! a real number. A container port absent from the table fails deliberately —
//! adding a publish is a design call, and this is where it gets recorded.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Directories swept: the shipped templates, plus the repo's own site infra,
/// which was scaffolded from the astro recipe and drifts with it (the same pair
/// of roots bd:mech-crate-q1w's env sweep walks, for the same reason).
fn swept_roots() -> Vec<PathBuf> {
    vec![
        repo_root().join("templates"),
        repo_root().join("site/docker/compose"),
    ]
}

/// Template fragments carry `{{PLACEHOLDER}}` tokens, including where YAML wants
/// a mapping key (`{{SERVICE_NAME}}:`), so the shipped file is not YAML until the
/// installer has expanded it. Swap each token for an inert scalar to read the
/// file with real YAML semantics instead of sniffing lines.
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

/// One `ports:` entry that reserves a host port.
#[derive(Debug)]
struct Publish {
    /// Repo-relative path of the compose file.
    rel: String,
    /// Compose service the entry belongs to.
    service: String,
    /// The entry exactly as shipped.
    spec: String,
    /// Host side of the mapping (after any bind address).
    host: String,
    /// Container side, protocol suffix stripped and any interpolation resolved to
    /// its default (rust-leptos drives both sides from one variable, so the
    /// container side is itself a `${…:-3001}`).
    container: String,
}

/// Split a short-syntax ports spec on `:`, ignoring the `:` inside a `${…}`
/// interpolation — `${DB_HOST_PORT:-0}:5432` is two fields, not three.
fn split_spec(spec: &str) -> Vec<String> {
    let mut fields = vec![String::new()];
    let mut depth = 0usize;
    let mut chars = spec.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '$' if chars.peek() == Some(&'{') => {
                chars.next();
                depth += 1;
                fields.last_mut().unwrap().push_str("${");
            }
            '}' if depth > 0 => {
                depth -= 1;
                fields.last_mut().unwrap().push('}');
            }
            ':' if depth == 0 => fields.push(String::new()),
            _ => fields.last_mut().unwrap().push(c),
        }
    }
    fields
}

/// Every host-port reservation across the swept roots.
///
/// Container-only entries (`- 3000`, a bare container port) reserve nothing on
/// the host and are skipped.
fn published_ports() -> Vec<Publish> {
    let mut out: Vec<Publish> = Vec::new();

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

    let root = repo_root();
    let mut files = Vec::new();
    for dir in swept_roots() {
        assert!(
            dir.is_dir(),
            "setup: swept root {} does not exist",
            dir.display()
        );
        walk(&dir, &mut files);
    }
    files.sort();

    for path in files {
        let body = std::fs::read_to_string(&path)
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
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        for (name, service_body) in services {
            // Undo neutralize_placeholders for the report, so an offender line
            // names the key as the shipped file spells it.
            let service = match name.as_str().unwrap_or("<non-string key>") {
                "MX_PLACEHOLDER" => "{{SERVICE_NAME}}".to_string(),
                other => other.to_string(),
            };
            let Some(entries) = service_body.get("ports").and_then(|v| v.as_sequence()) else {
                continue;
            };
            for entry in entries {
                let (spec, host, container) = match entry {
                    // Long syntax: `- target: 80` / `published: "8080"`.
                    serde_yaml::Value::Mapping(m) => {
                        let scalar = |k: &str| {
                            m.get(serde_yaml::Value::from(k)).map(|v| match v {
                                serde_yaml::Value::String(s) => s.clone(),
                                other => serde_yaml::to_string(other)
                                    .unwrap_or_default()
                                    .trim()
                                    .to_string(),
                            })
                        };
                        let Some(published) = scalar("published") else {
                            continue; // target only: no host reservation.
                        };
                        let target = scalar("target").unwrap_or_default();
                        (
                            format!("target: {target}, published: {published}"),
                            published,
                            target,
                        )
                    }
                    // Short syntax, string or bare int.
                    other => {
                        let spec = match other {
                            serde_yaml::Value::String(s) => s.clone(),
                            v => serde_yaml::to_string(v)
                                .unwrap_or_default()
                                .trim()
                                .to_string(),
                        };
                        let fields = split_spec(&spec);
                        match fields.len() {
                            // `- 3000`: container only.
                            1 => continue,
                            2 => (spec.clone(), fields[0].clone(), fields[1].clone()),
                            // `- 127.0.0.1:8001:8001`: bind address first.
                            3 => (spec.clone(), fields[1].clone(), fields[2].clone()),
                            _ => panic!("{rel}: cannot read ports entry `{spec}`"),
                        }
                    }
                };
                let container = container.split('/').next().unwrap_or_default().trim();
                let container = env_default(container).unwrap_or(container).to_string();
                out.push(Publish {
                    rel: rel.clone(),
                    service: service.clone(),
                    spec,
                    host: host.trim().to_string(),
                    container,
                });
            }
        }
    }

    assert!(
        out.len() >= 20,
        "setup: the shipped compose files publish ~25 host ports; the walker found \
         {} — it has stopped reading `ports:` blocks and every check below would \
         pass vacuously",
        out.len()
    );
    out
}

/// `${NAME:-<default>}` → `<default>`. `None` for any other shape.
fn env_default(host: &str) -> Option<&str> {
    let inner = host.strip_prefix("${")?.strip_suffix('}')?;
    let (_name, default) = inner.split_once(":-")?;
    Some(default)
}

#[derive(Debug, PartialEq, Eq)]
enum Class {
    /// Dialed by tooling, on demand. Ephemeral (`:-0`), discovered with
    /// `docker compose -p <project> port <service> <container port>`.
    Infra,
    /// Dialed by a URL a browser or a compiled-in client already holds. Keeps
    /// today's number as the default; overridable so a second stack can move.
    BrowserFacing,
}

/// Container port → class, with the reason. See the module header for the full
/// argument; this table is what the tests enforce.
const PORT_CLASSES: &[(&str, Class, &str)] = &[
    ("5432", Class::Infra, "postgres, for GUI clients"),
    ("6379", Class::Infra, "redis, for GUI clients"),
    ("9090", Class::Infra, "worker metrics, curl/Prometheus"),
    ("9229", Class::Infra, "Node inspector, IDE attach"),
    (
        "13714",
        Class::Infra,
        "laravel Inertia SSR, container-internal (127.0.0.1)",
    ),
    (
        "5173",
        Class::Infra,
        "laravel Vite, reached same-origin via the recipe's nginx",
    ),
    ("3000", Class::BrowserFacing, "legacy app.yml front door"),
    (
        "3001",
        Class::BrowserFacing,
        "cargo-leptos reload websocket, its own server",
    ),
    (
        "1024",
        Class::BrowserFacing,
        "zola live-reload websocket, number injected into the page",
    ),
    ("80", Class::BrowserFacing, "HTTP front door"),
    ("443", Class::BrowserFacing, "HTTPS front door"),
    ("8080", Class::BrowserFacing, "Traefik dashboard"),
];

fn class_of(container: &str) -> Option<&'static Class> {
    PORT_CLASSES
        .iter()
        .find(|(port, _, _)| *port == container)
        .map(|(_, class, _)| class)
}

/// bd:mech-crate-1a0 — the core net. A literal host port is a claim on the whole
/// workstation, so no shipped compose file may make one: every publish has to
/// route through the environment, where a second stack can redirect it.
#[test]
fn every_published_host_port_is_env_overridable() {
    let mut offenders: Vec<String> = Vec::new();

    for p in published_ports() {
        if env_default(&p.host).is_none() {
            offenders.push(format!(
                "{}: service `{}` publishes `{}` — host side `{}` is not \
                 `${{NAME:-<default>}}`",
                p.rel, p.service, p.spec, p.host
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "a fixed host port is a workstation-wide singleton: the second mx stack to \
         want it dies on `port is already allocated`, with nothing in the project to \
         edit. Publish through the environment instead (`${{DB_HOST_PORT:-0}}:5432` \
         for tooling ports, `${{HMR_HOST_PORT:-24678}}:24678` for browser-facing \
         ones):\n{}",
        offenders
            .iter()
            .map(|o| format!("  {o}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The shape alone is not the fix: `${DB_HOST_PORT:-5432}` is parameterized and
/// still collides on a bare `make dev`. Tooling ports must default to the
/// ephemeral `0`, and browser-facing ports must default to a real number so a
/// lone stack keeps the UX its docs promise.
#[test]
fn every_host_port_default_matches_its_class() {
    let mut offenders: Vec<String> = Vec::new();

    for p in published_ports() {
        let Some(default) = env_default(&p.host) else {
            continue; // reported by every_published_host_port_is_env_overridable
        };
        let at = format!("{}: service `{}` → `{}`", p.rel, p.service, p.spec);
        match class_of(&p.container) {
            Some(Class::Infra) => {
                if default != "0" {
                    offenders.push(format!(
                        "{at}: container port {} is tooling-only, so its host default \
                         must be the ephemeral `0`, not `{default}` — a pinned default \
                         still collides on an unaided second `make dev`",
                        p.container
                    ));
                }
            }
            Some(Class::BrowserFacing) => {
                if default == "0" || default.parse::<u16>().is_err() {
                    offenders.push(format!(
                        "{at}: container port {} is dialed by a URL the page already \
                         holds, so its host default must be a fixed port number, not \
                         `{default}`",
                        p.container
                    ));
                }
            }
            None => offenders.push(format!(
                "{at}: container port {} is not classified. Publishing a host port is a \
                 design call — decide whether it is tooling (ephemeral `:-0`) or \
                 browser-facing (pinned default) and add it to PORT_CLASSES with the \
                 reason",
                p.container
            )),
        }
    }

    assert!(
        offenders.is_empty(),
        "host-port defaults that do not match their class:\n{}",
        offenders
            .iter()
            .map(|o| format!("  {o}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The table may not rot: a class recorded for a port nothing publishes any more
/// is a design call about nothing, and hides that the port was retired.
#[test]
fn the_port_class_table_stays_honest() {
    let published = published_ports();
    let mut stale: Vec<String> = Vec::new();

    for (port, class, why) in PORT_CLASSES {
        if !published.iter().any(|p| p.container == *port) {
            stale.push(format!("{port} ({class:?}: {why})"));
        }
    }

    assert!(
        stale.is_empty(),
        "PORT_CLASSES classifies container ports nothing publishes any more — drop \
         the rows: {}",
        stale.join(", ")
    );
}
