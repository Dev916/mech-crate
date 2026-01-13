# ═══════════════════════════════════════════════════════════════════════════════
# Document Compilation (PDF from Markdown)
# ═══════════════════════════════════════════════════════════════════════════════

DOCS_SCRIPT_DIR := $(ROOT_DIR)/scripts/docs
DOCS_ARTIFACTS_DIR := $(ROOT_DIR)/artifacts/unyform
UNYFORM_DOCS_DIR := $(ROOT_DIR)/docs/unyform

# Default folder for generic compilation (can be overridden)
DOCS_FOLDER ?=
DOCS_OUTPUT ?=
DOCS_PREFIX ?=
DOCS_AUTHOR ?=

.PHONY: docs
docs: docs-deps docs-all ## Compile all unyform.ai documents to PDF

.PHONY: docs-deps
docs-deps: ## Install documentation compilation dependencies
	@echo "📦 Installing documentation dependencies..."
	@cd $(DOCS_SCRIPT_DIR) && npm install --silent 2>/dev/null || npm install
	@echo "✅ Dependencies installed"

.PHONY: docs-all
docs-all: ## Compile all predefined unyform documents
	@echo "📄 Compiling all unyform.ai documents..."
	@mkdir -p $(DOCS_ARTIFACTS_DIR)
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --all
	@echo ""
	@echo "✅ Documents compiled to: $(DOCS_ARTIFACTS_DIR)"

# ═══════════════════════════════════════════════════════════════════════════════
# Folder Compilation
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: docs-folder
docs-folder: docs-deps ## Compile all markdown files in a folder (DOCS_FOLDER=./path)
ifndef DOCS_FOLDER
	@echo "❌ Error: DOCS_FOLDER is required"
	@echo ""
	@echo "Usage: make docs-folder DOCS_FOLDER=./path/to/docs [DOCS_OUTPUT=./output] [DOCS_PREFIX=prefix]"
	@echo ""
	@echo "Options:"
	@echo "  DOCS_FOLDER  Path to folder containing markdown files (required)"
	@echo "  DOCS_OUTPUT  Output directory (default: DOCS_FOLDER/output)"
	@echo "  DOCS_PREFIX  Prefix for output filenames"
	@echo "  DOCS_AUTHOR  Default author for documents without frontmatter"
	@exit 1
endif
	@echo "📁 Compiling markdown files from: $(DOCS_FOLDER)"
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts \
		--folder=$(abspath $(DOCS_FOLDER)) \
		$(if $(DOCS_OUTPUT),--output=$(abspath $(DOCS_OUTPUT)),) \
		$(if $(DOCS_PREFIX),--prefix=$(DOCS_PREFIX),) \
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
		--file=$(abspath $(DOCS_FILE)) \
		$(if $(DOCS_OUTPUT),--output=$(abspath $(DOCS_OUTPUT)),)

# ═══════════════════════════════════════════════════════════════════════════════
# Individual unyform Documents
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: docs-whitepaper
docs-whitepaper: docs-deps ## Compile whitepaper only
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=whitepaper

.PHONY: docs-executive
docs-executive: docs-deps ## Compile executive summary only
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=executive-summary

.PHONY: docs-roadmap
docs-roadmap: docs-deps ## Compile roadmap only
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=roadmap

.PHONY: docs-competitive
docs-competitive: docs-deps ## Compile competitive analysis only
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=competitive-analysis

.PHONY: docs-prd
docs-prd: docs-deps ## Compile MVP PRD only
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=mvp-prd

.PHONY: docs-pitch
docs-pitch: docs-deps ## Compile pitch deck only
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=pitch-deck

.PHONY: docs-gtm
docs-gtm: docs-deps ## Compile GTM playbook only
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=gtm-playbook

.PHONY: docs-architecture
docs-architecture: docs-deps ## Compile technical architecture only
	@cd $(DOCS_SCRIPT_DIR) && npx tsx compile.ts --doc=tech-architecture

.PHONY: docs-pricing
docs-pricing: docs-deps ## Compile pricing strategy only
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
	@echo "✅ Artifacts cleaned"

.PHONY: docs-check
docs-check: ## Check documentation dependencies
	@echo "🔍 Checking documentation dependencies..."
	@echo ""
	@echo "Node.js:"
	@which node && node --version || echo "  ❌ Not found (required)"
	@echo ""
	@echo "npm:"
	@which npm && npm --version || echo "  ❌ Not found (required)"
	@echo ""
	@echo "Pandoc:"
	@which pandoc && pandoc --version | head -1 || echo "  ⚠️  Not found (install with: brew install pandoc)"
	@echo ""
	@echo "LaTeX (xelatex):"
	@which xelatex && xelatex --version | head -1 || echo "  ⚠️  Not found (install with: brew install --cask mactex-no-gui)"
	@echo ""
	@echo "Predefined documents:"
	@ls -1 $(UNYFORM_DOCS_DIR)/*.md 2>/dev/null | xargs -I{} basename {} || echo "  No documents found"

.PHONY: docs-help
docs-help: ## Show documentation help
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "           Document Compilation Help"
	@echo "═══════════════════════════════════════════════════════════════"
	@echo ""
	@echo "Predefined Documents (unyform.ai):"
	@echo "  make docs              Compile all predefined documents"
	@echo "  make docs-whitepaper   Compile whitepaper only"
	@echo "  make docs-executive    Compile executive summary only"
	@echo "  make docs-prd          Compile MVP PRD only"
	@echo "  make docs-pitch        Compile pitch deck only"
	@echo "  make docs-gtm          Compile GTM playbook only"
	@echo "  make docs-architecture Compile technical architecture only"
	@echo "  make docs-pricing      Compile pricing strategy only"
	@echo "  make docs-list         List available predefined documents"
	@echo ""
	@echo "Generic Compilation:"
	@echo "  make docs-folder DOCS_FOLDER=./path    Compile all .md files in folder"
	@echo "  make docs-file DOCS_FILE=./doc.md      Compile a single file"
	@echo ""
	@echo "Options for docs-folder:"
	@echo "  DOCS_OUTPUT=./path     Custom output directory"
	@echo "  DOCS_PREFIX=myprefix   Add prefix to output filenames"
	@echo "  DOCS_AUTHOR=\"Name\"     Default author for docs without frontmatter"
	@echo ""
	@echo "Utilities:"
	@echo "  make docs-clean        Clean artifacts"
	@echo "  make docs-check        Check dependencies"
	@echo ""
	@echo "Output: $(DOCS_ARTIFACTS_DIR)/"
	@echo ""
	@echo "Frontmatter Support:"
	@echo "  Documents can include YAML frontmatter for metadata:"
	@echo ""
	@echo "  ---"
	@echo "  title: My Document"
	@echo "  subtitle: Optional Subtitle"
	@echo "  author: Author Name"
	@echo "  toc: true"
	@echo "  ---"
	@echo ""
