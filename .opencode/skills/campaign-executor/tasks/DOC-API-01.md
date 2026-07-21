# DOC-API-01: Fix EMBEDDED_SDK.md — tipos u64→u128 + VantaConfig + last_reviewed

## Metadata
- **Plan file:** `docs/plans/2026-07-21-docs-api-audit-fixes.md`
- **Creado:** 2026-07-21T00:00
- **Estado:** ⬜ PENDING

## Blast Radius
**Callers:** Ninguno (doc-only). Otros docs/api/ que referencian EMBEDDED_SDK types podrían necesitar alineación.
**Callees:** `src/sdk/types.rs` (VantaNodeRecord, Edge), `src/sdk/api.rs` (get_node, put), `src/sdk/graph.rs`
**Implicaciones:** Solo documentación. No afecta código.

## Contrato
"grep '\bu64\b' en docs/api/EMBEDDED_SDK.md después del fix no encuentra node_id, edge.target, node_id en firmas de función. Solo hits válidos (configuraciones, sizes, etc.)"

## Herramientas
- Read, Edit, Grep, codegraph

## Steps

### Step 1: Leer EMBEDDED_SDK.md y mapear ocurrencias de u64 incorrectas
- **Archivos:** `docs/api/EMBEDDED_SDK.md`
- **Acción:** Leer el archivo completo. Identificar todas las líneas donde `node_id`, `id`, `target` están tipados como `u64` pero el código real usa `u128`.
- **Verify:** `grep -n "u64" docs/api/EMBEDDED_SDK.md` produce lista de líneas a corregir
- **Estado:** ⬜ PENDING

### Step 2: Leer código real para confirmar tipos exactos
- **Archivos:** `src/sdk/types.rs`, `src/sdk/api.rs`, `src/sdk/graph.rs`
- **Acción:** Verificar que `VantaNodeRecord.id`, `Edge.target`, `get_node()` parámetro, `put()` parámetro usan `u128`.
- **Verify:** `codegraph_explore "VantaNodeRecord Edge api.rs node_id u128"` confirma tipos
- **Estado:** ⬜ PENDING

### Step 3: Reemplazar u64 por u128 en EMBEDDED_SDK.md
- **Archivos:** `docs/api/EMBEDDED_SDK.md`
- **Acción:** Editar todas las ocurrencias de `u64` en contextos de node_id/edge target → `u128`. No cambiar `u64` en contextos válidos (timestamps, version numbers, config sizes).
- **Verify:** `grep -n "u64" docs/api/EMBEDDED_SDK.md` solo muestra hits válidos
- **Estado:** ⬜ PENDING

### Step 4: Revisar ejemplos de código EMBEDDED_SDK.md
- **Archivos:** `docs/api/EMBEDDED_SDK.md`
- **Acción:** Verificar que los ejemplos de código usan valores u128 (números grandes) para node_id, no números pequeños que funcionan con ambos tipos.
- **Verify:** Ejemplos usan `id: 42` (funciona con u64 y u128) — ok. Verificar que no hay conversiones forzadas.
- **Estado:** ⬜ PENDING

### Step 5: Bump last_reviewed a 2026-07-21
- **Archivos:** `docs/api/EMBEDDED_SDK.md`
- **Acción:** Cambiar `last_reviewed: 2026-07-01` → `2026-07-21`
- **Verify:** grep "last_reviewed" muestra 2026-07-21
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (independiente)

## Context Save Point
- **Fecha:** 2026-07-21T00:00
- **Branch:** develop (o nueva branch docs-api-fixes)
- **Decisiones:** Uso `u128` en toda la doc de IDs de nodos. Los timestamps y configs numéricas pueden quedar como `u64`.
