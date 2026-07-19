# Audit Report: full
**Date:** 2026-07-18T17:19:45-04:00
**Mode:** full

## Summary
- Phases completed: 8/8
- Blocking issues: 10 Critical (4 Security + 5 Performance + 3 Code Review - overlaps)
- Recommendations: 15+
- Veredict: ❌ FAIL — 10 Critical findings, Phase 8 certify blocked by clippy warnings

## Per-Phase Results

| Phase | Status | Wave | Details |
|-------|--------|------|---------|
| 0. Pre-check | ✅ | direct | `main`, 2 unstaged files |
| 1. CLI | ⚠️ | direct | fmt ✅ deny ✅ audit ✅ machete ✅ bloat ✅ — clippy ❌ (wasm Promise) |
| 2. Security | ✅ | sub-agent | 4 Critical, 6 Important |
| 3. Performance | ✅ | sub-agent | 5 Critical, 7 Important |
| 4. Code Review | ✅ | sub-agent | 3 Critical, 2 Important |
| 5. Root Cause | ✅ | sub-agent | wasm Promise import — fix <1 min |
| 6. Deep Module | ✅ | sub-agent | core 8/10, python 6/10, mcp 7/10, wasm 7/10 |
| 7. Full ISO | ✅ | sub-agent | FAIL — Reliability 5/10, Efficiency 4/10 |
| 8. Certify | ❌ | sub-agent | FAIL — clippy warnings on core + python |

## Findings by Phase

### Phase 2: Security
#### Critical
- `vantadb-wasm/src/lib.rs:562-626` — js `Reflect::set().unwrap()` panics in WASM FFI bridge (19 calls)
- `src/node.rs:254,275` — `SendPtr` + `from_raw_parts` mmap lifetime gap (UB on remap)
- `deny.toml:2-16` — 3 advisories accepted without expiration or review dates
- `vantadb-wasm/src/lib.rs:562-626` — OOM via WASM: untrusted JS input not bounded

#### Important
- `vantadb-mcp/src/lib.rs:912` — `unwrap_or("")` swallows malformed tool calls
- `src/python.rs:19-23` — `ClientEngine::default()` panics on storage open (PyO3 abort)
- `src/index/serialize.rs:528,598` — `MmapMut::map_mut` unsound on short file writes
- `src/storage/archive.rs:74,104` — MmapMut remap without fsync
- `src/storage/vfile.rs:227,249,261` — `mincore` unaligned address risk
- `src/storage/engine/ops.rs:553,672,994` — f32 re-interpret cast without alignment check

### Phase 3: Performance
#### Critical
- `src/wal.rs:318` — `postcard::to_allocvec` allocates per WAL append (N+1 allocations)
- `src/backends/fjall_backend.rs:197-209` — `scan_prefix()` materializes full Vec<(Vec<u8>, Vec<u8>)>
- `src/sdk/search.rs:246-250` — `lexical_search()` HashMap alloc per scored posting
- `src/sdk/serialization/mod.rs:228-248` — `memory_record_from_node()` 2-pass BTreeMap
- `src/index/search.rs:24-28` — `search_layer()` HashSet per HNSW layer (no reuse)

#### Important
- `src/sdk/serialization/mod.rs:307` — `metadata.clone()` after `take()`
- `src/planner.rs:120` — `fuse_rrf()` clones 2 Strings per entry (use node_id)
- `src/index/search.rs:251-255` — `select_neighbors()` stores full Vec<f32> per candidate
- `src/wal_sharded.rs:85-89` — `batch_append()` clones WalRecord per shard
- `src/wal.rs:348` — buffer estimate 128 bytes under-guesses for vectors
- `src/backends/fjall_backend.rs:128-131` — `get()` always `.to_vec()` (no zero-copy)
- `vantadb-python/src/lib.rs:320-356` — `node_to_pydict()` allocates 10+ Python objects per node

### Phase 4: Code Review
#### Critical
- `vantadb-python/src/lib.rs:1766-1767` — UB fix: `VantaVector` changed to `Box<[f32]>` ✅
- `vantadb-python/src/lib.rs:833` — `put_batch` positional API deprecated ✅
- `vantadb-wasm/src/opfs.rs:83-86` — `OpfsFile::delete` now works ✅

#### Important
- `vantadb-wasm/src/worker.rs:171-173` — `is_retryable` substring match on error string
- `vantadb-wasm/src/worker.rs:199-209` — `setTimeout` via eval-constructed string

### Phase 5: Root Cause
- **Clippy wasm:** `Promise` missing from `use js_sys::{Array, Promise, Reflect};` on line 20
- Fix applied ✅ — wasm now compiles

### Phase 6: Deep Module Review
| Module | Score | Top Issue |
|--------|-------|-----------|
| vantadb core | 8/10 | High `expect`/`unwrap` count, files >1000L |
| vantadb-python | 6/10 | Only 3 test files, `connect()` hardcodes path |
| vantadb-mcp | 7/10 | Single ~2000L file, hardcoded AXIOMS |
| vantadb-wasm | 7/10 | `unwrap()` in JS interop, `collect_all_deduped` unbounded |

### Phase 7: Full ISO 25010

| Dimension | Score | Key Issues |
|-----------|-------|------------|
| Functional Suitability | 7/10 | Core contract holds, WASM compilation blocked |
| Reliability | 5/10 | MmapFull UB, OOM via WASM, js panics |
| Usability | 6/10 | Conventional API, no usability audit |
| Efficiency | 4/10 | 5 Critical perf issues in hot paths |
| Maintainability | 6/10 | Proactive UB fix, but clippy errors persist |
| Portability | 6/10 | WASM now fixed, Rust/Python/TS present |

### Phase 8: Certification
#### Tests
- `cargo nextest run --profile audit -p vantadb`: **546 passed, 0 failed** ✅
- `cargo fmt --check`: PASS ✅

#### Blockers
- `cargo clippy -D warnings` on `vantadb` core: 9 unnecessary `unsafe` blocks (memmap2 safe), 4 dead methods
- `cargo clippy -D warnings` on `vantadb_py`: 1 deprecation warning for `put_batch`
- `deny.toml`: 2 stale advisory ignores (`RUSTSEC-2024-0436`, `RUSTSEC-2025-0134`)

**Verdict: ❌ FAIL**

## Scoreboard

| Category | Score (0-10) | Notes |
|----------|-------------|-------|
| Code Quality | 7 | Clippy warnings blocking certify |
| Security | 6 | 4 Critical unresolved |
| Performance | 4 | 5 Critical in hot paths |
| Architecture | 8 | Clean module boundaries |
| Tests | 9 | 546/546 passing |
| Docs | 6 | Missing API docs |

## FODA

| Dimensión | Hallazgos |
|-----------|-----------|
| **Fortalezas** | 546 tests pasan, 0 fallos. Arquitectura limpia. UB fix proactivo. CLI checks sólidos (fmt, deny, audit, machete, bloat). |
| **Oportunidades** | WAL allocation pooling, scan_prefix iterator, HNSW HashSet reuse. Podrían dar 20-40% mejora en search latency. |
| **Debilidades** | 10 Critical sin resolver. Clippy warnings bloquean certify. Reliability 5/10 — MmapFull UB y OOM paths. |
| **Amenazas** | 3 dependencias unmaintained sin revisión (atomic-polyfill, paste, rustls-pemfile). Wildcards permitidos en deny.toml. |

## Veredicto
❌ FAIL — El pipeline completo identificó 10 hallazgos Critical (4 Security + 5 Performance + 1 Code Review structural) y Phase 8 certify falló por clippy warnings en core y python. La estabilización requiere:
1. Remover `unsafe` innecesario en mmap wrappers (9 bloques)
2. Migrar `put_batch` deprecation o agregar `#[allow(deprecated)]`
3. Limpiar `deny.toml` stale advisories
4. Abordar 4 Security Criticals y 5 Performance Criticals
5. Eliminar 4 métodos dead code
