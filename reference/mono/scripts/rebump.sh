#!/bin/bash

for PROJECT in launchpad-research launchpad-news launchpad-funding launchpad-public-api; do
    pushd apps/$PROJECT
    git checkout feat-pro-1913-simon-ai-implementaition
    git pull origin feat-pro-1913-simon-ai-implementaition
    git commit --allow-empty -m "chore: rebump bloxt"
    git push origin feat-pro-1913-simon-ai-implementaition
    popd
done
