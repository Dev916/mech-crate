#!/bin/bash
#
# MechCrate Install Script
# Builds and installs the mx CLI globally
#
# Usage:
#   ./scripts/install.sh              # Install to /usr/local/bin
#   ./scripts/install.sh --prefix ~   # Install to ~/bin
#   ./scripts/install.sh --local      # Install to ~/.local/bin
#

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Defaults
PREFIX="${PREFIX:-/usr/local}"
INSTALL_DIR=""
SKIP_BUILD=false
SKIP_INIT=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --prefix)
            PREFIX="$2"
            shift 2
            ;;
        --local)
            PREFIX="${HOME}/.local"
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --skip-init)
            SKIP_INIT=true
            shift
            ;;
        --help|-h)
            echo "MechCrate Install Script"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --prefix DIR    Install to DIR/bin (default: /usr/local)"
            echo "  --local         Install to ~/.local/bin"
            echo "  --skip-build    Skip cargo build (use existing binaries)"
            echo "  --skip-init     Skip mx init (don't copy templates)"
            echo "  --help          Show this help"
            echo ""
            echo "Examples:"
            echo "  $0                      # Install to /usr/local/bin (may need sudo)"
            echo "  $0 --local              # Install to ~/.local/bin"
            echo "  $0 --prefix ~/opt       # Install to ~/opt/bin"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

INSTALL_DIR="${PREFIX}/bin"

# Detect script directory and mech-crate root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MECH_CRATE_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${CYAN}"
cat << 'EOF'
    ╭──────────────────────────────────────────────────╮
    │  🦝 MechCrate CLI Installer                      │
    ╰──────────────────────────────────────────────────╯
EOF
echo -e "${NC}"

echo -e "  ${BOLD}Install Directory:${NC} ${INSTALL_DIR}"
echo -e "  ${BOLD}MechCrate Root:${NC} ${MECH_CRATE_ROOT}"
echo ""

# Check for cargo
if ! command -v cargo &>/dev/null; then
    echo -e "${RED}Error: cargo not found. Install Rust first: https://rustup.rs/${NC}"
    exit 1
fi

# Build if needed
if [[ "$SKIP_BUILD" == "false" ]]; then
    echo -e "${CYAN}→${NC} Building release binaries..."
    (cd "$MECH_CRATE_ROOT" && cargo build --release -p mx-cli -p mx-mcp-server)
    echo -e "${GREEN}✓${NC} Build complete"
    echo ""
fi

# Check binaries exist
MX_BIN="$MECH_CRATE_ROOT/target/release/mx"
MCP_BIN="$MECH_CRATE_ROOT/target/release/mx-mcp"
INGEST_BIN="$MECH_CRATE_ROOT/target/release/mx-ingest"

if [[ ! -f "$MX_BIN" ]]; then
    echo -e "${RED}Error: mx binary not found at $MX_BIN${NC}"
    echo "Run: cargo build --release -p mx-cli"
    exit 1
fi

# Create install directory if needed
if [[ ! -d "$INSTALL_DIR" ]]; then
    echo -e "${CYAN}→${NC} Creating ${INSTALL_DIR}..."
    mkdir -p "$INSTALL_DIR" 2>/dev/null || sudo mkdir -p "$INSTALL_DIR"
fi

# Install binaries
echo -e "${CYAN}→${NC} Installing binaries to ${INSTALL_DIR}..."

install_binary() {
    local src="$1"
    local dst="$2"
    
    if [[ -f "$src" ]]; then
        if cp "$src" "$dst" 2>/dev/null; then
            chmod +x "$dst"
            echo -e "  ${GREEN}✓${NC} Installed $(basename "$dst")"
        elif sudo cp "$src" "$dst" 2>/dev/null; then
            sudo chmod +x "$dst"
            echo -e "  ${GREEN}✓${NC} Installed $(basename "$dst") (with sudo)"
        else
            echo -e "  ${RED}✗${NC} Failed to install $(basename "$dst")"
            return 1
        fi
    fi
}

install_binary "$MX_BIN" "$INSTALL_DIR/mx"
[[ -f "$MCP_BIN" ]] && install_binary "$MCP_BIN" "$INSTALL_DIR/mx-mcp"
[[ -f "$INGEST_BIN" ]] && install_binary "$INGEST_BIN" "$INSTALL_DIR/mx-ingest"

echo ""

# Initialize MechCrate if needed
if [[ "$SKIP_INIT" == "false" ]]; then
    echo -e "${CYAN}→${NC} Initializing MechCrate templates..."
    
    # Use the newly installed binary
    if command -v "$INSTALL_DIR/mx" &>/dev/null; then
        MECH_CRATE_ROOT="$MECH_CRATE_ROOT" "$INSTALL_DIR/mx" init --force
    else
        MECH_CRATE_ROOT="$MECH_CRATE_ROOT" "$MX_BIN" init --force
    fi
    echo ""
fi

# Verify installation
echo -e "${CYAN}→${NC} Verifying installation..."

if command -v mx &>/dev/null; then
    MX_PATH=$(command -v mx)
    MX_VERSION=$(mx --version 2>/dev/null || echo "unknown")
    echo -e "  ${GREEN}✓${NC} mx found at: $MX_PATH"
    echo -e "  ${GREEN}✓${NC} Version: $MX_VERSION"
else
    echo -e "  ${YELLOW}!${NC} mx not in PATH"
    echo ""
    echo -e "  Add to your shell profile:"
    echo ""
    if [[ "$INSTALL_DIR" == "${HOME}/.local/bin" ]]; then
        echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
    elif [[ "$INSTALL_DIR" == "${HOME}/bin" ]]; then
        echo "    export PATH=\"\$HOME/bin:\$PATH\""
    else
        echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi
fi

echo ""
echo -e "${GREEN}${BOLD}✓ Installation complete!${NC}"
echo ""
echo -e "  Quick start:"
echo -e "    ${CYAN}mx doctor${NC}         Check installation"
echo -e "    ${CYAN}mx recipes list${NC}   List available recipes"
echo -e "    ${CYAN}mx new myproject${NC}  Create a new project"
echo ""
