# Walkthrough — T3.1: Chaos Testing Expandido y Validación de Durabilidad

Hemos completado exitosamente la implementación, certificación y documentación de la tarea **T3.1** del Plan Maestro Unificado. Este documento resume los cambios en el core, los escenarios de caos validados, la automatización del loop de caos y los resultados de las pruebas.

## Cambios Realizados

### 1. Instrumentación de Nuevos Failpoints en el Core
Para validar la resiliencia del motor ante fallos de persistencia catastróficos, instrumentamos nuevos puntos de inyección de errores en el código fuente:
- **`mmap_flush_fail`** (en `src/storage.rs:395` / `VantaFile::flush()`): Simula un error I/O de Windows/OS al sincronizar el archivo de almacenamiento mapeado a disco (`msync`/`FlushViewOfFile`).
- **`hnsw_serialize_fail`** (en `src/index.rs:1547` y `src/index.rs:1636`): Simula un fallo de escritura o espacio en disco durante la serialización/persistencia física del índice HNSW (`CPIndex::persist_to_file` y `CPIndex::sync_to_mmap`).

### 2. Corrección del Bug de Durabilidad del Motor
- **Diagnóstico**: Durante la validación, descubrimos que `StorageEngine::flush()` no invocaba el método `.flush()` sobre el HNSW index de vectores subyacente. Esto causaba que el failpoint `mmap_flush_fail` nunca se ejerciera desde la interfaz del motor de almacenamiento.
- **Corrección**: Añadida la llamada a `self.vector_store.read().flush()?` dentro de `StorageEngine::flush()` en `src/storage.rs:1559`. Esto asegura durabilidad estricta y de extremo a extremo, propagando las órdenes de sincronización a todos los componentes de persistencia.

### 3. Suite de Pruebas de Caos (`chaos_integrity.rs`)
Se expandieron las pruebas en [chaos_integrity.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/storage/chaos_integrity.rs) a **4 escenarios completos de resiliencia**:
1. **`chaos_integrity_wal_failpoint_certification`**: Ejerce `wal_append_fail`, validando que ante un fallo en el log de transacciones, las operaciones de escritura se rechazan pero el estado en memoria no se corrompe y puede reanudar la operación.
2. **`chaos_integrity_storage_failpoint_certification`**: Ejerce `storage_insert_fail`, validando la recuperación automática tras fallos de inserción directa de registros de almacenamiento.
3. **`chaos_integrity_mmap_flush_failpoint_certification`**: Ejerce `mmap_flush_fail` en sincronización del almacenamiento y valida que el motor se recupera limpiamente.
4. **`chaos_integrity_hnsw_serialize_failpoint_certification`**: Ejerce `hnsw_serialize_fail` para garantizar la robustez durante fallos en la persistencia del índice espacial.

### 4. Automatización del Loop de Caos
- **Script Creado**: [chaos_loop.ps1](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/dev-tools/chaos_loop.ps1) en PowerShell.
- **Estrategia**: El script realiza una compilación única con `cargo test --test chaos_integrity --features failpoints --release --no-run`, extrae la ruta del binario compilado y ejecuta dicho binario directamente $N$ veces de forma secuencial.
- **Ventaja**: Esto evita compilar en cada iteración, reduciendo drásticamente la latencia por ciclo a ~580ms y permitiendo ejecutar 1,000 iteraciones en menos de 10 minutos (en lugar de horas). Produce un informe estructurado en `chaos_results.json` con metadatos y porcentaje de éxito.

### 5. Configuración de Nextest y Perfil Aislado
- **Nextest Config**: Se añadió el perfil `chaos` en [.config/nextest.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/.config/nextest.toml) con la directiva `test-threads = 1`.
- **Razón**: Los failpoints controlan estados globales compartidos dentro del proceso; ejecutar estas pruebas en paralelo generaría condiciones de carrera lógicas en las aserciones de fallos. El perfil `chaos` aísla de forma determinista la suite.

### 6. Documentación Operacional y README
- **Reliability Gate**: Se expandió el archivo [RELIABILITY_GATE.md](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/operations/RELIABILITY_GATE.md) integrando métricas y comandos para certificar fallos de caos, durabilidad en frío y estabilidad RSS.
- **README**: Se enlazó el documento de forma prominente en el [README.MD](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/README.MD) bajo la sección de desarrollo.

---

## Resultados de Pruebas y Certificación

1. **Ejecución de Tests Unitarios de Caos**:
   - Comando: `cargo test --test chaos_integrity --features failpoints --release -- --nocapture --test-threads=1`
   - **Resultado**: 100% exitoso. Todos los escenarios inyectaron correctamente los fallos y recuperaron la operatividad del motor.
2. **Smoke Run del Loop de Caos (10 iteraciones)**:
   - Comando: `.\dev-tools\chaos_loop.ps1 -Iterations 10 -Release`
   - **Resultado**: 10/10 PASS (100.00% éxito).
