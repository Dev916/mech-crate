//! `mx new --infra cloudflare` substitutes every `{{PLACEHOLDER}}` token the
//! `templates/infra/cloudflare/` tree ships (bd:mech-crate-pdr).
//!
//! The recipe placeholder net (`recipes_conformance.rs`) only walks
//! `templates/recipes/`, so the eleven `{{PROJECT_NAME}}` tokens in
//! `templates/infra/cloudflare/containers-worker` had no guard. The static half
//! of the infra net sits next to the recipe net in `recipes_conformance.rs`;
//! this is the behavioral half: a real `mx new --no-prompt --infra cloudflare`
//! against the repo templates must leave no SCREAMING_SNAKE `{{TOKEN}}`
//! anywhere in the generated project.

use std::path::{Path, PathBuf};

use assert_cmd::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/mx-cli should sit two levels below the repo root")
        .to_path_buf()
}

/// SCREAMING_SNAKE `{{NAME}}` tokens (optionally `{{ NAME }}` / `{{- NAME -}}`),
/// the installer-owned grammar. Foreign `{{ page.title }}`-style syntax is not
/// matched.
fn placeholder_tokens(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else { break };
        let inner = after[..end].trim_matches('-').trim();
        let screaming = inner.starts_with(|c: char| c.is_ascii_uppercase())
            && inner
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
        if screaming {
            out.push(inner.to_string());
        }
        rest = &after[end + 2..];
    }
    out
}

fn walk_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d)
            .unwrap_or_else(|e| panic!("setup: read {}: {e}", d.display()))
            .flatten()
        {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    out
}

#[test]
fn placeholder_scanner_matches_only_installer_tokens() {
    assert_eq!(
        placeholder_tokens("a {{PROJECT_NAME}} b {{ RUST_VERSION }} {{- X_1 -}}"),
        vec!["PROJECT_NAME", "RUST_VERSION", "X_1"]
    );
    assert!(placeholder_tokens("{{ page.title }} {{ config.base_url | safe }}").is_empty());
}

#[test]
fn mx_new_with_cloudflare_infra_leaves_no_placeholder_token() {
    let home = tempfile::tempdir().expect("setup: home tempdir");
    // `mx new` refuses to run until `mx init` has installed templates.
    std::fs::create_dir_all(home.path().join(".mech-crate/templates/recipes"))
        .expect("setup: fake mx init marker");
    let work = tempfile::tempdir().expect("setup: work tempdir");

    let out = Command::cargo_bin("mx")
        .expect("mx binary built")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("MECH_CRATE_ROOT", repo_root())
        .env_remove("MX_HOME")
        .args(["new", "pdr-demo", "--no-prompt", "--infra", "cloudflare"])
        .output()
        .expect("setup: spawn mx new");
    assert!(
        out.status.success(),
        "mx new failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let project = work.path().join("pdr-demo");
    let worker = project.join("infra/cloudflare/containers-worker");
    assert!(
        worker.join("wrangler.toml").is_file(),
        "setup: the cloudflare containers-worker must be installed"
    );

    let mut leftovers: Vec<String> = Vec::new();
    let mut files = walk_files(&project);
    files.sort();
    for file in files {
        let Ok(body) = std::fs::read_to_string(&file) else {
            continue;
        };
        let mut tokens = placeholder_tokens(&body);
        tokens.sort();
        tokens.dedup();
        for token in tokens {
            leftovers.push(format!(
                "  {} still has {{{{{token}}}}}",
                file.strip_prefix(&project).unwrap().display()
            ));
        }
    }
    assert!(
        leftovers.is_empty(),
        "`mx new --infra cloudflare` left placeholder tokens in the project:\n{}",
        leftovers.join("\n")
    );

    // The tokens became the project slug, not something else.
    let wrangler = std::fs::read_to_string(worker.join("wrangler.toml")).unwrap();
    assert!(
        wrangler.contains("name = \"pdr-demo-worker\""),
        "wrangler.toml should name the worker after the project:\n{wrangler}"
    );
    let pkg = std::fs::read_to_string(worker.join("package.json")).unwrap();
    assert!(pkg.contains("\"pdr-demo-containers-worker\""), "{pkg}");
    let index = std::fs::read_to_string(worker.join("src/index.ts")).unwrap();
    assert!(index.contains("'pdr-demo-ssr'"), "{index}");
}
