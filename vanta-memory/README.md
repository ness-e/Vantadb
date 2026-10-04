# vanta-memory

Cognitive memory layer for VantaDB: L0 capture → L1 extraction/dedup → L2 scenes → L3 persona, recall with scope, context engine (compression), offload, and dream consolidation.

> **Full API reference:** [`docs/api/VANTA_MEMORY.md`](../docs/api/VANTA_MEMORY.md) — canonical documentation (this README is a pointer, not a copy).

**Distribution status:** publishable (crates.io metadata ready; `cargo publish --dry-run` green + external smoke verified) with an explicit **hold** in `release-plz.toml` until the first publish is bootstrapped — crates.io Trusted Publishing requires the crate to exist first (bootstrap checklist in the hold entry; rationale in `docs/dev/tasks/DIST-01.md`).

**Binding scope:** Python exposes a minimal surface (`memory_capture` / `memory_recall`); TS/Node/WASM are declared core-only — see [`docs/api/BINDINGS_NAMESPACES.md`](../docs/api/BINDINGS_NAMESPACES.md) §Cognitive layer scope per binding.
