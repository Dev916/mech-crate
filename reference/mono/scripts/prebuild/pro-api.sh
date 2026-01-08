#!/bin/bash

rm -rf apps/pro-api/target

echo "target removed"

if docker volume ls | grep compose_pro-api; then
    docker volume ls -q | grep "pro-api" | xargs -r docker volume rm
fi
