# GOV-MCPDOCS — Sync docs/api/MCP.md with the 36 MCP tools (gate fix)

**Type:** docs · **Domain:** vanta-docs · **Date:** 2026-08-22
**Trigger:** Pre-commit hook `validate-docs-coverage.ps1` (section `vantadb-mcp tools`) blocking commits — 36/36 tools missing from `docs/api/MCP.md`. Accumulated violation of Regla 3 across campaigns P27/P30/P32.

## Impacto mapeado (Regla 0)

- Read complete: `vantadb-mcp/src/handlers/tools.rs` (`handle_tools_list` block), `docs/api/MCP.md` (16-line stub), `scripts/validate-docs-coverage.ps1` (extraction + `Check-Methods` logic).
- Inbound refs: pre-commit hook → validator; validator → `docs/api/MCP.md`.
- Outbound refs: MCP.md links to `skills/vantadb-mcp/SKILL.md`.
- Verdict: doc-only change. No Rust code touched. Zero runtime impact.

## Steps

- [x] ✅ Extract real tool list mechanically from `handle_tools_list()` using the same split logic as the validator → exactly 36 core tools (no `skill_*`/`code_*`/`wiki_*` in that block; those live in `code.rs`, `skills.rs`, `wiki.rs` and are dispatched via `tools/call` but never listed in `tools/list`).
- [x] ✅ Rewrite `docs/api/MCP.md`: full table of the 36 core tools grouped by domain (Memory CRUD 7, Search & Query 3, Graph 6, Context & Axioms 2, Collections 3, Maintenance/Indexes/Snapshots 9, Introspection & Utility 2, Backup & Bulk Import 4) with brief English descriptions, plus a section for the 20 extended tools (8 `code_*` + 6 `skill_*` + 6 `wiki_*`) counted from source.
- [x] ✅ Run gate: `pwsh scripts/validate-docs-coverage.ps1` → **0 gaps** (36/36 ok, all other sections still green).
- [x] ✅ Task file documenting the fix.

## Notes / Findings

- The old stub claimed "50 tools (30 core + 6 skill_* + 8 code_* + 6 wiki_*)" — wrong on every count. Real numbers verified from source: **36 core + 20 extended = 56**.
- **Observation (not fixed, out of scope):** `skill_*`/`code_*`/`wiki_*` tools are callable via `tools/call` but are NOT returned by `tools/list`, so LLM clients can't discover them. Candidate follow-up for `vanta-worker` (Rust) if discovery is desired.
- `skills/vantadb-mcp/references/api-reference.md` likely also carries stale tool counts ("§ MCP Tools (50)") — debt for a follow-up docs pass; this task only touched the gated file per scope.
- Validator's `Test-InDoc` matches bare word boundaries, so backticked tool names in tables satisfy the check.

## Verification contract

```powershell
pwsh scripts/validate-docs-coverage.ps1   # → "vantadb-mcp (tools) - 36 items ok en MCP.md", exit 0
```

Result: ✅ passed 2026-08-22.
