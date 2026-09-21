#!/bin/bash
# Setup script for VantaDB MCP server
# Installs the vanta-cli binary and exports the real VANTADB_* env vars
# (src/config.rs reads configuration from environment, not from a JSON file).

set -e

VANTADB_VERSION="0.5.0"
INSTALL_DIR="${HOME}/.vantadb"

echo "🚀 Setting up VantaDB MCP server (v${VANTADB_VERSION})..."

# Check if vanta-cli is already installed
if command -v vanta-cli &> /dev/null; then
    echo "✅ vanta-cli already installed"
    vanta-cli --version
else
    echo "📦 Installing vanta-cli from local repository..."
    # Install via cargo from local path if Rust is available
    if command -v cargo &> /dev/null; then
        # Detect script location to find VantaDB repository
        SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
        VANTADB_REPO="$(dirname "$(dirname "$(dirname "$SCRIPT_DIR")")")"

        echo "   Installing from: ${VANTADB_REPO}"
        cargo install --manifest-path "${VANTADB_REPO}/Cargo.toml" --bin vanta-cli
    else
        echo "❌ Rust/Cargo not found. Please install Rust first:"
        echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
fi

# Create data directory
mkdir -p "${INSTALL_DIR}"

# VantaDB configuration is read from env vars (src/config.rs). Export the
# real variables so the MCP server picks them up when launched below.
export VANTADB_STORAGE_PATH="${INSTALL_DIR}"
export VANTADB_MEMORY_LIMIT="512MB"

echo "✅ VantaDB MCP server setup complete!"
echo ""
echo "📋 Configuration (env vars):"
echo "   VANTADB_STORAGE_PATH: ${VANTADB_STORAGE_PATH}"
echo "   VANTADB_MEMORY_LIMIT: ${VANTADB_MEMORY_LIMIT}"
echo ""
echo "🎯 To start the MCP server:"
echo "   vanta-cli server --mcp --db ${INSTALL_DIR}"
echo ""
echo "📚 For more information, see the documentation in the docs/ directory of the VantaDB repository."