# Especificación Técnica: Mantenimiento Circadiano (Sleep Worker)

## 1. Meta Arquitectónica
Emular el ciclo del sueño biológico en ConnectomeDB. Durante el día (alta demanda de I/O), la base de datos debe ser extremadamente rápida alojando información transitoria (Memoria Corto Plazo / STN) en arreglos RAM (Cortex Volátil). 
Durante los periodos de inactividad, un hilo de limpieza en segundo plano (Sleep Worker) ejecutará una "Fase REM" para evaluar, degradar o consolidar los datos hacia la persistencia a largo plazo (LTN / RocksDB).

## 2. Componentes del Diseño

### Cortex Context (Capa RAM STN)
Actualmente el `StorageEngine` delega todo a `RocksDB`, confiando ciegamente en el BlockCache subyacente. Para habilitar un control heurístico, inyectaremos un HashMap Atómico (`cortex_ram`) que actúe como un L1 Cache explícito para nodos volátiles y mutaciones activas.
Además, se añade un `last_query_timestamp` (AtomicU64) para perfilar los periodos de inactividad.

### SleepWorker Daemon (`src/governance/sleep_worker.rs`)
Un loop de tokio desacoplado del pool principal.

- **Cadencia:** Se despierta cada `X` segundos (configurable, ej. 10s).
- **Inception Condition:** Solo opera si `now() - last_query_timestamp > 5000ms`.
- **Interrupción (Yield):** En medio del bucle pesado de iteración sobre memoria, si nota que el `last_query_timestamp` ha sido actualizado por una petición de usuario entrante, ejecuta `tokio::task::yield_now()` cesando su barrido inmediatamente.

### Algoritmos Heurísticos
1. **Olvido Bayesiano:** Por cada iteración REM sobre la RAM, el campo `hits` de las neuronas se divide en 2 (`hits *= 0.5`).
2. **Migración STN -> LTN (Consolidación):** Si `hits < UMBRAL` y no posee el flag `PINNED`, el nodo es movido del HashMap al RocksDB Column Family "default".
3. **Poda hacia el Shadow Archive:** Si el nodo (al consolidarse o encontrarse en el almacenamiento primario) posee un `trust_score < 0.2`, se migra físicamente como lápida hacia el `shadow_kernel`.
