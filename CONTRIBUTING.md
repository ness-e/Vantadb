# Contributing to VantaDB

We welcome contributions to VantaDB! Our goal is to build a high-performance embedded multimodel database without marketing overhead.

## Engineering Philosophy

1. **Precision & Consistency:** We use standard terminology. Avoid biological namespaces or exaggerated descriptors.
2. **Deterministic Debugging:** All core additions must have accompanying validation scripts utilizing brute-force validation (e.g., recall tests) if they involve statistical modeling or approximated distances.
3. **Rust Tooling:** The project utilizes standard `cargo` toolchains. Ensure code is locked to `stable`.

## Submitting Pull Requests

1. Fork the repository and formulate your changes.
2. Ensure you have run:
   - `cargo fmt --check`
   - `cargo clippy -- -D warnings`
   - `cargo test --release`
3. Include an objective breakdown of metric changes if optimizing algorithmic paths.

We look forward to reviewing your additions.
