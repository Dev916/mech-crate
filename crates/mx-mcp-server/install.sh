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

echo ""
echo "✅ Installation complete!"
echo ""
echo "Binary installed to:"
echo "  $INSTALL_DIR/mx-mcp"
echo ""
echo "Next steps:"
echo ""
echo "1. Provide a Postgres + pgvector backend for the techniques corpus."
echo "   Set database_url (Neon) or fallback_database_url in"
echo "   ~/.mech-crate/config/rag.toml, or start a local one:"
echo "   docker run -d --name mx-rag -p 5432:5432 \\"
echo "     -e POSTGRES_DB=mx_rag -e POSTGRES_HOST_AUTH_METHOD=trust \\"
echo "     pgvector/pgvector:pg17"
echo ""
echo "2. Ingest documentation into the corpus:"
echo "   mx rag ingest"
echo ""
echo "3. Configure your MCP client (e.g., Claude Desktop):"
echo ""
cat << EOF
{
  "mcpServers": {
    "mechcrate": {
      "command": "$INSTALL_DIR/mx-mcp",
      "env": {
        "MECH_CRATE_ROOT": "$SCRIPT_DIR/.."
      }
    }
  }
}
EOF
echo ""
