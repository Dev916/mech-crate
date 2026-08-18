//! CLI surface tests: the whole `mx` subcommand tree answers `--help`, and the
//! commands that can run without a daemon, a database or a network actually run.
//!
//! House style is `assert_cmd`'s `Command::cargo_bin` + `predicates` (see
//! `docs/development/MX_RUST_CLI_AND_MCP_SERVER.md`, Testing). Every test here
//! is hermetic: no Docker daemon (stub-bin PATH), no `~/.mech-crate` of the
//! developer running it (`HOME` is redirected at a tempdir), no network
//! (`UnyformClient` sees no credentials under the fake home, so it never
//! reaches out).

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::{contains, is_empty};

use mx_lib::test_support::{scaffold_project, StubBin};

/// Every top-level subcommand of `mx`, kebab-cased exactly as clap exposes it.
///
/// `subcommand_list_matches_clap_surface` fails if this drifts from the real
/// `Commands` enum, so a newly-added command cannot silently skip the sweep.
const SUBCOMMANDS: &[&str] = &[
    "init",
    "new",
    "add",
    "recipes",
    "dev",
    "up",
    "down",
    "logs",
    "restart",
    "sh",
    "ps",
    "build",
    "docs",
    "router",
    "infra",
    "mcp",
    "rag",
    "doctor",
    "unyform",
    "cc-plugin",
    "login",
    "logout",
    "whoami",
    "upgrade",
    "self-update",
];

/// Nested subcommands worth pinning: the ones with their own argument surface
/// that tests and docs reference by name.
const NESTED_SUBCOMMANDS: &[&[&str]] = &[
    &["rag", "ingest"],
    &["rag", "status"],
    &["rag", "gaps"],
    &["recipes", "list"],
    &["recipes", "info"],
];

fn mx() -> Command {
    Command::cargo_bin("mx").expect("mx binary built")
}

/// Path to the checked-in 2-doc corpus fixture.
fn ragdocs_fixture() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ragdocs")
}

#[test]
fn every_subcommand_has_help() {
    for sub in SUBCOMMANDS {
        mx().args([sub, "--help"])
            .assert()
            .success()
            .stdout(is_empty().not());
    }
}

#[test]
fn nested_subcommands_have_help() {
    for path in NESTED_SUBCOMMANDS {
        let mut cmd = mx();
        cmd.args(path.iter());
        cmd.arg("--help")
            .assert()
            .success()
            .stdout(is_empty().not());
    }
}

/// The sweep is only as good as its list — parse `mx --help` and demand the two
/// agree in both directions.
#[test]
fn subcommand_list_matches_clap_surface() {
    let out = mx().arg("--help").output().expect("run mx --help");
    assert!(out.status.success(), "mx --help must exit 0");
    let stdout = String::from_utf8(out.stdout).expect("utf-8 help output");

    let mut declared: Vec<String> = Vec::new();
    let mut in_commands = false;
    for line in stdout.lines() {
        if line.starts_with("Commands:") {
            in_commands = true;
            continue;
        }
        if !in_commands {
            continue;
        }
        // Entries sit at exactly two spaces of indent; wrapped description
        // lines are indented deeper, and the next section header is flush left.
        let Some(rest) = line.strip_prefix("  ") else {
            break;
        };
        if rest.starts_with(' ') {
            continue;
        }
        if let Some(name) = rest.split_whitespace().next() {
            declared.push(name.to_string());
        }
    }
    declared.retain(|n| n != "help");

    assert!(
        !declared.is_empty(),
        "could not parse any subcommands out of `mx --help`:\n{}",
        stdout
    );

    let mut missing: Vec<&String> = declared
        .iter()
        .filter(|d| !SUBCOMMANDS.contains(&d.as_str()))
        .collect();
    missing.sort();
    assert!(
        missing.is_empty(),
        "SUBCOMMANDS is missing {:?} — add them so the help sweep covers them",
        missing
    );

    let mut stale: Vec<&&str> = SUBCOMMANDS
        .iter()
        .filter(|s| !declared.iter().any(|d| d == *s))
        .collect();
    stale.sort();
    assert!(
        stale.is_empty(),
        "SUBCOMMANDS lists {:?}, which `mx --help` no longer exposes",
        stale
    );
}

#[test]
fn rag_dry_run_reports_two_docs_zero_warnings() {
    mx().args(["rag", "ingest", "--dry-run", "--path", ragdocs_fixture()])
        .assert()
        .success()
        .stdout(contains("2 docs").and(contains("0 warnings")));
}

/// Guards the assertion above from going vacuous: a doc without frontmatter
/// must be counted *and* warned about, so `0 warnings` is a real signal.
#[test]
fn rag_dry_run_warns_on_frontmatter_less_doc() {
    let dir = tempfile::tempdir().expect("tempdir");
    for name in ["a.md", "b.md"] {
        std::fs::copy(
            std::path::Path::new(ragdocs_fixture()).join(name),
            dir.path().join(name),
        )
        .expect("copy fixture doc");
    }
    std::fs::write(dir.path().join("c.md"), "# No Frontmatter\n\nbody\n").expect("write bare doc");

    mx().args(["rag", "ingest", "--dry-run", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(contains("3 docs").and(contains("1 warnings")));
}

#[test]
fn doctor_on_scaffolded_project_succeeds() {
    let dir = tempfile::tempdir().expect("tempdir");
    scaffold_project(dir.path());

    // doctor probes `docker --version` / `docker info` / `make`; the stubs keep
    // it off the real daemon so the test is fast and machine-independent.
    let sb = StubBin::new();
    sb.stub("docker", 0, "Docker version 99.0.0, build stub");
    sb.stub("make", 0, "");

    mx().current_dir(dir.path())
        .env("PATH", sb.path_env())
        .arg("doctor")
        .assert()
        .success()
        .stdout(contains("Project Structure").and(contains("docker/compose/")));
}

/// Outside a project `doctor` still reports rather than fails — pin that, since
/// a non-zero exit here would break `mx doctor` as a preflight check.
#[test]
fn doctor_outside_a_project_succeeds_with_a_notice() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sb = StubBin::new();
    sb.stub("docker", 0, "Docker version 99.0.0, build stub");
    sb.stub("make", 0, "");

    mx().current_dir(dir.path())
        .env("PATH", sb.path_env())
        .arg("doctor")
        .assert()
        .success()
        .stdout(contains("Not in a MechCrate project"));
}

#[cfg(unix)]
#[test]
fn recipes_list_lists_the_bundled_recipes() {
    let home = tempfile::tempdir().expect("tempdir");
    let mech = home.path().join(".mech-crate");
    std::fs::create_dir_all(&mech).expect("create fake mx home");
    // Symlink rather than copy: `is_initialized()` and `templates_dir()` both
    // resolve through it, and the real recipe tree is what we want listed.
    std::os::unix::fs::symlink(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../templates"),
        mech.join("templates"),
    )
    .expect("link templates into fake home");

    let out = mx()
        .env("HOME", home.path())
        .env_remove("MECH_CRATE_ROOT")
        .args(["recipes", "list"])
        .output()
        .expect("run mx recipes list");

    assert!(
        out.status.success(),
        "mx recipes list failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).expect("utf-8 output");
    let listed = stdout
        .lines()
        .filter(|l| l.trim_start().starts_with('•'))
        .count();
    assert!(
        listed >= 7,
        "expected at least 7 local recipes, saw {}:\n{}",
        listed,
        stdout
    );
}
