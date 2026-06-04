#!/usr/bin/env bash
# Initialize hermetic Python audit venv at target/audit-venv.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VENV_DIR="$REPO_ROOT/target/audit-venv"
PYTHON_DIR="$REPO_ROOT/vantadb-python"
VENV_PYTHON="$VENV_DIR/bin/python"

echo "VantaDB audit venv setup"
echo "  Repo: $REPO_ROOT"
echo "  Venv: $VENV_DIR"

if [[ ! -x "$VENV_PYTHON" ]]; then
    echo "Creating virtual environment..."
    python3 -m venv "$VENV_DIR"
else
    echo "Reusing existing virtual environment."
fi

echo "Installing pip, wheel, maturin, pytest..."
"$VENV_PYTHON" -m pip install --upgrade pip wheel maturin pytest --quiet

export VIRTUAL_ENV="$VENV_DIR"
cd "$PYTHON_DIR"
echo "Building and installing vantadb-python (maturin develop --release)..."
"$VENV_PYTHON" -m maturin develop --release
unset VIRTUAL_ENV
cd "$REPO_ROOT"

"$VENV_PYTHON" -c "import vantadb_py; print('import ok:', vantadb_py.__name__)"

echo ""
echo "Audit venv ready."
echo "  Python: $VENV_PYTHON"
echo "  Activate: source $VENV_DIR/bin/activate"
echo "  Test: $VENV_PYTHON -m pytest $PYTHON_DIR/tests/test_sdk.py -v"
