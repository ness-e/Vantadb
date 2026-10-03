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

Launcher (recall hooks): `vanta-cli mcp-call --db <DB_PATH> --tool memory_recall --args '{...}'`
(one-shot, no pwsh — WIRE-10; the binary the installer puts in `~/.vanta/bin`.
Per-message hooks splice the verbatim prompt as `{{prompt}}`, resolved
against the client's hook-input JSON on stdin by mcp-call itself — no `jq`
needed. `{{prompt}}` matches the documented Claude `UserPromptSubmit.prompt`
field; codex/cursor templates assume the same field name — a mismatch fails
loud (exit 2 naming the missing field), never silent. Save-state hooks print
a static reminder the agent acts on.
Secrets only via session env, never in files or args).

## Versioning (APIs change — pre-mortem #1)

Each template header pins `template_version` + `docs_verified` date + doc URL.
`VERSION.md` is the compat matrix. `tests/test-hooks.ps1` fails on missing
hook/event fields so a client schema change turns red, not silent.

## Tests

```powershell
pwsh -NoProfile -File skills/vantadb-mcp/assets/hooks/tests/test-hooks.ps1
```

Offline, no server: validates JSON, required hooks per client, and budget assertions.
