#!/usr/bin/env bash
# Get current version from Cargo.toml
#
# Usage: ./scripts/get-version.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

CARGO_TOML="$PROJECT_ROOT/Cargo.toml"

if [[ ! -f "$CARGO_TOML" ]]; then
    echo "Error: Cargo.toml not found at $CARGO_TOML" >&2
    exit 1
fi

# Extract version - handle various formats:
# version = "1.0.0"
# version = '1.0.0'
# version="1.0.0"
VERSION_LINE=$(grep -E '^version[[:space:]]*=' "$CARGO_TOML" | head -1)

if [[ -z "$VERSION_LINE" ]]; then
    echo "Error: No version line found in $CARGO_TOML" >&2
    exit 1
fi

# Try double quotes first, then single quotes
if [[ "$VERSION_LINE" =~ \"([^\"]+)\" ]]; then
    VERSION="${BASH_REMATCH[1]}"
elif [[ "$VERSION_LINE" =~ \'([^\']+)\' ]]; then
    VERSION="${BASH_REMATCH[1]}"
else
    echo "Error: Could not parse version from line: $VERSION_LINE" >&2
    exit 1
fi

if [[ -z "$VERSION" ]]; then
    echo "Error: Could not extract version from $CARGO_TOML" >&2
    echo "Expected format: version = \"x.y.z\"" >&2
    exit 1
fi

# Validate version format (basic semver check)
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$ ]]; then
    echo "Warning: Version '$VERSION' may not be valid semver" >&2
fi

echo "$VERSION"
