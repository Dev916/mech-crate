#!/bin/bash

for PROJECT in launchpad-research launchpad-news launchpad-funding launchpad-public-api; do
    pushd apps/$PROJECT
    git checkout main
    git pull
    git checkout -b connect/tiers
    git add Dockerfile
    git commit -m "chore: bump bloxt"
    git push origin connect/tiers
    gh pr create -f
    popd
done
