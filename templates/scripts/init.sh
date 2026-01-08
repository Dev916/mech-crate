#!/bin/bash
# Initialize the project environment
# Usage: ./scripts/init.sh

set -e

source ./scripts/.bashrc

print_info "Initializing MechCrate project..."

# Create required directories
mkdir -p ./tmp/up
mkdir -p ./data

# Check for secrets file
if [ ! -f "docker/.config/.env.secrets" ]; then
    if [ -f "docker/.config/.env.secrets.template" ]; then
        print_warn "Secrets file not found. Creating from template..."
        cp docker/.config/.env.secrets.template docker/.config/.env.secrets
        print_info "Please edit docker/.config/.env.secrets with your values"
    else
        print_warn "No secrets template found. Creating empty secrets file..."
        touch docker/.config/.env.secrets
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
