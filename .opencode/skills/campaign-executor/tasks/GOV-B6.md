# GOV-B6 — Skill MCP como fuente única (33 tools) + MCP.md stub

> Plan: `docs/plans/2026-08-22-doc-governance-plan.md` · Wave B · 🟡 🟠 · Appetite max 4h
> Estado: ✅ COMPLETED (2026-08-22)

## Steps

- ✅ S1: api-reference.md § "MCP Tools (33)" agregado (Core 15 tabla + skill_* 6 + code_* 8 + wiki_* 4, cada grupo con precondición).
- ✅ S2: SKILL.md → "33 tools" explícito (intro + heading + tabla resumen grupos); flip fuente única (api-reference = SoT, MCP.md = stub); mcp-protocol.md línea "15 tools" corregida a 33.
- ✅ S3: docs/api/MCP.md stub = 12 líneas ≤20, link relativo correcto (`../../skills/vantadb-mcp/references/api-reference.md`), fecha sync 2026-08-22. Inbound links verificados previos — ninguno roto.
- ✅ S4: Copia hash-SAME ×3 archivos (SKILL.md, api-reference.md, mcp-protocol.md) skills/ ↔ .opencode/skills/ — Get-FileHash idéntico:
  - SKILL.md: 5008CD6F…C8C792
  - api-reference.md: 85546773…16BDB4
  - mcp-protocol.md: 53258891…9F81A652
- ✅ S5: `python skills/vantadb-mcp/scripts/test-mcp.py` → **4/4 passed, exit 0** contra vanta-cli v0.5.0 en PATH.


## Impacto mapeado (Regla 0)

- Leídos completos: api-reference.md (384L), SKILL.md (410L), docs/api/MCP.md (375L), tools.rs (defs 17-187), skills.rs (40-136), code.rs (36-119), wiki.rs (42-98).
- Referencias entrantes a MCP.md: ver S3 — todas sobreviven al stub (stub conserva frontmatter + título + ruta).
- Referencias salientes del stub: ninguna nueva salvo el link a la skill (relativo correcto desde docs/api/ = ../../skills/vantadb-mcp/references/api-reference.md).
- Veredicto: cambio doc-only, cero código, cero riesgo producto.

## Context Save Point

- Fuente verificada por lead: 33 = 15 core (handlers/tools.rs base_tools) + 6 skill_* (skills.rs) + 8 code_* (code.rs) + 4 wiki_*(wiki.rs); extend en tools.rs:180-184.
