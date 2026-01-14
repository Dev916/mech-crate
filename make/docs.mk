# ═══════════════════════════════════════════════════════════════════════════════
# Document Compilation (Portable - just needs Node.js)
# ═══════════════════════════════════════════════════════════════════════════════

DOCS_SCRIPT_DIR := $(ROOT_DIR)/scripts/docs
DOCS_ARTIFACTS_DIR := $(ROOT_DIR)/artifacts/unyform
UNYFORM_DOCS_DIR := $(ROOT_DIR)/docs/unyform

# Variables for folder/file compilation
DOCS_FOLDER ?=
DOCS_FILE ?=
DOCS_OUTPUT ?=
DOCS_PREFIX ?=
DOCS_AUTHOR ?=

.PHONY: docs
docs: docs-deps docs-all ## Compile all unyform.ai documents to PDF/HTML

.PHONY: docs-deps
docs-deps: ## Install documentation dependencies (npm only)
	@if [ ! -d "$(DOCS_SCRIPT_DIR)/node_modules" ]; then \
		echo "📦 Installing documentation dependencies..."; \
		cd $(DOCS_SCRIPT_DIR) && npm install --silent 2>/dev/null || npm install; \
		echo "✅ Dependencies installed"; \
	fi

.PHONY: docs-all
docs-all: docs-deps ## Compile all predefined unyform documents
	@echo "📄 Compiling all unyform.ai documents..."
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --all
	@echo ""
	@echo "✅ Documents compiled to: $(DOCS_ARTIFACTS_DIR)"

# ═══════════════════════════════════════════════════════════════════════════════
# Folder and File Compilation
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: docs-folder
docs-folder: docs-deps ## Compile all markdown files in a folder (DOCS_FOLDER=./path)
ifndef DOCS_FOLDER
	@echo "❌ Error: DOCS_FOLDER is required"
	@echo ""
	@echo "Usage: make docs-folder DOCS_FOLDER=./path/to/docs [DOCS_OUTPUT=./output]"
	@exit 1
endif
	@echo "📁 Compiling markdown files from: $(DOCS_FOLDER)"
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts \
		"$(abspath $(DOCS_FOLDER))" \
		$(if $(DOCS_OUTPUT),--output="$(abspath $(DOCS_OUTPUT))",) \
		$(if $(DOCS_PREFIX),--prefix="$(DOCS_PREFIX)",) \
		$(if $(DOCS_AUTHOR),--author="$(DOCS_AUTHOR)",)

.PHONY: docs-file
docs-file: docs-deps ## Compile a single markdown file (DOCS_FILE=./path/to/file.md)
ifndef DOCS_FILE
	@echo "❌ Error: DOCS_FILE is required"
	@echo ""
	@echo "Usage: make docs-file DOCS_FILE=./path/to/doc.md [DOCS_OUTPUT=./output]"
	@exit 1
endif
	@echo "📄 Compiling: $(DOCS_FILE)"
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts \
		"$(abspath $(DOCS_FILE))" \
		$(if $(DOCS_OUTPUT),--output="$(abspath $(DOCS_OUTPUT))",)

# ═══════════════════════════════════════════════════════════════════════════════
# Individual unyform Documents
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: docs-whitepaper
docs-whitepaper: docs-deps ## Compile whitepaper
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=whitepaper

.PHONY: docs-executive
docs-executive: docs-deps ## Compile executive summary
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=executive-summary

.PHONY: docs-roadmap
docs-roadmap: docs-deps ## Compile roadmap
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=roadmap

.PHONY: docs-competitive
docs-competitive: docs-deps ## Compile competitive analysis
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=competitive-analysis

.PHONY: docs-prd
docs-prd: docs-deps ## Compile MVP PRD
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=mvp-prd

.PHONY: docs-pitch
docs-pitch: docs-deps ## Compile pitch deck
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=pitch-deck

.PHONY: docs-gtm
docs-gtm: docs-deps ## Compile GTM playbook
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=gtm-playbook

.PHONY: docs-architecture
docs-architecture: docs-deps ## Compile technical architecture
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=tech-architecture

.PHONY: docs-pricing
docs-pricing: docs-deps ## Compile pricing strategy
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=pricing-strategy

# ═══════════════════════════════════════════════════════════════════════════════
# Utilities
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: docs-list
docs-list: docs-deps ## List available predefined documents
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --list

.PHONY: docs-clean
docs-clean: ## Clean documentation artifacts
	@echo "🧹 Cleaning documentation artifacts..."
	@rm -rf $(DOCS_ARTIFACTS_DIR)
	@rm -rf $(DOCS_SCRIPT_DIR)/node_modules
	@echo "✅ Artifacts cleaned"

.PHONY: docs-check
docs-check: ## Check documentation dependencies
	@echo "🔍 Checking documentation dependencies..."
	@echo ""
	@echo "Required:"
	@echo -n "  Node.js: "; which node >/dev/null 2>&1 && echo "✅ $(shell node --version)" || echo "❌ Not found (brew install node)"
	@echo -n "  npm:     "; which npm >/dev/null 2>&1 && echo "✅ $(shell npm --version)" || echo "❌ Not found"
	@echo ""
	@echo "Optional (for PDF output):"
	@echo -n "  Pandoc:  "; which pandoc >/dev/null 2>&1 && echo "✅ $(shell pandoc --version | head -1)" || echo "⚠️  Not installed (brew install pandoc)"
	@echo -n "  XeLaTeX: "; which xelatex >/dev/null 2>&1 && echo "✅ Found" || echo "⚠️  Not installed (brew install --cask mactex-no-gui)"
	@echo ""
	@echo "Without Pandoc/LaTeX, HTML output will still be generated."

.PHONY: docs-help
docs-help: ## Show documentation help
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "     Portable Document Compiler - Just needs Node.js!"
	@echo "═══════════════════════════════════════════════════════════════"
	@echo ""
	@echo "Compiles Markdown → HTML (always) + PDF (if pandoc available)"
	@echo ""
	@echo "unyform.ai Documents:"
	@echo "  make docs              Compile all predefined documents"
	@echo "  make docs-whitepaper   Compile whitepaper"
	@echo "  make docs-prd          Compile MVP PRD"
	@echo "  make docs-pitch        Compile pitch deck"
	@echo "  make docs-list         List available documents"
	@echo ""
	@echo "Generic Compilation:"
	@echo "  make docs-folder DOCS_FOLDER=./path"
	@echo "  make docs-file DOCS_FILE=./doc.md"
	@echo ""
	@echo "Options:"
	@echo "  DOCS_OUTPUT=./path     Custom output directory"
	@echo "  DOCS_PREFIX=v2         Add prefix to filenames"
	@echo "  DOCS_AUTHOR=\"Name\"     Default author"
	@echo ""
	@echo "Utilities:"
	@echo "  make docs-deps         Install npm dependencies"
	@echo "  make docs-check        Check system dependencies"
	@echo "  make docs-clean        Remove artifacts"
	@echo ""
	@echo "Output: artifacts/<name>/"
	@echo "  ├── <name>.html   (always)"
	@echo "  ├── <name>.pdf    (if pandoc available)"
	@echo "  ├── <name>.md     (processed)"
	@echo "  └── diagrams/     (rendered PNGs)"
	@echo ""
