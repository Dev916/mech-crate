#!/bin/bash
#
# MechCrate Docs Library
# Portable Markdown to PDF conversion - just needs Node.js
#

# ─────────────────────────────────────────────────────────────────────────────
# Docs Command Handler
# ─────────────────────────────────────────────────────────────────────────────
docs_cmd() {
    local input=""
    local output=""
    local prefix=""
    local author=""
    local title=""
    local subtitle=""
    local markdown_only=0
    local html_only=0
    local verbose=0
    local no_recursive=0
    local no_toc=0
    local theme="dark"
    local show_help=0
    local list_unyform=0
    local compile_unyform=0
    local unyform_doc=""
    local order=""
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -o|--output)
                output="$2"
                shift 2
                ;;
            --prefix)
                prefix="$2"
                shift 2
                ;;
            --author)
                author="$2"
                shift 2
                ;;
            --title)
                title="$2"
                shift 2
                ;;
            --subtitle)
                subtitle="$2"
                shift 2
                ;;
            --theme)
                theme="$2"
                shift 2
                ;;
            --order)
                order="$2"
                shift 2
                ;;
            --markdown-only)
                markdown_only=1
                shift
                ;;
            --html-only)
                html_only=1
                shift
                ;;
            --no-recursive)
                no_recursive=1
                shift
                ;;
            --no-toc)
                no_toc=1
                shift
                ;;
            -v|--verbose)
                verbose=1
                shift
                ;;
            --list)
                list_unyform=1
                shift
                ;;
            --all|--unyform)
                compile_unyform=1
                shift
                ;;
            --doc)
                unyform_doc="$2"
                shift 2
                ;;
            --doc=*)
                unyform_doc="${1#*=}"
                shift
                ;;
            -h|--help)
                show_help=1
                shift
                ;;
            -*)
                error "Unknown option: $1"
                ;;
            *)
                # First positional arg is input
                if [ -z "$input" ]; then
                    input="$1"
                fi
                shift
                ;;
        esac
    done
    
    # Show help if requested
    if [ "$show_help" -eq 1 ]; then
        show_docs_help
        return 0
    fi
    
    # Check for Node.js (only required dependency)
    if ! check_node; then
        return 1
    fi
    
    # Ensure compile script dependencies are installed
    local docs_script_dir="$MECH_CRATE_ROOT/scripts/docs"
    if [ ! -d "$docs_script_dir/node_modules" ]; then
        info "Installing documentation dependencies (first run)..."
        (cd "$docs_script_dir" && npm install --silent) || {
            error "Failed to install documentation dependencies"
        }
        success "Dependencies installed"
    fi
    
    # Build arguments for the TypeScript tool
    local args=()
    
    # Handle list command
    if [ "$list_unyform" -eq 1 ]; then
        args+=("--list")
        (cd "$docs_script_dir" && npx tsx compile.ts "${args[@]}")
        return $?
    fi
    
    # Handle unyform compilation
    if [ "$compile_unyform" -eq 1 ]; then
        args+=("--all")
        [ -n "$output" ] && args+=("--output=$(realpath "$output" 2>/dev/null || echo "$output")")
        [ "$verbose" -eq 1 ] && args+=("--verbose")
        [ "$markdown_only" -eq 1 ] && args+=("--markdown-only")
        [ "$html_only" -eq 1 ] && args+=("--html-only")
        (cd "$docs_script_dir" && npx tsx compile.ts "${args[@]}")
        return $?
    fi
    
    # Handle specific unyform doc
    if [ -n "$unyform_doc" ]; then
        args+=("--doc=$unyform_doc")
        [ -n "$output" ] && args+=("--output=$(realpath "$output" 2>/dev/null || echo "$output")")
        [ "$verbose" -eq 1 ] && args+=("--verbose")
        [ "$markdown_only" -eq 1 ] && args+=("--markdown-only")
        [ "$html_only" -eq 1 ] && args+=("--html-only")
        (cd "$docs_script_dir" && npx tsx compile.ts "${args[@]}")
        return $?
    fi
    
    # If no input specified and not a special command, show help
    if [ -z "$input" ]; then
        show_docs_help
        return 0
    fi
    
    # Resolve input path
    if [[ "$input" != /* ]]; then
        input="$(pwd)/$input"
    fi
    
    # Add input (file or directory)
    args+=("$input")
    
    # Add common options
    [ -n "$output" ] && args+=("--output=$(realpath "$output" 2>/dev/null || echo "$output")")
    [ -n "$prefix" ] && args+=("--prefix=$prefix")
    [ -n "$author" ] && args+=("--author=$author")
    [ -n "$title" ] && args+=("--title=$title")
    [ -n "$subtitle" ] && args+=("--subtitle=$subtitle")
    [ -n "$theme" ] && args+=("--theme=$theme")
    [ -n "$order" ] && args+=("--order=$order")
    [ "$verbose" -eq 1 ] && args+=("--verbose")
    [ "$markdown_only" -eq 1 ] && args+=("--markdown-only")
    [ "$html_only" -eq 1 ] && args+=("--html-only")
    [ "$no_recursive" -eq 1 ] && args+=("--no-recursive")
    [ "$no_toc" -eq 1 ] && args+=("--no-toc")
    
    # Run the converter
    (cd "$docs_script_dir" && npx tsx compile.ts "${args[@]}")
}

# ─────────────────────────────────────────────────────────────────────────────
# Check Node.js (only required dependency)
# ─────────────────────────────────────────────────────────────────────────────
check_node() {
    # Check for Node.js
    if ! command -v node &> /dev/null; then
        error "Node.js not found."
        echo "   💡 Install with: brew install node"
        return 1
    fi
    
    # Check Node.js version
    local node_version=$(node -v | sed 's/v//' | cut -d. -f1)
    if [ "$node_version" -lt 18 ]; then
        error "Node.js 18+ required. Current version: $(node -v)"
        return 1
    fi
    
    # Check for npm
    if ! command -v npm &> /dev/null; then
        error "npm not found. Install Node.js to get npm."
        return 1
    fi
    
    return 0
}

# ─────────────────────────────────────────────────────────────────────────────
# Help
# ─────────────────────────────────────────────────────────────────────────────
show_docs_help() {
    echo ""
    echo -e "${BOLD}mx docs${NC} - Portable Markdown to PDF Compiler"
    echo ""
    echo "  Just needs Node.js - no other system dependencies required!"
    echo "  PDF generation via bundled Chromium - always works!"
    echo ""
    echo -e "${BOLD}USAGE${NC}"
    echo "    mx docs <input>              Convert file or folder to PDF"
    echo "    mx docs --all                Compile all unyform.ai documents"
    echo "    mx docs --doc=<name>         Compile specific unyform doc"
    echo "    mx docs --list               List available unyform documents"
    echo ""
    echo -e "${BOLD}ARGUMENTS${NC}"
    echo "    <input>    Markdown file (.md) or directory containing .md files"
    echo ""
    echo -e "${BOLD}OPTIONS${NC}"
    echo "    -o, --output <path>     Output directory for generated files"
    echo "    --title <title>         Document title"
    echo "    --subtitle <subtitle>   Document subtitle"
    echo "    --author <author>       Document author"
    echo "    --prefix <string>       Add prefix to output filenames"
    echo "    --theme <theme>         Mermaid theme: dark, light, forest, neutral"
    echo "    --order <files>         Comma-separated file order for directories"
    echo "    --markdown-only         Only generate processed markdown"
    echo "    --html-only             Only generate HTML (no PDF attempt)"
    echo "    --no-toc                Disable table of contents"
    echo "    --no-recursive          Don't scan subfolders (for directories)"
    echo "    -v, --verbose           Show detailed progress"
    echo "    -h, --help              Show this help"
    echo ""
    echo -e "${BOLD}UNYFORM COMMANDS${NC}"
    echo "    --all                   Compile all predefined unyform.ai documents"
    echo "    --doc=<name>            Compile a specific unyform document:"
    echo "                            whitepaper, executive-summary, roadmap,"
    echo "                            mvp-prd, pitch-deck, gtm-playbook,"
    echo "                            tech-architecture, pricing-strategy"
    echo "    --list                  List all available unyform documents"
    echo ""
    echo -e "${BOLD}FRONTMATTER${NC}"
    echo "    Documents can include YAML frontmatter for metadata:"
    echo ""
    echo "    ---"
    echo "    title: My Document"
    echo "    subtitle: Optional Subtitle"
    echo "    author: Author Name"
    echo "    toc: true"
    echo "    ---"
    echo ""
    echo -e "${BOLD}OUTPUT${NC}"
    echo "    artifacts/<name>/"
    echo "    ├── <name>.pdf      # PDF (always generated)"
    echo "    ├── <name>.html     # HTML version"
    echo "    ├── <name>.md       # Processed markdown"
    echo "    └── diagrams/       # Rendered Mermaid PNGs"
    echo ""
    echo -e "${BOLD}EXAMPLES${NC}"
    echo ""
    echo "    # Single file"
    echo "    mx docs docs/README.md"
    echo "    mx docs docs/spec.md -o artifacts/"
    echo ""
    echo "    # Folder (all .md files)"
    echo "    mx docs docs/guides/"
    echo "    mx docs docs/api/ --title \"API Documentation\""
    echo ""
    echo "    # unyform.ai documents"
    echo "    mx docs --all                    # Compile all"
    echo "    mx docs --doc=whitepaper         # Compile specific"
    echo "    mx docs --list                   # List available"
    echo ""
    echo -e "${BOLD}DEPENDENCIES${NC}"
    echo "    Required: Node.js 18+ (npm)"
    echo ""
    echo "    That's it! PDF generation uses bundled Chromium."
    echo "    No Pandoc, LaTeX, or other system tools needed."
    echo ""
}
