#!/bin/bash
# Initialize the project environment
# Usage: ./scripts/init.sh

set -e

source ./scripts/.bashrc

print_info "Initializing MechCrate project..."

# Create required directories
mkdir -p ./tmp/up
mkdir -p ./data

# Secrets: create the file and fill in real dev credentials.
#
# Delegated to scripts/generate-secrets.sh, the single generation point for every
# recipe. It runs on every `make dev` (via dev.sh) and is idempotent — it fills
# only values that are still empty or a placeholder — so a service added with
# `mx add` after `mx new` gets real credentials without a hand edit, and an
# existing stack's credentials are never rotated underneath it.
if [ -x ./scripts/generate-secrets.sh ]; then
    ./scripts/generate-secrets.sh
else
    # A project scaffolded before the generator shipped. `mx upgrade` adds it.
    print_warn "scripts/generate-secrets.sh missing - run 'mx upgrade' to add it"
    if [ ! -f "docker/.config/.env.secrets" ] && [ -f "docker/.config/.env.secrets.template" ]; then
        cp docker/.config/.env.secrets.template docker/.config/.env.secrets
        print_warn "Copied the secrets template verbatim - edit it before 'make dev'"
    fi
fi

# Create Docker network if it doesn't exist
NETWORK_NAME="${NETWORK_NAME:-mech-network}"
if ! docker network inspect "$NETWORK_NAME" &>/dev/null; then
    print_info "Creating Docker network: $NETWORK_NAME"
    docker network create "$NETWORK_NAME"
fi

print_success "Project initialized!"
print_info "Run 'make dev' to start development"
