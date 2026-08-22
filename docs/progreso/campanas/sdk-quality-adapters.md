# Calidad SDK y adapters — COMP/GH/REV/adapters

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### COMP-028: Semantic Cost Estimator (SCE) unificado
- **Fuente:** Backlog (Phase 10 — Competitive Features)
- **Fecha:** 2026-08-02
- **Objetivo:** Extraer la estimación de costos de query, distribuida en 3 componentes (ResourceGovernor, CBO, `select_filter_strategy`), a un módulo unificado `src/cost_estimator.rs` sin cambio de comportamiento público. Habilitador de OLD-21 (routing multi-índice HNSW/IVF/Flat).
- **Resultado:** ✅ `src/cost_estimator.rs` (nuevo): `CostEstimator<'a>` con `selectivity()` (lógica 1:1 de `get_estimated_selectivity`), `estimate_operator()` (Scan/FilterRelational/VectorSearch/Limit/Traverse/Sort/Project/Join/SubqueryFilter con stats ya disponibles, sin scans), `estimate_plan()` (encadena operadores, rows fluyen, bytes = operador pico), `select_filter_strategy()` y `FilterStrategy` movidos desde `sdk/search/mod.rs`. `StorageEngine::get_estimated_selectivity` conserva firma pública y delega (21 callers intactos). `ResourceGovernor::estimate_plan_cost` `pub(crate)` unwired con `#[allow(dead_code)]` (consumidor OLD-21 futuro). 4 tests unitarios nuevos. `cargo check`/`fmt`/`clippy -D warnings` exit 0; `cargo nextest --profile audit -p vantadb` 1776 passed. Commit `f7cb46e4`.
- **Ids:** `COMP-028`

### OLD-21: CP-Index formal (query routing inteligente)
- **Fuente:** Backlog (Phase 9 — Features perdidas con alto valor de mercado)
- **Fecha:** 2026-08-03
- **Objetivo:** Formalizar el query routing multi-índice (HNSW/IVF/Flat) usando el Semantic Cost Estimator de COMP-028. Habilitado por la dependencia COMP-028 ✅.
- **Resultado:** ✅ `CostEstimator::select_index_strategy()` en `src/cost_estimator.rs` (heurística: Flat si `nodes <= flat_threshold` — consistente con `CPIndex::use_flat_search`; IVF si `nodes >= 10_000`; HNSW default; respeta config explícita no-HNSW). Conectado en `vector_memory_search` (`src/sdk/search/mod.rs`) + métrica `record_vector_index_routing` (3 contadores en `src/metrics/core/mod.rs` + snapshot). Admission budget de `Executor::execute_plan` (`src/executor.rs`) ahora usa `ResourceGovernor::estimate_plan_cost` (removido `#[allow(dead_code)]`) en vez del MIB fijo; guard libera bytes en error path (test `test_execute_plan_frees_admission_on_error`). 6 tests nuevos (4 routing + 1 admission + 1 governor). `cargo check`/`fmt`/`clippy -D warnings` exit 0; `cargo nextest --profile audit --workspace --build-jobs 2` 1816 passed, 2 skipped. Firmas públicas intactas (search()/VantaMemorySearchRequest sin cambios); `planner.rs` classify/CBO no tocado.
- **Ids:** `OLD-21`

### TSK-107b: Audit logging enterprise (JSONL, timestamp + op)
- **Fuente:** Backlog (Phase 8 — Post-Launch & Enterprise)
- **Fecha:** 2026-08-02
- **Objetivo:** Módulo de audit log append-only en JSONL (timestamp ISO 8601 + operación) para compliance y debugging en producción. Opt-in vía config.
- **Resultado:** ✅ `src/audit.rs` (nuevo): `AuditEvent` (timestamp ISO 8601 UTC, op, namespace, key, outcome, reason) + `AuditLogger` (Mutex<BufWriter<File>>, append+flush por registro, `chrono`+`web_time` wasm-safe). Campo `VantaConfig.audit_log_path` + env `VANTADB_AUDIT_LOG_PATH` + builder `with_audit_log_path()`. Hooks en put, put_batch, delete (reason "memory delete"), delete_by_filter, export_namespace, export_all, import_file — todos vía `VantaEmbedded` (cubre CLI/WASM/Python/MCP/TS). No-op si no está configurado; fallos de audit solo warn, nunca fallan la operación. Consume el placeholder `reason` reservado en `ops.rs`. `cargo check`/`fmt --check`/`clippy -D warnings` exit 0; `cargo nextest --profile audit` 1772 passed, 2 skipped; `tests/audit_log.rs` 3/3 (timestamp Z, op, reason, no-op).
- **Ids:** `TSK-107b`

### GH-122: Docstrings en API pública del Python SDK
- **Fuente:** Backlog (Phase 11 — GitHub Issues)
- **Fecha:** 2026-08-02
- **Objetivo:** Docstrings completos (Args, Returns, Raises, ejemplo runnable) en `vantadb-python/src/lib.rs` para los 12 métodos públicos de `VantaDB`: `new()`, `put()`, `get_memory()`, `delete_memory()`, `list_memory()`, `search_memory()`, `insert()`, `rebuild_index()`, `export_namespace()`, `export_all()`, `import_file()`, `operational_metrics()`. Visibles como docstrings Python vía PyO3.
- **Resultado:** ✅ 12/12 métodos documentados con Args+Returns+Raises+ejemplo ` ```python ` self-contained (`VantaDB(":memory:", backend="memory")`, patrones de `tests/test_sdk.py`). Docstring de clase `VantaDB.__doc__` enriquecido con la doc del constructor (único canal PyO3 para la clase). `cargo check`/`fmt --check`/`clippy -D warnings` exit 0; `maturin develop` instalado; 11/12 `__doc__` verificados en Python. +382/−2 en `vantadb-python/src/lib.rs`.
- **Ids:** `GH-122`

### GH-142: Smoke tests de examples en CI
- **Fuente:** Backlog (Phase 11 — GitHub Issues)
- **Fecha:** 2026-08-02
- **Objetivo:** Verificar en CI que todos los examples (`examples/python/`, `examples/rust/`) corren como smoke tests (no benchmarks) y bloquear PRs que los rompan.
- **Resultado:** ✅ Workflow nuevo `ci-examples-12.yml`: job `rust-examples` (4 `cargo run --example` con rust-setup) + job `python-examples` (setup-python 3.11, wheel maturin local, 1 step por example, sin `continue-on-error`). Se detectaron y repararon 7 examples Python con drift de API (nunca corrieron en CI): `db.list()`→`list_memory()` (retorno `VantaListResult` iterable) y `search_memory(query_vector=None)`→`query_vector or []` (vector requerido). Local: 4/4 Rust exit 0, 10/10 Python exit 0. actionlint ok. Los examples solo importan `vantadb_py` + stdlib (sin libs de framework ni API keys).
- **Ids:** `GH-142`

### GH-144: i18n traducciones ES para showcase page
- **Fuente:** Backlog (Phase 11 — GitHub Issues)
- **Fecha:** 2026-08-05
- **Objetivo:** Completar claves i18n `showcasePage.*` en español para `web/src/app/showcase/page.tsx`.
- **Resultado:** ✅ Ya resuelto: `web/src/lib/dictionaries.ts` contiene 22 claves `showcasePage.*` completas en ES (L1370-1391) y EN (L2856-2877); la página usa `tt()` con fallback. Verificado 44 matches del token (22+22). Issue #144 cerrado con comentario de evidencia (backlog-validation F1, 2026-08-05).
- **Ids:** `GH-144`

### GH-124: Ejemplos doc-test para API pública Rust
- **Fuente:** Backlog (Phase 11 — GitHub Issues)
- **Fecha:** 2026-08-02
- **Objetivo:** Agregar ejemplos doc-test runnable (`/// ```rust`) a la API pública: `VantaEmbedded::open()`, `open_with_config()`, `put()`, `get()`, `delete()`, `search()` y `VantaConfig`.
- **Resultado:** ✅ 7 doc-tests nuevos (uno por función objetivo) usando `BackendKind::InMemory` + `":memory:"` (self-contained, sin dejar archivos). Se repararon 2 doc-tests pre-existentes rotos (`console.rs` `init_logging` con `LogFormat`; `lib.rs` `VantaMemoryInput` sin `Default`). `cargo test --doc -p vantadb` 11/11 pass; `cargo doc --no-deps` 0 warnings nuevos (20 pre-existentes); fmt y clippy clean.
- **Ids:** `GH-124`

### GH-127: Property-based tests para roundtrip de WAL
- **Fuente:** Backlog (Phase 11 — GitHub Issues)
- **Fecha:** 2026-08-02
- **Objetivo:** Proptest para roundtrip serialize→deserialize de cualquier `WalRecord` válido (7 variantes), payloads de varios tamaños (vacío, 1 byte, 64KB, ~1MB) y escrituras concurrentes.
- **Resultado:** ✅ `tests/proptest_wal_roundtrip.rs` creado. 4 tests: roundtrip bytes puro (1000 casos), payload buckets, file roundtrip batch, concurrent writes. nextest 4/4, clippy y fmt limpios.
- **Ids:** `GH-127`

### COMP-026: Multi-level LSM Compaction (L0→L1→L2→L3)
- **Fuente:** Backlog (Phase 10 — Competitive Features)
- **Fecha:** 2026-07-28
- **Objetivo:** Extender VantaFile de archivo único a múltiples niveles LSM con compactación independiente por nivel. SegmentRegistry, compact_level(), PipelineMode extendido.
- **Resultado:** ✅ `cargo check -p vantadb` — 0 errores. 13+ archivos modificados. L0+L1 implementados (ponytail: L3 archive deferido).
- **Ids:** `COMP-026`

### REC-007: WAL Compaction + Vacuum CLI
- **Fuente:** Backlog (Phase 8 — Post-Launch & Enterprise)
- **Fecha:** 2026-07-29
- **Objetivo:** Exponer `VantaEmbedded::compact_wal()` y `VantaEmbedded::vacuum()` como comandos CLI `vanta-cli wal compact` / `vanta-cli wal vacuum`. Binding directo sin lógica nueva.
- **Resultado:** ✅ `cargo check -p vantadb --features cli` — 0 errores. 4 archivos modificados.
- **Ids:** `REC-007`

### REC-010: py.typed marker + maturin wheel inclusion
- **Fuente:** Backlog (Phase 8 — Post-Launch & Enterprise)
- **Fecha:** 2026-07-29
- **Objetivo:** PEP 561 compliance — crear `py.typed` marker y configurar `[tool.maturin] include` en `pyproject.toml` para incluir stubs `.pyi` en el wheel.
- **Resultado:** ✅ `py.typed` creado (vacio) en `vantadb_py/`. `pyproject.toml` incluye `include = ["vantadb_py/py.typed", "vantadb_py/*.pyi"]`. 2 archivos modificados.
- **Ids:** `REC-010`

### COMP-006: Edge Label Interning (u32 label_id)
- **Fuente:** Backlog (Phase 10 — Competitive Features)
- **Fecha:** 2026-07-27
- **Objetivo:** `Edge.label: String` → `Edge.label_id: u32` con LabelIntern (HashMap<String, u32>). Reduce ~80MB heap para 1M edges.
- **Resultado:** ✅ 1,618 tests pasan. SDK público inalterado (VantaEdgeRecord.label sigue String).
- **Ids:** `COMP-006`

### COMP-018: Double-linked Relationship Chains
- **Fuente:** Backlog (Phase 10 — Competitive Features)
- **Fecha:** 2026-07-28
- **Objetivo:** Relations dirigidas con doble enlace. Edge.reverse + add_edge/remove_edge bidireccional + TraversalDirection + direction param en Rust SDK (4 métodos), bindings WASM y Python.
- **Resultado:** ✅ `cargo check` en 3 crates. 33 graph tests pasan. Backward compatible (default Forward).
- **Ids:** `COMP-018`

### REV-001: CI Rust TSan ABI mismatch
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Remove `-Zsanitizer=thread` flag incompatible with Rust 1.94.1; fix YAML indent error in `msrv` job
- **Resultado:** ✅ CI workflow validates (yamllint). Commit `35873e6`.
- **Ids:** `REV-001`

### REV-002: CI Web 21 ESLint errors
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Fix 19 prettier errors (auto-fix) + 3 react-hooks/exhaustive-deps warnings
- **Resultado:** ✅ `npm run lint` — 0 errors, 0 warnings. Commit `35873e6`.
- **Ids:** `REV-002`

### DRV-099: Haystack protocolo Document real
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** `write_documents` accepts both `dict` and `Document`; `filter_documents` returns real `Document` instances with typed meta conversion
- **Resultado:** ✅ `cargo check -p vantadb-haystack` passes, 9/9 Python tests pass. Commit `7fb0a1f`.
- **Ids:** `DRV-099`

### DRV-102: Langchain missing GIL release
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Release GIL in `add_texts`, `similarity_search_by_vector`, `delete` using pyo3 0.29 `detach()` API
- **Resultado:** ✅ `cargo check -p vantadb-langchain` passes, `cargo build` passes. Commit `3cc6888`.
- **Ids:** `DRV-102`

### DRV-103: LangChain metadata no-string ignorado
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Fallthrough chain String→bool→i64→f64 for non-string metadata values in add_texts
- **Resultado:** ✅ cargo fmt, check, clippy pasan. Commit `b83f0f9`.
- **Ids:** `DRV-103`

### DRV-110: LlamaIndex metadata no-string ignorado
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Mismo fallthrough chain String→bool→i64→f64 para LlamaIndex
- **Resultado:** ✅ cargo fmt, check, clippy pasan. Commit `b83f0f9`.
- **Ids:** `DRV-110`

### DRV-086: CrewAI metadata no-string ignorado
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Fallthrough chain con to_string() para compatibilidad BTreeMap<String,String>
- **Resultado:** ✅ cargo fmt, check, clippy pasan. Commit `b83f0f9`.
- **Ids:** `DRV-086`

### DRV-092: DSPy metadata no-string ignorado
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Fallthrough chain String→bool→i64→f64 para DSPy
- **Resultado:** ✅ cargo fmt, check, clippy pasan. Commit `b83f0f9`.
- **Ids:** `DRV-092`

### DRV-104: LangChain similarity_search no retorna metadata
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-15
- **Objetivo:** Extraer hit.record.metadata como BTreeMap<String,VantaValue> en Phase 1 (GIL released), convertir a PyDict en Phase 2 (GIL fresco)
- **Resultado:** ✅ cargo fmt, check, clippy pasan. Commit `1b2c183`.
- **Ids:** `DRV-104`

### DRV-105: LangChain delete() silenciosamente no-op en IDs malformados
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-15
- **Objetivo:** Retornar PyRuntimeError cuando id.split(':') produce formato inválido en vez de silenciar la operación
- **Resultado:** ✅ cargo fmt, check pasan. Commit `7de6a0e`.
- **Ids:** `DRV-105`

### DRV-106: LangChain from_texts class method
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-15
- **Objetivo:** Implementar from_texts como #[classmethod] PyO3 que crea store, llama add_texts, retorna instancia
- **Resultado:** ✅ cargo fmt, check pasan. Commit `d355389`.
- **Ids:** `DRV-106`

### DRV-111: LlamaIndex query() retorna metadata
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-15
- **Objetivo:** Agregar hit.record.metadata al result dict
- **Resultado:** ✅ cargo fmt, check pasan. Commit `e19642f`.
- **Ids:** `DRV-111`

<!-- movido a ARCHIVO_HISTORICO.md -->
<!-- movido a ARCHIVO_HISTORICO.md -->
### DRV-063: Ollama metadata no-string ignorado
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Handle bool/int/float metadata values in Ollama store() instead of silently dropping non-string values — same pattern as DRV-058
- **Resultado:** ✅ `cargo check -p vantadb-ollama` clean, clippy clean.
- **Ids:** `DRV-063`

### DRV-062: Ollama client recreado en cada embed()
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Cache `Py<PyAny>` Ollama client in struct field instead of recreating per `embed()` call — same pattern as DRV-057
- **Resultado:** ✅ `cargo check -p vantadb-ollama` clean, clippy clean.
- **Ids:** `DRV-062`

### DRV-058: OpenAI metadata no-string ignorado
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Handle bool/int/float metadata values in store() instead of silently dropping non-string values
- **Resultado:** ✅ `cargo check -p vantadb-openai` clean, clippy clean.
- **Ids:** `DRV-058`

### DRV-057: OpenAI client recreado en cada embed()
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Cache `Py<PyAny>` OpenAI client in struct field instead of recreating per `embed()` call — eliminates TLS handshake + connection pool churn
- **Resultado:** ✅ `cargo check -p vantadb-openai` clean, clippy clean.
- **Ids:** `DRV-057`
