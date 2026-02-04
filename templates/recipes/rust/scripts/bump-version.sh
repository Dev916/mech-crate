#!/usr/bin/env bash
# Bump version in Cargo.toml based on semver type
#
# Usage: ./scripts/bump-version.sh [major|minor|patch]
#        ./scripts/bump-version.sh auto  # Auto-detect from commits
#        ./scripts/bump-version.sh       # Same as auto

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Get bump type
BUMP_TYPE="${1:-auto}"

if [ "$BUMP_TYPE" = "auto" ]; then
    BUMP_TYPE=$("$SCRIPT_DIR/analyze-commits.sh")
    echo "Auto-detected bump type: $BUMP_TYPE"
fi

# Validate bump type
case "$BUMP_TYPE" in
    major|minor|patch) ;;
    *)
        echo "Error: Invalid bump type '$BUMP_TYPE'. Use major, minor, or patch."
        exit 1
        ;;
esac

# Get current version
CURRENT_VERSION=$("$SCRIPT_DIR/get-version.sh")
echo "Current version: $CURRENT_VERSION"

# Parse version components
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

# Bump version
case "$BUMP_TYPE" in
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    patch)
        PATCH=$((PATCH + 1))
        ;;
esac

NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
echo "New version: $NEW_VERSION"

# Update Cargo.toml
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
else
    # Linux
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
fi

# Update Cargo.lock
cargo check --quiet 2>/dev/null || true

# Also update doc url if present
if grep -q "html_root_url" src/lib.rs; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s|html_root_url = \"https://docs.rs/compress-json-rs/[^\"]*\"|html_root_url = \"https://docs.rs/compress-json-rs/$NEW_VERSION\"|" src/lib.rs
    else
        sed -i "s|html_root_url = \"https://docs.rs/compress-json-rs/[^\"]*\"|html_root_url = \"https://docs.rs/compress-json-rs/$NEW_VERSION\"|" src/lib.rs
    fi
fi

echo "$NEW_VERSION"
