# Epistemological Engine — Bayesian Trust & Semantic Archaeology

> **Status:** DEFERRED (v0.3.0+)  
> **Decision:** Static trust score field approved for v0.3.0. Full Bayesian arbiter and Dissonance Motor deferred due to over-engineering risk on 16GB hardware target.

---

## 1. Problem Statement

In a local-first AI database contaminated by LLM hallucinations, user errors, and noisy sensor inputs, the system needs mechanisms to:

1. **Quantify confidence** in each piece of stored knowledge
2. **Detect contradictions** between new and existing data
3. **Separate trusted facts from inferred opinions**
4. **Recover historical context** without polluting active inference

---

## 2. Component Architecture

### 2.1 Source Trust Score (Bayesian Confidence)

Each Neuron receives a trust score based on its origin:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SourceOrigin {
    System,      // Hard-coded axioms, system bootstrap data
    Sensor,      // Direct sensor input (IoT, API webhooks)
    User,        // Human-entered data
    Inferred,    // LLM-generated, agent-derived
    Archived,    // Recovered from Shadow Archive
}

impl SourceOrigin {
    pub fn default_trust(&self) -> f32 {
        match self {
            SourceOrigin::System   => 0.95,
            SourceOrigin::Sensor   => 0.85,
            SourceOrigin::User     => 0.70,
            SourceOrigin::Inferred => 0.50,
            SourceOrigin::Archived => 0.30,
        }
    }
}
```

**Fields to add to Neuron:**
```rust
pub struct Neuron {
    // ... existing fields ...
    pub source: SourceOrigin,
    pub trust_score: f32,          // 0.0 - 1.0, initialized from source.default_trust()
    pub trust_adjustments: u32,    // How many times the score has been updated
}
```

### 2.2 Predictive Trust Arbitration (Bayesian Update)

When a new fact conflicts with an existing one, update trust using Bayes' theorem:

```
P(trust | evidence) = P(evidence | trust) × P(trust) / P(evidence)
```

**Simplified implementation:**
```rust
pub fn bayesian_update(prior_trust: f32, evidence_strength: f32, agreement: bool) -> f32 {
    let likelihood = if agreement {
        evidence_strength
    } else {
        1.0 - evidence_strength
    };
    let posterior = prior_trust * likelihood;
    let normalizer = posterior + (1.0 - prior_trust) * (1.0 - likelihood);
    if normalizer < f32::EPSILON {
        prior_trust // Avoid division by zero
    } else {
        (posterior / normalizer).clamp(0.01, 0.99)
    }
}
```

### 2.3 Active Uncertainty Zone (AIZ)

When two high-confidence Neurons contradict each other:

```rust
#[derive(Clone, Debug)]
pub struct UncertaintyZone {
    pub neuron_a: u64,
    pub neuron_b: u64,
    pub conflict_field: String,
    pub trust_a: f32,
    pub trust_b: f32,
    pub created_at: u64,
    pub resolution: Option<AIZResolution>,
}

pub enum AIZResolution {
    UserResolved { winner: u64, timestamp: u64 },
    AutoResolved { winner: u64, method: String },
    Superseded { by_neuron: u64 },
}
```

**Rules:**
1. If `|trust_a - trust_b| > 0.3`, auto-resolve in favor of higher trust
2. If `|trust_a - trust_b| <= 0.3`, enter superposition (suspend auto-inference)
3. Inject warning into QueryResult metadata when queried
4. User can manually resolve via `RESOLVE CONFLICT#123 WINNER NODE#456`

### 2.4 Semantic Archaeology (Recovery Tiers)

```
┌─────────────────────────────┐
│     Active Lobe             │  ← Hot: instant queries, no noise
│     (Current truth)         │
├─────────────────────────────┤
│     Historical Vault        │  ← Warm: on-demand deep search
│     (Versioned snapshots)   │
├─────────────────────────────┤
│     Shadow Archive          │  ← Cold: glacial storage (see shadow_kernel.md)
│     (Pruned data ghosts)    │
└─────────────────────────────┘
```

**Activation:** Only query Historical Vault when:
- Active Lobe returns 0 results AND user explicitly requests `WITH ARCHAEOLOGY`
- Example: `FROM Persona WHERE bio ~ "rust" WITH ARCHAEOLOGY DEPTH 3`

### 2.5 Anachronism Warning

When archaeological data contradicts active data:

```rust
pub struct AnachronismWarning {
    pub active_neuron: u64,
    pub archived_neuron: u64,
    pub field: String,
    pub active_value: FieldValue,
    pub archived_value: FieldValue,
    pub time_delta_secs: u64,
}
```

Injected into `QueryResult` metadata:
```rust
pub struct QueryResult {
    pub nodes: Vec<Neuron>,
    pub is_partial: bool,
    pub exhaustivity: f32,
    pub source_type: SourceType,
    pub warnings: Vec<AnachronismWarning>,  // NEW
    pub uncertainty_zones: Vec<UncertaintyZone>,  // NEW
}
```

### 2.6 Dissonance Motor (Devil's Advocate)

Background audit process that:
1. Runs after every N writes (configurable, default: 100)
2. Checks if the modified Neuron's relational fields are logically consistent with its graph neighbors
3. Emits Prometheus metric `connectome_dissonance_alerts_total`
4. Example: If Neuron has `country: "Venezuela"` but all graph neighbors have `country: "Colombia"`, flag as potential data entry error

```rust
pub struct DissonanceAlert {
    pub neuron_id: u64,
    pub field: String,
    pub neuron_value: FieldValue,
    pub neighbor_consensus: FieldValue,
    pub confidence: f32,  // How strong is the disagreement
    pub timestamp: u64,
}
```

---

## 3. Implementation Phases

```
Phase 1 (v0.3.0): Add source + trust_score fields to Neuron. Static initial values.
Phase 2 (v0.3.0): AnachronismWarning in QueryResult.
Phase 3 (v0.4.0): Bayesian update function. AIZ detection on write.
Phase 4 (v0.4.0): Dissonance Motor as background tokio::spawn task.
Phase 5 (v0.5.0): Full Semantic Archaeology with `WITH ARCHAEOLOGY` IQL syntax.
```

---

## 4. Cargo Feature Flag

```toml
[features]
epistemological = []  # Enables trust scoring, AIZ, dissonance motor
```

When disabled, trust_score defaults to 1.0 and all epistemological checks are no-ops.

---

## 5. Risk Assessment

| Risk | Severity | Mitigation |
|---|---|---|
| CPU overhead from Bayesian updates on every write | Medium | Only run on conflicting writes, not all writes |
| RAM overhead from storing UncertaintyZones | Low | Capped at 1000 active zones, oldest auto-resolved |
| Complexity for new contributors | High | Entire system behind feature flag, disabled by default |
| Over-fitting trust scores | Medium | Floor at 0.01, ceiling at 0.99, reset mechanism |
