//! Known-broken TDD lane — mx-cli side.
//!
//! Every test here asserts the behavior a tracked bd defect promises *once
//! fixed*, and is `#[ignore]`d with that defect's id so the gate stays green.
//! `make test-known-broken` (`cargo nextest run --run-ignored only`) is the
//! scoreboard: these are expected RED. A lane test that turns green means the
//! fix landed — un-ignore it, move it into the gate, close the bd issue.
//!
//! House rule for this file: the *arrange* half of every test must succeed
//! today. A lane test that blows up on a missing fixture tells us nothing
//! about the defect, so setup failures carry a `setup:` prefix in their
//! message to distinguish them from the subject assertion.
//!
//! See `tests/KNOWN_BROKEN.md` at the repo root for the id ↔ test ↔ behavior
//! mapping.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mx_lib::test_support::{scaffold_project, StubBin};

// ── shared fixtures ──────────────────────────────────────────────────────────

/// The `mx` binary cargo built for this integration test.
fn mx() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mx"))
}

/// Repo root (`crates/mx-cli` sits two levels down).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/mx-cli should sit two levels below the repo root")
        .to_path_buf()
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

/// A project whose `Makefile`/`make/*.mk` are the real shipped templates, plus
/// a recording `scripts/build.sh` so a build can be observed without Docker.
///
/// Returns the path of the file the recorder appends its argv to.
fn template_project(root: &Path) -> PathBuf {
    scaffold_project(root);

    let templates = repo_root().join("templates");
    std::fs::copy(templates.join("Makefile.template"), root.join("Makefile"))
        .expect("setup: copy Makefile.template");
    for entry in std::fs::read_dir(templates.join("make")).expect("setup: read templates/make") {
        let src = entry.expect("setup: dir entry").path();
        if src.extension().and_then(|e| e.to_str()) == Some("mk") {
            let dst = root.join("make").join(src.file_name().unwrap());
            std::fs::copy(&src, &dst).expect("setup: copy make module");
        }
    }

    let recording = root.join("build-args.txt");
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> {}\n",
        recording.display()
    );
    let build_sh = root.join("scripts/build.sh");
    std::fs::write(&build_sh, script).expect("setup: write recording build.sh");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&build_sh, std::fs::Permissions::from_mode(0o755))
            .expect("setup: chmod build.sh");
    }

    recording
}

// ── 2. bd:mech-crate-wd9 — cloudflare account-id variable contract ───────────

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

/// `*_ACCOUNT_ID` names a makefile dereferences but never assigns — i.e. the
/// ones it expects the `-include`d credentials file to provide.
fn account_id_vars_consumed(mk: &str) -> Vec<String> {
    let mut assigned: HashSet<String> = HashSet::new();
    for line in mk.lines() {
        let t = line.trim().trim_start_matches("export ").trim();
        if let Some(eq) = t.find('=') {
            let name = t[..eq].trim().trim_end_matches([':', '?', '+']).trim();
            let ident = !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
            if ident {
                assigned.insert(name.to_string());
            }
        }
    }

    let mut out = Vec::new();
    let mut rest = mk;
    while let Some(i) = rest.find("$(") {
        let after = &rest[i + 2..];
        let end = after.find(')').unwrap_or(after.len());
        let name = &after[..end];
        if name.ends_with("ACCOUNT_ID") && !assigned.contains(name) {
            out.push(name.to_string());
        }
        rest = &after[end..];
    }
    out.sort();
    out.dedup();
    out
}

/// `mx infra setup cloudflare` writes one account-id variable; the deploy
/// toolchain reads another. Assert the CONTRACT: the name written is a name
/// `cloudflare.mk` actually consumes from the credentials file.
#[test]
#[ignore = "bd:mech-crate-wd9 mx writes CLOUDFLARE_ACCOUNT_ID, cloudflare.mk reads CF_ACCOUNT_ID"]
fn kb_cloudflare_account_id_var_is_one_contract() {
    let infra_rs = std::fs::read_to_string(repo_root().join("crates/mx-cli/src/commands/infra.rs"))
        .expect("setup: read infra.rs");
    let written = account_id_vars_written(&infra_rs);
    assert_eq!(
        written.len(),
        1,
        "setup: expected exactly one *_ACCOUNT_ID written by mx infra setup, found {written:?}"
    );

    let mk = std::fs::read_to_string(repo_root().join("templates/make/cloudflare.mk"))
        .expect("setup: read cloudflare.mk");
    let consumed = account_id_vars_consumed(&mk);
    assert!(
        !consumed.is_empty(),
        "setup: cloudflare.mk dereferences no *_ACCOUNT_ID at all"
    );

    assert!(
        consumed.contains(&written[0]),
        "mx infra setup writes {} but cloudflare.mk consumes {:?} — \
         infra-managed credentials are never read by the deploy toolchain",
        written[0],
        consumed
    );
}

// ── 3. bd:mech-crate-066 — infra link/unlink are stubs ───────────────────────

/// `mx infra link <provider>` must actually link: write the marker the Rust
/// resolver looks for, so the project resolves to the global credentials.
#[test]
#[ignore = "bd:mech-crate-066 mx infra link/unlink only print a message"]
fn kb_infra_link_writes_marker_and_inspect_resolves_global() {
    let home = tempfile::tempdir().expect("setup: home tempdir");
    let proj = tempfile::tempdir().expect("setup: project tempdir");
    scaffold_project(proj.path());

    let global_dir = home.path().join(".mech-crate/config/infra");
    std::fs::create_dir_all(&global_dir).expect("setup: global infra dir");
    std::fs::write(
        global_dir.join("cloudflare.env"),
        "CF_ACCOUNT_ID=global-account\n",
    )
    .expect("setup: write global cloudflare.env");

    let project_cfg = proj.path().join("infra/cloudflare");
    std::fs::create_dir_all(&project_cfg).expect("setup: project infra dir");
    std::fs::write(
        project_cfg.join(".env.cloudflare"),
        "CF_ACCOUNT_ID=project-account\n",
    )
    .expect("setup: write project .env.cloudflare");

    let inspect = || {
        mx().current_dir(proj.path())
            .env("HOME", home.path())
            .env_remove("MECH_CRATE_ROOT")
            .args(["infra", "inspect", "cloudflare"])
            .output()
            .expect("setup: run mx infra inspect")
    };

    let before = inspect();
    assert!(
        before.status.success() && stdout_of(&before).contains("project-account"),
        "setup: unlinked project must resolve to its own credentials, got: {}{}",
        stdout_of(&before),
        stderr_of(&before)
    );

    let link = mx()
        .current_dir(proj.path())
        .env("HOME", home.path())
        .env_remove("MECH_CRATE_ROOT")
        .args(["infra", "link", "cloudflare"])
        .output()
        .expect("setup: run mx infra link");
    assert!(
        link.status.success(),
        "setup: mx infra link must exit 0, got: {}",
        stderr_of(&link)
    );

    let marker = project_cfg.join(".env.linked");
    assert!(
        marker.exists(),
        "mx infra link must create the resolver's marker at {}",
        marker.display()
    );

    let after = inspect();
    assert!(
        after.status.success() && stdout_of(&after).contains("global-account"),
        "after linking, mx infra inspect must resolve to the global config, got: {}{}",
        stdout_of(&after),
        stderr_of(&after)
    );
}

// ── 4. bd:mech-crate-vxq — phantom `mx cf` command ───────────────────────────

/// `INFRA_CONFIG.md` and the MCP server's resource text both document `mx cf`.
/// If we keep the docs, the command has to exist (retire this test instead if
/// the issue is closed by purging the doc references).
#[test]
#[ignore = "bd:mech-crate-vxq documented `mx cf` subcommand does not exist"]
fn kb_mx_cf_subcommand_exists() {
    let out = mx()
        .args(["cf", "--help"])
        .output()
        .expect("setup: run mx cf");
    assert!(
        out.status.success(),
        "documented `mx cf` must answer --help, got: {}",
        stderr_of(&out)
    );
}

// ── 5. bd:mech-crate-9be — `recipes apply --fix` is a dead flag ──────────────

/// `--fix` is bound as `_fix` and ignored, so `apply` and `apply --fix` do the
/// same thing. When the advertised dependency-drift comparison lands, the two
/// runs must differ.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "bd:mech-crate-9be recipes apply --fix is bound as _fix and ignored"]
async fn kb_recipes_apply_fix_changes_behavior() {
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/v1/orgs/.+/recipes/demo$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "rec_1",
            "name": "demo",
            "description": "demo recipe",
            "version": "1.0.0",
            "patterns": [{
                "name": "Repository pattern",
                "description": "Data access behind a port",
                "rules": ["No SQL in controllers"]
            }],
            "dependencies": [{ "name": "vue", "version": "^3.5.0" }],
            "infrastructure": { "services": ["postgres"] }
        })))
        .mount(&server)
        .await;

    let home = tempfile::tempdir().expect("setup: home tempdir");
    let proj = tempfile::tempdir().expect("setup: project tempdir");
    scaffold_project(proj.path());

    let unyform_dir = home.path().join(".mech-crate/config/unyform");
    std::fs::create_dir_all(&unyform_dir).expect("setup: unyform config dir");
    std::fs::write(
        unyform_dir.join("credentials.json"),
        serde_json::json!({
            "api_key": "test-key",
            "url": server.uri(),
            "org_id": "org_test"
        })
        .to_string(),
    )
    .expect("setup: write credentials.json");

    let apply = |fix: bool| {
        let mut cmd = mx();
        cmd.current_dir(proj.path())
            .env("HOME", home.path())
            .env_remove("MECH_CRATE_ROOT")
            .args(["recipes", "apply", "demo"]);
        if fix {
            cmd.arg("--fix");
        }
        cmd.output().expect("setup: run mx recipes apply")
    };

    let plain = tokio::task::block_in_place(|| apply(false));
    let fixed = tokio::task::block_in_place(|| apply(true));

    assert!(
        plain.status.success(),
        "setup: mx recipes apply must succeed against the stub API, got: {}",
        stderr_of(&plain)
    );
    assert!(
        fixed.status.success(),
        "setup: mx recipes apply --fix must succeed against the stub API, got: {}",
        stderr_of(&fixed)
    );

    assert_ne!(
        stdout_of(&plain),
        stdout_of(&fixed),
        "--fix must perform the advertised dependency-drift comparison, \
         but its output is identical to a plain apply"
    );
}

// ── 7. bd:mech-crate-dpi — `mx build --platform` dropped by the make layer ───

/// The CLI forwards `platform=<p>` as a make variable, but `_build` never
/// passes it to `build.sh`, so it never reaches Docker.
#[test]
#[ignore = "bd:mech-crate-dpi make _build drops platform= before build.sh"]
fn kb_build_platform_reaches_the_build_script() {
    let proj = tempfile::tempdir().expect("setup: project tempdir");
    let recording = template_project(proj.path());

    let out = mx()
        .current_dir(proj.path())
        .args(["build", "svc", "--prod", "--platform", "linux/arm64"])
        .output()
        .expect("setup: run mx build");
    assert!(
        out.status.success(),
        "setup: mx build must reach the make layer, got: {}{}",
        stdout_of(&out),
        stderr_of(&out)
    );

    let recorded = std::fs::read_to_string(&recording).unwrap_or_default();
    assert!(
        !recorded.trim().is_empty(),
        "setup: scripts/build.sh was never invoked (recording at {} is empty)",
        recording.display()
    );

    assert!(
        recorded.contains("linux/arm64"),
        "mx build --platform linux/arm64 must reach build.sh, but it recorded: {}",
        recorded.trim()
    );
}

// ── 8. bd:mech-crate-fr6 — no `make rebuild` ─────────────────────────────────

/// `rebuild` is declared `.PHONY` in `common.mk` but never defined, so
/// `make rebuild` silently does nothing (exit 0, no recipe).
#[test]
#[ignore = "bd:mech-crate-fr6 rebuild is .PHONY-declared but has no recipe"]
fn kb_make_rebuild_is_a_real_target() {
    let proj = tempfile::tempdir().expect("setup: project tempdir");
    template_project(proj.path());

    let out = Command::new("make")
        .args(["-n", "rebuild", "s=svc"])
        .current_dir(proj.path())
        .output()
        .expect("setup: run make -n rebuild");
    assert!(
        out.status.success(),
        "setup: make -n rebuild must not error, got: {}",
        stderr_of(&out)
    );

    assert!(
        !stdout_of(&out).trim().is_empty(),
        "make rebuild must be a real target that runs something; \
         `make -n rebuild` printed no commands at all"
    );
}

// ── 11. bd:mech-crate-3jt — recipe `version` is dead metadata ────────────────

/// `recipe.json`'s `version` is parsed but never displayed, compared or gated.
/// Surfacing it in `mx recipes info` is the minimum bar.
#[cfg(unix)]
#[test]
#[ignore = "bd:mech-crate-3jt recipe.json version is parsed but never surfaced"]
fn kb_recipes_info_shows_the_recipe_version() {
    let recipe_json =
        std::fs::read_to_string(repo_root().join("templates/recipes/nuxt/recipe.json"))
            .expect("setup: read nuxt recipe.json");
    let recipe: serde_json::Value =
        serde_json::from_str(&recipe_json).expect("setup: parse nuxt recipe.json");
    let version = recipe["version"]
        .as_str()
        .expect("setup: nuxt recipe.json must declare a version")
        .to_string();

    let home = tempfile::tempdir().expect("setup: home tempdir");
    let mech = home.path().join(".mech-crate");
    std::fs::create_dir_all(&mech).expect("setup: fake mx home");
    std::os::unix::fs::symlink(repo_root().join("templates"), mech.join("templates"))
        .expect("setup: link templates into fake home");

    let out = mx()
        .env("HOME", home.path())
        .env_remove("MECH_CRATE_ROOT")
        .args(["recipes", "info", "nuxt"])
        .output()
        .expect("setup: run mx recipes info");
    assert!(
        out.status.success(),
        "setup: mx recipes info nuxt must exit 0, got: {}",
        stderr_of(&out)
    );

    let stdout = stdout_of(&out);
    assert!(
        stdout.contains(&version),
        "mx recipes info must surface the recipe version ({version}), got:\n{stdout}"
    );
}

// ── 12. bd:mech-crate-o5f — `mx router inspect` parity ───────────────────────

/// The Rust `inspect` is a bare alias of `status`; the shell version enumerates
/// the containers attached to the `devmesh-traefik` network.
#[test]
#[ignore = "bd:mech-crate-o5f router inspect is a bare alias of status"]
fn kb_router_inspect_lists_connected_services() {
    let home = tempfile::tempdir().expect("setup: home tempdir");
    let router_dir = home.path().join(".mech-crate/router");
    std::fs::create_dir_all(&router_dir).expect("setup: router install dir");
    std::fs::write(router_dir.join("docker-compose.yml"), "services: {}\n")
        .expect("setup: write router compose file");

    // One canned reply for every docker call: a network inspect payload with
    // two attached containers.
    let network_json = r#"[{"Name":"devmesh-traefik","Containers":{"abc123":{"Name":"shop-web-1"},"def456":{"Name":"shop-api-1"}}}]"#;
    let sb = StubBin::new();
    sb.stub("docker", 0, network_json);

    let out = mx()
        .env("HOME", home.path())
        .env("PATH", sb.path_env())
        .env_remove("MECH_CRATE_ROOT")
        .args(["router", "inspect"])
        .output()
        .expect("setup: run mx router inspect");
    assert!(
        out.status.success(),
        "setup: mx router inspect must exit 0, got: {}",
        stderr_of(&out)
    );
    let stdout = stdout_of(&out);
    assert!(
        stdout.contains("devmesh-traefik"),
        "setup: inspect should at least report the network, got:\n{stdout}"
    );

    for container in ["shop-web-1", "shop-api-1"] {
        assert!(
            stdout.contains(container),
            "mx router inspect must list the services connected to the router \
             network; {container} missing from:\n{stdout}"
        );
    }
}

// ── 13. bd:mech-crate-dqw — `mx doctor` has no router checks ─────────────────

/// doctor validates structure/docker/make and says nothing about the router,
/// which is the most common cause of a dead `http://<svc>.localhost`.
#[test]
#[ignore = "bd:mech-crate-dqw mx doctor performs zero router checks"]
fn kb_doctor_reports_router_status() {
    let home = tempfile::tempdir().expect("setup: home tempdir");
    let proj = tempfile::tempdir().expect("setup: project tempdir");
    scaffold_project(proj.path());

    let sb = StubBin::new();
    sb.stub("docker", 0, "Docker version 99.0.0, build stub");
    sb.stub("make", 0, "");

    let out = mx()
        .current_dir(proj.path())
        .env("HOME", home.path())
        .env("PATH", sb.path_env())
        .env_remove("MECH_CRATE_ROOT")
        .arg("doctor")
        .output()
        .expect("setup: run mx doctor");
    assert!(
        out.status.success(),
        "setup: mx doctor must exit 0, got: {}",
        stderr_of(&out)
    );
    let stdout = stdout_of(&out);
    assert!(
        stdout.contains("Project Structure"),
        "setup: doctor did not run its project checks, got:\n{stdout}"
    );

    assert!(
        stdout.to_lowercase().contains("router"),
        "mx doctor must report on the router (network, container, port 80), got:\n{stdout}"
    );
}
