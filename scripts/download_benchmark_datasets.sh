#!/usr/bin/env bash
# Raw GloVe embeddings (the `.txt` format) — the INPUT to
# scripts/download_ground_truth.py, not the bench layout itself.
#
# FIND-664: this used to fetch only 100d, so tests/certification/
# glove_dimension_scale.rs (which needs 100d AND 300d) could never be
# provisioned — and the cache that would have held 300d did not exist.
set -euo pipefail

DATA_DIR="data/benchmark"
GLOVE_ZIP="$DATA_DIR/glove.6B.zip"
GLOVE_FILES=("glove.6B.100d.txt" "glove.6B.300d.txt")

mkdir -p "$DATA_DIR"

# Download the archive once, extract every dimension this repo benches.
missing=()
for f in "${GLOVE_FILES[@]}"; do
    [ -f "$DATA_DIR/$f" ] || missing+=("$f")
done

if [ ${#missing[@]} -gt 0 ]; then
    if [ ! -f "$GLOVE_ZIP" ]; then
        echo "Downloading GloVe (glove.6B.zip)..."
        if ! curl -fSL --retry 3 --connect-timeout 15 -o "$GLOVE_ZIP" "https://huggingface.co/stanfordnlp/glove/resolve/main/glove.6B.zip"; then
            echo "HuggingFace mirror failed, falling back to Stanford..."
            curl -fSL --retry 3 --connect-timeout 15 -o "$GLOVE_ZIP" "https://nlp.stanford.edu/data/glove.6B.zip"
        fi
    fi
    for f in "${missing[@]}"; do
        echo "Extracting $f..."
        unzip -o "$GLOVE_ZIP" "$f" -d "$DATA_DIR"
    done
fi

# Validate line count on every extracted file (TBH-01: a truncated file is a
# silent skip downstream, not a smaller benchmark).
for f in "${GLOVE_FILES[@]}"; do
    VEC_COUNT=$(wc -l < "$DATA_DIR/$f")
    echo "$f: $VEC_COUNT vectors (expected 400000)"
    if [ "$VEC_COUNT" -lt 1000 ]; then
        echo "ERROR: $f too small ($VEC_COUNT lines)" >&2
        exit 1
    fi
done

echo "All datasets ready."
