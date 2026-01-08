.PHONY: restart _restart

# Restart a specific service using (s=[service]) or all services (make restart s=launchpad)
restart:
	@$(MAKE) _restart service=$(call get_service)

# Restart a specific service using (s=[service]) or all services
_restart:
	@./scripts/restart.sh $(service)