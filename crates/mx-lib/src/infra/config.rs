//! Infrastructure configuration management
//!
//! Two credential scopes per provider:
//!
//! - global  `~/.mech-crate/config/infra/<provider>.env` (`mx infra setup`)
//! - project `<root>/infra/<provider>/.env.<provider>`  (e.g. `make cf-setup`)
//!
//! plus the process environment above both. Each credential resolves on its
//! own, highest first: environment, then project, then global, the canonical
//! name before a deprecated alias within a scope. That is the order
//! `templates/make/cloudflare.mk` implements (`make cf-vars`), so
//! [`InfraConfig::resolve`] and the deploy toolchain always agree.
//!
//! **Linking** (bd:mech-crate-066). The one link mechanism is the line
//! `MX_INFRA_USE_GLOBAL=true` inside the project file. A linked project's file
//! is not read for values, so credentials resolve from the environment, then
//! the global file. `mx infra link` writes the flag (keeping whatever else the
//! file holds, so `unlink` restores it), and `cloudflare.mk`, `cf-init-app.sh`
//! and this resolver all read that same line. The older `.env.linked` marker
//! file was only ever read by this resolver and written by nothing; it is no
//! longer honored, and [`Resolution::deprecated_marker`] reports a stray one.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::MechCrateConfig;
use crate::error::{Error, Result};

/// The key, inside a project credentials file, that links it to the global one.
pub const LINK_FLAG: &str = "MX_INFRA_USE_GLOBAL";

/// The deprecated link marker file, beside the project credentials file.
pub const DEPRECATED_LINK_MARKER: &str = ".env.linked";

/// Comment `mx infra link` writes above the flag, removed again by `unlink`.
const LINK_COMMENT: &str =
    "# Linked to the global config by `mx infra link`; `mx infra unlink` removes this.";

/// Supported infrastructure providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InfraProvider {
    Cloudflare,
    DigitalOcean,
    Aws,
    Hetzner,
}

impl InfraProvider {
    /// Get all available providers
    pub fn all() -> &'static [InfraProvider] {
        &[
            InfraProvider::Cloudflare,
            InfraProvider::DigitalOcean,
            InfraProvider::Aws,
            InfraProvider::Hetzner,
        ]
    }

    /// Get the provider name as a string
    pub fn name(&self) -> &'static str {
        match self {
            InfraProvider::Cloudflare => "cloudflare",
            InfraProvider::DigitalOcean => "digitalocean",
            InfraProvider::Aws => "aws",
            InfraProvider::Hetzner => "hetzner",
        }
    }

    /// Parse a provider from its name or a known alias
    pub fn parse_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cloudflare" | "cf" => Some(InfraProvider::Cloudflare),
            "digitalocean" | "do" => Some(InfraProvider::DigitalOcean),
            "aws" | "amazon" => Some(InfraProvider::Aws),
            "hetzner" | "hz" => Some(InfraProvider::Hetzner),
            _ => None,
        }
    }

    /// Get the config file name for this provider
    pub fn config_filename(&self) -> &'static str {
        match self {
            InfraProvider::Cloudflare => "cloudflare.env",
            InfraProvider::DigitalOcean => "digitalocean.env",
            InfraProvider::Aws => "aws.env",
            InfraProvider::Hetzner => "hetzner.env",
        }
    }

    /// The credentials this provider's toolchain resolves, as
    /// `(canonical name, deprecated aliases)`, in display order.
    ///
    /// These are the keys that take part in the environment scope. For
    /// Cloudflare they are exactly the pair `cloudflare.mk` resolves, aliases
    /// included (bd:mech-crate-wd9).
    pub fn credential_keys(&self) -> &'static [(&'static str, &'static [&'static str])] {
        match self {
            InfraProvider::Cloudflare => &[
                ("CLOUDFLARE_ACCOUNT_ID", &["CF_ACCOUNT_ID"]),
                ("CLOUDFLARE_API_TOKEN", &["CF_API_TOKEN"]),
            ],
            InfraProvider::DigitalOcean => &[
                ("DO_API_TOKEN", &[]),
                ("DO_SPACES_KEY", &[]),
                ("DO_SPACES_SECRET", &[]),
            ],
            InfraProvider::Aws => &[
                ("AWS_ACCESS_KEY_ID", &[]),
                ("AWS_SECRET_ACCESS_KEY", &[]),
                ("AWS_REGION", &[]),
            ],
            InfraProvider::Hetzner => &[("HETZNER_API_TOKEN", &[]), ("HETZNER_LOCATION", &[])],
        }
    }
}

/// Where a resolved value came from. The names match `make cf-vars`'s
/// `CLOUDFLARE_ACCOUNT_ID_SOURCE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialScope {
    Environment,
    Project,
    Global,
}

impl CredentialScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            CredentialScope::Environment => "environment",
            CredentialScope::Project => "project",
            CredentialScope::Global => "global",
        }
    }
}

/// One resolved key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedKey {
    /// Canonical key name.
    pub key: String,
    /// Resolved value; `None` when no scope supplies it.
    pub value: Option<String>,
    /// Scope that supplied the value.
    pub scope: Option<CredentialScope>,
    /// The name actually read, which differs from `key` for a deprecated alias.
    pub read_as: Option<String>,
}

impl ResolvedKey {
    /// The deprecated alias the value was read under, if any.
    pub fn deprecated_alias(&self) -> Option<&str> {
        self.read_as.as_deref().filter(|name| *name != self.key)
    }
}

/// The resolved credential set for one provider.
#[derive(Debug, Clone)]
pub struct Resolution {
    pub provider: InfraProvider,
    pub global_path: PathBuf,
    pub global_exists: bool,
    /// `None` outside a project.
    pub project_path: Option<PathBuf>,
    pub project_exists: bool,
    /// The project file carries `MX_INFRA_USE_GLOBAL=true`.
    pub linked: bool,
    /// A stray `.env.linked` marker, which is no longer honored.
    pub deprecated_marker: Option<PathBuf>,
    /// Credential keys first (provider order), then any other keys the read
    /// files carry, sorted.
    pub keys: Vec<ResolvedKey>,
}

impl Resolution {
    pub fn get(&self, key: &str) -> Option<&ResolvedKey> {
        self.keys.iter().find(|k| k.key == key)
    }

    /// At least one key resolved to a value.
    pub fn is_configured(&self) -> bool {
        self.keys.iter().any(|k| k.value.is_some())
    }
}

/// Outcome of [`InfraConfig::link`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkOutcome {
    pub project_path: PathBuf,
    pub already_linked: bool,
    pub created_file: bool,
    pub removed_deprecated_marker: Option<PathBuf>,
}

/// Outcome of [`InfraConfig::unlink`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnlinkOutcome {
    pub project_path: PathBuf,
    pub was_linked: bool,
    pub removed_file: bool,
    pub removed_deprecated_marker: Option<PathBuf>,
}

/// `KEY=VALUE` assignments of an env file body, in order, comments and blank
/// lines skipped, surrounding quotes stripped from values.
pub fn parse_env_body(body: &str) -> Vec<(String, String)> {
    body.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let line = line.strip_prefix("export ").unwrap_or(line);
            let (key, value) = line.split_once('=')?;
            let value = value.trim().trim_matches('"').trim_matches('\'');
            Some((key.trim().to_string(), value.to_string()))
        })
        .collect()
}

/// Whether an env file body links its project to the global config: a line
/// that is exactly `MX_INFRA_USE_GLOBAL=true` (surrounding whitespace
/// allowed). `cloudflare.mk` and `cf-init-app.sh` use the same exact match.
pub fn body_is_linked(body: &str) -> bool {
    body.lines().any(is_link_line)
}

fn is_link_line(line: &str) -> bool {
    line.trim() == format!("{LINK_FLAG}=true")
}

fn is_flag_assignment(line: &str) -> bool {
    line.trim()
        .split_once('=')
        .map(|(k, _)| k.trim() == LINK_FLAG)
        .unwrap_or(false)
}

/// Add the link flag to an env file body. Returns `None` when it is already
/// linked. Any other value of the flag is replaced; every other line is kept.
pub fn set_link_flag(body: &str) -> Option<String> {
    if body_is_linked(body) {
        return None;
    }
    let kept: Vec<&str> = body.lines().filter(|l| !is_flag_assignment(l)).collect();
    let mut out = kept.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(LINK_COMMENT);
    out.push('\n');
    out.push_str(LINK_FLAG);
    out.push_str("=true\n");
    Some(out)
}

/// Remove the link flag (and the comment `set_link_flag` wrote) from an env
/// file body. The legacy `MX_INFRA_PROVIDER` / `MX_INFRA_LINKED_AT` lines the
/// old bash `mx infra link` wrote go with it. Returns `None` when not linked.
pub fn clear_link_flag(body: &str) -> Option<String> {
    if !body_is_linked(body) {
        return None;
    }
    let kept: Vec<&str> = body
        .lines()
        .filter(|l| {
            let t = l.trim();
            !(is_flag_assignment(l)
                || t == LINK_COMMENT
                || t.starts_with("MX_INFRA_PROVIDER=")
                || t.starts_with("MX_INFRA_LINKED_AT="))
        })
        .collect();
    let mut out = kept.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    Some(out)
}

/// Infrastructure configuration manager
#[derive(Debug)]
pub struct InfraConfig {
    global_config: MechCrateConfig,
    project_root: Option<PathBuf>,
}

impl InfraConfig {
    /// Create a new infrastructure config manager
    pub fn new(global_config: MechCrateConfig) -> Self {
        Self {
            global_config,
            project_root: None,
        }
    }

    /// Set the project root for project-level config resolution
    pub fn with_project(mut self, project_root: impl AsRef<Path>) -> Self {
        self.project_root = Some(project_root.as_ref().to_path_buf());
        self
    }

    /// Get the global config path for a provider
    pub fn global_config_path(&self, provider: InfraProvider) -> PathBuf {
        self.global_config
            .infra_dir()
            .join(provider.config_filename())
    }

    /// Get the project config path for a provider
    pub fn project_config_path(&self, provider: InfraProvider) -> Option<PathBuf> {
        self.project_root.as_ref().map(|root| {
            root.join("infra")
                .join(provider.name())
                .join(format!(".env.{}", provider.name()))
        })
    }

    /// The deprecated `.env.linked` marker path for a provider, if in a project.
    pub fn deprecated_marker_path(&self, provider: InfraProvider) -> Option<PathBuf> {
        self.project_config_path(provider)
            .and_then(|p| p.parent().map(|d| d.join(DEPRECATED_LINK_MARKER)))
    }

    /// Check if a provider is configured globally
    pub fn is_globally_configured(&self, provider: InfraProvider) -> bool {
        self.global_config_path(provider).exists()
    }

    /// Check if a provider is configured for the project
    pub fn is_project_configured(&self, provider: InfraProvider) -> bool {
        self.project_config_path(provider)
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Whether the project file carries `MX_INFRA_USE_GLOBAL=true`.
    pub fn is_linked(&self, provider: InfraProvider) -> bool {
        self.project_config_path(provider)
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|body| body_is_linked(&body))
            .unwrap_or(false)
    }

    /// The file values are read from when the environment is ignored: the
    /// project file unless it is absent or linked, else the global file.
    pub fn resolve_config_path(&self, provider: InfraProvider) -> Option<PathBuf> {
        if let Some(project_path) = self.project_config_path(provider) {
            if project_path.exists() && !self.is_linked(provider) {
                return Some(project_path);
            }
        }
        let global_path = self.global_config_path(provider);
        global_path.exists().then_some(global_path)
    }

    /// Resolve the provider's credential set against the process environment.
    pub fn resolve(&self, provider: InfraProvider) -> Result<Resolution> {
        self.resolve_with_env(provider, &|name| std::env::var(name).ok())
    }

    /// Resolve with an explicit environment lookup (tests pass a fixed one).
    pub fn resolve_with_env(
        &self,
        provider: InfraProvider,
        env: &dyn Fn(&str) -> Option<String>,
    ) -> Result<Resolution> {
        let global_path = self.global_config_path(provider);
        let global_exists = global_path.exists();
        let global = if global_exists {
            parse_env_body(&std::fs::read_to_string(&global_path)?)
        } else {
            Vec::new()
        };

        let project_path = self.project_config_path(provider);
        let project_exists = project_path.as_ref().map(|p| p.exists()).unwrap_or(false);
        let project_body = match (&project_path, project_exists) {
            (Some(p), true) => std::fs::read_to_string(p)?,
            _ => String::new(),
        };
        let linked = body_is_linked(&project_body);
        // A linked project's file is not read for values.
        let project = if linked {
            Vec::new()
        } else {
            parse_env_body(&project_body)
        };

        let deprecated_marker = self.deprecated_marker_path(provider).filter(|p| p.exists());

        let lookup = |pairs: &[(String, String)], name: &str| -> Option<String> {
            pairs
                .iter()
                .rev()
                .find(|(k, v)| k == name && !v.is_empty())
                .map(|(_, v)| v.clone())
        };

        let mut keys = Vec::new();
        let mut schema_names: Vec<&str> = Vec::new();
        for (canonical, aliases) in provider.credential_keys() {
            schema_names.push(canonical);
            schema_names.extend(aliases.iter().copied());
            let names: Vec<&str> = std::iter::once(*canonical)
                .chain(aliases.iter().copied())
                .collect();

            let mut found = None;
            'scopes: for scope in [
                CredentialScope::Environment,
                CredentialScope::Project,
                CredentialScope::Global,
            ] {
                for name in &names {
                    let value = match scope {
                        CredentialScope::Environment => env(name).filter(|v| !v.is_empty()),
                        CredentialScope::Project => lookup(&project, name),
                        CredentialScope::Global => lookup(&global, name),
                    };
                    if let Some(value) = value {
                        found = Some((value, scope, name.to_string()));
                        break 'scopes;
                    }
                }
            }
            keys.push(match found {
                Some((value, scope, read_as)) => ResolvedKey {
                    key: canonical.to_string(),
                    value: Some(value),
                    scope: Some(scope),
                    read_as: Some(read_as),
                },
                None => ResolvedKey {
                    key: canonical.to_string(),
                    value: None,
                    scope: None,
                    read_as: None,
                },
            });
        }

        // Any other keys the read files carry: project over global. `MX_INFRA_*`
        // keys are link state, not configuration.
        let mut others: HashMap<String, (String, CredentialScope)> = HashMap::new();
        for (pairs, scope) in [
            (&global, CredentialScope::Global),
            (&project, CredentialScope::Project),
        ] {
            for (k, v) in pairs {
                if v.is_empty() || k.starts_with("MX_INFRA_") || schema_names.contains(&k.as_str())
                {
                    continue;
                }
                others.insert(k.clone(), (v.clone(), scope));
            }
        }
        let mut other_keys: Vec<_> = others.into_iter().collect();
        other_keys.sort_by(|a, b| a.0.cmp(&b.0));
        for (key, (value, scope)) in other_keys {
            keys.push(ResolvedKey {
                read_as: Some(key.clone()),
                key,
                value: Some(value),
                scope: Some(scope),
            });
        }

        Ok(Resolution {
            provider,
            global_path,
            global_exists,
            project_path,
            project_exists,
            linked,
            deprecated_marker,
            keys,
        })
    }

    /// Load the resolved configuration for a provider as key-value pairs
    pub fn load_config(&self, provider: InfraProvider) -> Result<HashMap<String, String>> {
        let resolution = self.resolve(provider)?;
        if !resolution.is_configured() {
            return Err(Error::Config(format!("{} not configured", provider.name())));
        }
        Ok(resolution
            .keys
            .into_iter()
            .filter_map(|k| k.value.map(|v| (k.key, v)))
            .collect())
    }

    /// Get a specific configuration value
    pub fn get_value(&self, provider: InfraProvider, key: &str) -> Result<Option<String>> {
        let config = self.load_config(provider)?;
        Ok(config.get(key).cloned())
    }

    /// List all configured providers
    pub fn list_configured(&self) -> Vec<(InfraProvider, bool, bool)> {
        InfraProvider::all()
            .iter()
            .map(|&p| {
                (
                    p,
                    self.is_globally_configured(p),
                    self.is_project_configured(p),
                )
            })
            .collect()
    }

    fn require_project_path(&self, provider: InfraProvider) -> Result<PathBuf> {
        self.project_config_path(provider).ok_or(Error::NotAProject)
    }

    fn remove_deprecated_marker(&self, provider: InfraProvider) -> Result<Option<PathBuf>> {
        match self.deprecated_marker_path(provider) {
            Some(marker) if marker.exists() => {
                std::fs::remove_file(&marker)?;
                Ok(Some(marker))
            }
            _ => Ok(None),
        }
    }

    /// Link the project to the global config: write `MX_INFRA_USE_GLOBAL=true`
    /// into the project file, keeping everything else it holds. Idempotent.
    /// Requires a project and an existing global config.
    pub fn link(&self, provider: InfraProvider) -> Result<LinkOutcome> {
        let project_path = self.require_project_path(provider)?;
        let global_path = self.global_config_path(provider);
        if !global_path.exists() {
            return Err(Error::Config(format!(
                "no global {} config at {}; run `mx infra setup {}` first",
                provider.name(),
                global_path.display(),
                provider.name()
            )));
        }

        let existed = project_path.exists();
        let body = if existed {
            std::fs::read_to_string(&project_path)?
        } else {
            String::new()
        };
        let already_linked = match set_link_flag(&body) {
            None => true,
            Some(new_body) => {
                if let Some(dir) = project_path.parent() {
                    std::fs::create_dir_all(dir)?;
                }
                std::fs::write(&project_path, new_body)?;
                false
            }
        };
        let removed_deprecated_marker = self.remove_deprecated_marker(provider)?;

        Ok(LinkOutcome {
            project_path,
            already_linked,
            created_file: !existed,
            removed_deprecated_marker,
        })
    }

    /// Unlink the project: remove the flag. A project file left with no
    /// assignments (it held only the link) is deleted. Idempotent.
    pub fn unlink(&self, provider: InfraProvider) -> Result<UnlinkOutcome> {
        let project_path = self.require_project_path(provider)?;
        let mut was_linked = false;
        let mut removed_file = false;

        if project_path.exists() {
            let body = std::fs::read_to_string(&project_path)?;
            if let Some(new_body) = clear_link_flag(&body) {
                was_linked = true;
                if parse_env_body(&new_body).is_empty() {
                    std::fs::remove_file(&project_path)?;
                    removed_file = true;
                } else {
                    std::fs::write(&project_path, new_body)?;
                }
            }
        }
        let removed_deprecated_marker = self.remove_deprecated_marker(provider)?;

        Ok(UnlinkOutcome {
            project_path,
            was_linked,
            removed_file,
            removed_deprecated_marker,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn infra(home: &Path, project: Option<&Path>) -> InfraConfig {
        let cfg = MechCrateConfig {
            root: home.join(".mech-crate"),
        };
        match project {
            Some(p) => InfraConfig::new(cfg).with_project(p),
            None => InfraConfig::new(cfg),
        }
    }

    fn write(path: &Path, body: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    fn no_env(_: &str) -> Option<String> {
        None
    }

    fn setup(global: Option<&str>, project: Option<&str>) -> (tempfile::TempDir, InfraConfig) {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let proj = dir.path().join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        let infra = infra(&home, Some(&proj));
        if let Some(body) = global {
            write(&infra.global_config_path(InfraProvider::Cloudflare), body);
        }
        if let Some(body) = project {
            write(
                &infra
                    .project_config_path(InfraProvider::Cloudflare)
                    .unwrap(),
                body,
            );
        }
        (dir, infra)
    }

    #[test]
    fn the_link_flag_is_an_exact_line_match() {
        assert!(body_is_linked("MX_INFRA_USE_GLOBAL=true\n"));
        assert!(body_is_linked("A=1\n  MX_INFRA_USE_GLOBAL=true  \n"));
        assert!(!body_is_linked("MX_INFRA_USE_GLOBAL=false\n"));
        assert!(!body_is_linked("MX_INFRA_USE_GLOBAL=truest\n"));
        assert!(!body_is_linked("# MX_INFRA_USE_GLOBAL=true\n"));
        assert!(!body_is_linked(""));
    }

    #[test]
    fn set_then_clear_round_trips_the_body() {
        for original in ["", "A=1\n", "# c\nCLOUDFLARE_ACCOUNT_ID=x\nB=2\n"] {
            let linked = set_link_flag(original).expect("unlinked body gets the flag");
            assert!(body_is_linked(&linked));
            assert_eq!(set_link_flag(&linked), None, "set is idempotent");
            let cleared = clear_link_flag(&linked).expect("linked body loses the flag");
            assert_eq!(cleared, original, "round trip must be exact");
            assert_eq!(clear_link_flag(&cleared), None, "clear is idempotent");
        }
    }

    #[test]
    fn set_replaces_a_non_true_flag_value() {
        let out = set_link_flag("A=1\nMX_INFRA_USE_GLOBAL=false\n").unwrap();
        assert_eq!(out.matches("MX_INFRA_USE_GLOBAL=").count(), 1);
        assert!(body_is_linked(&out));
    }

    #[test]
    fn clear_drops_the_legacy_bash_link_metadata() {
        let legacy =
            "# header\nMX_INFRA_USE_GLOBAL=true\nMX_INFRA_PROVIDER=cloudflare\nMX_INFRA_LINKED_AT=2026-01-11\n";
        let out = clear_link_flag(legacy).unwrap();
        assert!(parse_env_body(&out).is_empty(), "{out}");
    }

    #[test]
    fn precedence_is_environment_then_project_then_global() {
        let (_d, infra) = setup(
            Some("CLOUDFLARE_ACCOUNT_ID=g\nCLOUDFLARE_API_TOKEN=gt\n"),
            Some("CLOUDFLARE_ACCOUNT_ID=p\n"),
        );
        let r = infra
            .resolve_with_env(InfraProvider::Cloudflare, &no_env)
            .unwrap();
        let id = r.get("CLOUDFLARE_ACCOUNT_ID").unwrap();
        assert_eq!(id.value.as_deref(), Some("p"));
        assert_eq!(id.scope, Some(CredentialScope::Project));
        let tok = r.get("CLOUDFLARE_API_TOKEN").unwrap();
        assert_eq!(tok.scope, Some(CredentialScope::Global));

        let env = |n: &str| (n == "CLOUDFLARE_ACCOUNT_ID").then(|| "e".to_string());
        let r = infra
            .resolve_with_env(InfraProvider::Cloudflare, &env)
            .unwrap();
        let id = r.get("CLOUDFLARE_ACCOUNT_ID").unwrap();
        assert_eq!(id.value.as_deref(), Some("e"));
        assert_eq!(id.scope, Some(CredentialScope::Environment));
    }

    #[test]
    fn a_project_alias_beats_a_global_canonical_name_and_is_tagged() {
        let (_d, infra) = setup(
            Some("CLOUDFLARE_ACCOUNT_ID=g\n"),
            Some("CF_ACCOUNT_ID=p-alias\n"),
        );
        let r = infra
            .resolve_with_env(InfraProvider::Cloudflare, &no_env)
            .unwrap();
        let id = r.get("CLOUDFLARE_ACCOUNT_ID").unwrap();
        assert_eq!(id.value.as_deref(), Some("p-alias"));
        assert_eq!(id.scope, Some(CredentialScope::Project));
        assert_eq!(id.deprecated_alias(), Some("CF_ACCOUNT_ID"));
        assert!(
            r.get("CF_ACCOUNT_ID").is_none(),
            "aliases fold into the canonical key"
        );
    }

    #[test]
    fn a_linked_project_file_is_not_read() {
        let (_d, infra) = setup(
            Some("CLOUDFLARE_ACCOUNT_ID=g\n"),
            Some("CLOUDFLARE_ACCOUNT_ID=p\nCF_DOCKER_PLATFORM=linux/arm64\nMX_INFRA_USE_GLOBAL=true\n"),
        );
        let r = infra
            .resolve_with_env(InfraProvider::Cloudflare, &no_env)
            .unwrap();
        assert!(r.linked);
        let id = r.get("CLOUDFLARE_ACCOUNT_ID").unwrap();
        assert_eq!(id.value.as_deref(), Some("g"));
        assert_eq!(id.scope, Some(CredentialScope::Global));
        assert!(r.get("CF_DOCKER_PLATFORM").is_none());
        assert!(r.get(LINK_FLAG).is_none(), "the flag is state, not config");
        assert_eq!(
            infra.resolve_config_path(InfraProvider::Cloudflare),
            Some(infra.global_config_path(InfraProvider::Cloudflare))
        );
    }

    #[test]
    fn the_deprecated_marker_is_reported_but_not_honored() {
        let (_d, infra) = setup(
            Some("CLOUDFLARE_ACCOUNT_ID=g\n"),
            Some("CLOUDFLARE_ACCOUNT_ID=p\n"),
        );
        let marker = infra
            .deprecated_marker_path(InfraProvider::Cloudflare)
            .unwrap();
        std::fs::write(&marker, "").unwrap();
        let r = infra
            .resolve_with_env(InfraProvider::Cloudflare, &no_env)
            .unwrap();
        assert!(!r.linked);
        assert_eq!(r.deprecated_marker.as_deref(), Some(marker.as_path()));
        assert_eq!(
            r.get("CLOUDFLARE_ACCOUNT_ID").unwrap().value.as_deref(),
            Some("p")
        );
    }

    #[test]
    fn link_unlink_round_trip_on_disk() {
        let (_d, infra) = setup(Some("CLOUDFLARE_ACCOUNT_ID=g\n"), Some("A=1\n"));
        let p = InfraProvider::Cloudflare;
        let path = infra.project_config_path(p).unwrap();

        let out = infra.link(p).unwrap();
        assert!(!out.already_linked && !out.created_file);
        assert!(infra.is_linked(p));
        assert!(infra.link(p).unwrap().already_linked);

        let out = infra.unlink(p).unwrap();
        assert!(out.was_linked && !out.removed_file);
        assert!(!infra.is_linked(p));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "A=1\n");
        assert!(!infra.unlink(p).unwrap().was_linked);
    }

    #[test]
    fn link_requires_a_project_and_a_global_config() {
        let dir = tempfile::tempdir().unwrap();
        let no_project = infra(dir.path(), None);
        assert!(matches!(
            no_project.link(InfraProvider::Cloudflare),
            Err(Error::NotAProject)
        ));
        assert!(matches!(
            no_project.unlink(InfraProvider::Cloudflare),
            Err(Error::NotAProject)
        ));

        let (_d, infra) = setup(None, None);
        let err = infra.link(InfraProvider::Cloudflare).unwrap_err();
        assert!(
            err.to_string().contains("mx infra setup cloudflare"),
            "{err}"
        );
        assert!(!infra.is_project_configured(InfraProvider::Cloudflare));
    }
}
