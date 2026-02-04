//! MechCrate Documentation Ingestion Tool
//!
//! This tool ingests MechCrate documentation into Weaviate for RAG queries.
//!
//! Usage:
//!   mx-ingest [--weaviate-url URL] [--mech-crate-root PATH] [--clear]

use clap::Parser;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{info, warn, Level};
use tracing_subscriber::{fmt, EnvFilter};

// We'll inline the necessary types here to avoid complex module dependencies
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "mx-ingest")]
#[command(about = "Ingest MechCrate documentation into Weaviate for RAG")]
#[command(version)]
struct Args {
    /// Weaviate endpoint URL
    #[arg(long, env = "WEAVIATE_URL", default_value = "http://localhost:8080")]
    weaviate_url: String,

    /// MechCrate root directory
    #[arg(long, env = "MECH_CRATE_ROOT")]
    mech_crate_root: Option<String>,

    /// Clear existing documents before ingesting
    #[arg(long)]
    clear: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Document {
    title: String,
    content: String,
    category: String,
    source: String,
}

struct Ingester {
    url: String,
    client: Client,
}

impl Ingester {
    fn new(url: &str) -> Self {
        Self {
            url: url.trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    async fn health_check(&self) -> bool {
        match self
            .client
            .get(format!("{}/v1/.well-known/ready", self.url))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    async fn ensure_schema(&self) -> anyhow::Result<()> {
        let schema = serde_json::json!({
            "class": "MechCrateDoc",
            "description": "MechCrate documentation for RAG",
            "vectorizer": "text2vec-transformers",
            "moduleConfig": {
                "text2vec-transformers": {
                    "vectorizeClassName": false
                }
            },
            "properties": [
                {
                    "name": "title",
                    "dataType": ["text"],
                    "description": "Document title",
                    "moduleConfig": {
                        "text2vec-transformers": {
                            "skip": false,
                            "vectorizePropertyName": false
                        }
                    }
                },
                {
                    "name": "content",
                    "dataType": ["text"],
                    "description": "Document content",
                    "moduleConfig": {
                        "text2vec-transformers": {
                            "skip": false,
                            "vectorizePropertyName": false
                        }
                    }
                },
                {
                    "name": "category",
                    "dataType": ["text"],
                    "description": "Document category",
                    "moduleConfig": {
                        "text2vec-transformers": {
                            "skip": true
                        }
                    }
                },
                {
                    "name": "source",
                    "dataType": ["text"],
                    "description": "Source file path",
                    "moduleConfig": {
                        "text2vec-transformers": {
                            "skip": true
                        }
                    }
                }
            ]
        });

        let response = self
            .client
            .post(format!("{}/v1/schema", self.url))
            .json(&schema)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            if !text.contains("already exists") {
                warn!("Schema creation issue: {}", text);
            }
        }

        Ok(())
    }

    async fn clear(&self) -> anyhow::Result<()> {
        let response = self
            .client
            .delete(format!("{}/v1/schema/MechCrateDoc", self.url))
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 404 {
            info!("Cleared existing documents");
        }

        Ok(())
    }

    async fn add_document(&self, doc: &Document) -> anyhow::Result<()> {
        let response = self
            .client
            .post(format!("{}/v1/objects", self.url))
            .json(&serde_json::json!({
                "class": "MechCrateDoc",
                "properties": {
                    "title": doc.title,
                    "content": doc.content,
                    "category": doc.category,
                    "source": doc.source
                }
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            anyhow::bail!("Failed to add document: {}", error);
        }

        Ok(())
    }
}

/// Extract documents from a markdown file
fn extract_documents(path: &Path, content: &str, category: &str) -> Vec<Document> {
    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let mut documents = Vec::new();

    // Split by major headings (## level)
    let mut current_title = file_name.to_string();
    let mut current_content = String::new();

    for line in content.lines() {
        if line.starts_with("## ") {
            // Save previous section if any
            if !current_content.trim().is_empty() {
                documents.push(Document {
                    title: current_title.clone(),
                    content: current_content.trim().to_string(),
                    category: category.to_string(),
                    source: path.to_string_lossy().to_string(),
                });
            }

            // Start new section
            current_title = line.trim_start_matches("## ").to_string();
            current_content = String::new();
        } else if line.starts_with("# ") {
            // Top-level heading becomes the base title
            let base_title = line.trim_start_matches("# ");
            if current_title == file_name {
                current_title = base_title.to_string();
            }
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Don't forget the last section
    if !current_content.trim().is_empty() {
        documents.push(Document {
            title: current_title,
            content: current_content.trim().to_string(),
            category: category.to_string(),
            source: path.to_string_lossy().to_string(),
        });
    }

    // If no sections were found, create one document for the whole file
    if documents.is_empty() && !content.trim().is_empty() {
        documents.push(Document {
            title: file_name.to_string(),
            content: content.to_string(),
            category: category.to_string(),
            source: path.to_string_lossy().to_string(),
        });
    }

    documents
}

/// Determine category from file path
fn categorize_path(path: &Path) -> &str {
    let path_str = path.to_string_lossy().to_lowercase();

    if path_str.contains("recipe") {
        "recipe"
    } else if path_str.contains("router") || path_str.contains("traefik") {
        "traefik"
    } else if path_str.contains("docker") || path_str.contains("compose") {
        "docker"
    } else if path_str.contains("infra") || path_str.contains("cloudflare") {
        "infra"
    } else if path_str.contains("command") || path_str.contains("help") {
        "command"
    } else {
        "structure"
    }
}

async fn find_markdown_files(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if !dir.exists() {
        return Ok(files);
    }

    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        if let Ok(mut entries) = fs::read_dir(&current).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();

                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().map(|e| e == "md").unwrap_or(false) {
                    files.push(path);
                }
            }
        }
    }

    Ok(files)
}

fn detect_mech_crate_root() -> Option<PathBuf> {
    let candidates = [
        std::env::current_dir().ok(),
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("dev/mech-crate")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.join("bin/mx").exists() {
            return Some(candidate);
        }

        let mut path = candidate.clone();
        for _ in 0..5 {
            if path.join("bin/mx").exists() {
                return Some(path);
            }
            if !path.pop() {
                break;
            }
        }
    }

    None
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    let level = if args.verbose { Level::DEBUG } else { Level::INFO };
    let filter = EnvFilter::from_default_env().add_directive(level.into());
    fmt().with_env_filter(filter).with_writer(std::io::stderr).init();

    // Find MechCrate root
    let root = match args.mech_crate_root {
        Some(path) => PathBuf::from(path),
        None => detect_mech_crate_root().ok_or_else(|| {
            anyhow::anyhow!("MechCrate root not found. Use --mech-crate-root to specify.")
        })?,
    };

    info!("MechCrate root: {:?}", root);

    // Initialize Weaviate client
    let ingester = Ingester::new(&args.weaviate_url);

    // Check health
    if !ingester.health_check().await {
        anyhow::bail!(
            "Weaviate is not available at {}. Start it with: docker compose up -d",
            args.weaviate_url
        );
    }

    info!("Weaviate is ready");

    // Clear if requested
    if args.clear {
        ingester.clear().await?;
    }

    // Ensure schema exists
    ingester.ensure_schema().await?;

    // Find all documentation
    let docs_dir = root.join("docs");
    let mut total_docs = 0;

    // Process docs directory
    if docs_dir.exists() {
        info!("Processing docs directory...");
        let files = find_markdown_files(&docs_dir).await?;

        for file in files {
            let content = fs::read_to_string(&file).await?;
            let category = categorize_path(&file);
            let documents = extract_documents(&file, &content, category);

            for doc in documents {
                info!("  Adding: {} [{}]", doc.title, doc.category);
                ingester.add_document(&doc).await?;
                total_docs += 1;
            }
        }
    }

    // Process README
    let readme = root.join("README.md");
    if readme.exists() {
        info!("Processing README.md...");
        let content = fs::read_to_string(&readme).await?;
        let documents = extract_documents(&readme, &content, "structure");

        for doc in documents {
            info!("  Adding: {} [{}]", doc.title, doc.category);
            ingester.add_document(&doc).await?;
            total_docs += 1;
        }
    }

    // Add built-in documentation
    info!("Adding built-in documentation...");

    let builtin_docs = vec![
        Document {
            title: "Creating a New Project".to_string(),
            content: r#"To create a new MechCrate project:

1. Run `mx new <project-name>` to scaffold the project structure
2. The project will include:
   - Makefile with all necessary targets
   - make/ directory with modular Make files
   - scripts/ directory with helper scripts
   - docker/ directory for Docker configuration
   - apps/ directory for service source code

3. Optionally add infrastructure with `--infra cloudflare` or other providers

4. After creation, cd into the project and run `mx add <service> --recipe=<type>` to add services."#.to_string(),
            category: "structure".to_string(),
            source: "built-in".to_string(),
        },
        Document {
            title: "Adding Services with Recipes".to_string(),
            content: r#"Recipes are pre-packaged application templates that include:
- Complete source code structure
- Multi-stage Dockerfile (dev and prod targets)
- Docker Compose files (production and development)
- Supervisor configuration for process management
- Nginx or HAProxy for internal reverse proxy
- Traefik labels for routing

Available recipes:
- laravel: PHP Laravel with Nginx, PHP-FPM, queues, Vite
- nuxt: Nuxt 3 SSR/SSG with Tailwind + DaisyUI
- astro: Astro static sites with Vue components
- rust-api: Rust Axum API with hexagonal architecture
- rust-leptos: Rust Leptos full-stack SSR
- rust-worker: Cloudflare Worker in Rust
- zola: Static site generator with Sass

Usage:
- `mx add myapi --recipe=laravel` - Add Laravel API
- `mx add web --recipe=nuxt` - Add Nuxt frontend
- `mx add api --recipe=rust-api --domain=api.example.com` - Add Rust API with custom domain"#.to_string(),
            category: "recipe".to_string(),
            source: "built-in".to_string(),
        },
        Document {
            title: "Traefik Routing and Networking".to_string(),
            content: r#"MechCrate uses Traefik as a global reverse proxy for hostname-based routing.

Setup:
1. Install the router: `mx router install`
2. Start the router: `mx router up`
3. All services join the `devmesh-traefik` network
4. Access services via hostname: http://myapp.localhost

Services configure routing with Docker labels:
```yaml
labels:
  - traefik.enable=true
  - traefik.http.routers.myapp.rule=Host(`myapp.localhost`)
  - traefik.http.routers.myapp.entrypoints=web
  - traefik.http.services.myapp.loadbalancer.server.port=80
  - traefik.docker.network=devmesh-traefik
```

Network architecture:
- `devmesh-traefik`: External network shared by all projects (for Traefik routing)
- `default`: Implicit network per compose file (for inter-service communication)

Only edge-facing services (main app) need to join devmesh-traefik. Internal services like databases and Redis use the default network."#.to_string(),
            category: "traefik".to_string(),
            source: "built-in".to_string(),
        },
        Document {
            title: "Docker Configuration Best Practices".to_string(),
            content: r#"MechCrate Docker configuration follows these patterns:

1. Multi-stage Dockerfiles:
   - base: Common dependencies
   - deps: Install application dependencies
   - development: Dev tools, hot-reload support
   - builder: Build production assets
   - production: Minimal runtime image

2. System files mirroring:
   docker/system/<service>/ mirrors the container filesystem
   - etc/supervisor/ - Process management configs
   - etc/nginx/ - Reverse proxy configs
   - usr/local/bin/entrypoint - Container entrypoint script

3. Compose files:
   - <service>.yml - Production configuration
   - <service>.dev.yml - Development overrides

4. Build context is always project root to access:
   - apps/<service>/ - Application source
   - docker/system/<service>/ - System files

5. Environment files in docker/.config/:
   - .env.shared - Shared across all services
   - .env.secrets - Secrets (gitignored)
   - .env.<service> - Per-service configuration"#.to_string(),
            category: "docker".to_string(),
            source: "built-in".to_string(),
        },
        Document {
            title: "Infrastructure Configuration".to_string(),
            content: r#"MechCrate supports multiple infrastructure providers:

Supported providers:
- cloudflare: Workers, Containers, KV, R2
- digitalocean: Droplets, App Platform, Spaces
- aws: EC2, ECS, S3, Lambda
- hetzner: Cloud VMs, Object Storage

Configuration hierarchy:
1. Project-local config (./infra/<provider>/.env.<provider>)
2. Global config (~/.mech-crate/config/infra/<provider>.env)

Setup workflow:
1. Configure globally: `mx infra setup cloudflare`
2. Create project with infra: `mx new myapp --infra cloudflare`
3. Link to global config: `mx infra link cloudflare`

Or use project-local credentials by running the provider setup from within the project.

Cloudflare-specific commands:
- `mx cf setup` - Configure credentials
- `mx cf init <app>` - Initialize a worker
- `mx cf deploy <app>` - Deploy to Cloudflare"#.to_string(),
            category: "infra".to_string(),
            source: "built-in".to_string(),
        },
    ];

    for doc in builtin_docs {
        info!("  Adding: {} [{}]", doc.title, doc.category);
        ingester.add_document(&doc).await?;
        total_docs += 1;
    }

    info!("Ingestion complete! Added {} documents.", total_docs);

    Ok(())
}
