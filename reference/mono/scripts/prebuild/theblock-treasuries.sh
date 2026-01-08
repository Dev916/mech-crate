#!/bin/bash

set -e

rm -rf apps/theblock-treasuries/node_modules

echo "node_modules removed"

if [ ! -f apps/theblock-treasuries/.npmrc ]; then
    ./scripts/create-npmrc.sh "apps/theblock-treasuries" $PACKAGES_TOKEN
fi
