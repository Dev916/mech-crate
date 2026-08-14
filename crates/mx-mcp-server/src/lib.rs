//! MechCrate MCP Server — library surface.
//!
//! The crate ships a single binary (`mx-mcp`), but the protocol, tool
//! registry, executors and project detection live here as a library so they
//! can be unit-tested directly (`cargo nextest run -p mx-mcp-server --lib`)
//! and reused by the stdio integration harness. `src/main.rs` is a thin
//! argument-parsing shell over [`mcp::server::McpServer`].

pub mod error;
pub mod mcp;
pub mod mx;
pub mod project;
pub mod tools;
pub mod unyform;
