#!/bin/bash

set -e

if [ ! -f apps/launchpad-public-api/.npmrc ]; then
    ./scripts/create-npmrc.sh "apps/launchpad-public-api" $PACKAGES_TOKEN
fi