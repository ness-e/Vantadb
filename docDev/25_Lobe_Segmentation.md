# Fase 25: Segmentación por Lóbulos (Storage Partitioning)

ConnectomeDB organiza el almacenamiento físico en **Lóbulos**, aprovechando la funcionalidad de **Column Families (CF)** de RocksDB. Esta compartimentación permite aplicar políticas de compresión, caché y auditoría diferenciadas por tipo de dato.

## Estructura de Lóbulos Predefinidos

### 1. Lóbulo Primario (Default)
Contiene la "Corteza Activa" del sistema.
- **Datos**: Nodos activos, relaciones (`edges`) y metadatos relacionales.
- **Optimización**: Cache agresivo en RAM. Prioridad alta en compactación L0/L1.
- **Uso**: Queries de tiempo real e inferencia inmediata.

### 2. Lóbulo de la Sombra (Shadow Kernel)
El "Subconsciente" o archivo forense.
- **Datos**: Nodos borrados (`AuditableTombstone`), registros de fallos de Axiomas y trazas de soberanía rechazada.
- **Optimización**: Compresión alta (ej. Zstd). Almacenamiento en capas de disco lento (Glacier Storage pattern).
- **Uso**: Auditoría post-mortem y Arqueología Semántica.

### 3. Lóbulo Histórico (Deep Memory)
Memoria consolidada de verdades inmutables.
- **Datos**: Neuronas de resumen (resultado del Olvido Bayesiano) y snapshots de estados de alta confianza.
- **Optimización**: Read-only y Bloom Filters exhaustivos. Prohibidas las mutaciones sin "Cirugía Lógica" manual.
- **Uso**: Entrenamiento continuo y transferencia de conocimiento.

---

## Ventajas Técnicas
1. **Aislamiento de I/O**: Las escrituras constantes en el Lóbulo Primario no bloquean las búsquedas pesadas en el Lóbulo Histórico.
2. **Escalabilidad Selectiva**: Es posible exportar un único Lóbulo (ej. el Histórico) para moverlo a otro nodo de ConnectomeDB (Federación de Lóbulos).
3. **Mantenimiento**: El `SleepWorker` puede compactar el Lóbulo de la Sombra de forma independiente, eliminando físicamente lápidas que han expirado su TTL sin afectar al Cortex.
