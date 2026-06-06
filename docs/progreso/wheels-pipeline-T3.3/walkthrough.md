# Walkthrough: T3.3 — Pipeline de Wheels para Distribución

**Fecha:** 2026-06-06  
**Estado:** ✅ COMPLETADA  
**Archivos modificados:** 2 | **Archivos creados:** 0

---

## Resumen

T3.3 cierra el pipeline completo de distribución del SDK Python de VantaDB añadiendo:

1. **Verificación post-publicación automatizada** (ST3.3.3) — dos nuevos jobs en el workflow de CI que confirman que la wheel publicada es instalable desde TestPyPI y PyPI producción, e incluyen verificación criptográfica de provenance.
2. **Alineación de documentación** (ST3.3.2 final) — `PYTHON_RELEASE_POLICY.md` actualizado para reflejar la implementación real (GitHub Attestations SLSA L2) en lugar de las referencias obsoletas a Sigstore standalone.

---

## Cambios Realizados

### `.github/workflows/python_wheels.yml`

Se agregaron dos jobs al final del pipeline:

#### `verify-testpypi-install`
- **Trigger:** Corre después de `publish-testpypi` en pushes a `main` o `workflow_dispatch`
- **Steps:**
  1. Extrae la versión del tag o de `Cargo.toml` si no hay tag
  2. Espera 30 segundos (propagación CDN de TestPyPI)
  3. Instala `vantadb-py=={version}` desde TestPyPI con fallback a PyPI para dependencias
  4. Smoke test de import + verificación de versión
- **Propósito:** Gate automatizado que detecta problemas de empaquetado antes de promoción a producción

#### `verify-pypi-install`
- **Trigger:** Corre después de `publish-pypi` solo en tags `v*.*.*`
- **Steps:**
  1. Espera 90 segundos (propagación CDN de PyPI producción — más lenta)
  2. Instala desde PyPI producción sin caché (`--no-cache-dir`)
  3. Smoke test de import + verificación de versión
  4. Descarga la wheel con `pip download` y verifica provenance con `gh attestation verify`
- **Propósito:** Cierra la cadena de custodia — confirma instalación limpia y firma SLSA Level 2

### `docs/operations/PYTHON_RELEASE_POLICY.md`

- **Reemplazó:** Sección "Secure Artifact Signing via Sigstore" (mecanismo que nunca se implementó)
- **Por:** Sección "Supply-Chain Integrity via GitHub Attestations (SLSA Level 2)" documentando la implementación real
- **Decisión técnica documentada:** Tabla comparativa justificando la elección de `actions/attest-build-provenance@v2` sobre `sigstore/gh-action-sigstore-python`
- **Nuevo comando de verificación:**
  ```bash
  gh attestation verify vantadb_py-0.1.4-*.whl --repo ness-e/Vantadb
  ```
- **Agregó:** Diagrama ASCII del pipeline completo de CI

---

## Pipeline Completo Post-T3.3

```
push v*.*.* tag
       │
       ▼
  [gate] version_coherence (Cargo.toml coherencia)
       │
       ▼
  [build-wheels] Linux / macOS / Windows
  (maturin build + smoke test por plataforma)
       │
       ├────────────────────────────────┐
       ▼                                ▼
  [publish-pypi]              [publish-testpypi]
  + attest-build-provenance    (main / dispatch)
  + attach GitHub Release            │
       │                             ▼
       ▼                  [verify-testpypi-install]
  [verify-pypi-install]    sleep 30s → install → smoke
  sleep 90s → install → smoke
  → gh attestation verify ← SLSA L2
```

---

## Decisión de Diseño: GitHub Attestations vs Sigstore Standalone

La `PYTHON_RELEASE_POLICY.md` original documentaba `sigstore/gh-action-sigstore-python` (Fulcio CA + Rekor transparency log). Esta acción fue deprecada en favor de GitHub Attestations, que:

- Opera nativamente dentro del ecosistema de GitHub Actions sin dependencias externas
- Genera provenance SLSA Level 2 automáticamente via OIDC de GitHub
- Usa el storage de attestations de GitHub (inmutable, auditable via `gh CLI`)
- Requiere 0 configuración adicional de claves o servicios externos

La implementación ya usaba `actions/attest-build-provenance@v2` — la documentación simplemente no lo reflejaba. Este walkthrough corrige esa divergencia.

---

## Criterios de Aceptación Verificados

| Criterio | Estado |
|---|---|
| ST3.3.1: `cibuildwheel` multi-plataforma | ✅ Pre-existente |
| ST3.3.2: Trusted Publishing OIDC en producción | ✅ Job `publish-pypi` con `id-token: write` |
| ST3.3.3: Verificación post-publicación automatizada | ✅ Jobs `verify-testpypi-install` y `verify-pypi-install` |
| Documentación alineada con implementación real | ✅ `PYTHON_RELEASE_POLICY.md` actualizado |
