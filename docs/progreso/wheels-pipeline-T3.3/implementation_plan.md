# T3.3 — Pipeline de Wheels: Completar Trusted Publishing + Verificación Post-Release

## Contexto

ST3.3.1 está certificado: el workflow `python_wheels.yml` compila wheels en Linux/macOS/Windows,
ejecuta smoke tests y publica en TestPyPI con OIDC (`id-token: write`).

Los dos gaps que cierran T3.3 son concretos y de bajo riesgo:

1. **ST3.3.2** — El job de producción `publish-pypi` existe pero NO tiene un gate de verificación
   post-publicación. El ciclo de release no tiene forma automatizada de confirmar que la wheel
   descargada desde PyPI funciona tras el upload.

2. **ST3.3.3** — La policy (`PYTHON_RELEASE_POLICY.md`) documenta `sigstore/gh-action-sigstore-python`
   pero la implementación usa `actions/attest-build-provenance@v2` (GitHub Attestations, estándar
   moderno equivalente). Hay que **alinear la documentación a la implementación real**, no al revés.

## Decisión de Diseño

| Opción | Decisión |
|---|---|
| Sigstore standalone (`sigstore/gh-action-sigstore-python`) | ❌ Descartado — obsoleto frente a GitHub Attestations |
| GitHub Attestations (`actions/attest-build-provenance@v2`) | ✅ **Adoptado** — ya implementado, SLSA Level 2, verificable con `gh` CLI |
| Job verify post-TestPyPI | ✅ **A implementar** — instala desde TestPyPI y ejecuta smoke tests |
| Job verify post-PyPI producción | ✅ **A implementar** — con retry y delay para propagación de CDN |

## Cambios Propuestos

---

### `.github/workflows/python_wheels.yml`

#### [MODIFY] python_wheels.yml

- Agregar job `verify-testpypi-install` que corre después de `publish-testpypi`:
  - Instala `vantadb-py` desde TestPyPI con `--extra-index-url pypi.org`
  - Ejecuta `python -c "import vantadb_py; print(vantadb_py.__version__)"` como smoke test
  - Solo corre cuando `publish-testpypi` es exitoso

- Agregar job `verify-pypi-install` que corre después de `publish-pypi` (producción):
  - Espera 60s para propagación del CDN de PyPI (`sleep 60`)
  - Instala desde PyPI producción limpia
  - Ejecuta smoke test de import + versión
  - Verifica provenance con `gh attestation verify` usando el repositorio de origen

---

### `docs/operations/PYTHON_RELEASE_POLICY.md`

#### [MODIFY] PYTHON_RELEASE_POLICY.md

- Reemplazar sección "Secure Artifact Signing via Sigstore" por "Supply-Chain Integrity via GitHub Attestations"
- Documentar el comando de verificación correcto: `gh attestation verify <wheel> --repo <org/repo>`
- Eliminar referencias al keyless Sigstore flow (Fulcio/Rekor) que no se usa
- Agregar sección de la verificación de provenance SLSA Level 2

---

### `docs/progreso/` (snapshot al finalizar)

- Crear `docs/progreso/wheels-pipeline-T3.3/` con implementation_plan, task y walkthrough.

## Plan de Verificación

### Automatizado (CI)
- El job `verify-testpypi-install` pasa en el siguiente dispatch manual del workflow
- El job `verify-pypi-install` pasa en el siguiente tag `v*.*.*`

### Manual (no requiere ejecución en esta sesión)
- Flujo completo de release documentado en `PYTHON_RELEASE_POLICY.md` actualizado
- El usuario puede triggear `workflow_dispatch` con `publish_testpypi=true` para validar
