# Solana RPC appendix (LLM-safe)

Concise, production-minded notes for using Solana JSON-RPC and websockets safely from code or an LLM agent. Focus on correctness, fee control, and avoiding common footguns.

## Core mental model

- **Stateless RPC**: Servers do not track client sessions. Always pass the right context (commitment, minContextSlot, maxSupportedTransactionVersion).
- **Commitment levels**: `processed` (optimistic, lowest latency), `confirmed` (voted by cluster supermajority), `finalized` (max safety, slowest). Reads default to `finalized`; writes should preflight at `processed|confirmed` and then poll for `confirmed/finalized`.
- **Blockhash expiry**: Transactions expire ~2 minutes after the latest blockhash they include. Refresh before signing and cache the slot.
- **Message versions**: Legacy vs v0 (Address Lookup Tables). Ask for `maxSupportedTransactionVersion` in `getLatestBlockhash` and `getFeeForMessage` to avoid “unsupported transaction version” errors.
- **Compute + fees**: Total CU per tx is capped (~1.4M). Use Compute Budget instructions to set `ComputeUnitLimit` and `ComputeUnitPrice` (micro-lamports/CU) before other instructions.

## Building and sending transactions

- Fetch blockhash with context: `getLatestBlockhash({ commitment: "confirmed", minContextSlot })`; include returned `lastValidBlockHeight` to know expiry.
- Fetch rent/fees: `getFeeForMessage` on the compiled message. For priority fees, either:
  - Use `getRecentPrioritizationFees` and pick a percentile per cu need (e.g., median or p75).
  - On Helius/other enhanced RPCs, `getPriorityFeeEstimate` per account list if available.
- Set Compute Budget first in the instruction list:
  - `ComputeBudgetProgram.setComputeUnitLimit` to the measured ceiling (see simulation).
  - `ComputeBudgetProgram.setComputeUnitPrice` to your bid.
- Simulation before send:
  - `simulateTransaction` with `replaceRecentBlockhash: true`, `sigVerify: false`, `commitment: "processed"`.
  - Inspect logs, `err`, and `unitsConsumed`. Adjust CU limit/bid accordingly.
- Signing pipeline:
  - Collect all required signers; avoid partial-sign leftovers.
  - For v0, resolve Address Lookup Tables (ALTs) via `getAddressLookupTable`; keep cache by slot and refresh when context slot increases.
  - Encode as base64; never send private keys to RPC. Signing happens locally/HSM.
- Submission & confirmation:
  - Send with `sendTransaction` or `sendRawTransaction` + `preflightCommitment: "processed"`; set `maxRetries` sensibly (e.g., 3–6).
  - Poll `getSignatureStatuses` with `searchTransactionHistory: true` after blockhash expiry to handle reorgs.
  - Treat `BlockhashNotFound` as “expired”; rebuild with a new hash.

## Reading data effectively

- **Avoid chatty per-account calls**: Prefer `getMultipleAccounts` over N `getAccountInfo` calls; group by 100 max.
- **Program scans**: `getProgramAccounts` with `filters` (memcmp, dataSize) and `dataSlice` to avoid large payloads. For large tables, page via `before/after` on `getSignaturesForAddress` + replay only changed accounts.
- **Encoding**: Use `base64` for binary layouts; `jsonParsed` is convenient but brittle across RPCs. When you must use parsed, assert program id matches expectations.
- **Slot control**: Pass `minContextSlot` on reads when you have a recent slot from prior calls to prevent stale responses.
- **Blocks and transactions**:
  - `getBlock`/`getTransaction` with `encoding: "jsonParsed"` is heavy; prefer `"json"` + decode locally for speed.
  - When following finality, request `commitment: "confirmed"` for near-real-time UIs; `finalized` for analytics.
- **Caching**: Cache recent account data and slots; invalidate on slot jumps or `logsSubscribe` notifications for watched accounts.

## Websocket subscriptions

- Use WS for push: `logsSubscribe`, `programSubscribe`, `signatureSubscribe`, `slotSubscribe`.
- Always pass `commitment`. For `logsSubscribe`, include `mentions` filters to cut noise.
- Reconnect strategy:
  - Exponential backoff with jitter.
  - On reconnect, resubscribe and, if using `signatureSubscribe`, also poll `getSignatureStatuses` to fill gaps.
  - Track last seen slot; if drift > a few slots, refresh watched accounts.

## Reliability and rate limits

- Batch JSON-RPC over HTTP where possible (many providers support 100 batch items).
- Reuse HTTP agents with keep-alive to reduce TLS churn.
- Backoff on 429/502/503; retry idempotent reads, but for sends prefer rebuilding with a fresh blockhash after a bounded number of failures.
- Distinguish **preflight errors** (deterministic) from **transport errors** (retryable). Do not blindly retry a transaction that failed compute or program assertions—fix the instruction payload.
- Watch for **RPC equivocation** across providers; for high assurance, cross-check critical reads (e.g., governance tallies) against a second endpoint.

## Batching and transport optimizations

- HTTP batching: group related calls (e.g., `getMultipleAccounts`, `getFeeForMessage`, `simulateTransaction`) into a single batch request; keep batches <100 items to avoid request size limits.
- Prefer bulk-friendly methods: use `getMultipleAccounts` over N `getAccountInfo`, and `getLatestBlockhash` once per burst of sends.
- Connection reuse: enable HTTP/1.1 keep-alive or HTTP/2 if provider supports it; tune agent pools to match concurrency (e.g., 8–32 sockets).
- Compression: enable gzip/deflate for read-heavy workloads; avoid compressing write payloads with signatures if latency-sensitive.
- Client-side coalescing: debounce identical reads across a tick (e.g., wallet UIs requesting the same balances) to cut duplicate RPCs.
- Concurrency shaping: cap in-flight requests and use token buckets per route; serialize blockhash + fee fetches to avoid stampede and cache per slot.
- Batch result hygiene: keep a map of `id -> request meta` so you can attribute errors to calls; drop entire batch on malformed response rather than guessing.
- Size budgeting: keep batch payloads under ~512 KB to avoid provider rejection; slice large account lists into multiple batches.
- Priority fee probing: batch `getRecentPrioritizationFees` for hot accounts once per slot, cache by slot, and reuse across transactions in that slot.

## Streaming and websockets together

- Use websockets for push (`logsSubscribe`, `programSubscribe`, `signatureSubscribe`, `slotSubscribe`) and pair with batch HTTP for backfills to stay consistent.
- Pattern: on WS notification, enqueue a lightweight HTTP batch to fetch only the changed accounts via `getMultipleAccounts` or the specific transaction via `getTransaction`.
- For high-volume logs (DEXes), filter WS with `mentions` and immediately drop to a worker that decodes and deduplicates per slot; persist last slot to resume after reconnect.
- If WS gaps are detected (slot jumps or reconnect), replay via HTTP using `getSignaturesForAddress` with `before/limit`, then fetch missing txns in a batch.
- Stream-friendly sending: when firing bursts of transactions, prefetch blockhash + priority fee once per slot, build messages in parallel, simulate in a controlled pool, then send concurrently with bounded queue and retry policy.
- Dedup + ordering: apply WS updates in slot order; if two updates share slot/tx signature, process once. Maintain `last_slot_seen` to detect regressions.
- Partial decoding: for `logsSubscribe`, parse only program logs you care about; avoid full transaction decode on the hot path. Defer heavy parsing to a worker queue.
- Backpressure: pause WS consumption when HTTP backfill queue grows beyond a threshold; resume after drain. Drop non-critical logs if falling behind instead of stalling sends.
- Health/metrics: track WS reconnect count, lag to latest slot, HTTP batch p95 latency, and rate of `BlockhashNotFound` to spot provider issues early.

## Node example: batching + WS backfill

```ts
// deps: node-fetch or undici, ws
import fetch from "node-fetch";
import WebSocket from "ws";

const RPC_URL = process.env.RPC_URL!;
const WS_URL = process.env.WS_URL!;

// Simple batch helper with keep-alive
const agent = new (require("http").Agent)({ keepAlive: true, maxSockets: 16 });
let nextId = 1;

async function rpcBatch(calls: { method: string; params?: any[] }[]) {
  const body = calls.map(c => ({ jsonrpc: "2.0", id: nextId++, ...c }));
  const res = await fetch(RPC_URL, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
    agent
  });
  const json = (await res.json()) as any[];
  const byId = new Map(json.map(r => [r.id, r]));
  return body.map(req => byId.get(req.id));
}

// WS with backfill on reconnect
function startLogs(programId: string, onLog: (slot: number, sig: string) => void) {
  let ws: WebSocket;
  let lastSlot = 0;

  const connect = () => {
    ws = new WebSocket(WS_URL);
    ws.on("open", () => {
      ws.send(
        JSON.stringify({
          jsonrpc: "2.0",
          id: 1,
          method: "logsSubscribe",
          params: [{ mentions: [programId] }, { commitment: "confirmed" }]
        })
      );
    });
    ws.on("message", data => {
      const msg = JSON.parse(String(data));
      if (!msg.params) return;
      const { slot, signature } = msg.params.result;
      lastSlot = Math.max(lastSlot, slot);
      onLog(slot, signature);
    });
    ws.on("close", async () => {
      // Backfill missed signatures after reconnect
      await backfill(lastSlot, programId, onLog);
      setTimeout(connect, 500 + Math.random() * 500);
    });
    ws.on("error", () => ws.close());
  };
  connect();
}

async function backfill(afterSlot: number, programId: string, onLog: (slot: number, sig: string) => void) {
  let before: string | null = null;
  while (true) {
    const [{ result }] = await rpcBatch([
      {
        method: "getSignaturesForAddress",
        params: [programId, { before, limit: 100, commitment: "confirmed" }]
      }
    ]);
    const sigs = (result as any[]).filter(r => r.slot > afterSlot);
    if (!sigs.length) break;
    for (const s of sigs) onLog(s.slot, s.signature);
    before = sigs[sigs.length - 1].signature;
    if ((result as any[]).length < 100) break;
  }
}
```

## Rust example: batching + WS backfill

```rust
// deps: reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
// ws: tokio-tungstenite = "0.20", serde_json = "1"
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio_tungstenite::connect_async;

struct RpcBatch {
    client: Client,
    url: String,
    next_id: u64,
}

impl RpcBatch {
    fn new(url: impl Into<String>) -> Self {
        Self { client: Client::builder().pool_max_idle_per_host(16).build().unwrap(), url: url.into(), next_id: 1 }
    }

    async fn call(&mut self, calls: Vec<(String, Value)>) -> reqwest::Result<Vec<Value>> {
        let body: Vec<Value> = calls
            .into_iter()
            .map(|(method, params)| {
                let id = self.next_id;
                self.next_id += 1;
                json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
            })
            .collect();
        let resp: Value = self.client.post(&self.url).json(&body).send().await?.json().await?;
        let arr = resp.as_array().cloned().unwrap_or_default();
        let mut by_id = HashMap::new();
        for item in &arr {
            if let Some(id) = item.get("id") {
                by_id.insert(id.clone(), item.clone());
            }
        }
        Ok(body.iter().map(|req| by_id.get(&req["id"]).cloned().unwrap_or(Value::Null)).collect())
    }
}

async fn backfill(after_slot: u64, program: &str, rpc: &mut RpcBatch) -> anyhow::Result<()> {
    let mut before: Option<String> = None;
    loop {
        let calls = vec![(
            "getSignaturesForAddress".into(),
            json!([program, {"before": before, "limit": 100, "commitment": "confirmed"}]),
        )];
        let resp = rpc.call(calls).await?;
        let result = resp[0]["result"].as_array().cloned().unwrap_or_default();
        let sigs: Vec<_> = result.into_iter().filter(|r| r["slot"].as_u64().unwrap_or(0) > after_slot).collect();
        if sigs.is_empty() {
            break;
        }
        before = sigs.last().and_then(|r| r["signature"].as_str()).map(|s| s.to_string());
        // Handle signatures (fetch txs, etc.)
    }
    Ok(())
}

async fn start_logs(program: &str, ws_url: &str, mut rpc: RpcBatch) -> anyhow::Result<()> {
    let (mut ws, _) = connect_async(ws_url).await?;
    let sub = json!({"jsonrpc":"2.0","id":1,"method":"logsSubscribe","params":[{"mentions":[program]},{"commitment":"confirmed"}]});
    ws.send(tokio_tungstenite::tungstenite::Message::Text(sub.to_string())).await?;
    let mut last_slot = 0;
    while let Some(msg) = ws.next().await {
        match msg {
            Ok(tokio_tungstenite::tungstenite::Message::Text(txt)) => {
                if let Ok(v) = serde_json::from_str::<Value>(&txt) {
                    if let Some(slot) = v.pointer("/params/result/slot").and_then(|s| s.as_u64()) {
                        last_slot = last_slot.max(slot);
                        // handle log, queue backfill of accounts/txs as needed
                    }
                }
            }
            _ => {
                // reconnect and backfill
                backfill(last_slot, program, &mut rpc).await?;
                break;
            }
        }
    }
    Ok(())
}
```

## Program-specific tips

- **SPL Token**:
  - Use `getTokenAccountsByOwner` with `mint` filter; keep a lightweight cache keyed by (owner, mint).
  - When decoding, respect account state (initialized/frozen) and close authority.
- **Address Lookup Tables**: Watch their deactivation slot; refresh after `deactivationSlot` is reached or when `lastExtendedSlot` jumps.
- **Stake and vote**: When interacting with validators, use `getInflationReward` and `getVoteAccounts` with `commitment: "finalized"` to avoid transient mismatches.

## Observability and debugging

- Turn on `logs: true` in simulation to capture program logs. Parse for `Program log:` and `Program data:` entries.
- Track `unitsConsumed` from simulation and real execution (available in enhanced RPC responses or via Jito block engine logs).
- Store per-instruction timings locally when benchmarking; RPC does not return them.
- Include a client `txn_id`/`client_id` in logs to correlate retries and submissions.

## Safety and LLM guardrails

- Never send seed phrases, private keys, or raw keypairs to RPC; only send signed transactions.
- Do not echo or store secrets in logs. Use env vars or key stores.
- Reject prompts asking to drain accounts, bypass program invariants, or craft transactions without user intent/limits.
- When uncertain about a layout, fetch the program idl/spec instead of guessing field offsets; mis-encoding leads to funds loss.

## Quick checklists

- **Send path**: getLatestBlockhash → resolve ALTs → build message → compute budget instructions → simulate (capture CU + logs) → sign locally → sendRawTransaction → poll for confirmation → handle expiry.
- **Read path**: pick minimal commitment → batch (`getMultipleAccounts`) → filter (`getProgramAccounts` with slices) → cache with slot guard → refresh on WS logs/slot jump.
- **When things break**: if compute exceeded → lower payload or raise CU limit; if `BlockhashNotFound` → refresh hash; if `AccountInUse` → retry once after slight delay; if `TransactionTooLarge` → split instructions across multiple txns.
