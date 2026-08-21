# MEM-02: F1 Exponer search profile en MCP/search

## Metadata
- **Plan file:** `docs/plans/archive/2026-08-18-vanta-memory.md` (Task 2) — **archivado 2026-08-21**
- **Fuente:** plan file Task 2 (MEM-02)
- **Tipo:** Rust
- **Creado:** 2026-08-21 (**retroactivo** — la tarea se ejecutó sin task file; única de las 24 de P27)
- **Estado:** ✅ COMPLETED (commit `32b09daf`, verify mecánico en su momento)

## Nota retroactiva
Esta tarea se completó en la sesión F1 (2026-08-18..19) sin crear task file — violación de trazabilidad D19 detectada en la auditoría post-campaña del 2026-08-21. El trabajo está verificado por commit + plan file + suite. Este documento restaura la trazabilidad mínima.

## Qué se hizo
- `vantadb-mcp`: parámetro opcional `search_profile` (mode/rrf_k/candidate_k) en el tool MCP de búsqueda → passthrough al planner.
- Commit: `32b09daf feat(mcp): search_profile passthrough en tool de búsqueda (MEM-02)`
- Plan file Task 2 marcada ✅ en su momento.

## Contrato
"`cargo check -p vantadb` pasa; el tool MCP de búsqueda acepta `search_profile` opcional y lo propaga al planner" — verificado en su sesión (suite completa pasando).

## Deuda documentada
Hereda las deudas de MEM-01 (mismo mecanismo): `rrf_k`/`candidate_k` propagados sin efecto hasta que el CBO fusione RRF (`ponytail:` en MEM-01.md). Vigente hasta F5+.

## Impacto mapeado (Regla 0 — reconstruido)
- **Archivos tocados:** `vantadb-mcp/src/handlers/tools.rs` (parámetro + validación), wiring del request.
- **Callers:** clientes MCP que pasen `search_profile`; default = sin perfil (comportamiento previo intacto).
