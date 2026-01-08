# Build commands
.PHONY: build _build

build: ## Build an image (s=[service] t=[tag])
	@$(MAKE) _build service=$(call get_service) tag=$(call get_tag)

_build:
	@./scripts/build.sh $(service) $(tag)
