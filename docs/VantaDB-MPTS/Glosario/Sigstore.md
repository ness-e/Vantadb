---
type: glossary-entry
status: stable
tags: [vantadb, glosario, seguridad, ci-cd]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
---

# Sigstore

## Definición

**Sigstore** es un proyecto de seguridad de cadena de suministro de software que proporciona firma, verificación y transparencia de artefactos sin necesidad de gestionar claves criptográficas.

## Componentes

| Componente | Función |
|------------|---------|
| **Cosign** | Firma y verificación de contenedores y artefactos |
| **Fulcio** | Autoridad certificadora para identidades OIDC |
| **Rekor** | Log de transparencia inmutable |

## Uso en VantaDB

### Firma de Wheels

```yaml
# .github/workflows/python_wheels.yml
- name: Sign wheels with Sigstore
  uses: sigstore/gh-action-sigstore-python@v2.1.1
  with:
    inputs: './dist/*.whl'
```

### Verificación

```bash
# Verificar firma de un wheel
python -m sigstore verify github \
  --cert-identity https://github.com/ness-e/Vantadb/.github/workflows/python_wheels.yml \
  --repository ness-e/Vantadb \
  vantadb_py-0.1.4-cp38-abi3-manylinux2014_x86_64.whl
```

## Flujo de Firma

```
1. CI genera wheel
2. CI solicita certificado a Fulcio (vía OIDC)
3. Fulcio emite certificado de corta duración
4. Cosign firma el wheel con el certificado
5. Firma y certificado se registran en Rekor
6. Usuario verifica contra Rekor
```

## Beneficios

| Característica | Tradicional | Sigstore |
|----------------|-------------|----------|
| **Gestión de claves** | Manual, compleja | Automática, sin claves |
| **Revocación** | Manual | Automática (certificados cortos) |
| **Transparencia** | Ninguna | Log público inmutable |
| **Verificación** | Requiere clave pública | Solo identidad del firmante |

## Attestations de GitHub

```bash
# Listar attestation de un release
gh attestation list --owner ness-e

# Verificar attestation específico
gh attestation verify <artifact> --owner ness-e
```

## Véase También

- [OIDC](OIDC.md) — Autenticación para firma
- [SLSA](SLSA.md) — Framework que Sigstore habilita
- [CI/CD](CI_CD.md) — Pipeline de firma

---

*Sigstore garantiza la provenance de artefactos VantaDB sin gestión de claves criptográficas.*

