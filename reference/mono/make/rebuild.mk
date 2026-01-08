.PHONY: rebuild _rebuild

# Rebuild main app image based off the latest app image. This is useful when you just need to copy over the latest files without a full rebuild. (make rebuild s=connect)
rebuild:
	@$(MAKE) _rebuild service=$(call get_service)

# Rebuild main app image based off the latest app image
_rebuild:
	@./scripts/rebuild.sh $(service)