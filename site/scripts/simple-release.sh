#!/bin/bash

# Simple release script that doesn't require conventional commits
# Just bumps version, creates tag, and pushes
#
# Usage:
#   ./simple-release.sh [patch|minor|major] [app-name]
#   ./simple-release.sh patch myapp
#   ./simple-release.sh minor myapp

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get the release type and app name
RELEASE_TYPE=${1:-patch}
APP_NAME=${2:-}

# Validate release type
if [[ ! "$RELEASE_TYPE" =~ ^(patch|minor|major)$ ]]; then
  echo -e "${RED}Error: Invalid release type '$RELEASE_TYPE'${NC}"
  echo "Usage: $0 [patch|minor|major] [app-name]"
  exit 1
fi

# Validate app name
if [ -z "$APP_NAME" ]; then
  echo -e "${RED}Error: App name is required${NC}"
  echo "Usage: $0 [patch|minor|major] [app-name]"
  echo ""
  echo "Available apps:"
  for dir in apps/*/; do
    if [ -f "$dir/package.json" ]; then
      app=$(basename "$dir")
      version=$(node -p "require('./$dir/package.json').version" 2>/dev/null || echo "unknown")
      echo "  - $app (v$version)"
    fi
  done
  exit 1
fi

APP_PATH="apps/$APP_NAME"

# Validate app exists
if [ ! -d "$APP_PATH" ]; then
  echo -e "${RED}Error: App directory not found: $APP_PATH${NC}"
  exit 1
fi

if [ ! -f "$APP_PATH/package.json" ]; then
  echo -e "${RED}Error: package.json not found in $APP_PATH${NC}"
  exit 1
fi

echo -e "${GREEN}🚀 Starting simple release process for ${BLUE}$APP_NAME${NC}..."

# Navigate to app directory
cd "$APP_PATH"

# Get current version from package.json
CURRENT_VERSION=$(node -p "require('./package.json').version")
echo -e "${YELLOW}Current version: $CURRENT_VERSION${NC}"

# Bump version using npm
echo -e "${GREEN}Bumping version ($RELEASE_TYPE)...${NC}"
npm version $RELEASE_TYPE --no-git-tag-version

# Get new version
NEW_VERSION=$(node -p "require('./package.json').version")
echo -e "${GREEN}New version: $NEW_VERSION${NC}"

# Go back to root
cd ../..

# Update manifest if it exists
if [ -f ".release-please-manifest.json" ]; then
  echo -e "${GREEN}Updating release manifest...${NC}"
  node -e "
    const fs = require('fs');
    const manifest = JSON.parse(fs.readFileSync('.release-please-manifest.json', 'utf8'));
    manifest['apps/$APP_NAME'] = '$NEW_VERSION';
    fs.writeFileSync('.release-please-manifest.json', JSON.stringify(manifest, null, 2) + '\n');
  " 2>/dev/null || true
fi

# Commit the version bump
echo -e "${GREEN}Committing changes...${NC}"
git add "$APP_PATH/package.json" "$APP_PATH/package-lock.json" .release-please-manifest.json 2>/dev/null || true
git commit -m "chore($APP_NAME): bump version to v$NEW_VERSION" || echo "No changes to commit"

# Create and push tag
# Convert app name to tag format (replace dots with dashes for cleaner tags)
TAG_PREFIX=$(echo "$APP_NAME" | tr '.' '-')
TAG="${TAG_PREFIX}-v${NEW_VERSION}"
echo -e "${GREEN}Creating tag: $TAG${NC}"
git tag -a "$TAG" -m "Release $APP_NAME v$NEW_VERSION"

# Push changes and tags
echo -e "${GREEN}Pushing to origin...${NC}"
git push origin HEAD
git push origin "$TAG"

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Release complete!${NC}"
echo -e "${BLUE}   App:     $APP_NAME${NC}"
echo -e "${BLUE}   Version: $NEW_VERSION${NC}"
echo -e "${BLUE}   Tag:     $TAG${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}GitHub Actions will handle the rest if you have workflows set up.${NC}"
