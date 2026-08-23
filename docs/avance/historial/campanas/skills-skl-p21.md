# Wave SKL (P21) — skills/vantadb sincronizadas

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

## SKL-01: Corregir y modernizar skills/vantadb/SKILL.md
- **Fuente:** `docs/Backlog.md` → P21
- **Fecha:** 2026-08-17
- **Objetivo:** dejar la skill `skills/vantadb/` sincronizada con el código real.
- **Resultado:** ✅ versión 0.5.0 / Rust 1.94.1 / Py ≥3.11; dep corregida; installer real (6B "test" → 2109B); claims falsos corregidos (IQL SÍ soportado, stemming SÍ soportado); paths vivos (docs/operations/BENCHMARKS.md, integrations/langchain/); benchmarks con fuente; features reales (ttl_ms, purge_expired, compact_wal, AsyncVantaDB, :memory:, hardware_profile); firmas API alineadas con pyi. Dirs vacíos references/ assets/ eliminados (ponytail).
- **Ids:** `SKL-01`

## SKL-02: Corregir skills/vantadb-mcp/ SKILL.md + references
- **Fuente:** `docs/Backlog.md` → P21
- **Fecha:** 2026-08-17
- **Objetivo:** alinear la skill MCP con el server real.
- **Resultado:** ✅ command real (`vanta-cli server --mcp --db` / `vantadb-server --mcp`); 15 tools reales documentados (incluye query_iql, collection_*, rehydrate); paths vivos (docs/api/MCP.md); bloque OpenCode; references verificadas contra código. Fix P2-01: "4 resources" → "2 listados + 2 servibles" (re-verificado 4/4).
- **Ids:** `SKL-02`

## SKL-03: Arreglar scripts/assets de skills/vantadb-mcp/
- **Fuente:** `docs/Backlog.md` → P21
- **Fecha:** 2026-08-17
- **Objetivo:** scripts y assets funcionales con el MCP server real.
- **Resultado:** ✅ `test-mcp.py` reescrito (1 proceso stdio, 4 requests JSON-RPC) — 4/4 passed exit 0 contra vanta-cli.exe v0.5.0 (15 tools, 2 resources, 4 prompts); `setup-vantadb.sh` 0.5.0 sin config.json muerto; `create-namespace.py` y `config-template.json` eliminados (nadie los lee); claude/cursor configs corregidos; `opencode-config.json` añadido (== docs/api/MCP.md:185-199).
- **Ids:** `SKL-03`

## SKL-04: Gate P2-01 review wave SKL
- **Fuente:** `docs/Backlog.md` → P21
- **Fecha:** 2026-08-17
- **Objetivo:** verificación read-only por agente distinto (vanta-review).
- **Resultado:** ✅ CHANGES-REQUIRED con 1 falla (SKILL.md:8 "4 resources" vs 2 reales en resources/list) → fix delegado a SKL-02 y re-verificado (test-mcp.py 4/4 exit 0). 13/13 checks + coherencia skill↔código OK (15/15 tools, 5/5 firmas pyi).
- **Ids:** `SKL-04`
