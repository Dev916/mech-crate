#!/bin/bash
# Run a command in a new container
# Usage: ./scripts/run.sh <service> [command]

set -e

source ./scripts/.bashrc

if [ -z "$1" ]; then
    echo "Usage: $0 <service> [command]"
    exit 1
fi

app_dev="false"
if [ "$3" == "1" ]; then
    app_dev="true"
fi

files=$(compose_context_files "$1" "$app_dev")

docker compose $files run --rm "$1" $2
