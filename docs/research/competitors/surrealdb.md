# **Análisis de Ingeniería Inversa de SurrealDB: Arquitectura de Persistencia, Lógica Cognitiva y Estrategias de Optimización para ConnectomeDB**

El desarrollo de una base de datos cognitiva como ConnectomeDB, inspirada en la neurobiología y construida sobre Rust, exige un análisis riguroso de los motores multimodelo contemporáneos. SurrealDB se presenta como el referente técnico más cercano debido a su capacidad para unificar documentos, grafos, vectores y lógica relacional en un único binario.1 El presente reporte desglosa la arquitectura de SurrealDB mediante un proceso de ingeniería inversa basado en su comportamiento lógico, documentación técnica y el análisis de su implementación en Rust, con el fin de proporcionar una hoja de ruta técnica para la construcción de ConnectomeDB.

## **Anatomía de la "Neurona" (Estructura de Datos)**

La unidad fundamental de información en SurrealDB no es un registro estático, sino una estructura dinámica que se adapta a múltiples paradigmas. Internamente, SurrealDB opera como una capa de abstracción sobre motores de almacenamiento de clave-valor (Key-Value), transformando datos complejos y relaciones en representaciones binarias ordenadas.3 Esta elección arquitectónica es fundamental para una base de datos cognitiva, ya que permite que cada "neurona" o nodo de información posea una flexibilidad total en su contenido sin sacrificar la eficiencia de la recuperación a bajo nivel.

### **Estructura Interna de Almacenamiento**

SurrealDB no utiliza un formato de almacenamiento propio de forma exclusiva, sino que separa la capa de cómputo (Query Layer) de la capa de almacenamiento (Storage Layer).3 En la capa de almacenamiento, la información se organiza en una lista ordenada de claves y valores. Cada motor de almacenamiento (RocksDB, TiKV, SurrealKV) debe soportar operaciones atómicas basadas en transacciones y la capacidad de leer y escribir claves individuales o rangos de claves.3

En el nivel lógico, un registro se identifica mediante un Record ID, que consta del nombre de la tabla y un identificador único (ej: persona:usuario\_01).6 A diferencia de los sistemas relacionales donde las claves foráneas son simples enteros o strings, los Record IDs en SurrealDB son punteros lógicos que el planificador de consultas utiliza para saltar directamente a la ubicación física del dato en el motor KV, eliminando la necesidad de escaneos de tabla para resolver enlaces.7

### **Formato de Serialización y Persistencia en Disco**

Para la persistencia y la comunicación entre capas, SurrealDB utiliza una implementación extendida de CBOR (Concise Binary Object Representation).8 La elección de CBOR sobre JSON responde a la necesidad de un formato binario compacto que soporte tipos de datos complejos de forma nativa, facilitando la integración con el ecosistema de Rust a través de serde.9

SurrealDB ha extendido el estándar CBOR con etiquetas (tags) personalizadas para representar sus tipos de datos únicos. Esta técnica es esencial para mantener la integridad de los metadatos sin incurrir en el overhead de la conversión de tipos en tiempo de ejecución.

| Tag CBOR | Tipo de Dato | Representación Interna | Función en ConnectomeDB |
| :---- | :---- | :---- | :---- |
| Tag 8 | Record ID | Array de dos valores: | Referencia sináptica directa |
| Tag 12 | Datetime | Segundos y nanosegundos (compacto) | Marcas de tiempo de activación |
| Tag 37 | UUID | Formato binario de 16 bytes | Identificador único de neurona |
| Tag 49 | Range | Estructura con límites inclusivos/exclusivos | Delimitación de espacios latentes |
| Tag 88 | Geometry | Punto geográfico (Lat, Lon) | Ubicación en el espacio cognitivo |

La serialización CBOR permite que SurrealDB maneje metadatos asociados a los objetos (como versiones temporales o permisos de acceso a nivel de registro) de manera eficiente, integrándolos directamente en la carga útil binaria o como claves prefijadas en el motor de almacenamiento.8

### **Gestión de Metadatos y Objetos**

En la versión 3.0, SurrealDB introdujo una separación crítica entre los valores y las expresiones, así como un nuevo modelo para los metadatos del catálogo (namespaces, bases de datos e índices).12 Anteriormente, los nombres de estos recursos se utilizaban directamente, pero ahora se han movido a un almacenamiento basado en IDs de tamaño fijo. Esto ha reducido el tamaño de las claves en disco de manera significativa; por ejemplo, una clave que antes ocupaba 80 bytes ahora se reduce a 42 bytes, optimizando el rendimiento de I/O y la utilización de la caché del sistema de archivos.12

Los metadatos de los objetos incluyen:

* **Computed Fields:** Campos que se evalúan solo cuando es necesario, sustituyendo al antiguo tipo future. Esto reduce la carga computacional por fila durante los escaneos.12  
* **Document Wrapper:** Un tipo de contenedor que separa el contenido del registro de los metadatos del sistema (como el sellado de tiempo de versión), evitando que los datos internos del motor "contaminen" las respuestas enviadas al usuario.12

## **Lógica de Recuperación y Búsqueda**

La recuperación de información en SurrealDB se bifurca en tres estrategias principales que deben coexistir para permitir una arquitectura cognitiva: búsqueda vectorial para similitud semántica, travesía de grafos para relaciones estructurales y filtrado híbrido para precisión contextual.13

### **Recuperación Vectorial: HNSW y Desempeño**

SurrealDB implementa el algoritmo HNSW (Hierarchical Navigable Small World) para la búsqueda de vecinos más cercanos aproximados (ANN).13 HNSW es un algoritmo basado en grafos que organiza los vectores en capas jerárquicas, donde las capas superiores actúan como "autopistas" para saltos largos y las capas inferiores permiten una búsqueda local refinada.16

El motor gestiona el compromiso entre precisión (Recall) y velocidad (Latency) mediante parámetros configurables en el momento de la definición del índice:

1. **M (Max Connections):** Determina cuántos vecinos se conectan a cada nodo. Un valor mayor incrementa el recall pero aumenta linealmente el consumo de RAM y el tiempo de construcción.16  
2. **efConstruction:** Controla la profundidad de la búsqueda durante la fase de inserción. SurrealDB sugiere valores entre 150 y 500 para equilibrar la calidad del grafo final con el tiempo de carga.13  
3. **efSearch:** El parámetro más crítico en tiempo de consulta. Un efSearch alto garantiza que se exploren más candidatos, mejorando la probabilidad de encontrar los verdaderos vecinos más cercanos a costa de latencia adicional.16

En términos de rendimiento, la arquitectura de SurrealDB 3.0 ha optimizado las búsquedas vectoriales, logrando una reducción de latencia de aproximadamente el 800% (de 38s a 4.5s en benchmarks específicos) en comparación con la versión 2.0.12 Esta mejora se debe a un nuevo planificador de consultas que minimiza el trabajo redundante dentro del motor de ejecución.12

### **Resolución de Grafos y Saltos entre Nodos**

A diferencia de las bases de datos de grafos puras que utilizan "Index-free Adjacency" (donde cada nodo físico contiene punteros directos a los offsets de sus vecinos), SurrealDB utiliza un enfoque basado en tablas de relación (Edge Tables) y Record IDs.7

* **Traversals:** La resolución de saltos se realiza mediante la sintaxis de flechas \-\>. Cuando se ejecuta una consulta como SELECT \-\>es\_amigo\_de-\>persona FROM usuario:1, el planificador no realiza un JOIN relacional tradicional. En su lugar, busca en la tabla de bordes (es\_amigo\_de) los registros donde el campo in coincida con usuario:1.20 Dado que estos campos están indexados, el salto es extremadamente rápido.  
* **Rich Edges:** Los bordes en SurrealDB son documentos completos. Pueden almacenar propiedades, timestamps y metadatos, lo que permite filtrar las relaciones durante la travesía (ej: SELECT \-\>conoce-\>persona).22  
* **Bidireccionalidad:** Desde la versión 3.0, SurrealDB permite definir referencias bidireccionales a nivel de esquema mediante la cláusula REFERENCE, lo que facilita la travesía inversa sin necesidad de definir manualmente tablas de bordes en ambas direcciones.19

### **Filtrado Híbrido y Fusión de Resultados**

La capacidad de combinar búsqueda vectorial con condiciones de texto o metadatos se resuelve mediante tres mecanismos de filtrado 24:

1. **Pre-filtering:** El filtro se aplica *durante* la travesía del grafo HNSW. Esto garantiza que se devuelvan ![][image1] resultados que cumplan la condición, pero puede ser muy costoso si el filtro es altamente selectivo (poca densidad de aciertos), obligando al motor a explorar gran parte del grafo.24  
2. **Post-filtering:** Se obtienen los ![][image1] vecinos más cercanos y luego se descartan los que no cumplen la condición. Es rápido pero puede devolver menos de ![][image1] resultados o incluso ninguno si los vecinos más cercanos no pasan el filtro.24  
3. **RRF (Reciprocal Rank Fusion):** Para combinar búsquedas de texto completo (BM25) y búsquedas vectoriales, SurrealDB utiliza la función search::rrf(). Esta técnica suma los recíprocos de los rangos de los documentos en ambas listas de resultados para generar una puntuación unificada, lo que mejora drásticamente el recall en sistemas RAG (Retrieval-Augmented Generation).25

## **Gestión de Memoria y Estado**

La eficiencia de ConnectomeDB dependerá de cómo maneje el estado masivo de una red cognitiva. SurrealDB ofrece lecciones valiosas sobre la gestión de caché y la concurrencia en entornos Rust.

### **Modelos de Memoria y Caché**

SurrealDB implementa múltiples estrategias de caché dependiendo del motor de almacenamiento utilizado. Para RocksDB, se utiliza un "Block Cache" que puede configurarse mediante la variable de entorno SURREAL\_ROCKSDB\_BLOCK\_CACHE\_SIZE. Por defecto, el sistema intenta utilizar aproximadamente el 50% de la memoria RAM disponible para este fin.28

En el caso del motor en memoria SurrealMX (introducido en 3.0), se utiliza un diseño basado en MVCC (Multi-Version Concurrency Control) y estructuras de datos bloque-libres (lock-free).19 SurrealMX organiza el proceso de confirmación de transacciones en un pipeline segmentado:

* **Analizar:** El motor procesa la consulta y genera el plan de ejecución.  
* **Validar:** Se comprueban las restricciones de esquema y permisos.  
* **Persistir/Confirmar:** La transacción se aplica al estado global. Este diseño permite que múltiples transacciones se encuentren en diferentes etapas del pipeline simultáneamente, emulando la segmentación de instrucciones de un procesador moderno.30

### **Arquitecturas Zero-Copy y Apache Arrow**

A pesar de ser una base de datos de alto rendimiento, SurrealDB aún no implementa de forma nativa una integración profunda con Apache Arrow para transferencias "Zero-Copy" en su núcleo de consulta.31 Actualmente, el sistema depende de la serialización CBOR para mover datos entre la capa de almacenamiento y la capa de cómputo.9

Sin embargo, el ecosistema de Rust y proyectos como DataFusion (que utiliza Arrow) demuestran que el futuro de las bases de datos analíticas y cognitivas reside en evitar la copia de datos entre procesos.31 Arrow permite que diferentes herramientas (ej: un motor de inferencia de IA y ConnectomeDB) lean el mismo buffer de memoria sin necesidad de serialización o transposición de columnas a filas, lo cual es una oportunidad crítica para ConnectomeDB.32

### **Concurrencia, Bloqueos y "Olvido"**

La gestión de bloqueos en SurrealDB ha presentado desafíos técnicos, especialmente en la indexación vectorial concurrente. Se han reportado problemas de "inanición" (starvation) del sistema cuando las búsquedas vectoriales lentas (que realizan E/S de disco) mantienen bloqueos de lectura (RwLock de Tokio) durante demasiado tiempo, impidiendo que las actualizaciones del índice progresen.35

Respecto al mecanismo de "Olvido", SurrealDB no posee una lógica de decaimiento biológico, pero utiliza:

* **TTL (Time-To-Live):** Aunque no es una función centralizada de "olvido cognitivo", se puede implementar mediante eventos y tablas de vista que eliminan datos subyacentes una vez procesados.36  
* **Compactación LSM:** Los motores SurrealKV y RocksDB realizan una compactación periódica para limpiar versiones antiguas de registros (garbage collection de versiones) y reclamar espacio en disco.37

## **Análisis de la Documentación y API**

SurrealDB destaca por una experiencia de desarrollador (DX) que simplifica la complejidad de los sistemas multimodelo a través de SurrealQL y sus SDKs.1

### **Innovaciones en el SDK y SurrealQL**

El lenguaje de consulta SurrealQL es una de las piezas más innovadoras, combinando la familiaridad de SQL con la potencia de los grafos.1 Sus características más destacadas son:

* **Record Links Directos:** Acceso a registros enlazados mediante notación de punto (ej: SELECT autor.nombre FROM post), lo que simplifica enormemente la sintaxis de consulta en comparación con los JOINs de SQL.7  
* **Live Queries:** Permiten a los desarrolladores suscribirse a cambios en tiempo real. Ejecutar un LIVE SELECT devuelve un UUID único que representa el flujo de notificaciones para el cliente.39  
* **Surrealism (Extensiones WASM):** Permite ejecutar plugins de WebAssembly directamente en el servidor. Esto es fundamental para ConnectomeDB, ya que permitiría inyectar lógica de procesamiento neuronal (ej: activaciones de redes LISP) que se ejecuten con rendimiento nativo cerca de los datos.19

### **Problemas Comunes en Escala Masiva**

Los desarrolladores han reportado varios puntos de fricción en implementaciones de gran tamaño:

1. **Consumo de RAM no lineal:** Durante inserciones masivas de datos (decenas de millones), el consumo de memoria puede crecer desproporcionadamente. Con SurrealKV, se han observado errores de OOM (Out of Memory) al superar los 30-40 millones de registros en sistemas de 16 GB, debido a que el índice debe residir en memoria para un rendimiento óptimo.38  
2. **Regresiones en Ordenamiento:** En la versión 3.0, algunas consultas que combinan WHERE y ORDER BY sobre campos JSON anidados han mostrado una degradación de rendimiento, siendo hasta 22 veces más lentas que las consultas sobre campos de nivel superior.41  
3. **Latencia en WebSockets:** Se identificaron fugas de memoria en el cliente de WebSockets del SDK de Rust al realizar operaciones de consulta masivas, un problema que fue corregido recientemente mediante la limpieza de solicitudes pendientes.42

## **Inspiración para ConnectomeDB (Features para extraer)**

Para que ConnectomeDB sea competitiva, debemos integrar y adaptar las siguientes lógicas probadas de SurrealDB al contexto neurobiológico.

### **1\. Sistema de Identificadores de Registro como "Sinapsis Lógicas"**

ConnectomeDB debe adoptar la lógica de Record IDs de SurrealDB (tabla:id), pero expandiéndola para soportar la jerarquía de una red neuronal. En SurrealDB, un ID puede ser un array o un objeto (ej: sensor\_readings:\[location:1, sensor:A, d'2024-01-01'\]), lo que permite realizar escaneos de partición extremadamente eficientes sin índices adicionales.36

**Adaptación:** Podemos implementar "Sinapsis Predictivas" donde los IDs de los bordes contengan el hash del contenido de los nodos que conectan, permitiendo verificar la integridad del grafo cognitivo a gran velocidad y realizar búsquedas de rango sobre el tiempo de activación de las neuronas.

### **2\. Capa de Lógica Embebida vía WASM (Surrealism)**

La capacidad de ejecutar lógica personalizada en el motor mediante WebAssembly es la forma más eficiente de implementar la lógica LISP de ConnectomeDB.19

**Adaptación:** En lugar de un intérprete LISP tradicional lento, ConnectomeDB puede compilar fragmentos de lógica LISP a bytecode de WASM en tiempo de ejecución. Estos "Enfermas Neuronales" (scripts de activación) se ejecutarían dentro de la sandbox de la base de datos, teniendo acceso directo a los vectores y al grafo sin el coste de serialización hacia una capa de aplicación externa.

### **3\. Fusión de Recuperación Multimodal (RRF Nativo)**

La implementación de search::rrf() en SurrealDB es el estándar de oro para sistemas de memoria de agentes.14

**Adaptación:** Para ConnectomeDB, esta lógica debe evolucionar hacia un "Ranking de Atención". En lugar de una fusión estática, ConnectomeDB debería permitir que la relevancia de un dato sea una función de su similitud vectorial (semántica), su conectividad en el grafo (importancia estructural) y su "potencial de acción" (recencia y frecuencia de uso), todo calculado en una única pasada de consulta.

## **Puntos Débiles (Oportunidad de Mercado)**

ConnectomeDB puede superar a SurrealDB abordando sus fallos estructurales y ofreciendo una arquitectura más ligera y moderna.

### **El Problema de la Memoria y la Dependencia de C++**

SurrealDB depende en gran medida de RocksDB para el almacenamiento persistente estable en un solo nodo.3 RocksDB está escrito en C++, lo que complica la compilación cruzada en Rust y limita la optimización profunda del recolector de basura de memoria en entornos embebidos.5 Además, el requisito de SurrealKV de mantener todo el índice en RAM para ser eficiente es una barrera para dispositivos de borde (edge) con recursos limitados.38

**Solución de ConnectomeDB:** Al utilizar un motor de almacenamiento 100% Rust (como un LSM-tree optimizado para memoria compartida) y una arquitectura basada en Apache Arrow para el procesamiento de vectores, ConnectomeDB puede reducir el consumo de RAM en un 60-70% al evitar copias innecesarias de datos entre el disco y el motor de ejecución.

### **Rigidez en la Arquitectura Distribuida**

La escalabilidad horizontal de SurrealDB depende de sistemas externos masivos como TiKV o FoundationDB.3 Esto hace que desplegar un clúster de SurrealDB sea una tarea compleja que requiere gestionar múltiples servicios de infraestructura.

**Solución de ConnectomeDB:** Implementar un protocolo de consenso ligero (como Raft o paxos) directamente en el binario de ConnectomeDB. Esto permitiría una arquitectura "Zero-Config Cluster", donde añadir un nuevo nodo cognitivo sea tan simple como apuntar a la dirección IP del nodo maestro, manteniendo la simplicidad de un único binario de Rust sin dependencias de sistemas de terceros.

### **Inexistencia de "Olvido Cognitivo" Dinámico**

SurrealDB es excelente para retener datos, pero falla en la gestión del ciclo de vida biológico de la información. El borrado es una operación binaria: o el dato existe, o no.36

**Solución de ConnectomeDB:** Introducir el concepto de "Decaimiento de Peso Sináptico". ConnectomeDB puede implementar un sistema donde los registros pierdan "fuerza" (weight) si no son consultados. Al alcanzar un umbral, el sistema podría automáticamente archivar o resumir la información (usando LLMs locales o lógica LISP) antes de eliminar los detalles innecesarios. Esto permitiría que la base de datos mantenga un tamaño constante y un rendimiento predecible a lo largo del tiempo, emulando la capacidad del cerebro humano para priorizar información relevante.

En conclusión, SurrealDB es una proeza técnica de la cual ConnectomeDB debe aprender, especialmente en su capacidad de unificar modelos y su lenguaje de consulta expresivo. Sin embargo, la oportunidad de ConnectomeDB reside en ser más eficiente (Zero-Copy), más pura (100% Rust) y más inteligente (Olvido y Activación biológica), convirtiéndose en el verdadero sistema operativo para la memoria de la Inteligencia Artificial moderna.

#### **Obras citadas**

1. Introduction | SurrealDB Docs, fecha de acceso: abril 3, 2026, [https://surrealdb.com/docs/surrealdb](https://surrealdb.com/docs/surrealdb)  
2. GitHub \- surrealdb/surrealdb: A scalable, distributed, collaborative, document-graph database, for the realtime web, fecha de acceso: abril 3, 2026, [https://github.com/surrealdb/surrealdb](https://github.com/surrealdb/surrealdb)  
3. Introduction | SurrealDB Docs, fecha de acceso: abril 3, 2026, [https://surrealdb.com/docs/surrealdb/introduction/architecture](https://surrealdb.com/docs/surrealdb/introduction/architecture)  
4. SurrealDB is not a database · Issue \#103 \- GitHub, fecha de acceso: abril 3, 2026, [https://github.com/surrealdb/surrealdb/issues/103](https://github.com/surrealdb/surrealdb/issues/103)  
5. Introducing Surreal  
6. Record IDs | SurrealQL | SurrealDB Docs, fecha de acceso: abril 3, 2026, [https://surrealdb.com/docs/surrealql/datamodel/ids](https://surrealdb.com/docs/surrealql/datamodel/ids)  
7. Beyond SQL Joins: Exploring SurrealDB's Multi-Model Relationships | Blog, fecha de acceso: abril 3, 2026, [https://surrealdb.com/blog/beyond-sql-joins-exploring-surrealdbs-multi-model-relationships](https://surrealdb.com/blog/beyond-sql-joins-exploring-surrealdbs-multi-model-relationships)  
8. CBOR Protocol| Integration \- SurrealDB, fecha de acceso: abril 3, 2026, [https://surrealdb.com/docs/surrealdb/integration/cbor](https://surrealdb.com/docs/surrealdb/integration/cbor)  
9. Understanding CBOR | Blog \- SurrealDB, fecha de acceso: abril 3, 2026, [https://surrealdb.com/blog/understanding-cbor](https://surrealdb.com/blog/understanding-cbor)  
10. surrealdb \- Rust \- Docs.rs, fecha de acceso: abril 3, 2026, [https://docs.rs/surrealdb/](https://docs.rs/surrealdb/)  
11. DEFINE ACCESS ... TYPE RECORD statement | SurrealQL | SurrealDB Docs, fecha de acceso: abril 3, 2026, [https://surrealdb.com/docs/surrealql/statements/define/access/record](https://surrealdb.com/docs/surrealql/statements/define/access/record)  
12. Introducing SurrealDB 3.0 \- the future of AI agent memory | Blog ..., fecha de acceso: abril 3, 2026, [https://surrealdb.com/blog/introducing-surrealdb-3-0--the-future-of-ai-agent-memory](https://surrealdb.com/blog/introducing-surrealdb-3-0--the-future-of-ai-agent-memory)  
13. Using SurrealDB as a Vector Database | Introduction, fecha de acceso: abril 3, 2026, [https://surrealdb.com/docs/surrealdb/models/vector](https://surrealdb.com/docs/surrealdb/models/vector)  
14. SurrealDB vs. Vector Databases | Why Surreal, fecha de acceso: abril 3, 2026, [https://surrealdb.com/why/vs-vector-databases](https://surrealdb.com/why/vs-vector-databases)  
15. Multi-Model Data Engineering with SurrealDB: Combining Graph, Document, and Relational Models in One Engine | by firman brilian | Medium, fecha de acceso: abril 3, 2026, [https://medium.com/@firmanbrilian/multi-model-data-engineering-with-surrealdb-combining-graph-document-and-relational-models-in-e9d4a6f4c235](https://medium.com/@firmanbrilian/multi-model-data-engineering-with-surrealdb-combining-graph-document-and-relational-models-in-e9d4a6f4c235)  
16. HNSW at Scale: Why Your RAG System Gets Worse as the Vector Database Grows, fecha de acceso: abril 3, 2026, [https://towardsdatascience.com/hnsw-at-scale-why-your-rag-system-gets-worse-as-the-vector-database-grows/](https://towardsdatascience.com/hnsw-at-scale-why-your-rag-system-gets-worse-as-the-vector-database-grows/)  
17. jean-pierreBoth/hnswlib-rs: Rust implementation of the HNSW algorithm (Malkov-Yashunin), fecha de acceso: abril 3, 2026, [https://github.com/jean-pierreBoth/hnswlib-rs](https://github.com/jean-pierreBoth/hnswlib-rs)  
18. Vector Search: Navigating Recall and Performance \- OpenSource Connections, fecha de acceso: abril 3, 2026, [https://opensourceconnections.com/blog/2025/02/27/vector-search-navigating-recall-and-performance/](https://opensourceconnections.com/blog/2025/02/27/vector-search-navigating-recall-and-performance/)  
19. SurrealDB 3.0 | SurrealDB, fecha de acceso: abril 3, 2026, [https://surrealdb.com/3.0](https://surrealdb.com/3.0)  
20. Using SurrealDB as a Graph Database | Data Models, fecha de acceso: abril 3, 2026, [https://surrealdb.com/docs/surrealdb/models/graph](https://surrealdb.com/docs/surrealdb/models/graph)  
21. RELATE statement | SurrealQL | SurrealDB Docs, fecha de acceso: abril 3, 2026, [https://surrealdb.com/docs/surrealql/statements/relate](https://surrealdb.com/docs/surrealql/statements/relate)  
22. Knowledge Graphs | Use Cases \- SurrealDB, fecha de acceso: abril 3, 2026, [https://surrealdb.com/use-cases/knowledge-graphs](https://surrealdb.com/use-cases/knowledge-graphs)  
23. Three ways to model data relationships in SurrealDB | Blog, fecha de acceso: abril 3, 2026, [https://surrealdb.com/blog/three-ways-to-model-data-relationships-in-surrealdb](https://surrealdb.com/blog/three-ways-to-model-data-relationships-in-surrealdb)  
24. Vector Query Filters \- Azure AI Search \- Microsoft Learn, fecha de acceso: abril 3, 2026, [https://learn.microsoft.com/en-us/azure/search/vector-search-filters](https://learn.microsoft.com/en-us/azure/search/vector-search-filters)  
25. Hybrid vector \+ text Search in the terminal with SurrealDB and Ratatui | Blog, fecha de acceso: abril 3, 2026, [https://surrealdb.com/blog/hybrid-vector-text-search-in-the-terminal-with-surrealdb-and-ratatui](https://surrealdb.com/blog/hybrid-vector-text-search-in-the-terminal-with-surrealdb-and-ratatui)  
26. Filtered Vector Search: State-of-the-art and Research Opportunities \- VLDB Endowment, fecha de acceso: abril 3, 2026, [https://www.vldb.org/pvldb/vol18/p5488-caminal.pdf](https://www.vldb.org/pvldb/vol18/p5488-caminal.pdf)  
27. Search functions | SurrealQL \- SurrealDB, fecha de acceso: abril 3, 2026, [https://surrealdb.com/docs/surrealql/functions/database/search](https://surrealdb.com/docs/surrealql/functions/database/search)  
28. Bug: Abnormal memory usage (v4) · Issue \#5541 \- GitHub, fecha de acceso: abril 3, 2026, [https://github.com/surrealdb/surrealdb/issues/5541](https://github.com/surrealdb/surrealdb/issues/5541)  
29. SurrealMX: In-memory storage with time travel and persistent storage | Blog \- SurrealDB, fecha de acceso: abril 3, 2026, [https://surrealdb.com/blog/surrealmx-in-memory-storage-with-time-travel-and-persistent-storage](https://surrealdb.com/blog/surrealmx-in-memory-storage-with-time-travel-and-persistent-storage)  
30. Running an in-memory SurrealDB server, fecha de acceso: abril 3, 2026, [https://surrealdb.com/docs/surrealdb/installation/running/memory](https://surrealdb.com/docs/surrealdb/installation/running/memory)  
31. Arrow Interop with Zero-Copy Memory Reads | by Yerachmiel Feltzman | Israeli Tech Radar, fecha de acceso: abril 3, 2026, [https://medium.com/israeli-tech-radar/the-apache-arrow-revolution-for-data-solutions-e59bb496c60c](https://medium.com/israeli-tech-radar/the-apache-arrow-revolution-for-data-solutions-e59bb496c60c)  
32. Apache Arrow Zero-Copy: The One Feature That Replaced Pandas Loops and Lets Me Query Billions of Rows in My DuckDB ELT Stack \- Medium, fecha de acceso: abril 3, 2026, [https://medium.com/@dwickyferi/apache-arrow-zero-copy-the-one-feature-that-replaced-pandas-loops-and-lets-me-query-billions-of-7355b4460596](https://medium.com/@dwickyferi/apache-arrow-zero-copy-the-one-feature-that-replaced-pandas-loops-and-lets-me-query-billions-of-7355b4460596)  
33. Database interfaces — list of Rust libraries/crates // Lib.rs, fecha de acceso: abril 3, 2026, [https://lib.rs/database](https://lib.rs/database)  
34. How the Apache Arrow Format Accelerates Query Result Transfer, fecha de acceso: abril 3, 2026, [https://arrow.apache.org/blog/2025/01/10/arrow-result-transfer/](https://arrow.apache.org/blog/2025/01/10/arrow-result-transfer/)  
35. HNSW Vector Search causes complete query starvation due to ReadLock held across await (Write-Biased Starvation) · Issue \#6819 · surrealdb/surrealdb \- GitHub, fecha de acceso: abril 3, 2026, [https://github.com/surrealdb/surrealdb/issues/6819](https://github.com/surrealdb/surrealdb/issues/6819)  
36. Using SurrealDB as a Time Series Database (TSDB), fecha de acceso: abril 3, 2026, [https://surrealdb.com/docs/surrealdb/models/time-series](https://surrealdb.com/docs/surrealdb/models/time-series)  
37. surrealdb/surrealkv: A low-level, versioned, embedded, ACID-compliant, key-value database for Rust \- GitHub, fecha de acceso: abril 3, 2026, [https://github.com/surrealdb/surrealkv](https://github.com/surrealdb/surrealkv)  
38. SurrealKV \- SurrealQL \- SurrealDB, fecha de acceso: abril 3, 2026, [https://surrealdb.com/docs/surrealdb/installation/running/surrealkv](https://surrealdb.com/docs/surrealdb/installation/running/surrealkv)  
39. LIVE SELECT statement | SurrealQL | SurrealDB Docs, fecha de acceso: abril 3, 2026, [https://surrealdb.com/docs/surrealql/statements/live](https://surrealdb.com/docs/surrealql/statements/live)  
40. Memory usage scales non-linearly during bulk inserts, causing OOM (v3.0.0-alpha.11) · surrealdb · Discussion \#6554 \- GitHub, fecha de acceso: abril 3, 2026, [https://github.com/orgs/surrealdb/discussions/6554](https://github.com/orgs/surrealdb/discussions/6554)  
41. Performance regressions 3.0 / existing performance issues · Issue \#6800 · surrealdb/surrealdb \- GitHub, fecha de acceso: abril 3, 2026, [https://github.com/surrealdb/surrealdb/issues/6800](https://github.com/surrealdb/surrealdb/issues/6800)  
42. When using the ws client, the memory keeps increasing. · Issue \#6822 \- GitHub, fecha de acceso: abril 3, 2026, [https://github.com/surrealdb/surrealdb/issues/6822](https://github.com/surrealdb/surrealdb/issues/6822)  
43. Real-time and event-driven best practices \- SurrealDB, fecha de acceso: abril 3, 2026, [https://surrealdb.com/docs/surrealdb/reference-guide/real-time-best-practices](https://surrealdb.com/docs/surrealdb/reference-guide/real-time-best-practices)  
44. SurrealDB 3.0 : r/rust \- Reddit, fecha de acceso: abril 3, 2026, [https://www.reddit.com/r/rust/comments/1r7phlj/surrealdb\_30/](https://www.reddit.com/r/rust/comments/1r7phlj/surrealdb_30/)  
45. Environment variables used for SurrealDB | SurrealDB Docs, fecha de acceso: abril 3, 2026, [https://surrealdb.com/docs/surrealdb/cli/env](https://surrealdb.com/docs/surrealdb/cli/env)

[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAsAAAAYCAYAAAAs7gcTAAAA4ElEQVR4Xu2SMQ5BQRCGRyHRSAiFQucEKh2dWqFwAVcQhTiAViNKCo2OSuUONCJRiEqi0igU/v/tjOzb7AUkvuTLm8zM7pvMeyK/TRsu4QOeglqUKjzDdViIwdtfcBgWYozgEzbDQkgObsWNwXFIEZZhxpqMGryKO8CDc7iBB9j3+hL8eQuwAcfwrbkUnJfNPTjT3ATeYN2aCGe8wL242+6aixKurAWnGnP+vMYJbPJXxtda8wB2NJasuC/mr4zNvKAEF7Ci+STgv7ASd5DweYQ72NXcF67KGg1+jNSsf3w+FoElrSo4UnUAAAAASUVORK5CYII=>