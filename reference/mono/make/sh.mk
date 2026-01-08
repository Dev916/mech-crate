.PHONY: sh _sh

# Shell into any given service (alias for `sh`)
bash:
	@$(MAKE) _sh service=$(call get_service)

# Shell into any given service
sh:
	@$(MAKE) _sh service=$(call get_service)

_sh:
	@./scripts/sh.sh $(service)