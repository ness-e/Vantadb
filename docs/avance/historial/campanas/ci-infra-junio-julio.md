# CI/infraestructura junio-julio — AUD-WORK, WASM, TSK, CLI-EPIC

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### Tarea: AUD-WORK — Corrección de CI y Auditoría de Workflows (2026-06-20)

- **Objetivo:** Corregir las fallas de la pipeline de CI de GitHub Actions (timeout en `crash_injection` y falla de permisos de `wal_write_failure_returns_error`) y aplicar los 9 hallazgos del reporte de auditoría de forma estructurada.
- **Commits:** `85f2beb`, `447224e`, `4030d36`, `ab09229`, `25dc38b`, `a3c2c04`, `aaf0428`, `26afb62`
- **Checklist Completado:**
- [x] Modificar `.config/nextest.toml`
- [x] Migrar exclusiones de `binary_id(...)` a `binary(...)`
- [x] Fix `hnsw_recall` a `hnsw_recall_certification`
- [x] Cambiar `not test(integrations_certification)` a `not binary(integration)`
- [x] Añadir exclusión de `mcp_tests` y `multilingual_tokenizer_integration`
- [x] Añadir exclusión de `memory_telemetry` y del unit test `concurrent_insert_preserves_hnsw_invariants`
- [x] Modificar `Cargo.toml`
- [x] Declarar `fjall_cold_copy_restore`, `property_durability`, `fuzz_proptest` y `multilingual_tokenizer_integration`
- [x] Añadir `required-features = ["failpoints"]` a `chaos_integrity` (`Cargo.toml:201`)
  - [x] Actualizar Workflows y Políticas
    - [x] Modificar `heavy_certification.yml` para incluir `--features cli,arrow` y clasificar `mcp_tests`, `multilingual_tokenizer_integration`, `columnar`, `memory_telemetry` y `concurrent_insert_preserves_hnsw_invariants`
- [x] Modificar `docs/operations/CI_POLICY.md`
- [x] Dividir quick CI (<30min) de la heavy certification semanal (`aaf0428`)
- [x] Reforzar la expresión de filtro de nextest (`a3c2c04`)
- [x] Restaurar filtro estricto binary_id de nextest con features cli (`25dc38b`)
- [x] Fix de extracción de versión en python_wheels.yml, mejorar comentario de test-threads (`26afb62`)
- [x] Entorno de Validación Local (Pre-push)
- [x] Añadir `numpy` al virtualenv de auditoría Python en `dev-tools/setup_venv.ps1`
- **Reporte original pendiente:**
- [x] ~~`Cargo.toml`: Añadir `required-features = ["failpoints"]` a `chaos_integrity`~~ → **Completado** en `Cargo.toml:37`
- [ ] `.config/nextest.toml`: Hacer `test-threads = 2` específico de Windows (actualmente global en `nextest.toml:67`)
- **Cambios y Resultados:**
- **Soporte robusto de workspace en Nextest:** Cambiar `binary_id(...)` a `binary(...)` en `nextest.toml` asegura que los binarios pesados sean excluidos efectivamente del PR Fast Gate, previniendo fallos de permisos root y timeouts de CI rápida.
- **Exclusiones de tests de larga duración:** Identificados y excluidos `memory_telemetry` (timeout local 180s) y el unit test lento `concurrent_insert_preserves_hnsw_invariants` (~68s) del fast gate, acelerando la pipeline.
- **Validación del Python SDK corregida:** Instalado `numpy` en el virtualenv tightening de auditoría (`dev-tools/setup_venv.ps1`) para que los tests de integración del Python SDK que dependen de NumPy pasen correctamente y no bloqueen el git pre-push.
- **Declaración explícita de tests:** Tests sin input explícito (`Glossary/test.md`) en `Cargo.toml` fueron declarados formalmente para evitar su desaparición por auto-discovery.
- **Clasificación en Heavy Certification:** `mcp_tests`, `multilingual_tokenizer_integration`, `memory_telemetry` y `concurrent_insert_preserves_hnsw_invariants` fueron clasificados para ejecutarse exclusivamente en `heavy_certification.yml` y documentados en `CI_POLICY.md`.
- **Ejecución de test columnar:** La feature `arrow` fue habilitada en los workflows y `columnar` se programó para evaluarse en CI.
- **CI pendiente:** `.config/nextest.toml` — `test-threads = 2` movido de global a `[profile.audit.overrides."cfg(target_os = \"windows\")".override]` solo-Windows.
<!-- dedup: filas tabla TSK-47, TSK-49, DISC-02, DISC-03, ROAD-06 cubren estas entradas (antes colgadas del bloque AUD-WORK) -->

<!-- dedup: fila tabla TSK-45 cubre esta entrada -->
<!-- dedup: fila tabla TSK-106b cubre esta entrada -->
<!-- dedup: fila tabla TSK-71 cubre esta entrada -->
### Fix de build WASM para Browser (wasm32-unknown-unknown) — pánico de SystemTime (2026-06-21)

- **Goal:** Remove `std::time::SystemTime::now()` panics when building `vantadb-wasm` for `wasm32-unknown-unknown` (target browser WASM).
- **Problem:** `SystemTime::now()` is not available in `wasm32-unknown-unknown`. Caused runtime panic when loading the WASM.
- **Fix:** Replace all occurrences of `std::time::SystemTime` and `std::time::UNIX_EPOCH` with `web_time::SystemTime` / `web_time::UNIX_EPOCH` (crate `web-time` v1.1.0, compatible with WASM and native).
- **Archivos modificados (11):**
- `src/binary_header.rs` — import + `verify_magic_number()`
- `src/segment_expiry_state.rs` — `SegmentExpiryState`
- `src/segment_redundancy.rs` — `SegmentRedundancy`
  - `src/sync_verification.rs` — `SyncVerification`
- `src/cluster_manager.rs` — `ClusterManager`
- `src/sdk.rs` — import + `now_ms()`
- `src/storage.rs` — import
- `src/wal.rs` — 2x `now()` + 2x `duration_since()`
- `src/cli_handlers.rs` — `now()` + `duration_since()`
- `src/executor.rs` — `now()` + `duration_since()`
- `src/gc.rs` — import
- **Verification:**
- `cargo build --target wasm32-unknown-unknown` (from `vantadb-wasm/`): ✅ no errors
- `load test --lib` (native): ✅ 48 tests, 0 failures

### TSK-112 — Empaquetar `vantadb-wasm` como SDK TypeScript en npm (2026-06-21)

- **Goal:** Compile, package and publish `vantadb-wasm` as a working TypeScript SDK on npm with integration tests, samples for Vercel AI SDK / LangChain / LlamaIndex, and professional README.
- **Commits:** *(pending)*
- **Checklist completed:**
- [x] `wasm-pack build --target bundler` from `vantadb-wasm/` — WASM binary compiled in `vantadb-wasm/pkg/`
  - [x] `vantadb-ts/package.json` — `main`, `types`, `exports`, `files`, `repository`, `homepage`, `bugs`, `prepublishOnly` configurados
- [x] `vantadb-ts/vantadb.ts` — TypeScript wrappers: `VantaDB` class, types `MemoryRecord`, `SearchResult`, `Capabilities`, `OperationalMetrics`, `ListPage`
- [x] `vantadb-ts/types.ts` — types `MemoryInput`, `VantaMemoryMetadata`, all u64s exposed as `string`
- [x] `vantadb-ts/README.md` — SDK docs with quick start, runtime support matrix (Node/Bun/Deno/browser), full API table
- [x] `vantadb-ts/test-runner.mjs` — Node.js ESM test runner with `--experimental-wasm-modules`, 26 integration tests
- [x] Fix u64 > 2^53 in WASM bindings: `memory_record_to_js()` + `search_hit_to_js()` manual helpers with `js_sys::Reflect`
- [x] Fix `read_header` alignment: `DiskNodeHeader::ref_from_bytes` (zerocopy) → `read_from_bytes` (owned copy) in `storage.rs:579`
- [x] Fix deref in `storage.rs:1535` — `*h` → `h` after change to owned header
- [x] Debug tracing cleanup (WARN/INFO logs removed)
- [x] Removing unused structs (`JsMemoryRecord`, `JsMemorySearchHit`, `JsMemoryListPage`)
- [x] Removal of unused deps (`esbuild`, `rollup`, `vite-plugin-wasm`, `vite-plugin-top-level-await`)
- **Files modified:**
- `vantadb-wasm/src/lib.rs` — `memory_record_to_js`, `search_hit_to_js`, `put`/`get`/`put_batch`/`list`/`search`/`search_vector` refactored to manual JsValue
- `src/storage.rs` — `read_header` return type: `Option<&DiskNodeHeader>` → `Option<DiskNodeHeader>`, 3 `.cloned()` removed, 1 `*h` → `h`
- `vantadb-ts/package.json` — npm metadata, scripts, devDeps cleaned up
- `vantadb-ts/vantadb.ts` — `searchVector` return type corrected to `{node_id: string; score: number}[]`
- **Files created:**
- `vantadb-ts/README.md` — TypeScript SDK documentation
- `vantadb-ts/test-runner.mjs` — test runner for Node.js ESM
- **Problema raíz diagnosticado:**
  - `StorageEngine::get` retornaba `None` porque `DiskNodeHeader::ref_from_bytes` requiere alineación 64-byte del buffer subyacente, pero el `Vec<u8>` en WASM (heap-allocated) solo garantiza 8-16 bytes de alineación. `read_header(offset=64)` fallaba silenciosamente.
- **Result:** 26/26 integration tests passed. Verified WASM + TypeScript builds.

### TSK-118 — Ejemplos TS: LangChain.js, LlamaIndex.TS, Vercel AI SDK (2026-06-21)

- **Objective:** Create functional examples of integration with the three most used JS/TS frameworks for RAG and agents.
- **Files created:**
  - `vantadb-ts/examples/vercel-ai-memory.mjs` — Vercel AI SDK tool calling + VantaDB as conversational memory
  - `vantadb-ts/examples/langchain-rag.mjs` — LangChain Document pipeline + OpenAIEmbeddings + VantaDB search
  - `vantadb-ts/examples/llamaindex-rag.mjs` — LlamaIndex document indexing + VantaDB vector search
- **Result:** 3 ESM examples with verified syntax. They require `npm install` from the respective SDKs to run.

### CLI-EPIC — Comandos CLI: backup, restore, doctor, inspect, stats, count, search-similar (2026-06-21)

- **Goal:** Expand the VantaDB CLI with 7 new commands for backup, diagnostic and inspection operations.
- **Checklist completado:**
- [x] `vanta-cli backup [--out <path>]` — backup with flush WAL, copy of vector_store + index + WAL, manifest with CRC32
- [x] `vanta-cli restore --in <path> [--force] [--rebuild]` — restore from backup, check CRC32, optionally rebuild indexes
- [x] `vanta-cli doctor` — health diagnostics: WAL, backend, memory, HNSW, indexes, operational metrics
- [x] `vanta-cli inspect --namespace <ns> --key <key>` — inspects a record with all its fields
- [x] `vanta-cli stats [--json]` — database statistics with formatted or JSON output
  - [x] `vanta-cli count --namespace <ns> [--filter key=val]` — conteo de registros
- [x] `vanta-cli search-similar --namespace <ns> --key <key> [--limit <N>]` — similarity search from an existing key
- [x] Fix WAL replay: `recover_state()` now writes `NodeMetadata` to the backend during replay — allows full restore without relying on internal Fjall files
- **Archivos modificados:** `src/cli.rs`, `src/cli_handlers.rs`, `src/bin/vanta-cli.rs`, `src/storage.rs`
- **Archivos creados:** `completions/_vanta-cli`, `completions/_vanta-cli.ps1`, `completions/vanta-cli.bash`, `completions/vanta-cli.fish`
- **Tests:** 46 CLI tests, all pass

<!-- movido a ARCHIVO_HISTORICO.md -->
<!-- movido a ARCHIVO_HISTORICO.md -->
<!-- movido a ARCHIVO_HISTORICO.md -->
<!-- movido a ARCHIVO_HISTORICO.md -->
<!-- movido a ARCHIVO_HISTORICO.md -->
### TSK-120 — Corrección de entorno CI ARM64 (Código de salida 127) (2026-06-22)

- **Goal:** Stabilize the Python Wheels build on `linux-arm64` by resolving the Docker interop bug (`exit code 127`) caused by upgrading `ubuntu-latest` to 24.04.
- **Checklist Completed:**
  - [x] Edit `.github/workflows/python_wheels.yml`
  - [x] Change `runs-on: ubuntu-latest` to `runs-on: ubuntu-22.04` in `build-wheels-arm64`
  - [x] Update `docker/setup-qemu-action` to `@v4`
  - [x] Update `nick-fields/retry` to `@v4`
- **Walkthrough and Changes:** Implemented pinning the runner OS to `ubuntu-22.04` for compatibility with the `maturin-action` QEMU and Docker ecosystem. Likewise, dependencies were updated to modern versions based on Node 20/24 to eliminate security warnings and ensure resilience in the pipeline.
