# Server/infra/security — ENT-04, P13, AUD-020, serie CI

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### 2026-08-02 — ENT-04: Pool de conexiones + circuit breaker (server-mode)

**Objetivo:** Robustez del server-mode bajo carga/concurrencia. Implementar pool de conexiones con límite de concurrencia y circuit breaker con estados closed/open/half-open. (No estaba implementado previamente — solo existía la métrica.)

| ID | Tarea | Resultado | Evidencia |
|----|-------|-----------|-----------|
| `ENT-04` | Connection pooling + circuit breaker | ✅ COMPLETADO | Módulos `src/connection_pool.rs` (pool con cola + semaphore, 4 tests) y `src/circuit_breaker.rs` (estados closed/open/half-open, probe, 5 tests), feature-gated `server` en `src/lib.rs`. `ServerState` en `src/cli_server.rs` construye ambos desde config (`server.pool.*`, `server.breaker.*`); middleware breaker como capa más externa vía `from_fn_with_state`; `execute_query` → `axum::response::Response` (timeout/closed → 503 + header `Retry-After`, pánico → 500). `record_oom()` incrementa umbral no-exit bajo `prometheus`. E2e en `vantadb-server/tests/server.rs`: 503 + `Retry-After: "30"` al abrir, probe half-open que cierra. Config y `.env` documentados en `docs/operations/CONFIGURATION.md`. |

**Verificación:** `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` ✅ | `cargo fmt --check` ✅ | Unit: `-E 'test(circuit_breaker) | test(connection_pool)'` → 9/9 ✅ | E2e: `-E 'test(circuit_breaker)'` → 2/2 ✅ | Workspace `cargo nextest run --profile audit --build-jobs 2 --workspace` → 1802/1802 ✅ | En pasada se corrigió error pre-existente `vantadb-wasm/src/worker.rs:203` (js-sys 0.3.103 `Reflect::apply` exige `&Function`). |

### 2026-08-05 — P13 Audit Report: AUDREP-01, AUDREP-04, AUDIT-03

**Objetivo:** Cerrar 3 hallazgos del audit report 2025-07-27 (P13, verificados contra código 2026-08-05) vía pipeline `/pipeline task AUDREP-01 AUDREP-04 AUDIT-03`.

| ID | Tarea | Resultado | Evidencia |
|----|-------|-----------|-----------|
| `AUDREP-01` | Storage-Panic: `compact_layout` panics sobre vstore truncado | ✅ COMPLETADO (`623e180a`) | Guard de bounds antes del `copy_from_slice` en `src/storage/archive.rs` → devuelve `VantaError::IoError("truncated")` en vez de panic fatal. Test de regresión `test_compact_layout_truncated_vstore_errors_not_panic` (escribe header de 100k vectores en file 4096 B, assert `Err(..truncated..)`). |
| `AUDREP-04` | Storage-Durabilidad: flush tragado + sin sync_all antes de rename | `623e180a` | `tmp_mmap.flush().map_err(VantaError::IoError)?` (antes `let _ =`) + `drop(tmp_mmap); tmp_file.sync_all()?;` antes del `rename` final en `compact_layout`. |
| `AUDIT-03` | Miri guard sobre el CORE Rust (7 bloques UB de INV-024) | `88ed3642` | `MIRIFLAGS=-Zmiri-tree-borrows cargo +nightly miri test -p vantadb` 10/10 limpio; auditó 6 source files (`vfile.rs`, `ops.rs`, `search.rs`, `stats.rs`, `metrics/core/mod.rs`), corregidos `// SAFETY:` comments. Premisa re-escalada: `vantadb-python` (0 dev-deps, cdylib) queda fuera — bound cubierta por AUDIT-04. Commit scoped con `--no-verify` autorizado por gate de fmt pre-existente. |

**Verificación:** `cargo check -p vantadb` ✅ | `cargo nextest -p vantadb --lib compact` → 30/30 ✅ | `cargo clippy -p vantadb --all-targets -- -D warnings` ✅ | Miri 10/10 ✅ | Nota: `cargo clippy --all-targets` reporta 3 lints en `benchmarks/graphrag_bench.rs` — archivo local de campaña MKT-16 (untracked, `[[example]]` sin commitear en Cargo.toml), NO parte de estas tareas; se resuelve cuando esa campaña commitee.

### 2026-08-05 — Limpieza y consolidación `docs/audit-reports/` + `docs/reviews/`

**Objetivo:** Depurar y unificar la documentación de audits/reviews: eliminar duplicados, reclasificar archivos por función real (audit vs investigación vs proceso), y añadir al backlog los hallazgos nuevos verificados en código.

**Resultado:**

| Acción | Detalle |
|--------|---------|
| **Reclasificación de archivos (fs, sin git)** | `vectara-competitive-research` y `meta-001-root-cause-analysis` → `docs/research/` (no son audits: research y RCA de proceso). `backlog-validation`, `progreso-readme-part1/2/3`, `progreso-sistema` → `docs/audit-reports/archive/` (intermedios superados). `audit-reports/` quedó solo con audits legítimos. |
| **Hallazgos nuevos verificados en código** | Bloque `## NV` añadido a `docs/Backlog.md` (Phase 13): `NV-01` 🟠 sq8 panic OOB, `NV-02` 🟡 expects cli_server, `NV-03` 🟡 licencia wasm ausente, `NV-04` 🟠 UB alineación grow_zeroed, `NV-05` 🟢 divergencia deny/audit.toml. |
| **Duplicados cerrados** | `AUDREP-51` tachado (== duplicado de `INV-001`: mismo advisory RUSTSEC-2023-0089). `SEC-01`/`SEC-02` en `backlog-guide.md` tachados como ya resueltos (bincode 2.0, rustls-pemfile v2). |
| **Enlaces actualizados** | Rutas `vectara`/`meta-001` en `Backlog.md` (`META-001`/`NUEVO-21`/`GH-119`) y `progreso/README.md` → `docs/research/`. `docs/reports/INDEX.md` marcó archivados y consolidador. |
| **No duplicados** | `rayon` y `next.config ignoreBuildErrors` verificados en código: ya eran `AUDREP-07`/`AUDREP-19` — correctamente omitidos. |

**Verificación:** Verificación manual en código real de cada candidato nuevo (no solo sub-agentes); 0 links vivos rotos a las rutas movidas (refs en `plans/`, `blog/`, backups y `BACKLOG_HISTORY.md` conservadas como snapshots históricos intencionales). Sin cambios de código — solo documentación.

### 2026-08-08 — P13 Audit Report: NV-02, NV-03, NV-05

**Objetivo:** Cerrar 3 hallazgos restantes del bloque NV del audit report (P13) vía pipeline `/pipeline task NV-02 NV-03 NV-05`.

| ID | Tarea | Resultado | Evidencia |
|----|-------|-----------|-----------|
| `NV-02` | Server-Robustez: `expect`/`unwrap` en `cli_server.rs` | ✅ COMPLETADO | On-disk review: handlers HTTP ya propagaban errores (AUDREP-32); único `expect` restante (GovernorConfig build, línea 159) reemplazado por `match` que loguea y degrada a sin rate-limit. Delegado a vanta-worker (detectó `finish()` → `Option`, usó `Some/None`). `cargo check -p vantadb --features server` + `cargo check -p vantadb` ✅. |
| `NV-03` | Packaging-Licencia: `vantadb-wasm` sin LICENSE | ✅ COMPLETADO | `LICENSE` Apache-2.0 copiado del raíz a `vantadb-wasm/`; hash MD5 idéntico (`0BA1CD7F…`). |
| `NV-05` | Config-Dependencias: divergencia `deny.toml` vs `.cargo/audit.toml` | ✅ COMPLETADO | Ignore `RUSTSEC-2024-0436` (paste) añadido a `deny.toml` con mismo comentario que audit.toml; `cargo deny check advisories` → "advisories ok" (warning "no crate matched advisory criteria" benigno — política unificada). |

**Verificación:** `cargo check -p vantadb --features server` ✅ | `cargo check -p vantadb` ✅ | `cargo deny check advisories` ✅ | `rg "\.expect\(|\.unwrap\(" src/cli_server.rs` → solo tests + constante infalible. Sin `--no-verify`; cambios mínimos (1 fix Rust, 1 archivo nuevo, 1 config).



### AUD-020: Tests HTTP auth/RBAC/rate-limit en vantadb-server
- **Fuente:** Plan 2026-08-09-residual-hardening.md Task 23
- **Fecha:** 2026-08-11
- **Objetivo:** Arreglar 9 tests HTTP rotos por ERR-027 (query inválido → 400) y añadir cobertura de auth/RBAC/rate-limit.
- **Resultado:** ✅ cargo test -p vantadb-server --test server = 19/19 pass (15 originales + 4 RBAC). Commits: `90f85d9f` (tests), `24a15cdf` (fmt drift pre-existente en src/sdk/api.rs).
- **Ids:** `AUD-020`

### CI-04: CodeQL multi-lenguaje (rust + python + javascript-typescript)
- **Fuente:** Plan 2026-08-12-ci-deuda.md Task 3
- **Fecha:** 2026-08-12
- **Objetivo:** Ampliar `sec-codeql-30.yml` de `languages: rust` a `rust, python, javascript-typescript` para cubrir web/ (Next.js) y bindings Python; extender timeout 30→45 min (Risk Register); sin tocar queries (suite default).
- **Resultado:** ✅ actionlint exit 0 (pre-commit hook ok). Commits: `202af1f6` (workflow+task file), `6477aa87` (sync plan+task file). "CodeQL job corre sin error" pendiente de verificación en CI tras push.
- **Ids:** `CI-04`

### CI-03: SBOM multi-ecosistema (rust + npm + python)
- **Fuente:** Plan 2026-08-12-ci-deuda.md Task 2
- **Fecha:** 2026-08-12
- **Objetivo:** Ampliar `release-sbom-64.yml` para generar artifacts SBOM de Rust (cargo-cyclonedx, existente) + npm (`@cyclonedx/cyclonedx-npm`) + Python (`cyclonedx-bom`) y sincronizar docs.
- **Resultado:** ✅ 3 artifacts: `sbom.json` (Rust, intacto), `sbom-web.json` (npm, `--package-lock-only`), `sbom-python.json` (Python, `--pyproject`; root component — `vantadb-python` sin deps declaradas). Docs sincronizadas: `docs/workflow/release-sbom-64.md`, `docs/ci-cd-guide.md`. actionlint exit 0 (2× + pre-commit hook ok). Commit: `a8735174`.
- **Ids:** `CI-03`

### CI-02: Fuzzing en PRs (gate acotado)
- **Fuente:** Plan 2026-08-12-ci-deuda.md Task 1
- **Fecha:** 2026-08-12
- **Objetivo:** Agregar job `fuzz-pr` acotado (5-8 min wall-clock) al workflow `fuzz-40.yml` que corra en `pull_request` sobre persistencia/WAL, con timeout 15 min, ubuntu-only, y `on.pull_request.paths: [src/**, fuzz/**]` para no alargar el Fast Gate; fuzz semanal completo sigue con `if: github.event_name != 'pull_request'`.
- **Resultado:** ✅ actionlint exit 0 (job `fuzz-pr` con `timeout-minutes: 15`, `-max_total_time=75` × 4 targets en matriz, sin `continue-on-error`). Commit: `1c8029f1`.
- **Ids:** `CI-02`

### CI-05: Benchmark baseline fijo
- **Fuente:** Plan 2026-08-12-ci-deuda.md Task 4
- **Fecha:** 2026-08-12
- **Objetivo:** `perf-bench-40.yml` corría sin baseline fijo (solo push a main + workflow_dispatch), sin detectar regresión de 2 commits consecutivos. Agregar comparación contra baseline versionado con umbral de regresión 15% (mediana de 3 runs) y rebaseline manual vía `workflow_dispatch` con `update_baseline=true`.
- **Resultado:** ✅ actionlint exit 0; lógica validada con test sintético 3 caminos (regresión -30% detectada, baseline vacío no-op, baseline igual sin falsos positivos). Baseline inicial `benchmarks/python_baseline.json` vacío → gate no-op hasta rebaseline manual (números v0.5.0 no disponibles — gh auth roto). Commits: `adec84e7`, `56ebc126`, `9026000b`.
- **Ids:** `CI-05`

### CI-06: Tests gate en release workflows
- **Fuente:** Plan 2026-08-12-ci-deuda.md Task 5
- **Fecha:** 2026-08-12
- **Objetivo:** `release-binaries-63.yml` y `release-npm-61.yml` publicaban sin correr tests. Agregar job `tests` a ambos (cargo nextest --profile audit / wasm-pack build + npm test) como `needs` del publish, reusando el patrón del Fast Gate.
- **Resultado:** ✅ actionlint exit 0 en ambos workflows + pre-commit hook ok. Cadena release-npm completa: tests → publish-wasm → publish-ts. Commits: `3ca9e3e0`, `720bb7ab`.
- **Ids:** `CI-06`

### CI-07: SHA pinning de acciones
- **Fuente:** Plan 2026-08-12-ci-deuda.md Task 6
- **Fecha:** 2026-08-12
- **Objetivo:** supply chain hardening — los 17 workflows mezclaban tags y SHAs en 129 `uses:`. Pinear toda acción de terceros a SHA de 40 hex del tag actual (sin major bump silencioso), batch por grupo de workflows.
- **Resultado:** ✅ 67 refs tag/branch → SHA verificados (API GitHub + `git ls-remote`, no memoria) en 16 workflows (+67/−67, comentarios de versión preservados `@sha # v4.4.0`). Grep final: 0 uses de terceros sin 40-hex; 34 internos `./.github/...` correctamente sin pinear. Hallazgo: `release-plz/action@release-plz-v0.3.160` era tag muerto (git ls-remote vacío) → restaurado al tag vigente v0.5.131. actionlint exit 0. Commits: `faec5826`, `73bbf6e1`, `97c21d81`, `b84e4186`, `117c1ac4`.
- **Ids:** `CI-07`

### CI-01 (pre-commit-config): Registrar prettier + ruff + cargo fmt en pre-commit
- **Fuente:** Backlog (fila stale — ya implementado)
- **Fecha:** 2026-08-09 (plan residual-hardening Task 20) / verificado 2026-08-14
- **Objetivo:** `.pre-commit-config.yaml` con los 3 formatters — cargo-fmt (hook local), ruff-check + ruff-format (scoped `vantadb-python/`), prettier (scoped `web/`, rev `v3.1.0`, exclude `.next`/`node_modules`/`package-lock.json`).
- **Resultado:** ✅ config completo + commit `501758a3` "ci: CI-01 pre-commit hooks config (rustfmt, ruff, prettier scoped)". Fila del backlog eliminada al verificar que ya existía. *Nota: no confundir con el CI-01 viejo (workflows GitHub Actions, 2026-07-03, líneas arriba) — ID reutilizado.*
- **Ids:** `CI-01 (pre-commit-config)`

### P2-7: Serialización zero-copy del sparse vector (formato persistido)
- **Fuente:** Backlog § Phase 4 (deuda indexada AGENTS.md Regla 5 + AUDIT-02 2026-08-06)
- **Fecha:** 2026-08-12
- **Objetivo:** Eliminar la serialización full JSON del write path del sparse vector (antes `FieldValue::String(serde_json)` en `SPARSE_VECTOR_EXT_KEY`, `from_str`/`to_string` ~1.49% del hot-path de búsqueda). Redesign del formato de persistencia con compat de lectura backward.
- **Resultado:** ✅ ADR-019: sparse se persiste como `FieldValue::ListFloat(Vec<f64>)` con pares intercalados `[dim, val]` (u32→f64 y f32→f64 lossless, orden determinista por BTreeMap). Write path `sparse_vector_to_field` sin serde_json; read path dual: `ListFloat` decode directo + `String` legacy (`serde_json::from_str`) para compat backward; faltante → `None` (PERF-07); corrupto (odd-length / JSON inválido) → warn + `None`. `VantaMemoryRecord.sparse_vector` público intacto. Sin migración one-shot — nodos viejos migran lazy en próximo put; shim legacy hasta gate de versionado de storage. 7 tests nuevos en serialization + 1 integración en search (recall idéntico). 1885/1885 tests + clippy `-D warnings` + fmt --check ✅. Review P2-01 (vanta-review) APPROVE. Commit `2f1a94e1`.
- **Ids:** `P2-7`

### AUD-025: BM25 zero-alloc hot path (per-posting allocations)
- **Fuente:** `docs/Backlog.md` § Hallazgos pendientes (audit-full-20260812-231204)
- **Fecha:** 2026-08-14
- **Objetivo:** Eliminar alocaciones por posting en el hot path BM25 (token.clone + String record_key por posting; doc_stats_cache re-fetch por token).
- **Resultado:** ✅ `src/text_index.rs:565` `posting_record_key` → `&str` zero-alloc (`strip_prefix` + `from_utf8`); `src/sdk/search/phrase.rs` matcher genericizado (`K: AsRef<str> + Ord`, helper `find_positions`); `src/sdk/search/mod.rs:383-448` sin `token.clone()`/`String::from`/`format!` por posting, `doc_stats_cache` keyed por `u128 node_id` con guard de mismatch. `cargo check -p vantadb` ✅, clippy ✅, fmt ✅, 104 tests (13 phrase + 91 search) ✅. Commit `96b258ba`.
- **Ids:** `AUD-025`

### AUD-026: Dropped cli/arrow/tantivy from native DLL default features
- **Fuente:** `docs/Backlog.md` § Hallazgos pendientes (audit-full-20260812-231204)
- **Fecha:** 2026-08-14
- **Objetivo:** `vantadb-node` era el único cdylib que arrastraba cli/arrow/tantivy (6.7MiB debug) vía default features del crate raíz.
- **Resultado:** ✅ fix 1 línea `vantadb-node/Cargo.toml:24` — `vantadb = { path = "..", default-features = false, features = ["fjall", "memmap2", "rayon"] }` (python/wasm ya resuelto). `cargo check --manifest-path vantadb-node/Cargo.toml` ✅, `cargo tree -e features` sin tantivy/arrow/clap/indicatif. Commit `404f1625`.
- **Ids:** `AUD-026`

### AUD-027: Least-privilege per-job permissions in release workflow
- **Fuente:** `docs/Backlog.md` § Hallazgos pendientes (audit-full-20260812-231204)
- **Fecha:** 2026-08-14
- **Objetivo:** Validar el jump release-plz v0.3→v0.5.131 (cambio de comportamiento en permisos/inputs) antes del primer release real post-jump.
- **Resultado:** ✅ hallazgo refinado: pin `release-plz/action@2eb1d8bcb7 # v0.5.131` era CORRECTO (tag del action vs CLI 0.3.160); cambio real = permisos movidos de workflow-level a por-job (release: `contents: write, pull-requests: read, id-token: write`; PR: `contents: write, pull-requests: write`); Trusted Publishing intacto, sin `CARGO_REGISTRY_TOKEN`; `release-plz.toml` sin cambios. `yaml.safe_load` OK + actionlint exit 0. Commit `d66b267d`.
- **Ids:** `AUD-027`

### AUD-029: Re-correr contrato CI-05 desde la raíz (verify-log alineado)
- **Fuente:** `docs/Backlog.md` § Hallazgos pendientes (audit-full-20260812-231204)
- **Fecha:** 2026-08-14
- **Objetivo:** `verify-log.jsonl` registraba CI-05 con `passed:false, exitCode:-1` (harness/cwd/quoting en la sesión de audit) pese a artifact `benchmarks/python_baseline.json` válido.
- **Resultado:** ✅ re-ejecución del contrato desde la raíz del workspace: `python -c "import json; json.load(open('benchmarks/python_baseline.json')); print('OK')"` → `OK` exit 0, entry nueva en verify-log con `passed:true` (2026-08-14T05:47:47Z). Entrada vieja intacta (append-only); sin cambios de código ni harness. Fila del backlog eliminada.
- **Ids:** `AUD-029`

### AUD-032: Split del monolito vantadb-mcp en 12 módulos
- **Fuente:** `docs/Backlog.md` § Hallazgos pendientes (audit-full-20260812-231204)
- **Fecha:** 2026-08-14
- **Objetivo:** `vantadb-mcp` 1607L en 1 archivo con solo 2 tests — peor ratio tests/líneas del workspace.
- **Resultado:** ✅ `src/lib.rs` → facade (`#![warn(missing_docs)]`, 8 mods, 10 `pub use` documentados) + 12 módulos: `{config,axioms,error,protocol,metrics,validation,server}.rs` + `handlers/{initialize,resources,prompts,tools}.rs`; slicing 1:1 (internals `pub(crate)`, `#[allow(deprecated)]` ×3 preservado); tests migrados; `tests/version_coherence.rs:97` → `vantadb-mcp/src/handlers/initialize.rs`; review P2-01 approve. Nextest coherence 1/1 + clippy `-D warnings` exit 0. Commit `1099bfe4`.
- **Ids:** `AUD-032`

### AUD-033: validación de args CLI + suite de tests en vantadb-server
- **Fuente:** `docs/Backlog.md` § Hallazgos pendientes (audit-full-20260812-231204)
- **Fecha:** 2026-08-14
- **Objetivo:** `vantadb-server` sin tests (0 #[test]) + arg-scan manual ignora flags desconocidos.
- **Resultado:** ✅ `main.rs`: `is_known_flag` (-h/--help/--mcp) + `validate_args`; flag desconocido → `eprintln!("error: unrecognized argument ...")` + hint + `exit(2)`; help precedence intacta. `tests/cli_args.rs` (nuevo, 5 tests, proceso vía `CARGO_BIN_EXE` + `output_with_timeout`). Nextest 5/5. Commit `ef0dfc5c`.
- **Ids:** `AUD-033`

### AUD-038: remover `#![allow(unused_unsafe)]` obsoleto
- **Fuente:** `docs/Backlog.md` § Hallazgos pendientes (audit-full-20260812-231204)
- **Fecha:** 2026-08-16
- **Objetivo:** el allow a nivel crate enmascara unsafe muerto.
- **Resultado:** ✅ Removido `#![allow(unused_unsafe)]` de `src/lib.rs:3`. Hallazgo: los 22 usos de `unsafe` del crate son todos genuinos (mmap vfile_mmap.rs, `from_raw_parts`, `mem::zeroed` PSAPI, `unsafe impl Send/Sync` vfile.rs/accumulator.rs, bloques FFI metrics/archive/maintenance/graph) — el allow era config obsoleta, no enmascarador. 0 warnings unused_unsafe tras removerlo. Check/clippy -D warnings/nextest 1886/fmt exit 0. Commit `1e610225`.
- **Ids:** `AUD-038`

### AUD-040: batch_append WAL sin alocar por record
- **Fuente:** `docs/Backlog.md` § Hallazgos pendientes (audit-full-20260812-231204)
- **Fecha:** 2026-08-16
- **Objetivo:** `batch_append` alocaba Vec por record (`to_allocvec`).
- **Resultado:** ✅ `src/wal.rs`: `to_allocvec` (1 alloc/record) → Vec reutilizable `with_capacity(128)` + `postcard::to_io` (2 allocs/batch, clear conserva capacidad); framing `[len u32 LE][payload][crc u32 LE]` byte-idéntico + test regresión `test_batch_append_byte_format_matches_append` (bytes = append secuencial + replay limpio). `Cargo.toml`: postcard features `alloc` → `alloc,use-std` (aditiva). Nota: `serialize_into` no existe en postcard 1.1.3 (API std es `to_io`); diseño con sub-slice CRC abortaba por UB-check crc32c (`util::split` exige alineación 8) → Vec propio evita el landmine. Nextest 1887. Commits `a5001f4d`.
- **Ids:** `AUD-040`

### AUD-041: bench sparse_hot_path — arms ListFloat (P2-7)
- **Fuente:** `docs/Backlog.md` § Hallazgos pendientes (audit-full-20260812-231204)
- **Fecha:** 2026-08-16
- **Objetivo:** el bench solo medía el path viejo (serde_json).
- **Resultado:** ✅ `benches/sparse_hot_path.rs` (+42L): 2 arms nuevos en grupo `sparse/hot-path` — `listfloat_encode_one` (SparseVector → interleaved Vec<f64>) y `listfloat_decode_one` (→ BTreeMap validado), mirrors inline de `sparse_vector_to_field/from_field` (helpers privados a sdk::serialization, mismo patrón que arms serde_json). Arms serde_json intactos para comparación critcmp. Check --benches/clippy/test --no-run/fmt exit 0. Commit `ec4eaeff`.
- **Ids:** `AUD-041`

### AUD-034: dedupe transacción IDB en helper único
- **Fuente:** `docs/Backlog.md` § Hallazgos pendientes (audit-full-20260812-231204)
- **Fecha:** 2026-08-16
- **Objetivo:** transacción IndexedDB duplicada ×4 (write/del × lock/no-lock).
- **Resultado:** ✅ `vantadb-wasm/src/idb.rs`: los 4 bloques IDBTransaction duplicados → helper único `runWriteTx(db, key, op, resolve, reject)` + 2 call sites de 1 línea; diff +15/−32 (−37.8%). Lock `navigator.locks.request("vantadb-write")` preservado (resolveTx en tx.oncomplete — necesario o la siguiente escritura deadlockea); notify `channel.postMessage` idéntico; errores lock/no-lock mismos observables. API pública Rust (`IdbStorage::write_file`/`delete_file`, 13+20 callers) intacta. Check/clippy/nextest 1/fmt exit 0. Commit `b255f982`.
- **Ids:** `AUD-034`

### AUD-037: error explícito de backend + unificar new()/connect() en Python
- **Fuente:** `docs/Backlog.md` § Hallazgos pendientes (audit-full-20260812-231204)
- **Fecha:** 2026-08-16
- **Objetivo:** fallback silencioso a Fjall con backend desconocido + duplicación new()/connect().
- **Resultado:** ✅ `vantadb-python/src/lib.rs`: `parse_backend_kind(Option<&str>) -> PyResult<BackendKind>` + `open_vantadb(...)` — `None`→fjall, `"rocksdb"`, `"memory"`, otro→`ValueError` descriptivo (antes: `tracing::warn!` + fallback silencioso). `new()` y `connect()` delegan en `open_vantadb` (connect normaliza `""`/`":memory:"` + `py.detach` libera GIL durante open). Docstrings actualizados ("raise ValueError"). pytest 89 passed, 4 deselected (maturin develop); smoke: `backend='bogus'` → `ValueError: Unknown backend "bogus"`. Hallazgo colateral: `VantaDB(':memory:')` con default lanza OSError en Windows (Fjall trata `":memory:"` como dir) — pre-existente, documentado. Commit `47153977`.
- **Ids:** `AUD-037`

### AUD-043: collect_all_deduped — dedup con u128 node-ids (P2-8)
- **Fuente:** `docs/Backlog.md` — Hallazgos pendientes (audit-full-20260812-231204)
- **Fecha:** 2026-08-16
- **Objetivo:** `collect_all_deduped` O(n) dedup con `HashSet<(String,String)>` — 2 alocaciones de String por record.
- **Resultado:** ✅ `vantadb-wasm/src/lib.rs:556`: `HashSet<(String,String)>` → `HashSet<u128>` por `record.node_id` (XxHash3_128 sobre `namespace\0key`, determinístico 1:1 verificado en `src/sdk/serialization/mod.rs:54-60` con test `test_memory_node_id_deterministic`; 19 callers). Cero alocación por record (u128 Copy). Paginación/MAX_RECORDS intacta. + test `test_collect_all_deduped_no_duplicates` (`#[wasm_bindgen_test]`, 3 records + overwrite → sin duplicados). Check/clippy/fmt exit 0; 7 warnings pre-existentes del core (`vfile_*.rs`, fuera de scope). Commit `9dcbff5a`.
- **Ids:** `AUD-043`
