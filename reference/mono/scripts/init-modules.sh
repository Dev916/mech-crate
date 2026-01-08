#!/bin/bash

apps=$(cat ./scripts/apps.txt)
infra=$(cat ./scripts/infra.txt)
libs=$(cat ./scripts/libs.txt)

read -p "Do you want to add app modules? (y/n) " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    for app in $apps; do
        git submodule add git@github.com:TheBlockCrypto/$app.git apps/$app
    done
fi

read -p "Do you want to add infra modules? (y/n) " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    for inf in $infra; do
        git submodule add git@github.com:TheBlockCrypto/$inf.git infra/$inf
    done
fi

read -p "Do you want to add lib modules? (y/n) " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    for lib in $libs; do
        git submodule add git@github.com:TheBlockCrypto/$lib.git lib/$lib
    done
fi
