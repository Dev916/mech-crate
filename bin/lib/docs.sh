#!/bin/bash
#
# MechCrate Docs Library
# Markdown to PDF conversion with Mermaid diagram support
#

# ─────────────────────────────────────────────────────────────────────────────
# Docs Command Handler
# ─────────────────────────────────────────────────────────────────────────────
docs_cmd() {
    local input=""
    local output=""
    local prefix=""
    local author=""
    local markdown_only=0
    local verbose=0
    local no_recursive=0
    local show_help=0
    local list_unyform=0
    local compile_unyform=0
    local unyform_doc=""
    
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
            --markdown-only)
                markdown_only=1
                shift
                ;;
            --no-recursive)
                no_recursive=1
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
            --unyform)
                compile_unyform=1
                shift
                ;;
            --doc)
                unyform_doc="$2"
                shift 2
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
    
    # Check dependencies
    if ! check_docs_deps; then
        return 1
    fi
    
    # Ensure compile script dependencies are installed
    local docs_script_dir="$MECH_CRATE_ROOT/scripts/docs"
    if [ ! -d "$docs_script_dir/node_modules" ]; then
        info "Installing documentation dependencies..."
        (cd "$docs_script_dir" && npm install --silent) || {
            error "Failed to install documentation dependencies"
        }
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
        (cd "$docs_script_dir" && npx tsx compile.ts "${args[@]}")
        return $?
    fi
    
    # Handle specific unyform doc
    if [ -n "$unyform_doc" ]; then
        args+=("--doc=$unyform_doc")
        [ -n "$output" ] && args+=("--output=$(realpath "$output" 2>/dev/null || echo "$output")")
        [ "$verbose" -eq 1 ] && args+=("--verbose")
        [ "$markdown_only" -eq 1 ] && args+=("--markdown-only")
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
    
    # Determine if input is file or folder
    if [ -d "$input" ]; then
        args+=("--folder=$input")
        [ "$no_recursive" -eq 1 ] && args+=("--no-recursive")
    elif [ -f "$input" ]; then
        args+=("--file=$input")
    else
        error "Input not found: $input"
    fi
    
    # Add common options
    [ -n "$output" ] && args+=("--output=$(realpath "$output" 2>/dev/null || echo "$output")")
    [ -n "$prefix" ] && args+=("--prefix=$prefix")
    [ -n "$author" ] && args+=("--author=$author")
    [ "$verbose" -eq 1 ] && args+=("--verbose")
    [ "$markdown_only" -eq 1 ] && args+=("--markdown-only")
    
    # Run the converter
    (cd "$docs_script_dir" && npx tsx compile.ts "${args[@]}")
}

# ─────────────────────────────────────────────────────────────────────────────
# Check Dependencies
# ─────────────────────────────────────────────────────────────────────────────
check_docs_deps() {
    local missing=0
    
    # Check for Node.js
    if ! command -v node &> /dev/null; then
        warn "Node.js not found. Install with: brew install node"
        missing=1
    else
        # Check Node.js version
        local node_version=$(node -v | sed 's/v//' | cut -d. -f1)
        if [ "$node_version" -lt 18 ]; then
            warn "Node.js 18+ required. Current version: $(node -v)"
            missing=1
        fi
    fi
    
    # Check for npm
    if ! command -v npm &> /dev/null; then
        warn "npm not found. Install Node.js to get npm."
        missing=1
    fi
    
    # Check for Pandoc
    if ! command -v pandoc &> /dev/null; then
        warn "Pandoc not found. PDF generation requires Pandoc."
        echo "   💡 Install with: brew install pandoc"
        missing=1
    fi
    
    # Check for XeLaTeX (optional but recommended)
    if ! command -v xelatex &> /dev/null; then
        warn "XeLaTeX not found. PDF generation may be limited."
        echo "   💡 Install with: brew install --cask mactex-no-gui"
    fi
    
    if [ "$missing" -eq 1 ]; then
        echo ""
        error "Missing required dependencies. Please install them first."
    fi
    
    return 0
}

# ─────────────────────────────────────────────────────────────────────────────
# Help
# ─────────────────────────────────────────────────────────────────────────────
show_docs_help() {
    echo ""
    echo -e "${BOLD}mx docs${NC} - Markdown to PDF with Mermaid diagrams"
    echo ""
    echo -e "${BOLD}USAGE${NC}"
    echo "    mx docs <input>              Convert file or folder to PDF"
    echo "    mx docs --unyform            Compile all unyform.ai documents"
    echo "    mx docs --doc=<name>         Compile specific unyform doc"
    echo "    mx docs --list               List available unyform documents"
    echo ""
    echo -e "${BOLD}ARGUMENTS${NC}"
    echo "    <input>    Markdown file (.md) or directory containing .md files"
    echo ""
    echo -e "${BOLD}OPTIONS${NC}"
    echo "    -o, --output <path>     Output directory for generated PDFs"
    echo "    --prefix <string>       Add prefix to output filenames"
    echo "    --author <author>       Default author for docs without frontmatter"
    echo "    --markdown-only         Only generate processed markdown, no PDF"
    echo "    --no-recursive          Don't scan subfolders (for directories)"
    echo "    -v, --verbose           Show detailed progress"
    echo "    -h, --help              Show this help"
    echo ""
    echo -e "${BOLD}UNYFORM COMMANDS${NC}"
    echo "    --unyform               Compile all predefined unyform.ai documents"
    echo "    --doc=<name>            Compile a specific unyform document:"
    echo "                            whitepaper, executive-summary, roadmap,"
    echo "                            competitive-analysis, mvp-prd, pitch-deck,"
    echo "                            gtm-playbook, tech-architecture, pricing-strategy"
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
    echo "    date: January 2025"
    echo "    ---"
    echo ""
    echo -e "${BOLD}EXAMPLES${NC}"
    echo ""
    echo "    # Single file"
    echo "    mx docs docs/README.md"
    echo "    mx docs docs/spec.md -o artifacts/"
    echo ""
    echo "    # Folder (all .md files)"
    echo "    mx docs docs/guides/"
    echo "    mx docs docs/api/ -o artifacts/api-docs/"
    echo "    mx docs ./specs --prefix=v2 --author=\"Engineering Team\""
    echo ""
    echo "    # unyform.ai documents"
    echo "    mx docs --unyform                    # Compile all"
    echo "    mx docs --doc=whitepaper             # Compile specific"
    echo "    mx docs --list                       # List available"
    echo ""
    echo -e "${BOLD}OUTPUT${NC}"
    echo "    For files:   ./output/<filename>.pdf"
    echo "    For folders: <folder>/output/<filename>.pdf"
    echo "    Custom:      Use -o to specify output directory"
    echo ""
    echo -e "${BOLD}FEATURES${NC}"
    echo "    • Renders Mermaid diagrams as high-resolution images"
    echo "    • Auto-detects title from frontmatter or filename"
    echo "    • Generates table of contents (configurable)"
    echo "    • Syntax highlighting for code blocks"
    echo "    • Recursive folder scanning"
    echo "    • Professional LaTeX-based PDF output"
    echo ""
    echo -e "${BOLD}DEPENDENCIES${NC}"
    echo "    Required: Node.js 18+, npm, Pandoc"
    echo "    Recommended: XeLaTeX (brew install --cask mactex-no-gui)"
    echo ""
}
