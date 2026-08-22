# Task GOV-C4 — Regeneración completa master-index.md

## Estado: ✅ COMPLETED

## Steps
- ✅ S1: Inventario docs/ (carpetas primer nivel + sueltos)
- ✅ S2: Regenerar docs/master-index.md (fix 2 rotas, api nuevos, frase blog, carpetas faltantes, regla mantenimiento, frontmatter)
- ✅ S3: Verificación AUD-007 (0 rotas) + contrato carpetas

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `docs/master-index.md` (217L), inventario `Get-ChildItem docs -Recurse -Depth 1`
- **Referencias hacia dentro:** inbound links a `docs/master-index.md` desde README de docs y otros índices (solo cambia contenido, no la ruta)
- **Referencias salientes:** todos los markdown links relativos desde docs/ — verificados con AUD-007
- **Veredicto:** cambio seguro, solo reescritura de contenido del índice; ninguna ruta propia cambia.

## Contexto clave (auditoría heredada)
- Rotas: `audit-reports/audit-full-2026-07-18.md` (→ existe en `reviews/archive/audit-full-2026-07-18.md`), `plans/PROMPT-MAESTRO-FREEZE.md` (→ movido a `plans/archive/`)
- API nuevos sin indexar: VANTA_MEMORY.md, WASM_PERSISTENCE.md, WASM_STANDALONE.md; MCP.md = stub → fuente única skills/vantadb-mcp/references/api-reference.md
- Blog: posts viven en `docs/blog/`, NO `web/content/blog/` (7 archivos)
- case_studies archivado hoy (GOV-B1): respetar nota existente
- Exclusiones deliberadas: `_templates` (plantillas internas), `.obsidian` (config vault), `TDAM-VANTADB` (vacía, pendiente borrado)

## Context Save Point
- S1 ✅ S2 ✅ S3 ✅ — AUD-007: 136 links verificados, 0 rotas; 0 carpetas sin indexar; exclusiones con motivo; last_reviewed 2026-08-22; regla mantenimiento en cabecera; frase blog corregida a `docs/blog/`; MCP enlazado como stub hacia skills/vantadb-mcp/references/api-reference.md. Sin commit (PROHIBIDO git en esta tarea).
