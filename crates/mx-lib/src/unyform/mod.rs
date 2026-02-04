//! Unyform API client
//!
//! Handles authentication and recipe management with Unyform.ai.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::MechCrateConfig;
use crate::error::{Error, Result};

/// Unyform API client
#[derive(Debug)]
pub struct UnyformClient {
    config: MechCrateConfig,
    http: reqwest::Client,
    base_url: String,
}

/// Stored credentials
#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub api_key: Option<String>,
    pub url: String,
    pub org_id: Option<String>,
}

/// Session information (tokens)
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,
}

/// User information from API
#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub organizations: Vec<OrgInfo>,
}

/// Organization information
#[derive(Debug, Deserialize)]
pub struct OrgInfo {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub role: String,
}

/// Recipe listing response
#[derive(Debug, Deserialize)]
pub struct RecipeListResponse {
    pub recipes: Vec<RecipeSummary>,
}

/// Recipe summary
#[derive(Debug, Deserialize, Serialize)]
pub struct RecipeSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
}

/// Full recipe data
#[derive(Debug, Deserialize, Serialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub patterns: Vec<serde_json::Value>,
    pub dependencies: Vec<serde_json::Value>,
    pub infrastructure: Option<serde_json::Value>,
}

/// Recipe version info
#[derive(Debug, Deserialize)]
pub struct RecipeVersion {
    pub version: String,
    pub is_latest: bool,
    pub generated_at: String,
}

/// Recipe versions response
#[derive(Debug, Deserialize)]
pub struct RecipeVersionsResponse {
    pub versions: Vec<RecipeVersion>,
}

impl UnyformClient {
    /// Default Unyform API URL
    pub const DEFAULT_URL: &'static str = "https://api.unyform.ai";

    /// Create a new Unyform client
    pub fn new() -> Self {
        let config = MechCrateConfig::default();
        Self {
            config,
            http: reqwest::Client::new(),
            base_url: Self::DEFAULT_URL.to_string(),
        }
    }

    /// Create client with custom config
    pub fn with_config(config: MechCrateConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
            base_url: Self::DEFAULT_URL.to_string(),
        }
    }

    /// Get the credentials file path
    fn credentials_path(&self) -> PathBuf {
        self.config.unyform_dir().join("credentials.json")
    }

    /// Get the session file path
    fn session_path(&self) -> PathBuf {
        self.config.unyform_dir().join("session.json")
    }

    /// Check if user is logged in
    pub fn is_logged_in(&self) -> bool {
        self.credentials_path().exists()
    }

    /// Load stored credentials
    pub fn load_credentials(&self) -> Result<Credentials> {
        let path = self.credentials_path();
        if !path.exists() {
            return Err(Error::AuthRequired);
        }

        let content = std::fs::read_to_string(&path)?;
        let creds: Credentials = serde_json::from_str(&content)?;
        Ok(creds)
    }

    /// Save credentials
    pub fn save_credentials(&self, creds: &Credentials) -> Result<()> {
        self.config.ensure_dirs()?;
        let path = self.credentials_path();
        let content = serde_json::to_string_pretty(creds)?;
        std::fs::write(&path, content)?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    /// Load session
    pub fn load_session(&self) -> Result<Session> {
        let path = self.session_path();
        if !path.exists() {
            return Err(Error::AuthRequired);
        }

        let content = std::fs::read_to_string(&path)?;
        let session: Session = serde_json::from_str(&content)?;
        Ok(session)
    }

    /// Save session
    pub fn save_session(&self, session: &Session) -> Result<()> {
        self.config.ensure_dirs()?;
        let path = self.session_path();
        let content = serde_json::to_string_pretty(session)?;
        std::fs::write(&path, content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    /// Clear credentials and session (logout)
    pub fn logout(&self) -> Result<()> {
        let creds_path = self.credentials_path();
        let session_path = self.session_path();

        if creds_path.exists() {
            std::fs::remove_file(&creds_path)?;
        }
        if session_path.exists() {
            std::fs::remove_file(&session_path)?;
        }

        Ok(())
    }

    /// Get the default organization ID
    pub fn get_default_org(&self) -> Result<String> {
        let creds = self.load_credentials()?;
        creds
            .org_id
            .ok_or_else(|| Error::Config("No default organization set".into()))
    }

    /// Get the API URL (from credentials or default)
    pub fn get_url(&self) -> String {
        self.load_credentials()
            .map(|c| c.url)
            .unwrap_or_else(|_| Self::DEFAULT_URL.to_string())
    }

    /// Get authentication header value
    pub fn get_auth_header(&self) -> Result<String> {
        // Try session token first
        if let Ok(session) = self.load_session() {
            let now = chrono::Utc::now().timestamp();
            if session.expires_at > now {
                return Ok(format!("Bearer {}", session.access_token));
            }
        }

        // Fall back to API key
        let creds = self.load_credentials()?;
        if let Some(api_key) = creds.api_key {
            return Ok(format!("Bearer {}", api_key));
        }

        Err(Error::AuthRequired)
    }

    /// Login with API key
    pub async fn login_with_api_key(
        &self,
        api_key: &str,
        url: Option<&str>,
    ) -> Result<UserInfo> {
        let base_url = url.unwrap_or(Self::DEFAULT_URL);

        // Verify the API key by fetching user info
        let response = self
            .http
            .get(format!("{}/v1/auth/me", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Api(format!(
                "Authentication failed: {}",
                response.status()
            )));
        }

        let user: UserInfo = response.json().await?;

        // Save credentials
        let creds = Credentials {
            api_key: Some(api_key.to_string()),
            url: base_url.to_string(),
            org_id: user.organizations.first().map(|o| o.id.clone()),
        };
        self.save_credentials(&creds)?;

        Ok(user)
    }

    /// Get current user info
    pub async fn whoami(&self) -> Result<UserInfo> {
        let auth = self.get_auth_header()?;
        let url = self.get_url();

        let response = self
            .http
            .get(format!("{}/v1/auth/me", url))
            .header("Authorization", &auth)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Api(format!(
                "Failed to get user info: {}",
                response.status()
            )));
        }

        let user: UserInfo = response.json().await?;
        Ok(user)
    }

    /// List recipes for the organization
    pub async fn list_recipes(&self) -> Result<RecipeListResponse> {
        let auth = self.get_auth_header()?;
        let url = self.get_url();
        let org = self.get_default_org()?;

        let response = self
            .http
            .get(format!("{}/v1/orgs/{}/recipes", url, org))
            .header("Authorization", &auth)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Api(format!(
                "Failed to list recipes: {}",
                response.status()
            )));
        }

        let recipes: RecipeListResponse = response.json().await?;
        Ok(recipes)
    }

    /// Get a specific recipe
    pub async fn get_recipe(&self, name: &str, version: Option<&str>) -> Result<Recipe> {
        let auth = self.get_auth_header()?;
        let url = self.get_url();
        let org = self.get_default_org()?;

        let endpoint = match version {
            Some(v) => format!("{}/v1/orgs/{}/recipes/{}/versions/{}", url, org, name, v),
            None => format!("{}/v1/orgs/{}/recipes/{}", url, org, name),
        };

        let response = self
            .http
            .get(&endpoint)
            .header("Authorization", &auth)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Api(format!(
                "Failed to get recipe: {}",
                response.status()
            )));
        }

        let recipe: Recipe = response.json().await?;
        Ok(recipe)
    }

    /// Get recipe versions
    pub async fn get_recipe_versions(&self, name: &str) -> Result<RecipeVersionsResponse> {
        let auth = self.get_auth_header()?;
        let url = self.get_url();
        let org = self.get_default_org()?;

        let response = self
            .http
            .get(format!("{}/v1/orgs/{}/recipes/{}/versions", url, org, name))
            .header("Authorization", &auth)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Api(format!(
                "Failed to get recipe versions: {}",
                response.status()
            )));
        }

        let versions: RecipeVersionsResponse = response.json().await?;
        Ok(versions)
    }

    /// Cache a recipe locally
    pub fn cache_recipe(&self, org: &str, recipe: &Recipe) -> Result<PathBuf> {
        let cache_dir = self
            .config
            .recipes_dir()
            .join(org)
            .join(&recipe.name)
            .join(&recipe.version);

        std::fs::create_dir_all(&cache_dir)?;

        let recipe_file = cache_dir.join("recipe.json");
        let content = serde_json::to_string_pretty(recipe)?;
        std::fs::write(&recipe_file, content)?;

        // Create/update "latest" symlink
        let latest_link = cache_dir.parent().unwrap().join("latest");
        let _ = std::fs::remove_file(&latest_link);

        #[cfg(unix)]
        std::os::unix::fs::symlink(&recipe.version, &latest_link)?;

        Ok(cache_dir)
    }

    /// List cached recipes
    pub fn list_cached_recipes(&self) -> Result<Vec<(String, String, Vec<String>)>> {
        let recipes_dir = self.config.recipes_dir();
        let mut result = Vec::new();

        if !recipes_dir.exists() {
            return Ok(result);
        }

        // org/name/version structure
        for org_entry in std::fs::read_dir(&recipes_dir)? {
            let org_entry = org_entry?;
            if !org_entry.path().is_dir() {
                continue;
            }

            let org = org_entry.file_name().to_string_lossy().to_string();

            for recipe_entry in std::fs::read_dir(org_entry.path())? {
                let recipe_entry = recipe_entry?;
                if !recipe_entry.path().is_dir() {
                    continue;
                }

                let name = recipe_entry.file_name().to_string_lossy().to_string();
                let mut versions = Vec::new();

                for version_entry in std::fs::read_dir(recipe_entry.path())? {
                    let version_entry = version_entry?;
                    let version_name = version_entry.file_name().to_string_lossy().to_string();

                    // Skip "latest" symlink
                    if version_name != "latest" && version_entry.path().is_dir() {
                        versions.push(version_name);
                    }
                }

                if !versions.is_empty() {
                    versions.sort();
                    result.push((org.clone(), name, versions));
                }
            }
        }

        Ok(result)
    }

    /// Clear recipe cache
    pub fn clear_cache(&self) -> Result<()> {
        let recipes_dir = self.config.recipes_dir();
        if recipes_dir.exists() {
            std::fs::remove_dir_all(&recipes_dir)?;
            std::fs::create_dir_all(&recipes_dir)?;
        }
        Ok(())
    }
}

impl Default for UnyformClient {
    fn default() -> Self {
        Self::new()
    }
}
