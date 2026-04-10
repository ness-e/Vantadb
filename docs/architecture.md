# VantaDB Architecture

VantaDB is designed as a hybrid memory-mapped system integrating three traditionally separate paradigms: Relational Metadata, Vector Search, and Graph Adjacency. It executes directly in the Python memory space via a PyO3 bridge.

## Core Abstraction: `UnifiedNode`

The system relies on a single continuous struct array in Rust:
```rust
pub struct UnifiedNode {
    pub id: u64,
    pub bitset: u128,              // Fast pre-filtering mask
    pub vector: VectorRepresentations, // Dense array float32
    pub edges: Vec<Edge>,          // Adjacency traversals
    pub relational: BTreeMap<String, FieldValue>, // Metadata
}
```

## Hybrid Persistence

VantaDB operates on an ephemeral/persistent hybrid model.
1. RAM-only `HashMap` cache for fast iterations.
2. Direct disk `mmap` backing up the main storage files when instantiating `VantaDB(path="./data")`.

## The HNSW Index implementation

VantaDB directly builds a Hierarchical Navigable Small World algorithm in `index.rs` around the vectors of `UnifiedNode`:
- `M`: Max connections bounded by configuration.
- `ef_construction`: Deep exploration during inserting constraints.
- `ef_search`: Greedy beam search bounds.

The nodes themselves are referenced into HNSW layers using memory offsets, maintaining an edge connection matrix that facilitates approximate nearest neighbor lookups in logarithmic time.

## Concurrency
Uses background locking through standard Rust synchronization primitives `RwLock` ensuring deterministic reads and isolated writes.
