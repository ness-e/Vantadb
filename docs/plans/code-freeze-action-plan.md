# Plan de Acción Consolidado — VantaDB Code Freeze

> Este documento rastrea la resolución de la deuda técnica crítica.
> No se fusionarán nuevas características (features) hasta que los elementos P0 estén resueltos.

---

## P0 — Bloqueante (Arreglar antes de cualquier feature)

*Requiere intervención inmediata en la infraestructura de CI.*

| ID | Área | Hallazgo | Archivo:Línea | Estado |
|:--:|:----:|----------|:-------------:|:------:|
| **P0-1** | CI | `cancel-in-progress: true` mata jobs de test a los 25 min si hay push rápido. 14 workflows afectados: `ci-rust-10`, `ci-web-11`, `heavy-certification-50`, `heavy-bench-nightly-51`, `fuzz-40`, `perf-bench-40`, `gate-docs-21`, `sec-codeql-30`, `release-adapters-62`, `release-npm-61`, `release-wheels-60`. Solución: `cancel-in-progress: true` solo para lint jobs (fmt, clippy), no para `test`/`coverage`. | `.github/workflows/*.yml` | ✅ |
| **P0-2** | CI | `memory-concurrency` con `continue-on-error: true` silencia fallos críticos (`memory_brutality`, `concurrency_parity`, `memory_api`, `memory_telemetry`, `edge_cases`). Tests que nadie monitorea. Solución: remover `continue-on-error` o crear GitHub Issues con tag `flaky` para cada test que falle. | `heavy-certification-50.yml:166` | ✅ |
| **P0-3** | CI | `failpoint-tests` sin `timeout-minutes` definido. Si un failpoint se cuelga (ej. panic en `crash_injection`), el job corre hasta 360 min (default GH Actions). Solución: agregar `timeout-minutes: 30` al job. | `heavy-certification-50.yml:89-107` | ⏳ |
| **P0-4** | CI | Miri con `continue-on-error: true` — Undefined Behavior en bindings FFI (PyO3, wasm-bindgen) no bloquea CI. Solución: evaluar si Miri ya es estable para el toolchain actual y cambiar a `continue-on-error: false`, o crear Issue para seguimiento. | `ci-rust-10.yml:332` | ⏳ |

---

## P1 — Alta (Planificar post-freeze)

*Deuda técnica y de rendimiento que afecta la eficiencia pero no compromete la integridad estructural.*

| ID | Área | Hallazgo | Archivo:Línea | Estado |
|:--:|:----:|----------|:-------------:|:------:|
| **P1-1** | CI | Benchmark dataset (GloVe) descargado en cada `test` job, incluso para PRs de docs/lint. Solución: mover a solo `coverage` o agregar `if: steps.cache-benchmark.outputs.cache-hit != 'true'`. | `ci-rust-10.yml:104-106` | ⏳ |
| **P1-2** | CI | Windows `test-threads=2` + timeout 25 min flaquea bajo carga del runner. `slow-timeout` 60s aborta tests I/O lentos. Solución: evaluar si `test-threads=2` sigue siendo necesario o aumentar timeout. | `nextest.toml:67`, `ci-rust-10.yml:142` | ⏳ |
| **P1-3** | CI | Cache GloVe con key fija (`glove-100d-v1`) que nunca se invalida. Si el script `download_benchmark_datasets.sh` cambia, la cache sigue sirviendo el viejo. Solución: usar hash del script como cache key. | `ci-rust-10.yml:102` | ⏳ |
| **P1-4** | CI | macOS carece de `rust-setup action` — no usa el action compartido `.github/actions/rust-setup` que Linux y Windows sí usan. Solución: unificar con `rust-setup`. | `ci-rust-10.yml:152` | ⏳ |
| **P1-5** | WASM | `wasm-opt = false` en `vantadb-wasm/Cargo.toml` — bundle 30-50% más grande, sin optimización post-compile. Nota ponytail: "re-enable when CI binaryen catches up to bulk-memory + sign-extension features". Verificar si binaryen ya soporta estas features. | `vantadb-wasm/Cargo.toml:15` | ⏳ |
| **P1-6** | WASM | Worker timeout 5s sin retry — `OpfsWorkerProxy` usa `WORKER_TIMEOUT_MS = 5000`. Si `createWritable` está bloqueado por otro tab, la operación falla sin reintento. Solución: agregar retry con exponential backoff. | `worker.rs:27` | ⏳ |
| **P1-7** | CI | Version extraction con `grep '^version'` frágil para workspace crates. `vantadb-python/Cargo.toml` usa `version.workspace = true`. El grep sobre el root `Cargo.toml` puede extraer la versión correcta por casualidad, o fallar si el root también usa workspace. | `release-wheels-60.yml:210`, `release-npm-61.yml:117` | ⏳ |
| **P1-8** | CI | Inconsistencia de timeouts en `heavy-certification-50.yml`: job timeout 180 min vs step timeout 150 min. El step timeout es **menor** que el job timeout. Solución: alinear a 150 min ambos o a 180 min ambos. | `heavy-certification-50.yml:28,36` | ⏳ |
| **P1-9** | WASM | SIMD duplicado: `vantadb-wasm/src/simd.rs` expone `cosine_distance_f32` con `wasm_simd128`, separado de los kernels SIMD en `src/index/distance.rs`. Riesgo de drift entre ambas implementaciones. | `vantadb-wasm/src/simd.rs`, `src/index/distance.rs` | ⏳ |
| **P1-10** | CI | PyPI "Wait for CDN propagation" de 90s durmiendo en CI. Si el CDN tarda más, el smoke test falla por timeout del step (10 min). Solución: reemplazar sleep con reintento con exponential backoff. | `release-wheels-60.yml:256-259` | ⏳ |

---

## P2 — Baja (Deuda cosmética / API interna)

*Optimizaciones de código y refactorizaciones menores.*

| ID | Área | Hallazgo | Archivo:Línea | Estado | Esfuerzo |
|:--:|:----:|----------|:-------------:|:------:|:--------:|
| **P2-1** | WASM | `OpfsFile::delete()` público pero siempre devuelve error. El delete real está en `OpfsStorage::delete_file()`. Solución: implementar `delete()` o marcar `#[doc(hidden)]`. | `opfs.rs:83-87` | ⏳ | 🟢 30 min |
| **P2-2** | PyO3 | `VantaVector.__array_interface__` expone puntero raw de `Vec<f32>` (riesgo de UB si Rust realloc mientras NumPy referencia). Solución: usar `Vec::leak()` o `Pin<Box<[f32]>>`. | `vantadb-python/src/lib.rs:1754` | ⏳ | 🟡 2-4 hr |
| **P2-3** | PyO3 | LRU cache `thread_local` `RefCell` con capacidad 64, O(n) linear scan (`Vec::iter().position()`). Solución: reemplazar con `LinkedHashMap` o aceptar que con cap 64 no importa. | `vantadb-python/src/lib.rs:34-36` | ⏳ | 🟢 15 min |
| **P2-4** | Docs | `docs/bitacora.md` (757 líneas) — bitácora personal de desarrollo en directorio de documentación técnica. Solución: mover a `docs/progreso/` o eliminar. | `docs/bitacora.md` | ⏳ | 🟢 5 min |
| **P2-5** | PyO3 | `put_batch()` dual API (positional tuples + keyword arrays) en el mismo método — ~60 líneas de branching y `Option` dispatch. Solución: deprecar la variante legacy formalmente con `#[deprecated]`. | `vantadb-python/src/lib.rs:824-887` | ⏳ | 🟢 1 hr |
| **P2-6** | PyO3 | Match no exhaustivo en `map_vanta_error()` — 20+ variantes de `VantaError` manejadas + `_ => PyRuntimeError` catch-all. Nuevas variantes caen silenciosamente en el catch-all. Solución: marcar `VantaError` como `#[non_exhaustive]`. | `vantadb-python/src/lib.rs:688-712` | ⏳ | 🟢 15 min |
| **P2-7** | WASM | Serialización completa `serde-wasm-bindgen` en cada llamada. Sin zero-copy path para vectores f32 de 384/768 dims (~2-5µs overhead por vector). | `vantadb-wasm/src/lib.rs:895-901` | ⏳ | 🟡 4-8 hr |
| **P2-8** | WASM | `collect_all_deduped()` O(n) en memoria — carga TODOS los registros con `usize::MAX` de límite. Para DBs con millones de registros, explota el heap WASM (~2GB límite). | `vantadb-wasm/src/lib.rs:394-413` | ⏳ | 🟡 2-4 hr |
| **P2-9** | WASM | `extract_vector()` no es zero-copy real en PyO3 — `PyBuffer::as_slice()` devuelve `&[ReadOnlyCell<f32>]`, cada `cell.get()` es lectura atómica. Limitación de PyO3. | `vantadb-python/src/lib.rs:231-234` | ⏳ | 🟡 4-8 hr |
| **P2-10** | PyO3 | `FlatBufferView` no usado realmente en hot paths de `search()`. Existe pero `row_to_vec()` copia cada fila. | `vantadb-python/src/types.rs:19-38` | ⏳ | 🟢 30 min |
| **P2-11** | Docs | `docs/auditoria-completa.md` (305 líneas) — snapshot de auditoría 2026-07-14. Los hallazgos ya fueron integrados. | `docs/auditoria-completa.md` | ⏳ | 🟢 5 min |
| **P2-12** | Docs | `docs/archived-decisions/experimental-quarantine-2024-06.md` — ADR de 2024, experimental obsolete. | `docs/archived-decisions/experimental-quarantine-2024-06.md` | ⏳ | 🟢 5 min |
| **P2-13** | Docs | `docs/Investigaciones/cargo-check-optimizacion.md` — investigación posiblemente ya aplicada. | `docs/Investigaciones/cargo-check-optimizacion.md` | ⏳ | 🟢 10 min |

---

## Progreso

| Fecha | Ítems resueltos | Notas |
|:----:|:---------------:|:------|
| 2026-07-17 | P0-1 | `cancel-in-progress` movido a solo lint jobs en ci-rust-10.yml; desactivado en ci-web-11, heavy-bench-nightly-51, perf-bench-40, sec-codeql-30 |

<!--
  Estados: ⏳ Pendiente, 🔄 En progreso, ✅ Completado, ❌ Bloqueado
  Al resolver un ítem, mover la línea a Resueltos y agregar la fecha.
-->
