//! The `aws` infra lane (bd:mech-crate-7vf): `mx new --infra aws` scaffolding
//! and the pure-make half of `make/aws.mk`.
//!
//! Two lanes must never mix: the emulator lane (fake credentials, local
//! endpoint) and the sandbox gate (real credentials, real AWS). These tests pin
//! the guards between them. Nothing here reaches Docker, the network or AWS:
//! the `aws` CLI is a stub script on a private `PATH`, and `HOME` is a tempdir
//! so the developer's own `~/.mech-crate/config/infra/aws.env` never leaks in.

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

use mx_lib::test_support::scaffold_project;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/mx-cli should sit two levels below the repo root")
        .to_path_buf()
}

/// A project carrying the real shipped Makefile, make modules and scripts.
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
    std::fs::create_dir_all(root.join("infra/aws")).expect("setup: infra/aws");
}

/// A stub `aws` that answers `sts get-caller-identity` with `account`.
fn stub_aws(dir: &Path, account: &str) -> PathBuf {
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).expect("setup: stub bin dir");
    let path = bin.join("aws");
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\n# records the endpoint the caller saw, then answers as the account\n\
             echo \"${{AWS_ENDPOINT_URL:-unset}}\" > \"{}/endpoint-seen\"\necho {account}\n",
            dir.display()
        ),
    )
    .expect("setup: write aws stub");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("setup: chmod aws stub");
    }
    bin
}

/// `make` inside `project` with a hermetic HOME and no ambient AWS variables.
fn make_in(project: &Path, home: &Path) -> Command {
    let mut cmd = Command::new("make");
    cmd.current_dir(project)
        .env("HOME", home)
        .env_remove("MX_HOME")
        .env_remove("MAKEFLAGS")
        .env_remove("AWS_ENDPOINT_URL")
        .env_remove("AWS_ACCESS_KEY_ID")
        .env_remove("AWS_SECRET_ACCESS_KEY")
        .env_remove("AWS_SESSION_TOKEN")
        .env_remove("AWS_REGION")
        .env_remove("AWS_DEFAULT_REGION")
        .env_remove("AWS_PROFILE")
        .env_remove("AWS_SANDBOX_ACCOUNT_ID")
        .env_remove("AWS_ROUTER_HOST");
    cmd
}

fn write_global_aws(home: &Path, body: &str) {
    let dir = home.join(".mech-crate/config/infra");
    std::fs::create_dir_all(&dir).expect("setup: global infra dir");
    std::fs::write(dir.join("aws.env"), body).expect("setup: write global aws.env");
}

fn with_path(cmd: &mut Command, front: &Path) {
    let path = std::env::var("PATH").unwrap_or_default();
    cmd.env("PATH", format!("{}:{path}", front.display()));
}

#[test]
fn aws_env_prints_the_emulator_lane_and_never_a_real_credential() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());
    write_global_aws(
        home.path(),
        "AWS_ACCESS_KEY_ID=AKIAREALKEY\nAWS_SECRET_ACCESS_KEY=real-secret\nAWS_REGION=eu-west-1\n",
    );

    make_in(project.path(), home.path())
        .args(["aws-env", "PROJECT_NAME=shop"])
        .assert()
        .success()
        .stdout(contains(
            "export AWS_ENDPOINT_URL=http://aws.shop.localhost",
        ))
        .stdout(contains("export AWS_ACCESS_KEY_ID=000000000000"))
        .stdout(contains("export AWS_REGION=us-east-1"))
        .stdout(contains("unset AWS_PROFILE"))
        .stdout(contains("AKIAREALKEY").not())
        .stdout(contains("real-secret").not());
}

#[test]
fn aws_vars_reports_scopes_and_masks_secrets() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());
    write_global_aws(
        home.path(),
        "AWS_ACCESS_KEY_ID=AKIAGLOBAL\nAWS_SECRET_ACCESS_KEY=global-secret\nAWS_REGION=us-west-2\n",
    );
    std::fs::write(
        project.path().join("infra/aws/.env.aws"),
        "AWS_SANDBOX_ACCOUNT_ID=111122223333\n",
    )
    .unwrap();

    make_in(project.path(), home.path())
        .arg("aws-vars")
        .assert()
        .success()
        .stdout(contains("AWS_SANDBOX_ACCOUNT_ID=111122223333"))
        .stdout(contains("AWS_SANDBOX_ACCOUNT_ID_SOURCE=project"))
        .stdout(contains("AWS_ACCESS_KEY_ID=***configured***"))
        .stdout(contains("AWS_ACCESS_KEY_ID_SOURCE=global"))
        .stdout(contains("AWS_REGION=us-west-2"))
        .stdout(contains("AWS_PROJECT_LINKED=false"))
        .stdout(contains("AKIAGLOBAL").not())
        .stdout(contains("global-secret").not());
}

#[test]
fn a_linked_project_ignores_its_own_file() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());
    write_global_aws(home.path(), "AWS_SANDBOX_ACCOUNT_ID=999900001111\n");
    std::fs::write(
        project.path().join("infra/aws/.env.aws"),
        "AWS_SANDBOX_ACCOUNT_ID=111122223333\nMX_INFRA_USE_GLOBAL=true\n",
    )
    .unwrap();

    make_in(project.path(), home.path())
        .arg("aws-vars")
        .assert()
        .success()
        .stdout(contains("AWS_SANDBOX_ACCOUNT_ID=999900001111"))
        .stdout(contains("AWS_SANDBOX_ACCOUNT_ID_SOURCE=global"))
        .stdout(contains("AWS_PROJECT_LINKED=true"));
}

#[test]
fn sandbox_check_refuses_while_an_emulator_endpoint_is_exported() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());
    write_global_aws(home.path(), "AWS_SANDBOX_ACCOUNT_ID=111122223333\n");
    let stub = stub_aws(home.path(), "111122223333");

    let mut cmd = make_in(project.path(), home.path());
    with_path(&mut cmd, &stub);
    cmd.env("AWS_ENDPOINT_URL", "http://aws.shop.localhost")
        .arg("aws-check-sandbox")
        .assert()
        .failure()
        .stderr(contains("AWS_ENDPOINT_URL is set"));
    assert!(
        !home.path().join("endpoint-seen").exists(),
        "the guard must fail before the AWS CLI is ever invoked"
    );
}

#[test]
fn sandbox_check_refuses_without_a_configured_account() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());

    make_in(project.path(), home.path())
        .arg("aws-check-sandbox")
        .assert()
        .failure()
        .stderr(contains("No AWS_SANDBOX_ACCOUNT_ID configured"));
}

#[test]
fn sandbox_check_refuses_a_different_account() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());
    write_global_aws(home.path(), "AWS_SANDBOX_ACCOUNT_ID=111122223333\n");
    let stub = stub_aws(home.path(), "444455556666");

    let mut cmd = make_in(project.path(), home.path());
    with_path(&mut cmd, &stub);
    cmd.arg("aws-check-sandbox")
        .assert()
        .failure()
        .stderr(contains("Caller is account 444455556666"))
        .stderr(contains("sandbox is 111122223333"));
}

#[test]
fn sandbox_command_runs_against_the_matching_account_with_no_endpoint_override() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());
    write_global_aws(
        home.path(),
        "AWS_SANDBOX_ACCOUNT_ID=111122223333\nAWS_REGION=us-west-2\n",
    );
    let stub = stub_aws(home.path(), "111122223333");

    let mut cmd = make_in(project.path(), home.path());
    with_path(&mut cmd, &stub);
    cmd.args(["aws-sandbox", "c=printenv AWS_REGION"])
        .assert()
        .success()
        .stdout(contains("Caller is the sandbox account 111122223333"))
        .stdout(contains("us-west-2"));
    assert_eq!(
        std::fs::read_to_string(home.path().join("endpoint-seen"))
            .unwrap()
            .trim(),
        "unset",
        "the sandbox lane must reach the AWS CLI with no AWS_ENDPOINT_URL"
    );
}

#[test]
fn emulator_cli_forces_fake_credentials_over_real_ones() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());
    let stub = stub_aws(home.path(), "000000000000");

    let mut cmd = make_in(project.path(), home.path());
    with_path(&mut cmd, &stub);
    cmd.env("AWS_ACCESS_KEY_ID", "AKIAREALKEY")
        .env("AWS_PROFILE", "production")
        .args(["aws-cli", "c=s3 ls", "PROJECT_NAME=shop"])
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(home.path().join("endpoint-seen"))
            .unwrap()
            .trim(),
        "http://aws.shop.localhost",
        "aws-cli must point the CLI at the emulator"
    );
}

/// `mx new` with templates taken from this checkout and a hermetic HOME.
fn mx_new(cwd: &Path, home: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    std::fs::create_dir_all(home.join(".mech-crate/templates/recipes")).expect("setup: mx home");
    Command::cargo_bin("mx")
        .expect("mx binary built")
        .current_dir(cwd)
        .env("HOME", home)
        .env_remove("MX_HOME")
        .env("MECH_CRATE_ROOT", repo_root())
        .arg("new")
        .args(args)
        .arg("--no-prompt")
        .assert()
}

#[test]
fn new_with_infra_aws_scaffolds_the_lane() {
    let cwd = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    mx_new(cwd.path(), home.path(), &["Shop", "--infra", "aws"])
        .success()
        .stdout(contains("AWS infrastructure added"));

    let project = cwd.path().join("Shop");
    let compose = std::fs::read_to_string(project.join("infra/aws/compose.yml"))
        .expect("infra/aws/compose.yml scaffolded");
    assert!(
        compose.contains("Host(`${AWS_ROUTER_HOST:-aws.shop.localhost}`)"),
        "the router host default carries the project slug:\n{compose}"
    );
    assert!(!compose.contains("{{PROJECT_NAME}}"));
    assert!(project.join("infra/aws/aws.local.env").exists());
    assert!(project.join("infra/aws/README.md").exists());
    assert!(project.join("make/aws.mk").exists());
    assert!(
        !project.join("make/cloudflare.mk").exists(),
        "cloudflare.mk is removed when Cloudflare was not selected"
    );
    let gitignore = std::fs::read_to_string(project.join(".gitignore")).unwrap();
    assert!(gitignore.contains("infra/aws/.env.aws"));
}

#[test]
fn new_without_aws_ships_no_aws_module() {
    let cwd = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    mx_new(cwd.path(), home.path(), &["plain"]).success();

    let project = cwd.path().join("plain");
    assert!(!project.join("make/aws.mk").exists());
    assert!(!project.join("infra/aws").exists());
}

/// `mx` inside `project` with the same hermetic environment as [`make_in`].
fn mx_in(project: &Path, home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("mx").expect("mx binary built");
    cmd.current_dir(project)
        .env("HOME", home)
        .env_remove("MX_HOME")
        .env_remove("MAKEFLAGS")
        .env_remove("AWS_ENDPOINT_URL")
        .env_remove("AWS_ACCESS_KEY_ID")
        .env_remove("AWS_SECRET_ACCESS_KEY")
        .env_remove("AWS_SESSION_TOKEN")
        .env_remove("AWS_REGION")
        .env_remove("AWS_DEFAULT_REGION")
        .env_remove("AWS_PROFILE")
        .env_remove("AWS_SANDBOX_ACCOUNT_ID")
        .env_remove("AWS_ROUTER_HOST");
    cmd
}

#[test]
fn mx_aws_vars_runs_the_make_target() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());
    write_global_aws(home.path(), "AWS_SANDBOX_ACCOUNT_ID=111122223333\n");

    for alias in ["vars", "config"] {
        mx_in(project.path(), home.path())
            .args(["aws", alias])
            .assert()
            .success()
            .stdout(contains("AWS_SANDBOX_ACCOUNT_ID=111122223333"))
            .stdout(contains("AWS_SANDBOX_ACCOUNT_ID_SOURCE=global"));
    }
}

#[test]
fn mx_aws_cli_passes_flags_through_to_the_emulator_lane() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());
    let bin = home.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    // A stub `aws` that records its argv and the endpoint it was given.
    let stub = bin.join("aws");
    std::fs::write(
        &stub,
        format!(
            "#!/bin/sh\necho \"$AWS_ENDPOINT_URL|$AWS_ACCESS_KEY_ID|$*\" > \"{}/seen\"\n",
            home.path().display()
        ),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let mut cmd = mx_in(project.path(), home.path());
    with_path(&mut cmd, &bin);
    cmd.env("AWS_ACCESS_KEY_ID", "AKIAREALKEY")
        .env("AWS_ROUTER_HOST", "aws.shop.localhost")
        .args(["aws", "cli", "s3", "ls", "--recursive", "s3://uploads"])
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(home.path().join("seen"))
            .unwrap()
            .trim(),
        "http://aws.shop.localhost|000000000000|s3 ls --recursive s3://uploads"
    );
}

#[test]
fn mx_aws_sandbox_is_refused_with_an_emulator_endpoint_exported() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    template_project(project.path());
    write_global_aws(home.path(), "AWS_SANDBOX_ACCOUNT_ID=111122223333\n");
    let stub = stub_aws(home.path(), "111122223333");

    let mut cmd = mx_in(project.path(), home.path());
    with_path(&mut cmd, &stub);
    cmd.env("AWS_ENDPOINT_URL", "http://aws.shop.localhost")
        .args(["aws", "sandbox", "terraform", "apply"])
        .assert()
        .failure()
        .stderr(contains("AWS_ENDPOINT_URL is set"));
}

#[test]
fn mx_aws_explains_a_project_without_the_lane() {
    let project = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    scaffold_project(project.path());

    mx_in(project.path(), home.path())
        .args(["aws", "up"])
        .assert()
        .failure()
        .stderr(contains("mx new <name> --infra aws"));
}
