# Plan de Implementación: Optimización y Estabilización de Workflows de Certificación (HNSW & Fuzzing)

Este plan aborda los fallos de tiempo de ejecución y configuración en los workflows de integración continua de VantaDB, específicamente `Certification: hnsw_validation` y `Certification: fuzz_resilience`.

Además, incluye la corrección de los tests unitarios (`mmap_index`) y la optimización de los tests de estabilidad de carga (`hardware_profiles`) para evitar timeouts en compilaciones unoptimized (debug) locales y pre-push.

## Análisis y Deducción de Impacto (FMEA)

### 1. Eliminación de límite de tiempo en `hnsw_validation`

*   **Riesgo (FMEA):** Si se elimina por completo el límite de tiempo (`timeout-minutes`), cualquier regresión de código que introduzca un deadlock o bucle infinito en el índice HNSW mantendrá la máquina virtual de GitHub Actions encendida hasta el límite máximo de la plataforma (6 horas / 360 minutos). Esto consumirá cuotas de minutos de forma innecesaria.
*   **Mitigación:** En lugar de eliminar todo límite, removeremos el límite estricto a nivel de paso (step) y asignaremos un límite global muy holgado pero seguro al trabajo (job) de **120 minutos** (2 horas), garantizando que el test tenga tiempo suficiente para completarse pero no cause fugas infinitas de recursos.

### 2. Fallo de instalación y compilación en `fuzz_resilience`

*   **Riesgo (FMEA):** La instrucción `cargo install cargo-fuzz` compila la herramienta desde el código fuente en cada ejecución. Esto tarda entre 12 y 18 minutos en entornos de CI de GitHub Actions y es propenso a fallos por incompatibilidades con Rust nightly o dependencias rotas en crates.io.
*   **Mitigación:** Reemplazar la compilación manual por la acción optimizada `taiki-e/install-action@v2` pasando el parámetro `tool: cargo-fuzz`, la cual descarga binarios precompilados de forma instantánea (~3 segundos).

*   **Riesgo (FMEA):** Cargo infiere que `fuzz/` pertenece al workspace raíz porque hereda de su estructura jerárquica de directorios, pero dado que está excluido con `exclude = ["fuzz"]` en el `Cargo.toml` raíz, genera un error fatal: `current package believes it's in a workspace when it's not`.
*   **Mitigación:** Añadir un bloque `[workspace]` vacío en `fuzz/Cargo.toml` para indicarle a Cargo explícitamente que es un crate independiente.

### 3. Falsa alarma en test `mmap_vector_index_certification`

*   **Descripción:** El test de serialización `tests/storage/mmap_index.rs` fallaba localmente porque validaba una firma legacy (`VNTHNSW1`) en lugar del formato `VNDX` y versión 4 unificados por la cabecera `VantaHeader`.
*   **Solución:** Actualizar la aserción de cabecera a los bytes reales de `VNDX` y versión 4.

### 4. Timeout en test `hardware_certification_full` en modo Debug/Unoptimized

*   **Descripción:** El pre-push ejecuta el perfil de Nextest `audit` (compilación debug/unoptimized). Insertar 100K vectores de 128 d en modo debug tarda >180 segundos en almacenamiento persistente de disco, detonando el timeout.
*   **Solución:** Escalar condicionalmente el número de inserciones (10K vectores en debug; 100K vectores en release) usando `cfg!(debug_assertions)`.

---

## Cambios Propuestos

### Componente: CI/CD Workflows

#### [MODIFY] [heavy_certification.yml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/.github/workflows/heavy_certification.yml)

1. En el job `hnsw-validation`:
   * Incrementar `timeout-minutes` del job de 20 a 120 minutos.
   * Remover `timeout-minutes: 15` del step `Run HNSW validation`.
2. En el job `fuzz-resilience`:
   * Reemplazar:

     ```yaml
     - name: Install cargo-fuzz
       run: cargo install cargo-fuzz
     ```

     Por:

     ```yaml
     - name: Install cargo-fuzz
       uses: taiki-e/install-action@v2
       with:
         tool: cargo-fuzz
     ```

### Componente: Fuzzing Crate

#### [MODIFY] [Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/fuzz/Cargo.toml)

* Declarar el crate de fuzzing de forma aislada mediante un bloque `[workspace]` vacío.

### Componente: Tests

#### [MODIFY] [mmap_index.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/storage/mmap_index.rs)

* Actualizar la aserción del formato de cabecera en el test de integración para que valide `[b'V', b'N', b'D', b'X', 4, 0, 0, 0]`.

#### [MODIFY] [hardware_profiles.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/certification/hardware_profiles.rs)

* Reducir las inserciones de prueba de 100K a 10K vectores cuando se ejecuta en modo debug (`cfg!(debug_assertions)`), manteniendo los 100K originales solo en release.

---

## Plan de Verificación

### Verificación Manual (Solicitada al Usuario)

1. Correr las pruebas unitarias y de integración localmente.
2. Confirmar la subida y el disparo del pipeline en GitHub.
