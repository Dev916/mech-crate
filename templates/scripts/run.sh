#!/bin/bash
# Run a command in a new container
# Usage: ./scripts/run.sh <service> [command]

set -e

source ./scripts/.bashrc

if [ -z "$1" ]; then
    echo "Usage: $0 <service> [command]"
    exit 1
fi

# One container gets the command: a list has no meaning here (bd:mech-crate-3kq).
mech_require_single_service "$1" "make run" || exit 1

app_dev="false"
if [ "$3" == "1" ]; then
    app_dev="true"
fi

files=$(compose_context_files "$1" "$app_dev")

docker compose -p "$COMPOSE_PROJECT_NAME" $files run --rm "$1" $2
