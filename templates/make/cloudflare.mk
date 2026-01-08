# ═══════════════════════════════════════════════════════════════════════════════
# Cloudflare Workers & Containers (Multi-App Support)
# ═══════════════════════════════════════════════════════════════════════════════

CF_ROOT := $(ROOT_DIR)/infra/cloudflare
CF_APPS_DIR := $(CF_ROOT)/apps
CF_ENV_FILE := $(CF_ROOT)/.env.cloudflare
CF_DOCKER_PLATFORM ?= linux/amd64

# App parameter (use: make cf-deploy a=myapp.co)
a ?=
APP_NAME := $(a)

# Load Cloudflare credentials if they exist
-include $(CF_ENV_FILE)
export CLOUDFLARE_ACCOUNT_ID ?= $(CF_ACCOUNT_ID)

# Computed paths for specific app
CF_APP_DIR = $(CF_APPS_DIR)/$(APP_NAME)
CF_APP_WRANGLER = $(CF_APP_DIR)/wrangler.toml
CF_APP_DOCKERFILE = $(ROOT_DIR)/docker/dockerfiles/$(APP_NAME)/app

# Version from app's package.json or fallback
APP_VERSION = $(shell node -p "require('$(CF_APP_DIR)/package.json').version" 2>/dev/null || echo "0.0.1")
CF_IMAGE_TAG = v$(APP_VERSION)

# Registry image path
CF_REGISTRY_IMAGE = registry.cloudflare.com/$(CF_ACCOUNT_ID)/$(APP_NAME)
CF_IMAGE_URI = $(CF_REGISTRY_IMAGE):$(CF_IMAGE_TAG)

# ═══════════════════════════════════════════════════════════════════════════════
# Setup & Configuration
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: cf-setup cf-login cf-whoami cf-status cf-init cf-list

cf-setup: ## Run Cloudflare setup wizard
	@./scripts/cf-setup.sh

cf-login: ## Login to Cloudflare
	@npx wrangler login && echo "✓ Logged in! Run 'make cf-setup' to save your account ID."

cf-whoami: ## Show Cloudflare auth status
	@npx wrangler whoami

cf-status: ## Show all Cloudflare apps status
	@echo "╭────────────────────────────────────────────────────────────╮"
	@echo "│  🌐 Cloudflare Apps Status                                 │"
	@echo "╰────────────────────────────────────────────────────────────╯"
	@echo ""
	@if [ -f "$(CF_ENV_FILE)" ]; then \
		echo "Account ID: $$(grep CF_ACCOUNT_ID $(CF_ENV_FILE) | cut -d= -f2)"; \
	else \
		echo "⚠  Not configured. Run 'make cf-setup' first."; \
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

cf-init: ## Init new CF app (a=app type=worker|cron|container)
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

cf-push: ## Push to CF registry (a=domain.com)
ifndef a
	$(error Usage: make cf-push a=myapp.co)
endif
	@echo "Pushing $(APP_NAME):$(CF_IMAGE_TAG) to Cloudflare..."
	@cd $(ROOT_DIR) && npx wrangler containers push $(APP_NAME):$(CF_IMAGE_TAG)

cf-publish: cf-build cf-push ## Build & push (a=domain.com)

cf-images: ## List CF registry images (a=domain.com)
ifndef a
	$(error Usage: make cf-images a=myapp.co)
endif
	@npx wrangler containers images list --filter $(APP_NAME)

cf-sync-image: ## Sync wrangler.toml image tag (a=domain.com)
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

cf-deploy-preview: cf-install ## Deploy to preview (a=domain.com)
ifndef a
	$(error Usage: make cf-deploy-preview a=myapp.co)
endif
	@echo "Deploying $(APP_NAME) to preview..."
	@cd $(ROOT_DIR) && npx wrangler deploy --config $(CF_APP_WRANGLER) --env preview

cf-deploy: ## Deploy to production (a=domain.com)
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

cf-deploy-all: ## Deploy ALL apps to production
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
