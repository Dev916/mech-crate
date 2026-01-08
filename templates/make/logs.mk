# Log commands
.PHONY: logs _logs

# Tail service logs (s=[service] for specific)
logs:
	@$(MAKE) _logs service=$(call get_service_optional)

_logs:
	@./scripts/logs.sh $(service)
