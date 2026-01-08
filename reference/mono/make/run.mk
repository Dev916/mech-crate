.PHONY: run _run

# Run a one-off command in a service. (make s=[service], cmd=[command]) (ex. make run s=connect cmd=import-data)
run:
	@$(MAKE) _run service=$(call get_service) cmd="$(call get_cmd)" d=$(d)

_run:
	@./scripts/run.sh "$(service)" "$(cmd)" $(d)