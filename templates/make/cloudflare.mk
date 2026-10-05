# ═══════════════════════════════════════════════════════════════════════════════
# Cloudflare Workers & Containers (Multi-App Support)
# ═══════════════════════════════════════════════════════════════════════════════

CF_ROOT := $(ROOT_DIR)/infra/cloudflare
CF_APPS_DIR := $(CF_ROOT)/apps
CF_ENV_FILE := $(CF_ROOT)/.env.cloudflare
MX_HOME ?= $(HOME)/.mech-crate
CF_GLOBAL_ENV_FILE := $(MX_HOME)/config/infra/cloudflare.env
CF_DOCKER_PLATFORM ?= linux/amd64

# App parameter (use: make cf-deploy a=myapp.co)
a ?=
APP_NAME := $(a)

# ═══════════════════════════════════════════════════════════════════════════════
# Credential resolution
# ═══════════════════════════════════════════════════════════════════════════════
#
# One canonical pair: CLOUDFLARE_ACCOUNT_ID + CLOUDFLARE_API_TOKEN. Those are
# the two names wrangler itself reads from the environment, so the value a
# credentials file carries reaches the deploy with no translation step.
# CF_ACCOUNT_ID / CF_API_TOKEN are deprecated aliases, still read so that an
# `.env.cloudflare` written by an older `make cf-setup` keeps working.
#
# Two credentials files, two scopes:
#   global  $(MX_HOME)/config/infra/cloudflare.env   <- mx infra setup cloudflare
#   project infra/cloudflare/.env.cloudflare         <- make cf-setup
#
# Precedence, highest first: the environment (or the make command line), then
# the project file, then the global file. Each scope is captured on its own so a
# project file carrying only the deprecated alias still beats a global file
# carrying the canonical name.

CF_ID_FROM_ENV := $(CLOUDFLARE_ACCOUNT_ID)
CF_ID_FROM_ENV_ALIAS := $(CF_ACCOUNT_ID)
CF_TOKEN_FROM_ENV := $(CLOUDFLARE_API_TOKEN)
CF_TOKEN_FROM_ENV_ALIAS := $(CF_API_TOKEN)

CLOUDFLARE_ACCOUNT_ID :=
CF_ACCOUNT_ID :=
CLOUDFLARE_API_TOKEN :=
CF_API_TOKEN :=
-include $(CF_GLOBAL_ENV_FILE)
CF_ID_FROM_GLOBAL := $(CLOUDFLARE_ACCOUNT_ID)
CF_ID_FROM_GLOBAL_ALIAS := $(CF_ACCOUNT_ID)
CF_TOKEN_FROM_GLOBAL := $(CLOUDFLARE_API_TOKEN)
CF_TOKEN_FROM_GLOBAL_ALIAS := $(CF_API_TOKEN)

CLOUDFLARE_ACCOUNT_ID :=
CF_ACCOUNT_ID :=
CLOUDFLARE_API_TOKEN :=
CF_API_TOKEN :=
-include $(CF_ENV_FILE)
CF_ID_FROM_PROJECT := $(CLOUDFLARE_ACCOUNT_ID)
CF_ID_FROM_PROJECT_ALIAS := $(CF_ACCOUNT_ID)
CF_TOKEN_FROM_PROJECT := $(CLOUDFLARE_API_TOKEN)
CF_TOKEN_FROM_PROJECT_ALIAS := $(CF_API_TOKEN)

export CLOUDFLARE_ACCOUNT_ID := $(strip $(or \
	$(CF_ID_FROM_ENV),$(CF_ID_FROM_ENV_ALIAS), \
	$(CF_ID_FROM_PROJECT),$(CF_ID_FROM_PROJECT_ALIAS), \
	$(CF_ID_FROM_GLOBAL),$(CF_ID_FROM_GLOBAL_ALIAS)))
export CLOUDFLARE_API_TOKEN := $(strip $(or \
	$(CF_TOKEN_FROM_ENV),$(CF_TOKEN_FROM_ENV_ALIAS), \
	$(CF_TOKEN_FROM_PROJECT),$(CF_TOKEN_FROM_PROJECT_ALIAS), \
	$(CF_TOKEN_FROM_GLOBAL),$(CF_TOKEN_FROM_GLOBAL_ALIAS)))

# Which scope supplied the account id, for cf-vars / cf-status / cf-doctor.
CF_ACCOUNT_ID_SOURCE := $(strip \
	$(if $(or $(CF_ID_FROM_ENV),$(CF_ID_FROM_ENV_ALIAS)),environment, \
	$(if $(or $(CF_ID_FROM_PROJECT),$(CF_ID_FROM_PROJECT_ALIAS)),project, \
	$(if $(or $(CF_ID_FROM_GLOBAL),$(CF_ID_FROM_GLOBAL_ALIAS)),global,none))))

# Credentials files still spelling the account id the deprecated way.
CF_DEPRECATED_ALIAS_FILES := $(strip \
	$(if $(CF_ID_FROM_PROJECT_ALIAS),$(CF_ENV_FILE),) \
	$(if $(CF_ID_FROM_GLOBAL_ALIAS),$(CF_GLOBAL_ENV_FILE),))

# Computed paths for specific app
CF_APP_DIR = $(CF_APPS_DIR)/$(APP_NAME)
CF_APP_WRANGLER = $(CF_APP_DIR)/wrangler.toml
CF_APP_DOCKERFILE = $(ROOT_DIR)/docker/dockerfiles/$(APP_NAME)/app

# Version from app's package.json or fallback
APP_VERSION = $(shell node -p "require('$(CF_APP_DIR)/package.json').version" 2>/dev/null || echo "0.0.1")
CF_IMAGE_TAG = v$(APP_VERSION)

# Registry image path
CF_REGISTRY_IMAGE = registry.cloudflare.com/$(CLOUDFLARE_ACCOUNT_ID)/$(APP_NAME)
CF_IMAGE_URI = $(CF_REGISTRY_IMAGE):$(CF_IMAGE_TAG)

# ═══════════════════════════════════════════════════════════════════════════════
# Setup & Configuration
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: cf-setup cf-login cf-whoami cf-status cf-init cf-list cf-vars cf-check-credentials

cf-setup: ## Run Cloudflare setup wizard
	@./scripts/cf-setup.sh

cf-vars: ## Show the resolved Cloudflare credential variables (no secrets)
	@echo "CLOUDFLARE_ACCOUNT_ID=$(CLOUDFLARE_ACCOUNT_ID)"
	@echo "CLOUDFLARE_ACCOUNT_ID_SOURCE=$(CF_ACCOUNT_ID_SOURCE)"
	@echo "CLOUDFLARE_API_TOKEN=$(if $(CLOUDFLARE_API_TOKEN),***configured***,)"
	@echo "CF_ENV_FILE=$(CF_ENV_FILE)"
	@echo "CF_GLOBAL_ENV_FILE=$(CF_GLOBAL_ENV_FILE)"
	@$(if $(CF_DEPRECATED_ALIAS_FILES),echo "DEPRECATED_CF_ACCOUNT_ID_IN=$(CF_DEPRECATED_ALIAS_FILES)",true)

cf-check-credentials: ## Fail early when no Cloudflare account id resolves
ifeq ($(strip $(CLOUDFLARE_ACCOUNT_ID)),)
	@echo "✗ No Cloudflare account id found." >&2
	@echo "" >&2
	@echo "  Looked for CLOUDFLARE_ACCOUNT_ID (deprecated alias CF_ACCOUNT_ID) in:" >&2
	@echo "    project: $(CF_ENV_FILE)" >&2
	@echo "    global:  $(CF_GLOBAL_ENV_FILE)" >&2
	@echo "" >&2
	@echo "  Configure credentials with either:" >&2
	@echo "    mx infra setup cloudflare   # global config, shared by every project" >&2
	@echo "    make cf-setup               # this project's config, which wins over the global one" >&2
	@exit 1
else
	@echo "✓ Cloudflare account id resolved from the $(CF_ACCOUNT_ID_SOURCE) config"
	@$(if $(CF_DEPRECATED_ALIAS_FILES),echo "⚠ CF_ACCOUNT_ID is deprecated; rename it to CLOUDFLARE_ACCOUNT_ID in: $(CF_DEPRECATED_ALIAS_FILES)",true)
endif

cf-login: ## Login to Cloudflare
	@npx wrangler login && echo "✓ Logged in! Run 'make cf-setup' to save your account ID."

cf-whoami: ## Show Cloudflare auth status
	@npx wrangler whoami

cf-status: ## Show all Cloudflare apps status
	@echo "╭────────────────────────────────────────────────────────────╮"
	@echo "│  🌐 Cloudflare Apps Status                                 │"
	@echo "╰────────────────────────────────────────────────────────────╯"
	@echo ""
	@if [ -n "$(CLOUDFLARE_ACCOUNT_ID)" ]; then \
		echo "Account ID: $(CLOUDFLARE_ACCOUNT_ID) (from the $(CF_ACCOUNT_ID_SOURCE) config)"; \
	else \
		echo "⚠  Not configured. Run 'mx infra setup cloudflare' or 'make cf-setup' first."; \
	fi
	@echo ""
	@echo "Configured Apps:"
	@if [ -d "$(CF_APPS_DIR)" ]; then \
		for app in $(CF_APPS_DIR)/*/; do \
			if [ -f "$$app/wrangler.toml" ]; then \
				name=$$(basename $$app); \
				version=$$(node -p "require('$$app/package.json').version" 2>/dev/null || echo "0.0.1"); \
				echo "  • $$name (v$$version)"; \
			fi; \
		done; \
	else \
		echo "  (none)"; \
	fi
	@echo ""

cf-init: cf-check-credentials ## Init new CF app (a=app type=worker|cron|container)
ifndef a
	$(error Usage: make cf-init a=myapp [type=worker|cron|container])
endif
ifdef type
	@./scripts/cf-init-app.sh "$(APP_NAME)" --type=$(type)
else
	@./scripts/cf-init-app.sh "$(APP_NAME)"
endif

cf-list: ## List all CF apps
	@echo "Configured Cloudflare Apps:"
	@if [ -d "$(CF_APPS_DIR)" ]; then \
		ls -1 $(CF_APPS_DIR) 2>/dev/null | while read app; do \
			if [ -f "$(CF_APPS_DIR)/$$app/wrangler.toml" ]; then \
				echo "  • $$app"; \
			fi; \
		done; \
	else \
		echo "  (none - run 'make cf-init a=myapp.co' to create one)"; \
	fi

# ═══════════════════════════════════════════════════════════════════════════════
# Container Build & Push
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: cf-build cf-push cf-publish cf-images cf-sync-image

cf-build: ## Build container (a=domain.com)
ifndef a
	$(error Usage: make cf-build a=myapp.co)
endif
	@echo "Building $(APP_NAME):$(CF_IMAGE_TAG)..."
	@docker buildx build \
		--platform $(CF_DOCKER_PLATFORM) \
		-f $(CF_APP_DOCKERFILE) \
		-t $(APP_NAME):$(CF_IMAGE_TAG) \
		--build-arg APP_VERSION=$(APP_VERSION) \
		--load \
		$(ROOT_DIR)

cf-push: cf-check-credentials ## Push to CF registry (a=domain.com)
ifndef a
	$(error Usage: make cf-push a=myapp.co)
endif
	@echo "Pushing $(APP_NAME):$(CF_IMAGE_TAG) to Cloudflare..."
	@cd $(ROOT_DIR) && npx wrangler containers push $(APP_NAME):$(CF_IMAGE_TAG)

cf-publish: cf-build cf-push ## Build & push (a=domain.com)

cf-images: cf-check-credentials ## List CF registry images (a=domain.com)
ifndef a
	$(error Usage: make cf-images a=myapp.co)
endif
	@npx wrangler containers images list --filter $(APP_NAME)

cf-sync-image: cf-check-credentials ## Sync wrangler.toml image tag (a=domain.com)
ifndef a
	$(error Usage: make cf-sync-image a=myapp.co)
endif
	@echo "Syncing $(APP_NAME) wrangler.toml to $(CF_IMAGE_URI)..."
	@perl -0pi -e 's!image = "[^"]+"!image = "$(CF_IMAGE_URI)"!g' $(CF_APP_WRANGLER)

# ═══════════════════════════════════════════════════════════════════════════════
# Worker Development & Deployment
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: cf-install cf-dev cf-deploy-preview cf-deploy cf-deploy-all

cf-install: ## Install worker deps (a=domain.com)
ifndef a
	$(error Usage: make cf-install a=myapp.co)
endif
	@cd $(CF_APP_DIR) && npm install

cf-dev: cf-install ## Run worker locally (a=domain.com)
ifndef a
	$(error Usage: make cf-dev a=myapp.co)
endif
	@cd $(CF_APP_DIR) && npm run dev

cf-deploy-preview: cf-check-credentials cf-install ## Deploy to preview (a=domain.com)
ifndef a
	$(error Usage: make cf-deploy-preview a=myapp.co)
endif
	@echo "Deploying $(APP_NAME) to preview..."
	@cd $(ROOT_DIR) && npx wrangler deploy --config $(CF_APP_WRANGLER) --env preview

cf-deploy: cf-check-credentials ## Deploy to production (a=domain.com)
ifndef a
	$(error Usage: make cf-deploy a=myapp.co)
endif
	@echo "╭────────────────────────────────────────────────────────────╮"
	@echo "│  🚀 Deploying $(APP_NAME) to Cloudflare                    │"
	@echo "╰────────────────────────────────────────────────────────────╯"
	@$(MAKE) cf-publish a=$(APP_NAME)
	@$(MAKE) cf-sync-image a=$(APP_NAME)
	@$(MAKE) cf-install a=$(APP_NAME)
	@cd $(ROOT_DIR) && npx wrangler deploy --config $(CF_APP_WRANGLER) --env production
	@echo ""
	@echo "✓ $(APP_NAME) deployed successfully!"

cf-deploy-all: cf-check-credentials ## Deploy ALL apps to production
	@echo "╭────────────────────────────────────────────────────────────╮"
	@echo "│  🚀 Deploying ALL apps to Cloudflare                       │"
	@echo "╰────────────────────────────────────────────────────────────╯"
	@for app in $(CF_APPS_DIR)/*/; do \
		if [ -f "$$app/wrangler.toml" ]; then \
			name=$$(basename $$app); \
			echo ""; \
			echo "→ Deploying $$name..."; \
			$(MAKE) cf-deploy a=$$name || exit 1; \
		fi; \
	done
	@echo ""
	@echo "✓ All apps deployed!"

# ═══════════════════════════════════════════════════════════════════════════════
# Logs & Monitoring
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: cf-logs cf-logs-preview cf-restart cf-container-status

cf-logs: ## Tail worker logs (a=domain.com)
ifndef a
	$(error Usage: make cf-logs a=myapp.co)
endif
	@npx wrangler tail $(APP_NAME)-worker --env production

cf-logs-preview: ## Tail preview logs (a=domain.com)
ifndef a
	$(error Usage: make cf-logs-preview a=myapp.co)
endif
	@npx wrangler tail $(APP_NAME)-worker --env preview

cf-restart: ## Restart container (a=domain.com)
ifndef a
	$(error Usage: make cf-restart a=myapp.co)
endif
	@echo "Restarting $(APP_NAME) container..."
	@curl -X POST "https://$(APP_NAME)/_container/restart" 2>/dev/null || \
		echo "Note: Restart endpoint may not be accessible. Deploy again to restart."

cf-container-status: ## Check container status (a=domain.com)
ifndef a
	$(error Usage: make cf-container-status a=myapp.co)
endif
	@curl -s "https://$(APP_NAME)/_container/status" | jq . 2>/dev/null || \
		curl -s "https://$(APP_NAME)/_container/status"
