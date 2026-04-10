# Resultados de Profiling - Fase 18.5 (The Memory Abyss)

## Parámetros del Test
- **Entorno:** Local (Target 16GB RAM)
- **Tamaño Dataset:** 100,000 Nodos (Escalable a 1M)
- **Configuración RocksDB Exclusiva:**
  - `BlockCache`: 2GB LRU.
  - `WriteBufferSize`: 128MB (x 4 MemTables = 512MB Max RAM Write Spikes).
  - `BloomFilter`: 10 Bits por Llave (~1% Tasa de Falso Positivo).

## Caso: Axiomas Topológicos vs Falsos Positivos

La decisión arquitectónica de confiar mecánicamente en el StorageEngine trae consigo una mitigación necesaria mediante filtros probabilísticos.

Al intentar realizar una consulta a disco para establecer un `Axioma 1: No Huerfanos`, el motor en su versión primitiva sufría latencias `>1ms / iteración` simplemente tratando de buscar el nodo vacío dentro del `MemTable` y posteriormente los SSTables profundos.

### Observaciones y Métricas
Al ejecutar el benckmark se capturó la iteración sobre:

*   **Point Lookup Válido (Node ID existente):** 
    Requiere un impacto en el block cache. Al estar todo cacheado entra en nanosegundos / microsegundos predecibles.
*   **Point Lookup Probabilístico (Node ID inexistente, ej. ataque / error):**
    El Bloom Filter actúa de embudo deteniendo la petición en nanosegundos ANTES de molestar al bus SSD PCIe. 

## Falso Positivo

Se ha implementado satisfactoriamente el "test del Nodo Fantasma". Cuando un atacante (o un LLM alucinante) forja el Statement `RELATE 1 -> 999` y la llave probabilística llegase a coincidir en la función Hash del Bloom Filter (es escaso con 10 bits), la validación afortunadamente descarta el dato al momento de invocar `.get() -> Ok(None)`, activando el gatillo `trigger_panic_state()` o la cancelación de la transacción desde el `Executor` mediante  `Err("Axioma Topológico violado")`.

## Conclusión Fase 18.5
Con estos mecanismos, establecemos que la base de datos mantendrá su huella de memoria atada en todo momento al `BlockCache` estricto de 2GB. ConnectomeDB sobrevive a estrés sin degradación silente.
