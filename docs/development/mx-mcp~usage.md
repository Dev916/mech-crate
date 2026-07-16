---
title: "MechCrate MCP Server Usage"
category: process
languages: []
complexity: intermediate
use_cases:
  - "understanding MCP server capabilities"
  - "building and starting the MCP server"
  - "accessing mx commands through MCP"
  - "using RAG documentation retrieval"
summary: "Overview of the MechCrate MCP server that lets LLMs manage projects, orchestrate services, configure infrastructure, and retrieve documentation."
---

# MechCrate MCP Server

A Model Context Protocol (MCP) server that enables LLMs to interact with MechCrate projects, providing full operational capabilities for project management, service orchestration, infrastructure configuration, and intelligent documentation retrieval.

## Features

- **Full MX Command Access**: Create projects, add services, manage router, configure infrastructure
- **Project Makefile Operations**: dev, up, down, logs, shell, restart, build, and more
- **Project Analysis**: Detect projects, list services, inspect configuration
- **Techniques Corpus**: Hybrid semantic + lexical search over development techniques, backed by Postgres + pgvector
- **Comprehensive Tool Descriptions**: Detailed documentation for LLM understanding
- **Resilient Corpus Access**: Neon primary with a local Postgres fallback; RAG tools degrade to an offline message instead of crashing the server

## Quick Start

### 1. Build the Server

```bash
# From mech-crate root:
mx mcp build

# Or manually:
cd mcp-server && cargo build --release
```

### 2. Ingest the Techniques Corpus

The corpus lives in Postgres + pgvector. Point it at a database via
`~/.mech-crate/config/rag.toml` (Neon primary, local Postgres fallback), or spin
up a local instance:

```bash
docker run -d --name mx-rag -p 5432:5432 \
  -e POSTGRES_DB=mx_rag -e POSTGRES_HOST_AUTH_METHOD=trust pgvector/pgvector:pg17
```

Then ingest `docs/development` (embeddings use `OPENAI_API_KEY`):

```bash
mx rag ingest          # scan, chunk, embed, upsert (idempotent)
mx rag status          # backend, doc/chunk counts, embedding model
```

### 3. Get Client Configuration

```bash
mx mcp config
```

This outputs the MCP client configuration for the server.

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

### Alternative: Direct Binary

Point the client straight at the built binary. The server connects to the
corpus lazily on the first RAG call and never blocks startup on it:

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

Pass `--no-rag` to disable the corpus entirely (RAG tools then report offline).

## Available Tools (47 total)

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

### Techniques Corpus / RAG (8 tools)

| Tool | Description |
|------|-------------|
| `rag_context` | **Primary entry point.** Describe what you're `working_on` (+ optional language/category) and get relevant techniques grouped by source doc |
| `rag_search` | Hybrid semantic + lexical search across the techniques corpus |
| `rag_search_category` | Search within a specific category (theory, patterns, concurrency, database, etc.) |
| `rag_find_implementation` | Find implementations for a concept, filtered by language |
| `rag_get_guidance` | Get architecture/design guidance with optional constraints |
| `rag_compare_approaches` | Compare two approaches side by side with relevant chunks |
| `rag_find_related` | Discover related techniques from other docs |
| `rag_health` | Report the active backend (`neon`/`local`/`offline`), doc/chunk counts, and embedding model |

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
│  │                    Tool Registry (47 tools)                  ││
│  │  Comprehensive LLM descriptions for intelligent tool use    ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │           CorpusStore (8 rag_* tools, hybrid search)         ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────┬───────────────────────────────────┘
                              │ SQL (Neon primary → local fallback)
┌─────────────────────────────▼───────────────────────────────────┐
│                    Postgres + pgvector                           │
│  ┌─────────────────┐  ┌─────────────────────────────────────┐  │
│  │ technique_docs  │  │ technique_chunks                    │  │
│  │ technique_chunks│  │ HNSW cosine + pg_trgm (0.85/0.15)   │  │
│  └─────────────────┘  └─────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘

Embeddings come from an OpenAI-compatible `/embeddings` endpoint
(`text-embedding-3-small`, 1536 dims) via the `EmbeddingProvider` trait.
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
mx mcp status         # Show corpus backend + doc/chunk counts
mx mcp config         # Show MCP client configuration
mx mcp run            # Run MCP server interactively
mx mcp test           # Test MCP server response
mx mcp info           # Show MCP server information
mx mcp help           # Show help
```

Corpus ingestion lives under `mx rag`:

```bash
mx rag ingest          # Ingest docs/development into the pgvector corpus
mx rag ingest --clear  # Clear and re-ingest
mx rag ingest --dry-run  # Parse/chunk only (no DB or embeddings)
mx rag status          # Backend, doc/chunk counts, embedding model
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `MECH_CRATE_ROOT` | MechCrate installation directory | Auto-detected |
| `MX_RAG_DATABASE_URL` | Neon (primary) corpus URL | From `rag.toml`, else local fallback |
| `MX_RAG_FALLBACK_DATABASE_URL` | Local Postgres fallback URL | `postgres://postgres@localhost:5432/mx_rag` |
| `OPENAI_API_KEY` | Embedding API key (or `MX_RAG_EMBEDDING_API_KEY`) | — |
| `MX_RAG_EMBEDDING_MODEL` | Embedding model | `text-embedding-3-small` |
| `RUST_LOG` | Log level | `info` |

Corpus config is read from `~/.mech-crate/config/rag.toml`; the env vars above override it.

## Troubleshooting

### Corpus Offline

```bash
# Show the active backend (neon / local / offline) and counts
mx rag status

# Start a local pgvector instance if neither Neon nor local is reachable
docker run -d --name mx-rag -p 5432:5432 \
  -e POSTGRES_DB=mx_rag -e POSTGRES_HOST_AUTH_METHOD=trust pgvector/pgvector:pg17

# Or set the primary URL
export MX_RAG_DATABASE_URL=postgres://...neon.tech/mx_rag
```

The MCP server never crashes when the corpus is unreachable — the `rag_*` tools
return an actionable offline message instead.

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

Confirm the corpus is populated, then re-ingest if needed:

```bash
mx rag status
mx rag ingest --clear
```

If `rag_health` reports search running in trigram-only mode, no embedding API key
is configured — set `OPENAI_API_KEY` and run `mx rag ingest --reembed`.

## License

MIT

---

🦝 Built with MechCrate
