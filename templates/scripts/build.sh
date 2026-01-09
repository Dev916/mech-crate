#!/bin/bash
# Build a service image
#
# Usage:
#   ./scripts/build.sh <service> [tag] [mode] [push] [extra_args...]
#
# Arguments:
#   service   - Service name (required)
#   tag       - Image tag (default: latest)
#   mode      - Build mode: dev or prod (default: dev)
#   push      - Push to registry: 0 or 1 (default: 0)
#   extra     - Extra docker build arguments (e.g., --platform=linux/amd64)
#
# Examples:
#   ./scripts/build.sh myservice                    # Dev build, latest tag
#   ./scripts/build.sh myservice v1.0.0             # Dev build, custom tag
#   ./scripts/build.sh myservice latest prod        # Production build
#   ./scripts/build.sh myservice v1.0.0 prod 1     # Production build & push

set -e

source ./scripts/.bashrc

# ─────────────────────────────────────────────────────────────────────────────
# Parse Arguments
# ─────────────────────────────────────────────────────────────────────────────
SERVICE=$(echo "$1" | tr '[:upper:]' '[:lower:]')
TAG=${2:-latest}
MODE=${3:-dev}
PUSH=${4:-0}
shift 4 2>/dev/null || true
EXTRA_ARGS="$@"

if [ -z "$SERVICE" ]; then
    echo "Usage: $0 <service> [tag] [mode] [push] [extra_args...]"
    echo ""
    echo "Arguments:"
    echo "  service   Service name (required)"
    echo "  tag       Image tag (default: latest)"
    echo "  mode      Build mode: dev or prod (default: dev)"
    echo "  push      Push to registry: 0 or 1 (default: 0)"
    echo ""
    echo "Available services:"
    ls -1 docker/dockerfiles/ 2>/dev/null | sed 's/^/  - /'
    exit 1
fi

# ─────────────────────────────────────────────────────────────────────────────
# Determine Dockerfile
# ─────────────────────────────────────────────────────────────────────────────
DOCKERFILE_DIR="docker/dockerfiles/$SERVICE"

if [ "$MODE" = "prod" ] || [ "$MODE" = "production" ]; then
    # Production: try app.prod first, fallback to app with production target
    if [ -f "$DOCKERFILE_DIR/app.prod" ]; then
        DOCKERFILE="$DOCKERFILE_DIR/app.prod"
        TARGET="production"
        BUILD_MODE="production"
    elif [ -f "$DOCKERFILE_DIR/app" ]; then
        DOCKERFILE="$DOCKERFILE_DIR/app"
        TARGET="production"
        BUILD_MODE="production"
    else
        print_error "Dockerfile not found in: $DOCKERFILE_DIR"
        exit 1
    fi
else
    # Development: use regular app dockerfile with development target
    if [ -f "$DOCKERFILE_DIR/app" ]; then
        DOCKERFILE="$DOCKERFILE_DIR/app"
        TARGET="development"
        BUILD_MODE="development"
    else
        print_error "Dockerfile not found: $DOCKERFILE_DIR/app"
        exit 1
    fi
fi

# ─────────────────────────────────────────────────────────────────────────────
# Image Metadata
# ─────────────────────────────────────────────────────────────────────────────

# Get image version from git if available
if command -v git &> /dev/null && git rev-parse --git-dir > /dev/null 2>&1; then
    GIT_BRANCH=$(git rev-parse --abbrev-ref HEAD | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g')
    GIT_SHA=$(git rev-parse --short=8 HEAD)
    GIT_DIRTY=""
    if ! git diff --quiet HEAD 2>/dev/null; then
        GIT_DIRTY="-dirty"
    fi
    IMAGE_VERSION="${GIT_BRANCH}:${GIT_SHA}${GIT_DIRTY}"
else
    IMAGE_VERSION="local"
fi

# Build timestamp
BUILD_TIME=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Project name from env or directory
PROJECT_NAME="${PROJECT_NAME:-$(basename $(pwd))}"

# Determine image name/tag
if [ "$MODE" = "prod" ] || [ "$MODE" = "production" ]; then
    IMAGE_NAME="${PROJECT_NAME}/${SERVICE}:${TAG}"
    # Also tag with mode for clarity
    IMAGE_NAME_FULL="${PROJECT_NAME}/${SERVICE}:${TAG}-prod"
else
    IMAGE_NAME="${PROJECT_NAME}/${SERVICE}:${TAG}"
    IMAGE_NAME_FULL="${PROJECT_NAME}/${SERVICE}:${TAG}-dev"
fi

# ─────────────────────────────────────────────────────────────────────────────
# Pre-build Checks
# ─────────────────────────────────────────────────────────────────────────────

# Display build info
echo ""
print_info "Building image..."
echo "  Service:    $SERVICE"
echo "  Tag:        $TAG"
echo "  Mode:       $BUILD_MODE"
echo "  Image:      $IMAGE_NAME"
echo "  Dockerfile: $DOCKERFILE"
echo "  Target:     $TARGET"
echo "  Version:    $IMAGE_VERSION"
echo "  Push:       $([ "$PUSH" = "1" ] && echo "yes" || echo "no")"
[ -n "$EXTRA_ARGS" ] && echo "  Extra args: $EXTRA_ARGS"
echo ""

# Run prebuild script if exists
if [ -f "scripts/prebuild/${SERVICE}.sh" ]; then
    print_info "Running prebuild script..."
    scripts/prebuild/${SERVICE}.sh
fi

# ─────────────────────────────────────────────────────────────────────────────
# Build Image
# ─────────────────────────────────────────────────────────────────────────────

# Build arguments
BUILD_ARGS=(
    --build-arg "IMAGE_VERSION=$IMAGE_VERSION"
    --build-arg "IMAGE_TAG=$TAG"
    --build-arg "BUILD_TIME=$BUILD_TIME"
    --build-arg "BUILD_MODE=$BUILD_MODE"
)

# Production-specific optimizations
if [ "$BUILD_MODE" = "production" ]; then
    BUILD_ARGS+=(
        --build-arg "NODE_ENV=production"
        --build-arg "RUST_ENV=production"
        --build-arg "APP_ENV=production"
    )
fi

# Build the image
DOCKER_BUILDKIT=1 docker build \
    -t "$IMAGE_NAME" \
    -t "$IMAGE_NAME_FULL" \
    "${BUILD_ARGS[@]}" \
    --target "$TARGET" \
    --file="$DOCKERFILE" \
    $EXTRA_ARGS \
    .

print_success "Image built: $IMAGE_NAME"

# ─────────────────────────────────────────────────────────────────────────────
# Post-build Actions
# ─────────────────────────────────────────────────────────────────────────────

# Update env file with new tag
ENV_VAR=$(convert_service_to_env_var "$SERVICE")
setenv "$ENV_VAR" "$TAG"

# Show image size
if command -v docker &> /dev/null; then
    IMAGE_SIZE=$(docker images "$IMAGE_NAME" --format "{{.Size}}" 2>/dev/null | head -1)
    if [ -n "$IMAGE_SIZE" ]; then
        echo "  Image size: $IMAGE_SIZE"
    fi
fi

# Push if requested
if [ "$PUSH" = "1" ]; then
    echo ""
    print_info "Pushing image to registry..."
    docker push "$IMAGE_NAME"
    docker push "$IMAGE_NAME_FULL"
    print_success "Image pushed: $IMAGE_NAME"
fi

# Production build summary
if [ "$BUILD_MODE" = "production" ]; then
    echo ""
    print_success "Production build complete!"
    echo ""
    echo "  To run: docker run -p 8080:8080 $IMAGE_NAME"
    echo "  To push: docker push $IMAGE_NAME"
fi
