# VantaDB - Agent Instructions

Welcome to the VantaDB codebase.

If you are an AI assistant, an LLM, or an autonomous coding agent operating within this repository, you must strictly adhere to the following governance protocols:

## 1. No Biological Metaphors

VantaDB was originally a prototype riddled with overhyped conceptual language ("neurons", "synapses", "cognitive architecture"). We have explicitly **purged** all biological and pseudo-neural metaphors.

- **DO NOT** use words like `neuron`, `synapse`, `brain`, `cognitive`, `hallucination`, `dream`, or `immune system`.
- **INSTEAD**, use mathematically and technically descriptive terms: `node`, `edge`, `vector index`, `background worker`, `invalidation mechanism`, `garbage collection`.
- We hold ourselves to professional database engineering standards.

## 2. Technical Honesty & Precision

- Never promise impossible O(1) complexities for high-dimensional search.
- When generating documentation, clarify standard algorithms used (e.g., standard HNSW, Memory-Mapped persistence).
- Do not add "hype" adjectives to pull requests or commit messages.

## 3. Architecture Overview

VantaDB is a Rust-based, embedded, zero-copy multimodel database engine.

- **Data Model:** `UnifiedNode` contains an ID, a dense `f32` vector, relations, and outward edge lists.
- **Index:** `CPIndex` implements the `HNSW` algorithm. It uses a graph layout pinned via `mmap` if persistent.
- **C-ABI / Python:** We export a subset of functionalities through `src/engine.rs` exposing a C-ABI layer which is consumed by `vantadb-python`.

## 4. Stability

- Always compile and run `cargo check` / `cargo test` when proposing changes.
- Ensure that modifications to core index algorithms do not break the tests in `tests/certification/stress_protocol.rs`. Target recall > 90% is non-negotiable.
