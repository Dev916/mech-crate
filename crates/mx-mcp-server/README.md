# MechCrate MCP Server

A Model Context Protocol (MCP) server that enables LLMs to interact with MechCrate projects, providing full operational capabilities for project management, service orchestration, infrastructure configuration, and intelligent development-techniques retrieval.

## Features

- **Full MX Command Access**: Create projects, add services, manage router, configure infrastructure
- **Project Makefile Operations**: dev, up, down, logs, shell, restart, build, and more
- **Project Analysis**: Detect projects, list services, inspect configuration
- **Techniques Corpus (RAG)**: Semantic + lexical hybrid search over the development-techniques corpus, backed by Postgres + pgvector
- **Comprehensive Tool Descriptions**: Detailed documentation for LLM understanding
- **Graceful Degradation**: If the corpus is unreachable, RAG tools report an actionable offline message; the server never crashes

## Quick Start

### 1. Build the Server

```bash
# From mech-crate root:
mx mcp build

# Or manually:
cd crates/mx-mcp-server && cargo build --release
```

### 2. Provide a Corpus Backend & Ingest Documentation

The corpus uses Postgres + pgvector. Point it at a Neon database (`database_url`)
or a local Postgres (`fallback_database_url`) via `~/.mech-crate/config/rag.toml`,
or start a local pgvector container:

```bash
docker run -d --name mx-rag -p 5432:5432 \
  -e POSTGRES_DB=mx_rag -e POSTGRES_HOST_AUTH_METHOD=trust \
  pgvector/pgvector:pg17

# Ingest documentation into the corpus
mx rag ingest
```

Embeddings use an OpenAI-compatible `/embeddings` endpoint (default model
`text-embedding-3-small`); set `OPENAI_API_KEY` (or `MX_RAG_EMBEDDING_API_KEY`).
Without an embedding key, search degrades to trigram-only.

### 3. Get Client Configuration

```bash
mx mcp config
```

### 4. Configure MCP Client

The `mx mcp config` command outputs the configuration. Example for Claude Desktop (`~/.claude/claude_desktop_config.json`):

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

The server connects to the corpus on startup (Neon primary → local fallback) and
logs which backend is active. Pass `--no-rag` to skip the corpus entirely.

## Corpus Backend

The techniques corpus is a Postgres + pgvector store. On startup the server tries
the primary `database_url` (Neon) with a short connect timeout, then falls back to
`fallback_database_url` (local Postgres), and logs which backend is active.

Configuration lives in `~/.mech-crate/config/rag.toml`, env-overridable:

```toml
database_url = "postgres://...neon.tech/mx_rag"            # primary (Neon)
fallback_database_url = "postgres://postgres@localhost:5432/mx_rag"  # local
embedding_base_url = "https://api.openai.com/v1"
embedding_model = "text-embedding-3-small"
# api key via OPENAI_API_KEY env (or embedding_api_key)
```

If neither backend is reachable, RAG tools return an actionable offline message
and `rag_health` reports `offline` — the server keeps running.

## Available Tools

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

### Techniques Corpus (8 tools)

| Tool | Description |
|------|-------------|
| `rag_context` | **(primary)** Get techniques relevant to what you are working on right now |
| `rag_search` | Semantic + lexical hybrid search over the techniques corpus |
| `rag_search_category` | Search within one category (theory, patterns, concurrency, database, etc.) |
| `rag_find_implementation` | Find code-bearing technique content for a pattern in a given language |
| `rag_get_guidance` | Get architecture/design guidance for a problem, optionally with constraints |
| `rag_compare_approaches` | Compare two or more approaches/technologies |
| `rag_find_related` | Find techniques related to a topic, excluding the topic's own doc |
| `rag_health` | Corpus backend (neon/local/offline), doc/chunk counts, embedding model, last ingest |

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
│  │                    Tool Registry                             ││
│  │  Comprehensive LLM descriptions for intelligent tool use    ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │           Techniques Corpus (8 rag_* query modes)            ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────┬───────────────────────────────────┘
                              │ SQL (sqlx)
┌─────────────────────────────▼───────────────────────────────────┐
│                   Postgres + pgvector                            │
│  ┌─────────────────┐  ┌─────────────────────────────────────┐  │
│  │ technique_docs  │  │ technique_chunks                    │  │
│  │ (metadata)      │  │ (hnsw cosine + pg_trgm, embeddings) │  │
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

### Get Techniques for the Current Task

```
User: I'm designing a retry/backoff strategy for an async Rust job queue.

LLM uses: rag_context(working_on="designing a retry/backoff strategy for an async Rust job queue", language="rust")
```

### Find Code Examples

```
User: Show me an implementation of lens/prism optics in TypeScript.

LLM uses: rag_find_implementation(pattern="lens/prism optics", language="typescript")
```

### Get Architecture Guidance

```
User: I'm choosing between embedding and referencing for MongoDB order documents.

LLM uses: 
1. rag_compare_approaches(approaches=["embedding documents", "referencing documents"], criteria=["query patterns", "consistency"])
2. rag_get_guidance(problem="embedding vs referencing for MongoDB order documents", constraints=["read-heavy", "orders change rarely"])
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

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `MX_RAG_DATABASE_URL` | Primary corpus database (Neon) | From `rag.toml` |
| `MX_RAG_FALLBACK_DATABASE_URL` | Local Postgres fallback | `postgres://postgres@localhost:5432/mx_rag` |
| `MX_RAG_EMBEDDING_BASE_URL` | OpenAI-compatible embeddings endpoint | `https://api.openai.com/v1` |
| `MX_RAG_EMBEDDING_MODEL` | Embedding model | `text-embedding-3-small` |
| `MX_RAG_EMBEDDING_API_KEY` / `OPENAI_API_KEY` | Embedding API key | (none) |
| `MECH_CRATE_ROOT` | MechCrate installation directory | Auto-detected |
| `RUST_LOG` | Log level | `info` |

## Troubleshooting

### Corpus Offline

`rag_health` reports `offline` and RAG tools return an offline message when
neither backend is reachable. Check your `~/.mech-crate/config/rag.toml`, or start
a local pgvector container and re-ingest:

```bash
docker run -d --name mx-rag -p 5432:5432 \
  -e POSTGRES_DB=mx_rag -e POSTGRES_HOST_AUTH_METHOD=trust \
  pgvector/pgvector:pg17
mx rag ingest
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

Confirm the corpus is populated and re-ingest if needed:

```bash
mx rag status
mx rag ingest
```

## License

MIT

---

🦝 Built with MechCrate
