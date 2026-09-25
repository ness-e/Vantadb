# FND-04 — Zero-copy Arrow en bindings (Python/Node)

**Plan:** 2026-08-16-wave-p20-tsys.md · **Prio:** 🟡 · **Esfuerzo:** 🔴 · **Tipo:** research/analysis (multi: rust, python, typescript)

## Objetivo

Analizar viabilidad de zero-copy Arrow en bindings Python (`vantadb-python`) y Node
(`vantadb-node`): ¿el RecordBatch del core (`src/columnar.rs`) puede llegar sin copia a
Python (pyarrow) y a Node (apache-arrow / TypedArray)? Entregable: **plan firmado** —
mapa path actual (archivo:línea) + decisión explícita (implementar: plan detallado |
diferir: ADR con razón y señal de re-apertura medible). NO implementación.

## Archivos clave

- `src/columnar.rs` (core Arrow RecordBatch), `src/sdk/api.rs`, `src/sdk/search/mod.rs`,
  `src/sdk/serialization/` (postcard/JSONL — NO Arrow IPC)
- `vantadb-python/src/convert.rs`, `types.rs`, `vector.rs` (PyO3, pyo3 0.29)
- `vantadb-wasm/src/lib.rs` (PERF-08 precedente Float32Array), `vantadb-ts/src/vantadb.ts`
- `vantadb-node/src/lib.rs` (napi-rs)
- Entregables: `docs/Investigaciones/FND-04-arrow-zero-copy.md` + task file

## Impacto mapeado (Regla 0)

**Archivos leídos completos (DISCOVERY):**
- `src/columnar.rs:22` `nodes_to_record_batch()` — RecordBatch id UInt64 + `vector_d0..N`
  Float32Array por dim. Único uso: tests (`tests/logic/columnar.rs:8,23`). **Desconectado**
  de API pública — ningún binding lo recibe hoy.
- `src/sdk/serialization/` — postcard wire format + JSONL export/import + sparse vector.
  NO es Arrow IPC (grep `arrow::ipc` → 0 matches en repo).
- `src/sdk/api.rs:545` `list()` → `Vec<VantaMemoryRecord>` (copias owned, vector clonado);
  `src/sdk/search/mod.rs:69` `search()` → `Vec<VantaMemorySearchHit>`.
- Python input YA zero-copy: `extract_vector()` `convert.rs:177-221` (PyBuffer: NumPy,
  memoryview, bytes, array.array) + `FlatBufferView` `types.rs:18-40`. Output CON copia:
  getter `.vector` `types.rs:86-97` → `try_numpy_array()` `convert.rs:153-167` →
  `__array_interface__` `vector.rs:59-83` (PyBytes owned copy — fix SEC-01/AUDIT-01 anti-UAF).
- WASM: PERF-08 ya resuelto output zero-copy (`lib.rs:1428-1447` Float32Array + copy_from);
  input pendiente (`lib.rs:1462-1474` from_js serde_wasm_bindgen).
- TS (`vantadb-ts/src/vantadb.ts:1`): wrapper WASM → hereda Float32Array.
- Node (`vantadb-node/src/lib.rs:136-162`): peor path — `serde_json::to_value` JSON completo
  de VantaMemoryRecord incl. vector; input parse de Value → Vec<f32>.

**Referencias:** ninguna — tarea analysis-only, crea 2 archivos nuevos, no modifica código.
**Veredicto:** impacto nulo sobre runtime/build/tests. Contrato: plan firmado con decisión.

## Steps

### STEP 1 — DISCOVERY (mapa path actual) — ✅
- [x] codegraph_explore + lectura directa de columnar.rs, sdk serialization, api.rs, search/mod.rs
- [x] Mapear Python: convert.rs, types.rs, vector.rs (input PyBuffer zero-copy; output copy)
- [x] Mapear WASM/TS (PERF-08 precedente) y Node (napi-rs serde_json)
- [x] Validar opciones técnicas contra docs oficiales (Regla 0):
  - napi-rs TypedArray/Float32Array zero-copy: https://napi.rs/docs/concepts/typed-array
  - Arrow C Data Interface: `arrow::ffi` (feature `ffi`) `to_ffi`/`export_array_into_raw` —
    https://docs.rs/arrow/59.2.0/arrow/ffi/ (arrow 59.2.0 ya en Cargo.toml)
  - `arrow-c-data` crate NO existe en crates.io (404) — el mecanismo real es `arrow::ffi`

### STEP 2 — ANÁLISIS VIABILIDAD + DECISIÓN — ✅
- [x] Costo/beneficio por binding (Python: pyarrow C data interface vs estado actual;
  Node: serde_json vs Buffer/IPC vs precedente WASM)
- [x] Decisión: **DIFERIR con ADR** — ver documento para razones completas

### STEP 3 — ENTREGABLES — ✅
- [x] Escribir `docs/Investigaciones/FND-04-arrow-zero-copy.md`
- [x] Crear task file FND-04.md

### STEP 4 — VERIFY + CIERRE — ✅
- [x] Verificar contrato mecánico: documento existe + path archivo:línea + decisión explícita + señal de re-apertura
- [x] Devolver bloque RESULTADO

## Contract (verify mecánico)

- [x] `docs/Investigaciones/FND-04-arrow-zero-copy.md` existe con path actual (archivo:línea)
- [x] Decisión explícita: implementar (plan) o diferir (ADR + señal de re-apertura medible)
- [x] SDK/core NO modificado (git status: solo archivos nuevos)
- [x] NO git add/commit (lead commitea)
- [x] Task file FND-04.md creado

## Fuentes / Evidencia

- napi-rs TypedArray (zero-copy Rust↔Node): https://napi.rs/docs/concepts/typed-array
- arrow::ffi C Data Interface (arrow 59.2.0): https://docs.rs/arrow/59.2.0/arrow/ffi/
- pyarrow C Data Interface: https://arrow.apache.org/docs/format/CDataInterface.html
- PERF-08 task file (precedente WASM zero-copy): tasks/PERF-08.md

## Notas

- La crate `arrow-c-data` no existe (crates.io 404); el camino real Python es
  `arrow::ffi::to_ffi`/`export_array_into_raw` + `pyarrow.Array._import_from_c`.
- RecordBatch core solo cubre id + vectores flatten — NO el shape rico de
  VantaMemoryRecord (namespace/key/payload/metadata). Zero-copy Arrow full requiere
  API core nueva que emita RecordBatch desde search/list (fuera de scope bindings).
- Output Python con copia es decisión deliberada de seguridad (SEC-01/AUDIT-01 UAF);
  revertir a zero-copy reintroduce el riesgo.
- Señal de re-apertura: benchmark query grande (top_k=10_000, 1536-dim, ≥1M records,
  ≥40MB vectores) con overhead de serialización >30% del tiempo total, o necesidad
  reportada de interop analítica masiva pandas/polars.