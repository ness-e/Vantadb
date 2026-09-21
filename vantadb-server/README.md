# vantadb-server
Server binary: HTTP API (default) + MCP stdio (`--mcp`); storage resolves via `VANTADB_STORAGE_PATH` (default `./vantadb_data`, gitignored — never commit local data).
Run: `cargo run -p vantadb-server` · MCP: `vantadb-server --mcp` · Tests: `cargo test -p vantadb-server` (certification reports land in CWD, gitignored).
Config: `VANTADB_HOST`, `VANTADB_PORT`, `VANTADB_STORAGE_PATH` — see `docs/operations/CONFIGURATION.md`.
Compose: `docker-compose.yml` (dev) · `docker-compose.prod.yml` (prod).
