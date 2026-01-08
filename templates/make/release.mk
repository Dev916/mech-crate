# ═══════════════════════════════════════════════════════════════════════════════
# Release Workflow - App-Specific Releases
# ═══════════════════════════════════════════════════════════════════════════════
#
# Usage:
#   make release app=myapp           - Auto-determine version from commits
#   make release-patch app=myapp     - Force patch bump (x.y.Z)
#   make release-minor app=myapp     - Force minor bump (x.Y.0)
#   make release-major app=myapp     - Force major bump (X.0.0)
#   make release-dry app=myapp       - Preview without making changes
#   make release-full app=myapp      - Create and push in one command
#   make release-simple app=myapp    - Simple release (no conventional commits)
#
# This workflow:
#   1. Runs standard-version to bump version + generate CHANGELOG
#   2. Syncs version to manifest files
#   3. Copies CHANGELOG to public folder for the releases page
#   4. Creates git commit and tag
#
# ═══════════════════════════════════════════════════════════════════════════════

# Helper to get app parameter (required for release commands)
get_app = $(if $(app),$(app),$(if $(a),$(a),$(error "Error: No app specified. Use 'app' or 'a' parameter (e.g., app=myapp)")))

# Paths
RELEASE_SYNC_SCRIPT := $(ROOT_DIR)/scripts/release-sync-versions.mjs
SIMPLE_RELEASE_SCRIPT := $(ROOT_DIR)/scripts/simple-release.sh
APP_VERSION_SCRIPT := $(ROOT_DIR)/scripts/app-version.mjs

# ─────────────────────────────────────────────────────────────────────────────
# Post-release hook: sync versions and copy changelog
# ─────────────────────────────────────────────────────────────────────────────
define post_release
	@echo "📋 Syncing versions across manifest files..."
	@node $(RELEASE_SYNC_SCRIPT) --app $(1) 2>/dev/null || true
	@echo "📄 Copying CHANGELOG.md to public folder..."
	@if [ -f "$(ROOT_DIR)/apps/$(1)/CHANGELOG.md" ] && [ -d "$(ROOT_DIR)/apps/$(1)/public" ]; then \
		cp "$(ROOT_DIR)/apps/$(1)/CHANGELOG.md" "$(ROOT_DIR)/apps/$(1)/public/CHANGELOG.md"; \
	fi
	@git add "$(ROOT_DIR)/apps/$(1)/public/CHANGELOG.md" .release-please-manifest.json 2>/dev/null || true
	@git commit --amend --no-edit 2>/dev/null || true
endef

# ─────────────────────────────────────────────────────────────────────────────
# Main Release Commands (standard-version based)
# ─────────────────────────────────────────────────────────────────────────────

.PHONY: release
release: ## Create a new release (auto-determines version bump from commits) app=[app]
	$(eval APP := $(call get_app))
	@echo "🚀 Creating release for $(APP)..."
	@cd $(ROOT_DIR)/apps/$(APP) && yarn release
	$(call post_release,$(APP))
	@echo ""
	@echo "✅ Release created for $(APP)!"
	@git push --follow-tags origin main
	@echo "✅ Release pushed!"

.PHONY: release-patch
release-patch: ## Create a patch release (x.y.Z) app=[app]
	$(eval APP := $(call get_app))
	@echo "🚀 Creating patch release for $(APP)..."
	@cd $(ROOT_DIR)/apps/$(APP) && yarn release:patch
	$(call post_release,$(APP))
	@echo "✅ Patch release created for $(APP)!"

.PHONY: release-minor
release-minor: ## Create a minor release (x.Y.0) app=[app]
	$(eval APP := $(call get_app))
	@echo "🚀 Creating minor release for $(APP)..."
	@cd $(ROOT_DIR)/apps/$(APP) && yarn release:minor
	$(call post_release,$(APP))
	@echo "✅ Minor release created for $(APP)!"

.PHONY: release-major
release-major: ## Create a major release (X.0.0) app=[app]
	$(eval APP := $(call get_app))
	@echo "🚀 Creating major release for $(APP)..."
	@cd $(ROOT_DIR)/apps/$(APP) && yarn release:major
	$(call post_release,$(APP))
	@echo "✅ Major release created for $(APP)!"

.PHONY: release-dry
release-dry: ## Preview what the next release would look like (no changes) app=[app]
	$(eval APP := $(call get_app))
	@echo "👀 Dry run - previewing next release for $(APP)..."
	@cd $(ROOT_DIR)/apps/$(APP) && yarn release:dry

.PHONY: release-first
release-first: ## Create the first release (use when starting fresh) app=[app]
	$(eval APP := $(call get_app))
	@echo "🚀 Creating first release for $(APP)..."
	@cd $(ROOT_DIR)/apps/$(APP) && yarn release:first
	$(call post_release,$(APP))
	@echo "✅ First release created for $(APP)!"

# ─────────────────────────────────────────────────────────────────────────────
# Simple Release (no conventional commits required)
# ─────────────────────────────────────────────────────────────────────────────

.PHONY: release-simple
release-simple: ## Simple release - just bumps version and tags (default: patch) app=[app] type=[patch|minor|major]
	$(eval APP := $(call get_app))
	$(eval TYPE := $(if $(type),$(type),patch))
	@echo "🚀 Running simple release for $(APP) ($(TYPE))..."
	@$(SIMPLE_RELEASE_SCRIPT) $(TYPE) $(APP)

.PHONY: release-simple-patch
release-simple-patch: ## Simple patch release app=[app]
	$(eval APP := $(call get_app))
	@$(SIMPLE_RELEASE_SCRIPT) patch $(APP)

.PHONY: release-simple-minor
release-simple-minor: ## Simple minor release app=[app]
	$(eval APP := $(call get_app))
	@$(SIMPLE_RELEASE_SCRIPT) minor $(APP)

.PHONY: release-simple-major
release-simple-major: ## Simple major release app=[app]
	$(eval APP := $(call get_app))
	@$(SIMPLE_RELEASE_SCRIPT) major $(APP)

# ─────────────────────────────────────────────────────────────────────────────
# Push & Combined Commands
# ─────────────────────────────────────────────────────────────────────────────

.PHONY: release-push
release-push: ## Push release commit and tags to origin
	@echo "📤 Pushing release to origin..."
	@git push --follow-tags origin main
	@echo "✅ Release pushed!"

.PHONY: release-full
release-full: ## Create and push a release in one command app=[app]
	$(eval APP := $(call get_app))
	@$(MAKE) release app=$(APP)
	@echo "🎉 Release complete and pushed!"

.PHONY: release-patch-push
release-patch-push: ## Create and push patch release app=[app]
	$(eval APP := $(call get_app))
	@$(MAKE) release-patch app=$(APP)
	@$(MAKE) release-push
	@echo "🎉 Patch release complete and pushed!"

.PHONY: release-minor-push
release-minor-push: ## Create and push minor release app=[app]
	$(eval APP := $(call get_app))
	@$(MAKE) release-minor app=$(APP)
	@$(MAKE) release-push
	@echo "🎉 Minor release complete and pushed!"

.PHONY: release-major-push
release-major-push: ## Create and push major release app=[app]
	$(eval APP := $(call get_app))
	@$(MAKE) release-major app=$(APP)
	@$(MAKE) release-push
	@echo "🎉 Major release complete and pushed!"

# ─────────────────────────────────────────────────────────────────────────────
# Utility Commands
# ─────────────────────────────────────────────────────────────────────────────

.PHONY: release-sync
release-sync: ## Sync manifest versions to match package.json app=[app]
	$(eval APP := $(call get_app))
	@echo "🔄 Syncing versions for $(APP)..."
	@node $(RELEASE_SYNC_SCRIPT) --app $(APP)
	@if [ -f "$(ROOT_DIR)/apps/$(APP)/CHANGELOG.md" ] && [ -d "$(ROOT_DIR)/apps/$(APP)/public" ]; then \
		cp "$(ROOT_DIR)/apps/$(APP)/CHANGELOG.md" "$(ROOT_DIR)/apps/$(APP)/public/CHANGELOG.md"; \
	fi
	@echo "✅ Versions synced for $(APP)!"

.PHONY: release-changelog
release-changelog: ## Copy CHANGELOG to public folder (for manual updates) app=[app]
	$(eval APP := $(call get_app))
	@if [ -f "$(ROOT_DIR)/apps/$(APP)/CHANGELOG.md" ] && [ -d "$(ROOT_DIR)/apps/$(APP)/public" ]; then \
		cp "$(ROOT_DIR)/apps/$(APP)/CHANGELOG.md" "$(ROOT_DIR)/apps/$(APP)/public/CHANGELOG.md"; \
		echo "✅ CHANGELOG copied to public folder for $(APP)"; \
	else \
		echo "⚠️  CHANGELOG.md or public folder not found for $(APP)"; \
	fi

.PHONY: release-version
release-version: ## Show current version for an app app=[app]
	$(eval APP := $(call get_app))
	@node $(APP_VERSION_SCRIPT) --app $(APP)

.PHONY: release-list-apps
release-list-apps: ## List all apps available for release
	@echo "📦 Available apps for release:"
	@for dir in $(ROOT_DIR)/apps/*/; do \
		if [ -f "$$dir/package.json" ]; then \
			app=$$(basename "$$dir"); \
			version=$$(node -p "require('$$dir/package.json').version" 2>/dev/null || echo "unknown"); \
			echo "  - $$app (v$$version)"; \
		fi \
	done
