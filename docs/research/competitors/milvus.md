# **Análisis de Ingeniería Inversa de Milvus: Arquitectura, Mecánica de Datos y Estrategias de Optimización para ConnectomeDB**

La evolución de las bases de datos vectoriales ha alcanzado un punto de inflexión con Milvus, un sistema que ha trascendido la simple búsqueda de similitud para convertirse en una plataforma de gestión de datos distribuidos para inteligencia artificial de escala masiva. Desde la perspectiva de la arquitectura de sistemas, Milvus representa una implementación sofisticada del principio de desagregación de recursos, separando no solo el cómputo del almacenamiento, sino también diferenciando las cargas de trabajo de lectura, escritura e indexación en microservicios independientes.1 Para el desarrollo de ConnectomeDB, un sistema que busca emular la complejidad neurobiológica mediante grafos y lógica LISP, el estudio de la mecánica interna de Milvus ofrece lecciones críticas sobre cómo manejar la dimensionalidad masiva y la consistencia eventual en entornos de nube.

## **Anatomía de la "Neurona": Estructura de Datos y Jerarquía de Almacenamiento**

En la arquitectura de Milvus, la unidad fundamental de información, que podríamos conceptualizar como una neurona en el contexto de ConnectomeDB, es la entidad. Sin embargo, la eficiencia del sistema no reside en el manejo individual de estas entidades, sino en su organización jerárquica diseñada para maximizar el rendimiento del procesamiento masivamente paralelo (MPP).1 La estructura se organiza desde el nivel lógico de colecciones y particiones hasta la unidad física de ejecución conocida como segmento.2

### **Organización Lógica y Segmentación Dinámica**

Una colección en Milvus es el equivalente a una tabla en una base de datos relacional, definida por un esquema que puede contener múltiples campos vectoriales y escalares.1 La partición es una subdivisión lógica que permite a los usuarios organizar los datos para optimizar la búsqueda mediante la poda de particiones.2 La verdadera innovación técnica ocurre a nivel de segmento. Milvus divide los datos en segmentos, que son paquetes de datos que contienen un número determinado de entidades. Existen dos estados críticos para un segmento: el segmento creciente (growing) y el segmento sellado (sealed).2

| Componente Jerárquico | Función Técnica | Persistencia y Estado |
| :---- | :---- | :---- |
| Colección | Contenedor lógico de nivel superior con esquema definido. | Metadatos en etcd 1 |
| Partición | Subdivisión para aislamiento de datos y optimización de consultas. | Metadatos en etcd 2 |
| Segmento Creciente | Buffer en memoria para ingesta en tiempo real. | Almacenamiento en WAL / Memoria 2 |
| Segmento Sellado | Bloque de datos inmutable optimizado para búsqueda ANN. | Almacenamiento de objetos (S3/MinIO) 2 |

Los segmentos crecientes reciben datos mediante un registro de escritura anticipada (Write-Ahead Log o WAL), utilizando sistemas como Kafka o Pulsar para garantizar la durabilidad antes de que los datos sean persistidos en el almacenamiento de objetos.1 Una vez que un segmento alcanza un tamaño umbral, se sella, se vuelve inmutable y se activa el proceso de indexación. Esta inmutabilidad es clave para la arquitectura de Milvus, ya que permite que los nodos de consulta (Query Nodes) carguen datos desde el almacenamiento de objetos sin preocuparse por conflictos de escritura, facilitando una escalabilidad horizontal casi lineal.2

### **Almacenamiento Columnar y Serialización en Parquet**

La transición de Milvus V1 a V2 marcó un cambio fundamental en el formato de almacenamiento de los segmentos, pasando de archivos binarios personalizados a Apache Parquet.8 Parquet, al ser un formato orientado a columnas, permite a Milvus realizar una lectura selectiva de campos. En una consulta que solo requiere el ID y el vector de una entidad, el sistema puede omitir por completo la lectura de otros metadatos escalares, reduciendo drásticamente el uso de ancho de banda de entrada/salida (I/O).8

La serialización de metadatos en Parquet permite además aprovechar las estadísticas integradas en los footers de los archivos. Milvus utiliza estas estadísticas, como los valores mínimos y máximos de cada columna dentro de un grupo de filas (row group), para realizar una poda de datos temprana.8 Si una consulta busca una entidad con un ID específico, el sistema puede determinar si ese ID existe en un segmento particular simplemente leyendo unos pocos bytes del footer, evitando la descarga y el escaneo de todo el archivo desde el almacenamiento de objetos.8

### **Esquema Dinámico y JSON Shredding**

Para sistemas que requieren la flexibilidad de una base de datos cognitiva, Milvus implementa el soporte para esquemas dinámicos y campos JSON.5 El mecanismo subyacente, conocido como "JSON Shredding", es una técnica de optimización que descompone los objetos JSON en un almacenamiento columnar.10 En lugar de tratar el JSON como un blob de texto opaco, Milvus identifica las claves frecuentes y las almacena en columnas físicas dedicadas, aplicando técnicas de inferencia de tipos en tiempo de ejecución.10

| Tipo de Clave JSON | Estrategia de Almacenamiento | Beneficio de Rendimiento |
| :---- | :---- | :---- |
| Claves con tipo (Typed) | Columnas dedicadas con tipos fuertes (INT, FLOAT, VARCHAR). | Escaneo directo de columnas 10 |
| Claves dinámicas | Columnas dinámicas basadas en el tipo observado. | Flexibilidad sin pérdida de rendimiento 10 |
| Claves compartidas (Sparse) | Columna binaria compacta con índice invertido de claves. | Filtrado acelerado hasta 89x 10 |

Este enfoque permite que las consultas de filtrado sobre metadatos complejos sean casi tan rápidas como las consultas sobre campos escalares predefinidos. Para ConnectomeDB, esta capacidad de "shredding" es esencial para manejar la naturaleza evolutiva de las conexiones neuronales y sus atributos lógicos sin sacrificar la velocidad de recuperación.10

## **Lógica de Recuperación y Búsqueda: El Motor Knowhere y la Ejecución de Consultas**

La recuperación de información en Milvus es un proceso de dos etapas que combina la búsqueda de vecinos más cercanos (ANN) con el filtrado escalar booleano.11 El corazón de esta operación es Knowhere, el motor de ejecución vectorial de Milvus escrito en C++, que encapsula y extiende bibliotecas como Faiss y Hnswlib.13

### **El Motor Knowhere y la Aceleración de Hardware**

Knowhere no es simplemente un wrapper; es un motor de computación heterogénea que decide dinámicamente si ejecutar una tarea en la CPU o en la GPU.13 Una de las innovaciones más potentes de Knowhere es su capacidad para la selección automática de instrucciones SIMD (Single Instruction, Multiple Data). Durante el tiempo de ejecución, el motor detecta las capacidades del procesador (SSE, AVX, AVX2 o AVX-512) y enlaza los punteros de función a la versión más optimizada del algoritmo de cálculo de distancia.13 El soporte para AVX-512, en particular, puede mejorar el rendimiento de la construcción de índices y las consultas entre un 20% y un 30% en comparación con AVX2.13

Para el cálculo de similitud, Milvus soporta una amplia gama de métricas, incluyendo la distancia Euclídea ![][image1], el producto interno (IP) y la similitud de coseno, así como métricas específicas para vectores binarios como Jaccard y Hamming.14 En el contexto de ConnectomeDB, la capacidad de Knowhere para manejar estructuras de datos complejas mediante el uso de OffsetBaseIndex —donde solo se almacenan los IDs en el archivo de índice para reducir el tamaño— ofrece una vía para gestionar grafos de conocimiento densos con un uso eficiente del almacenamiento.13

### **Algoritmos de Indexación Vectorial**

Milvus permite a los arquitectos de bases de datos elegir entre múltiples tipos de índices según las necesidades de latencia y precisión del caso de uso.

| Algoritmo | Estructura de Datos | Complejidad de Búsqueda | Trade-off Principal |
| :---- | :---- | :---- | :---- |
| HNSW | Grafo de proximidad multicapa. | ![][image2] | Alto consumo de memoria RAM 18 |
| IVF\_FLAT | Celdas de Voronoi (Clustering k-means). | ![][image3] | Requiere entrenamiento y ajuste de nprobe 18 |
| DiskANN | Grafo optimizado para SSD/NVMe. | Basada en I/O de disco. | Baja memoria, mayor latencia 2 |
| SCANN | Cuantización con re-ranking anisotrópico. | Basada en registros CPU. | Alta precisión a gran escala 2 |

HNSW se ha convertido en el estándar de facto para casos de uso de baja latencia debido a su excelente recall y velocidad, aunque su penalización en memoria es significativa, a menudo duplicando el tamaño de los vectores originales debido a la estructura de grafos adyacentes.18 Por otro lado, la familia de índices IVF (Inverted File) ofrece una mayor eficiencia de memoria mediante técnicas de cuantización escalar o de producto (PQ/SQ8), permitiendo comprimir vectores de 32 bits a 8 bits o incluso menos, a costa de una ligera pérdida de precisión.20

### **Mecanismo de Bitset para Filtrado y Eliminación**

Una de las piezas de ingeniería más críticas en Milvus es el uso de bitsets para implementar el filtrado de atributos y las eliminaciones lógicas (soft deletes).25 Un bitset es un array compacto de bits donde cada posición corresponde a una fila en un segmento. Milvus genera bitsets en tiempo real durante la evaluación de expresiones escalares.25

El flujo de ejecución de una consulta híbrida es el siguiente:

1. **Evaluación de Expresión:** El motor de consulta procesa la condición escalar (por ejemplo, status \== "active") y genera un bitset de filtrado donde los bits en "1" representan las entidades que cumplen la condición.25  
2. **Gestión de Eliminaciones:** Se consulta el bitset de eliminación persistente (donde "1" significa borrado).25  
3. **Integración con ANN:** El motor Knowhere recibe estos bitsets y, durante el recorrido del grafo (en HNSW) o el escaneo de clusters (en IVF), omite cualquier ID cuya posición en el bitset combinado sea "1".13

Este diseño permite que el filtrado se realice *durante* la búsqueda vectorial, lo que se conoce como pre-filtrado, evitando el problema común de recuperar vectores que luego son descartados por no cumplir los criterios de metadatos.11 Además, el soporte para "Time Travel" se basa en esta misma lógica: Milvus filtra las entidades comparando sus marcas de tiempo individuales con el timestamp de la consulta, permitiendo ver el estado de la base de datos en cualquier punto del pasado.25

### **Árboles de Ejecución y Gramática de Consultas**

Milvus utiliza ANTLR para generar árboles de sintaxis abstracta (PlanAST) a partir de las expresiones de los usuarios.12 La gramática de Milvus permite operaciones lógicas complejas (AND, OR, NOT), operadores de comparación (==, \>, \<=), y funciones avanzadas como array\_contains o json\_contains.12 Recientemente, el sistema ha integrado soporte para geometrías geoespaciales (WKT/WKB) mediante un índice R-Tree, permitiendo búsquedas que combinan proximidad semántica y restricciones geográficas.28

Para ConnectomeDB, el análisis del PlanAST de Milvus revela una oportunidad de integración con LISP. Mientras que Milvus traduce expresiones de tipo SQL a un plan de ejecución binario, ConnectomeDB puede mapear directamente las expresiones lógicas LISP a estructuras de nodos de consulta que operen sobre los bitsets del motor de búsqueda, unificando la inferencia simbólica con la recuperación sub-simbólica.12

## **Gestión de Memoria y Estado: El Desafío de la Desagregación**

La gestión de memoria en Milvus es una danza compleja entre el heap de Go (plano de control) y la memoria nativa de C++ (plano de datos), mediada por CGO.2 Este diseño presenta desafíos únicos de rendimiento y estabilidad que son fundamentales para cualquier arquitecto que construya un sistema similar en Rust.

### **Memory Mapping (MMap) y Tiered Storage**

Para manejar conjuntos de datos que superan la capacidad de la RAM física, Milvus introdujo el soporte para MMap (Memory-mapped files) en la versión 2.3.20 Al utilizar MMap, Milvus permite que el sistema operativo gestione la carga y descarga de páginas de datos desde el disco local a la memoria según sea necesario. Esto reduce el uso de memoria RAM entre un 60% y un 80% en comparación con la carga completa en memoria, manteniendo latencias estables ya que los datos residen en discos NVMe locales en lugar de almacenamiento de objetos remoto.20

En la versión 2.6, Milvus dio un paso más con el "Tiered Storage" (Almacenamiento por niveles). Este sistema implementa un esquema de carga bajo demanda (lazy loading) donde solo los metadatos esenciales se cargan al inicio. Los vectores y los archivos de índice se descargan del almacenamiento de objetos solo cuando son tocados por una consulta, utilizando una política de expulsión LRU (Least Recently Used) para gestionar el espacio en el disco local y la memoria.20

| Métrica | Carga Completa (Full Load) | MMap (Local SSD) | Tiered Storage (S3 \+ LRU) |
| :---- | :---- | :---- | :---- |
| Latencia P99 | \< 20 ms | 20-40 ms | 100-500 ms (Cache miss) 20 |
| Capacidad de Escala | Limitada por RAM | Limitada por Disco Local | Virtualmente Ilimitada 20 |
| Costo Operativo | Muy Alto ( ) | Medio ($$) | Bajo ($) 20 |

Esta jerarquía de memoria es vital para ConnectomeDB. Al estar escrito en Rust, ConnectomeDB puede gestionar estas transiciones de memoria con mayor seguridad y menor overhead que Go, utilizando bibliotecas de mapeo de memoria nativas y controlando el ciclo de vida de los datos sin la interferencia del recolector de basura.32

### **El Problema de la Copia de Datos en CGO**

Un análisis técnico profundo de Milvus revela cuellos de botella en la frontera entre Go y C++. Actualmente, la reducción de resultados de búsqueda (ReduceSearchResults) se realiza en memoria asignada por C, pero antes de enviarse a través de gRPC, estos datos se copian a un slice de bytes en el heap de Go.34 Esta copia innecesaria añade latencia y presión al GC, especialmente en cargas de trabajo con un topK grande o un número elevado de consultas concurrentes (NQ).34 Existen propuestas en la comunidad para implementar un camino de "zero-copy" que permita a gRPC referenciar directamente la memoria asignada por C, liberándola solo después de que el mensaje haya sido serializado.34

### **Interoperabilidad con Apache Arrow**

Milvus utiliza Apache Arrow como formato de intercambio para garantizar la interoperabilidad y minimizar la serialización.36 Arrow define un layout de memoria columnar que es idéntico tanto en disco como en RAM, permitiendo que diferentes procesos (como un nodo de datos y un nodo de consulta) lean la misma estructura de memoria sin transformaciones.36 En entornos de memoria desagregada, como los habilitados por tecnologías de interconexión rápida, Milvus puede aprovechar Arrow para permitir que múltiples nodos accedan a un pool de memoria compartido (Cluster Shared Memory), eliminando la necesidad de transferencia de datos a través de la red en ciertos escenarios.36

## **Análisis de la Documentación y API: La Perspectiva del Desarrollador**

La API de Milvus ha evolucionado hacia la simplicidad con el MilvusClient, abstrayendo la complejidad de las conexiones gRPC y la gestión de esquemas.2 Sin embargo, la brecha entre la "facilidad de uso" y el "rendimiento en producción" sigue siendo significativa.

### **Niveles de Consistencia y Time Tick**

Un punto crítico que a menudo causa confusión en los desarrolladores es el parámetro de consistency\_level. Milvus, al ser un sistema distribuido que utiliza un modelo de logs para la ingesta, no garantiza la visibilidad inmediata por defecto (consistencia eventual).40 El sistema utiliza un mecanismo de "Time Tick" para sincronizar el tiempo a través de todos los componentes.

| Nivel de Consistencia | Latencia de Escritura a Lectura | Impacto en Rendimiento |
| :---- | :---- | :---- |
| Strong | Inmediata (Espera al Time Tick) | Alta latencia de consulta (\~200ms) 40 |
| Session | Visible para el mismo cliente | Medio |
| Bounded | Retraso controlado (default) | Óptimo para throughput 40 |
| Eventually | Sin garantías de tiempo | Máximo rendimiento |

Para ConnectomeDB, que integra lógica LISP, el manejo de la consistencia es primordial. Si un "pensamiento" (vector) se inserta en la base de datos, las deducciones lógicas posteriores deben poder "ver" ese dato inmediatamente. Milvus demuestra que lograr esto en un sistema distribuido requiere un compromiso en la latencia de búsqueda, un área donde una arquitectura en Rust con una gestión de hilos más eficiente podría innovar.33

### **Integración de Modelos: De la Base de Datos al Pipeline**

Milvus 2.6 introdujo el módulo "Function", que permite integrar el proceso de embedding directamente en la base de datos.41 En lugar de que la aplicación cliente gestione la llamada a APIs de OpenAI o Cohere, Milvus lo hace internamente durante la ingesta y la búsqueda.41 Esto simplifica el código de la aplicación (glue code), pero introduce dependencias externas en el motor de la base de datos, una decisión de diseño que debe ser evaluada cuidadosamente para ConnectomeDB, donde la soberanía de los datos y el procesamiento local pueden ser prioridades.42

## **Inspiración para ConnectomeDB: Características para Extraer**

Al realizar ingeniería inversa sobre Milvus, surgen varias características "premium" que ConnectomeDB debería considerar para su implementación en Rust.

### **1\. Motor de Indexación Enchufable (Knowhere-like)**

La capacidad de Milvus para integrar Faiss, Hnswlib y otros motores bajo una interfaz común (VecIndex) es una estrategia maestra.13 ConnectomeDB debería construir una abstracción similar en Rust que permita cambiar el motor de búsqueda (por ejemplo, pasar de HNSW a un índice de grafos nativo de ConnectomeDB) sin alterar la capa de consulta LISP. El uso de CGO en Milvus es una debilidad; ConnectomeDB puede usar FFI de Rust para interactuar con bibliotecas C++ con mayor seguridad y menores costos de cambio de contexto.

### **2\. Segmentación y Compactación de Datos**

El modelo de segmentos inmutables de Milvus es fundamental para la estabilidad en escala.2 La compactación de segmentos (merging de pequeños segmentos en grandes archivos Parquet) reduce el número de llamadas a la API de almacenamiento de objetos y optimiza el rendimiento de búsqueda al reducir la fragmentación del índice.6 ConnectomeDB puede implementar una compactación de grafos similar, donde los nodos y aristas se agrupan físicamente para maximizar la localidad de caché durante los recorridos lógicos.

### **3\. Bitsets SIMD-Acelerados para Razonamiento**

La implementación de bitsets en Milvus es puramente para filtrado y borrado.25 ConnectomeDB puede extender este concepto: usar bitsets para representar estados de activación en una red neuronal o resultados de inferencia lógica que se inyectan en la búsqueda vectorial. Si el motor de Rust utiliza instrucciones AVX-512 nativas para realizar operaciones booleanas entre estos bitsets de "activación" y los bitsets de "datos", se obtendría una simbiosis sin precedentes entre lógica y vectores.15

### **4\. Zero-Copy IPC con Apache Arrow**

ConnectomeDB debería adoptar Apache Arrow desde el primer día para todas las transferencias de datos internas.36 Al ser Rust un lenguaje con control total sobre el layout de memoria, la integración con Arrow es natural y permite que los resultados de una búsqueda se pasen a un motor de razonamiento LISP o a un visualizador de grafos sin una sola copia de memoria, superando el cuello de botella que Milvus enfrenta con CGO.34

## **Puntos Débiles: La Oportunidad de Mercado para ConnectomeDB**

A pesar de su éxito, Milvus tiene debilidades inherentes a su arquitectura de microservicios y su elección de lenguajes que ConnectomeDB puede capitalizar.

### **Complejidad Operativa (The Kubernetes Burden)**

Milvus es un "monstruo" operativo. Para un despliegue completo, se requieren nodos para Query, Data, Index, Proxy, además de etcd, Pulsar/Kafka y MinIO.1 Esto genera una "tensión de configuración" donde equipos pequeños se ven abrumados por la gestión de infraestructura.43

| Factor | Milvus (Cluster) | ConnectomeDB (Propuesta) |
| :---- | :---- | :---- |
| Despliegue | K8s, Helm, Operadores complejos 43 | Binario único de Rust (Edge/Cloud) |
| Dependencias | etcd, Pulsar, S3, MinIO 1 | Embebido (RocksDB/TiKV) o S3 opcional |
| Facilidad de uso | Requiere expertos en infraestructura 43 | Plug-and-play para desarrolladores de IA |

ConnectomeDB puede ganar mercado ofreciendo una experiencia "serverless-first" o embebida, similar a lo que Milvus Lite intenta tímidamente, pero con la potencia total de un sistema escrito en un lenguaje de sistemas moderno como Rust.3

### **Latencia en Escenarios de Alto Throughput**

Milvus muestra una degradación significativa del rendimiento cuando el parámetro limit (Top-K) aumenta.35 Recuperar 100 resultados es órdenes de magnitud más rápido que recuperar 5000, debido al costo de mover esos datos a través de la red y procesarlos en la capa de reducción del Proxy.34 ConnectomeDB, al unificar grafos y vectores, puede utilizar el contexto del grafo para limitar la búsqueda vectorial a vecindarios lógicamente relevantes, evitando el escaneo masivo y la reducción costosa que penaliza a Milvus.

### **Rigidez en el Razonamiento Híbrido**

Milvus es, en esencia, un motor de búsqueda con filtros.11 No puede realizar inferencias sobre los datos mientras busca. Si ConnectomeDB permite que la lógica LISP defina el "camino" de la búsqueda vectorial en tiempo real —por ejemplo, cambiando la métrica de distancia o el peso de las dimensiones basándose en reglas lógicas durante el recorrido del grafo— superará la capacidad de Milvus para manejar tareas cognitivas complejas.12

### **Consumo de Memoria de HNSW**

El índice HNSW de Milvus es extremadamente costoso en términos de RAM, lo que eleva los costos de infraestructura en la nube (AWS instancias R6i son caras).20 El uso de técnicas de cuantización ayuda, pero a menudo degrada el recall significativamente.20 ConnectomeDB tiene la oportunidad de investigar estructuras de grafos de búsqueda más ligeras o de aprovechar mejor el almacenamiento en disco mediante implementaciones nativas de Rust que tengan un control más granular sobre las páginas de memoria que MMap.20

## **Síntesis Técnica para el Arquitecto de ConnectomeDB**

Milvus es una obra maestra de la ingeniería distribuida, pero su arquitectura refleja las limitaciones de una era donde la separación de servicios era la única forma de escalar. Al construir ConnectomeDB en Rust, se tiene la oportunidad de colapsar esta complejidad en un sistema que es a la vez más eficiente, más predecible y fundamentalmente más inteligente.

Las lecciones de Milvus son claras:

1. **Formatos Estándar:** Usar Parquet y Arrow no es opcional; es la base de la interoperabilidad moderna.8  
2. **Hardware-First:** El software debe adaptarse al hardware (SIMD, GPU) en tiempo de ejecución, no al revés.13  
3. **Desacoplamiento Inteligente:** Separar el almacenamiento es vital para el costo, pero el plano de control puede y debe ser más integrado para reducir la latencia.1  
4. **Lógica en el Núcleo:** Los filtros no son suficientes. El futuro de las bases de datos vectoriales es el razonamiento integrado, donde la lógica (LISP en el caso de ConnectomeDB) sea un ciudadano de primera clase en el motor de ejecución, no un filtro posterior aplicado a los resultados.12

La oportunidad para ConnectomeDB reside en la intersección de la eficiencia del lenguaje Rust y la flexibilidad de la neurobiología, creando un sistema que no solo almacene vectores, sino que los conecte y los procese con la elegancia de un sistema nervioso digital. En este sentido, Milvus no es un competidor a vencer, sino un mapa de infraestructura sobre el cual ConnectomeDB puede construir la siguiente capa de inteligencia de datos.

#### **Obras citadas**

1. Milvus Vector Database: Uses, Architecture && Quick Tutorial, fecha de acceso: abril 4, 2026, [https://cloudian.com/guides/ai-infrastructure/milvus-vector-database-uses-architecture-quick-tutorial/](https://cloudian.com/guides/ai-infrastructure/milvus-vector-database-uses-architecture-quick-tutorial/)  
2. Milvus Vector Database \- Augment Code, fecha de acceso: abril 4, 2026, [https://www.augmentcode.com/open-source/milvus-io/milvus](https://www.augmentcode.com/open-source/milvus-io/milvus)  
3. Milvus is a high-performance, cloud-native vector database built for scalable vector ANN search \- GitHub, fecha de acceso: abril 4, 2026, [https://github.com/milvus-io/milvus](https://github.com/milvus-io/milvus)  
4. Milvus 2.3.4: Faster Searches, Expanded Data Support, Improved Monitoring, and More, fecha de acceso: abril 4, 2026, [https://milvus.io/blog/milvus-2-3-4-faster-searches-expanded-data-support-improved-monitoring-and-more.md](https://milvus.io/blog/milvus-2-3-4-faster-searches-expanded-data-support-improved-monitoring-and-more.md)  
5. JSON Field | Milvus Documentation, fecha de acceso: abril 4, 2026, [https://milvus.io/docs/use-json-fields.md](https://milvus.io/docs/use-json-fields.md)  
6. Clustering Compaction | Milvus Documentation, fecha de acceso: abril 4, 2026, [https://milvus.io/docs/clustering-compaction.md](https://milvus.io/docs/clustering-compaction.md)  
7. Milvus Architecture Overview, fecha de acceso: abril 4, 2026, [https://milvus.io/docs/architecture\_overview.md](https://milvus.io/docs/architecture_overview.md)  
8. A Deep Dive into Data Addressing in Storage Systems: From HashMap to HDFS, Kafka, Milvus, and Iceberg, fecha de acceso: abril 4, 2026, [https://milvus.io/blog/data-addressing-storage-systems.md](https://milvus.io/blog/data-addressing-storage-systems.md)  
9. Milvus Supports Imports of Apache Parquet Files for Enhanced Data Processing Efficiency, fecha de acceso: abril 4, 2026, [https://milvus.io/blog/milvus-supports-apache-parquet-file-supports.md](https://milvus.io/blog/milvus-supports-apache-parquet-file-supports.md)  
10. JSON Shredding in Milvus: 88.9x Faster JSON Filtering with Flexibility, fecha de acceso: abril 4, 2026, [https://milvus.io/blog/json-shredding-in-milvus-faster-json-filtering-with-flexibility.md](https://milvus.io/blog/json-shredding-in-milvus-faster-json-filtering-with-flexibility.md)  
11. Filtered Search | Milvus Documentation, fecha de acceso: abril 4, 2026, [https://milvus.io/docs/filtered-search.md](https://milvus.io/docs/filtered-search.md)  
12. How Does the Database Understand and Execute Your Query? \- Milvus Blog, fecha de acceso: abril 4, 2026, [https://milvus.io/blog/deep-dive-7-query-expression.md](https://milvus.io/blog/deep-dive-7-query-expression.md)  
13. Knowhere | Milvus Documentation, fecha de acceso: abril 4, 2026, [https://milvus.io/docs/knowhere.md](https://milvus.io/docs/knowhere.md)  
14. What Powers Similarity Search in Milvus Vector Database?, fecha de acceso: abril 4, 2026, [https://milvus.io/blog/deep-dive-8-knowhere.md](https://milvus.io/blog/deep-dive-8-knowhere.md)  
15. Unleashing AI's Potential: Exploring the Intel AVX-512 Integration with the Milvus Vector Database, fecha de acceso: abril 4, 2026, [https://community.intel.com/t5/Blogs/Tech-Innovation/Artificial-Intelligence-AI/Unleashing-AI-s-Potential-Exploring-the-Intel-AVX-512/post/1567262?profile.language=ko](https://community.intel.com/t5/Blogs/Tech-Innovation/Artificial-Intelligence-AI/Unleashing-AI-s-Potential-Exploring-the-Intel-AVX-512/post/1567262?profile.language=ko)  
16. Conduct a Hybrid Search Milvus v2.2.x documentation, fecha de acceso: abril 4, 2026, [https://milvus.io/docs/v2.2.x/hybridsearch.md](https://milvus.io/docs/v2.2.x/hybridsearch.md)  
17. Conduct a Hybrid Search Milvus v2.3.x documentation, fecha de acceso: abril 4, 2026, [https://milvus.io/docs/v2.3.x/hybridsearch.md](https://milvus.io/docs/v2.3.x/hybridsearch.md)  
18. IVF vs HNSW Indexing in Milvus \- Medium, fecha de acceso: abril 4, 2026, [https://medium.com/@techlatest.net/ivf-vs-hnsw-indexing-in-milvus-ba18ad91e8d3](https://medium.com/@techlatest.net/ivf-vs-hnsw-indexing-in-milvus-ba18ad91e8d3)  
19. Vector Databases Explained in 3 Levels of Difficulty \- MachineLearningMastery.com, fecha de acceso: abril 4, 2026, [https://machinelearningmastery.com/vector-databases-explained-in-3-levels-of-difficulty/](https://machinelearningmastery.com/vector-databases-explained-in-3-levels-of-difficulty/)  
20. How to Cut Vector Database Costs by Up to 80%: A Practical Milvus Optimization Guide, fecha de acceso: abril 4, 2026, [https://milvus.io/blog/how-to-cut-vector-database-costs-by-up-to-80-a-practical-milvus-optimization-guide.md](https://milvus.io/blog/how-to-cut-vector-database-costs-by-up-to-80-a-practical-milvus-optimization-guide.md)  
21. Understanding IVF Vector Index: How It Works and When to Choose It Over HNSW \- Milvus, fecha de acceso: abril 4, 2026, [https://milvus.io/blog/understanding-ivf-vector-index-how-It-works-and-when-to-choose-it-over-hnsw.md](https://milvus.io/blog/understanding-ivf-vector-index-how-It-works-and-when-to-choose-it-over-hnsw.md)  
22. Milvus vs Redis: Vector Database vs Unified Real-Time Platform 2026, fecha de acceso: abril 4, 2026, [https://redis.io/blog/milvus-vs-redis-vector-database-comparison/](https://redis.io/blog/milvus-vs-redis-vector-database-comparison/)  
23. How to Debug Slow Search Requests in Milvus, fecha de acceso: abril 4, 2026, [https://milvus.io/blog/how-to-debug-slow-requests-in-milvus.md](https://milvus.io/blog/how-to-debug-slow-requests-in-milvus.md)  
24. What is Milvus | Milvus Documentation, fecha de acceso: abril 4, 2026, [https://milvus.io/docs/overview.md](https://milvus.io/docs/overview.md)  
25. Bitset | Milvus Documentation, fecha de acceso: abril 4, 2026, [https://milvus.io/docs/bitset.md](https://milvus.io/docs/bitset.md)  
26. Time Travel Milvus v2.2.x documentation, fecha de acceso: abril 4, 2026, [https://milvus.io/docs/v2.2.x/timetravel\_ref.md](https://milvus.io/docs/v2.2.x/timetravel_ref.md)  
27. Basic Operators | Milvus Documentation, fecha de acceso: abril 4, 2026, [https://milvus.io/docs/basic-operators.md](https://milvus.io/docs/basic-operators.md)  
28. How to Use Hybrid Spatial and Vector Search with Milvus, fecha de acceso: abril 4, 2026, [https://milvus.io/blog/hybrid-spatial-and-vector-search-with-milvus-264.md](https://milvus.io/blog/hybrid-spatial-and-vector-search-with-milvus-264.md)  
29. Generating Milvus Query Filter Expressions with Large Language Models, fecha de acceso: abril 4, 2026, [https://milvus.io/docs/generating\_milvus\_query\_filter\_expressions.md](https://milvus.io/docs/generating_milvus_query_filter_expressions.md)  
30. Modify Collection | Milvus Documentation, fecha de acceso: abril 4, 2026, [https://milvus.io/docs/modify-collection.md](https://milvus.io/docs/modify-collection.md)  
31. Milvus Tiered Storage: 80% Less Vector Search Cost with On-Demand Hot–Cold Data Loading, fecha de acceso: abril 4, 2026, [https://milvus.io/blog/milvus-tiered-storage-80-less-vector-search-cost-with-on-demand-hot%E2%80%93cold-data-loading.md](https://milvus.io/blog/milvus-tiered-storage-80-less-vector-search-cost-with-on-demand-hot%E2%80%93cold-data-loading.md)  
32. Vector Databases Compared: Pinecone, Qdrant, Weaviate, Milvus and More, fecha de acceso: abril 4, 2026, [https://letsdatascience.com/blog/vector-databases-compared-pinecone-qdrant-weaviate-milvus-and-more](https://letsdatascience.com/blog/vector-databases-compared-pinecone-qdrant-weaviate-milvus-and-more)  
33. Qdrant vs Milvus: Detailed Comparison \- IngestIQ, fecha de acceso: abril 4, 2026, [https://www.ingestiq.ai/resources/comparisons/qdrant-vs-milvus](https://www.ingestiq.ai/resources/comparisons/qdrant-vs-milvus)  
34. enhance: zero-copy search results to reduce memory allocation and ..., fecha de acceso: abril 4, 2026, [https://github.com/milvus-io/milvus/issues/48668](https://github.com/milvus-io/milvus/issues/48668)  
35. Performance: Slowdown When Increasing Search Limit \#47345 \- GitHub, fecha de acceso: abril 4, 2026, [https://github.com/milvus-io/milvus/discussions/47345](https://github.com/milvus-io/milvus/discussions/47345)  
36. Leveraging Apache Arrow for Zero-copy, Zero-serialization Cluster Shared Memory \- arXiv, fecha de acceso: abril 4, 2026, [https://arxiv.org/html/2404.03030v1](https://arxiv.org/html/2404.03030v1)  
37. Memory and IO Interfaces — Apache Arrow v23.0.1, fecha de acceso: abril 4, 2026, [https://arrow.apache.org/docs/python/memory.html](https://arrow.apache.org/docs/python/memory.html)  
38. Arrow Interop with Zero-Copy Memory Reads | by Yerachmiel Feltzman | Israeli Tech Radar, fecha de acceso: abril 4, 2026, [https://medium.com/israeli-tech-radar/the-apache-arrow-revolution-for-data-solutions-e59bb496c60c](https://medium.com/israeli-tech-radar/the-apache-arrow-revolution-for-data-solutions-e59bb496c60c)  
39. Leveraging Apache Arrow for Zero-copy, Zero-serialization Cluster Shared Memory, fecha de acceso: abril 4, 2026, [https://www.alphaxiv.org/overview/2404.03030v1](https://www.alphaxiv.org/overview/2404.03030v1)  
40. What factors affect search in queue latency? · milvus-io milvus · Discussion \#22075 \- GitHub, fecha de acceso: abril 4, 2026, [https://github.com/milvus-io/milvus/discussions/22075](https://github.com/milvus-io/milvus/discussions/22075)  
41. Embedding Function Overview | Milvus Documentation, fecha de acceso: abril 4, 2026, [https://milvus.io/docs/embedding-function-overview.md](https://milvus.io/docs/embedding-function-overview.md)  
42. Introducing the Embedding Function: How Milvus 2.6 Streamlines Vectorization and Semantic Search, fecha de acceso: abril 4, 2026, [https://milvus.io/blog/data-in-and-data-out-in-milvus-2-6.md](https://milvus.io/blog/data-in-and-data-out-in-milvus-2-6.md)  
43. Deploying Milvus on Kubernetes Just Got Easier with the Milvus Operator, fecha de acceso: abril 4, 2026, [https://milvus.io/blog/deploying-milvus-on-kubernetes-just-got-easier-with-the-milvus-operator.md](https://milvus.io/blog/deploying-milvus-on-kubernetes-just-got-easier-with-the-milvus-operator.md)  
44. Milvus vs Qdrant — which one would you trust for enterprise SaaS vector search? \- Reddit, fecha de acceso: abril 4, 2026, [https://www.reddit.com/r/vectordatabase/comments/1npa1ul/milvus\_vs\_qdrant\_which\_one\_would\_you\_trust\_for/](https://www.reddit.com/r/vectordatabase/comments/1npa1ul/milvus_vs_qdrant_which_one_would_you_trust_for/)

[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABUAAAAYCAYAAAAVibZIAAABC0lEQVR4XmNgGAW0BhxAPA+IHwHxfyB+AcQrgXg2EDsglJEOBIH4JBD/AeIANDmygRYQP2OAuFYZTY5sEMsA8foWBkhwUAVMZoAYWoMuQQkAhecPIHZDl6AEvAXi20Asgy5BCQB5fQ0Qs6JLkAtEgfgzENuhS0DBaiAWAGIdIJ4GxJxAzAjE5UBsjKQOBYAMu8gAMRwdgDRXQNmRDJB07A3lg5IeTA4DgCRwed0aiHWhbEkgLoHSIIDTUJBXtgNxJpq4BBA3APEnBuyWgUAcA5r3nYD4KwMkgvBhkHexAQUgPo0uSAkARdoqBiqmaVBQtAOxGZQfjiRHFgAZCMrO9UAcBsVBKCpGAVUBAKuYL56cdzifAAAAAElFTkSuQmCC>

[image2]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAD0AAAAWCAYAAABzCZQcAAADWElEQVR4Xu2XWahNYRTHl1CEzJGkiwcyPEiGKKWEkiEhKY/3gSRFSJ54UB6kDC8iKRnfDMmUBwlPSFKmMqVMIUUU63fW/s5dZ529Twe37If7r3/7nrW+vb9vzd8V6UAHOvAHmKLsEoVlR2/lXuVrZatygHK48oDyi3KtslN1dRtYc1s5Nfs9T3lG+UP5WDkkk5cOROmJ8oKyX9B1VR5WflUuqlVVdCeV+6TWIfx9VEpu9BvlJWWfqMgwQyzaN6R2zTTly+wZcURKbHSL8r1yUpB7cHAMeKuc4OR7pN4RCaU1mvQ8KPXpGZGM/ilWs2Cg8q6Y4XkoMpo9JyuXZc8i8N5gsfVwlnJdzYpasL6b+90rk3V2sgpIy0/ZsxGILlH+ppwdZCvTooBoNE5doHwq5iiM5onMOxwDd4q9f0ps/S3lduUzt86DnnRd+UA5TrlbrFwPKc9LyMQdyl/Kvl6YgyVi6+jsYzIZEf+ePfMQjV6q/KhcXl1hQIYuYYNY00zYJhaYmcrpTp6Ak/YrF4qd8YVyvtMjQ1cB4b+aCRthhJi3Se3VTr5ZGvcCbzSO4X32Y18PZCmDSM+zYt9NSA7H+DxgNJHkvefKkU5HBtWcsVmjMY41N5X9nXyVNG90ikKR0T4ajEUizYHTuCRtWzJ9ER6JpXQPJxsqYU8+eFoaG019vBIbadSNB4f8kCNP8EYTRaJZZDTjkLEI6C/UIaOQfa8pR2e6RuD7lKvHCmm7VFXvHwhJu+5JEHBZ+VksxSLS7K7WS4A3epDyjlg08L4HMj/2yKytbeqm4ZssIKjHlOPFpgBNsargmrlJ6kfWWOVDKY4ktUMNccgIxsRxsUixKcBxNC2/F893yrnZb9CayTgXXCPmWF9aEfSC6FAcTVkQ0I3K9U4nPcW8dEVsQ3hRbBwVZQBITYcSwXkJqQd4ppSeKHZPvy+WCffEnOsxSkwfv8Fdns6eB7KJqPpz4NBdynNi9xCvq4DUmiM2OyFjqW6o54B6iR5uBuniEOub73Dhiek9THlCLHPygIH+YpKAnH+cYhb/Mxhj1GSj9GsWNE7m7JYgJ0qMLHSlQOoL8aB/Czo1tytuU3yXMsBYZLEU/ivoC9zh2xM0wMViNR7LoDSgB7R77bQnfgO82sG4ZGp86AAAAABJRU5ErkJggg==>

[image3]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAI8AAAAVCAYAAAB/nr22AAAEg0lEQVR4Xu2ZXahVRRiG35AgsTJ/oh9CjpkgBkJIFBHRRZTRRV6UhNBNIKEXJhUa0kVXBUEXUUEQxSGKfkUoK0kzNDFNKZUoqZSDFxaFSYUQGOT3+K2PNWtaa2/PuWrvPS+8HNfMrG9mz7zz/SylgoKCgoKCocAc4zrjNuO3xqeN8xojmrjA+JDxTeP0rO//iDHjCuMq4/xmV8FUMdP4ovGEfGPnyjf3FeNfxrVyoeS42fiVBucg7pEL/Q/jgqyvYAq4yXjUuNU4O+u70DhuPG1c3uw656X2yoU1SEA0x40X5R0Fk8cv8jB1Wd5R4Ta599mj5piVctFdm7QNAvA+/+SNBZPHmPGk8casPcXVxp+MvxlvqNq4tVuMb8m90yDhGePfeWPB5HCLPPbztxcQDMJhw++s2ri9vItXSkFetF4eAgkN5ExLjO8Y3zP+bny8GgceMX5o/MG4SZ7M7jfulnvDq6qxqc1H5V6Q3OWIcalqXC9/92v53Ni+I+lfLM/rJoyvGz82Hq7aA8z3pXwd2GDMR+r2zCMJbuC/xll5R4b75OPY9Njkx9T0RAFC2KvyyovDOWPcVfVxKBw4XgxvBhAUhxJriedrjMeMz8tzlGjHJgd6qXFz9Q5rAazzlHGjcVrVxpzYidBK3kbI2lc9Y2en8QP5mhmPuKkgQ+CM+Vx+YQoMM+Q3m83vBTZwXD5uh/GSqv1JNUUQeND4hHwc48mnwjNEqEvfeyppx6tEBRShkkNGHNiMg0YIHDSe6qA89ALmChGkYO2sC7wgz9/CY8Y6Y27ERXGQhmIKigPqndvxeePyvHFYEZvWTzxsGDeX27omaccDtIkHu4ghwsP7qg8Cb/KjmiJkbFQ/CCgqoAiVh+SfAVKbbTkWgiGs3p13yH9jKj5sxkHzl2fmulX1niA2hLZdLhwE1AVs8A4edyRwvuJh0xlDSU5pHmBj28QTiPAQISVtw2aKtnYOj3lT8cW4NiAshIlAUyA63iHktAkaUZCHISCKBkSMB6Od3xYi7wVsIdyH845hBqU2G5u7+QC37k95LpGDw+2q0sihCDfpDQ8P9ol8PjzEc6pFTF968BNqfj5IbbYBT4U3TEHIJf+5v3om2eeQ762esU3iPWFcqDrPYj05FhmvyxtHGdwYks8N+u/XY6oWKpkud03OQO4QB5Eibvi4aruIheR5tfzQ3pXnQjE2FSLr+k51LgNSm21AkKl3ZF481a/Vv0EIODwcv/tn1RUk875s/L56Bry7zPiZmuspMFwsv41szqqKn8pzgC6PBCJXoErKgaDwaHi2wBWqS3Bue3izqOTeMH5jfE2eY1Cip2izmeMLeQLNhaA6oty+PR0gD18IiOqNdXBJUiDst+XrwJPxG5+V//dNQQvYsLvk31ggtzxK3S5wI19SM/kNcIMRV26D9ivVHJ9WP+QnXXlGl80chLcuG4FYRy9gA8H3m69giiDMTKj/R8ZeyHOjghECXosvsA/kHX2AlyFUEbLIhcgz2pLvgiEH+Qlfe7uS64KCniCPiLK6YIhxFpam+1yx6AbJAAAAAElFTkSuQmCC>