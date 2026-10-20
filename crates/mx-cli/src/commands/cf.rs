//! `mx cf` - Cloudflare workflow proxied to the project's `make cf-*` targets
//!
//! Same shape as `mx dev/up/down/logs`: find the project root, run one make
//! target there. Every subcommand maps 1:1 to a `cf-<name>` target in
//! `make/cloudflare.mk`, and an app argument becomes make's `a=<app>`, so
//! `mx cf deploy api.example.com` is exactly `make cf-deploy a=api.example.com`.
//! Credential resolution (environment, then project file, then global file)
//! stays in the make module; this wrapper adds nothing to it.

use anyhow::{bail, Result};
use clap::{Args, Subcommand, ValueEnum};

use super::dev::{find_project_root, run_make_in_project};

/// Cloudflare Workers and Containers (proxied to `make cf-*`)
#[derive(Args, Debug)]
pub struct CfCommand {
    #[command(subcommand)]
    action: CfAction,

    /// Show the make invocation before running it
    #[arg(short, long)]
    verbose: bool,
}

/// One Cloudflare app, named by its domain (the `a=` make variable).
#[derive(Args, Debug)]
pub struct AppArg {
    /// App name, e.g. api.example.com
    app: String,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum AppType {
    Worker,
    Cron,
    Container,
}

impl AppType {
    fn as_str(self) -> &'static str {
        match self {
            AppType::Worker => "worker",
            AppType::Cron => "cron",
            AppType::Container => "container",
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum CfAction {
    /// Run the Cloudflare setup wizard (writes this project's credentials)
    Setup,
    /// Show the resolved Cloudflare credential variables and their source (no secrets)
    #[command(visible_alias = "config")]
    Vars,
    /// Fail early when no Cloudflare account id resolves
    CheckCredentials,
    /// Login to Cloudflare (wrangler login)
    Login,
    /// Show Cloudflare auth status
    Whoami,
    /// Show all Cloudflare apps' status
    Status,
    /// Initialize a new Cloudflare app
    Init {
        /// App name, e.g. api.example.com
        app: String,
        /// App type (the init script prompts when omitted)
        #[arg(long = "type", value_enum)]
        kind: Option<AppType>,
    },
    /// List all Cloudflare apps
    List,
    /// Build an app's container image
    Build(AppArg),
    /// Push an app's image to the Cloudflare registry
    Push(AppArg),
    /// Build and push an app's image
    Publish(AppArg),
    /// List an app's images in the Cloudflare registry
    Images(AppArg),
    /// Sync an app's wrangler.toml image tag
    SyncImage(AppArg),
    /// Install an app's worker dependencies
    Install(AppArg),
    /// Run an app's worker locally
    Dev(AppArg),
    /// Deploy an app to preview
    DeployPreview(AppArg),
    /// Deploy an app to production
    Deploy(AppArg),
    /// Deploy all apps to production
    DeployAll,
    /// Tail an app's worker logs
    Logs(AppArg),
    /// Tail an app's preview logs
    LogsPreview(AppArg),
    /// Restart an app's container
    Restart(AppArg),
    /// Check an app's container status
    ContainerStatus(AppArg),
}

impl CfAction {
    /// The make target and its variables for this action.
    fn make_invocation(&self) -> (&'static str, Vec<String>) {
        let app = |a: &AppArg| vec![format!("a={}", a.app)];
        match self {
            CfAction::Setup => ("cf-setup", vec![]),
            CfAction::Vars => ("cf-vars", vec![]),
            CfAction::CheckCredentials => ("cf-check-credentials", vec![]),
            CfAction::Login => ("cf-login", vec![]),
            CfAction::Whoami => ("cf-whoami", vec![]),
            CfAction::Status => ("cf-status", vec![]),
            CfAction::Init { app, kind } => {
                let mut vars = vec![format!("a={app}")];
                if let Some(kind) = kind {
                    vars.push(format!("type={}", kind.as_str()));
                }
                ("cf-init", vars)
            }
            CfAction::List => ("cf-list", vec![]),
            CfAction::Build(a) => ("cf-build", app(a)),
            CfAction::Push(a) => ("cf-push", app(a)),
            CfAction::Publish(a) => ("cf-publish", app(a)),
            CfAction::Images(a) => ("cf-images", app(a)),
            CfAction::SyncImage(a) => ("cf-sync-image", app(a)),
            CfAction::Install(a) => ("cf-install", app(a)),
            CfAction::Dev(a) => ("cf-dev", app(a)),
            CfAction::DeployPreview(a) => ("cf-deploy-preview", app(a)),
            CfAction::Deploy(a) => ("cf-deploy", app(a)),
            CfAction::DeployAll => ("cf-deploy-all", vec![]),
            CfAction::Logs(a) => ("cf-logs", app(a)),
            CfAction::LogsPreview(a) => ("cf-logs-preview", app(a)),
            CfAction::Restart(a) => ("cf-restart", app(a)),
            CfAction::ContainerStatus(a) => ("cf-container-status", app(a)),
        }
    }
}

impl CfCommand {
    pub async fn run(&self) -> Result<()> {
        let root = find_project_root()?;
        if !root.join("make/cloudflare.mk").is_file() {
            bail!(
                "This project has no Cloudflare make module (make/cloudflare.mk), so there \
                 are no cf-* targets to run.\n\n\
                 Create projects with Cloudflare support via: mx new <name> --infra cloudflare\n\
                 Project root: {}",
                root.display()
            );
        }
        let (target, vars) = self.action.make_invocation();
        run_make_in_project(&root, target, &vars, self.verbose)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Harness {
        #[command(subcommand)]
        action: CfAction,
    }

    fn invocation(args: &[&str]) -> (&'static str, Vec<String>) {
        let h = Harness::try_parse_from(std::iter::once("cf").chain(args.iter().copied()))
            .expect("parse");
        h.action.make_invocation()
    }

    #[test]
    fn deploy_maps_app_to_make_a_var() {
        assert_eq!(
            invocation(&["deploy", "api.example.com"]),
            ("cf-deploy", vec!["a=api.example.com".to_string()])
        );
    }

    #[test]
    fn init_passes_type_when_given() {
        assert_eq!(
            invocation(&["init", "job.example.com", "--type", "cron"]),
            (
                "cf-init",
                vec!["a=job.example.com".to_string(), "type=cron".to_string()]
            )
        );
        assert_eq!(
            invocation(&["init", "job.example.com"]),
            ("cf-init", vec!["a=job.example.com".to_string()])
        );
    }

    #[test]
    fn config_alias_runs_cf_vars() {
        assert_eq!(invocation(&["config"]), ("cf-vars", vec![]));
    }

    #[test]
    fn kebab_names_match_make_targets() {
        assert_eq!(invocation(&["check-credentials"]).0, "cf-check-credentials");
        assert_eq!(
            invocation(&["container-status", "x.example.com"]).0,
            "cf-container-status"
        );
        assert_eq!(invocation(&["deploy-all"]).0, "cf-deploy-all");
    }

    /// Every target the wrapper names must exist in the shipped make module.
    #[test]
    fn every_mapped_target_exists_in_cloudflare_mk() {
        let mk = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../templates/make/cloudflare.mk"
        ))
        .expect("read templates/make/cloudflare.mk");
        let cases: &[&[&str]] = &[
            &["setup"],
            &["vars"],
            &["check-credentials"],
            &["login"],
            &["whoami"],
            &["status"],
            &["init", "a"],
            &["list"],
            &["build", "a"],
            &["push", "a"],
            &["publish", "a"],
            &["images", "a"],
            &["sync-image", "a"],
            &["install", "a"],
            &["dev", "a"],
            &["deploy-preview", "a"],
            &["deploy", "a"],
            &["deploy-all"],
            &["logs", "a"],
            &["logs-preview", "a"],
            &["restart", "a"],
            &["container-status", "a"],
        ];
        for args in cases {
            let (target, _) = invocation(args);
            assert!(
                mk.lines().any(|l| l.starts_with(&format!("{target}:"))),
                "make/cloudflare.mk has no `{target}:` target"
            );
        }
    }
}
