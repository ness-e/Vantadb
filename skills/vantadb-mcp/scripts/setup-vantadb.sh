#!/bin/bash
# Setup script for VantaDB MCP server
# Installs the vanta-cli binary and exports the real VANTADB_* env vars
# (src/config.rs reads configuration from environment, not from a JSON file).

set -e

# The version is read from the workspace Cargo.toml, never hard-coded: the
# script installs from the local checkout, so the manifest is the source of
# truth (a pinned literal goes stale on every release).
INSTALL_DIR="${HOME}/.vantadb"

# This script lives at <repo>/skills/vantadb-mcp/scripts (and at
# <repo>/.opencode/skills/vantadb-mcp/scripts in the editor mirror), so the
# repo root is NOT a fixed number of "dirname" hops away: walk up until the
# workspace Cargo.toml shows up.
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
VANTADB_REPO="$(dirname "$(dirname "$(dirname "$SCRIPT_DIR")")")"
while [ ! -f "${VANTADB_REPO}/Cargo.toml" ] && [ "${VANTADB_REPO}" != "/" ] && [ "${VANTADB_REPO}" != "." ]; do
    VANTADB_REPO="$(dirname "${VANTADB_REPO}")"
done
VANTADB_VERSION="unknown"
if [ -f "${VANTADB_REPO}/Cargo.toml" ]; then
    VANTADB_VERSION="$(grep -A 20 '^\[workspace\.package\]' "${VANTADB_REPO}/Cargo.toml" | grep -m1 '^version = ' | sed 's/.*"\(.*\)".*/\1/')"
fi

echo "🚀 Setting up VantaDB MCP server (v${VANTADB_VERSION})..."

# Check if vanta-cli is already installed
if command -v vanta-cli &> /dev/null; then
    echo "✅ vanta-cli already installed"
    vanta-cli --version
else
    echo "📦 Installing vanta-cli from local repository..."
    # Install via cargo from local path if Rust is available
    if command -v cargo &> /dev/null; then
        if [ ! -f "${VANTADB_REPO}/Cargo.toml" ]; then
            echo "❌ No encuentro el raiz del repo VantaDB (Cargo.toml) desde ${SCRIPT_DIR}"
            exit 1
        fi

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