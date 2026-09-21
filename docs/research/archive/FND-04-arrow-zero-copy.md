# FND-04 — Zero-copy Arrow en bindings (Python/Node): plan firmado

- **Plan file:** docs/plans/2026-08-16-wave-p20-tsys.md (Task FND-04)
- **Fecha:** 2026-08-16
- **Tipo:** analysis + plan firmado (NO implementación)
- **Autor:** vanta-worker
- **Estado:** ✅ COMPLETO — decisión: **DIFERIR (ADR)** con señal de re-apertura medible

## Resumen ejecutivo

El core de VantaDB **ya construye un RecordBatch Arrow** (`src/columnar.rs:22`
`nodes_to_record_batch()`) con schema id (UInt64) + `vector_d0..N` (Float32Array por
dimensión), pero **está desconectado**: solo lo usan tests (`tests/logic/columnar.rs:8,23`)
y ninguna API pública del SDK lo expone a bindings. No existe ningún uso de `arrow::ipc`
writer en el repo (grep: 0 matches). El wire-format real del SDK
(`src/sdk/serialization/`) es postcard + JSONL, NO Arrow.

Análisis por binding: Python input **ya es zero-copy** (PyBuffer, `convert.rs:177-221`);
Python output usa copia deliberada de seguridad (SEC-01/AUDIT-01); WASM/TS **ya resuelto**
por PERF-08 (Float32Array zero-copy, `lib.rs:1428-1447`); Node nativo usa
`serde_json::to_value` (peor path, `vantadb-node/src/lib.rs:136-162`) pero admite
TypedArray zero-copy nativo (napi-rs).

**Decisión: diferir.** El RecordBatch core no cubre el shape de VantaMemoryRecord
(metadata/payload/namespace/key), pyarrow no es dependencia deseada, el input Python ya
es zero-copy, y el hot path JS/WASM ya está resuelto. La mejora restante (output Node
vía TypedArray) es un cambio barato y acotado que se firma como plan futuro con umbral
medible de re-apertura.

---

## 1. Path actual mapeado (archivo:línea)

### 1.1 Core — Arrow ya disponible pero desconectado

| Ubicación | Qué hay |
|---|---|
| `src/columnar.rs:22` | `nodes_to_record_batch()` → `arrow::record_batch::RecordBatch` (id UInt64 + `vector_d0..N` Float32Array por dim) |
| `tests/logic/columnar.rs:8,23` | Únicos callers del RecordBatch (tests) |
| `Cargo.toml:44` | `arrow = { version = "59", features = ["ipc"], optional = true }` — feature `arrow` en default (`Cargo.toml:97`) |
| `src/sdk/serialization/` | postcard wire format + JSONL export/import + sparse vector — NO Arrow IPC |
| `src/sdk/api.rs:545-619` | `list()` → `VantaMemoryListPage` con `Vec<VantaMemoryRecord>` (copias owned, vector clonado) |
| `src/sdk/search/mod.rs:69` | `search()` → `Vec<VantaMemorySearchHit>` (copia) |

Conclusión: **el RecordBatch existe pero ninguna API pública lo entrega.** Para zero-copy
Arrow real se necesitaría una API core nueva que emita RecordBatch desde search/list —
eso es trabajo de core SDK, no de bindings.

### 1.2 Python (`vantadb-python/`, PyO3 0.29)

| Ubicación | Qué hay | Zero-copy? |
|---|---|---|
| `convert.rs:177-221` `extract_vector()` | Input vía `PyBuffer` (NumPy, memoryview, bytes, bytearray, array.array) | ✅ YA zero-copy |
| `types.rs:18-40` `FlatBufferView` | Input put_batch vía buffer protocol | ✅ YA zero-copy |
| `types.rs:86-97` getter `.vector` | → `try_numpy_array()` `convert.rs:153-167` → numpy array via `VantaVector` | ❌ copia |
| `vector.rs:59-83` `__array_interface__` | Expone PyBytes **owned copy** — fix SEC-01/AUDIT-01 anti-UAF (getter devuelve owned PyBytes, no puntero crudo) | ❌ copia deliberada |

Costo de copia en Python hoy: Vec<f32> → PyBytes (owned, seguridad) → numpy.array =
2 copias. Revertir a zero-copy real reintroduce el UAF que AUDIT-01 corrigió.

**Interop Arrow (opción explorada):** `arrow::ffi` (feature `ffi` de arrow 59.2.0,
`to_ffi`/`export_array_into_raw`) + `pyarrow.Array._import_from_c` es el mecanismo
oficial (Apache Arrow C Data Interface). La crate `arrow-c-data` **no existe** en
crates.io (verificado, 404). Requiere que el cliente tenga pyarrow instalado — dependencia
pesada (~90MB wheel) que hoy NO está en el stack.

### 1.3 WASM/TS — ya resuelto (precedente PERF-08)

| Ubicación | Qué hay | Zero-copy? |
|---|---|---|
| `vantadb-wasm/src/lib.rs:1428-1447` | Output `js_sys::Float32Array::new_with_length` + `copy_from` (sanitize NaN/Inf → 0.0) | ✅ YA zero-copy |
| `vantadb-wasm/src/lib.rs:1462-1474` | Input `from_js` serde_wasm_bindgen | ❌ pendiente (parseo, no serialización de vectores) |
| `vantadb-ts/src/vantadb.ts:1` | Wrapper WASM — hereda Float32Array | ✅ |

### 1.4 Node nativo (`vantadb-node/`, napi-rs)

| Ubicación | Qué hay | Zero-copy? |
|---|---|---|
| `lib.rs:136-142` (`list`) | `serde_json::to_value(&out)` — JSON completo de VantaMemoryRecord incl. vector | ❌ peor path |
| `lib.rs:156-163` (`search`) | `serde_json::to_value(&out)` | ❌ peor path |
| `lib.rs:434+` (`get_f32_vec`) | Input parse de JS Array/typed array → Vec<f32> | ❌ copia |

napi-rs **soporta TypedArray zero-copy nativo** (`Float32Array`, `TypedArray` — docs
oficiales napi.rs). Mismo patrón que PERF-08, ~2h de trabajo. Pero el resto del record
(payload/metadata) igual pasaría por JSON/estructuras — solo el vector se beneficiaría.

---

## 2. Opciones por binding (costo/beneficio)

### 2.1 Python

| Opción | Esfuerzo | Beneficio | Riesgo |
|---|---|---|---|
| **A. Estado actual** (PyBytes owned + numpy) | 0 | Seguro (AUDIT-01) | 2 copias de Vec<f32> |
| B. `arrow::ffi` → pyarrow `_import_from_c` | 🔴 alto (API core nueva + binding + pyarrow dep) | Zero-copy verdadero a pyarrow/pandas/polars | pyarrow ~90MB dep; RecordBatch solo cubre id+vectores, no el shape completo; requiere re-apertura del fix SEC-01 (riesgo UAF) |
| C. `__buffer__`/memoryview output | 🟡 medio | Elimina 1 copia intermedia | Requiere unsafe buffer export (Regla 2: SAFETY); sigue habiendo ≥1 copia |

**Veredicto Python:** DIFERIR. Input ya es zero-copy; output con copia es decisión de
seguridad deliberada; pyarrow no es dependencia deseada; volúmenes de retrieval top-k
son pequeños (100 records × 1536 dims = 614KB → copias en µs, dominadas por HNSW traversal).

### 2.2 Node nativo

| Opción | Esfuerzo | Beneficio | Riesgo |
|---|---|---|---|
| **A. Estado actual** (serde_json completo) | 0 | Simple | JSON de vector ineficiente |
| B. `Float32Array` para vector (patrón PERF-08) | 🟢 2h | Elimina JSON del vector en output | Record completo sigue por JSON; cambio de shape para hosts Node |
| C. Buffer/IPC apache-arrow JS | 🔴 alto | Interop con dataframe JS | dep `apache-arrow` JS nueva (~pesada); mismo problema de schema parcial |

**Veredicto Node:** DIFERIR, pero B es el candidato natural si aparece la señal de
re-apertura (ver §4). Es cambio de binding puro, sin tocar core, mismo patrón ya
probado en WASM.

---

## 3. Decisión: DIFERIR (ADR) + plan futuro firmado

**Decisión:** NO implementar zero-copy Arrow en bindings Python/Node hoy. ADR de
diferimiento con las siguientes razones. **Formalizado en**
`docs/architecture/adr/ADR-025-zero-copy-arrow-deferred.md` (misma decisión/evidencia):

1. **RecordBatch desconectado y de schema parcial.** El único RecordBatch del core
   (`src/columnar.rs:22`) tiene solo id + vectores flatten — no cubre
   namespace/key/payload/metadata de `VantaMemoryRecord`. Zero-copy Arrow full exige
   API core nueva (emisión de RecordBatch desde search/list), fuera del scope de bindings.
2. **Input Python ya es zero-copy** (`extract_vector` PyBuffer, `FlatBufferView`). El
   único costo restante es el output, y es una copia deliberada de seguridad
   (SEC-01/AUDIT-01: owned PyBytes anti-UAF). Revertir reintroduce riesgo de memoria.
3. **Hot path JS ya resuelto por PERF-08** (Float32Array zero-copy en WASM, heredado
   por TS). El binding con peor serialización (Node nativo) no es el path más usado
   (TS envuelve WASM).
4. **pyarrow no es dependencia deseada** (~90MB wheel, no está en el stack). El
   mecanismo correcto (`arrow::ffi` C Data Interface) requiere que el cliente tenga
   pyarrow y además unsafe FFI en la frontera (Regla 2: requiere audit).
5. **Volúmenes actuales no justifican el costo.** Retrieval top-k típico: 10-100 records
   × 384-1536 dims = 15KB-614KB de vectores. Copia en µs-ms vs HNSW traversal en ms.
   El caso masivo (>40MB de vectores por respuesta) no pasa hoy por search/list.

### Plan futuro firmado (si aparece la señal)

**Output Node via TypedArray (≈2h, binding puro):**
1. En `vantadb-node/src/lib.rs` list/search: construir `Float32Array` para `vector` con
   `napi::bindgen_prelude::Float32Array::new_with_length` + `copy_from_slice` (mismo
   patrón que `vantadb-wasm/src/lib.rs:1428-1447`).
2. Mantener payload/metadata/keys como JSON — solo el vector cambia de shape.
3. Actualizar tipos TS en `vantadb-ts/src/types.ts` a `Float32Array | number[]` (ya hay
   precedente en PERF-08).
4. Verify: `cargo check -p vantadb-node`, `cargo clippy -p vantadb-node --all-targets -- -D warnings`, `cargo fmt --check`, + test node de shape.

**Input WASM pendiente** (`lib.rs:1462-1474` from_js) — problema de parseo, no de
serialización de vectores; seguir como está hasta que haya señal de costo.

---

## 4. Señal de re-apertura (medible)

Reabrir FND-04 (o crear task derivada) si **cualquiera** de estos umbrales se cumple:

1. **Benchmark de query grande:** `search_memory`/`list_memory` con top_k=10_000 y
   vectores 1536-dim en namespace ≥1M records (≥40MB de vectores por respuesta) muestra
   overhead de serialización/boundary (Rust Vec → numpy/JSON) >30% del tiempo total de
   la query.
2. **Caso de uso reportado:** interop analítica masiva (pandas/polars/dataframe) como
   requisito de producto → entonces pyarrow C Data Interface vuelve a la mesa (requiere
   API core nueva + decisión de dependencia pyarrow).
3. **Profiling Node:** `serde_json::to_value` en list/search >30% del tiempo de query
   con payloads >10MB → implementar opción B (§2.2) directamente (~2h).

## 5. Verificación de contrato

- [x] Path actual mapeado con archivo:línea (sección 1)
- [x] Decisión explícita con ADR de diferimiento + plan futuro (sección 3)
- [x] Señal de re-apertura medible (sección 4)
- [x] SDK/core NO modificado
- [x] NO git add/commit (lead commitea)

## Fuentes

- napi-rs TypedArray zero-copy: https://napi.rs/docs/concepts/typed-array
- Apache Arrow C Data Interface: https://arrow.apache.org/docs/format/CDataInterface.html
- arrow::ffi (arrow 59.2.0): https://docs.rs/arrow/59.2.0/arrow/ffi/
- Precedente PERF-08: tasks/PERF-08.md (`vantadb-wasm/src/lib.rs:1428-1447`)
- Fix SEC-01/AUDIT-01: `vantadb-python/src/vector.rs:59-83` (getter devuelve PyBytes owned copy)