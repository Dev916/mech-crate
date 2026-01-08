#!/bin/bash
# Build a service image
# Usage: ./scripts/build.sh <service> [tag]

set -e

source ./scripts/.bashrc

SERVICE=$(echo "$1" | tr '[:upper:]' '[:lower:]')
TAG=${2:-latest}

if [ -z "$SERVICE" ]; then
    echo "Usage: $0 <service> [tag]"
    echo "Available services:"
    ls -1 docker/dockerfiles/ 2>/dev/null | sed 's/^/  - /'
    exit 1
fi

# Check if dockerfile exists
DOCKERFILE="docker/dockerfiles/$SERVICE/app"
if [ ! -f "$DOCKERFILE" ]; then
    echo "Dockerfile not found: $DOCKERFILE"
    exit 1
fi

# Get image version from git if available
if command -v git &> /dev/null && git rev-parse --git-dir > /dev/null 2>&1; then
    IMAGE_VERSION=$(git rev-parse --abbrev-ref HEAD | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g'):$(git rev-parse --short=8 HEAD)
else
    IMAGE_VERSION="local"
fi

# Determine image name/tag
IMAGE_NAME="${PROJECT_NAME:-$(basename $(pwd))}/$SERVICE:$TAG"

echo "Building image..."
echo "  Service: $SERVICE"
echo "  Tag: $TAG"
echo "  Image: $IMAGE_NAME"
echo "  Version: $IMAGE_VERSION"
echo "  Dockerfile: $DOCKERFILE"
echo ""

# Run prebuild script if exists
if [ -f "scripts/prebuild/${SERVICE}.sh" ]; then
    echo "Running prebuild script..."
    scripts/prebuild/${SERVICE}.sh
fi

# Build the image
DOCKER_BUILDKIT=1 docker build \
    -t "$IMAGE_NAME" \
    --build-arg "IMAGE_VERSION=$IMAGE_VERSION" \
    --build-arg "IMAGE_TAG=$TAG" \
    --target production \
    --file="$DOCKERFILE" \
    .

print_success "Image built: $IMAGE_NAME"

# Update env file with new tag
ENV_VAR=$(convert_service_to_env_var "$SERVICE")
setenv "$ENV_VAR" "$TAG"
