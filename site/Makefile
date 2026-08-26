# MechCrate Project Makefile
# 🦝 Crate Raccoon is ready to serve

.SILENT:
SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c
ROOT_DIR := $(abspath $(CURDIR))
PROJECT_NAME ?= $(notdir $(ROOT_DIR))
# MK_MODULES (not MAKEFILES - that's reserved in GNU Make)
MK_MODULES := $(wildcard make/*.mk)

# Key generation utilities
BYTES ?= 32
FORMAT ?= hex

.PHONY: make-key
make-key: ## Generate a secret key (BYTES=32, FORMAT=hex|base64|uuid)
	@case "$(FORMAT)" in \
		hex)    openssl rand -hex $(BYTES) ;; \
		base64) openssl rand -base64 $(BYTES) ;; \
		uuid)   uuidgen | tr '[:upper:]' '[:lower:]' ;; \
		*)      echo "Unknown format: $(FORMAT). Use hex, base64, or uuid" && exit 1 ;; \
	esac

# Include all make modules
ifneq ($(strip $(MK_MODULES)),)
include $(MK_MODULES)
endif

# ─────────────────────────────────────────────────────────────────────────────
# Core Targets
# ─────────────────────────────────────────────────────────────────────────────

.PHONY: help init test ps doctor

help: ## Show available commands
	@./scripts/help.sh

init: ## Initialize project environment
	@./scripts/init.sh

test: ## Run tests
	@./scripts/test.sh

ps: ## List running services
	@./scripts/ps.sh

doctor: ## Check project health
	@./scripts/doctor.sh

# ─────────────────────────────────────────────────────────────────────────────
# Astro site (apps/site) — local entry points for what site.yml runs in CI
# ─────────────────────────────────────────────────────────────────────────────
#
# These live here rather than in make/*.mk because make/ holds the modules the
# mx astro recipe generated (build.mk, dev.mk, …), which a future `mx add`
# refresh may rewrite. Project-specific targets belong in this file, alongside
# `make-key`.
#
# `build`/`test` above are the recipe's Docker-service targets, so these carry
# the `site-` prefix to avoid shadowing them. Both mirror
# `.github/workflows/site.yml` exactly — same scripts, same order — so a green
# `make site-test && make site-build` locally means a green build job in CI.

SITE_APP_DIR := $(ROOT_DIR)/apps/site

.PHONY: site-install site-test site-build

site-install: ## Install the Astro app's locked dependencies
	@cd $(SITE_APP_DIR) && PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 npm ci

site-test: ## Run the Astro app's vitest suite + diagram drift gate
	@[ -d "$(SITE_APP_DIR)/node_modules" ] || $(MAKE) site-install
	@cd $(SITE_APP_DIR) && npm run diagrams:check
	@cd $(SITE_APP_DIR) && npm test

site-build: ## Build the static site into apps/site/dist
	@[ -d "$(SITE_APP_DIR)/node_modules" ] || $(MAKE) site-install
	@cd $(SITE_APP_DIR) && npm run build
