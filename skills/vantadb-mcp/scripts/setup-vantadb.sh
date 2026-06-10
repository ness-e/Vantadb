#!/bin/bash
# Setup script for VantaDB MCP server
# This script installs VantaDB and configures it for MCP usage

set -e

VANTADB_VERSION="0.1.4"
INSTALL_DIR="${HOME}/.vantadb"
CONFIG_FILE="${INSTALL_DIR}/config.json"

echo "🚀 Setting up VantaDB MCP server..."

# Create installation directory
mkdir -p "${INSTALL_DIR}"

# Check if VantaDB is already installed
if command -v vanta-server &> /dev/null; then
    echo "✅ VantaDB is already installed"
    vanta-server --version
else
    echo "📦 Installing VantaDB..."
    # Install via cargo if Rust is available
    if command -v cargo &> /dev/null; then
        cargo install vantadb --version ${VANTADB_VERSION}
    else
        echo "❌ Rust/Cargo not found. Please install Rust first:"
        echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
fi

# Create default configuration
echo "📝 Creating default configuration..."
cat > "${CONFIG_FILE}" << EOF
{
  "storage_path": "${INSTALL_DIR}/data",
  "memory_limit_bytes": 512000000,
  "max_blocking_threads": 4,
  "read_only": false,
  "hnsw": {
    "m": 16,
    "ef_construction": 200,
    "ef_search": 50
  }
}
EOF

# Create data directory
mkdir -p "${INSTALL_DIR}/data"

echo "✅ VantaDB MCP server setup complete!"
echo ""
echo "📋 Configuration:"
echo "   Install directory: ${INSTALL_DIR}"
echo "   Config file: ${CONFIG_FILE}"
echo "   Data directory: ${INSTALL_DIR}/data"
echo ""
echo "🎯 To start the MCP server:"
echo "   vanta-server --mcp --path ${INSTALL_DIR}"
echo ""
echo "📚 For more information, see: https://docs.vantadb.io"
