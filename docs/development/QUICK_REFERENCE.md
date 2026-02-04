# MechCrate CLI Quick Reference

## Build Commands

```bash
# Build all (debug)
cargo build

# Build all (release)
cargo build --release

# Build specific crate
cargo build -p mx-cli
cargo build -p mx-mcp-server
cargo build -p mx-lib

# Watch and rebuild
cargo watch -x 'build -p mx-cli'
```

## Test Commands

```bash
# All tests
cargo test

# Specific crate
cargo test -p mx-lib

# With output
cargo test -- --nocapture
```

## Quality Commands

```bash
# Format
cargo fmt

# Lint
cargo clippy

# All checks
cargo fmt && cargo clippy && cargo test
```

## Binaries

| Binary | Location | Purpose |
|--------|----------|---------|
| `mx` | `target/release/mx` | Main CLI |
| `mx-mcp` | `target/release/mx-mcp` | MCP server for AI agents |

## Crate Structure

```
crates/
├── mx-lib/          # Shared library (use from CLI and MCP)
├── mx-cli/          # CLI commands
└── mx-mcp-server/   # MCP JSON-RPC server
```

## Adding a Command (Checklist)

1. [ ] Create `crates/mx-cli/src/commands/mycommand.rs`
2. [ ] Add `pub mod mycommand;` to `commands/mod.rs`
3. [ ] Add import in `main.rs`
4. [ ] Add variant to `Commands` enum
5. [ ] Add handler in `match cli.command`
6. [ ] Build and test: `cargo build -p mx-cli && ./target/debug/mx mycommand --help`

## Adding an MCP Tool (Checklist)

1. [ ] Add logic to `mx-lib` if needed
2. [ ] Add `ToolHandler::MyTool` variant in `tools/mod.rs`
3. [ ] Add `ToolDefinition` in `register_tools()`
4. [ ] Add execute handler in `execute_tool()`
5. [ ] Build and test: `cargo build -p mx-mcp-server`

## Key Imports

```rust
// CLI commands
use anyhow::Result;
use clap::Args;
use console::style;
use dialoguer::{Select, Input, Confirm};

// Shared library
use mx_lib::{templates_dir, is_initialized, home_dir};
use mx_lib::recipe::RecipeInstaller;
use mx_lib::project::ProjectDetector;
```

## Debug Logging

```bash
RUST_LOG=debug mx doctor
RUST_LOG=mx_lib=trace mx new my-project
```

## Path Resolution Priority

1. `MECH_CRATE_ROOT` env var
2. `~/.mech-crate/templates`
3. Relative to executable

## Installation Workflow

```bash
# 1. Build
cargo build --release

# 2. Initialize (copies templates to ~/.mech-crate)
# MECH_CRATE_ROOT only needed for first init from dev environment
MECH_CRATE_ROOT=$(pwd) ./target/release/mx init

# 3. Use anywhere - no env vars needed!
./target/release/mx recipes list
./target/release/mx doctor
./target/release/mx new my-project
```

## Common Commands

```bash
mx init                  # Install MechCrate templates
mx init --update         # Update templates (keep config)
mx doctor                # Check system health
mx recipes list          # List available recipes
mx new myproject         # Create new project
mx add api --recipe rust-api  # Add service with recipe
mx router install        # Install global Traefik
mx router up             # Start router
mx mcp build             # Build MCP server
mx mcp start             # Start Weaviate RAG backend
mx upgrade --dry-run     # Preview project upgrade
```
