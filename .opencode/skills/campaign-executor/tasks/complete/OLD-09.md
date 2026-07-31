# OLD-09: Olvido Bayesiano (Bayesian Hit Decay)

**Fuente:** Backlog Phase 9 (Old Docs Rescue)  
**Estado:** ⚠️ Parcial  
**Effort:** 🟡 3-4d  
**Dependencias:** Ninguna. `EvictionPolicy` existe en `src/eviction.rs`

## Gate
✅ DO — `EvictionPolicy` ya implementa hit counts + recency weights. Falta el decay bayesiano formal: modelo Beta-Binomial para inferir probabilidad de re-uso futuro dado historial de hits.

## Objetivo
Implementar Bayesian Hit Decay en `EvictionPolicy`:
- Modelar hits como distribución Beta(α, β) donde α = hits + 1, β = time_since_last_hit + 1
- Score bayesiano = E[Beta(α, β)] = α / (α + β) = probabilidad posterior de re-uso
- Usar eviction basada en score vs. umbral configurable
- Feature-gated detrás de `bayesian_decay`

## Archivos

| Archivo | Qué hacer |
|---------|-----------|
| `src/eviction.rs` | Agregar `BayesianDecay` struct + scoring + eviction |
| `Cargo.toml` | Feature flag `bayesian_decay` |
| `src/config.rs` | Opción `eviction.bayesian_threshold` |
| Tests | Unit tests + integration con `EvictionPolicy` |

## Pasos

### 1. Entender EvictionPolicy actual
Leer `src/eviction.rs` — entender cómo funciona hit count + recency weight actual.

### 2. Agregar BayesianDecay
```rust
pub struct BayesianDecay {
    prior_alpha: f64,  // default 1.0
    prior_beta: f64,   // default 1.0
    threshold: f64,    // default 0.3, score por debajo = candidate
}

impl BayesianDecay {
    /// Posterior score given hits and time since last hit (seconds).
    pub fn score(&self, hits: u64, seconds_since_last_hit: f64) -> f64 {
        let alpha = self.prior_alpha + hits as f64;
        let beta = self.prior_beta + seconds_since_last_hit.max(0.0);
        alpha / (alpha + beta)
    }
    
    pub fn should_evict(&self, score: f64) -> bool {
        score < self.threshold
    }
}
```

### 3. Integrar con EvictionPolicy
Que el `EvictionPolicy` existente pueda usar `BayesianDecay` como scorer alternativo cuando la feature está activa.

### 4. Tests
```rust
#[test]
fn test_bayesian_score_high_hits() {
    let decay = BayesianDecay::default();
    let score = decay.score(100, 3600.0); // 100 hits, 1h ago
    assert!(score > 0.9);
}

#[test]
fn test_bayesian_score_no_hits() {
    let decay = BayesianDecay::default();
    let score = decay.score(0, 86400.0); // 0 hits, 1d ago
    assert!(score < 0.5);
}

#[test]
fn test_bayesian_eviction_threshold() {
    let decay = BayesianDecay { threshold: 0.3, ..Default::default() };
    assert!(decay.should_evict(0.2));
    assert!(!decay.should_evict(0.5));
}
```

### 5. Verificación
```bash
cargo check --features bayesian_decay -p vantadb
cargo nextest run --features bayesian_decay -p vantadb -- eviction::bayesian
cargo clippy --features bayesian_decay -p vantadb -- -D warnings
```

### 6. Progreso
- Marcar OLD-09 ✅ en Backlog.md
- Agregar entry en progreso/README.md
- Auto-commit
