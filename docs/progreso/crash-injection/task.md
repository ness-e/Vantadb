# Checklist: Pruebas de Inyección de Caídas — Crash-injection tests (AUD-02)

## 1. Creación de Binario Auxiliar de Caos
- [x] Crear el archivo `src/bin/crash_helper.rs`.
- [x] Configurar la base de datos con `sync_mode = SyncMode::Always` para transacciones durables estrictas.
- [x] Implementar el bucle de inserción que imprima confirmaciones `WRITTEN:<id>` a stdout con `flush()` inmediato.
- [x] Simular demoras artificiales cortas entre inserciones para permitir interrupciones en cualquier instante.

## 2. Implementación de Test de Integración
- [x] Crear el archivo `tests/storage/crash_injection.rs`.
- [x] Implementar la compilación inicial automática de `crash_helper` al arrancar el test.
- [x] Implementar el ciclo de 100 iteraciones consecutivas.
- [x] Para cada iteración:
  - [x] Lanzar el proceso del hijo.
  - [x] Leer de forma reactiva de su stdout los IDs insertados.
  - [x] Interrumpir en caliente enviando `SIGKILL` / `TerminateProcess` mediante `child.kill()`.
  - [x] Esperar la liberación de recursos del proceso muerto.
  - [x] Reabrir la base de datos en frío.
  - [x] Validar la consistencia estructural del índice HNSW (`hnsw.validate_index()`).
  - [x] Recuperar cada uno de los nodos reportados y certificar que existan sin corrupción.

## 3. Integración en el Pipeline
- [x] Verificar que no esté excluido en el `default-filter` de `profile.audit` en `.config/nextest.toml`.
- [x] Integrar `crash_injection` en el job `failpoint-tests` de `.github/workflows/heavy_certification.yml`.

## 4. Verificación y Ejecución
- [x] Ejecutar de forma local: `cargo test --test crash_injection`.
- [x] Ejecutar la suite completa rápida: `cargo test -j 1 -- --skip test_benchmark_internal_10k --skip sift1m_competitive_benchmark`.
- [x] Ejecutar Nextest con el perfil de auditoría: `cargo nextest run --profile audit`.
