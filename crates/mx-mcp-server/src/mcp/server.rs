//! MCP Server Implementation
//!
//! Main server logic for handling MCP requests and dispatching to tools.

use serde_json::json;
use std::path::PathBuf;
use tracing::{debug, error, info};

use crate::error::{error_codes, McpError, McpResult};
use crate::mcp::protocol::*;
use crate::mcp::transport::StdioTransport;
use crate::mx::MxExecutor;
use crate::project::ProjectDetector;
use crate::rag::WeaviateClient;
use crate::tools::ToolRegistry;

/// MCP Server
pub struct McpServer {
    mech_crate_root: PathBuf,
    weaviate_url: String,
    tools: ToolRegistry,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(weaviate_url: String, mech_crate_root: Option<String>) -> McpResult<Self> {
        let root = match mech_crate_root {
            Some(path) => PathBuf::from(path),
            None => Self::detect_mech_crate_root()?,
        };

        info!("MechCrate root: {:?}", root);

        let tools = ToolRegistry::new();

        Ok(Self {
            mech_crate_root: root,
            weaviate_url,
            tools,
        })
    }

    /// Detect MechCrate root by looking for bin/mx
    fn detect_mech_crate_root() -> McpResult<PathBuf> {
        // Try common locations
        let candidates = [
            std::env::current_dir().ok(),
            std::env::var("HOME").ok().map(|h| PathBuf::from(h).join("dev/mech-crate")),
            std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".mech-crate")),
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
        let weaviate = WeaviateClient::new(&self.weaviate_url);

        info!("MCP Server ready, waiting for requests...");

        while let Some(request) = request_rx.recv().await {
            debug!("Handling request: {} (id: {:?})", request.method, request.id);

            let response = match request.method.as_str() {
                "initialize" => self.handle_initialize(&request),
                "initialized" => {
                    // Notification, no response needed
                    continue;
                }
                "tools/list" => self.handle_tools_list(&request),
                "tools/call" => {
                    self.handle_tool_call(&request, &mx, &project_detector, &weaviate)
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

        JsonRpcResponse::success(
            request.id.clone(),
            serde_json::to_value(result).unwrap(),
        )
    }

    /// Handle tools/list request
    fn handle_tools_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let result = ToolsListResult {
            tools: self.tools.list_all(),
        };

        JsonRpcResponse::success(
            request.id.clone(),
            serde_json::to_value(result).unwrap(),
        )
    }

    /// Handle tools/call request
    async fn handle_tool_call(
        &self,
        request: &JsonRpcRequest,
        mx: &MxExecutor,
        project_detector: &ProjectDetector,
        weaviate: &WeaviateClient,
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
            .execute(&params.name, params.arguments, mx, project_detector, weaviate)
            .await;

        match result {
            Ok(call_result) => {
                JsonRpcResponse::success(request.id.clone(), serde_json::to_value(call_result).unwrap())
            }
            Err(e) => {
                let error_result = ToolCallResult::error(e.to_string());
                JsonRpcResponse::success(request.id.clone(), serde_json::to_value(error_result).unwrap())
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
| nuxt | Nuxt 3 SSR/SSG with Tailwind + DaisyUI |
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
"#.to_string()
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
| `mx cf setup` | Cloudflare setup wizard |
| `mx cf init <app>` | Initialize a Cloudflare worker |
| `mx cf deploy <app>` | Deploy to Cloudflare |

## Build Options

| Option | Description |
|--------|-------------|
| `--prod` | Build production-optimized image |
| `-t <tag>` | Custom image tag |
| `--push` | Push image after build |
"#.to_string()
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
"#.to_string()
    }
}
