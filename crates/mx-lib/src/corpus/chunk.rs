//! Heading-aware markdown chunker.
//!
//! Splits on `##` headings; sections over `max_chars` are sub-split on
//! blank-line paragraph boundaries (hq `chunk_text` port). Every chunk's
//! content is prefixed with its `Doc Title > Heading` path so chunks are
//! self-contextualizing when retrieved in isolation.

/// Default chunk size cap in characters.
pub const DEFAULT_CHUNK_CHARS: usize = 1200;

/// One retrievable chunk of a technique doc.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub heading_path: String,
    pub content: String,
}

/// Chunk a markdown body under `doc_title`.
pub fn chunk_markdown(doc_title: &str, body: &str, max_chars: usize) -> Vec<Chunk> {
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut current_heading = String::new();
    let mut current = String::new();

    for line in body.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            sections.push((current_heading.clone(), std::mem::take(&mut current)));
            current_heading = h.trim().to_string();
        } else if line.starts_with("# ") {
            // Top-level heading: part of the preamble text, not a section break.
            current.push_str(line);
            current.push('\n');
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }
    sections.push((current_heading, current));

    let mut chunks = Vec::new();
    for (heading, text) in sections {
        if text.trim().is_empty() {
            continue;
        }
        let heading_path = if heading.is_empty() {
            doc_title.to_string()
        } else {
            format!("{} > {}", doc_title, heading)
        };
        for piece in pack_paragraphs(&text, max_chars) {
            chunks.push(Chunk {
                heading_path: heading_path.clone(),
                content: format!("{}\n\n{}", heading_path, piece),
            });
        }
    }
    chunks
}

/// hq `chunk_text` port: pack paragraphs into pieces up to `max_chars`;
/// hard-split single oversized paragraphs.
fn pack_paragraphs(text: &str, max_chars: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for para in text.split("\n\n") {
        let para = para.trim();
        if para.is_empty() {
            continue;
        }
        if para.len() > max_chars {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
            out.extend(hard_split(para, max_chars));
            continue;
        }
        if current.len() + para.len() + 2 > max_chars && !current.is_empty() {
            out.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(para);
    }
    if !current.trim().is_empty() {
        out.push(current);
    }
    out
}

fn hard_split(s: &str, max: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    for ch in s.chars() {
        if buf.len() + ch.len_utf8() > max {
            out.push(std::mem::take(&mut buf));
        }
        buf.push(ch);
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_h2_headings() {
        let body = "intro para\n\n## First\n\nalpha\n\n## Second\n\nbeta";
        let chunks = chunk_markdown("Doc", body, 1200);
        let paths: Vec<_> = chunks.iter().map(|c| c.heading_path.as_str()).collect();
        assert_eq!(paths, vec!["Doc", "Doc > First", "Doc > Second"]);
    }

    #[test]
    fn chunk_content_is_prefixed_with_heading_path() {
        let body = "## First\n\nalpha";
        let chunks = chunk_markdown("Doc", body, 1200);
        assert!(chunks[0].content.starts_with("Doc > First\n\n"));
        assert!(chunks[0].content.contains("alpha"));
    }

    #[test]
    fn oversized_section_sub_splits_under_cap() {
        let para = "x".repeat(500);
        let body = format!("## Big\n\n{para}\n\n{para}\n\n{para}");
        let chunks = chunk_markdown("Doc", &body, 600);
        assert!(chunks.len() >= 3);
        assert!(chunks
            .iter()
            .all(|c| c.content.len() <= 600 + "Doc > Big\n\n".len()));
        assert!(chunks.iter().all(|c| c.heading_path == "Doc > Big"));
    }

    #[test]
    fn empty_body_yields_no_chunks() {
        assert!(chunk_markdown("Doc", "   \n  ", 1200).is_empty());
    }
}
