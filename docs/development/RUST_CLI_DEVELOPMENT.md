# MechCrate Rust CLI Development Guide

This guide covers how to develop, build, and extend the MechCrate CLI (`mx`) and MCP server.

> **Related Documentation:**
> - [MX Rust CLI and MCP Server Guide](./MX_RUST_CLI_AND_MCP_SERVER.md) - Comprehensive build and management guide
> - [MX Quick Reference](./MX_QUICK_REFERENCE.md) - Quick reference card for daily development

## Architecture Overview

MechCrate uses a Cargo workspace with three crates:

```
mech-crate/
├── Cargo.toml              # Workspace manifest
├── crates/
│   ├── mx-lib/             # Shared library (core logic)
│   ├── mx-cli/             # CLI binary (user-facing commands)
│   └── mx-mcp-server/      # MCP server (AI agent interface)
├── templates/              # Recipe templates and project scaffolding
└── bin/                    # Legacy bash scripts (being phased out)
```

### Crate Responsibilities

| Crate | Purpose | Output |
|-------|---------|--------|
| `mx-lib` | Core business logic, shared across CLI and MCP | Library |
| `mx-cli` | User-facing CLI commands | `mx` binary |
| `mx-mcp-server` | JSON-RPC server for AI agents | `mx-mcp` binary |

## Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- Docker (for testing recipes)
- Make (for project Makefiles)

## Building

### Quick Build

```bash
# Build all crates (debug)
cargo build

# Build all crates (release)
cargo build --release

# Build specific crate
cargo build -p mx-cli --release
cargo build -p mx-mcp-server --release
```

### Output Locations

```
target/debug/mx           # Debug CLI binary
target/debug/mx-mcp       # Debug MCP server binary
target/release/mx         # Release CLI binary
target/release/mx-mcp     # Release MCP server binary
```

### Development Build with Watch

```bash
# Install cargo-watch if needed
cargo install cargo-watch

# Rebuild on changes
cargo watch -x 'build -p mx-cli'
```

## Project Structure

### mx-lib (Shared Library)

```
crates/mx-lib/
├── Cargo.toml
└── src/
    ├── lib.rs              # Module exports
    ├── error.rs            # Error types (thiserror)
    ├── paths.rs            # Path resolution (~/.mech-crate)
    ├── config.rs           # Global configuration
    ├── project.rs          # Project detection and analysis
    ├── recipe/
    │   ├── mod.rs          # Recipe module exports
    │   ├── parser.rs       # recipe.json parsing
    │   └── installer.rs    # Recipe installation logic
    ├── template/
    │   ├── mod.rs          # Template module exports
    │   └── engine.rs       # Tera template engine
    ├── docker/
    │   └── mod.rs          # Docker/Compose wrappers
    ├── infra/
    │   ├── mod.rs          # Infrastructure module
    │   └── config.rs       # Provider configuration
    ├── router/
    │   └── mod.rs          # Traefik router management
    ├── mcp/
    │   └── mod.rs          # MCP server & Weaviate management
    ├── upgrade/
    │   └── mod.rs          # Project upgrade functionality
    └── unyform/
        └── mod.rs          # Unyform API client
```

### mx-cli (CLI Binary)

```
crates/mx-cli/
├── Cargo.toml
└── src/
    ├── main.rs             # Entry point, clap setup
    └── commands/
        ├── mod.rs          # Command module exports
        ├── init.rs         # mx init
        ├── new.rs          # mx new
        ├── add.rs          # mx add
        ├── recipes.rs      # mx recipes
        ├── dev.rs          # mx dev/up/down/logs/etc
        ├── build.rs        # mx build
        ├── router.rs       # mx router
        ├── infra.rs        # mx infra
        ├── doctor.rs       # mx doctor
        ├── unyform.rs      # mx unyform/login/logout
        ├── mcp.rs          # mx mcp
        └── upgrade.rs      # mx upgrade
```

### mx-mcp-server (MCP Server)

```
crates/mx-mcp-server/
├── Cargo.toml
└── src/
    ├── main.rs             # Entry point, JSON-RPC server
    ├── tools/
    │   └── mod.rs          # Tool definitions and handlers
    ├── project.rs          # Project analysis for AI
    ├── rag/
    │   └── mod.rs          # Documentation search
    └── unyform/
        └── mod.rs          # Unyform tools for AI
```

## Adding a New CLI Command

### 1. Create the Command Module

Create `crates/mx-cli/src/commands/mycommand.rs`:

```rust
//! `mx mycommand` - Description of what it does

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
        println!(
            "{} Running mycommand...",
            style("→").cyan().bold()
        );

        // Use mx-lib for core logic
        // ...

        println!(
            "{} Done!",
            style("✓").green().bold()
        );

        Ok(())
    }
}
```

### 2. Register the Command

In `crates/mx-cli/src/commands/mod.rs`:

```rust
pub mod mycommand;  // Add this line
```

In `crates/mx-cli/src/main.rs`:

```rust
// Add import
use commands::mycommand::MyCommand;

// Add to Commands enum
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...
    
    /// Short description
    Mycommand(MyCommand),
}

// Add handler in main()
let result = match cli.command {
    // ... existing handlers ...
    Commands::Mycommand(cmd) => cmd.run().await,
};
```

### 3. Add Subcommands (Optional)

For commands with subcommands like `mx recipes list`:

```rust
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct MyCommand {
    #[command(subcommand)]
    command: MySubcommands,
}

#[derive(Subcommand, Debug)]
enum MySubcommands {
    /// List items
    List(ListArgs),
    /// Show item info
    Info(InfoArgs),
}

#[derive(Args, Debug)]
struct ListArgs {
    #[arg(short, long)]
    all: bool,
}

#[derive(Args, Debug)]
struct InfoArgs {
    name: String,
}

impl MyCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            MySubcommands::List(args) => self.list(args).await,
            MySubcommands::Info(args) => self.info(args).await,
        }
    }

    async fn list(&self, args: &ListArgs) -> Result<()> {
        // ...
    }

    async fn info(&self, args: &InfoArgs) -> Result<()> {
        // ...
    }
}
```

## Adding Shared Logic to mx-lib

### 1. Create a New Module

Create `crates/mx-lib/src/mymodule.rs` or `crates/mx-lib/src/mymodule/mod.rs`:

```rust
//! My module description

use crate::error::{Error, Result};

pub struct MyService {
    // fields
}

impl MyService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            // ...
        })
    }

    pub fn do_something(&self) -> Result<()> {
        // ...
        Ok(())
    }
}
```

### 2. Export from lib.rs

In `crates/mx-lib/src/lib.rs`:

```rust
pub mod mymodule;

// Optionally re-export for convenience
pub use mymodule::MyService;
```

### 3. Use in CLI or MCP Server

```rust
use mx_lib::MyService;

let service = MyService::new()?;
service.do_something()?;
```

## Adding an MCP Tool

### 1. Define the Tool Handler

In `crates/mx-mcp-server/src/tools/mod.rs`:

```rust
// Add to ToolHandler enum
pub enum ToolHandler {
    // ... existing handlers ...
    MyTool,
}

// Add tool definition
pub fn register_tools(registry: &mut ToolRegistry) {
    // ... existing tools ...

    registry.register(ToolDefinition {
        name: "my_tool".to_string(),
        description: "Description for AI agents".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "param1": {
                    "type": "string",
                    "description": "Parameter description"
                }
            },
            "required": ["param1"]
        }),
        handler: ToolHandler::MyTool,
    });
}

// Add execute handler
pub async fn execute_tool(
    handler: &ToolHandler,
    params: &serde_json::Value,
) -> ToolResult {
    match handler {
        // ... existing handlers ...
        ToolHandler::MyTool => {
            let param1 = params["param1"].as_str()
                .ok_or("param1 is required")?;
            
            // Use mx-lib for logic
            let result = mx_lib::my_module::do_something(param1)?;
            
            Ok(serde_json::json!({
                "success": true,
                "result": result
            }))
        }
    }
}
```

## Error Handling

### Using thiserror in mx-lib

```rust
// crates/mx-lib/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Template error: {0}")]
    Template(#[from] tera::Error),

    #[error("Not a MechCrate project")]
    NotAProject,

    #[error("Recipe not found: {0}")]
    RecipeNotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),

    // Add new error variants here
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Using anyhow in CLI Commands

```rust
use anyhow::{Result, Context, bail};

pub async fn run(&self) -> Result<()> {
    // Add context to errors
    let file = std::fs::read_to_string(&path)
        .context("Failed to read config file")?;

    // Bail with custom message
    if !valid {
        bail!("Invalid configuration: {}", reason);
    }

    Ok(())
}
```

## Testing

### Unit Tests

```rust
// In any module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        let result = do_something();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_something() {
        let result = async_do_something().await;
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

# Specific test
cargo test -p mx-lib test_something

# With output
cargo test -- --nocapture
```

### Integration Tests

Create `crates/mx-cli/tests/integration_test.rs`:

```rust
use assert_cmd::Command;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("mx").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("MechCrate CLI"));
}
```

## Path Resolution

MechCrate uses a priority-based path resolution system:

```rust
use mx_lib::{templates_dir, is_initialized, home_dir};

// Get templates directory (auto-resolved)
let templates = templates_dir()?;

// Check if mx init has been run
if !is_initialized() {
    bail!("Run 'mx init' first");
}

// Get ~/.mech-crate path
let home = home_dir()?;
```

Resolution order:
1. `MECH_CRATE_ROOT` env var (development override)
2. `~/.mech-crate/templates` (standard installation)
3. Relative to executable (portable mode)

## Common Patterns

### Interactive Prompts (CLI only)

```rust
use dialoguer::{Select, Input, Confirm, MultiSelect};
use console::style;

// Single selection
let options = vec!["Option A", "Option B", "Option C"];
let selection = Select::new()
    .with_prompt("Choose an option")
    .items(&options)
    .default(0)
    .interact()?;

// Text input
let name: String = Input::new()
    .with_prompt("Project name")
    .default("my-project".into())
    .interact_text()?;

// Confirmation
let proceed = Confirm::new()
    .with_prompt("Continue?")
    .default(true)
    .interact()?;

// Multiple selection
let selected = MultiSelect::new()
    .with_prompt("Select features")
    .items(&["Feature A", "Feature B", "Feature C"])
    .interact()?;
```

### Progress Bars

```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(total_items as u64);
pb.set_style(
    ProgressStyle::default_bar()
        .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("█▓░"),
);

for item in items {
    pb.set_message(format!("Processing {}", item.name));
    // process item
    pb.inc(1);
}

pb.finish_with_message("Done!");
```

### Styled Output

```rust
use console::style;

// Colors
println!("{}", style("Success!").green());
println!("{}", style("Warning!").yellow());
println!("{}", style("Error!").red());

// Formatting
println!("{}", style("Bold text").bold());
println!("{}", style("Dim text").dim());
println!("{}", style("Underline").underlined());

// Common patterns
println!("{} Processing...", style("→").cyan().bold());
println!("{} Complete!", style("✓").green().bold());
println!("{} Failed!", style("✗").red().bold());
```

## Dependencies

Key dependencies used across the workspace:

| Dependency | Purpose |
|------------|---------|
| `clap` | CLI argument parsing |
| `tokio` | Async runtime |
| `serde` / `serde_json` | Serialization |
| `anyhow` | Error handling (CLI) |
| `thiserror` | Error types (library) |
| `tracing` | Logging |
| `reqwest` | HTTP client |
| `tera` | Template engine |
| `dialoguer` | Interactive prompts |
| `console` | Terminal styling |
| `indicatif` | Progress bars |
| `walkdir` | Directory traversal |
| `dirs` | Platform paths |

## Installation Workflow

The CLI uses a two-step workflow:

### 1. Build
```bash
cargo build --release
```

### 2. Initialize
```bash
# Initialize MechCrate (copies templates to ~/.mech-crate/)
# Must set MECH_CRATE_ROOT for first-time init
MECH_CRATE_ROOT=$(pwd) ./target/release/mx init
```

### 3. Use Anywhere
```bash
# After init, mx works from any directory
./target/release/mx recipes list
./target/release/mx doctor
./target/release/mx new my-project
```

### Path Resolution

The CLI resolves templates automatically:
1. `MECH_CRATE_ROOT` env var (development override)
2. `~/.mech-crate/templates` (standard installation)
3. Relative to executable (fallback)

## Release Build

```bash
# Build optimized release binaries
cargo build --release

# Strip debug symbols (smaller binary)
strip target/release/mx
strip target/release/mx-mcp

# Check binary sizes
ls -lh target/release/mx target/release/mx-mcp
```

## Debugging

### Enable Debug Logging

```bash
# Set log level
RUST_LOG=debug mx doctor
RUST_LOG=mx_lib=trace mx new my-project

# Specific modules
RUST_LOG=mx_lib::recipe=debug mx add api --recipe rust-api
```

### Using the Verbose Flag

```bash
mx -v new my-project
mx --verbose recipes list
```

## IDE Setup

### VS Code with rust-analyzer

`.vscode/settings.json`:
```json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.linkedProjects": [
        "./Cargo.toml"
    ]
}
```

### Cursor

The workspace is already configured for optimal rust-analyzer support via the workspace Cargo.toml.

## Contributing

1. Create a feature branch
2. Make changes following the patterns above
3. Run `cargo fmt` and `cargo clippy`
4. Add tests for new functionality
5. Update documentation if needed
6. Submit a PR

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run all checks
cargo fmt && cargo clippy && cargo test
```
