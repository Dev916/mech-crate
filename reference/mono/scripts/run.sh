#!/bin/bash

if [ -z "$1" ]; then
    echo "Usage: $0 <service> [command]"
    exit 1
fi

source ./scripts/.bashrc

app_dev="false"

if [ "$3" == "1" ]; then
    app_dev="true"
fi

files=$(compose_context_files "$1" "$app_dev")

docker compose $files run --rm "$1" $2
