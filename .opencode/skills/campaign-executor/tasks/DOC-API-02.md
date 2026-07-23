# DOC-API-02: Fix openapi.yaml — remover semantic_cluster, alinear NodeDTO, bump version

## Metadata
- **Plan file:** `docs/plans/2026-07-21-docs-api-audit-fixes.md`
- **Creado:** 2026-07-21T00:00
- **Completado:** 2026-07-22
- **Estado:** ✅ COMPLETED

## Blast Radius
**Callers:** Ninguno directo. HTTP_API.md referencia openapi.yaml. MCP.md tiene versión `0.1.5` que debería coincidir.
**Callees:** `src/sdk/types.rs` (VantaNodeRecord real)
**Implicaciones:** Solo documentación. No afecta servidor real.

## Contrato
"openapi.yaml NodeDTO tiene exactamente los campos de VantaNodeRecord real: id, fields, vector, vector_dimensions, edges, confidence_score, importance, hits, last_accessed, epoch, tier, is_alive. Sin semantic_cluster, relational."

## Herramientas
- Read, Edit, Grep, codegraph

## Steps

### Step 1: Leer código real VantaNodeRecord
- **Archivos:** `src/sdk/types.rs`
- **Acción:** Verificar campos reales de `VantaNodeRecord` o `VantaMemoryRecord`. Buscar el struct que expone la API REST.
- **Verify:** Lista completa de campos documentada
- **Estado:** ✅ COMPLETED

### Step 2: Leer openapi.yaml completo
- **Archivos:** `docs/api/openapi.yaml`
- **Acción:** Mapear NodeDTO actual vs VantaNodeRecord real. Identificar campos que existen en el YAML pero no en código (`semantic_cluster`, `bitset`, `flags`, `ext_metadata`). Identificar campos que existen en código pero no en YAML.
- **Verify:** Lista de diferencias
- **Estado:** ✅ COMPLETED

### Step 3: Corregir NodeDTO schema
- **Archivos:** `docs/api/openapi.yaml`
- **Acción:** Reemplazar NodeDTO completo con campos reales. Eliminar `semantic_cluster`, `bitset`, `flags`, `ext_metadata`. Mantener campos de VantaNodeRecord.
- **Verify:** openapi.yaml es válido (parser YAML) y campos coinciden con VantaNodeRecord
- **Estado:** ✅ COMPLETED

### Step 4: Bump version en openapi.yaml
- **Archivos:** `docs/api/openapi.yaml`
- **Acción:** Version ya es `0.4.0` (project version). No requiere bump.
- **Verify:** ✅ SKIPPED — versión actual ya correcta
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (independiente)

## Context Save Point
- **Fecha:** 2026-07-21T00:00
- **Branch:** develop o docs-api-fixes
- **Decisiones:** La versión openapi se alinea con la versión del proyecto (0.4.0 actual). NodeDTO se mapea 1:1 con VantaNodeRecord.
