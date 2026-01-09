# Build commands
# Supports development and production builds
#
# Usage:
#   make build s=myservice                   # Dev build (default)
#   make build s=myservice prod=1            # Production build
#   make build-prod s=myservice              # Production build (alias)
#   make build s=myservice t=v1.0.0          # Custom tag
#   make build s=myservice prod=1 push=1    # Build & push to registry

.PHONY: build build-prod build-dev _build

# ─────────────────────────────────────────────────────────────────────────────
# Build Target Configuration
# ─────────────────────────────────────────────────────────────────────────────
BUILD_MODE ?= dev
PUSH_IMAGE ?= 0

# Determine production mode from flags
ifdef prod
    BUILD_MODE := prod
endif

ifdef production
    BUILD_MODE := prod
endif

# ─────────────────────────────────────────────────────────────────────────────
# Main Build Targets
# ─────────────────────────────────────────────────────────────────────────────

build: ## Build image (s=[service] t=[tag] prod=[0|1] push=[0|1])
	@$(MAKE) _build service=$(call get_service) tag=$(call get_tag) mode=$(BUILD_MODE) push=$(PUSH_IMAGE)

build-dev: ## Build development image (s=[service] t=[tag])
	@$(MAKE) _build service=$(call get_service) tag=$(call get_tag) mode=dev push=0

build-prod: ## Build production image (s=[service] t=[tag] push=[0|1])
	@$(MAKE) _build service=$(call get_service) tag=$(call get_tag) mode=prod push=$(PUSH_IMAGE)

_build:
	@./scripts/build.sh $(service) $(tag) $(mode) $(push)

# ─────────────────────────────────────────────────────────────────────────────
# Multi-Platform Builds (for CI/CD)
# ─────────────────────────────────────────────────────────────────────────────

.PHONY: build-multiplatform

build-multiplatform: ## Build multi-platform production image (s=[service] t=[tag])
	@echo "Building multi-platform image..."
	@./scripts/build.sh $(call get_service) $(call get_tag) prod 0 --platform=linux/amd64,linux/arm64
