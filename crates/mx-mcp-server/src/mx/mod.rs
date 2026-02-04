//! MX Command Executor
//!
//! Handles execution of mx CLI commands and Make targets.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info};

use crate::error::{McpError, McpResult};

/// Result of command execution
#[derive(Debug)]
pub struct CommandOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

impl CommandOutput {
    /// Format as a readable result
    pub fn format(&self) -> String {
        let mut result = String::new();
        
        if !self.stdout.is_empty() {
            result.push_str(&self.stdout);
        }
        
        if !self.stderr.is_empty() {
            if !result.is_empty() {
                result.push_str("\n\n--- stderr ---\n");
            }
            result.push_str(&self.stderr);
        }
        
        if let Some(code) = self.exit_code {
            if code != 0 {
                result.push_str(&format!("\n\nExit code: {}", code));
            }
        }
        
        if result.is_empty() {
            result = "(no output)".to_string();
        }
        
        result
    }
}

/// Executor for mx commands
pub struct MxExecutor {
    mx_path: PathBuf,
}

impl MxExecutor {
    /// Create a new executor
    pub fn new(mech_crate_root: PathBuf) -> Self {
        let mx_path = mech_crate_root.join("bin/mx");
        Self { mx_path }
    }

    /// Execute an mx command
    pub async fn execute(&self, args: &[&str], working_dir: Option<&Path>) -> McpResult<CommandOutput> {
        let cwd = working_dir.unwrap_or_else(|| Path::new("."));
        
        info!("Executing: mx {} in {:?}", args.join(" "), cwd);
        
        let output = Command::new(&self.mx_path)
            .args(args)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| McpError::CommandFailed(format!("Failed to execute mx: {}", e)))?;

        let result = CommandOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
        };

        debug!("Command result: success={}, exit_code={:?}", result.success, result.exit_code);

        Ok(result)
    }

    /// Execute mx help command
    pub async fn help(&self) -> McpResult<CommandOutput> {
        self.execute(&["help"], None).await
    }

    /// Create a new project
    pub async fn new_project(
        &self,
        name: &str,
        with_services: Option<&[&str]>,
        with_infra: Option<&[&str]>,
        no_prompt: bool,
    ) -> McpResult<CommandOutput> {
        let mut args = vec!["new", name];
        
        if let Some(services) = with_services {
            args.push("--with");
            args.extend(services.iter().copied());
        }
        
        if let Some(infra) = with_infra {
            args.push("--infra");
            args.extend(infra.iter().copied());
        }
        
        if no_prompt {
            args.push("--no-prompt");
        }
        
        self.execute(&args, None).await
    }

    /// Add a service to an existing project
    pub async fn add_service(
        &self,
        name: &str,
        recipe: Option<&str>,
        domain: Option<&str>,
        working_dir: &Path,
    ) -> McpResult<CommandOutput> {
        let mut args = vec!["add", name];
        
        if let Some(r) = recipe {
            args.push("--recipe");
            args.push(r);
        }
        
        if let Some(d) = domain {
            let domain_arg = format!("--domain={}", d);
            // We need to handle this differently due to lifetime
            let args_with_domain: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            let mut final_args: Vec<&str> = args_with_domain.iter().map(|s| s.as_str()).collect();
            final_args.push(&domain_arg);
            return self.execute_owned(final_args, Some(working_dir)).await;
        }
        
        self.execute(&args, Some(working_dir)).await
    }

    async fn execute_owned(&self, args: Vec<&str>, working_dir: Option<&Path>) -> McpResult<CommandOutput> {
        self.execute(&args, working_dir).await
    }

    /// List available recipes
    pub async fn list_recipes(&self) -> McpResult<CommandOutput> {
        self.execute(&["recipes"], None).await
    }

    /// Show recipe info
    pub async fn recipe_info(&self, recipe: &str) -> McpResult<CommandOutput> {
        self.execute(&["recipes", "info", recipe], None).await
    }

    /// Router commands
    pub async fn router(&self, subcommand: &str) -> McpResult<CommandOutput> {
        self.execute(&["router", subcommand], None).await
    }

    /// Infrastructure commands
    pub async fn infra(&self, subcommand: &str, provider: Option<&str>) -> McpResult<CommandOutput> {
        let mut args = vec!["infra", subcommand];
        if let Some(p) = provider {
            args.push(p);
        }
        self.execute(&args, None).await
    }

    /// Run doctor check
    pub async fn doctor(&self, working_dir: Option<&Path>) -> McpResult<CommandOutput> {
        self.execute(&["doctor"], working_dir).await
    }

    /// Upgrade project
    pub async fn upgrade(&self, working_dir: &Path, diff: bool, yes: bool) -> McpResult<CommandOutput> {
        let mut args = vec!["upgrade"];
        if diff {
            args.push("--diff");
        }
        if yes {
            args.push("--yes");
        }
        self.execute(&args, Some(working_dir)).await
    }

    /// Build command
    pub async fn build(
        &self,
        service: &str,
        prod: bool,
        tag: Option<&str>,
        push: bool,
        working_dir: &Path,
    ) -> McpResult<CommandOutput> {
        let mut args = vec!["build", service];
        
        if prod {
            args.push("--prod");
        }
        
        if let Some(t) = tag {
            args.push("-t");
            args.push(t);
        }
        
        if push {
            args.push("--push");
        }
        
        self.execute(&args, Some(working_dir)).await
    }
}

/// Executor for Make commands in a project
pub struct MakeExecutor;

impl MakeExecutor {
    /// Execute a make target
    pub async fn execute(target: &str, args: &[(&str, &str)], working_dir: &Path) -> McpResult<CommandOutput> {
        let mut cmd_args = vec![target];
        
        // Add variable assignments
        let var_strings: Vec<String> = args.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
        let var_refs: Vec<&str> = var_strings.iter().map(|s| s.as_str()).collect();
        cmd_args.extend(var_refs);
        
        info!("Executing: make {} in {:?}", cmd_args.join(" "), working_dir);
        
        let output = Command::new("make")
            .args(&cmd_args)
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| McpError::CommandFailed(format!("Failed to execute make: {}", e)))?;

        Ok(CommandOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
        })
    }

    /// Start services in dev mode
    pub async fn dev(service: Option<&str>, working_dir: &Path) -> McpResult<CommandOutput> {
        let args = service.map(|s| vec![("s", s)]).unwrap_or_default();
        Self::execute("dev", &args, working_dir).await
    }

    /// Start services in production mode
    pub async fn up(service: Option<&str>, working_dir: &Path) -> McpResult<CommandOutput> {
        let args = service.map(|s| vec![("s", s)]).unwrap_or_default();
        Self::execute("up", &args, working_dir).await
    }

    /// Stop services
    pub async fn down(service: Option<&str>, working_dir: &Path) -> McpResult<CommandOutput> {
        let args = service.map(|s| vec![("s", s)]).unwrap_or_default();
        Self::execute("down", &args, working_dir).await
    }

    /// View logs
    pub async fn logs(service: Option<&str>, working_dir: &Path) -> McpResult<CommandOutput> {
        let args = service.map(|s| vec![("s", s)]).unwrap_or_default();
        Self::execute("logs", &args, working_dir).await
    }

    /// Restart service
    pub async fn restart(service: &str, working_dir: &Path) -> McpResult<CommandOutput> {
        Self::execute("restart", &[("s", service)], working_dir).await
    }

    /// Shell into service
    pub async fn shell(service: &str, _command: Option<&str>, working_dir: &Path) -> McpResult<CommandOutput> {
        // For shell commands, we need to execute differently since it's interactive
        // Instead, we provide information about how to shell in
        let output = format!(
            "To shell into service '{}', run:\n  cd {:?} && make sh s={}\n\nOr use docker exec directly:\n  docker exec -it {} bash",
            service, working_dir, service, service
        );
        
        Ok(CommandOutput {
            success: true,
            stdout: output,
            stderr: String::new(),
            exit_code: Some(0),
        })
    }

    /// List running services
    pub async fn ps(working_dir: &Path) -> McpResult<CommandOutput> {
        Self::execute("ps", &[], working_dir).await
    }

    /// Run help
    pub async fn help(working_dir: &Path) -> McpResult<CommandOutput> {
        Self::execute("help", &[], working_dir).await
    }

    /// Generate secret key
    pub async fn make_key(bytes: u32, format: &str, working_dir: &Path) -> McpResult<CommandOutput> {
        let bytes_str = bytes.to_string();
        Self::execute("make-key", &[("BYTES", &bytes_str), ("FORMAT", format)], working_dir).await
    }
}
