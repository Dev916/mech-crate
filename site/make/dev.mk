# Development mode commands
.PHONY: dev _dev

dev: ## Start services in dev mode (s=[service] for specific)
	@$(MAKE) _dev service=$(call get_service_optional)

_dev:
	@./scripts/dev.sh $(service)
