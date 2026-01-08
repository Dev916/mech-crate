#!/bin/bash

set -e

if [ ! -f apps/launchpad/bunfig.toml ]; then
    ./scripts/create-bunfig.sh "apps/launchpad" $PACKAGES_TOKEN
fi
