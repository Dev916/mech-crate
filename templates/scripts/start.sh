#!/bin/bash
# Start services from saved state
# Usage: ./scripts/start.sh

set -e

if [ ! -f "./tmp/start.txt" ]; then
    echo "No saved state found!"
    echo "Use 'make save' to save current state first."
    exit 1
fi

services=$(cat ./tmp/start.txt)

if [ -z "$services" ]; then
    echo "No services to start!"
    exit 1
fi

echo "Starting services from saved state..."

for service in $services; do
    echo "Starting $service..."
    make dev s=$service
done

echo "All services started."
