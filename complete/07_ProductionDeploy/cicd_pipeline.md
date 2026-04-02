# Continuous Integration & Deployment (CI/CD)
> **Status**: 🟡 In Progress — FASE 5

## 1. Automated Pipeline (GitHub Actions)
Each commit pushed per our strict Git Pipeline rules must pass:
1. **Compilation Check**: `cargo build`
2. **Unit Tests**: Full `cargo test` covering storage, parsers, and node logic.
3. **Lints & Formats**: Enforcing `rustfmt` and `clippy`.

## 2. Release Automation
On creating a new GitHub semantic tag (`v0.2.0`), a dedicated pipeline will compile release binaries for:
- `x86_64-unknown-linux-gnu` (Server/Docker deployments)
- `aarch64-unknown-linux-gnu` (ARM servers)
- `aarch64-apple-darwin` (Mac M1/M2 dev environments)

## 3. Crate Publishing
The API allows publishing directly to `crates.io` leveraging `cargo publish`, utilizing API tokens mapped to GitHub repository secrets.
