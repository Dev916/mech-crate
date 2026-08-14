//! Known-broken TDD lane — the two Unyform clients.
//!
//! This lane test lives in `mx-mcp-server` because it is the only crate where
//! BOTH clients are in scope: the CLI client (`mx_lib::unyform`) and the MCP
//! server's own (`mx_mcp_server::unyform`). See `tests/KNOWN_BROKEN.md`.

use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Both clients list recipes for the same organization, so both must address
/// it with the same path segment. Today the CLI sends the org **id** and the
/// MCP server sends the org **slug** — one of them is wrong against
/// api.unyform.ai.
#[tokio::test]
#[ignore = "bd:mech-crate-rnj CLI client sends the org id, MCP client sends the org slug"]
async fn kb_both_unyform_clients_use_the_same_org_path_segment() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "recipes": []
        })))
        .mount(&server)
        .await;

    let home = tempfile::tempdir().expect("setup: home tempdir");
    let unyform_dir = home.path().join(".mech-crate/config/unyform");
    std::fs::create_dir_all(&unyform_dir).expect("setup: unyform config dir");

    // The CLI client resolves the org from `credentials.org_id`; the MCP client
    // resolves it from the cached session's first organization. Both are
    // present and describe the SAME org, so any disagreement in the request
    // path is a client bug, not a fixture asymmetry.
    std::fs::write(
        unyform_dir.join("credentials.json"),
        serde_json::json!({
            "api_key": "test-key",
            "url": server.uri(),
            "org_id": "org_abc123"
        })
        .to_string(),
    )
    .expect("setup: write credentials.json");
    std::fs::write(
        unyform_dir.join("session.json"),
        serde_json::json!({
            "access_token": "test-token",
            "refresh_token": null,
            "expires_at": "2099-01-01T00:00:00Z",
            "user": {
                "id": "usr_1",
                "email": "dev@example.com",
                "name": "Dev",
                "avatar_url": null,
                "organizations": [{
                    "id": "org_abc123",
                    "name": "Acme",
                    "slug": "acme",
                    "role": "owner"
                }]
            }
        })
        .to_string(),
    )
    .expect("setup: write session.json");

    std::env::set_var("HOME", home.path());

    let cli_client = mx_lib::unyform::UnyformClient::new();
    let mcp_client = mx_mcp_server::unyform::UnyformClient::new();

    cli_client
        .list_recipes()
        .await
        .expect("setup: CLI client must reach the stub API");
    mcp_client
        .list_recipes()
        .await
        .expect("setup: MCP client must reach the stub API");

    let paths: Vec<String> = server
        .received_requests()
        .await
        .expect("setup: recorded requests")
        .iter()
        .map(|r| r.url.path().to_string())
        .collect();

    assert_eq!(
        paths.len(),
        2,
        "setup: expected one request per client, recorded {paths:?}"
    );
    assert_eq!(
        paths[0], paths[1],
        "the CLI and MCP Unyform clients must address the same org segment, \
         but they requested {paths:?}"
    );
}
