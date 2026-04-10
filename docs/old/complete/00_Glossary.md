# ConnectomeDB: Glosario de Alineación Semántica

Este documento define la terminología dual de ConnectomeDB. Para mantener el rigor técnico sin perder la potencia de la metáfora biográfica, cada componente de software tiene un alias de dominio.

| Término Biológico | Equivalente Técnico | Descripción en el Sistema |
|:---:|:---:|:---|
| **Neuron** (Neurona) | `UnifiedNode` | La unidad mínima de información. Contiene vectores, grafos y campos relacionales. |
| **Synapse** (Sinapsis) | `Edge` | Conexión pesada y dirigida entre dos neuronas. |
| **Cortex** (Corteza) | `Query Planner` | El motor que decide la ruta de ejecución y optimiza la consulta entre las 3 dimensiones. |
| **Lobe** (Lóbulo) | `Partition / CF` | Región funcional de almacenamiento físico (implementado vía RocksDB Column Families). |
| **Shadow Kernel** (Núcleo Sombra)| `Audit Layer` | Capa de subconsciente donde residen las lápidas (tombstones) y datos en cuarentena. |
| **Cognitive Fuel** (Combustible) | `Resource Quota` | Límite de computación para evitar bucles infinitos en reglas lógicas (DoS protection). |
| **Axon** (Axón) | `Stream / WAL` | El flujo de datos secuencial que garantiza la durabilidad (Write-Ahead Log). |
| **Sleep Worker** | `GC / Maintenance` | Proceso en segundo plano que consolida la memoria y aplica el Olvido Bayesiano. |
| **Neural Index** | `HNSW Index` | Estructura de navegación vectorial optimizada para búsqueda semántica. |
| **Amygdala Budget** | `semantic_valence guard` | Presupuesto que protege el 5% de nodos de mayor valencia semántica contra el olvido. |
| **Neural Summary** | `NeuralSummary node` | Neurona de Resumen creada por compresión LLM de un grupo de nodos degenerados. |
| **Rehydration** | `StorageEngine::rehydrate()` | Arqueología Semántica: recuperación zero-copy de nodos archivados en el Shadow Kernel. |
| **StaleContext** | `ExecutionResult::StaleContext` | Señal no-bloqueante emitida cuando un resumen tiene TrustScore crítico (< 0.4). |
| **Quantum Neuron** | `QuantumNeuron` | (v0.5.0) Nodo en superposición que mantiene candidatos contradictorios hasta colapso. |
| **Synaptic Depression** | `Edge decay` | (v0.5.0) Decaimiento circadiano del peso de edges no traversados. |

---

### Reglas de Codificación
1. **Nomenclature in Code**: Se prefiere el uso de nombres biológicos para `Structs` y `Traits` públicos para mejorar la expresividad de la API.
2. **Nomenclature in Internals**: Los nombres técnicos pueden usarse en módulos de bajo nivel (ej: `storage.rs`) para facilitar el mantenimiento por desarrolladores de bases de datos tradicionales.
