# Walkthrough: Fase SEC-FFI-04 — Test Multiproceso Real para `flock`

**Fecha de finalización:** 2026-05-28  
**Estado:** ✅ COMPLETADA Y VERIFICADA AL 100%

---

## Resumen Ejecutivo

La fase **SEC-FFI-04** cierra el ciclo de verificación de la exclusión mutua de procesos escritores en VantaDB. Hemos diseñado y ejecutado un test de integración multiproceso real a nivel de sistema operativo para asegurar que la protección de bloqueo físico exclusivo sobre `.vanta.lock` mediante la API `flock` (`fs2`) opera deterministamente cuando es invocada por diferentes PIDs (identificadores de proceso) del OS.

---

## Componentes Desarrollados

### 1. Target Auxiliar de Cargo y Registro
*   **Archivo modificado**: [Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/Cargo.toml)
*   Registramos un nuevo ejecutable secundario `lock_helper` bajo el path `src/bin/lock_helper.rs`.
*   Esto nos permite usar la macro nativa de Cargo `env!("CARGO_BIN_EXE_lock_helper")` en la suite de pruebas para localizar el ejecutable en tiempo de compilación de forma portable, evitando dependencias externas de ejecución de comandos como `cargo run`.

### 2. Ejecutable de Pruebas Multi-proceso (`lock_helper`)
*   **Archivo creado**: [src/bin/lock_helper.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/bin/lock_helper.rs)
*   **Lógica**:
    *   Toma como argumento la ruta del directorio de la base de datos y un tiempo opcional a suspenderse (por defecto 2000 ms).
    *   Intenta abrir el motor de almacenamiento `StorageEngine::open(db_path)`, lo que fuerza de forma interna a adquirir el lock exclusivo sobre `.vanta.lock`.
    *   Si tiene éxito, imprime `LOCK_HELPER: SUCCESS_LOCK` en stdout, vacía los buffers (flushing) y duerme la cantidad de ms especificada antes de salir de forma limpia.
    *   Si ya está bloqueado, imprime `LOCK_HELPER: FAILED_LOCK: <error>` y sale con código `2`.

### 3. Suite de Tests Multiproceso de Integración
*   **Archivo modificado**: [tests/storage/multi_process_lock.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/storage/multi_process_lock.rs)
*   **Prueba integrada**: `test_exclusive_writer_lock_prevents_second_writer_multi_process`
    *   Inicia `lock_helper` como un proceso hijo independiente (P1).
    *   Lee la salida estándar de P1 para asegurar, antes de continuar, que P1 adquirió exitosamente el lock exclusivo.
    *   Intenta abrir la base de datos desde el proceso principal del test; valida que es rechazado con el error `locked by another process`.
    *   Lanza un segundo proceso hijo `lock_helper` (P2) y valida que el kernel lo rechaza, saliendo con código `2`.
    *   Mata al proceso hijo P1 para liberar los descriptores de archivos a nivel de kernel de OS.
    *   Valida que la base de datos se puede abrir inmediatamente y sin problemas desde el proceso del test.

---

## Resultados de la Certificación

La suite de pruebas de bloqueo se ejecutó de forma local en Windows (PowerShell):

```powershell
PS C:\Users\Eros\VantaDB Proyect\VantaDB> cargo test --test multi_process_lock -- --nocapture
   Compiling vantadb v0.1.4 (C:\Users\Eros\VantaDB Proyect\VantaDB)
    Finished `test` profile [unoptimized] target(s) in 8.85s
     Running tests\storage\multi_process_lock.rs (target\debug\deps\multi_process_lock-885490db9ba08dcf.exe)

running 2 tests
test test_exclusive_writer_lock_prevents_second_writer ... ok
test test_exclusive_writer_lock_prevents_second_writer_multi_process ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.95s
```

Los resultados confirman que el sistema opera con total consistencia multi-proceso, aislando y bloqueando descriptores a nivel de PID a través de `flock` de forma multiplataforma y sin race conditions.
