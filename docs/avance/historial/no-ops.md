---
title: "Historial — No-ops / SKIPs"
type: historial
status: active
tags: [vantadb, avance, noops, skip, historial]
last_reviewed: 2026-08-07
aliases: []
---

# Historial — No-ops / SKIPs

> Tareas del backlog que resultaron ser **no-op** (ya implementadas en código real) o **SKIP** (decidido no hacer por razón válida). Documentado para que nadie las re-abra sin verificar.

## No-ops verificados contra código

| ID | Tarea | Por qué no-op |
|----|-------|---------------|
| DRV-031 | Doc comment duplicado | ✅ SKIP — Side effect de refactor previo, doc existe 1 vez |
| DRV-026 | Redundant unwrap en three_way_merge | ✅ SKIP — Código usa match sobre .get(), sin unwrap |
| DRV-116 | 10 warnings compilación | ✅ SKIP — `cargo check -p vantadb` + `-p vantadb-mcp` = 0 warnings |
| DRV-040 | unsafe sin SAFETY en simd.rs | ✅ SKIP — No existe archivo simd.rs en el proyecto |
| DRV-109 | LlamaIndex GIL release (pyo3 detach) | ✅ no-op — ya era correcto desde el inicio; `cargo check -p vantadb-llamaindex` pasa, commit no-op |
| DRV-126 | Paginación offset-based → keyset pagination | ✅ RESUELTO — SearchResults ya implementa paginación keyset + offset-based en `src/sdk/search/mod.rs`. Skip, no se necesita DRV. |
| SEC-14 | cargo-deny passing con licencias correctas | ✅ RESUELTO — `cargo deny check` pasa en CI; licencias MIT/Apache-2.0 solamente en deny.toml |
| NUEVO-20 | Dockerfile multi-stage | ✅ RESUELTO — `Dockerfile` ya existe con build multi-stage; CI lo usa para release |
| SEC-01 | bincode 1.x→2.0 | ✅ ya migrado (via AUD-03) |
| SEC-02 | rustls-pemfile deprecation | ✅ ya en v2 |
| NUEVO-12 | BroadcastChannel | ✅ ya existe |
| NUEVO-11 | IdbStorage | ✅ ya existe |
| NUEVO-10 | Benchmark suite | ✅ ya existe con perf-bench-40.yml y resultados |
| VFY-001 | catch blocks TS SDK | ✅ pre-fixed — todos los catches tienen `throw wrapWasmError(e, ...)` |
| VFY-002 | TS SDK get_nns_by_id spawn batching | ✅ pre-fixed — no tiene get_nns_by_id; `search()`/`searchVector()` directos |
| VFY-006 | add_node/remove_node lock contention | ✅ Corregido — DashMap (locking por shard) + AtomicUsize/AtomicU128 (lock-free). Único Mutex es rng |
| VFY-007 | remove_node O(n²) neighbor fixup | ✅ Corregido — archivo real `src/index/graph.rs` (no `core.rs`) |
| VFY-009 | 39 inline styles | ✅ SKIP — todos dinámicos |
| TSK-106 | GitHub Discussions | ❌ SKIP — requiere humano |

## SKIP por gate (features ya implementadas, sin re-investigar)

- **INV-019 (Advanced Tokenizer):** SKIP por gate — feature ya implementada, validada por audit 2026-07-28 como "Más completo de lo reportado". Sin code changes. Gap detectado: `docs/api/ADVANCED_TOKENIZER.md` no existe (ticket separado).
- **DRV-041 (worker.rs Promise):** Corregido — `_reject` sí se invoca (línea 254), usa serde_wasm_bindgen (no serde_json round-trip). Descripción no coincide con código real. Document-only.
- **DRV-060-063 (P1-5 wasm-opt):** ya implementado.

## SKIP del pipeline 2026-07-13 (Sesión 2)

- TSK-103 → cubierto por MKT-15/NUEVO-10, no implementado.
- OLD-01 → DEFER.

## No implementado (NUNCA — no confundir con no-op)

| ID | Item | Estado real |
|----|------|-------------|
| TSK-111 | Expanded Filter Operators | ❌ NUNCA IMPLEMENTADO — engine tiene 6 operadores (`Eq, Neq, Gt, Lt, Gte, Lte`) en `src/query.rs`/`physical_plan.rs` para IQL, pero `matches_memory_filters()` solo hace `==`. `FilterOperator`/`MemoryFilter` nunca existieron en `src/sdk.rs` |
| TSK-119 | `delete_by_filter()` | ❌ Solo CLI handler (`cmd_delete_by_filter`), nunca fue SDK; eliminado en AUD-09 |
| TSK-86 | `similar_to_key()` | ❌ Nunca implementado en ningún lenguaje (hasta REC-002 ✅ 2026-07-31) |

> **Fuentes:** `docs/progreso/ARCHIVO_HISTORICO.md` §No-ops/SKIPs, `docs/progreso/README.md` §RESUELTO, `docs/progreso/bitacora.md` Sesión 2.