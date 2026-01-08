include make/common.mk

include make/append.mk
include make/build.mk
include make/dev.mk
include make/down.mk
include make/exec.mk
include make/import-vector.mk
include make/logs.mk
include make/pull.mk
include make/reboot.mk
include make/rebuild.mk
include make/restart.mk
include make/run.mk
include make/sh.mk
include make/up.mk
include make/start.mk
include make/add-app.mk
include make/launch.mk
include make/update.mk
include make/bounce.mk
include make/init-stack.mk
include make/ecs.mk

# Run the MCP server (Rust stdio)
.PHONY: mcp mcp/build mcp/run
# Intelligent runner: builds if target binary missing or out-of-date
mcp:
	@$(MAKE) mcp/run

# Explicit build target for CI or manual rebuilds
mcp/build:
	@echo "Building mono-mcp..." 1>&2
	@cargo build --quiet --manifest-path mcp/mono-mcp/Cargo.toml 1>&2

# Run without rebuilding if binary is up-to-date; build if missing
mcp/run:
	@BIN="mcp/mono-mcp/target/debug/mono-mcp"; \
	if [ ! -x "$$BIN" ]; then \
	  echo "mono-mcp binary not found; building..." 1>&2; \
	  $(MAKE) mcp/build; \
	fi; \
	exec "$$BIN"

# Display this help message
help:
	@./scripts/help.sh

# Init the environment. Install dependencies, initialze the databases, and WordPress
init:
	@./scripts/init.sh

# Run tests (not implemented)
test:
	@./scripts/test.sh

# List all running services
ps:
	@./scripts/ps.sh

# Import the latest snapshot of wordpress data from production (mysql data and payloads)
import-data:
	@./scripts/import-data.sh

# Not implemented
deploy:
	@./scripts/deploy.sh

# Pull all of the latest code from main or master in each application. (ex. make latest)
latest:
	@./scripts/latest.sh

# Provision environment for simon ai setup. This will install the necessary dependencies and setup the environment for connect and Simon AI. (ex. make simon)
init-simon:
	@./scripts/simon.sh

# Stop all services no matter what
halt:
	@./scripts/halt.sh

# hop into psql prompt for the postgres
psql:
	@./scripts/psql.sh

# hop into redis-cli prompt for redis
redis:
	@./scripts/redis-cli.sh

# hop into redis-cli prompt for redis
redis-cli:
	@./scripts/redis-cli.sh

# bump package for all services
bump-packages:
	@./scripts/bump-packages.sh $(p) $(v)

# Save current state of running services
save:
	@./scripts/save.sh

# Resume from the saved state of running services
resume:
	@./scripts/resume.sh

stats:
	@./scripts/server/stats.sh

# Follow laravel logs for campus
campus-logs:
	@./scripts/campus-logs.sh

# Deploy hook for the server
deploy-hook:
	@./scripts/server/deploy-hook.sh

# Install mono cli
install-mono:
	@./scripts/utils/install_cli.sh

# Shim docker compose to use docker-compose (e.g for linux environments)
shim-docker-compose:
	@./scripts/utils/shim_docker_compose.sh

sqs-init:
	@./scripts/sqs-init.sh

sqs-worker:
	@./scripts/sqs-worker.sh