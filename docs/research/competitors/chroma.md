# **Análisis de Ingeniería Inversa de Chroma: Arquitectura, Mecánica Vectorial y Estrategias para la Construcción de ConnectomeDB**

El diseño de bases de datos cognitivas exige una comprensión profunda de las estructuras de datos que permiten la persistencia de información multidimensional y su recuperación mediante mecanismos de similitud. Chroma se presenta como una infraestructura de datos de código abierto optimizada para aplicaciones de inteligencia artificial, específicamente para sistemas de Generación Aumentada por Recuperación (RAG).1 Este informe técnico desglosa la arquitectura de Chroma desde una perspectiva de ingeniería de sistemas senior, analizando su transición hacia un núcleo basado en Rust y las implicaciones que esto tiene para el desarrollo de ConnectomeDB.

## **Anatomía de la Neurona: Estructura de Datos e Internos de Almacenamiento**

En el ecosistema de Chroma, la "neurona" o unidad de información fundamental se conceptualiza como un registro compuesto que integra cuatro componentes esenciales: un identificador único, un vector de incrustación (embedding), metadatos estructurados y el documento o contenido original.2 La gestión de estos elementos no se realiza de forma monolítica, sino mediante un modelo de almacenamiento híbrido que delega responsabilidades a subsistemas especializados para optimizar tanto la integridad como el rendimiento.

### **Almacenamiento Interno y Modelado de Datos**

Chroma utiliza un enfoque de almacenamiento que combina características de bases de datos documentales y relacionales. Internamente, la información se organiza en una jerarquía lógica compuesta por inquilinos (tenants), bases de datos y colecciones.2 Esta estructura permite el aislamiento multitenant desde el nivel más alto de la arquitectura. Las colecciones actúan como contenedores de almacenamiento independientes, análogos a las tablas en una base de datos relacional, donde cada elemento dentro de la colección posee una firma vectorial única.2

La persistencia en disco de estos componentes se divide entre la gestión de metadatos y la gestión del índice vectorial. El análisis de los internos revela que Chroma emplea SQLite como motor para el almacenamiento de metadatos, identificadores y el texto de los documentos.5 Esta decisión arquitectónica proporciona garantías ACID (Atomicidad, Consistencia, Aislamiento y Durabilidad) para las operaciones sobre datos estructurados, permitiendo que el sistema herede la robustez de un motor relacional maduro para la gestión de estados y esquemas.5

### **Gestión de Metadatos y Serialización**

Los metadatos asociados a un objeto en Chroma se manejan como pares clave-valor. El sistema permite el filtrado por estos campos durante la consulta, lo que implica que deben estar indexados de manera eficiente. En la implementación local, el archivo chroma.sqlite3 centraliza esta información.5 Los metadatos soportan tipos de datos básicos como cadenas, enteros, flotantes y booleanos, además de una capacidad reciente para manejar arreglos de estos tipos, lo que facilita el modelado de etiquetas o categorías múltiples para una sola entrada.7

Para la persistencia de vectores, Chroma utiliza archivos binarios planos que permiten un acceso directo a la memoria. En el directorio de almacenamiento, se identifican archivos específicos que gestionan la estructura del índice HNSW. La serialización de estos vectores no utiliza formatos pesados como JSON, sino representaciones binarias directas que minimizan la latencia de lectura y escritura.5

| Archivo de Almacenamiento | Propósito Técnico | Mecanismo de Datos |
| :---- | :---- | :---- |
| chroma.sqlite3 | Gestión de metadatos, IDs y texto original | Motor Relacional (SQLite) con soporte ACID.5 |
| data\_level0.bin | Almacenamiento de vectores de incrustación | Acceso secuencial binario para vectores float32.5 |
| link\_lists.bin | Estructura de conexiones del grafo HNSW | Listas de adyacencia binarias para navegación rápida.5 |
| header.bin | Metadatos del índice y configuración | Cabecera de 100 bytes con dimensiones y tipos.5 |

## **Lógica de Recuperación y Búsqueda: Mecanismos de Similitud**

La recuperación de información en Chroma se sustenta en la búsqueda de vecinos más cercanos aproximados (ANN), un requisito crítico para manejar la dimensionalidad de las incrustaciones modernas. El sistema implementa algoritmos que balancean la precisión de los resultados con la latencia de la consulta, permitiendo búsquedas en milisegundos incluso sobre grandes volúmenes de datos.3

### **Indexación Vectorial mediante HNSW**

El algoritmo predominante en Chroma para la búsqueda vectorial es Hierarchical Navigable Small World (HNSW).9 HNSW construye una estructura de grafo multicapa donde cada capa actúa como un filtro de resolución. La búsqueda comienza en las capas superiores, que son más dispersas y permiten saltos de largo alcance a través del espacio vectorial, y desciende progresivamente hacia las capas inferiores más densas hasta localizar la vecindad exacta del vector de consulta.5

El compromiso entre Recall (exhaustividad) y Latencia se gestiona a través de parámetros configurables por el usuario al crear una colección. Estos parámetros definen la conectividad del grafo y la profundidad de la búsqueda. Un aumento en la conectividad mejora la precisión pero incrementa el consumo de memoria y el tiempo de construcción del índice.11

| Parámetro HNSW | Descripción Técnica | Impacto en el Rendimiento |
| :---- | :---- | :---- |
| M (max\_neighbors) | Número máximo de conexiones por nodo | Mayor valor aumenta el recall y el uso de RAM.12 |
| ef\_construction | Tamaño de la lista de candidatos en indexación | Determina la calidad inicial del grafo construido.10 |
| ef\_search | Tamaño de la lista de candidatos en consulta | Aumentar este valor mejora el recall pero sube la latencia.12 |

### **Filtrado Híbrido y Búsqueda Semántica-Léxica**

Chroma implementa un modelo de búsqueda híbrida que combina la similitud semántica (vectores densos) con la coincidencia de palabras clave (vectores dispersos).14 Esta lógica es fundamental para superar las limitaciones de la búsqueda puramente vectorial, que a veces falla en identificar términos técnicos exactos o identificadores únicos como números de pieza o citas legales.14

El filtrado híbrido se realiza típicamente mediante un proceso de pre-filtrado de metadatos. El motor utiliza los índices de SQLite para reducir el conjunto de candidatos antes de ejecutar la búsqueda vectorial en el grafo HNSW.16 Sin embargo, si la selectividad del filtro es muy alta (es decir, el filtro elimina la gran mayoría de los datos), esto puede llevar al problema de la fragmentación del grafo, donde los nodos restantes quedan desconectados, impidiendo que el algoritmo HNSW navegue correctamente.16

Para integrar los resultados de las búsquedas densas y dispersas, Chroma utiliza Reciprocal Rank Fusion (RRF). Este algoritmo asigna una puntuación a cada documento basada en su clasificación en múltiples listas de resultados, permitiendo una combinación equilibrada de relevancia semántica y precisión léxica.19

## **Gestión de Memoria y Estado: Optimizaciones para Carga Masiva**

La eficiencia en la gestión de la memoria es quizás el aspecto más crítico de cualquier base de datos que maneje vectores de alta dimensión. Chroma ha evolucionado hacia un núcleo en Rust para abordar los cuellos de botella de rendimiento asociados con Python y el manejo ineficiente de hilos.22

### **Caching y Arquitecturas Zero-Copy**

En su arquitectura de caching, Chroma se ha apoyado en bibliotecas como foyer, un sistema de caché híbrido escrito en Rust.23 Foyer gestiona automáticamente el movimiento de datos entre la memoria RAM y el almacenamiento en disco, optimizando el rendimiento mediante el uso de estructuras de datos intrusivas que minimizan la sobrecarga de gestión.23

Una característica destacada de foyer es su abstracción de caché en memoria de tipo "Zero-Copy".23 Al aprovechar el sistema de tipos de Rust, los datos pueden ser accedidos directamente desde los buffers de la caché sin necesidad de serialización o copia intermedia, lo que reduce drásticamente el uso de CPU y la latencia en aplicaciones de alto rendimiento. Aunque Chroma no utiliza Apache Arrow como formato de intercambio primario en todas sus capas, la integración de componentes de Rust que favorecen el acceso directo a memoria apunta a una filosofía arquitectónica similar.24

### **Concurrencia y Mecanismos de Bloqueo**

La gestión de la concurrencia en escrituras masivas ha sido históricamente un punto de fricción en la versión basada en Python de Chroma. En el modo de persistencia local, los bloqueos a nivel de archivo de SQLite impiden que múltiples hilos realicen escrituras simultáneas de manera eficiente.27 Sin embargo, la migración a un núcleo de Rust permite implementar un modelo de concurrencia basado en el paso de mensajes y bloqueos de granularidad fina, eliminando las limitaciones del Global Interpreter Lock (GIL).22

Para la gestión de la durabilidad y la consistencia en entornos distribuidos, Chroma ha desarrollado WAL3 (Write-Ahead Log versión 3).28 Este componente implementa un registro linealizable sobre almacenamiento de objetos (como S3), utilizando cabeceras de condición If-Match para garantizar la atomicidad sin necesidad de sistemas de bloqueo externos complejos.28

| Componente | Tecnología | Función en ConnectomeDB |
| :---- | :---- | :---- |
| Caché | foyer (Rust) | Gestión híbrida RAM/Disco con Zero-Copy.23 |
| Log de Escritura | WAL3 (Rust) | Durabilidad en almacenamiento de objetos con setsum.28 |
| Concurrencia | Multithreading nativo | Paralelismo de consultas y actualizaciones sin GIL.22 |

### **Mecanismos de Olvido y Compactación**

Chroma no implementa un mecanismo de "olvido" biológico per se, pero utiliza sistemas de recolección de basura y compactación para gestionar el crecimiento del almacenamiento. El sistema WAL3 incluye capacidades para recomputar sumas de verificación del log (setsum) en tiempo ![][image1] durante las operaciones de recolección de basura, lo que permite eliminar registros antiguos sin comprometer la integridad de la prueba de durabilidad del sistema completo.29 En el nivel de SQLite, la base de datos de metadatos requiere mantenimiento para evitar el crecimiento excesivo, un problema reportado comúnmente donde la base de datos puede aumentar su tamaño significativamente incluso con actualizaciones mínimas.30

## **Análisis de la Documentación y API: Interfaz de Desarrollo**

La popularidad de Chroma radica en gran medida en su enfoque "developer-first", priorizando una API intuitiva sobre configuraciones complejas de infraestructura. Este análisis revisa las capacidades de su SDK y la estructura de su lenguaje de consulta.

### **Lenguaje de Consulta y Flexibilidad**

Chroma utiliza un lenguaje de consulta basado en diccionarios JSON, similar a la sintaxis de MongoDB, lo que facilita su adopción por parte de desarrolladores web.8 Las consultas permiten combinar filtros de metadatos, filtros de contenido de documentos y búsqueda por similitud vectorial en una sola llamada.32

JSON

{  
  "where": {  
    "$and": \[  
      {"category": {"$eq": "science"}},  
      {"year": {"$gte": 2022}}  
    \]  
  },  
  "where\_document": {"$contains": "neurona"}  
}

Esta sintaxis ofrece una gran flexibilidad para el filtrado de metadatos, soportando operadores lógicos ($and, $or) y operadores de comparación ($gt, $lt, $in).7 Sin embargo, la flexibilidad se ve limitada en operaciones de grafos; el SDK no soporta nativamente traversals o consultas de adyacencia directa entre registros, lo que representa una oportunidad de diferenciación para ConnectomeDB.

### **Innovaciones en el SDK**

Una de las funciones más potentes de Chroma es su capacidad para manejar la generación de incrustaciones de forma transparente. Si un usuario añade documentos de texto sin proporcionar vectores, Chroma utiliza funciones de incrustación predeterminadas (como Sentence Transformers) para automatizar el proceso.6 Esta abstracción permite que los desarrolladores pasen de "texto crudo" a "búsqueda semántica" en pocas líneas de código.

Además, la introducción de Chroma Cloud y el soporte para búsqueda híbrida con RRF marcan un avance hacia capacidades de nivel empresarial que antes estaban reservadas para sistemas más complejos como Pinecone o Milvus.15

### **Errores Comunes en Escala**

A pesar de sus fortalezas, la comunidad de desarrolladores ha reportado varios puntos de dolor al operar Chroma a gran escala:

* **Consumo de Memoria**: Debido a que HNSW reside en RAM, el escalado a millones de vectores requiere instancias con capacidades de memoria masivas.36  
* **Bloqueos de SQLite**: En entornos de alta concurrencia, los errores de Database is locked son frecuentes cuando se utilizan persistencias locales bajo carga de escritura pesada.27  
* **Crecimiento de Almacenamiento**: Se han documentado casos donde la base de datos de metadatos crece desproporcionadamente en comparación con la cantidad de datos insertados, afectando la latencia de recuperación.30

## **Inspiración para ConnectomeDB: Funcionalidades Críticas**

Para que ConnectomeDB se posicione como una base de datos cognitiva superior, debe adoptar y mejorar las mejores prácticas de Chroma, adaptándolas a una arquitectura inspirada en la neurobiología y escrita íntegramente en Rust.

### **1\. Implementación de WAL3 con Verificación de Integridad Continua**

La lógica de WAL3 de Chroma es una obra maestra de ingeniería para sistemas que dependen de almacenamiento de objetos.28 ConnectomeDB debería implementar un log de escritura similar que utilice setsum (una suma de verificación asociativa y conmutativa). Esto permitiría que cada "neurona" añadida al sistema genere una prueba criptográfica de que el estado global es correcto.29

En el contexto de una arquitectura cognitiva, esto se traduce en una "memoria duradera" que puede ser auditada en tiempo real. La implementación en Rust debe utilizar operaciones atómicas y evitar el uso de mmap para garantizar la seguridad frente a fallos de alimentación o errores de escritura, aprendiendo de las críticas a bibliotecas experimentales de WAL en el ecosistema de Rust.37

### **2\. Recuperación Híbrida Semántica-Simbólica-Grafo**

Chroma ha validado que la búsqueda puramente vectorial es insuficiente para aplicaciones reales y ha recurrido a la búsqueda híbrida con RRF.14 ConnectomeDB debe elevar este concepto integrando la lógica LISP para consultas simbólicas y la adyacencia de grafos para la navegación asociativa.

La adaptación consistiría en utilizar el grafo HNSW no solo para la búsqueda de vecinos más cercanos, sino como una estructura de adyacencia que permita saltos entre conceptos relacionados (traversals). Mientras que Chroma se detiene en la vecindad vectorial, ConnectomeDB podría navegar por las conexiones del grafo para descubrir relaciones de segundo y tercer orden, emulando la propagación de señales en un conectoma biológico.5

### **3\. Caching Inteligente con Sharding y Zero-Copy**

La integración de la biblioteca foyer en el ecosistema de Rust demuestra cómo gestionar cachés de alto rendimiento.23 ConnectomeDB debe implementar una capa de caché fragmentada (sharded) para reducir la contención de bloqueos durante el acceso concurrente de múltiples agentes cognitivos.

El uso de estructuras de datos intrusivas en Rust, como las empleadas por foyer, permitiría que las "neuronas" residan en memoria de forma que el motor de lógica LISP pueda operar sobre ellas sin realizar copias adicionales.23 Esto es vital para sistemas cognitivos que requieren procesos de razonamiento recursivo, donde el costo de la copia de datos podría asfixiar el rendimiento del sistema.

## **Puntos Débiles de Chroma: La Ventaja Competitiva de ConnectomeDB**

Chroma presenta vulnerabilidades estratégicas que representan una oportunidad clara para ConnectomeDB.

### **El "Muro de RAM" de HNSW**

La dependencia de que el índice resida completamente en RAM es el talón de Aquiles de Chroma.36 Para aplicaciones que requieren memorias a largo plazo de escala masiva, esto es prohibitivo desde el punto de vista del costo.

**Superioridad de ConnectomeDB**: ConnectomeDB puede superar esto implementando algoritmos de búsqueda vectorial optimizados para disco, como DiskANN o SPANN.12 Al gestionar inteligentemente la jerarquía de memoria (NVMe \-\> RAM \-\> Caché L3), ConnectomeDB podría ofrecer una escala de miles de millones de neuronas con una fracción del presupuesto de RAM de Chroma, manteniendo latencias competitivas gracias al uso de io\_uring en Rust para E/S asíncrona.22

### **Dependencia de Motores Externos (SQLite)**

Aunque SQLite es fiable, no está optimizado para los patrones de acceso de una base de datos cognitiva masiva. La sobrecarga de traducción de esquemas y los bloqueos de concurrencia limitan la fluidez del aprendizaje del sistema.27

**Superioridad de ConnectomeDB**: Al construir un motor de metadatos nativo en Rust que utilice un formato columnar como Apache Arrow internamente, ConnectomeDB puede ofrecer filtrado de alta velocidad y concurrencia de escritura real.22 La eliminación de la capa de SQLite reduciría el bloat de almacenamiento y permitiría un control total sobre las políticas de compactación y "olvido" de datos.

### **Limitaciones en la Navegación de Relaciones**

Chroma es, en esencia, un motor de búsqueda de vecinos planos con filtros adjuntos. No entiende la estructura de red del conocimiento que está almacenando.2

**Superioridad de ConnectomeDB**: Al integrar adyacencia libre de índices (Index-free Adjacency) y una interfaz LISP, ConnectomeDB permite realizar consultas relacionales profundas que son imposibles en Chroma. El sistema no solo encontraría "vectores similares", sino que podría responder a consultas complejas sobre la topología del conectoma, permitiendo que la IA navegue por el conocimiento de forma asociativa, no solo estadística.29

En resumen, Chroma ofrece una lección valiosa sobre la importancia de la experiencia del desarrollador y la simplicidad de la API. Sin embargo, su arquitectura subyacente presenta limitaciones de escalabilidad y flexibilidad que ConnectomeDB, mediante una ingeniería rigurosa en Rust y una visión inspirada en la neurobiología, está preparada para superar, proporcionando una infraestructura de datos verdaderamente cognitiva para la próxima generación de agentes inteligentes.

#### **Fuentes citadas**

1. How to Create ChromaDB Integration \- OneUptime, acceso: abril 3, 2026, [https://oneuptime.com/blog/post/2026-01-30-chromadb-integration/view](https://oneuptime.com/blog/post/2026-01-30-chromadb-integration/view)  
2. Architecture Overview \- Chroma Docs, acceso: abril 3, 2026, [https://docs.trychroma.com/reference/architecture/overview](https://docs.trychroma.com/reference/architecture/overview)  
3. Introduction to ChromaDB \- GeeksforGeeks, acceso: abril 3, 2026, [https://www.geeksforgeeks.org/nlp/introduction-to-chromadb/](https://www.geeksforgeeks.org/nlp/introduction-to-chromadb/)  
4. A Comprehensive Beginner's Guide to ChromaDB | by Syeedmdtalha \- Medium, acceso: abril 3, 2026, [https://medium.com/@syeedmdtalha/a-comprehensive-beginners-guide-to-chromadb-eb2fa22ee22f](https://medium.com/@syeedmdtalha/a-comprehensive-beginners-guide-to-chromadb-eb2fa22ee22f)  
5. Learning Vector Databases: A Practical Deep Dive with ChromaDB \- Hamman Samuel, PhD, acceso: abril 3, 2026, [https://hammansamuel.medium.com/learning-vector-databases-a-practical-deep-dive-with-chromadb-71884bbe2d99](https://hammansamuel.medium.com/learning-vector-databases-a-practical-deep-dive-with-chromadb-71884bbe2d99)  
6. Learn How to Use Chroma DB: A Step-by-Step Guide | DataCamp, acceso: abril 3, 2026, [https://www.datacamp.com/tutorial/chromadb-tutorial-step-by-step-guide](https://www.datacamp.com/tutorial/chromadb-tutorial-step-by-step-guide)  
7. Filtering with Where \- Chroma Docs, acceso: abril 3, 2026, [https://docs.trychroma.com/cloud/search-api/filtering](https://docs.trychroma.com/cloud/search-api/filtering)  
8. Metadata Filtering \- Chroma Docs, acceso: abril 3, 2026, [https://docs.trychroma.com/docs/querying-collections/metadata-filtering](https://docs.trychroma.com/docs/querying-collections/metadata-filtering)  
9. Vector Stores and ChromaDB: The Complete Guide to Building AI Memory | by Suyeshrimal | Jan, 2026 | Medium, acceso: abril 3, 2026, [https://medium.com/@suyeshrimal/vector-stores-and-chromadb-the-complete-guide-to-building-ai-memory-ba33e07d1a72](https://medium.com/@suyeshrimal/vector-stores-and-chromadb-the-complete-guide-to-building-ai-memory-ba33e07d1a72)  
10. ChromaDBQueryEngine \- AG2 Documentation, acceso: abril 3, 2026, [https://docs.ag2.ai/latest/docs/api-reference/autogen/agentchat/contrib/rag/ChromaDBQueryEngine/](https://docs.ag2.ai/latest/docs/api-reference/autogen/agentchat/contrib/rag/ChromaDBQueryEngine/)  
11. ChromaDB \- by Nishtha kukreti \- Medium, acceso: abril 3, 2026, [https://medium.com/@nishthakukreti.01/chromadb-fb20279e244c](https://medium.com/@nishthakukreti.01/chromadb-fb20279e244c)  
12. Configure Collections \- Chroma Docs, acceso: abril 3, 2026, [https://docs.trychroma.com/docs/collections/configure](https://docs.trychroma.com/docs/collections/configure)  
13. ChromaDB \- Voxta Documentation, acceso: abril 3, 2026, [https://doc.voxta.ai/docs/chromadb/](https://doc.voxta.ai/docs/chromadb/)  
14. Look at Your Data \- Chroma Docs, acceso: abril 3, 2026, [https://docs.trychroma.com/guides/build/look-at-your-data](https://docs.trychroma.com/guides/build/look-at-your-data)  
15. Sparse vector support is here\! \- Chroma, acceso: abril 3, 2026, [https://www.trychroma.com/project/sparse-vector-search](https://www.trychroma.com/project/sparse-vector-search)  
16. Metadata filtering in Vector databases | by Kandaanusha | Mar, 2026 | Medium, acceso: abril 3, 2026, [https://medium.com/@kandaanusha/metadata-filtering-in-vector-databases-e3ebe61c8f76](https://medium.com/@kandaanusha/metadata-filtering-in-vector-databases-e3ebe61c8f76)  
17. Metadata Filtering and Hybrid Search for Vector Databases \- Dataquest, acceso: abril 3, 2026, [https://www.dataquest.io/blog/metadata-filtering-and-hybrid-search-for-vector-databases/](https://www.dataquest.io/blog/metadata-filtering-and-hybrid-search-for-vector-databases/)  
18. A Complete Guide to Filtering in Vector Search \- Qdrant, acceso: abril 3, 2026, [https://qdrant.tech/articles/vector-search-filtering/](https://qdrant.tech/articles/vector-search-filtering/)  
19. Hybrid Search with RRF \- Chroma Docs, acceso: abril 3, 2026, [https://docs.trychroma.com/cloud/search-api/hybrid-search](https://docs.trychroma.com/cloud/search-api/hybrid-search)  
20. Chroma Hybrid Search \- Agno, acceso: abril 3, 2026, [https://docs.agno.com/knowledge/vector-stores/chroma/usage/chroma-hybrid-search](https://docs.agno.com/knowledge/vector-stores/chroma/usage/chroma-hybrid-search)  
21. The Good and Bad of ChromaDB for RAG: Based on Our Experience \- AltexSoft, acceso: abril 3, 2026, [https://www.altexsoft.com/blog/chroma-pros-and-cons/](https://www.altexsoft.com/blog/chroma-pros-and-cons/)  
22. Chroma DB Vs Qdrant \- Key Differences \- Airbyte, acceso: abril 3, 2026, [https://airbyte.com/data-engineering-resources/chroma-db-vs-qdrant](https://airbyte.com/data-engineering-resources/chroma-db-vs-qdrant)  
23. Architecture | foyer \- GitHub Pages, acceso: abril 3, 2026, [https://foyer-rs.github.io/foyer/docs/design/architecture](https://foyer-rs.github.io/foyer/docs/design/architecture)  
24. foyer-rs/foyer: Hybrid in-memory and disk cache in Rust \- GitHub, acceso: abril 3, 2026, [https://github.com/foyer-rs/foyer](https://github.com/foyer-rs/foyer)  
25. Foyer: A Hybrid Cache in Rust — Past, Present, and Future ..., acceso: abril 3, 2026, [https://blog.mrcroxx.com/posts/foyer-a-hybrid-cache-in-rust-past-present-and-future/](https://blog.mrcroxx.com/posts/foyer-a-hybrid-cache-in-rust-past-present-and-future/)  
26. "Zero-Copy In-Memory Cache Abstraction: Leveraging Rust's robust type system, th... | Hacker News, acceso: abril 3, 2026, [https://news.ycombinator.com/item?id=45401337](https://news.ycombinator.com/item?id=45401337)  
27. Resolving Concurrency Bottlenecks in LangChain's RunnableParallel with ChromaDB PersistentClient \- Stack Overflow, acceso: abril 3, 2026, [https://stackoverflow.com/questions/79903575/resolving-concurrency-bottlenecks-in-langchains-runnableparallel-with-chromadb](https://stackoverflow.com/questions/79903575/resolving-concurrency-bottlenecks-in-langchains-runnableparallel-with-chromadb)  
28. chroma/rust/wal3/README.md at main · chroma-core/chroma · GitHub, acceso: abril 3, 2026, [https://github.com/chroma-core/chroma/blob/main/rust/wal3/README.md](https://github.com/chroma-core/chroma/blob/main/rust/wal3/README.md)  
29. wal3: A Write-Ahead Log for Chroma, Build on Object Storage, acceso: abril 3, 2026, [https://www.trychroma.com/engineering/wal3](https://www.trychroma.com/engineering/wal3)  
30. \[Bug\]: Upserting the same data causes the SQLite db to grow by 50-100% \#2143 \- GitHub, acceso: abril 3, 2026, [https://github.com/chroma-core/chroma/issues/2143](https://github.com/chroma-core/chroma/issues/2143)  
31. Metadata-Based Filtering in RAG Systems | CodeSignal Learn, acceso: abril 3, 2026, [https://codesignal.com/learn/courses/scaling-up-rag-with-vector-databases-in-rust/lessons/metadata-based-filtering-in-rag-systems-with-rust-and-qdrant](https://codesignal.com/learn/courses/scaling-up-rag-with-vector-databases-in-rust/lessons/metadata-based-filtering-in-rag-systems-with-rust-and-qdrant)  
32. ChromaDB · Actions · GitHub Marketplace, acceso: abril 3, 2026, [https://github.com/marketplace/actions/chromadb](https://github.com/marketplace/actions/chromadb)  
33. Query and Get \- Chroma Docs, acceso: abril 3, 2026, [https://docs.trychroma.com/docs/querying-collections/query-and-get](https://docs.trychroma.com/docs/querying-collections/query-and-get)  
34. docs/docs/usage-guide.md at main · chroma-core/docs \- GitHub, acceso: abril 3, 2026, [https://github.com/chroma-core/docs/blob/main/docs/usage-guide.md](https://github.com/chroma-core/docs/blob/main/docs/usage-guide.md)  
35. Chroma Docs: Introduction, acceso: abril 3, 2026, [https://docs.trychroma.com/docs/overview/introduction](https://docs.trychroma.com/docs/overview/introduction)  
36. Single-Node Performance \- Chroma Docs, acceso: abril 3, 2026, [https://docs.trychroma.com/guides/performance/single-node](https://docs.trychroma.com/guides/performance/single-node)  
37. Walrus: A 1 Million ops/sec, 1 GB/s Write Ahead Log in Rust \- Reddit, acceso: abril 3, 2026, [https://www.reddit.com/r/rust/comments/1o0hbtz/walrus\_a\_1\_million\_opssec\_1\_gbs\_write\_ahead\_log/](https://www.reddit.com/r/rust/comments/1o0hbtz/walrus_a_1_million_opssec_1_gbs_write_ahead_log/)  
38. Knowledge Graph-based Retrieval-Augmented Generation for Schema Matching \- arXiv, acceso: abril 3, 2026, [https://arxiv.org/html/2501.08686v1](https://arxiv.org/html/2501.08686v1)  
39. Apache Arrow | Apache Arrow, acceso: abril 3, 2026, [https://arrow.apache.org/](https://arrow.apache.org/)  
40. Apache Arrow Zero-Copy: The One Feature That Replaced Pandas Loops and Lets Me Query Billions of Rows in My DuckDB ELT Stack | by Dwicky Feri, acceso: abril 3, 2026, [https://dwickyferi.medium.com/apache-arrow-zero-copy-the-one-feature-that-replaced-pandas-loops-and-lets-me-query-billions-of-7355b4460596](https://dwickyferi.medium.com/apache-arrow-zero-copy-the-one-feature-that-replaced-pandas-loops-and-lets-me-query-billions-of-7355b4460596)

[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACgAAAAYCAYAAACIhL/AAAACUElEQVR4Xu2WPWgVQRSFj4igaPwHk2CRaKc2KkGxsEihRYixCClS2FloIdj4U1naKigEgvCwUBTRRlCUBNKEIEEQq0CiIiQIYqEiWAh6DncHZ29m523is3sffLy8ubOz992ZvRugTZsVsYaeoud9IMNheoNu8oEqttBbdImepTthNx6n3+mF4rvnKH1Fe32goJveoxd9gKyDrT9W/F3JEbpAn9PtLqYLG/QHPV0OYQedgSUfc4g+oO9h1/2mV0oz/nIAdu8TPhDzib6kW32g4DisitMozxmFLb4nGhO7aT9sF64in6B25TZ95gOBHvqF9rnxGG3TPP1MD0bjT+l95LdHieUSFNoZrb0MLXwH9gtS5ysQEvxFB6Jxnddz0fcUdRLcSz/6QXGMfi0+c6hq+oU/UT4rulbbn6NOgpvpFBJFug67eJsPOIZh81SxfdG43/IUdRLUTj6iHfHgRtiDoYtz6Fc1YPMmUV5E267tz1EnQXGXdsYDupFu2CzBcAO1E7WVmFYnWForlLVZgouwNqRe6XkHayk5VpKgzmIJ9TE9mRt8oEBN+xvsDKZo1p5EnQTDbq73gfCquYzlT9B+Oot05QJq3kN+MEJrXoMlqE9/j4DO3pwfDOhlrfYxAXsHyxewJ/RSNC/FG1gn8Kiqqq4S86YqriLoGFWi19dJOlKoVrK2NCONGrx/sleDtv+tH2wFqsYHNG/0OVQcveNzZ3TV6EzdpA+Rfx/nOENf010+0Eq66BPkHyiPKveYDvrA/0IVrPp3LYVayr+e3TaV/AHmpnfzu/gzAQAAAABJRU5ErkJggg==>