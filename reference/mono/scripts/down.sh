#!/bin/bash

if ! ls tmp/up/*.txt 1>/dev/null 2>&1; then
    echo "No previous runs found. Doing normal down.."
    docker compose down -t0
    exit 0
fi

files=$(cat tmp/up/*.txt)

echo "docker compose $files stop $1"

docker compose $files stop $1
docker compose $files rm -f $1

if [ -z $1 ]; then
    rm -r tmp/up/*.txt
fi
