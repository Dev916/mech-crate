//! Compose project isolation (bd:mech-crate-71u).
//!
//! Every mx project keeps its compose files in `docker/compose/`, so
//! `docker compose` derives the SAME default project name ("compose", the
//! parent directory) for every mx project on the machine — one stack then
//! adopts/recreates another's containers and volumes. `templates/scripts/.bashrc`
//! is the authoritative source of a per-project `COMPOSE_PROJECT_NAME`, and
//! every script that shells out to compose passes `-p "$COMPOSE_PROJECT_NAME"`.
//!
//! These tests hold both halves: a static sweep of the shipped scripts, and a
//! behavioral run of two scratch projects against a `docker` stub on `PATH`
//! that records its argv.

use std::path::{Path, PathBuf};

fn templates_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates")
}

fn scripts_root() -> PathBuf {
    templates_root().join("scripts")
}

// ── Static sweep over the shipped scripts ────────────────────────────────────

/// Top-level shipped scripts, sorted: `(file name, contents)`.
fn shipped_scripts() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = std::fs::read_dir(scripts_root())
        .expect("setup: templates/scripts must exist")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
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

/// Lines that actually invoke compose — `echo "docker compose …"` lines are
/// documentation of the command, not an invocation, so they are excluded.
fn compose_invocations(body: &str) -> Vec<&str> {
    body.lines()
        .map(str::trim)
        .filter(|l| l.contains("docker compose"))
        .filter(|l| !l.starts_with('#'))
        .filter(|l| !l.starts_with("echo "))
        // `docker compose version` is a capability probe, not a stack operation.
        .filter(|l| !l.contains("docker compose version"))
        .collect()
}

/// Every compose invocation in the shipped scripts must carry this project's own
/// compose project name. Without `-p`, compose falls back to the compose file's
/// parent directory (`docker/compose/` → "compose") which every mx project
/// shares.
#[test]
fn every_compose_invocation_pins_the_project_name() {
    let mut offenders: Vec<String> = Vec::new();
    let mut checked = 0;

    for (name, body) in shipped_scripts() {
        for line in compose_invocations(&body) {
            checked += 1;
            if !line.contains(r#"-p "$COMPOSE_PROJECT_NAME""#) {
                offenders.push(format!("scripts/{name}: {line}"));
            }
        }
    }

    assert!(
        checked >= 8,
        "setup: expected the shipped scripts to invoke compose at least 8 times, saw {checked}"
    );
    assert!(
        offenders.is_empty(),
        "compose invocations missing `-p \"$COMPOSE_PROJECT_NAME\"` \
         (they would run under the shared default project name):\n{}",
        offenders.join("\n")
    );
}

/// `$COMPOSE_PROJECT_NAME` is only set by `scripts/.bashrc`, so any script that
/// references it must source the helper library first — otherwise the flag
/// expands to an empty string and compose silently falls back to the default.
#[test]
fn scripts_using_the_project_name_source_the_helper_library() {
    let mut offenders: Vec<String> = Vec::new();

    for (name, body) in shipped_scripts() {
        if name == ".bashrc" || !body.contains("COMPOSE_PROJECT_NAME") {
            continue;
        }
        if !body.contains("source ./scripts/.bashrc") {
            offenders.push(format!("scripts/{name}"));
        }
    }

    assert!(
        offenders.is_empty(),
        "these scripts use $COMPOSE_PROJECT_NAME without sourcing ./scripts/.bashrc, \
         so it would expand to empty:\n{}",
        offenders.join("\n")
    );
}

/// The single authoritative point: `.bashrc` derives the name and exports it,
/// while leaving an explicit environment value (e.g. an e2e harness pinning its
/// own namespace) in charge.
#[test]
fn the_helper_library_is_the_single_authoritative_source() {
    let bashrc = std::fs::read_to_string(scripts_root().join(".bashrc"))
        .expect("setup: templates/scripts/.bashrc must exist");

    assert!(
        bashrc.contains("mech_compose_project_name()"),
        ".bashrc must define the derivation function"
    );
    assert!(
        bashrc.contains(r#": "${COMPOSE_PROJECT_NAME:=$(mech_compose_project_name)}""#),
        ".bashrc must default COMPOSE_PROJECT_NAME (`:=` so an explicit value wins)"
    );
    assert!(
        bashrc.contains("export COMPOSE_PROJECT_NAME"),
        ".bashrc must export COMPOSE_PROJECT_NAME for child processes"
    );

    // No other shipped script may define its own derivation or assign the
    // variable itself — one source only. (Calling `mech_compose_project_name`
    // is fine and encouraged: that is reuse of the single sanitizer, which is
    // how `doctor.sh` works out the legacy default name.)
    for (name, body) in shipped_scripts() {
        if name == ".bashrc" {
            continue;
        }
        assert!(
            !body.contains("mech_compose_project_name()"),
            "scripts/{name} defines its own derivation; .bashrc is the single source"
        );
        assert!(
            !body.contains("COMPOSE_PROJECT_NAME="),
            "scripts/{name} assigns COMPOSE_PROJECT_NAME itself; .bashrc owns that"
        );
    }
}

// ── Behavioral: derivation + sanitization ────────────────────────────────────

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

/// Lay down a runnable mx project skeleton at `root`: the shipped top-level
/// scripts plus one compose file, so `./scripts/dev.sh api` has a real context.
fn scaffold_from_templates(root: &Path, service: &str) {
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
    std::fs::write(
        root.join(format!("docker/compose/{service}.yml")),
        format!("services:\n  {service}:\n    image: alpine:3\n"),
    )
    .unwrap();
}

/// Run `bash -c` inside `cwd` and return trimmed stdout, panicking on failure.
fn bash_in(cwd: &Path, path_prefix: Option<&Path>, script: &str) -> String {
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
    // A COMPOSE_PROJECT_NAME inherited from the developer's shell would mask the
    // derivation under test.
    cmd.env_remove("COMPOSE_PROJECT_NAME");
    let out = cmd.output().expect("run bash");
    assert!(
        out.status.success(),
        "bash failed ({:?}) in {}:\nstdout: {}\nstderr: {}",
        out.status.code(),
        cwd.display(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Compose project names must match `[a-z0-9][a-z0-9_-]*`.
fn is_valid_compose_project_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

#[test]
fn project_name_is_derived_from_the_directory_and_sanitized() {
    let tmp = tempfile::tempdir().unwrap();

    // (directory name, expected compose project name)
    let cases: &[(&str, &str)] = &[
        ("alpha", "alpha"),
        ("Beta-Project", "beta-project"),
        ("My Shop 2", "my-shop-2"),
        ("under_scores", "under_scores"),
        ("9lives", "9lives"),
        ("nexus.city", "nexus-city"),
        // Leading junk is stripped so the name stays compose-legal.
        ("--weird--", "weird--"),
        // Nothing legal survives → a stable fallback, never an empty name.
        ("@@@", "mx-project"),
    ];

    for (dir_name, expected) in cases {
        let root = tmp.path().join(dir_name);
        std::fs::create_dir_all(root.join("scripts")).unwrap();
        std::fs::copy(scripts_root().join(".bashrc"), root.join("scripts/.bashrc")).unwrap();

        let got = bash_in(
            &root,
            None,
            r#"source ./scripts/.bashrc && printf '%s' "$COMPOSE_PROJECT_NAME""#,
        );
        assert_eq!(
            got, *expected,
            "directory {dir_name:?} should derive compose project {expected:?}"
        );
        assert!(
            is_valid_compose_project_name(&got),
            "{got:?} is not a legal compose project name"
        );
    }
}

/// The derivation must not depend on the caller's working directory — it is
/// anchored to the location of `scripts/.bashrc`.
#[test]
fn project_name_is_independent_of_the_callers_cwd() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("anchored");
    std::fs::create_dir_all(root.join("scripts")).unwrap();
    std::fs::create_dir_all(root.join("apps/deep/nested")).unwrap();
    std::fs::copy(scripts_root().join(".bashrc"), root.join("scripts/.bashrc")).unwrap();

    let from_subdir = bash_in(
        &root.join("apps/deep/nested"),
        None,
        r#"source ../../../scripts/.bashrc && printf '%s' "$COMPOSE_PROJECT_NAME""#,
    );
    assert_eq!(from_subdir, "anchored");
}

/// An explicit `COMPOSE_PROJECT_NAME` always wins, so harnesses (e.g.
/// `scripts/test-e2e.sh`) can namespace their own runs.
#[test]
fn explicit_project_name_in_the_environment_wins() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("alpha");
    std::fs::create_dir_all(root.join("scripts")).unwrap();
    std::fs::copy(scripts_root().join(".bashrc"), root.join("scripts/.bashrc")).unwrap();

    let got = bash_in(
        &root,
        None,
        r#"export COMPOSE_PROJECT_NAME=mxe2e-pinned
           source ./scripts/.bashrc
           printf '%s' "$COMPOSE_PROJECT_NAME""#,
    );
    assert_eq!(got, "mxe2e-pinned");
}

// ── Behavioral: two scratch projects stay isolated ───────────────────────────

/// The regression net for the incident itself: two mx projects on one machine
/// must put DISTINCT project names on their compose invocations. `docker` is
/// stubbed on `PATH` and records argv, so this asserts the real command line
/// `make dev` would run.
#[test]
fn two_projects_carry_distinct_project_names_into_compose() {
    let tmp = tempfile::tempdir().unwrap();

    // Same recipe-shaped service in both; only the directory name differs — and
    // the second name exercises sanitization on the way through.
    let cases: &[(&str, &str)] = &[("alpha", "alpha"), ("Beta Svc", "beta-svc")];
    let mut seen: Vec<String> = Vec::new();

    for (dir_name, expected_project) in cases {
        let root = tmp.path().join(dir_name);
        scaffold_from_templates(&root, "api");
        let stub = stub_docker(&root);

        bash_in(&root, Some(&stub), "./scripts/dev.sh api");

        let calls = std::fs::read_to_string(root.join("calls.log"))
            .expect("the docker stub must have been invoked");
        let compose_calls: Vec<&str> = calls
            .lines()
            .filter(|l| l.starts_with("compose "))
            .collect();
        assert!(
            !compose_calls.is_empty(),
            "no `docker compose` call recorded for {dir_name}; log:\n{calls}"
        );

        for call in &compose_calls {
            assert!(
                call.contains(&format!("-p {expected_project} ")),
                "{dir_name}: compose call is missing `-p {expected_project}`: {call}"
            );
        }
        assert!(
            compose_calls.iter().any(|c| c.contains("up -d")),
            "{dir_name}: dev.sh never reached `up -d`; calls:\n{calls}"
        );

        seen.push((*expected_project).to_string());
    }

    assert_eq!(
        seen,
        vec!["alpha".to_string(), "beta-svc".to_string()],
        "the two projects must derive different compose project names"
    );
}

// ── Behavioral: the `make doctor` migration check ────────────────────────────

/// A `docker` stub that answers doctor's probes and replays `ps_lines` for the
/// one `docker ps` call that asks for compose labels.
fn stub_docker_for_doctor(dir: &Path, ps_lines: &str) -> PathBuf {
    let bin = dir.join("stub-bin");
    std::fs::create_dir_all(&bin).unwrap();
    let lines = dir.join("ps-lines.txt");
    std::fs::write(&lines, ps_lines).unwrap();
    std::fs::write(
        bin.join("docker"),
        format!(
            r#"#!/bin/sh
case "$1" in
  ps)
    case "$*" in
      *working_dir*) cat '{lines}' ;;
      *) echo deadbeefcafe ;;
    esac ;;
  --version) echo "Docker version 99.9.9, build stub" ;;
  *) echo "2.99.0" ;;
esac
exit 0
"#,
            lines = lines.display()
        ),
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

/// Pinning the project name orphans containers that were started under the old
/// shared default ("compose", from `docker/compose/`): `make down` can no longer
/// see them. `make doctor` has to say so — that is the migration's only warning
/// to an existing project.
#[test]
fn doctor_surfaces_containers_orphaned_by_the_project_name_change() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("delta");
    scaffold_from_templates(&root, "api");
    // Canonical path: the label compose writes is the real path, and doctor
    // compares it against `$(pwd)`.
    let canonical = bash_in(&root, None, "pwd");
    let stub = stub_docker_for_doctor(
        &root,
        &format!("compose|api|delta-legacy-api|{canonical}/docker/compose\n"),
    );

    let out = bash_in(&root, Some(&stub), "./scripts/doctor.sh");

    assert!(
        out.contains("Compose project name: delta"),
        "doctor must report the pinned name:\n{out}"
    );
    assert!(
        out.contains("delta-legacy-api"),
        "doctor must name the orphaned container:\n{out}"
    );
    assert!(
        out.contains("docker rm -f"),
        "doctor must say how to clear the orphan:\n{out}"
    );
}

/// The same check must NOT cry wolf over another project that happens to be
/// sitting in the shared default namespace with a same-named service — telling a
/// developer to `docker rm -f` someone else's database is worse than silence.
/// Ownership is decided by compose's own `project.working_dir` label.
#[test]
fn doctor_leaves_another_stack_in_the_default_namespace_alone() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("epsilon");
    scaffold_from_templates(&root, "db");
    let stub = stub_docker_for_doctor(
        &root,
        // Same service name ("db"), same default project, DIFFERENT project.
        "compose|db|someone-elses-db|/Users/dev/other-project/docker/compose\n\
         clever|db|clever-db-1|/Users/dev/clever/docker/compose\n",
    );

    let out = bash_in(&root, Some(&stub), "./scripts/doctor.sh");

    assert!(
        out.contains("someone-elses-db") && out.contains("/Users/dev/other-project"),
        "doctor should say which stack owns the default namespace:\n{out}"
    );
    assert!(
        !out.contains("docker rm -f"),
        "doctor must NOT advise removing another project's container:\n{out}"
    );
    assert!(
        !out.contains("clever-db-1"),
        "containers under an unrelated project name are nobody's business here:\n{out}"
    );
    assert!(
        out.contains("No orphans left"),
        "with no orphan of its own, doctor should say so:\n{out}"
    );
}

/// The teardown path matters just as much: `make down` must address the same
/// namespace `make dev` created, or it tears down nothing (or, without `-p`,
/// someone else's stack).
#[test]
fn the_down_path_addresses_the_same_project_name() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("gamma");
    scaffold_from_templates(&root, "api");
    let stub = stub_docker(&root);

    bash_in(&root, Some(&stub), "./scripts/dev.sh api");
    bash_in(&root, Some(&stub), "./scripts/down.sh");

    let calls = std::fs::read_to_string(root.join("calls.log")).unwrap();
    let down_calls: Vec<&str> = calls
        .lines()
        .filter(|l| l.starts_with("compose ") && l.contains("down"))
        .collect();
    assert!(
        !down_calls.is_empty(),
        "down.sh recorded no compose call; log:\n{calls}"
    );
    for call in &down_calls {
        assert!(
            call.contains("-p gamma "),
            "down.sh must target the project's own namespace: {call}"
        );
    }
}
