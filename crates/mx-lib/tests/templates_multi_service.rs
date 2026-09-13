//! Multi-service selection — `make dev s="api site"` (bd:mech-crate-3kq).
//!
//! The pitch is "run only the services you need", but the shipped templates
//! could only ever run ONE: `templates/make/dev.mk` expanded the `s=` value
//! unquoted, so the second name arrived as a make goal ("No rule to make target
//! 'site'"), and `compose_context_files` in `templates/scripts/.bashrc` resolved
//! exactly one `docker/compose/<service>.yml`.
//!
//! Both halves are held here: a static sweep that every `.mk` pass-through is
//! quoted (so a list survives the make layer as ONE argument), and behavioral
//! runs of the real shipped scripts against a `docker` stub on `PATH` that
//! records argv — i.e. assertions on the exact command line `make dev` builds.
//!
//! The matrix this file pins (which targets take a list) is also the one
//! `scripts/.bashrc` documents:
//!
//! | target                        | list? | why                               |
//! |-------------------------------|-------|-----------------------------------|
//! | dev, up, down, stop, restart  | yes   | compose takes N service operands  |
//! | logs                          | yes   | compose takes N service operands  |
//! | build, run, exec, sh/bash     | no    | one image / one container         |

use std::path::{Path, PathBuf};
use std::process::Output;

fn templates_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates")
}

fn scripts_root() -> PathBuf {
    templates_root().join("scripts")
}

fn make_root() -> PathBuf {
    templates_root().join("make")
}

// ── Static sweep: the make layer must not word-split a service list ──────────

/// Shipped make modules, sorted: `(file name, contents)`.
fn make_modules() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = std::fs::read_dir(make_root())
        .expect("setup: templates/make must exist")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("mk"))
        .map(|p| {
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            let body = std::fs::read_to_string(&p)
                .unwrap_or_else(|e| panic!("setup: read {}: {e}", p.display()));
            (name, body)
        })
        .collect();
    out.sort();
    out
}

/// Every make expansion that can carry a whitespace-separated value. Unquoted,
/// `make dev s="api site"` becomes `make _dev service=api site` — "site" turns
/// into a goal and make dies with "No rule to make target".
const SPLITTABLE_EXPANSIONS: &[&str] = &[
    "$(call get_service)",
    "$(call get_service_optional)",
    "$(service)",
    "$(call get_cmd)",
    "$(cmd)",
];

#[test]
fn make_modules_quote_every_value_that_can_contain_spaces() {
    let mut offenders: Vec<String> = Vec::new();
    let mut checked = 0;

    for (name, body) in make_modules() {
        for line in body.lines() {
            // Recipe lines only (tab-indented): those are the pass-throughs that
            // reach a shell. The `get_service`/`get_cmd` definitions in
            // common.mk expand the value inside `$(if …)`, where make itself —
            // not a shell — does the reading, so quotes there would become part
            // of the value.
            if !line.starts_with('\t') {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            for needle in SPLITTABLE_EXPANSIONS {
                let uses = line.matches(needle).count();
                if uses == 0 {
                    continue;
                }
                checked += uses;
                let quoted = line.matches(&format!("\"{needle}\"")).count();
                if quoted < uses {
                    offenders.push(format!("make/{name}: {trimmed}"));
                }
            }
        }
    }

    assert!(
        checked >= 10,
        "setup: expected the make modules to pass a service/cmd value through \
         at least 10 times, saw {checked}"
    );
    assert!(
        offenders.is_empty(),
        "these make pass-throughs are unquoted, so a value with a space \
         (`s=\"api site\"`) word-splits into extra make goals / script argv:\n{}",
        offenders.join("\n")
    );
}

/// The specific chain the defect was reported against, spelled out so a future
/// edit that drops either pair of quotes fails with the reason attached.
#[test]
fn the_dev_chain_passes_a_service_list_as_one_argument() {
    let dev_mk = std::fs::read_to_string(make_root().join("dev.mk"))
        .expect("setup: templates/make/dev.mk must exist");

    assert!(
        dev_mk.contains(r#"service="$(call get_service_optional)""#),
        "dev.mk must quote the s= pass-through to the recursive make:\n{dev_mk}"
    );
    assert!(
        dev_mk.contains(r#"./scripts/dev.sh "$(service)""#),
        "dev.mk must hand dev.sh the whole list as ONE argument:\n{dev_mk}"
    );
}

// ── Static sweep: the list matrix ────────────────────────────────────────────

/// Scripts whose underlying compose verb takes N service operands.
const LIST_CAPABLE_SCRIPTS: &[&str] = &["dev.sh", "up.sh", "down.sh", "stop.sh", "logs.sh"];

/// Scripts that act on exactly one image/container and must refuse a list.
const SINGLE_SERVICE_SCRIPTS: &[&str] = &["build.sh", "run.sh", "exec.sh", "sh.sh"];

#[test]
fn single_service_scripts_guard_against_a_list_and_list_capable_ones_do_not() {
    for name in SINGLE_SERVICE_SCRIPTS {
        let body = std::fs::read_to_string(scripts_root().join(name))
            .unwrap_or_else(|e| panic!("setup: read scripts/{name}: {e}"));
        assert!(
            body.contains("mech_require_single_service"),
            "scripts/{name} acts on a single container/image, so it must refuse \
             a service list loudly via mech_require_single_service"
        );
    }

    for name in LIST_CAPABLE_SCRIPTS {
        let body = std::fs::read_to_string(scripts_root().join(name))
            .unwrap_or_else(|e| panic!("setup: read scripts/{name}: {e}"));
        assert!(
            !body.contains("mech_require_single_service"),
            "scripts/{name} passes its operands straight to compose, which takes \
             a list — it must not refuse one"
        );
    }

    let bashrc = std::fs::read_to_string(scripts_root().join(".bashrc"))
        .expect("setup: templates/scripts/.bashrc must exist");
    assert!(
        bashrc.contains("mech_require_single_service()"),
        ".bashrc is the single place the refusal message lives"
    );
}

// ── Fixtures: a runnable project skeleton + a recording `docker` ─────────────

/// A `docker` stub on `PATH` that appends its argv to `calls.log`.
fn stub_docker(dir: &Path) -> PathBuf {
    let bin = dir.join("stub-bin");
    std::fs::create_dir_all(&bin).unwrap();
    let log = dir.join("calls.log");
    std::fs::write(
        bin.join("docker"),
        format!("#!/bin/sh\necho \"$@\" >> '{}'\nexit 0\n", log.display()),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(bin.join("docker"), std::fs::Permissions::from_mode(0o755))
            .unwrap();
    }
    bin
}

/// Lay down a project whose `Makefile`, `make/*.mk` and `scripts/` are the real
/// shipped templates, with one compose file + dev override per service.
fn scaffold_project(root: &Path, services: &[&str]) {
    for dir in [
        "make",
        "scripts",
        "docker/compose",
        "docker/.config",
        "tmp/up",
    ] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }

    std::fs::copy(
        templates_root().join("Makefile.template"),
        root.join("Makefile"),
    )
    .expect("setup: copy Makefile.template");

    for entry in std::fs::read_dir(make_root()).unwrap().flatten() {
        let src = entry.path();
        if src.extension().and_then(|e| e.to_str()) == Some("mk") {
            std::fs::copy(&src, root.join("make").join(src.file_name().unwrap())).unwrap();
        }
    }

    for entry in std::fs::read_dir(scripts_root()).unwrap().flatten() {
        let src = entry.path();
        if !src.is_file() {
            continue;
        }
        let dest = root.join("scripts").join(src.file_name().unwrap());
        std::fs::copy(&src, &dest).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }

    for service in services {
        std::fs::write(
            root.join(format!("docker/compose/{service}.yml")),
            format!("services:\n  {service}:\n    image: alpine:3\n"),
        )
        .unwrap();
        std::fs::write(
            root.join(format!("docker/compose/{service}.dev.yml")),
            format!("services:\n  {service}:\n    environment:\n      - DEV=1\n"),
        )
        .unwrap();
    }
}

/// Run `bash -c` in `cwd` with the stub dir first on `PATH`.
fn run_in(cwd: &Path, path_prefix: Option<&Path>, script: &str) -> Output {
    let mut cmd = std::process::Command::new("bash");
    cmd.arg("-c").arg(script).current_dir(cwd);
    if let Some(prefix) = path_prefix {
        cmd.env(
            "PATH",
            format!(
                "{}:{}",
                prefix.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        );
    }
    // An inherited COMPOSE_PROJECT_NAME would mask the per-project derivation.
    cmd.env_remove("COMPOSE_PROJECT_NAME");
    cmd.output().expect("run bash")
}

fn combined(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn run_ok(cwd: &Path, path_prefix: Option<&Path>, script: &str) -> String {
    let out = run_in(cwd, path_prefix, script);
    assert!(
        out.status.success(),
        "`{script}` failed ({:?}) in {}:\n{}",
        out.status.code(),
        cwd.display(),
        combined(&out)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// The `docker compose …` argv lines the stub recorded, oldest first.
fn compose_calls(root: &Path) -> Vec<String> {
    let log = std::fs::read_to_string(root.join("calls.log")).unwrap_or_default();
    log.lines()
        .filter(|l| l.starts_with("compose "))
        .map(|l| l.to_string())
        .collect()
}

fn count(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

// ── Behavioral: compose_context_files resolves a whole list ──────────────────

#[test]
fn compose_context_files_resolves_every_service_in_a_list() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("ctx");
    scaffold_project(&root, &["api", "site"]);

    let files = run_ok(
        &root,
        None,
        r#"source ./scripts/.bashrc && compose_context_files "api site" "true""#,
    );

    for expected in [
        "-f docker/compose/api.yml",
        "-f docker/compose/site.yml",
        "-f docker/compose/api.dev.yml",
        "-f docker/compose/site.dev.yml",
    ] {
        assert!(
            files.contains(expected),
            "a two-service context must carry `{expected}`; got: {files}"
        );
    }
}

#[test]
fn compose_context_files_emits_each_file_once() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("dedupe");
    scaffold_project(&root, &["api"]);
    // Context left behind by a previous run — the same file must not come back
    // a second time just because it was asked for again.
    std::fs::write(root.join("tmp/up/up-1.txt"), "-f docker/compose/api.yml\n").unwrap();

    let files = run_ok(
        &root,
        None,
        r#"source ./scripts/.bashrc && compose_context_files "api api" "true""#,
    );

    assert_eq!(
        count(&files, "-f docker/compose/api.yml"),
        1,
        "api.yml should appear once across the request and the saved context: {files}"
    );
    assert_eq!(
        count(&files, "-f docker/compose/api.dev.yml"),
        1,
        "the dev override should appear once: {files}"
    );
}

#[test]
fn compose_context_files_names_an_unknown_service_in_a_list() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("unknown");
    scaffold_project(&root, &["api"]);

    let out = run_in(
        &root,
        None,
        r#"source ./scripts/.bashrc && compose_context_files "api nope" "true""#,
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.trim().is_empty(),
        "a list with an unknown name must not resolve to a partial context \
         (that would silently start a subset): {stdout}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("nope"),
        "the unknown name itself must be reported, not just 'no service found': {stderr}"
    );
    assert!(
        !stderr.contains("'api'") && !stderr.contains("\"api\""),
        "the known name is not the problem and must not be blamed: {stderr}"
    );
}

// ── Behavioral: the whole `make dev s="a b"` chain ───────────────────────────

#[test]
fn make_dev_with_a_service_list_starts_exactly_the_selected_pair() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("pair");
    scaffold_project(&root, &["api", "site", "worker"]);
    let stub = stub_docker(&root);

    run_ok(&root, Some(&stub), r#"make dev s="api site""#);

    let calls = compose_calls(&root);
    let up: Vec<&String> = calls.iter().filter(|c| c.contains("up -d")).collect();
    assert_eq!(
        up.len(),
        1,
        "expected exactly one `up -d` call; calls:\n{}",
        calls.join("\n")
    );
    let up = up[0];

    for expected in [
        "-f docker/compose/api.yml",
        "-f docker/compose/site.yml",
        "-f docker/compose/api.dev.yml",
        "-f docker/compose/site.dev.yml",
    ] {
        assert!(up.contains(expected), "`{expected}` missing from: {up}");
    }
    assert!(
        !up.contains("worker"),
        "only the selected services may be started: {up}"
    );
    assert!(
        up.trim_end().ends_with("up -d api site"),
        "both names must reach compose as service operands: {up}"
    );
    // Wave-1 invariant (bd:mech-crate-71u) holds for multi-service calls too.
    assert!(
        up.contains("-p pair "),
        "a multi-service call must still pin this project's compose name: {up}"
    );
}

#[test]
fn make_dev_with_a_single_service_is_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("single");
    scaffold_project(&root, &["api", "site"]);
    let stub = stub_docker(&root);

    run_ok(&root, Some(&stub), "make dev s=api");

    let calls = compose_calls(&root);
    let up = calls
        .iter()
        .find(|c| c.contains("up -d"))
        .unwrap_or_else(|| panic!("no `up -d` recorded; calls:\n{}", calls.join("\n")));

    assert!(up.contains("-f docker/compose/api.yml"), "{up}");
    assert!(up.contains("-f docker/compose/api.dev.yml"), "{up}");
    assert!(
        !up.contains("site"),
        "a single-service request must not pull in another service: {up}"
    );
    assert!(up.trim_end().ends_with("up -d api"), "{up}");
    assert!(up.contains("-p single "), "{up}");
}

#[test]
fn make_dev_with_an_unknown_name_in_a_list_fails_loudly() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("loud");
    scaffold_project(&root, &["api"]);
    let stub = stub_docker(&root);

    let out = run_in(&root, Some(&stub), r#"make dev s="api nope""#);
    let text = combined(&out);

    assert!(
        !out.status.success(),
        "an unknown service in the list must fail the command:\n{text}"
    );
    assert!(
        text.contains("nope"),
        "the failure must name the service it could not find:\n{text}"
    );
    assert!(
        compose_calls(&root).iter().all(|c| !c.contains("up -d")),
        "nothing may be started when part of the request is unresolvable:\n{text}"
    );
}

#[test]
fn list_capable_targets_pass_every_name_to_compose() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("verbs");
    scaffold_project(&root, &["api", "site"]);
    let stub = stub_docker(&root);

    run_ok(&root, Some(&stub), r#"make dev s="api site""#);

    // (target, the compose verb its script runs)
    for (target, verb) in [("stop", "stop"), ("logs", "logs")] {
        std::fs::remove_file(root.join("calls.log")).ok();
        run_ok(
            &root,
            Some(&stub),
            &format!(r#"make {target} s="api site""#),
        );

        let calls = compose_calls(&root);
        let hit = calls
            .iter()
            .find(|c| c.contains(&format!(" {verb} ")) || c.contains(&format!(" {verb} -")))
            .unwrap_or_else(|| {
                panic!(
                    "make {target} recorded no `{verb}` call; calls:\n{}",
                    calls.join("\n")
                )
            });
        assert!(
            hit.contains("api") && hit.contains("site"),
            "make {target} dropped a name from the list: {hit}"
        );
        assert!(
            hit.contains("-p verbs "),
            "make {target} must pin the project name: {hit}"
        );
    }
}

#[test]
fn single_service_only_targets_refuse_a_list_loudly() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("refuse");
    scaffold_project(&root, &["api", "site"]);
    let stub = stub_docker(&root);
    std::fs::create_dir_all(root.join("docker/dockerfiles/api")).unwrap();
    std::fs::write(root.join("docker/dockerfiles/api/app"), "FROM alpine:3\n").unwrap();

    for invocation in [
        r#"make build s="api site""#,
        r#"make sh s="api site""#,
        r#"make exec s="api site" c=ls"#,
        r#"make run s="api site" c=ls"#,
    ] {
        let out = run_in(&root, Some(&stub), invocation);
        let text = combined(&out);
        assert!(
            !out.status.success(),
            "`{invocation}` must fail rather than silently act on the first name:\n{text}"
        );
        assert!(
            text.contains("single service"),
            "`{invocation}` must say it takes a single service only:\n{text}"
        );
        assert!(
            text.contains("api site"),
            "`{invocation}` must echo back what it was given:\n{text}"
        );
    }
}
