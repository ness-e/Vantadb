#!/bin/sh
set -e

# VantaDB installer for Linux and macOS.
# Downloads the precompiled vanta-cli binary and puts it in ~/.vanta/bin

INSTALL_DIR="$HOME/.vanta/bin"
BINARY_NAME="vanta-cli"

# Detect OS and architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

# Normalize architecture names from uname to our conventions
case "$ARCH" in
  x86_64|amd64)
    ARCH_NORM="amd64"
    ;;
  aarch64|arm64)
    ARCH_NORM="arm64"
    ;;
  *)
    echo "❌ Unsupported architecture: $ARCH"
    echo "Supported architectures: x86_64 (amd64), aarch64 (arm64)"
    exit 1
    ;;
esac

case "$OS" in
  linux*)
    SUFFIX="linux-$ARCH_NORM"
    ;;
  darwin*)
    SUFFIX="macos-$ARCH_NORM"
    ;;
  *)
    echo "❌ Unsupported Operating System: $OS"
    exit 1
    ;;
esac

# Fetch the latest release tag from GitHub API
echo "🔍 Fetching latest VantaDB release version..."
LATEST_RELEASE=$(curl -sL https://api.github.com/repos/ness-e/Vantadb/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_RELEASE" ]; then
  # Fallback version if API fails
  LATEST_RELEASE="v0.1.4"
  echo "⚠️ Could not fetch latest release via API. Falling back to default version $LATEST_RELEASE"
fi

DOWNLOAD_URL="https://github.com/ness-e/Vantadb/releases/download/$LATEST_RELEASE/vanta-cli-$SUFFIX"

echo "📥 Downloading VantaDB CLI ($LATEST_RELEASE) for $SUFFIX..."
mkdir -p "$INSTALL_DIR"

if ! curl -L -f -o "$INSTALL_DIR/$BINARY_NAME" "$DOWNLOAD_URL"; then
  echo "❌ Failed to download binary from $DOWNLOAD_URL"
  echo "Please check if a release exists for tag $LATEST_RELEASE"
  exit 1
fi

chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "✨ VantaDB CLI successfully installed to $INSTALL_DIR/$BINARY_NAME"
echo ""
echo "💡 To use it immediately, add it to your PATH:"
echo "   export PATH=\"\$PATH:$INSTALL_DIR\""
echo ""
echo "To make this change permanent, add that line to your ~/.bashrc or ~/.zshrc."
