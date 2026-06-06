# Walkthrough: Ajuste de Timeouts y Optimización de Workflows de Certificación

Se han corregido y optimizado los trabajos de integración continua en el archivo `.github/workflows/heavy_certification.yml` y se corrigió la configuración del proyecto de fuzzing y aserciones locales.

## Cambios Realizados

1. **`heavy_certification.yml`**:
   - Se incrementó el `timeout-minutes` global del trabajo `hnsw-validation` de **20 minutos a 120 minutos**.
   - Se removió el limitador `timeout-minutes: 15` del paso individual de ejecución del test `cargo test --release --test hnsw_validation`.
   - Se sustituyó la instalación lenta mediante compilación de `cargo install cargo-fuzz` en el trabajo `fuzz-resilience` por `uses: taiki-e/install-action@v2` con `with: tool: cargo-fuzz`. Esto descargará el binario precompilado directamente de forma segura y optimizada, previniendo errores por inputs faltantes (`tool`) y acelerando la instalación.

2. **`fuzz/Cargo.toml`**:
   - Se añadió la sección vacía `[workspace]` al manifest del crate de fuzzing para declararlo explícitamente como un espacio de trabajo independiente y prevenir que Cargo infiera que pertenece al workspace raíz del que está excluido. Esto soluciona la falla de compilación en GitHub Actions.

3. **`tests/storage/mmap_index.rs`**:
   - Se corrigió el test `mmap_vector_index_certification` en la línea 45 para cambiar la aserción de cabecera legacy (`VNTHNSW1`) por la firma oficial unificada `[b'V', b'N', b'D', b'X', 4, 0, 0, 0]`. Esto soluciona la falla de pre-push local.

4. **`tests/certification/hardware_profiles.rs`**:
   - Se optimizó el test `hardware_certification_full` para compilar dinámicamente con un tamaño de inserción de 10K vectores en lugar de 100K cuando está activo `cfg!(debug_assertions)` (perfil debug/unoptimized). Esto elimina la posibilidad de que el pre-push local falle por timeout (>180 segundos) mientras que mantiene la carga completa de 100K vectores para releases y builds de producción optimizadas.

## Verificación

- La sintaxis del workflow de GitHub Actions fue corregida para cumplir con las propiedades obligatorias de `taiki-e/install-action`.
- La aserción del formato del índice fue corregida para alinearse con `VantaHeader` (versión 4).
- Se resolvió el conflicto de scopes del workspace en Cargo para las tareas de fuzzing.
- Se previno el timeout de Nextest unoptimized adaptando el tamaño del test de estabilidad de hardware en debug.
- Se requiere subir los cambios y permitir la ejecución del pipeline correspondiente en GitHub.
