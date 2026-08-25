# FIND-04 — Cross-SDK `search()` parity (Python ↔ TypeScript)

> **Plan:** `docs/plans/2026-08-24-batch-review-mod-find.md`
> **Campaign:** 4b9e337a-2fd0-4625-9cba-e26ea37f780b
> **Tipo:** docs · **Estado:** ⬜ PENDING
> **Contrato:** tabla presente en ambos READMEs, link al doc de namespaces
> **Regla batch:** worker NO commitea — el lead verifica mecánico y commitea.

## Objetivo
Documentar la paridad cross-SDK de `search()` entre el Python SDK (`vantadb-python/`)
y el TypeScript/WASM SDK (`vantadb-ts/`): qué capacidades/parámetros están disponibles
en cada binding, y enlazar el doc canónico `docs/api/BINDINGS_NAMESPACES.md` desde
ambos READMEs de SDK.

## Archivos clave
- `vantadb-python/README.md` (añadir tabla + link BINDINGS_NAMESPACES.md)
- `vantadb-ts/README.md` (añadir tabla; link a BINDINGS_NAMESPACES.md ya existe)
- `docs/api/BINDINGS_NAMESPACES.md` (referencia canónica, confirmar cobertura)
- Fuentes de verificación: `vantadb-python/src/lib.rs`, `vantadb-ts/src/vantadb.ts`,
  `vantadb-ts/src/types.ts`, `docs/api/PYTHON_SDK.md`

## Hallazgos de verificación (código REAL, no inventado)
- **Python `search(vector, top_k=10)`** (`lib.rs:1596`) → `engine.search_vector` → pure vector ANN,
  devuelve `(node_id, distance)`; **sin** namespace/filters/text/distance_metric/explain.
- **Python híbrido** = `search_memory(namespace, query_vector, filters, text_query, top_k,
  distance_metric, method, explain, exclude_superseded)` (`lib.rs:1236`).
- Python también: `search_batch`, `search_batch_requests` (Python-only), `explain_memory_search`.
- **TS `search(request: SearchRequest)`** (`vantadb-ts/src/vantadb.ts:595`) → **hybrid**;
  `SearchRequest = {namespace, query_vector, filters?, text_query?, top_k?, distance_metric?, explain?}`
  (`types.ts:61`).
- **TS pure ANN** = `searchVector(vector, topK?)`; explain = `explainSearch(request)`.
- **Divergencia clave (hazard de paridad):** el nombre `search()` significa cosas distintas:
  Python = pure ANN; TS = hybrid. Coincide con `BINDINGS_NAMESPACES.md:31`.

## Steps
1. ⬜ Crear task file + verificar firmas reales de `search()` en ambos SDKs (codegraph/grep). ✅ hechos arriba.
2. ⬜ Editar `vantadb-python/README.md`: sección "Cross-SDK Search Parity" (tabla comparativa) + link a `docs/api/BINDINGS_NAMESPACES.md`.
3. ⬜ Editar `vantadb-ts/README.md`: sección "Cross-SDK Search Parity" (tabla comparativa).
4. ⬜ Verify: tabla presente en ambos READMEs; ruta `docs/api/BINDINGS_NAMESPACES.md` existe.
5. ⬜ Verificación mecánica para el lead (docs coverage; no compile requerido — solo markdown).

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** ambos READMEs, `BINDINGS_NAMESPACES.md`, `lib.rs` (search region),
  `types.ts` (SearchRequest), `PYTHON_SDK.md` (grep).
- **Referencias hacia dentro:** READMEs son raíz de sus crates; `docs/api/BINDINGS_NAMESPACES.md:7`
  ya enlaza `vantadb-ts/README.md#domain-sub-clients` y `PYTHON_SDK.md`. `PYTHON_SDK.md:60` ya enlaza
  `BINDINGS_NAMESPACES.md`.
- **Referencias salientes (añadidas):** Python README → `../docs/api/BINDINGS_NAMESPACES.md` (+
  `../docs/api/PYTHON_SDK.md` ya enlazado en TS README:194). Ambos READMEs → tabla de paridad.
- **Veredicto:** aditivo, solo markdown. No rompe nada. Sin cambios de código/semántica.

## Context Save Point
- Verificación completa de firmas. Ambos READMEs leídos. TS README ya enlaza BINDINGS; Python no.
