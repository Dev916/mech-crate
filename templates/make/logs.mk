# Log commands
.PHONY: logs _logs

logs: ## Tail service logs (s="[service ...]" for a subset)
	@$(MAKE) _logs service="$(call get_service_optional)"

_logs:
	@./scripts/logs.sh "$(service)"
