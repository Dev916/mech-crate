//! Cloudflare credential contract (bd:mech-crate-wd9).
//!
//! Two defects with one root: **two names for one credential**, and **one scope
//! for a two-scope config**.
//!
//! `mx infra setup cloudflare` wrote `CLOUDFLARE_ACCOUNT_ID` into the *global*
//! file while the whole deploy toolchain read `CF_ACCOUNT_ID` out of the
//! *project* file, and `cloudflare.mk` `-include`d the project file only. So
//! infra-managed credentials were never consumed by a single `make cf-*` target,
//! and `cf-init` answered "CF_ACCOUNT_ID not set" for a machine that was fully
//! configured.
//!
//! The contract these tests hold:
//!
//! 1. One canonical pair, `CLOUDFLARE_ACCOUNT_ID` + `CLOUDFLARE_API_TOKEN` (the
//!    names wrangler itself reads from the environment, so a credentials file
//!    value reaches the deploy with no translation step). `CF_ACCOUNT_ID` /
//!    `CF_API_TOKEN` survive as deprecated aliases that consumers *read* and no
//!    writer emits.
//! 2. Both scopes are read, global then project, and the **project wins**,
//!    per-scope, so a project file carrying only the deprecated alias still beats
//!    a global file carrying the canonical name.
//! 3. Neither scope populated fails loudly, naming `mx infra setup cloudflare`.
//!
//! The static half pins the name contract across the Rust writer and the shipped
//! templates; the behavioral half runs real `make` against scratch projects with
//! a scratch `HOME`, because the precedence lives in makefile include order and
//! nothing but `make` evaluates that faithfully.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const CANONICAL_ACCOUNT_ID: &str = "CLOUDFLARE_ACCOUNT_ID";
const DEPRECATED_ACCOUNT_ID: &str = "CF_ACCOUNT_ID";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn templates_root() -> PathBuf {
    repo_root().join("templates")
}

fn read(path: &PathBuf) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("setup: read {}: {e}", path.display()))
}

fn cloudflare_mk() -> String {
    read(&templates_root().join("make/cloudflare.mk"))
}

fn infra_rs() -> String {
    read(&repo_root().join("crates/mx-cli/src/commands/infra.rs"))
}

// ── Static half: the name contract ───────────────────────────────────────────

/// Every `NAME_ACCOUNT_ID={}` literal the Rust side writes into a config file.
fn account_id_vars_written(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in src.lines() {
        if let Some(pos) = line.find("ACCOUNT_ID={}") {
            let end = pos + "ACCOUNT_ID".len();
            let start = line[..end]
                .rfind(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
                .map(|i| i + 1)
                .unwrap_or(0);
            out.push(line[start..end].to_string());
        }
    }
    out.sort();
    out.dedup();
    out
}

/// `*_ACCOUNT_ID` names a makefile dereferences as `$(NAME)`.
fn account_id_vars_dereferenced(mk: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = mk;
    while let Some(i) = rest.find("$(") {
        let after = &rest[i + 2..];
        let end = after.find(')').unwrap_or(after.len());
        let name = &after[..end];
        if name.ends_with("ACCOUNT_ID") {
            out.push(name.to_string());
        }
        rest = &after[end..];
    }
    out.sort();
    out.dedup();
    out
}

/// The original defect, as a contract: `mx infra setup cloudflare` writes exactly
/// one account-id variable, and it is one `cloudflare.mk` reads.
#[test]
fn the_written_account_id_variable_is_one_the_mk_reads() {
    let written = account_id_vars_written(&infra_rs());
    assert_eq!(
        written,
        vec![CANONICAL_ACCOUNT_ID.to_string()],
        "mx infra setup cloudflare must write exactly the canonical account-id \
         variable; found {written:?}"
    );

    let dereferenced = account_id_vars_dereferenced(&cloudflare_mk());
    assert!(
        dereferenced.contains(&written[0]),
        "mx infra setup writes {} but cloudflare.mk only dereferences {:?}, so \
         infra-managed credentials would never be read by the deploy toolchain",
        written[0],
        dereferenced
    );
}

/// `cloudflare.mk` must read both credentials files, global first so the project
/// `-include` that follows overrides it.
#[test]
fn the_mk_includes_the_global_credentials_file_before_the_project_one() {
    let mk = cloudflare_mk();

    let global_include = mk
        .find("-include $(CF_GLOBAL_ENV_FILE)")
        .expect("cloudflare.mk must -include the global credentials file");
    let project_include = mk
        .find("-include $(CF_ENV_FILE)")
        .expect("cloudflare.mk must -include the project credentials file");

    assert!(
        global_include < project_include,
        "the global credentials file must be included BEFORE the project one, so \
         the project's values override the global ones"
    );

    assert!(
        mk.contains("CF_GLOBAL_ENV_FILE := $(MX_HOME)/config/infra/cloudflare.env"),
        "the global credentials path must be the one mx infra setup writes \
         (~/.mech-crate/config/infra/cloudflare.env)"
    );
}

/// Names one file assigns a **non-empty** value to: `KEY=value` at the start of a
/// line (or after `export`), which is what a makefile `-include` or a `source`
/// would define. Shell parameter expansions (`${KEY:-…}`) and comparisons are
/// reads, not writes; `KEY=` / `KEY :=` with nothing on the right is a *clear*,
/// which is how the per-scope resolution isolates one credentials file from the
/// next.
fn names_assigned_a_value(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in body.lines() {
        let t = line.trim().trim_start_matches("export ").trim();
        let Some(eq) = t.find('=') else { continue };
        let name = t[..eq].trim().trim_end_matches([':', '?', '+']).trim();
        let value = t[eq + 1..].trim().trim_matches('"').trim_matches('\'');
        if value.is_empty() {
            continue;
        }
        if !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        {
            out.push(name.to_string());
        }
    }
    out
}

/// Every shipped file under `templates/` that mentions the deprecated alias, plus
/// the repo's own `bin/lib/` shell mx. The alias may be **read** (back-compat for
/// an `.env.cloudflare` an older `cf-setup.sh` wrote) but never **written**: a
/// writer that emits it recreates the split this issue is about.
#[test]
fn nothing_shipped_writes_the_deprecated_account_id_alias() {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else {
                out.push(path);
            }
        }
    }

    let mut files = Vec::new();
    walk(&templates_root(), &mut files);
    walk(&repo_root().join("bin/lib"), &mut files);
    files.sort();

    let mut mentions = 0;
    let mut offenders: Vec<String> = Vec::new();

    for path in &files {
        // Prose is not a writer. A `.md` example naming the old variable is a doc
        // correction, tracked separately, not a credential this tree emits.
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            continue;
        }
        let Ok(body) = std::fs::read_to_string(path) else {
            continue; // binary or unreadable fixture
        };
        if !body.contains(DEPRECATED_ACCOUNT_ID) {
            continue;
        }
        mentions += 1;

        // A local shell variable named exactly the alias is the writer's own
        // scratch slot and is just as bad: whatever it holds ends up in a file.
        if names_assigned_a_value(&body)
            .iter()
            .any(|n| n == DEPRECATED_ACCOUNT_ID)
        {
            offenders.push(
                path.strip_prefix(repo_root())
                    .unwrap_or(path)
                    .display()
                    .to_string(),
            );
        }
    }

    assert!(
        mentions >= 2,
        "setup: expected the shipped tree to still READ the deprecated alias in \
         several places (back-compat), saw {mentions} mention(s). If the alias \
         was dropped entirely, retire this test rather than weakening it"
    );
    assert!(
        offenders.is_empty(),
        "{} shipped file(s) still ASSIGN {DEPRECATED_ACCOUNT_ID}; it is a \
         read-only deprecated alias, and writing it is what made \
         infra-managed credentials invisible to the deploy toolchain:\n  {}",
        offenders.len(),
        offenders.join("\n  ")
    );
}

// ── Behavioral half: real `make` against scratch scopes ──────────────────────

/// A scratch project whose `Makefile` / `make/*.mk` / `scripts/` are the shipped
/// templates verbatim, plus the `infra/cloudflare/` tree `cf-setup.sh` creates.
fn template_project(root: &Path) -> PathBuf {
    for dir in ["make", "scripts", "infra/cloudflare/apps"] {
        std::fs::create_dir_all(root.join(dir))
            .unwrap_or_else(|e| panic!("setup: create {dir}: {e}"));
    }

    let templates = templates_root();
    std::fs::copy(templates.join("Makefile.template"), root.join("Makefile"))
        .expect("setup: copy Makefile.template");
    for entry in std::fs::read_dir(templates.join("make")).expect("setup: read templates/make") {
        let src = entry.expect("setup: dir entry").path();
        if src.extension().and_then(|e| e.to_str()) == Some("mk") {
            std::fs::copy(&src, root.join("make").join(src.file_name().unwrap()))
                .expect("setup: copy make module");
        }
    }

    root.to_path_buf()
}

/// Run one make target in `project` with `home` standing in for `$HOME`, and with
/// every credential name scrubbed from the environment: that scope has the
/// highest precedence, so a developer's own shell would otherwise decide
/// the outcome of every assertion below.
fn make_in(project: &Path, home: &Path, args: &[&str]) -> Output {
    Command::new("make")
        .current_dir(project)
        .args(args)
        .env("HOME", home)
        .env_remove("MX_HOME")
        .env_remove("CLOUDFLARE_ACCOUNT_ID")
        .env_remove("CLOUDFLARE_API_TOKEN")
        .env_remove(DEPRECATED_ACCOUNT_ID)
        .env_remove("CF_API_TOKEN")
        .env_remove("MAKEFLAGS")
        .output()
        .expect("setup: run make")
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

/// `KEY=VALUE` line of `make cf-vars` output.
fn var(out: &Output, key: &str) -> String {
    let stdout = stdout_of(out);
    stdout
        .lines()
        .find_map(|l| l.strip_prefix(&format!("{key}=")))
        .unwrap_or_else(|| {
            panic!(
                "cf-vars printed no {key} line; got:\n{stdout}{}",
                stderr_of(out)
            )
        })
        .trim()
        .to_string()
}

fn write_global(home: &Path, body: &str) {
    let dir = home.join(".mech-crate/config/infra");
    std::fs::create_dir_all(&dir).expect("setup: global infra dir");
    std::fs::write(dir.join("cloudflare.env"), body).expect("setup: write global cloudflare.env");
}

fn write_project(project: &Path, body: &str) {
    std::fs::write(project.join("infra/cloudflare/.env.cloudflare"), body)
        .expect("setup: write project .env.cloudflare");
}

/// Global credentials alone must reach the deploy toolchain. This is the whole
/// defect: before the fix, `cloudflare.mk` never looked at the global file, so a
/// machine configured by `mx infra setup cloudflare` deployed nothing.
#[test]
fn a_populated_global_file_alone_reaches_the_make_layer() {
    let home = tempfile::tempdir().expect("setup: home tempdir");
    let proj = tempfile::tempdir().expect("setup: project tempdir");
    let project = template_project(proj.path());

    write_global(
        home.path(),
        "CLOUDFLARE_ACCOUNT_ID=global-acct\nCLOUDFLARE_API_TOKEN=global-token\n",
    );

    let out = make_in(&project, home.path(), &["cf-vars"]);
    assert!(
        out.status.success(),
        "setup: make cf-vars must exit 0, got: {}{}",
        stdout_of(&out),
        stderr_of(&out)
    );

    assert_eq!(
        var(&out, CANONICAL_ACCOUNT_ID),
        "global-acct",
        "the global credentials file must be resolved when no project file exists"
    );
    assert_eq!(var(&out, "CLOUDFLARE_ACCOUNT_ID_SOURCE"), "global");
}

/// Both scopes populated: the project file wins.
#[test]
fn the_project_file_overrides_the_global_one() {
    let home = tempfile::tempdir().expect("setup: home tempdir");
    let proj = tempfile::tempdir().expect("setup: project tempdir");
    let project = template_project(proj.path());

    write_global(home.path(), "CLOUDFLARE_ACCOUNT_ID=global-acct\n");
    write_project(&project, "CLOUDFLARE_ACCOUNT_ID=project-acct\n");

    let out = make_in(&project, home.path(), &["cf-vars"]);
    assert!(
        out.status.success(),
        "setup: make cf-vars must exit 0, got: {}{}",
        stdout_of(&out),
        stderr_of(&out)
    );

    assert_eq!(
        var(&out, CANONICAL_ACCOUNT_ID),
        "project-acct",
        "a project credentials file must override the global one"
    );
    assert_eq!(var(&out, "CLOUDFLARE_ACCOUNT_ID_SOURCE"), "project");
}

/// Back-compat, and the reason precedence is resolved per scope rather than by a
/// flat `?=` chain: a project file written by an older `cf-setup.sh` carries only
/// `CF_ACCOUNT_ID`, and it must still beat a global file carrying the canonical
/// name. A naive `CLOUDFLARE_ACCOUNT_ID ?= $(CF_ACCOUNT_ID)` after both includes
/// inverts this and silently deploys to the wrong account.
#[test]
fn a_project_file_with_only_the_deprecated_alias_still_beats_the_global_file() {
    let home = tempfile::tempdir().expect("setup: home tempdir");
    let proj = tempfile::tempdir().expect("setup: project tempdir");
    let project = template_project(proj.path());

    write_global(home.path(), "CLOUDFLARE_ACCOUNT_ID=global-acct\n");
    write_project(&project, "CF_ACCOUNT_ID=legacy-project-acct\n");

    let out = make_in(&project, home.path(), &["cf-vars"]);
    assert!(
        out.status.success(),
        "setup: make cf-vars must exit 0, got: {}{}",
        stdout_of(&out),
        stderr_of(&out)
    );

    assert_eq!(
        var(&out, CANONICAL_ACCOUNT_ID),
        "legacy-project-acct",
        "the deprecated alias must still be read, at its own scope's precedence"
    );
    assert_eq!(var(&out, "CLOUDFLARE_ACCOUNT_ID_SOURCE"), "project");

    let check = make_in(&project, home.path(), &["cf-check-credentials"]);
    assert!(
        check.status.success(),
        "a project file using the deprecated alias must still pass the guard, got: {}{}",
        stdout_of(&check),
        stderr_of(&check)
    );
    assert!(
        stdout_of(&check).contains("deprecated"),
        "reading the deprecated alias must warn about it; got:\n{}",
        stdout_of(&check)
    );
}

/// Neither scope populated: fail loudly, and name the command that fixes it. The
/// old behavior was an empty variable carried silently into `wrangler deploy`.
#[test]
fn missing_credentials_fail_with_an_actionable_message() {
    let home = tempfile::tempdir().expect("setup: home tempdir");
    let proj = tempfile::tempdir().expect("setup: project tempdir");
    let project = template_project(proj.path());

    let vars = make_in(&project, home.path(), &["cf-vars"]);
    assert_eq!(
        var(&vars, CANONICAL_ACCOUNT_ID),
        "",
        "setup: no credentials file exists, so nothing may resolve"
    );
    assert_eq!(var(&vars, "CLOUDFLARE_ACCOUNT_ID_SOURCE"), "none");

    let out = make_in(&project, home.path(), &["cf-check-credentials"]);
    assert!(
        !out.status.success(),
        "unconfigured credentials must fail, not carry an empty account id into a \
         deploy; got:\n{}",
        stdout_of(&out)
    );

    let message = format!("{}{}", stdout_of(&out), stderr_of(&out));
    for needle in [
        "mx infra setup cloudflare",
        "make cf-setup",
        CANONICAL_ACCOUNT_ID,
    ] {
        assert!(
            message.contains(needle),
            "the failure must name {needle:?}; got:\n{message}"
        );
    }
}

/// The resolved account id has to reach the commands that use it, not just the
/// var dump: the container registry path is built from it.
#[test]
fn the_resolved_account_id_reaches_the_deploy_pipeline() {
    let home = tempfile::tempdir().expect("setup: home tempdir");
    let proj = tempfile::tempdir().expect("setup: project tempdir");
    let project = template_project(proj.path());

    write_global(home.path(), "CLOUDFLARE_ACCOUNT_ID=global-acct\n");

    let out = make_in(&project, home.path(), &["-n", "cf-sync-image", "a=demo"]);
    assert!(
        out.status.success(),
        "setup: make -n cf-sync-image must exit 0, got: {}{}",
        stdout_of(&out),
        stderr_of(&out)
    );

    let printed = stdout_of(&out);
    assert!(
        printed.contains("registry.cloudflare.com/global-acct/demo"),
        "the registry path must carry the resolved account id; got:\n{printed}"
    );
}
