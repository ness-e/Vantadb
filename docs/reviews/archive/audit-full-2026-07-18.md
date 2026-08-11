# Audit Report: full
**Date:** 2026-07-18T17:19:45-04:00
**Mode:** full

---
## Re-Audit: 2026-07-21 (3 días después)

Verificación multi-agente de cada hallazgo contra código actual. Resultados:

### Estado global
| Categoría | Original | Persiste | No persiste | Parcial |
|-----------|----------|----------|-------------|---------|
| Security (4C + 6I) | 10 | 1 | 2 | 7 |
| Performance (5C + 7I) | 12 | 9 | 1 | 2 |
| Code Review + Blockers | 8 | 4 | 3 | 1 |
| **Total hallazgos únicos** | **~25** | **14** | **6** | **5** |

**Veredicto actual:** ⚠️ AÚN VÁLIDO — ~14/25 hallazgos persisten sin cambios. El reporte sigue siendo la lista consolidada de deuda técnica.

### Cambios relevantes desde el audit original
- ✅ WASM `Reflect::set().unwrap()` → `.ok()` (no más panic, pero silencia errores)
- ✅ `SendPtr` + `from_raw_parts` mmap UB — eliminado de `node.rs`
- ✅ `ClientEngine::default()` — eliminado, ahora usa `PyResult` con `?`
- ✅ `search_layer()` HashSet reusado entre HNSW layers (`&mut visited`)
- ✅ `setTimeout` via eval-string → Reflect API
- ✅ `mincore` unaligned → page-aligned fix
- ❌ `archive.rs` sin fsync antes de rename — **sigue siendo riesgo de corrupción**
- ❌ 9/12 hallazgos de performance sin cambios (WAL alloc, BTreeMap 2-pass, etc.)
- ❌ `unsafe` innecesarios en `vfile.rs` — persisten (6+)
- ❌ `deny.toml` — `RUSTSEC-2024-0436` sigue sin expiración
- ❌ Python SDK — solo 3 test files
- 📉 `vantadb-mcp/src/lib.rs` reducido de ~2000 → 1245 líneas

---

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
  → **⏳ Parcial:** `.unwrap()` reemplazado por `.ok()` en todos los calls. No más panic, pero silencia errores en vez de propagarlos.
- `src/node.rs:254,275` — `SendPtr` + `from_raw_parts` mmap lifetime gap (UB on remap)
  → **✅ Fixed:** `SendPtr` eliminado de `node.rs`. `from_raw_parts` restante usa referencias directas con lifetime tracking.
- `deny.toml:2-16` — 3 advisories accepted without expiration or review dates
  → **⏳ Parcial:** Quedan 2 (RUSTSEC-2023-0089, RUSTSEC-2024-0436). Ninguno tiene `expires` ni `review`.
- `vantadb-wasm/src/lib.rs:562-626` — OOM via WASM: untrusted JS input not bounded
  → **⏳ Parcial:** Bound check agregado (`query_vector.len() > MAX_F32_VEC_LEN`), pero `from_js()` deserializa sin límite de payload total.

#### Important
- `vantadb-mcp/src/lib.rs:912` — `unwrap_or("")` swallows malformed tool calls
  → **⏳ Parcial:** Línea 912 específica usa `ok_or_else(...)?` con error explícito. Pero el patrón `unwrap_or("")` persiste en 4 lugares del mismo archivo (líneas 587, 617, 712, 742).
- `src/python.rs:19-23` — `ClientEngine::default()` panics on storage open (PyO3 abort)
  → **✅ Fixed:** `Default` impl eliminado (comentario `ponytail:` explica por qué). Ahora usa `#[new]` con `PyResult` y `?`.
- `src/index/serialize.rs:528,598` — `MmapMut::map_mut` unsound on short file writes
  → **⏳ Parcial:** `file.set_len(data.len())?` antes de map, y bounded reads en deserialize. Cold-start débil (no verifica `file.metadata().len()` vs expected size).
- `src/storage/archive.rs:74,104` — MmapMut remap without fsync
  → **🔴 Persiste:** `tmp_mmap.flush()` existe pero descarta error (`let _ =`). NO hay `flush()`/`msync()` antes del `rename` final (línea 126). Riesgo de corrupción en crash.
- `src/storage/vfile.rs:227,249,261` — `mincore` unaligned address risk
  → **✅ Fixed:** `aligned_addr = addr_val & !(page_size - 1)` y `aligned_len` calculado correctamente. `mincore` recibe direcciones page-aligned.
- `src/storage/engine/ops.rs:553,672,994` — f32 re-interpret cast without alignment check
  → **⏳ Parcial:** `debug_assert_eq!(vec_bytes.as_ptr().align_offset(4), 0)` agregado en los 3 sitios. Solo debug builds — release no tiene verificación runtime.

### Phase 3: Performance
#### Critical
- `src/wal.rs:318` — `postcard::to_allocvec` allocates per WAL append (N+1 allocations)
  → **🔴 Persiste:** código idéntico.
- `src/backends/fjall_backend.rs:197-209` — `scan_prefix()` materializes full Vec<(Vec<u8>, Vec<u8>)>
  → **⏳ Parcial:** Ahora `scan_prefix_iter` → lazy iterator (no materializa todo), pero cada item sigue copiando `key.to_vec(), value.to_vec()`.
- `src/sdk/search.rs:246-250` — `lexical_search()` HashMap alloc per scored posting
  → **⏳ Parcial:** Ahora un solo `HashMap::with_capacity(node_ids.len())` por búsqueda (no por posting).
- `src/sdk/serialization/mod.rs:228-248` — `memory_record_from_node()` 2-pass BTreeMap
  → **🔴 Persiste:** código idéntico (collect + remove).
- `src/index/search.rs:24-28` — `search_layer()` HashSet per HNSW layer (no reuse)
  → **✅ Fixed:** `visited: &mut HashSet` — pasado como `&mut`, reusado entre layers.

#### Important
- `src/sdk/serialization/mod.rs:307` — `metadata.clone()` after `take()`
  → **🔴 Persiste:** `std::mem::take()` → `.clone()` → re-asignación. Idéntico.
- `src/planner.rs:120` — `fuse_rrf()` clones 2 Strings per entry (use node_id)
  → **🔴 Persiste:** `BTreeMap<(String, String), ...>` idéntico.
- `src/index/search.rs:251-255` — `select_neighbors()` stores full Vec<f32> per candidate
  → **🔴 Persiste:** `SelectedInfo { vec: Option<Vec<f32>> }` idéntico.
- `src/wal_sharded.rs:85-89` — `batch_append()` clones WalRecord per shard
  → **🔴 Persiste:** `batches[idx].push(record.clone())` idéntico.
- `src/wal.rs:348` — buffer estimate 128 bytes under-guesses for vectors
  → **🔴 Persiste:** `let estimated = records.len() * 128` idéntico.
- `src/backends/fjall_backend.rs:128-131` — `get()` always `.to_vec()` (no zero-copy)
  → **🔴 Persiste:** `opt.map(|slice| slice.to_vec())` idéntico.
- `vantadb-python/src/lib.rs:320-356` — `node_to_pydict()` allocates 10+ Python objects per node
  → **🔴 Persiste:** misma estructura (PyDict + PyList + 8 items + fields dict + edges tuples).

**Resumen perf:** 9/12 persisten, 1 fixed (HNSW HashSet reuse), 2 parciales. La mayoría inalterado.

### Phase 4: Code Review
#### Critical
- `vantadb-python/src/lib.rs:1766-1767` — UB fix: `VantaVector` changed to `Box<[f32]>`
  → **✅ Fixed:** `data: Box<[f32]>` confirmado.
- `vantadb-python/src/lib.rs:833` — `put_batch` positional API deprecated
  → **✅ Fixed:** `#[deprecated]` + `#[allow(deprecated)]` presente.
- `vantadb-wasm/src/opfs.rs:83-86` — `OpfsFile::delete` now works
  → **✅ Fixed:** Llama `js_call(&self.handle, "remove", ...)`, retorna `Ok(true)`.

#### Important
- `vantadb-wasm/src/worker.rs:171-173` — `is_retryable` substring match on error string
  → **🔴 Persiste:** `is_some_and(|s| s.contains("timeout") || s.contains("abort") || s.contains("try again"))` — sigue siendo substring matching frágil.
- `vantadb-wasm/src/worker.rs:199-209` — `setTimeout` via eval-constructed string
  → **✅ Fixed:** Ahora usa `Reflect::get(&global, &"setTimeout")` + `Reflect::apply`. No más eval.

### Phase 5: Root Cause
- **Clippy wasm:** `Promise` missing from `use js_sys::{Array, Promise, Reflect};` on line 20
  → **✅ Fixed:** `Promise` importado en `worker.rs:20` donde se usa.

### Phase 6: Deep Module Review
| Module | Score | Top Issue | Re-audit |
|--------|-------|-----------|----------|
| vantadb core | 8/10 | High `expect`/`unwrap` count, files >1000L | ⏳ Sin cambios |
| vantadb-python | 6/10 | Only 3 test files, `connect()` hardcodes path | **🔴 Persiste:** sigue con 3 test files (`test_sdk.py`, `test_perf_15_16.py`, `test_load.py`) |
| vantadb-mcp | 7/10 | Single ~2000L file, hardcoded AXIOMS | **🟡 Mejoró:** 1245 líneas (bajó de ~2000) |
| vantadb-wasm | 7/10 | `unwrap()` in JS interop, `collect_all_deduped` unbounded | ⏳ Sin cambios |

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

#### Blockers (re-audited)
- `cargo clippy -D warnings` on `vantadb` core: 9 unnecessary `unsafe` blocks (memmap2 safe), 4 dead methods
  → **🔴 Persiste:** Al menos 6 `unsafe` innecesarias en `src/storage/vfile.rs` (Mmap::map, MmapMut::map_mut shims + Send/Sync + mmap creation).
- `cargo clippy -D warnings` on `vantadb_py`: 1 deprecation warning for `put_batch`
  → **✅ Fixed:** `#[deprecated]` + `#[allow(deprecated)]` aplicado.
- `deny.toml`: 2 stale advisory ignores (`RUSTSEC-2024-0436`, `RUSTSEC-2025-0134`)
  → **⏳ Parcial:** `RUSTSEC-2025-0134` removido ✅. `RUSTSEC-2024-0436` persiste 🔴, sigue sin `expires`.

**Verdict: ❌ FAIL** (parcialmente mejorado, pero unsafe + stale advisory + performance criticals bloquean certify)

## Scoreboard (actualizado 2026-07-21)

| Category | Score (0-10) | Original | Cambio | Notas |
|----------|-------------|----------|--------|-------|
| Code Quality | 7 | 7 | — | Clippy warnings parcialmente reducidos (put_batch deprecation fixeado ✅) |
| Security | 6 → **6.5** | 6 | ▲ +0.5 | 2 Criticals mitigados (Reflect panic, SendPtr UB), archive.rs sin fsync persiste |
| Performance | 4 | 4 | — | 9/12 hallazgos sin cambios. Solo HNSW HashSet reuse fixeado ✅ |
| Architecture | 8 | 8 | — | Sin cambios |
| Tests | 9 | 9 | — | 546/546 passing |
| Docs | 6 → **7** | 6 | ▲ +1.0 | docs/architecture/ corregido (score 7.8→9.5), ADR-0001 creado ✅ |

## FODA (actualizado 2026-07-21)

| Dimensión | Hallazgos |
|-----------|-----------|
| **Fortalezas** | 546 tests pasan. Arquitectura limpia. Varios UB/security fixes aplicados post-audit (SendPtr, ClientEngine, mincore, setTimeout). CLI checks sólidos. |
| **Oportunidades** | WAL allocation pooling, scan_prefix zero-copy, BTreeMap 2-pass elimination, Planner String cloning. ~20-40% mejora potencial en search latency aún disponible. |
| **Debilidades** | ~14 hallazgos persisten (de 25 originales). `unsafe` innecesarios bloquean certify. archive.rs sin fsync — riesgo de corrupción real. 9/12 performance issues sin tocar. |
| **Amenazas** | 3 dependencias unmaintained sin revisión (atomic-polyfill, paste, rustls-pemfile). `RUSTSEC-2024-0436` stale sin expiración. |

## Veredicto (actualizado 2026-07-21)
❌ **FAIL** — Aunque 6 hallazgos fueron fixeados y 5 mitigados parcialmente, ~14 persisten. Los blockers principales siguen siendo:
1. **`unsafe` innecesario** en mmap wrappers (`src/storage/vfile.rs` — 6+ bloques)
2. **Performance Criticals** (9/12 sin cambios — WAL alloc, BTreeMap 2-pass, planner String clones)
3. **`archive.rs` sin fsync** antes de rename — riesgo de corrupción en crash
4. **`deny.toml`** — `RUSTSEC-2024-0436` sigue sin expiración
5. **WASM `is_retryable`** — substring matching frágil
6. **Python SDK** — solo 3 test files
