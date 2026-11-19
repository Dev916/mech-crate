# ═══════════════════════════════════════════════════════════════════════════════
# AWS: local emulator lane + sandbox-account gate
# ═══════════════════════════════════════════════════════════════════════════════
#
# Two lanes that must never mix:
#
#   emulator  aws-up / aws-env / aws-cli ...   fake credentials, local endpoint
#   sandbox   aws-check-sandbox / aws-sandbox  real credentials, real AWS
#
# The emulator targets force the fake credentials and drop AWS_PROFILE. The
# sandbox targets refuse to run while AWS_ENDPOINT_URL is set and refuse any
# account other than AWS_SANDBOX_ACCOUNT_ID.

AWS_ROOT := $(ROOT_DIR)/infra/aws
AWS_COMPOSE_FILE := $(AWS_ROOT)/compose.yml
AWS_ENV_FILE := $(AWS_ROOT)/.env.aws
MX_HOME ?= $(HOME)/.mech-crate
AWS_GLOBAL_ENV_FILE := $(MX_HOME)/config/infra/aws.env

# Emulator endpoint as the workstation sees it (through the mx router).
AWS_ROUTER_HOST ?= aws.$(PROJECT_NAME).localhost
AWS_EMULATOR_URL ?= http://$(AWS_ROUTER_HOST)
AWS_EMULATOR_ACCOUNT ?= 000000000000
AWS_EMULATOR_REGION ?= us-east-1

# ═══════════════════════════════════════════════════════════════════════════════
# Sandbox credential resolution
# ═══════════════════════════════════════════════════════════════════════════════
#
# Same scopes and precedence as make/cloudflare.mk: the environment, then the
# project file infra/aws/.env.aws, then the global file written by
# `mx infra setup aws`. A project file carrying MX_INFRA_USE_GLOBAL=true
# (`mx infra link aws`) is linked: its values are ignored.

AWS_ENDPOINT_FROM_ENV := $(AWS_ENDPOINT_URL)
AWS_KEY_FROM_ENV := $(AWS_ACCESS_KEY_ID)
AWS_SECRET_FROM_ENV := $(AWS_SECRET_ACCESS_KEY)
AWS_REGION_FROM_ENV := $(AWS_REGION)
AWS_SANDBOX_FROM_ENV := $(AWS_SANDBOX_ACCOUNT_ID)

AWS_ACCESS_KEY_ID :=
AWS_SECRET_ACCESS_KEY :=
AWS_REGION :=
AWS_SANDBOX_ACCOUNT_ID :=
-include $(AWS_GLOBAL_ENV_FILE)
AWS_KEY_FROM_GLOBAL := $(AWS_ACCESS_KEY_ID)
AWS_SECRET_FROM_GLOBAL := $(AWS_SECRET_ACCESS_KEY)
AWS_REGION_FROM_GLOBAL := $(AWS_REGION)
AWS_SANDBOX_FROM_GLOBAL := $(AWS_SANDBOX_ACCOUNT_ID)

AWS_ACCESS_KEY_ID :=
AWS_SECRET_ACCESS_KEY :=
AWS_REGION :=
AWS_SANDBOX_ACCOUNT_ID :=
MX_INFRA_USE_GLOBAL :=
-include $(AWS_ENV_FILE)
AWS_PROJECT_LINKED := $(if $(filter true,$(strip $(MX_INFRA_USE_GLOBAL))),true,false)
ifeq ($(AWS_PROJECT_LINKED),true)
AWS_KEY_FROM_PROJECT :=
AWS_SECRET_FROM_PROJECT :=
AWS_REGION_FROM_PROJECT :=
AWS_SANDBOX_FROM_PROJECT :=
else
AWS_KEY_FROM_PROJECT := $(AWS_ACCESS_KEY_ID)
AWS_SECRET_FROM_PROJECT := $(AWS_SECRET_ACCESS_KEY)
AWS_REGION_FROM_PROJECT := $(AWS_REGION)
AWS_SANDBOX_FROM_PROJECT := $(AWS_SANDBOX_ACCOUNT_ID)
endif

# Not exported: the emulator targets must not inherit real credentials. Only
# aws-check-sandbox and aws-sandbox pass these to a child, explicitly.
unexport AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY AWS_REGION AWS_SANDBOX_ACCOUNT_ID
AWS_ACCESS_KEY_ID := $(strip $(or $(AWS_KEY_FROM_ENV),$(AWS_KEY_FROM_PROJECT),$(AWS_KEY_FROM_GLOBAL)))
AWS_SECRET_ACCESS_KEY := $(strip $(or $(AWS_SECRET_FROM_ENV),$(AWS_SECRET_FROM_PROJECT),$(AWS_SECRET_FROM_GLOBAL)))
AWS_REGION := $(strip $(or $(AWS_REGION_FROM_ENV),$(AWS_REGION_FROM_PROJECT),$(AWS_REGION_FROM_GLOBAL)))
AWS_SANDBOX_ACCOUNT_ID := $(strip $(or $(AWS_SANDBOX_FROM_ENV),$(AWS_SANDBOX_FROM_PROJECT),$(AWS_SANDBOX_FROM_GLOBAL)))

AWS_KEY_SOURCE := $(strip \
	$(if $(AWS_KEY_FROM_ENV),environment, \
	$(if $(AWS_KEY_FROM_PROJECT),project, \
	$(if $(AWS_KEY_FROM_GLOBAL),global,none))))
AWS_SANDBOX_SOURCE := $(strip \
	$(if $(AWS_SANDBOX_FROM_ENV),environment, \
	$(if $(AWS_SANDBOX_FROM_PROJECT),project, \
	$(if $(AWS_SANDBOX_FROM_GLOBAL),global,none))))

# Environment for a child that talks to the emulator: fake credentials forced,
# anything that could reach real AWS removed.
AWS_EMULATOR_ENV = env -u AWS_PROFILE -u AWS_SESSION_TOKEN \
	AWS_ENDPOINT_URL=$(AWS_EMULATOR_URL) \
	AWS_ACCESS_KEY_ID=$(AWS_EMULATOR_ACCOUNT) \
	AWS_SECRET_ACCESS_KEY=test \
	AWS_REGION=$(AWS_EMULATOR_REGION) \
	AWS_DEFAULT_REGION=$(AWS_EMULATOR_REGION) \
	AWS_EC2_METADATA_DISABLED=true

# Environment for a child that talks to the sandbox account: no endpoint
# override, resolved credentials passed only when a file or the environment
# supplied them (otherwise the AWS CLI's own chain, e.g. SSO, applies).
AWS_SANDBOX_ENV = env -u AWS_ENDPOINT_URL \
	$(if $(AWS_ACCESS_KEY_ID),AWS_ACCESS_KEY_ID=$(AWS_ACCESS_KEY_ID),) \
	$(if $(AWS_SECRET_ACCESS_KEY),AWS_SECRET_ACCESS_KEY=$(AWS_SECRET_ACCESS_KEY),) \
	$(if $(AWS_REGION),AWS_REGION=$(AWS_REGION) AWS_DEFAULT_REGION=$(AWS_REGION),)

# ═══════════════════════════════════════════════════════════════════════════════
# Emulator lane
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: aws-up aws-down aws-logs aws-health aws-reset aws-env aws-cli

aws-up: ## Start the local AWS emulator (joins this project's compose project)
	@source ./scripts/.bashrc && AWS_ROUTER_HOST=$(AWS_ROUTER_HOST) docker compose -p "$$COMPOSE_PROJECT_NAME" -f $(AWS_COMPOSE_FILE) up -d aws
	@echo "✓ AWS emulator: $(AWS_EMULATOR_URL)  (in compose: http://aws:4566)"

aws-down: ## Stop the local AWS emulator (v=1 also drops its state volume)
	@source ./scripts/.bashrc && docker compose -p "$$COMPOSE_PROJECT_NAME" -f $(AWS_COMPOSE_FILE) rm -sf $(if $(v),-v,) aws
	@$(if $(v),source ./scripts/.bashrc && docker volume rm -f "$${COMPOSE_PROJECT_NAME}_aws_state" >/dev/null,true)

aws-logs: ## Tail the local AWS emulator logs
	@source ./scripts/.bashrc && docker compose -p "$$COMPOSE_PROJECT_NAME" -f $(AWS_COMPOSE_FILE) logs -f --tail=200 aws

aws-health: ## Wait for the local AWS emulator to answer
	@curl --retry 30 --retry-connrefused --retry-all-errors --retry-delay 1 -sf -o /dev/null $(AWS_EMULATOR_URL)/_localstack/health \
		|| { echo "✗ No emulator answering at $(AWS_EMULATOR_URL). Run 'make aws-up' and check 'mx router status'." >&2; exit 1; }
	@echo "✓ AWS emulator healthy at $(AWS_EMULATOR_URL)"

aws-reset: ## Wipe all local AWS emulator state (recreates the container and its volume)
	@$(MAKE) aws-down v=1
	@$(MAKE) aws-up
	@$(MAKE) aws-health
	@echo "✓ AWS emulator state cleared"

aws-env: ## Print exports for the emulator lane: eval "$(make aws-env)"
	@echo "export AWS_ENDPOINT_URL=$(AWS_EMULATOR_URL)"
	@echo "export AWS_ACCESS_KEY_ID=$(AWS_EMULATOR_ACCOUNT)"
	@echo "export AWS_SECRET_ACCESS_KEY=test"
	@echo "export AWS_REGION=$(AWS_EMULATOR_REGION)"
	@echo "export AWS_DEFAULT_REGION=$(AWS_EMULATOR_REGION)"
	@echo "export AWS_EC2_METADATA_DISABLED=true"
	@echo "unset AWS_PROFILE AWS_SESSION_TOKEN"

aws-cli: ## Run the AWS CLI against the emulator (c="s3 ls")
	@$(AWS_EMULATOR_ENV) MX_AWS_ARGS="$(call get_cmd)" bash -c 'aws $$MX_AWS_ARGS'

# ═══════════════════════════════════════════════════════════════════════════════
# Sandbox gate
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: aws-vars aws-check-sandbox aws-sandbox

aws-vars: ## Show the resolved sandbox variables and their source (no secrets)
	@echo "AWS_SANDBOX_ACCOUNT_ID=$(AWS_SANDBOX_ACCOUNT_ID)"
	@echo "AWS_SANDBOX_ACCOUNT_ID_SOURCE=$(AWS_SANDBOX_SOURCE)"
	@echo "AWS_ACCESS_KEY_ID=$(if $(AWS_ACCESS_KEY_ID),***configured***,)"
	@echo "AWS_ACCESS_KEY_ID_SOURCE=$(AWS_KEY_SOURCE)"
	@echo "AWS_SECRET_ACCESS_KEY=$(if $(AWS_SECRET_ACCESS_KEY),***configured***,)"
	@echo "AWS_REGION=$(AWS_REGION)"
	@echo "AWS_ENV_FILE=$(AWS_ENV_FILE)"
	@echo "AWS_GLOBAL_ENV_FILE=$(AWS_GLOBAL_ENV_FILE)"
	@echo "AWS_PROJECT_LINKED=$(AWS_PROJECT_LINKED)"
	@echo "AWS_ENDPOINT_URL_IN_ENVIRONMENT=$(if $(AWS_ENDPOINT_FROM_ENV),$(AWS_ENDPOINT_FROM_ENV),)"
	@echo "AWS_EMULATOR_URL=$(AWS_EMULATOR_URL)"

aws-check-sandbox: ## Refuse unless the caller is the configured sandbox account
ifneq ($(strip $(AWS_ENDPOINT_FROM_ENV)),)
	@echo "✗ AWS_ENDPOINT_URL is set ($(AWS_ENDPOINT_FROM_ENV))." >&2
	@echo "  The sandbox lane talks to real AWS; an emulator endpoint must not ride along." >&2
	@echo "  Run 'unset AWS_ENDPOINT_URL' (or open a shell without 'make aws-env') and retry." >&2
	@exit 1
else ifeq ($(strip $(AWS_SANDBOX_ACCOUNT_ID)),)
	@echo "✗ No AWS_SANDBOX_ACCOUNT_ID configured." >&2
	@echo "" >&2
	@echo "  Set the 12-digit id of the non-production account this project may deploy to, in:" >&2
	@echo "    project: $(AWS_ENV_FILE)" >&2
	@echo "    global:  $(AWS_GLOBAL_ENV_FILE)" >&2
	@echo "  Credentials come from the same files ('mx infra setup aws') or the AWS CLI's own chain." >&2
	@exit 1
else
	@actual="$$($(AWS_SANDBOX_ENV) aws sts get-caller-identity --query Account --output text 2>&1)" || { \
		echo "✗ Could not identify the AWS caller: $$actual" >&2; exit 1; }; \
	if [ "$$actual" != "$(AWS_SANDBOX_ACCOUNT_ID)" ]; then \
		echo "✗ Caller is account $$actual, but this project's sandbox is $(AWS_SANDBOX_ACCOUNT_ID)." >&2; \
		echo "  Refusing: wrong profile or wrong credentials file ('make aws-vars' shows the source)." >&2; \
		exit 1; \
	fi; \
	echo "✓ Caller is the sandbox account $(AWS_SANDBOX_ACCOUNT_ID)"
endif

aws-sandbox: aws-check-sandbox ## Run a command against the sandbox account (c="terraform apply")
	@$(AWS_SANDBOX_ENV) bash -c "$(call get_cmd)"
