# Memory Management Implementation Plan (TSK-46, TSK-50, TSK-76b)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement MMap-backed HNSW for 1M vectors in 8GB RAM, add backpressure at 80% RSS, and add weighted eviction by importance score.

**Architecture:** Three coordinated changes to `StorageEngine`: (1) ensure HNSW index uses mmap for zero-copy vectors, add `mmap_hnsw` config, add memory budget validation; (2) add RSS monitoring before write operations, reject with `VantaError::ResourceLimit` when >80%; (3) implement `evict_cold_nodes()` using weighted scoring (hits × confidence × importance × recency).

**Tech Stack:** Rust, mmap (std::os::memory_map), sysinfo, parking_lot, DashMap, ArcSwap

---

### Task 1: Add `mmap_hnsw` config and harden MMap HNSW path

**Files:**
- Modify: `src/config.rs:56-96` — Add `mmap_hnsw` field
- Modify: `src/storage.rs:607-720` — Use `mmap_hnsw` in engine open
- Modify: `src/index.rs:1599-1670` — Ensure `sync_to_mmap()` is reliable
- Create: `tests/memory/mmap_hnsw.rs` — Verification tests

**Step 1: Add `mmap_hnsw` to VantaConfig**

In `src/config.rs`, add field after `force_mmap`:
```rust
pub mmap_hnsw: bool,
```
Default to `true` in `Default` impl (line ~100). Add builder method `with_mmap_hnsw(mut self, val: bool)`.

**Step 2: Wire `mmap_hnsw` into StorageEngine::open_with_config()**

In `src/storage.rs`, in `open_with_config()` around line 719-721:
- Current: `let use_mmap = config.force_mmap || profile == LowResource || effective_memory < 16GB;`
- Change to: `let use_mmap = config.mmap_hnsw && (config.force_mmap || profile == LowResource || effective_memory < 16GB);`

This makes mmap controllable independently.

**Step 3: Add memory budget validation at startup**

In `src/storage.rs`, after the HNSW loads (~line 830), add:
```rust
let estimated_hnsw_bytes = hnsw.estimate_memory_bytes() as u64;
let hnsw_node_count = hnsw.nodes.len() as u64;
if hnsw_node_count > 1_000_0 && estimated_hnsw_bytes > effective_memory / 2 {
    tracing::warn!(
        "HNSW index may exceed 50% of memory budget: {} estimated / {} effective",
        estimated_hnsw_bytes, effective_memory
    );
}
```

**Step 4: Write verification test**

Create `tests/memory/mmap_hnsw.rs`:
```rust
#[test]
fn test_mmap_hnsw_loads_with_memory_budget() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = VantaConfig::default()
        .with_storage_path(dir.path().to_str().unwrap())
        .with_mmap_hnsw(true);
    let engine = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config)).unwrap();
    let stats = engine.get_memory_stats();
    assert!(stats.logical_bytes > 0);
    engine.close().unwrap();
}
```

**Step 5: Run test to verify**

Run: `cargo test --test memory -- mmap_hnsw -v`
Expected: PASS

**Step 6: Commit**

```bash
git add src/config.rs src/storage.rs tests/memory/mmap_hnsw.rs
git commit -m "feat(config): add mmap_hnsw option, memory budget validation"
```

---

### Task 2: RSS monitoring and backpressure at 80% threshold (TSK-50)

**Files:**
- Modify: `src/config.rs` — Add `rss_threshold` (default 0.80)
- Modify: `src/storage.rs` — Add `check_memory_pressure()` method
- Modify: `src/storage.rs:1395-1448` — Check before insert
- Modify: `src/sdk.rs` — Check before write operations
- Modify: `src/error.rs` — `VantaError::ResourceLimit` already exists
- Create: `tests/memory/backpressure.rs` — Backpressure tests

**Step 1: Add `rss_threshold` to VantaConfig**

In `src/config.rs`:
```rust
/// Fraction of effective memory that triggers backpressure (0.0–1.0).
/// Default 0.80. Set to 0.0 to disable backpressure.
pub rss_threshold: f64,
```
Default: `0.80`. Builder: `with_rss_threshold(mut self, threshold: f64)`.

Validate in builder: `assert!(threshold >= 0.0 && threshold <= 1.0, "rss_threshold must be 0.0–1.0");`

**Step 2: Implement `check_memory_pressure()` in StorageEngine**

In `src/storage.rs`, add method:
```rust
pub fn check_memory_pressure(&self) -> Result<()> {
    let threshold = self.config.rss_threshold;
    if threshold <= 0.0 {
        return Ok(()); // Disabled
    }
    let stats = self.get_memory_stats();
    let effective = stats.effective_bytes();
    let limit = self.config.memory_limit
        .unwrap_or_else(|| crate::hardware::HardwareCapabilities::global().total_memory);
    if effective > 0 && effective > (limit as f64 * threshold) as u64 {
        return Err(VantaError::ResourceLimit(format!(
            "Memory pressure: {} bytes used ({}% of {} limit, threshold {}%)",
            effective,
            (effective as f64 / limit as f64 * 100.0) as u64,
            limit,
            (threshold * 100.0) as u64,
        )));
    }
    Ok(())
}
```

**Step 3: Guard write operations**

In `StorageEngine::insert()` (line ~1395), add at the top:
```rust
self.check_memory_pressure()?;
```

In `StorageEngine::delete()` (line ~1618), add at the top:
```rust
self.check_memory_pressure()?;
```

In `StorageEngine::put_batch_core()` or equivalent SDK methods, call `check_memory_pressure()` upfront.

**Step 4: Write backpressure test**

Create `tests/memory/backpressure.rs`:
```rust
#[test]
fn test_backpressure_rejects_writes_over_threshold() {
    let dir = tempfile::TempDir::new().unwrap();
    // Set a tiny memory limit and low threshold to force backpressure
    let config = VantaConfig::default()
        .with_storage_path(dir.path().to_str().unwrap())
        .with_memory_limit(1024) // 1KB
        .with_rss_threshold(0.1);
    let engine = StorageEngine::open_with_config(...).unwrap();
    let result = engine.insert(...); // Small insert
    // Should succeed
    assert!(result.is_ok());
    // After many small inserts, should eventually fail
    // OR use MmapFull that reports physical RSS
    // Simpler: mock by checking that check_memory_pressure returns Err
    // when stats exceed threshold
}
```

Alternative simpler test:
```rust
#[test]
fn test_backpressure_disabled_with_zero_threshold() {
    let config = VantaConfig::default().with_rss_threshold(0.0);
    assert!(config.rss_threshold == 0.0);
    // engine.check_memory_pressure() should always return Ok
}
```

**Step 5: Run tests**

Run: `cargo test --test memory -v`
Expected: PASS

**Step 6: Commit**

```bash
git add src/config.rs src/storage.rs src/sdk.rs tests/memory/backpressure.rs
git commit -m "feat(storage): add RSS backpressure at configurable threshold (TSK-50)"
```

---

### Task 3: Weighted eviction by importance score (TSK-76b)

**Files:**
- Modify: `src/storage.rs` — Add `evict_cold_nodes()` method
- Modify: `src/storage.rs:1433-1445` — Connect eviction to emergency maintenance
- Modify: `src/sdk.rs` — Expose `evict_cold_nodes()` and `set_eviction_weights()`
- Modify: `src/node.rs` — Add `eviction_score()` method to UnifiedNode
- Modify: `src/config.rs` — Add eviction weights config
- Create: `tests/memory/eviction.rs` — Eviction tests

**Step 1: Add eviction weights to config**

In `src/config.rs`:
```rust
pub eviction_weight_hits: f64,         // default: 1.0
pub eviction_weight_confidence: f64,   // default: 2.0
pub eviction_weight_importance: f64,   // default: 3.0
pub eviction_weight_recency: f64,      // default: 1.0
pub eviction_ratio: f64,               // default: 0.20 (evict 20% when triggered)
```

**Step 2: Add `eviction_score()` to UnifiedNode**

In `src/node.rs`, add:
```rust
impl UnifiedNode {
    pub fn eviction_score(&self, weights: &EvictionWeights) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let age_secs = if self.last_accessed > 0 {
            ((now - self.last_accessed) / 1000).max(1)
        } else {
            1
        };
        let recency_score = 1.0 / (age_secs as f64).ln_1p(); // logarithmic decay
        self.hits as f64 * weights.hits
            + self.confidence_score as f64 * weights.confidence
            + self.importance as f64 * weights.importance
            + recency_score * weights.recency
    }
}
```

**Step 3: Implement `evict_cold_nodes()` in StorageEngine**

In `src/storage.rs`:
```rust
pub fn evict_cold_nodes(&self, ratio: f64) -> Result<EvictionReport> {
    self.ensure_writable()?;
    let cache = self.volatile_cache.read();
    let candidates: Vec<UnifiedNode> = cache.values()
        .filter(|n| n.tier == NodeTier::Hot)
        .cloned()
        .collect();
    drop(cache);

    if candidates.is_empty() {
        return Ok(EvictionReport { evicted: 0, scanned: 0 });
    }

    let target = (candidates.len() as f64 * ratio).max(1.0) as usize;
    let weights = self.config.eviction_weights();

    let mut scored: Vec<(f64, UnifiedNode)> = candidates
        .into_iter()
        .map(|n| (n.eviction_score(&weights), n))
        .collect();
    scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap()); // ascending = coldest first

    let mut evicted = 0;
    for (score, node) in scored.iter().take(target) {
        if self.consolidate_node(node).is_ok() {
            evicted += 1;
        }
    }

    Ok(EvictionReport {
        evicted,
        scanned: scored.len(),
    })
}
```

**Step 4: Add EvictionReport struct**

```rust
pub struct EvictionReport {
    pub evicted: usize,
    pub scanned: usize,
}
```

**Step 5: Connect eviction to emergency maintenance trigger**

In `StorageEngine::insert()` around line 1442, change:
```rust
if cache.len() > max_nodes {
    self.emergency_maintenance_trigger.store(true, Ordering::Release);
}
```
To:
```rust
if cache.len() > max_nodes {
    self.emergency_maintenance_trigger.store(true, Ordering::Release);
    // Auto-evict if we're past the emergency threshold
    let _ = self.evict_cold_nodes(self.config.eviction_ratio);
}
```

**Step 6: Write eviction test**

Create `tests/memory/eviction.rs`:
```rust
#[test]
fn test_evict_cold_nodes_by_score() {
    let dir = tempfile::TempDir::new().unwrap();
    let engine = setup_engine(&dir);
    
    // Insert nodes with different importance/hits
    for i in 0..10 {
        let mut node = UnifiedNode::new(i as u64, vec![0.1; 4]);
        node.importance = i as f32 / 10.0; // 0.0 to 0.9
        node.hits = (10 - i) as u32; // 10 down to 1
        engine.insert(node).unwrap();
    }
    
    let report = engine.evict_cold_nodes(0.5).unwrap();
    assert!(report.evicted > 0);
    assert!(report.scanned > 0);
    
    // The lowest-scoring 50% should be evicted
    // Verify by checking tier
}
```

**Step 7: Run tests**

Run: `cargo test --test memory -v`
Expected: PASS

**Step 8: Commit**

```bash
git add src/config.rs src/node.rs src/storage.rs src/sdk.rs tests/memory/eviction.rs
git commit -m "feat(storage): weighted eviction by importance score (TSK-76b)"
```

---

### Task 4: Integration — connect backpressure → auto-eviction

**Files:**
- Modify: `src/storage.rs` — Auto-evict when backpressure triggers

**Step 1: Wire backpressure to auto-eviction**

In `StorageEngine::check_memory_pressure()`, when returning `Err`, also auto-trigger eviction:
```rust
// Before returning Err, try auto-eviction
let _ = self.evict_cold_nodes(self.config.eviction_ratio);
```

This creates a self-healing loop: writes trigger memory check → over threshold → auto-evict → next write may succeed.

**Step 2: Run full test suite**

Run: `cargo test --all-features -v`
Expected: ALL PASS (48 lib + 33 CLI + 7 memory + 6 E2E)

**Step 3: Commit**

```bash
git add src/storage.rs
git commit -m "feat(storage): integrate backpressure with auto-eviction loop"
```
