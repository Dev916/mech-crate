# MX Rust CLI & MCP Server Quick Reference

## Build Commands

```bash
# Build all (debug)
cargo build

# Build all (release)
cargo build --release

# Build specific crate
cargo build -p mx-cli --release
cargo build -p mx-mcp-server --release

# Watch mode
cargo watch -x 'build -p mx-cli'
```

## Binary Locations

```
target/debug/mx           # Debug CLI
target/debug/mx-mcp       # Debug MCP server
target/release/mx         # Release CLI
target/release/mx-mcp     # Release MCP server
```

## Running

```bash
# CLI
./target/release/mx --help
./target/release/mx doctor

# MCP Server
./target/release/mx-mcp
RUST_LOG=debug ./target/release/mx-mcp
./target/release/mx-mcp --mech-crate-root /path --weaviate-url http://localhost:8080
```

## MCP Server Management

```bash
mx mcp build          # Build MCP server
mx mcp start          # Start Weaviate
mx mcp stop           # Stop Weaviate
mx mcp status         # Check status
mx mcp logs -f        # Follow logs
mx mcp ingest --clear # Re-ingest docs
mx mcp config         # Show client config
```

## Testing

```bash
cargo test                        # All tests
cargo test -p mx-lib              # Specific crate
cargo test test_name              # Specific test
cargo test -- --nocapture         # With output
RUST_LOG=debug cargo test         # With logging
```

## Linting

```bash
cargo fmt                         # Format code
cargo clippy -- -D warnings       # Lint check
cargo fmt && cargo clippy && cargo test  # Full check
```

## Workspace Structure

```
crates/
├── mx-lib/           # Shared library (core logic)
├── mx-cli/           # CLI binary (mx)
└── mx-mcp-server/    # MCP server (mx-mcp)
```

## Adding a CLI Command

1. Create `crates/mx-cli/src/commands/mycommand.rs`
2. Add `pub mod mycommand;` to `commands/mod.rs`
3. Add variant to `Commands` enum in `main.rs`
4. Add handler in `main()` match

## Adding an MCP Tool

1. Add `MyTool` variant to `ToolHandler` enum
2. Add `ToolDefinition` in `define_all_tools()`
3. Add handler case in `execute()` match

## Environment Variables

| Variable | Description |
|----------|-------------|
| `MECH_CRATE_ROOT` | MechCrate installation path |
| `WEAVIATE_URL` | Weaviate endpoint |
| `RUST_LOG` | Log level (debug, info, warn, error) |

## MCP Client Config (Claude Desktop)

```json
{
  "mcpServers": {
    "mechcrate": {
      "command": "/path/to/mx-mcp",
      "env": {
        "MECH_CRATE_ROOT": "/path/to/mech-crate"
      }
    }
  }
}
```

## Key Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace manifest |
| `crates/mx-cli/src/main.rs` | CLI entry point |
| `crates/mx-mcp-server/src/main.rs` | MCP entry point |
| `crates/mx-mcp-server/src/tools/mod.rs` | Tool definitions (44 tools) |
| `crates/mx-lib/src/lib.rs` | Library exports |

## MCP Tool Categories

| Category | Count | Prefix |
|----------|-------|--------|
| Global MX | 13 | `mx_` |
| Project | 3 | `mx_` |
| Make | 9 | `make_` |
| Analysis | 4 | `project_`, `service_` |
| RAG | 7 | `rag_` |
| Unyform | 8 | `unyform_` |

## Debugging

```bash
# Verbose CLI
mx -v doctor

# Debug logging
RUST_LOG=debug mx new my-project
RUST_LOG=mx_lib::recipe=trace mx add api

# MCP debug
RUST_LOG=debug ./target/release/mx-mcp
```

## Common Issues

| Issue | Solution |
|-------|----------|
| Build errors | `cargo clean && cargo build` |
| MCP root not found | Set `MECH_CRATE_ROOT` |
| Weaviate unavailable | `mx mcp stop && mx mcp start` |
| RAG no results | `mx mcp ingest --clear` |
