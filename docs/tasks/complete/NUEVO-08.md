# NUEVO-08: Learning path estructurado en tutorials/ (5-7 ejemplos)

## Metadata
- **Plan file:** ninguno activo (backlog directo)
- **Fuente:** docs/Backlog.md línea 118 — "Learning path estructurado en tutorials/ (5-7 ejemplos)" / audit backlog-validation-2026-07-28
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟠
- **Tipo:** Docs / Content (tutoriales)
- **Turns estimados:** 15-30
- **Creado:** 2026-08-02
- **Estado:** ✅ COMPLETADO

## Contexto verificado (2026-08-02)
- `docs/tutorials/` tiene **4 tutoriales** (meta 5-7):
  - `01-ai-agent-memory.md` — **status: draft**, 253 líneas, last_reviewed 2026-07-03
  - `02-local-rag-pipeline.md` — **status: draft**, 323 líneas, last_reviewed 2026-07-03
  - `03-migrating-from-chromadb.md` — **status: draft**, 256 líneas, last_reviewed 2026-08-02 (actualizado por NUEVO-07 con API real)
  - `migration-from-lancedb.md` — **status: active**, 375 líneas, last_reviewed 2026-08-02
- `docs/book/src/tutorials/` tiene copias del mdBook (`index.md` + los 4). **Las copias del book NO son idénticas** a `docs/tutorials/` (hash diff) — verificar cuál es la fuente canónica y si el book requiere sync.
- NO existe `docs/tutorials/index.md` (la book tiene `docs/book/src/tutorials/index.md`).
- `docs/book/src/SUMMARY.md` líneas 8-12 listan los 4 tutoriales bajo "Tutorials".

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/book/src/SUMMARY.md`, `docs/book/src/tutorials/index.md`, `docs/README.md`, web (si hay links), `docs/master-index.md` |
| Callees | API real de `vantadb_py` (NUEVO-07 ya corrigió migraciones); ejemplos de código en los tutorials |
| Implicaciones | Tutoriales con API inventada = documentación incorrecta. Los draft DEBEN usar la API real (`vantadb_py.VantaDB`, `space`, `put`, `hybrid_search`) |

**RIESGO:** medio — docs, pero el contenido debe reflejar API real (no inventada).

## Contrato
"`docs/tutorials/` tiene 5-7 tutoriales, todos con status: active (o al menos sin 'draft'), cada uno con código verificable contra la API real de `vantadb_py` (sin `vantadb.connect`/`db.space` inventados), y un índice/learning path estructurado que los ordena por complejidad creciente; las copias del mdBook (`docs/book/src/tutorials/`) están sincronizadas con la fuente canónica."

## Herramientas necesarias
- Read/Grep (docs)
- Python root `.venv` (tiene `vantadb_py` 0.5.0) para validar snippets si aplica
- No cargo-mcp (no se toca Rust)

## Investigation Notes
- Prioridad del backlog: "4/7, algunos draft". El gap es (a) 3 tutoriales en draft (01, 02, 03) → validar contra API real y promover a active; (b) completar 1-3 tutoriales más para llegar a 5-7; (c) learning path estructurado (orden, índices, progresión).
- Los tutoriales draft 01/02/03 no fueron tocados por NUEVO-07 (solo migraciones). Verificar si usan API inventada.
- El learning path debería: partir de lo básico (DB embeddable, put/get) → agent memory → RAG → migraciones → avanzado. Revisar qué estructura pide el backlog.

## Steps

### Step 1: Auditar los 4 tutoriales contra la API real
- **Archivos:** `docs/tutorials/*.md`
- **Acción:** verificar cada tutorial contra la API real de `vantadb_py` (NUEVO-07 corrigió migraciones; verificar 01/02/03). Corregir API inventada (`vantadb.connect`, `db.space`, etc.) si existe. Corregir frontmatter `status`.
- **Verify:** grep — 0 ocurrencias de API inventada; cada tutorial con status correcto
- **Estado:** ✅ COMPLETADO

### Step 2: Definir learning path estructurado + índice
- **Archivos:** `docs/tutorials/index.md` (crear) + `docs/book/src/tutorials/index.md` (sync)
- **Acción:** índice con progresión (DB basics → agent memory → RAG → migraciones → avanzado). Actualizar `docs/book/src/SUMMARY.md` si cambia la lista.
- **Verify:** índice existe y lista todos los tutoriales en orden
- **Estado:** ✅ COMPLETADO

### Step 3: Completar tutoriales faltantes (llegar a 5-7)
- **Archivos:** 1-3 tutoriales nuevos en `docs/tutorials/`
- **Acción:** según gap vs meta 5-7. Candidatos naturales: `04-hybrid-search-basics.md`, `05-embedding-integrations.md` (OpenAI/Ollama/LiteLLM), o `06-wasm-in-browser.md`. Validar contra API real.
- **Verify:** count = 5-7 tutoriales; nuevos con status: active y código verificable
- **Estado:** ✅ COMPLETADO

### Step 4: Promover drafts a active + sync mdBook
- **Archivos:** `docs/tutorials/*.md` (frontmatter) + `docs/book/src/tutorials/`
- **Acción:** cambiar `status: draft` → `active` en 01/02/03 tras validar. Sincronizar copias del book con la fuente canónica.
- **Verify:** grep — 0 tutoriales con status: draft; hashes book == fuente (o sync documentado)
- **Estado:** ✅ COMPLETADO

### Step 5: Verificación final de coverage + links
- **Archivos:** `docs/master-index.md`, `docs/README.md` (si referencian tutorials)
- **Acción:** verificar que el índice de docs lista el learning path; links internos sin 404.
- **Verify:** `scripts/validate-docs-coverage.ps1` (nota: falla por gaps preexistentes no relacionados — documentar); grep links rotos
- **Estado:** ✅ COMPLETADO

## Dependencias
- NUEVO-07 (ya completado) — migraciones con API real sirven de referencia de sintaxis correcta

## Notas
- Los tutoriales son contenido público de docs — Español NO (docs técnicas en inglés por Regla Doc Language Split).
- No tocar web/ salvo que haya links explícitos a tutorials (verificar primero).
- La fuente canónica probable es `docs/tutorials/` con `docs/book/src/tutorials/` como build del mdBook — confirmar antes de editar el book.

## Context Save Point
- **Fecha:** 2026-08-02
- **Branch:** develop
- **Decisiones:** — (pendientes del sub-agente)
- **Próxima tarea:** INV-006 (paralela, blog)

## Verification Log
- **2026-08-02 (sub-agente vanta-docs):**
  - `rg -l "vantadb\.connect|db\.space|\.similar_to|space\.configure" docs/tutorials docs/book/src/tutorials` → **0 resultados** ✅
  - `rg -c "status: draft" docs/tutorials` → **0** (exit 1) ✅
  - Tutoriales en `docs/tutorials/` (excl. index.md): **6** (01, 02, 03, 04, 05, migration-from-lancedb) — dentro de meta 5-7 ✅
  - Todos con `status: active` ✅
  - Snippets validados contra root `.venv` (`vantadb_py 0.5.0`): `put` (metadata+vector), `search_memory` (vector-only / hybrid `text_query` / keyword-only `[]` / `filters`), `get_memory`, `delete_memory`, `list_memory`, `put_batch(None, keys=..., vectors=...)`, `add_edge(node_id)` + `graph_bfs`, `explain_memory_search`, `export_namespace`, `rebuild_index`, `list_namespaces` → **ALL OK** ✅
  - **Hallazgo:** `put_batch()` keyword API requiere `entries=None` como primer arg posicional en 0.5.0 (el docstring omite el `None` y falla en runtime) — documentado en 02/05 con la forma correcta.
  - Fuente canónica confirmada: `docs/tutorials/` — el book (`docs/book/src/tutorials/`) usa `{{#include}}` stubs → sincronización automática por diseño.
  - Links internos: todos los targets existen (tutorials + `../api/PYTHON_SDK.md`) ✅
  - `scripts/validate-docs-coverage.ps1` → falla por gaps PREEXISTENTES no relacionados: script referencia `src/sdk/search.rs` inexistente + gaps en CONFIGURATION.md/EMBEDDED_SDK.md/PYTHON_SDK.md (bulk_import, graph_*, etc.) ajenos a tutorials. No corregidos (fuera de scope).
