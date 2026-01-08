#!/bin/bash

if [ -z "$1" ]; then
    echo "Usage: $0 <service>"
    exit 1
fi

base_file=docker/compose/${1}.yml
files=""

if [ -f "$base_file" ]; then
    files="-f $base_file"
else
    echo "File $base_file not found"
    exit 1
fi

docker compose $files pull $1
