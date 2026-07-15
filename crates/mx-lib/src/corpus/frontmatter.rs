//! YAML frontmatter parser for technique docs.

use serde::Deserialize;

/// Metadata parsed from a technique doc's YAML frontmatter.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct TechniqueMeta {
    pub title: Option<String>,
    pub category: Option<String>,
    pub languages: Vec<String>,
    pub complexity: Option<String>,
    pub use_cases: Vec<String>,
    pub summary: Option<String>,
}

/// Split a markdown document into (frontmatter, body).
///
/// Frontmatter must start at byte 0 with `---\n` and end at the next line
/// equal to `---`. Malformed YAML yields `(None, full_content)` — callers
/// warn and fall back to heuristics; ingestion never aborts on one bad doc.
pub fn parse_frontmatter(content: &str) -> (Option<TechniqueMeta>, &str) {
    let Some(rest) = content.strip_prefix("---\n") else {
        return (None, content);
    };
    let Some(end) = rest.find("\n---") else {
        return (None, content);
    };
    let yaml = &rest[..end];
    let after = &rest[end + 4..];
    let body = after.trim_start_matches('\n');
    match serde_yaml::from_str::<TechniqueMeta>(yaml) {
        Ok(meta) => (Some(meta), body),
        Err(_) => (None, content),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_frontmatter() {
        let doc = "---\ntitle: Rust Concurrency\ncategory: concurrency\nlanguages: [rust]\ncomplexity: expert\nuse_cases:\n  - lock-free structures\nsummary: Deep dive.\n---\n\n# Body\n\ntext";
        let (meta, body) = parse_frontmatter(doc);
        let meta = meta.expect("meta");
        assert_eq!(meta.title.as_deref(), Some("Rust Concurrency"));
        assert_eq!(meta.category.as_deref(), Some("concurrency"));
        assert_eq!(meta.languages, vec!["rust"]);
        assert_eq!(meta.complexity.as_deref(), Some("expert"));
        assert_eq!(meta.use_cases, vec!["lock-free structures"]);
        assert!(body.starts_with("# Body"));
    }

    #[test]
    fn missing_frontmatter_returns_none_and_full_body() {
        let doc = "# Just a doc\n\ncontent";
        let (meta, body) = parse_frontmatter(doc);
        assert!(meta.is_none());
        assert_eq!(body, doc);
    }

    #[test]
    fn malformed_yaml_returns_none_and_full_body() {
        let doc = "---\ntitle: [unclosed\n---\nbody";
        let (meta, body) = parse_frontmatter(doc);
        assert!(meta.is_none());
        assert_eq!(body, doc);
    }
}
