#!/usr/bin/env bash
# validate_python_sdk.sh — Build wheel, install in clean venv, run pytest.
# Usage: ./dev-tools/validate_python_sdk.sh [--skip-build]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WHEEL_DIR="$REPO_ROOT/target/wheels"
VENV_DIR="$REPO_ROOT/.validate-venv"
PYTHON_DIR="$REPO_ROOT/vantadb-python"

echo "═══════════════════════════════════════════════════════════"
echo "  VantaDB Python SDK Validation"
echo "═══════════════════════════════════════════════════════════"

# ── Step 1: Build wheel ──────────────────────────────────────
if [[ "${1:-}" != "--skip-build" ]]; then
    echo ""
    echo "▸ Building wheel with maturin..."
    python3 -m maturin build \
        --manifest-path "$PYTHON_DIR/Cargo.toml" \
        --out "$WHEEL_DIR" \
        --release
    echo "  ✓ Wheel built successfully."
else
    echo ""
    echo "▸ Skipping build (--skip-build)."
fi

# ── Step 2: Create clean venv ────────────────────────────────
echo ""
echo "▸ Creating clean virtual environment..."
rm -rf "$VENV_DIR"
python3 -m venv "$VENV_DIR"
# shellcheck disable=SC1091
source "$VENV_DIR/bin/activate"
python -m pip install --upgrade pip pytest --quiet

# ── Step 3: Install wheel ────────────────────────────────────
echo ""
echo "▸ Installing latest wheel..."
WHEEL="$(ls -t "$WHEEL_DIR"/vantadb_py-*.whl 2>/dev/null | head -n 1)"
if [[ -z "$WHEEL" ]]; then
    echo "  ✗ No wheel found in $WHEEL_DIR. Run without --skip-build."
    deactivate
    rm -rf "$VENV_DIR"
    exit 1
fi
python -m pip install --force-reinstall "$WHEEL" --quiet
echo "  ✓ Installed: $(basename "$WHEEL")"

# ── Step 4: Run pytest ───────────────────────────────────────
echo ""
echo "▸ Running Python SDK tests..."
python -m pytest "$PYTHON_DIR/tests/test_sdk.py" -v
RESULT=$?

# ── Step 5: Cleanup ──────────────────────────────────────────
echo ""
echo "▸ Cleaning up..."
deactivate
rm -rf "$VENV_DIR"

if [[ $RESULT -eq 0 ]]; then
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "  ✓ Python SDK validation PASSED"
    echo "═══════════════════════════════════════════════════════════"
else
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "  ✗ Python SDK validation FAILED (exit code: $RESULT)"
    echo "═══════════════════════════════════════════════════════════"
    exit $RESULT
fi
