#!/bin/bash
set -e

SERVER_PID=""

# ── Cleanup trap: kills server on ANY exit (success or failure) ──
cleanup() {
  if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
    echo "[cleanup] Stopping server (PID $SERVER_PID)..."
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf "$VANTADB_STORAGE_PATH"
}
trap cleanup EXIT

echo "=== VantaDB Smoke Test ==="

# ── Environment ──
export VANTADB_HOST=127.0.0.1
export VANTADB_PORT=8080
export VANTADB_STORAGE_PATH=vantadb_smoke_data
export RUST_LOG=info

# Clean old data if any
rm -rf "$VANTADB_STORAGE_PATH"

# ── 1. Build & Start ──
echo "[1/7] Building & starting server..."
cargo build --release --bin vanta-server

./target/release/vanta-server &
SERVER_PID=$!
echo "       Server PID: $SERVER_PID"

# Wait for server to boot
sleep 5

# ── 2. Verify data directory was created ──
echo "[2/7] Verifying data directory..."
if [ -d "$VANTADB_STORAGE_PATH" ]; then
  echo "       Data directory '$VANTADB_STORAGE_PATH' exists. ✔"
else
  echo "       ERROR: Data directory '$VANTADB_STORAGE_PATH' not found!"
  exit 1
fi

# ── 3. Health check ──
echo "[3/7] Verifying health endpoint (127.0.0.1:8080)..."
curl -sf http://127.0.0.1:8080/health | grep -q '"success":true'
echo "       Health check passed. ✔"

# ── 4. Insert data ──
echo "[4/7] Inserting node..."
curl -sf -X POST http://127.0.0.1:8080/api/v2/query \
  -H "Content-Type: application/json" \
  -d '{"query": "(INSERT :node {:content \"smoke test content\"})"}' | grep -q '"success":true'
echo "       Insert passed. ✔"

# ── 5. Second insert (validates engine stability) ──
echo "[5/7] Second insert (stability check)..."
curl -sf -X POST http://127.0.0.1:8080/api/v2/query \
  -H "Content-Type: application/json" \
  -d '{"query": "(INSERT :node {:content \"second smoke entry\"})"}' | grep -q '"success":true'
echo "       Second insert passed. ✔"

# ── 6. Restart server & re-verify ──
echo "[6/7] Restarting server..."
kill "$SERVER_PID" 2>/dev/null || true
wait "$SERVER_PID" 2>/dev/null || true
sleep 2

./target/release/vanta-server &
SERVER_PID=$!
echo "       Restarted with PID: $SERVER_PID"
sleep 5

# ── 7. Confirm server responds after restart ──
echo "[7/7] Confirming post-restart health..."
curl -sf http://127.0.0.1:8080/health | grep -q '"success":true'
echo "       Post-restart health passed. ✔"

echo ""
echo "=== ✅ Smoke Test PASSED ==="
