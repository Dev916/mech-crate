#!/bin/bash

BASE_TAG=ghcr.io/theblockcrypto/$1:latest
CURRENT_DIR=$(pwd)
IMAGE_VERSION=$(cd apps/$1 && git describe --tags --always --dirty)
cd $CURRENT_DIR
BASE_FILE=docker/dockerfiles/$1/${2:-app}

echo "Building image locally...."
echo
echo

docker build \
    -t $BASE_TAG \
    --build-arg "IMAGE_VERSION=$IMAGE_VERSION" \
    --build-arg "IMAGE_TAG=$BASE_TAG" \
    --no-cache \
    --file=$BASE_FILE \
    ./
