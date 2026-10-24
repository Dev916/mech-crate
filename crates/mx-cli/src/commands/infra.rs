//! `mx infra` command - Manage infrastructure providers

use anyhow::Result;
use clap::{Args, Subcommand};
use console::style;
use dialoguer::{Input, Password};

use mx_lib::config::MechCrateConfig;
use mx_lib::infra::{InfraConfig, InfraProvider, Resolution, DEPRECATED_LINK_MARKER, LINK_FLAG};
use mx_lib::project::ProjectDetector;

/// Body of the global Cloudflare credentials file
/// (`~/.mech-crate/config/infra/cloudflare.env`).
///
/// One canonical name per credential: `CLOUDFLARE_ACCOUNT_ID` and
/// `CLOUDFLARE_API_TOKEN` are the names wrangler itself reads from the
/// environment, and the names `templates/make/cloudflare.mk` plus
/// `templates/scripts/cf-init-app.sh` resolve. The older `CF_ACCOUNT_ID` spelling
/// is still *read* by those consumers as a deprecated alias, but nothing writes
/// it any more: two names for one credential is what made infra-managed
/// credentials invisible to the deploy toolchain (bd:mech-crate-wd9).
fn cloudflare_config_body(account_id: &str, api_token: &str) -> String {
    format!(
        "# Cloudflare Configuration\n\
         # Global credentials, written by `mx infra setup cloudflare`.\n\
         # make/cloudflare.mk reads this file first, then the project's own\n\
         # infra/cloudflare/.env.cloudflare, so a project-local value wins.\n\
         CLOUDFLARE_ACCOUNT_ID={}\n\
         CLOUDFLARE_API_TOKEN={}\n",
        account_id, api_token
    )
}

/// Manage infrastructure providers
#[derive(Args, Debug)]
pub struct InfraCommand {
    #[command(subcommand)]
    command: InfraSubcommand,
}

#[derive(Subcommand, Debug)]
enum InfraSubcommand {
    /// Setup a provider
    Setup {
        /// Provider name
        provider: Option<String>,
    },
    /// List configured providers
    List,
    /// Alias for list
    Ls,
    /// Show provider details
    Inspect {
        /// Provider name
        provider: String,
    },
    /// Link project to global config
    Link {
        /// Provider name
        provider: String,
    },
    /// Unlink project from global config
    Unlink {
        /// Provider name
        provider: String,
    },
    /// Remove provider configuration
    Remove {
        /// Provider name
        provider: String,
    },
}

impl InfraCommand {
    pub async fn run(&self) -> Result<()> {
        let config = MechCrateConfig::new()?;

        // Try to get project root, but don't fail if not in a project
        let project_root = ProjectDetector::new().find_root_from_cwd().ok();

        let infra = if let Some(ref root) = project_root {
            InfraConfig::new(config).with_project(root)
        } else {
            InfraConfig::new(config)
        };

        match &self.command {
            InfraSubcommand::Setup { provider } => self.setup(&infra, provider.as_deref()).await,
            InfraSubcommand::List | InfraSubcommand::Ls => self.list(&infra).await,
            InfraSubcommand::Inspect { provider } => self.inspect(&infra, provider).await,
            InfraSubcommand::Link { provider } => self.link(&infra, provider).await,
            InfraSubcommand::Unlink { provider } => self.unlink(&infra, provider).await,
            InfraSubcommand::Remove { provider } => self.remove(&infra, provider).await,
        }
    }

    async fn setup(&self, infra: &InfraConfig, provider_name: Option<&str>) -> Result<()> {
        let provider = match provider_name {
            Some(name) => InfraProvider::parse_name(name)
                .ok_or_else(|| anyhow::anyhow!("Unknown provider: {}", name))?,
            None => {
                // Prompt for provider selection
                let providers: Vec<&str> = InfraProvider::all().iter().map(|p| p.name()).collect();

                let selection = dialoguer::Select::new()
                    .with_prompt("Select provider to configure")
                    .items(&providers)
                    .default(0)
                    .interact()?;

                InfraProvider::all()[selection]
            }
        };

        println!(
            "{} Setting up {}",
            style("→").cyan().bold(),
            style(provider.name()).green()
        );

        match provider {
            InfraProvider::Cloudflare => self.setup_cloudflare(infra).await,
            InfraProvider::DigitalOcean => self.setup_digitalocean(infra).await,
            InfraProvider::Aws => self.setup_aws(infra).await,
            InfraProvider::Hetzner => self.setup_hetzner(infra).await,
        }
    }

    async fn setup_cloudflare(&self, infra: &InfraConfig) -> Result<()> {
        let account_id: String = Input::new()
            .with_prompt("Cloudflare Account ID")
            .interact_text()?;

        let api_token: String = Password::new()
            .with_prompt("Cloudflare API Token")
            .interact()?;

        let path = infra.global_config_path(InfraProvider::Cloudflare);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, cloudflare_config_body(&account_id, &api_token))?;

        println!(
            "{} Cloudflare configured: {}",
            style("✓").green().bold(),
            path.display()
        );
        println!(
            "  {} reads this file first, then the project's own \
             infra/cloudflare/.env.cloudflare, so a project value wins.",
            style("make cf-*").cyan()
        );

        Ok(())
    }

    async fn setup_digitalocean(&self, infra: &InfraConfig) -> Result<()> {
        let api_token: String = Password::new()
            .with_prompt("DigitalOcean API Token")
            .interact()?;

        let spaces_key: String = Input::new()
            .with_prompt("Spaces Access Key (optional)")
            .allow_empty(true)
            .interact_text()?;

        let spaces_secret: String = if !spaces_key.is_empty() {
            Password::new()
                .with_prompt("Spaces Secret Key")
                .interact()?
        } else {
            String::new()
        };

        let mut config = format!(
            "# DigitalOcean Configuration\n\
             DO_API_TOKEN={}\n",
            api_token
        );

        if !spaces_key.is_empty() {
            config.push_str(&format!(
                "DO_SPACES_KEY={}\n\
                 DO_SPACES_SECRET={}\n",
                spaces_key, spaces_secret
            ));
        }

        let path = infra.global_config_path(InfraProvider::DigitalOcean);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, config)?;

        println!(
            "{} DigitalOcean configured: {}",
            style("✓").green().bold(),
            path.display()
        );

        Ok(())
    }

    async fn setup_aws(&self, infra: &InfraConfig) -> Result<()> {
        let access_key: String = Input::new()
            .with_prompt("AWS Access Key ID")
            .interact_text()?;

        let secret_key: String = Password::new()
            .with_prompt("AWS Secret Access Key")
            .interact()?;

        let region: String = Input::new()
            .with_prompt("AWS Region")
            .default("us-east-1".to_string())
            .interact_text()?;

        let config = format!(
            "# AWS Configuration\n\
             AWS_ACCESS_KEY_ID={}\n\
             AWS_SECRET_ACCESS_KEY={}\n\
             AWS_REGION={}\n",
            access_key, secret_key, region
        );

        let path = infra.global_config_path(InfraProvider::Aws);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, config)?;

        println!(
            "{} AWS configured: {}",
            style("✓").green().bold(),
            path.display()
        );

        Ok(())
    }

    async fn setup_hetzner(&self, infra: &InfraConfig) -> Result<()> {
        let api_token: String = Password::new()
            .with_prompt("Hetzner API Token")
            .interact()?;

        let location: String = Input::new()
            .with_prompt("Default Location")
            .default("nbg1".to_string())
            .interact_text()?;

        let config = format!(
            "# Hetzner Configuration\n\
             HETZNER_API_TOKEN={}\n\
             HETZNER_LOCATION={}\n",
            api_token, location
        );

        let path = infra.global_config_path(InfraProvider::Hetzner);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, config)?;

        println!(
            "{} Hetzner configured: {}",
            style("✓").green().bold(),
            path.display()
        );

        Ok(())
    }

    async fn list(&self, infra: &InfraConfig) -> Result<()> {
        let providers = infra.list_configured();

        println!("{}", style("Infrastructure Providers").bold());
        println!("{}", style("─".repeat(40)).dim());

        for (provider, global, project) in &providers {
            let status = if infra.is_linked(*provider) {
                style("global (project linked)").cyan()
            } else if *project {
                style("project").green()
            } else if *global {
                style("global").cyan()
            } else {
                style("not configured").dim()
            };

            println!(
                "  {} {} - {}",
                style("•").cyan(),
                style(provider.name()).bold(),
                status
            );
        }

        Ok(())
    }

    async fn inspect(&self, infra: &InfraConfig, provider_name: &str) -> Result<()> {
        let provider = parse_provider(provider_name)?;
        let resolution = infra.resolve(provider)?;

        if let Some(marker) = &resolution.deprecated_marker {
            eprintln!("{}", deprecated_marker_warning(marker, provider));
        }

        println!(
            "{} {}",
            style("Configuration for").bold(),
            style(provider.name()).green()
        );
        println!("{}", style("─".repeat(40)).dim());
        println!("{}", render_header(&resolution));
        println!();
        println!("{}", render_keys(&resolution));

        if !resolution.is_configured() {
            let why = if resolution.linked && !resolution.global_exists {
                "the project is linked to global, but there is no global config"
            } else {
                "no scope supplies a value"
            };
            anyhow::bail!(
                "{} is not configured ({}): run `mx infra setup {}`",
                provider.name(),
                why,
                provider.name()
            );
        }

        Ok(())
    }

    async fn link(&self, infra: &InfraConfig, provider_name: &str) -> Result<()> {
        let provider = parse_provider(provider_name)?;
        require_project(infra, provider)?;

        let outcome = infra.link(provider)?;
        if outcome.already_linked {
            println!(
                "{} {} is already linked to the global config ({})",
                style("✓").green().bold(),
                provider.name(),
                outcome.project_path.display()
            );
        } else {
            println!(
                "{} Linked {} to the global config: {}=true in {}",
                style("✓").green().bold(),
                provider.name(),
                LINK_FLAG,
                outcome.project_path.display()
            );
            println!(
                "  Credentials now resolve from the environment, then {}; this project's own \
                 values are ignored (kept, and back in effect after `mx infra unlink {}`).",
                infra.global_config_path(provider).display(),
                provider.name()
            );
        }
        if let Some(marker) = &outcome.removed_deprecated_marker {
            println!(
                "  Removed the deprecated {} marker {}; the link is now the {} line.",
                DEPRECATED_LINK_MARKER,
                marker.display(),
                LINK_FLAG
            );
        }
        Ok(())
    }

    async fn unlink(&self, infra: &InfraConfig, provider_name: &str) -> Result<()> {
        let provider = parse_provider(provider_name)?;
        require_project(infra, provider)?;

        let outcome = infra.unlink(provider)?;
        if !outcome.was_linked {
            println!(
                "{} {} is not linked; nothing to do",
                style("✓").green().bold(),
                provider.name()
            );
        } else if outcome.removed_file {
            println!(
                "{} Unlinked {}: removed {} (it held only the link)",
                style("✓").green().bold(),
                provider.name(),
                outcome.project_path.display()
            );
        } else {
            println!(
                "{} Unlinked {}: removed {} from {}; its own values apply again",
                style("✓").green().bold(),
                provider.name(),
                LINK_FLAG,
                outcome.project_path.display()
            );
        }
        if let Some(marker) = &outcome.removed_deprecated_marker {
            println!(
                "  Removed the deprecated {} marker {}.",
                DEPRECATED_LINK_MARKER,
                marker.display()
            );
        }
        Ok(())
    }

    async fn remove(&self, infra: &InfraConfig, provider_name: &str) -> Result<()> {
        let provider = parse_provider(provider_name)?;

        let path = infra.global_config_path(provider);
        if path.exists() {
            std::fs::remove_file(&path)?;
            println!(
                "{} Removed {} configuration",
                style("✓").green().bold(),
                provider.name()
            );
        } else {
            println!("{} {} not configured", style("!").yellow(), provider.name());
        }

        Ok(())
    }
}

fn parse_provider(name: &str) -> Result<InfraProvider> {
    InfraProvider::parse_name(name).ok_or_else(|| {
        let known: Vec<&str> = InfraProvider::all().iter().map(|p| p.name()).collect();
        anyhow::anyhow!("Unknown provider: {} (known: {})", name, known.join(", "))
    })
}

/// Link state lives in a project file, so link/unlink need a project root.
fn require_project(infra: &InfraConfig, provider: InfraProvider) -> Result<()> {
    if infra.project_config_path(provider).is_none() {
        anyhow::bail!(
            "not inside an mx project: `mx infra link`/`unlink` record the link in the \
             project's infra/{}/.env.{}; run it from the project root or below",
            provider.name(),
            provider.name()
        );
    }
    Ok(())
}

fn deprecated_marker_warning(marker: &std::path::Path, provider: InfraProvider) -> String {
    format!(
        "warning: {} is a deprecated link marker and is ignored. The link is now the line \
         {}=true in the project's .env.{}; run `mx infra link {}` to migrate (it removes the \
         marker) or delete the marker.",
        marker.display(),
        LINK_FLAG,
        provider.name(),
        provider.name()
    )
}

fn is_secret(key: &str) -> bool {
    key.contains("SECRET") || key.contains("TOKEN") || key.contains("KEY")
}

/// Link state and the two scope paths (plain text, no styling).
fn render_header(r: &Resolution) -> String {
    let presence = |exists: bool| if exists { "present" } else { "missing" };
    let mut out = Vec::new();
    let link = match (&r.project_path, r.linked) {
        (None, _) => "no project (outside an mx project: global config only)".to_string(),
        (Some(p), true) => format!(
            "linked to global ({}=true in {}): project values ignored",
            LINK_FLAG,
            p.display()
        ),
        (Some(_), false) => "not linked (project values win over global)".to_string(),
    };
    out.push(format!("  link:    {}", link));
    out.push(format!(
        "  global:  {} ({})",
        r.global_path.display(),
        presence(r.global_exists)
    ));
    if let Some(p) = &r.project_path {
        out.push(format!(
            "  project: {} ({})",
            p.display(),
            presence(r.project_exists)
        ));
    }
    out.join("\n")
}

/// One line per resolved key: `KEY = value  [source]`, secrets masked.
fn render_keys(r: &Resolution) -> String {
    let width = r.keys.iter().map(|k| k.key.len()).max().unwrap_or(0);
    r.keys
        .iter()
        .map(|k| {
            let (value, source) = match (&k.value, k.scope) {
                (Some(v), Some(scope)) => {
                    let shown = if is_secret(&k.key) {
                        "********".to_string()
                    } else {
                        v.clone()
                    };
                    let source = match k.deprecated_alias() {
                        Some(alias) => format!("[{}, deprecated alias {}]", scope.as_str(), alias),
                        None => format!("[{}]", scope.as_str()),
                    };
                    (shown, source)
                }
                _ => ("(not set)".to_string(), String::new()),
            };
            format!("  {:width$} = {}  {}", k.key, value, source, width = width)
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::cloudflare_config_body;
    use super::{render_header, render_keys};
    use mx_lib::infra::{CredentialScope, InfraProvider, Resolution, ResolvedKey};

    fn key(k: &str, v: Option<&str>, scope: Option<CredentialScope>, read_as: &str) -> ResolvedKey {
        ResolvedKey {
            key: k.to_string(),
            value: v.map(str::to_string),
            scope,
            read_as: v.map(|_| read_as.to_string()),
        }
    }

    fn resolution(linked: bool, keys: Vec<ResolvedKey>) -> Resolution {
        Resolution {
            provider: InfraProvider::Cloudflare,
            global_path: "/h/.mech-crate/config/infra/cloudflare.env".into(),
            global_exists: true,
            project_path: Some("/p/infra/cloudflare/.env.cloudflare".into()),
            project_exists: true,
            linked,
            deprecated_marker: None,
            keys,
        }
    }

    /// bd:mech-crate-066: inspect names each key's source, tags a deprecated
    /// alias, masks secrets, and shows an unresolved key rather than hiding it.
    #[test]
    fn inspect_rendering_names_sources_and_masks_secrets() {
        let r = resolution(
            false,
            vec![
                key(
                    "CLOUDFLARE_ACCOUNT_ID",
                    Some("acct"),
                    Some(CredentialScope::Project),
                    "CF_ACCOUNT_ID",
                ),
                key(
                    "CLOUDFLARE_API_TOKEN",
                    Some("tok-secret"),
                    Some(CredentialScope::Global),
                    "CLOUDFLARE_API_TOKEN",
                ),
                key("CF_DOCKER_PLATFORM", None, None, ""),
            ],
        );
        let out = render_keys(&r);
        assert!(
            out.contains("CLOUDFLARE_ACCOUNT_ID = acct  [project, deprecated alias CF_ACCOUNT_ID]"),
            "{out}"
        );
        assert!(
            out.contains("CLOUDFLARE_API_TOKEN  = ********  [global]"),
            "{out}"
        );
        assert!(!out.contains("tok-secret"));
        assert!(out.contains("CF_DOCKER_PLATFORM    = (not set)"), "{out}");
    }

    #[test]
    fn inspect_header_shows_the_link_state() {
        assert!(render_header(&resolution(true, vec![])).contains("linked to global"));
        assert!(render_header(&resolution(false, vec![])).contains("not linked"));
    }

    /// `KEY=VALUE` assignments of a credentials file body, comments skipped.
    fn assignments(body: &str) -> Vec<(String, String)> {
        body.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| {
                let (k, v) = l
                    .split_once('=')
                    .unwrap_or_else(|| panic!("non-assignment line in credentials body: {l:?}"));
                (k.to_string(), v.to_string())
            })
            .collect()
    }

    /// bd:mech-crate-wd9: the writer must emit the canonical names the deploy
    /// toolchain resolves. Writing `CF_ACCOUNT_ID` here (or in addition) is what
    /// made `mx infra setup cloudflare` credentials invisible to `make cf-*`.
    #[test]
    fn cloudflare_writer_emits_the_canonical_credential_names() {
        let pairs = assignments(&cloudflare_config_body("acct-123", "tok-456"));

        assert_eq!(
            pairs,
            vec![
                ("CLOUDFLARE_ACCOUNT_ID".to_string(), "acct-123".to_string()),
                ("CLOUDFLARE_API_TOKEN".to_string(), "tok-456".to_string()),
            ],
            "the global cloudflare.env must carry exactly the canonical pair, in order"
        );
    }

    /// The deprecated alias is read by the consumers but never written, so the
    /// file can never disagree with itself about which name is authoritative.
    #[test]
    fn cloudflare_writer_never_emits_the_deprecated_alias() {
        let body = cloudflare_config_body("acct-123", "tok-456");
        let account_id_keys: Vec<String> = assignments(&body)
            .into_iter()
            .map(|(k, _)| k)
            .filter(|k| k.ends_with("ACCOUNT_ID"))
            .collect();

        assert_eq!(
            account_id_keys,
            vec!["CLOUDFLARE_ACCOUNT_ID".to_string()],
            "exactly one account-id variable may be written; found {account_id_keys:?}"
        );
    }

    /// A value is written verbatim: no quoting, no trailing whitespace, nothing a
    /// makefile `-include` or a `source` would mangle.
    #[test]
    fn cloudflare_writer_round_trips_values_verbatim() {
        let body = cloudflare_config_body("0123abc-def", "v1.0-AbC_123");
        for (key, value) in assignments(&body) {
            assert!(
                !value.starts_with('"') && !value.starts_with('\''),
                "{key} must not be quoted, got {value:?}"
            );
            assert_eq!(value.trim(), value, "{key} must not carry stray whitespace");
        }
        assert!(
            body.ends_with('\n'),
            "the credentials body must end with a newline"
        );
    }
}
