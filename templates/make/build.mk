# Build commands
.PHONY: build _build

# Build an image for a given service (s=[service] t=[tag])
build:
	@$(MAKE) _build service=$(call get_service) tag=$(call get_tag)

_build:
	@./scripts/build.sh $(service) $(tag)
