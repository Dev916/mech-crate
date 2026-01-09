//! Webhook Job
//!
//! Sends HTTP webhooks for job notifications.
//! Supports retries with exponential backoff.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::domain::models::{Job, JobResult};
use crate::worker::WorkerState;

#[derive(Debug, Deserialize)]
struct WebhookPayload {
    /// Target URL
    url: String,
    /// HTTP method (POST, PUT, PATCH)
    #[serde(default = "default_method")]
    method: String,
    /// Request body
    body: serde_json::Value,
    /// Custom headers
    #[serde(default)]
    headers: std::collections::HashMap<String, String>,
    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    timeout: u64,
    /// Retry count
    #[serde(default = "default_retries")]
    retries: u32,
}

fn default_method() -> String { "POST".to_string() }
fn default_timeout() -> u64 { 30 }
fn default_retries() -> u32 { 3 }

#[derive(Debug, Serialize)]
struct WebhookResult {
    url: String,
    status_code: u16,
    response_body: Option<String>,
    attempts: u32,
    duration_ms: u64,
}

pub async fn process(job: &Job, state: &WorkerState) -> anyhow::Result<JobResult> {
    let payload: WebhookPayload = serde_json::from_value(job.payload.clone())?;
    
    info!(url = payload.url, method = payload.method, "Sending webhook");

    let client = Client::builder()
        .timeout(Duration::from_secs(payload.timeout))
        .build()?;

    let mut last_error = None;
    let mut attempts = 0;
    let start = std::time::Instant::now();

    for attempt in 0..=payload.retries {
        attempts = attempt + 1;
        
        let mut request = match payload.method.to_uppercase().as_str() {
            "POST" => client.post(&payload.url),
            "PUT" => client.put(&payload.url),
            "PATCH" => client.patch(&payload.url),
            _ => anyhow::bail!("Unsupported HTTP method: {}", payload.method),
        };

        // Add headers
        for (key, value) in &payload.headers {
            request = request.header(key, value);
        }

        // Add body
        request = request.json(&payload.body);

        match request.send().await {
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.ok();

                if status.is_success() {
                    info!(
                        status = status.as_u16(),
                        attempts,
                        "Webhook succeeded"
                    );

                    return Ok(JobResult {
                        output: serde_json::to_value(WebhookResult {
                            url: payload.url,
                            status_code: status.as_u16(),
                            response_body: body,
                            attempts,
                            duration_ms: start.elapsed().as_millis() as u64,
                        })?,
                    });
                } else {
                    warn!(
                        status = status.as_u16(),
                        attempt,
                        "Webhook returned non-success status"
                    );
                    last_error = Some(format!("HTTP {}: {:?}", status.as_u16(), body));
                }
            }
            Err(e) => {
                warn!(
                    error = %e,
                    attempt,
                    "Webhook request failed"
                );
                last_error = Some(e.to_string());
            }
        }

        // Exponential backoff before retry
        if attempt < payload.retries {
            let delay = Duration::from_millis(100 * 2u64.pow(attempt));
            debug!(delay_ms = delay.as_millis(), "Retrying webhook");
            tokio::time::sleep(delay).await;
        }
    }

    anyhow::bail!(
        "Webhook failed after {} attempts: {}",
        attempts,
        last_error.unwrap_or_default()
    )
}
