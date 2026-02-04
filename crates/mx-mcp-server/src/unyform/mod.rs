//! Unyform API Client
//!
//! HTTP client for interacting with the Unyform.ai API.
//! Handles authentication, recipe management, and user operations.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Unyform API errors
#[derive(Debug, Error)]
pub enum UnyformError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API error: {0}")]
    Api(String),

    #[error("Not authenticated")]
    NotAuthenticated,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Unyform API client
pub struct UnyformClient {
    client: Client,
    config_dir: PathBuf,
    default_url: String,
}

/// Credential storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub api_key: Option<String>,
    pub url: Option<String>,
    pub org_id: Option<String>,
}

/// Session storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<String>,
    pub user: Option<UserInfo>,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub organizations: Vec<OrgInfo>,
}

/// Organization information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub role: String,
}

/// Login response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

/// Recipe summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub generated_at: String,
    pub patterns_count: usize,
    pub source_repos_count: usize,
}

/// Recipe list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeListResponse {
    pub recipes: Vec<RecipeSummary>,
}

/// Recipe version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeVersion {
    pub version: String,
    pub generated_at: String,
    pub is_latest: bool,
}

/// Recipe versions response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeVersionsResponse {
    pub recipe_id: String,
    pub recipe_name: String,
    pub versions: Vec<RecipeVersion>,
}

/// Full recipe data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub generated_at: String,
    pub patterns: Vec<serde_json::Value>,
    pub dependencies: serde_json::Value,
    pub infrastructure: serde_json::Value,
    pub metadata: serde_json::Value,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiError {
    error: ApiErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
}

impl UnyformClient {
    /// Create a new Unyform client
    pub fn new() -> Self {
        let config_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".mech-crate")
            .join("config")
            .join("unyform");

        Self {
            client: Client::new(),
            config_dir,
            default_url: "https://api.unyform.ai".to_string(),
        }
    }

    /// Get the API URL
    fn get_url(&self) -> Result<String, UnyformError> {
        if let Ok(creds) = self.load_credentials() {
            if let Some(url) = creds.url {
                return Ok(url);
            }
        }
        Ok(self.default_url.clone())
    }

    /// Load credentials from file
    pub fn load_credentials(&self) -> Result<Credentials, UnyformError> {
        let path = self.config_dir.join("credentials.json");
        if !path.exists() {
            return Ok(Credentials {
                api_key: None,
                url: None,
                org_id: None,
            });
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// Load session from file
    pub fn load_session(&self) -> Result<Option<Session>, UnyformError> {
        let path = self.config_dir.join("session.json");
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(Some(serde_json::from_str(&content)?))
    }

    /// Save credentials to file
    pub fn save_credentials(&self, creds: &Credentials) -> Result<(), UnyformError> {
        std::fs::create_dir_all(&self.config_dir)?;
        let path = self.config_dir.join("credentials.json");
        let content = serde_json::to_string_pretty(creds)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Save session to file
    pub fn save_session(&self, session: &Session) -> Result<(), UnyformError> {
        std::fs::create_dir_all(&self.config_dir)?;
        let path = self.config_dir.join("session.json");
        let content = serde_json::to_string_pretty(session)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Clear all credentials
    pub fn clear_credentials(&self) -> Result<(), UnyformError> {
        let creds_path = self.config_dir.join("credentials.json");
        let session_path = self.config_dir.join("session.json");
        
        if creds_path.exists() {
            std::fs::remove_file(&creds_path)?;
        }
        if session_path.exists() {
            std::fs::remove_file(&session_path)?;
        }
        Ok(())
    }

    /// Check if user is logged in
    pub fn is_logged_in(&self) -> bool {
        if let Ok(session) = self.load_session() {
            if session.is_some() {
                return true;
            }
        }
        if let Ok(creds) = self.load_credentials() {
            if creds.api_key.is_some() {
                return true;
            }
        }
        false
    }

    /// Get the auth header
    fn get_auth_header(&self) -> Result<String, UnyformError> {
        if let Ok(Some(session)) = self.load_session() {
            return Ok(format!("Bearer {}", session.access_token));
        }
        if let Ok(creds) = self.load_credentials() {
            if let Some(api_key) = creds.api_key {
                return Ok(format!("Bearer {}", api_key));
            }
        }
        Err(UnyformError::NotAuthenticated)
    }

    /// Get the default organization slug
    pub fn get_default_org(&self) -> Result<String, UnyformError> {
        if let Ok(Some(session)) = self.load_session() {
            if let Some(user) = session.user {
                if let Some(org) = user.organizations.first() {
                    return Ok(org.slug.clone());
                }
            }
        }
        Err(UnyformError::Config("No organization found".to_string()))
    }

    /// Login with API key
    pub async fn login_with_api_key(&self, api_key: &str, url: Option<&str>) -> Result<LoginResponse, UnyformError> {
        let base_url = url.unwrap_or(&self.default_url);
        
        let response = self.client
            .post(&format!("{}/v1/auth/login", base_url))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "api_key": api_key
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ApiError = response.json().await.map_err(|_| {
                UnyformError::Api("Login failed".to_string())
            })?;
            return Err(UnyformError::Api(error.error.message));
        }

        let login_resp: LoginResponse = response.json().await?;

        // Save credentials and session
        let creds = Credentials {
            api_key: Some(api_key.to_string()),
            url: Some(base_url.to_string()),
            org_id: login_resp.user.organizations.first().map(|o| o.id.clone()),
        };
        self.save_credentials(&creds)?;

        let session = Session {
            access_token: login_resp.access_token.clone(),
            refresh_token: Some(login_resp.refresh_token.clone()),
            expires_at: None, // Would calculate from expires_in
            user: Some(login_resp.user.clone()),
        };
        self.save_session(&session)?;

        Ok(login_resp)
    }

    /// Logout
    pub async fn logout(&self) -> Result<(), UnyformError> {
        if let Ok(url) = self.get_url() {
            if let Ok(auth) = self.get_auth_header() {
                let _ = self.client
                    .post(&format!("{}/v1/auth/logout", url))
                    .header("Authorization", auth)
                    .send()
                    .await;
            }
        }
        self.clear_credentials()
    }

    /// Get current user info
    pub async fn whoami(&self) -> Result<UserInfo, UnyformError> {
        let url = self.get_url()?;
        let auth = self.get_auth_header()?;

        let response = self.client
            .get(&format!("{}/v1/auth/me", url))
            .header("Authorization", &auth)
            .send()
            .await?;

        if !response.status().is_success() {
            // Fall back to cached user info
            if let Ok(Some(session)) = self.load_session() {
                if let Some(user) = session.user {
                    return Ok(user);
                }
            }
            return Err(UnyformError::Api("Failed to get user info".to_string()));
        }

        Ok(response.json().await?)
    }

    /// List recipes for the default organization
    pub async fn list_recipes(&self) -> Result<RecipeListResponse, UnyformError> {
        let url = self.get_url()?;
        let auth = self.get_auth_header()?;
        let org = self.get_default_org()?;

        let response = self.client
            .get(&format!("{}/v1/orgs/{}/recipes", url, org))
            .header("Authorization", &auth)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ApiError = response.json().await.map_err(|_| {
                UnyformError::Api("Failed to list recipes".to_string())
            })?;
            return Err(UnyformError::Api(error.error.message));
        }

        Ok(response.json().await?)
    }

    /// Get recipe versions
    pub async fn get_recipe_versions(&self, recipe_name: &str) -> Result<RecipeVersionsResponse, UnyformError> {
        let url = self.get_url()?;
        let auth = self.get_auth_header()?;
        let org = self.get_default_org()?;

        let response = self.client
            .get(&format!("{}/v1/orgs/{}/recipes/{}/versions", url, org, recipe_name))
            .header("Authorization", &auth)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ApiError = response.json().await.map_err(|_| {
                UnyformError::Api("Failed to get versions".to_string())
            })?;
            return Err(UnyformError::Api(error.error.message));
        }

        Ok(response.json().await?)
    }

    /// Get a specific recipe version
    pub async fn get_recipe(&self, recipe_name: &str, version: Option<&str>) -> Result<Recipe, UnyformError> {
        let url = self.get_url()?;
        let auth = self.get_auth_header()?;
        let org = self.get_default_org()?;

        // If no version specified, get latest
        let version = if let Some(v) = version {
            v.to_string()
        } else {
            let versions = self.get_recipe_versions(recipe_name).await?;
            versions.versions
                .iter()
                .find(|v| v.is_latest)
                .or_else(|| versions.versions.first())
                .map(|v| v.version.clone())
                .ok_or_else(|| UnyformError::Api("No versions found".to_string()))?
        };

        let response = self.client
            .get(&format!("{}/v1/orgs/{}/recipes/{}/versions/{}", url, org, recipe_name, version))
            .header("Authorization", &auth)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ApiError = response.json().await.map_err(|_| {
                UnyformError::Api("Failed to get recipe".to_string())
            })?;
            return Err(UnyformError::Api(error.error.message));
        }

        Ok(response.json().await?)
    }

    /// Cache a recipe locally
    pub fn cache_recipe(&self, org: &str, recipe: &Recipe) -> Result<PathBuf, UnyformError> {
        let recipes_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".mech-crate")
            .join("recipes")
            .join(org)
            .join(&recipe.name)
            .join(&recipe.version);

        std::fs::create_dir_all(&recipes_dir)?;

        // Save recipe
        let recipe_path = recipes_dir.join("recipe.json");
        std::fs::write(&recipe_path, serde_json::to_string_pretty(recipe)?)?;

        // Save manifest
        let manifest = serde_json::json!({
            "pulled_at": chrono::Utc::now().to_rfc3339(),
            "org": org,
            "name": &recipe.name,
            "version": &recipe.version
        });
        let manifest_path = recipes_dir.join("manifest.json");
        std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

        // Update latest symlink (Unix only)
        #[cfg(unix)]
        {
            let latest_link = recipes_dir.parent().unwrap().join("latest");
            let _ = std::fs::remove_file(&latest_link);
            let _ = std::os::unix::fs::symlink(&recipe.version, &latest_link);
        }

        Ok(recipe_path)
    }

    /// List cached recipes
    pub fn list_cached_recipes(&self) -> Result<Vec<(String, String, Vec<String>)>, UnyformError> {
        let recipes_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".mech-crate")
            .join("recipes");

        let mut result = Vec::new();

        if !recipes_dir.exists() {
            return Ok(result);
        }

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
                let recipe = recipe_entry.file_name().to_string_lossy().to_string();

                let mut versions = Vec::new();
                for ver_entry in std::fs::read_dir(recipe_entry.path())? {
                    let ver_entry = ver_entry?;
                    if !ver_entry.path().is_dir() {
                        continue;
                    }
                    let ver = ver_entry.file_name().to_string_lossy().to_string();
                    if ver != "latest" {
                        versions.push(ver);
                    }
                }

                if !versions.is_empty() {
                    result.push((org.clone(), recipe, versions));
                }
            }
        }

        Ok(result)
    }

    /// Clear cached recipes
    pub fn clear_cache(&self) -> Result<(), UnyformError> {
        let recipes_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".mech-crate")
            .join("recipes");

        if recipes_dir.exists() {
            std::fs::remove_dir_all(&recipes_dir)?;
        }
        Ok(())
    }
}

impl Default for UnyformClient {
    fn default() -> Self {
        Self::new()
    }
}

// Add dirs crate for home directory detection
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }
}
