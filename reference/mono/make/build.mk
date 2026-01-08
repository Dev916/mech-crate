.PHONY: build

# Build an image for a given service s=[service]
build:
	@$(MAKE) _build service=$(call get_service) tag=$(call get_tag)

# Build an image for a given service service=[service]
_build:
	@./scripts/build.sh $(service) $(tag)
