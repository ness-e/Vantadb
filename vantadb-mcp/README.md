# vantadb-mcp

> **Status: active** (core-promise until 1.0 — the "Agent" surface: the door for AI IDEs and agents).

Model Context Protocol (MCP) server for VantaDB. It runs as a **stdio
JSON-RPC 2.0 server**: your MCP client spawns the process and every memory
operation (store, search, graph, wiki, skills) becomes a tool the agent can
call. No daemon, no network port.

This crate is a library (`publish = false`). The shipped entry point is the
`vantadb-server` binary, and the CLI wrapper `vanta-cli server --mcp` is the
canonical launcher.

## Run

```bash
vanta-cli server --mcp --db /absolute/path/to/.vantadb
```

(Advanced: run `vantadb-server --mcp` directly with
`VANTADB_STORAGE_PATH=/absolute/path/to/.vantadb`.)

> **Absolute paths only.** MCP clients spawn the server process without a
> shell, so `~` arrives literally and the open fails. Use an absolute path
> (e.g. `C:/Users/<you>/.vantadb` on Windows, `/Users/<you>/.vantadb` on
> macOS).

## Client configuration

Claude Desktop, Cursor and OpenCode all use the same command — only the config
file differs:

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vanta-cli",
      "args": ["server", "--mcp", "--db", "/absolute/path/to/.vantadb"]
    }
  }
}
```

## Tool surface

**82 listed tools in 8 families** (88 defined − 6 absorbed `code_*` projections):

| Family | Count | Source module |
|--------|-------|---------------|
| Core | 49 | `handlers/tools.rs` |
| `code_*` | 3 listed (9 defined) | `code.rs` + `code_index.rs` |
| `skill_*` | 7 | `skills.rs` |
| `wiki_*` | 6 | `wiki.rs` |
| `context_assemble` | 1 | `context.rs` |
| `scene_*` | 5 | `scenes.rs` |
| `thread_*` | 6 | `threads.rs` |
| `dream_*` | 5 | `dreams.rs` |

Every tool carries MCP annotations (`title`, `readOnlyHint`,
`destructiveHint`, `idempotentHint`, `openWorldHint`).

## Reference

- [`docs/api/MCP.md`](../docs/api/MCP.md) — canonical reference: tool
  definitions, surface profiles, error codes, prompts, resources, embedding
  providers, governance.
- [`server.json`](../server.json) — MCP registry manifest
  (`io.github.ness-e/vantadb`, version 0.9.0).
- [`skills/vantadb-mcp/`](../skills/vantadb-mcp/) — per-IDE setup skill.

## Tests

```bash
cargo test -p vantadb-mcp
```

## License

Apache-2.0
