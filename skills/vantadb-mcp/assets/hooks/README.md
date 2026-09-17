# VantaDB memory hooks per client (FIND-106)

Deterministic "always" hooks implementing [`references/recall-policy.md`](../references/recall-policy.md)
(FIND-103). This file is HOW per client; the policy is WHAT.

## Hook map (all clients)

| Policy hook | VantaDB tools | OpenCode (`opencode/`) | Claude (`.claude/`) | Cursor (`.cursor/`) | Codex (`.codex/`) |
|---|---|---|---|---|---|
| `SessionStart` recall | `memory_recall` scope agent top_k 5 → `additionalContext` | `session.created` | `SessionStart` | `sessionStart` | `SessionStart` |
| Per-message recall | `memory_recall` query = verbatim user text | `tool.execute.before` | `UserPromptSubmit` | `beforeSubmitPrompt` | `UserPromptSubmit` |
| `PreCompact` save-state | `thread_send` +/`scene_write` | `experimental.session.compacting` | `PreCompact` | `preCompact` | `PreCompact` |
| `Stop` auto-capture | `thread_send` role+content | `session.idle` | `Stop` | `stop` | `Stop` |

Rules: query is verbatim (never paraphrase); empty recall injects NOTHING
(say "no recuerdo nada sobre X"); inject `prepend_context` verbatim (never re-summarize);
temporal fallback = last 30 days + explicit warning. Full rules: `TOKEN-BUDGET.md` + policy §3-§5.

## Install per client (copy, replace `<DB_PATH>`/`<REPO>`, trust per client flow)

- **OpenCode:** copy `opencode/vantadb-memory.js` → `<project>/.opencode/plugins/` (or global
  `~/.config/opencode/plugins/`). Docs: `https://opencode.ai/docs/plugins/`.
- **Claude Code:** merge `claude/settings.example.json` into `.claude/settings.json`
  (project) or `~/.claude/settings.json` (user). Review with client. Docs:
  `https://docs.anthropic.com/en/docs/claude-code/hooks`.
- **Cursor:** copy `cursor/hooks.json` → `<project>/.cursor/hooks.json` (runs from project root).
  Docs: `https://cursor.com/docs/agent/hooks`.
- **Codex:** copy `codex/hooks.json` → `<repo>/.codex/hooks.json` (or `~/.codex/hooks.json`);
  review/trust via `/hooks`. Docs: `https://developers.openai.com/codex/hooks`.

Launcher (all script hooks): `pwsh -NoProfile -File <REPO>/vanta-mcp-local.ps1 -DbPath <DB_PATH>`
(see `../install/` from FIND-104; secrets only via session env, never in files).

## Versioning (APIs change — pre-mortem #1)

Each template header pins `template_version` + `docs_verified` date + doc URL.
`VERSION.md` is the compat matrix. `tests/test-hooks.ps1` fails on missing
hook/event fields so a client schema change turns red, not silent.

## Tests

```powershell
pwsh -NoProfile -File skills/vantadb-mcp/assets/hooks/tests/test-hooks.ps1
```

Offline, no server: validates JSON, required hooks per client, and budget assertions.
