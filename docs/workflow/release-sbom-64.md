# `release-sbom-64.yml` — RELEASE: SBOM — Generate

## ¿Qué hace?

Genera SBOMs (Software Bill of Materials) en formato CycloneDX JSON para los tres ecosistemas del proyecto y los sube como artifacts separados:

- **Rust**: `sbom.json` — todas las dependencias Cargo del workspace
- **npm**: `sbom-web.json` — dependencias de `web/` desde `package-lock.json` (modo `--package-lock-only`, sin instalar node_modules)
- **Python**: `sbom-python.json` — bindings `vantadb-python/` desde `pyproject.toml` (root component; hoy sin dependencias de terceros declaradas)

## ¿Cómo lo hace?

Un solo job `sbom`:

1. Instala `cargo-cyclonedx` y ejecuta `cargo cyclonedx --format json --override-filename sbom`
2. Genera SBOM npm con `npx @cyclonedx/cyclonedx-npm --package-lock-only`
3. Genera SBOM Python con `cyclonedx-py requirements - --pyproject vantadb-python/pyproject.toml` (paquete `cyclonedx-bom`)
4. Sube `sbom.json`, `sbom-web.json` y `sbom-python.json` como artifacts separados

## ¿Qué tests usa?

No ejecuta tests.

## ¿Qué verifica?

No verifica nada. Genera un inventario completo de dependencias por ecosistema.

## Funcionalidad final

Producir SBOMs en formato CycloneDX estándar para cumplimiento de seguridad, transparencia de supply chain y auditoría de dependencias en Rust, npm y Python.

## ¿Cuándo se ejecuta?

- **Push** de tag `v*` (cualquier tag que empiece con `v`)
- **Workflow dispatch** manual
