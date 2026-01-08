.PHONY: import-vector

# Initialize schema and populate Weaviate from WordPress DB
import-vector:
	@echo "Importing vector db"
	@./scripts/import-vector.sh $(d)