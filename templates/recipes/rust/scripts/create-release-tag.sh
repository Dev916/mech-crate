#!/usr/bin/env bash
# Create a release tag with changelog
#
# Usage: ./scripts/create-release-tag.sh [version]
#        ./scripts/create-release-tag.sh  # Uses version from Cargo.toml

set -uo pipefail
# Note: removed -e to handle errors manually with better messages

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Helper function for error messages
die() {
    echo "Error: $1" >&2
    exit "${2:-1}"
}

# Helper function for verbose output
log() {
    echo "[create-release-tag] $1"
}

cd "$PROJECT_ROOT" || die "Failed to change to project root: $PROJECT_ROOT"

# Check we're in a git repository
if ! git rev-parse --git-dir >/dev/null 2>&1; then
    die "Not a git repository: $PROJECT_ROOT"
fi

# Get version - either from argument or from Cargo.toml
if [[ -n "${1:-}" ]]; then
    VERSION="$1"
    log "Using provided version: $VERSION"
else
    log "Extracting version from Cargo.toml..."
    if ! VERSION=$("$SCRIPT_DIR/get-version.sh"); then
        die "Failed to get version from Cargo.toml. Run get-version.sh manually to debug."
    fi
    log "Extracted version: $VERSION"
fi

# Validate version is not empty
if [[ -z "$VERSION" ]]; then
    die "Version is empty. Please provide a version or check Cargo.toml"
fi

TAG_NAME="v$VERSION"

log "Creating release tag: $TAG_NAME"

# Check if tag already exists
if git rev-parse "$TAG_NAME" >/dev/null 2>&1; then
    die "Tag $TAG_NAME already exists. Use 'git tag -d $TAG_NAME' to delete it first if needed."
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD -- 2>/dev/null; then
    echo "Warning: You have uncommitted changes. Consider committing first." >&2
fi

# Get the last tag (if any)
log "Finding previous tag..."
LAST_TAG=""
if git describe --tags --abbrev=0 >/dev/null 2>&1; then
    LAST_TAG=$(git describe --tags --abbrev=0)
    log "Previous tag: $LAST_TAG"
else
    log "No previous tags found, will include recent commits"
fi

# Generate changelog from commits
log "Generating changelog..."
if [[ -z "$LAST_TAG" ]]; then
    # No previous tag - get recent commits (limit to 50)
    CHANGELOG=$(git log --pretty=format:"- %s (%h)" -50 2>/dev/null) || true
else
    # Get commits since last tag
    CHANGELOG=$(git log "${LAST_TAG}..HEAD" --pretty=format:"- %s (%h)" 2>/dev/null) || true
fi

# Handle empty changelog
if [[ -z "$CHANGELOG" ]]; then
    log "Warning: No commits found for changelog"
    CHANGELOG="- No changes recorded"
fi

COMMIT_COUNT=$(echo "$CHANGELOG" | wc -l | tr -d ' ')
log "Found $COMMIT_COUNT commit(s) for changelog"

# Create tag message
TAG_MESSAGE="Release $VERSION

## Changes

$CHANGELOG

## Installation

\`\`\`toml
[dependencies]
compress-json-rs = \"$VERSION\"
\`\`\`
"

# Create annotated tag
log "Creating annotated tag..."
if ! git tag -a "$TAG_NAME" -m "$TAG_MESSAGE"; then
    die "Failed to create tag $TAG_NAME"
fi

echo ""
echo "✓ Successfully created tag: $TAG_NAME"
echo ""
echo "Tag message:"
echo "─────────────────────────────────────"
echo "$TAG_MESSAGE"
echo "─────────────────────────────────────"
echo ""
echo "Next steps:"
echo "  - Push the tag:    git push origin $TAG_NAME"
echo "  - Or push all:     git push origin --tags"
echo "  - Delete if wrong: git tag -d $TAG_NAME"
