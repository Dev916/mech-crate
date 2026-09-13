# Service up commands (production mode)
.PHONY: up _up

up: ## Start services in production mode (s="[service ...]" for a subset)
	@$(MAKE) _up service="$(call get_service_optional)"

_up:
	@./scripts/up.sh "$(service)"
