#!/usr/bin/env bash
# Full release process: bump version, commit, tag, push, publish
#
# Usage: ./scripts/release.sh [major|minor|patch|auto]
#        ./scripts/release.sh              # Auto-detect bump type
#        ./scripts/release.sh --dry-run    # Show what would happen
#
# This script will:
#   1. Analyze commits to determine version bump (if auto)
#   2. Bump version in Cargo.toml
#   3. Commit the version change
#   4. Create annotated release tag
#   5. Push commits and tag to origin
#   6. Publish to crates.io

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Parse arguments
DRY_RUN=false
BUMP_TYPE="auto"
SKIP_PUBLISH=false
NO_PUSH=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --skip-publish)
            SKIP_PUBLISH=true
            shift
            ;;
        --no-push)
            NO_PUSH=true
            shift
            ;;
        major|minor|patch|auto)
            BUMP_TYPE="$1"
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [major|minor|patch|auto] [--dry-run] [--skip-publish] [--no-push]"
            echo ""
            echo "Options:"
            echo "  major         Bump major version (x.0.0)"
            echo "  minor         Bump minor version (0.x.0)"
            echo "  patch         Bump patch version (0.0.x)"
            echo "  auto          Auto-detect from commits (default)"
            echo "  --dry-run     Show what would happen without making changes"
            echo "  --skip-publish Skip publishing to crates.io"
            echo "  --no-push     Skip pushing to remote"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    Release Process                          ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

if $DRY_RUN; then
    echo "🔍 DRY RUN MODE - No changes will be made"
    echo ""
fi

# Step 0: Pre-flight checks
echo "📋 Step 0: Pre-flight checks"

# Check for uncommitted changes
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "   ⚠️  You have uncommitted changes"
    if ! $DRY_RUN; then
        echo "   Please commit or stash them first."
        exit 1
    fi
else
    echo "   ✅ Working directory is clean"
fi

# Run tests
echo "   Running tests..."
if cargo test --quiet 2>/dev/null; then
    echo "   ✅ All tests pass"
else
    echo "   ❌ Tests failed"
    exit 1
fi

# Step 1: Determine version bump
echo ""
echo "📊 Step 1: Analyzing commits"

if [ "$BUMP_TYPE" = "auto" ]; then
    BUMP_TYPE=$("$SCRIPT_DIR/analyze-commits.sh")
fi
echo "   Bump type: $BUMP_TYPE"

# Step 2: Get current and new version
CURRENT_VERSION=$("$SCRIPT_DIR/get-version.sh")
echo "   Current version: $CURRENT_VERSION"

# Calculate new version
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"
case "$BUMP_TYPE" in
    major) MAJOR=$((MAJOR + 1)); MINOR=0; PATCH=0 ;;
    minor) MINOR=$((MINOR + 1)); PATCH=0 ;;
    patch) PATCH=$((PATCH + 1)) ;;
esac
NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
TAG_NAME="v$NEW_VERSION"

echo "   New version: $NEW_VERSION"
echo "   Tag name: $TAG_NAME"

# Check if tag exists
if git rev-parse "$TAG_NAME" >/dev/null 2>&1; then
    echo "   ❌ Error: Tag $TAG_NAME already exists"
    exit 1
fi

if $DRY_RUN; then
    echo ""
    echo "🔍 Dry run summary:"
    echo "   Would bump version from $CURRENT_VERSION to $NEW_VERSION"
    echo "   Would create tag $TAG_NAME"
    if ! $NO_PUSH; then
        echo "   Would push to origin"
    fi
    if ! $SKIP_PUBLISH; then
        echo "   Would publish to crates.io"
    fi
    echo ""
    echo "Run without --dry-run to execute."
    exit 0
fi

# Step 3: Bump version
echo ""
echo "📝 Step 2: Bumping version"
"$SCRIPT_DIR/bump-version.sh" "$BUMP_TYPE" > /dev/null
echo "   ✅ Updated Cargo.toml to $NEW_VERSION"

# Step 4: Commit version change
echo ""
echo "💾 Step 3: Committing version change"
git add Cargo.toml Cargo.lock src/lib.rs 2>/dev/null || git add Cargo.toml Cargo.lock
git commit -m "chore: release v$NEW_VERSION" --quiet
echo "   ✅ Created commit"

# Step 5: Create tag
echo ""
echo "🏷️  Step 4: Creating release tag"
"$SCRIPT_DIR/create-release-tag.sh" "$NEW_VERSION" > /dev/null
echo "   ✅ Created tag $TAG_NAME"

# Step 6: Push
if ! $NO_PUSH; then
    echo ""
    echo "🚀 Step 5: Pushing to origin"
    git push origin HEAD --quiet
    git push origin "$TAG_NAME" --quiet
    echo "   ✅ Pushed commits and tag"
fi

# Step 7: Publish
if ! $SKIP_PUBLISH; then
    echo ""
    echo "📦 Step 6: Publishing to crates.io"
    "$SCRIPT_DIR/publish.sh"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    Release Complete! 🎉                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Version: $NEW_VERSION"
echo "Tag: $TAG_NAME"
echo ""
echo "Links:"
echo "  📦 https://crates.io/crates/compress-json-rs"
echo "  📚 https://docs.rs/compress-json-rs"
