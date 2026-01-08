#!/bin/bash

set -e

STACK=$1

if [ -z "$STACK" ]; then
  echo "Error: Stack name is required"
  echo "Usage: $0 <stack-name>"
  echo "Available stacks: campus, connect, tbco"
  exit 1
fi

# Create temp directory
mkdir -p ./tmp/up

# Define which repos are needed for each stack
declare -A STACK_REPOS

# Campus stack requires campus-lms and theblock.pro
STACK_REPOS[campus]="apps/campus-lms apps/theblock.pro"

# Connect stack requires launchpad-api, launchpad, and connect
STACK_REPOS[connect]="apps/launchpad-api apps/launchpad apps/connect"

# tbco stack requires theblock.pro only
STACK_REPOS[tbco]="apps/theblock.pro"

# Get the repos for this stack
REPOS="${STACK_REPOS[$STACK]}"

if [ -z "$REPOS" ]; then
  echo "Error: Unknown stack '$STACK'"
  echo "Available stacks: campus, connect, tbco"
  exit 1
fi

echo "Initializing stack: $STACK"
echo "Will initialize the following repos:"
for repo in $REPOS; do
  echo "  - $repo"
done
echo ""

# Initialize only the specific submodules for this stack
for repo in $REPOS; do
  echo "Initializing submodule: $repo"
  git submodule init "$repo"
  
  echo "Updating submodule: $repo"
  git submodule update --remote "$repo"
  
  # Pull latest changes for this submodule
  echo "Pulling latest changes for: $repo"
  (cd "$repo" && git pull origin $(git rev-parse --abbrev-ref HEAD) || echo "Warning: Could not pull latest for $repo")
  
  echo ""
done

# Run stack-specific initialization if needed
case "$STACK" in
  campus)
    echo "Running campus-specific initialization..."
    # Import campus-lms data if needed for campus
    if [ -f "./scripts/run.sh" ]; then
      echo "Importing campus-lms data for campus..."
      ./scripts/run.sh campus-lms import-data || echo "Warning: Could not import campus-lms data"
    fi
    ;;
  connect)
    echo "Running connect-specific initialization..."
    # Add any connect-specific initialization here
    ;;
  tbco)
    echo "Running tbco-specific initialization..."
    # Add any tbco-specific initialization here
    ;;
esac

echo ""
echo "Stack '$STACK' initialization complete!"
echo ""
echo "You can now launch the stack with: make launch s=$STACK"
