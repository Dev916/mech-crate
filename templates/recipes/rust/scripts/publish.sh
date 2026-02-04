#!/usr/bin/env bash
# Publish to crates.io
#
# Usage: ./scripts/publish.sh [--dry-run]
#
# Prerequisites:
#   - cargo login (one-time setup with your crates.io token)
#   - Clean git working directory
#   - All tests passing

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

DRY_RUN=""
if [ "${1:-}" = "--dry-run" ]; then
    DRY_RUN="--dry-run"
    echo "=== DRY RUN MODE ==="
fi

VERSION=$("$SCRIPT_DIR/get-version.sh")
echo "Publishing version: $VERSION"

# Pre-flight checks
echo ""
echo "Running pre-flight checks..."

# Check for uncommitted changes
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "Error: You have uncommitted changes. Please commit or stash them first."
    exit 1
fi

# Run tests
echo "Running tests..."
cargo test --quiet

# Run clippy
echo "Running clippy..."
cargo clippy --quiet -- -D warnings 2>/dev/null || {
    echo "Warning: Clippy found issues (continuing anyway)"
}

# Check formatting
echo "Checking formatting..."
cargo fmt --check 2>/dev/null || {
    echo "Warning: Code is not formatted. Run 'cargo fmt' to fix."
}

# Build docs to ensure they compile
echo "Building documentation..."
cargo doc --no-deps --quiet

# Verify package
echo "Verifying package..."
cargo package --list

echo ""
echo "=== Publishing to crates.io ==="

if [ -n "$DRY_RUN" ]; then
    cargo publish --dry-run
    echo ""
    echo "Dry run complete. Run without --dry-run to actually publish."
else
    cargo publish
    echo ""
    echo "✅ Successfully published compress-json-rs v$VERSION to crates.io!"
    echo ""
    echo "View at: https://crates.io/crates/compress-json-rs"
    echo "Docs at: https://docs.rs/compress-json-rs"
fi
