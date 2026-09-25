//! `site/scripts/` copies of shipped `templates/scripts/` stay byte-identical.
//!
//! `site/` is this repository's own mx project (the one that builds
//! mechcrate.dev), scaffolded from `templates/` and carrying verbatim copies of
//! some of the scripts mx ships. A copy with no test behind it is a copy that
//! goes stale silently: bd:mech-crate-wd9 rewrote the Cloudflare credential
//! resolution in `templates/scripts/cf-setup.sh` and
//! `templates/scripts/cf-init-app.sh`, and the `site/scripts/` twins kept
//! writing the deprecated `CF_ACCOUNT_ID` spelling with nothing failing.
//!
//! So the pairs below are pinned. The list is explicit on purpose: most of
//! `site/scripts/` has legitimately diverged from `templates/` as the templates
//! moved on, and a test that discovered its own expectation from whatever
//! happens to match today would assert nothing.
//!
//! If this test goes red there are two correct fixes, and which one applies is a
//! judgement about the file, not about the test:
//!
//! 1. Re-copy the template over the site twin, so the site project keeps running
//!    the same script mx hands a new project.
//! 2. Delete the site twin and drop its row here, if `site/` does not use the
//!    script at all. `site/` was scaffolded without `--infra cloudflare`, so it
//!    has no `make/cloudflare.mk` and nothing there invokes the Cloudflare pair
//!    today. Keeping them is a deliberate call: they are the copies a future
//!    `--infra cloudflare` on this project would rely on, and pinning them costs
//!    less than discovering the drift from a broken deploy.
//!
//! What is *not* pinned: the scripts that already differ between the two trees
//! (`build.sh`, `doctor.sh`, `init.sh`, and the small compose wrappers). Those
//! differences predate this test and settling them is separate work.

use std::path::{Path, PathBuf};

/// `site/scripts/<name>` must equal `templates/scripts/<name>`, byte for byte.
const PINNED: &[&str] = &["cf-setup.sh", "cf-init-app.sh"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("setup: read {}: {e}", path.display()))
}

/// The first line that differs, for a failure message that names the drift
/// instead of dumping two files.
fn first_difference(left: &str, right: &str) -> String {
    let mut l = left.lines();
    let mut r = right.lines();
    let mut line_no = 0usize;
    loop {
        line_no += 1;
        match (l.next(), r.next()) {
            (None, None) => return "no line differs (trailing bytes only)".to_string(),
            (Some(a), Some(b)) if a == b => continue,
            (a, b) => {
                return format!(
                    "line {line_no}:\n  templates: {}\n  site:      {}",
                    a.unwrap_or("<end of file>"),
                    b.unwrap_or("<end of file>")
                );
            }
        }
    }
}

#[test]
fn every_pinned_pair_exists_on_both_sides() {
    let root = repo_root();
    for name in PINNED {
        for tree in ["templates/scripts", "site/scripts"] {
            let path = root.join(tree).join(name);
            assert!(
                path.is_file(),
                "setup: {} is pinned but {} does not exist",
                name,
                path.display()
            );
        }
    }
    assert!(!PINNED.is_empty(), "setup: the pinned list is empty");
}

#[test]
fn site_scripts_match_the_templates_they_were_copied_from() {
    let root = repo_root();
    for name in PINNED {
        let template = root.join("templates/scripts").join(name);
        let site = root.join("site/scripts").join(name);
        let (want, got) = (read(&template), read(&site));
        if want == got {
            continue;
        }
        let detail = match (String::from_utf8(want), String::from_utf8(got)) {
            (Ok(w), Ok(g)) => first_difference(&w, &g),
            _ => "binary contents differ".to_string(),
        };
        panic!(
            "site/scripts/{name} has drifted from templates/scripts/{name}.\n\n{detail}\n\n\
             Re-copy the template over the site twin, or delete the twin and drop it from \
             PINNED in this test. See the module docblock."
        );
    }
}

/// The reason the pair above is pinned at all: the shipped Cloudflare scripts
/// resolve the canonical credential name, so a stale site copy writing only the
/// deprecated alias is exactly the drift this file exists to catch.
#[test]
fn the_pinned_cloudflare_scripts_carry_the_canonical_credential_name() {
    let root = repo_root();
    for name in ["cf-setup.sh", "cf-init-app.sh"] {
        for tree in ["templates/scripts", "site/scripts"] {
            let path = root.join(tree).join(name);
            let body = String::from_utf8(read(&path)).expect("setup: script is utf-8");
            assert!(
                body.contains("CLOUDFLARE_ACCOUNT_ID"),
                "{} never mentions CLOUDFLARE_ACCOUNT_ID, so it cannot be resolving \
                 the canonical credential name",
                path.display()
            );
        }
    }
}
