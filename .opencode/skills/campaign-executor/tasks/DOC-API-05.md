# DOC-API-05: Fix TS_SDK.md — connect_idb, searchVector vs search_vector, types

## Metadata
- **Plan file:** `docs/plans/2026-07-21-docs-api-audit-fixes.md`
- **Creado:** 2026-07-21T00:00
- **Estado:** ⬜ PENDING

## Blast Radius
**Callers:** Ninguno. Doc-only.
**Callees:** `vantadb-wasm/src/lib.rs` (WASM bindings), `vantadb-ts/src/vantadb.ts` (TS wrapper)
**Implicaciones:** Usuarios TS no encuentran `connect_idb()` (IndexedDB). `searchVector()` puede no existir como método directo.

## Contrato
"TS_SDK.md documenta connect_idb() para IndexedDB y usa el nombre correcto searchVector/search_vector según el wrapper TS real."

## Herramientas
- Read, Edit, Grep, codegraph

## Steps

### Step 1: Leer TS_SDK.md actual
- **Archivos:** `docs/api/TS_SDK.md`
- **Acción:** Identificar líneas que mencionan `connect()`, `searchVector()`, `MemoryRecord`.
- **Verify:** Lista de afirmaciones a verificar
- **Estado:** ⬜ PENDING

### Step 2: Verificar API real del WASM + TS wrapper
- **Archivos:** `vantadb-wasm/src/lib.rs`, `vantadb-ts/src/vantadb.ts`, `vantadb-ts/src/types.ts`
- **Acción:** Confirmar: ¿Cómo se conecta realmente a IndexedDB? (connect_idb). ¿searchVector existe o es search_vector? (wrapper camelCase). ¿MemoryRecord.node_id es string o number? Tipos de u128 vs u64 en serialización JS.
- **Verify:** Mapeo completo de nombres WASM → TS
- **Estado:** ⬜ PENDING

### Step 3: Actualizar TS_SDK.md sección de conexión
- **Archivos:** `docs/api/TS_SDK.md`
- **Acción:** Agregar `connect_idb()` como método de conexión IndexedDB. Aclarar relación entre `searchVector()` (TS wrapper) y `search_vector()` (WASM raw). Corregir tipos si es necesario.
- **Verify:** Las 3 correcciones aplicadas
- **Estado:** ⬜ PENDING

### Step 4: Bump last_reviewed
- **Archivos:** `docs/api/TS_SDK.md`
- **Acción:** Cambiar `last_reviewed: 2026-07-04` → `2026-07-21`
- **Verify:** grep "last_reviewed" muestra 2026-07-21
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (independiente)

## Context Save Point
- **Fecha:** 2026-07-21T00:00
- **Branch:** develop o docs-api-fixes
- **Decisiones:** El TS wrapper (vantadb-ts) expone API camelCase. El WASM raw (vantadb-wasm) expone snake_case. La doc se alinea con la API pública (TS wrapper). `connect_idb()` es el entry point para IndexedDB.
