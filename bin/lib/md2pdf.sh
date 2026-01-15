#!/bin/bash
#
# MechCrate md2pdf Library
# Markdown to PDF conversion with Mermaid diagram support
#

# ─────────────────────────────────────────────────────────────────────────────
# md2pdf Command Handler
# ─────────────────────────────────────────────────────────────────────────────
md2pdf_cmd() {
    local input=""
    local output=""
    local title=""
    local subtitle=""
    local author=""
    local theme="dark"
    local order=""
    local markdown_only=0
    local html_only=0
    local no_toc=0
    local no_numbers=0
    local show_help=0
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -o|--output)
                output="$2"
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
            --author)
                author="$2"
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
            --no-toc)
                no_toc=1
                shift
                ;;
            --no-numbers)
                no_numbers=1
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
    
    # Show help if requested or no input
    if [ "$show_help" -eq 1 ] || [ -z "$input" ]; then
        show_md2pdf_help
        return 0
    fi
    
    # Check dependencies
    if ! check_md2pdf_deps; then
        return 1
    fi
    
    # Resolve input path
    if [[ "$input" != /* ]]; then
        input="$(pwd)/$input"
    fi
    
    # Build arguments for the TypeScript tool
    local args=("$input")
    
    [ -n "$output" ] && args+=("--output" "$output")
    [ -n "$title" ] && args+=("--title" "$title")
    [ -n "$subtitle" ] && args+=("--subtitle" "$subtitle")
    [ -n "$author" ] && args+=("--author" "$author")
    [ -n "$theme" ] && args+=("--theme" "$theme")
    [ -n "$order" ] && args+=("--order" "$order")
    [ "$markdown_only" -eq 1 ] && args+=("--markdown-only")
    [ "$html_only" -eq 1 ] && args+=("--html-only")
    [ "$no_toc" -eq 1 ] && args+=("--no-toc")
    [ "$no_numbers" -eq 1 ] && args+=("--no-numbers")
    
    # Run the tool
    local md2pdf_dir="$TEMPLATES_DIR/scripts/md2pdf"
    
    # Ensure dependencies are installed
    if [ ! -d "$md2pdf_dir/node_modules" ]; then
        info "Installing md2pdf dependencies..."
        (cd "$md2pdf_dir" && npm install --silent) || {
            error "Failed to install md2pdf dependencies"
        }
    fi
    
    # Run the converter
    (cd "$md2pdf_dir" && npx tsx md2pdf.ts "${args[@]}")
}

# ─────────────────────────────────────────────────────────────────────────────
# Check Dependencies
# ─────────────────────────────────────────────────────────────────────────────
check_md2pdf_deps() {
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
    
    # Check for Pandoc (optional but recommended)
    if ! command -v pandoc &> /dev/null; then
        warn "Pandoc not found. PDF generation will be limited."
        echo "   💡 Install with: brew install pandoc"
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
show_md2pdf_help() {
    echo ""
    echo -e "${BOLD}mx md2pdf${NC} - Markdown to PDF with Mermaid diagrams"
    echo ""
    echo -e "${BOLD}USAGE${NC}"
    echo "    mx md2pdf <input> [options]"
    echo ""
    echo -e "${BOLD}ARGUMENTS${NC}"
    echo "    <input>    Markdown file or directory of markdown files"
    echo ""
    echo -e "${BOLD}OPTIONS${NC}"
    echo "    -o, --output <path>     Output PDF path"
    echo "    --title <title>         Document title"
    echo "    --subtitle <subtitle>   Document subtitle"
    echo "    --author <author>       Document author (default: MechCrate)"
    echo "    --theme <theme>         Mermaid theme: dark, light, forest, neutral"
    echo "    --order <files>         File order for directories (comma-separated)"
    echo "    --markdown-only         Only generate processed markdown"
    echo "    --html-only             Only generate HTML"
    echo "    --no-toc                Disable table of contents"
    echo "    --no-numbers            Disable section numbering"
    echo "    -h, --help              Show this help"
    echo ""
    echo -e "${BOLD}EXAMPLES${NC}"
    echo "    mx md2pdf docs/README.md"
    echo "    mx md2pdf docs/guide/ --title \"User Guide\""
    echo "    mx md2pdf docs/spec.md -o artifacts/spec.pdf"
    echo "    mx md2pdf docs/api/ --order \"intro.md,endpoints.md\""
    echo "    mx md2pdf docs/whitepaper.md --theme light"
    echo ""
    echo -e "${BOLD}OUTPUT${NC}"
    echo "    Creates an output directory containing:"
    echo "    • PDF file"
    echo "    • Processed markdown"
    echo "    • HTML version"
    echo "    • Rendered Mermaid diagrams (as PNG)"
    echo ""
    echo -e "${BOLD}FEATURES${NC}"
    echo "    • Renders Mermaid diagrams as high-resolution images"
    echo "    • Supports dark/light themes for diagrams"
    echo "    • Generates table of contents"
    echo "    • Syntax highlighting for code blocks"
    echo "    • Combines multiple markdown files from directories"
    echo ""
    echo -e "${BOLD}DEPENDENCIES${NC}"
    echo "    Required: Node.js 18+, npm"
    echo "    For PDF: Pandoc (brew install pandoc)"
    echo "    Best PDF: XeLaTeX (brew install --cask mactex-no-gui)"
    echo ""
}
