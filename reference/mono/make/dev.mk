.PHONY: dev _dev

# Start a specific service stack using (s=[service]) or all services. (ex. make dev s=launchpad)
dev:
	@$(MAKE) _dev service=$(call get_service_optional)

# Start a specific service stack using (s=[service]) or all services. (ex. make dev s=launchpad)
_dev:
	@./scripts/dev.sh $(service)