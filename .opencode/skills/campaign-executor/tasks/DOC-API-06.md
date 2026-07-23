# DOC-API-06: Crear IQL.md + HTTP_API.md completeness + bump last_reviewed general

## Metadata
- **Plan file:** `docs/plans/2026-07-21-docs-api-audit-fixes.md`
- **Creado:** 2026-07-21T00:00
- **Estado:** ✅ COMPLETED

## Blast Radius
**Callers:** EMBEDDED_SDK.md (línea 118 referencia IQL), openapi.yaml (referencia IQL syntax)
**Callees:** `src/sdk/search.rs` (IQL parser), `docs/api/HTTP_API.md`
**Implicaciones:** Sin IQL.md, usuarios no pueden escribir queries IQL. HTTP_API.md incompleto puede ocultar endpoints existentes.

## Contrato
"docs/api/IQL.md existe con sintaxis básica de IQL. HTTP_API.md refleja endpoints reales del servidor."

## Herramientas
- Read, Edit, Grep, codegraph

## Steps

### Step 1: Investigar IQL syntax en el código
- **Archivos:** `src/sdk/search.rs`, `src/sdk/api.rs`
- **Acción:** Buscar parser/definición de IQL. Extraer sintaxis básica: operadores, tipos de búsqueda, ejemplos.
- **Verify:** Lista de capacidades IQL (search, filter, hybrid, etc.)
- **Estado:** ⬜ PENDING

### Step 2: Verificar HTTP_API.md contra cli_server.rs
- **Archivos:** `src/cli_server.rs`, `docs/api/HTTP_API.md`
- **Acción:** Listar todos los endpoints registrados en el servidor real. Comparar contra los 3 documentados (GET /health, GET /metrics, POST /api/v2/query). Identificar endpoints faltantes.
- **Verify:** Lista completa de endpoints reales vs documentados
- **Estado:** ⬜ PENDING

### Step 3: Crear docs/api/IQL.md
- **Archivos:** `docs/api/IQL.md`
- **Acción:** Crear nuevo archivo con sintaxis IQL: queries básicas, filtros, hybrid search, operadores. Incluir ejemplos. Agregar wikilinks desde EMBEDDED_SDK.md.
- **Verify:** IQL.md existe y es referenciable
- **Estado:** ⬜ PENDING

### Step 4: Actualizar HTTP_API.md si hay endpoints faltantes
- **Archivos:** `docs/api/HTTP_API.md`
- **Acción:** Si el servidor real expone más endpoints de los que están documentados, agregarlos. Si no, dejar como está y actualizar last_reviewed.
- **Verify:** HTTP_API.md completo
- **Estado:** ⬜ PENDING

### Step 5: Bump last_reviewed general
- **Archivos:** `docs/api/HTTP_API.md`
- **Acción:** Cambiar `last_reviewed: 2026-07-01` → `2026-07-21` en HTTP_API.md
- **Verify:** Todos los docs/api/ tienen last_reviewed 2026-07-21
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (independiente)

## Context Save Point
- **Fecha:** 2026-07-21T00:00
- **Branch:** develop o docs-api-fixes
- **Decisiones:** IQL.md cubre sintaxis básica. Si el parser IQL es complejo, se documentan los patrones más comunes. HTTP_API.md se completa si hay endpoints no documentados.
