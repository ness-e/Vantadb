# SDKB-01 — Mapa namespace ↔ método + diseño de sub-clientes

> Plan: `docs/plans/2026-08-22-vantadb-bindings-sdk.md` (Task 1)
> Contrato: tabla committeada cubriendo 100% de métodos públicos de cada SDK,
> cada uno a exactamente un dominio (memory/graph/conversation/skills/wiki/system),
> con los aún-no-reexpuestos marcados.

## Impacto mapeado (Regla 0)

- Archivos leídos completos: ninguno requerido — archivo NUEVO (`docs/api/BINDINGS_NAMESPACES.md`).
- Referencias entrantes: plan Tasks 2/3/4 citan el mapa como contrato.
- Referencias salientes: referencia decisiones D42/D43/D45 del plan file.
- Veredicto: aditivo puro, riesgo cero. NO se toca código (regla explícita de la tarea).

## Superficie verificada (grep real, no la lista del plan)

| SDK | Count | Fuente |
|---|---|---|
| WASM | 43 `pub fn` (2 constructors `new`/`open`, 1 lifecycle `close`) | `rg 'pub fn (\w+)' vantadb-wasm/src/lib.rs` |
| TS | 38 métodos públicos clase `VantaDB` | grep métodos en `vantadb-ts/src/vantadb.ts`; `native.ts` implementa subset (10) |
| Python | ~43 métodos pyclass + `connect()` módulo | grep `fn \w+` en `vantadb-python/src/lib.rs` |

### Diferencias entre SDKs encontradas (hallazgos nuevos vs Paso 0 del lead)

1. **`supersede`: Python SOLAMENTE** — ni wasm ni TS lo exponen (el contexto del lead lo listaba para wasm: incorrecto).
2. **Python `insert`/`get`/`delete` son node-level** (`id: u128`, dominio graph), NO memory ns/key como en TS/WASM.
3. **Solo wasm+TS**: `delete_by_filter`, `search_vector`, `audit_text_index_deep`, `export_namespace_filtered`, `import_records`.
4. **Solo wasm**: nada exclusivo (es superconjunto de TS salvo bulk_import*).
   - Corrección: `bulk_import`/`bulk_import_bytes` existen en wasm Y Python pero NO en TS.
5. **Solo Python**: `supersede`, `put_batch_raw`, `search_batch`, `search_batch_requests`, `hardware_profile`, `graph_page_rank`, `graph_degree_centrality`, `recover_archived_nodes`, `insert/get/delete` node-level.
6. **wasm tiene `graph_degree`; Python tiene `graph_degree_centrality`** (nombres distintos, misma operación).
7. **conversation/skills/wiki**: cero métodos expuestos hoy en los 3 SDKs (core-only, D43). Wiki recibe solo `recover_archived_nodes`.

## Steps

### Step 1 ✅ DONE — Crear `docs/api/BINDINGS_NAMESPACES.md`
Tablas por SDK (método→dominio exactamente uno) + diseño sub-clientes v1
(TS getters delegantes frozen; Python: recomendación pyclass delegante con `#[getter]`,
fallback helper functions según stop-condition del plan).
Totales: WASM 43 (mem 12/graph 10/sys 21) · TS 38 (12/10/16) · Python 45 filas
(44 pymethods + connect: mem 15/graph 10/wiki 1/sys 18).

### Step 2 ✅ DONE — Verify mecánico cobertura 100%
- Conteo filas por sección vs grep real: WASM 43=43 ✔ · TS 38=38 ✔ · PY 45=45 ✔ (44 pymethods públicos verificados por set-diff; `__repr__` y helpers privados excluidos)
- Sin duplicados por sección (regex `(?m)^\| \`(\w+)\``): unique==rows en los 3.
- `scripts/validate-docs-coverage.ps1` → **0 gaps** ✅

### Step 3 ✅ DONE — Cierre
Sin commit (regla de la tarea). Recitation enviada vía MCP.

## Context Save Point

(nada aún — sin trabajo parcial)

## Verificación

- `scripts/validate-docs-coverage.ps1`
- Conteo mecánico inline (Step 2)

## Deuda / notas

- Sin commit (regla de la tarea: NO commitear).
- Inglés técnico para el doc.
