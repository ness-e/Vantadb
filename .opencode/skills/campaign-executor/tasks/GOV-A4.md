# Task GOV-A4 — Harness de snippets de documentación

- **Plan:** `docs/plans/2026-08-22-doc-governance-plan.md` (Task 7)
- **Estado:** ✅ COMPLETED (sin commit — PROHIBIDO git en esta tarea; el lead commitea)
- **Appetite:** max 3h · 🟡 · Prioridad 🔴
- **Contrato:** script extrae bloques ```python de tutorials/ + QUICKSTART, ejecuta contra DB temporal de vantadb-py, reporta PASS/FAIL/SKIP por snippet; corrida inicial detecta las roturas conocidas como FAIL.

## Impacto mapeado (Regla 0)

**Archivos leídos completos / verificados:**
- `docs/plans/2026-08-22-doc-governance-plan.md` — definición de GOV-A4 (líneas 149-167) y GOV-B3 (224-237, consumidor del harness).
- `vantadb-python/vantadb_py/__init__.py` (vía codegraph) — wrapper sync/async; `graph_bfs(roots, max_depth=999999)` en :332.
- `vantadb-python/src/lib.rs` (vía codegraph) — pymodule `vantadb_py`, clase `VantaDB` con `put`, `search_memory(namespace, query_vector, ..., text_query=, top_k=)`, `flush`, `close`.
- Bloques python de `docs/tutorials/*.md` y `docs/QUICKSTART.md` inventariados (~57 bloques).

**Referencias hacia dentro (entrantes):**
- GOV-B3 (`docs/tutorials/03-migrating-from-chromadb.md:178`, `migration-from-lancedb.md:281`) consume el harness como guard anti-regresión.
- Plan §GOV-B3 contrato: "harness de GOV-A4 pasa 100% sobre docs corregidos".

**Referencias hacia fuera (salientes):**
- El script NUEVO no es importado por nadie hoy. Sin gate CI en esta tarea (integración a gate-docs = GOV-B3).
- Depende runtime de: venv `.venv/` con `vantadb`/`vantadb_py` instalados (editable desde `vantadb-python/`), Python stdlib only.

**Veredicto de impacto:** BAJO — archivo nuevo aislado, cero modificaciones a código existente o docs. Riesgo único: resultados ruidosos por snippets no autocontenidos → mitigado con directiva `# vanta-skip` + auto-skip por dependencia externa ausente.

## Steps

1. ✅ Discovery: firma real `graph_bfs(roots, max_depth)` confirmada (wrapper py :332); roturas conocidas localizadas en 03:178 y lancedb:281; API put/get/search_memory probada contra venv OK.
2. ✅ Implementar `dev-tools/validate_doc_snippets.py` (extractor + header común + subprocess + reporte + exit code).
3. ✅ Corrida inicial: graph_bfs ×2 detectados como FAIL; QUICKSTART put/get PASS; cero crashes del harness.
4. ✅ Cierre: recitation + RESULTADO.

## Context Save Point

- Venv usable: `.\.venv\Scripts\python.exe`; `import vantadb` resuelve a `vantadb-python/vantadb/__init__.py` (shim editable).
- Constructor `VantaDB(path, memory_limit_bytes=...)` aceptado (probe OK 2026-08-22).
- Los snippets de tutoriales asumen estado de bloques previos (`db`, `query_vector`) → algunos FAIL por NameError son esperados y honestos; los arregla GOV-B3.
- PROHIBIDO corregir tutoriales (GOV-B3). PROHIBIDO git.
