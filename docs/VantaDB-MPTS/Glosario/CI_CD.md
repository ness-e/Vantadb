---
type: glosario-entry
status: stable
tags: [devops, automation, ci, cd]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Continuous Integration, Continuous Deployment]
---

# CI/CD — Continuous Integration / Continuous Deployment

## Definición

**CI/CD** es la práctica de **automatizar la integración, testing y despliegue** de código, permitiendo releases frecuentes y confiables mediante pipelines que validan cada cambio antes de llegar a producción.

## Componentes

### CI (Continuous Integration)

- **Build automático** en cada push/PR
- **Tests automatizados** (unit, integration, e2e)
- **Linting y formatting**
- **Security scanning**

### CD (Continuous Deployment)

- **Build de artifacts** (binarios, wheels)
- **Publicación automática** (PyPI, crates.io)
- **Deploy a producción** (si todos los tests pasan)

## CI/CD en VantaDB

### Workflows de GitHub Actions

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
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt --check
```

### Pipelines Principales

| Workflow | Trigger | Propósito |
|----------|---------|-----------|
| **rust_ci.yml** | push/PR | Tests, lint, format |
| **python_wheels.yml** | tag `v*` | Build + publish wheels |
| **release.yml** | tag `v*` | Build binarios multi-platform |
| **heavy_certification.yml** | weekly | Stress tests, chaos testing |

## Publicación a PyPI

```yaml
# OIDC trusted publishing (sin API tokens)
- name: Publish to PyPI
  uses: pypa/gh-action-pypi-publish@release/v1
  with:
    packages-dir: dist/
```

## Métricas de CI/CD

| Métrica | Objetivo | Actual |
|---------|----------|--------|
| **Build time** | <15 min | ~12.5 min |
| **Test coverage** | >80% | ~75% |
| **Deployment frequency** | Semanal | Quincenal |
| **Change failure rate** | <5% | ~3% |

## Véase También

- [Benchmarks](Benchmarks.md) — Integrados en CI
- [Chaos Testing](Chaos Testing.md) — Tests de robustez
- [Failpoints](Failpoints.md) — Inyección de fallos

---

*CI/CD es la columna vertebral de releases confiables y frecuentes.*

