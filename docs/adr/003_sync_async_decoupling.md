# ADR 003: Desacoplamiento Sync/Async y Aislamiento de Ejecución Concurrente

## Estado

Estado: Aprobado

## Contexto

El diseño original de VantaDB acoplaba la biblioteca principal (`vantadb`) con el runtime asíncrono Tokio. Esto obligaba a los consumidores de la base de datos (p. ej., aplicaciones de línea de comandos, servicios síncronos nativos o wrappers de lenguajes dinámicos como Python/PyO3) a levantar y coordinar un runtime asíncrono pesado únicamente para interactuar con la base de datos local embebida.
Adicionalmente, mezclar operaciones de bloqueo intensivo de disco e indexación masiva de CPU (como el recorrido de grafos HNSW) directamente dentro del pool de hilos de Tokio para el tráfico de red provocaba inanición (starvation) de tareas de red y degradación grave de la latencia P99 del servidor.

## Decisión

Para resolver este cuello de botella arquitectónico y garantizar un motor embebido de grado industrial altamente portable y con latencias estables, se implementó un desacoplamiento estricto de hilos de ejecución en dos niveles:

1. **Purificación Síncrona del Núcleo (`vantadb`):**
   * Retirar por completo todas las dependencias y abstracciones de Tokio, `async/await`, canales de comunicación asíncronos y futuros del crate `vantadb`.
   * El núcleo se compila de forma 100% síncrona pura mediante el flag `--no-default-features`.
   * Toda la concurrencia a nivel de almacenamiento y de índices se gestiona internamente mediante primitivas de sincronización síncronas estándar (`std::sync::{Arc, RwLock, Mutex}`) altamente eficientes, permitiendo una integración limpia con RocksDB y Fjall.

2. **Aislamiento en la Frontera del Servidor (`vantadb-server`):**
   * El servidor (`vantadb-server`) sigue haciendo uso de Tokio para gestionar la infraestructura de red, despacho de conexiones TCP y el servidor de protocolo MCP.
   * Sin embargo, toda llamada al motor síncrono subyacente se despacha en la frontera mediante hilos dedicados de bloqueo usando la primitiva `tokio::task::spawn_blocking`.
   * Para evitar el agotamiento descontrolado de hilos de sistema debido a ráfagas de consultas intensivas, se implementó un pool y semáforo estricto gobernado por el parámetro `VantaConfig::max_blocking_threads`:

     ```rust
     let permit = self.blocking_semaphore.acquire().await?;
     let db = self.db.clone();
     let result = tokio::task::spawn_blocking(move || {
         db.execute_hybrid(query)
     }).await?;
     ```

## Consecuencias

### Beneficios

* **Portabilidad Total de Integración (SDK de Python):** El wrapper de PyO3 `vantadb-python` interactúa directamente de forma nativa síncrona con el motor embebido de VantaDB, sin necesidad de levantar hilos de Tokio ni de acoplarse a loops de eventos asíncronos de Python (asyncio), permitiendo llamadas limpias y rápidas.
* **Control de Latencia P99 Bajo Estrés:** Al limitar y aislar el número máximo de hilos concurrentes que pueden bloquear la CPU o el almacenamiento, el reactor de red de Tokio permanece libre en todo momento para enrutar conexiones TCP y responder solicitudes MCP de forma inmediata.
* **Mantenibilidad:** El código del núcleo de almacenamiento es mucho más limpio, fácil de razonar y debugear sin abstracciones asíncronas innecesarias o ciclos de vida complejos de futuros.

### Deuda Técnica / Costos

* **Context Switch Overhead:** La separación en la frontera Sync/Async introduce un costo mínimo de cambio de contexto de hilos al despachar tareas a `spawn_blocking`. Sin embargo, las ganancias en estabilidad del reactor de red compensan con creces este costo.
* **Regulación y Ajuste de Semáforo:** El parámetro `max_blocking_threads` debe ser calibrado cuidadosamente de acuerdo con los núcleos físicos disponibles en el hardware de despliegues para evitar contención de CPU excesiva o colas de espera prolongadas en el servidor.
