# GOV-B6 — Skill MCP como fuente única (79 tools = 49 core + 30 ext) + MCP.md stub

> Plan: `docs/plans/2026-09-02-alta-prioridad-paralelo.md` · Wave2 · 🟡 🟠 · Appetite max 4h
> Estado: ✅ COMPLETED (2026-09-02)

## Steps

- ✅ S1: api-reference.md § "MCP Tools (33)" agregado (Core 15 tabla + skill_* 6 + code_* 8 + wiki_* 4, cada grupo con precondición) — base 2026-08-22.
- ✅ S2: SKILL.md → "33 tools" explícito (intro + heading + tabla resumen grupos); flip fuente única (api-reference = SoT, MCP.md = stub); mcp-protocol.md línea "15 tools" corregida a 33 — base 2026-08-22.
- ✅ S3: docs/api/MCP.md stub + profiles (≤20L link relativo `../../skills/vantadb-mcp/references/api-reference.md`), fecha sync 2026-08-22 — inbound links verificados previos, ninguno roto.
- ✅ S4: Copia hash-SAME ×2 archivos (SKILL.md, api-reference.md) skills/ ↔ .opencode/skills/ — Get-FileHash idéntico (verificado 2026-09-02: 4C45… / F7B6…).
- ✅ S5: `python skills/vantadb-mcp/scripts/test-mcp.py` → **4/4 passed, exit 0** contra vanta-cli v0.5.0 en PATH — base 2026-08-22.
- ✅ S6: Drift 33→79 cerrado 2026-09-02 — `vantadb-mcp/src/handlers/tools.rs` base_tools 49 + 30 ext =79 (6+8+6+1+3+6); SKILL.md 73→79 (49 core), api-reference 72→79 (42→49), docs/api/MCP.md 76→79 (46→49, last_reviewed 2026-09-02); verify Select-String 79×2 + hash-SAME + cargo check --workspace Finished.

## Impacto mapeado (Regla 0)

- Leídos completos: api-reference.md (521L), SKILL.md (599L), docs/api/MCP.md (453L), tools.rs (base_tools 49), skills.rs (6), code.rs (8), wiki.rs (6), context.rs (1), scenes.rs (3), threads.rs (6), codegraph_explore 4 files/66 símbolos.
- Referencias entrantes a MCP.md: ver S3 — todas sobreviven al stub (stub conserva frontmatter + título + ruta).
- Referencias salientes del stub: ninguna nueva salvo el link a la skill (relativo correcto desde docs/api/ = ../../skills/vantadb-mcp/references/api-reference.md).
- Veredicto: cambio doc-only, cero código, cero riesgo producto. Disjoint con GOV-C1/C2 (no toca .config/nextest.toml, docs/TEST_MAP.md, docs/Backlog.md).

## Context Save Point

- Fuente verificada 2026-09-02: 79 = 49 core (handlers/tools.rs base_tools: memory_* 9 + search/IQL/graph 13 + axioms 3 + collections 3 + maintenance 12 + introspection 4 + backup/bulk/embed 5) + 6 skill_* (skills.rs) + 8 code_* (code.rs) + 6 wiki_* (wiki.rs) + 1 context_assemble (context.rs) + 3 scene_* (scenes.rs) + 6 thread_* (threads.rs); extend en tools.rs:1003-1010.
- Contrato verificado: `Select-String -Path "skills/vantadb-mcp/references/api-reference.md" -Pattern "79 tools"` Count 2 ≥1 ✅ + `Test-Path .opencode/skills/vantadb-mcp/SKILL.md` True ✅ + `cargo check --workspace` Finished 8.18s ✅ + hash-SAME ×2 ✅
