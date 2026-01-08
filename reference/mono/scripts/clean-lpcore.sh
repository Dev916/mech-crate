#!/bin/bash

# Define the submodule path
SUBMODULE_PATH="apps/launchpad-api-entities"

# Step 1: Remove the submodule entry from .gitmodules
echo "Removing submodule entry from .gitmodules..."
git config -f .gitmodules --remove-section submodule.$SUBMODULE_PATH

# Step 2: Remove the submodule entry from .git/config
echo "Removing submodule entry from .git/config..."
git config -f .git/config --remove-section submodule.$SUBMODULE_PATH

# Step 3: Remove the submodule directory from the working directory
echo "Removing submodule directory..."
rm -rf $SUBMODULE_PATH

# Step 4: Remove submodule references from the Git index
echo "Removing submodule references from Git index..."
git rm --cached $SUBMODULE_PATH

# Step 5: Commit the changes
echo "Committing changes..."
git commit -m "Removed submodule $SUBMODULE_PATH"

# Step 6: Remove the submodule directory from .git/modules
echo "Removing submodule directory from .git/modules..."
rm -rf .git/modules/$SUBMODULE_PATH

# Step 7: Update the git configuration
echo "Updating Git configuration..."
git config --remove-section submodule.$SUBMODULE_PATH

echo "Submodule $SUBMODULE_PATH has been successfully removed."
