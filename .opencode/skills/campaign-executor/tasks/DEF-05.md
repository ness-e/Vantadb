# DEF-05: lexical_search — HashMap sin capacidad + node.clone() redundante

## Metadata
- **Plan file:** `docs/plans/2026-07-19-deferred-fixes.md`
- **Fuente:** Investigación vanta-tuner
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟡
- **Tipo:** Rust
- **Turns estimados:** 5-8
- **Creado:** 2026-07-19T14:00
- **last-synced:** 2026-07-19T14:00
- **Estado:** ⬜ PENDING

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/sdk/search.rs` (search, hybrid_search, explain_memory_search, debug_memory_search_plan_for_tests — 6 code paths, 9 call sites) |
| Callees | `src/sdk/search.rs:246-250` (HashMap alloc), `src/sdk/search.rs:260` (node.clone()), `src/sdk/search.rs:318-322` (vector_memory_search) |
| Implicaciones | `HashMap::new()` sin capacidad causa re-hashing O(log N). `node.clone()` clona UnifiedNode completo (~300-800 bytes). Fix: `with_capacity(node_ids.len())` + memory_record_from_node toma `&UnifiedNode`. Mismo patrón en `vector_memory_search` (line 318). Sin cambios de API pública. |

## Contrato
```
cargo check -p vantadb && cargo nextest run --profile audit --workspace --build-jobs 2 pasa
```

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, test)
- rust-analyzer-mcp (goto def, references)
- codegraph_explore (blast radius)

## Investigation Notes
- `HashMap<u128, UnifiedNode>` en lexical_search (line 246) sin capacidad pre-asignada
- `HashMap<u128, UnifiedNode>` en vector_memory_search (line 318) mismo patrón
- `node.clone()` en line 260 y 328 — memory_record_from_node toma `&UnifiedNode` en vez de `UnifiedNode`
- Unambiguous fix: pre-allocar con `HashMap::with_capacity(node_ids.len())` + cambiar signature de memory_record_from_node

## Steps

### Step 1: with_capacity en lexical_search
- **Archivos:** `src/sdk/search.rs:246-250`
- **Acción:** Cambiar `HashMap::new()` → `HashMap::with_capacity(node_ids.len())`
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 2: with_capacity en vector_memory_search
- **Archivos:** `src/sdk/search.rs:318-322`
- **Acción:** Cambiar `HashMap::new()` → `HashMap::with_capacity(candidate_ids.len())`
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: Eliminar node.clone() — memory_record_from_node toma &UnifiedNode
- **Archivos:** `src/sdk/search.rs:260,328` y definición de `memory_record_from_node` (buscar en search.rs)
- **Acción:** Cambiar signature `memory_record_from_node(node: UnifiedNode)` → `memory_record_from_node(node: &UnifiedNode)`. Remover `.clone()` en call sites. Ajustar body interno si toma ownership.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 4: Full verify
- **Acción:** `cargo check -p vantadb && cargo clippy -p vantadb -- -D warnings && cargo nextest run --profile audit --workspace --build-jobs 2`
- **Verify:** 0 errors, 0 warnings, 0 test failures
- **Estado:** ⬜ PENDING

### Step 5: Commit
- **Acción:** `git add -A && git commit -m "perf: pre-alloc HashMap in lexical_search + avoid node.clone() hot path"`
- **Verify:** Commit creado
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna
