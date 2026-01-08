# Service down commands
.PHONY: down _down

# Stop and remove services (s=[service] for specific)
down:
	@$(MAKE) _down service=$(call get_service_optional)

_down:
	@./scripts/down.sh $(service)
