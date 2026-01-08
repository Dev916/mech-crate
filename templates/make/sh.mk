# Shell commands
.PHONY: sh bash _sh

# Shell into service (alias for sh, s=[service])
bash:
	@$(MAKE) _sh service=$(call get_service)

# Shell into a running service (s=[service])
sh:
	@$(MAKE) _sh service=$(call get_service)

_sh:
	@./scripts/sh.sh $(service)
