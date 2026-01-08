.PHONY: add-app


# Add a new app to the project where s=[service]
add-app:
	@$(MAKE) _add-app app=$(call get_service)

# Add a new app to the project where s=[service]
_add-app:
	@./scripts/add-app.sh $(app)