#!/bin/bash

IMAGE_TAG=ghcr.io/theblockcrypto/connect:latest
ZULIP_GIT_URL=https://github.com/zulip/zulip.git
ZULIP_GIT_REF=8.4

docker buildx build \
    --build-arg ZULIP_GIT_URL=$ZULIP_GIT_URL \
    --build-arg ZULIP_GIT_REF=$ZULIP_GIT_REF \
    --tag $IMAGE_TAG \
    --file docker/dockerfiles/connect/app \
    .
