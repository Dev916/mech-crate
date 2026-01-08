.PHONY: pull _pull

# Pull the latest image for a service
pull:
	@$(MAKE) _pull service=$(call get_service)

_pull:
	@echo "Pulling service: $(service)"
	@./scripts/pull.sh $(service)