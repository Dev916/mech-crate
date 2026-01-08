# Shell commands
.PHONY: sh bash _sh

bash: ## Shell into service (alias for sh, s=[service])
	@$(MAKE) _sh service=$(call get_service)

sh: ## Shell into a running service (s=[service])
	@$(MAKE) _sh service=$(call get_service)

_sh:
	@./scripts/sh.sh $(service)
