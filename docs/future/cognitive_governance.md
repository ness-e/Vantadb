# Cognitive Governance — Entropy-Based Lifecycle Management

> **Status:** DEFERRED (v0.3.0+)  
> **Decision:** Pinning Semántico is trivial and approved for immediate implementation in next cycle. Full entropy-based pruning, Shadow Archive, and Apoptosis are v0.3.0.

---

## 1. Problem Statement

ConnectomeDB cannot grow infinitely without degrading:
- **Performance:** More neurons = slower scans, larger HNSW index, more RAM pressure
- **Semantic precision:** Stale data pollutes vector similarity results for AI agents
- **Logical coherence:** Outdated neurons create anachronistic connections in the graph

The system needs **intelligent self-maintenance** — the ability to evaluate, demote, archive, and ultimately forget data based on its actual contribution to the knowledge graph.

---

## 2. Multivariable Valuation Algorithm

Each neuron's **cognitive weight** is calculated dynamically:

```
W(n) = α·F(n) + β·R(n) + γ·C(n) + δ·Pu(n)
```

Where:
- **F(n)** = Frequency: How often this neuron is accessed (read count / total reads)
- **R(n)** = Recency: Time since last access, decayed exponentially: `e^(-λ·Δt)`
- **C(n)** = Centrality: Graph degree centrality (in-degree + out-degree / max degree in graph)
- **Pu(n)** = User Priority: Manual override (0.0 to ∞, where ∞ = PINNED)

**Default weights:** `α=0.3, β=0.3, γ=0.2, δ=0.2`

### Proposed Neuron Fields

```rust
pub struct CognitiveMetadata {
    pub access_count: u64,       // Incremented on every get()
    pub last_accessed: u64,      // Unix timestamp of last read
    pub created_at: u64,         // Unix timestamp of creation
    pub user_priority: f32,      // 0.0 = no preference, f32::INFINITY = PINNED
    pub cognitive_weight: f32,   // Last calculated weight
    pub weight_calculated_at: u64,
}

impl CognitiveMetadata {
    pub fn calculate_weight(&mut self, config: &GovernanceConfig, graph_centrality: f32) -> f32 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        // Frequency (normalized 0-1, saturates at 1000 accesses)
        let frequency = (self.access_count as f32 / 1000.0).min(1.0);
        
        // Recency (exponential decay, λ = 0.0001 → half-life ≈ 1.9 hours)
        let delta_t = (now - self.last_accessed) as f32;
        let recency = (-config.decay_lambda * delta_t).exp();
        
        // User priority (clamped, but INFINITY means PINNED)
        if self.user_priority.is_infinite() {
            self.cognitive_weight = f32::INFINITY;
            return f32::INFINITY; // PINNED neurons never decay
        }
        let priority = self.user_priority.min(1.0);
        
        self.cognitive_weight = config.alpha * frequency
            + config.beta * recency
            + config.gamma * graph_centrality
            + config.delta * priority;
        
        self.weight_calculated_at = now;
        self.cognitive_weight
    }
}
```

### Governance Configuration

```rust
pub struct GovernanceConfig {
    pub alpha: f32,          // Frequency weight (default: 0.3)
    pub beta: f32,           // Recency weight (default: 0.3)
    pub gamma: f32,          // Centrality weight (default: 0.2)
    pub delta: f32,          // User priority weight (default: 0.2)
    pub decay_lambda: f32,   // Exponential decay rate (default: 0.0001)
    pub entropy_threshold: f32,    // Below this, neuron is candidate for pruning (default: 0.1)
    pub archive_threshold: f32,    // Below this, move to Shadow Archive (default: 0.05)
    pub apoptosis_threshold: f32,  // Below this, permanent deletion (default: 0.01)
    pub sweep_interval_secs: u64,  // How often GC runs (default: 300 = 5 min)
    pub max_shadow_archive_size: usize, // Cap on archived neurons (default: 100_000)
}
```

---

## 3. Degradation Mechanisms

### 3.1 Semantic Pinning (Immunity) — READY FOR IMPLEMENTATION

The simplest feature, requiring only one new flag:

```rust
impl NodeFlags {
    pub const PINNED: u32 = 1 << 6;  // Immune to all GC operations
}
```

**User interface:**
```sql
-- Pin a critical neuron (immune to pruning)
UPDATE NODE#42 SET _pinned = true

-- Unpin
UPDATE NODE#42 SET _pinned = false
```

**GC integration:** Add single check at top of sweep loop:
```rust
if node.flags.is_set(NodeFlags::PINNED) {
    continue; // Skip all degradation checks
}
```

### 3.2 Entropy-Based Pruning (Semantic Decay)

Neurons whose cognitive weight falls below `entropy_threshold` are candidates for demotion:

```
Active (RAM/RocksDB) → Shadow Archive (cold storage)
```

**Criteria for pruning:**
1. `cognitive_weight < entropy_threshold` (0.1)
2. Not PINNED
3. No incoming edges from any PINNED neuron
4. Not accessed in the last `7 * 24 * 3600` seconds (7 days)

### 3.3 Shadow Archive (Irrelevance Backup)

Cold storage for pruned neurons. Stores only:

```rust
pub struct SemanticGhost {
    pub original_id: u64,
    pub vector_hash: u64,         // xxHash of the original vector (for similarity recovery)
    pub field_summary: String,    // Compact JSON of key relational fields
    pub edge_count: u32,          // How many edges it had
    pub pruned_at: u64,           // When was it archived
    pub pruned_reason: PruneReason,
    pub cognitive_weight_at_death: f32,
}

pub enum PruneReason {
    EntropyDecay,
    TTLExpired,
    UserDeleted,
    ApoptosisRedundancy,
}
```

**Storage:** Separate RocksDB column family: `"shadow_archive"`  
**Indexing:** Hash-based lookup by `original_id` (not in HNSW — frozen data doesn't participate in similarity search)

### 3.4 Apoptosis (Definitive Deletion)

Permanent bit-level elimination when:
1. Neuron has been in Shadow Archive for > 30 days
2. `cognitive_weight_at_death < apoptosis_threshold` (0.01)
3. No recovery requests in the archive period
4. Redundancy check passes: at least 3 other neurons with vector similarity > 0.95 exist in active store

Before apoptosis, create an **Amnesia Tombstone**.

### 3.5 Amnesia Tombstone (Black Box)

Auditable record of what was deleted and why:

```rust
pub struct AmnesiaTombstone {
    pub original_id: u64,
    pub deleted_at: u64,
    pub cause: PruneReason,
    pub vector_base_hash: u64,    // For forensic recovery possibility
    pub cognitive_weight_history: Vec<(u64, f32)>,  // timestamp, weight pairs
    pub edge_targets: Vec<u64>,   // Who was this connected to
    pub approximate_size_bytes: u32,
}
```

**Storage:** Append-only log file: `tombstones.bincode`  
**Purpose:** Analyze "Cortex Selection Bias" — is the system systematically forgetting certain types of data?

---

## 4. GC Integration (Enhanced gc.rs)

```rust
pub struct CognitiveGcWorker {
    storage: Arc<StorageEngine>,
    config: GovernanceConfig,
    shadow_archive: ColumnFamily,
    tombstone_log: File,
}

impl CognitiveGcWorker {
    /// Main sweep loop (runs every `sweep_interval_secs`)
    pub async fn sweep_cycle(&mut self) -> GcReport {
        let mut report = GcReport::default();
        
        // Phase 1: Recalculate cognitive weights for all neurons
        // Phase 2: Identify candidates below entropy_threshold
        // Phase 3: Move candidates to Shadow Archive (create SemanticGhost)
        // Phase 4: Check Shadow Archive for apoptosis candidates
        // Phase 5: Apoptosis + Tombstone creation
        // Phase 6: Emit Prometheus metrics
        
        report
    }
}

pub struct GcReport {
    pub neurons_scanned: usize,
    pub neurons_archived: usize,
    pub neurons_apoptosed: usize,
    pub tombstones_created: usize,
    pub shadow_archive_size: usize,
    pub elapsed_ms: u64,
}
```

---

## 5. Dashboard Metrics

| Metric | Type | Description |
|---|---|---|
| `connectome_cognitive_weight_avg` | Gauge | Average weight across all active neurons |
| `connectome_neurons_archived_total` | Counter | Total neurons moved to Shadow Archive |
| `connectome_neurons_apoptosed_total` | Counter | Total neurons permanently deleted |
| `connectome_shadow_archive_size` | Gauge | Current size of Shadow Archive |
| `connectome_forget_pressure` | Gauge | % of neurons below entropy_threshold |
| `connectome_pinned_neurons` | Gauge | Number of PINNED neurons |

---

## 6. IQL Syntax Extensions

```sql
-- Pin a neuron
PIN NODE#42

-- Unpin a neuron  
UNPIN NODE#42

-- View cognitive metadata
FROM Persona#42 FETCH _cognitive_weight, _access_count, _last_accessed

-- Query Shadow Archive
FROM ARCHIVE WHERE original_id = 42

-- Recover from archive back to active
RECOVER ARCHIVE#42

-- View amnesia log
SHOW TOMBSTONES LIMIT 10
```

---

## 7. Implementation Phases

```
Phase 1 (v0.2.1): NodeFlags::PINNED + GC skip check (1 hour)
Phase 2 (v0.3.0): CognitiveMetadata fields on Neuron + access tracking
Phase 3 (v0.3.0): GovernanceConfig + weight calculation
Phase 4 (v0.3.0): Shadow Archive (RocksDB column family)
Phase 5 (v0.3.1): Apoptosis + AmnesiaTombstone
Phase 6 (v0.4.0): Full IQL syntax (PIN, UNPIN, ARCHIVE queries)
Phase 7 (v0.4.0): Dashboard metrics integration
```

---

## 8. Cargo Feature Flag

```toml
[features]
cognitive_governance = []  # Enables entropy pruning, Shadow Archive, Apoptosis
```

When disabled, GC falls back to simple TTL-based eviction (current gc.rs behavior).
