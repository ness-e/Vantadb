# Codex — configuración manual (FIND-104 Stop rule)

No hay formato MCP de Codex verificado en este repo (solo `opencode.json`,
`claude-desktop.json` y `cursor.json` en `skills/vantadb-mcp/assets/`).
No se inventa plantilla: seguí la documentación oficial de Codex
(`hooks.json` / MCP settings del cliente) y cableá este comando:

```text
pwsh -NoProfile -File <REPO>/vanta-mcp-local.ps1 -DbPath <DB_PATH>
```

Proxy (opcional, default-on): `http://127.0.0.1:8096`
(arranque: `vanta-proxy "<DB_PATH>/vanta-proxy.toml"`; apagado: Ctrl+C).

Secrets: nunca en archivos — solo env de sesión
(`VANTADB_OPENAI_API_KEY` / `OPENAI_API_KEY`), jamás en claro en logs.
