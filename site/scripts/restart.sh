#!/bin/bash
# Restart a service
# Usage: ./scripts/restart.sh <service>

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <service>"
    exit 1
fi

./scripts/down.sh "$1"
./scripts/dev.sh "$1"
