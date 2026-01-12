#!/bin/bash
#
# MechCrate Init Code Rules
# Initializes coding rules and development documentation for a project
#

# ─────────────────────────────────────────────────────────────────────────────
# Configuration
# ─────────────────────────────────────────────────────────────────────────────
ICR_CURSOR_DIR=".cursor"
ICR_CURSOR_CONFIG="$ICR_CURSOR_DIR/config.json"
ICR_DOCS_DIR="docs/development"

# ─────────────────────────────────────────────────────────────────────────────
# Init Code Rules Command
# ─────────────────────────────────────────────────────────────────────────────
init_code_rules_cmd() {
    local target_dir="${1:-.}"
    local skip_docs=false
    local skip_cursor=false
    
    # Parse flags
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --skip-docs)
                skip_docs=true
                shift
                ;;
            --skip-cursor)
                skip_cursor=true
                shift
                ;;
            --help|-h)
                init_code_rules_help
                return 0
                ;;
            *)
                if [[ ! "$1" =~ ^- ]]; then
                    target_dir="$1"
                fi
                shift
                ;;
        esac
    done
    
    # Resolve to absolute path
    target_dir="$(cd "$target_dir" 2>/dev/null && pwd)" || {
        error "Directory does not exist: $target_dir"
    }
    
    echo -e "${CYAN}╭──────────────────────────────────────────────────────╮${NC}"
    echo -e "${CYAN}│${NC}  🔧 ${BOLD}Initializing Code Rules${NC}"
    echo -e "${CYAN}│${NC}     ${MAGENTA}$target_dir${NC}"
    echo -e "${CYAN}╰──────────────────────────────────────────────────────╯${NC}"
    echo ""
    
    # Change to target directory
    pushd "$target_dir" > /dev/null
    
    # 1. Copy development documentation from MechCrate (must be first)
    if [[ "$skip_docs" == false ]]; then
        _copy_dev_docs
    fi
    
    # 2. Setup Cursor configuration (uses local docs)
    if [[ "$skip_cursor" == false ]]; then
        _setup_cursor_config
    fi
    
    # 3. Update .gitignore
    _update_gitignore
    
    popd > /dev/null
    
    echo ""
    success "Done! Project is now configured with coding rules and documentation."
    echo ""
    info "Locations:"
    echo "    • Cursor: $ICR_CURSOR_CONFIG"
    echo "    • Rules:  $ICR_DOCS_DIR/instructions.md"
    echo "    • Docs:   $ICR_DOCS_DIR/"
    echo ""
    info "The rules include MechCrate MCP/RAG integration:"
    echo "    • 7 RAG tools for semantic documentation search"
    echo "    • Guidance on when to query before implementing"
}

# ─────────────────────────────────────────────────────────────────────────────
# Setup Cursor IDE configuration
# ─────────────────────────────────────────────────────────────────────────────
_setup_cursor_config() {
    mkdir -p "$ICR_CURSOR_DIR"
    
    local local_rules="./$ICR_DOCS_DIR/instructions.md"
    
    # Verify local rules exist
    if [[ ! -f "$local_rules" ]]; then
        warn "Local instructions not found at $local_rules"
        info "Run without --skip-docs to copy documentation first"
        return 1
    fi
    
    cat > "$ICR_CURSOR_CONFIG" <<EOF
{
  "rules": "$local_rules",
  "style": "functional, test-driven, clean abstractions, use MCP RAG tools"
}
EOF
    success "Configured $ICR_CURSOR_CONFIG → $local_rules"
}

# ─────────────────────────────────────────────────────────────────────────────
# Update .gitignore with appropriate entries
# ─────────────────────────────────────────────────────────────────────────────
_update_gitignore() {
    if [[ -f ".gitignore" ]]; then
        if ! grep -q "^\.cursor/config\.json$" .gitignore 2>/dev/null; then
            echo ".cursor/config.json" >> .gitignore
            success "Updated .gitignore"
        else
            info ".gitignore already configured"
        fi
    else
        echo ".cursor/config.json" > .gitignore
        success "Created .gitignore"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Copy development documentation from MechCrate
# ─────────────────────────────────────────────────────────────────────────────
_copy_dev_docs() {
    local source_docs="$MECH_CRATE_ROOT/docs/development"
    
    mkdir -p "$ICR_DOCS_DIR"
    
    local copied=0
    
    # Copy from MechCrate's docs/development if available
    if [[ -d "$source_docs" ]]; then
        info "Copying development documentation from MechCrate..."
        for file in "$source_docs"/*.md; do
            if [[ -f "$file" ]]; then
                local filename=$(basename "$file")
                if [[ ! -f "$ICR_DOCS_DIR/$filename" ]]; then
                    cp "$file" "$ICR_DOCS_DIR/"
                    ((copied++))
                fi
            fi
        done
        if [[ $copied -gt 0 ]]; then
            success "Copied $copied documentation files to $ICR_DOCS_DIR/"
        else
            info "All documentation files already exist"
        fi
    else
        warn "MechCrate docs not found at $source_docs"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Help text
# ─────────────────────────────────────────────────────────────────────────────
init_code_rules_help() {
    echo -e "${BOLD}mx icr${NC} - Initialize coding rules and development documentation"
    echo ""
    echo -e "${BOLD}USAGE:${NC}"
    echo "    mx icr [OPTIONS] [DIRECTORY]"
    echo "    mx init-code-rules [OPTIONS] [DIRECTORY]"
    echo ""
    echo -e "${BOLD}ARGUMENTS:${NC}"
    echo "    DIRECTORY    Target project directory (default: current directory)"
    echo ""
    echo -e "${BOLD}OPTIONS:${NC}"
    echo "    --skip-docs      Skip copying development documentation"
    echo "    --skip-cursor    Skip Cursor IDE configuration"
    echo "    -h, --help       Show this help message"
    echo ""
    echo -e "${BOLD}DESCRIPTION:${NC}"
    echo "    Sets up coding rules and development documentation for a project."
    echo "    The rules include MechCrate MCP/RAG integration for AI-assisted development."
    echo ""
    echo "    This command will:"
    echo "    1. Copy MechCrate development docs to docs/development/"
    echo "    2. Configure .cursor/config.json to use local docs/development/instructions.md"
    echo "    3. Update .gitignore with appropriate entries"
    echo ""
    echo -e "${BOLD}MCP/RAG INTEGRATION:${NC}"
    echo "    The instructions.md file includes guidance for using MechCrate's 7 RAG tools:"
    echo "    • rag_search              - Semantic search across all docs"
    echo "    • rag_search_category     - Search specific categories"
    echo "    • rag_find_implementation - Find code examples"
    echo "    • rag_get_guidance        - Get architecture guidance"
    echo "    • rag_compare_approaches  - Compare technologies"
    echo "    • rag_find_related        - Discover related docs"
    echo "    • rag_health              - Check RAG availability"
    echo ""
    echo -e "${BOLD}EXAMPLES:${NC}"
    echo "    mx icr                    # Initialize in current directory"
    echo "    mx icr ./my-project       # Initialize in specific directory"
    echo "    mx icr --skip-cursor      # Only copy docs, skip Cursor config"
    echo ""
}
