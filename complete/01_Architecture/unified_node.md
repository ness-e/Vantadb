# UnifiedNode — Core Data Structure

> **Fase**: 01_Architecture | **Decision**: Approved

## 1. Design Goal

Single struct unifying **vector**, **graph**, and **relational** data with cache-friendly layout. The header (id + bitset + cluster + flags) fits a 64-byte L1 cache line for fast scanning; heavy data lives behind heap pointers.

## 2. Struct Layout

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnifiedNode {
    pub id: u64,               //  8B — globally unique
    pub bitset: u128,          // 16B — 128 boolean filter dims
    pub semantic_cluster: u32, //  4B — super-node cluster ID
    pub flags: NodeFlags,      //  4B — ACTIVE|INDEXED|DIRTY|TOMBSTONE
    // ── cache line boundary (32B so far) ──
    pub vector: VectorData,    //  → heap (enum tag 8B + ptr)
    pub edges: Vec<Edge>,      //  → heap (24B: ptr+len+cap)
    pub relational: RelFields, //  → heap (BTreeMap)
}
// Header: 32B inline + 3 heap pointers ≈ 56B before heap data
```

## 3. Memory Layout (ASCII)

```
UnifiedNode (stack/inline portion):
┌──────────┬──────────────────┬─────────────┬───────────┐
│ id: u64  │   bitset: u128   │cluster: u32 │flags: u32 │
│  8 bytes │    16 bytes      │  4 bytes    │ 4 bytes   │
├──────────┴──────────────────┴─────────────┴───────────┤
│ vector: VectorData (enum tag + inline data or ptr)    │
│ edges: Vec<Edge> (ptr, len, cap = 24 bytes)           │
│ relational: BTreeMap<String, FieldValue>              │
└───────────────────────────────────────────────────────┘
         │                │              │
         ▼                ▼              ▼
    ┌─────────┐    ┌───────────┐   ┌──────────┐
    │ f32×1536│    │Edge,Edge, │   │"pais"→VZ │
    │ = 6KB   │    │Edge...    │   │"activo"→T│
    └─────────┘    └───────────┘   └──────────┘
    Heap: ~6KB     Heap: ~32B/edge  Heap: variable
```

## 4. VectorData Enum

```rust
pub enum VectorData {
    F32(Vec<f32>),                              // Hot: full precision
    I8 { data: Vec<i8>, scale: f32, offset: f32 }, // Warm: quantized
    None,                                       // No vector
}
```

| Variant | Memory/1536d | Precision | Use Case |
|---------|-------------|-----------|----------|
| F32 | 6,144 B | Full | Active search, <1M vectors |
| I8 | 1,544 B | ~95% recall | Tiered storage, >1M vectors |
| None | 0 B | — | Pure graph/relational nodes |

**Trade-off**: PQ (Product Quantization) deferred to Fase 3. I8 linear quantization is sufficient for MVP with ~95% recall at 4× compression.

## 5. Bitset Design (u128)

128 bits mapped to categorical boolean filters:

```
Bit allocation example:
  0-7:   Country code (8 bits → 256 countries)
  8-15:  Role/category (8 bits → 256 roles)
  16:    is_active
  17:    is_verified
  18:    has_vector
  19:    has_edges
  20-31: Reserved (application-defined)
  32-127: User-extensible
```

**Key operations** (single CPU instruction each):
```rust
// Set bits
node.bitset |= 1u128 << 16;  // mark active

// Filter: "active AND country=5"
let mask: u128 = (1 << 16) | (1 << 5);
let match = (node.bitset & mask) == mask; // ONE instruction
```

**Why u128 over BitVec**: Fits in two 64-bit registers. Bitwise AND is 2 instructions vs BitVec's heap allocation + loop. For scan-heavy workloads (brute-force vector search with filter), this saves ~40% CPU time.

## 6. Edge Struct

```rust
pub struct Edge {
    pub target: u64,     // 8B
    pub label: String,   // 24B (ptr+len+cap) + heap
    pub weight: f32,     // 4B
}   // 36B total per edge
```

**Fase 2 optimization**: Intern edge labels (u32 label_id → lookup table) to save 20B/edge and enable faster label matching.

## 7. NodeFlags

```rust
pub struct NodeFlags(pub u32);

impl NodeFlags {
    pub const ACTIVE: u32     = 1 << 0;
    pub const INDEXED: u32    = 1 << 1;
    pub const DIRTY: u32      = 1 << 2;
    pub const TOMBSTONE: u32  = 1 << 3;
    pub const HAS_VECTOR: u32 = 1 << 4;
    pub const HAS_EDGES: u32  = 1 << 5;
}
```

## 8. Serialization

| Format | Use | Size (1536d node) |
|--------|-----|-------------------|
| bincode | WAL records | ~6.2 KB |
| Arrow IPC | Columnar export (Fase 2) | ~6.0 KB |
| Custom | RocksDB value (Fase 2) | ~6.1 KB |

Bincode chosen for Fase 1: zero-config, fast, compact. No schema evolution needed yet.

## 9. Memory Estimate per Node

| Component | Bytes | Notes |
|-----------|-------|-------|
| Header (inline) | 56 | id+bitset+cluster+flags+enum tags |
| Vector FP32×1536 | 6,144 | Most nodes |
| 4 edges avg | 144 | 36B × 4 |
| 3 relational fields | ~200 | String keys + values |
| **Total** | **~6,544** | |

**1M nodes × 6.5KB = ~6.2GB** — fits in 16GB with headroom for index + OS.
