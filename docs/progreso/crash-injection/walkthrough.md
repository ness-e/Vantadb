# Walkthrough: Fase 3 — Pruebas de Inyección de Caídas (Crash-injection tests) (AUD-02)

## Objetivo

Certificar la resiliencia física de durabilidad de VantaDB ante caídas abruptas de proceso en caliente (como terminaciones forzosas `SIGKILL` o `TerminateProcess` por caídas de energía o fallos de hardware) utilizando su Write-Ahead Log (WAL) con sumas de comprobación CRC32C, garantizando que el motor de base de datos se recupere a un estado consistente en frío con una tasa de éxito de 100/100 recuperaciones correctas sin pérdida de datos ni corrupción del índice HNSW.

---

## Cambios Realizados

### 1. Registro y Configuración de Objetivos
- **[Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/Cargo.toml)**
  - Declarado el binario de caos `crash_helper` como un target ejecutable `[[bin]]`.
  - Declarado el test de inyección de caídas `crash_injection` como un target `[[test]]`.

### 2. Creación del Binario de Caos
- **[crash_helper.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/bin/crash_helper.rs)** [NEW]
  - Programa Rust optimizado que abre `StorageEngine` forzando la política de persistencia duradera síncrona estricta (`SyncMode::Always`).
  - Realiza inserciones masivas de nodos y escribe un identificador `WRITTEN:<node_id>` a `stdout` con `flush()` inmediato para cada registro confirmado síncronamente.
  - Implementa un pequeño retraso de 5ms entre escrituras para que el test pueda interceptarlo en cualquier instante.

### 3. Suite de Pruebas de Integración y Recuperación en Frío
- **[crash_injection.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/storage/crash_injection.rs)** [NEW]
  - Test de integración portátil (Windows/Linux/macOS) que compila y ejecuta `crash_helper` en un subproceso hijo y repite el siguiente ciclo **100 veces consecutivas**:
    1. Iniciar el subproceso auxiliar escribiendo sobre un directorio de base de datos limpio.
    2. Leer la salida estándar del hijo reactivamente para almacenar las transacciones confirmadas.
    3. Matar abruptamente al hijo (`child.kill()`) tras alcanzar una cantidad pseudo-aleatoria de confirmaciones.
    4. Reabrir `StorageEngine` desde el test principal.
    5. Validar que cada nodo reportado como `WRITTEN` se recupere exitosamente y sin discrepancias de ID.
    6. Validar que el índice HNSW sea estructuralmente válido (`validate_index()`).

### 4. Integración en el Pipeline de CI y Nextest
- **[.github/workflows/heavy_certification.yml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/.github/workflows/heavy_certification.yml)**
  - Integrada la suite `crash_injection` dentro del job `failpoint-tests` para ser validada en ejecuciones semanales y manuales en GitHub Actions.
- **[.config/nextest.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/.config/nextest.toml)**
  - La prueba corre de manera predeterminada en el perfil `audit` de CI rápido al no estar excluida de los filtros.

### 5. Corrección de Bug en Test de Locking Multi-proceso
- **[multi_process_lock.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/storage/multi_process_lock.rs)**
  - **Bug corregido:** La suite de pruebas de locking esperaba recibir la variante de error genérica `VantaError::Execution(msg)` en caso de conflicto de exclusión mutua. Sin embargo, tras las mejoras de la Fase 1, el motor ahora retorna la variante especializada `VantaError::DatabaseBusy(msg)`.
  - **Fix:** Actualizados los asertos y matches para validar la nueva variante de error especializada, restableciendo la compilación y aprobación de los tests.

---

## Verificación

### 1. Test Específico de Caídas (AUD-02)
- El test de inyección de caídas pasó exitosamente en Windows:
  - **Resultado:** `test_crash_injection_and_cold_recovery_loop ... ok` (100% de recuperaciones consistentemente correctas sobre 100 iteraciones en 100.34 segundos).

### 2. Suite Completa Rápida de Nextest
- Todos los tests de la suite rápida de auditoría pasaron sin errores:
  - **Comando:** `cargo nextest run --profile audit`
  - **Resultado:** `Summary [ 154.958s] 150 tests run: 150 passed (1 slow), 9 skipped`
  - **Cobertura:** Validó integridad de serialización, recuperación de fallas, bloqueos multi-proceso, concurrencia RCU, y consistencia relacional.
