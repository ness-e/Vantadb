#!/usr/bin/env bash
set -euo pipefail

DATA_DIR="data/benchmark"
GLOVE_ZIP="$DATA_DIR/glove.6B.zip"
GLOVE_TXT="$DATA_DIR/glove.6B.100d.txt"

mkdir -p "$DATA_DIR"

# Download GloVe-100
if [ ! -f "$GLOVE_TXT" ]; then
    if [ ! -f "$GLOVE_ZIP" ]; then
        echo "Downloading GloVe-100 (glove.6B.zip)..."
        curl -L -o "$GLOVE_ZIP" "https://nlp.stanford.edu/data/glove.6B.zip"
    fi
    echo "Extracting glove.6B.100d.txt..."
    unzip -o "$GLOVE_ZIP" "glove.6B.100d.txt" -d "$DATA_DIR"
fi

# Validate line count
VEC_COUNT=$(wc -l < "$GLOVE_TXT")
echo "GloVe-100: $VEC_COUNT vectors (expected 400000)"
if [ "$VEC_COUNT" -lt 1000 ]; then
    echo "ERROR: GloVe file too small ($VEC_COUNT lines)"
    exit 1
fi

echo "All datasets ready."
