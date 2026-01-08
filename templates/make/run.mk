# Run commands
.PHONY: run _run exec _exec

# Run command in new container (s=[service] c=[cmd])
run:
	@$(MAKE) _run service=$(call get_service) cmd=$(call get_cmd)

_run:
	@./scripts/run.sh $(service) $(cmd)

# Exec command in running container (s=[service] c=[cmd])
exec:
	@$(MAKE) _exec service=$(call get_service) cmd=$(call get_cmd)

_exec:
	@./scripts/exec.sh $(service) $(cmd)
