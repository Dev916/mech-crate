//! MCP Transport Layer
//!
//! Handles stdio-based communication for the MCP protocol.

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use tracing::{debug, error, trace};

use crate::error::{McpError, McpResult};
use crate::mcp::protocol::{JsonRpcRequest, JsonRpcResponse};

/// Stdio transport for MCP communication
pub struct StdioTransport {
    tx: mpsc::Sender<JsonRpcResponse>,
}

impl StdioTransport {
    /// Create a new stdio transport and start the I/O loops
    pub fn new() -> (Self, mpsc::Receiver<JsonRpcRequest>) {
        let (request_tx, request_rx) = mpsc::channel::<JsonRpcRequest>(100);
        let (response_tx, mut response_rx) = mpsc::channel::<JsonRpcResponse>(100);

        // Spawn stdin reader task
        tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let mut reader = BufReader::new(stdin);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        debug!("EOF on stdin, shutting down");
                        break;
                    }
                    Ok(_) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }

                        trace!("Received: {}", trimmed);

                        match serde_json::from_str::<JsonRpcRequest>(trimmed) {
                            Ok(request) => {
                                if request_tx.send(request).await.is_err() {
                                    error!("Request channel closed");
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse request: {} - input: {}", e, trimmed);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading stdin: {}", e);
                        break;
                    }
                }
            }
        });

        // Spawn stdout writer task
        tokio::spawn(async move {
            let mut stdout = tokio::io::stdout();

            while let Some(response) = response_rx.recv().await {
                match serde_json::to_string(&response) {
                    Ok(json) => {
                        trace!("Sending: {}", json);
                        if let Err(e) = stdout.write_all(json.as_bytes()).await {
                            error!("Failed to write to stdout: {}", e);
                            break;
                        }
                        if let Err(e) = stdout.write_all(b"\n").await {
                            error!("Failed to write newline: {}", e);
                            break;
                        }
                        if let Err(e) = stdout.flush().await {
                            error!("Failed to flush stdout: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to serialize response: {}", e);
                    }
                }
            }
        });

        (Self { tx: response_tx }, request_rx)
    }

    /// Send a response
    pub async fn send(&self, response: JsonRpcResponse) -> McpResult<()> {
        self.tx
            .send(response)
            .await
            .map_err(|_| McpError::Protocol("Response channel closed".to_string()))
    }
}
