//! Doc-vs-CLI drift guard: every `mx ...` invocation the docs teach must name
//! a command the CLI actually has.
//!
//! bd:mech-crate-vxq shipped because `INFRA_CONFIG.md` and the MCP server's
//! `mx://docs/commands` resource taught `mx cf ...` long after the
//! command was gone. This test scans those sources for `mx` invocations
//! (inline code spans and command lines inside fenced blocks), keeps the
//! leading run of bare-word tokens (`mx infra setup cloudflare` ->
//! `infra setup cloudflare`), and asserts `mx <tok1> [<tok2>] --help` exits 0.
//! A phantom top-level or nested subcommand fails clap's parse; a trailing
//! positional (`mx new myproject`) still parses, so the check stays honest
//! without hard-coding argument shapes.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use assert_cmd::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/mx-cli should sit two levels below the repo root")
        .to_path_buf()
}

/// Files whose `mx` command claims are pinned to the real CLI.
const SOURCES: &[&str] = &[
    "docs/development/INFRA_CONFIG.md",
    "docs/development/mx-cloudflare-deploy.md",
    "crates/mx-mcp-server/src/mcp/server.rs",
];

fn is_word(tok: &str) -> bool {
    !tok.is_empty()
        && tok
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && tok.chars().next().is_some_and(|c| c.is_ascii_lowercase())
}

/// The first one or two bare-word tokens after `mx`, if any.
fn command_path(invocation: &str) -> Option<Vec<String>> {
    let mut toks = invocation.split_whitespace();
    if toks.next() != Some("mx") {
        return None;
    }
    let path: Vec<String> = toks
        .take_while(|t| is_word(t))
        .take(2)
        .map(str::to_string)
        .collect();
    (!path.is_empty()).then_some(path)
}

/// Every `mx ...` invocation in `text`: inline `` `mx ...` `` spans, plus
/// lines that start with `mx ` (command lines in fenced blocks).
fn invocations(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.lines() {
        let mut rest = line;
        while let Some(start) = rest.find("`mx ") {
            let after = &rest[start + 1..];
            let end = after.find('`').unwrap_or(after.len());
            found.push(after[..end].to_string());
            rest = &after[end..];
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with("mx ") {
            let cmd = trimmed.split('#').next().unwrap_or(trimmed);
            found.push(cmd.trim().to_string());
        }
    }
    found
}

fn documented_paths() -> BTreeSet<(String, Vec<String>)> {
    let root = repo_root();
    let mut paths = BTreeSet::new();
    for src in SOURCES {
        let text = std::fs::read_to_string(root.join(src))
            .unwrap_or_else(|e| panic!("setup: read {src}: {e}"));
        for inv in invocations(&text) {
            if let Some(p) = command_path(&inv) {
                paths.insert((src.to_string(), p));
            }
        }
    }
    paths
}

#[test]
fn scanner_finds_the_documented_commands() {
    let paths = documented_paths();
    let flat: Vec<String> = paths.iter().map(|(_, p)| p.join(" ")).collect();
    // A scanner that silently finds nothing would pass the drift check below.
    for expected in ["infra setup", "infra link", "dev", "cf setup", "cf deploy"] {
        assert!(
            flat.iter().any(|f| f == expected),
            "scanner should find `mx {expected}` in the documented sources, found: {flat:?}"
        );
    }
}

#[test]
fn every_documented_mx_command_exists() {
    let mut phantom = Vec::new();
    for (src, path) in documented_paths() {
        let out = Command::cargo_bin("mx")
            .expect("mx binary built")
            .args(&path)
            .arg("--help")
            .output()
            .expect("run mx --help");
        if !out.status.success() {
            phantom.push(format!(
                "{src}: `mx {}` ({})",
                path.join(" "),
                String::from_utf8_lossy(&out.stderr)
                    .lines()
                    .next()
                    .unwrap_or("")
            ));
        }
    }
    assert!(
        phantom.is_empty(),
        "documented mx commands the CLI does not have:\n  {}",
        phantom.join("\n  ")
    );
}

#[test]
fn command_path_keeps_leading_words_only() {
    assert_eq!(
        command_path("mx infra setup cloudflare"),
        Some(vec!["infra".into(), "setup".into()])
    );
    assert_eq!(command_path("mx add <name>"), Some(vec!["add".into()]));
    assert_eq!(command_path("mx dev [s=service]"), Some(vec!["dev".into()]));
    assert_eq!(
        command_path("mx cf deploy myapp"),
        Some(vec!["cf".into(), "deploy".into()])
    );
    assert_eq!(command_path("mx --help"), None);
    assert_eq!(command_path("mxc foo"), None);
}
