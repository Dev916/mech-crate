# Development mode commands
#
# `s=` takes one service or a whitespace-separated list (`s="api site"`). The
# quotes below are load-bearing: unquoted, the second name becomes a make goal
# ("No rule to make target 'site'") and only the first service ever starts
# (bd:mech-crate-3kq).
.PHONY: dev _dev

dev: ## Start services in dev mode (s="[service ...]" for a subset)
	@$(MAKE) _dev service="$(call get_service_optional)"

_dev:
	@./scripts/dev.sh "$(service)"
