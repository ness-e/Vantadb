# MOD-20: Excepciones Python tipadas (VantaError) + query_structured() estructurado

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-core-server-mcp.md
- **Fuente:** plan file Task 7 (MOD-20) — backlog DX Python
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Tipo:** Python SDK (PyO3) + Docs
- **Turns estimados:** 18
- **Creado:** 2026-08-25
- **last-synced:** 2026-08-25
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0

## Spec (confirmada por el orquestador en el prompt de la tarea)

**Problema 1 — Excepciones genéricas:** `map_vanta_error` (`vantadb-python/src/convert.rs:716-741`)
emite builtins Python (`KeyError`, `ValueError`, `TimeoutError`, `FileNotFoundError`, `OSError`,
`RuntimeError` catch-all). No existe una jerarquía propia `VantaError` tipada; los variants raros
del core colapsan en `RuntimeError` indistinguibles.

**Problema 2 — `query()`:** `VantaDB.query` (`vantadb-python/src/lib.rs:1735`) retorna un string
formateado (`format_query_result`) en vez de una estructura. Los tests existentes
(`test_sdk.py:944-960`) dependen del string → **aditivo**: añadir `query_structured()` (dict),
mantener `query()` intacto (ponytail, decidido por el orquestador).

### Decisión de diseño (api-and-interface-design + source-driven)

1. **Jerarquía de excepciones con herencia SIMPLE** — la doc oficial de Python desaconseja el
   multiple inheritance de excepciones (conflictos de layout de memoria) y `create_exception!`
   de PyO3 solo soporta una base única. Por eso: `VantaError(RuntimeError)` como base y subclases
   que heredan solo de `VantaError`. Fuente: docs.python.org/3/library/exceptions.html
   ("recommended to only subclass one exception type at a time") + pyo3.rs/main/exception
   (single base).
2. **`VantaError(RuntimeError)`** → preserva `except RuntimeError` (backward compat, opción
   "mantener RuntimeError como alias/fallback" que eligió el orquestador). Todas las subclases
   son `RuntimeError` transitivamente.
3. **Migración documentada:** los builtins `KeyError`/`ValueError` que emiten las operaciones de
   memoria dejan de emitirse en favor de `NotFoundError`/`ValidationError` (subclases VantaError).
   Se actualizan los tests internos que los assertaban y se documenta en `docs/api/PYTHON_SDK.md`.
   Bar de backward-compat del orquestador = `RuntimeError` (preservado); builtins específicos =
   migración documentada.
4. **`query_structured()` aditivo** — no rompe `query()` ni sus callers.

### Mapeo variant core → subclase Python

| VantaError core (src/error.rs) | Subclase Python |
|---|---|
| `NotFound`, `NodeNotFound` | `NotFoundError` |
| `ValidationError`, `DuplicateNode`, `DimensionMismatch`, `SerializationError`, `InvalidInput`, `SchemaError`, `NodeIdCollision`, `IqlParseError`, `IqlError` | `ValidationError` |
| `IncompatibleFormat`, `WALVersionMismatch`, `WalError` | `CorruptError` |
| `IoError`, `BackendError` | `StorageError` |
| `ExecutionConflict`, `CycleDetected` | `ConflictError` |
| `UnsupportedOperation` | `UnsupportedError` |
| `ResourceLimit` | `ResourceLimitError` |
| `DatabaseBusy`, `NotInitialized` | `BusyError` |
| `NoVectorForKey` | `NoVectorError` |
| `Timeout` | `TimeoutError` (vantadb, no el builtin) |
| `RuntimeError`, `Generic`, `CliError`, `SearchError`, `RestoreError`, `BackupError`, resto | `VantaError` (base) |

### Estructura de `query_structured()` (dict con discriminante `kind`)
- `Read(nodes)` → `{"kind": "read", "nodes": [{"id": str, "tier": str, "confidence": float, "hits": int}, ...]}`
- `Write{...}` → `{"kind": "write", "affected_nodes": int, "message": str, "node_id": str|None}`
- `StaleContext{node_id}` → `{"kind": "stale_context", "node_id": str}`
- `u128` ids como `str` (evita pérdida de precisión; patrón ya usado en MCP/CLI).

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `map_vanta_error` usado por TODOS los métodos de `VantaDB`/subclients en `vantadb-python/src/lib.rs` y `vector.rs`; `query()` usado por `AsyncVantaDB.query` (`__init__.py`), `SystemClient` (macro `forward_to_db!`), tests `test_sdk.py`/`test_subclients.py`/`test_async_smoke.py` |
| Callees | `create_exception!` (PyO3); `VantaError` core (`src/error.rs`); `VantaQueryResult` (`src/sdk/types.rs:735`); `VantaNodeRecord` (`src/sdk/serialization/graph_types.rs:56`) |
| Implicaciones | No rompe `query()` (aditivo). Cambia tipos de excepción de builtins a VantaError (migración documentada). `except RuntimeError`/`except Exception` intactos. Anti-drift MOD-18 (`test_stub_drift.py`) exige declarar excepciones + `query_structured` en los `.pyi` |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-python/src/convert.rs` (786L), `vantadb-python/src/lib.rs`
  (2328L, ranges clave), `vantadb-python/vantadb_py/__init__.py` (417L), `vantadb-python/vantadb_py/vantadb_py.pyi`
  (435L), `vantadb-python/tests/test_sdk.py` (ranges asserts), `vantadb-python/tests/test_stub_drift.py` (330L),
  `src/sdk/types.rs:733-753` (VantaQueryResult), `src/sdk/serialization/graph_types.rs:56-82` (VantaNodeRecord),
  `src/error.rs` (VantaError enum)
- **Referencias hacia dentro:** `map_vanta_error` importado en lib.rs:34 (`use crate::convert::{...map_vanta_error...}`)
  y usado en ~40 métodos; `format_query_result` importado en lib.rs:34, usado en `query()` (lib.rs:1743)
- **Referencias entrantes (a los archivos editados):** `__init__.py` importa de `.vantadb_py` (nombres en
  `__init__.py:12-20`); `vantadb_py.pyi` es la fuente de verdad que valida `test_stub_drift.py`;
  adapters Python (`integrations/*`) usan `search_memory`, no `query`/excepciones VantaError
- **Veredicto impacto:** medio — API pública del binding, pero cambio aditivo + `except RuntimeError`
  preservado. No toca core (`src/`), solo binding + docs

## Contrato
`python -m pytest vantadb-python/tests/` pasa; `query_structured()` retorna dict (estructura) sin
romper `query()` (str); errores tipados VantaError (jerarquía) no solo RuntimeError, con
`except RuntimeError`/`except VantaError` funcionando; `cargo check -p vantadb_py` + fmt + clippy
pasan; `docs/api/PYTHON_SDK.md` actualizada (excepciones + query_structured) en el MISMO cambio.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** `query()` retorna `str` (no tocar); `VantaError(RuntimeError)` para
  backward-compat de `except RuntimeError`; jerarquía de herencia SIMPLE (no multiple); ids `u128` en
  dicts como `str`; stubs `.pyi` siempre en sync con el módulo compilado (anti-drift MOD-18).
- **Comandos de verificación:** `python -m pytest vantadb-python/tests/ -q` (verde);
  `cargo check -p vantadb_py` (ok); `cargo fmt --check`; `cargo clippy -p vantadb_py --all-targets -- -D warnings`
- **Deuda pendiente:** ninguna (o migración de builtins documentada = deuda asumida y documentada, Regla 6)

## Deuda técnica (Regla 6 — MUST)
**Saldo neto:** Cambio de builtins (KeyError/ValueError) → VantaError subclasses = deuda de migración
de la API de errores. Moneda de pago: elimina la deuda P2-6 (match no exhaustivo VantaError en
types.rs — ver Notas). Documentado en `docs/api/PYTHON_SDK.md` (sección Migration).

## Definition of Done
- **Task:** contrato verificable + fmt/clippy/pytest verdes + tests del cambio (typed errors + query_structured)
- **Commit:** NO aplica — sub-agente NO commitea (el lead verifica y commitea por tarea)
- **Release:** no aplica a esta tarea individual

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — FFI PyO3 (trust boundary: input de usuario en `query_structured(iql_query)` y
      mapeo de errores). Cargada `security-and-hardening`. Hallazgo: `query_structured` pasa el
      string IQL al core `engine.query` (misma ruta que `query()` ya existente — sin cambio de
      superficie de input; el parser IQL del core valida). No se agregan dependencias → no `cargo audit`.
      Los mensajes de error no exponen detalles internos (usan `Display` del core, igual que hoy).
- [ ] **PERFORMANCE** — no aplica: no toca hot paths (solo traducción de errores y un conversor a
      dict fuera de la búsqueda). No requiere benchmark.

## Steps

### Step 1: Jerarquía de excepciones en convert.rs + registro en pymodule
- **Archivos:** `vantadb-python/src/convert.rs`, `vantadb-python/src/lib.rs`
- **Acción:** definir `create_exception!` (VantaError base RuntimeError + 10 subclases) y registrarlas
  en `#[pymodule]` (`m.add(...)`).
- **Verify:** `cargo check -p vantadb_py`
- **Estado:** ✅

### Step 2: Reescribir map_vanta_error a subclases VantaError
- **Archivos:** `vantadb-python/src/convert.rs`
- **Acción:** mapear cada variant core a su subclase (alias `CoreError` para evitar colisión de
  nombre con la excepción `VantaError`); fallback → `VantaError` base.
- **Verify:** `cargo check -p vantadb_py`
- **Estado:** ✅

### Step 3: Exponer excepciones en __init__.py + stubs
- **Archivos:** `vantadb-python/vantadb_py/__init__.py`, `__init__.pyi`, `vantadb_py.pyi`
- **Acción:** importar excepciones en `__init__.py` (+ `__all__`); declararlas en ambos `.pyi`.
- **Verify:** `python -m pytest vantadb-python/tests/test_stub_drift.py -q`
- **Estado:** ✅

### Step 4: query_structured() (conversor + método + AsyncVantaDB + SystemClient + stubs)
- **Archivos:** `vantadb-python/src/convert.rs` (query_result_to_pydict), `vantadb-python/src/lib.rs`
  (método + forward_to_db SystemClient), `__init__.py` (AsyncVantaDB), `.pyi` ×2
- **Acción:** agregar `query_structured()` que retorna dict; wrapper async; subclient system; stubs.
- **Verify:** `cargo check -p vantadb_py` + `python -m pytest vantadb-python/tests/test_stub_drift.py -q`
- **Estado:** ✅

### Step 5: Tests (typed errors + query_structured) + migrar asserts existentes
- **Archivos:** `vantadb-python/tests/test_typed_errors.py` (nuevo), `vantadb-python/tests/test_sdk.py`
- **Acción:** tests RED→GREEN para jerarquía de errores y query_structured; actualizar asserts
  KeyError/ValueError (test_sdk.py:1468,1501-1505) → NotFoundError/ValidationError.
- **Verify:** `python -m pytest vantadb-python/tests/test_typed_errors.py vantadb-python/tests/test_sdk.py -q`
- **Estado:** ✅

### Step 6: Documentar en docs/api/PYTHON_SDK.md
- **Archivos:** `docs/api/PYTHON_SDK.md`
- **Acción:** sección de excepciones (jerarquía + tabla de mapeo + migración) y `query_structured`.
- **Verify:** leer doc; consistencia con implementación
- **Estado:** ✅

### Step 7: Verify full + review
- **Archivos:** — (verificación)
- **Acción:** `cargo check -p vantadb_py`, `cargo fmt --check`, `cargo clippy -p vantadb_py --all-targets -- -D warnings`, `python -m pytest vantadb-python/tests/ -q`; actualizar task file; recitation.
- **Verify:** todos los comandos verdes
- **Estado:** ✅

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review (sesión ses_fc7cdcf4dffeNulPSO3gLxycRR)
- **Enfoque:** ✅ approve — single-inheritance + VantaError(RuntimeError) correcto; mapping exhaustivo (26+ variants, wildcard `_` correcto para `#[non_exhaustive]`); query_structured aditivo, u128-as-str seguro, sin superficie FFI nueva.
- **Cómo se probó:** cargo check -p vantadb_py + pytest 132 passed + probes runtime de jerarquía (isinstance checks).
- **Recomendaciones no bloqueantes 🟡 aplicadas:** docstrings `Raises` OSError/PermissionError/FileNotFoundError → StorageError (drift corregido en lib.rs).
- **Recomendación 🟡 aceptada como deuda:** `TimeoutError` ensombrece el builtin en `from vantadb_py import *`; documentado en stub + PYTHON_SDK.md (Migration table).
- **Veredicto:** ✅ approve

## Notas
- P2-6 (match no exhaustivo VantaError en `vantadb-python/src/types.rs:365`): la tabla de mapeo de
  este task cubre todos los variants del core (incluye el fallback `_`), por lo que el mapping es
  exhaustivo por construcción. Documentado.
- El `#[non_exhaustive]` del enum core exige mantener el wildcard `_` → fallback a `VantaError` base.
- No rompe `except Exception` ni `except RuntimeError` (VantaError ⊂ RuntimeError ⊂ Exception).
- **Deuda aceptada (Regla 6):** migración de builtins (KeyError/ValueError/OSError/TimeoutError-builtin)
  → VantaError subclasses es un cambio de API documentado en PYTHON_SDK.md. `TimeoutError` (vantadb)
  ensombrece el builtin en `import *` — aceptado, documentado.
- **Estado server:** `campaign_update_task_state` bloqueado por FIND-06 (sesión concurrente,
  one-task-at-a-time). Implementación + verificación completas; el lead reconcilia estado y commitea.

