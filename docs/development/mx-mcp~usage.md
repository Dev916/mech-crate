# MechCrate MCP Server

A Model Context Protocol (MCP) server that enables LLMs to interact with MechCrate projects, providing full operational capabilities for project management, service orchestration, infrastructure configuration, and intelligent documentation retrieval.

## Features

- **Full MX Command Access**: Create projects, add services, manage router, configure infrastructure
- **Project Makefile Operations**: dev, up, down, logs, shell, restart, build, and more
- **Project Analysis**: Detect projects, list services, inspect configuration
- **RAG Documentation**: Semantic search with specialized query modes via Weaviate
- **Comprehensive Tool Descriptions**: Detailed documentation for LLM understanding
- **Auto-Start Weaviate**: Automatically starts the RAG backend with dynamic port allocation
- **Port Conflict Resolution**: Handles multiple instances with automatic port allocation

## Quick Start

### 1. Build the Server

```bash
# From mech-crate root:
mx mcp build

# Or manually:
cd mcp-server && cargo build --release
```

### 2. Start Weaviate & Ingest Documentation

```bash
# Start Weaviate (auto-allocates ports if 8080 is busy)
mx mcp start

# Ingest documentation
mx mcp ingest
```

### 3. Get Client Configuration

```bash
mx mcp config
```

This generates a wrapper script that auto-starts Weaviate and provides the correct configuration.

### 4. Configure MCP Client

The `mx mcp config` command outputs the configuration. Example for Claude Desktop (`~/.claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "mechcrate": {
      "command": "/Users/you/.mech-crate/mcp/mx-mcp-wrapper.sh",
      "env": {
        "MECH_CRATE_ROOT": "/path/to/mech-crate"
      }
    }
  }
}
```

### Alternative: Direct Binary (Weaviate auto-starts)

The MCP server can auto-start Weaviate when launched:

```json
{
  "mcpServers": {
    "mechcrate": {
      "command": "/path/to/mech-crate/mcp-server/target/release/mx-mcp",
      "env": {
        "MECH_CRATE_ROOT": "/path/to/mech-crate"
      }
    }
  }
}
```

## Port Allocation

When multiple Weaviate instances need to run (or port 8080 is busy), the server automatically allocates ports:

| Service | Default | Range |
|---------|---------|-------|
| Weaviate HTTP | 8080 | 8080-8179 |
| Weaviate gRPC | 50051 | 50051-50150 |

Port state is stored in `~/.mech-crate/mcp/`:
- `.weaviate-http-port` - Currently allocated HTTP port
- `.weaviate-grpc-port` - Currently allocated gRPC port

Override the ranges with environment variables:
```bash
MX_MCP_HTTP_PORT_RANGE=9080-9179 mx mcp start
```

## Available Tools (44 total)

### Global MX Commands

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

### Project Commands

| Tool | Description |
|------|-------------|
| `mx_add_service` | Add a service to a project (with optional recipe) |
| `mx_upgrade` | Update project with latest scaffolding |
| `mx_build` | Build Docker image for a service |

### Make Commands (Project Operations)

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

### Project Analysis

| Tool | Description |
|------|-------------|
| `project_analyze` | Analyze project structure and services |
| `project_list` | Find all MechCrate projects in a directory |
| `project_detect` | Detect if a path is within a project |
| `service_info` | Get details about a specific service |

### RAG Documentation (7 tools)

| Tool | Description |
|------|-------------|
| `rag_search` | Semantic search across all documentation |
| `rag_search_category` | Search within a specific category (recipe, command, docker, etc.) |
| `rag_find_implementation` | Find code examples - Dockerfiles, compose, configs, scripts |
| `rag_get_guidance` | Get architecture/design guidance with optional constraints |
| `rag_compare_approaches` | Compare recipes, providers, or implementation strategies |
| `rag_find_related` | Discover related documentation for a topic |
| `rag_health` | Check Weaviate availability |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        MCP Client (LLM)                         │
└─────────────────────────────┬───────────────────────────────────┘
                              │ JSON-RPC over stdio
┌─────────────────────────────▼───────────────────────────────────┐
│                       mx-mcp Server                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ MX Executor │  │ Make Exec.  │  │ Project Detector        │  │
│  │ (bin/mx)    │  │ (make)      │  │ (analyze, discover)     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Tool Registry (44 tools)                  ││
│  │  Comprehensive LLM descriptions for intelligent tool use    ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │              Weaviate RAG Client (7 query modes)             ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────┬───────────────────────────────────┘
                              │ HTTP (auto-allocated port)
┌─────────────────────────────▼───────────────────────────────────┐
│                        Weaviate                                  │
│  ┌─────────────────┐  ┌─────────────────────────────────────┐  │
│  │ MechCrateDoc    │  │ text2vec-transformers              │  │
│  │ (documentation) │  │ (sentence-transformers)            │  │
│  └─────────────────┘  └─────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Example LLM Interactions

### Create a New Project

```
User: Create a new project called "myapp" with Cloudflare infrastructure

LLM uses: mx_new(name="myapp", with_infra=["cloudflare"], working_directory="/Users/me/projects")
```

### Add a Laravel API Service

```
User: Add a Laravel API service to the myapp project

LLM uses: mx_add_service(name="api", recipe="laravel", project_path="/Users/me/projects/myapp")
```

### Start Development

```
User: Start the API service in development mode

LLM uses: 
1. mx_router_up()  # Ensure router is running
2. make_dev(service="api", project_path="/Users/me/projects/myapp")
```

### Query Documentation

```
User: How do I configure Traefik routing for my services?

LLM uses: rag_search(query="configure Traefik routing labels for services", limit=5)
```

### Find Code Examples

```
User: Show me how to write a multi-stage Dockerfile

LLM uses: rag_find_implementation(pattern="multi-stage Dockerfile", language="dockerfile")
```

### Get Architecture Guidance

```
User: I need to choose between Laravel and Nuxt for my project. It needs SSR and good SEO.

LLM uses: 
1. rag_compare_approaches(approaches=["laravel", "nuxt"], criteria=["SSR", "SEO"])
2. rag_get_guidance(problem="choosing between Laravel and Nuxt for SSR with SEO", constraints=["needs SSR", "SEO important"])
```

### Analyze Project Structure

```
User: What services does this project have?

LLM uses: project_analyze(project_path="/Users/me/projects/myapp")
```

## Development

### Running Tests

```bash
cargo test
```

### Building Debug Version

```bash
cargo build
```

### Logging

Set `RUST_LOG` for debug output:

```bash
RUST_LOG=debug ./target/release/mx-mcp
```

## MX MCP Commands

```bash
mx mcp build          # Build the MCP server binary
mx mcp start          # Start Weaviate RAG backend (with port allocation)
mx mcp stop           # Stop Weaviate
mx mcp status         # Show Weaviate container status
mx mcp logs           # Tail Weaviate logs
mx mcp ingest         # Ingest documentation into Weaviate
mx mcp ingest --clear # Clear and re-ingest
mx mcp config         # Show MCP client configuration
mx mcp run            # Run MCP server (auto-starts Weaviate)
mx mcp test           # Test MCP server response
mx mcp info           # Show MCP server information
mx mcp help           # Show help
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `WEAVIATE_URL` | Weaviate endpoint | Auto-detected from stored port |
| `MECH_CRATE_ROOT` | MechCrate installation directory | Auto-detected |
| `RUST_LOG` | Log level | `info` |
| `MX_MCP_HTTP_PORT_RANGE` | HTTP port allocation range | `8080-8179` |
| `MX_MCP_GRPC_PORT_RANGE` | gRPC port allocation range | `50051-50150` |

## Troubleshooting

### Weaviate Not Available

```bash
# Check if containers are running
mx mcp status

# View logs
mx mcp logs

# Restart
mx mcp stop && mx mcp start
```

### MechCrate Root Not Found

Set explicitly:

```bash
./mx-mcp --mech-crate-root /path/to/mech-crate
```

Or via environment:

```bash
export MECH_CRATE_ROOT=/path/to/mech-crate
```

### RAG Search Returns No Results

Re-ingest documentation:

```bash
mx mcp ingest --clear
```

### Port Conflicts

Check allocated ports:

```bash
mx mcp info
```

Force new port allocation:

```bash
rm ~/.mech-crate/mcp/.weaviate-*-port
mx mcp start
```

## License

MIT

---

🦝 Built with MechCrate
