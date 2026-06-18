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
if command -v vantadb-server &> /dev/null; then
    echo "✅ VantaDB is already installed"
    vantadb-server --version
else
    echo "📦 Installing VantaDB from local repository..."
    # Install via cargo from local path if Rust is available
    if command -v cargo &> /dev/null; then
        # Detect script location to find VantaDB repository
        SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
        VANTADB_REPO="$(dirname "$(dirname "$(dirname "$SCRIPT_DIR")")")"

        echo "   Installing from: ${VANTADB_REPO}"
        cargo install --path "${VANTADB_REPO}/vantadb-server"
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
echo "   vantadb-server --mcp --path ${INSTALL_DIR}"
echo ""
echo "📚 For more information, see the documentation in the docs/ directory of the VantaDB repository."
