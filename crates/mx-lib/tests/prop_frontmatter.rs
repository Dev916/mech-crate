//! Property tests for the YAML frontmatter parser.
//!
//! `appendix-rust.md` A6 — property tests for decoders. Two invariants:
//!
//! 1. **Round-trip**: rendering a `TechniqueMeta` to YAML between `---` fences
//!    and reparsing yields the same metadata (`parse(render(meta)) == meta`).
//! 2. **Never panics**: arbitrary junk — including fence-shaped junk that
//!    reaches serde_yaml — returns `(None, content)` instead of unwinding.
//!    Ingestion of a corpus must never abort on one malformed doc.
//!
//! Scalars are generated newline-free and ≤ 40 chars so the emitter always
//! writes one `key: value` line per field: a value can therefore never
//! introduce a line that starts with `---` and forge an early fence.

use mx_lib::corpus::frontmatter::{parse_frontmatter, TechniqueMeta};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

/// 256 cases; counterexamples persist into the committed seed corpus.
fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            "tests/proptest-regressions/prop_frontmatter.txt",
        ))),
        ..ProptestConfig::default()
    }
}

fn scalar() -> impl Strategy<Value = Option<String>> {
    prop::option::of("[A-Za-z0-9 ._-]{0,40}")
}

fn list() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec("[A-Za-z0-9._-]{1,20}", 0..4)
}

fn meta() -> impl Strategy<Value = TechniqueMeta> {
    (scalar(), scalar(), list(), scalar(), list(), scalar()).prop_map(
        |(title, category, languages, complexity, use_cases, summary)| TechniqueMeta {
            title,
            category,
            languages,
            complexity,
            use_cases,
            summary,
        },
    )
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn prop_frontmatter_round_trips(
        meta in meta(),
        body in "[A-Za-z0-9 #.\n]{0,120}",
    ) {
        let yaml = serde_yaml::to_string(&meta).expect("meta serializes");
        let doc = format!("---\n{yaml}---\n\n{body}");

        let (parsed, rest) = parse_frontmatter(&doc);
        let parsed = parsed.expect("rendered frontmatter must reparse");

        prop_assert_eq!(&parsed, &meta);
        prop_assert_eq!(rest, body.trim_start_matches('\n'));
    }

    /// Arbitrary input, no fence: parser returns the content untouched.
    #[test]
    fn prop_frontmatter_never_panics_on_junk(doc in any::<String>()) {
        let (meta, body) = parse_frontmatter(&doc);
        prop_assert!(body.len() <= doc.len());
        if meta.is_none() && !doc.starts_with("---\n") {
            prop_assert_eq!(body, doc.as_str());
        }
    }

    /// Fence-shaped junk drives arbitrary bytes into serde_yaml itself.
    #[test]
    fn prop_frontmatter_never_panics_on_fenced_junk(
        junk in any::<String>(),
        body in any::<String>(),
    ) {
        let doc = format!("---\n{junk}\n---\n{body}");
        let (meta, rest) = parse_frontmatter(&doc);
        // Either it parsed, or it degraded to (None, whole document).
        prop_assert!(meta.is_some() || rest == doc);
    }
}
