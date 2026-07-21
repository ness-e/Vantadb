#!/bin/sh
set -e

# VantaDB installer for Linux and macOS.
# Downloads the release tarball and extracts vanta-cli to ~/.vanta/bin

INSTALL_DIR="$HOME/.vanta/bin"
BINARY_NAME="vanta-cli"

# Detect OS and architecture (Rust triple)
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
  x86_64|amd64)
    ARCH_NORM="x86_64"
    ;;
  aarch64|arm64)
    ARCH_NORM="aarch64"
    ;;
  *)
    echo "❌ Unsupported architecture: $ARCH"
    echo "Supported: x86_64 (amd64), aarch64 (arm64)"
    exit 1
    ;;
esac

case "$OS" in
  linux*)
    TARGET="$ARCH_NORM-unknown-linux-gnu"
    ;;
  darwin*)
    TARGET="$ARCH_NORM-apple-darwin"
    ;;
  *)
    echo "❌ Unsupported OS: $OS"
    echo "Supported: linux, darwin (macOS)"
    exit 1
    ;;
esac

# Fetch the latest release tag from GitHub API
echo "🔍 Fetching latest VantaDB release version..."
LATEST_RELEASE=$(curl -sL --ssl-reqd https://api.github.com/repos/ness-e/Vantadb/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_RELEASE" ]; then
  echo "❌ Could not fetch latest release version from GitHub API."
  echo "Visit https://github.com/ness-e/Vantadb/releases"
  exit 1
fi

TARBALL="vantadb-$TARGET.tar.gz"
DOWNLOAD_URL="https://github.com/ness-e/Vantadb/releases/download/$LATEST_RELEASE/$TARBALL"
CHECKSUM_URL="$DOWNLOAD_URL.sha256"

echo "📥 Downloading VantaDB CLI ($LATEST_RELEASE) for $TARGET..."
mkdir -p "$INSTALL_DIR"

TMPDIR=$(mktemp -d)
if ! curl -L -f --ssl-reqd -o "$TMPDIR/$TARBALL" "$DOWNLOAD_URL"; then
  echo "❌ Failed to download $DOWNLOAD_URL"
  rm -rf "$TMPDIR"
  exit 1
fi

# Verify checksum
if EXPECTED_HASH=$(curl -sLf --ssl-reqd "$CHECKSUM_URL" 2>/dev/null); then
  COMPUTED_HASH=$(sha256sum "$TMPDIR/$TARBALL" | cut -d' ' -f1)
  if [ "$EXPECTED_HASH" != "$COMPUTED_HASH" ]; then
    echo "❌ Checksum mismatch!"
    rm -rf "$TMPDIR"
    exit 1
  fi
  echo "✅ Checksum verified"
else
  echo "⚠️ No checksum file at $CHECKSUM_URL — skipping verification"
fi

# Extract vanta-cli from tarball
tar xzf "$TMPDIR/$TARBALL" -C "$TMPDIR" "$BINARY_NAME"
cp "$TMPDIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"
rm -rf "$TMPDIR"

echo "✨ VantaDB CLI successfully installed to $INSTALL_DIR/$BINARY_NAME"
echo ""
echo "💡 To use it immediately, add it to your PATH:"
echo "   export PATH=\"\$PATH:$INSTALL_DIR\""
echo ""
echo "To make this change permanent, add that line to your ~/.bashrc or ~/.zshrc."
