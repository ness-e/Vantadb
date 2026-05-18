# Contributing to VantaDB

We welcome contributions to VantaDB! Our goal is to build a high-performance embedded multimodel
database without marketing overhead.

## Engineering Philosophy

1. **Precision & Consistency:** We use standard terminology. Avoid biological namespaces or
   exaggerated descriptors.
2. **Deterministic Debugging:** All core additions must have accompanying validation scripts
   utilizing brute-force validation (e.g., recall tests) if they involve statistical modeling or
   approximated distances.
3. **Rust Tooling:** The project utilizes standard `cargo` toolchains. Ensure code is locked to
   `stable`.

## CI Pipeline Policy

VantaDB uses a **two-tier CI strategy** to balance PR velocity with comprehensive coverage:

| Tier                                                | Trigger                 | Timeout | Scope                                                                |
| --------------------------------------------------- | ----------------------- | ------- | -------------------------------------------------------------------- |
| **Fast Gate** (`rust_ci.yml`)                       | Every push/PR to `main` | 30 min  | fmt, clippy, unit tests, integration tests                           |
| **Heavy Certification** (`heavy_certification.yml`) | Manual / scheduled      | 60 min  | stress_protocol, hnsw_validation, sift_validation, competitive_bench |

### Fast Gate Rules

- Keep it deterministic and offline.
- Do not add external network calls, remote services, Ollama requirements, dataset downloads, or
  Docker-only dependencies to this lane.
- If a check is slow, resource-heavy, or requires special infrastructure, it belongs in heavy
  certification, not in the Fast Gate.

### Heavy Certification Rules

- Use this lane for `stress_protocol`, HNSW recall/certification work, SIFT validation, and
  competitive benchmarks.
- Do not expand the Fast Gate to cover long-running certification jobs just to validate a specialist
  change.

For full details, see [`docs/operations/CI_POLICY.md`](docs/operations/CI_POLICY.md).

## Submitting Pull Requests

1. Fork the repository and formulate your changes.
2. Keep pull requests focused. Separate docs-only, test-only, and product-code changes when
   possible.
3. Run the minimum local validation expected by the Fast Gate:
   - `cargo fmt --check`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo nextest run --profile audit --workspace`
   - `cargo bench --no-run`
4. If your change touches Python packaging, build the local binding and run the SDK tests in an
   isolated environment.
5. If your change touches heavier paths, document why and validate them through the appropriate
   manual or scheduled certification workflow instead of extending the Fast Gate.
6. Include an objective breakdown of metric changes if optimizing algorithmic paths.
7. Target the `main` branch. Branch protection requires the Fast Gate checks to pass before merge.

## Pull Request Checklist

- The PR description explains what changed, why it changed, and how it was validated.
- Public API, SDK, CLI, or documentation changes are reflected in the relevant docs.
- New behavior includes focused tests or a clear reason tests were not added.
- Heavy benchmarks or certification tests are not moved into the Fast Gate without justification.
- Generated data, local databases, build artifacts, and private logs are not committed.

## Commit Messages

Use concise Conventional Commit style when possible:

- `feat: add memory export command`
- `fix: repair stale text index cleanup`
- `docs: clarify Python SDK install status`
- `test: cover hybrid search filter isolation`

## Reporting Issues

Use the provided Issue Templates:

- **Bug Report** — for crashes, incorrect results, or regressions.
- **Feature Request** — for new capabilities or API changes.
- **Documentation Issue** — for unclear, stale, or missing docs.

We look forward to reviewing your additions.
