#!/usr/bin/env bash
# Generate changelog from git commits
#
# Usage: ./scripts/changelog.sh [since-tag]
#        ./scripts/changelog.sh          # Since last tag
#        ./scripts/changelog.sh v0.1.0   # Since specific tag

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

SINCE_TAG="${1:-}"

if [ -z "$SINCE_TAG" ]; then
    SINCE_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
fi

VERSION=$("$SCRIPT_DIR/get-version.sh")
DATE=$(date +%Y-%m-%d)

echo "# Changelog"
echo ""
echo "## [$VERSION] - $DATE"
echo ""

# Get commits
if [ -z "$SINCE_TAG" ]; then
    COMMITS=$(git log --pretty=format:"%s|%h|%an" 2>/dev/null)
else
    COMMITS=$(git log "${SINCE_TAG}..HEAD" --pretty=format:"%s|%h|%an" 2>/dev/null)
fi

if [ -z "$COMMITS" ]; then
    echo "No changes since last release."
    exit 0
fi

# Categorize commits
declare -a BREAKING=()
declare -a FEATURES=()
declare -a FIXES=()
declare -a DOCS=()
declare -a OTHER=()

while IFS='|' read -r MSG HASH AUTHOR; do
    if echo "$MSG" | grep -qiE "^.*!:|BREAKING[ _-]?CHANGE"; then
        BREAKING+=("- $MSG ($HASH)")
    elif echo "$MSG" | grep -qiE "^feat(\(.+\))?:"; then
        FEATURES+=("- ${MSG#feat*: } ($HASH)")
    elif echo "$MSG" | grep -qiE "^fix(\(.+\))?:"; then
        FIXES+=("- ${MSG#fix*: } ($HASH)")
    elif echo "$MSG" | grep -qiE "^docs(\(.+\))?:"; then
        DOCS+=("- ${MSG#docs*: } ($HASH)")
    else
        OTHER+=("- $MSG ($HASH)")
    fi
done <<< "$COMMITS"

# Print categorized changelog
if [ ${#BREAKING[@]} -gt 0 ]; then
    echo "### ⚠️ Breaking Changes"
    echo ""
    printf '%s\n' "${BREAKING[@]}"
    echo ""
fi

if [ ${#FEATURES[@]} -gt 0 ]; then
    echo "### ✨ Features"
    echo ""
    printf '%s\n' "${FEATURES[@]}"
    echo ""
fi

if [ ${#FIXES[@]} -gt 0 ]; then
    echo "### 🐛 Bug Fixes"
    echo ""
    printf '%s\n' "${FIXES[@]}"
    echo ""
fi

if [ ${#DOCS[@]} -gt 0 ]; then
    echo "### 📚 Documentation"
    echo ""
    printf '%s\n' "${DOCS[@]}"
    echo ""
fi

if [ ${#OTHER[@]} -gt 0 ]; then
    echo "### 🔧 Other Changes"
    echo ""
    printf '%s\n' "${OTHER[@]}"
    echo ""
fi
