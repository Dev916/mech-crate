#!/bin/bash

set -e

if [ ! -f apps/launchpad-api/.npmrc ]; then
    ./scripts/create-npmrc.sh "apps/launchpad-api" $PACKAGES_TOKEN
fi
