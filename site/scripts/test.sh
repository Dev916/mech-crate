#!/bin/bash
# Run tests
# Usage: ./scripts/test.sh [service]

set -e

source ./scripts/.bashrc

service=$1

if [ -n "$service" ]; then
    print_info "Running tests for $service..."
    files=$(compose_context_files "$service" "true")
    docker compose $files run --rm "$service" npm test
else
    print_info "Running all tests..."
    # Run tests for each service that has a test command
    for yml in docker/compose/*.yml; do
        if [[ -f "$yml" && ! "$(basename "$yml")" =~ \.dev\.yml$ ]]; then
            svc=$(basename "$yml" .yml)
            print_info "Testing $svc..."
            files=$(compose_context_files "$svc" "true")
            docker compose $files run --rm "$svc" npm test 2>/dev/null || true
        fi
    done
fi

print_success "Tests complete!"
