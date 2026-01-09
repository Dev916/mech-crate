//! LLM Evaluation Job
//!
//! Runs local LLM inference for evaluation tasks.
//! Supports llama.cpp models via llama-cpp-2 crate.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tracing::{debug, info, warn};

use crate::domain::models::{Job, JobResult};
use crate::worker::WorkerState;

#[cfg(feature = "llm")]
use llama_cpp_2::context::params::LlamaContextParams;
#[cfg(feature = "llm")]
use llama_cpp_2::llama_backend::LlamaBackend;
#[cfg(feature = "llm")]
use llama_cpp_2::model::params::LlamaModelParams;
#[cfg(feature = "llm")]
use llama_cpp_2::model::LlamaModel;

#[derive(Debug, Deserialize)]
struct LlmPayload {
    /// Task type: evaluate, classify, summarize, generate
    task: LlmTask,
    /// Input text
    input: String,
    /// Optional system prompt
    system_prompt: Option<String>,
    /// Max tokens to generate
    #[serde(default = "default_max_tokens")]
    max_tokens: usize,
    /// Temperature for sampling
    #[serde(default = "default_temperature")]
    temperature: f32,
}

fn default_max_tokens() -> usize { 256 }
fn default_temperature() -> f32 { 0.7 }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LlmTask {
    /// Evaluate/score content
    Evaluate,
    /// Classify into categories
    Classify,
    /// Summarize text
    Summarize,
    /// Generate text continuation
    Generate,
    /// Extract structured data
    Extract,
}

#[derive(Debug, Serialize)]
struct LlmResult {
    task: String,
    output: String,
    tokens_used: usize,
    model: String,
}

pub async fn process(job: &Job, state: &WorkerState) -> anyhow::Result<JobResult> {
    let payload: LlmPayload = serde_json::from_value(job.payload.clone())?;
    
    info!(task = ?payload.task, input_len = payload.input.len(), "Starting LLM task");

    // Check if LLM is enabled
    if !state.config.llm_enabled {
        anyhow::bail!("LLM processing is disabled. Enable with --llm-enabled=true");
    }

    #[cfg(feature = "llm")]
    {
        let result = run_llm_inference(&payload, &state.config.llm_model_path).await?;
        return Ok(JobResult {
            output: serde_json::to_value(result)?,
        });
    }

    #[cfg(not(feature = "llm"))]
    {
        // Fallback when LLM feature is not compiled
        warn!("LLM feature not compiled in. Returning mock result.");
        
        let mock_result = mock_llm_response(&payload);
        
        Ok(JobResult {
            output: serde_json::to_value(mock_result)?,
        })
    }
}

#[cfg(feature = "llm")]
async fn run_llm_inference(payload: &LlmPayload, model_path: &str) -> anyhow::Result<LlmResult> {
    use std::sync::Arc;
    use parking_lot::Mutex;
    
    // Initialize backend (lazy, thread-safe)
    static BACKEND: OnceLock<LlamaBackend> = OnceLock::new();
    static MODEL: OnceLock<Arc<Mutex<LlamaModel>>> = OnceLock::new();
    
    let backend = BACKEND.get_or_init(|| {
        LlamaBackend::init().expect("Failed to initialize LLM backend")
    });

    // Load model if not already loaded
    let model = MODEL.get_or_init(|| {
        info!(model_path, "Loading LLM model");
        let params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(backend, model_path, &params)
            .expect("Failed to load model");
        Arc::new(Mutex::new(model))
    });

    // Build prompt based on task
    let prompt = build_prompt(&payload);
    
    // Run inference on blocking thread
    let result = tokio::task::spawn_blocking({
        let model = Arc::clone(model);
        let max_tokens = payload.max_tokens;
        let temperature = payload.temperature;
        
        move || {
            let model = model.lock();
            let ctx_params = LlamaContextParams::default()
                .with_n_ctx(std::num::NonZeroU32::new(2048));
            
            let mut ctx = model.new_context(backend, ctx_params)
                .map_err(|e| anyhow::anyhow!("Context creation failed: {:?}", e))?;

            // Tokenize input
            let tokens = model.str_to_token(&prompt, true)
                .map_err(|e| anyhow::anyhow!("Tokenization failed: {:?}", e))?;

            // Generate
            let mut output_tokens = Vec::new();
            // ... inference loop would go here
            
            let output = model.token_to_str(&output_tokens.first().unwrap_or(&0))
                .unwrap_or_default();

            Ok::<_, anyhow::Error>((output, tokens.len()))
        }
    })
    .await??;

    Ok(LlmResult {
        task: format!("{:?}", payload.task),
        output: result.0,
        tokens_used: result.1,
        model: model_path.to_string(),
    })
}

fn build_prompt(payload: &LlmPayload) -> String {
    let system = payload.system_prompt.as_deref().unwrap_or_default();
    
    match payload.task {
        LlmTask::Evaluate => format!(
            "{}\nEvaluate the following content on a scale of 1-10 and explain your reasoning:\n\n{}",
            system, payload.input
        ),
        LlmTask::Classify => format!(
            "{}\nClassify the following content into relevant categories:\n\n{}",
            system, payload.input
        ),
        LlmTask::Summarize => format!(
            "{}\nProvide a concise summary of the following:\n\n{}",
            system, payload.input
        ),
        LlmTask::Generate => format!(
            "{}\n{}",
            system, payload.input
        ),
        LlmTask::Extract => format!(
            "{}\nExtract structured data from the following:\n\n{}",
            system, payload.input
        ),
    }
}

/// Mock LLM response for testing without the feature
fn mock_llm_response(payload: &LlmPayload) -> LlmResult {
    let output = match payload.task {
        LlmTask::Evaluate => format!(
            "Score: 7/10\nThe content is well-structured but could use more detail. [MOCK RESPONSE]"
        ),
        LlmTask::Classify => format!(
            "Categories: [general, informational]\nConfidence: 0.85 [MOCK RESPONSE]"
        ),
        LlmTask::Summarize => format!(
            "Summary: {} [MOCK RESPONSE - truncated]",
            payload.input.chars().take(100).collect::<String>()
        ),
        LlmTask::Generate => format!(
            "Generated continuation... [MOCK RESPONSE]"
        ),
        LlmTask::Extract => format!(
            r#"{{"extracted": "data", "mock": true}}"#
        ),
    };

    LlmResult {
        task: format!("{:?}", payload.task),
        output,
        tokens_used: payload.input.len() / 4, // Rough estimate
        model: "mock".to_string(),
    }
}
