# Template versions (FIND-106)

| Template | template_version | Docs source | docs_verified |
|---|---|---|---|
| `opencode/vantadb-memory.js` | 1.2.0 | https://opencode.ai/docs/plugins/ | 2026-11-19 |
| `claude/settings.example.json` | 1.1.0 | https://docs.anthropic.com/en/docs/claude-code/hooks | 2026-09-17 |
| `cursor/hooks.json` | 1.1.0 | https://cursor.com/docs/agent/hooks | 2026-09-17 |
| `codex/hooks.json` | 1.1.0 | https://developers.openai.com/codex/hooks | 2026-09-17 |
| Policy consumed | FIND-103 `c01baa90` | `../references/recall-policy.md` | 2026-09-17 |
| Installer split | FIND-104 `a1bea54b` | `../install/` (untouched) | 2026-09-17 |

Bump `template_version` + `docs_verified` when a client doc changes the hooked event/schema.
`tests/test-hooks.ps1` asserts the 4 policy hooks exist per template, so drift turns red.
WIRE-10 (1.1.0): launcher `pwsh vanta-mcp-local.ps1` → `vanta-cli mcp-call`
one-shot; `vanta-mcp-local.ps1` itself stays INTACTO (other consumers use it).
FIND-169 (1.2.0): `session.idle` auto-capture was a deterministic no-op
(`thread_send` without the required `thread_id` → MCP -32602). Now resolves a
stable per-session thread (deterministic title `vanta-opencode-idle-<sessionID>`,
reused via `thread_list`) and subscribes both the `session.idle` key and the
generic `event` hook — current opencode only delivers bus events via `event`.
