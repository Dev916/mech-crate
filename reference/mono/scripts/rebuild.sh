#!/bin/bash

echo "Rebuilding Building image locally..."
echo
echo

if [ -z "$1" ]; then
    echo "Usage: $0 <service>"
    exit 1
fi

dockerfile=docker/dockerfiles/$1/rebuild

if [ ! -f $dockerfile ]; then
    echo "Dockerfile $dockerfile not found"
    exit 1
fi

SERVICE=$(echo $1 | tr '[:upper:]' '[:lower:]')
BASE_TAG=ghcr.io/theblockcrypto/$SERVICE:latest
CURRENT_DIR=$(pwd)
IMAGE_VERSION=$(cd apps/$1 && git describe --tags --always --dirty)
cd $CURRENT_DIR
BASE_FILE=docker/dockerfiles/$1/rebuild

echo "Building image locally...."
echo "Base Tag: $BASE_TAG"
echo "Image Version: $IMAGE_VERSION"
echo "Dockerfile: $BASE_FILE"
echo
echo

if [ -f scripts/prebuild/${service}.sh ]; then
    echo "Running prebuild script for $service"
    scripts/prebuild/${service}.sh
fi

docker build \
    -t $BASE_TAG \
    --build-arg "IMAGE_VERSION=$IMAGE_VERSION" \
    --build-arg "IMAGE_TAG=$BASE_TAG" \
    --no-cache \
    --file=$BASE_FILE \
    ./