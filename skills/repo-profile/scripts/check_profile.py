#!/usr/bin/env python3
"""Mechanical acceptance gate for a corpus repo profile.

Usage:
    check_profile.py <path/to/docs/development/repos/slug.md> [--max-lines N]
                     [--target /path/to/profiled/repo] [--no-ingest]

Checks (spec: docs/superpowers/specs/2026-09-01-repo-profiles-corpus-design.md §5):
  frontmatter keys · category/publish/title format · the eleven ## sections in
  order · line-count bounds · secret patterns · HEAD sha recorded in Identity ·
  target repo untouched · `mx rag ingest --dry-run` reports 0 warnings.

Exit 0 = pass. Any failure prints ✗ lines and exits 1.
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path

REQUIRED_KEYS = [
    "title", "category", "languages", "complexity", "use_cases", "summary",
    "provenance", "researched", "publish", "repo", "local_path", "status",
    "visibility", "owner", "sources",
]

SECTIONS = [
    "Identity", "What It Does", "Capabilities", "Architecture",
    "Repository Layout", "How It Was Built", "Relationships",
    "Notable Techniques", "State, Gaps and Drift", "Quick Reference", "Sources",
]

SECRET_PATTERNS = [
    (r"postgres(?:ql)?://[^l\s]", "postgres DSN"),
    (r"sk-[A-Za-z0-9_-]{8,}", "sk- API key"),
    (r"AKIA[0-9A-Z]{12,}", "AWS access key"),
    (r"[Bb]earer [A-Za-z0-9._-]{16,}", "bearer token"),
    (r"xox[bcdeops]-[A-Za-z0-9-]{8,}", "Slack token"),
    (r"github_pat_[A-Za-z0-9_]{20,}", "GitHub PAT"),
    (r"gh[pousr]_[A-Za-z0-9]{20,}", "GitHub token"),
]

MIN_LINES = 120


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("profile")
    ap.add_argument("--max-lines", type=int, default=500)
    ap.add_argument("--target", help="path of the profiled repo; asserted untouched")
    ap.add_argument("--no-ingest", action="store_true", help="skip mx rag ingest --dry-run")
    args = ap.parse_args()

    path = Path(args.profile).resolve()
    failures: list[str] = []
    passes: list[str] = []

    def check(cond: bool, label: str) -> None:
        (passes if cond else failures).append(label)

    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()

    # Frontmatter block
    fm = ""
    if lines and lines[0] == "---":
        try:
            end = lines[1:].index("---") + 1
            fm = "\n".join(lines[1:end])
        except ValueError:
            pass
    check(bool(fm), "frontmatter block present")
    for key in REQUIRED_KEYS:
        check(re.search(rf"^{key}:", fm, re.M) is not None, f"frontmatter key: {key}")
    check(re.search(r"^category:\s*repos\s*$", fm, re.M) is not None, "category is exactly 'repos'")
    check(re.search(r"^publish:\s*false\s*$", fm, re.M) is not None, "publish: false")
    check(re.search(r"^title:.*\(Repo Profile\)", fm, re.M) is not None, "title ends with (Repo Profile)")

    # Sections, in order
    got = [m.group(1).strip() for m in re.finditer(r"^## (.+)$", text, re.M)]
    check(got == SECTIONS, f"sections exact order ({'ok' if got == SECTIONS else 'got: ' + ' | '.join(got)})")

    # Size
    n = len(lines)
    check(MIN_LINES <= n <= args.max_lines, f"line count {n} within [{MIN_LINES}, {args.max_lines}]")

    # Secrets
    for pat, label in SECRET_PATTERNS:
        hits = [i + 1 for i, l in enumerate(lines) if re.search(pat, l)]
        check(not hits, f"no {label}" + (f" (lines {hits})" if hits else ""))

    # No template markers survive
    leftover = [i + 1 for i, l in enumerate(lines) if "{{" in l or "tpl:" in l]
    check(not leftover, "no template markers ({{…}} / tpl:) left" + (f" (lines {leftover[:6]})" if leftover else ""))

    # Identity carries a short sha (staleness anchor)
    ident = text.split("## Identity", 1)[-1].split("## ", 1)[0]
    check(re.search(r"\b[0-9a-f]{7,40}\b", ident) is not None, "Identity records a HEAD sha")

    # Synthesis discipline: heading exists only as ### (never ##) — optional presence
    check(re.search(r"^## Synthesis \(inferred\)", text, re.M) is None,
          "Synthesis (inferred) is ### level, not ##")

    # Target repo untouched
    if args.target:
        r = subprocess.run(["git", "-C", args.target, "status", "--porcelain"],
                           capture_output=True, text=True)
        check(r.returncode == 0 and r.stdout.strip() == "",
              f"target repo untouched ({args.target})")

    # Ingest dry run over the docs tree this profile sits in
    if not args.no_ingest:
        docs_dir = path.parent.parent  # …/docs/development
        repo_root = docs_dir.parent.parent
        mx = repo_root / "bin" / "mx"
        cmd = [str(mx) if mx.exists() else "mx", "rag", "ingest", "--path", str(docs_dir), "--dry-run"]
        r = subprocess.run(cmd, capture_output=True, text=True)
        m = re.search(r"Dry run: (\d+) docs, (\d+) chunks, (\d+) warnings", r.stdout + r.stderr)
        check(m is not None and m.group(3) == "0",
              f"mx rag ingest --dry-run 0 warnings ({m.group(0) if m else (r.stdout + r.stderr).strip()[:120]})")

    for p in passes:
        print(f"  ✓ {p}")
    for f in failures:
        print(f"  ✗ {f}")
    print(f"{'PASS' if not failures else 'FAIL'} — {len(passes)} ok, {len(failures)} failing: {path.name}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
