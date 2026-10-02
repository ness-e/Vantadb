# vantadb-server
Server binary: HTTP API (default) + MCP stdio (`--mcp`); storage resolves via `VANTADB_STORAGE_PATH` (default `./vantadb_data`, gitignored — never commit local data).
Run: `cargo run -p vantadb-server` · MCP: `vantadb-server --mcp` · Tests: `cargo test -p vantadb-server` (certification reports land in CWD, gitignored).
Config: `VANTADB_HOST`, `VANTADB_PORT`, `VANTADB_STORAGE_PATH` — see `docs/user/operations/CONFIGURATION.md`.
(Deployment: systemd o Kubernetes — ver `DEPLOYMENT_GUIDE.md`; docker retirado 2026-10-02.)
