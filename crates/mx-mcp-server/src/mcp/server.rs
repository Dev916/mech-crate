//! MCP Server Implementation
//!
//! Main server logic for handling MCP requests and dispatching to tools.

use serde_json::json;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use crate::error::{error_codes, McpError, McpResult};
use crate::mcp::protocol::*;
use crate::mcp::transport::StdioTransport;
use crate::mx::MxExecutor;
use crate::project::ProjectDetector;
use crate::tools::ToolRegistry;
use mx_lib::corpus::{CorpusStore, RagConfig};

/// How long the server waits after a failed corpus connect before a RAG tool
/// call may trigger another attempt. Long enough not to hammer a down database
/// on every call, short enough that a Neon cold start or a laptop waking from
/// sleep heals within one working minute.
const CORPUS_RETRY_COOLDOWN: Duration = Duration::from_secs(30);

/// The server's handle on the techniques corpus.
///
/// The connection used to be made exactly once at startup; a failure there
/// (Neon compute still waking, VPN not yet up, laptop offline) left every
/// `rag_*` tool answering "corpus is offline" for the rest of the process,
/// which in a long-lived MCP session means for the rest of the day. This
/// wrapper remembers the failure and lets the next RAG tool call retry once
/// the cooldown has passed.
pub(crate) struct CorpusLink {
    store: Option<CorpusStore>,
    disabled: bool,
    last_attempt: Option<Instant>,
    cooldown: Duration,
}

impl CorpusLink {
    pub(crate) fn new(disabled: bool) -> Self {
        Self {
            store: None,
            disabled,
            last_attempt: None,
            cooldown: CORPUS_RETRY_COOLDOWN,
        }
    }

    #[cfg(test)]
    fn with_cooldown(mut self, cooldown: Duration) -> Self {
        self.cooldown = cooldown;
        self
    }

    pub(crate) fn store(&self) -> Option<&CorpusStore> {
        self.store.as_ref()
    }

    /// Whether a connect attempt is due at `now`: never when RAG is disabled or
    /// already connected; immediately when none was ever made; otherwise only
    /// once the cooldown since the last failure has elapsed.
    pub(crate) fn should_retry(&self, now: Instant) -> bool {
        if self.disabled || self.store.is_some() {
            return false;
        }
        match self.last_attempt {
            None => true,
            Some(at) => now.duration_since(at) >= self.cooldown,
        }
    }

    /// Connect if an attempt is due. Returns whether the corpus is usable afterwards.
    pub(crate) async fn ensure(&mut self, cfg: &RagConfig) -> bool {
        let now = Instant::now();
        if !self.should_retry(now) {
            return self.store.is_some();
        }
        self.last_attempt = Some(now);
        match CorpusStore::connect(cfg).await {
            Ok(s) => {
                info!(
                    "Techniques corpus connected ({} backend)",
                    s.backend().label()
                );
                self.store = Some(s);
                true
            }
            Err(e) => {
                warn!(
                    "Techniques corpus unavailable: {e}. RAG tools report offline; the next RAG call after {}s retries.",
                    self.cooldown.as_secs()
                );
                false
            }
        }
    }
}

/// MCP Server
pub struct McpServer {
    mech_crate_root: PathBuf,
    no_rag: bool,
    tools: ToolRegistry,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(mech_crate_root: Option<String>, no_rag: bool) -> McpResult<Self> {
        let root = match mech_crate_root {
            Some(path) => PathBuf::from(path),
            None => Self::detect_mech_crate_root()?,
        };

        info!("MechCrate root: {:?}", root);

        Ok(Self {
            mech_crate_root: root,
            no_rag,
            tools: ToolRegistry::new(),
        })
    }

    /// Detect MechCrate root by looking for bin/mx
    fn detect_mech_crate_root() -> McpResult<PathBuf> {
        // Try common locations
        let candidates = [
            std::env::current_dir().ok(),
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join("dev/mech-crate")),
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".mech-crate")),
        ];

        for candidate in candidates.into_iter().flatten() {
            if candidate.join("bin/mx").exists() {
                return Ok(candidate);
            }
            // Also check parent directories
            let mut path = candidate.clone();
            for _ in 0..5 {
                if path.join("bin/mx").exists() {
                    return Ok(path);
                }
                if !path.pop() {
                    break;
                }
            }
        }

        Err(McpError::MechCrateRootNotFound)
    }

    /// Run the MCP server
    pub async fn run(self) -> McpResult<()> {
        let (transport, mut request_rx) = StdioTransport::new();

        let mx = MxExecutor::new(self.mech_crate_root.clone());
        let project_detector = ProjectDetector::new();

        let rag_config = RagConfig::load();
        let mut corpus = CorpusLink::new(self.no_rag);
        if self.no_rag {
            info!("RAG disabled (--no-rag)");
        } else {
            corpus.ensure(&rag_config).await;
        }

        info!("MCP Server ready, waiting for requests...");

        while let Some(request) = request_rx.recv().await {
            debug!(
                "Handling request: {} (id: {:?})",
                request.method, request.id
            );

            let response = match request.method.as_str() {
                "initialize" => self.handle_initialize(&request),
                "initialized" => {
                    // Notification, no response needed
                    continue;
                }
                "tools/list" => self.handle_tools_list(&request),
                "tools/call" => {
                    // A RAG tool arriving while the corpus is down is the moment to
                    // retry the connection (subject to the cooldown), so a database
                    // that came up after startup is picked up without a restart.
                    if Self::is_rag_tool_call(&request) {
                        corpus.ensure(&rag_config).await;
                    }
                    self.handle_tool_call(&request, &mx, &project_detector, corpus.store())
                        .await
                }
                "resources/list" => self.handle_resources_list(&request),
                "resources/read" => self.handle_resource_read(&request).await,
                "shutdown" => {
                    info!("Shutdown requested");
                    let response = JsonRpcResponse::success(request.id, json!({}));
                    transport.send(response).await?;
                    break;
                }
                _ => {
                    error!("Unknown method: {}", request.method);
                    JsonRpcResponse::error(
                        request.id,
                        error_codes::METHOD_NOT_FOUND,
                        format!("Method not found: {}", request.method),
                    )
                }
            };

            transport.send(response).await?;
        }

        info!("MCP Server shutting down");
        transport.close().await;
        Ok(())
    }

    /// Handle initialize request
    fn handle_initialize(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                resources: Some(ResourcesCapability {
                    subscribe: Some(false),
                    list_changed: Some(false),
                }),
                prompts: None,
            },
            server_info: ServerInfo {
                name: "mx-mcp-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        JsonRpcResponse::success(request.id.clone(), serde_json::to_value(result).unwrap())
    }

    /// Handle tools/list request
    fn handle_tools_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let result = ToolsListResult {
            tools: self.tools.list_all(),
        };

        JsonRpcResponse::success(request.id.clone(), serde_json::to_value(result).unwrap())
    }

    /// Whether a `tools/call` request names a techniques-corpus (`rag_*`) tool.
    fn is_rag_tool_call(request: &JsonRpcRequest) -> bool {
        request
            .params
            .as_ref()
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .is_some_and(|n| n.starts_with("rag_"))
    }

    /// Handle tools/call request
    async fn handle_tool_call(
        &self,
        request: &JsonRpcRequest,
        mx: &MxExecutor,
        project_detector: &ProjectDetector,
        corpus: Option<&CorpusStore>,
    ) -> JsonRpcResponse {
        let params: ToolCallParams = match request.params.as_ref() {
            Some(p) => match serde_json::from_value(p.clone()) {
                Ok(p) => p,
                Err(e) => {
                    return JsonRpcResponse::error(
                        request.id.clone(),
                        error_codes::INVALID_PARAMS,
                        format!("Invalid parameters: {}", e),
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(
                    request.id.clone(),
                    error_codes::INVALID_PARAMS,
                    "Missing parameters".to_string(),
                );
            }
        };

        let result = self
            .tools
            .execute(&params.name, params.arguments, mx, project_detector, corpus)
            .await;

        match result {
            Ok(call_result) => JsonRpcResponse::success(
                request.id.clone(),
                serde_json::to_value(call_result).unwrap(),
            ),
            Err(e) => {
                let error_result = ToolCallResult::error(e.to_string());
                JsonRpcResponse::success(
                    request.id.clone(),
                    serde_json::to_value(error_result).unwrap(),
                )
            }
        }
    }

    /// Handle resources/list request
    fn handle_resources_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let resources = vec![
            Resource {
                uri: "mx://docs/recipes".to_string(),
                name: "Available Recipes".to_string(),
                description: Some("List of all available MechCrate recipes".to_string()),
                mime_type: Some("text/plain".to_string()),
            },
            Resource {
                uri: "mx://docs/commands".to_string(),
                name: "MX Commands".to_string(),
                description: Some("Complete list of mx command operations".to_string()),
                mime_type: Some("text/plain".to_string()),
            },
            Resource {
                uri: "mx://docs/project-structure".to_string(),
                name: "Project Structure".to_string(),
                description: Some("MechCrate project directory structure".to_string()),
                mime_type: Some("text/plain".to_string()),
            },
        ];

        let result = ResourcesListResult { resources };
        JsonRpcResponse::success(request.id.clone(), serde_json::to_value(result).unwrap())
    }

    /// Handle resources/read request
    async fn handle_resource_read(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let uri = request
            .params
            .as_ref()
            .and_then(|p| p.get("uri"))
            .and_then(|u| u.as_str())
            .unwrap_or("");

        let content = match uri {
            "mx://docs/recipes" => self.get_recipes_docs(),
            "mx://docs/commands" => self.get_commands_docs(),
            "mx://docs/project-structure" => self.get_structure_docs(),
            _ => format!("Unknown resource: {}", uri),
        };

        let result = json!({
            "contents": [{
                "uri": uri,
                "mimeType": "text/plain",
                "text": content
            }]
        });

        JsonRpcResponse::success(request.id.clone(), result)
    }

    fn get_recipes_docs(&self) -> String {
        r#"# MechCrate Recipes

Recipes are pre-packaged stack components that include all configuration,
Docker setup, and Traefik routing.

## Available Recipes

| Recipe | Description |
|--------|-------------|
| laravel | PHP Laravel with Nginx, PHP-FPM, queues, Vite |
| nuxt | Nuxt app scaffolded by nuxi init, Nitro server |
| astro | Astro static site with Vue components |
| rust-api | Rust Axum API with hexagonal architecture |
| rust-leptos | Rust Leptos full-stack SSR app |
| rust-worker | Rust Cloudflare Worker |
| zola | Static site generator with Sass |

## Usage

```bash
# Add a service using a recipe
mx add myservice --recipe=laravel

# With custom domain
mx add api --recipe=rust-api --domain=api.example.com
```

## Recipe Structure

Each recipe provides:
- Application source code template
- Docker configuration (Dockerfile, compose files)
- System files (supervisor, nginx/haproxy configs)
- Environment templates
- Traefik routing labels
"#
        .to_string()
    }

    fn get_commands_docs(&self) -> String {
        r#"# MechCrate Commands

## Global Commands (run anywhere)

| Command | Description |
|---------|-------------|
| `mx new <name>` | Create a new MechCrate project |
| `mx router install` | Install the global Traefik router |
| `mx router up` | Start the global router |
| `mx router down` | Stop the global router |
| `mx router inspect` | Show router status and dashboard URL |
| `mx infra setup` | Configure global infrastructure credentials |
| `mx recipes` | List available recipes |
| `mx doctor` | Check dependencies and system health |

## Project Commands (run from project root)

| Command | Description |
|---------|-------------|
| `mx add <name>` | Add a new service to the project |
| `mx add <name> --recipe=<type>` | Add service using a recipe |
| `mx upgrade` | Update project with latest scaffolding |
| `mx dev [s=service]` | Start services in development mode |
| `mx up [s=service]` | Start services in production mode |
| `mx down [s=service]` | Stop services |
| `mx logs [s=service]` | Tail service logs |
| `mx sh s=<service>` | Shell into a running service |
| `mx build <service>` | Build a service image |
| `mx restart s=<service>` | Restart a service |
| `mx ps` | List running services |

## Infrastructure Commands

| Command | Description |
|---------|-------------|
| `mx infra setup [provider]` | Configure provider credentials |
| `mx infra list` | List configured providers |
| `mx infra link <provider>` | Link project to global config |
| `mx cf setup` | Cloudflare setup wizard (project-local credentials) |
| `mx cf vars` | Show resolved Cloudflare credentials and their source (alias: `mx cf config`) |
| `mx cf check-credentials` | Fail early when no Cloudflare account id resolves |
| `mx cf init <app> [--type=worker\|cron\|container]` | Initialize a Cloudflare app |
| `mx cf deploy <app>` | Deploy an app to Cloudflare production |

`mx cf <cmd>` runs the project's `make cf-<cmd>` target (an app becomes `a=<app>`); `mx cf --help` lists every one.

## Build Options

| Option | Description |
|--------|-------------|
| `--prod` | Build production-optimized image |
| `-t <tag>` | Custom image tag |
| `--push` | Push image after build |
"#
        .to_string()
    }

    fn get_structure_docs(&self) -> String {
        r#"# MechCrate Project Structure

```
project-name/
├── Makefile              # Root makefile
├── apps/                 # Application source code
│   └── <service>/        # Each service's source
│       ├── src/          # Source code
│       ├── package.json  # Dependencies
│       └── ...
├── make/                 # Make modules
│   ├── common.mk         # Shared helpers
│   ├── dev.mk            # Development commands
│   ├── up.mk             # Service management
│   └── ...
├── scripts/              # Shell scripts
│   ├── .bashrc           # Helper functions
│   ├── dev.sh            # Development script
│   └── ...
├── docker/
│   ├── .config/          # Environment files
│   │   ├── .env.shared   # Shared config
│   │   ├── .env.secrets  # Secrets (gitignored)
│   │   └── .env.<svc>    # Per-service config
│   ├── compose/          # Compose files
│   │   ├── <svc>.yml     # Service baseline
│   │   └── <svc>.dev.yml # Dev overrides
│   ├── system/           # System-level files
│   │   └── <service>/    # Maps to container /
│   │       ├── etc/      # Config files (supervisor, nginx)
│   │       └── usr/      # Scripts (entrypoint)
│   └── dockerfiles/      # Dockerfiles
│       └── <service>/
│           └── app       # Dockerfile
└── infra/                # Infrastructure (optional)
    └── cloudflare/       # Cloudflare workers
```

## Key Conventions

1. **Build Context**: Dockerfiles are built from project root
2. **System Files**: `docker/system/<svc>/` mirrors container filesystem
3. **Networking**: Services join `devmesh-traefik` network for routing
4. **Labels**: Traefik labels define hostname routing rules
"#
        .to_string()
    }
}

#[cfg(test)]
mod corpus_link_tests {
    use super::*;

    #[test]
    fn disabled_link_never_retries() {
        let link = CorpusLink::new(true);
        assert!(!link.should_retry(Instant::now()));
    }

    #[test]
    fn first_attempt_is_immediate() {
        let link = CorpusLink::new(false);
        assert!(link.should_retry(Instant::now()));
    }

    #[test]
    fn failed_attempt_waits_for_the_cooldown_then_retries() {
        let mut link = CorpusLink::new(false).with_cooldown(Duration::from_secs(30));
        let t0 = Instant::now();
        link.last_attempt = Some(t0); // a failed connect leaves store None
        assert!(!link.should_retry(t0 + Duration::from_secs(1)));
        assert!(!link.should_retry(t0 + Duration::from_secs(29)));
        assert!(link.should_retry(t0 + Duration::from_secs(30)));
    }

    #[test]
    fn rag_tool_calls_are_recognised_by_name() {
        let call = |name: &str| JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(json!({ "name": name, "arguments": {} })),
        };
        assert!(McpServer::is_rag_tool_call(&call("rag_search")));
        assert!(McpServer::is_rag_tool_call(&call("rag_health")));
        assert!(!McpServer::is_rag_tool_call(&call("mx_doctor")));
        let no_params = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(1),
            method: "tools/call".to_string(),
            params: None,
        };
        assert!(!McpServer::is_rag_tool_call(&no_params));
    }
}
