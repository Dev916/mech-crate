//! Embedding providers behind the `EmbeddingProvider` trait (adapter pattern).
//!
//! First implementation: any OpenAI-compatible `/embeddings` endpoint
//! (OpenAI, Ollama's /v1, LM Studio, ...). Model + dims are recorded per
//! chunk in the store, so a provider switch cannot silently mix vector spaces.

use async_trait::async_trait;
use serde_json::json;

/// Max inputs per `/embeddings` request (hq convention).
pub const EMBED_SUB_BATCH: usize = 256;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    fn model(&self) -> &str;
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>>;
}

pub struct OpenAiCompatEmbedder {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAiCompatEmbedder {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }

    fn parse_embedding(item: &serde_json::Value) -> anyhow::Result<Vec<f32>> {
        let arr = item["embedding"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("missing embedding in response"))?;
        Ok(arr
            .iter()
            .filter_map(|x| x.as_f64().map(|f| f as f32))
            .collect())
    }

    async fn post(&self, input: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let resp = self
            .http
            .post(format!("{}/embeddings", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&json!({ "model": self.model, "input": input }))
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("embeddings http {}", resp.status()));
        }
        Ok(resp.json().await?)
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAiCompatEmbedder {
    fn model(&self) -> &str {
        &self.model
    }

    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let v = self.post(json!(text)).await?;
        Self::parse_embedding(&v["data"][0])
    }

    /// Sub-batches at [`EMBED_SUB_BATCH`]; sorts each response by `data[i].index`
    /// so output order matches input order regardless of API ordering.
    async fn embed_batch(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        let mut out: Vec<Vec<f32>> = Vec::with_capacity(texts.len());
        for sub in texts.chunks(EMBED_SUB_BATCH) {
            let v = self.post(json!(sub)).await?;
            let data = v["data"]
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("missing embeddings data"))?;
            let mut indexed: Vec<(usize, Vec<f32>)> = Vec::with_capacity(data.len());
            for (i, item) in data.iter().enumerate() {
                let idx = item["index"].as_u64().map(|x| x as usize).unwrap_or(i);
                indexed.push((idx, Self::parse_embedding(item)?));
            }
            indexed.sort_by_key(|(i, _)| *i);
            out.extend(indexed.into_iter().map(|(_, e)| e));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn embedding_response(n: usize) -> serde_json::Value {
        let data: Vec<_> = (0..n)
            .map(|i| serde_json::json!({ "index": n - 1 - i, "embedding": [ (n - 1 - i) as f64, 0.0 ] }))
            .collect();
        serde_json::json!({ "data": data })
    }

    #[tokio::test]
    async fn embed_single_parses_vector() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(embedding_response(1)))
            .mount(&server)
            .await;
        let e = OpenAiCompatEmbedder::new(&server.uri(), "test-key", "test-model");
        let v = e.embed("hello").await.unwrap();
        assert_eq!(v.len(), 2);
    }

    #[tokio::test]
    async fn embed_batch_orders_by_index() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(embedding_response(3)))
            .mount(&server)
            .await;
        let e = OpenAiCompatEmbedder::new(&server.uri(), "k", "m");
        let texts: Vec<String> = (0..3).map(|i| format!("t{i}")).collect();
        let vs = e.embed_batch(&texts).await.unwrap();
        // Response listed indices in reverse; output must be input-ordered.
        assert_eq!(vs[0][0], 0.0);
        assert_eq!(vs[2][0], 2.0);
    }

    #[tokio::test]
    async fn http_error_is_err() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;
        let e = OpenAiCompatEmbedder::new(&server.uri(), "k", "m");
        assert!(e.embed("x").await.is_err());
    }
}
