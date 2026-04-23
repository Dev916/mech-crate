//! Traefik router management
//!
//! Manages the global Traefik router for local development.

use std::net::TcpListener;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::config::MechCrateConfig;
use crate::docker::Compose;
use crate::error::{Error, Result};
use crate::paths;

/// Router state file names
const DASHBOARD_PORT_FILE: &str = ".dashboard-port";
const NETWORK_NAME: &str = "devmesh-traefik";

/// Dashboard port allocation range (matches shell implementation and docker-compose template)
const DASHBOARD_PORT_START: u16 = 7680;
const DASHBOARD_PORT_END: u16 = 7799;

/// Router manager
#[derive(Debug)]
pub struct Router {
    config: MechCrateConfig,
}

impl Router {
    /// Create a new router manager
    pub fn new(config: MechCrateConfig) -> Self {
        Self { config }
    }

    /// Get the router installation directory
    pub fn install_dir(&self) -> PathBuf {
        self.config.router_dir()
    }

    /// Check if router is installed
    pub fn is_installed(&self) -> bool {
        self.install_dir().join("docker-compose.yml").exists()
    }

    /// Get the Docker network name
    pub fn network_name(&self) -> &'static str {
        NETWORK_NAME
    }

    /// Ensure the Docker network exists
    pub fn ensure_network(&self) -> Result<()> {
        if !crate::docker::Docker::network_exists(NETWORK_NAME) {
            crate::docker::Docker::create_network(NETWORK_NAME)?;
        }
        Ok(())
    }

    /// Find a free port in the given range
    pub fn find_free_port(&self, start: u16, end: u16) -> Result<u16> {
        for port in start..=end {
            if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)) {
                drop(listener);
                return Ok(port);
            }
        }
        Err(Error::Other(format!(
            "No free port found in range {}-{}",
            start, end
        )))
    }

    /// Get or allocate the dashboard port
    pub fn dashboard_port(&self) -> Result<u16> {
        let port_file = self.install_dir().join(DASHBOARD_PORT_FILE);

        if port_file.exists() {
            let content = std::fs::read_to_string(&port_file)?;
            if let Ok(port) = content.trim().parse::<u16>() {
                // Validate cached port is within the expected range
                if port >= DASHBOARD_PORT_START && port <= DASHBOARD_PORT_END {
                    return Ok(port);
                }
                // Stale port outside expected range -- re-allocate
                tracing::warn!(
                    "Cached dashboard port {} is outside range {}-{}, re-allocating",
                    port, DASHBOARD_PORT_START, DASHBOARD_PORT_END
                );
            }
        }

        // Allocate a new port in the correct range
        let port = self.find_free_port(DASHBOARD_PORT_START, DASHBOARD_PORT_END)?;
        std::fs::write(&port_file, port.to_string())?;
        Ok(port)
    }

    /// Install the router from templates
    pub fn install(&self) -> Result<()> {
        // Get source templates directory
        let templates_dir = paths::templates_dir()?;
        let router_template = templates_dir.join("router");

        if !router_template.exists() {
            return Err(Error::Config(format!(
                "Router template not found at {}. Run 'mx init' first.",
                router_template.display()
            )));
        }

        let install_dir = self.install_dir();

        // Create install directory
        std::fs::create_dir_all(&install_dir)?;

        // Copy all files from template
        self.copy_dir(&router_template, &install_dir)?;

        // Set proper permissions on acme.json
        let acme_json = install_dir.join("letsencrypt").join("acme.json");
        if acme_json.exists() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&acme_json, std::fs::Permissions::from_mode(0o600))?;
            }
        }

        // Try to create the network (non-fatal if Docker isn't available)
        if let Err(e) = self.ensure_network() {
            tracing::warn!("Could not create Docker network: {}. Will retry on start.", e);
        }

        Ok(())
    }

    /// Copy directory recursively
    fn copy_dir(&self, from: &Path, to: &Path) -> Result<()> {
        for entry in WalkDir::new(from) {
            let entry = entry.map_err(|e| Error::Io(e.into()))?;
            let relative = entry
                .path()
                .strip_prefix(from)
                .map_err(|e| Error::Other(e.to_string()))?;
            let dest = to.join(relative);

            if entry.file_type().is_dir() {
                std::fs::create_dir_all(&dest)?;
            } else if entry.file_type().is_file() {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(entry.path(), &dest)?;
            }
        }
        Ok(())
    }

    /// Check if router is running
    pub fn is_running(&self) -> bool {
        if !self.is_installed() {
            return false;
        }

        // Important: we must query the same compose *project* name we use for `up`/`down`.
        // Otherwise `docker compose ps` may look at a different project (derived from CWD),
        // and incorrectly report the router as stopped even when it's running.
        let compose = Compose::new(self.install_dir())
            .with_file("docker-compose.yml")
            .with_project_name("mx-router");

        let output = compose.run(&["ps", "-q"]);

        output
            .map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
            .unwrap_or(false)
    }

    /// Start the router
    pub fn start(&self) -> Result<()> {
        if !self.is_installed() {
            return Err(Error::Other("Router not installed. Run 'mx router install' first.".into()));
        }

        self.ensure_network()?;
        let port = self.dashboard_port()?;

        let compose = Compose::new(self.install_dir())
            .with_file("docker-compose.yml")
            .with_project_name("mx-router")
            .with_env("MX_ROUTER_DASHBOARD_PORT", port.to_string());

        let output = compose.run(&["up", "-d"])?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!("Failed to start router: {}", stderr)));
        }

        tracing::info!("Router started. Dashboard: http://localhost:{}", port);
        Ok(())
    }

    /// Stop the router
    pub fn stop(&self) -> Result<()> {
        if !self.is_installed() {
            return Ok(());
        }

        let compose = Compose::new(self.install_dir())
            .with_file("docker-compose.yml")
            .with_project_name("mx-router");

        let output = compose.run(&["down"])?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!("Failed to stop router: {}", stderr)));
        }

        Ok(())
    }

    /// Get router status information
    pub fn status(&self) -> Result<RouterStatus> {
        Ok(RouterStatus {
            installed: self.is_installed(),
            running: self.is_running(),
            network: NETWORK_NAME.to_string(),
            dashboard_port: self.dashboard_port().ok(),
            install_dir: self.install_dir(),
        })
    }

    /// Get logs
    pub fn logs(&self, follow: bool) -> Result<std::process::Output> {
        if !self.is_installed() {
            return Err(Error::Other("Router not installed".into()));
        }

        let compose = Compose::new(self.install_dir())
            .with_file("docker-compose.yml")
            .with_project_name("mx-router");

        compose.logs(None, follow)
    }
}

/// Router status information
#[derive(Debug)]
pub struct RouterStatus {
    pub installed: bool,
    pub running: bool,
    pub network: String,
    pub dashboard_port: Option<u16>,
    pub install_dir: PathBuf,
}

impl Default for Router {
    fn default() -> Self {
        Self::new(MechCrateConfig::default())
    }
}
