#!/bin/bash

apps=$(cat ./scripts/apps.txt)
infra=$(cat ./scripts/infra.txt)

read -p "🚨🚨🚨 Are you sure you want to initialize the dockerfiles for the following apps, this will destroy all existing dockerfiles currently in place: $apps? (y/n) " -n 1 -r
echo

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
fi

for app in $apps; do
    mkdir docker/dockerfiles/$app
    touch docker/dockerfiles/$app/base
    touch docker/dockerfiles/$app/app
    touch docker/dockerfiles/$app/rebuild

    mkdir docker/system/$app
done
