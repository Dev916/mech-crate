# MX Rust CLI and MCP Server Development Guide

This document provides a comprehensive guide for building, managing, and extending the MechCrate Rust CLI (`mx`) and MCP server (`mx-mcp`).

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Project Structure](#project-structure)
3. [Building](#building)
4. [Running](#running)
5. [CLI Commands](#cli-commands)
6. [MCP Server](#mcp-server)
7. [Development Workflow](#development-workflow)
8. [Adding Features](#adding-features)
9. [Testing](#testing)
10. [Deployment](#deployment)
11. [Troubleshooting](#troubleshooting)

---

## Architecture Overview

MechCrate uses a Cargo workspace with three crates that share common dependencies:

```
┌─────────────────────────────────────────────────────────────────┐
│                         mx-cli                                   │
│                    (User-facing CLI)                            │
│                    Binary: `mx`                                 │
└─────────────────────────┬───────────────────────────────────────┘
                          │ depends on
┌─────────────────────────▼───────────────────────────────────────┐
│                         mx-lib                                   │
│                    (Shared Library)                             │
│         Project detection, recipes, templates, infra            │
└─────────────────────────▲───────────────────────────────────────┘
                          │ depends on
┌─────────────────────────┴───────────────────────────────────────┐
│                     mx-mcp-server                               │
│                  (MCP Server for AI Agents)                     │
│                  Binary: `mx-mcp`                               │
│  ┌─────────────┐  ┌───────────────┐  ┌───────────────────────┐ │
│  │   Tools     │  │   RAG/Weaviate │  │   JSON-RPC Protocol   │ │
│  │  (44 tools) │  │   Integration  │  │   (stdio transport)   │ │
│  └─────────────┘  └───────────────┘  └───────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Crate Responsibilities

| Crate | Purpose | Output Binary |
|-------|---------|---------------|
| `mx-lib` | Core business logic shared across CLI and MCP | Library (no binary) |
| `mx-cli` | User-facing CLI commands with interactive prompts | `mx` |
| `mx-mcp-server` | JSON-RPC server for AI agents (Claude, Cursor, etc.) | `mx-mcp` |

---

## Project Structure

```
mech-crate/
├── Cargo.toml                     # Workspace manifest
├── Cargo.lock                     # Dependency lock file
├── crates/
│   ├── mx-lib/                    # Shared library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs             # Module exports
│   │       ├── error.rs           # Error types (thiserror)
│   │       ├── paths.rs           # Path resolution (~/.mech-crate)
│   │       ├── config.rs          # Global configuration
│   │       ├── project.rs         # Project detection
│   │       ├── recipe/            # Recipe management
│   │       │   ├── mod.rs
│   │       │   ├── parser.rs      # recipe.json parsing
│   │       │   └── installer.rs   # Recipe installation
│   │       ├── template/          # Tera template engine
│   │       │   ├── mod.rs
│   │       │   └── engine.rs
│   │       ├── docker/            # Docker/Compose wrappers
│   │       │   └── mod.rs
│   │       ├── infra/             # Infrastructure providers
│   │       │   ├── mod.rs
│   │       │   └── config.rs
│   │       ├── router/            # Traefik router
│   │       │   └── mod.rs
│   │       └── unyform/           # Unyform API client
│   │           └── mod.rs
│   │
│   ├── mx-cli/                    # CLI binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs            # Entry point, clap setup
│   │       └── commands/
│   │           ├── mod.rs         # Command module exports
│   │           ├── init.rs        # mx init
│   │           ├── new.rs         # mx new
│   │           ├── add.rs         # mx add
│   │           ├── recipes.rs     # mx recipes
│   │           ├── dev.rs         # mx dev/up/down/logs
│   │           ├── build.rs       # mx build
│   │           ├── router.rs      # mx router
│   │           ├── infra.rs       # mx infra
│   │           ├── doctor.rs      # mx doctor
│   │           ├── unyform.rs     # mx unyform/login/logout
│   │           ├── mcp.rs         # mx mcp
│   │           └── upgrade.rs     # mx upgrade
│   │
│   └── mx-mcp-server/             # MCP server binary
│       ├── Cargo.toml
│       ├── README.md              # MCP-specific documentation
│       ├── docker-compose.yml     # Weaviate for RAG
│       └── src/
│           ├── main.rs            # Entry point
│           ├── error.rs           # MCP error types
│           ├── mcp/               # MCP protocol implementation
│           │   ├── mod.rs
│           │   ├── protocol.rs    # JSON-RPC types
│           │   ├── server.rs      # Server logic
│           │   └── transport.rs   # stdio transport
│           ├── tools/             # Tool registry (44 tools)
│           │   └── mod.rs
│           ├── mx/                # MX command executor
│           │   └── mod.rs
│           ├── project/           # Project analysis for AI
│           │   └── mod.rs
│           ├── rag/               # Weaviate RAG client
│           │   └── mod.rs
│           ├── unyform/           # Unyform integration
│           │   └── mod.rs
│           ├── weaviate/          # Weaviate management
│           │   └── mod.rs
│           └── bin/
│               └── ingest.rs      # Documentation ingestion binary
│
├── bin/                           # Legacy bash scripts (being phased out)
│   ├── mx                         # Main bash script
│   └── lib/                       # Bash libraries
│       ├── mcp.sh                 # MCP bash helpers
│       └── ...
│
├── templates/                     # Recipe templates
│   └── recipes/
│       ├── laravel/
│       ├── nuxt/
│       ├── rust-api/
│       └── ...
│
└── docs/                          # Documentation
    └── development/
```

---

## Building

### Prerequisites

- **Rust 1.75+**: Install via [rustup](https://rustup.rs/)
- **Docker**: For testing recipes and running Weaviate
- **Make**: For project Makefiles

### Quick Build

```bash
# Navigate to mech-crate root
cd /path/to/mech-crate

# Build all crates in debug mode
cargo build

# Build all crates in release mode (optimized)
cargo build --release

# Build specific crate
cargo build -p mx-cli --release
cargo build -p mx-mcp-server --release
cargo build -p mx-lib
```

### Output Locations

```
target/debug/mx           # Debug CLI binary
target/debug/mx-mcp       # Debug MCP server binary
target/debug/mx-ingest    # Debug ingestion binary

target/release/mx         # Release CLI binary
target/release/mx-mcp     # Release MCP server binary
target/release/mx-ingest  # Release ingestion binary
```

### Development Build with Watch

```bash
# Install cargo-watch
cargo install cargo-watch

# Rebuild CLI on file changes
cargo watch -x 'build -p mx-cli'

# Rebuild MCP server on file changes
cargo watch -x 'build -p mx-mcp-server'

# Rebuild and run tests on changes
cargo watch -x 'test -p mx-lib'
```

### Release Build with Optimizations

The workspace is configured for optimal release builds:

```toml
# Cargo.toml [profile.release]
[profile.release]
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit for better optimization
strip = true         # Strip debug symbols
panic = "abort"      # Smaller binary, no unwinding
```

Build and strip:

```bash
cargo build --release

# Binaries are already stripped via Cargo.toml
ls -lh target/release/mx target/release/mx-mcp
```

---

## Running

### CLI Binary

```bash
# Run from target directory
./target/release/mx --help
./target/release/mx new my-project
./target/release/mx recipes list

# Or install globally
cargo install --path crates/mx-cli

# Then use from anywhere
mx --help
mx doctor
```

### MCP Server

The MCP server runs as a stdio-based JSON-RPC server:

```bash
# Direct execution
./target/release/mx-mcp

# With debug logging
RUST_LOG=debug ./target/release/mx-mcp

# With explicit paths
./target/release/mx-mcp \
  --mech-crate-root /path/to/mech-crate \
  --weaviate-url http://localhost:8080

# Disable RAG
./target/release/mx-mcp --no-rag
```

### MCP Server Management via CLI

The CLI provides commands to manage the MCP server:

```bash
mx mcp build          # Build the MCP server binary
mx mcp start          # Start Weaviate RAG backend
mx mcp stop           # Stop Weaviate
mx mcp status         # Show Weaviate container status
mx mcp logs           # View Weaviate logs
mx mcp logs -f        # Follow logs
mx mcp ingest         # Ingest documentation into Weaviate
mx mcp ingest --clear # Clear and re-ingest
mx mcp config         # Show MCP client configuration
mx mcp run            # Run MCP server directly
mx mcp info           # Show MCP server information
```

---

## CLI Commands

The CLI uses [clap](https://docs.rs/clap/) for argument parsing with derive macros:

### Command Structure

```rust
// main.rs
#[derive(Parser)]
#[command(name = "mx")]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize MechCrate
    Init(InitCommand),
    /// Create a new project
    New(NewCommand),
    /// Add a service
    Add(AddCommand),
    // ... more commands
}
```

### Available Commands

| Command | Description | Implementation |
|---------|-------------|----------------|
| `mx init` | Initialize MechCrate installation | `commands/init.rs` |
| `mx new <name>` | Create a new project | `commands/new.rs` |
| `mx add <name>` | Add a service to project | `commands/add.rs` |
| `mx recipes [list\|info]` | Manage recipes | `commands/recipes.rs` |
| `mx dev [s=service]` | Start development mode | `commands/dev.rs` |
| `mx up [s=service]` | Start production mode | `commands/dev.rs` |
| `mx down [s=service]` | Stop services | `commands/dev.rs` |
| `mx logs [s=service]` | View service logs | `commands/dev.rs` |
| `mx restart s=<svc>` | Restart a service | `commands/dev.rs` |
| `mx sh s=<service>` | Shell into container | `commands/dev.rs` |
| `mx ps` | List running services | `commands/dev.rs` |
| `mx build <service>` | Build Docker image | `commands/build.rs` |
| `mx router [install\|up\|down\|status\|inspect]` | Manage Traefik router | `commands/router.rs` |
| `mx infra [setup\|list\|link]` | Manage infrastructure | `commands/infra.rs` |
| `mx doctor` | Check system health | `commands/doctor.rs` |
| `mx upgrade` | Update project scaffolding | `commands/upgrade.rs` |
| `mx mcp [build\|start\|stop\|...]` | Manage MCP server | `commands/mcp.rs` |
| `mx unyform [...]` | Unyform integration | `commands/unyform.rs` |
| `mx login` | Login to Unyform | `commands/unyform.rs` |
| `mx logout` | Logout from Unyform | `commands/unyform.rs` |
| `mx whoami` | Show current user | `commands/unyform.rs` |

---

## MCP Server

### Protocol Implementation

The MCP server implements the [Model Context Protocol](https://modelcontextprotocol.io/) specification:

```
┌────────────────────────────────────────────────────────────────┐
│                      MCP Client (LLM)                          │
│              (Claude Desktop, Cursor, etc.)                    │
└─────────────────────────┬──────────────────────────────────────┘
                          │ JSON-RPC 2.0 over stdio
┌─────────────────────────▼──────────────────────────────────────┐
│                      mx-mcp Server                              │
│  ┌────────────────────────────────────────────────────────┐   │
│  │                  StdioTransport                         │   │
│  │         (Read from stdin, write to stdout)             │   │
│  └────────────────────────────────────────────────────────┘   │
│  ┌────────────────────────────────────────────────────────┐   │
│  │                   McpServer                             │   │
│  │    ├── handle_initialize()                             │   │
│  │    ├── handle_tools_list()                             │   │
│  │    ├── handle_tool_call()                              │   │
│  │    ├── handle_resources_list()                         │   │
│  │    └── handle_resource_read()                          │   │
│  └────────────────────────────────────────────────────────┘   │
│  ┌────────────────────────────────────────────────────────┐   │
│  │                  ToolRegistry                           │   │
│  │         (44 tools with comprehensive descriptions)     │   │
│  └────────────────────────────────────────────────────────┘   │
│  ┌────────────────────────────────────────────────────────┐   │
│  │              MxExecutor / MakeExecutor                  │   │
│  │           (Execute mx and make commands)               │   │
│  └────────────────────────────────────────────────────────┘   │
│  ┌────────────────────────────────────────────────────────┐   │
│  │                  WeaviateClient                         │   │
│  │              (RAG documentation search)                │   │
│  └────────────────────────────────────────────────────────┘   │
└─────────────────────────┬──────────────────────────────────────┘
                          │ HTTP
┌─────────────────────────▼──────────────────────────────────────┐
│                      Weaviate                                   │
│  ┌─────────────────────────┐  ┌──────────────────────────┐    │
│  │   MechCrateDoc class    │  │  text2vec-transformers   │    │
│  │   (documentation store) │  │  (sentence embeddings)   │    │
│  └─────────────────────────┘  └──────────────────────────┘    │
└────────────────────────────────────────────────────────────────┘
```

### Available Tools (44 total)

#### Global MX Commands (13 tools)

| Tool | Description |
|------|-------------|
| `mx_new` | Create a new MechCrate project |
| `mx_recipes_list` | List available recipes |
| `mx_recipe_info` | Get details about a specific recipe |
| `mx_router_install` | Install the global Traefik router |
| `mx_router_up` | Start the global router |
| `mx_router_down` | Stop the global router |
| `mx_router_status` | Show router container status |
| `mx_router_inspect` | Show router details and connected services |
| `mx_infra_setup` | Configure infrastructure provider credentials |
| `mx_infra_list` | List configured providers |
| `mx_infra_link` | Link project to global credentials |
| `mx_doctor` | Check system health |
| `mx_help` | Show mx command help |

#### Project Commands (3 tools)

| Tool | Description |
|------|-------------|
| `mx_add_service` | Add a service to a project (with optional recipe) |
| `mx_upgrade` | Update project with latest scaffolding |
| `mx_build` | Build Docker image for a service |

#### Make Commands (9 tools)

| Tool | Description |
|------|-------------|
| `make_dev` | Start services in development mode |
| `make_up` | Start services in production mode |
| `make_down` | Stop services |
| `make_logs` | View service logs |
| `make_restart` | Restart a service |
| `make_shell` | Get shell access information |
| `make_ps` | List running services |
| `make_help` | Show available make targets |
| `make_key` | Generate cryptographic keys |

#### Project Analysis (4 tools)

| Tool | Description |
|------|-------------|
| `project_analyze` | Analyze project structure and services |
| `project_list` | Find all MechCrate projects in a directory |
| `project_detect` | Detect if a path is within a project |
| `service_info` | Get details about a specific service |

#### RAG Documentation (7 tools)

| Tool | Description |
|------|-------------|
| `rag_search` | Semantic search across all documentation |
| `rag_search_category` | Search within a specific category |
| `rag_find_implementation` | Find code examples - Dockerfiles, configs, scripts |
| `rag_get_guidance` | Get architecture/design guidance |
| `rag_compare_approaches` | Compare recipes, providers, or strategies |
| `rag_find_related` | Discover related documentation |
| `rag_health` | Check Weaviate availability |

#### Unyform Integration (8 tools)

| Tool | Description |
|------|-------------|
| `unyform_login` | Authenticate with Unyform.ai |
| `unyform_logout` | Clear credentials and session |
| `unyform_whoami` | Show current authentication status |
| `unyform_recipes_list` | List organizational recipes |
| `unyform_recipes_pull` | Pull a recipe to local cache |
| `unyform_recipes_apply` | Apply a recipe to a project |
| `unyform_recipes_versions` | List available versions |
| `unyform_recipes_cache` | Manage cached recipes |

### Configuring MCP Clients

#### Claude Desktop

Add to `~/.claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "mechcrate": {
      "command": "/path/to/mech-crate/target/release/mx-mcp",
      "env": {
        "MECH_CRATE_ROOT": "/path/to/mech-crate"
      }
    }
  }
}
```

#### Cursor

The MCP server is already configured in this workspace via the MechCrate MCP extension.

#### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `WEAVIATE_URL` | Weaviate endpoint | Auto-detected |
| `MECH_CRATE_ROOT` | MechCrate installation directory | Auto-detected |
| `RUST_LOG` | Log level | `info` |

---

## Development Workflow

### Daily Development

```bash
# 1. Start development
cd /path/to/mech-crate

# 2. Run in watch mode
cargo watch -x 'build -p mx-cli'

# 3. In another terminal, test changes
./target/debug/mx --help
./target/debug/mx doctor

# 4. Run tests
cargo test -p mx-lib

# 5. Check lints
cargo clippy -- -D warnings
```

### Adding a New CLI Command

1. **Create the command module** in `crates/mx-cli/src/commands/mycommand.rs`:

```rust
use anyhow::Result;
use clap::Args;
use console::style;

/// Short description for help text
#[derive(Args, Debug)]
pub struct MyCommand {
    /// Argument description
    #[arg(short, long)]
    flag: bool,

    /// Positional argument
    name: Option<String>,
}

impl MyCommand {
    pub async fn run(&self) -> Result<()> {
        println!("{} Running mycommand...", style("→").cyan().bold());
        // Implementation using mx-lib
        println!("{} Done!", style("✓").green().bold());
        Ok(())
    }
}
```

2. **Register in `commands/mod.rs`**:

```rust
pub mod mycommand;
```

3. **Add to `main.rs`**:

```rust
use commands::mycommand::MyCommand;

#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...
    /// Short description
    Mycommand(MyCommand),
}

// In main():
Commands::Mycommand(cmd) => cmd.run().await,
```

### Adding an MCP Tool

1. **Add handler variant** in `crates/mx-mcp-server/src/tools/mod.rs`:

```rust
enum ToolHandler {
    // ... existing handlers ...
    MyTool,
}
```

2. **Add tool definition** in `define_all_tools()`:

```rust
ToolDefinition {
    tool: Tool {
        name: "my_tool".to_string(),
        description: r#"Comprehensive description for LLM.

Include:
- What it does
- When to use it
- Expected inputs
- Output format"#.to_string(),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: Some(json!({
                "param1": {
                    "type": "string",
                    "description": "Parameter description"
                }
            })),
            required: Some(vec!["param1".to_string()]),
        },
    },
    handler: ToolHandler::MyTool,
}
```

3. **Add handler logic** in `execute()`:

```rust
ToolHandler::MyTool => {
    let param1 = args.get("param1")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidArguments("'param1' is required".to_string()))?;
    
    // Use mx-lib or execute commands
    let result = do_something(param1)?;
    
    Ok(ToolCallResult::text(result))
}
```

### Adding Shared Logic to mx-lib

1. **Create module** in `crates/mx-lib/src/mymodule.rs`:

```rust
use crate::error::{Error, Result};

pub struct MyService {
    // fields
}

impl MyService {
    pub fn new() -> Result<Self> {
        Ok(Self { /* ... */ })
    }

    pub fn do_something(&self) -> Result<String> {
        // Implementation
        Ok("result".to_string())
    }
}
```

2. **Export from `lib.rs`**:

```rust
pub mod mymodule;
pub use mymodule::MyService;
```

3. **Use in CLI or MCP**:

```rust
use mx_lib::MyService;

let service = MyService::new()?;
let result = service.do_something()?;
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_function() {
        let result = do_something();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = do_something_async().await;
        assert!(result.is_ok());
    }
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p mx-lib
cargo test -p mx-cli
cargo test -p mx-mcp-server

# Specific test
cargo test -p mx-lib test_something

# With output
cargo test -- --nocapture

# With logging
RUST_LOG=debug cargo test -- --nocapture
```

### Integration Tests

Create `crates/mx-cli/tests/integration_test.rs`:

```rust
use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("mx").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("MechCrate CLI"));
}

#[test]
fn test_doctor_command() {
    let mut cmd = Command::cargo_bin("mx").unwrap();
    cmd.arg("doctor")
        .assert()
        .success();
}
```

Add test dependencies to `Cargo.toml`:

```toml
[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
tempfile = "3.10"
```

---

## Deployment

### Building Release Binaries

```bash
# Build optimized release
cargo build --release

# Verify binary sizes
ls -lh target/release/mx target/release/mx-mcp

# Expected sizes (approximately):
# mx:     ~5-8 MB
# mx-mcp: ~8-12 MB
```

### Installation Locations

```bash
# Install CLI globally
cargo install --path crates/mx-cli

# Install MCP server
cargo install --path crates/mx-mcp-server

# Or copy binaries manually
cp target/release/mx ~/.local/bin/
cp target/release/mx-mcp ~/.local/bin/
```

### CI/CD Build

```yaml
# Example GitHub Actions
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Build
        run: cargo build --release
        
      - name: Test
        run: cargo test
        
      - name: Clippy
        run: cargo clippy -- -D warnings
        
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: binaries
          path: |
            target/release/mx
            target/release/mx-mcp
```

---

## Troubleshooting

### Common Issues

#### Build Errors

```bash
# Clear build cache
cargo clean

# Update dependencies
cargo update

# Check Rust version
rustc --version  # Should be 1.75+
```

#### MCP Server Not Found

```bash
# Set MECH_CRATE_ROOT explicitly
export MECH_CRATE_ROOT=/path/to/mech-crate

# Or pass as argument
./target/release/mx-mcp --mech-crate-root /path/to/mech-crate
```

#### Weaviate Connection Issues

```bash
# Check if Weaviate is running
mx mcp status

# View logs
mx mcp logs

# Restart Weaviate
mx mcp stop && mx mcp start

# Re-ingest documentation
mx mcp ingest --clear
```

#### RAG Returns No Results

```bash
# Check Weaviate health
./target/release/mx-mcp --no-rag  # Bypass RAG

# Re-ingest documentation
mx mcp ingest --clear

# Check port allocation
cat ~/.mech-crate/mcp/.weaviate-http-port
```

### Debug Logging

```bash
# CLI debug logging
mx -v doctor
RUST_LOG=debug mx new my-project

# MCP server debug logging
RUST_LOG=debug ./target/release/mx-mcp

# Specific module logging
RUST_LOG=mx_lib::recipe=trace mx add api --recipe rust-api
```

### Profiling

```bash
# Install profiling tools
cargo install flamegraph

# Generate flamegraph
cargo flamegraph -p mx-cli -- doctor

# Memory profiling
cargo install cargo-instruments  # macOS only
cargo instruments -t Allocations -p mx-cli -- doctor
```

---

## Key Dependencies

| Dependency | Purpose |
|------------|---------|
| `clap` | CLI argument parsing with derive macros |
| `tokio` | Async runtime |
| `serde` / `serde_json` | JSON serialization |
| `anyhow` | Error handling (CLI) |
| `thiserror` | Error type definitions (library) |
| `tracing` | Structured logging |
| `reqwest` | HTTP client |
| `tera` | Template engine |
| `dialoguer` | Interactive prompts |
| `console` | Terminal styling |
| `indicatif` | Progress bars |
| `walkdir` | Directory traversal |
| `dirs` | Platform-specific paths |

---

## Contributing

1. Create a feature branch
2. Make changes following patterns above
3. Run lints and tests:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

4. Update documentation if needed
5. Submit a PR

---

## License

MIT

---

Built with MechCrate
