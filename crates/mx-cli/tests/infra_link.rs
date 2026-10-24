//! `mx infra link` / `unlink` / `inspect` (bd:mech-crate-066).
//!
//! One link mechanism: `MX_INFRA_USE_GLOBAL=true` inside the project's
//! `infra/<provider>/.env.<provider>`. A linked project's own file is not read
//! for credentials, so they resolve from the environment, then the global
//! `~/.mech-crate/config/infra/<provider>.env`. The Rust resolver
//! (`mx infra inspect`), `make/cloudflare.mk` (`make cf-vars`) and
//! `scripts/cf-init-app.sh` all read that one flag, and the agreement tests
//! below run all three against the same scratch project.
//!
//! The losing mechanism, the `.env.linked` marker file, is no longer honored by
//! anything: `inspect` warns about a stray one and `link` removes it.
//!
//! Every test redirects `HOME` at a tempdir and scrubs every credential name
//! from the environment, so the developer's own config never leaks in.

use std::path::{Path, PathBuf};
use std::process::Output;

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

use mx_lib::test_support::scaffold_project;

const ENV_SCRUB: &[&str] = &[
    "MX_HOME",
    "MECH_CRATE_ROOT",
    "MAKEFLAGS",
    "MX_INFRA_USE_GLOBAL",
    "CLOUDFLARE_ACCOUNT_ID",
    "CLOUDFLARE_API_TOKEN",
    "CF_ACCOUNT_ID",
    "CF_API_TOKEN",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/mx-cli should sit two levels below the repo root")
        .to_path_buf()
}

/// A project carrying the real shipped Makefile, make modules and
/// `cf-init-app.sh`, so the deploy toolchain can be asked what it resolves.
fn template_project(root: &Path) {
    scaffold_project(root);
    let templates = repo_root().join("templates");
    std::fs::copy(templates.join("Makefile.template"), root.join("Makefile"))
        .expect("setup: copy Makefile.template");
    for entry in std::fs::read_dir(templates.join("make")).expect("setup: read templates/make") {
        let src = entry.expect("setup: dir entry").path();
        if src.extension().and_then(|e| e.to_str()) == Some("mk") {
            std::fs::copy(&src, root.join("make").join(src.file_name().unwrap()))
                .expect("setup: copy make module");
        }
    }
    std::fs::copy(
        templates.join("scripts/cf-init-app.sh"),
        root.join("scripts/cf-init-app.sh"),
    )
    .expect("setup: copy cf-init-app.sh");
}

fn scrubbed(mut cmd: Command, cwd: &Path, home: &Path) -> Command {
    cmd.current_dir(cwd).env("HOME", home);
    for key in ENV_SCRUB {
        cmd.env_remove(key);
    }
    cmd
}

fn mx_in(cwd: &Path, home: &Path) -> Command {
    scrubbed(
        Command::cargo_bin("mx").expect("mx binary built"),
        cwd,
        home,
    )
}

fn run(cmd: &mut Command) -> Output {
    cmd.output().expect("setup: spawn")
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn write_global_cf(home: &Path, body: &str) {
    let dir = home.join(".mech-crate/config/infra");
    std::fs::create_dir_all(&dir).expect("setup: global infra dir");
    std::fs::write(dir.join("cloudflare.env"), body).expect("setup: global cloudflare.env");
}

fn project_cf_file(project: &Path) -> PathBuf {
    project.join("infra/cloudflare/.env.cloudflare")
}

fn write_project_cf(project: &Path, body: &str) {
    let path = project_cf_file(project);
    std::fs::create_dir_all(path.parent().unwrap()).expect("setup: project infra dir");
    std::fs::write(path, body).expect("setup: project .env.cloudflare");
}

/// The `KEY=VALUE` line of `make cf-vars` output.
fn make_var(out: &Output, key: &str) -> String {
    let stdout = stdout_of(out);
    stdout
        .lines()
        .find_map(|l| l.strip_prefix(&format!("{key}=")))
        .unwrap_or_else(|| panic!("cf-vars printed no {key} line; got:\n{stdout}"))
        .trim()
        .to_string()
}

/// The inspect line for `key`: `  KEY = value  [source]`.
fn inspect_line(out: &Output, key: &str) -> String {
    let stdout = stdout_of(out);
    stdout
        .lines()
        .find(|l| l.split_whitespace().next() == Some(key))
        .unwrap_or_else(|| panic!("inspect printed no {key} line; got:\n{stdout}"))
        .to_string()
}

fn make_cf_vars(project: &Path, home: &Path) -> Output {
    let mut cmd = Command::new("make");
    cmd.arg("cf-vars");
    let out = run(&mut scrubbed(cmd, project, home));
    assert!(
        out.status.success(),
        "setup: make cf-vars must exit 0, got: {}{}",
        stdout_of(&out),
        stderr_of(&out)
    );
    out
}

/// Run the shipped `cf-init-app.sh` non-interactively and return the account id
/// it wrote into the generated `wrangler.toml`, plus its stdout.
fn bash_resolved_account(project: &Path, home: &Path, app: &str) -> (String, String) {
    let mut cmd = Command::new("bash");
    cmd.args(["scripts/cf-init-app.sh", app, "--type=worker", "--yes"]);
    let mut cmd = scrubbed(cmd, project, home);
    // `--yes` does not cover the worker option prompts; blank answers take
    // every default (no route, no KV, no vars).
    cmd.write_stdin("\n".repeat(16));
    let out = run(&mut cmd);
    assert!(
        out.status.success(),
        "setup: cf-init-app.sh must exit 0, got: {}{}",
        stdout_of(&out),
        stderr_of(&out)
    );
    let wrangler = std::fs::read_to_string(
        project
            .join("infra/cloudflare/apps")
            .join(app)
            .join("wrangler.toml"),
    )
    .expect("setup: cf-init-app.sh must write wrangler.toml");
    let account = wrangler
        .lines()
        .find_map(|l| l.trim().strip_prefix("account_id = "))
        .expect("setup: wrangler.toml carries account_id")
        .trim_matches('"')
        .to_string();
    (account, stdout_of(&out))
}

// ── honest errors ────────────────────────────────────────────────────────────

#[test]
fn link_outside_a_project_fails_and_writes_nothing() {
    let home = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    write_global_cf(home.path(), "CLOUDFLARE_ACCOUNT_ID=dummy-global\n");

    mx_in(elsewhere.path(), home.path())
        .args(["infra", "link", "cloudflare"])
        .assert()
        .failure()
        .stderr(contains("project"));
    assert!(
        !elsewhere.path().join("infra").exists(),
        "a failed link must not create infra/ outside a project"
    );
}

#[test]
fn unlink_outside_a_project_fails() {
    let home = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();

    mx_in(elsewhere.path(), home.path())
        .args(["infra", "unlink", "cloudflare"])
        .assert()
        .failure()
        .stderr(contains("project"));
}

#[test]
fn link_and_unlink_reject_an_unknown_provider() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    scaffold_project(project.path());

    for verb in ["link", "unlink"] {
        mx_in(project.path(), home.path())
            .args(["infra", verb, "nosuchcloud"])
            .assert()
            .failure()
            .stderr(contains("Unknown provider: nosuchcloud"));
    }
    assert!(!project.path().join("infra").exists());
}

#[test]
fn link_without_a_global_config_fails_naming_setup() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    scaffold_project(project.path());

    mx_in(project.path(), home.path())
        .args(["infra", "link", "cloudflare"])
        .assert()
        .failure()
        .stderr(contains("mx infra setup cloudflare"));
    assert!(!project_cf_file(project.path()).exists());
}

// ── link / unlink round trip ─────────────────────────────────────────────────

#[test]
fn link_unlink_round_trip_is_persistent_idempotent_and_reversible() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    scaffold_project(project.path());
    write_global_cf(home.path(), "CLOUDFLARE_ACCOUNT_ID=dummy-global\n");
    let original = "# my project creds\nCLOUDFLARE_ACCOUNT_ID=dummy-project\n";
    write_project_cf(project.path(), original);

    mx_in(project.path(), home.path())
        .args(["infra", "link", "cloudflare"])
        .assert()
        .success()
        .stdout(contains("Linked cloudflare"));
    let linked = std::fs::read_to_string(project_cf_file(project.path())).unwrap();
    assert!(
        linked
            .lines()
            .any(|l| l.trim() == "MX_INFRA_USE_GLOBAL=true"),
        "link must write the flag into the project file, got:\n{linked}"
    );
    assert!(
        linked.contains("CLOUDFLARE_ACCOUNT_ID=dummy-project"),
        "link must keep the project's own credentials (ignored, not destroyed), got:\n{linked}"
    );

    // Idempotent: a second link changes nothing.
    mx_in(project.path(), home.path())
        .args(["infra", "link", "cloudflare"])
        .assert()
        .success()
        .stdout(contains("already linked"));
    assert_eq!(
        std::fs::read_to_string(project_cf_file(project.path())).unwrap(),
        linked
    );

    mx_in(project.path(), home.path())
        .args(["infra", "unlink", "cloudflare"])
        .assert()
        .success()
        .stdout(contains("Unlinked cloudflare"));
    assert_eq!(
        std::fs::read_to_string(project_cf_file(project.path())).unwrap(),
        original,
        "unlink must restore the project file exactly"
    );

    // Idempotent: unlinking an unlinked project is a no-op, not an error.
    mx_in(project.path(), home.path())
        .args(["infra", "unlink", "cloudflare"])
        .assert()
        .success()
        .stdout(contains("not linked"));
    assert_eq!(
        std::fs::read_to_string(project_cf_file(project.path())).unwrap(),
        original
    );
}

#[test]
fn unlink_removes_a_project_file_that_link_created() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    scaffold_project(project.path());
    write_global_cf(home.path(), "CLOUDFLARE_ACCOUNT_ID=dummy-global\n");

    mx_in(project.path(), home.path())
        .args(["infra", "link", "cloudflare"])
        .assert()
        .success();
    assert!(project_cf_file(project.path()).exists());

    mx_in(project.path(), home.path())
        .args(["infra", "unlink", "cloudflare"])
        .assert()
        .success();
    assert!(
        !project_cf_file(project.path()).exists(),
        "a project file holding nothing but the link must go away on unlink"
    );
}

#[test]
fn link_works_from_a_subdirectory_of_the_project() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    scaffold_project(project.path());
    write_global_cf(home.path(), "CLOUDFLARE_ACCOUNT_ID=dummy-global\n");

    mx_in(&project.path().join("docker/compose"), home.path())
        .args(["infra", "link", "cf"])
        .assert()
        .success();
    assert!(
        project_cf_file(project.path()).exists(),
        "link must write at the project root, not the cwd"
    );
}

// ── inspect: resolved set with per-key source ────────────────────────────────

#[test]
fn inspect_merges_scopes_and_names_each_keys_source() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    scaffold_project(project.path());
    write_global_cf(
        home.path(),
        "CLOUDFLARE_ACCOUNT_ID=dummy-global\nCLOUDFLARE_API_TOKEN=dummy-global-token\n",
    );
    // The project carries only the deprecated alias for the account id.
    write_project_cf(project.path(), "CF_ACCOUNT_ID=dummy-project-alias\n");

    let out = run(mx_in(project.path(), home.path()).args(["infra", "inspect", "cloudflare"]));
    assert!(out.status.success(), "inspect: {}", stderr_of(&out));

    let id = inspect_line(&out, "CLOUDFLARE_ACCOUNT_ID");
    assert!(id.contains("dummy-project-alias"), "{id}");
    assert!(
        id.contains("[project, deprecated alias CF_ACCOUNT_ID]"),
        "{id}"
    );

    let token = inspect_line(&out, "CLOUDFLARE_API_TOKEN");
    assert!(token.contains("[global]"), "{token}");
    assert!(token.contains("********"), "{token}");
    assert!(
        !stdout_of(&out).contains("dummy-global-token"),
        "inspect must never print a secret"
    );
    assert!(stdout_of(&out).contains("not linked"));
}

#[test]
fn inspect_reports_the_environment_scope_first() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    scaffold_project(project.path());
    write_global_cf(home.path(), "CLOUDFLARE_ACCOUNT_ID=dummy-global\n");
    write_project_cf(project.path(), "CLOUDFLARE_ACCOUNT_ID=dummy-project\n");

    let out = run(mx_in(project.path(), home.path())
        .env("CLOUDFLARE_ACCOUNT_ID", "dummy-env")
        .args(["infra", "inspect", "cloudflare"]));
    assert!(out.status.success(), "inspect: {}", stderr_of(&out));
    let id = inspect_line(&out, "CLOUDFLARE_ACCOUNT_ID");
    assert!(
        id.contains("dummy-env") && id.contains("[environment]"),
        "{id}"
    );
}

#[test]
fn inspect_of_a_linked_project_ignores_the_project_credentials() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    scaffold_project(project.path());
    write_global_cf(home.path(), "CLOUDFLARE_ACCOUNT_ID=dummy-global\n");
    write_project_cf(project.path(), "CLOUDFLARE_ACCOUNT_ID=dummy-project\n");

    mx_in(project.path(), home.path())
        .args(["infra", "link", "cloudflare"])
        .assert()
        .success();

    let out = run(mx_in(project.path(), home.path()).args(["infra", "inspect", "cloudflare"]));
    assert!(out.status.success(), "inspect: {}", stderr_of(&out));
    let id = inspect_line(&out, "CLOUDFLARE_ACCOUNT_ID");
    assert!(
        id.contains("dummy-global") && id.contains("[global]"),
        "{id}"
    );
    assert!(
        stdout_of(&out).contains("linked to global"),
        "inspect must show the link state, got:\n{}",
        stdout_of(&out)
    );
    assert!(
        !stdout_of(&out).contains("MX_INFRA_USE_GLOBAL ="),
        "the link flag is state, not a credential row"
    );
}

#[test]
fn inspect_with_nothing_configured_fails_naming_setup() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    scaffold_project(project.path());

    mx_in(project.path(), home.path())
        .args(["infra", "inspect", "cloudflare"])
        .assert()
        .failure()
        .stderr(contains("mx infra setup cloudflare"));
}

// ── the losing mechanism: the `.env.linked` marker ───────────────────────────

#[test]
fn a_stray_env_linked_marker_is_ignored_with_a_deprecation_warning() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    scaffold_project(project.path());
    write_global_cf(home.path(), "CLOUDFLARE_ACCOUNT_ID=dummy-global\n");
    write_project_cf(project.path(), "CLOUDFLARE_ACCOUNT_ID=dummy-project\n");
    let marker = project.path().join("infra/cloudflare/.env.linked");
    std::fs::write(&marker, "").unwrap();

    let out = run(mx_in(project.path(), home.path()).args(["infra", "inspect", "cloudflare"]));
    assert!(out.status.success(), "inspect: {}", stderr_of(&out));
    assert!(
        stderr_of(&out).contains(".env.linked") && stderr_of(&out).contains("deprecated"),
        "inspect must warn about the deprecated marker, got stderr:\n{}",
        stderr_of(&out)
    );
    let id = inspect_line(&out, "CLOUDFLARE_ACCOUNT_ID");
    assert!(
        id.contains("dummy-project") && id.contains("[project]"),
        "the marker no longer links anything: {id}"
    );

    // `link` migrates it: the flag is written and the marker removed.
    mx_in(project.path(), home.path())
        .args(["infra", "link", "cloudflare"])
        .assert()
        .success()
        .stdout(contains(".env.linked"));
    assert!(!marker.exists(), "link must remove the deprecated marker");
}

// ── resolver agreement: mx, make and bash read the same state ────────────────

#[test]
fn rust_make_and_bash_agree_on_link_state() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    template_project(project.path());
    write_global_cf(
        home.path(),
        "CLOUDFLARE_ACCOUNT_ID=dummy-global\nCLOUDFLARE_API_TOKEN=dummy-global-token\n",
    );
    write_project_cf(project.path(), "CF_ACCOUNT_ID=dummy-project\n");

    // Unlinked: all three say project.
    let inspect = run(mx_in(project.path(), home.path()).args(["infra", "inspect", "cloudflare"]));
    assert!(inspect_line(&inspect, "CLOUDFLARE_ACCOUNT_ID").contains("dummy-project"));
    let vars = make_cf_vars(project.path(), home.path());
    assert_eq!(make_var(&vars, "CLOUDFLARE_ACCOUNT_ID"), "dummy-project");
    assert_eq!(make_var(&vars, "CLOUDFLARE_ACCOUNT_ID_SOURCE"), "project");
    assert_eq!(make_var(&vars, "CF_PROJECT_LINKED"), "false");
    let (bash_id, _) = bash_resolved_account(project.path(), home.path(), "before");
    assert_eq!(bash_id, "dummy-project");

    // Linked: all three say global, and the deprecated-alias warning on the
    // (now ignored) project file goes away.
    mx_in(project.path(), home.path())
        .args(["infra", "link", "cloudflare"])
        .assert()
        .success();
    let inspect = run(mx_in(project.path(), home.path()).args(["infra", "inspect", "cloudflare"]));
    let line = inspect_line(&inspect, "CLOUDFLARE_ACCOUNT_ID");
    assert!(
        line.contains("dummy-global") && line.contains("[global]"),
        "{line}"
    );
    let vars = make_cf_vars(project.path(), home.path());
    assert_eq!(make_var(&vars, "CLOUDFLARE_ACCOUNT_ID"), "dummy-global");
    assert_eq!(make_var(&vars, "CLOUDFLARE_ACCOUNT_ID_SOURCE"), "global");
    assert_eq!(make_var(&vars, "CF_PROJECT_LINKED"), "true");
    assert!(!stdout_of(&vars).contains("DEPRECATED_CF_ACCOUNT_ID_IN"));
    let (bash_id, bash_out) = bash_resolved_account(project.path(), home.path(), "linked");
    assert_eq!(bash_id, "dummy-global");
    assert!(
        bash_out.contains("resolved from the global config"),
        "{bash_out}"
    );

    // `mx cf vars` (the wrapper) sees the same thing.
    mx_in(project.path(), home.path())
        .args(["cf", "vars"])
        .assert()
        .success()
        .stdout(contains("CLOUDFLARE_ACCOUNT_ID_SOURCE=global"))
        .stdout(contains("CF_PROJECT_LINKED=true"));

    // Unlinked again: back to project everywhere.
    mx_in(project.path(), home.path())
        .args(["infra", "unlink", "cloudflare"])
        .assert()
        .success();
    let vars = make_cf_vars(project.path(), home.path());
    assert_eq!(make_var(&vars, "CLOUDFLARE_ACCOUNT_ID_SOURCE"), "project");
    assert_eq!(make_var(&vars, "CF_PROJECT_LINKED"), "false");
    let (bash_id, _) = bash_resolved_account(project.path(), home.path(), "after");
    assert_eq!(bash_id, "dummy-project");
}

#[test]
fn a_linked_project_with_no_global_config_fails_the_credential_check() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    template_project(project.path());
    write_project_cf(
        project.path(),
        "CLOUDFLARE_ACCOUNT_ID=dummy-project\nMX_INFRA_USE_GLOBAL=true\n",
    );

    let mut cmd = Command::new("make");
    cmd.arg("cf-check-credentials");
    scrubbed(cmd, project.path(), home.path())
        .assert()
        .failure()
        .stderr(contains("MX_INFRA_USE_GLOBAL=true"));

    mx_in(project.path(), home.path())
        .args(["infra", "inspect", "cloudflare"])
        .assert()
        .failure()
        .stderr(contains("mx infra setup cloudflare"));
}

#[test]
fn infra_list_shows_a_linked_project() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    scaffold_project(project.path());
    write_global_cf(home.path(), "CLOUDFLARE_ACCOUNT_ID=dummy-global\n");
    mx_in(project.path(), home.path())
        .args(["infra", "link", "cloudflare"])
        .assert()
        .success();

    mx_in(project.path(), home.path())
        .args(["infra", "list"])
        .assert()
        .success()
        .stdout(contains("cloudflare").and(contains("linked")));
}
