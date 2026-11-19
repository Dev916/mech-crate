//! `mx aws` - the AWS lane proxied to the project's `make aws-*` targets
//!
//! Same shape as `mx cf`: find the project root, run one make target there.
//! Every subcommand maps 1:1 to an `aws-<name>` target in `make/aws.mk`. The
//! two lanes stay where they are enforced, in the make module: the emulator
//! targets force fake credentials, and the sandbox targets refuse to run with
//! an emulator endpoint exported or against the wrong account. This wrapper
//! adds nothing to either guard.

use anyhow::{bail, Result};
use clap::{Args, Subcommand};

use super::dev::{find_project_root, run_make_in_project};

/// Local AWS emulator and sandbox-account gate (proxied to `make aws-*`)
#[derive(Args, Debug)]
pub struct AwsCommand {
    #[command(subcommand)]
    action: AwsAction,

    /// Show the make invocation before running it
    #[arg(short, long)]
    verbose: bool,
}

/// A command line passed through as make's `c=` variable.
#[derive(Args, Debug)]
pub struct PassThrough {
    /// The command and its arguments
    #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
    command: Vec<String>,
}

impl PassThrough {
    fn as_make_var(&self) -> String {
        format!("c={}", self.command.join(" "))
    }
}

#[derive(Subcommand, Debug)]
pub enum AwsAction {
    /// Start the local AWS emulator in this project's compose project
    Up,
    /// Stop the local AWS emulator
    Down {
        /// Also drop the emulator's state volume
        #[arg(long)]
        volumes: bool,
    },
    /// Tail the local AWS emulator logs
    Logs,
    /// Wait for the local AWS emulator to answer
    Health,
    /// Recreate the local AWS emulator with empty state
    Reset,
    /// Print exports for the emulator lane: eval "$(mx aws env)"
    Env,
    /// Run the AWS CLI against the emulator, e.g. `mx aws cli s3 ls`
    Cli(PassThrough),
    /// Show the resolved sandbox variables and their source (no secrets)
    #[command(visible_alias = "config")]
    Vars,
    /// Refuse unless the caller is the configured sandbox account
    CheckSandbox,
    /// Run a command against the sandbox account, e.g. `mx aws sandbox terraform plan`
    Sandbox(PassThrough),
}

impl AwsAction {
    /// The make target and its variables for this action.
    fn make_invocation(&self) -> (&'static str, Vec<String>) {
        match self {
            AwsAction::Up => ("aws-up", vec![]),
            AwsAction::Down { volumes } => (
                "aws-down",
                if *volumes {
                    vec!["v=1".to_string()]
                } else {
                    vec![]
                },
            ),
            AwsAction::Logs => ("aws-logs", vec![]),
            AwsAction::Health => ("aws-health", vec![]),
            AwsAction::Reset => ("aws-reset", vec![]),
            AwsAction::Env => ("aws-env", vec![]),
            AwsAction::Cli(p) => ("aws-cli", vec![p.as_make_var()]),
            AwsAction::Vars => ("aws-vars", vec![]),
            AwsAction::CheckSandbox => ("aws-check-sandbox", vec![]),
            AwsAction::Sandbox(p) => ("aws-sandbox", vec![p.as_make_var()]),
        }
    }
}

impl AwsCommand {
    pub async fn run(&self) -> Result<()> {
        let root = find_project_root()?;
        if !root.join("make/aws.mk").is_file() {
            bail!(
                "This project has no AWS make module (make/aws.mk), so there are no \
                 aws-* targets to run.\n\n\
                 Create projects with the AWS lane via: mx new <name> --infra aws\n\
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
        action: AwsAction,
    }

    fn invocation(args: &[&str]) -> (&'static str, Vec<String>) {
        let h = Harness::try_parse_from(std::iter::once("aws").chain(args.iter().copied()))
            .expect("parse");
        h.action.make_invocation()
    }

    #[test]
    fn cli_joins_its_arguments_into_the_c_var_including_flags() {
        assert_eq!(
            invocation(&["cli", "s3", "ls", "--recursive", "s3://uploads"]),
            (
                "aws-cli",
                vec!["c=s3 ls --recursive s3://uploads".to_string()]
            )
        );
    }

    #[test]
    fn sandbox_passes_a_whole_command_line() {
        assert_eq!(
            invocation(&["sandbox", "terraform", "plan", "-out", "tf.plan"]),
            (
                "aws-sandbox",
                vec!["c=terraform plan -out tf.plan".to_string()]
            )
        );
    }

    #[test]
    fn cli_and_sandbox_require_a_command() {
        for sub in ["cli", "sandbox"] {
            assert!(
                Harness::try_parse_from(["aws", sub]).is_err(),
                "`mx aws {sub}` with no command must be a usage error"
            );
        }
    }

    #[test]
    fn down_volumes_maps_to_v_1() {
        assert_eq!(invocation(&["down"]), ("aws-down", vec![]));
        assert_eq!(
            invocation(&["down", "--volumes"]),
            ("aws-down", vec!["v=1".to_string()])
        );
    }

    #[test]
    fn config_alias_runs_aws_vars() {
        assert_eq!(invocation(&["config"]), ("aws-vars", vec![]));
    }

    /// Every target the wrapper names must exist in the shipped make module.
    #[test]
    fn every_mapped_target_exists_in_aws_mk() {
        let mk = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../templates/make/aws.mk"
        ))
        .expect("read templates/make/aws.mk");
        let cases: &[&[&str]] = &[
            &["up"],
            &["down"],
            &["logs"],
            &["health"],
            &["reset"],
            &["env"],
            &["cli", "x"],
            &["vars"],
            &["check-sandbox"],
            &["sandbox", "x"],
        ];
        for args in cases {
            let (target, _) = invocation(args);
            assert!(
                mk.lines().any(|l| l.starts_with(&format!("{target}:"))),
                "make/aws.mk has no `{target}:` target"
            );
        }
    }
}
