# Restart commands
.PHONY: restart _restart

# Restart a service (s=[service] required)
restart:
	@$(MAKE) _restart service=$(call get_service)

_restart:
	@./scripts/restart.sh $(service)
