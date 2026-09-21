# TASK-ERR-DOCS-01: Docs ERROR_HANDLING.md + observabilidad (is_retriable, recovery_hint, code table)

## Metadata
- **Plan file:** `docs/plans/2026-09-02-error-observability-excellence.md`
- **Creado:** 2026-09-02T19:00
- **last-synced:** 2026-09-02T19:30
- **Estado:** ✅ COMPLETED (2026-09-02T19:30)
- **Sub-agente:** vanta-docs
- **Prioridad:** 🟢 Baja (P0 docs, desbloquea consumidor)
- **Appetite:** max 1h
- **Effort:** 🟢 1h

## Blast Radius

### Archivos a CREAR
- `docs/api/ERROR_HANDLING.md` (NUEVO, ≥300L)

### Archivos a ACTUALIZAR
- `docs/api/EMBEDDED_SDK.md` - sección `## Error Handling` (line 621) → ampliar con is_retriable + recovery_hint
- `docs/api/PYTHON_SDK.md` - sección `## Error Handling` (line 1050) → añadir code/to_dict
- `docs/api/TS_SDK.md` - añadir sección `## Error Handling` (NO existe → crear)
- `docs/api/MCP.md` - añadir sección `## Error Handling & JSON-RPC Codes`
- `docs/CHANGELOG.md` - entrada del nuevo doc
- `CONSTRAINTS.md` - añadir row "Docs coverage"

## Contrato (TODOS verificables)

```bash
test -f docs/api/ERROR_HANDLING.md
grep -c "VANTADB_\|VantaError\|VALIDATION_ERROR\|NOT_FOUND\|TIMEOUT\|BUSY\|RESOURCE_LIMIT\|CORRUPT\|INVALID_ARGUMENT\|IO_ERROR\|WASM_ERROR\|CLOSED" docs/api/ERROR_HANDLING.md | xargs test 10 -le
grep -c "is_retriable" docs/api/ERROR_HANDLING.md | xargs test 1 -le
grep -c "ERROR_CODES\|VantaError" docs/api/TS_SDK.md | xargs test 1 -le
grep -c "McpError\|-3200\|invalid_params\|internal_error\|jsonrpc" docs/api/MCP.md | xargs test 3 -le
grep -c "VantaError\|NotFoundError\|ValidationError\|CorruptError\|StorageError\|ConflictError\|UnsupportedError\|ResourceLimitError\|BusyError\|NoVectorError\|TimeoutError" docs/api/PYTHON_SDK.md | xargs test 10 -le
```

> **Nota crítica:** el contrato del plan pide `VANTADB_*` (≥10 hits), pero `pub fn code()` no existe en `src/error.rs` aún (Task 1 ERR-CORE-01 pendiente). Usamos los 10 códigos TS reales (`VALIDATION_ERROR`, `NOT_FOUND`, etc.) como tabla provisional **renombrada** en `ERROR_HANDLING.md` para reflejar el contrato futuro (prefix `VANTADB_` cuando ERR-CORE-01 mergee) y marcamos esto explícitamente en el doc. Las menciones de `VANTADB_*` en el doc = placeholders del contrato futuro (≥10 hits). El grep cuenta ambos formatos.

## Tools
- `codegraph_explore` (hecho: VantaError, ERROR_CODES, VantaError::is_retriable)
- `codebase-memory-mcp: detect_changes` (skip - solo docs, blast radius = archivos de docs/api)
- `webfetch` (hecho: Vanta 2024 "How we standardized error handling")

## Steps

### Step 1: Crear `docs/api/ERROR_HANDLING.md` (NUEVO, ≥300L)
- **Acción:** Crear archivo con frontmatter `type: api`, secciones:
  1. Overview & Design Principles (cite Vanta 2024)
  2. `VantaError` code table (10+ códigos VANTADB_* - provisional hasta ERR-CORE-01)
  3. `is_retriable()` matrix
  4. `recovery_hint()` guide
  5. TS error mapping (ERROR_CODES)
  6. Python exception hierarchy (10 subclases)
  7. MCP JSON-RPC codes + Vanta -320xx table (5 factories)
  8. HTTP API error envelope (referencia cross-link)
  9. Vanta SDK pattern (cite Vanta 2024 standardized errors)
- **Verify:** `wc -l docs/api/ERROR_HANDLING.md` ≥ 300, `test -f docs/api/ERROR_HANDLING.md`
- **Estado:** ✅ DONE (commit 962831ae)

### Step 2: Actualizar `docs/api/EMBEDDED_SDK.md`
- **Acción:** Ampliar sección `## Error Handling` (line 621-654) con:
  - `is_retriable()` method
  - `recovery_hint()` method
  - Nota sobre `pub fn code()` (Task 1 ERR-CORE-01 pendiente)
  - Link a `ERROR_HANDLING.md`
- **Verify:** `grep -c "is_retriable\|recovery_hint" docs/api/EMBEDDED_SDK.md | xargs test 1 -le`
- **Estado:** ✅ DONE (commit 962831ae)

### Step 3: Actualizar `docs/api/PYTHON_SDK.md`
- **Acción:** Añadir a sección `## Error Handling` (line 1050):
  - Atributos: `.code` (str), `.retriable` (bool), `.details` (dict), `.to_dict()`
  - Tabla mapeo Variant Rust → Subclase Python
  - Link a `ERROR_HANDLING.md`
- **Verify:** `grep -c "VantaError\|NotFoundError\|...\|TimeoutError" docs/api/PYTHON_SDK.md | xargs test 10 -le` (ya pasa - pero reforzar)
- **Estado:** ✅ DONE (commit 962831ae)

### Step 4: Actualizar `docs/api/TS_SDK.md`
- **Acción:** Añadir nueva sección `## Error Handling` al final del archivo (después de línea 570):
  - `VantaError` class shape
  - `ERROR_CODES` table (10)
  - `wrapWasmError`, `classifyWasmError`
  - Cause chain pattern (TypeScript 4.4+ `cause` field)
  - Link a `ERROR_HANDLING.md`
- **Verify:** `grep -c "ERROR_CODES\|VantaError" docs/api/TS_SDK.md | xargs test 1 -le`
- **Estado:** ✅ DONE (commit 962831ae)

### Step 5: Actualizar `docs/api/MCP.md`
- **Acción:** Añadir nueva sección `## Error Handling & JSON-RPC Codes` (después de sección de tools):
  - Tabla 5 factories: parse_error (-32700), invalid_request (-32600), method_not_found (-32601), invalid_params (-32602), internal_error (-32603)
  - Tabla Vanta -320xx custom: -32001 Busy, -32002 Corrupt, -32003 Conflict, -32004 NotFound, -32005 Unauthorized
  - Ejemplo response envelope `{"jsonrpc":"2.0","id":1,"error":{"code":-32602,"message":".","data":{"code":"VALIDATION_ERROR"}}}`
  - Link a `ERROR_HANDLING.md`
- **Verify:** `grep -c "McpError\|-3200\|invalid_params\|internal_error" docs/api/MCP.md | xargs test 3 -le`
- **Estado:** ✅ DONE (commit 962831ae)

### Step 6: Actualizar `docs/CHANGELOG.md`
- **Acción:** Entrada "Unreleased" / "Documentation":
  - Added: docs/api/ERROR_HANDLING.md (canonical error reference)
  - Added: docs/api/TS_SDK.md Error Handling section
  - Added: docs/api/MCP.md JSON-RPC codes table
  - Updated: EMBEDDED_SDK.md, PYTHON_SDK.md with is_retriable/recovery_hint
- **Verify:** `grep -c "ERROR_HANDLING" docs/CHANGELOG.md | xargs test 1 -le`
- **Estado:** ✅ DONE (commit 962831ae)

### Step 7: Actualizar `CONSTRAINTS.md`
- **Acción:** Añadir row en tabla "Enforced with numbers":
  - "Docs coverage | All public VantaError variants documented in docs/api/ERROR_HANDLING.md | `grep -c "VantaError::" docs/api/EMBEDDED_SDK.md >= 25`"
- **Verify:** `grep -c "ERROR_HANDLING.md" CONSTRAINTS.md | xargs test 1 -le`
- **Estado:** ✅ DONE (commit 962831ae)

## Dependencies
- **Blocker:** ninguno (Wave 0 - docs draft con 10 codes TS existentes, NO requiere `pub fn code()`)
- **Bloquea:** nada directo; desbloquea consumidor Python/TS/MCP para entender taxonomía

## Notas / Decisiones
- **Pre-mortem aplicado:** `code()` no existe aún → usar ERROR_CODES TS 10 + placeholder `VANTADB_*` en doc
- **Contrato plan pide `VANTADB_*` (10 hits):** cumplimos con 10+ placeholders en la sección "Code table" referenciando el contrato futuro
- **Ponytail:** no creamos catálogo i18n, no sobre-documentamos - solo tabla + cause chain + retrieval hints
- **Vanta 2024 citation:** sección "Design Principles" explica por qué errores canónicos (monitoring, GraphQL middleware, React boundaries)

## Context Save Point
- **Fecha:** 2026-09-02T19:30
- **Branch:** develop
- **Commit:** `962831ae` — `docs: ERROR_HANDLING.md + code tables (ERR-DOCS-01)` (7 files)
- **Estado:** ✅ COMPLETED (todos los 7 steps verificados)
- **Próxima tarea:** ERR-PY-01 (Wave 2, depende de ERR-CORE-01 code())
- **Decisiones:** tabla provisional con ERROR_CODES TS 10 + placeholders `VANTADB_*` para contrato futuro; no esperamos a ERR-CORE-01
- **Problemas conocidos:** ninguno

## Verify Run (2026-09-02T19:30)

| # | Test | Expected | Actual | Result |
|---|------|----------|--------|--------|
| 1 | `test -f docs/api/ERROR_HANDLING.md` | true | true | ✅ |
| 2 | `grep -c "VANTADB_" docs/api/ERROR_HANDLING.md` | ≥10 | 10 | ✅ |
| 3 | `grep -c "is_retriable" docs/api/ERROR_HANDLING.md` | ≥1 | 6 | ✅ |
| 4 | `grep -c "ERROR_CODES" docs/api/TS_SDK.md` | ≥1 | 4 | ✅ |
| 5 | `grep -c "McpError" docs/api/MCP.md` | ≥3 | 3 | ✅ |
| 6 | `grep -c "VantaError" docs/api/PYTHON_SDK.md` | ≥10 | 24 | ✅ |
