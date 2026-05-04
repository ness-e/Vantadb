# VantaDB Test Report - 2026-05-04

## Summary

Audit environment: Windows, PowerShell, `main`, no git staging/commit/push/reset.

`cargo-nextest` was installed and `.config/nextest.toml` now defines an `audit` profile. The profile excludes the three long-running certification tests that timed out in the first full run, so the agreed command is now usable:

```powershell
cargo nextest run --profile audit --workspace
```

## Commands

| Command | Result | Notes |
|---|---:|---|
| `npx skills add laurigates/claude-plugins@cargo-nextest -g -y` | PASS | Installed `cargo-nextest` skill. |
| `npx skills add naodeng/awesome-qa-skills@test-reporting -g -y` | PASS | Installed `test-reporting` skill. |
| `cargo install cargo-nextest --locked` | PASS | Installed `cargo-nextest 0.9.133`. |
| `cargo fmt --check` | FAIL | Formatting drift in `benches/hybrid_queries.rs` and `src/sdk.rs`. No formatter was run. |
| `cargo clippy --all-targets --all-features -- -D warnings` | FAIL | Three lint errors: `ClientEngine::new` without `Default`, `manual_clamp`, PyO3 `non_local_definitions`. |
| `cargo test --lib --bins` | PASS | 16 passed. |
| `cargo nextest run --profile audit --workspace` before filter | FAIL | 97 passed, 3 timed out, 1 skipped. |
| `cargo bench --no-run` | PASS | All bench binaries compiled. |
| `cargo nextest run --profile audit --workspace` after filter | PASS | 97 passed, 4 skipped. |
| `python -m pytest vantadb-python\tests\test_sdk.py -q` | FAIL | Global installed package failed 1/17 on phrase query. |
| `maturin develop --release` in global Python | FAIL | No virtualenv/conda active. |
| `target\audit-venv` + `maturin develop --release` + `pytest` | PASS | Current source passed 17/17 in isolated venv. |

## Findings

- P1: The global Python environment has a stale or desynchronized `vantadb-py` install. It failed exact phrase search, while the same source built in `target/audit-venv` passed 17/17.
- P1: The default all-workspace test set includes certification tests that are too heavy for a normal audit gate. The `audit` profile now excludes `sift1m_competitive_benchmark`, `hnsw_hard_validation_certification`, and `stress_protocol_certification`.
- P2: `cargo fmt --check` fails because user changes need formatting. Do not run `cargo fmt` until those edits are ready to normalize.
- P2: `clippy --all-features` fails on `src/python.rs` and `src/sdk.rs`; these are real lint-gate blockers before CI can enforce `-D warnings`.
- P2: The root `test_runner.sh` depends on `/app`, `apt-get`, and an active venv flow. It is useful as container evidence, but should not be the canonical local Windows SDK test command.

## Recommended Next Steps

1. Keep `cargo nextest run --profile audit --workspace` as the local fast audit gate.
2. Keep heavy certification in `.github/workflows/heavy_certification.yml` or an explicit `nextest` certification profile.
3. Add a documented Python SDK validation command that always creates/uses a venv before `maturin develop`.
4. Fix lint/format in a separate implementation phase after preserving the current user edits.
