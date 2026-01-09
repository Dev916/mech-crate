//! LLM Infrastructure
//!
//! Local LLM integration using llama-cpp-2.
//! Only compiled when the `llm` feature is enabled.

#[cfg(feature = "llm")]
mod llama;

#[cfg(feature = "llm")]
pub use llama::*;

/// Placeholder for when LLM feature is not enabled
#[cfg(not(feature = "llm"))]
pub struct MockLlm;

#[cfg(not(feature = "llm"))]
impl MockLlm {
    pub fn new() -> Self {
        Self
    }

    pub async fn generate(&self, _prompt: &str, _max_tokens: usize) -> anyhow::Result<String> {
        Ok("[LLM feature not enabled]".to_string())
    }
}
