# Restart commands
.PHONY: restart _restart

restart: ## Restart services (s="[service ...]" required)
	@$(MAKE) _restart service="$(call get_service)"

_restart:
	@./scripts/restart.sh "$(service)"
