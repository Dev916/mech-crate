#!/usr/bin/env bash
set -euo pipefail

# inject.sh: Copies Docker and core scripts into a service's app folder
# Usage: ./scripts/inject.sh <service>
if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <service>"
  exit 1
fi
SERVICE="$1"

# Determine project root (parent of scripts/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="${SCRIPT_DIR}/.."
APP_DIR="${ROOT_DIR}/apps/${SERVICE}"

if [ ! -d "${APP_DIR}" ]; then
  echo "Error: Service directory not found: ${APP_DIR}"
  exit 1
fi

echo "Injecting Docker and scripts into: ${APP_DIR}"

# Copy docker/ folder
DEST_DOCKER="${APP_DIR}/docker"
rm -rf "${DEST_DOCKER}"
echo "  - Copying docker/ to ${DEST_DOCKER}"
cp -R "${ROOT_DIR}/docker" "${DEST_DOCKER}"

# Copy core scripts
DEST_SCRIPTS="${APP_DIR}/scripts"
rm -rf "${DEST_SCRIPTS}"
mkdir -p "${DEST_SCRIPTS}"
CORE_SCRIPTS=(up.sh down.sh build.sh rebuild.sh logs.sh)
for script in "${CORE_SCRIPTS[@]}"; do
  if [ -f "${ROOT_DIR}/scripts/${script}" ]; then
    echo "  - Copying ${script} to ${DEST_SCRIPTS}/"
    cp "${ROOT_DIR}/scripts/${script}" "${DEST_SCRIPTS}/"
    chmod +x "${DEST_SCRIPTS}/${script}"
  else
    echo "  ! Warning: ${script} not found in root scripts"
  fi
done

# Copy .bashrc for helper functions
if [ -f "${ROOT_DIR}/scripts/.bashrc" ]; then
  echo "  - Copying .bashrc to ${DEST_SCRIPTS}/"
  cp "${ROOT_DIR}/scripts/.bashrc" "${DEST_SCRIPTS}/"
else
  echo "  ! Warning: .bashrc not found in root scripts"
fi

echo "Injection complete."
echo "  To use, cd ${APP_DIR} and run:"
echo "    scripts/up.sh ${SERVICE}"