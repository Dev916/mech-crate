//! MCP Transport Layer
//!
//! Handles stdio-based communication for the MCP protocol.

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use tracing::{debug, error, trace};

use crate::error::{McpError, McpResult};
use crate::mcp::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

/// Outgoing message (response or notification)
#[derive(Debug)]
#[allow(dead_code)]
pub enum OutgoingMessage {
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

/// Stdio transport for MCP communication
pub struct StdioTransport {
    tx: mpsc::Sender<OutgoingMessage>,
}

impl StdioTransport {
    /// Create a new stdio transport and start the I/O loops
    pub fn new() -> (Self, mpsc::Receiver<JsonRpcRequest>) {
        let (request_tx, request_rx) = mpsc::channel::<JsonRpcRequest>(100);
        let (message_tx, mut message_rx) = mpsc::channel::<OutgoingMessage>(100);

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

                        // Try parsing as request first, then notification
                        match serde_json::from_str::<JsonRpcRequest>(trimmed) {
                            Ok(request) => {
                                if request_tx.send(request).await.is_err() {
                                    error!("Request channel closed");
                                    break;
                                }
                            }
                            Err(e) => {
                                // Could be a notification (no id field) - log and ignore
                                if trimmed.contains("\"id\"") {
                                    error!("Failed to parse request: {} - input: {}", e, trimmed);
                                } else {
                                    debug!("Received notification (ignored): {}", trimmed);
                                }
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

            while let Some(message) = message_rx.recv().await {
                let json_result = match &message {
                    OutgoingMessage::Response(r) => serde_json::to_string(r),
                    OutgoingMessage::Notification(n) => serde_json::to_string(n),
                };

                match json_result {
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
                        error!("Failed to serialize message: {}", e);
                    }
                }
            }
        });

        (Self { tx: message_tx }, request_rx)
    }

    /// Send a response
    pub async fn send(&self, response: JsonRpcResponse) -> McpResult<()> {
        self.tx
            .send(OutgoingMessage::Response(response))
            .await
            .map_err(|_| McpError::Protocol("Message channel closed".to_string()))
    }

    /// Send a notification (no response expected)
    #[allow(dead_code)]
    pub async fn notify(&self, notification: JsonRpcNotification) -> McpResult<()> {
        self.tx
            .send(OutgoingMessage::Notification(notification))
            .await
            .map_err(|_| McpError::Protocol("Message channel closed".to_string()))
    }

    /// Send a log notification
    #[allow(dead_code)]
    pub async fn log(&self, level: &str, message: impl Into<String>) -> McpResult<()> {
        self.notify(JsonRpcNotification::log(level, message, Some("mx-mcp")))
            .await
    }
}
