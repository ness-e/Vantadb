# Plan de Implementación: Fase SEC-FFI-04 — Test Multiproceso Real para `flock`

Este plan de implementación detalla la estrategia para certificar de forma empírica y a nivel de sistema operativo que la exclusión mutua por archivo físico (`.vanta.lock` vía `fs2`) funciona correctamente entre múltiples procesos de OS independientes de VantaDB.

## User Review Required

> [!NOTE]
> Para la ejecución del test multiproceso, utilizaremos la macro `env!("CARGO_BIN_EXE_lock_helper")` provista por Cargo. Esto requiere registrar un nuevo binario de prueba `lock_helper` en `Cargo.toml`. La ventaja de este diseño es que el test se ejecuta sin invocar de manera externa al ejecutable de `cargo`, garantizando robustez y compatibilidad multiplataforma instantánea.

## Proposed Changes

### 🛠️ Workspace Configuration

#### [MODIFY] [Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/Cargo.toml)
*   Registrar el binario auxiliar `lock_helper` bajo la sección de ejecutables, asegurándose de que no interfiera con el binario principal `vanta-cli`.

```toml
[[bin]]
name = "lock_helper"
path = "src/bin/lock_helper.rs"
test = false
```

---

### 📦 Componente 1: lock_helper Executable

#### [NEW] [lock_helper.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/bin/lock_helper.rs)
*   Crear un binario minimalista y multiplataforma que:
    1.  Tome la ruta del directorio de base de datos como argumento.
    2.  Intente abrir el motor de almacenamiento `StorageEngine` (lo que dispara internamente el bloqueo exclusivo sobre `.vanta.lock`).
    3.  Si tiene éxito, imprima `LOCK_HELPER: SUCCESS_LOCK` en stdout y permanezca en espera por un tiempo determinado (e.g., pasado por argumento) o hasta que su stdout/stdin se cierre.
    4.  Si falla (porque ya está bloqueado por otro proceso), imprima `LOCK_HELPER: FAILED_LOCK: <error>` y retorne exit code `2`.

---

### 🧪 Componente 2: Integration Tests Hardening

#### [MODIFY] [multi_process_lock.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/storage/multi_process_lock.rs)
*   Mantener el test intra-proceso existente (renombrándolo para claridad si es necesario).
*   Implementar un nuevo test de integración `test_exclusive_writer_lock_prevents_second_writer_multi_process`:
    1.  Crear un directorio temporal.
    2.  Iniciar `lock_helper` como un subproceso (Proceso P1) usando `std::process::Command`.
    3.  Leer el flujo de stdout de P1 hasta capturar `LOCK_HELPER: SUCCESS_LOCK` para asegurar de manera determinista que P1 tiene el lock exclusivo de la base de datos.
    4.  Con P1 aún en ejecución, intentar abrir un `StorageEngine` desde el proceso principal del test de Cargo sobre el mismo directorio.
    5.  Asegurar que el proceso del test recibe de manera inmediata un error `VantaError::Execution` con el mensaje `"Database at '...' is locked by another process"`.
    6.  Lanzar un segundo subproceso (Proceso P2) de `lock_helper` intentando adquirir el lock. Asegurar que P2 termina de inmediato con exit code `2` y reporta fallo en stdout.
    7.  Terminar el Proceso P1 (enviando kill o esperando que expire su tiempo de bloqueo) y confirmar que se ha liberado el lock de manera limpia.
    8.  Intentar abrir el `StorageEngine` desde el proceso del test una vez más y confirmar que ahora tiene éxito al abrir la base de datos libre de bloqueos.

---

## Verification Plan

### Automated Tests
*   Ejecutar la suite de exclusión mutua localmente:
    ```powershell
    cargo test --test multi_process_lock -- --nocapture
    ```
*   Compilar y validar lints:
    ```powershell
    cargo check --all-targets
    cargo clippy -p vantadb --all-targets -- -D warnings
    ```
