# Bug Fix — DRV-002: put_batch duplica put()

- **Fecha:** 2026-07-23
- **Branch:** `develop`
- **Objetivo:** Refactor `src/sdk/api.rs` — `put_batch()` llama a `put_one()` en vez de duplicar lógica

## Tareas

### 1. Extraer helper `put_one` (REFACTOR)

**Archivos clave:** `src/sdk/api.rs:66-196`

**Descripción:** Extraer la lógica de validación + compute + insert de `put()` en un método privado `put_one()` que ambos `put()` y `put_batch()` usan.

**Criterio de éxito:** `put_batch` llama a `put_one` en el closure en vez de duplicar ~50 líneas. `cargo check` pasa. Tests existentes pasan.

**Esfuerzo:** 🟢 ~30min

**Detalles:**
- `put_one(input: VantaMemoryInput) -> Result<VantaMemoryRecord>`:
  1. validate_namespace/validate_key/validate_metadata
  2. engine_handle()
  3. get existing → node_id collision check
  4. compute timestamp, version, expires_at_ms
  5. build VantaMemoryRecord
  6. insert → replace_derived_indexes
- `put()` → delega a `put_one()`
- `put_batch()` → valida todos en loop, chunk, par_iter/iter → `put_one()`

## DO (always)
- ✅ `cargo check` must pass
- ✅ All existing tests must pass
- ✅ No new dependencies

## DEFER
- WASM/Python bindings mirror this pattern — deferred to separate PR

## SKIP
- No cambiar lógica de negocio ni firma pública

---

## Workflow: refactor
| Estado | Workflow |
|--------|----------|
| `audit` | Analyze code structure |
| `migrate` | Extraer put_one → adaptar put y put_batch |
| `verify` | cargo check + tests |
| `review` | Code review check |
| `accept` | Approve changes |
| `close` | Close task |

## Harness config
- Mode: sequential, one task
- Stall threshold: 2
- Timeout per iteration: 300s
