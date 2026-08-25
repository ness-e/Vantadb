# MOD-19 — Exponer API core faltante en binding Python (PyO3)

> **Campaign:** 4b9e337a-2fd0-4625-9cba-e26ea37f780b · **Plan:** docs/plans/2026-08-24-batch-review-mod-find.md
> **Estado:** ✅ COMPLETO (2026-08-24; sync 2026-08-25 — commiteado en `dc65c242` feat(python): MOD-19) · **Tipo:** python (Python SDK) · **Contrato:** pytest pasa; `similar_to_key`/`count`/`delete_by_filter` expuestos

## Objetivo
Exponer en `vantadb-python` (PyO3) las funciones core que faltan: `count`, `delete_by_filter`, `similar_to_key` (y cualquier equivalente core faltante), con tipos/nombres consistentes con la convención del SDK Python y del resto del ecosistema (CLI/MCP/TS).

## Impacto mapeado (Regla 0)

### Archivos leídos (completos)
- `vantadb-python/src/lib.rs` — patrón PyO3 (flat methods, `py.detach`, `enter(&self.op_gate)`, `forward_to_db!`), métodos memory existentes (`get_memory`, `delete_memory`, `list_memory`, `search_memory`, `explain_memory_search`), sub-client `MemoryClient` (línea 227), `parse_search_request`, module registration.
- `vantadb-python/src/convert.rs` — `py_dict_to_metadata`, `py_any_to_value` (línea 36) → reutilizo para construir `VantaValue` de filtros.
- `docs/api/PYTHON_SDK.md` — convención snake_case, sección Roadmap lista `delete_by_filter`/`similar_to_key`/`count` como no implementadas.
- `docs/api/BINDINGS_NAMESPACES.md` — tabla Python: memory 15; lista `delete_by_filter` en "Not exposed in Python".
- `vantadb-python/vantadb_py/__init__.pyi`, `vantadb_py/vantadb_py.pyi` — stubs type de VantaDB/MemoryClient/AsyncVantaDB.
- `vantadb-python/vantadb_py/__init__.py` — `AsyncVantaDB` (wrapper `asyncio.to_thread`).
- `.opencode/rules/python-bindings.md`, `.opencode/rules/api-contract.md` — reglas normativas.

### Funciones core verificadas (src/)
- `src/sdk/api.rs:1595` `similar_to_key(&self, namespace, key, top_k) -> Result<Vec<VantaMemorySearchHit>>`
- `src/sdk/api.rs:1487` `count(&self, namespace, filter: Option<VantaMemoryFilter>) -> Result<u64>`
- `src/sdk/api.rs:1418` `delete_by_filter(&self, namespace, filter: VantaMemoryFilter) -> Result<u64>`
- Formato canónico de `VantaMemoryFilter` (filter_ops) en CLI (`parse_filter_json` crud.rs:394) / MCP (`parse_filter_ops` validation.rs:262) / TS (`VantaMemoryFilterItem`): flat → `$eq`, o `{"field": {"$op": value}}`.

### Referencias entrantes (dependen de lo que cambio)
- `vantadb-python/vantadb_py/*.pyi` + `vantadb_py/__init__.py` — deben reflejar los métodos nuevos (typing + Async).
- `docs/api/PYTHON_SDK.md`, `docs/api/BINDINGS_NAMESPACES.md` — sincronizar en el mismo PR (Regla 3).
- `tests/test_sdk.py`, `tests/test_subclients.py` — extender para cubrir los 3 métodos + sub-client `memory`.

### Referencias salientes (lo que cambio referencia)
- `vantadb::sdk::{VantaMemoryFilter, VantaMemoryFilterItem, VantaFilterOp, VantaMemorySearchHit}` (core, ya usado).
- `py_any_to_value`/`VantaValue` (convert.rs).

### Veredicto de impacto
Aditivo, sin romper firmas existentes (Regla D45 → minor). Nuevos métodos flat + forwarding sub-client + Async + stubs + docs. No toco `src/` core (prohibido: api-contract R-8 — lógica vive en core, binding es glue). No toco `wal.rs`/`vector/`/`storage/`.

## Spec (contracto de diseño)

### Firma flat (snake_case, matching core + ecosistema)
```rust
fn count(&self, py, namespace, filters: Option<&Bound<PyDict>>) -> PyResult<u64>
fn delete_by_filter(&self, py, namespace, filters: &Bound<PyDict>) -> PyResult<u64>
fn similar_to_key(&self, py, namespace, key, top_k: usize) -> PyResult<Vec<VantaPySearchHit>>
```
- `filters` (count/delete_by_filter) usa el formato canónico filter_ops: `{"field": value}` → `$eq`, o `{"field": {"$eq": v, "$gte": v2, ...}}`. Helper nuevo `py_dict_to_filter_ops` en convert.rs.
- `delete_by_filter` requiere filtro no vacío (el core rechaza vacío con `InvalidInput` — propagar, no inventar).
- `similar_to_key` mapea `VantaMemorySearchHit { record, score }` → `VantaPySearchHit { inner: record, score }` (mismo mapping que `search_memory`).

### Sub-client `memory`
Añadir `count`, `delete_by_filter`, `similar_to_key` a `forward_to_db!(MemoryClient { ... })`.

### Async + stubs + docs
- `AsyncVantaDB`: `count`, `delete_by_filter`, `similar_to_key`.
- `.pyi` (ambos) + `PYTHON_SDK.md` (mover 3 de Roadmap a API Reference) + `BINDINGS_NAMESPACES.md` (memory 15→18, quitar de "Not exposed in Python").

## Steps
1. ✅ `convert.rs`: helper `py_dict_to_filter_ops` (+ imports `VantaFilterOp`/`VantaMemoryFilterItem`) — formato canónico flat→`$eq` / `{"$op": value}` (CLI/MCP/TS).
2. ✅ `lib.rs`: 3 flat methods (`delete_by_filter`, `count`, `similar_to_key`) + `forward_to_db!(MemoryClient)` — patrón `enter(&op_gate)` + `py.detach` + `top_k.min(MAX_K)`.
3. ✅ Stubs `.pyi` (vantadb_py.pyi + __init__.pyi: VantaDB/MemoryClient/AsyncVantaDB) + `AsyncVantaDB` (__init__.py).
4. ✅ Tests en `tests/test_sdk.py` (4 tests) + `test_subclients.py` (3 identity tests).
5. ✅ Docs: `PYTHON_SDK.md` (3 métodos a API Reference, Roadmap vacío, Async list) + `BINDINGS_NAMESPACES.md` (memory 15→18, totals 44→47, removed de "Not exposed").
6. ✅ Verify: `cargo check -p vantadb_py` ✅ · `cargo fmt --check` ✅ · `cargo clippy -p vantadb_py --all-targets` ✅ (exit 0) · `python -m pytest tests/` ✅ 118 passed / 4 deselected / 0 failed · docs-coverage `pwsh scripts/validate-docs-coverage.ps1` ✅ 0 gaps.

## Contrato / DoD
- `python -m pytest vantadb-python/tests/ -v` ✅ (tests nuevos para los 3 métodos).
- `count`/`delete_by_filter`/`similar_to_key` expuestos en flat + `db.memory.*` + `AsyncVantaDB`.
- `cargo fmt --check` + `cargo clippy` (vantadb-python) limpios.
- Docs actualizadas en el mismo PR.

## Context Save Point
- Binding usa `py.detach` (GIL release) + `enter(&self.op_gate)` (durability gate). Patrón a replicar.
- `VantaPySearchHit { inner, score }` se construye igual que en `search_memory`.
- Formato canónico filtro operator-dict = CLI/MCP/TS (consistencia cross-SDK).
- **Estado: ✅ COMPLETO (2026-08-24).** No commit — el lead verifica mecánico y commitea.
  - Comando de re-verificación exacto: `cd vantadb-python && python -m maturin develop && python -m pytest tests/ -v` (118 passed) + `cargo fmt --check` + `cargo clippy -p vantadb_py --all-targets` + `pwsh scripts/validate-docs-coverage.ps1` (0 gaps).
  - Nota: `scripts/validate-docs-coverage.ps1` NO parsea con Windows PowerShell 5.1 — usar `pwsh` (PS7).
