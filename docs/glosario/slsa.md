---
type: glossary-entry
status: stable
tags: [vantadb, glosario, seguridad, ci-cd]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
---

# SLSA (Supply-chain Levels for Software Artifacts)

## Definición

**SLSA** (Supply-chain Levels for Software Artifacts) es un framework de seguridad que define niveles de madurez para la cadena de suministro de software, desde código fuente hasta artefactos binarios.

## Niveles SLSA

| Nivel | Descripción | Requisitos |
|-------|-------------|------------|
| **SLSA 0** | Sin garantías | Ninguno |
| **SLSA 1** | Build documentado | Script de build, provenance básica |
| **SLSA 2** | Build firmado | [OIDC](OIDC.md), [Sigstore](Sigstore.md), source versionado |
| **SLSA 3** | Build aislado | Hermetic builds, no user input |
| **SLSA 4** | Two-party review | Code review + approval |

## VantaDB: SLSA Level 2

### Requisitos Cumplidos

✅ **Source versionado:** GitHub con commits firmados
✅ **Build script:** GitHub Actions workflows
✅ **Build service:** GitHub-hosted runners
✅ **Provenance:** Generada automáticamente por GitHub
✅ **Firma:** [Sigstore](Sigstore.md) via OIDC

### Verificación de Artefactos

```bash
# Verificar wheel de PyPI
gh attestation verify \
  --owner ness-e \
  vantadb_py-0.1.4-cp38-abi3-manylinux2014_x86_64.whl

# Verificar binary de GitHub Release
gh attestation verify \
  --owner ness-e \
  vantadb-server-linux-x86_64.tar.gz
```

### Provenance Format

```json
{
  "_type": "https://in-toto.io/Statement/v0.1",
  "predicateType": "https://slsa.dev/provenance/v0.2",
  "subject": [
    {
      "name": "vantadb_py-0.1.4-cp38-abi3-manylinux2014_x86_64.whl",
      "digest": {
        "sha256": "abc123..."
      }
    }
  ],
  "predicate": {
    "builder": {
      "id": "https://github.com/actions/runner"
    },
    "buildType": "https://github.com/actions/runner/github-actions",
    "invocation": {
      "configSource": {
        "uri": "git+https://github.com/ness-e/Vantadb@refs/tags/v0.1.4",
        "digest": {
          "sha1": "def456..."
        },
        "entryPoint": ".github/workflows/python_wheels.yml"
      }
    }
  }
}
```

## Beneficios de Seguridad

| Ataque | SLSA 0 | SLSA 2 |
|--------|--------|--------|
| **Compromiso de source** | ❌ | ✅ (versionado + firmado) |
| **Compromiso de build** | ❌ | ✅ (provenance verificable) |
| **Dependency confusion** | ❌ | ⚠️ (parcial) |
| **Typo-squatting** | ❌ | ❌ |
| **Malicious maintainer** | ❌ | ⚠️ (parcial) |

## Roadmap

| Fase | Nivel | Estado |
|------|-------|--------|
| **Actual** | SLSA 2 | ✅ Implementado |
| **FASE 4** | SLSA 3 | ⬜ Planeado |
| **FASE 5** | SLSA 4 | ⬜ Planeado |

## Véase También

- [OIDC](OIDC.md) — Autenticación para builds
- [Sigstore](Sigstore.md) — Firma de artefactos
- [CI/CD](CI_CD.md) — Pipeline de build

---

*SLSA Level 2 garantiza la provenance verificable de todos los artefactos de VantaDB.*

