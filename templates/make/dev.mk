# Development mode commands
.PHONY: dev _dev

# Start services in dev mode (s=[service] for specific)
dev:
	@$(MAKE) _dev service=$(call get_service_optional)

_dev:
	@./scripts/dev.sh $(service)
