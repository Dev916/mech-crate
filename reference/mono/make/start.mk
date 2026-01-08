.PHONY: start


# Start specific services specified in tmp/start.txt
start:
	@$(MAKE) _start service=$(call get_service_optional)

_start:
	@./scripts/start.sh $(service)