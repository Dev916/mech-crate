//! `mx cf` - the thin wrapper over a project's `make cf-*` targets.
//!
//! Each test runs the real shipped `Makefile.template` + `make/*.mk` in a
//! tempdir project, with `HOME` redirected at a tempdir so the developer's own
//! `~/.mech-crate/config/infra/cloudflare.env` never leaks in. No test reaches
//! wrangler or the network: only the credential-resolution targets
//! (`cf-vars`, `cf-check-credentials`) run, and they are pure make.

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

use mx_lib::test_support::scaffold_project;

fn mx() -> Command {
    Command::cargo_bin("mx").expect("mx binary built")
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/mx-cli should sit two levels below the repo root")
        .to_path_buf()
}

/// A project carrying the real shipped Makefile and make modules.
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
}

/// `mx` run inside `project`, with a hermetic HOME and no ambient CF env.
fn mx_in(project: &Path, home: &Path) -> Command {
    let mut cmd = mx();
    cmd.current_dir(project)
        .env("HOME", home)
        .env_remove("MX_HOME")
        .env_remove("CLOUDFLARE_ACCOUNT_ID")
        .env_remove("CLOUDFLARE_API_TOKEN")
        .env_remove("CF_ACCOUNT_ID")
        .env_remove("CF_API_TOKEN");
    cmd
}

fn write_global_cf(home: &Path, body: &str) {
    let dir = home.join(".mech-crate/config/infra");
    std::fs::create_dir_all(&dir).expect("setup: global infra dir");
    std::fs::write(dir.join("cloudflare.env"), body).expect("setup: global cloudflare.env");
}

#[test]
fn cf_help_lists_the_wrapped_targets() {
    let assert = mx().args(["cf", "--help"]).assert().success();
    let out = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    for sub in [
        "setup",
        "vars",
        "check-credentials",
        "init",
        "status",
        "deploy",
        "deploy-all",
        "logs",
    ] {
        assert!(
            out.lines()
                .any(|l| l.split_whitespace().next() == Some(sub)),
            "`mx cf --help` must list `{sub}`, got:\n{out}"
        );
    }
}

#[test]
fn cf_vars_reports_the_global_scope_through_the_wrapper() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());
    write_global_cf(
        home.path(),
        "CLOUDFLARE_ACCOUNT_ID=dummy-global-account\nCLOUDFLARE_API_TOKEN=dummy-token\n",
    );

    mx_in(project.path(), home.path())
        .args(["cf", "vars"])
        .assert()
        .success()
        .stdout(contains("CLOUDFLARE_ACCOUNT_ID=dummy-global-account"))
        .stdout(contains("CLOUDFLARE_ACCOUNT_ID_SOURCE=global"))
        .stdout(contains("CLOUDFLARE_API_TOKEN=***configured***"))
        .stdout(contains("dummy-token").not());
}

#[test]
fn cf_config_is_an_alias_for_vars() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());

    mx_in(project.path(), home.path())
        .args(["cf", "config"])
        .assert()
        .success()
        .stdout(contains("CLOUDFLARE_ACCOUNT_ID_SOURCE=none"));
}

#[test]
fn cf_check_credentials_passes_with_global_config() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());
    write_global_cf(home.path(), "CLOUDFLARE_ACCOUNT_ID=dummy-global-account\n");

    mx_in(project.path(), home.path())
        .args(["cf", "check-credentials"])
        .assert()
        .success()
        .stdout(contains("resolved from the global config"));
}

#[test]
fn cf_check_credentials_fails_loudly_without_config() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());

    mx_in(project.path(), home.path())
        .args(["cf", "check-credentials"])
        .assert()
        .failure()
        .stderr(contains("No Cloudflare account id found"))
        .stderr(contains("mx infra setup cloudflare"));
}

#[test]
fn cf_deploy_requires_an_app() {
    mx().args(["cf", "deploy"])
        .assert()
        .failure()
        .stderr(contains("<APP>"));
}

#[test]
fn cf_init_rejects_an_unknown_type() {
    mx().args(["cf", "init", "api.example.com", "--type", "lambda"])
        .assert()
        .failure()
        .stderr(contains("worker"));
}

#[test]
fn cf_without_the_cloudflare_module_explains_how_to_add_it() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());
    std::fs::remove_file(project.path().join("make/cloudflare.mk")).unwrap();

    mx_in(project.path(), home.path())
        .args(["cf", "vars"])
        .assert()
        .failure()
        .stderr(contains("make/cloudflare.mk"));
}

#[test]
fn cf_outside_a_project_errors() {
    let dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    mx_in(dir.path(), home.path())
        .args(["cf", "vars"])
        .assert()
        .failure()
        .stderr(contains("Not in a MechCrate project"));
}
