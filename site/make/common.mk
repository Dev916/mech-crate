# MechCrate Common Make Helpers
# Shared functions and variables for all make modules

.PHONY: help init up dev down exec restart build rebuild test logs ps stop start sh

# Helper function to get the service parameter (required)
# Usage: $(call get_service) - errors if not provided
get_service = $(if $(service),$(service),$(if $(s),$(s),$(error "Error: No service specified. Use 'service' or 's' parameter.")))

# Helper function to optionally get the service parameter
# Usage: $(call get_service_optional) - warns if not provided
get_service_optional = $(if $(service),$(service),$(if $(s),$(s),$(info "Info: No service specified. Operating on all services.")))

# Helper function to get the command parameter (required)
# Usage: $(call get_cmd)
get_cmd = $(if $(cmd),$(cmd),$(if $(c),$(c),$(error "Error: No command specified. Use 'cmd' or 'c' parameter.")))

# Helper function to optionally get the tag parameter
# Usage: $(call get_tag) - defaults to 'latest'
get_tag = $(if $(tag),$(tag),$(if $(t),$(t),latest))

# Project name (derived from directory name)
PROJECT_NAME ?= $(notdir $(CURDIR))

# Docker compose command
COMPOSE := docker compose

# Default network name
NETWORK_NAME ?= mech-network
