# ConnectomeDB — Future Development Specifications

This directory contains **complete technical designs** for features that have been architecturally validated but **deferred** from the current release cycle.

Each document is a self-contained specification ready for implementation when the project reaches the appropriate maturity level.

## Contents

| Document | Feature | Prerequisite Version |
|---|---|---|
| `electric_vs_chemical_synapse.md` | Hardware-aware Edge resolution (RAM ptr vs disk I/O) | v0.3.0+ |
| `epistemological_engine.md` | Bayesian Trust Scoring, Uncertainty Zones, Dissonance Motor | v0.3.0+ |
| `shadow_kernel.md` | Dual-truth layer per Neuron (Consensus vs Narrative) | v0.3.0+ (opt-in feature flag) |
| `cognitive_governance.md` | Entropy-based pruning, Shadow Archive, Semantic Apoptosis | v0.3.0+ |

## Policy

- These specs are **living documents** — update them as the core engine evolves.
- Before implementing any feature here, verify alignment with the current `src/` architecture.
- Each feature should have its own `Cargo.toml` feature flag when implemented.
