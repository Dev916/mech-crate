.PHONY: down _down

# Stop and remove a specific service using (s=[service]) or  all services (alias for make stop)
down:
	@$(MAKE) _down service=$(call get_service_optional)

# Stop and remove a specific service using (s=[service]) or  all services (alias for make stop)
_down:
	@./scripts/down.sh $(service)