#!/bin/bash

set -e

if [ ! -f apps/theblock.co/.npmrc ]; then
    ./scripts/create-npmrc.sh "apps/theblock.co" $PACKAGES_TOKEN
fi
