.PHONY: launch _launch

# Start a specific service stack using (s=[service]) or all services. (ex. make dev s=launchpad)
launch:
	@$(MAKE) _launch service=$(call get_service_optional)

# Start a specific service stack using (s=[service]) or all services. (ex. make dev s=launchpad)
_launch:
	@./scripts/launch.sh $(service)
