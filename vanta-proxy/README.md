# vanta-proxy

> **Status: frozen** — not published until 1.0.0 (repository Freeze List). Development resumes after 1.0.0.

LLM wire proxy with memory writeback: forwards `/v1/...` traffic to an
upstream LLM and records selected exchanges into VantaDB.

## Run

```bash
vanta-proxy [path/to/config.toml]   # default: ./config.toml or $VANTA_PROXY_CONFIG
vanta-proxy --help
```

Copy `config.toml` next to the binary (or point at it), set the upstream API
key inside, and start. Full reference: `docs/api/PROXY.md`.
