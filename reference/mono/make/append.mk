.PHONY: append _append

# Append a service to the running dev stack in dev mode. Run this after starting other services in dev mode to add another service in dev mode. (ex. make append s=launchpad)
append:
	@$(MAKE) _append service=$(call get_service)

# Append a service to the running stack in dev mode
_append:
	@./scripts/append.sh $(service)