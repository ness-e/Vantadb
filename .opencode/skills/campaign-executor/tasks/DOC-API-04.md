# DOC-API-04: Fix PYTHON_SDK.md — métodos faltantes + tipos correctos

## Metadata
- **Plan file:** `docs/plans/2026-07-21-docs-api-audit-fixes.md`
- **Creado:** 2026-07-21T00:00
- **Estado:** ⬜ PENDING

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
- **Estado:** ⬜ PENDING

### Step 2: Leer código real PyO3
- **Archivos:** `vantadb-python/src/lib.rs`
- **Acción:** Extraer firmas completas de: `search_batch` (L1432), `flush`, `VantaVector` (class export), `delete_by_filter`, `similar_to_key`, `count`. Verificar firmas exactas (parámetros, tipos, defaults).
- **Verify:** Lista de firmas correctas extraídas
- **Estado:** ⬜ PENDING

### Step 3: Verificar tipo vector parameter
- **Archivos:** `vantadb-python/src/lib.rs`
- **Acción:** Encontrar `extract_vector()` helper o el parámetro `vector` en `insert`. Confirmar qué tipos acepta (List[float], VantaVector, np.ndarray, buffer protocol).
- **Verify:** Documentación del parámetro vector actualizada
- **Estado:** ⬜ PENDING

### Step 4: Agregar métodos faltantes a PYTHON_SDK.md
- **Archivos:** `docs/api/PYTHON_SDK.md`
- **Acción:** Agregar secciones para `search_batch(vectors, top_k, ...)`, `flush()`, `VantaVector` class reference, `delete_by_filter(filter_expr)`, `similar_to_key(key)`, `count()`. Actualizar firma de `insert` para vector correcto.
- **Verify:** Las 6 funciones están documentadas con firma correcta
- **Estado:** ⬜ PENDING

### Step 5: Bump last_reviewed
- **Archivos:** `docs/api/PYTHON_SDK.md`
- **Acción:** Cambiar `last_reviewed: 2026-07-01` → `2026-07-21`
- **Verify:** grep "last_reviewed" muestra 2026-07-21
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (independiente)

## Context Save Point
- **Fecha:** 2026-07-21T00:00
- **Branch:** develop o docs-api-fixes
- **Decisiones:** El tipo del parámetro `vector` en Python acepta List[float], VantaVector, np.ndarray, y buffer protocol. Se documenta como `VectorInput` (tipo unión).
