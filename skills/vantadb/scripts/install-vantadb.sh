#!/usr/bin/env bash
# VantaDB installer — Python SDK (primary) + Rust SDK (dependency).
#
# Usage:
#   ./install-vantadb.sh            # install vantadb-py in the active Python env
#   ./install-vantadb.sh --venv .venv   # create/use a venv, then install
set -euo pipefail

PY_MIN_MAJOR=3
PY_MIN_MINOR=11

die() { echo "error: $*" >&2; exit 1; }

# ── locate Python ────────────────────────────────────────────────────────────
if [ "${1:-}" = "--venv" ]; then
    [ -n "${2:-}" ] || die "--venv requires a path"
    VENV_DIR="$2"
    if [ ! -x "$VENV_DIR/bin/python" ]; then
        echo "creating venv at $VENV_DIR ..."
        python3 -m venv "$VENV_DIR"
    fi
    PY="$VENV_DIR/bin/python"
    shift 2
else
    PY="${PYTHON:-python3}"
fi

command -v "$PY" >/dev/null 2>&1 || die "Python not found ($PY). Install Python >= $PY_MIN_MAJOR.$PY_MIN_MINOR first."

ver="$("$PY" -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')"
IFS=. read -r major minor <<< "$ver"
if [ "$major" -lt $PY_MIN_MAJOR ] || { [ "$major" -eq $PY_MIN_MAJOR ] && [ "$minor" -lt $PY_MIN_MINOR ]; }; then
    die "Python $ver detected; VantaDB requires >= $PY_MIN_MAJOR.$PY_MIN_MINOR"
fi

# ── install ──────────────────────────────────────────────────────────────────
echo "installing vantadb-py on Python $ver ..."
"$PY" -m pip install --upgrade pip
"$PY" -m pip install vantadb-py

"$PY" -c "import vantadb_py; print('vantadb_py', vantadb_py.__version__())" \
    || die "installation succeeded but import check failed"

# ── Rust SDK alternative ─────────────────────────────────────────────────────
echo
echo "Rust SDK: add to Cargo.toml (library crate, not a cargo install binary):"
cat <<'EOF'
[dependencies]
vantadb = "0.5.0"
EOF