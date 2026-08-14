//! Wiremock embedding server fixture.
//!
//! A local stand-in for any OpenAI-compatible `/embeddings` endpoint, so the
//! embed path and future ingest-resilience tests exercise real HTTP without
//! ever touching a paid API. Mirrors the three behaviours the client has to
//! survive: happy-path (with the response array deliberately out of order),
//! rate limiting, and server errors.

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

/// A running fake embedding server.
///
/// The server shuts down when this value drops — keep it alive for the
/// duration of the test.
pub struct EmbedServer {
    /// Base URL, e.g. `http://127.0.0.1:51763`. Append `/embeddings`.
    pub uri: String,
    _server: MockServer,
}

impl EmbedServer {
    /// 200s: one `{index, embedding}` object per input, `dims` floats wide.
    ///
    /// The `data` array is returned REVERSED and the embedding's first
    /// component carries its own index, so a client that forgets to sort by
    /// `index` produces visibly wrong vectors rather than silently-correct
    /// ones.
    pub async fn ok(dims: usize) -> Self {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ShuffledOk { dims })
            .mount(&server)
            .await;
        Self::wrap(server)
    }

    /// Always 429, with a `retry-after-ms` header set to `retry_after_ms`.
    pub async fn rate_limited(retry_after_ms: u64) -> Self {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("retry-after-ms", retry_after_ms.to_string().as_str()),
            )
            .mount(&server)
            .await;
        Self::wrap(server)
    }

    /// Always `status`, with an empty body.
    pub async fn failing(status: u16) -> Self {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(status))
            .mount(&server)
            .await;
        Self::wrap(server)
    }

    fn wrap(server: MockServer) -> Self {
        Self {
            uri: server.uri(),
            _server: server,
        }
    }
}

/// Responder that sizes its reply to the request's `input` and reverses it.
struct ShuffledOk {
    dims: usize,
}

impl Respond for ShuffledOk {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body: serde_json::Value =
            serde_json::from_slice(&request.body).unwrap_or_else(|_| json!({}));

        // `input` is either a single string or an array of them.
        let n = match &body["input"] {
            serde_json::Value::Array(a) => a.len(),
            serde_json::Value::Null => 0,
            _ => 1,
        };

        let data: Vec<serde_json::Value> = (0..n)
            .map(|i| {
                // Emitted in reverse: position i carries logical index n-1-i.
                let idx = n - 1 - i;
                let mut embedding = vec![0.0f32; self.dims];
                if let Some(first) = embedding.first_mut() {
                    *first = idx as f32;
                }
                json!({ "index": idx, "embedding": embedding })
            })
            .collect();

        ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "model": body["model"],
            "data": data,
        }))
    }
}
