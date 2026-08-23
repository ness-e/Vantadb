# Auditoría Integral 2026-06-19 + correcciones documentales

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

## Auditoría Integral (2026-06-19) — COMPLETADO ✅

Auditoría automatizada de 44 hallazgos ejecutada y resuelta en su totalidad el mismo día. Cada hallazgo fue delegado a un agente especializado para su diagnóstico y corrección.

### 🔴 Críticos (7/7 ✅)

| ID | Fix | Impact |
|----|-----|---------|
| AUD-01 | `abi3-py311` → `abi3-py38` en `vantadb-python/Cargo.toml:13` | PyPI wheels ahora soportan Python 3.8–3.10 |
| AUD-02 | 16 `.unwrap()` → `?` + error handling (index.rs:13, storage.rs:1, wal.rs:2) | Eliminados panics en runtime por datos corruptos |
| AUD-03 | `bincode 1.3` → `2.0` (serde feature, 8 archivos, 27 call sites) | RUSTSEC-2025-0141 resuelto |
| AUD-04 | `pyo3 0.24` → `0.29` (3 breaking changes migrados: `PyObject`→`Py<PyAny>`, `.downcast()`→`.cast()`, `.allow_threads()`→`.detach()`) | RUSTSEC-2026-0176/0177 resueltos |
| AUD-05 | 18 links reparados en README.md + README_ES.md | Contribute/Security/Support → `.github/`, Python SDK → `docs/api/`, Benchmarks → `docs/operations/` |
| AUD-06 | `chaos_testing.rs` → `chaos_integrity.rs` en DURABILITY_GUARANTEES.md:287 | Referencia corregida |
| AUD-07 | `README.MD` → `README.md` en README_ES.md:24 | Case-sensitive FS fix |

### 🟡 Medios (14/14 ✅)

| ID | Fix |
|----|-----|
| AUD-08 | Auditoría completa de 39 ítems unsafe (33 bloques, 4 impls, 1 pub fn, 1 extern fn). 77% low-risk, 20.5% medium, 2.6% high. Reporte detallado. |
| AUD-09 | `static TEST_RESULTS` eliminado. `MULTI_PROGRESS` migrado a `thread_local!` + `RefCell`. |
| AUD-10 | Env vars guardadas/restauradas en prefetch_benchmark.rs. |
| AUD-11 | ~153 assertions con mensajes descriptivos (basic_node, benchmark_internal, test_sdk.py ~85, mcp_tests.rs 58, mcp_integration.rs 3). |
| AUD-12 | `StdRng::seed_from_u64(42)` en hnsw_recall.rs + prefetch_benchmark.rs. Benchmarks reproducibles. |
| AUD-13 | `TempDir` en basic_node.rs y benchmark_internal.rs. |
| AUD-14 | `ttl_ms: int \| None = None` agregado a `AsyncVantaDB.put()`. |
| AUD-15 | `tower 0.4` → `0.5` unified via dev-dep upgrade. |
| AUD-16 | 3 stale advisory ignores removidos de deny.toml. `cargo deny check` → OK. |
| AUD-17 | `rust-toolchain.toml`: `1.94.1` → `stable`. |
| AUD-18 | Windows CI ahora ejecuta `cargo test --workspace`. |
| AUD-19 | `curl -s` → `curl -sL` en install.sh. |
| AUD-20 | Detección `aarch64`/`arm64` + `x86_64`/`amd64` en install.sh. Unknown arches → hard-fail. |
| AUD-21 | Ref a `ROADMAP.md` en CHANGELOG.md comentada como TODO. |

### 🔵 Bajos (23/23 ✅)

| ID | Fix |
|----|-----|
| AUD-22 | `governor.request_allocation()` error ahora propaga via `?`. |
| AUD-23 | 4 sitios de flush/eviction ahora logean `tracing::warn!`. |
| AUD-24 | `compact_layout_bfs()` (249L → 53L orquestador + 3 helpers). |
| AUD-25 | `add()` (214L → 8L dispatcher + `validate_node`, `insert_hnsw`, `update_metadata`). |
| AUD-26 | `open_with_config()` (271L → 59L pipeline + 4 helpers). |
| AUD-27 | Backend string inválido → `tracing::warn!`. |
| AUD-28 | `distance_metric` inválido → `tracing::warn!`. |
| AUD-29 | 6 archivos unificados a `ness-e/Vantadb`. |
| AUD-30 | `time.sleep(0.01)` → `_wait_until()` retry loop (5-10s timeout). |
| AUD-31 | `arrow`, `rocksdb`, `fjall` feature-gated (default incluye las 3). |
| AUD-32 | `nightly_bench.yml`: `checkout@v4` → `@v6`. |
| AUD-33 | `heavy_certification.yml`: `install-action@nextest` → `@v2` + `tool:`. |
| AUD-34 | Commit count: `237` → `460` en progreso docs. |
| AUD-35 | 4 sleeps reemplazados: `wait_for_port()`, `JoinHandle::await`, 1 justificado con comentario. |
| AUD-36 | Mensaje agregado a `assert_eq!`, `assert!(true)` ya no existía. |
| AUD-37 | `tests/edge_cases.rs` creado: 25 tests cubriendo 17 categorías (NaN, Inf, empty, unicode, TTL, etc.). |
| AUD-38 | Tokio features: `"full"` → granulares (`rt`, `sync`, `time`, `macros`, etc.). |
| AUD-39 | `wide = "=1.2.0"` → `">=1.2, <2"`. |
| AUD-40 | `[workspace.package]` creado. 3 sub-crates migrados a `version.workspace = true`. |
| AUD-41 | `maturin-action@v1` → `@v2`. |
| AUD-42 | `vantadb-mcp` agregado al build/release/hash/attest en release.yml. |
| AUD-43 | Free disk space + 6GB swap agregados a nightly_bench.yml. |
| AUD-44 | `setup-python@v5` → `@v6`. |

### 2026-06-22 (2ª pasada) — Cobertura documental completa

- **HTTP_API.md:** Nuevo — documenta `GET /health`, `GET /metrics`, `POST /api/v2/query` con auth, rate limiting, TLS, payloads y ejemplos curl.
- **PYTHON_SDK.md:** +27 métodos nativos Rust añadidos (node/graph API, maintenance, export/import, text index, utilities, observability). Tabla de tipos de retorno 26→52 filas.
- **CONFIGURATION.md:** +9 comandos CLI documentados (audit-index, repair-text-index, query, status, search, delete, completions, namespace, server). Nueva sección de 14 Cargo features con descripciones.
- **vantadb-ts/README.md:** +9 métodos TS añadidos (exportNamespace, exportAll, importRecords, importFile, auditTextIndex, auditTextIndexDeep, repairTextIndex, query, generateSnippet).
- **Master Index.md:** EMBEDDED_SDK.md marcado como ❌ Faltante (pendiente de creación). HTTP_API.md corregido a Done.
- **EMBEDDED_SDK.md:** Nuevo — referencia completa de `VantaEmbedded` (~45 métodos, ~15 tipos de datos, 5 tipos de reporte).
- **Cobertura documental 100% completa:** todos los archivos del Master Index existen y están actualizados.

### 2026-06-22 — Corrección de Documentación (ADVANCED_TOKENIZER, CONFIGURATION, PYTHON_SDK, Master Index)

- **ADVANCED_TOKENIZER.md:** `VantaDB`→`VantaEmbedded`, `put_memory`→`put`, `search_memory`→`search`, imports corregidos (`tokenizer::` en vez de `text_index::`), sección "Future Enhancements" obsoleta eliminada y reemplazada por runtime config real.
- **CONFIGURATION.md:** Tabla expandida de ~15 a 26 campos. Env vars corregidas (`VANTADB_THREADS`→`VANTADB_MAX_BLOCKING_THREADS`, `HOST`/`PORT` como fallbacks). Secciones de enums, CLI y notas operativas agregadas.
- **PYTHON_SDK.md:** Versión actualizada 0.1.1→0.1.5. ~20 métodos faltantes documentados (put_batch, consolidate, knowledge, ask, chat, from_file/url, etc.). Tabla completa de tipos de retorno. Changelog expandido.
- **Master Index.md:** 4 anchors TOC rotos reparados. `[progress](../../progreso/README.md)`→ruta relativa. Glosario 47→50 términos. Enlaces cruzados normalizados.
- **Checkpoint.md:** Nuevo — resumen anclado del vault MPTS con cobertura, backlog activo y estado actual.
