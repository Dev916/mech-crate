#!/bin/bash
#
# MechCrate Install Script
# Adds mx command to your PATH
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MX_BIN="$SCRIPT_DIR/bin/mx"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}"
cat << 'EOF'
    ╭──────────────────────────────────╮
    │  🦝 MechCrate Installer          │
    ╰──────────────────────────────────╯
EOF
echo -e "${NC}"

# Check if mx exists
if [[ ! -f "$MX_BIN" ]]; then
    echo "Error: mx not found at $MX_BIN"
    exit 1
fi

# Make sure it's executable
chmod +x "$MX_BIN"

# Determine install method
INSTALL_DIR="/usr/local/bin"
SYMLINK_PATH="$INSTALL_DIR/mx"

# Check if we can write to /usr/local/bin
if [[ -w "$INSTALL_DIR" ]] || [[ -w "$(dirname "$INSTALL_DIR")" ]]; then
    # Create symlink
    if [[ -L "$SYMLINK_PATH" ]]; then
        rm "$SYMLINK_PATH"
    fi
    ln -sf "$MX_BIN" "$SYMLINK_PATH"
    echo -e "${GREEN}✓${NC} Installed mx to $SYMLINK_PATH"
    echo ""
    echo "You can now use 'mx' from anywhere!"
else
    # Need sudo
    echo "Installing to $INSTALL_DIR requires sudo..."
    sudo ln -sf "$MX_BIN" "$SYMLINK_PATH"
    echo -e "${GREEN}✓${NC} Installed mx to $SYMLINK_PATH"
    echo ""
    echo "You can now use 'mx' from anywhere!"
fi

# Verify installation
if command -v mx &> /dev/null; then
    echo ""
    echo -e "${GREEN}✓${NC} Installation verified!"
    mx version
else
    echo ""
    echo -e "${YELLOW}⚠${NC} mx installed but not found in current shell."
    echo "   Try opening a new terminal or run: hash -r"
fi

echo ""
echo -e "${CYAN}🦝 Crate Raccoon says: Ready to scaffold!${NC}"
echo ""
echo "Quick start:"
echo "  mx new my-project    # Create a new project"
echo "  mx help              # Show all commands"
