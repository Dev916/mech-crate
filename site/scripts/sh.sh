#!/bin/bash
# Shell into a running service
# Usage: ./scripts/sh.sh <service>

set -e

source ./scripts/.bashrc

if [ -z "$1" ]; then
    echo "Usage: $0 <service>"
    exit 1
fi

files=$(compose_context_files "$1" "true")

docker compose $files exec "$1" sh
