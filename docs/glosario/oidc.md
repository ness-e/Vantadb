---
type: glossary-entry
status: stable
tags: [vantadb, glosario, seguridad, ci-cd]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
---

# OIDC (OpenID Connect)

## Definición

**OIDC** (OpenID Connect) es un protocolo de autenticación basado en OAuth 2.0 que permite a aplicaciones verificar la identidad de usuarios y obtener información de perfil de manera segura.

## Uso en VantaDB

### Publicación Segura a PyPI

OIDC elimina la necesidad de almacenar tokens API de larga duración en secrets de GitHub:

```yaml
# .github/workflows/python_wheels.yml
permissions:
  id-token: write  # Requerido para OIDC

jobs:
  publish:
    environment: pypi
    steps:
      - uses: pypa/gh-action-pypi-publish@release/v1
        with:
          # Sin password/token - usa OIDC automáticamente
          attestations: true
```

### Flujo de Autenticación

```
1. GitHub Actions solicita token OIDC a GitHub
2. GitHub emite token firmado con claims del workflow
3. PyPI verifica firma de GitHub y claims
4. PyPI emite token de corta duración para upload
5. Workflow usa token temporal para publicar
```

## Beneficios de Seguridad

| Método | Riesgo | Duración |
|--------|--------|----------|
| **API Token** | Robo de secret, acceso permanente | Hasta revocación manual |
| **OIDC** | Sin secrets almacenados | Minutos (token efímero) |

## Configuración en PyPI

### Trusted Publisher

1. Ir a https://pypi.org/manage/account/publishing/
2. Agregar "Trusted Publisher"
3. Configurar:
   - Repository owner: `ness-e`
   - Repository name: `Vantadb`
   - Workflow: `python_wheels.yml`
   - Environment: `pypi`

## Verificación de Attestations

```bash
# Verificar provenance de un wheel
gh attestation verify \
  vantadb_py-0.1.4-cp38-abi3-manylinux2014_x86_64.whl \
  --owner ness-e
```

## Véase También

- [CI/CD](CI_CD.md) — Pipeline de publicación
- [Sigstore](Sigstore.md) — Firma de artefactos
- [SLSA](SLSA.md) — Framework de seguridad

---

*OIDC proporciona publicación segura sin secrets de larga duración, reduciendo superficie de ataque.*

