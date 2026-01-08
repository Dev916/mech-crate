#!/bin/bash

# Initialize submodules
echo "Initializing submodules..."
git submodule init

# Update submodules to pull the latest changes
echo "Updating submodules..."
git submodule update --remote

# Pull latest changes from each submodule
echo "Pulling latest changes from each submodule..."
git submodule foreach git pull origin $(git rev-parse --abbrev-ref HEAD)

echo "Submodules are up to date."
