#!/bin/bash
#
# MechCrate MCP Server Installation
#

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
INSTALL_DIR="${MX_MCP_INSTALL_DIR:-$HOME/.local/bin}"

echo "🦝 MechCrate MCP Server Installer"
echo ""

# Check for Rust
if ! command -v cargo &>/dev/null; then
    echo "❌ Rust/Cargo not found. Please install from https://rustup.rs"
    exit 1
fi

# Build release binary
echo "📦 Building release binary..."
cd "$SCRIPT_DIR"
cargo build --release

# Create install directory if needed
mkdir -p "$INSTALL_DIR"

# Copy binaries
echo "📋 Installing to $INSTALL_DIR..."
cp target/release/mx-mcp "$INSTALL_DIR/"
cp target/release/mx-ingest "$INSTALL_DIR/"

echo ""
echo "✅ Installation complete!"
echo ""
echo "Binaries installed to:"
echo "  $INSTALL_DIR/mx-mcp"
echo "  $INSTALL_DIR/mx-ingest"
echo ""
echo "Next steps:"
echo ""
echo "1. Start Weaviate (for RAG support):"
echo "   cd $SCRIPT_DIR && docker compose up -d"
echo ""
echo "2. Ingest documentation:"
echo "   $INSTALL_DIR/mx-ingest --mech-crate-root $SCRIPT_DIR/.."
echo ""
echo "3. Configure your MCP client (e.g., Claude Desktop):"
echo ""
cat << EOF
{
  "mcpServers": {
    "mechcrate": {
      "command": "$INSTALL_DIR/mx-mcp",
      "args": ["--weaviate-url", "http://localhost:8080"],
      "env": {
        "MECH_CRATE_ROOT": "$SCRIPT_DIR/.."
      }
    }
  }
}
EOF
echo ""
