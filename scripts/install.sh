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
LATEST_RELEASE=$(curl -sL --ssl-reqd https://api.github.com/repos/ness-e/Vantadb/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_RELEASE" ]; then
  echo "❌ Could not fetch latest release version from GitHub API."
  echo "Please check your internet connection or visit https://github.com/ness-e/Vantadb/releases"
  exit 1
fi

DOWNLOAD_URL="https://github.com/ness-e/Vantadb/releases/download/$LATEST_RELEASE/vanta-cli-$SUFFIX"
CHECKSUM_URL="https://github.com/ness-e/Vantadb/releases/download/$LATEST_RELEASE/vanta-cli-$SUFFIX.sha256"

echo "📥 Downloading VantaDB CLI ($LATEST_RELEASE) for $SUFFIX..."
mkdir -p "$INSTALL_DIR"

if ! curl -L -f --ssl-reqd -o "$INSTALL_DIR/$BINARY_NAME" "$DOWNLOAD_URL"; then
  echo "❌ Failed to download binary from $DOWNLOAD_URL"
  echo "Please check if a release exists for tag $LATEST_RELEASE"
  exit 1
fi

# Verify checksum if available (optional — GitHub releases may not include .sha256 files)
if EXPECTED_HASH=$(curl -sLf --ssl-reqd "$CHECKSUM_URL" 2>/dev/null); then
  COMPUTED_HASH=$(sha256sum "$INSTALL_DIR/$BINARY_NAME" | cut -d' ' -f1)
  if [ "$EXPECTED_HASH" != "$COMPUTED_HASH" ]; then
    echo "❌ Checksum mismatch! Expected: $EXPECTED_HASH"
    echo "  Computed: $COMPUTED_HASH"
    rm -f "$INSTALL_DIR/$BINARY_NAME"
    exit 1
  fi
  echo "✅ Checksum verified"
else
  echo "⚠️ No checksum file found at $CHECKSUM_URL — skipping verification"
fi

chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "✨ VantaDB CLI successfully installed to $INSTALL_DIR/$BINARY_NAME"
echo ""
echo "💡 To use it immediately, add it to your PATH:"
echo "   export PATH=\"\$PATH:$INSTALL_DIR\""
echo ""
echo "To make this change permanent, add that line to your ~/.bashrc or ~/.zshrc."
