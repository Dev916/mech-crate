//! Compose hygiene of the shipped templates (bd:mech-crate-xhf + 4n4 + v6z).
//!
//! Source-level checks on `templates/`, deliberately NOT delegated to
//! `docker compose config` — `config` is blind to every defect in here:
//!
//!   * a pinned `container_name` is perfectly valid compose and renders clean.
//!     It only explodes at `docker run` time, and only once a SECOND stack wants
//!     the same name: container names are a Docker-daemon-wide namespace, so two
//!     mx projects both declaring `container_name: db` collide with
//!     "Conflict. The container name "/db" is already in use". Pinning
//!     `COMPOSE_PROJECT_NAME` (wave-1, bd:mech-crate-71u) isolated projects'
//!     compose namespaces but cannot isolate a name the file hard-codes.
//!   * `external: true` tells compose "this network is someone else's to
//!     create". `config` renders it without looking, so a network nothing
//!     creates is a clean `config` and a failed `up`.
//!
//! Parsing is line-based on purpose. These files are *templates* — they carry
//! `{{SERVICE_NAME}}` where YAML wants a mapping key, so they are not reliably
//! YAML until the installer has expanded them. The shapes under test (a
//! `container_name:` line, a top-level `networks:` block, a `healthcheck.test`
//! line) are flat enough to read directly, and reading the shipped source is
//! what makes the check cover every file rather than only the ones some other
//! test happens to install.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn templates_root() -> PathBuf {
    repo_root().join("templates")
}

/// Every compose file shipped under `templates/`, repo-relative path + body.
///
/// A compose file is one that lands in a project's `docker/compose/` directory,
/// plus the router's own top-level `docker-compose.yml`. The other YAML under
/// `templates/` (traefik config, a recipe's GitHub workflows) is not compose.
fn shipped_compose_files() -> Vec<(String, String)> {
    let root = templates_root();
    let mut out: Vec<(String, String)> = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("setup: read {}: {e}", dir.display()))
            .flatten()
        {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().is_none_or(|e| e != "yml" && e != "yaml") {
                continue;
            }
            let rel = path
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let is_compose = rel.contains("docker/compose/")
                || path
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("docker-compose."));
            if !is_compose {
                continue;
            }
            let body = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("setup: read {}: {e}", path.display()));
            out.push((format!("templates/{rel}"), body));
        }
    }
    out.sort();
    assert!(
        out.len() >= 30,
        "setup: expected the shipped compose corpus, found {} files",
        out.len()
    );
    out
}

// ── container_name is a global namespace (bd:mech-crate-xhf) ─────────────────

/// The one compose file allowed to pin a container name, and why.
///
/// The global router is a workstation singleton by design: exactly one
/// `mx-router` per machine, started by `mx router up`, addressed by that name by
/// `mx router status` and by `scripts/test-e2e.sh`. There is no second instance
/// for it to collide with — unlike a project's `db`, of which a developer may
/// have a dozen.
const CONTAINER_NAME_ALLOWED: &[&str] = &["templates/router/docker-compose.yml"];

/// bd:mech-crate-xhf — with compose project names pinned (wave-1), two projects
/// whose compose files both say `container_name: db` cannot both boot: the
/// second `up` fails on the Docker-wide name. Dropping the pin hands naming back
/// to compose, which derives `<project>-<service>-<index>` — unique per project
/// for free. Service names are unchanged, so `docker compose exec db` and
/// `depends_on: db` keep working; only scripts that addressed a container by a
/// fixed name had to move (they now go through `docker compose -p … exec`).
#[test]
fn no_shipped_compose_file_pins_a_container_name() {
    let mut offenders: Vec<String> = Vec::new();

    for (rel, body) in shipped_compose_files() {
        if CONTAINER_NAME_ALLOWED.contains(&rel.as_str()) {
            continue;
        }
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }
            if trimmed.starts_with("container_name:") {
                offenders.push(format!("{rel}:{}: {}", i + 1, line.trim()));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "container names are a Docker-daemon-wide namespace: two mx projects that \
         both pin one cannot run at the same time. Drop these and let compose \
         derive `<project>-<service>-<index>`:\n{}",
        offenders
            .iter()
            .map(|o| format!("  {o}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The allowance may not rot into a blanket exemption: every file it lists must
/// exist and must still actually pin a name.
#[test]
fn the_container_name_allowance_stays_honest() {
    for rel in CONTAINER_NAME_ALLOWED {
        let path = repo_root().join(rel);
        let body = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("CONTAINER_NAME_ALLOWED lists {rel}, which does not exist"));
        assert!(
            body.lines()
                .any(|l| l.trim_start().starts_with("container_name:")),
            "CONTAINER_NAME_ALLOWED lists {rel}, but it no longer pins a container \
             name — drop the allowance"
        );
    }
}

// ── External networks must be ones mx creates (bd:mech-crate-4n4) ────────────

/// External networks mx itself brings into existence, with the code that does it.
///
/// `external: true` is a promise that something else created the network. These
/// are the only two promises mx can keep.
const EXTERNAL_NETWORKS_MX_CREATES: &[(&str, &str)] = &[
    // `mx router up` → Router::ensure_network() → `docker network create`.
    ("devmesh-traefik", "crates/mx-lib/src/router/mod.rs"),
    // `make init` → scripts/init.sh → `docker network create "$NETWORK_NAME"`.
    ("mech-network", "templates/scripts/init.sh"),
];

/// Top-level `networks:` entries of one compose file that declare `external: true`,
/// as the network's *effective* name — the `name:` override when present (compose
/// looks the network up by that, not by the mapping key), else the key itself.
fn external_network_names(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_networks = false;
    // (effective name, saw `external: true`) for the entry being read.
    let mut current: Option<(String, bool)> = None;

    let flush = |current: &mut Option<(String, bool)>, out: &mut Vec<String>| {
        if let Some((name, external)) = current.take() {
            if external {
                out.push(name);
            }
        }
    };

    for line in body.lines() {
        if line.trim_start().starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if indent == 0 {
            flush(&mut current, &mut out);
            in_networks = line.trim_end() == "networks:";
            continue;
        }
        if !in_networks {
            continue;
        }
        let trimmed = line.trim();
        if indent == 2 {
            // A new network entry: `<key>:`.
            flush(&mut current, &mut out);
            if let Some(key) = trimmed.strip_suffix(':') {
                current = Some((key.to_string(), false));
            }
            continue;
        }
        // Attributes of the entry in hand.
        if let Some((name, external)) = current.as_mut() {
            if let Some(v) = trimmed.strip_prefix("name:") {
                *name = v.trim().to_string();
            } else if trimmed.starts_with("external:") {
                *external = trimmed.split_once(':').unwrap().1.trim() == "true";
            }
        }
    }
    flush(&mut current, &mut out);
    out
}

/// bd:mech-crate-4n4 — the rust-worker recipe joined `networks: internal:` with
/// `external: true`, and nothing in mx ever created a network called `internal`.
/// `docker compose config` resolved it happily; `make dev` died at runtime with
/// `network internal declared as external, but could not be found`. The fix was
/// to drop the network entirely so the worker sits on compose's per-project
/// default network, which is where its `db` and `redis` siblings already are.
///
/// The net for the class: every `external: true` network any shipped compose file
/// joins must be one mx actually creates.
#[test]
fn every_external_network_is_one_mx_creates() {
    let known: Vec<&str> = EXTERNAL_NETWORKS_MX_CREATES
        .iter()
        .map(|(name, _)| *name)
        .collect();
    let mut phantoms: Vec<String> = Vec::new();
    let mut seen = 0;

    for (rel, body) in shipped_compose_files() {
        for name in external_network_names(&body) {
            seen += 1;
            if !known.contains(&name.as_str()) {
                phantoms.push(format!("{rel}: external network `{name}`"));
            }
        }
    }

    assert!(
        seen > 0,
        "setup: the parser found no external networks at all — it has stopped reading \
         the `networks:` block"
    );
    assert!(
        phantoms.is_empty(),
        "these compose files join an external network nothing in mx creates, so \
         `docker compose config` passes and `up` fails at runtime. Known-created: \
         {known:?}\n{}",
        phantoms
            .iter()
            .map(|p| format!("  {p}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The allowlist is only as good as its evidence: each name must still be created
/// by the file claimed for it, so retiring a creator fails here instead of
/// quietly re-opening the phantom-network hole.
#[test]
fn every_allowed_external_network_still_has_a_creator() {
    for (name, creator) in EXTERNAL_NETWORKS_MX_CREATES {
        let path = repo_root().join(creator);
        let body = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("setup: read claimed creator {creator}: {e}"));
        assert!(
            body.contains(name),
            "{creator} is claimed to create the external network `{name}` but never \
             mentions it"
        );
        assert!(
            body.contains("network create") || body.contains("create_network"),
            "{creator} is claimed to create the external network `{name}` but runs no \
             network creation"
        );
    }
}

// ── The db healthcheck must probe the real role (bd:mech-crate-v6z) ──────────

/// bd:mech-crate-v6z — every shipped `db.yml` probed readiness with
/// `pg_isready -U ${DB_USER:-postgres}`. `${…}` in a compose file is interpolated
/// by the compose CLI from the compose PROJECT DIRECTORY's `.env`, which no mx
/// scaffold has — so the fallback ALWAYS won and the probe asked about the role
/// `postgres`, which stopped existing once T2 (bd:mech-crate-rqc) started
/// generating per-project credentials. And `pg_isready` answers "server is up"
/// for a role that does not exist, so the check passed vacuously.
///
/// Two requirements, both asserted: read the role from the CONTAINER's runtime
/// environment (`$${…}`, the escape that defers interpolation past the CLI) and
/// actually authenticate as it.
#[test]
fn the_db_healthcheck_probes_the_configured_role_at_container_runtime() {
    let mut checked = 0;
    let mut failures: Vec<String> = Vec::new();

    for (rel, body) in shipped_compose_files() {
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || !trimmed.contains("pg_isready") {
                continue;
            }
            checked += 1;
            let at = format!("{rel}:{}", i + 1);
            // `${DB_USER…}` without the doubled `$` is resolved by the compose CLI
            // against a `.env` that is not there.
            if trimmed.contains("${DB_USER") {
                failures.push(format!(
                    "{at}: interpolates `${{DB_USER}}` at compose-CLI time, where it is \
                     always empty — use `$${{POSTGRES_USER}}` to read the container's \
                     own environment"
                ));
            }
            if !trimmed.contains("$${POSTGRES_USER") {
                failures.push(format!(
                    "{at}: does not read the role from the container runtime \
                     (`$${{POSTGRES_USER}}`)"
                ));
            }
            // pg_isready reports PQPING_OK for a nonexistent role — the server
            // answered, which is all it asks. Authenticating takes a real query.
            if !trimmed.contains("psql") {
                failures.push(format!(
                    "{at}: `pg_isready` alone cannot see a broken role (it reports the \
                     server up even when the role does not exist) — authenticate too"
                ));
            }
        }
    }

    assert!(
        checked >= 4,
        "setup: expected every shipped db.yml to carry a postgres healthcheck, saw \
         {checked}"
    );
    assert!(
        failures.is_empty(),
        "postgres healthchecks that cannot fail on a genuinely broken role:\n{}",
        failures
            .iter()
            .map(|f| format!("  {f}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
