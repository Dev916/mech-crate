# Stop commands
.PHONY: stop _stop

stop: ## Stop services without removing (s=[service] for specific)
	@$(MAKE) _stop service=$(call get_service_optional)

_stop:
	@./scripts/stop.sh $(service)
