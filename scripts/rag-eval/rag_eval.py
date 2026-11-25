#!/usr/bin/env python3
"""Retrieval evaluation harness for the mx techniques corpus (trial tooling).

Drives a locally built `mx-mcp` over stdio (initialize, then tools/call
rag_context) and scores the answers. Stdlib only.

  run        ask every question, write one JSON line per answer
  score      metrics for one run (optionally with a retrieval-score abstain rule)
  jev-sweep  replay a MX_RAG_JEV_TRACE file at several thresholds, offline
  stub       a stand-in /systemone server for plumbing checks. NOT a model:
             its answers are word overlap, so its numbers measure nothing.

The server under test takes its configuration from the environment
(MX_RAG_DATABASE_URL, MX_RAG_RERANK, MX_RAG_JEV_*), so the same `run` command
measures the baseline and the Jev stage.
"""

import argparse
import hashlib
import json
import re
import statistics
import subprocess
import sys
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

DOC_RE = re.compile(r"^## .+ \((\S+\.md)\)$", re.M)
SCORE_RE = re.compile(r"^### .* \(score (-?\d+\.\d+)\)$", re.M)
PRICE_PER_MTOK = 0.042  # USD per million input tokens (vendor list price)


class Mcp:
    def __init__(self, binary):
        self.p = subprocess.Popen(
            [binary],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
        )
        self.n = 0
        self.call(
            "initialize",
            {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "rag-eval", "version": "0"},
            },
        )

    def call(self, method, params):
        self.n += 1
        req = {"jsonrpc": "2.0", "id": self.n, "method": method, "params": params}
        self.p.stdin.write(json.dumps(req) + "\n")
        self.p.stdin.flush()
        while True:
            line = self.p.stdout.readline()
            if not line:
                raise RuntimeError("mx-mcp closed stdout")
            msg = json.loads(line)
            if msg.get("id") == self.n:
                if "error" in msg:
                    raise RuntimeError(msg["error"])
                return msg["result"]

    def rag_context(self, query, limit):
        res = self.call(
            "tools/call",
            {"name": "rag_context", "arguments": {"working_on": query, "limit": limit}},
        )
        return "".join(c.get("text", "") for c in res.get("content", []))

    def close(self):
        self.p.stdin.close()
        self.p.wait(timeout=10)


def cmd_run(a):
    questions = json.load(open(a.questions))
    mcp = Mcp(a.bin)
    mcp.rag_context("warm up the connection", a.limit)
    with open(a.out, "w") as out:
        for q in questions:
            t0 = time.perf_counter()
            text = mcp.rag_context(q["query"], a.limit)
            ms = (time.perf_counter() - t0) * 1000
            if "Corpus search failed" in text or "corpus is offline" in text:
                raise RuntimeError(f"search failed for {q['id']}: {text[:200]}")
            if "lexical-only search" in text:
                raise RuntimeError("embeddings unavailable: run is not comparable")
            row = {
                **q,
                "ms": round(ms, 1),
                "docs": DOC_RE.findall(text),
                "scores": [float(s) for s in SCORE_RE.findall(text)],
                "sha": hashlib.sha256(text.encode()).hexdigest()[:16],
            }
            out.write(json.dumps(row) + "\n")
            print(f"{q['id']:24s} {ms:7.0f}ms {row['docs']}", file=sys.stderr)
    mcp.close()


def pct(xs, p):
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(round(p / 100 * (len(xs) - 1))))]


def metrics(rows):
    """rows: dicts with expect, docs (after any abstain rule), group."""
    pos = [r for r in rows if r["expect"]]
    neg = [r for r in rows if not r["expect"]]
    hit = lambda r: any(d in r["expect"] for d in r["docs"])
    answered_neg = sum(1 for r in neg if r["docs"])
    correct_abst = sum(1 for r in neg if not r["docs"]) + sum(1 for r in pos if r["docs"])
    out = {
        "hit@k": f"{sum(map(hit, pos))}/{len(pos)}",
        "fp_on_negatives": f"{answered_neg}/{len(neg)}",
        "abstention_accuracy": f"{correct_abst}/{len(rows)}",
        "wrongly_abstained_positives": sum(1 for r in pos if not r["docs"]),
    }
    for g in sorted({r["group"] for r in pos}):
        gp = [r for r in pos if r["group"] == g]
        out[f"hit@k[{g}]"] = f"{sum(map(hit, gp))}/{len(gp)}"
    for g in sorted({r["group"] for r in neg}):
        gn = [r for r in neg if r["group"] == g]
        out[f"fp[{g}]"] = f"{sum(1 for r in gn if r['docs'])}/{len(gn)}"
    return out


def load(path):
    return [json.loads(line) for line in open(path)]


def with_score_rule(rows, floor):
    """Free competitor: abstain when the top retrieval score is below `floor`."""
    return [
        {**r, "docs": r["docs"] if r["scores"] and max(r["scores"]) >= floor else []}
        for r in rows
    ]


def cmd_score(a):
    rows = load(a.run)
    ms = [r["ms"] for r in rows]
    m = metrics(rows)
    m["latency_p50_ms"] = round(statistics.median(ms))
    m["latency_p95_ms"] = round(pct(ms, 95))
    print(json.dumps(m, indent=1))
    if a.misses:
        for r in rows:
            if r["expect"] and not any(d in r["expect"] for d in r["docs"]):
                print(f"MISS {r['id']}: {r['query']!r} -> {r['docs']}")
    if a.sweep_score:
        print("\nretrieval-score abstain rule (abstain when top score < floor):")
        for floor in [x / 100 for x in range(30, 71, 2)]:
            mm = metrics(with_score_rule(rows, floor))
            print(
                f"  floor {floor:.2f}  hit {mm['hit@k']:6s} fp {mm['fp_on_negatives']:6s}"
                f" abst_acc {mm['abstention_accuracy']:6s}"
                f" lost_pos {mm['wrongly_abstained_positives']}"
            )


def cmd_jev_sweep(a):
    """Replay a trace taken with MX_RAG_JEV_THRESHOLD=0 (nothing dropped)."""
    questions = {q["query"]: q for q in json.load(open(a.questions))}
    traces = [t for t in load(a.trace) if t["query"] in questions]
    tokens = [t["input_tokens"] for t in traces]
    stage = [t["stage_ms"] for t in traces]
    judged = sum(1 for t in traces for c in t["chunks"] if c["status"] == "judged")
    errors = sum(1 for t in traces for c in t["chunks"] if c["status"] == "error")
    private = sum(1 for t in traces for c in t["chunks"] if c["status"] == "private")
    print(f"queries {len(traces)}  judged {judged}  private(unsent) {private}  errors {errors}")
    print(
        f"stage latency p50 {statistics.median(stage):.0f}ms p95 {pct(stage, 95):.0f}ms;"
        f" input tokens/query mean {statistics.mean(tokens):.0f};"
        f" cost/query ${statistics.mean(tokens) * PRICE_PER_MTOK / 1e6:.6f}"
    )
    for thr in [x / 100 for x in range(5, 96, 5)]:
        rows = []
        for t in traces:
            kept = [
                c["path"]
                for c in t["chunks"]
                if c["probability"] is None or c["probability"] >= thr
            ][: a.limit]
            rows.append({**questions[t["query"]], "docs": kept})
        mm = metrics(rows)
        print(
            f"  thr {thr:.2f}  hit {mm['hit@k']:6s} fp {mm['fp_on_negatives']:6s}"
            f" abst_acc {mm['abstention_accuracy']:6s}"
            f" lost_pos {mm['wrongly_abstained_positives']}"
        )


class Stub(BaseHTTPRequestHandler):
    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        state = body["state"]
        words = {w for w in re.findall(r"[a-z0-9]+", state["query"].lower()) if len(w) > 3}
        text = state["passage"]["text"].lower()
        p = sum(1 for w in words if w in text) / max(1, len(words))
        answers = {k: {"type": "noul", "noul": round(p, 3)} for k in body["questions"]}
        usage = {"input_tokens": len(json.dumps(state)) // 4, "output_tokens": 1}
        out = json.dumps({"model": "stub", "answers": answers, "usage": usage}).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(out)))
        self.end_headers()
        self.wfile.write(out)

    def log_message(self, *args):
        pass


def cmd_stub(a):
    ThreadingHTTPServer(("127.0.0.1", a.port), Stub).serve_forever()


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = ap.add_subparsers(required=True)
    r = sub.add_parser("run")
    r.add_argument("--bin", default="target/release/mx-mcp")
    r.add_argument("--questions", default="scripts/rag-eval/questions.json")
    r.add_argument("--limit", type=int, default=3)
    r.add_argument("--out", required=True)
    r.set_defaults(fn=cmd_run)
    s = sub.add_parser("score")
    s.add_argument("run")
    s.add_argument("--misses", action="store_true")
    s.add_argument("--sweep-score", action="store_true")
    s.set_defaults(fn=cmd_score)
    j = sub.add_parser("jev-sweep")
    j.add_argument("trace")
    j.add_argument("--questions", default="scripts/rag-eval/questions.json")
    j.add_argument("--limit", type=int, default=3)
    j.set_defaults(fn=cmd_jev_sweep)
    t = sub.add_parser("stub")
    t.add_argument("--port", type=int, default=18099)
    t.set_defaults(fn=cmd_stub)
    a = ap.parse_args()
    a.fn(a)


if __name__ == "__main__":
    main()
