# SDKB-03: Sub-clientes Python (espejo de SDKB-02)

## Metadata
- **Plan file:** docs/plans/2026-08-22-vantadb-bindings-sdk.md (Task 3)
- **Creado:** 2026-08-22T12:00
- **last-synced:** 2026-08-22T18:30
- **Estado:** ✅ COMPLETED

## Impacto mapeado (Regla 0)
- **Leídos completos:** `docs/api/BINDINGS_NAMESPACES.md` (mapa canon, §Python 44 métodos + §Sub-Client Design), `vantadb-python/src/lib.rs` L1-400 + firmas de todos los métodos via grep (44 pymethods, OpGate, open_vantadb), `vantadb-python/pyproject.toml` (pytest addopts `-m 'not slow'`), `vantadb_py/__init__.pyi` (stubs), `vantadb/__init__.py` (alias re-export), `tests/test_sdk.py` head (fixtures/convenciones), `.opencode/rules/python-bindings.md` (R-1/R-2: batch GIL-release — no aplica, cero lógica batch nueva).
- **Referencias hacia dentro:** `vantadb_py/__init__.py` re-exporta la clase nativa → los getters suben gratis; `vantadb/__init__.py` alias idem; `.pyi` es el contrato de tipos.
- **Referencias entrantes:** tests (`test_sdk.py`, `test_async_smoke.py`, `test_load.py`, `test_migration.py`, `test_perf_15_16.py`) — suite existente NO se toca; `AsyncVantaDB` (Python puro) envuelve VantaDB — fuera de scope D42.
- **Veredicto:** cambio ADITIVO en lib.rs (4 pyclasses delegantes + 4 getters). Cero cambios en métodos planos → backward-compat garantizado por construcción. PyO3 0.29 confirmado en Cargo.toml; patrón `#[pyo3(signature = (*args, **kwargs))]` validado contra docs oficiales pyo3.rs v0.29.

## Blast Radius
- Callers: Python users de `db.memory.*` — ninguno hoy (API nueva).
- Callees: métodos planos de `VantaDB` vía `call_method` (delegación pura, D43: cero lógica nueva).
- Implicaciones: ciclo de refs delegate→db solo mientras el usuario retenga el delegante (documentado); getters construyen instancia fresca por acceso.

## Contrato
"`pytest` pasa (suite existente intacta = backward-compat); tests nuevos espejo SDKB-02: `db.memory.*`, `db.graph.*`, `db.system.*`, `db.wiki.*` delegan al método plano con resultado/firma idénticos"

## Herramientas
- cargo check/clippy -p vantadb_py, maturin develop --release (venv target/audit-venv), pytest

## Steps
### Step 1: Delegantes Rust + getters en lib.rs
- **Archivos:** `vantadb-python/src/lib.rs`
- **Acción:** 4 `#[pyclass]` (MemoryClient/GraphClient/SystemClient/WikiClient) con `db: Py<VantaDB>`, forwarding varargs vía macro; `#[getter] memory/graph/system/wiki` sobre VantaDB.
- **Verify:** `cargo check -p vantadb_py` ✅
- **Estado:** ✅ COMPLETED (sesión previa; verificado en cierre)

### Step 2: Stubs .pyi + tests espejo
- **Archivos:** `vantadb_py/__init__.pyi`, `tests/test_subclients.py` (nuevo)
- **Acción:** stubs de las 4 clases + properties; tests de identidad resultado/firma vs método plano (espejo SDKB-02).
- **Verify:** build maturin + `python -m pytest` completo exit 0
- **Estado:** ✅ COMPLETED (sesión previa; verificado en cierre)

### Step 3: Verify mecánico full + cierre
- **Acción:** fmt --check del crate, clippy sin warnings nuevos, suite pytest completa, cierre MCP (sin commit — orden explícito del orquestador).
- **Estado:** ✅ COMPLETED — `cargo fmt -p vantadb_py` aplicado a los bloques `forward_to_db!` (solo whitespace, re-check OK), `cargo check -p vantadb_py` ✅, `cargo clippy -p vantadb_py --all-targets` sin warnings propios, suite completa: `target\audit-venv\Scripts\python.exe -m pytest vantadb-python\tests -v` → **105 passed, 4 skipped, 4 deselected, exit 0**. Sin commit (orden del orquestador).

## Dependencias
- Task 2: SDKB-02 (✅ COMPLETED) — patrón TS espejado.

## Notas
- Mapa canon: `docs/api/BINDINGS_NAMESPACES.md` §Python. Naming hazard: `get/delete/insert` son node-level (graph) en Python.
- Decisión D42: solo capa Python. D43: agrupación v1 only.
- Ponytail: property-getter elegido (PyO3 lo soporta sin fricción); helper functions solo si falla (stop condition).

## Context Save Point
- **Fecha:** 2026-08-22T18:30
- **Branch:** develop
- **CI pendiente:** no (sin commit)
- **Decisiones:** delegantes pyclass + forwarding varargs macro (vs firmas tipadas duplicadas ~500L) porque el contrato exige firma idéntica y varargs la garantiza por construcción con ~5L/método.
- **Problemas conocidos:** ninguno. 4 tests skipped = test_migration.py (chromadb/lancedb no instalados en venv — pre-existente). 4 deselected = marker `slow` (FX-3).
- **Próxima tarea:** Task 4 del plan

## Recitation (§12.3 canónica, cierre 2026-08-22T18:30)
- **activeGoal:** SDKB-03 — sub-clientes Python (db.memory/graph/system/wiki) con delegación pura a métodos planos.
- **lastAction:** verify mecánico full: fmt aplicado+check OK, cargo check ✅, clippy limpio, suite pytest completa verde; task file + estado MCP cerrados.
- **result:** OK
- **nextAction:** ninguna en esta tarea — orquestador lanza Task 4 del plan `docs/plans/2026-08-22-vantadb-bindings-sdk.md`.
- **contract:** verificacion=`target\audit-venv\Scripts\python.exe -m pytest vantadb-python\tests -v` → 105 passed/4 skipped/4 deselected exit 0 ✅; evidencia: claim=delegación db.memory.*/db.graph.*/db.system.*/db.wiki.* idéntica al método plano → evidencia=`vantadb-python/tests/test_subclients.py` (16 tests PASSED) → confianza alta; claim=backward-compat suite intacta → evidencia=test_sdk.py/test_async_smoke/test_load/test_perf_15_16 PASSED sin modificaciones → confianza alta; artefactos=`vantadb-python/src/lib.rs`, `vantadb_py/__init__.pyi`, `vantadb-python/tests/test_subclients.py`; invariantes=métodos planos NO tocar, cero lógica nueva en delegantes (D43), solo capa Python (D42); deuda=ninguna; queda_pendiente=commit lo hace el lead (orden explícito).
- **nextTask:** Task 4
