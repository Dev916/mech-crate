.PHONY: init-stack

# Initialize a specific stack for users with limited repo access
init-stack:
	@if [ -z "$(s)" ]; then \
		echo "Error: Stack name is required"; \
		echo "Usage: make init-stack s=<stack-name>"; \
		echo "Available stacks: campus, connect, tbco"; \
		exit 1; \
	fi
	@./scripts/init-stack.sh $(s)
