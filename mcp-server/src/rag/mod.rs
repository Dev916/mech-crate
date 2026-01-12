//! RAG Integration with Weaviate
//!
//! Provides documentation retrieval and semantic search capabilities
//! for MechCrate documentation.

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::{McpError, McpResult};

/// Weaviate client for RAG queries
pub struct WeaviateClient {
    url: String,
    client: Client,
}

/// Document from Weaviate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub title: String,
    pub content: String,
    pub category: String,
    pub source: String,
    #[serde(default)]
    pub distance: f32,
}

/// Search result from Weaviate
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WeaviateResponse {
    data: Option<WeaviateData>,
    errors: Option<Vec<WeaviateError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WeaviateData {
    #[serde(rename = "Get")]
    get: Option<WeaviateGet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WeaviateGet {
    #[serde(rename = "MechCrateDoc")]
    mech_crate_doc: Option<Vec<WeaviateDoc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WeaviateDoc {
    title: String,
    content: String,
    category: String,
    source: String,
    #[serde(rename = "_additional")]
    additional: Option<WeaviateAdditional>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WeaviateAdditional {
    distance: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WeaviateError {
    message: String,
}

impl WeaviateClient {
    /// Create a new Weaviate client
    pub fn new(url: &str) -> Self {
        Self {
            url: url.trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    /// Check if Weaviate is available
    pub async fn health_check(&self) -> bool {
        match self.client.get(format!("{}/v1/.well-known/ready", self.url)).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Search for documents by query
    pub async fn search(&self, query: &str, limit: usize) -> McpResult<Vec<Document>> {
        let graphql_query = format!(
            r#"{{
                Get {{
                    MechCrateDoc(
                        nearText: {{
                            concepts: ["{}"]
                        }}
                        limit: {}
                    ) {{
                        title
                        content
                        category
                        source
                        _additional {{
                            distance
                        }}
                    }}
                }}
            }}"#,
            query.replace('"', "\\\""),
            limit
        );

        let response = self
            .client
            .post(format!("{}/v1/graphql", self.url))
            .json(&serde_json::json!({ "query": graphql_query }))
            .send()
            .await
            .map_err(|e| McpError::Weaviate(format!("Request failed: {}", e)))?;

        let result: WeaviateResponse = response
            .json()
            .await
            .map_err(|e| McpError::Weaviate(format!("Failed to parse response: {}", e)))?;

        if let Some(errors) = result.errors {
            if !errors.is_empty() {
                return Err(McpError::Weaviate(
                    errors.iter().map(|e| e.message.clone()).collect::<Vec<_>>().join(", "),
                ));
            }
        }

        let docs = result
            .data
            .and_then(|d| d.get)
            .and_then(|g| g.mech_crate_doc)
            .unwrap_or_default();

        Ok(docs
            .into_iter()
            .map(|d| Document {
                title: d.title,
                content: d.content,
                category: d.category,
                source: d.source,
                distance: d.additional.and_then(|a| a.distance).unwrap_or(0.0),
            })
            .collect())
    }

    /// Search by category
    pub async fn search_by_category(&self, query: &str, category: &str, limit: usize) -> McpResult<Vec<Document>> {
        let graphql_query = format!(
            r#"{{
                Get {{
                    MechCrateDoc(
                        nearText: {{
                            concepts: ["{}"]
                        }}
                        where: {{
                            path: ["category"]
                            operator: Equal
                            valueText: "{}"
                        }}
                        limit: {}
                    ) {{
                        title
                        content
                        category
                        source
                        _additional {{
                            distance
                        }}
                    }}
                }}
            }}"#,
            query.replace('"', "\\\""),
            category,
            limit
        );

        let response = self
            .client
            .post(format!("{}/v1/graphql", self.url))
            .json(&serde_json::json!({ "query": graphql_query }))
            .send()
            .await
            .map_err(|e| McpError::Weaviate(format!("Request failed: {}", e)))?;

        let result: WeaviateResponse = response
            .json()
            .await
            .map_err(|e| McpError::Weaviate(format!("Failed to parse response: {}", e)))?;

        let docs = result
            .data
            .and_then(|d| d.get)
            .and_then(|g| g.mech_crate_doc)
            .unwrap_or_default();

        Ok(docs
            .into_iter()
            .map(|d| Document {
                title: d.title,
                content: d.content,
                category: d.category,
                source: d.source,
                distance: d.additional.and_then(|a| a.distance).unwrap_or(0.0),
            })
            .collect())
    }

}

/// Format search results for display
pub fn format_search_results(docs: &[Document]) -> String {
    if docs.is_empty() {
        return "No relevant documentation found.".to_string();
    }

    let mut result = String::new();
    
    for (i, doc) in docs.iter().enumerate() {
        result.push_str(&format!("## {} [{}]\n\n", doc.title, doc.category));
        result.push_str(&doc.content);
        result.push_str("\n\n");
        
        if i < docs.len() - 1 {
            result.push_str("---\n\n");
        }
    }

    result
}
