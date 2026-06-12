# Plan de Implementación: Pruebas de Inyección de Caídas — Crash-injection tests (AUD-02)

Este plan define las acciones para diseñar e implementar una suite automatizada de pruebas de inyección de caídas físicas (Crash-injection tests) en VantaDB para cumplir con el requisito de auditoría **AUD-02**. 

El objetivo es certificar que VantaDB tolera caídas de proceso abruptas (`SIGKILL` / `TerminateProcess`) en caliente sin sufrir corrupción física ni pérdida de consistencia transaccional, logrando un ratio de éxito de 100/100 recuperaciones correctas.

---

## User Review Required

> [!IMPORTANT]
> **Diseño Multi-Plataforma Portátil:**
> Se implementará una arquitectura de sub-proceso hijo:
> 1. Un programa auxiliar en Rust `crash_helper` insertará registros continuamente en modo fsync estricto (`SyncMode::Always`).
> 2. El test de integración padre monitoreará la salida del hijo y lo matará forzosamente en un punto intermedio (`child.kill()`).
> 3. El test reabrirá la base de datos en frío y validará que todos los registros reportados por el hijo como confirmados existan y el índice HNSW sea 100% consistente.
>
> Este enfoque funciona de manera idéntica en Windows, macOS y Linux sin depender de scripts externos de shell de comandos de sistema específicos de Unix.

---

## Proposed Changes

### 1. Nuevo Binario de Soporte de Caos

#### [NEW] [crash_helper.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/bin/crash_helper.rs)
- Crear un binario auxiliar que:
  - Reciba por argumentos el path de la base de datos y el número total de nodos a insertar.
  - Configure `VantaConfig` en modo de durabilidad estricta (`SyncMode::Always`) para garantizar que cada retorno exitoso de `insert` se haya sincronizado a disco.
  - Ejecute inserciones secuenciales de `UnifiedNode` y envíe a `stdout` confirmaciones del formato `WRITTEN:<node_id>` con un `flush()` de stdout inmediato.
  - Simule un pequeño delay de 5-10ms entre operaciones para dar un margen de caída realista.

---

### 2. Suite de Pruebas de Integración de Caídas

#### [NEW] [crash_injection.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/storage/crash_injection.rs)
- Crear una prueba de integración `test_crash_injection_and_cold_recovery_loop` que:
  - Compile una vez al inicio el binario `crash_helper` en el perfil correcto (debug o release).
  - Ejecute un bucle de **100 iteraciones consecutivas**.
  - Para cada iteración:
    1. Cree un directorio temporal único.
    2. Lance `crash_helper` en un proceso hijo.
    3. Lea la salida estándar (`stdout`) del hijo de forma reactiva y almacene los IDs confirmados.
    4. Después de recibir una cantidad aleatoria de confirmaciones (entre 10 y 80), invoque `child.kill()` para terminar abruptamente el proceso del hijo.
    5. Reabra el motor `StorageEngine` desde el test principal (proceso padre).
    6. Verifique que todos los registros que el hijo reportó como `WRITTEN` se recuperen exitosamente vía `engine.get()`.
    7. Valide la consistencia interna estructural del grafo HNSW llamando a `hnsw.validate_index()`.

---

### 3. Habilitación del Test en el CI

#### [MODIFY] [.config/nextest.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/.config/nextest.toml)
- Asegurar que la nueva prueba de integración no esté en el filtro de exclusiones de `profile.audit`, de forma que sea ejecutada en todos los envíos de pull requests y commits del pipeline principal de CI (`VantaDB CI` en `rust_ci.yml`).

---

## Verification Plan

### Automated Tests
- Ejecutar de forma local la suite de inyección de caídas para certificar que compila y pasa:
  ```powershell
  cargo test --test crash_injection
  ```
- Validar que pase la suite de Nextest con el perfil de auditoría para garantizar integración perfecta:
  ```powershell
  cargo nextest run --profile audit --workspace
  ```
