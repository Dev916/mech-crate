#!/bin/bash

set -e

APP_PATH=$1
PACKAGE_PAT=$2

if [ -z "$APP_PATH" ]; then
  echo "Usage: $0 <app_path>"
  exit 1
fi

if [ ! -d "$APP_PATH" ]; then
  echo "Directory $APP_PATH does not exist"
  exit 1
fi

while [ -z "$PACKAGE_PAT" ]; do
  read -p "Please enter a Personal Access Token with package read/write permissions: " PACKAGE_PAT
  if [ -z "$PACKAGE_PAT" ]; then
    echo "Token cannot be empty. Please provide a valid Personal Access Token."
  fi
done

cat << EOF > $APP_PATH/.npmrc
[install.scopes]
"@theblockcrypto" = { token = "$PACKAGE_PAT", url = "https://npm.pkg.github.com/" }
EOF
