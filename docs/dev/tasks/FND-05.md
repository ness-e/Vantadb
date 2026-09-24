# FND-05 — SDK idiomático (no wrapper 1:1 de Rust)

**Plan:** 2026-08-16-wave-p20-tsys.md · **Prio:** 🟡 · **Esfuerzo:** 🟡 · **Tipo:** research/analysis (multi: rust, python, typescript)

## Objetivo

Investigar si la API expuesta de `vantadb-python` y `vantadb-ts` es idiomática de cada
lenguaje (context managers, type hints modernos, async nativo, tipos discriminados) o
un reflejo 1:1 del core Rust. Entregable: lista de gaps (archivo:línea) + prototipo de
1 método idiomático por SDK en `docs/examples/`. NO rewrite.

## Archivos clave

- `vantadb-python/src/lib.rs`, `vantadb-python/src/types.rs` (PyO3)
- `vantadb-python/vantadb_py/vantadb_py.pyi`, `vantadb-python/vantadb_py/__init__.pyi`, `vantadb-python/vantadb_py/__init__.py`
- `vantadb-ts/src/vantadb.ts`, `vantadb-ts/src/native.ts`, `vantadb-ts/src/types.ts`, `vantadb-ts/src/errors.ts`
- Entregables: `docs/Investigaciones/FND-05-sdk-idiomatico.md` + prototipos en `docs/examples/`

## Impacto mapeado (Regla 0)

**Archivos leídos completos (DISCOVERY):**
- `vantadb-python/src/lib.rs` (parcial 760-1000, 1860-1964 + codegraph 55 símbolos), `types.rs` (grep pyclass), `vector.rs` (grep)
- `vantadb-python/vantadb_py/vantadb_py.pyi` (183L), `__init__.pyi` (294L), `__init__.py` (390L), `pyproject.toml` (53L), `README.md` (94L)
- `vantadb-ts/src/vantadb.ts` (938L, parcial + codegraph), `native.ts` (283L), `types.ts` (201L), `errors.ts` (57L), `guards.ts` (grep), `tsconfig.json` (15L), `package.json` (61L), `README.md` (158L parcial)

**Referencias hacia dentro:** ninguno de los archivos que CREO es referenciado por
código existente (docs + examples nuevos).

**Referencias salientes de lo que toco:** solo archivos nuevos — sin imports ni
dependencias de código.

**Veredicto:** NO se modifica el SDK real (`vantadb-python/src`, `vantadb-ts/src`)
ni ningún archivo existente. Solo se crean 4 archivos nuevos (task file, análisis,
2 prototipos). Impacto nulo sobre runtime/build/tests. Contrato de la tarea:
análisis con gaps citados + ≥1 prototipo idiomático en `docs/examples/`.

## Steps

### STEP 1 — DISCOVERY (inventario API pública Python + TS) — ⬜
- [x] codegraph_explore bindings (55 símbolos: VantaDB, open_vantadb, search_memory, etc.)
- [x] Leer stubs `.pyi` (183L + 294L), `__init__.py` (390L), types.ts, native.ts, errors.ts
- [x] Verificar presencia de CM/async en Rust (grep `__enter__|__aenter__|asyncio|async` → 0 matches en src/)
- [x] Leer READMEs (Python 94L, TS 158L) — pulidos por FND-18
- [x] Verificar retorno real: `get_memory` → `VantaPyMemoryRecord | None` (lib.rs:793), `list_memory` → `VantaListResult` (lib.rs:880), `put` → `VantaPyMemoryRecord`
- [x] Verificar exports: `__init__.py` NO exporta `connect` (solo VantaDB, VantaListResult, VantaMemoryRecord, VantaSearchHit, VantaVector, __version__); `__init__.pyi:293` declara `connect` → drift
- [x] Verificar `__version__`: lib.rs:1962 `m.add("__version__", ...)` = atributo string; stubs lo declaran `def __version__() -> str` → drift
- [x] TS: `VantaDB` sync WASM + `NativeVantaDB` async (native.ts), ambos con `close()`; sin `Symbol.dispose`/`asyncDispose`; tsconfig ES2022 sin `esnext.disposable`
- [x] Evidencia idiomática: sqlite3 CM (commit/rollback, NO close) + TS 5.2 `await using`/`AsyncDisposable` (devblogs.microsoft.com)

**Resultado:** inventario completo. Hallazgos clave: Python `VantaDB` sin `__enter__/__exit__` (con `close()`); `AsyncVantaDB` ya idiomático (`async with`); 4 drifts de stub (get_memory/list_memory/put/connect/__version__); TS `VantaMetadata` tagged-union refleja enum Rust (fricción), sin `await using`.

### STEP 2 — ANÁLISIS IDIOMÁTICO + GAPS — ⬜
- [x] Comparar contra sqlite3/chromadb/duckdb (Python) y libsql/duckdb-node (TS)
- [x] Enumerar gaps con archivo:línea y severidad (rompe-uso / fricción / cosmético)
- [x] Escribir `docs/Investigaciones/FND-05-sdk-idiomatico.md`

### STEP 3 — PROTOTIPOS — ✅
- [x] `docs/examples/fnd05_python_context_manager.py` — `with VantaDB(path) as db:` (wrapper que agrega `__enter__/__exit__` → close)
- [x] `docs/examples/fnd05_ts_async_dispose.ts` — `await using db = await connectDisposable()` (wrapper `AsyncDisposable`)

### STEP 4 — VERIFY + CIERRE — ✅
- [x] Verificar contrato mecánico: análisis existe + gaps citados + ≥1 prototipo
- [x] `python -m py_compile` del prototipo Python → `PY_SYNTAX_OK`
- [x] `npx tsc --noEmit` NO aplica (prototipo en docs/examples/ no compilado por tsconfig include src/**) — sintaxis TS verificada manualmente (archivo de referencia, no se compila con el SDK)
- [x] Devolver bloque RESULTADO (✅ COMPLETO, 4/4 steps)

## Contract (verify mecánico)

- [ ] `docs/Investigaciones/FND-05-sdk-idiomatico.md` existe con gaps citados archivo:línea
- [ ] ≥1 prototipo idiomático en `docs/examples/` (python o ts)
- [ ] SDK real NO modificado (git status: solo archivos nuevos)
- [ ] NO git add/commit (lead commitea)
- [ ] Task file FND-05.md creado

## Fuentes / Evidencia

- sqlite3 Connection as context manager (commit/rollback, no close): https://docs.python.org/3/library/sqlite3.html
- TS 5.2 `using`/`await using`, `Symbol.asyncDispose`, `AsyncDisposable`, lib `esnext.disposable`: https://devblogs.microsoft.com/typescript/announcing-typescript-5-2/
- chromadb Python API (metadata as plain dict): https://docs.trychroma.com/
- libsql client (async, close()): https://docs.turso.tech/sdk/ts

## Notas

- P2-5 (`put_batch` dual API) ya documentado en AGENTS.md Regla 6 — se referencia, no se re-analiza a fondo.
- PERF-08 zero-copy `Float32Array` ya reflejado en `MemoryRecord.vector?: Float32Array | number[]` — union correcta, no es gap.
- VantaMemoryRecord/VantaSearchHit/VantaListResult como pyclass con getters tipados = idiomático Python 👍 (no gap).
- VantaError con `code` tipado en TS = idiomático 👍 (chromadb/duckdb no lo tienen).