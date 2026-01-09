#!/bin/bash
#
# MechCrate Init Code Rules
# Initializes coding rules and development documentation for a project
#

# ─────────────────────────────────────────────────────────────────────────────
# Configuration
# ─────────────────────────────────────────────────────────────────────────────
ICR_GLOBAL_RULES="$HOME/.codex/instructions.md"
ICR_LOCAL_RULES="CODING_RULES.md"
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
    local skip_rules=false
    
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
            --skip-rules)
                skip_rules=true
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
    
    # 1. Setup global rules and local symlink
    if [[ "$skip_rules" == false ]]; then
        _setup_coding_rules
    fi
    
    # 2. Setup Cursor configuration
    if [[ "$skip_cursor" == false ]]; then
        _setup_cursor_config
    fi
    
    # 3. Update .gitignore
    _update_gitignore
    
    # 4. Copy development documentation from MechCrate
    if [[ "$skip_docs" == false ]]; then
        _copy_dev_docs
    fi
    
    popd > /dev/null
    
    echo ""
    success "Done! Project is now configured with coding rules and documentation."
    echo ""
    info "Locations:"
    echo "    • Rules:  $ICR_LOCAL_RULES → $ICR_GLOBAL_RULES"
    echo "    • Cursor: $ICR_CURSOR_CONFIG"
    echo "    • Docs:   $ICR_DOCS_DIR/"
}

# ─────────────────────────────────────────────────────────────────────────────
# Setup global rules file and local symlink
# ─────────────────────────────────────────────────────────────────────────────
_setup_coding_rules() {
    # Ensure global rules exist
    if [[ ! -f "$ICR_GLOBAL_RULES" ]]; then
        warn "Global instructions file not found at $ICR_GLOBAL_RULES"
        info "Creating a blank template..."
        mkdir -p "$(dirname "$ICR_GLOBAL_RULES")"
        cat > "$ICR_GLOBAL_RULES" <<'EOF'
🚫 NEVER DELETE THIS FILE 🚫

# Codex Execution Rules

## Functional Design Principles
- Pure functions where possible
- Immutable data structures
- Explicit side effects

## Software Design Principles
- Test-driven development
- Clean abstractions
- Single responsibility
- Composition over inheritance

## Code Style
- Clear naming conventions
- Comprehensive documentation
- Consistent formatting

(Add your project-specific principles here)
EOF
        success "Created global rules at $ICR_GLOBAL_RULES"
    else
        success "Found global rules at $ICR_GLOBAL_RULES"
    fi
    
    # Create local symlink if not exists
    if [[ ! -e "$ICR_LOCAL_RULES" ]]; then
        ln -s "$ICR_GLOBAL_RULES" "$ICR_LOCAL_RULES"
        success "Linked $ICR_LOCAL_RULES → $ICR_GLOBAL_RULES"
    else
        info "$ICR_LOCAL_RULES already exists, skipping symlink"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Setup Cursor IDE configuration
# ─────────────────────────────────────────────────────────────────────────────
_setup_cursor_config() {
    mkdir -p "$ICR_CURSOR_DIR"
    
    if [[ ! -f "$ICR_CURSOR_CONFIG" ]]; then
        cat > "$ICR_CURSOR_CONFIG" <<'EOF'
{
  "rules": "./CODING_RULES.md",
  "style": "functional, test-driven, clean abstractions"
}
EOF
        success "Created $ICR_CURSOR_CONFIG"
    else
        info "$ICR_CURSOR_CONFIG already exists, skipping"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Update .gitignore with appropriate entries
# ─────────────────────────────────────────────────────────────────────────────
_update_gitignore() {
    local updated=false
    
    if [[ -f ".gitignore" ]]; then
        if ! grep -q "^CODING_RULES.md$" .gitignore 2>/dev/null; then
            echo "CODING_RULES.md" >> .gitignore
            updated=true
        fi
        if ! grep -q "^\.cursor/config\.json$" .gitignore 2>/dev/null; then
            echo ".cursor/config.json" >> .gitignore
            updated=true
        fi
        if [[ "$updated" == true ]]; then
            success "Updated .gitignore"
        else
            info ".gitignore already configured"
        fi
    else
        cat > .gitignore <<'EOF'
CODING_RULES.md
.cursor/config.json
EOF
        success "Created .gitignore"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Copy development documentation from MechCrate
# ─────────────────────────────────────────────────────────────────────────────
_copy_dev_docs() {
    local source_docs="$MECH_CRATE_ROOT/docs/development"
    local codex_docs="$HOME/.codex"
    
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
    
    # Also copy from ~/.codex if available (legacy/personal docs)
    if [[ -d "$codex_docs" ]]; then
        local codex_copied=0
        for file in "$codex_docs"/*.md; do
            if [[ -f "$file" ]]; then
                local filename=$(basename "$file")
                # Skip instructions.md as it's the rules file
                if [[ "$filename" != "instructions.md" && ! -f "$ICR_DOCS_DIR/$filename" ]]; then
                    cp "$file" "$ICR_DOCS_DIR/"
                    ((codex_copied++))
                fi
            fi
        done
        if [[ $codex_copied -gt 0 ]]; then
            success "Copied $codex_copied personal docs from ~/.codex/"
        fi
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
    echo "    --skip-rules     Skip CODING_RULES.md symlink setup"
    echo "    -h, --help       Show this help message"
    echo ""
    echo -e "${BOLD}DESCRIPTION:${NC}"
    echo "    Sets up coding rules and development documentation for a project."
    echo ""
    echo "    This command will:"
    echo "    1. Create/link CODING_RULES.md to ~/.codex/instructions.md"
    echo "    2. Configure .cursor/config.json for Cursor IDE"
    echo "    3. Update .gitignore with appropriate entries"
    echo "    4. Copy development docs to docs/development/"
    echo ""
    echo -e "${BOLD}EXAMPLES:${NC}"
    echo "    mx icr                    # Initialize in current directory"
    echo "    mx icr ./my-project       # Initialize in specific directory"
    echo "    mx icr --skip-docs        # Skip documentation copy"
    echo ""
}
