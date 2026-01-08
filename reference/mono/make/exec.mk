.PHONY: exec _exec

# Execute a command in a running service. (make s=[service], cmd=[command]) (ex. make exec s=launchpad cmd=ls)
exec:
	@$(MAKE) _exec service=$(call get_service) cmd="$(call get_cmd)" d=$(d)

_exec:
	@./scripts/exec.sh "$(service)" "$(cmd)" $(d)
