# MOD-21 — Nits agrupados Python (python.md de P32)

- **Plan:** `docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md` (Task 7)
- **Estado:** ⬜ PENDING
- **Appetite:** max 2h · **Esfuerzo:** 🟢 · **Prioridad:** 🟢
- **Archivos clave:** `vantadb-python/src/convert.rs:36-147`, `vantadb-python/src/lib.rs` (graph_bfs), `dist/`
- **Contrato:** pytest pasa; artefactos stale gitignored/removidos; graph_bfs direction expuesto; docs PYTHON_SDK
- **Verificación real:** 🟡 VERIFICAR (backlog P32 python.md)

## Discovery (2026-08-25)

### Verificación nit por nit

1. **Artefactos stale commiteados (wheels/.pyd/.pdb en dist/)** → ✅ YA RESUELTO.
   - `git ls-files dist/` = vacío; `git ls-files "*.pyd" "*.pdb" "*.whl"` = vacío.
   - `dist/vantadb_py-0.1.5-cp311-abi3-win_amd64.whl` (2MB) en disco, ignorado por `.gitignore:50 **/dist/` (confirmado `git check-ignore -v`).
   - `.gitignore` raíz cubre `*.pdb`; `vantadb-python/vantadb_py/.gitignore` cubre `*.pyd`, `*.so`, `__pycache__/`.
   - NO borrar .gitignore ni el whl (el build los genera; Regla 0 leída).

2. **async graph_bfs pierde direction** → ❌ REAL. Fix necesario.
   - Binding sync `VantaDB::graph_bfs` (lib.rs:1925-1938) ya expone `direction="Forward"` + `parse_direction`.
   - `AsyncVantaDB.graph_bfs` (vantadb_py/__init__.py:368-371) solo pasa `(roots, max_depth)` — pierde `direction`. Igual `graph_dfs` (:373-376).
   - Stub `__init__.pyi:210-215` tampoco lo declara.
   - Fix aditivo: agregar `direction="Forward"` al wrapper async + stub.

3. **Validación inconsistente** → ✅ YA RESUELTO/CONSISTENTE (documentar).
   - El binding NO duplica validación de namespace/key: todos los métodos directos (`put`, `get_memory`, `delete_memory`, `search_memory`, `list_memory`) delegan al core SDK; errores se propagan como `ValidationError` Python vía `map_vanta_error` (MOD-20).
   - Única validación binding-local: builder de dict `search_batch_requests` (lib.rs:2232-2236) exige campo `namespace` — necesaria porque el dict no tiene firma typed; consistente con wire format CLI/MCP.
   - No hay divergencia entre métodos → sin cambio de código.

4. **MAX_K clamp silencioso** → ❌ REAL. Fix necesario.
   - 6 call sites con `top_k.min(MAX_K)` silencioso (lib.rs:1107, 1273, 1612, 1651, 2135, 2254-2255). Sin warning ni docstring que documente el cap.
   - Fix: helper `clamp_top_k(requested)` con `tracing::warn!` (tracing ya es dep del binding, usado en :2120) + reemplazar los 6 sites + documentar en docstrings de search.

5. **connect sin read_only/backend** → ❌ REAL. Fix necesario.
   - `connect` (lib.rs:2329-2338) firma `(path, memory_limit=None)` y hardcodea `open_vantadb(py, path, memory_limit, false, None)`.
   - `VantaDB::new` ya acepta `read_only`/`backend` (lib.rs:378-393) — connect queda desalineado.
   - Stub `vantadb_py.pyi:484` idem.
   - Fix aditivo: agregar `read_only=false, backend=None` a `connect` + stub + docstring.

### Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `vantadb-python/vantadb_py/__init__.py` (442L) — wrapper AsyncVantaDB completo.
- `vantadb-python/src/convert.rs` (865L) — `parse_direction`, `map_vanta_error`, `py_dict_to_filter_ops`; `convert.rs:36-147` = jerarquía excepciones MOD-20 + py_any_to_value (NO tocar).
- `vantadb-python/src/lib.rs` (2365L, secciones: 40-159 OpGate/MAX_K, 376-393 new, 1095-1118 similar_to_key, 1258-1287 search_memory, 1600-1661 search/search_batch, 1925-1960 graph_bfs/dfs sync, 2120-2149 explain, 2230-2289 request builder, 2325-2338 connect).
- `vantadb-python/vantadb_py/vantadb_py.pyi` (486L) — stub sync: graph_bfs/dfs ya con direction (:269-274, :435-440), connect (:484) sin read_only/backend.
- `vantadb-python/vantadb_py/__init__.pyi` (216+ L) — stub async: graph_bfs/dfs sin direction (:210-215).
- `.gitignore` raíz + `vantadb-python/vantadb_py/.gitignore`.

**Referencias hacia dentro (de los archivos a editar):**
- `__init__.py` → importa `connect`, `VantaDB` desde `.vantadb_py` (:12-31); re-exporta en `__all__` (:33-54).
- `__init__.pyi` → declara la clase `AsyncVantaDB` (mismo shape que `__init__.py`).
- `vantadb_py.pyi` → declara `VantaDB` + `connect` + sub-clientes SDKB-03.
- `lib.rs connect` → registrado vía `wrap_pyfunction!(connect, m)` (:2350).

**Referencias entrantes:**
- `connect` usado en docs (QUICKSTART/PYTHON_SDK) y posiblemente ejemplos; firma aditiva (kwargs con default) NO rompe callers existentes `connect(path)` / `connect(path, memory_limit)`.
- `AsyncVantaDB.graph_bfs(roots, max_depth)` — llamadas existentes siguen válidas (direction default).
- Cambio `top_k.min(MAX_K)` → `clamp_top_k(top_k)` sin cambio de comportamiento salvo warning observable.

**Veredicto:** cambios aditivos/seguros; sin tocar `convert.rs:36-147` (solo lectura para contexto), sin tocar core (`src/sdk/`, `wal.rs`, `vector/`, `storage/`), sin borrar .gitignore ni dist/.

## Steps

- [x] S1: Fix `connect` (lib.rs:2329-2338 + docstring + stub vantadb_py.pyi:484)
- [x] S2: Fix async `graph_bfs`/`graph_dfs` direction (__init__.py + __init__.pyi)
- [x] S3: Fix MAX_K clamp silencioso (helper `clamp_top_k` + 6 sites + docstrings)
- [x] S4: Docs PYTHON_SDK (docs/api/PYTHON_SDK.md) — connect + graph_bfs/dfs
- [x] S5: Verify: pytest 132 passed + cargo check + fmt + clippy ✅

## Implementación (2026-08-25)

- `vantadb-python/src/lib.rs`: helper `clamp_top_k(requested)` (warning `tracing::warn!` cuando top_k > MAX_K, ERR-022) reemplaza los 6 `top_k.min(MAX_K)` silenciosos (similar_to_key, search_memory, search, search_batch, explain_memory_search, request-builder search_batch_requests). `connect` ahora firma `(path, memory_limit=None, read_only=false, backend=None)` y pasa ambos a `open_vantadb` (antes hardcodeaba `false, None`).
- `vantadb-python/vantadb_py/__init__.py`: `AsyncVantaDB.graph_bfs`/`graph_dfs` ahora pasan `direction="Forward"` (antes se perdía).
- Stubs: `vantadb_py.pyi` connect + `__init__.pyi` graph_bfs/dfs con direction.
- `tests/test_stub_drift.py`: firma esperada de connect actualizada (4 params) — anti-drift de MOD-18.
- `docs/api/PYTHON_SDK.md`: connect + graph_bfs/dfs documentados con los parámetros nuevos.

## Nits ya-resueltos (documentados, sin código)

- **Nit 1 (artefactos stale):** `git ls-files dist/` y `*.pyd|*.pdb|*.whl` vacíos; whl en disco ignorado por `.gitignore:50 **/dist/`; `.pyd`/`.pdb` cubiertos por .gitignore raíz (`*.pdb`) + `vantadb_py/.gitignore` (`*.pyd`, `*.so`). Nada que remover.
- **Nit 3 (validación inconsistente):** el binding no duplica validación namespace/key — todos los métodos directos delegan al core SDK y propagan `ValidationError` Python vía `map_vanta_error` (MOD-20). La única validación binding-local (request-builder exige `namespace`) es necesaria por la naturaleza dict del input, consistente con CLI/MCP.

## Verification

- `python -m pytest tests/` (venv, binding rebuild con maturin develop --release): **132 passed, 4 deselected** ✅ (131 + 1 tras fix anti-drift)
- `cargo check -p vantadb_py`: Finished ✅
- `cargo fmt --check -p vantadb_py`: exit 0 ✅
- `cargo clippy -p vantadb_py --all-targets -- -D warnings`: Finished sin warnings ✅
- `git diff --stat`: 6 archivos, +58/-25 (solo alcance MOD-21)

## Context Save Point

No commit (regla: lead verifica y commitea). El MCP state machine bloquea `update_task_state` para MOD-21 mientras MCP-34a quede in-progress en el server (plan la tiene COMMITTED) — el lead debe cerrar MCP-34a o forzar el estado de MOD-21.