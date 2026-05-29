# Plan de Implementación: Write-Ahead Log Hardening, Durabilidad ACID y Auto-healing (Fase SEC-WAL)

Este plan establece las especificaciones técnicas para garantizar la durabilidad ACID (la "D" de ACID), la tolerancia a fallos del hardware (corrupción parcial de sectores) y la resiliencia en VantaDB mediante un Write-Ahead Log (WAL) ultra-resistente y políticas de sincronización deterministas.

---

## 1. Goal Description

El objetivo principal de la fase **SEC-WAL** es mitigar los riesgos de pérdida o corrupción silenciosa de datos debidos a caídas repentinas de energía (*power loss*), fallos catastróficos del sistema operativo o fallos físicos en sectores del disco.

Para ello implementaremos:
1.  **Garantía de Durabilidad ACID Configurable (`SyncMode`):** Asegurar que en `SyncMode::Always`, cada operación de escritura (`insert`, `update`, `delete`, `write_batch`) fuerce físicamente un `fsync`/`fdatasync` tanto en el WAL como en el backend de almacenamiento (`Fjall`/`RocksDB`) y en el almacén de vectores (`VantaFile`) antes de retornar éxito al usuario.
2.  **Algoritmo de Auto-healing Tolerante a Fallos con Escaneo hacia Adelante (*Scan-Forward*):** Modificar el mecanismo de lectura y validación del WAL en `src/wal.rs` para que, en caso de detectar un registro corrupto (fallo de CRC32C Castagnoli o truncamiento parcial), escanee secuencialmente byte a byte en busca del siguiente registro válido (verificado por CRC y deserialización estructural). Solo se truncará el WAL si no se encuentra ningún registro válido aguas abajo hasta alcanzar el final del archivo (EOF).
3.  **Suite de Pruebas de Caos e Inyección de Fallos:** Diseñar pruebas en `tests/storage/chaos_integrity.rs` and `tests/storage/wal_resilience.rs` que simulen corrupciones intermedias y finales en el WAL, asegurando que el motor de almacenamiento de VantaDB se auto-recupere perfectamente y restaure el estado consistente sin pérdida de transacciones válidas posteriores.

---

## 2. User Review Required

> [!IMPORTANT]
> **Penalización de Rendimiento en `SyncMode::Always`:**
> El modo `SyncMode::Always` garantiza máxima seguridad transaccional, pero obliga al cabezal del disco a esperar la confirmación física de escritura en cada transacción (latencia típica de ~1ms a ~10ms en SSDs). Hemos verificado que la propagación del `fsync` afectará a:
> - El archivo de WAL (`vanta.wal`).
> - La persistencia del KV backend (`Fjall::db.persist()` / `RocksDb::db.flush()`).
> - Las cabeceras y vectores en el archivo mapeado `vector_store.vanta`.
>
> **Recomendación FMEA:** Mantener `SyncMode::Periodic` (el valor por defecto) para cargas de trabajo estándar, y usar `SyncMode::Always` únicamente en entornos críticos o financieros.

> [!WARNING]
> **Estrategia de Alineación del WAL:**
> Al escanear hacia adelante tras una corrupción intermedia, la porción corrupta del archivo WAL se mantendrá físicamente en el disco (actuando como un "agujero" de datos omitido en el replay). Los nuevos registros válidos se escribirán al final del archivo WAL. Esto asegura un rendimiento append-only puro, sin bloqueos costosos para reescribir archivos en caliente.

---

## 3. Proposed Changes

### Componente 1: Propagación y Sincronización en `StorageEngine`

#### [MODIFY] [src/storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs)
**Cambios:** En las operaciones de mutación física del motor (`insert`, `delete`, `consolidate_node`, `insert_to_cf`), comprobaremos si el modo de sincronización de la configuración es `SyncMode::Always`. Si lo es, forzaremos la descarga a disco físico de todos los componentes afectados (WAL, vector_store, backend) antes de responder exitosamente.

```diff
     pub fn insert(&self, node: &UnifiedNode) -> Result<()> {
         self.ensure_writable()?;
         #[cfg(feature = "failpoints")]
         fail::fail_point!("storage_insert_fail", |_| {
             Err(VantaError::IoError(std::io::Error::new(
                 std::io::ErrorKind::Other,
                 "Simulated Storage insert catastrophic I/O failure",
             )))
         });
         #[cfg(feature = "governance")]
         if self.admission_filter.is_blocked(node.id) {
             return Err(VantaError::Execution(format!(
                 "Node {} is blocked by AdmissionFilter (recently rejected)",
                 node.id
             )));
         }
 
         self.touch_activity();
 
         let mut active_node = node.clone();
         active_node.last_accessed = SystemTime::now()
             .duration_since(UNIX_EPOCH)
             .unwrap_or_default()
             .as_millis() as u64;
 
         if let Some(ref mut wal_writer) = *self.wal.lock() {
             wal_writer.append(&crate::wal::WalRecord::Insert(active_node.clone()))?;
         }
 
         let storage_offset = self.append_to_vstore(&active_node)?;
 
         let key = active_node.id.to_le_bytes();
         let metadata = NodeMetadata {
             relational: active_node.relational.clone(),
             edges: active_node.edges.clone(),
         };
         let metadata_val = bincode::serialize(&metadata)
             .map_err(|e| VantaError::SerializationError(e.to_string()))?;
         self.backend
             .put(BackendPartition::Default, &key, &metadata_val)?;
 
         {
             let mut hnsw = self.hnsw.write();
             hnsw.add(
                 active_node.id,
                 active_node.bitset,
                 active_node.vector.clone(),
                 storage_offset,
             );
         }
 
         if active_node.tier == crate::node::NodeTier::Hot {
             let mut cache = self.volatile_cache.write();
             cache.insert(active_node.id, active_node.clone());
 
             let caps = crate::hardware::HardwareCapabilities::global();
             let cache_cap_bytes = caps.total_memory / 4;
             let approx_node_size = 1536;
             let max_nodes = (cache_cap_bytes / approx_node_size) as usize;
 
             if cache.len() > max_nodes {
                 self.emergency_maintenance_trigger
                     .store(true, Ordering::Release);
             }
         }
 
+        // En SyncMode::Always, forzamos flush/sync en disco para garantizar durabilidad física
+        if self.config.sync_mode == crate::config::SyncMode::Always {
+            if let Some(ref mut wal_writer) = *self.wal.lock() {
+                wal_writer.sync()?;
+            }
+            self.vector_store.read().flush()?;
+            self.backend.flush()?;
+        }
+
         Ok(())
     }
```

---

### Componente 2: Algoritmo de Escaneo hacia Adelante (*Scan-Forward*) en `wal.rs`

#### [MODIFY] [src/wal.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/wal.rs)
**Cambios:** Reescribir el loop de escaneo secuencial en `WalWriter::open` y la lectura secuencial en `WalReader` para soportar tolerancia a fallos intermedia:
1.  Si ocurre un error de lectura o falla el CRC32C, en lugar de romper el loop inmediatamente, entraremos en modo *Scan-Forward*.
2.  Buscaremos secuencialmente byte a byte la firma de un registro que:
    *   Tenga un prefijo `len: u32` válido.
    *   Su carga útil de tamaño `len` y sus 4 bytes de CRC almacenado coincidan perfectamente (`crc32c(payload) == stored_crc`).
    *   Se pueda deserializar exitosamente con `bincode::deserialize::<WalRecord>`.
3.  Si encontramos un registro válido downstream, se registra y se continúa el replay.
4.  Si llegamos a EOF sin encontrar ningún otro registro válido, truncamos el archivo en el byte inmediatamente posterior al último registro válido confirmado.

---

### Componente 3: Suite de Caos Extendido e Inyección de Corrupción

#### [MODIFY] [tests/storage/wal_resilience.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/storage/wal_resilience.rs)
**Cambios:** Añadiremos un nuevo test de estrés `test_wal_middle_corruption_auto_healing()` para validar:
1.  La escritura de múltiples nodos.
2.  La corrupción deliberada de bytes intermedios en el archivo WAL.
3.  La escritura de nuevos nodos válidos adicionales posteriores a la sección corrupta.
4.  El reinicio de la base de datos y la correcta restauración de todos los nodos sanos (incluyendo los que estaban *después* del agujero de corrupción), confirmando que el auto-healing no los descartó.

---

## 4. Verification Plan

### Automated Tests (A ejecutar localmente para certificar)
1.  **Ejecución de la Suite de Resiliencia del WAL:**
    ```powershell
    cargo test --test wal_resilience -- --nocapture
    ```
2.  **Ejecución de la Suite de Caos e Inyección de Fallos:**
    ```powershell
    cargo test --test chaos_integrity --features failpoints -- --nocapture
    ```
3.  **Verificación de Regresiones Generales:**
    ```powershell
    cargo check --all-targets --features "experimental,failpoints"
    ```
