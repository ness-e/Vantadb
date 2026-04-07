# ElectricSynapse vs ChemicalSynapse — Hardware-Aware Edge Resolution

> **Status:** DEFERRED (v0.3.0+)  
> **Decision:** Concept validated but deferred due to Rust ownership complexity and minimal latency gain at current scale.

---

## 1. Concept

Differentiate between two types of connections (Edges/Synapses) based on their memory residency:

| Type | Resolution | Latency | Rust Implementation |
|---|---|---|---|
| **ElectricSynapse** | Direct pointer in RAM | ~1-5ns | `Arc<RwLock<Neuron>>` or raw `&Neuron` reference |
| **ChemicalSynapse** | Disk-backed lookup via ID | ~50-500μs | `u64` ID → `StorageEngine::get(id)` (current `Edge`) |

The biological metaphor:
- **Electric synapses** in the brain transmit signals almost instantaneously via gap junctions (direct electrical coupling).
- **Chemical synapses** require neurotransmitter release, diffusion, and receptor binding — slower but more modulable.

---

## 2. Current Architecture (v0.2.0)

All edges today are effectively "Chemical":

```rust
pub struct Edge {
    pub target: u64,     // Always a lookup ID — never a direct pointer
    pub label: String,
    pub weight: f32,
}
```

Resolution always requires:
```rust
// O(1) HashMap lookup but still involves hashing + cache miss potential
let target_node = engine.nodes.read().get(&edge.target).cloned();
```

---

## 3. Proposed Design for ElectricSynapse

### Option A: Arc-based (Safe, Higher Memory)
```rust
pub enum SynapseType {
    /// Direct RAM pointer — instantaneous resolution, no disk I/O
    Electric(Arc<RwLock<Neuron>>),
    /// ID-based lookup — requires StorageEngine::get()  
    Chemical { target_id: u64, label: String, weight: f32 },
}
```

**Pros:** Safe Rust, no unsafe code.  
**Cons:** `Arc<RwLock<>>` adds 16-24 bytes overhead per edge. Reference counting costs. Potential deadlocks in cyclic graphs.

### Option B: Weak Reference (Avoids Cycles)
```rust
pub enum SynapseType {
    Electric(Weak<RwLock<Neuron>>),
    Chemical { target_id: u64, label: String, weight: f32 },
}
```

**Pros:** Breaks cycles via `Weak`.  
**Cons:** `upgrade()` can fail if node was dropped (dangling synapse). Requires fallback to Chemical.

### Option C: Arena Allocator (Best Performance)
```rust
pub struct NeuronArena {
    neurons: Vec<Neuron>,  // Contiguous memory
}

pub struct ElectricSynapse {
    index: usize,  // Direct index into the arena
}
```

**Pros:** Cache-friendly, zero overhead per reference, no Arc/Weak.  
**Cons:** Cannot remove neurons from middle of arena without fragmentation. Requires generational indices for safety.

---

## 4. Why Deferred

1. **Ownership complexity in Rust:** Cyclic graph structures with direct pointers require either `unsafe`, `Arc<Mutex>`, or arena allocators — all adding complexity to the core engine.
2. **Minimal measurable gain:** The difference between `HashMap::get()` (~20ns) and a direct pointer (~1-5ns) is negligible compared to the total query pipeline (~4ms).
3. **BFS traversal is not the bottleneck:** Current benchmarks show BFS at depth=3 completes in <5ms. The bottleneck is vector similarity computation, not edge resolution.

---

## 5. Conditions for Revisiting

Implement ElectricSynapse when:
- [ ] Graph traversal benchmarks show >50% of query time spent on edge resolution
- [ ] Dataset exceeds 10M edges where HashMap contention becomes measurable
- [ ] A specific use case (real-time graph streaming) requires sub-microsecond edge resolution
- [ ] Arena allocator pattern is validated in a separate prototype

---

## 6. Implementation Roadmap (When Ready)

```
Phase 1: Implement NeuronArena with generational indices
Phase 2: Create SynapseType enum (Electric/Chemical)
Phase 3: Modify graph.rs BFS to use arena-direct resolution for Electric
Phase 4: Add Cargo feature flag: `electric_synapses`
Phase 5: Benchmark comparison: Electric vs Chemical at 1M/10M/100M edges
```

---

## 7. References

- Rust Arena allocators: `typed-arena`, `bumpalo`, `generational-arena` crates
- Graph ownership patterns in Rust: https://rust-unofficial.github.io/too-many-lists/
- Mechanical Sympathy: Martin Thompson's work on cache-oblivious data structures
