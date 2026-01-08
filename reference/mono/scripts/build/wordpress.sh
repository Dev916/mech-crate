#!/bin/bash

BASE_TAG=ghcr.io/theblockcrypto/wordpress:latest
BASE_FILE=Dockerfile
IMAGE_VERSION=$(git describe --tags --always --dirty)
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
