# COMP-017: Accumulators for parallel graph algorithms

**Estado:** ❌ No implementado → 🟡 En progreso
**Esfuerzo:** 🟡 1-2 sem
**Dependencias:** Ninguna
**Bloquea:** COMP-022 (Graph Data Science — PageRank, centrality)

---

## Objetivo

Agregar **GraphAccumulator** — acumuladores atómicos thread-safe para algoritmos de grafos paralelos (PageRank, centralidad). Usar `AtomicU64` vía `f64::to_bits`/`f64::from_bits` para operaciones lock-free `fetch_add`.

## Contexto

- `src/graph.rs` — `GraphTraverser` con BFS/DFS, filtered, topological sort. No tiene soporte de acumuladores.
- `src/edge_index.rs` — `EdgeIndex` con `DashSet<(u128, u128)>` para tracking de aristas.
- `src/node.rs:478` — `Edge { target: u128, label_id: u32, weight: f32 }`
- `src/node.rs:806` — `UnifiedNode` con `edges: Vec<Edge>`, `label_index: HashMap<u32, Vec<u128>>`
- `src/sdk/graph.rs` — SDK wrapper con `graph_bfs`, `graph_dfs`, etc.
- `dashmap` ya es dependencia directa en workspace Cargo.toml (line 34)
- `rayon` disponible como feature opcional (line 75)

## Diseño

### 1. `GraphAccumulator` struct (en `src/accumulator.rs` — archivo nuevo)

```rust
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Thread-safe accumulator for parallel graph algorithms.
/// Stores f64 values encoded as AtomicU64 for lock-free fetch_add.
pub(crate) struct GraphAccumulator {
    /// Per-node accumulator values (f64 encoded as AtomicU64 bits).
    values: DashMap<u128, AtomicU64>,
}
```

Métodos:
- `new()` → Self
- `set(node_id: u128, value: f64)` → reemplaza el valor atómico
- `get(node_id: u128)` → Option<f64> — lee el valor actual
- `add(node_id: u128, delta: f64)` → f64 — `fetch_add` atómico, devuelve el valor *anterior*
- `snapshot()` → HashMap<u128, f64> — captura instantánea consistente
- `clear()` — reinicia todos los acumuladores
- `keys()` → Vec<u128> — IDs de todos los nodos con acumulador

### 2. Integración en `GraphTraverser` (en `src/graph.rs`)

No tocar los métodos existentes. Agregar:

```rust
pub fn traverse_with_accumulator(
    &self,
    roots: &[u128],
    max_depth: usize,
    acc: &GraphAccumulator,
    apply_fn: impl Fn(u128, &[Edge], &GraphAccumulator) -> f64,
) -> Result<Vec<u128>>
```

Que hace BFS + por cada nodo descubre edges y llama a `apply_fn(node_id, edges, acc)` para que el caller pueda actualizar acumuladores. Devuelve los nodos visitados.

Alternativamente (más simple, más ponytail): agregar método auxiliar `apply_to_edges` que dado un set de nodos visitados y un accumulator, itera y aplica una función:

```rust
pub fn accumulate_contributions(
    &self,
    nodes: &[u128],
    acc: &GraphAccumulator,
) -> Result<()>
```

### 3. SDK API (en `src/sdk/graph.rs`)

Agregar:
- `graph_create_accumulator()` → `GraphAccumulator`
- `graph_accumulator_add(acc, node_id, delta)` → Result<f64>
- `graph_accumulator_get(acc, node_id)` → Result<Option<f64>>
- `graph_accumulator_snapshot(acc)` → Result<HashMap<u128, f64>>

### 4. Exposición en `src/lib.rs`

Agregar `pub mod accumulator;` (si se crea archivo separado) o re-exportar desde `src/graph.rs`.

### 5. Tests

- `test_accumulator_basic` — set/get/add, verificar atomicidad
- `test_accumulator_concurrent` — 8 threads haciendo `add` concurrente, verificar suma exacta
- `test_accumulator_snapshot` — snapshot consistente
- `test_accumulator_integration` — `GraphTraverser` + `GraphAccumulator` sobre un grafo pequeño

## Archivos a modificar/crear

| Archivo | Acción |
|---------|--------|
| `src/accumulator.rs` | **CREAR** — `GraphAccumulator` struct + impl |
| `src/graph.rs` | **MODIFICAR** — agregar métodos accumulator-aware |
| `src/sdk/graph.rs` | **MODIFICAR** — agregar SDK methods para accumulator |
| `src/lib.rs` | **MODIFICAR** — exportar `pub mod accumulator` |
| `Cargo.toml` | NO TOCAR — `dashmap` ya está |

## Criterio de aceptación

- `cargo check -p vantadb` pasa
- Tests nuevos pasan: `cargo test -p vantadb accumulator`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` pasa
- API pública expuesta via SDK
- `GraphAccumulator::add()` es lock-free (solo `AtomicU64::fetch_add`)
- `GraphAccumulator` es `Send + Sync`
