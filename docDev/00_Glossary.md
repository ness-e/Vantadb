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

---

### Reglas de Codificación
1. **Nomenclature in Code**: Se prefiere el uso de nombres biológicos para `Structs` y `Traits` públicos para mejorar la expresividad de la API.
2. **Nomenclature in Internals**: Los nombres técnicos pueden usarse en módulos de bajo nivel (ej: `storage.rs`) para facilitar el mantenimiento por desarrolladores de bases de datos tradicionales.
