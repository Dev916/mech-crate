#!/usr/bin/env python3
"""Score the relevance stage against passage-level judge labels (trial tooling).

Inputs: a key file mapping passage id -> {set, query, rank, p, score} (built
from MX_RAG_JEV_TRACE lines taken at threshold 0) and judge label files mapping
passage id -> 0/1/2, where 2 means "directly helps answer the query".

A query counts as answerable when at least one of its retrieved candidates is
labelled 2. That is answerable *from what retrieval surfaced*, which is the
only thing a filter behind retrieval can be right or wrong about.
"""

import collections
import glob
import json
import sys

LIMIT = 3


def auc(pos, neg):
    """Probability a random relevant passage outranks a random non-relevant one."""
    if not pos or not neg:
        return float("nan")
    wins = sum((p > n) + 0.5 * (p == n) for p in pos for n in neg)
    return wins / (len(pos) * len(neg))


def query_metrics(queries, keep):
    """keep(chunk) -> bool. Returns counts over queries for a top-LIMIT answer."""
    m = collections.Counter()
    for chunks in queries:
        answerable = any(c["label"] == 2 for c in chunks)
        shown = [c for c in chunks if keep(c, chunks)][:LIMIT]
        m["answerable" if answerable else "unanswerable"] += 1
        m["shown_chunks"] += len(shown)
        m["shown_relevant"] += sum(c["label"] == 2 for c in shown)
        if answerable:
            if any(c["label"] == 2 for c in shown):
                m["useful"] += 1
            elif not shown:
                m["wrong_abstain"] += 1
            else:
                m["junk_only"] += 1
        else:
            m["correct_abstain" if not shown else "false_positive"] += 1
    return m


def row(name, m):
    prec = m["shown_relevant"] / m["shown_chunks"] if m["shown_chunks"] else float("nan")
    return (
        f"  {name:28s} useful {m['useful']:2d}/{m['answerable']:2d}"
        f"  wrong-abstain {m['wrong_abstain']:2d}  junk-only {m['junk_only']:2d}"
        f"  | FP {m['false_positive']:2d}/{m['unanswerable']:2d}"
        f"  | precision of shown chunks {prec:.2f} ({m['shown_relevant']}/{m['shown_chunks']})"
    )


def main():
    key = json.load(open(sys.argv[1]))
    labels = {}
    for path in glob.glob(sys.argv[2]):
        labels.update(json.load(open(path)))
    missing = [k for k in key if k not in labels]
    if missing:
        sys.exit(f"{len(missing)} passages unlabelled, e.g. {missing[:3]}")
    for name in sorted({v["set"] for v in key.values()}):
        byq = collections.defaultdict(list)
        for pid, v in key.items():
            if v["set"] == name:
                byq[v["query"]].append({**v, "label": int(labels[pid])})
        queries = [sorted(cs, key=lambda c: c["rank"]) for cs in byq.values()]
        chunks = [c for cs in queries for c in cs]
        rel = [c for c in chunks if c["label"] == 2]
        non = [c for c in chunks if c["label"] < 2]
        counts = collections.Counter(c["label"] for c in chunks)
        print(f"\n===== {name}: {len(queries)} queries, {len(chunks)} passages =====")
        print(f"judge labels: 2={counts[2]} 1={counts[1]} 0={counts[0]}")
        print(
            f"passage AUC (label 2 vs rest): jev {auc([c['p'] for c in rel], [c['p'] for c in non]):.3f}"
            f"   retrieval score {auc([c['score'] for c in rel], [c['score'] for c in non]):.3f}"
        )
        for lab in (2, 1, 0):
            ps = sorted(c["p"] for c in chunks if c["label"] == lab)
            if ps:
                print(f"  label {lab}: median jev p {ps[len(ps) // 2]:.2f}  (n={len(ps)})")
        print("calibration (jev p bin -> share labelled 2):")
        for lo in (0.0, 0.2, 0.4, 0.6, 0.8):
            b = [c for c in chunks if lo <= c["p"] < lo + 0.2 or (lo == 0.8 and c["p"] == 1.0)]
            if b:
                print(f"  {lo:.1f}-{lo + 0.2:.1f}: {sum(c['label'] == 2 for c in b) / len(b):.2f} (n={len(b)})")
        print("query level, top 3:")
        print(row("baseline (no filter)", query_metrics(queries, lambda c, cs: True)))
        for floor in (0.36, 0.42, 0.46):
            print(
                row(
                    f"score floor {floor:.2f}",
                    query_metrics(queries, lambda c, cs, f=floor: cs[0]["score"] >= f),
                )
            )
        for thr in (0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7):
            print(row(f"jev threshold {thr:.2f}", query_metrics(queries, lambda c, cs, t=thr: c["p"] >= t)))


if __name__ == "__main__":
    main()
