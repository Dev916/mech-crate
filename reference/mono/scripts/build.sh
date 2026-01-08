#!/bin/bash

set -e

source scripts/.bashrc

SERVICE=$(echo $1 | tr '[:upper:]' '[:lower:]')

TAG=${2:-latest}

BASE_TAG=ghcr.io/theblockcrypto/$SERVICE:$TAG

ENV_VAR=$(convert_service_to_env_var "$SERVICE")

CURRENT_DIR=$(pwd)

if [ -d "apps/$1" ]; then
    IMAGE_VERSION=$(cd apps/$1 && \
        git rev-parse --abbrev-ref HEAD | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g'):$(cd apps/$1 && \
        git rev-parse --short=8 HEAD)
else
    IMAGE_VERSION="latest"
fi

BASE_FILE=docker/dockerfiles/$1/app

cd $CURRENT_DIR

if [ -f scripts/prebuild/${service}.sh ]; then
    echo "Running prebuild script for $service"
    scripts/prebuild/${service}.sh
fi

echo "Building image locally...."
echo "Base Tag: $BASE_TAG"
echo "Image Version: $IMAGE_VERSION"
echo "Dockerfile: $BASE_FILE"
echo
echo

BUILDKIT_PROGRESS=plain docker build \
    -t $BASE_TAG \
    --build-arg "IMAGE_VERSION=$IMAGE_VERSION" \
    --build-arg "IMAGE_TAG=$BASE_TAG" \
    --build-arg "NEW_RELIC_LICENSE_KEY=$NEW_RELIC_LICENSE_KEY" \
    --no-cache \
    --file=$BASE_FILE \
    .

setenv "$ENV_VAR" "$TAG"
