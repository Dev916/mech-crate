//! Property tests for the heading-aware markdown chunker.
//!
//! `appendix-rust.md` A6 — "property tests for invariants and decoders/encoders".
//! Two invariants the chunker must never violate, whatever the document:
//!
//! 1. **Budget**: a chunk's content is at most `max_chars` plus the
//!    `heading_path\n\n` prefix the chunker prepends for self-context.
//! 2. **Preservation**: no body paragraph is dropped — every input paragraph
//!    survives verbatim inside some chunk.
//!
//! Generators stay inside the realistic document alphabet (no `#` inside
//! paragraph text, so a paragraph can never masquerade as a heading).

use mx_lib::corpus::chunk::{chunk_markdown, Chunk, DEFAULT_CHUNK_CHARS};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

/// 256 cases, with shrunk counterexamples persisted into the committed
/// `tests/proptest-regressions/` seed corpus (paths are relative to the
/// package root, which is cargo's cwd for test binaries).
fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            "tests/proptest-regressions/prop_chunker.txt",
        ))),
        ..ProptestConfig::default()
    }
}

/// A markdown block: an `##` section heading or a body paragraph.
#[derive(Debug, Clone)]
enum Block {
    Heading(String),
    Para(String),
}

impl Block {
    fn render(&self) -> String {
        match self {
            Block::Heading(h) => format!("## {h}"),
            Block::Para(p) => p.clone(),
        }
    }
}

/// Headings always start with a letter, so `trim()` inside the chunker can
/// never empty them out.
fn heading() -> impl Strategy<Value = Block> {
    "[A-Za-z][A-Za-z ]{0,29}".prop_map(|h| Block::Heading(h.trim_end().to_string()))
}

/// Paragraphs always start with an alphanumeric, so `trim()` leaves a
/// non-empty needle to search for.
fn para(pattern: &'static str) -> impl Strategy<Value = Block> {
    pattern.prop_map(Block::Para)
}

/// A document as an alternating-ish stream of headings and paragraphs.
fn blocks(para_pattern: &'static str, max_blocks: usize) -> impl Strategy<Value = Vec<Block>> {
    prop::collection::vec(
        prop_oneof![1 => heading(), 3 => para(para_pattern)],
        0..max_blocks,
    )
}

fn render(blocks: &[Block]) -> String {
    blocks
        .iter()
        .map(Block::render)
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn concat(chunks: &[Chunk]) -> String {
    chunks
        .iter()
        .map(|c| c.content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

proptest! {
    #![proptest_config(config())]

    /// Paragraphs up to 600 chars against budgets down to 40 exercise both
    /// packing and the oversized-paragraph hard split.
    #[test]
    fn prop_chunk_respects_budget(
        title in "[A-Za-z][A-Za-z ]{0,39}",
        doc in blocks("[A-Za-z0-9][A-Za-z0-9 ,.]{0,600}", 12),
        max in 40usize..=DEFAULT_CHUNK_CHARS,
    ) {
        let body = render(&doc);
        for c in chunk_markdown(&title, &body, max) {
            let prefix_len = c.heading_path.len() + 2;
            prop_assert!(
                c.content.len() <= max + prefix_len,
                "chunk of {} bytes exceeds budget {} (+prefix {}) under path {:?}",
                c.content.len(), max, prefix_len, c.heading_path,
            );
        }
    }

    /// Every paragraph shorter than the budget stays intact in some chunk.
    #[test]
    fn prop_chunk_preserves_paragraph_content(
        title in "[A-Za-z][A-Za-z ]{0,39}",
        doc in blocks("[A-Za-z0-9][A-Za-z0-9 ,.]{0,199}", 16),
    ) {
        let body = render(&doc);
        let chunks = chunk_markdown(&title, &body, DEFAULT_CHUNK_CHARS);
        let joined = concat(&chunks);
        for block in &doc {
            if let Block::Para(p) = block {
                let needle = p.trim();
                prop_assert!(
                    joined.contains(needle),
                    "paragraph {:?} lost from {} chunk(s)",
                    needle, chunks.len(),
                );
            }
        }
    }

    /// A document with no `##` sections still round-trips its text under the
    /// bare doc title.
    #[test]
    fn prop_chunk_preamble_only_uses_doc_title(
        title in "[A-Za-z][A-Za-z ]{0,39}",
        text in "[A-Za-z0-9][A-Za-z0-9 ,.]{0,199}",
    ) {
        let chunks = chunk_markdown(&title, &text, DEFAULT_CHUNK_CHARS);
        prop_assert_eq!(chunks.len(), 1);
        prop_assert_eq!(&chunks[0].heading_path, &title);
        prop_assert!(chunks[0].content.contains(text.trim()));
    }
}
