# Fase 4 (AUD-09): Checksums CRC32C por registro en el WAL

El objetivo de esta fase es validar y certificar que VantaDB implementa de manera consistente la verificación de integridad mediante checksums CRC32C (Castagnoli) a nivel de registros individuales del WAL. Se resolverá la supuesta discrepancia señalada en la auditoría probando formalmente la capacidad de detección y auto-sanación (Scan-Forward) ante fallos selectivos del CRC.

## Diagnóstico y Estado Actual

Tras analizar [src/wal.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/wal.rs), se confirma lo siguiente:
1. **Estructura del Registro WAL**: Cada registro se almacena en el disco bajo el formato `[len: u32][payload: bincode][crc: u32]`.
2. **Cálculo del CRC**: Se calcula sobre el `payload` serializado por bincode usando la función optimizada por hardware `crc32c::crc32c`.
3. **Mecanismo de Verificación**: Durante la lectura en `WalReader::next_record` y la inicialización en `WalWriter::open`, se extrae el `stored_crc` y se compara con el CRC calculado sobre la carga útil leída.
4. **Auto-sanación (Scan-Forward)**: Si `stored_crc != computed_crc` o la deserialización de la carga útil falla, el sistema entra en modo de búsqueda hacia adelante byte a byte para recuperar la siguiente transacción válida alineada, descartando o truncando la parte corrupta.

Sin embargo, los tests actuales corrompen bloques arbitrarios que rompen tanto el aliento de bincode como los metadatos. No contamos con una prueba que demuestre formalmente que un registro con **payload completamente válido en términos de deserialización bincode pero con CRC corrupto (o viceversa)** es detectado e interceptado por el chequeo de CRC32C, activando la auto-sanación de manera robusta.

## Cambios Propuestos

### Componente Storage / WAL Tests

Se implementará un nuevo caso de prueba en [tests/storage/wal_resilience.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/storage/wal_resilience.rs) que certifique la validación fina del CRC:

#### [MODIFY] [wal_resilience.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/storage/wal_resilience.rs)

Se añadirá el test `test_wal_selective_crc_corruption_recovery` que realizará los siguientes pasos:
1. Inicializar un almacenamiento de base de datos con WAL habilitado y escribir 3 nodos (`Insert`).
2. Localizar la posición física del segundo registro en el archivo WAL.
3. Leer el registro, ubicar con precisión el offset del campo `crc` al final del segundo registro.
4. Modificar **únicamente** los bytes del checksum CRC de ese segundo registro (dejando el payload binario y la longitud intactos). Esto garantiza que `bincode` podría deserializarlo correctamente, pero el CRC detectará la corrupción y disparará el auto-healing.
5. Iniciar la base de datos de nuevo, forzando la recuperación del WAL (borrando el vector store e índice en disco).
6. Verificar que la base de datos se recupera saltando el nodo 2 (corrupto únicamente en CRC) pero **recuperando con éxito** el nodo 1 y el nodo 3 (cuyos CRCs permanecieron correctos).
7. Repetir la validación de truncamiento al final del WAL si el CRC del último registro está dañado.

## Plan de Verificación

### Pruebas Automatizadas
- Ejecutar el test específico:
  `cargo test --test wal_resilience test_wal_selective_crc_corruption_recovery`
- Ejecutar todas las pruebas del perfil `audit` o del workspace en general para garantizar que no hay regresiones:
  `cargo nextest run --profile audit` o `cargo test` (manual por el usuario según las directrices).

### Verificación Manual
- Solicitar al usuario que ejecute los comandos y comparta los resultados para certificar el éxito de la prueba.
