# Plan: PERF-24, PERF-26, NUEVO-07, NUEVO-11→15

## Order (dependencies bottom-up)

### Phase 1: Unstaged changes → commit
- `vantadb-python/src/lib.rs`: bool→int ordering fix
- `vantadb-mcp/src/lib.rs`: VantaEmbedded search refactor
- `vantadb-ts/src/__tests__/vanta.test.ts`: test cleanup

### Phase 2: PERF-24 (GIL scope)
- `vantadb-python/src/lib.rs`
- Audit methods that take `py: Python` but don't detach

### Phase 3: PERF-26 (lazy serialization)
- `vantadb-python/src/lib.rs`, `vantadb-python/src/types.rs`
- Defer metadata serialization until access

### Phase 4: NUEVO-07 (migration tools)
- `vantadb-python/vantadb_py/migrate/` — new module
- ChromaDB→Vanta + LanceDB→Vanta scripts

### Phase 5: NUEVO-11→15 (WASM features)
- Start with NUEVO-14 (bundle size, easiest)
- Then NUEVO-15 (coverage CI)
- Then NUEVO-11 (IndexedDB)
- Then NUEVO-12 (multi-tab)
- Then NUEVO-13 (auto-tuning)
