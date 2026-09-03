//! Self-update: the GitHub Releases index client (effectful).
//!
//! The one place that knows GitHub's release JSON (spec §3.1, §3.4). The base
//! URL is a seam: `MX_RELEASES_API` points it at wiremock in tests and could
//! point it at a private mirror later.

use serde::Deserialize;

use crate::error::{Error, Result};
use crate::selfupdate::version::{self, Version};

/// The public releases repo the client talks to when nothing overrides it.
pub const DEFAULT_BASE_URL: &str = "https://api.github.com/repos/unyform-ai/mech-crate-releases";

/// Environment variable overriding [`DEFAULT_BASE_URL`].
pub const BASE_URL_ENV: &str = "MX_RELEASES_API";

/// One downloadable file attached to a release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    /// File name as published, e.g. `mx-v0.1.2-universal-apple-darwin.tar.gz`.
    pub name: String,
    /// Direct download URL (GitHub's `browser_download_url`).
    pub url: String,
    /// Size in bytes, as reported by the index.
    pub size: u64,
}

/// A published release: its version and the assets attached to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    /// Version parsed from the release's `tag_name` (a leading `v` is fine).
    pub version: Version,
    /// Every asset attached to the release, in the order the index lists them.
    pub assets: Vec<Asset>,
}

impl Release {
    /// The asset with exactly this file name, if the release has one.
    pub fn asset(&self, name: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.name == name)
    }
}

/// Read-only client for the release channel.
pub struct ReleaseIndex {
    base_url: String,
    token: Option<String>,
    client: reqwest::Client,
}

impl ReleaseIndex {
    /// Build a client from the environment: base URL from `MX_RELEASES_API`
    /// (default [`DEFAULT_BASE_URL`]), bearer token from `GITHUB_TOKEN` then
    /// `GH_TOKEN`. A token is never required — the releases repo is public;
    /// it only raises the unauthenticated rate limit.
    pub fn from_env() -> Self {
        let base = std::env::var(BASE_URL_ENV)
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let token = ["GITHUB_TOKEN", "GH_TOKEN"]
            .iter()
            .find_map(|k| std::env::var(k).ok())
            .filter(|s| !s.trim().is_empty());
        Self::new(base, token)
    }

    /// Build a client against an explicit base URL and optional bearer token.
    /// A trailing slash on `base_url` is trimmed.
    pub fn new(base_url: impl Into<String>, token: Option<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            token,
            client: reqwest::Client::new(),
        }
    }

    /// Cap every request at `timeout` (connect plus response).
    ///
    /// The notifier's background refresh must not linger on a black-holed
    /// network; a client that cannot be rebuilt keeps the default (no cap).
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        if let Ok(client) = reqwest::Client::builder().timeout(timeout).build() {
            self.client = client;
        }
        self
    }

    /// The base URL every request is built from, without a trailing slash.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// The bearer token that will be sent, if any.
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// The reqwest client, so a download can reuse this connection pool.
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }

    /// The newest published, non-draft, non-prerelease release.
    pub async fn latest(&self) -> Result<Release> {
        self.release_at(&format!("{}/releases/latest", self.base_url))
            .await
    }

    /// The release tagged `v{version}` — the `--to` path.
    pub async fn get(&self, version: &Version) -> Result<Release> {
        self.release_at(&format!("{}/releases/tags/v{version}", self.base_url))
            .await
    }

    /// GET `url`, map the status to an error variant, and parse the body.
    async fn release_at(&self, url: &str) -> Result<Release> {
        let body = self.get_json(url).await?;
        let raw: RawRelease = serde_json::from_str(&body)?;
        Ok(Release {
            version: version::parse(&raw.tag_name)?,
            assets: raw
                .assets
                .into_iter()
                .map(|a| Asset {
                    name: a.name,
                    url: a.browser_download_url,
                    size: a.size,
                })
                .collect(),
        })
    }

    /// GET `url` with the GitHub headers, returning the body of a 2xx and
    /// mapping every other status to an error that names the URL.
    async fn get_json(&self, url: &str) -> Result<String> {
        let mut req = self
            .client
            .get(url)
            .header(
                reqwest::header::USER_AGENT,
                concat!("mx/", env!("CARGO_PKG_VERSION")),
            )
            .header(reqwest::header::ACCEPT, "application/vnd.github+json");
        if let Some(token) = &self.token {
            req = req.header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"));
        }

        let resp = req.send().await?;
        let status = resp.status();
        if status.is_success() {
            return Ok(resp.text().await?);
        }

        let body = resp.text().await.unwrap_or_default();
        let message = github_message(&body);
        match status.as_u16() {
            404 => Err(Error::NotFound(url.to_string())),
            403 | 429 if is_rate_limit(&body, message.as_deref()) => Err(Error::RateLimited(
                message.unwrap_or_else(|| format!("{status} from {url}")),
            )),
            _ => Err(Error::Api(match message {
                Some(m) => format!("{status} from {url}: {m}"),
                None => format!("{status} from {url}"),
            })),
        }
    }
}

/// GitHub error bodies are `{"message": "...", ...}`; pull that out if present.
fn github_message(body: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()?
        .get("message")?
        .as_str()
        .map(str::to_string)
}

/// A 403 is a rate limit only when the body says so; other 403s are real
/// permission errors and must not be reported as "try again later".
fn is_rate_limit(body: &str, message: Option<&str>) -> bool {
    let haystack = message.unwrap_or(body).to_ascii_lowercase();
    haystack.contains("rate limit") || haystack.contains("secondary rate")
}

/// GitHub's release JSON, narrowed to the fields the updater uses.
#[derive(Deserialize)]
struct RawRelease {
    tag_name: String,
    #[serde(default)]
    assets: Vec<RawAsset>,
}

/// GitHub's release-asset JSON, narrowed the same way.
#[derive(Deserialize)]
struct RawAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::selfupdate::index::ReleaseIndex;
    use crate::selfupdate::version::parse;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    const LATEST_BODY: &str = include_str!("../../tests/fixtures/selfupdate/releases-latest.json");
    const RATE_LIMITED_BODY: &str =
        include_str!("../../tests/fixtures/selfupdate/rate-limited.json");
    const NOT_FOUND_BODY: &str = include_str!("../../tests/fixtures/selfupdate/not-found.json");

    fn json_ok(body: &str) -> ResponseTemplate {
        ResponseTemplate::new(200)
            .set_body_string(body)
            .insert_header("content-type", "application/json")
    }

    #[tokio::test]
    async fn latest_parses_the_recorded_body() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases/latest"))
            .respond_with(json_ok(LATEST_BODY))
            .mount(&server)
            .await;

        let idx = ReleaseIndex::new(server.uri(), None);
        let release = idx.latest().await.unwrap();

        assert_eq!(release.version, parse("0.1.2").unwrap());
        let names: Vec<&str> = release.assets.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "mx-v0.1.2-universal-apple-darwin.tar.gz",
                "mx-v0.1.2-universal-apple-darwin.tar.gz.sha256",
            ]
        );
    }

    #[tokio::test]
    async fn assets_carry_the_browser_download_url_and_size() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases/latest"))
            .respond_with(json_ok(LATEST_BODY))
            .mount(&server)
            .await;

        let release = ReleaseIndex::new(server.uri(), None)
            .latest()
            .await
            .unwrap();
        let tarball = release
            .asset("mx-v0.1.2-universal-apple-darwin.tar.gz")
            .expect("tarball asset");
        assert_eq!(
            tarball.url,
            "https://github.com/unyform-ai/mech-crate-releases/releases/download/v0.1.2/mx-v0.1.2-universal-apple-darwin.tar.gz"
        );
        assert_eq!(tarball.size, 18_446_732);
        assert!(release.asset("nope.tar.gz").is_none());
    }

    #[tokio::test]
    async fn every_request_sends_user_agent_and_accept() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases/latest"))
            .and(header(
                "user-agent",
                format!("mx/{}", env!("CARGO_PKG_VERSION")).as_str(),
            ))
            .and(header("accept", "application/vnd.github+json"))
            .respond_with(json_ok(LATEST_BODY))
            .mount(&server)
            .await;

        ReleaseIndex::new(server.uri(), None)
            .latest()
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn a_token_is_sent_as_a_bearer() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases/latest"))
            .and(header("authorization", "Bearer ghp_testtoken"))
            .respond_with(json_ok(LATEST_BODY))
            .mount(&server)
            .await;

        ReleaseIndex::new(server.uri(), Some("ghp_testtoken".to_string()))
            .latest()
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn without_a_token_no_authorization_header_is_sent() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases/latest"))
            .and(|req: &Request| !req.headers.contains_key("authorization"))
            .respond_with(json_ok(LATEST_BODY))
            .mount(&server)
            .await;

        ReleaseIndex::new(server.uri(), None)
            .latest()
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn get_hits_the_tag_endpoint() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases/tags/v0.1.2"))
            .respond_with(json_ok(LATEST_BODY))
            .mount(&server)
            .await;

        let release = ReleaseIndex::new(server.uri(), None)
            .get(&parse("0.1.2").unwrap())
            .await
            .unwrap();
        assert_eq!(release.version, parse("0.1.2").unwrap());
    }

    #[tokio::test]
    async fn a_rate_limited_403_surfaces_the_github_message() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases/latest"))
            .respond_with(
                ResponseTemplate::new(403)
                    .set_body_string(RATE_LIMITED_BODY)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;

        let err = ReleaseIndex::new(server.uri(), None)
            .latest()
            .await
            .unwrap_err();
        match err {
            Error::RateLimited(msg) => {
                assert!(msg.contains("API rate limit exceeded"), "message was {msg}");
            }
            other => panic!("expected RateLimited, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn a_forbidden_403_that_is_not_a_rate_limit_is_an_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases/latest"))
            .respond_with(
                ResponseTemplate::new(403)
                    .set_body_string(r#"{"message":"Resource not accessible by integration"}"#)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;

        let err = ReleaseIndex::new(server.uri(), None)
            .latest()
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Api(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn a_404_is_not_found_and_names_the_url() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases/tags/v9.9.9"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_string(NOT_FOUND_BODY)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;

        let err = ReleaseIndex::new(server.uri(), None)
            .get(&parse("9.9.9").unwrap())
            .await
            .unwrap_err();
        match err {
            Error::NotFound(url) => {
                assert!(url.ends_with("/releases/tags/v9.9.9"), "url was {url}")
            }
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn other_non_2xx_statuses_are_api_errors_naming_status_and_url() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases/latest"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let err = ReleaseIndex::new(server.uri(), None)
            .latest()
            .await
            .unwrap_err();
        match err {
            Error::Api(msg) => {
                assert!(msg.contains("500"), "message was {msg}");
                assert!(msg.contains("/releases/latest"), "message was {msg}");
            }
            other => panic!("expected Api, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn an_unparseable_tag_name_is_an_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases/latest"))
            .respond_with(json_ok(r#"{"tag_name":"nightly","assets":[]}"#))
            .mount(&server)
            .await;

        assert!(ReleaseIndex::new(server.uri(), None)
            .latest()
            .await
            .is_err());
    }

    #[test]
    fn from_env_defaults_to_the_public_releases_repo() {
        std::env::remove_var("MX_RELEASES_API");
        std::env::remove_var("GITHUB_TOKEN");
        std::env::remove_var("GH_TOKEN");
        let idx = ReleaseIndex::from_env();
        assert_eq!(
            idx.base_url(),
            "https://api.github.com/repos/unyform-ai/mech-crate-releases"
        );
        assert_eq!(idx.token(), None);
    }

    #[test]
    fn from_env_reads_the_base_url_override_and_either_token_var() {
        std::env::set_var("MX_RELEASES_API", "http://localhost:9/mirror/");
        std::env::remove_var("GITHUB_TOKEN");
        std::env::set_var("GH_TOKEN", "gh_token");
        assert_eq!(ReleaseIndex::from_env().token(), Some("gh_token"));

        // GITHUB_TOKEN wins over GH_TOKEN.
        std::env::set_var("GITHUB_TOKEN", "github_token");
        let idx = ReleaseIndex::from_env();
        assert_eq!(idx.token(), Some("github_token"));
        // A trailing slash on the override must not produce a double slash.
        assert_eq!(idx.base_url(), "http://localhost:9/mirror");
    }
}
