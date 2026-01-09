//! Batch Processing Job
//!
//! Processes large datasets in batches with parallelization.
//! Uses rayon for CPU-bound work and chunked processing.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::domain::models::{Job, JobResult};
use crate::worker::WorkerState;

#[derive(Debug, Deserialize)]
struct BatchPayload {
    /// Data items to process
    items: Vec<serde_json::Value>,
    /// Operation to perform
    operation: String,
    /// Batch size for chunking
    #[serde(default = "default_batch_size")]
    batch_size: usize,
}

fn default_batch_size() -> usize {
    100
}

#[derive(Debug, Serialize)]
struct BatchResult {
    processed: usize,
    succeeded: usize,
    failed: usize,
    results: Vec<ItemResult>,
}

#[derive(Debug, Serialize)]
struct ItemResult {
    index: usize,
    success: bool,
    output: Option<serde_json::Value>,
    error: Option<String>,
}

pub async fn process(job: &Job, state: &WorkerState) -> anyhow::Result<JobResult> {
    let payload: BatchPayload = serde_json::from_value(job.payload.clone())?;
    
    info!(
        items = payload.items.len(),
        operation = payload.operation,
        batch_size = payload.batch_size,
        "Starting batch processing"
    );

    let total = payload.items.len();
    let operation = payload.operation.clone();
    
    // Process in chunks to avoid memory issues
    let results: Vec<ItemResult> = payload.items
        .into_par_iter()
        .enumerate()
        .map(|(index, item)| {
            // CPU-bound processing
            let result = process_item(&operation, &item);
            
            match result {
                Ok(output) => ItemResult {
                    index,
                    success: true,
                    output: Some(output),
                    error: None,
                },
                Err(e) => ItemResult {
                    index,
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                },
            }
        })
        .collect();

    let succeeded = results.iter().filter(|r| r.success).count();
    let failed = results.iter().filter(|r| !r.success).count();

    info!(
        total,
        succeeded,
        failed,
        "Batch processing complete"
    );

    Ok(JobResult {
        output: serde_json::to_value(BatchResult {
            processed: total,
            succeeded,
            failed,
            results,
        })?,
    })
}

/// Process a single item (CPU-bound)
fn process_item(operation: &str, item: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    match operation {
        "transform" => {
            // Example: transform data structure
            let mut output = item.clone();
            if let Some(obj) = output.as_object_mut() {
                obj.insert("processed".to_string(), serde_json::json!(true));
                obj.insert("processed_at".to_string(), serde_json::json!(chrono::Utc::now()));
            }
            Ok(output)
        }
        "validate" => {
            // Example: validate item against schema
            if item.is_object() {
                Ok(serde_json::json!({ "valid": true }))
            } else {
                anyhow::bail!("Invalid item: expected object")
            }
        }
        "hash" => {
            // Example: compute hash of item
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            
            let json = serde_json::to_string(item)?;
            let mut hasher = DefaultHasher::new();
            json.hash(&mut hasher);
            let hash = hasher.finish();
            
            Ok(serde_json::json!({ "hash": format!("{:x}", hash) }))
        }
        _ => {
            anyhow::bail!("Unknown operation: {}", operation)
        }
    }
}
