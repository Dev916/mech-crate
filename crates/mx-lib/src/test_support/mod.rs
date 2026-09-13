//! Shared test fixtures for the mx workspace.
//!
//! Gated behind the `test-support` feature so neither the fixtures nor their
//! dependencies (`wiremock`, `tempfile`) are ever linked into a release
//! binary. Other crates pull them in as a dev-dependency:
//!
//! ```toml
//! [dev-dependencies]
//! mx-lib = { path = "../mx-lib", features = ["test-support"] }
//! ```
//!
//! These are the adapter fakes the ports in this crate are contract-tested
//! against: [`StubBin`] fakes the process-execution edge, [`EmbedServer`]
//! fakes the embeddings HTTP edge, [`scaffold_project`] fakes the
//! on-disk project an mx command expects to find, and [`write_fake_bundle`]
//! / [`pack_bundle`] fake a published release for the self-updater.

mod bundle;
mod embed_server;
mod scaffold;
mod stub_bin;

pub use bundle::{pack_bundle, sha256_sidecar, write_fake_bundle};
pub use embed_server::EmbedServer;
pub use scaffold::scaffold_project;
pub use stub_bin::StubBin;

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn stub_bin_intercepts_and_records() {
        let sb = StubBin::new();
        sb.stub("docker", 0, "STUBBED-DOCKER-OK");
        let out = Command::new("docker")
            .args(["network", "inspect", "devmesh-traefik"])
            .env("PATH", sb.path_env())
            .output()
            .unwrap();
        assert!(out.status.success());
        assert_eq!(
            String::from_utf8_lossy(&out.stdout).trim(),
            "STUBBED-DOCKER-OK"
        );
        let calls = sb.invocations("docker");
        assert_eq!(calls.len(), 1);
        assert!(calls[0].contains("network inspect devmesh-traefik"));
    }

    #[test]
    fn stub_bin_scripts_exit_codes() {
        let sb = StubBin::new();
        sb.stub("docker", 3, "");
        let out = Command::new("docker")
            .env("PATH", sb.path_env())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(3));
    }

    #[test]
    fn scaffold_passes_strict_detection() {
        let dir = tempfile::tempdir().unwrap();
        scaffold_project(dir.path());
        let det = crate::project::ProjectDetector::strict();
        assert!(det.is_project(dir.path()));
        assert!(dir.path().join("docker/compose").is_dir());
    }

    #[tokio::test]
    async fn embed_server_modes() {
        let ok = EmbedServer::ok(4).await;
        let resp = reqwest::Client::new()
            .post(format!("{}/embeddings", ok.uri))
            .json(&serde_json::json!({"model":"m","input":["a","b"]}))
            .send()
            .await
            .unwrap();
        assert!(resp.status().is_success());
        let v: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(v["data"].as_array().unwrap().len(), 2);

        let rl = EmbedServer::rate_limited(1200).await;
        let resp = reqwest::Client::new()
            .post(format!("{}/embeddings", rl.uri))
            .json(&serde_json::json!({"model":"m","input":"a"}))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 429);
        assert_eq!(resp.headers()["retry-after-ms"], "1200");

        let boom = EmbedServer::failing(500).await;
        let resp = reqwest::Client::new()
            .post(format!("{}/embeddings", boom.uri))
            .json(&serde_json::json!({"model":"m","input":"a"}))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 500);
    }

    /// The happy-path server returns `data` out of order on purpose, so a
    /// client that ignores `index` is caught. Pin that contract.
    #[tokio::test]
    async fn embed_server_ok_shuffles_and_sizes_vectors() {
        let ok = EmbedServer::ok(3).await;
        let v: serde_json::Value = reqwest::Client::new()
            .post(format!("{}/embeddings", ok.uri))
            .json(&serde_json::json!({"model":"m","input":["a","b","c"]}))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let data = v["data"].as_array().unwrap();
        assert_eq!(data.len(), 3);
        // Reversed: position 0 carries logical index 2.
        let indices: Vec<u64> = data.iter().map(|d| d["index"].as_u64().unwrap()).collect();
        assert_eq!(indices, vec![2, 1, 0]);
        for item in data {
            let e = item["embedding"].as_array().unwrap();
            assert_eq!(e.len(), 3, "embedding width must equal requested dims");
            assert_eq!(e[0].as_f64().unwrap(), item["index"].as_f64().unwrap());
        }
    }
}
