.PHONY: help append init up dev down exec restart build rebuild test logs ps stop start import-data reboot rebuild port bash exec run deploy latest simon sh test inject

# Helper function to get the service parameter
get_service = $(if $(service),$(service),$(if $(s),$(s),$(error "Error: No service specified. Use 'service' or 's' parameter.")))

# Helper function to optionally get the service parameter
get_service_optional = $(if $(service),$(service),$(if $(s),$(s),$(info "Warning: No service specified.")))

# Helper function to get the command parameter
get_cmd = $(if $(cmd),$(cmd),$(if $(c),$(c),$(error "Error: No command specified. Use 'cmd' or 'c' parameter.")))

# Helper function to optionally get the tag parameter
get_tag = $(if $(tag),$(tag),$(if $(t),$(t),$(info "Info: no tag specified. Defaulting to 'latest'.")))
# Inject docker and scripts into the app context
.PHONY: inject
inject:
	@bash scripts/inject.sh $(call get_service)
