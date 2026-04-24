#!/bin/bash
set -e

echo "=== VantaDB Smoke Test ==="

# 1. Start server in background
echo "[1/6] Starting server..."
export VANTADB_HOST=127.0.0.1
export VANTADB_PORT=8080
export VANTADB_STORAGE_PATH=vantadb_smoke_data
export RUST_LOG=info

# Clean old data if any
rm -rf $VANTADB_STORAGE_PATH

cargo run --release --bin vanta-server &
SERVER_PID=$!

echo "Server PID: $SERVER_PID"

# Wait for server to start
sleep 5

# 2. Verify port
echo "[2/6] Verifying port 8080..."
curl -s http://127.0.0.1:8080/health | grep -q '"success":true'
echo "Health check passed."

# 3. Insert data
echo "[3/6] Inserting node..."
curl -s -X POST http://127.0.0.1:8080/api/v2/query \
  -H "Content-Type: application/json" \
  -d '{"query": "INSERT {\"content\": \"smoke test content\"}"}' | grep -q '"success":true'
echo "Insert passed."

# 4. Read data
echo "[4/6] Reading node..."
curl -s -X POST http://127.0.0.1:8080/api/v2/query \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT * WHERE content == \"smoke test content\""}' | grep -q "smoke test content"
echo "Read passed."

# 5. Restart server
echo "[5/6] Restarting server..."
kill $SERVER_PID
sleep 2

cargo run --release --bin vanta-server &
SERVER_PID=$!
sleep 5

# 6. Confirm persistence
echo "[6/6] Confirming persistence..."
curl -s -X POST http://127.0.0.1:8080/api/v2/query \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT * WHERE content == \"smoke test content\""}' | grep -q "smoke test content"
echo "Persistence passed."

# Cleanup
kill $SERVER_PID
rm -rf $VANTADB_STORAGE_PATH

echo "=== Smoke Test SUCCESS ==="
