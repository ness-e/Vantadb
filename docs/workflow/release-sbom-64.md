# `release-sbom-64.yml` — RELEASE: SBOM — Generate

## ¿Qué hace?

Genera un SBOM (Software Bill of Materials) en formato CycloneDX JSON con todas las dependencias Rust del proyecto y lo sube como artifact.

## ¿Cómo lo hace?

Un solo job `sbom`:

1. Instala `cargo-cyclonedx`
2. Ejecuta `cargo cyclonedx --format json --override-filename sbom`
3. Sube `sbom.json` como artifact

## ¿Qué tests usa?

No ejecuta tests.

## ¿Qué verifica?

No verifica nada. Genera un inventario completo de dependencias.

## Funcionalidad final

Producir un SBOM en formato CycloneDX estándar para cumplimiento de seguridad, transparencia de supply chain y auditoría de dependencias.

## ¿Cuándo se ejecuta?

- **Push** de tag `v*` (cualquier tag que empiece con `v`)
- **Workflow dispatch** manual
