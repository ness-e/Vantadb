#!/bin/bash
# Script to update Homebrew formula SHA256 checksums after a new release.
# Usage: ./scripts/update-homebrew-formula.sh v0.1.5

VERSION=${1:-v0.1.5}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FORMULA="$SCRIPT_DIR/../vantadb.rb"

echo "Updating $FORMULA for version $VERSION"

# Update version in formula
if [[ "$OSTYPE" == "darwin"* ]]; then
  sed -i '' "s/version \".*\"/version \"${VERSION#v}\"/" "$FORMULA"
else
  sed -i "s/version \".*\"/version \"${VERSION#v}\"/" "$FORMULA"
fi

# Download each asset and compute its SHA256, then update the formula
for URL in $(grep -oP 'https://github\.com[^"'"'"']+' "$FORMULA"); do
  echo "  Downloading: $URL"
  SHA=$(curl -sL "$URL" | shasum -a 256 | cut -d' ' -f1)
  echo "  SHA256: $SHA"
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i "" "s|sha256 \".*\"$|sha256 \"$SHA\"|" "$FORMULA"
  else
    sed -i "s|sha256 \".*\"$|sha256 \"$SHA\"|" "$FORMULA"
  fi
done

echo "Done. Updated $FORMULA with SHA256 checksums."
