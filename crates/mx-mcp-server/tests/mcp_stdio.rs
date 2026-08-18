//! MCP stdio integration harness.
//!
//! Spawns the real `mx-mcp` binary and drives line-delimited JSON-RPC over its
//! stdin/stdout: initialize → tools/list → tools/call. Every spawn pins BOTH
//! `MX_RAG_DATABASE_URL` and `MX_RAG_FALLBACK_DATABASE_URL` so a developer's
//! `~/.mech-crate/config/rag.toml` (which may point at Neon) can never leak
//! into a test run — env vars win over the config file in `RagConfig::load`.
//!
//! Child cleanup is a `Drop` guard: stdin closed, `kill()`, `wait()`. Nothing
//! survives a panicking test, so the suite adds no orphan `mx-mcp` processes.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

/// Unreachable Postgres: port 1 is never bound, so `connect` fails fast.
const DEAD_DB: &str = "postgres://postgres@localhost:1/nope";
/// Upper bound on any single request/response round trip.
const RPC_TIMEOUT: Duration = Duration::from_secs(30);
/// Upper bound on a graceful (stdin-EOF driven) child exit.
const EXIT_TIMEOUT: Duration = Duration::from_secs(15);

/// Repo root — the server needs a MechCrate root and must not go hunting for one.
fn repo_root() -> String {
    format!("{}/../..", env!("CARGO_MANIFEST_DIR"))
}

struct McpChild {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<String>,
    reader: Option<JoinHandle<()>>,
}

impl McpChild {
    fn spawn(envs: &[(&str, &str)]) -> Self {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_mx-mcp"));
        cmd.arg("--mech-crate-root")
            .arg(repo_root())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // Logs go to stderr; discard them so a chatty child can never block
            // on a pipe nobody drains.
            .stderr(Stdio::null());
        for (k, v) in envs {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn().expect("spawn mx-mcp");
        let stdin = child.stdin.take().expect("child stdin");
        let stdout = child.stdout.take().expect("child stdout");

        // Read stdout on a helper thread so `recv_id` can honour a deadline
        // instead of blocking forever on a wedged child.
        let (tx, lines) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                match line {
                    Ok(l) => {
                        if tx.send(l).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            child,
            stdin: Some(stdin),
            lines,
            reader: Some(reader),
        }
    }

    fn send(&mut self, v: Value) {
        let stdin = self.stdin.as_mut().expect("stdin open");
        writeln!(stdin, "{v}").expect("write request");
        stdin.flush().expect("flush request");
    }

    /// Read responses until one carries `id`, or the deadline expires.
    fn recv_id(&mut self, id: u64) -> Value {
        let deadline = Instant::now() + RPC_TIMEOUT;
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .unwrap_or_default();
            match self.lines.recv_timeout(remaining) {
                Ok(line) => {
                    if let Ok(v) = serde_json::from_str::<Value>(&line) {
                        if v["id"] == json!(id) {
                            return v;
                        }
                    }
                }
                Err(RecvTimeoutError::Timeout) => panic!("timeout waiting for id {id}"),
                Err(RecvTimeoutError::Disconnected) => panic!("eof before id {id}"),
            }
        }
    }

    fn init(&mut self) {
        self.send(json!({
            "jsonrpc": "2.0", "id": 0, "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "mcp_stdio", "version": "0"}
            }
        }));
        let v = self.recv_id(0);
        assert_eq!(
            v["result"]["serverInfo"]["name"], "mx-mcp-server",
            "unexpected initialize result: {v}"
        );
    }

    /// Close stdin and wait for the child to exit on its own (EOF shutdown).
    fn wait_for_clean_exit(&mut self) -> ExitStatus {
        drop(self.stdin.take());
        let deadline = Instant::now() + EXIT_TIMEOUT;
        loop {
            match self.child.try_wait().expect("try_wait") {
                Some(status) => return status,
                None => {
                    assert!(
                        Instant::now() < deadline,
                        "child did not exit after stdin EOF"
                    );
                    std::thread::sleep(Duration::from_millis(25));
                }
            }
        }
    }

    fn pid(&self) -> u32 {
        self.child.id()
    }
}

impl Drop for McpChild {
    fn drop(&mut self) {
        drop(self.stdin.take());
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(h) = self.reader.take() {
            let _ = h.join();
        }
    }
}

/// Is `pid` still a live process? Used to prove the Drop guard reaps children.
fn pid_alive(pid: u32) -> bool {
    Command::new("ps")
        .args(["-p", &pid.to_string()])
        .output()
        .expect("run ps")
        .status
        .success()
}

#[test]
fn tools_list_includes_rag_context() {
    let mut c = McpChild::spawn(&[
        ("MX_RAG_DATABASE_URL", DEAD_DB),
        ("MX_RAG_FALLBACK_DATABASE_URL", DEAD_DB),
    ]);
    c.init();
    c.send(json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}));
    let v = c.recv_id(1);
    let names: Vec<&str> = v["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(names.contains(&"rag_context"), "tools/list: {names:?}");
    assert!(names.contains(&"rag_health"), "tools/list: {names:?}");
}

#[test]
fn rag_health_reports_reachable_backend() {
    // Skip (not fail) when no test database is configured — same contract as
    // the mx-lib corpus tests. `#[ignore]` is reserved for the known-broken lane.
    let Ok(url) = std::env::var("MX_RAG_TEST_DATABASE_URL") else {
        return;
    };
    // Both URLs pinned at the test container: whichever one wins, the backend
    // label may read "neon" (primary) or "local" (fallback) — the claim under
    // test is reachability, not the label.
    let mut c = McpChild::spawn(&[
        ("MX_RAG_DATABASE_URL", &url),
        ("MX_RAG_FALLBACK_DATABASE_URL", &url),
    ]);
    c.init();
    c.send(json!({
        "jsonrpc": "2.0", "id": 2, "method": "tools/call",
        "params": {"name": "rag_health", "arguments": {}}
    }));
    let v = c.recv_id(2);
    let text = v["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let status: Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("rag_health should return JSON status, got {text:?} ({e})"));
    let backend = status["backend"].as_str().unwrap_or_default();
    assert!(
        backend == "local" || backend == "neon",
        "unreachable backend in {text}"
    );
    assert!(status["chunks"].is_number(), "no chunk count in {text}");
}

#[test]
fn offline_rag_health_is_graceful() {
    let mut c = McpChild::spawn(&[
        ("MX_RAG_DATABASE_URL", DEAD_DB),
        ("MX_RAG_FALLBACK_DATABASE_URL", DEAD_DB),
    ]);
    c.init();
    c.send(json!({
        "jsonrpc": "2.0", "id": 3, "method": "tools/call",
        "params": {"name": "rag_health", "arguments": {}}
    }));
    let v = c.recv_id(3);
    let text = v["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(text.contains("offline"), "{text}");
    assert!(text.contains("rag.toml"), "{text}");
    // An unreachable corpus degrades the RAG tools, never the process.
    let status = c.wait_for_clean_exit();
    assert!(status.success(), "child exited with {status}");
}

#[test]
fn dropping_the_harness_leaves_no_orphan() {
    let pid = {
        let c = McpChild::spawn(&[
            ("MX_RAG_DATABASE_URL", DEAD_DB),
            ("MX_RAG_FALLBACK_DATABASE_URL", DEAD_DB),
        ]);
        let pid = c.pid();
        assert!(pid_alive(pid), "child {pid} never started");
        pid
    };
    assert!(!pid_alive(pid), "orphan mx-mcp left behind at pid {pid}");
}
