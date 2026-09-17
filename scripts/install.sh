#!/bin/sh
set -e

# VantaDB installer for Linux and macOS.
# Downloads the release tarball and extracts vanta-cli to ~/.vanta/bin,
# then chains to the interactive setup wizard (FIND-104).
#
# One-liner (no clone, no rustup):
#   curl -fsSL https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.sh | sh
# Flags: --dry-run (simulate, no effects) --no-wizard (skip chain)
#        --wizard-non-interactive (wizard with defaults) --help
# Trust: TLS required + official repo URL + sha256 of the payload verified
# in-script. To verify manually, download the file and compare against the
# published .sha256 asset before piping to sh.

INSTALL_DIR="$HOME/.vanta/bin"
BINARY_NAME="vanta-cli"
WIZARD_FILE="setup-embeddings.ps1"

DRY_RUN=0
WITH_WIZARD=1
WIZARD_NONINTERACTIVE=0

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    --no-wizard) WITH_WIZARD=0 ;;
    --wizard-non-interactive) WIZARD_NONINTERACTIVE=1 ;;
    --help|-h)
      echo "Usage: install.sh [--dry-run] [--no-wizard] [--wizard-non-interactive]"
      echo "  --dry-run: print the installer->wizard chain without changes or network"
      echo "  --no-wizard: install the CLI only; run the wizard later manually"
      echo "  --wizard-non-interactive: chain the wizard with defaults (no prompts)"
      exit 0
      ;;
    *)
      echo "❌ Unknown option: $arg (see --help)"
      exit 1
      ;;
  esac
done

# --- Dry-run (FIND-105 AC(a)): simulate installer -> wizard, no effects ---
# Runs before OS detection so it works on any machine (incl. CI/Windows).
if [ "$DRY_RUN" = "1" ]; then
  echo "[dry-run] install.sh --dry-run: no changes, no network."
  echo "[dry-run] would: detect target (\$(uname -s)/\$(uname -m)) + fetch latest tag (api.github.com/repos/ness-e/Vantadb/releases/latest)"
  echo "[dry-run] would: download vantadb-<target>.tar.gz + .sha256, verify checksum"
  echo "[dry-run] would: backup $INSTALL_DIR/$BINARY_NAME (.bak-<stamp>) + install"
  if [ "$WITH_WIZARD" = "1" ]; then
    if [ "$WIZARD_NONINTERACTIVE" = "1" ]; then MODE=" with -NonInteractive"; else MODE=" (interactive)"; fi
    echo "[dry-run] would: chain wizard $WIZARD_FILE (<release-tag>/$WIZARD_FILE)$MODE via pwsh, else print manual next-step"
  else
    echo "[dry-run] would: skip wizard (--no-wizard)"
  fi
  echo "[dry-run] chain OK: installer -> wizard (simulated, exit 0)"
  exit 0
fi

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

# Idempotency (FIND-105 pre-mortem #2): back up any previous binary.
if [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
  STAMP=$(date +%Y%m%d-%H%M%S)
  cp "$INSTALL_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME.bak-$STAMP"
  echo "💾 Backed up previous binary to $BINARY_NAME.bak-$STAMP"
fi
cp "$TMPDIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "✨ VantaDB CLI successfully installed to $INSTALL_DIR/$BINARY_NAME"

# --- Wizard chain (FIND-105): installer -> setup-embeddings interactivo ---
if [ "$WITH_WIZARD" = "0" ]; then
  echo ""
  echo "⏭️  Wizard skipped (--no-wizard). Run later:"
  echo "   pwsh $WIZARD_FILE -NonInteractive"
else
  echo ""
  echo "🧙 Chaining to the interactive setup wizard (FIND-104)..."
  WIZARD_URL="https://raw.githubusercontent.com/ness-e/Vantadb/$LATEST_RELEASE/$WIZARD_FILE"
  if [ "$WIZARD_NONINTERACTIVE" = "1" ]; then WIZARD_ARGS="-NonInteractive"; else WIZARD_ARGS=""; fi
  if command -v pwsh >/dev/null 2>&1 \
    && curl -sL -f --ssl-reqd -o "$TMPDIR/$WIZARD_FILE" "$WIZARD_URL" \
    && pwsh -NoProfile -File "$TMPDIR/$WIZARD_FILE" $WIZARD_ARGS; then
    echo "✅ Wizard completed"
  else
    echo "⚠️ Wizard did not run — CLI is installed; run manually:"
    echo "   pwsh $WIZARD_FILE -NonInteractive"
    echo "   (tip: re-run the wizard from a downloaded file for full interactivity)"
  fi
fi
rm -rf "$TMPDIR"

echo ""
echo "💡 To use it immediately, add it to your PATH:"
echo "   export PATH=\"\$PATH:$INSTALL_DIR\""
echo ""
echo "To make this change permanent, add that line to your ~/.bashrc or ~/.zshrc."
