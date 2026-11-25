//! Optional relevance stage behind retrieval (TRIAL, off by default).
//!
//! When `MX_RAG_RERANK=jev` and a key is configured, every retrieved chunk is
//! put to a decision model as one closed question ("does this passage answer
//! the query?"). Chunks whose probability falls below the threshold are
//! dropped; when every judged chunk is dropped the search abstains (no hits).
//!
//! Rules that hold whatever the judge does:
//! - Retrieval order is preserved. The stage filters, it does not re-sort.
//! - Unjudged chunks are kept. A chunk is unjudged when its doc path is under
//!   a private prefix (never sent to the third party) or when its judge call
//!   failed (fail open: a provider outage degrades to plain retrieval).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde_json::json;

use super::config::RerankConfig;
use super::store::TechHit;

/// Max concurrent judge calls per search (the public endpoint rate-limits).
const JUDGE_CONCURRENCY: usize = 8;
const JUDGE_TIMEOUT: Duration = Duration::from_secs(5);
/// Over-fetch cap, so a large `limit` cannot fan out without bound.
const MAX_CANDIDATES: i64 = 20;
const QUESTION_ID: &str = "answers_query";
/// One atomic judgement, worded literally (the model reads questions literally).
const QUESTION: &str =
    "Does `passage.text` contain information that directly helps answer `query`?";

/// One judge answer: probability that the passage answers the query.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Judgement {
    pub probability: f64,
    /// Billable input tokens reported by the provider (0 when unknown).
    pub input_tokens: u64,
}

/// The passage as sent to the judge.
#[derive(Debug, Clone)]
pub struct Passage {
    pub title: String,
    pub heading_path: String,
    pub text: String,
}

#[async_trait]
pub trait RelevanceJudge: Send + Sync {
    fn model(&self) -> &str;
    async fn judge(&self, query: &str, passage: &Passage) -> anyhow::Result<Judgement>;
}

/// TypeSafe System One (`POST {base}/systemone`), one noul question per call.
pub struct JevJudge {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl JevJudge {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}

#[async_trait]
impl RelevanceJudge for JevJudge {
    fn model(&self) -> &str {
        &self.model
    }

    async fn judge(&self, query: &str, passage: &Passage) -> anyhow::Result<Judgement> {
        let body = json!({
            "model": self.model,
            "state": {
                "query": query,
                "passage": {
                    "title": passage.title,
                    "heading": passage.heading_path,
                    "text": passage.text,
                },
            },
            "questions": {
                QUESTION_ID: {
                    "type": "noul",
                    "instructions": QUESTION,
                },
            },
        });
        let resp = self
            .http
            .post(format!("{}/systemone", self.base_url))
            .bearer_auth(&self.api_key)
            .timeout(JUDGE_TIMEOUT)
            .json(&body)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("systemone http {}", resp.status()));
        }
        let v: serde_json::Value = resp.json().await?;
        let probability = v["answers"][QUESTION_ID]["noul"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("missing noul answer in response"))?;
        Ok(Judgement {
            probability,
            input_tokens: v["usage"]["input_tokens"].as_u64().unwrap_or(0),
        })
    }
}

/// The filter stage: judge, drop below threshold, keep order, truncate.
pub struct RerankStage {
    judge: Arc<dyn RelevanceJudge>,
    threshold: f64,
    private_prefixes: Vec<String>,
    trace_path: Option<PathBuf>,
}

impl RerankStage {
    pub fn new(
        judge: Arc<dyn RelevanceJudge>,
        threshold: f64,
        private_prefixes: Vec<String>,
    ) -> Self {
        Self {
            judge,
            threshold,
            private_prefixes,
            trace_path: None,
        }
    }

    pub fn with_trace(mut self, path: Option<PathBuf>) -> Self {
        self.trace_path = path;
        self
    }

    /// Build from config. `None` unless the stage is switched on AND usable,
    /// so the default path through `CorpusStore::search` is untouched.
    pub fn from_config(cfg: &RerankConfig) -> Option<Self> {
        let provider = cfg.provider.as_deref()?;
        if provider != "jev" {
            tracing::warn!("unknown MX_RAG_RERANK provider ({provider}): relevance stage off");
            return None;
        }
        let Some(key) = cfg.api_key.as_deref() else {
            tracing::warn!("MX_RAG_RERANK=jev but no MX_RAG_JEV_API_KEY: relevance stage off");
            return None;
        };
        let judge = Arc::new(JevJudge::new(&cfg.base_url, key, &cfg.model));
        Some(
            Self::new(judge, cfg.threshold, cfg.private_prefixes.clone())
                .with_trace(cfg.trace_path.clone()),
        )
    }

    /// How many candidates to retrieve so dropped hits can be backfilled.
    /// Never fewer than `limit`.
    pub fn candidates(&self, limit: i64) -> i64 {
        (limit * 2).min(MAX_CANDIDATES).max(limit)
    }

    fn is_private(&self, path: &str) -> bool {
        self.private_prefixes
            .iter()
            .any(|p| path.starts_with(p.as_str()))
    }

    /// Filter `hits` (retrieval order) down to at most `limit` kept hits.
    pub async fn apply(&self, query: &str, hits: Vec<TechHit>, limit: usize) -> Vec<TechHit> {
        let started = Instant::now();
        let permits = Arc::new(tokio::sync::Semaphore::new(JUDGE_CONCURRENCY));
        let mut calls = tokio::task::JoinSet::new();
        for (i, h) in hits.iter().enumerate() {
            if self.is_private(&h.path) {
                continue;
            }
            let judge = self.judge.clone();
            let permits = permits.clone();
            let query = query.to_string();
            let passage = Passage {
                title: h.title.clone(),
                heading_path: h.heading_path.clone(),
                text: h.content.clone(),
            };
            calls.spawn(async move {
                let _permit = permits.acquire().await;
                (i, judge.judge(&query, &passage).await)
            });
        }
        // None = unjudged (private, or the call failed): kept.
        let mut verdicts: Vec<Option<Judgement>> = vec![None; hits.len()];
        let mut failed = vec![false; hits.len()];
        while let Some(joined) = calls.join_next().await {
            match joined {
                Ok((i, Ok(j))) => verdicts[i] = Some(j),
                Ok((i, Err(e))) => {
                    failed[i] = true;
                    tracing::warn!("relevance judge failed ({e}); keeping chunk unjudged");
                }
                Err(e) => tracing::warn!("relevance judge task failed ({e})"),
            }
        }
        let keep: Vec<bool> = verdicts
            .iter()
            .map(|v| v.map(|j| j.probability >= self.threshold).unwrap_or(true))
            .collect();
        if let Some(path) = &self.trace_path {
            self.write_trace(path, query, &hits, &verdicts, &failed, &keep, started);
        }
        hits.into_iter()
            .zip(keep)
            .filter_map(|(h, k)| k.then_some(h))
            .take(limit)
            .collect()
    }

    /// Append one JSON line describing this search. Measurement aid only:
    /// every failure is traced and dropped. Passage text is never written.
    #[allow(clippy::too_many_arguments)]
    fn write_trace(
        &self,
        path: &std::path::Path,
        query: &str,
        hits: &[TechHit],
        verdicts: &[Option<Judgement>],
        failed: &[bool],
        keep: &[bool],
        started: Instant,
    ) {
        use std::io::Write;
        let chunks: Vec<serde_json::Value> = hits
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let status = match (verdicts[i], failed[i]) {
                    (Some(_), _) => "judged",
                    (None, true) => "error",
                    (None, false) => "private",
                };
                json!({
                    "path": h.path,
                    "heading_path": h.heading_path,
                    "score": h.score,
                    "status": status,
                    "probability": verdicts[i].map(|j| j.probability),
                    "kept": keep[i],
                })
            })
            .collect();
        let line = json!({
            "query": query,
            "model": self.judge.model(),
            "threshold": self.threshold,
            "stage_ms": started.elapsed().as_secs_f64() * 1000.0,
            "input_tokens": verdicts.iter().flatten().map(|j| j.input_tokens).sum::<u64>(),
            "chunks": chunks,
        });
        let res = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut f| writeln!(f, "{line}"));
        if let Err(e) = res {
            tracing::debug!("relevance trace write failed (ignored): {e}");
        }
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Judge scripted by passage text: a probability, or an error when absent.
    #[derive(Default)]
    pub(crate) struct FakeJudge {
        pub answers: HashMap<String, f64>,
        pub seen: Mutex<Vec<String>>,
    }

    impl FakeJudge {
        pub(crate) fn new(answers: &[(&str, f64)]) -> Self {
            Self {
                answers: answers.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                seen: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl RelevanceJudge for FakeJudge {
        fn model(&self) -> &str {
            "fake"
        }

        async fn judge(&self, _query: &str, passage: &Passage) -> anyhow::Result<Judgement> {
            self.seen.lock().unwrap().push(passage.text.clone());
            match self.answers.get(&passage.text) {
                Some(p) => Ok(Judgement {
                    probability: *p,
                    input_tokens: 10,
                }),
                None => Err(anyhow::anyhow!("scripted failure")),
            }
        }
    }

    pub(crate) fn hit(path: &str, content: &str) -> TechHit {
        TechHit {
            title: "T".into(),
            path: path.into(),
            heading_path: "T > H".into(),
            category: "patterns".into(),
            languages: vec![],
            summary: None,
            use_cases: vec![],
            content: content.into(),
            score: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::testing::{hit, FakeJudge};
    use super::*;
    use wiremock::matchers::{body_partial_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn stage(judge: Arc<FakeJudge>, threshold: f64) -> RerankStage {
        RerankStage::new(judge, threshold, vec!["repos/".to_string()])
    }

    fn contents(hits: &[TechHit]) -> Vec<&str> {
        hits.iter().map(|h| h.content.as_str()).collect()
    }

    #[tokio::test]
    async fn drops_below_threshold_and_preserves_retrieval_order() {
        let judge = Arc::new(FakeJudge::new(&[("a", 0.9), ("b", 0.1), ("c", 0.6)]));
        let hits = vec![hit("x.md", "a"), hit("x.md", "b"), hit("y.md", "c")];
        let out = stage(judge, 0.5).apply("q", hits, 5).await;
        // c scores below a on relevance but order is retrieval order, not judge order.
        assert_eq!(contents(&out), vec!["a", "c"]);
    }

    #[tokio::test]
    async fn threshold_is_inclusive() {
        let judge = Arc::new(FakeJudge::new(&[("a", 0.5)]));
        let out = stage(judge, 0.5)
            .apply("q", vec![hit("x.md", "a")], 5)
            .await;
        assert_eq!(out.len(), 1);
    }

    #[tokio::test]
    async fn abstains_when_every_hit_is_below_threshold() {
        let judge = Arc::new(FakeJudge::new(&[("a", 0.2), ("b", 0.1)]));
        let hits = vec![hit("x.md", "a"), hit("x.md", "b")];
        let out = stage(judge, 0.5).apply("q", hits, 5).await;
        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn private_chunks_are_never_sent_and_are_kept() {
        let judge = Arc::new(FakeJudge::new(&[("public", 0.1)]));
        let hits = vec![hit("repos/hq.md", "secret"), hit("x.md", "public")];
        let out = stage(judge.clone(), 0.5).apply("q", hits, 5).await;
        assert_eq!(contents(&out), vec!["secret"]);
        assert_eq!(*judge.seen.lock().unwrap(), vec!["public".to_string()]);
    }

    #[tokio::test]
    async fn judge_failure_fails_open_per_chunk() {
        // "b" has no scripted answer -> judge errors -> b is kept unjudged.
        let judge = Arc::new(FakeJudge::new(&[("a", 0.1)]));
        let hits = vec![hit("x.md", "a"), hit("x.md", "b")];
        let out = stage(judge, 0.5).apply("q", hits, 5).await;
        assert_eq!(contents(&out), vec!["b"]);
    }

    #[tokio::test]
    async fn backfills_from_overfetch_then_truncates_to_limit() {
        let judge = Arc::new(FakeJudge::new(&[
            ("a", 0.1),
            ("b", 0.9),
            ("c", 0.9),
            ("d", 0.9),
        ]));
        let hits = vec![
            hit("x.md", "a"),
            hit("x.md", "b"),
            hit("x.md", "c"),
            hit("x.md", "d"),
        ];
        let st = stage(judge, 0.5);
        assert_eq!(st.candidates(2), 4);
        assert_eq!(st.candidates(50), 50, "never fetch fewer than the limit");
        assert_eq!(st.candidates(15), 20, "over-fetch is capped");
        let out = st.apply("q", hits, 2).await;
        assert_eq!(contents(&out), vec!["b", "c"]);
    }

    #[tokio::test]
    async fn trace_file_records_each_judgement() {
        let dir = tempfile::tempdir().unwrap();
        let trace = dir.path().join("trace.jsonl");
        let judge = Arc::new(FakeJudge::new(&[("a", 0.9), ("b", 0.1)]));
        let hits = vec![hit("x.md", "a"), hit("x.md", "b"), hit("repos/r.md", "p")];
        stage(judge, 0.5)
            .with_trace(Some(trace.clone()))
            .apply("the query", hits, 5)
            .await;
        let line = std::fs::read_to_string(&trace).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["query"], "the query");
        assert_eq!(v["model"], "fake");
        assert_eq!(v["input_tokens"], 20);
        assert_eq!(v["chunks"][0]["probability"], 0.9);
        assert_eq!(v["chunks"][1]["kept"], false);
        assert_eq!(v["chunks"][2]["status"], "private");
        assert!(v["chunks"][2]["probability"].is_null());
        // Private chunk text never reaches the trace either.
        assert!(!line.contains("\"p\""));
    }

    fn cfg(provider: Option<&str>, key: Option<&str>) -> RerankConfig {
        RerankConfig {
            provider: provider.map(String::from),
            api_key: key.map(String::from),
            ..RerankConfig::default()
        }
    }

    #[test]
    fn from_config_is_off_unless_provider_and_key_are_both_set() {
        assert!(RerankStage::from_config(&cfg(None, None)).is_none());
        assert!(RerankStage::from_config(&cfg(None, Some("k"))).is_none());
        assert!(RerankStage::from_config(&cfg(Some("jev"), None)).is_none());
        assert!(RerankStage::from_config(&cfg(Some("other"), Some("k"))).is_none());
        assert!(RerankStage::from_config(&cfg(Some("jev"), Some("k"))).is_some());
    }

    fn passage() -> Passage {
        Passage {
            title: "Doc".into(),
            heading_path: "Doc > Section".into(),
            text: "body text".into(),
        }
    }

    #[tokio::test]
    async fn jev_judge_sends_one_noul_and_parses_probability() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/systemone"))
            .and(header("authorization", "Bearer test-key"))
            .and(body_partial_json(serde_json::json!({
                "model": "jev-test",
                "state": {
                    "query": "how do I retry?",
                    "passage": { "title": "Doc", "heading": "Doc > Section", "text": "body text" }
                },
                "questions": { "answers_query": { "type": "noul" } }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "model": "jev-test",
                "answers": { "answers_query": { "type": "noul", "noul": 0.83 } },
                "usage": { "input_tokens": 344, "output_tokens": 1 }
            })))
            .mount(&server)
            .await;
        let j = JevJudge::new(&server.uri(), "test-key", "jev-test");
        let got = j.judge("how do I retry?", &passage()).await.unwrap();
        assert_eq!(
            got,
            Judgement {
                probability: 0.83,
                input_tokens: 344
            }
        );
    }

    #[tokio::test]
    async fn jev_judge_http_error_and_malformed_body_are_errors() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/systemone"))
            .respond_with(ResponseTemplate::new(429))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        let j = JevJudge::new(&server.uri(), "k", "m");
        assert!(j.judge("q", &passage()).await.is_err());

        Mock::given(method("POST"))
            .and(path("/systemone"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "answers": {} })),
            )
            .mount(&server)
            .await;
        assert!(j.judge("q", &passage()).await.is_err());
    }
}
