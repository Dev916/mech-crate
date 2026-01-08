#!/bin/bash

set -e

rm -rf apps/theblock.pro/node_modules

echo "node_modules removed"

if [ ! -f apps/theblock.pro/.npmrc ]; then
    ./scripts/create-npmrc.sh "apps/theblock.pro" $PACKAGES_TOKEN
fi
