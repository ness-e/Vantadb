# DOC-API-04: Fix PYTHON_SDK.md — métodos faltantes + tipos correctos

## Metadata
- **Plan file:** `docs/plans/2026-07-21-docs-api-audit-fixes.md`
- **Creado:** 2026-07-21T00:00
- **Estado:** ✅ COMPLETED

## Blast Radius
**Callers:** Ninguno. Doc-only.
**Callees:** `vantadb-python/src/lib.rs` (PyO3 bindings)
**Implicaciones:** Usuarios Python no encuentran métodos que existen. Tipo incorrecto `vector: List[float]` puede confundir a usuarios de NumPy.

## Contrato
"PYTHON_SDK.md documenta: search_batch, flush, VantaVector, delete_by_filter, similar_to_key, count con firmas que coinciden con el código real."

## Herramientas
- Read, Edit, Grep, codegraph

## Steps

### Step 1: Leer PYTHON_SDK.md actual
- **Archivos:** `docs/api/PYTHON_SDK.md`
- **Acción:** Identificar qué métodos están documentados actualmente. Mapear lagunas.
- **Verify:** Lista de métodos documentados vs no documentados
- **Nota:** Todos los 37 métodos públicos existen con firma. Faltan descripciones y ejemplos en ~20 métodos.
- **Estado:** ✅ COMPLETED

### Step 2: Leer código real PyO3
- **Archivos:** `vantadb-python/src/lib.rs`
- **Acción:** Extraer firmas completas de: `search_batch` (L1432), `flush`, `VantaVector` (class export), `delete_by_filter`, `similar_to_key`, `count`. Verificar firmas exactas (parámetros, tipos, defaults).
- **Verify:** Firmas verificadas. `search_batch` / `flush` / `VantaVector` existen. `delete_by_filter`, `similar_to_key`, `count` NO existen en el código Rust.
- **Nota:** `vector` acepta `List[float]`, `VantaVector`, `np.ndarray`, o cualquier buffer protocol. Documentado como `VectorInput`.
- **Estado:** ✅ COMPLETED

### Step 3: Verificar tipo vector parameter
- **Archivos:** `vantadb-python/src/lib.rs`
- **Acción:** Confirmar qué tipos acepta (List[float], VantaVector, np.ndarray, buffer protocol) vía `extract_vector()`.
- **Verify:** Documentación del parámetro vector actualizada a `VectorInput` correcta.
- **Estado:** ✅ COMPLETED

### Step 4: Agregar métodos/contenido faltante a PYTHON_SDK.md
- **Archivos:** `docs/api/PYTHON_SDK.md`
- **Acción:** Agregar descripciones y ejemplos a: `insert`, `get`, `delete`, `search`, `search_batch`, `add_edge`, `graph_bfs`, `graph_dfs`, `graph_topological_sort`, `graph_is_dag`, `flush`, `compact_wal`, `purge_expired`, `rebuild_index`, `compact_layout`, `list_namespaces`, `export_namespace`, `export_all`, `import_file`, `audit_text_index`, `repair_text_index`, `operational_metrics`, `capabilities`, `hardware_profile`, `generate_snippet`, `close`, `put_batch_raw`.
- **Nota:** `delete_by_filter`, `similar_to_key`, `count` no existen en el código. Se mantienen como "(not yet exposed)".
- **Estado:** ✅ COMPLETED

### Step 5: last_reviewed actualizado
- **Archivos:** `docs/api/PYTHON_SDK.md`
- **Estado:** ✅ (ya estaba en `2026-07-21`)

## Dependencias
- Ninguna (independiente)

## Context Save Point
- **Fecha:** 2026-07-21T00:00
- **Branch:** develop o docs-api-fixes
- **Decisiones:** El tipo del parámetro `vector` en Python acepta List[float], VantaVector, np.ndarray, y buffer protocol. Se documenta como `VectorInput` (tipo unión).
