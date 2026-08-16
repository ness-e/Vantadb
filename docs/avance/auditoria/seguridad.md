---
title: "Auditoría — Seguridad"
type: audit-log
status: active
tags: [vantadb, avance, security, audit, fuzz, miri, ffi]
last_reviewed: 2026-08-07
aliases: []
---

# Auditoría — Seguridad

> Registro consolidado de hallazgos de seguridad, auditorías de código (AUD), fuzzing, Miri, FFI/unsafe. IDs originales conservados.

## Auditoría masiva 2026-06-19 (AUD-01..44) — resumen por severidad

- 🔴 **8 críticos** — TODOS resueltos (ver tabla codefix)
- 🟡 **14 medios** — todos abordados, 1 parcial
- 🟢 **22 bajos/sugerencia** — la mayoría aplicados o documentados
- 🔵 2 no aplicables (WASM-only paths sin efectos en server)

> Detalle completo por hallazgo: `docs/historial/autopsias-2026-06-19.md`

### Hallazgos críticos resueltos (AUD)
| ID | Hallazgo | Fix |
|---|---|---|
| AUD-01 | Panic path no atrapado (unwrap en archivar reader) | `handle_panic` + tests |
| AUD-02 | Chain de retry WAL sin supresión de duplicados | dedup vía `seen` |
| AUD-03 | Sink context channel full → block | bounded channel + async |
| AUD-04 | Cast unsafe sin check de alineación (`rkyv_archives.rs:54-71`) | fix alineación |
| AUD-05 | `.ok()` silencia errores UTF-8 | `map_err` + LogError |
| AUD-06 | N+1 query en `scan_nodes()` | batch lookup |
| AUD-07 | `ensure_indexes_current` 3 scans → 1 | unify |
| AUD-08 | `memory_record_to_node_owned` clones | reduce clones |

### Resueltos medios (AUD-10..)
| ID | Hallazgo | Fix |
|---|---|---|
| AUD-10 | `mapped_file_resident_bytes()` removida | delete function |
| AUD-11 | `wal_path` asignado pero nunca leído | remove field/param |
| AUD-12 | `HybridSearchResult` `#[allow(dead_code)]` | remove promote |
| AUD-13 | `cfg(feature = "encryption")` no activo en bench | feature |
| AUD-14 | `memory_usage()` mut → const | const fn |
| AUD-15 | Deadlock `Mutex` 2 locks — pattern `lock(L1)->lock(L2)` | reorder |
| AUD-18 | manual elided lifetime | fix |
| AUD-19 | test feature-gated | cfg gating |
| AUD-20 | .gitignore legal the Fixtures | cleanup |
| AUD-21 | dead `table` module | remove |
| AUD-22 | `spin` dependency + `Mutex` manual | replace with usefrom |
| AUD-23 | `web` demo commit + `ai-string-size` | allow |
| AUD-26/27/28/29/31 | minor cleanup | done |
| AUD-30 | unused `open` in FFI | remove |
| AUD-32 | obsolete tmdb | update |
| AUD-33 | framework repo: meta | done |

### AUD-09
- `delete_by_filter`, `count`, `similar_to_key` — despect given CLI-only (removed de public SDK). Ver `2026-07-28-sdk-gap-audit.md`.

### AUD-24/25
- AUD-24: coordenadas × 3 SI (loop unroll). ✅
- AUD-25: dead cli flags → cleanup.

### AUD-010? nota — `docs/historial/autopsias-2026-06-19.md`

### AUD-020: Tests HTTP auth/RBAC/rate-limit — ✅ 2026-08-11
- `cargo test -p vantadb-server --test server` → 19/19; root cause: helpers mandaban `{"query":"test"}`/`SELECT 1` (IQL inválido → 400 correcto post-ERR-027); fix: `SELECT * FROM Node`. RBAC vía `token_role_map` ya conectado. Backlog row removido; registrado en progreso.

---

## Auditoría de bindings (IA)

- **VFY-002:** CDV->VitaBuf — objeto CODeXCE (Jobs) deprecado; fix definitivo del vm NO completado. Ver `historial/autopsias-2026-06-19.md`.

## SEC-01 / SEC-02: Security audit FFI/bindings (P8-06)
- **Fecha:** 2026-07-28
- **Resultado:** ✅ SEC-01 (auditoría de FFI del core con PyO3/WASM) y SEC-02 (supply chain: dependencias con `unsafe` no auditadas) pasan gate; fixes aplicados en batch CODE-036/056/057/058/061/062/063.

### Batch SEC (detalle)
| ID | Tarea | Commit |
|---|---|---|
| CODE-036 | TLS 1.3 only (relajado a 1.2) | `df1479a` |
| CODE-056 | Duplicate reqwest 0.12+0.13 | `df1479a` |
| CODE-057 | debug=0 en test profile | `df1479a` |
| CODE-058 | Ignored advisories sin rationale | `df1479a` |
| CODE-061 | SIGBUS handler no signal-safe | `df1479a` |
| CODE-062 | Cursor reset sin zero-fill | `df1479a` |
| CODE-063 | grow_to puede shrink | `df1479a` |

---

## UnSafe & FFI review

### INV-024: Auditoría de unsafe (2026-06-19)
- **Fecha:** 2026-06-19
- **Resultado:** ✅ Audit MISRA-like de `unsafe` blocks. Hallazgos en `docs/historial/autopsias-2026-06-19.md`.

### Fuzz (DRV-133)
- **Resultado:** Fuzz de WAL/Disk persistence (chaos) — estado en Backlog fase chaos.

### AUDIT-01: Fix UAF PyO3 `__array_interface__` (release-blocker)
- **Fecha:** 2026-08-05
- **Resultado:** ✅ Fix de use-after-free en `__array_interface__` Python (release-blocker). Detalle en snapshot-2026-08-07.

### AUDIT-02: Sparse hot-path micro-opt (gate de medición) — WONTFIX
- **Fecha:** 2026-08-06
- **Resultado:** ✅ Declarado WONTFIX — gate de medición. Ver `decisiones/wontfix.md`.

### P13 Audit Report: AUDREP-01, AUDREP-04, AUDIT-03
- **Fecha:** 2026-08-05
- **Resultado:** ✅ Reporte de auditoría (Miri/correctness). Detalle en snapshot-2026-08-07.

### AUD-020: Tests HTTP auth/RBAC/rate-limit en vantadb-server
- **Fecha:** 2026-08-11
- **Resultado:** ✅ 9 tests rotos por ERR-027 arreglados (query inválido → 400; fix: `SELECT * FROM Node`) + 4 tests RBAC HTTP nuevos (reader 403, writer/admin 200, reader GET /metrics 200). `cargo test -p vantadb-server --test server` = 19/19. Commits `90f85d9f`, `24a15cdf` (fmt drift).

### AUD-031: Panic-hardening engine embebido (unwrap/expect alcanzables)
- **Fecha:** 2026-08-13
- **Resultado:** ✅ 5/5 unwraps `active.iter().next().unwrap()` en `src/storage/engine/ops.rs` convertidos a propagación de error (`ok_or_else` en insert/get/delete; `if let Some` en helpers con comentario de decisión). `parser/mod.rs` no-test = 0 unwraps (151 en tests). No se tocaron los 1381−5 restantes (tests/benches/paths internos ya hardened). Contrato ✅: check, nextest 1885 passed, clippy all-targets/all-features, fmt. Review vanta-review approve post-fix. Commit `c7185d25`.

### AUD-023: Validar dims de sparse vector en decode (P2-7)
- **Fecha:** 2026-08-13
- **Resultado:** ✅ `sparse_vector_from_field` valida dims (`is_finite`, >= 0, <= u32::MAX, entera) y devuelve `None` en vez de saturar silencioso via `as u32`. Test de rechazo TDD (NaN/+inf/negativa/out-of-range/no-entera). Warning corrupto actualizado. Contrato ✅: check, fmt, clippy workspace -D warnings, nextest 1913 passed, validate-docs-coverage 0 gaps. Commit `(AUD-023)`.

### AUD-024: Eliminar heap clones por op en drain_hnsw_batch_locked
- **Fecha:** 2026-08-13
- **Resultado:** ✅ Refactor de ownership en `src/storage/engine/ops.rs`: `for op in ops` (consumir la Vec tomada vía `mem::take`) + pasar `op.bitset`/`op.vector` por valor (HnswGraph::add ya los toma por valor) → 0 heap clones por insert en el drain. Mismo fix en `try_push_pending_hnsw` (drain opportunista). Perf bench_concurrent 10k inserts: 178.11s → 137.95s (-22.5%). Contrato ✅: check, fmt, clippy -D warnings, nextest 1913 passed, docs-coverage 0 gaps. Commit `e4c2ff8e`.

### AUD-039: LRU eviction O(1) con crate `lru` en python bindings (P2-3)
- **Fecha:** 2026-08-13
- **Resultado:** ✅ Reemplazo del LRU hand-rolled en `vantadb-python/src/convert.rs` (evicción O(n) `min_by_key`) por `lru::LruCache` (O(1), hash + lista doble); `const CACHE_CAPACITY: NonZeroUsize = 64`; call sites `.cloned()` y `let _ = put(...)`. Dep `lru = "0.16"` en vantadb-python (ya resuelta 0.16.4 en lockfile por el core). Colateral: fix test_load.py (query vectors non-zero; core rechaza zero-norm desde ERR-028). Perf: O(1) vs O(64), microbench 78-80 ops/s sin regresión. Contrato ✅: check -p vantadb_py, fmt, clippy -D warnings, nextest 1913 passed, pytest 85 passed, docs-coverage 0 gaps. Commit `af905c65`.

### AUD-022: Pin SHA sccache-action (supply-chain CI)
- **Fecha:** 2026-08-13
- **Resultado:** ✅ `.github/actions/rust-setup/action.yml:73` — única acción externa sin pin SHA: `mozilla-actions/sccache-action@v0.0.11` → `@fd02668681acd5f960e1372061bee5e3e987195c # v0.0.11` (SHA verificada vía GitHub API 2026-08-13). Anotación alineada a AUD-028. YAML OK. Commit `(AUD-022)`.

### AUD-030: Gate de regresión bench en PRs + baseline auto-commiteado
- **Fecha:** 2026-08-13
- **Resultado:** ✅ `heavy-bench-nightly-51.yml`: (1) trigger `pull_request` con paths filter (benches/**, benchmarks/**, scripts/bench_regression.py, Cargo.toml) — el gate corre en PRs que tocan el sistema de bench sin que el resto pague 2hrs; (2) step "Update and commit baseline (nightly only)" en analyze — `update-baseline` + commit/push, solo en schedule y solo si no hay regresión (`has_regression != 'True'`); `permissions.contents: write`. El modo `update-baseline` de bench_regression.py ya existía pero no tenía caller → baseline nunca se promovía. YAML OK. Commit `(AUD-030)`.

### AUD-028: Anotar 78 SHA pins con versión (# vX.Y.Z) en GitHub Actions
- **Fecha:** 2026-08-13
- **Resultado:** ✅ 78 líneas `uses: repo@sha` sin anotar (74 del audit + 4 `pypa/gh-action-pypi-publish` omitidas del primer map de edición) → `# vX.Y.Z` en 11 archivos de `.github/**`. Versiones resueltas contra tags reales upstream (`git ls-remote --tags` para tags exactos; `git clone --filter=blob:none` + `git describe --tags` para commits intermedios: rust-cache v2.9.1, install-action v2.83.4, attest-build-provenance v4.1.1). dtolnay/rust-toolchain → `# v1` (único tag del repo). SHAs intactos (diff aditivo). Grep pins sin anotar = 0; actionlint 10/10; YAML 23/23. Review vanta-audit approve (16/16 correspondencias verificadas independientemente). Commit `8e9f5eb1`.

### AUD-035: Split megafiles core (patrón REVIEW-05)
- **Fecha:** 2026-08-16
- **Resultado:** ✅ 3 splits. **Split 1** `src/sdk/search/mod.rs` 2521L → 8 submódulos (`lexical.rs` 225L, `vector.rs` 216L, `sparse.rs` 74L, `hybrid.rs` 47L, `explain.rs` 196L, `audit.rs` 52L, `debug_ops.rs` 380L, `multi.rs` 89L) + `tests.rs` (53 tests), mod.rs orquestador 330L. Commit `5d96b536`. **Split 2** `src/storage/engine/ops.rs` 2131L → orquestador 331L + `delete.rs/get.rs/insert.rs/txn.rs` (mod.rs cableado + doc-comments). **Split 3** `src/index/search.rs` 2054L → `search/mod.rs` 52L (orquestador + `impl VecIndex for CPIndex` verbatim) + `pool.rs` 16L + `profile.rs` 80L + `layer.rs` 393L + `neighbors.rs` 63L + `nearest.rs` 163L + `alternate.rs` 116L + `tests.rs` 1379L. Signaturas públicas intactas (`search_nearest` pub, `search_layer`/`select_neighbors`/`search_ivf`/`search_scann`/`search_with_method` pub(crate), `BatchInsertOptions`/`InsertMode` re-exportados); visibilidad `pub(crate)`/`pub(super)` mínima. Contrato ✅: check, clippy -D warnings, fmt, nextest 1886 passed (0 failures, 1 skipped). Commits `5d96b536` + `552f08a8`.

---

## Prevención de breaking changes

- `cargo semver-checks` como gate pre-publish obligatorio (vanta-lead).

## Fuentes
- `docs/Backlog.md` fases security/audit.
- `docs/historial/autopsias-2026-06-19.md` (AUD-01..44).