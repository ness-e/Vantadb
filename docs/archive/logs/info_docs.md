# Optimización de GitHub Actions (ConectomeDB)

Este documento detalla las modificaciones realizadas el 3 de abril de 2026 para estabilizar el flujo de CI en la versión gratuita de GitHub Actions, evitando errores de Out of Memory (OOM).

## Problema Identificado
El proyecto superaba los 7GB de RAM provistos por el runner gratuito al intentar compilar en modo `--release` con `codegen-units = 1`. Específicamente, el paso de linkeo con RocksDB y las optimizaciones máximas consumen demasiada memoria para un entorno limitado.

## Modificaciones de Estabilidad (v0.4.0+)

### 1. Cambio a Modo Debug en CI
Se eliminó el flag `--release` de `cargo build` y `cargo test` en el archivo `.github/workflows/rust_ci.yml`. 
- **Razón:** El modo Debug es mucho más ligero y rápido de compilar. Para la validación continua de commits, es suficiente detectar errores lógicos sin necesidad de optimizaciones binarias costosas.

### 2. Configuración de Cargo en CI
Se añadió la variable de entorno `CARGO_INCREMENTAL: 0`.
- **Razón:** Deshabilitar la compilación incremental en entornos de CI (donde el caché se maneja externamente) reduce el uso de RAM y evita el desperdicio de espacio en disco durante la compilación.

### 3. Swap Estratégico (6GB)
Se aumentó el espacio de intercambio configurado en el runner de 4GB a 6GB.
- **Razón:** Proporciona un colchón de seguridad adicional para el linker durante las fases críticas de la compilación de RocksDB y otros módulos complejos.

### 4. Limitación de Paralelismo
Se mantiene `-j 2` y `--test-threads=2` (o se asume el default de 2 cores del runner).
- **Razón:** Previene que el compilador intente spawnear más hilos de los que el hardware físico puede procesar concurrentemente, lo cual suele disparar picos de memoria.

## Impacto en Producción
Estas modificaciones **solo afectan al flujo de CI**. Los binarios de producción generados por `release.yml` mantienen sus configuraciones de alto rendimiento:
- `opt-level = 3`
- `codegen-units = 1`
- `lto = "thin"`

---
*Documento autogenerado por Antigravity para persistencia de decisiones arquitectónicas.*
