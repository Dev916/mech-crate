#!/bin/bash

make down

echo "Importing data..."
make import-data
make down

echo "Syncing Vector DB..."
make run s=WordPress c="cli seed syncVectorDb" d=1

make down

echo "Setting up connect base data..."

make run s=connect c=import-data d=1
make run s=connect c=provision-bot-permissions

make down

echo "Launching connect 🚀🚀🚀🚀..."

make dev
make append s=connect

make append s=launchpad-api
make append s=launchpad
