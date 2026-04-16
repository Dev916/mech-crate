//! Tool Registry and Definitions
//!
//! Defines all MCP tools available for LLM interaction with MechCrate.
//! Each tool includes comprehensive documentation for LLM understanding.

use serde_json::{json, Value};
use tracing::{debug, info};

use crate::error::{McpError, McpResult};
use crate::mcp::protocol::{Tool, ToolCallResult, ToolInputSchema};
use crate::mx::{MakeExecutor, MxExecutor};
use crate::project::ProjectDetector;
use crate::rag::{format_search_results, WeaviateClient};
use crate::unyform::UnyformClient;

/// Tool registry containing all available tools
pub struct ToolRegistry {
    tools: Vec<ToolDefinition>,
}

struct ToolDefinition {
    tool: Tool,
    handler: ToolHandler,
}

enum ToolHandler {
    // Global mx commands
    MxNew,
    MxRecipesList,
    MxRecipeInfo,
    MxRouterInstall,
    MxRouterUp,
    MxRouterDown,
    MxRouterStatus,
    MxRouterInspect,
    MxInfraSetup,
    MxInfraList,
    MxInfraLink,
    MxDoctor,
    MxHelp,

    // Project-level mx commands
    MxAddService,
    MxUpgrade,
    MxBuild,

    // Make commands (project-level)
    MakeDev,
    MakeUp,
    MakeDown,
    MakeLogs,
    MakeRestart,
    MakeShell,
    MakePs,
    MakeHelp,
    MakeKey,

    // Project analysis
    ProjectAnalyze,
    ProjectList,
    ProjectDetect,
    ServiceInfo,

    // RAG queries
    RagSearch,
    RagSearchCategory,
    RagFindImplementation,
    RagGetGuidance,
    RagCompareApproaches,
    RagFindRelated,
    RagHealth,

    // Document compilation
    MxDocsCompile,
    MxDocsList,

    // Unyform integration
    UnyformLogin,
    UnyformLogout,
    UnyformWhoami,
    UnyformRecipesList,
    UnyformRecipesPull,
    UnyformRecipesApply,
    UnyformRecipesVersions,
    UnyformRecipesCache,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let tools = Self::define_all_tools();
        Self { tools }
    }

    /// Define all tools with comprehensive descriptions
    fn define_all_tools() -> Vec<ToolDefinition> {
        vec![
            // ─────────────────────────────────────────────────────────────────
            // Global MX Commands
            // ─────────────────────────────────────────────────────────────────
            ToolDefinition {
                tool: Tool {
                    name: "mx_new".to_string(),
                    description: r#"Create a new MechCrate project.

This command scaffolds a complete project structure with:
- Docker configuration (compose files, Dockerfiles, system files)
- Make modules for common operations (dev, up, down, logs, sh, etc.)
- Scripts for project automation
- Optional infrastructure setup (Cloudflare, AWS, etc.)

The project follows MechCrate conventions:
- apps/ contains service source code
- docker/ contains all Docker-related files
- make/ contains Make modules
- scripts/ contains shell scripts

IMPORTANT: This command should be run in the parent directory where you want the project created.
After creation, cd into the project directory.

Example workflow:
1. Run mx_new to create project
2. cd into the project
3. Run mx_add_service to add services
4. Run make_dev to start development"#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "name": {
                                "type": "string",
                                "description": "Project name (will be used as directory name)"
                            },
                            "with_infra": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Infrastructure providers to include: cloudflare, aws, digitalocean, hetzner"
                            },
                            "working_directory": {
                                "type": "string",
                                "description": "Directory where the project should be created (defaults to current directory)"
                            }
                        })),
                        required: Some(vec!["name".to_string()]),
                    },
                },
                handler: ToolHandler::MxNew,
            },

            ToolDefinition {
                tool: Tool {
                    name: "mx_recipes_list".to_string(),
                    description: r#"List all available MechCrate recipes.

Recipes are pre-packaged stack components that include:
- Application source code template
- Docker configuration (multi-stage Dockerfile)
- System files (supervisor, nginx/haproxy configs)
- Compose files (production and development)
- Environment templates
- Traefik routing labels

Available recipes typically include:
- laravel: PHP Laravel with Nginx, PHP-FPM, queues
- nuxt: Nuxt 3 SSR/SSG with Tailwind + DaisyUI
- astro: Astro static sites with Vue components
- rust-api: Rust Axum API with hexagonal architecture
- rust-leptos: Rust Leptos full-stack SSR
- rust-worker: Cloudflare Worker in Rust
- zola: Static site generator

Use mx_recipe_info for detailed information about a specific recipe."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: None,
                        required: None,
                    },
                },
                handler: ToolHandler::MxRecipesList,
            },

            ToolDefinition {
                tool: Tool {
                    name: "mx_recipe_info".to_string(),
                    description: r#"Get detailed information about a specific recipe.

Returns:
- Recipe description and features
- Available options (domain, port, etc.)
- Services created by the recipe
- Usage examples"#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "recipe": {
                                "type": "string",
                                "description": "Recipe name (e.g., laravel, nuxt, rust-api)"
                            }
                        })),
                        required: Some(vec!["recipe".to_string()]),
                    },
                },
                handler: ToolHandler::MxRecipeInfo,
            },

            ToolDefinition {
                tool: Tool {
                    name: "mx_router_install".to_string(),
                    description: r#"Install the global Traefik router for MechCrate.

The router enables:
- Running multiple projects simultaneously
- Hostname-based routing (e.g., myapp.localhost, api.localhost)
- No port conflicts between services
- Automatic service discovery via Docker labels

This should be run once per workstation. The router is installed to ~/.mech-crate/router.

After installation, run mx_router_up to start the router."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: None,
                        required: None,
                    },
                },
                handler: ToolHandler::MxRouterInstall,
            },

            ToolDefinition {
                tool: Tool {
                    name: "mx_router_up".to_string(),
                    description: r#"Start or update the global Traefik router.

This starts Traefik on ports 80 and 443, plus a dynamically allocated dashboard port.

After starting, services can be accessed via hostnames like:
- http://myapp.localhost
- http://api.localhost

Use mx_router_inspect to see the dashboard URL and connected services."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: None,
                        required: None,
                    },
                },
                handler: ToolHandler::MxRouterUp,
            },

            ToolDefinition {
                tool: Tool {
                    name: "mx_router_down".to_string(),
                    description: "Stop the global Traefik router.".to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: None,
                        required: None,
                    },
                },
                handler: ToolHandler::MxRouterDown,
            },

            ToolDefinition {
                tool: Tool {
                    name: "mx_router_status".to_string(),
                    description: "Show the status of the global Traefik router containers.".to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: None,
                        required: None,
                    },
                },
                handler: ToolHandler::MxRouterStatus,
            },

            ToolDefinition {
                tool: Tool {
                    name: "mx_router_inspect".to_string(),
                    description: r#"Show detailed router information including:
- State directory location
- Docker network name
- Dashboard URL and port
- Currently connected services

This is useful for debugging routing issues or finding the dashboard URL."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: None,
                        required: None,
                    },
                },
                handler: ToolHandler::MxRouterInspect,
            },

            ToolDefinition {
                tool: Tool {
                    name: "mx_infra_setup".to_string(),
                    description: r#"Configure global infrastructure credentials for a provider.

Supported providers:
- cloudflare: Workers & Containers (account ID, API token)
- digitalocean: Droplets & App Platform (API token, Spaces keys)
- aws: Amazon Web Services (access keys, region)
- hetzner: Hetzner Cloud (API token, location)

Credentials are stored in ~/.mech-crate/config/infra/ and can be shared across projects using mx_infra_link."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "provider": {
                                "type": "string",
                                "description": "Infrastructure provider: cloudflare, digitalocean, aws, hetzner"
                            }
                        })),
                        required: None,
                    },
                },
                handler: ToolHandler::MxInfraSetup,
            },

            ToolDefinition {
                tool: Tool {
                    name: "mx_infra_list".to_string(),
                    description: "List all configured infrastructure providers (global and project-level).".to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: None,
                        required: None,
                    },
                },
                handler: ToolHandler::MxInfraList,
            },

            ToolDefinition {
                tool: Tool {
                    name: "mx_infra_link".to_string(),
                    description: r#"Link a project to global infrastructure credentials.

This allows the project to use globally configured credentials instead of project-specific ones.
Must be run from within a MechCrate project."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "provider": {
                                "type": "string",
                                "description": "Infrastructure provider to link: cloudflare, digitalocean, aws, hetzner"
                            },
                            "project_path": {
                                "type": "string",
                                "description": "Path to the MechCrate project"
                            }
                        })),
                        required: Some(vec!["provider".to_string(), "project_path".to_string()]),
                    },
                },
                handler: ToolHandler::MxInfraLink,
            },

            ToolDefinition {
                tool: Tool {
                    name: "mx_doctor".to_string(),
                    description: r#"Check system health and dependencies.

Verifies:
- Docker installation and status
- Required commands (make, bash, etc.)
- MechCrate installation
- Project structure (if in a project)"#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "project_path": {
                                "type": "string",
                                "description": "Optional: path to a project to check"
                            }
                        })),
                        required: None,
                    },
                },
                handler: ToolHandler::MxDoctor,
            },

            ToolDefinition {
                tool: Tool {
                    name: "mx_help".to_string(),
                    description: "Show complete mx command help and usage information.".to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: None,
                        required: None,
                    },
                },
                handler: ToolHandler::MxHelp,
            },

            // ─────────────────────────────────────────────────────────────────
            // Project-Level MX Commands
            // ─────────────────────────────────────────────────────────────────

            ToolDefinition {
                tool: Tool {
                    name: "mx_add_service".to_string(),
                    description: r#"Add a new service to an existing MechCrate project.

This creates:
- Source code directory in apps/<name>/
- Docker compose files in docker/compose/
- Dockerfile in docker/dockerfiles/<name>/
- System files in docker/system/<name>/
- Environment config in docker/.config/

When using a recipe (recommended), all configuration is pre-populated with best practices including:
- Multi-stage Dockerfile with dev/prod targets
- Supervisor configuration for process management
- Nginx or HAProxy for internal reverse proxy
- Traefik labels for routing

IMPORTANT: Must be run from within a MechCrate project."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "name": {
                                "type": "string",
                                "description": "Service name (used for directory names, compose service names, etc.)"
                            },
                            "recipe": {
                                "type": "string",
                                "description": "Recipe to use: laravel, nuxt, rust-api, rust-leptos, astro, zola, etc."
                            },
                            "domain": {
                                "type": "string",
                                "description": "Custom domain for Traefik routing (defaults to <name>.localhost)"
                            },
                            "project_path": {
                                "type": "string",
                                "description": "Path to the MechCrate project"
                            }
                        })),
                        required: Some(vec!["name".to_string(), "project_path".to_string()]),
                    },
                },
                handler: ToolHandler::MxAddService,
            },

            ToolDefinition {
                tool: Tool {
                    name: "mx_upgrade".to_string(),
                    description: r#"Update a project with the latest MechCrate scaffolding.

This updates:
- Make modules
- Scripts
- Docker templates

Does NOT modify:
- Application source code
- Project-specific configuration
- Environment files

Use --diff to preview changes before applying."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "project_path": {
                                "type": "string",
                                "description": "Path to the MechCrate project"
                            },
                            "diff": {
                                "type": "boolean",
                                "description": "Show diffs before updating"
                            },
                            "yes": {
                                "type": "boolean",
                                "description": "Auto-accept all updates"
                            }
                        })),
                        required: Some(vec!["project_path".to_string()]),
                    },
                },
                handler: ToolHandler::MxUpgrade,
            },

            ToolDefinition {
                tool: Tool {
                    name: "mx_build".to_string(),
                    description: r#"Build a Docker image for a service.

Build modes:
- Development (default): Full dev dependencies, debug tools
- Production (--prod): Optimized, minimal image

Options:
- tag: Custom image tag (e.g., v1.0.0)
- push: Push to registry after build"#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "service": {
                                "type": "string",
                                "description": "Service to build"
                            },
                            "project_path": {
                                "type": "string",
                                "description": "Path to the MechCrate project"
                            },
                            "prod": {
                                "type": "boolean",
                                "description": "Build production-optimized image"
                            },
                            "tag": {
                                "type": "string",
                                "description": "Custom image tag"
                            },
                            "push": {
                                "type": "boolean",
                                "description": "Push image after build"
                            }
                        })),
                        required: Some(vec!["service".to_string(), "project_path".to_string()]),
                    },
                },
                handler: ToolHandler::MxBuild,
            },

            // ─────────────────────────────────────────────────────────────────
            // Make Commands (Project-Level Operations)
            // ─────────────────────────────────────────────────────────────────

            ToolDefinition {
                tool: Tool {
                    name: "make_dev".to_string(),
                    description: r#"Start services in development mode.

This runs docker compose with development overrides:
- Source code mounted for hot-reload
- Debug ports exposed
- Development environment variables
- Relaxed health checks

Requires the global router to be running (mx_router_up).
Access services via hostname (e.g., http://myapp.localhost)."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "service": {
                                "type": "string",
                                "description": "Specific service to start (omit for all services)"
                            },
                            "project_path": {
                                "type": "string",
                                "description": "Path to the MechCrate project"
                            }
                        })),
                        required: Some(vec!["project_path".to_string()]),
                    },
                },
                handler: ToolHandler::MakeDev,
            },

            ToolDefinition {
                tool: Tool {
                    name: "make_up".to_string(),
                    description: r#"Start services in production mode.

Uses production compose configuration:
- Optimized container settings
- No source code mounting
- Production environment variables
- Full health checks enabled"#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "service": {
                                "type": "string",
                                "description": "Specific service to start (omit for all)"
                            },
                            "project_path": {
                                "type": "string",
                                "description": "Path to the MechCrate project"
                            }
                        })),
                        required: Some(vec!["project_path".to_string()]),
                    },
                },
                handler: ToolHandler::MakeUp,
            },

            ToolDefinition {
                tool: Tool {
                    name: "make_down".to_string(),
                    description: "Stop running services.".to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "service": {
                                "type": "string",
                                "description": "Specific service to stop (omit for all)"
                            },
                            "project_path": {
                                "type": "string",
                                "description": "Path to the MechCrate project"
                            }
                        })),
                        required: Some(vec!["project_path".to_string()]),
                    },
                },
                handler: ToolHandler::MakeDown,
            },

            ToolDefinition {
                tool: Tool {
                    name: "make_logs".to_string(),
                    description: r#"View service logs.

Returns recent log output from running containers.
Useful for debugging issues or monitoring service behavior."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "service": {
                                "type": "string",
                                "description": "Specific service (omit for all)"
                            },
                            "project_path": {
                                "type": "string",
                                "description": "Path to the MechCrate project"
                            }
                        })),
                        required: Some(vec!["project_path".to_string()]),
                    },
                },
                handler: ToolHandler::MakeLogs,
            },

            ToolDefinition {
                tool: Tool {
                    name: "make_restart".to_string(),
                    description: "Restart a specific service.".to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "service": {
                                "type": "string",
                                "description": "Service to restart"
                            },
                            "project_path": {
                                "type": "string",
                                "description": "Path to the MechCrate project"
                            }
                        })),
                        required: Some(vec!["service".to_string(), "project_path".to_string()]),
                    },
                },
                handler: ToolHandler::MakeRestart,
            },

            ToolDefinition {
                tool: Tool {
                    name: "make_shell".to_string(),
                    description: r#"Get shell access information for a service.

NOTE: Interactive shell sessions cannot be executed via MCP.
This tool provides the command to run manually."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "service": {
                                "type": "string",
                                "description": "Service to shell into"
                            },
                            "project_path": {
                                "type": "string",
                                "description": "Path to the MechCrate project"
                            }
                        })),
                        required: Some(vec!["service".to_string(), "project_path".to_string()]),
                    },
                },
                handler: ToolHandler::MakeShell,
            },

            ToolDefinition {
                tool: Tool {
                    name: "make_ps".to_string(),
                    description: "List running services in the project.".to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "project_path": {
                                "type": "string",
                                "description": "Path to the MechCrate project"
                            }
                        })),
                        required: Some(vec!["project_path".to_string()]),
                    },
                },
                handler: ToolHandler::MakePs,
            },

            ToolDefinition {
                tool: Tool {
                    name: "make_help".to_string(),
                    description: "Show available make targets for a project.".to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "project_path": {
                                "type": "string",
                                "description": "Path to the MechCrate project"
                            }
                        })),
                        required: Some(vec!["project_path".to_string()]),
                    },
                },
                handler: ToolHandler::MakeHelp,
            },

            ToolDefinition {
                tool: Tool {
                    name: "make_key".to_string(),
                    description: r#"Generate a cryptographically secure secret key.

Formats:
- hex: Hexadecimal string (default)
- base64: Base64 encoded
- uuid: UUID v4

Useful for generating API keys, JWT secrets, encryption keys, etc."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "project_path": {
                                "type": "string",
                                "description": "Path to the MechCrate project"
                            },
                            "bytes": {
                                "type": "integer",
                                "description": "Number of bytes (default: 32)"
                            },
                            "format": {
                                "type": "string",
                                "description": "Output format: hex, base64, uuid"
                            }
                        })),
                        required: Some(vec!["project_path".to_string()]),
                    },
                },
                handler: ToolHandler::MakeKey,
            },

            // ─────────────────────────────────────────────────────────────────
            // Project Analysis
            // ─────────────────────────────────────────────────────────────────

            ToolDefinition {
                tool: Tool {
                    name: "project_analyze".to_string(),
                    description: r#"Analyze a MechCrate project structure.

Returns detailed information about:
- Project name and root directory
- All services and their configuration
- Infrastructure providers configured
- Available compose files
- Make targets available"#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "project_path": {
                                "type": "string",
                                "description": "Path to the MechCrate project"
                            }
                        })),
                        required: Some(vec!["project_path".to_string()]),
                    },
                },
                handler: ToolHandler::ProjectAnalyze,
            },

            ToolDefinition {
                tool: Tool {
                    name: "project_list".to_string(),
                    description: "Find all MechCrate projects in a directory.".to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "search_path": {
                                "type": "string",
                                "description": "Directory to search for projects"
                            }
                        })),
                        required: Some(vec!["search_path".to_string()]),
                    },
                },
                handler: ToolHandler::ProjectList,
            },

            ToolDefinition {
                tool: Tool {
                    name: "project_detect".to_string(),
                    description: r#"Detect if a path is within a MechCrate project.

Walks up the directory tree to find the project root.
Returns the root path if found, or indicates no project was found."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "path": {
                                "type": "string",
                                "description": "Path to check"
                            }
                        })),
                        required: Some(vec!["path".to_string()]),
                    },
                },
                handler: ToolHandler::ProjectDetect,
            },

            ToolDefinition {
                tool: Tool {
                    name: "service_info".to_string(),
                    description: r#"Get detailed information about a specific service.

Returns:
- Whether Dockerfile exists
- Compose file presence
- App directory location
- Configuration status"#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "service": {
                                "type": "string",
                                "description": "Service name"
                            },
                            "project_path": {
                                "type": "string",
                                "description": "Path to the MechCrate project"
                            }
                        })),
                        required: Some(vec!["service".to_string(), "project_path".to_string()]),
                    },
                },
                handler: ToolHandler::ServiceInfo,
            },

            // ─────────────────────────────────────────────────────────────────
            // RAG Documentation Queries
            // ─────────────────────────────────────────────────────────────────

            ToolDefinition {
                tool: Tool {
                    name: "rag_search".to_string(),
                    description: r#"Search MechCrate documentation using semantic search.

Use this to find relevant documentation about:
- How to structure projects
- Recipe configuration
- Docker best practices
- Traefik routing setup
- Infrastructure configuration
- Make target usage

The search uses embeddings to find semantically similar content,
even if the exact words don't match."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "query": {
                                "type": "string",
                                "description": "Search query (natural language)"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of results (default: 5)"
                            }
                        })),
                        required: Some(vec!["query".to_string()]),
                    },
                },
                handler: ToolHandler::RagSearch,
            },

            ToolDefinition {
                tool: Tool {
                    name: "rag_search_category".to_string(),
                    description: r#"Search documentation within a specific category.

Categories:
- recipe: Recipe authoring and configuration
- command: MX command documentation
- structure: Project structure and conventions
- docker: Docker configuration and best practices
- traefik: Routing and networking
- infra: Infrastructure providers"#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "query": {
                                "type": "string",
                                "description": "Search query"
                            },
                            "category": {
                                "type": "string",
                                "description": "Category to search: recipe, command, structure, docker, traefik, infra"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum results (default: 5)"
                            }
                        })),
                        required: Some(vec!["query".to_string(), "category".to_string()]),
                    },
                },
                handler: ToolHandler::RagSearchCategory,
            },

            ToolDefinition {
                tool: Tool {
                    name: "rag_find_implementation".to_string(),
                    description: r#"Find code implementation examples in MechCrate documentation.

Use this to find:
- Code snippets for specific patterns
- Implementation examples for Dockerfiles, compose files, etc.
- Configuration examples (nginx, supervisor, traefik)
- Script examples (bash, make)

Returns results prioritizing content with code blocks."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "pattern": {
                                "type": "string",
                                "description": "The pattern or feature to find implementations for (e.g., 'multi-stage Dockerfile', 'supervisor config')"
                            },
                            "language": {
                                "type": "string",
                                "description": "Optional language filter: dockerfile, yaml, bash, rust, php, etc."
                            }
                        })),
                        required: Some(vec!["pattern".to_string()]),
                    },
                },
                handler: ToolHandler::RagFindImplementation,
            },

            ToolDefinition {
                tool: Tool {
                    name: "rag_get_guidance".to_string(),
                    description: r#"Get architecture and design guidance from MechCrate documentation.

Use this when you need help with:
- Choosing between different approaches
- Understanding MechCrate conventions
- Design decisions for project structure
- Best practices for Docker, compose, routing

The search contextualizes results around the given problem and constraints."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "problem": {
                                "type": "string",
                                "description": "The design problem or decision to get guidance on"
                            },
                            "constraints": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Optional constraints to consider (e.g., 'must use Rust', 'needs hot-reload')"
                            }
                        })),
                        required: Some(vec!["problem".to_string()]),
                    },
                },
                handler: ToolHandler::RagGetGuidance,
            },

            ToolDefinition {
                tool: Tool {
                    name: "rag_compare_approaches".to_string(),
                    description: r#"Compare different approaches or technologies in MechCrate context.

Use this to compare:
- Different recipes (laravel vs nuxt vs rust-api)
- Different infrastructure providers (cloudflare vs aws)
- Different configuration approaches
- Alternative implementation strategies

Returns relevant documentation for each approach for comparison."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "approaches": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "List of approaches/technologies to compare (e.g., ['laravel', 'nuxt'] or ['cloudflare', 'hetzner'])"
                            },
                            "criteria": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Optional criteria to focus comparison on (e.g., ['performance', 'ease of use'])"
                            }
                        })),
                        required: Some(vec!["approaches".to_string()]),
                    },
                },
                handler: ToolHandler::RagCompareApproaches,
            },

            ToolDefinition {
                tool: Tool {
                    name: "rag_find_related".to_string(),
                    description: r#"Find documentation related to a specific topic or document.

Use this for:
- Discovering related concepts
- Finding prerequisites for a topic
- Exploring connected documentation
- Understanding broader context

Useful when you need to expand understanding of a topic."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "topic": {
                                "type": "string",
                                "description": "Topic or document name to find related documentation for"
                            },
                            "max_results": {
                                "type": "integer",
                                "description": "Maximum number of related documents (default: 5)"
                            }
                        })),
                        required: Some(vec!["topic".to_string()]),
                    },
                },
                handler: ToolHandler::RagFindRelated,
            },

            ToolDefinition {
                tool: Tool {
                    name: "rag_health".to_string(),
                    description: "Check if the Weaviate RAG server is available.".to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: None,
                        required: None,
                    },
                },
                handler: ToolHandler::RagHealth,
            },

            // ─────────────────────────────────────────────────────────────────
            // Document Compilation
            // ─────────────────────────────────────────────────────────────────

            ToolDefinition {
                tool: Tool {
                    name: "mx_docs_compile".to_string(),
                    description: r#"Compile Markdown documents to professional PDF and HTML.

Converts markdown files to beautifully formatted PDF and HTML documents with:
- Mermaid diagram rendering (auto-converted to PNG images)
- Syntax-highlighted code blocks
- Table of contents generation
- Cover page with optional logo and company name
- YAML frontmatter support for metadata (title, subtitle, author, date, abstract)

Modes of operation:
1. Single file: Provide 'input' pointing to a .md file
2. Directory: Provide 'input' pointing to a folder (compiles all .md files)
3. Config-based (specific doc): Provide 'config' path and 'doc' name
4. Config-based (all docs): Provide 'config' path and set 'all' to true

The config mode uses a docs.json file that defines document collections:
```json
{
  "name": "My Docs",
  "logo": "path/to/logo.png",
  "companyName": "My Company",
  "outputDir": "artifacts/docs",
  "defaults": { "author": "Team", "theme": "light" },
  "documents": {
    "readme": { "file": "README.md", "title": "Overview" },
    "guide": { "file": "guide.md", "title": "User Guide" }
  }
}
```

Output: Creates a directory with PDF, HTML, processed Markdown, and diagram images.

Requirements: Node.js 18+ (dependencies auto-installed on first run)."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "input": {
                                "type": "string",
                                "description": "Path to a markdown file (.md) or directory containing markdown files"
                            },
                            "output": {
                                "type": "string",
                                "description": "Output directory for generated files (PDF, HTML, Markdown, diagrams)"
                            },
                            "title": {
                                "type": "string",
                                "description": "Document title (overrides frontmatter)"
                            },
                            "subtitle": {
                                "type": "string",
                                "description": "Document subtitle (overrides frontmatter)"
                            },
                            "author": {
                                "type": "string",
                                "description": "Document author (overrides frontmatter)"
                            },
                            "theme": {
                                "type": "string",
                                "description": "Mermaid diagram theme: dark, light, forest, neutral (default: light)",
                                "enum": ["dark", "light", "forest", "neutral"]
                            },
                            "logo": {
                                "type": "string",
                                "description": "Path to logo image for cover page (PNG, JPG, or SVG)"
                            },
                            "company_name": {
                                "type": "string",
                                "description": "Company name to display on the cover page"
                            },
                            "no_logo": {
                                "type": "boolean",
                                "description": "Set to true to disable the logo on the cover page"
                            },
                            "config": {
                                "type": "string",
                                "description": "Path to docs.json config file or directory containing it"
                            },
                            "doc": {
                                "type": "string",
                                "description": "Compile a specific document by key name from docs.json config"
                            },
                            "all": {
                                "type": "boolean",
                                "description": "Set to true to compile all documents defined in docs.json config"
                            },
                            "format": {
                                "type": "string",
                                "description": "Output format: pdf (default, generates all formats), html (HTML only, no PDF), markdown (processed markdown only)",
                                "enum": ["pdf", "html", "markdown"]
                            },
                            "no_toc": {
                                "type": "boolean",
                                "description": "Disable table of contents generation"
                            }
                        })),
                        required: None,
                    },
                },
                handler: ToolHandler::MxDocsCompile,
            },

            ToolDefinition {
                tool: Tool {
                    name: "mx_docs_list".to_string(),
                    description: r#"List available documents from a docs.json configuration.

Shows all documents defined in a docs.json config file, including their key names,
titles, descriptions, and whether the source files exist.

Use this to discover what documents are available before compiling with mx_docs_compile."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "config": {
                                "type": "string",
                                "description": "Path to docs.json config file or directory containing it. If not specified, searches common locations (docs/docs.json, ./docs.json)"
                            }
                        })),
                        required: None,
                    },
                },
                handler: ToolHandler::MxDocsList,
            },

            // ─────────────────────────────────────────────────────────────────
            // Unyform Integration
            // ─────────────────────────────────────────────────────────────────

            ToolDefinition {
                tool: Tool {
                    name: "unyform_login".to_string(),
                    description: r#"Authenticate with Unyform.ai for organizational recipes.

Login methods:
- API key (for CI/automation): Provide api_key parameter
- Browser OAuth: Provide browser=true to get OAuth URL (user must complete flow manually)

After login, credentials are stored in ~/.mech-crate/config/unyform/"#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "api_key": {
                                "type": "string",
                                "description": "API key for authentication (for CI/automation)"
                            },
                            "url": {
                                "type": "string",
                                "description": "Custom Unyform instance URL (defaults to https://api.unyform.ai)"
                            }
                        })),
                        required: None,
                    },
                },
                handler: ToolHandler::UnyformLogin,
            },

            ToolDefinition {
                tool: Tool {
                    name: "unyform_logout".to_string(),
                    description: "Clear Unyform credentials and session.".to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: None,
                        required: None,
                    },
                },
                handler: ToolHandler::UnyformLogout,
            },

            ToolDefinition {
                tool: Tool {
                    name: "unyform_whoami".to_string(),
                    description: r#"Show current Unyform authentication status.

Returns:
- User email and name
- Organizations the user belongs to
- Current API URL"#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: None,
                        required: None,
                    },
                },
                handler: ToolHandler::UnyformWhoami,
            },

            ToolDefinition {
                tool: Tool {
                    name: "unyform_recipes_list".to_string(),
                    description: r#"List organizational recipes from Unyform.

Requires authentication. Returns recipes from the user's default organization."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: None,
                        required: None,
                    },
                },
                handler: ToolHandler::UnyformRecipesList,
            },

            ToolDefinition {
                tool: Tool {
                    name: "unyform_recipes_pull".to_string(),
                    description: r#"Pull an organizational recipe from Unyform to local cache.

Downloads the recipe JSON to ~/.mech-crate/recipes/<org>/<name>/<version>/
Use unyform_recipes_apply to apply the cached recipe to a project."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "name": {
                                "type": "string",
                                "description": "Recipe name"
                            },
                            "version": {
                                "type": "string",
                                "description": "Specific version to pull (defaults to latest)"
                            }
                        })),
                        required: Some(vec!["name".to_string()]),
                    },
                },
                handler: ToolHandler::UnyformRecipesPull,
            },

            ToolDefinition {
                tool: Tool {
                    name: "unyform_recipes_apply".to_string(),
                    description: r#"Apply an organizational recipe to the current project.

This will:
1. Pull the recipe if not cached
2. Create coding rules in .cursor/rules/
3. Compare dependencies and warn of drift
4. Suggest infrastructure configurations

Use --fix flag to auto-update dependencies."#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "name": {
                                "type": "string",
                                "description": "Recipe name"
                            },
                            "version": {
                                "type": "string",
                                "description": "Specific version to apply (defaults to latest)"
                            },
                            "project_path": {
                                "type": "string",
                                "description": "Path to the project to apply recipe to"
                            },
                            "fix": {
                                "type": "boolean",
                                "description": "Auto-fix dependency drift"
                            }
                        })),
                        required: Some(vec!["name".to_string(), "project_path".to_string()]),
                    },
                },
                handler: ToolHandler::UnyformRecipesApply,
            },

            ToolDefinition {
                tool: Tool {
                    name: "unyform_recipes_versions".to_string(),
                    description: "List available versions for an organizational recipe.".to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "name": {
                                "type": "string",
                                "description": "Recipe name"
                            }
                        })),
                        required: Some(vec!["name".to_string()]),
                    },
                },
                handler: ToolHandler::UnyformRecipesVersions,
            },

            ToolDefinition {
                tool: Tool {
                    name: "unyform_recipes_cache".to_string(),
                    description: r#"Manage locally cached organizational recipes.

Actions:
- list (default): Show all cached recipes
- clear: Remove all cached recipes"#.to_string(),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: Some(json!({
                            "action": {
                                "type": "string",
                                "description": "Action to perform: list, clear (default: list)"
                            }
                        })),
                        required: None,
                    },
                },
                handler: ToolHandler::UnyformRecipesCache,
            },
        ]
    }

    /// List all tools
    pub fn list_all(&self) -> Vec<Tool> {
        self.tools.iter().map(|t| t.tool.clone()).collect()
    }

    /// Execute a tool
    pub async fn execute(
        &self,
        name: &str,
        args: Value,
        mx: &MxExecutor,
        project_detector: &ProjectDetector,
        weaviate: &WeaviateClient,
    ) -> McpResult<ToolCallResult> {
        let tool_def = self
            .tools
            .iter()
            .find(|t| t.tool.name == name)
            .ok_or_else(|| McpError::ToolNotFound(name.to_string()))?;

        info!("Executing tool: {}", name);
        debug!("Arguments: {}", args);

        match &tool_def.handler {
            ToolHandler::MxNew => {
                let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'name' is required".to_string())
                })?;
                let _working_dir = args.get("working_directory").and_then(|v| v.as_str());
                let with_infra: Option<Vec<&str>> = args
                    .get("with_infra")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect());

                let output = mx.new_project(
                    name,
                    None,
                    with_infra.as_deref(),
                    true, // no_prompt for MCP
                ).await?;

                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxRecipesList => {
                let output = mx.list_recipes().await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxRecipeInfo => {
                let recipe = args.get("recipe").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'recipe' is required".to_string())
                })?;
                let output = mx.recipe_info(recipe).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxRouterInstall => {
                let output = mx.router("install").await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxRouterUp => {
                let output = mx.router("up").await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxRouterDown => {
                let output = mx.router("down").await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxRouterStatus => {
                let output = mx.router("status").await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxRouterInspect => {
                let output = mx.router("inspect").await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxInfraSetup => {
                let provider = args.get("provider").and_then(|v| v.as_str());
                let output = mx.infra("setup", provider).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxInfraList => {
                let output = mx.infra("list", None).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxInfraLink => {
                let provider = args.get("provider").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'provider' is required".to_string())
                })?;
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;
                let output = mx.execute(&["infra", "link", provider], Some(project_path.as_ref())).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxDoctor => {
                let project_path = args.get("project_path").and_then(|v| v.as_str());
                let output = mx.doctor(project_path.map(|s| s.as_ref())).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxHelp => {
                let output = mx.help().await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxAddService => {
                let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'name' is required".to_string())
                })?;
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;
                let recipe = args.get("recipe").and_then(|v| v.as_str());
                let domain = args.get("domain").and_then(|v| v.as_str());

                let output = mx.add_service(name, recipe, domain, project_path.as_ref()).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxUpgrade => {
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;
                let diff = args.get("diff").and_then(|v| v.as_bool()).unwrap_or(false);
                let yes = args.get("yes").and_then(|v| v.as_bool()).unwrap_or(true);

                let output = mx.upgrade(project_path.as_ref(), diff, yes).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxBuild => {
                let service = args.get("service").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'service' is required".to_string())
                })?;
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;
                let prod = args.get("prod").and_then(|v| v.as_bool()).unwrap_or(false);
                let tag = args.get("tag").and_then(|v| v.as_str());
                let push = args.get("push").and_then(|v| v.as_bool()).unwrap_or(false);

                let output = mx.build(service, prod, tag, push, project_path.as_ref()).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MakeDev => {
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;
                let service = args.get("service").and_then(|v| v.as_str());

                let output = MakeExecutor::dev(service, project_path.as_ref()).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MakeUp => {
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;
                let service = args.get("service").and_then(|v| v.as_str());

                let output = MakeExecutor::up(service, project_path.as_ref()).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MakeDown => {
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;
                let service = args.get("service").and_then(|v| v.as_str());

                let output = MakeExecutor::down(service, project_path.as_ref()).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MakeLogs => {
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;
                let service = args.get("service").and_then(|v| v.as_str());

                let output = MakeExecutor::logs(service, project_path.as_ref()).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MakeRestart => {
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;
                let service = args.get("service").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'service' is required".to_string())
                })?;

                let output = MakeExecutor::restart(service, project_path.as_ref()).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MakeShell => {
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;
                let service = args.get("service").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'service' is required".to_string())
                })?;

                let output = MakeExecutor::shell(service, None, project_path.as_ref()).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MakePs => {
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;

                let output = MakeExecutor::ps(project_path.as_ref()).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MakeHelp => {
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;

                let output = MakeExecutor::help(project_path.as_ref()).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MakeKey => {
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;
                let bytes = args.get("bytes").and_then(|v| v.as_u64()).unwrap_or(32) as u32;
                let format = args.get("format").and_then(|v| v.as_str()).unwrap_or("hex");

                let output = MakeExecutor::make_key(bytes, format, project_path.as_ref()).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::ProjectAnalyze => {
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;

                let project = project_detector.analyze(project_path.as_ref()).await?;
                let result = format!(
                    "Project: {}\nRoot: {:?}\n\nServices ({}):\n{}\n\nInfrastructure: {}\nProviders: {}\n\nCompose Files:\n{}\n\nMake Targets:\n{}",
                    project.name,
                    project.root,
                    project.services.len(),
                    project.services.iter().map(|s| format!(
                        "  - {} (dockerfile: {}, compose: {}, dev: {})",
                        s.name, s.has_dockerfile, s.has_compose, s.has_dev_compose
                    )).collect::<Vec<_>>().join("\n"),
                    if project.has_infra { "yes" } else { "no" },
                    project.infra_providers.join(", "),
                    project.compose_files.iter().map(|f| format!("  - {}", f)).collect::<Vec<_>>().join("\n"),
                    project.make_targets.iter().map(|t| format!("  - {}", t)).collect::<Vec<_>>().join("\n"),
                );
                Ok(ToolCallResult::text(result))
            }

            ToolHandler::ProjectList => {
                let search_path = args.get("search_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'search_path' is required".to_string())
                })?;

                let projects = project_detector.find_all_projects(search_path.as_ref()).await?;
                let result = if projects.is_empty() {
                    "No MechCrate projects found.".to_string()
                } else {
                    format!(
                        "Found {} project(s):\n{}",
                        projects.len(),
                        projects.iter().map(|p| format!("  - {:?}", p)).collect::<Vec<_>>().join("\n")
                    )
                };
                Ok(ToolCallResult::text(result))
            }

            ToolHandler::ProjectDetect => {
                let path = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'path' is required".to_string())
                })?;

                match ProjectDetector::find_project_root(path.as_ref()) {
                    Some(root) => Ok(ToolCallResult::text(format!("Project root: {:?}", root))),
                    None => Ok(ToolCallResult::text("No MechCrate project found at or above this path.".to_string())),
                }
            }

            ToolHandler::ServiceInfo => {
                let service = args.get("service").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'service' is required".to_string())
                })?;
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;

                let svc = project_detector.get_service(project_path.as_ref(), service).await?;
                let result = format!(
                    "Service: {}\nDockerfile: {}\nCompose: {}\nDev Compose: {}\nApp Directory: {:?}",
                    svc.name,
                    svc.has_dockerfile,
                    svc.has_compose,
                    svc.has_dev_compose,
                    svc.app_dir
                );
                Ok(ToolCallResult::text(result))
            }

            ToolHandler::RagSearch => {
                let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'query' is required".to_string())
                })?;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

                match weaviate.search(query, limit).await {
                    Ok(docs) => Ok(ToolCallResult::text(format_search_results(&docs))),
                    Err(e) => Ok(ToolCallResult::text(format!(
                        "RAG search failed (Weaviate may not be running): {}",
                        e
                    ))),
                }
            }

            ToolHandler::RagSearchCategory => {
                let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'query' is required".to_string())
                })?;
                let category = args.get("category").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'category' is required".to_string())
                })?;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

                match weaviate.search_by_category(query, category, limit).await {
                    Ok(docs) => Ok(ToolCallResult::text(format_search_results(&docs))),
                    Err(e) => Ok(ToolCallResult::text(format!(
                        "RAG search failed (Weaviate may not be running): {}",
                        e
                    ))),
                }
            }

            ToolHandler::RagFindImplementation => {
                let pattern = args.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'pattern' is required".to_string())
                })?;
                let language = args.get("language").and_then(|v| v.as_str());

                // Build query for code implementations
                let query = if let Some(lang) = language {
                    format!("code implementation {} {} example", pattern, lang)
                } else {
                    format!("code implementation {} example", pattern)
                };

                match weaviate.search(&query, 5).await {
                    Ok(docs) => {
                        // Filter to prefer code-heavy results
                        let filtered: Vec<_> = docs
                            .into_iter()
                            .filter(|d| {
                                d.content.contains("```") ||
                                d.content.contains("FROM ") ||  // Dockerfile
                                d.content.contains("services:") ||  // compose
                                d.content.contains("fn ") ||
                                d.content.contains("function ")
                            })
                            .collect();

                        if filtered.is_empty() {
                            // Fall back to unfiltered if no code found
                            match weaviate.search(&query, 5).await {
                                Ok(all) => Ok(ToolCallResult::text(format_search_results(&all))),
                                Err(e) => Ok(ToolCallResult::text(format!("Search failed: {}", e))),
                            }
                        } else {
                            Ok(ToolCallResult::text(format_search_results(&filtered)))
                        }
                    }
                    Err(e) => Ok(ToolCallResult::text(format!(
                        "RAG search failed (Weaviate may not be running): {}", e
                    ))),
                }
            }

            ToolHandler::RagGetGuidance => {
                let problem = args.get("problem").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'problem' is required".to_string())
                })?;
                let constraints: Option<Vec<String>> = args
                    .get("constraints")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());

                // Build query with constraints
                let query = if let Some(ref c) = constraints {
                    format!("architecture design pattern {} constraints: {}", problem, c.join(", "))
                } else {
                    format!("architecture design pattern best practice {}", problem)
                };

                match weaviate.search(&query, 7).await {
                    Ok(docs) => {
                        // Group by source document for better context
                        let mut grouped: std::collections::HashMap<String, Vec<&crate::rag::Document>> = 
                            std::collections::HashMap::new();
                        for doc in &docs {
                            grouped.entry(doc.source.clone()).or_default().push(doc);
                        }

                        let mut result = format!("## Guidance for: {}\n\n", problem);
                        if let Some(c) = constraints {
                            result.push_str(&format!("**Constraints:** {}\n\n", c.join(", ")));
                        }
                        result.push_str("---\n\n");

                        for (source, chunks) in grouped {
                            result.push_str(&format!("### From: {}\n\n", source));
                            for chunk in chunks {
                                result.push_str(&chunk.content);
                                result.push_str("\n\n");
                            }
                        }

                        Ok(ToolCallResult::text(result))
                    }
                    Err(e) => Ok(ToolCallResult::text(format!(
                        "RAG search failed (Weaviate may not be running): {}", e
                    ))),
                }
            }

            ToolHandler::RagCompareApproaches => {
                let approaches: Vec<String> = args
                    .get("approaches")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .ok_or_else(|| McpError::InvalidArguments("'approaches' is required".to_string()))?;

                let criteria: Option<Vec<String>> = args
                    .get("criteria")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());

                let mut result = format!("## Comparison: {}\n\n", approaches.join(" vs "));
                if let Some(ref c) = criteria {
                    result.push_str(&format!("**Focus:** {}\n\n", c.join(", ")));
                }
                result.push_str("---\n\n");

                for approach in &approaches {
                    let query = if let Some(ref c) = criteria {
                        format!("{} {} {}", approach, c.join(" "), approaches.join(" vs "))
                    } else {
                        format!("{} features usage documentation", approach)
                    };

                    result.push_str(&format!("### {}\n\n", approach));

                    match weaviate.search(&query, 3).await {
                        Ok(docs) => {
                            if docs.is_empty() {
                                result.push_str("No specific documentation found.\n\n");
                            } else {
                                for doc in docs {
                                    result.push_str(&format!("**{}**\n", doc.title));
                                    result.push_str(&doc.content);
                                    result.push_str("\n\n");
                                }
                            }
                        }
                        Err(e) => {
                            result.push_str(&format!("Search error: {}\n\n", e));
                        }
                    }
                }

                Ok(ToolCallResult::text(result))
            }

            ToolHandler::RagFindRelated => {
                let topic = args.get("topic").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'topic' is required".to_string())
                })?;
                let max_results = args.get("max_results").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

                // Normalize topic for search
                let query = topic.replace(".md", "").replace("-", " ").replace("_", " ");

                match weaviate.search(&query, max_results + 3).await {
                    Ok(docs) => {
                        // Filter out the source topic itself
                        let filtered: Vec<_> = docs
                            .into_iter()
                            .filter(|d| !d.source.contains(topic) && !d.title.contains(topic))
                            .take(max_results)
                            .collect();

                        if filtered.is_empty() {
                            Ok(ToolCallResult::text(format!(
                                "No related documents found for: {}", topic
                            )))
                        } else {
                            let mut result = format!("## Documents Related to: {}\n\n", topic);
                            for doc in &filtered {
                                result.push_str(&format!("### {} [{}]\n\n", doc.title, doc.category));
                                result.push_str(&doc.content);
                                result.push_str("\n\n---\n\n");
                            }
                            Ok(ToolCallResult::text(result))
                        }
                    }
                    Err(e) => Ok(ToolCallResult::text(format!(
                        "RAG search failed (Weaviate may not be running): {}", e
                    ))),
                }
            }

            ToolHandler::RagHealth => {
                let healthy = weaviate.health_check().await;
                let result = if healthy {
                    "Weaviate RAG server is available and ready."
                } else {
                    "Weaviate RAG server is not available. Start it with: docker compose up -d"
                };
                Ok(ToolCallResult::text(result))
            }

            // ─────────────────────────────────────────────────────────────────
            // Document Compilation Handlers
            // ─────────────────────────────────────────────────────────────────

            ToolHandler::MxDocsCompile => {
                let mut cmd_args: Vec<String> = vec!["docs".to_string()];

                // Determine mode: input file/dir, --doc, or --all
                if let Some(doc) = args.get("doc").and_then(|v| v.as_str()) {
                    cmd_args.push(format!("--doc={}", doc));
                } else if args.get("all").and_then(|v| v.as_bool()).unwrap_or(false) {
                    cmd_args.push("--all".to_string());
                } else if let Some(input) = args.get("input").and_then(|v| v.as_str()) {
                    cmd_args.push(input.to_string());
                } else {
                    return Err(McpError::InvalidArguments(
                        "One of 'input', 'doc', or 'all' is required. Provide a markdown file/directory path, a doc name from config, or set all=true.".to_string()
                    ));
                }

                // Config path
                if let Some(config) = args.get("config").and_then(|v| v.as_str()) {
                    cmd_args.push(format!("--config={}", config));
                }

                // Output directory
                if let Some(output) = args.get("output").and_then(|v| v.as_str()) {
                    cmd_args.push(format!("--output={}", output));
                }

                // Metadata overrides
                if let Some(title) = args.get("title").and_then(|v| v.as_str()) {
                    cmd_args.push(format!("--title={}", title));
                }
                if let Some(subtitle) = args.get("subtitle").and_then(|v| v.as_str()) {
                    cmd_args.push(format!("--subtitle={}", subtitle));
                }
                if let Some(author) = args.get("author").and_then(|v| v.as_str()) {
                    cmd_args.push(format!("--author={}", author));
                }
                if let Some(theme) = args.get("theme").and_then(|v| v.as_str()) {
                    cmd_args.push(format!("--theme={}", theme));
                }

                // Branding
                if let Some(logo) = args.get("logo").and_then(|v| v.as_str()) {
                    cmd_args.push(format!("--logo={}", logo));
                }
                if let Some(company_name) = args.get("company_name").and_then(|v| v.as_str()) {
                    cmd_args.push(format!("--company-name={}", company_name));
                }
                if args.get("no_logo").and_then(|v| v.as_bool()).unwrap_or(false) {
                    cmd_args.push("--no-logo".to_string());
                }

                // Format
                match args.get("format").and_then(|v| v.as_str()) {
                    Some("html") => cmd_args.push("--html-only".to_string()),
                    Some("markdown") => cmd_args.push("--markdown-only".to_string()),
                    _ => {} // pdf is default (generates all)
                }

                // Table of contents
                if args.get("no_toc").and_then(|v| v.as_bool()).unwrap_or(false) {
                    cmd_args.push("--no-toc".to_string());
                }

                let arg_refs: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
                let output = mx.execute(&arg_refs, None).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            ToolHandler::MxDocsList => {
                let mut cmd_args: Vec<String> = vec!["docs".to_string(), "--list".to_string()];

                if let Some(config) = args.get("config").and_then(|v| v.as_str()) {
                    cmd_args.push(format!("--config={}", config));
                }

                let arg_refs: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
                let output = mx.execute(&arg_refs, None).await?;
                Ok(ToolCallResult::text(output.format()))
            }

            // ─────────────────────────────────────────────────────────────────
            // Unyform Integration Handlers
            // ─────────────────────────────────────────────────────────────────

            ToolHandler::UnyformLogin => {
                let unyform = UnyformClient::new();
                let api_key = args.get("api_key").and_then(|v| v.as_str());
                let url = args.get("url").and_then(|v| v.as_str());

                match api_key {
                    Some(key) => {
                        match unyform.login_with_api_key(key, url).await {
                            Ok(resp) => {
                                let org_name = resp.user.organizations.first()
                                    .map(|o| o.name.as_str())
                                    .unwrap_or("N/A");
                                Ok(ToolCallResult::text(format!(
                                    "Logged in as {} ({})\nOrganization: {}",
                                    resp.user.name,
                                    resp.user.email,
                                    org_name
                                )))
                            }
                            Err(e) => Ok(ToolCallResult::text(format!("Login failed: {}", e))),
                        }
                    }
                    None => {
                        Ok(ToolCallResult::text(
                            "API key required. Use: unyform_login with api_key parameter\n\n\
                             For browser OAuth, run `mx login --browser` from terminal."
                        ))
                    }
                }
            }

            ToolHandler::UnyformLogout => {
                let unyform = UnyformClient::new();
                match unyform.logout().await {
                    Ok(_) => Ok(ToolCallResult::text("Logged out successfully")),
                    Err(e) => Ok(ToolCallResult::text(format!("Logout failed: {}", e))),
                }
            }

            ToolHandler::UnyformWhoami => {
                let unyform = UnyformClient::new();
                if !unyform.is_logged_in() {
                    return Ok(ToolCallResult::text(
                        "Not logged in. Use unyform_login to authenticate."
                    ));
                }

                match unyform.whoami().await {
                    Ok(user) => {
                        let orgs = user.organizations.iter()
                            .map(|o| format!("  {} ({})", o.slug, o.role))
                            .collect::<Vec<_>>()
                            .join("\n");
                        Ok(ToolCallResult::text(format!(
                            "Unyform Account\n\
                             ────────────────────────────────────\n\
                             Email: {}\n\
                             Name: {}\n\n\
                             Organizations:\n{}",
                            user.email,
                            user.name,
                            orgs
                        )))
                    }
                    Err(e) => Ok(ToolCallResult::text(format!("Failed to get user info: {}", e))),
                }
            }

            ToolHandler::UnyformRecipesList => {
                let unyform = UnyformClient::new();
                if !unyform.is_logged_in() {
                    return Ok(ToolCallResult::text(
                        "Not logged in. Use unyform_login to authenticate."
                    ));
                }

                match unyform.list_recipes().await {
                    Ok(resp) => {
                        if resp.recipes.is_empty() {
                            Ok(ToolCallResult::text(
                                "No recipes found.\n\n\
                                 Recipes are generated from your connected repositories.\n\
                                 Connect a repo and run analysis from the Unyform dashboard."
                            ))
                        } else {
                            let recipes = resp.recipes.iter()
                                .map(|r| format!(
                                    "  {} @ v{} - {}",
                                    r.name,
                                    r.version,
                                    r.description.as_deref().unwrap_or("No description")
                                ))
                                .collect::<Vec<_>>()
                                .join("\n");
                            Ok(ToolCallResult::text(format!(
                                "Organization Recipes\n\
                                 ────────────────────────────────────\n{}",
                                recipes
                            )))
                        }
                    }
                    Err(e) => Ok(ToolCallResult::text(format!("Failed to list recipes: {}", e))),
                }
            }

            ToolHandler::UnyformRecipesPull => {
                let unyform = UnyformClient::new();
                if !unyform.is_logged_in() {
                    return Ok(ToolCallResult::text(
                        "Not logged in. Use unyform_login to authenticate."
                    ));
                }

                let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'name' is required".to_string())
                })?;
                let version = args.get("version").and_then(|v| v.as_str());

                match unyform.get_recipe(name, version).await {
                    Ok(recipe) => {
                        let org = unyform.get_default_org().unwrap_or_else(|_| "unknown".to_string());
                        match unyform.cache_recipe(&org, &recipe) {
                            Ok(path) => Ok(ToolCallResult::text(format!(
                                "Recipe cached: {:?}\n\n\
                                 Run unyform_recipes_apply to apply to a project.",
                                path
                            ))),
                            Err(e) => Ok(ToolCallResult::text(format!("Failed to cache recipe: {}", e))),
                        }
                    }
                    Err(e) => Ok(ToolCallResult::text(format!("Failed to pull recipe: {}", e))),
                }
            }

            ToolHandler::UnyformRecipesApply => {
                let unyform = UnyformClient::new();
                let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'name' is required".to_string())
                })?;
                let project_path = args.get("project_path").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'project_path' is required".to_string())
                })?;
                let version = args.get("version").and_then(|v| v.as_str());
                let _fix = args.get("fix").and_then(|v| v.as_bool()).unwrap_or(false);

                // Pull recipe if not cached
                let recipe = match unyform.get_recipe(name, version).await {
                    Ok(r) => r,
                    Err(e) => {
                        return Ok(ToolCallResult::text(format!(
                            "Failed to get recipe: {}\n\n\
                             Make sure you're logged in: unyform_login",
                            e
                        )));
                    }
                };

                // Apply patterns
                let rules_dir = std::path::PathBuf::from(project_path).join(".cursor").join("rules");
                if let Err(e) = std::fs::create_dir_all(&rules_dir) {
                    return Ok(ToolCallResult::text(format!("Failed to create rules directory: {}", e)));
                }

                let org_slug = name.split('-').next().unwrap_or(name);
                let rules_file = rules_dir.join(format!("{}-patterns.md", org_slug));

                let mut rules_content = format!("# {} Coding Patterns\n\n", recipe.name);
                rules_content.push_str("Generated from organizational recipe.\n\n");

                for pattern in &recipe.patterns {
                    if let Some(obj) = pattern.as_object() {
                        if let (Some(name), Some(desc)) = (obj.get("name"), obj.get("description")) {
                            rules_content.push_str(&format!("## {}\n\n", name.as_str().unwrap_or("")));
                            rules_content.push_str(&format!("{}\n\n", desc.as_str().unwrap_or("")));
                            
                            if let Some(rules) = obj.get("rules").and_then(|r| r.as_array()) {
                                rules_content.push_str("### Rules\n\n");
                                for rule in rules {
                                    if let Some(r) = rule.as_str() {
                                        rules_content.push_str(&format!("- {}\n", r));
                                    }
                                }
                                rules_content.push('\n');
                            }
                        }
                    }
                }

                if let Err(e) = std::fs::write(&rules_file, &rules_content) {
                    return Ok(ToolCallResult::text(format!("Failed to write rules file: {}", e)));
                }

                let result = format!(
                    "Applying recipe: {} v{}\n\
                     ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n\
                     ✓ Created {:?} ({} coding rules)\n\n\
                     Dependencies: Check your Cargo.toml/package.json against recipe\n\
                     Infrastructure: Review recipe infrastructure config\n\n\
                     Recipe applied!",
                    recipe.name,
                    recipe.version,
                    rules_file,
                    recipe.patterns.len()
                );

                Ok(ToolCallResult::text(result))
            }

            ToolHandler::UnyformRecipesVersions => {
                let unyform = UnyformClient::new();
                if !unyform.is_logged_in() {
                    return Ok(ToolCallResult::text(
                        "Not logged in. Use unyform_login to authenticate."
                    ));
                }

                let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                    McpError::InvalidArguments("'name' is required".to_string())
                })?;

                match unyform.get_recipe_versions(name).await {
                    Ok(resp) => {
                        let versions = resp.versions.iter()
                            .map(|v| format!(
                                "  v{}{} - {}",
                                v.version,
                                if v.is_latest { " (latest)" } else { "" },
                                v.generated_at
                            ))
                            .collect::<Vec<_>>()
                            .join("\n");
                        Ok(ToolCallResult::text(format!(
                            "Versions for {}\n\
                             ────────────────────────────────────\n{}",
                            name,
                            versions
                        )))
                    }
                    Err(e) => Ok(ToolCallResult::text(format!("Failed to get versions: {}", e))),
                }
            }

            ToolHandler::UnyformRecipesCache => {
                let unyform = UnyformClient::new();
                let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("list");

                match action {
                    "clear" => {
                        match unyform.clear_cache() {
                            Ok(_) => Ok(ToolCallResult::text("Recipe cache cleared")),
                            Err(e) => Ok(ToolCallResult::text(format!("Failed to clear cache: {}", e))),
                        }
                    }
                    _ => {
                        match unyform.list_cached_recipes() {
                            Ok(recipes) => {
                                if recipes.is_empty() {
                                    Ok(ToolCallResult::text(
                                        "No cached recipes.\n\n\
                                         Pull recipes with unyform_recipes_pull"
                                    ))
                                } else {
                                    let list = recipes.iter()
                                        .map(|(org, name, versions)| {
                                            format!("  {}/{}: {}", org, name, versions.join(", "))
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                    Ok(ToolCallResult::text(format!(
                                        "Cached Recipes\n\
                                         ────────────────────────────────────\n{}",
                                        list
                                    )))
                                }
                            }
                            Err(e) => Ok(ToolCallResult::text(format!("Failed to list cache: {}", e))),
                        }
                    }
                }
            }
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
