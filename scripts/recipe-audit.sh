#!/usr/bin/env bash
# Audit every shipped recipe lockfile for known security advisories.
#
# A recipe's composer.lock is copied into every scaffold, so a lock that pins a
# release with a published advisory ships that advisory to every new project
# (bd:mech-crate-hca: the laravel lock sat on framework v12.47.0 with 70
# advisories across 20 packages). This runs `composer audit --locked` against
# each recipe's composer.json + composer.lock in a scratch directory, so it
# needs composer and network access to packagist, and nothing else.
#
# Exit 0: every lock is clean. Exit 1: at least one lock has advisories.
# Exit 2: composer is not installed (the audit could not run).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RECIPES="${RECIPE_AUDIT_ROOT:-$ROOT/templates/recipes}"

if ! command -v composer >/dev/null 2>&1; then
  echo "recipe-audit: composer is not installed; cannot audit recipe locks" >&2
  exit 2
fi

status=0
found=0
while IFS= read -r lock; do
  found=$((found + 1))
  dir="$(dirname "$lock")"
  rel="${dir#"$ROOT"/}"
  scratch="$(mktemp -d)"
  cp "$dir/composer.json" "$lock" "$scratch/"
  if out="$(cd "$scratch" && composer audit --locked --no-interaction --format=plain 2>&1)"; then
    echo "✓ $rel/composer.lock: no known advisories"
  else
    status=1
    echo "✗ $rel/composer.lock has known advisories:" >&2
    echo "$out" | grep -E '^(Found|Package|Severity|Advisory ID|Title)' | sed 's/^/    /' >&2
    echo "  Regenerate it under the recipe's PHP version: composer update -W --no-install --no-scripts" >&2
  fi
  rm -rf "$scratch"
done < <(find "$RECIPES" -name composer.lock -not -path '*/vendor/*' -not -path '*/node_modules/*' | sort)

if [ "$found" -eq 0 ]; then
  echo "recipe-audit: no composer.lock found under $RECIPES" >&2
  exit 1
fi
exit "$status"
