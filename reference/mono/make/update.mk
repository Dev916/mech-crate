.PHONY: update

# Update specific services specified in tmp/start.txt
update:
	@$(MAKE) _update service=$(call get_service_optional)

_update:
	@./scripts/update.sh $(service) 