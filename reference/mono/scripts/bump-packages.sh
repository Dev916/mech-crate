#!/bin/bash

apps=$(cat ./scripts/node-apps.txt)
package=$1
version=$2

if [ -z $package ]; then
    echo "Please provide a package name"
    exit 1
fi

if [ -z $version ]; then
    echo "Please provide a version number"
    exit 1
fi

read -p "Bump package: $package to version: $version ?" -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    for app in $apps; do
        ./scripts/bump-package.sh $package $version "./apps/$app"

        pushd "./apps/$app"
        git add package.json
        git commit -m "chore(npm): Bump $package to version $version"
        git push
        echo "Successfully bumped $PACKAGE_NAME to version $NEW_VERSION and pushed the changes."
        popd
    done
fi
