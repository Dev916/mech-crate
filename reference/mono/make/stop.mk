.PHONY: down _down

# Stop a specific service using (s=[service]) or all services. (ex. make stop s=launchpad)
stop:
	@$(MAKE) _stop service=$(call get_service_optional)

# Stop a specific service using (s=[service]) or all services. (ex. make stop s=launchpad)
_down:
	@./scripts/stop.sh $(service)