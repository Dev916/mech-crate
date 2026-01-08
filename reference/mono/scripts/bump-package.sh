#!/bin/bash

# Ensure the script exits if any command fails
set -e

# Check if the required arguments are provided
if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
    echo "Usage: $0 <package-name> <new-version> [<path-to-package-json>]"
    exit 1
fi

PACKAGE_NAME=$1
NEW_VERSION=$2
PACKAGE_JSON_PATH=${3:-.}

# Ensure the package.json path is correct
PACKAGE_JSON_FILE="$PACKAGE_JSON_PATH/package.json"

if [ ! -f "$PACKAGE_JSON_FILE" ]; then
    echo "Error: package.json not found at path $PACKAGE_JSON_FILE"
    exit 1
fi

# Check if jq is installed
if ! command -v jq &>/dev/null; then
    echo "jq could not be found, please install jq to use this script."
    exit 1
fi

# Update the package version in package.json
jq --arg package "$PACKAGE_NAME" --arg version "$NEW_VERSION" '.dependencies[$package] = $version' "$PACKAGE_JSON_FILE" >temp.json && mv temp.json "$PACKAGE_JSON_FILE"
