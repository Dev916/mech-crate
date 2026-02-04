//! Docker and Compose integration
//!
//! Provides utilities for working with Docker containers and compose files.

use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};

/// Docker command executor
#[derive(Debug, Default)]
pub struct Docker;

impl Docker {
    /// Check if Docker is available
    pub fn is_available() -> bool {
        Command::new("docker")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Check if Docker daemon is running
    pub fn is_running() -> bool {
        Command::new("docker")
            .args(["info"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get Docker version
    pub fn version() -> Result<String> {
        let output = Command::new("docker")
            .args(["--version"])
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to run docker: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(Error::CommandFailed("Docker not available".into()))
        }
    }

    /// Check if a network exists
    pub fn network_exists(name: &str) -> bool {
        Command::new("docker")
            .args(["network", "inspect", name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Create a Docker network
    pub fn create_network(name: &str) -> Result<()> {
        let output = Command::new("docker")
            .args(["network", "create", name])
            .output()
            .map_err(|e| Error::CommandFailed(format!("Failed to create network: {}", e)))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(Error::CommandFailed(format!(
                "Failed to create network: {}",
                stderr
            )))
        }
    }
}

/// Docker Compose command executor
#[derive(Debug)]
pub struct Compose {
    /// Working directory
    pub working_dir: std::path::PathBuf,
    /// Compose files to use
    pub files: Vec<std::path::PathBuf>,
    /// Project name
    pub project_name: Option<String>,
}

impl Compose {
    /// Create a new Compose instance
    pub fn new(working_dir: impl AsRef<Path>) -> Self {
        Self {
            working_dir: working_dir.as_ref().to_path_buf(),
            files: Vec::new(),
            project_name: None,
        }
    }

    /// Add a compose file
    pub fn with_file(mut self, file: impl AsRef<Path>) -> Self {
        self.files.push(file.as_ref().to_path_buf());
        self
    }

    /// Set the project name
    pub fn with_project_name(mut self, name: impl Into<String>) -> Self {
        self.project_name = Some(name.into());
        self
    }

    /// Run docker compose with the given arguments
    pub fn run(&self, args: &[&str]) -> Result<std::process::Output> {
        let mut cmd = Command::new("docker");
        cmd.arg("compose");
        cmd.current_dir(&self.working_dir);

        for file in &self.files {
            cmd.args(["-f", file.to_str().unwrap_or_default()]);
        }

        if let Some(ref name) = self.project_name {
            cmd.args(["-p", name]);
        }

        cmd.args(args);

        cmd.output()
            .map_err(|e| Error::CommandFailed(format!("Failed to run docker compose: {}", e)))
    }

    /// Start services
    pub fn up(&self, detached: bool) -> Result<std::process::Output> {
        let mut args = vec!["up"];
        if detached {
            args.push("-d");
        }
        self.run(&args)
    }

    /// Stop services
    pub fn down(&self) -> Result<std::process::Output> {
        self.run(&["down"])
    }

    /// Get service logs
    pub fn logs(&self, service: Option<&str>, follow: bool) -> Result<std::process::Output> {
        let mut args = vec!["logs"];
        if follow {
            args.push("-f");
        }
        if let Some(svc) = service {
            args.push(svc);
        }
        self.run(&args)
    }

    /// List running containers
    pub fn ps(&self) -> Result<std::process::Output> {
        self.run(&["ps"])
    }
}
