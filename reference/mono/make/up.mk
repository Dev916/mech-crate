.PHONY: up _up

# Start a specific service using (s=[service]) or all services. (ex. make up s=theblock.co)
up:
	@$(MAKE) _up service=$(call get_service_optional)

_up:
	@./scripts/up.sh $(service)