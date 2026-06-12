# ADR 002: Resiliencia Física del WAL, Validación CRC32C y Mecanismo de Auto-Healing

## Estado

Estado: Aprobado

## Contexto

En escenarios de caídas del sistema de producción (p. ej., fallos de energía, kernel panics o terminación abrupta del proceso), el archivo de Write-Ahead Log (WAL) puede quedar en un estado corrupto o parcialmente escrito en su sección final (trailer).
La ausencia de validaciones de integridad binaria estrictas y de recuperación tolerante a fallos permitía que registros incompletos o corruptos fuesen interpretados en el arranque del motor, provocando caídas catastróficas del sistema o, peor aún, corrupción silenciosa de datos que se propagaba al índice HNSW y al almacenamiento relacional.

## Decisión

Para blindar la consistencia física del motor ante fallos de persistencia catastróficos, se implementó un rediseño de la capa WAL basado en tres pilares:

1. **Estructura Binaria Robusta con Versionado:** Cada registro del WAL se serializa bajo una estructura binaria con cabeceras explícitamente estructuradas y una versión de protocolo mutable:
   * `version: u32`: Identificador de versión del WAL. Se prohíbe y rechaza explícitamente `version = 0` para evitar decodificar ruido binario de archivos vacíos o pre-asignados con ceros.
   * `payload_len: u32`: Longitud del payload de datos.
   * `crc32c: u32`: Checksum calculado de forma redundante sobre el payload de datos completo utilizando la variante CRC32C (Castagnoli).
   * `payload`: Bytes correspondientes a la mutación (`Put`, `Delete`, etc.).

2. **Auto-Healing ante Caídas Catastróficas:** En la fase de arranque y recuperación, el lector del WAL analiza secuencialmente cada frame binario. Si encuentra una corrupción (fallo de CRC32C, cabecera corrupta o EOF prematuro a mitad de un registro):
   * Detiene inmediatamente la decodificación de forma controlada y segura.
   * Emite logs detallados de nivel de advertencia (`tracing::warn!`) especificando el offset del byte corrupto.
   * Ejecuta una rutina de **truncamiento físico automático (auto-healing)** para cortar el archivo del WAL exactamente en la posición del último registro consistente y saludable.
   * Limpia cualquier residuo binario corrupto posterior al corte, permitiendo al motor continuar el arranque normal con las transacciones confirmadas hasta ese punto.

3. **Garantías de Coherencia en Checkpoints:** Al invocar la consolidación de base de datos (`checkpoint`), se asegura la descarga física a disco (`flush()` y `sync()`) de las tablas activas antes de actualizar el puntero `checkpoint_seq: u64` en el metadata index, garantizando un punto de consistencia exacto sobre el cual restaurar.

## Consecuencias

### Beneficios

* **Resiliencia Extrema a Fallos de Energía:** Las pruebas de caos con inyección activa de fallos binarios demuestran que VantaDB arranca siempre sin errores catastróficos y en un estado lógicamente coherente, independientemente de qué tan corrupto quede el final del WAL.
* **Prevención de Corrupción Silenciosa:** Ninguna mutación parcial o corrupta por escritura incompleta en disco puede llegar a ser inyectada en la memoria viva del motor, protegiendo las bases de datos de embeddings e índices relacionales.
* **Fuzzing Integrado:** Se integró una suite exclusiva en `heavy_certification.yml` que estresa sistemáticamente el decodificador de WAL inyectando fallos e interrupciones aleatorias, certificando una estabilidad del 100%.

### Deuda Técnica / Costos

* **Pérdida Controlada de Transacciones no Confirmadas:** El truncamiento físico del trailer implica descartar la última mutación incompleta o no sincronizada físicamente a disco al momento de la caída. Este es un trade-off estándar en base de datos industriales y se considera la única alternativa segura frente a la inyección de datos corruptos.
* **Cálculo de Checksum en CPU:** El cómputo del CRC32C añade un mínimo overhead al escribir registros en caliente. Para mitigar esto, se hace uso de librerías altamente optimizadas que explotan instrucciones SIMD a nivel de CPU siempre que estén disponibles.
