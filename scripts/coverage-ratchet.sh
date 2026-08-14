#!/usr/bin/env bash
# Coverage ratchet — fail if workspace LINE coverage drops more than EPSILON
# below the floor recorded in .coverage-floor.
#
# Usage:
#   scripts/coverage-ratchet.sh           # measure, compare against the floor
#   scripts/coverage-ratchet.sh --bump    # rewrite the floor to the current value
#   make coverage  /  make coverage BUMP=1
#
# TEST DATABASE — policy: THIS SCRIPT supplies it, the caller does not have to.
#   MX_RAG_TEST_DATABASE_URL defaults to postgres://postgres@localhost:55433/mx_rag
#   (local container `mx-rag-test`; in CI the pgvector service container exports
#   its own value, which wins). The DB-backed corpus tests skip themselves when
#   that var is unset, so measuring without a database records a floor that is
#   NOT comparable to CI's. Rather than silently reporting a lower number, the
#   script refuses to run when the database is unreachable:
#       docker start mx-rag-test        # or: make test-int
#
# KNOWN-BROKEN LANE — deliberately NOT measured. Plain `cargo llvm-cov nextest`
# skips `#[ignore = "bd:…"]` tests (no --run-ignored is passed), so the ratchet
# only ever sees the gating suite.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FLOOR_FILE="$REPO_ROOT/.coverage-floor"
EPSILON=0.25

mode="${1:-}"
case "$mode" in
  "" | --bump) ;;
  *)
    echo "usage: $(basename "$0") [--bump]" >&2
    exit 2
    ;;
esac

export MX_RAG_TEST_DATABASE_URL="${MX_RAG_TEST_DATABASE_URL:-postgres://postgres@localhost:55433/mx_rag}"

# Reachability probe: host:port out of the URL, then a bare TCP connect.
hostport="${MX_RAG_TEST_DATABASE_URL#*@}"
hostport="${hostport%%/*}"
db_host="${hostport%%:*}"
db_port="${hostport##*:}"
[ "$db_port" = "$db_host" ] && db_port=5432
if ! (exec 3<>"/dev/tcp/${db_host}/${db_port}") 2>/dev/null; then
  echo "test database unreachable at ${db_host}:${db_port}" >&2
  echo "  MX_RAG_TEST_DATABASE_URL=${MX_RAG_TEST_DATABASE_URL}" >&2
  echo "  start it with: docker start mx-rag-test   (or: make test-int)" >&2
  echo "  refusing to record/compare a floor measured without the DB tests." >&2
  exit 2
fi

cd "$REPO_ROOT"
if ! summary="$(cargo llvm-cov nextest --workspace --summary-only 2>&1)"; then
  printf '%s\n' "$summary" | tail -40 >&2
  echo "coverage run failed (see above)" >&2
  exit 2
fi

# TOTAL row (llvm-cov 0.8.x):
#   Regions Missed Cover% | Functions Missed Executed% | Lines Missed Cover% | Branches Missed Cover%
# Line coverage is the THIRD percentage on the row. Counting '%'-suffixed fields
# instead of using a fixed column keeps this stable when the branch columns
# appear/disappear (they render as "-" when no branch data is emitted).
current="$(printf '%s\n' "$summary" | awk '
  /^TOTAL/ {
    n = 0
    for (i = 1; i <= NF; i++) {
      if ($i ~ /%$/ && ++n == 3) { sub(/%$/, "", $i); printf "%.1f", $i + 0; exit }
    }
  }')"
[ -n "$current" ] || { echo "could not parse coverage from the llvm-cov summary" >&2; exit 2; }

if [ "$mode" = "--bump" ]; then
  printf '%s\n' "$current" > "$FLOOR_FILE"
  echo "coverage: ${current}% (floor: ${current}%)"
  echo "floor bumped to ${current}%"
  exit 0
fi

[ -f "$FLOOR_FILE" ] || { echo "missing $FLOOR_FILE — record it with: make coverage BUMP=1" >&2; exit 2; }
floor="$(tr -d '[:space:]%' < "$FLOOR_FILE")"
[ -n "$floor" ] || { echo "$FLOOR_FILE is empty — record it with: make coverage BUMP=1" >&2; exit 2; }

echo "coverage: ${current}% (floor: ${floor}%)"
if [ "$(echo "$current >= $floor - $EPSILON" | bc -l)" != "1" ]; then
  echo "COVERAGE DROP: ${current}% < floor ${floor}% - ${EPSILON}" >&2
  exit 1
fi
