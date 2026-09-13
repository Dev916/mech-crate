---
title: "MX Rust CLI and MCP Server Development Guide"
category: architecture
languages: [rust]
complexity: advanced
use_cases:
  - "understanding the mx workspace architecture"
  - "adding CLI commands or MCP tools"
  - "building and running mx-cli and mx-mcp"
  - "learning the development workflow"
summary: "A comprehensive guide for building, managing, and extending the MechCrate Rust CLI (mx) and MCP server."
---

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
│  │   Tools     │  │  RAG/pgvector  │  │   JSON-RPC Protocol   │ │
│  │  (47 tools) │  │   corpus       │  │   (stdio transport)   │ │
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
│       └── src/
│           ├── main.rs            # Entry point (--mech-crate-root, --no-rag)
│           ├── error.rs           # MCP error types
│           ├── mcp/               # MCP protocol implementation
│           │   ├── mod.rs
│           │   ├── protocol.rs    # JSON-RPC types
│           │   ├── server.rs      # Server logic
│           │   └── transport.rs   # stdio transport
│           ├── tools/             # Tool registry (47 tools, incl. 8 rag_*)
│           │   └── mod.rs
│           ├── mx/                # MX command executor
│           │   └── mod.rs
│           ├── project/           # Project analysis for AI
│           │   └── mod.rs
│           └── unyform/           # Unyform integration
│               └── mod.rs
│
│           # RAG lives in mx-lib::corpus (pgvector CorpusStore);
│           # ingestion is `mx rag ingest`, not a separate binary.
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
- **Docker**: For testing recipes and running local pgvector (techniques corpus)
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
target/debug/mx           # Debug CLI binary (includes `mx rag` ingestion)
target/debug/mx-mcp       # Debug MCP server binary

target/release/mx         # Release CLI binary (includes `mx rag` ingestion)
target/release/mx-mcp     # Release MCP server binary
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

# With explicit root
./target/release/mx-mcp --mech-crate-root /path/to/mech-crate

# Disable the techniques corpus (rag_* tools report offline)
./target/release/mx-mcp --no-rag
```

### MCP Server Management via CLI

The CLI provides commands to manage the MCP server:

```bash
mx mcp build          # Build the MCP server binary
mx mcp status         # Show corpus backend + doc/chunk counts
mx mcp config         # Show MCP client configuration
mx mcp run            # Run MCP server directly
mx mcp info           # Show MCP server information
```

Corpus ingestion is a separate CLI surface:

```bash
mx rag ingest          # Ingest docs/development into the pgvector corpus
mx rag ingest --clear  # Clear and re-ingest
mx rag ingest --dry-run  # Parse/chunk only (no DB or embeddings)
mx rag status          # Backend, doc/chunk counts, embedding model
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
│  │         (47 tools with comprehensive descriptions)     │   │
│  └────────────────────────────────────────────────────────┘   │
│  ┌────────────────────────────────────────────────────────┐   │
│  │              MxExecutor / MakeExecutor                  │   │
│  │           (Execute mx and make commands)               │   │
│  └────────────────────────────────────────────────────────┘   │
│  ┌────────────────────────────────────────────────────────┐   │
│  │            CorpusStore (mx_lib::corpus)                 │   │
│  │        (8 rag_* tools, hybrid technique search)        │   │
│  └────────────────────────────────────────────────────────┘   │
└─────────────────────────┬──────────────────────────────────────┘
                          │ SQL (Neon primary → local fallback)
┌─────────────────────────▼──────────────────────────────────────┐
│                  Postgres + pgvector                            │
│  ┌─────────────────────────┐  ┌──────────────────────────┐    │
│  │ technique_docs/_chunks  │  │  OpenAI-compatible        │    │
│  │ HNSW cosine + pg_trgm   │  │  embeddings (1536 dims)   │    │
│  └─────────────────────────┘  └──────────────────────────┘    │
└────────────────────────────────────────────────────────────────┘
```

### Available Tools (47 total)

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

#### Techniques Corpus / RAG (8 tools)

| Tool | Description |
|------|-------------|
| `rag_context` | **Primary.** Describe what you're `working_on` → techniques grouped by source doc |
| `rag_search` | Hybrid semantic + lexical search across the techniques corpus |
| `rag_search_category` | Search within a specific category (theory, patterns, concurrency, ...) |
| `rag_find_implementation` | Find implementations for a concept, filtered by language |
| `rag_get_guidance` | Get architecture/design guidance with optional constraints |
| `rag_compare_approaches` | Compare two approaches side by side |
| `rag_find_related` | Discover related techniques from other docs |
| `rag_health` | Report backend (`neon`/`local`/`offline`), doc/chunk counts, embedding model |

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
| `MECH_CRATE_ROOT` | MechCrate installation directory | Auto-detected |
| `MX_RAG_DATABASE_URL` | Neon (primary) corpus URL | From `rag.toml`, else local fallback |
| `MX_RAG_FALLBACK_DATABASE_URL` | Local Postgres fallback URL | `postgres://postgres@localhost:5432/mx_rag` |
| `OPENAI_API_KEY` | Embedding API key (or `MX_RAG_EMBEDDING_API_KEY`) | — |
| `MX_RAG_EMBEDDING_MODEL` | Embedding model | `text-embedding-3-small` |
| `RUST_LOG` | Log level | `info` |

Corpus configuration is read from `~/.mech-crate/config/rag.toml`; the `MX_RAG_*`
and `OPENAI_API_KEY` env vars override it.

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

## Release and self-update

### Release channel

Releases are GitHub Releases on the public repo `unyform-ai/mech-crate-releases`,
one per git tag `vX.Y.Z` on `Dev916/mech-crate`. `GET .../releases/latest` is
the source of truth for "what is the newest version": GitHub returns only
published, non-draft, non-prerelease releases from it, which is exactly the
contract `mx self-update` relies on.

Assets per release, named by `scripts/package.sh`:

```
mx-v<version>-universal-apple-darwin.tar.gz          (+ .sha256)
mx-v<version>-x86_64-unknown-linux-musl.tar.gz       (+ .sha256)
mx-v<version>-aarch64-unknown-linux-musl.tar.gz      (+ .sha256)
```

Each tarball extracts to `mx-v<version>/` containing `bin/{mx,mx-mcp}`,
`bin/lib/*.sh`, `templates/`, `scripts/`, the two license files and a
`VERSION` file. That directory is a complete MechCrate root:
`paths::mech_crate_root()` walks up from the executable to the first
directory with `scripts/`, so nothing about path resolution changes between a
checkout and an installed release.

### The pipeline (`.github/workflows/release.yml`)

```
tag v0.1.2 (or workflow_dispatch with version=0.1.2)
  └─ resolve-version   validates the version shape
     └─ test           fmt + clippy + nextest + coverage — the release gate
        └─ create-draft   one draft release per tag (prerelease when the
           │              version has a suffix); skipped with a warning when
           │              RELEASES_REPO_TOKEN is absent, so a dry run still builds
           ├─ macos     aarch64 + x86_64 builds, lipo, codesign (Developer ID
           │            when the APPLE_* secrets exist, ad-hoc otherwise),
           │            notarize, package, upload into the draft
           ├─ linux     cross builds for both musl triples, proven static,
           │            package, upload into the draft
           └─ publish   flips the draft live once every platform uploaded
```

The draft-then-publish shape is what keeps `releases/latest` from ever
pointing at a release with a missing platform. Secrets: `RELEASES_REPO_TOKEN`
(a fine-grained PAT owned by the unyform-ai org with contents:write on the
releases repo), and the six `APPLE_*` values for Developer ID signing and
notarization. Without the Apple secrets binaries are ad-hoc signed: they run
when installed by `mx self-update` or the curl installer (no quarantine
attribute), but a browser download would be refused by Gatekeeper.

To cut a release: bump `[workspace.package] version` in `Cargo.toml`, commit,
`git tag vX.Y.Z`, push the tag. To rehearse without a tag: run the workflow by
hand with a version such as `0.1.3-rc.1`; the suffix marks it prerelease.

### Install layout

```
~/.mech-crate/
  releases/mx-v0.1.2/       # an extracted tarball, untouched
  current -> releases/mx-v0.1.2
  templates/                # refreshed from current/templates
  version                   # mirrors current/VERSION
  tmp/                      # scratch during extraction, always emptied
  mcp/mx-mcp-wrapper.sh     # regenerated to point at current/ if it exists
~/.local/bin/{mx,mx-mcp} -> ~/.mech-crate/current/bin/{mx,mx-mcp}
```

`crates/mx-lib/src/selfupdate/layout.rs` is the only writer of that layout.
An update extracts under `tmp/<uuid>`, renames the bundle into `releases/`,
then replaces `current` by renaming a fresh symlink over it — never
rm-then-ln — so the running process keeps its inode and a crash at any point
leaves the previous install live. `--rollback` is the same flip in reverse;
the previous release is kept and older ones pruned.

### The engine (`crates/mx-lib/src/selfupdate/`)

| File | Role | IO |
|---|---|---|
| `version.rs` | semver parse/compare, `current()` | none |
| `target.rs` | host triple, asset / checksum / bundle-dir names | none |
| `kind.rs` | `InstallKind` {Release, Homebrew, Source, Bare} from the exe path | none (repo probe injected) |
| `plan.rs` | `UpdatePlan` {UpToDate, Download, DelegateBrew, RebuildSource} | none |
| `index.rs` | `ReleaseIndex`: GitHub releases client, `MX_RELEASES_API` override | HTTP |
| `fetch.rs` | streamed download to `.part` + rename, `.sha256` parse, verify | HTTP, fs |
| `layout.rs` | extract / adopt / flip / previous / prune / shims | fs |
| `verify.rs` | new binary answers `--version`; `codesign --verify` on Mach-O | process |
| `refresh.rs` | templates swap, version file, MCP wrapper; shared recursive copy | fs |
| `notify.rs` | once-a-day cache and the hint decision | fs |

The pure half is unit-tested on literal values; the effectful half is
contract-tested against wiremock and tempdirs. `crates/mx-cli/src/commands/
self_update.rs` is a thin shell: detect → plan → print → confirm → execute.
`crates/mx-cli/tests/self_update.rs` runs the whole command hermetically
(`MX_SELFUPDATE_EXE` and `HOMEBREW_PREFIX` pin the install kind under test;
`mx_lib::test_support::write_fake_bundle` / `pack_bundle` fake a release).

### Installer and update hint

`site/apps/site/public/install.sh`, served at `https://mechcrate.dev/install.sh`,
is POSIX sh: it resolves the release, downloads and verifies the tarball,
extracts it, and runs `<bundle>/bin/mx self-update --from-dir <bundle> --yes`
so the Rust code stays the single writer of the layout.

Every non-`self-update`, non-`mcp` command reads
`~/.mech-crate/cache/update-check.json`; when it is older than a day a
detached `mx self-update --refresh-cache` is spawned and, if the cache says
a newer release exists, one line is printed to stderr (TTY only, never under
`CI` or `MX_NO_UPDATE_CHECK=1`, or with `check = false` in
`~/.mech-crate/config/update.toml`).

Design record: `docs/superpowers/specs/2026-09-02-mx-self-update-design.md`.

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

#### Corpus Connection Issues

```bash
# Show the active backend (neon / local / offline) and counts
mx rag status

# Start a local pgvector instance if neither Neon nor local is reachable
docker run -d --name mx-rag -p 5432:5432 \
  -e POSTGRES_DB=mx_rag -e POSTGRES_HOST_AUTH_METHOD=trust pgvector/pgvector:pg17

# Or point at Neon
export MX_RAG_DATABASE_URL=postgres://...neon.tech/mx_rag

# Re-ingest the corpus
mx rag ingest --clear
```

#### RAG Returns No Results

```bash
# Confirm the corpus is populated
mx rag status

# Re-ingest the corpus
mx rag ingest --clear

# If search is trigram-only, set the embedding key and re-embed
export OPENAI_API_KEY=sk-...
mx rag ingest --reembed
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
