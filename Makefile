# MechCrate Project Makefile
# 🦝 Crate Raccoon

ROOT_DIR := $(shell pwd)
CARGO := cargo
PREFIX ?= /usr/local

.PHONY: build build-release install install-local uninstall init test test-unit test-integration lint fmt clean help

# Include documentation module
-include make/docs.mk

# ─────────────────────────────────────────────────────────────────────────────
# Build
# ─────────────────────────────────────────────────────────────────────────────

## Build debug binaries
build:
	@echo "Building debug binaries..."
	$(CARGO) build -p mx-cli -p mx-mcp-server

## Build release binaries
build-release:
	@echo "Building release binaries..."
	$(CARGO) build --release -p mx-cli -p mx-mcp-server
	@echo ""
	@echo "Binaries:"
	@ls -lh target/release/mx target/release/mx-mcp target/release/mx-ingest 2>/dev/null || true

# ─────────────────────────────────────────────────────────────────────────────
# Install
# ─────────────────────────────────────────────────────────────────────────────

## Install mx globally to $(PREFIX)/bin (default: /usr/local/bin)
install: build-release
	@./scripts/install.sh --prefix $(PREFIX) --skip-build

## Install mx to ~/.local/bin (no sudo needed)
install-local: build-release
	@./scripts/install.sh --local --skip-build

## Uninstall mx from $(PREFIX)/bin
uninstall:
	@echo "Removing mx binaries..."
	@rm -f $(PREFIX)/bin/mx $(PREFIX)/bin/mx-mcp $(PREFIX)/bin/mx-ingest 2>/dev/null || \
		sudo rm -f $(PREFIX)/bin/mx $(PREFIX)/bin/mx-mcp $(PREFIX)/bin/mx-ingest
	@echo "✓ mx uninstalled from $(PREFIX)/bin"

## Initialize MechCrate (copy templates to ~/.mech-crate)
init: build-release
	@MECH_CRATE_ROOT=$(ROOT_DIR) ./target/release/mx init --force

# ─────────────────────────────────────────────────────────────────────────────
# Test
# ─────────────────────────────────────────────────────────────────────────────

## Run all tests
test: test-unit test-integration

## Run unit tests
test-unit:
	@echo "Running unit tests..."
	$(CARGO) test -p mx-lib

## Run CLI integration tests
test-integration: build-release init
	@echo "Running integration tests..."
	$(CARGO) test -p mx-cli

## Run bash smoke tests
test-smoke: init
	@echo "Running smoke tests..."
	@./tests/testbed/testbed.sh

# ─────────────────────────────────────────────────────────────────────────────
# Quality
# ─────────────────────────────────────────────────────────────────────────────

## Run clippy linter
lint:
	$(CARGO) clippy --all-targets -- -D warnings

## Format code
fmt:
	$(CARGO) fmt

## Check formatting
fmt-check:
	$(CARGO) fmt --check

## Run all quality checks
check: fmt-check lint test

# ─────────────────────────────────────────────────────────────────────────────
# Development
# ─────────────────────────────────────────────────────────────────────────────

## Run mx CLI directly (debug)
run:
	@MECH_CRATE_ROOT=$(ROOT_DIR) $(CARGO) run -p mx-cli -- $(ARGS)

## Watch and rebuild on changes
watch:
	$(CARGO) watch -x 'build -p mx-cli'

# ─────────────────────────────────────────────────────────────────────────────
# Maintenance
# ─────────────────────────────────────────────────────────────────────────────

## Clean build artifacts
clean:
	$(CARGO) clean
	@rm -rf target/

## Update dependencies
update:
	$(CARGO) update

# ─────────────────────────────────────────────────────────────────────────────
# Help
# ─────────────────────────────────────────────────────────────────────────────

## Show this help
help:
	@echo ""
	@echo "🦝 MechCrate Development"
	@echo ""
	@echo "Build:"
	@echo "  make build          Build debug binaries"
	@echo "  make build-release  Build release binaries"
	@echo ""
	@echo "Install:"
	@echo "  make install        Install to /usr/local/bin (may need sudo)"
	@echo "  make install-local  Install to ~/.local/bin"
	@echo "  make uninstall      Remove installed binaries"
	@echo "  make init           Initialize templates (~/.mech-crate)"
	@echo ""
	@echo "Test:"
	@echo "  make test           Run all tests"
	@echo "  make test-unit      Run unit tests"
	@echo "  make test-integration Run integration tests"
	@echo "  make test-smoke     Run bash smoke tests"
	@echo ""
	@echo "Quality:"
	@echo "  make lint           Run clippy"
	@echo "  make fmt            Format code"
	@echo "  make check          Run all checks (fmt, lint, test)"
	@echo ""
	@echo "Development:"
	@echo "  make run ARGS=...   Run mx with arguments"
	@echo "  make watch          Watch and rebuild"
	@echo "  make clean          Clean build artifacts"
	@echo ""
	@echo "Examples:"
	@echo "  make run ARGS='doctor'"
	@echo "  make run ARGS='recipes list'"
	@echo "  make install PREFIX=~/opt"
	@echo ""
