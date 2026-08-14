# MechCrate Project Makefile
# 🦝 Crate Raccoon

ROOT_DIR := $(shell pwd)
CARGO := cargo
PREFIX ?= /usr/local

.PHONY: build build-release install install-local uninstall init lint fmt clean help
.PHONY: test test-unit test-int test-known-broken coverage test-e2e test-mutants test-smoke

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

## Rebuild and install to bin/ (symlinks in /usr/local/bin pick it up)
upgrade: build-release
	@echo "Installing to bin/..."
	@cp target/release/mx bin/mx && chmod +x bin/mx && echo "  ✓ bin/mx"
	@test -f target/release/mx-mcp && cp target/release/mx-mcp bin/mx-mcp && chmod +x bin/mx-mcp && echo "  ✓ bin/mx-mcp" || true
	@test -f target/release/mx-ingest && cp target/release/mx-ingest bin/mx-ingest && chmod +x bin/mx-ingest && echo "  ✓ bin/mx-ingest" || true
	@echo ""
	@echo "🦝 mx upgraded! Run 'mx --version' to verify."

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
# Testing
# ─────────────────────────────────────────────────────────────────────────────

TEST_DB_URL ?= postgres://postgres@localhost:55433/mx_rag

## Run the full gate suite (what CI runs)
test:
	$(CARGO) nextest run --workspace --profile ci
	$(CARGO) test --workspace --doc

## Fast unit-only loop (no DB)
test-unit:
	$(CARGO) nextest run --workspace --lib --bins

## Integration with the local pgvector container
test-int:
	@docker start mx-rag-test 2>/dev/null || docker run -d --name mx-rag-test -p 55433:5432 -e POSTGRES_DB=mx_rag -e POSTGRES_HOST_AUTH_METHOD=trust pgvector/pgvector:pg17
	@sleep 2
	MX_RAG_TEST_DATABASE_URL=$(TEST_DB_URL) $(CARGO) nextest run --workspace --profile ci
	MX_RAG_TEST_DATABASE_URL=$(TEST_DB_URL) $(CARGO) test --workspace --doc

## Known-broken TDD lane (expected red; scoreboard)
# --profile ci: fail-fast off, so the scoreboard reports every lane test rather
# than cancelling at the first red.
test-known-broken:
	-MX_RAG_TEST_DATABASE_URL=$(TEST_DB_URL) $(CARGO) nextest run --workspace --profile ci --run-ignored only

## Coverage with ratchet check (BUMP=1 to raise the floor)
coverage:
	./scripts/coverage-ratchet.sh $(if $(BUMP),--bump,)

## E2E smoke: scaffold -> make dev -> router URL -> teardown (real Docker)
# E2E_RECIPES selects recipes (default rust-api); CI dispatches "rust-api laravel".
# Reuses an already-running mx-router and leaves it exactly as found.
test-e2e:
	./scripts/test-e2e.sh

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
	@echo "  make upgrade        Rebuild and reinstall to ~/.local/bin"
	@echo "  make uninstall      Remove installed binaries"
	@echo "  make init           Initialize templates (~/.mech-crate)"
	@echo ""
	@echo "Test:"
	@echo "  make test           Run the full gate suite (nextest + doc-tests)"
	@echo "  make test-unit      Fast unit-only loop (no DB)"
	@echo "  make test-int       Full suite against the local pgvector container"
	@echo "  make test-known-broken  Known-broken TDD lane (expected red)"
	@echo "  make coverage       Coverage ratchet (BUMP=1 raises the floor)"
	@echo "  make test-e2e       E2E smoke: scaffold -> router -> URL (real Docker)"
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
