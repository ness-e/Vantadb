---
title: CI/CD ΓÇö Continuous Integration / Continuous Deployment
kind: glossary
status: stable
description: "##Definition"
aliases: [Continuous Integration, Continuous Deployment]
tags: [devops, automation, ci, cd]
type: glossary-entry
last_reviewed: "2026-09-15"
links: "[[README.md]]"
---

# CI/CD ΓÇö Continuous Integration / Continuous Deployment

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
# .github/workflows/ci-rust-10.yml
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

| Workflow | Trigger | Prop├│sito |
|----------|---------|-----------|
| **ci-rust-10.yml** | push/PR | Tests, lint, format |
| **release-wheels-60.yml** | tag `v*` | Build + publish wheels |
| **release.yml** | tag `v*` | Build binarios multi-platform |
| **heavy-certification-50.yml** | weekly | Stress tests, chaos testing |

##Publishing to PyPI

```yaml
# OIDC trusted publishing (sin API tokens)
- name: Publish to PyPI
  uses: pypa/gh-action-pypi-publish@release/v1
  with:
    packages-dir: dist/
```

## CI/CD metrics

| M├⌐trica | Objetivo | Actual |
|---------|----------|--------|
| **Build time** | <15 min | ~12.5 min |
| **Test coverage** | >80% | ~75% |
| **Deployment frequency** | Semanal | Quincenal |
| **Change failure rate** | <5% | ~3% |

## See Also

- [benchmarks](./benchmarks.md) ΓÇö Integrated in CI
- [chaos-testing](./chaos-testing.md) ΓÇö Robustness tests
- [failpoints](./failpoints.md) ΓÇö Fault injection

---

*CI/CD is the backbone of reliable and frequent releases.*

