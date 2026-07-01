---
type: glossary-entry
status: stable
tags: [devops, automation, ci, cd]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Continuous Integration, Continuous Deployment]
---
# CI/CD — Continuous Integration / Continuous Deployment

##Definition

**CI/CD** is the practice of **automating the integration, testing and deployment** of code, allowing frequent and reliable releases through pipelines that validate each change before reaching production.

## Components

### CI (Continuous Integration)

- **Automatic build** on each push/PR
- **Automated tests** (unit, integration, e2e)
- **Linting and formatting**
- **Security scanning**

### CD (Continuous Deployment)

- **Build artifacts** (binaries, wheels)
- **Automatic publishing** (PyPI, crates.io)
- **Deploy to production** (if all tests pass)

## CI/CD in VantaDB

### GitHub Actions Workflows

```yaml
# .github/workflows/rust_ci.yml
name: Rust CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: charge test --all-features
      - run: clippy charge -- -D warnings
      - run: fmt charge --check
```

### Main Pipelines

| Workflow | Trigger | Propósito |
|----------|---------|-----------|
| **rust_ci.yml** | push/PR | Tests, lint, format |
| **python_wheels.yml** | tag `v*` | Build + publish wheels |
| **release.yml** | tag `v*` | Build binarios multi-platform |
| **heavy_certification.yml** | weekly | Stress tests, chaos testing |

##Publishing to PyPI

```yaml
# OIDC trusted publishing (sin API tokens)
- name: Publish to PyPI
  uses: pypa/gh-action-pypi-publish@release/v1
  with:
    packages-dir: dist/
```

## CI/CD metrics

| Métrica | Objetivo | Actual |
|---------|----------|--------|
| **Build time** | <15 min | ~12.5 min |
| **Test coverage** | >80% | ~75% |
| **Deployment frequency** | Semanal | Quincenal |
| **Change failure rate** | <5% | ~3% |

## See Also

- [[benchmarks]] — Integrated in CI
- [[chaos-testing]] — Robustness tests
- [[failpoints]] — Fault injection

---

*CI/CD is the backbone of reliable and frequent releases.*

