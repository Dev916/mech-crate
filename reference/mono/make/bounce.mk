.PHONY: bounce _bounce

# Start a specific service stack using (s=[service]) or all services. (ex. make dev s=launchpad)
bounce:
	@$(MAKE) _bounce service=$(call get_service_optional)

# Start a specific service stack using (s=[service]) or all services. (ex. make dev s=launchpad)
_bounce:
	@./scripts/bounce.sh $(service)