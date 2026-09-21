# Template versions (FIND-106)

| Template | template_version | Docs source | docs_verified |
|---|---|---|---|
| `opencode/vantadb-memory.js` | 1.0.0 | https://opencode.ai/docs/plugins/ | 2026-09-17 |
| `claude/settings.example.json` | 1.0.0 | https://docs.anthropic.com/en/docs/claude-code/hooks | 2026-09-17 |
| `cursor/hooks.json` | 1.0.0 | https://cursor.com/docs/agent/hooks | 2026-09-17 |
| `codex/hooks.json` | 1.0.0 | https://developers.openai.com/codex/hooks | 2026-09-17 |
| Policy consumed | FIND-103 `c01baa90` | `../references/recall-policy.md` | 2026-09-17 |
| Installer split | FIND-104 `a1bea54b` | `../install/` (untouched) | 2026-09-17 |

Bump `template_version` + `docs_verified` when a client doc changes the hooked event/schema.
`tests/test-hooks.ps1` asserts the 4 policy hooks exist per template, so drift turns red.
