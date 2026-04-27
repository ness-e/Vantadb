# Contributing to VantaDB

We welcome contributions to VantaDB! Our goal is to build a high-performance embedded multimodel database without marketing overhead.

## Engineering Philosophy

1. **Precision & Consistency:** We use standard terminology. Avoid biological namespaces or exaggerated descriptors.
2. **Deterministic Debugging:** All core additions must have accompanying validation scripts utilizing brute-force validation (e.g., recall tests) if they involve statistical modeling or approximated distances.
3. **Rust Tooling:** The project utilizes standard `cargo` toolchains. Ensure code is locked to `stable`.

## CI Pipeline Policy

VantaDB uses a **two-tier CI strategy** to balance PR velocity with comprehensive coverage:

| Tier | Trigger | Timeout | Scope |
|------|---------|---------|-------|
| **Fast Gate** (`rust_ci.yml`) | Every push/PR to `main` | 30 min | fmt, clippy, unit tests, integration tests |
| **Heavy Certification** (`heavy_certification.yml`) | Manual / scheduled | 60 min | stress_protocol, hnsw_validation, sift_validation, competitive_bench |

> **Important:** Do not add network-dependent tests or large dataset downloads to the Fast Gate. If your contribution includes a heavy benchmark or stress test, target `heavy_certification.yml` instead.

For full details, see [`docs/operations/CI_POLICY.md`](docs/operations/CI_POLICY.md).

## Submitting Pull Requests

1. Fork the repository and formulate your changes.
2. Ensure you have run:
   - `cargo fmt --check`
   - `cargo clippy -- -D warnings`
   - `cargo test --release`
3. Include an objective breakdown of metric changes if optimizing algorithmic paths.
4. Target the `main` branch. Branch protection requires status checks to pass before merge.

## Reporting Issues

Use the provided Issue Templates:

- **Bug Report** — for crashes, incorrect results, or regressions.
- **Feature Request** — for new capabilities or API changes.

We look forward to reviewing your additions.
