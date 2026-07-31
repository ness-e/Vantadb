# COMP-027: Multiple Index Types (IVF, DiskANN, SCANN)

**Effort:** 🟠 5-10d
**Dependencies:** COMP-008 (VecIndex trait) ✅
**Type:** Rust core — index algorithms
**Workflow:** feature-add

---

## Problem

Currently VantaDB has two index backends:
- `CPIndex` (HNSW) — implements `VecIndex`, the default
- `IvfIndex` — exists as a struct with build/search but does NOT implement `VecIndex`; it's embedded inside `CPIndex` as a lazy-build option (`search_nearest` checks `index_type == Ivf`)

Additionally, there's `flat_search()` in `src/index/flat.rs` that does brute-force search, but it's a free function, not a VecIndex impl.

There's no DiskANN or SCANN implementation at all.

## Scope

### 1. `FlatIndex` — standalone VecIndex wrapping flat_search

Create a simple struct that wraps `flat_search` behind `VecIndex`. This lets the planner choose flat scan explicitly (not just via CPIndex's `flat_threshold` heuristic).

**Files:** `src/index/flat.rs`

```rust
pub struct FlatIndex {
    nodes: Vec<(u128, FilterBitset, VectorRepresentations)>,
    config: FlatConfig,
}

pub struct FlatConfig {
    pub distance_metric: DistanceMetric,
}
```

Implement `VecIndex for FlatIndex`:
- `search()` → calls `flat_search()` from existing module
- `add()` → pushes to internal vec
- `len()` → `self.nodes.len()`
- `estimate_memory_bytes()` → sum of vector sizes
- `is_empty()` → `self.nodes.is_empty()`

### 2. `IvfIndex` — standalone VecIndex implementation

`IvfIndex` already exists in `src/index/ivf.rs` with `build()` and `search()`. It just needs:
- `impl VecIndex for IvfIndex`
- Modifications to its search signature to match VecIndex's trait requirements

**Files:** `src/index/ivf.rs`, `src/index/mod.rs`

```rust
impl VecIndex for IvfIndex {
    fn search(&self, query_vec, query_mask, top_k, vector_store, distance_metric) -> Vec<(u128, f32)> {
        // Already has the method, ensure signature match
        IvfIndex::search(self, query_vec, top_k, query_mask)
    }
    fn add(&self, id, bitset, vec_data, storage_offset) {
        // IVF is read-only after build — add() is a no-op or rebuilds
        // Since IVF is typically batch-built, this can be a no-op with log warning
    }
    fn len(&self) -> usize { /* sum of inverted list lengths */ }
    fn estimate_memory_bytes(&self) -> usize { /* centroids + inverted lists */ }
    fn is_empty(&self) -> bool { self.centroids.is_empty() }
}
```

Also extract the IVF lazy-build from `CPIndex::search_nearest()` (src/index/search.rs:516-527) into the engine layer so it chooses IndexType::Ivf → uses standalone IvfIndex.

### 3. DiskANN — Vamana graph index

DiskANN is a disk-based ANN algorithm using a Vamana graph. Implementation:

**Files:** `src/index/diskann.rs` (new)

```rust
pub struct DiskAnnIndex {
    graph: HashMap<u128, Vec<u128>>,  // Vamana adjacency list
    vectors: HashMap<u128, Vec<f32>>,
    medoid: u128,
    config: DiskAnnConfig,
}

pub struct DiskAnnConfig {
    pub search_list_size: usize,    // L
    pub search_list_size_construction: usize, // R
    pub alpha: f32,                  // graph pruning parameter (>1)
}
```

Implement `VecIndex for DiskAnnIndex`:
- `search()` → greedy search with bounded priority queue
- `add()` → insert into Vamana graph with pruning
- `len()` → node count

**Minimal Vamana search:**
1. Start from `medoid`
2. Maintain priority queue of candidates
3. Prune to `search_list_size` (L)
4. Return top_k results

### 4. SCANN — Vector compression + anisotropic quantization

SCANN (Squeezed Cannes) uses product quantization (PQ) or scalar quantization (SQ) for compressed search:

**Files:** `src/index/scann.rs` (new)

```rust
pub struct ScannIndex {
    codes: Vec<u128>,           // compressed vector codes
    codebook: Vec<Vec<f32>>,    // PQ codebook
    ids: Vec<u128>,
    config: ScannConfig,
}
```

For ponytail: implement a simplified version using **scalar quantization (SQ8)**:
- Compress f32 vectors to u8 (1 byte per dimension)
- Search: decompress and score (or use lookup tables)
- Much simpler than full PQ while showing the concept

**Implement `VecIndex for ScannIndex`**

### 5. Planner Integration (`src/index/mod.rs`)

Update `IndexType` enum:
```rust
pub enum IndexType {
    Hnsw,
    Ivf,
    Flat,
    DiskAnn,
    Scann,
}
```

Add factory method or create logic in the engine layer:
```rust
pub fn create_index(index_type: IndexType, config: &HnswConfig) -> Arc<dyn VecIndex> {
    match index_type {
        IndexType::Hnsw => Arc::new(CPIndex::new_with_config(config.clone())),
        IndexType::Ivf => Arc::new(IvfIndex::new(config.distance_metric)),
        IndexType::Flat => Arc::new(FlatIndex::new(config.distance_metric)),
        IndexType::DiskAnn => Arc::new(DiskAnnIndex::new(DiskAnnConfig::default())),
        IndexType::Scann => Arc::new(ScannIndex::new(config.distance_metric)),
    }
}
```

### 6. Serialization (compatibility with existing data)

- `IvfIndex` — already has serialize/deserialize
- `FlatIndex` — serde derive
- `DiskAnnIndex` — custom serialize (large graph structure)
- `ScannIndex` — PQ codebook + compressed codes

### 7. Tests

- `test_flat_index_basic` — search with known vectors
- `test_ivf_vecindex_trait` — IvfIndex implements VecIndex
- `test_diskann_basic` — minimal Vamana search
- `test_scann_basic` — SQ8 search
- `test_create_index_happy_path` — factory returns correct type
- `test_create_index_serialize_roundtrip` — each index survives serde

## Implementation Order

1. **FlatIndex** (smallest, wraps existing) — 1h
2. **IvfIndex standalone** (extract from CPIndex) — 2h
3. **Planner integration** (IndexType enum, factory) — 1h
4. **DiskAnn** (new module, Vamana graph) — 5-8h
5. **SCANN** (new module, SQ8) — 3-5h
6. **Serialization** — 2h
7. **Tests** — 2h

## Key Constraints

- **Ponytail**: DiskAnn and SCANN are simplified versions (not full implementations). Full DiskAnn with disk I/O and SCANN with anisotropic PQ are future work.
- **No regression**: existing CPIndex/HNSW behavior unchanged
- **VecIndex trait unchanged**: the trait is the contract; new types just implement it
- **Backward compat**: serialized HNSW data still loads

## Files to Create/Modify

| File | Action |
|------|--------|
| `src/index/mod.rs` | Add `IndexType::Flat`, `IndexType::DiskAnn`, `IndexType::Scann`. Add `create_index()` factory. |
| `src/index/flat.rs` | Add `FlatIndex` struct + `impl VecIndex for FlatIndex` |
| `src/index/ivf.rs` | Add `impl VecIndex for IvfIndex` |
| `src/index/diskann.rs` | NEW — `DiskAnnIndex` with Vamana graph |
| `src/index/scann.rs` | NEW — `ScannIndex` with SQ8 compression |
| `src/index/search.rs` | Extract IVF path to use standalone IvfIndex |
| `src/index/serialize.rs` | Add DiskAnn/Scann serialization |
| `src/index/core.rs` | Tests for new index types |

## Verification

```bash
cargo check -p vantadb
cargo test -p vantadb -- index 2>&1 | grep -E "(PASS|FAIL|test.*flat|test.*ivf|test.*diskann|test.*scann)"
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## References

- `src/index/mod.rs:41-77` — VecIndex trait
- `src/index/mod.rs:23-30` — IndexType enum (currently Hnsw + Ivf)
- `src/index/ivf.rs:51-57` — IvfIndex struct
- `src/index/ivf.rs:234-268` — IvfIndex::search()
- `src/index/flat.rs:7-50` — flat_search() function
- `src/index/search.rs:516-527` — IVF lazy-build inside CPIndex
- `src/index/search.rs:626-657` — CPIndex impl VecIndex (pattern to follow)
- COMP-008 task file: `.opencode/skills/campaign-executor/tasks/COMP-008.md`
