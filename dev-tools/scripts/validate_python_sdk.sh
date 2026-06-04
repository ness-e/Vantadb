#!/usr/bin/env bash
# validate_python_sdk.sh — Build wheel, install in audit venv, run pytest.
# Usage: ./dev-tools/scripts/validate_python_sdk.sh [--skip-build]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
WHEEL_DIR="$REPO_ROOT/target/wheels"
VENV_DIR="$REPO_ROOT/target/audit-venv"
PYTHON_DIR="$REPO_ROOT/vantadb-python"
SETUP_SCRIPT="$REPO_ROOT/dev-tools/setup_venv.sh"
VENV_PYTHON="$VENV_DIR/bin/python"

echo "============================================================"
echo "  VantaDB Python SDK Validation"
echo "============================================================"

if [[ ! -x "$VENV_PYTHON" ]]; then
    echo ""
    echo "▸ Audit venv missing; running setup_venv.sh..."
    if [[ -x "$SETUP_SCRIPT" ]]; then
        bash "$SETUP_SCRIPT"
    else
        echo "  Creating audit venv..."
        python3 -m venv "$VENV_DIR"
        "$VENV_PYTHON" -m pip install --upgrade pip wheel maturin pytest --quiet
        export VIRTUAL_ENV="$VENV_DIR"
        (cd "$PYTHON_DIR" && "$VENV_PYTHON" -m maturin develop --release)
        unset VIRTUAL_ENV
    fi
fi

if [[ "${1:-}" != "--skip-build" ]]; then
    echo ""
    echo "> Building wheel with maturin..."
    export VIRTUAL_ENV="$VENV_DIR"
    (cd "$PYTHON_DIR" && "$VENV_PYTHON" -m maturin build --out "$WHEEL_DIR" --release)
    unset VIRTUAL_ENV
    echo "  [OK] Wheel built successfully."
else
    echo ""
    echo "> Skipping build (--skip-build)."
fi

# Ensure editable install when venv already exists
export VIRTUAL_ENV="$VENV_DIR"
(cd "$PYTHON_DIR" && "$VENV_PYTHON" -m maturin develop --release) >/dev/null 2>&1 || true
unset VIRTUAL_ENV

# ── Step 2: Ensure audit venv tooling ────────────────────────
echo ""
echo "▸ Ensuring audit venv dependencies..."
"$VENV_PYTHON" -m pip install --upgrade pip pytest --quiet

# ── Step 3: Install wheel ────────────────────────────────────
echo ""
echo "▸ Installing latest wheel into audit venv..."
WHEEL="$(ls -t "$WHEEL_DIR"/vantadb_py-*.whl 2>/dev/null | head -n 1)"
if [[ -z "$WHEEL" ]]; then
    echo "  ✗ No wheel found in $WHEEL_DIR. Run without --skip-build."
    exit 1
fi
"$VENV_PYTHON" -m pip install --force-reinstall "$WHEEL" --quiet
echo "  ✓ Installed: $(basename "$WHEEL")"

# ── Step 4: Run pytest ───────────────────────────────────────
echo ""
echo "▸ Running Python SDK tests..."
"$VENV_PYTHON" -m pytest "$PYTHON_DIR/tests/test_sdk.py" -v
RESULT=$?

"$VENV_PYTHON" -c "import vantadb_py"
if [[ $RESULT -eq 0 ]]; then
    echo ""
    echo "============================================================"
    echo "  [OK] Python SDK validation PASSED"
    echo "============================================================"
else
    echo ""
    echo "============================================================"
    echo "  [FAIL] Python SDK validation FAILED (exit code: $RESULT)"
    echo "============================================================"
    exit $RESULT
fi
