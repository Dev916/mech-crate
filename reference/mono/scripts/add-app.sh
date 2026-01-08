#!/bin/bash

if [ -z "$1" ]; then
    echo "Usage: add-app.sh <app-name>"
    exit 1
fi

APP=$1

git submodule add git@github.com:TheBlockCrypto/$APP.git apps/$APP
echo "✅ Added $APP as a submodule"

touch docker/.config/.env.$APP
echo "✅ Created .env.$APP file"

mkdir docker/dockerfiles/$APP
touch docker/dockerfiles/$APP/app
touch docker/dockerfiles/$APP/base
touch docker/dockerfiles/$APP/rebuild
echo "✅ Created $APP Dockerfiles"

touch docker/compose/$APP.yml
touch docker/compose/$APP-dev.yml
echo "✅ Created $APP Docker Compose files"

mkdir docker/system/$APP
echo "✅ Created $APP system directory"

echo "${APP}_IMAGE_TAG=latest" >>docker/compose/.env
echo "$APP" >>scripts/apps.txt
echo "✅ Added $APP build config files"

echo "🎉🎉🎉 Congrats!!! App successfully added! ☕️"
