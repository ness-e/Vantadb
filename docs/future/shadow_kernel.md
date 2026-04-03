# Shadow Kernel — Dual-Truth Layer per Neuron

> **Status:** DEFERRED (v0.3.0+)  
> **Decision:** MUST be opt-in feature flag (`shadow_kernel` in Cargo.toml). Duplicates RAM per node from ~6.5KB to ~13KB. Not viable as default on 16GB hardware target.

---

## 1. Concept

Each Neuron internally maintains two parallel views of its data:

| Layer | Purpose | Mutability | Source |
|---|---|---|---|
| **Consensus Layer** | Objective, sensor-verified truth | Read-Only (only updated by system/sensors) | Axioms, confirmed data, verified sources |
| **Narrative Layer** | User-modified, subjective truth | Read-Write (user can modify freely) | User edits, LLM inferences, manual overrides |

When queried, the system can compare both layers to detect **drift** — how far the user's narrative has deviated from consensus reality.

---

## 2. Proposed Struct Changes

```rust
/// Shadow Kernel: dual truth representation
/// WARNING: Approximately doubles memory per Neuron (~6.5KB → ~13KB)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShadowKernel {
    /// Consensus Layer (Read-Only): ground truth from sensors/axioms
    pub consensus: RelFields,
    /// Narrative Layer (Read-Write): user-modified view
    pub narrative: RelFields,
    /// Drift score: 0.0 = identical, 1.0 = completely diverged
    pub drift_score: f32,
    /// When was drift last calculated
    pub drift_calculated_at: u64,
}

impl ShadowKernel {
    pub fn new() -> Self {
        Self {
            consensus: BTreeMap::new(),
            narrative: BTreeMap::new(),
            drift_score: 0.0,
            drift_calculated_at: 0,
        }
    }

    /// Calculate drift between consensus and narrative
    pub fn calculate_drift(&mut self) -> f32 {
        let total_fields = self.consensus.len().max(self.narrative.len());
        if total_fields == 0 {
            self.drift_score = 0.0;
            return 0.0;
        }

        let mut disagreements = 0;
        for (key, consensus_value) in &self.consensus {
            match self.narrative.get(key) {
                Some(narrative_value) if narrative_value != consensus_value => {
                    disagreements += 1;
                }
                None => disagreements += 1,  // Field removed from narrative
                _ => {}
            }
        }
        // Fields added to narrative that don't exist in consensus
        for key in self.narrative.keys() {
            if !self.consensus.contains_key(key) {
                disagreements += 1;
            }
        }

        self.drift_score = disagreements as f32 / total_fields as f32;
        self.drift_calculated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.drift_score
    }

    /// Apply a user modification (only affects narrative layer)
    pub fn user_modify(&mut self, key: String, value: FieldValue) {
        self.narrative.insert(key, value);
    }

    /// Apply a system/sensor update (updates consensus, optionally syncs narrative)
    pub fn system_update(&mut self, key: String, value: FieldValue, sync_narrative: bool) {
        self.consensus.insert(key.clone(), value.clone());
        if sync_narrative {
            self.narrative.insert(key, value);
        }
    }
}
```

### Modified Neuron (when feature enabled):

```rust
#[cfg(feature = "shadow_kernel")]
pub struct Neuron {
    // ... existing fields ...
    pub shadow: Option<ShadowKernel>,  // None = disabled for this neuron
}

#[cfg(not(feature = "shadow_kernel"))]
pub struct Neuron {
    // ... existing fields (no shadow) ...
}
```

---

## 3. RAM Impact Analysis

| Scenario | Without Shadow | With Shadow (100%) | With Shadow (10%) |
|---|---|---|---|
| 100K nodes | 650 MB | 1,300 MB | 715 MB |
| 500K nodes | 3.25 GB | 6.50 GB | 3.58 GB |
| 1M nodes | 6.50 GB | 13.00 GB | 7.15 GB |

> **Critical constraint:** At 1M nodes with shadow enabled for ALL nodes, RAM consumption reaches 13GB — leaving only 3GB for OS, HNSW index, and Ollama. This is UNACCEPTABLE for the 16GB target.

**Mitigation strategies:**
1. Shadow is `Option<ShadowKernel>` — NOT all neurons need it
2. Auto-enable only for neurons with `trust_score < 0.7` (contested data)
3. Maximum shadow population: 10% of total neurons (configurable)
4. GC can strip shadow from neurons with `drift_score == 0.0` (no divergence)

---

## 4. Query Interface

```sql
-- Compare consensus vs narrative for a specific neuron
FROM Persona#42 FETCH shadow.consensus.pais, shadow.narrative.pais, shadow.drift_score

-- Find neurons with high drift (user has significantly modified sensor data)
FROM Persona WHERE shadow.drift_score > 0.5

-- Force query to use only consensus layer (ignore user modifications)
FROM Persona WHERE pais = "VE" WITH LAYER consensus
```

---

## 5. Cargo Feature Flag

```toml
[features]
default = ["core"]
shadow_kernel = []  # Enables dual-truth ShadowKernel per Neuron
```

When `shadow_kernel` is disabled:
- `ShadowKernel` struct doesn't compile
- No RAM overhead
- All queries operate on `neuron.relational` directly (current behavior)

---

## 6. Implementation Roadmap

```
Phase 1: Add ShadowKernel struct behind #[cfg(feature = "shadow_kernel")]
Phase 2: Modify Neuron struct with conditional compilation
Phase 3: Add drift_score calculation in write path (only when shadow exists)
Phase 4: IQL syntax for LAYER consensus/narrative and shadow.* fields
Phase 5: GC integration — strip empty shadows, cap total shadow population
Phase 6: Dashboard metric: connectome_shadow_drift_avg gauge
```

---

## 7. Relationship to Other Future Features

- **Epistemological Engine:** Shadow Kernel provides the data layer that the Trust Arbiter uses to detect contradictions
- **Cognitive Governance:** Drift score feeds into the multivariable valuation algorithm for neuron importance
- **Axiom Engine:** Consensus layer is where Iron Axioms enforce their invariants
