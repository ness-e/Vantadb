# **Análisis de Ingeniería del Sustrato \[pgvector\]: Arquitectura para el Motor Cognitivo VantaDB**

El diseño de VantaDB exige una comprensión profunda de las tecnologías de almacenamiento y búsqueda de vectores existentes para superar las limitaciones de los sistemas convencionales. \[pgvector\] se ha consolidado como el estándar de facto para la integración de capacidades vectoriales en sistemas de gestión de bases de datos relacionales (RDBMS), específicamente dentro del ecosistema de PostgreSQL. Este informe técnico realiza una disección exhaustiva de la arquitectura interna de \[pgvector\], analizando su estructura de datos, lógica de recuperación, gestión de memoria y el comportamiento de su API bajo carga. El objetivo es identificar los componentes críticos que deben ser asimilados, optimizados o descartados en la construcción de VantaDB, una base de datos cognitiva escrita en Rust que busca unificar grafos, vectores y lógica simbólica LISP.

## **Anatomía de la "Neurona": Estructura de Datos e Internos de Almacenamiento**

En la arquitectura de \[pgvector\], la unidad básica de información, que podríamos denominar analógicamente como "neurona" en el contexto de VantaDB, se define mediante el tipo de datos vector. Este tipo no es un objeto monolítico, sino una extensión del sistema de tipos de PostgreSQL que aprovecha la infraestructura de almacenamiento de longitud variable conocida como varlena.1

### **Representación Interna y Formato Binario**

Internamente, \[pgvector\] almacena la información como un array contiguo de números de punto flotante de precisión simple (float32). Cada objeto vectorial comienza con un encabezado (header) que especifica la dimensionalidad del vector, seguido inmediatamente por los valores binarios de las coordenadas.3 Esta disposición lineal es fundamental para la eficiencia de las operaciones de producto escalar y distancia euclidiana, permitiendo un acceso a memoria predecible que beneficia las jerarquías de caché de la CPU.

| Atributo de Almacenamiento | Implementación en \[pgvector\] | Implicación Técnica |
| :---- | :---- | :---- |
| **Formato de Datos** | Array binario contiguo (float32) | Facilita operaciones SIMD y reduce la fragmentación de memoria.3 |
| **Gestión de Metadatos** | Colocalización Relacional | Los metadatos se almacenan en columnas adyacentes de la misma fila.4 |
| **Serialización en Disco** | Páginas de 8KB (Standard PG) | Los vectores se dividen en páginas físicas gestionadas por el heap de Postgres.6 |
| **Mecanismo de Desbordamiento** | TOAST (The Oversized-Attribute Storage Technique) | Vectores de gran tamaño se mueven fuera de la página principal hacia tablas satélite.1 |

Para vectores de alta dimensionalidad, como los modelos de 1,536 dimensiones comunes en LLMs, el tamaño del objeto alcanza aproximadamente los 6 KiB.8 Dado que el tamaño de página estándar de PostgreSQL es de 8 KB, un solo vector ocupa la mayor parte de una página física. Cuando el tamaño del vector supera el umbral crítico (generalmente 2 KB para atributos individuales), el sistema activa el mecanismo TOAST, almacenando el vector en una ubicación externa y dejando solo un puntero en la fila original.1 Este comportamiento introduce una latencia oculta significativa en búsquedas masivas, ya que el motor debe realizar saltos de E/S adicionales para reconstruir el vector antes de calcular las distancias.11

### **Manejo de Metadatos y Persistencia**

\[pgvector\] adopta un enfoque puramente relacional para el manejo de metadatos. A diferencia de las bases de datos vectoriales especializadas que utilizan motores de documentos (como JSONB) o almacenes clave-valor separados para los atributos descriptivos, \[pgvector\] utiliza las columnas estándar de SQL.4 Esta "adyacencia directa" permite que el optimizador de consultas de PostgreSQL aplique filtros estructurados (ej. WHERE user\_id \= 5\) antes de realizar la búsqueda vectorial, una técnica conocida como pre-filtrado.13

La persistencia en disco se realiza mediante el formato de serialización nativo de PostgreSQL. En las fases de carga masiva, \[pgvector\] soporta el formato binario de la sentencia COPY, lo que permite transferir arrays de floats directamente desde el cliente al motor sin la sobrecarga de parsear representaciones de texto como '\[1.2, 3.4\]'.15 Este flujo de datos binario es esencial para la eficiencia en VantaDB, donde la ingesta de señales sensoriales debe ser de baja latencia.

## **Lógica de Recuperación y Búsqueda: Mapeo Sináptico y Heurística**

La búsqueda en \[pgvector\] se divide en dos paradigmas: búsqueda exacta (K-NN) mediante escaneo secuencial y búsqueda aproximada (ANN) mediante estructuras de indexación. Para VantaDB, el interés radica en cómo \[pgvector\] implementa los algoritmos HNSW e IVFFlat para gestionar el compromiso entre precisión (recall) y velocidad (latency).

### **Hierarchical Navigable Small World (HNSW)**

HNSW es el algoritmo de indexación más avanzado en \[pgvector\] desde la versión 0.5.0. Construye un grafo multicapa donde cada nodo es un vector y las aristas representan la proximidad en el espacio de alta dimensión.17 El proceso de búsqueda se asemeja a una navegación por "mundos pequeños": comienza en una capa superior dispersa (autopistas) para localizar la región general y desciende gradualmente a capas más densas (calles locales) hasta encontrar los vecinos más cercanos en la capa base.19

La lógica de HNSW en \[pgvector\] gestiona el compromiso de rendimiento mediante tres parámetros clave:

* **m**: El número máximo de conexiones por nodo. Un valor más alto mejora el recall pero aumenta linealmente el consumo de RAM y el tiempo de construcción del índice.21  
* **ef\_construction**: El tamaño de la lista de candidatos durante la construcción. Determina la calidad de las interconexiones del grafo.21  
* **ef\_search**: Un parámetro de tiempo de ejecución que define cuántos nodos explorar durante una consulta. Aumentar este valor permite recuperar precisión a costa de latencia de CPU.8

### **Inverted File Flat (IVFFlat)**

IVFFlat particiona el espacio vectorial en regiones de Voronoi utilizando el algoritmo K-means.23 Durante la fase de entrenamiento, el índice identifica "centroides" que actúan como representantes de clusters de datos. En el momento de la consulta, el sistema solo escanea los vectores dentro de los clusters cuyos centroides son más cercanos al vector de consulta, lo que se controla mediante el parámetro ivfflat.probes.23

Una debilidad crítica detectada en la lógica de IVFFlat es su naturaleza estática. Si la distribución de los datos cambia drásticamente después de la construcción del índice, los centroides dejan de ser representativos, lo que provoca una caída drástica en el recall. Esto obliga a realizar reconstrucciones periódicas (REINDEX), lo cual es ineficiente para sistemas cognitivos en constante aprendizaje como VantaDB.13

### **Filtrado Híbrido e Iterative Index Scans**

El filtrado híbrido (combinar similitud vectorial con condiciones escalares) es donde \[pgvector\] ha innovado significativamente en su versión 0.8.0 mediante los "Iterative Index Scans".28 Anteriormente, si un filtro era muy selectivo, el índice vectorial podía no devolver suficientes resultados que cumplieran ambas condiciones (problema de sobre-filtrado).14

La nueva lógica iterativa permite que el índice actúe como un generador de estados:

1. El motor escanea el índice vectorial para obtener un lote de candidatos.  
2. Aplica el filtro estructurado (ej. WHERE category\_id \= 10).  
3. Si el número de resultados es inferior al LIMIT solicitado, el índice continúa la exploración desde el punto donde se quedó en su cola de prioridad interna, en lugar de reiniciar la búsqueda.14

Este mecanismo soporta dos modos operativos: strict\_order, que garantiza un ordenamiento perfecto por distancia pero puede ser lento, y relaxed\_order, que prioriza la velocidad devolviendo resultados conforme se encuentran en el grafo.29

## **Gestión de Memoria y Estado: Concurrencia y Contención**

Como extensión de PostgreSQL, \[pgvector\] está sujeto a las limitaciones y fortalezas del gestor de memoria del núcleo. VantaDB, al ser desarrollado en Rust, tiene la oportunidad de evitar cuellos de botella específicos que afectan a \[pgvector\] a gran escala.

### **Caché y Shared Buffers**

\[pgvector\] no implementa una caché propia; depende totalmente del pool de shared\_buffers de PostgreSQL.31 El rendimiento óptimo de HNSW se alcanza cuando todo el grafo reside en RAM. Sin embargo, debido a que el grafo de HNSW tiene un acceso a memoria altamente aleatorio, si las páginas del índice no caben en los buffers compartidos, el sistema cae en una "caminata aleatoria de disco", donde cada salto en el grafo implica un acceso a almacenamiento persistente, degradando el rendimiento en órdenes de magnitud.33

### **Concurrencia y Bloqueos (Locks)**

Un punto crítico reportado en despliegues masivos es la contención de bloqueos durante escaneos simultáneos del índice HNSW. La implementación en hnswscan.c utiliza LockPage(..., HNSW\_SCAN\_LOCK, ShareLock) para proteger la estructura del grafo mientras se atraviesa.34

Se ha identificado una barrera de escalabilidad en aproximadamente 32 conexiones concurrentes. A partir de este umbral, el tiempo de espera en el evento LWLock:LockManager crece exponencialmente.34 El problema radica en que, aunque los bloqueos son compartidos (lectura), la sobrecarga del gestor de bloqueos de PostgreSQL para coordinar múltiples backends accediendo a las mismas páginas de índice crea un cuello de botella serial. En VantaDB, el uso de tipos de Rust como Arc\<RwLock\<T\>\> o estructuras de datos libres de bloqueos (lock-free) debe ser la norma para evitar esta saturación.35

### **Mecanismos de "Olvido" y Compactación**

\[pgvector\] maneja la eliminación de datos mediante el proceso nativo de VACUUM de PostgreSQL. Cuando se elimina un vector, el nodo correspondiente en el grafo HNSW se marca como "muerto". El sistema no reequilibra el grafo inmediatamente, sino que el proceso de autovacuum intenta reutilizar el espacio en las páginas de índice para nuevas inserciones.36

Esta lógica carece de una "compactación inteligente" basada en la relevancia biológica de los datos. En un sistema cognitivo, el olvido debería ser selectivo (borrar lo menos accedido o lo menos correlacionado). En \[pgvector\], la eliminación de nodos puede dejar partes del grafo menos conectadas o incluso inalcanzables, afectando permanentemente el recall a menos que se realice un REINDEX completo.38

## **Análisis de la Documentación y API: Innovación y Fricción**

La documentación de \[pgvector\] revela una API que prioriza la integración sintáctica sobre la abstracción total. Su lenguaje de consulta es una extensión directa de SQL, utilizando operadores de distancia específicos:

| Operador | Función de Distancia | Caso de Uso Biológico / VantaDB |
| :---- | :---- | :---- |
| \<-\> | L2 (Euclidiana) | Medir distancias absolutas en mapas sensoriales. |
| \<=\> | Coseno | Similitud semántica independientemente de la magnitud de la señal. |
| \<\#\> | Producto Escalar Negativo | Maximizar la activación sináptica (especialmente con vectores normalizados).16 |
| \<+\> | L1 (Manhattan) | Robusto frente a valores atípicos en señales ruidosas.16 |

### **Flexibilidad y Sintaxis**

La mayor innovación del SDK es la capacidad de crear "índices de expresión". Por ejemplo, es posible almacenar vectores en precisión completa (fp32) pero indexarlos como precisión media (fp16 o halfvec) para ahorrar el 50% de RAM sin perder recall significativo.39 Esta flexibilidad permite a los arquitectos diseñar sistemas de almacenamiento jerárquico.

Sin embargo, los desarrolladores reportan fricciones importantes en el escalado:

1. **Errores OOM (Out-of-Memory):** Durante la construcción de índices HNSW en datasets de miles de millones de registros, el motor de Postgres a menudo falla porque el algoritmo intenta asignar grandes bloques de memoria para el grafo sin una gestión granular de la presión de RAM.40  
2. **Trampas del Query Planner:** Si las estadísticas de la tabla no están perfectamente actualizadas, el optimizador puede decidir ignorar el índice HNSW y optar por un escaneo secuencial en una tabla de 10 millones de filas, causando picos de latencia de varios segundos.14

## **Inspiración para VantaDB: Features para extraer**

Para que VantaDB sea competitivo frente a gigantes como Pinecone o Milvus, pero mantenga la versatilidad de un RDBMS, debemos implementar y evolucionar tres lógicas clave extraídas de \[pgvector\].

### **1\. El Patrón "Iterative Yield" (Búsqueda en Stream)**

Debemos adoptar la lógica de escaneo iterativo de la versión 0.8.0 de \[pgvector\]. En el contexto de Rust, esto se traduce en implementar el trait Iterator para las búsquedas vectoriales. En lugar de devolver un bloque de ![][image1] resultados, el motor debe "entregar" candidatos uno a uno conforme atraviesa el grafo. Esto permite que una capa superior de lógica LISP evalúe restricciones simbólicas complejas (ej. "Encuentra objetos similares a 'Neuron' pero que solo posean la propiedad 'Inhibitoria'") sin reiniciar la búsqueda costosa en el espacio vectorial.

### **2\. Cuantización Estadística Binaria (SBQ)**

VantaDB debe integrar Statistical Binary Quantization, una mejora sobre la cuantización binaria estándar vista en extensiones como pgvectorscale. SBQ reduce los floats de 4 bytes a representaciones de 1 solo bit mientras mantiene un recall del 99% mediante técnicas de re-ranking estadístico.39 Esto imita el procesamiento "grueso a fino" del sistema visual humano: un escaneo rápido de baja fidelidad para identificar regiones candidatas seguido de un análisis de alta precisión.

### **3\. Dispatching SIMD Nativo con Rust**

\[pgvector\] 0.7.0 introdujo soporte para AVX-512 mediante dispatching dinámico en C.44 VantaDB debe llevar esto más allá utilizando el soporte nativo de Rust para instrucciones vectoriales (crates como std::simd). La arquitectura cognitiva debe detectar las capacidades de la CPU en el arranque y compilar o seleccionar versiones optimizadas de las funciones de distancia para AVX2, AVX-512 o ARM NEON, logrando una paridad de rendimiento con el hardware sin sacrificar la seguridad de memoria de Rust.

## **Puntos Débiles (Oportunidad de Mercado): Superando a \[pgvector\]**

\[pgvector\] sufre de una "deuda arquitectónica" al estar atado a PostgreSQL, lo que abre brechas claras que VantaDB puede explotar.

### **El Cisma RAM-Disco**

El HNSW de \[pgvector\] es fundamentalmente un algoritmo "in-memory" forzado a vivir en una base de datos de disco. Su dependencia de los shared\_buffers lo hace extremadamente ineficiente si el dataset no cabe en RAM.19 VantaDB superará esto implementando una variante de **DiskANN** (específicamente el grafo Vamana), que optimiza la disposición de los nodos en bloques SSD para permitir búsquedas de milisegundos en datasets que superan por 10 veces la memoria RAM disponible.43

### **Contención de Bloqueos en el Gestor de Relaciones**

PostgreSQL utiliza procesos pesados y un gestor de bloqueos centralizado que causa la saturación a las 32 conexiones mencionada anteriormente.34 VantaDB, utilizando el modelo de concurrencia de Rust y un motor de almacenamiento Log-Structured Merge-Tree (LSM-Tree) para los vectores, puede eliminar los bloqueos de página de lectura. Al tratar el índice como una estructura inmutable que se compacta en segundo plano, VantaDB puede escalar a cientos de hilos concurrentes sin contención en el "hot path" de búsqueda.

### **Falta de Lógica Simbólica Profunda**

\[pgvector\] permite filtrado SQL, pero no puede manejar la lógica recursiva o el razonamiento simbólico necesario para tareas cognitivas avanzadas. Al integrar un intérprete LISP directamente en el kernel de Rust de VantaDB, podemos realizar un "Filtrado de Unión Tardía" (Late-Binding Filtering). El intérprete LISP puede guiar la travesía del grafo HNSW, decidiendo qué ramas explorar no solo por distancia geométrica, sino por coherencia lógica con el contexto de la consulta, logrando lo que \[pgvector\] solo simula mediante tablas de unión SQL.

## **Conclusión**

La ingeniería inversa de \[pgvector\] revela un sistema robusto pero limitado por su anfitrión. Su éxito radica en la simplicidad de su API y su integración relacional. VantaDB debe asimilar la eficiencia de sus estructuras de datos lineales y su lógica de escaneo iterativo, pero debe romper con el modelo de memoria de PostgreSQL. Al adoptar una arquitectura de hilos nativa en Rust, implementar grafos optimizados para disco (DiskANN) y unificar la búsqueda vectorial con lógica simbólica LISP, VantaDB tiene la oportunidad de ofrecer una plataforma superior para la próxima generación de aplicaciones neurobiológicas y de inteligencia artificial.

#### **Fuentes citadas**

1. implementing in- memory vector search algorithms for PostgreSQL, acceso: abril 7, 2026, [https://www.postgresql.eu/events/pgconfeu2024/sessions/session/5830/slides/609/pgconfeu-2024-vectors-internal.pdf](https://www.postgresql.eu/events/pgconfeu2024/sessions/session/5830/slides/609/pgconfeu-2024-vectors-internal.pdf)  
2. Storing and querying vector data in Postgres with pgvector \- pganalyze, acceso: abril 7, 2026, [https://pganalyze.com/blog/5mins-postgres-vectors-pgvector](https://pganalyze.com/blog/5mins-postgres-vectors-pgvector)  
3. pgvector: Key features, tutorial, and pros and cons \[2026 guide\], acceso: abril 7, 2026, [https://www.instaclustr.com/education/vector-database/pgvector-key-features-tutorial-and-pros-and-cons-2026-guide/](https://www.instaclustr.com/education/vector-database/pgvector-key-features-tutorial-and-pros-and-cons-2026-guide/)  
4. docs/ai/key-vector-database-concepts-for-understanding-pgvector.md at latest \- GitHub, acceso: abril 7, 2026, [https://github.com/timescale/docs/blob/latest/ai/key-vector-database-concepts-for-understanding-pgvector.md](https://github.com/timescale/docs/blob/latest/ai/key-vector-database-concepts-for-understanding-pgvector.md)  
5. PostgreSQL as a Vector Database: A Pgvector Tutorial \- Tiger Data, acceso: abril 7, 2026, [https://www.tigerdata.com/blog/postgresql-as-a-vector-database-using-pgvector](https://www.tigerdata.com/blog/postgresql-as-a-vector-database-using-pgvector)  
6. Documentation: 18: 66.6. Database Page Layout \- PostgreSQL, acceso: abril 7, 2026, [https://www.postgresql.org/docs/current/storage-page-layout.html](https://www.postgresql.org/docs/current/storage-page-layout.html)  
7. pgvector, a guide for DBA \- Part 2: Indexes (update march 2026\) \- dbi services, acceso: abril 7, 2026, [https://www.dbi-services.com/blog/pgvector-a-guide-for-dba-part-2-indexes-update-march-2026/](https://www.dbi-services.com/blog/pgvector-a-guide-for-dba-part-2-indexes-update-march-2026/)  
8. Improve the performance of generative AI workloads on Amazon Aurora with Optimized Reads and pgvector | AWS Database Blog, acceso: abril 7, 2026, [https://aws.amazon.com/blogs/database/accelerate-generative-ai-workloads-on-amazon-aurora-with-optimized-reads-and-pgvector/](https://aws.amazon.com/blogs/database/accelerate-generative-ai-workloads-on-amazon-aurora-with-optimized-reads-and-pgvector/)  
9. Load vector embeddings up to 67x faster with pgvector and Amazon Aurora \- AWS, acceso: abril 7, 2026, [https://aws.amazon.com/blogs/database/load-vector-embeddings-up-to-67x-faster-with-pgvector-and-amazon-aurora/](https://aws.amazon.com/blogs/database/load-vector-embeddings-up-to-67x-faster-with-pgvector-and-amazon-aurora/)  
10. Best practices for using pgvector, acceso: abril 7, 2026, [https://postgresql.us/events/pgconfnyc2024/sessions/session/1862/slides/172/pgvector\_best\_practices\_pgconfnyc2024.pdf](https://postgresql.us/events/pgconfnyc2024/sessions/session/1862/slides/172/pgvector_best_practices_pgconfnyc2024.pdf)  
11. llms-full.txt \- Nile Postgres, acceso: abril 7, 2026, [https://www.thenile.dev/docs/llms-full.txt](https://www.thenile.dev/docs/llms-full.txt)  
12. pgvector: Vector Search in PostgreSQL (Full Guide) \- Tiger Data, acceso: abril 7, 2026, [https://www.tigerdata.com/learn/postgresql-extensions-pgvector](https://www.tigerdata.com/learn/postgresql-extensions-pgvector)  
13. The Case Against pgvector | Alex Jacobs, acceso: abril 7, 2026, [https://alex-jacobs.com/posts/the-case-against-pgvector/](https://alex-jacobs.com/posts/the-case-against-pgvector/)  
14. Announcing: pgvector 0.8.0 released and available on Nile, acceso: abril 7, 2026, [https://www.thenile.dev/blog/pgvector-080](https://www.thenile.dev/blog/pgvector-080)  
15. pgvector-java/examples/loading/src/main/java/com/example/Example.java at master \- GitHub, acceso: abril 7, 2026, [https://github.com/pgvector/pgvector-java/blob/master/examples/loading/src/main/java/com/example/Example.java](https://github.com/pgvector/pgvector-java/blob/master/examples/loading/src/main/java/com/example/Example.java)  
16. Deep Dive into Vector Similarity Search within Postgres and pgvector \- UserJot, acceso: abril 7, 2026, [https://userjot.com/blog/pgvector-deep-dive](https://userjot.com/blog/pgvector-deep-dive)  
17. Understanding vector search and HNSW index with pgvector \- Neon, acceso: abril 7, 2026, [https://neon.com/blog/understanding-vector-search-and-hnsw-index-with-pgvector](https://neon.com/blog/understanding-vector-search-and-hnsw-index-with-pgvector)  
18. PGVector: HNSW vs IVFFlat — A Comprehensive Study | by BavalpreetSinghh | Medium, acceso: abril 7, 2026, [https://medium.com/@bavalpreetsinghh/pgvector-hnsw-vs-ivfflat-a-comprehensive-study-21ce0aaab931](https://medium.com/@bavalpreetsinghh/pgvector-hnsw-vs-ivfflat-a-comprehensive-study-21ce0aaab931)  
19. Vector Database Basics: HNSW | Tiger Data, acceso: abril 7, 2026, [https://www.tigerdata.com/blog/vector-database-basics-hnsw](https://www.tigerdata.com/blog/vector-database-basics-hnsw)  
20. How to Create HNSW Index \- OneUptime, acceso: abril 7, 2026, [https://oneuptime.com/blog/post/2026-01-30-vector-db-hnsw-index/view](https://oneuptime.com/blog/post/2026-01-30-vector-db-hnsw-index/view)  
21. Faster similarity search performance with pgvector indexes | Google Cloud Blog, acceso: abril 7, 2026, [https://cloud.google.com/blog/products/databases/faster-similarity-search-performance-with-pgvector-indexes](https://cloud.google.com/blog/products/databases/faster-similarity-search-performance-with-pgvector-indexes)  
22. pgvector/pgvector: Open-source vector similarity search for Postgres \- GitHub, acceso: abril 7, 2026, [https://github.com/pgvector/pgvector](https://github.com/pgvector/pgvector)  
23. Nearest Neighbor Indexes: What Are IVFFlat Indexes in Pgvector and How Do They Work, acceso: abril 7, 2026, [https://www.tigerdata.com/blog/nearest-neighbor-indexes-what-are-ivfflat-indexes-in-pgvector-and-how-do-they-work](https://www.tigerdata.com/blog/nearest-neighbor-indexes-what-are-ivfflat-indexes-in-pgvector-and-how-do-they-work)  
24. Use pgvector for Vector Similarity Search | Apache Cloudberry (Incubating), acceso: abril 7, 2026, [https://cloudberry.apache.org/docs/advanced-analytics/pgvector-search](https://cloudberry.apache.org/docs/advanced-analytics/pgvector-search)  
25. Optimize generative AI applications with pgvector indexing: A deep dive into IVFFlat and HNSW techniques | AWS Database Blog, acceso: abril 7, 2026, [https://aws.amazon.com/blogs/database/optimize-generative-ai-applications-with-pgvector-indexing-a-deep-dive-into-ivfflat-and-hnsw-techniques/](https://aws.amazon.com/blogs/database/optimize-generative-ai-applications-with-pgvector-indexing-a-deep-dive-into-ivfflat-and-hnsw-techniques/)  
26. IVFFlat vs HNSW in pgvector: Which Index Should You Use? \- DEV Community, acceso: abril 7, 2026, [https://dev.to/philip\_mcclarence\_2ef9475/ivfflat-vs-hnsw-in-pgvector-which-index-should-you-use-305p](https://dev.to/philip_mcclarence_2ef9475/ivfflat-vs-hnsw-in-pgvector-which-index-should-you-use-305p)  
27. pgvector Index Selection: IVFFlat vs HNSW for PostgreSQL Vector Search \- Medium, acceso: abril 7, 2026, [https://medium.com/@philmcc/pgvector-index-selection-ivfflat-vs-hnsw-for-postgresql-vector-search-6eff26aaa90c](https://medium.com/@philmcc/pgvector-index-selection-ivfflat-vs-hnsw-for-postgresql-vector-search-6eff26aaa90c)  
28. pgvector 0.8.0 Released\! \- PostgreSQL, acceso: abril 7, 2026, [https://www.postgresql.org/about/news/pgvector-080-released-2952/](https://www.postgresql.org/about/news/pgvector-080-released-2952/)  
29. Supercharging vector search performance and relevance with pgvector 0.8.0 on Amazon Aurora PostgreSQL | AWS Database Blog, acceso: abril 7, 2026, [https://aws.amazon.com/blogs/database/supercharging-vector-search-performance-and-relevance-with-pgvector-0-8-0-on-amazon-aurora-postgresql/](https://aws.amazon.com/blogs/database/supercharging-vector-search-performance-and-relevance-with-pgvector-0-8-0-on-amazon-aurora-postgresql/)  
30. An In-Depth Study of Filter-Agnostic Vector Search on a PostgreSQL Database System \- arXiv, acceso: abril 7, 2026, [https://arxiv.org/pdf/2603.23710](https://arxiv.org/pdf/2603.23710)  
31. 30 years of PostgreSQL buffer manager locking design evolution | by Dichen Li | Medium, acceso: abril 7, 2026, [https://medium.com/@dichenldc/30-years-of-postgresql-buffer-manager-locking-design-evolution-e6e861d7072f](https://medium.com/@dichenldc/30-years-of-postgresql-buffer-manager-locking-design-evolution-e6e861d7072f)  
32. Understanding PostgreSQL Shared Buffers for Performance Tuning \- Medium, acceso: abril 7, 2026, [https://medium.com/@jramcloud1/02-postgresql-performance-tuning-understanding-postgresql-shared-buffers-for-performance-tuning-0a61086edee7](https://medium.com/@jramcloud1/02-postgresql-performance-tuning-understanding-postgresql-shared-buffers-for-performance-tuning-0a61086edee7)  
33. ScaNN for AlloyDB: The postgres vector index that works well for all sizes \- Google Cloud, acceso: abril 7, 2026, [https://cloud.google.com/blog/products/databases/how-scann-for-alloydb-vector-search-compares-to-pgvector-hnsw](https://cloud.google.com/blog/products/databases/how-scann-for-alloydb-vector-search-compares-to-pgvector-hnsw)  
34. High LWLock Contention During Concurrent HNSW Index Scans · Issue \#766 \- GitHub, acceso: abril 7, 2026, [https://github.com/pgvector/pgvector/issues/766](https://github.com/pgvector/pgvector/issues/766)  
35. HNSW Vector Search causes complete query starvation due to ReadLock held across await (Write-Biased Starvation) · Issue \#6819 · surrealdb/surrealdb \- GitHub, acceso: abril 7, 2026, [https://github.com/surrealdb/surrealdb/issues/6819](https://github.com/surrealdb/surrealdb/issues/6819)  
36. Index locking issue · Issue \#281 · pgvector/pgvector \- GitHub, acceso: abril 7, 2026, [https://github.com/pgvector/pgvector/issues/281](https://github.com/pgvector/pgvector/issues/281)  
37. Signal-driven health monitoring for HNSW indices w/ pgvector | by Jake Casto \- Medium, acceso: abril 7, 2026, [https://medium.com/engineering-layers/signal-driven-health-monitoring-for-hnsw-indices-w-pgvector-ba35d9a6e575](https://medium.com/engineering-layers/signal-driven-health-monitoring-for-hnsw-indices-w-pgvector-ba35d9a6e575)  
38. Vector Indexes: HNSW vs IVFFLAT vs IVF\_RaBitQ \- Kodesage, acceso: abril 7, 2026, [https://kodesage.ai/blog/vector-indexes-hnsw-vs-ivfflat-vs-ivf-rabitq](https://kodesage.ai/blog/vector-indexes-hnsw-vs-ivfflat-vs-ivf-rabitq)  
39. Scalar and binary quantization for pgvector vector search and storage \- Jonathan Katz, acceso: abril 7, 2026, [https://jkatz05.com/post/postgres/pgvector-scalar-binary-quantization/](https://jkatz05.com/post/postgres/pgvector-scalar-binary-quantization/)  
40. Question about the memory allocation when building hnsw index · Issue \#843 \- GitHub, acceso: abril 7, 2026, [https://github.com/pgvector/pgvector/issues/843](https://github.com/pgvector/pgvector/issues/843)  
41. OOM errors \- during insertion and HNSW l1 INDEX builds on version "0.7.2" \#643 \- GitHub, acceso: abril 7, 2026, [https://github.com/pgvector/pgvector/issues/643](https://github.com/pgvector/pgvector/issues/643)  
42. I Spent a Week Researching What to Use Instead of pgvector. Here's the Honest Answer. | by Victoria Mycolaivna \- Medium, acceso: abril 7, 2026, [https://medium.com/@vhrechukha/i-spent-a-week-researching-what-to-use-instead-of-pgvector-heres-the-honest-answer-d6a2ce0a0613](https://medium.com/@vhrechukha/i-spent-a-week-researching-what-to-use-instead-of-pgvector-heres-the-honest-answer-d6a2ce0a0613)  
43. PostgreSQL 18 \+ pgvector: The Definitive Guide to Building Production-Grade RAG Pipelines | by Mohit soni \- Medium, acceso: abril 7, 2026, [https://medium.com/@mohitsoni\_/postgresql-18-pgvector-the-definitive-guide-to-building-production-grade-rag-pipelines-239ee9c0e56f](https://medium.com/@mohitsoni_/postgresql-18-pgvector-the-definitive-guide-to-building-production-grade-rag-pipelines-239ee9c0e56f)  
44. pgvector 0.7.0 Released\! \- PostgreSQL, acceso: abril 7, 2026, [https://www.postgresql.org/about/news/pgvector-070-released-2852/](https://www.postgresql.org/about/news/pgvector-070-released-2852/)  
45. AVX-512: First Impressions on Performance and Programmability | Shihab Khan, acceso: abril 7, 2026, [https://shihab-shahriar.github.io/blog/2026/AVX-512-First-Impressions-on-Performance-and-Programmability/](https://shihab-shahriar.github.io/blog/2026/AVX-512-First-Impressions-on-Performance-and-Programmability/)  
46. Implementing Filtered Semantic Search Using Pgvector and JavaScript | by Team Timescale, acceso: abril 7, 2026, [https://medium.com/timescale/implementing-filtered-semantic-search-using-pgvector-and-javascript-7c6eb4894c36](https://medium.com/timescale/implementing-filtered-semantic-search-using-pgvector-and-javascript-7c6eb4894c36)

[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA0AAAAgCAYAAADJ2fKUAAABLElEQVR4Xu2SvSvGURTHv8LiLVJeShllMshkxGhXRpuMymaxKJtRSpRNySJiUxarf8AgGVCKzcvn/M7v6t7r9zw9RvV86lO3e8653XPulZr8oh1ncBl3cBN7k4wK+vECn/ELr7A7yajDorxoOw/UY1deZMUN0Yc3+ITjWawmk/gmL7QDGqKqH5vqfLk3He0XtOCe0n6G8AQ3cB1vcbCMFeT9jOJxuV6QH3aHw6HAiPuZwiN5obEkL7JHt+v+EPp5wDMciYO1+HNRPIQX/MB7XFF2nZh8CD14KD9kLcpLmMBXpZ/UhvGO59gh/9Cr8gMLqh7VHtT27C8as7iPbSHBkvNPakmf8utZXwc4F4KdeImPOBY2oQtP5dO8xi1lQ7EEM6cVB+T92ISb/CO+AauiQTPgpSogAAAAAElFTkSuQmCC>
