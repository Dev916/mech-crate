//! Property tests for the placeholder / template transforms.
//!
//! `appendix-rust.md` A6 — invariants over encoders. The transforms exist in
//! two places: recipe placeholder transforms (`Recipe::build_placeholders`,
//! exercised here through the public API rather than by widening visibility)
//! and the Tera filters (`TemplateEngine::render_string`). Every property is
//! asserted against both surfaces, so the duplicated implementations cannot
//! drift apart silently.
//!
//! Inputs are printable ASCII — the realistic domain for a service name typed
//! at the CLI. The transforms keep any Unicode alphanumeric they are handed,
//! so the ASCII-only charset guarantees below are deliberately scoped to that
//! domain rather than to `any::<String>()`.

use std::collections::HashMap;

use mx_lib::recipe::Recipe;
use mx_lib::template::TemplateEngine;
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

/// 256 cases; counterexamples persist into the committed seed corpus.
fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            "tests/proptest-regressions/prop_transforms.txt",
        ))),
        ..ProptestConfig::default()
    }
}

const RECIPE_JSON: &str = r#"{
  "name": "prop-transforms",
  "placeholders": {
    "SLUG":  { "source": "name", "transform": "slug" },
    "UPPER": { "source": "name", "transform": "upper" },
    "CRATE": { "source": "name", "transform": "rust_crate" }
  }
}"#;

/// Run a service name through the recipe placeholder transforms.
fn transform(service_name: &str, key: &str) -> String {
    let recipe = Recipe::parse(RECIPE_JSON).expect("fixture recipe parses");
    recipe
        .build_placeholders(service_name, &HashMap::new())
        .get(key)
        .cloned()
        .unwrap_or_else(|| panic!("placeholder {key} missing"))
}

/// Run a value through the equivalent Tera filter.
fn filter(name: &str, value: &str) -> String {
    let mut engine = TemplateEngine::new().expect("engine");
    let mut vars = HashMap::new();
    vars.insert("name".to_string(), value.to_string());
    engine
        .render_string(&format!("{{{{ name | {name} }}}}"), &vars)
        .expect("filter renders")
}

proptest! {
    #![proptest_config(config())]

    /// slug: output charset is `[a-z0-9-]`, and slugging a slug is a no-op.
    #[test]
    fn prop_slug_is_idempotent_with_kebab_charset(name in "[ -~]{0,48}") {
        let slug = transform(&name, "SLUG");

        prop_assert!(
            slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "slug {:?} of {:?} escapes [a-z0-9-]", slug, name,
        );
        prop_assert_eq!(transform(&slug, "SLUG"), slug.clone(), "slug is not idempotent");
        prop_assert_eq!(filter("slug", &name), slug, "tera slug filter disagrees");
    }

    /// upper_snake: output charset is `[A-Z0-9_]`.
    #[test]
    fn prop_upper_snake_charset(name in "[ -~]{0,48}") {
        let upper = transform(&name, "UPPER");

        prop_assert!(
            upper.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'),
            "upper {:?} of {:?} escapes [A-Z0-9_]", upper, name,
        );
        prop_assert_eq!(transform(&upper, "UPPER"), upper.clone(), "upper is not idempotent");
        prop_assert_eq!(filter("upper_snake", &name), upper, "tera upper_snake filter disagrees");
    }

    /// rust_crate: a name that starts with a letter yields a lexically valid
    /// Rust identifier (`[a-z][a-z0-9_]*`). The transform does not guard
    /// leading digits, hence the leading-letter domain.
    #[test]
    fn prop_rust_crate_is_valid_identifier(name in "[A-Za-z][ -~]{0,40}") {
        let krate = transform(&name, "CRATE");

        let mut chars = krate.chars();
        let first = chars.next().expect("non-empty crate name");
        prop_assert!(
            first.is_ascii_lowercase(),
            "crate name {:?} of {:?} does not start with [a-z]", krate, name,
        );
        prop_assert!(
            krate.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "crate name {:?} of {:?} escapes [a-z0-9_]", krate, name,
        );
        prop_assert_eq!(transform(&krate, "CRATE"), krate.clone(), "rust_crate is not idempotent");
        prop_assert_eq!(filter("rust_crate", &name), krate, "tera rust_crate filter disagrees");
    }
}
