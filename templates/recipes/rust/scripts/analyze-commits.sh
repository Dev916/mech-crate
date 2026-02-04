#!/usr/bin/env bash
# Analyze commits since last tag to determine version bump type
# Uses conventional commits: feat, fix, docs, chore, BREAKING CHANGE
#
# Output: major, minor, or patch
#
# Usage: ./scripts/analyze-commits.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Get the last tag, or use initial commit if no tags exist
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

if [ -z "$LAST_TAG" ]; then
    # No tags exist, get all commits
    COMMITS=$(git log --pretty=format:"%s" 2>/dev/null || echo "")
else
    # Get commits since last tag
    COMMITS=$(git log "${LAST_TAG}..HEAD" --pretty=format:"%s" 2>/dev/null || echo "")
fi

if [ -z "$COMMITS" ]; then
    echo "patch"
    exit 0
fi

# Check for breaking changes (major bump)
if echo "$COMMITS" | grep -qiE "^.*!:|BREAKING[ _-]?CHANGE"; then
    echo "major"
    exit 0
fi

# Check for features (minor bump)
if echo "$COMMITS" | grep -qiE "^feat(\(.+\))?:"; then
    echo "minor"
    exit 0
fi

# Default to patch for fixes, docs, chore, etc.
echo "patch"
