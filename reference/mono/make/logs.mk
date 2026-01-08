.PHONY: logs _logs

# Tail logs of a service using (s=[service]) or all services
logs:
	@$(MAKE) _logs service=$(call get_service_optional)

# Tail logs of a service using (s=[service]) or all services
_logs:
	@./scripts/logs.sh $(service)