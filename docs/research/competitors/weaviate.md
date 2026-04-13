# **Arquitectura de Almacenamiento y Recuperación en Weaviate: Un Análisis de Ingeniería de Sistemas para el Desarrollo de VantaDB**

La construcción de una base de datos cognitiva como VantaDB exige una comprensión profunda de los paradigmas actuales en el procesamiento de información multidimensional. Weaviate se ha consolidado como un referente en el sector de las bases de datos vectoriales debido a su enfoque en la escalabilidad y la integración de modelos de aprendizaje automático. Sin embargo, para un Ingeniero de Sistemas Senior, la superficie comercial de Weaviate es solo el envoltorio de una serie de decisiones arquitectónicas complejas que involucran la gestión de memoria en entornos con recolector de basura, la persistencia mediante estructuras log-structured y la búsqueda en grafos de alta dimensionalidad. El siguiente análisis desglosa la infraestructura interna de Weaviate, analizando sus componentes críticos y evaluando cómo sus fortalezas y debilidades pueden informar el diseño de VantaDB en Rust, un lenguaje que ofrece garantías de seguridad y rendimiento que Weaviate, basado en Go, lucha por emular en escenarios de carga extrema.

## **Anatomía de la "Neurona" (Estructura de Datos Interna)**

En el diseño de VantaDB, el concepto de "neurona" se traduce técnicamente en la unidad mínima de información y su conectividad. En Weaviate, esta unidad se denomina objeto de datos, el cual reside dentro de una jerarquía organizativa que comienza en la clase o colección.1 Cada clase en el esquema definido por el usuario da lugar a la creación de un índice interno independiente.2 Este índice no es una estructura monolítica, sino un contenedor para múltiples shards, que son las unidades lógicas de almacenamiento capaces de distribuir la carga de trabajo entre diferentes recursos de cómputo.2

### **Almacenamiento Interno: El Paradigma LSM-Tree**

Weaviate ha evolucionado su motor de almacenamiento hacia una arquitectura basada en Log-Structured Merge-Tree (LSM-Tree) para el almacenamiento de objetos y el índice invertido.2 Antes de la versión 1.5.0, el sistema utilizaba mecanismos de B+Tree, pero la transición a LSM-Tree fue motivada por la necesidad de manejar ingestas masivas de datos con una latencia de escritura constante.2

En esta arquitectura, cada shard se compone de tres pilares fundamentales: un almacén de objetos (key-value store), un índice invertido y un almacén de índice vectorial.2 El almacén de objetos utiliza el enfoque LSM, lo que significa que las operaciones de escritura se registran primero en una estructura en memoria denominada Memtable.2 Una vez que esta Memtable alcanza un umbral configurado, Weaviate vuelca su contenido en un segmento de disco ordenado.2 Esta estrategia permite que las escrituras ocurran a la velocidad de la memoria, delegando la organización física del disco a procesos de compactación en segundo plano.2 Para optimizar las lecturas, Weaviate implementa filtros de Bloom, que permiten determinar con rapidez si un objeto específico *no* se encuentra en un segmento determinado, evitando así accesos innecesarios a disco.2

| Componente de Shard | Tipo de Estructura | Mecanismo de Acceso | Persistencia |
| :---- | :---- | :---- | :---- |
| **Object Store** | Key-Value (LSM-Tree) | UUID / Búsqueda Lineal | Segmentos SSTable 2 |
| **Inverted Index** | Mapa de Términos (LSM) | Filtrado por Propiedad | Segmentos SSTable 2 |
| **Vector Index** | Grafo Proximidad (HNSW) | Búsqueda ANN (Vecinos Cercanos) | Commit Log / Snapshots 2 |

### **Gestión de Metadatos y Formatos de Representación**

Los metadatos asociados a un objeto en Weaviate no se almacenan de forma aislada, sino que forman parte integral de la representación del objeto como un documento JSON.1 Cada objeto posee un UUID (Identificador Único Universal) que actúa como la clave primaria en el almacén de objetos y garantiza la unicidad en todas las colecciones.1 El sistema soporta UUIDs deterministas, lo cual es fundamental para flujos de trabajo de actualización donde se desea mantener la identidad del objeto sin generar duplicados innecesarios.1

En términos de serialización, aunque Weaviate presenta una interfaz externa basada en JSON y GraphQL, la persistencia interna y la comunicación de alta velocidad dependen de formatos binarios.3 Con la introducción de gRPC en la versión 1.19.0, el sistema comenzó a utilizar Protocol Buffers (Protobuf) para la transferencia de datos entre el cliente y el servidor, lo que reduce significativamente el overhead de serialización y el tamaño de los paquetes de red.3 Los archivos .proto definen los servicios de búsqueda y operaciones por lotes, permitiendo una comunicación contract-based que es notablemente más eficiente que el procesamiento de cadenas JSON a gran escala.3

## **Lógica de Recuperación y Búsqueda**

La eficiencia de una base de datos cognitiva depende de su capacidad para recuperar información basada tanto en la semántica (vectores) como en las relaciones estructurales (grafos). Weaviate aborda este desafío mediante una implementación personalizada de algoritmos de vanguardia que buscan equilibrar la precisión con la latencia.

### **Indexación Vectorial: HNSW y Estrategias de Cuantización**

El núcleo del motor de búsqueda de Weaviate es una implementación personalizada del algoritmo Hierarchical Navigable Small World (HNSW).4 A diferencia de otros motores que utilizan librerías genéricas, Weaviate ha optimizado HNSW para soportar operaciones CRUD completas en tiempo real, permitiendo actualizaciones y eliminaciones inmediatas en el grafo de proximidad.4

El algoritmo HNSW organiza los vectores en múltiples capas de grafos.5 La capa superior contiene una representación muy dispersa del conjunto de datos, actuando como una red de "autopistas" para saltar rápidamente entre regiones del espacio vectorial.5 A medida que la búsqueda desciende por las capas, el grafo se vuelve más denso, permitiendo una navegación de grano fino hasta encontrar los vecinos más cercanos en la capa base (capa 0), que contiene todos los vectores.5

El compromiso entre recall (exhaustividad) y latencia se controla mediante parámetros específicos que VantaDB debe considerar cuidadosamente:

1. **maxConnections (M):** Define el número máximo de aristas que cada nodo puede tener en el grafo. Un ![][image1] mayor aumenta la precisión al proporcionar más rutas de navegación, pero incrementa linealmente el consumo de memoria, ya que cada conexión requiere entre 8 y 10 bytes de RAM.8  
2. **efConstruction:** Determina cuántos vecinos se exploran durante la fase de inserción para construir el grafo. Un valor alto produce un índice de mayor calidad pero ralentiza significativamente la ingesta.9  
3. **ef (Search):** El tamaño de la lista dinámica durante la consulta. Weaviate permite el uso de un ef dinámico que se ajusta automáticamente basándose en el límite de resultados solicitado, optimizando la velocidad para consultas pequeñas y la precisión para recuperaciones extensas.5

Para manejar el crecimiento explosivo de la memoria, Weaviate implementa técnicas de cuantización de vectores que transforman representaciones de punto flotante de 32 bits en formatos más compactos.11

| Técnica de Cuantización | Reducción de Tamaño | Impacto en Precisión | Características Técnicas |
| :---- | :---- | :---- | :---- |
| **Binary Quantization (BQ)** | 32x | Moderado/Alto | Convierte dimensiones en bits (1 o 0). Ideal para vectores de alta dimensionalidad.11 |
| **Product Quantization (PQ)** | \~24x | Variable | Divide el vector en segmentos y los cuantiza mediante un codebook entrenado.11 |
| **Scalar Quantization (SQ)** | 4x | Bajo | Transforma floats de 32 bits en enteros de 8 bits mediante buckets.11 |
| **Rotational Quantization (RQ)** | Variable | Muy Bajo | Aplica rotaciones aleatorias para distribuir la información uniformemente antes de cuantizar.11 |

### **Lógica de Grafos y Relaciones entre Nodos**

A pesar de que Weaviate se autodenomina base de datos de grafos en algunos contextos, su implementación técnica de las relaciones difiere de las bases de datos de grafos nativas que utilizan adyacencia sin índices (index-free adjacency).16 En Weaviate, las relaciones se denominan "cross-references" y se almacenan como punteros lógicos basados en la clase y el UUID del objeto de destino.17

Cuando Weaviate resuelve un salto entre nodos (traversal), realiza lo que internamente se parece a un "join" de clave-valor. El sistema recupera el UUID de la referencia y luego busca ese ID en el shard correspondiente de la clase destino.17 Esto implica que las consultas de grafos profundas (multi-hop) pueden experimentar una degradación de rendimiento, ya que cada salto requiere una resolución de ID independiente.18 Sin embargo, Weaviate mitiga esto mediante el módulo **Ref2Vec-Centroid**, que permite que el vector de un objeto sea calculado dinámicamente como el centroide de los vectores de sus objetos referenciados.19 Esta es una característica de "inspiración cognitiva" crítica, ya que permite que la representación de un concepto evolucione según sus relaciones.

### **Filtrado Híbrido: La Intersección de Semántica y Lógica**

Una de las capacidades más potentes de Weaviate es el filtrado híbrido, que permite combinar la búsqueda vectorial con condiciones lógicas sobre los metadatos (ej: "buscar vectores similares a 'cerebro' pero solo en documentos publicados después de 2023").4

Este proceso se realiza en dos etapas:

1. **Generación de la Allow-List:** El índice invertido (basado en LSM-Tree) produce una lista de IDs de objetos que cumplen con los criterios de filtrado.18  
2. **Búsqueda Vectorial Restringida:** Durante la navegación del grafo HNSW, el buscador ignora activamente cualquier nodo cuyo ID no esté presente en la allow-list.18

Para la fusión de resultados en búsquedas híbridas que incluyen términos de búsqueda (BM25), Weaviate utiliza Reciprocal Rank Fusion (RRF).20 Este algoritmo suma los recíprocos de los rangos de un objeto en diferentes listas de resultados para generar una puntuación final combinada, lo que lo hace robusto frente a las diferencias de escala entre las puntuaciones de similitud de coseno y las puntuaciones BM25.22

![][image2]  
Donde ![][image3] es el conjunto de documentos, ![][image4] es el conjunto de rankings y ![][image5] es una constante de suavizado.22

## **Gestión de Memoria y Estado**

Como sistema escrito en Go, Weaviate enfrenta desafíos únicos en la gestión de memoria que un sistema en Rust como VantaDB puede evitar mediante el control determinista de recursos.

### **El Desafío del Recolector de Basura y el Mapeo de Memoria**

El rendimiento de Weaviate está intrínsecamente ligado al comportamiento del recolector de basura (GC) de Go.8 En escenarios de ingesta masiva, el sistema puede asignar memoria más rápido de lo que el GC puede liberarla, lo que lleva a situaciones de Out-of-Memory (OOM) y la intervención del OOM-Killer del kernel.8 Para mitigar esto, Weaviate depende de variables de entorno como GOMEMLIMIT, que intenta forzar al GC a ser más agresivo cuando se alcanza el 80-90% de la memoria disponible.8

Para la persistencia y el acceso rápido a los datos en disco, Weaviate utiliza archivos mapeados en memoria (mmap).18 Esto permite que el sistema operativo gestione la jerarquía de caché, cargando páginas de disco en la RAM de forma perezosa.25 Sin embargo, el uso de mmap puede causar bloqueos de hilos (stalling) bajo una fuerte presión de I/O, ya que el runtime de Go puede no ser consciente de que una dirección de memoria está causando un fallo de página en el disco.25 Para solucionar esto, Weaviate introdujo el soporte para pread en la estrategia de acceso LSM, lo que permite que el runtime estacione la goroutine que espera el I/O, mejorando la capacidad de respuesta general del sistema.25

### **Concurrencia y Lock Striping para Escrituras Masivas**

La gestión de la concurrencia en Weaviate es un estudio de caso en optimización de bloqueos. Para evitar condiciones de carrera durante la importación paralela de objetos con el mismo UUID, el equipo de ingeniería implementó un patrón de **Lock Striping**.27 En lugar de un único mutex global que penalizaría el rendimiento en un 20%, o un bloqueo por cada objeto (que consumiría gigabytes de RAM), Weaviate utiliza un conjunto fijo de 128 bloqueos.27

Cada objeto entrante se asigna a uno de estos 128 "buckets" mediante una función hash de su UUID.27 Esto garantiza que dos objetos con el mismo ID siempre intenten adquirir el mismo bloqueo, manteniendo la consistencia, mientras que objetos con diferentes IDs fluyen a través de diferentes bloqueos en paralelo.27 Esta técnica reduce la congestión a 1/128 de la solución original consumiendo solo 1 KB de memoria adicional.27

### **Mecanismos de Consolidación y Olvido**

Weaviate maneja el ciclo de vida de los datos mediante procesos de compactación y limpieza de "tombstones" (lápidas).9 En el índice HNSW, los objetos eliminados no se borran físicamente del grafo de inmediato, ya que esto requeriría una reconstrucción costosa de las conexiones de los vecinos.7 En su lugar, se marcan con un "tombstone".9

Un proceso de limpieza asíncrono recorre el grafo periódicamente para eliminar estos nodos y reconectar el grafo de forma que se mantenga la navegabilidad.9 Además, la función de **Object TTL** permite definir políticas de expiración automática basadas en el tiempo de creación o actualización, lo cual es vital para sistemas que gestionan flujos de datos temporales o cachés cognitivos.29

## **Análisis de la Documentación y API**

La interfaz de Weaviate está diseñada para ocultar la complejidad del aprendizaje automático tras abstracciones de alto nivel, aunque esto a veces introduce fricciones en despliegues a gran escala.

### **Lenguaje de Consulta: La Flexibilidad de GraphQL y gRPC**

Weaviate utiliza GraphQL como su lenguaje de consulta principal debido a su capacidad para definir esquemas complejos y permitir que los clientes soliciten exactamente los campos que necesitan.1 La sintaxis de Weaviate para GraphQL es notablemente expresiva, permitiendo realizar búsquedas vectoriales (nearVector), búsquedas por texto (nearText) y filtros condicionales (where) en una sola expresión anidada.31

Recientemente, el SDK ha introducido el **Query Agent**, una función innovadora que utiliza modelos de lenguaje (LLMs) para traducir preguntas en lenguaje natural ("¿Qué productos de cuero cuestan menos de 100 dólares?") en consultas estructuradas de Weaviate.33 El agente analiza el esquema, determina las colecciones necesarias y ejecuta la búsqueda, devolviendo resultados con citas de las fuentes.33 Esto representa un cambio de paradigma hacia bases de datos "agent-friendly".

### **Errores Comunes y Desafíos de Escalabilidad**

A través del análisis de los foros de la comunidad y reportes técnicos, se identifican varios puntos de dolor recurrentes para los desarrolladores:

1. **Corrupción de Datos en Raft:** En clústeres multi-nodo, se han reportado problemas con el consenso de Raft, lo que puede llevar a la corrupción de metadatos o estados inconsistentes del clúster.35  
2. **Shards en Modo Read-Only:** Cuando el uso de disco cruza un umbral crítico (watermark), Weaviate cambia automáticamente los shards a modo solo lectura para prevenir la corrupción de archivos LSM, lo que a menudo confunde a los usuarios que no han configurado alertas de monitoreo.35  
3. **Latencia en Búsqueda Híbrida:** La combinación de resultados de BM25 y vectores puede ser lenta si el conjunto de candidatos es muy grande, especialmente cuando se usan integraciones de modelos externos que añaden latencia de red.35

## **Inspiración para VantaDB (Features para Extraer)**

Para que VantaDB sea competitiva en el ecosistema de Rust, no solo debe igualar el rendimiento de Weaviate, sino superar sus limitaciones estructurales utilizando las capacidades nativas del lenguaje.

### **1\. Implementación de HFresh para Almacenamiento Híbrido RAM/Disco**

VantaDB debería adoptar la lógica del nuevo índice **HFresh** (introducido en Weaviate 1.36).15 HFresh se inspira en el algoritmo SPFresh y resuelve la limitación fundamental de HNSW: la necesidad de mantener todos los vectores en memoria.29

HFresh funciona dividiendo el espacio vectorial en "postings" o regiones almacenadas en disco dentro de un almacén LSM.15 Solo se mantiene en memoria un índice HNSW muy pequeño compuesto por los "centroides" de estas regiones.15

* **Adaptación en Rust:** Podemos utilizar el crate io\_uring para realizar lecturas asíncronas de los postings en disco con una latencia mínima, superando el rendimiento de pread o mmap en Linux. Esto permitiría a VantaDB manejar billones de vectores en hardware con RAM limitada.

### **2\. Cuantización Rotacional (RQ) con Optimización SIMD**

La adopción de **Rotational Quantization (RQ)** es esencial para mantener un recall alto (\>98%) con una compresión agresiva sin necesidad de fases de entrenamiento prolongadas.11

* **Adaptación en Rust:** Rust permite el uso de instrucciones intrínsecas SIMD (como AVX-512 en x86 o NEON en ARM) para realizar las rotaciones pseudoaleatorias y el cálculo de distancias de Hamming de forma extremadamente eficiente. A diferencia de Go, donde la optimización de bajo nivel a menudo requiere ensamblador externo, Rust permite una integración segura y portable de estas optimizaciones.

### **3\. Representación Relacional mediante Ref2Vec**

La capacidad de vectorizar un objeto basándose en sus conexiones (**Ref2Vec**) es la base de una base de datos cognitiva.19 En Weaviate, esto se usa para recomendaciones de "usuario como consulta".19

* **Adaptación en Rust:** En VantaDB, esta lógica puede extenderse para que las neuronas (nodos) propaguen su "influencia semántica" a través de las aristas del grafo. Utilizando el sistema de tipos de Rust, podemos definir grafos donde los pesos de las conexiones afectan dinámicamente al embedding del nodo, permitiendo un aprendizaje continuo sin necesidad de re-entrenar modelos externos.

## **Puntos Débiles (Oportunidad de Mercado)**

Weaviate presenta debilidades significativas derivadas de su elección de lenguaje y arquitectura de grafos que VantaDB puede capitalizar para ofrecer un producto superior.

### **La "Tasa de Latencia" del Recolector de Basura**

El problema más grave de Weaviate es el impacto del GC de Go en la latencia de cola (p99).8 En sistemas de tiempo real, las pausas de "Stop-the-World" del GC son inaceptables.

* **Ventaja de VantaDB:** Rust no tiene GC. El uso de punteros inteligentes (Arc, Box) y el sistema de propiedad aseguran que la memoria se libere de forma determinista. Esto permite que VantaDB ofrezca latencias predecibles incluso bajo cargas de escritura masiva, atrayendo a clientes de sistemas críticos como trading financiero o robótica autónoma.

### **La Falta de Adyacencia de Grafo Real**

Como se analizó, Weaviate usa UUIDs para simular relaciones, lo que lo convierte en un sistema de "tablas vinculadas" más que en un grafo real.16 Las travesías de múltiples saltos son ineficientes porque requieren múltiples búsquedas de índice.18

* **Ventaja de VantaDB:** Al implementar **Index-free Adjacency**, donde cada nodo contiene punteros de memoria directos a sus vecinos, VantaDB puede realizar travesías de grafos órdenes de magnitud más rápido. Esto es crucial para la lógica LISP, donde el código a menudo navega por estructuras de datos profundamente anidadas.

### **Complejidad y Huella de Memoria**

Weaviate requiere una cantidad considerable de RAM solo para mantener el runtime de Go y las estructuras de gestión de shards, lo que dificulta su despliegue en dispositivos de borde (Edge).8

* **Ventaja de VantaDB:** Un binario de Rust puede ser extremadamente compacto y eficiente. VantaDB puede diseñarse para ejecutarse en entornos con memoria restringida, permitiendo que la "inteligencia" de la base de datos resida localmente en dispositivos móviles o sensores, eliminando la dependencia de la nube que a menudo se critica en las soluciones actuales de IA.

En conclusión, Weaviate es un sistema robusto y bien pensado para la era del Big Data vectorial, pero sus cimientos en Go y su modelo de grafos indirecto limitan su potencial para aplicaciones cognitivas de próxima generación. VantaDB, al combinar la eficiencia de Rust con una arquitectura de grafo nativa y técnicas de compresión avanzadas como RQ y HFresh, tiene la oportunidad de posicionarse como la infraestructura definitiva para sistemas de IA que requieren tanto velocidad de búsqueda como profundidad de razonamiento relacional.

#### **Obras citadas**

1. Data structure \- Weaviate Documentation, fecha de acceso: abril 3, 2026, [https://docs.weaviate.io/weaviate/concepts/data](https://docs.weaviate.io/weaviate/concepts/data)  
2. Storage \- Weaviate Documentation, fecha de acceso: abril 3, 2026, [https://docs.weaviate.io/weaviate/concepts/storage](https://docs.weaviate.io/weaviate/concepts/storage)  
3. gRPC \- Weaviate Documentation, fecha de acceso: abril 3, 2026, [https://docs.weaviate.io/weaviate/api/grpc](https://docs.weaviate.io/weaviate/api/grpc)  
4. Vector Search Explained | Weaviate, fecha de acceso: abril 3, 2026, [https://weaviate.io/blog/vector-search-explained](https://weaviate.io/blog/vector-search-explained)  
5. Vector Indexing | Weaviate Documentation, fecha de acceso: abril 3, 2026, [https://docs.weaviate.io/weaviate/concepts/vector-index](https://docs.weaviate.io/weaviate/concepts/vector-index)  
6. HNSW \- Weaviate Knowledge Cards, fecha de acceso: abril 3, 2026, [https://weaviate.io/learn/knowledgecards/hnsw](https://weaviate.io/learn/knowledgecards/hnsw)  
7. HNSW for Vector Databases Explained | by Siddharth Jain \- Medium, fecha de acceso: abril 3, 2026, [https://medium.com/@sidjain1412/hnsw-for-vector-databases-explained-dcda67dd0664](https://medium.com/@sidjain1412/hnsw-for-vector-databases-explained-dcda67dd0664)  
8. Resource Planning | Weaviate Documentation, fecha de acceso: abril 3, 2026, [https://docs.weaviate.io/weaviate/concepts/resources](https://docs.weaviate.io/weaviate/concepts/resources)  
9. Vector index \- Weaviate Documentation, fecha de acceso: abril 3, 2026, [https://docs.weaviate.io/weaviate/config-refs/indexing/vector-index](https://docs.weaviate.io/weaviate/config-refs/indexing/vector-index)  
10. How to planning HNSW index ef, efConstruction and maxConnections parameters with PQ?, fecha de acceso: abril 3, 2026, [https://forum.weaviate.io/t/how-to-planning-hnsw-index-ef-efconstruction-and-maxconnections-parameters-with-pq/9579](https://forum.weaviate.io/t/how-to-planning-hnsw-index-ef-efconstruction-and-maxconnections-parameters-with-pq/9579)  
11. Compression (Vector Quantization) \- Weaviate Documentation, fecha de acceso: abril 3, 2026, [https://docs.weaviate.io/weaviate/concepts/vector-quantization](https://docs.weaviate.io/weaviate/concepts/vector-quantization)  
12. MessagePack: It's like JSON. but fast and small., fecha de acceso: abril 3, 2026, [https://msgpack.org/](https://msgpack.org/)  
13. Weaviate 1.24 Release, fecha de acceso: abril 3, 2026, [https://weaviate.io/blog/weaviate-1-24-release](https://weaviate.io/blog/weaviate-1-24-release)  
14. HNSW+PQ \- Exploring ANN algorithms Part 2.1 | Weaviate, fecha de acceso: abril 3, 2026, [https://weaviate.io/blog/ann-algorithms-hnsw-pq](https://weaviate.io/blog/ann-algorithms-hnsw-pq)  
15. Indexing \- Weaviate Documentation, fecha de acceso: abril 3, 2026, [https://docs.weaviate.io/weaviate/starter-guides/managing-resources/indexing](https://docs.weaviate.io/weaviate/starter-guides/managing-resources/indexing)  
16. Graph Databases Explained: Better Way to Represent Connections \- Cognee, fecha de acceso: abril 3, 2026, [https://www.cognee.ai/blog/fundamentals/graph-databases-explained](https://www.cognee.ai/blog/fundamentals/graph-databases-explained)  
17. Manage relationships with cross-references \- Weaviate Documentation, fecha de acceso: abril 3, 2026, [https://docs.weaviate.io/weaviate/tutorials/cross-references](https://docs.weaviate.io/weaviate/tutorials/cross-references)  
18. FAQ \- Weaviate Documentation, fecha de acceso: abril 3, 2026, [https://docs.weaviate.io/weaviate/more-resources/faq](https://docs.weaviate.io/weaviate/more-resources/faq)  
19. What is Ref2Vec and why you need it for your recommendation system \- Weaviate, fecha de acceso: abril 3, 2026, [https://weaviate.io/blog/ref2vec-centroid](https://weaviate.io/blog/ref2vec-centroid)  
20. Hybrid search | Weaviate Documentation, fecha de acceso: abril 3, 2026, [https://docs.weaviate.io/weaviate/concepts/search/hybrid-search](https://docs.weaviate.io/weaviate/concepts/search/hybrid-search)  
21. Index types and performance | Weaviate Documentation, fecha de acceso: abril 3, 2026, [https://docs.weaviate.io/weaviate/more-resources/performance](https://docs.weaviate.io/weaviate/more-resources/performance)  
22. Hybrid Search Explained | Weaviate, fecha de acceso: abril 3, 2026, [https://weaviate.io/blog/hybrid-search-explained](https://weaviate.io/blog/hybrid-search-explained)  
23. Hybrid search \- VectorDB \- Mintlify, fecha de acceso: abril 3, 2026, [https://mintlify.com/avnlp/vectordb/features/hybrid-search](https://mintlify.com/avnlp/vectordb/features/hybrid-search)  
24. Mastering Golang's Concurrency and Memory Management: GMP, Garbage Collection, and Channel Handling \- Charles Wan, fecha de acceso: abril 3, 2026, [https://charleswan111.medium.com/mastering-golangs-concurrency-and-memory-management-gmp-garbage-collection-and-channel-handling-212dea055961](https://charleswan111.medium.com/mastering-golangs-concurrency-and-memory-management-gmp-garbage-collection-and-channel-handling-212dea055961)  
25. Weaviate 1.21 Release, fecha de acceso: abril 3, 2026, [https://weaviate.io/blog/weaviate-1-21-release](https://weaviate.io/blog/weaviate-1-21-release)  
26. Rethinking Vector Search at Scale: Weaviate's Native, Efficient and Optimized Multi-Tenancy, fecha de acceso: abril 3, 2026, [https://weaviate.io/blog/weaviate-multi-tenancy-architecture-explained](https://weaviate.io/blog/weaviate-multi-tenancy-architecture-explained)  
27. How we solved a race condition with the Lock Striping pattern ..., fecha de acceso: abril 3, 2026, [https://weaviate.io/blog/lock-striping-pattern](https://weaviate.io/blog/lock-striping-pattern)  
28. weaviate/adapters/repos/db/vector/hnsw/delete.go at main \- GitHub, fecha de acceso: abril 3, 2026, [https://github.com/weaviate/weaviate/blob/master/adapters/repos/db/vector/hnsw/delete.go](https://github.com/weaviate/weaviate/blob/master/adapters/repos/db/vector/hnsw/delete.go)  
29. Weaviate 1.36 Release, fecha de acceso: abril 3, 2026, [https://weaviate.io/blog/weaviate-1-36-release](https://weaviate.io/blog/weaviate-1-36-release)  
30. Weaviate 1.35 Release, fecha de acceso: abril 3, 2026, [https://weaviate.io/blog/weaviate-1-35-release](https://weaviate.io/blog/weaviate-1-35-release)  
31. Hybrid search | Weaviate Documentation, fecha de acceso: abril 3, 2026, [https://docs.weaviate.io/weaviate/search/hybrid](https://docs.weaviate.io/weaviate/search/hybrid)  
32. Best practices \- Weaviate Documentation, fecha de acceso: abril 3, 2026, [https://docs.weaviate.io/weaviate/best-practices](https://docs.weaviate.io/weaviate/best-practices)  
33. Query Agent \- Weaviate, fecha de acceso: abril 3, 2026, [https://weaviate.io/product/query-agent](https://weaviate.io/product/query-agent)  
34. Query Agent \- Weaviate Documentation, fecha de acceso: abril 3, 2026, [https://docs.weaviate.io/agents/query](https://docs.weaviate.io/agents/query)  
35. Support \- Weaviate Community Forum, fecha de acceso: abril 3, 2026, [https://forum.weaviate.io/c/support/6](https://forum.weaviate.io/c/support/6)  
36. I got tired of Go's GC choking on millions of vectors, so I built a custom Byte Arena and Userspace WAL for my Vector DB (280k ops/sec) : r/golang \- Reddit, fecha de acceso: abril 3, 2026, [https://www.reddit.com/r/golang/comments/1rku0n9/i\_got\_tired\_of\_gos\_gc\_choking\_on\_millions\_of/](https://www.reddit.com/r/golang/comments/1rku0n9/i_got_tired_of_gos_gc_choking_on_millions_of/)

[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABUAAAAYCAYAAAAVibZIAAABVUlEQVR4Xu2SsStGURjGH0lZFElKps9mk8RktEgM1u8PsFgMZDEYyGAxmPBllUExWJCJlLLLIFnEYlU8T889X+d897ob0/3Vb7nve885zzkvUPEf7NNn+k0/6HhaThik93DvId2ivUlHRic9opdw82xabtJBN+g13FfKED2na3DzSlpuos1W6Rn9bKnlmKINOkO/6F5SNYq9S0fhq3pIy3kUaYGO0Xd6Cl9JoI2u0wk6B2/ciOqFHMMLDtBHekt7orqSLMGL78BXpEOUcgIvIrXgExxX9MOxu2kXvYDT6BClHMCnUGRFj39ahmOLYfqKfJIcKqo5oJdXPMXV/MUzqAdUrR59K0QLxrvOwz9uwuMTCFejk8aHKKR110l4Bl+QLhomQ3equ/2VGr2i7dG3cG83SKMvwgn0+oWM0De4SWruprNaH72Dx0hoyENf7HZWr6j4C34AbzVJ/4fy324AAAAASUVORK5CYII=>

[image2]: <data:image/png;base64,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>

[image3]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABEAAAAYCAYAAAAcYhYyAAAA7UlEQVR4Xu2SrwoCQRCHx2AU/6CgD2CxmMQ3EKNZ8AEsPoMI4gMYLYdJxGwR7GLRYrEZxKAYRFAQ9DfsLeyNt7dd/OAr+5vd29k5oj8uknAG7/AF53DkO4Z7+IQFvcFGCR7hFuZEFoN1uIApkQVokLqFR2pTGJw35aLJAL5hWwYGfAi3HZeBZgkvsCIDgxupuoQMNAe4gmkZGPBNIw9xvQfDh0S2wwUtuWjAX+eajgw0PF5XKz3Yp4ib8ng9shdU4Ykc/8iQ7KMtwh3cyMAkC9f0Pdo87MIrnMJMIPWpwQepxwrzDCewTPY2//w2H9KwMZ6TqNQTAAAAAElFTkSuQmCC>

[image4]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA8AAAAYCAYAAAAlBadpAAABC0lEQVR4XmNgGAXcQLwGiL8C8R8g3gbEs6H4OBBfBmJzuGosQAuInwHxRSAWRZO7CcRPgVgHTRwOAhggti4AYkZUKYZ9QPwfiIvRxOGgnQGiIBNdggHiGlxyYAAy/S0Qm6JLAMEPBoi/ZdAlYOARA3b/qgLxDSA2QhNHASD/3gXiOQyIkD4JxLeBmBNJHVYA8lM2EEtBsSEQH2aAGIgXgKIJm5PtgPgzEPOiiaMAUDQtYMCMIlDoglzEgSaOAiYzYEYDyKClDBDNMD4flIYDESA+zYCZ/FgZIEkWplkXiKdBxRmcGCBpGSQJww+AWAGiFgz8gfgTEKcA8S4GTAsIAmEgDmKAxMAooDsAAIArNUfqYvUHAAAAAElFTkSuQmCC>

[image5]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAsAAAAYCAYAAAAs7gcTAAAA4ElEQVR4Xu2SMQ5BQRCGRyHRSAiFQucEKh2dWqFwAVcQhTiAViNKCo2OSuUONCJRiEqi0igU/v/tjOzb7AUkvuTLm8zM7pvMeyK/TRsu4QOeglqUKjzDdViIwdtfcBgWYozgEzbDQkgObsWNwXFIEZZhxpqMGryKO8CDc7iBB9j3+hL8eQuwAcfwrbkUnJfNPTjT3ATeYN2aCGe8wL242+6aixKurAWnGnP+vMYJbPJXxtda8wB2NJasuC/mr4zNvKAEF7Ci+STgv7ASd5DweYQ72NXcF67KGg1+jNSsf3w+FoElrSo4UnUAAAAASUVORK5CYII=>
