#!/bin/bash

apps=$(cat ./scripts/apps.txt)
infra=$(cat ./scripts/infra.txt)
libs=$(cat ./scripts/libs.txt)

pull_latest() {
    DEFAULT_BRANCH="main"
    pushd $1
    git fetch
    if git show-ref --verify --quiet refs/heads/main; then
        DEFAULT_BRANCH="main"
    elif git show-ref --verify --quiet refs/heads/master; then
        DEFAULT_BRANCH="master"
    else
        echo "Error: Neither 'main' nor 'master' branch found."
    fi
    git checkout $DEFAULT_BRANCH
    git pull origin $DEFAULT_BRANCH
    popd >/dev/null
}

for app in $apps; do
    pull_latest apps/$app
done

for inf in $infra; do
    pull_latest infra/$inf
done

for lib in $libs; do
    pull_latest lib/$lib
done
