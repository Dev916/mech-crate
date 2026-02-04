//! Error types for the MCP server

use thiserror::Error;

/// Main error type for the MCP server
#[derive(Error, Debug)]
pub enum McpError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    #[error("Not in a MechCrate project")]
    NotInProject,

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Weaviate error: {0}")]
    Weaviate(String),

    #[error("MechCrate root not found")]
    MechCrateRootNotFound,

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
}

/// Result type alias for MCP operations
pub type McpResult<T> = Result<T, McpError>;

/// JSON-RPC error codes
pub mod error_codes {
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
}
