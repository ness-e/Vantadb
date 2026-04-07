# **Análisis de Ingeniería Inversa y Arquitectura de Sistemas de Qdrant para el Desarrollo de ConnectomeDB**

El diseño de sistemas de bases de datos cognitivas, como el proyecto ConnectomeDB, exige una comprensión profunda de las estructuras de datos de alto rendimiento, la gestión de memoria de bajo nivel y la orquestación de concurrencia en lenguajes de programación de sistemas como Rust.1 Qdrant representa un estado del arte en la convergencia de la búsqueda vectorial y el filtrado de metadatos, alejándose de las implementaciones tradicionales tipo "wrapper" para construir un motor nativo optimizado desde los primeros principios.3 Este informe detalla los mecanismos internos de Qdrant, analizando su motor de almacenamiento Gridstore, su implementación de grafos HNSW filtrables y su sofisticado modelo de gestión de segmentos.

## **Arquitectura de Almacenamiento Segmentada y el Motor Gridstore**

La arquitectura de Qdrant se fundamenta en la segmentación de datos, donde cada colección se divide en unidades independientes denominadas segmentos.4 Esta decisión arquitectónica es crítica para permitir la escalabilidad horizontal y la optimización en caliente sin interrupciones del servicio. Cada segmento funciona como una base de datos en miniatura, poseyendo su propio almacenamiento de vectores, almacenamiento de carga útil (payload), índices y mappers de identidad.4

### **El Motor Customizado Gridstore**

Para superar las limitaciones de latencia y las pausas de compactación inherentes a motores de propósito general como los basados en Log-Structured Merge-trees (LSM-trees), Qdrant implementó Gridstore.3 Este motor está optimizado para las características específicas de los datos vectoriales: identificadores secuenciales internos y valores de tamaño variable.5

La arquitectura de Gridstore se desglosa en tres capas funcionales que garantizan el acceso en tiempo constante y la reutilización eficiente del espacio:

| Capa | Responsabilidad Técnica | Mecanismo de Implementación |
| :---- | :---- | :---- |
| Capa de Datos | Almacenamiento físico de valores. | Estructura de bloques de tamaño fijo (típicamente 128 bytes) organizados en archivos de página. 5 |
| Capa de Máscara | Control de ocupación de bloques. | Uso de bitmasks para rastrear la disponibilidad de cada bloque, permitiendo eliminaciones lógicas rápidas. 5 |
| Capa de Gaps | Gestión de la fragmentación. | Índice de nivel superior que identifica rangos de bloques libres para optimizar la asignación de nuevas inserciones. 5 |

El diseño de Gridstore permite que la recuperación de un valor basado en su ID interno sea una operación de complejidad ![][image1]. Dado que los IDs internos son enteros secuenciales, el sistema utiliza un array de punteros (Tracker) donde la posición del ID corresponde directamente a la ubicación del puntero que referencia el inicio del dato en el grid de bloques.5 Este enfoque elimina la necesidad de recorrer estructuras de árbol para búsquedas por clave, una ventaja significativa para los procesos de re-puntuación (rescoring) y recuperación de metadatos en ConnectomeDB.

### **Gestión de Segmentos y Estrategias de Optimización**

La mutabilidad en Qdrant se gestiona mediante un modelo híbrido de segmentos añadibles (appendable) y no añadibles (non-appendable).4 Mientras que al menos un segmento debe permitir la escritura, los segmentos más grandes suelen transformarse en estructuras de solo lectura para maximizar la eficiencia del índice HNSW.4 El proceso de optimización es orquestado por un sistema de proxies que permite que un segmento siga siendo legible mientras se reconstruye una versión más eficiente del mismo en segundo plano.6

Existen tres tipos principales de optimizadores que mantienen la salud del sistema:

1. **Optimización de Vacío (Vacuum):** Identifica segmentos con un alto porcentaje de registros eliminados (basado en el deleted\_threshold) y compacta el espacio físico para liberar recursos de memoria y disco.6  
2. **Optimización de Mezcla (Merge):** Combina múltiples segmentos pequeños en uno solo para reducir la sobrecarga de búsqueda. El sistema intenta mantener un número de segmentos cercano al número de núcleos de CPU disponibles para maximizar el paralelismo de las consultas.6  
3. **Optimización de Indexación:** Determina el momento en que un segmento ha acumulado suficientes datos (según el indexing\_threshold\_kb) para justificar la construcción de un índice HNSW y el paso a almacenamiento mapeado en memoria (mmap).6

## **Implementación de Grafos HNSW y Navegación Topológica**

El núcleo de la búsqueda de similitud en Qdrant es el algoritmo Hierarchical Navigable Small World (HNSW), que permite realizar búsquedas de vecinos más cercanos aproximados (ANN) con una escalabilidad sublineal.7 Para ConnectomeDB, entender cómo Qdrant extiende este grafo para soportar filtros es esencial.

### **Dinámica del Grafo Jerárquico**

HNSW organiza los vectores en una estructura de capas múltiples. La capa superior es la más dispersa, permitiendo saltos rápidos a través del espacio vectorial, mientras que las capas inferiores aumentan su densidad hasta llegar a la capa base, donde se encuentran todos los puntos.7 La navegación comienza en un punto de entrada en la capa superior y desciende progresivamente hacia las regiones de mayor densidad.8

Los parámetros que gobiernan la construcción y búsqueda en este grafo son fundamentales para el equilibrio entre precisión y latencia:

| Parámetro | Definición Técnica | Impacto en el Sistema |
| :---- | :---- | :---- |
| **![][image2]** | Número máximo de conexiones por nodo. | Aumentar ![][image2] mejora la precisión (recall) pero incrementa linealmente el uso de memoria RAM (proporcional a ![][image3]). 8 |
| ![][image4] | Candidatos evaluados durante la inserción. | Define la calidad del grafo; valores más altos resultan en mejores conexiones pero aumentan el tiempo de indexación. 8 |
| ![][image5] | Candidatos evaluados durante la búsqueda. | Parámetro dinámico que ajusta la profundidad de la exploración durante la consulta. 8 |

### **Compresión mediante Delta Encoding y Almacenamiento Inline**

En la versión 1.13, Qdrant introdujo la codificación delta para los enlaces del grafo HNSW.9 Dado que los IDs de los vecinos cercanos suelen ser numéricamente próximos en un sistema de almacenamiento secuencial, almacenar solo la diferencia (delta) entre IDs permite una reducción del 30% en el consumo de memoria del índice sin introducir latencia de descompresión significativa.9

Además, la técnica de almacenamiento inline (introducida en v1.16) permite incrustar vectores cuantizados directamente en los nodos del grafo HNSW.10 Este enfoque optimiza las operaciones de E/S en disco, ya que una sola lectura de página (4KB) puede recuperar tanto los IDs de los vecinos como sus representaciones vectoriales comprimidas, eliminando múltiples lecturas aleatorias durante el recorrido del grafo.10

## **Micro-Ingeniería de la Cuantización y Rendimiento SIMD**

La cuantización en Qdrant no es solo una técnica de compresión, sino una estrategia de aceleración de cómputo. Al reducir la precisión de los vectores, se habilita el uso de instrucciones SIMD (Single Instruction, Multiple Data) que pueden procesar múltiples dimensiones en un solo ciclo de reloj de la CPU.11

### **Variantes de Cuantización y sus Trade-offs**

Qdrant implementa tres familias principales de cuantización, cada una con características específicas de rendimiento y fidelidad:

1. **Cuantización Escalar (SQ):** Convierte valores float32 a uint8 analizando la distribución de los datos y mapeando linealmente el rango a 256 niveles.11 Reduce el uso de memoria en un factor de 4 y es compatible con la mayoría de los modelos de embeddings comerciales.12  
2. **Cuantización Binaria (BQ):** Representa cada dimensión como un solo bit (1 para valores positivos, 0 para negativos).11 Logra una compresión de 32x y una aceleración de hasta 40x mediante el uso de operaciones XOR y Popcount a nivel de hardware.11 Es especialmente efectiva para vectores de alta dimensionalidad (![][image6]) como los generados por modelos tipo OpenAI o Cohere.12  
3. **Cuantización de Producto (PQ):** Divide el vector en sub-vectores y cuantiza cada segmento de manera independiente utilizando centroides aprendidos mediante k-means.11 Es la técnica de compresión más agresiva pero requiere una calibración cuidadosa del modelo.11

La implementación soporta también precisiones intermedias como 1.5 bits y 2 bits, que mitigan la pérdida de información en vectores de dimensiones medias (512-1024).11

| Método de Cuantización | Factor de Compresión | Aceleración de Búsqueda | Casos de Uso Recomendados |
| :---- | :---- | :---- | :---- |
| Scalar (int8) | 4x | \~2-3x | Propósito general, alta fidelidad. 11 |
| Binary (1-bit) | 32x | Up to 40x | Vectores muy grandes, baja latencia extrema. 12 |
| Binary (2-bit) | 16x | \~20-30x | Equilibrio entre BQ y SQ para dimensiones medias. 11 |

### **Rescoring Asimétrico y Almacenamiento Dual**

Para mantener una alta precisión a pesar de la cuantización, Qdrant utiliza una estrategia de re-puntuación.12 Los vectores cuantizados se mantienen preferiblemente en RAM para una búsqueda rápida de candidatos, mientras que los vectores originales se almacenan en disco (on\_disk=true). Una vez identificados los ![][image7] candidatos más cercanos mediante la versión comprimida, el sistema recupera los vectores originales para realizar un cálculo de distancia exacto.12

## **Filtrado Avanzado y el Algoritmo ACORN**

Qdrant resuelve el problema del filtrado en la búsqueda vectorial mediante un enfoque de una sola etapa, donde las condiciones de filtrado se evalúan durante el recorrido del grafo HNSW, evitando el pre-filtrado (que puede llevar a un escaneo completo de la colección) y el post-filtrado (que puede reducir el número de resultados finales por debajo del límite solicitado).3

### **La Lógica de ACORN-1**

Cuando se aplican múltiples filtros con baja selectividad (donde pocos puntos cumplen los criterios), el grafo HNSW puede fragmentarse, impidiendo que el algoritmo encuentre rutas hacia los vecinos relevantes. El algoritmo ACORN (introducido en v1.16) extiende la búsqueda permitiendo explorar no solo los vecinos directos de un nodo, sino también los vecinos de sus vecinos (segundo salto) si los primeros han sido filtrados.10

Los resultados de benchmarks demuestran la eficacia de esta técnica:

| Configuración de Filtro | Precisión (Recall) con HNSW Estándar | Precisión (Recall) con ACORN | Latencia Adicional |
| :---- | :---- | :---- | :---- |
| Alta Selectividad (Densa) | 90%+ | 90%+ | Mínima |
| Baja Selectividad (4% de puntos) | \~53% | \~97% | Significativa (10x+) |

Análisis basado en el dataset deep-image-96 con dos campos de metadatos filtrados.10

Esta capacidad es de vital importancia para ConnectomeDB, donde las consultas lógicas inspiradas en LISP podrían generar filtros altamente complejos y selectivos sobre la estructura del grafo cognitivo.

## **Concurrencia y Sistemas en Rust: El Modelo de Qdrant**

Como sistema escrito en Rust, Qdrant aprovecha las abstracciones de "Fearless Concurrency" para gestionar el paralelismo sin los riesgos de seguridad de memoria tradicionales.20

### **Estructuras de Datos y Sincronización**

El motor evita la contención de bloqueos mediante el uso de estructuras de datos diseñadas para el acceso concurrente masivo. En lugar de proteger estructuras globales con un Mutex, que serializaría todas las operaciones, Qdrant emplea:

* **ArcSwap para Lecturas Livianas:** Para configuraciones y metadatos que cambian con poca frecuencia, se utiliza ArcSwap. Este patrón permite que los hilos lectores obtengan una referencia instantánea (Arc) sin incrementar contadores de referencia globales, eliminando la contención en el bus de memoria en sistemas con más de 100 núcleos.22  
* **DashMap para Sharding de Locks:** Las estructuras internas de gestión de segmentos utilizan mapas fragmentados que distribuyen el riesgo de contención entre múltiples bloqueos más pequeños.21  
* **AtomicPointers y Epoch-based Reclamation:** Para la gestión de memoria en estructuras lock-free, Qdrant utiliza técnicas de recolección de basura basadas en épocas (vía crates como crossbeam-epoch), garantizando que la memoria de un nodo eliminado no se libere hasta que todos los lectores activos hayan terminado su operación.20

### **Async I/O y Rayon para Paralelismo de Datos**

Qdrant utiliza un modelo de ejecución dual. Para la gestión de peticiones de red y orquestación de tareas, emplea un runtime asíncrono basado en Tokio.23 Sin embargo, para las tareas computacionalmente intensivas como el cálculo de distancias vectoriales y la construcción del grafo HNSW, utiliza Rayon. Rayon implementa un algoritmo de robo de trabajo (work-stealing) que garantiza que todos los núcleos de CPU disponibles se utilicen de manera eficiente sin la sobrecarga de la gestión manual de hilos.23

A partir de versiones recientes, el sistema también soporta io\_uring para realizar operaciones de E/S de disco asíncronas, lo que es crucial cuando los vectores e índices se almacenan en mmap sobre discos NVMe rápidos.26

## **Motor de Inferencia y Lógica de Consulta: Recomendación y Fusión**

Qdrant no se limita a la búsqueda de similitud punto a punto; integra una lógica de consulta rica que permite expresar intenciones complejas.

### **El API de Recomendación: Estrategias de Puntaje**

El API de recomendación permite realizar búsquedas basadas en múltiples ejemplos positivos y negativos.27 Qdrant implementa dos estrategias para resolver este problema:

1. **Average Vector:** Calcula un centroide basado en los ejemplos. La fórmula técnica aplicada es: ![][image8] Donde ![][image9] son ejemplos positivos y ![][image10] negativos. Esta estrategia es eficiente porque reduce la recomendación a una única búsqueda vectorial estándar.28  
2. **Best Score:** En lugar de promediar, mide la similitud de un candidato contra cada ejemplo por separado. El puntaje final se determina eligiendo el máximo entre los positivos y penalizando si la proximidad a un negativo es mayor.28 Se utiliza una función sigmoidea para normalizar los resultados: ![][image11] 29

### **Búsqueda Híbrida y Fusión de Rango Recíproco (RRF)**

Para combinar la búsqueda semántica densa con la búsqueda por palabras clave dispersa (como SPLADE o BM25), Qdrant utiliza la Fusión de Rango Recíproco (RRF).3 RRF permite fusionar listas de resultados de diferentes modelos sin necesidad de normalizar los puntajes originales: ![][image12] Donde ![][image13] es una constante (típicamente 60\) que suaviza el impacto de los resultados en posiciones bajas.15

## **Gestión de Memoria y el Componente IdTracker**

En despliegues de gran escala, el consumo de memoria RAM está dominado por el IdTracker, que mapea los identificadores externos (UUIDs o enteros de 64 bits) a los IDs secuenciales internos.16

Para una escala de 400 millones de vectores, el análisis de ingeniería inversa revela los siguientes requisitos de memoria residente:

| Componente del IdTracker | Estructura de Datos | Consumo por Punto | Total (400M Puntos) |
| :---- | :---- | :---- | :---- |
| Versiones de Puntos | u64 (comprimido) | 4 bytes | 1.5 GB |
| ID Interno a Externo | Vec\<u128\> | 16 bytes | 6.4 GB |
| ID Externo a Interno | Mapping Mixto | \~12 bytes | 4.8 GB |

Nota: Optimizaciones introducidas en v1.13.5 redujeron el consumo total de este componente en un 50%, permitiendo manejar 400M de puntos con aproximadamente 12.4 GB de RAM residente.16

Es vital distinguir entre esta memoria residente (Data Memory), que el proceso debe mantener para evitar fallos, y la Cache Memory (mmap), que el sistema operativo puede desalojar según la presión de memoria.16

## **Despliegue Distribuido y Estabilidad del Sistema**

Qdrant escala mediante el uso de fragmentación (sharding) y replicación. Cada shard es un almacén de puntos independiente que puede distribuirse entre múltiples nodos.18

### **El Write-Ahead-Log (WAL) y Consistencia**

Para garantizar la durabilidad ante fallos de alimentación, cada operación se registra primero en el Write-Ahead-Log (WAL) antes de aplicarse a los segmentos.6 El WAL asigna números secuenciales a cada operación, lo que permite la recuperación del estado tras un reinicio inesperado. Si una operación llega con un número secuencial inferior a la versión actual del punto en el segmento, se ignora, garantizando la idempotencia.6

### **Desafíos de E/S y Compatibilidad POSIX**

Un hallazgo crítico en la ingeniería inversa de Qdrant es su dependencia de semánticas POSIX estrictas. Se han identificado fallos graves de integridad de datos cuando se ejecutan contenedores Qdrant sobre sistemas de archivos no compatibles, como montajes directos desde Windows (NTFS) a través de WSL o sistemas FUSE.32 El uso de mmap sobre estos sistemas de archivos puede provocar pánicos en el motor Gridstore debido a fallos en la persistencia de las páginas de memoria mapeadas.33

## **Conclusiones Técnicas para el Desarrollo de ConnectomeDB**

El análisis técnico de Qdrant proporciona un plano detallado para ConnectomeDB. Las lecciones clave incluyen:

1. **Prevalencia de la Inmutabilidad:** La separación de datos en segmentos inmutables permite optimizar los grafos HNSW de forma agresiva, una técnica que ConnectomeDB debería adoptar para manejar estados de memoria cognitiva estables versus dinámicos.  
2. **Cuantización como Cómputo:** La cuantización no es solo para ahorrar espacio, sino para transformar cálculos de punto flotante costosos en operaciones de bits ultrarrápidas.  
3. **Filtrado en una Sola Etapa:** La integración de filtros lógicos dentro del recorrido del grafo (ACORN) es superior a cualquier estrategia de pre o post-filtrado para mantener el recall en consultas complejas.  
4. **Soberanía de Rust:** El uso de crates para concurrencia lock-free y el aprovechamiento de mmap demuestran que ConnectomeDB debe priorizar la gestión de memoria de bajo nivel para alcanzar un rendimiento de grado de producción.

Este análisis confirma que Qdrant es un motor de búsqueda de alto rendimiento diseñado para la era de la inteligencia artificial, cuya arquitectura modular y enfoque en la eficiencia de hardware lo convierten en la referencia técnica más relevante para el desarrollo de bases de datos inspiradas en la neurobiología. 1

#### **Obras citadas**

1. Deep Dive into Qdrant Vector Database Agents \- Sparkco, fecha de acceso: abril 3, 2026, [https://sparkco.ai/blog/deep-dive-into-qdrant-vector-database-agents](https://sparkco.ai/blog/deep-dive-into-qdrant-vector-database-agents)  
2. llms-full.txt \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/llms-full.txt](https://qdrant.tech/llms-full.txt)  
3. Qdrant \- Vector Search Engine, fecha de acceso: abril 3, 2026, [https://qdrant.tech/](https://qdrant.tech/)  
4. Storage \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/documentation/manage-data/storage/](https://qdrant.tech/documentation/manage-data/storage/)  
5. Introducing Gridstore: Qdrant's Custom Key-Value Store, fecha de acceso: abril 3, 2026, [https://qdrant.tech/articles/gridstore-key-value-storage/](https://qdrant.tech/articles/gridstore-key-value-storage/)  
6. Optimizer \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/documentation/operations/optimizer/](https://qdrant.tech/documentation/operations/optimizer/)  
7. What is a Vector Database? \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/articles/what-is-a-vector-database/](https://qdrant.tech/articles/what-is-a-vector-database/)  
8. HNSW Indexing Fundamentals \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/course/essentials/day-2/what-is-hnsw/](https://qdrant.tech/course/essentials/day-2/what-is-hnsw/)  
9. Qdrant 1.13 \- GPU Indexing, Strict Mode & New Storage Engine, fecha de acceso: abril 3, 2026, [https://qdrant.tech/blog/qdrant-1.13.x/](https://qdrant.tech/blog/qdrant-1.13.x/)  
10. Qdrant 1.16 \- Tiered Multitenancy & Disk-Efficient Vector Search, fecha de acceso: abril 3, 2026, [https://qdrant.tech/blog/qdrant-1.16.x/](https://qdrant.tech/blog/qdrant-1.16.x/)  
11. Quantization \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/documentation/manage-data/quantization/](https://qdrant.tech/documentation/manage-data/quantization/)  
12. Vector Quantization Methods \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/course/essentials/day-4/what-is-quantization/](https://qdrant.tech/course/essentials/day-4/what-is-quantization/)  
13. Vector Search Resource Optimization Guide \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/articles/vector-search-resource-optimization/](https://qdrant.tech/articles/vector-search-resource-optimization/)  
14. Binary Quantization \- Andrey Vasnetsov | Vector Space Talks \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/blog/binary-quantization/](https://qdrant.tech/blog/binary-quantization/)  
15. Hybrid Queries \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/documentation/concepts/hybrid-queries/](https://qdrant.tech/documentation/concepts/hybrid-queries/)  
16. Large-Scale Search \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/documentation/tutorials-operations/large-scale-search/](https://qdrant.tech/documentation/tutorials-operations/large-scale-search/)  
17. Optimizing Memory for Bulk Uploads \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/articles/indexing-optimization/](https://qdrant.tech/articles/indexing-optimization/)  
18. Qdrant Overview, fecha de acceso: abril 3, 2026, [https://qdrant.tech/documentation/overview/](https://qdrant.tech/documentation/overview/)  
19. Search \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/documentation/concepts/search/\#filtering](https://qdrant.tech/documentation/concepts/search/#filtering)  
20. How to Build a Lock-Free Data Structure in Rust \- OneUptime, fecha de acceso: abril 3, 2026, [https://oneuptime.com/blog/post/2026-01-30-how-to-build-a-lock-free-data-structure-in-rust/view](https://oneuptime.com/blog/post/2026-01-30-how-to-build-a-lock-free-data-structure-in-rust/view)  
21. Fearless Concurrency Ep.7: Lock-Free Structures and Channels for Scalable Rust Code, fecha de acceso: abril 3, 2026, [https://www.ardanlabs.com/blog/2024/12/fearless-concurrency-ep7-lock-free-structures-and-channels-for-scalable-rust-code.html](https://www.ardanlabs.com/blog/2024/12/fearless-concurrency-ep7-lock-free-structures-and-channels-for-scalable-rust-code.html)  
22. Concurrency Deep Dive: Memory Models, Lock-Free, and RCU \- DEV Community, fecha de acceso: abril 3, 2026, [https://dev.to/kanywst/concurrency-deep-dive-memory-models-lock-free-and-rcu-11mp](https://dev.to/kanywst/concurrency-deep-dive-memory-models-lock-free-and-rcu-11mp)  
23. Rust Concurrency: 10 Patterns Beyond Locks | by Nexumo \- Medium, fecha de acceso: abril 3, 2026, [https://medium.com/@Nexumo\_/rust-concurrency-10-patterns-beyond-locks-e1598e78e65e](https://medium.com/@Nexumo_/rust-concurrency-10-patterns-beyond-locks-e1598e78e65e)  
24. Announcing aarc 0.1.0 \- atomic variants of Arc and Weak for easy lock-freedom : r/rust, fecha de acceso: abril 3, 2026, [https://www.reddit.com/r/rust/comments/1bilk82/announcing\_aarc\_010\_atomic\_variants\_of\_arc\_and/](https://www.reddit.com/r/rust/comments/1bilk82/announcing_aarc_010_atomic_variants_of_arc_and/)  
25. cool-japan/scirs: SciRS2 \- Scientific Computing and AI in Rust \- GitHub, fecha de acceso: abril 3, 2026, [https://github.com/cool-japan/scirs](https://github.com/cool-japan/scirs)  
26. Performance Tuning \- Qdrant \- Mintlify, fecha de acceso: abril 3, 2026, [https://mintlify.com/qdrant/qdrant/operations/performance-tuning](https://mintlify.com/qdrant/qdrant/operations/performance-tuning)  
27. Recommendation Engines: Personalization & Data Handling \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/recommendations/](https://qdrant.tech/recommendations/)  
28. Deliver Better Recommendations with Qdrant's new API, fecha de acceso: abril 3, 2026, [https://qdrant.tech/articles/new-recommendation-api/](https://qdrant.tech/articles/new-recommendation-api/)  
29. Explore \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/documentation/search/explore/](https://qdrant.tech/documentation/search/explore/)  
30. Demo: Implementing a Hybrid Search System \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/course/essentials/day-3/hybrid-search-demo/](https://qdrant.tech/course/essentials/day-3/hybrid-search-demo/)  
31. Collections \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/documentation/manage-data/collections/](https://qdrant.tech/documentation/manage-data/collections/)  
32. Troubleshooting \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/documentation/operations/common-errors/](https://qdrant.tech/documentation/operations/common-errors/)  
33. Panic occurred in gridstore.rs: OutputTooSmall { expected: 4, actual: 0 } causing collection instability · Issue \#6758 \- GitHub, fecha de acceso: abril 3, 2026, [https://github.com/qdrant/qdrant/issues/6758](https://github.com/qdrant/qdrant/issues/6758)  
34. Payload \- Qdrant, fecha de acceso: abril 3, 2026, [https://qdrant.tech/documentation/manage-data/payload/](https://qdrant.tech/documentation/manage-data/payload/)

[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACgAAAAYCAYAAACIhL/AAAACUElEQVR4Xu2WPWgVQRSFj4igaPwHk2CRaKc2KkGxsEihRYixCClS2FloIdj4U1naKigEgvCwUBTRRlCUBNKEIEEQq0CiIiQIYqEiWAh6DncHZ29m523is3sffLy8ubOz992ZvRugTZsVsYaeoud9IMNheoNu8oEqttBbdImepTthNx6n3+mF4rvnKH1Fe32goJveoxd9gKyDrT9W/F3JEbpAn9PtLqYLG/QHPV0OYQedgSUfc4g+oO9h1/2mV0oz/nIAdu8TPhDzib6kW32g4DisitMozxmFLb4nGhO7aT9sF64in6B25TZ95gOBHvqF9rnxGG3TPP1MD0bjT+l95LdHieUSFNoZrb0MLXwH9gtS5ysQEvxFB6Jxnddz0fcUdRLcSz/6QXGMfi0+c6hq+oU/UT4rulbbn6NOgpvpFBJFug67eJsPOIZh81SxfdG43/IUdRLUTj6iHfHgRtiDoYtz6Fc1YPMmUV5E267tz1EnQXGXdsYDupFu2CzBcAO1E7WVmFYnWForlLVZgouwNqRe6XkHayk5VpKgzmIJ9TE9mRt8oEBN+xvsDKZo1p5EnQTDbq73gfCquYzlT9B+Oot05QJq3kN+MEJrXoMlqE9/j4DO3pwfDOhlrfYxAXsHyxewJ/RSNC/FG1gn8Kiqqq4S86YqriLoGFWi19dJOlKoVrK2NCONGrx/sleDtv+tH2wFqsYHNG/0OVQcveNzZ3TV6EzdpA+Rfx/nOENf010+0Eq66BPkHyiPKveYDvrA/0IVrPp3LYVayr+e3TaV/AHmpnfzu/gzAQAAAABJRU5ErkJggg==>

[image2]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA8AAAAWCAYAAAAfD8YZAAABC0lEQVR4Xu2SsUpDQRBFbwiBBBE0CIpYJZ1gaWORH7DQH7CzFkQQ/YLUkioEgo2dhYWdEAjYiK0IaUIQERsLsbGw8MybHVwkf5BcOPD2vtndufOeNNcsqQyrsJLWFVhL2HNoXV5n9YVK0IYbeIVdeIQ+3MMzNOEIevAEd7Bkm7fgSn7iIBXU7AXahg+YwHny9uBHfokW5K214CsVhA7khfuZ14E32Mw8ncB3trY4l/Aib9u0DA/yDheTV9x8DeMwNL0wujuTH17IpjqC2zD0l9eGGbKW3+Vz2ggzPzE0La91MZRP+jTMQ/iEnTBQVx6jkXnH8hnYZ70I0zLXleVAVXnu/zLPfqa8dmb0C1QALcux3wUbAAAAAElFTkSuQmCC>

[image3]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADMAAAAWCAYAAABtwKSvAAABtElEQVR4Xu2VPSiFURjHH0lR5CPfmUjJYFAog8FgkVAGs1kWktnCYhIpGxnEIF/FYDAQRpNsBhMGUorB/99zXj335N73cu8d5PzqV/d9zrnvPc+5z3mOSCAQCAT+GaWw2A96VMMiP5gJebAS5nvP9bA8muQ++7FUdMFT2OwPOAbgkeg7swIXPg0P4Cpsg8dwE27BJzgFd+EOXIcPcAUWSDxM6Aq2e/EReAjrvHhGNIouugy+i+5kgxtjohvwWRJ3dw3ew1YTSwUTOYed7nlUNJGKrxlZogQWwia4LYm7zaRu4YmJscQunOmWW0QLvBHduJwyBCe9WB98g3Mm1gNf4KKJpQM3aRbuiZZYTuHiuFDLDPyA/SbGeUyGc9kwWIpxMJEF0TPJzsVDz1JL57s/JiqdKhNj6e3DO9ESJLXwGp6Jlkq3xDcBji87o7k8KzwzY5KDhHiQeaDti5kAE2FCTIx0wEfRsuPCllw8GfwX+G+wvPykuRlsLhOS5YQGRcvJ0gtfRX8sgs2CLfxStH2Pm7HvGBZt69H95cMLdV6S30O/grtmS4xwAfYitfEa0cQCgcAf5hPxcj6bLxh6jQAAAABJRU5ErkJggg==>

[image4]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAGIAAAAWCAYAAAA7FknZAAAEiklEQVR4Xu2YXYiWVRDHJ0Qo/E5FEYlEBUm98CPCLhQkxBDJC/UiNKIbCcQLifRCFMQQuhDFFEGMxRvXSgTJREstFS2UyAIlEoW0DzAVFSEwyP/PeWbf8559HnbdTduV5w9/2PfMOWfOmZkzM8+a1ahRo0aNGo8Hh8WPxQ/F++LSZnGvRz+xbz7YE/G3uEI8Ld4TZzeLezUmib+Jq3NBT8RN8RVxgDgkk/V2vCv+Ky7MBT0NGP9ncWQueErwmfi7+FIueFLAwKPEEWKfTJZitPi5+Gwu6CI60kuuxunMyV9frI2zdLRXgD3TdSkIsmPme6Vgv1Q/f+d7MAfdw8RnkvFOgQVnxLPiTvEL8aA4OJ1k/mQpzDzb4A1xRjrpETFR/N5cL5H4nfhiIXtOfF+8Kn5q3hz8IvYv5KTGE+Ip8YK4WfyymHfJ/B75Hbjr2+Jl83nnxTcK2W5rvhtEH4GH4z4S/xLXmjcpOItznWOxua69Yqt4TXyvGAfLxD/EKckYZ3kt/cGCd4q/wUDxuDgvJmXAIf9Fl0QOvmUeRRh9n/nlwzAYEgOPL36DyeJ6c8NsL34zn3Xzk3nsne4VmGXupKnmDv/B3AEp/rH2d39V3GVufOSrxOHmQcRvbIdzFhXjOJi5gBdDBgmngrD7HX7w9JjMgTHsVvErcw8TbWWgrSPqeJJdBQfbb955vZ7JAnPEQ+YOSjHW3JAvmEdgXPLXZA6XbDF/rS8n4wAncl9e9jpxULO4Mu2iC+dz5o2ZDCBjDrq3ibfNnQe4C+vSLgwH/mkeSA8vxQUYwPAYN8+LOTgozxqHdBUUQYphGiE5CIqy9pFLEQihn/Xsw1ggxsryPLKvzaMYh5DahiZy9l+T/M6BQZlTBWrZT+ZtfaRFHJev43fb+YgWoiaeUGcwU7ybDz4iiBQipizyAKmKXFt2YS7F90sgoi2N0jfNDc08IjQ+zEhx5GjG0MvLz18N+4Re5lGQo+izhiCsCh6QnycySAQde2B8goBge4iI7jJHTBDH5YPmKYxNugOMcd3a52YQesnHeVoZY97RpAU4jzaMvscaz54IJWeH80lrkVapi2QEMgOIVB2Gpp5wjnAk86qCJxD1KmpTZJ1Y94G4wDxQ2l48CnaIF2PAPArmiket0b2kwIt8zHUH5P0D4rfWSAtE3RLxG3O9GPatQgaeNy/mFNdA1Ic0xdE6UuAZj64LUHxpDJZboykhUrckv2lS0I9D0EdHNK2QAc5Uli5ToAcjRzOD4XEM6Y67EnwEB/f4xJJ/oxBdreatHJOo9lT/vIiBeGbM6S4w3ElrdC0Yb6019GIc6gjGQI6x83MR2UQ4LyAuxLpN4hXzFpzCCXAK0f2jNe5K55M2A9HJsN8Ra9+w4IQowFWI4EY/elaa6+VV4GS6KkDXxgNgrAmd+QiKVNaSjQcowos7YP61Sg6u+vjhUkRP1blYw9o8VcR42b7MjbtWAZ1tkZqAtfl+VYiPPRDnST8G0/FOIf7vwkvYYOWdSI0nAPIiRa7F/MubJ1XjfwLPZ7q1TwE1ajw9eADwEfArphmUlgAAAABJRU5ErkJggg==>

[image5]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEMAAAAWCAYAAACbiSE3AAADHElEQVR4Xu2WS8iNYRDHR1LkfsslSuxcNsLCAqXIioVbYmMnIrKQWGChsHMpib5ckkshlwWlRGShiBJCCQu5hJSi+P++eeec5zzezznn257zr3+dZ55555mZZ2aeY9ZGG2200T1MFd+IP8V52V5L4qb4QhyTb7QiqIzzYq98oxXxW9ySC1sV38VZYm9xtDhC7Fmj4Wvkw7I1+v1DqQugw3c9Mjk2yuQp8KnMn2ZAxY809wN7XWKA+EzcK94Sj4hPxIeJDsYOiFfFj+Jcc93j4mXxs7i4ol3FKPG6uU30Llk1cYPMW/OtuLmQAc46IQ4xd/yCeeWuSHSawTHxpbmv2H0qzqjRSDBR/CGutuoNLRT/VDTMZpona6D5sD1tHgwgmSQGeQqCOiseNrdLwB/E6cV6n7hcfGT+bSRpgvkMIxHYOGjdb2OCPin2S2TbxKPJugZs4GCK3eZPbYDAcYynN5UDvv0kdmTy8eIr86RSAXOsmmxsYZP1V/NkBzibtg2QpIv2r4/1QPI4e5m4SzwjPhfXWxctN1i8Lw5PZDh51/zGcnAAt1QmW5TJCXSteQvh1C9xZY2G9zFnRZUReDzzAZ57KgxfGwXJ5gI4d765jXqzqXKrqRK3xG3tt2pFAMr2inkJByJxj80DC4f7mFcCbYXtHeatSN+moNKohEC0CIEEFojbk3UjiKQSW8NYZbWzAZCEKF36jXIH4SgJCUTi6GuCXmOePGYKdqPPh4oPrDZwwGxKB29cDn0NsHVInFLRaAz4csrKk8EFpm3ZCT7oEN8nssgorcMtM+SiamJepINsQyFjb5x5OfYVb4i3izVYYj7RJxfrAE7tLH5zzlbzJBII4Ls9xV6zoG2/ZLKx4jUrefmixPNXYKP51OcpjF4GJCEfdrxEr82fz3uJfLZ53zP4eFbviJOS/QA3/828LXh2qYJN5nOGgcezmPrQDEjgOvGcuQ/ESkxlfnSC2y/7E4I8/lwF0CsbQvFHLbeDHvrs/Q/xfWqbJJWdRfKX1iE6KbBd5l8bCaLt+cPZ8mB+vTNvozbMH4xpubCl8RdaTJrZOH/DSgAAAABJRU5ErkJggg==>

[image6]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADsAAAAXCAYAAAC1Szf+AAACfElEQVR4Xu2WX2iPURjHHy0XyjLcSMNaKeVClCyWK+1Ga2stsVpJKa4JF1JI2vXiBrV2tbIsMpKSuHG5UpM/IVwofy5IUYrv13Mev/M+O7+39/31W9Leb33a7zzn7Lzne85z/ohUqlTpf9RSsM4Hm6BFoBs8BJtdnWkjuA/egVeh7LUEHAO/wEdwGazJtJirNnDXB6kV4ALYBxa7ukZ1EHwAP8AnsDVb/UfbwBtwQHRiyNsQN7WC6+C06NjY7xfw3rWLxX6Og5++ItYp8Fy04TJXV1ZdYDW4J2mzNMG626IrZ/KxIdEJ6AllGjknuspsy368+O0X4Luv8OJH9oNZcBGszdSWkxlKmd0JvoJxF2c5bn9C1NjTvy10RT+LZo7fHivBNDgs2k8htYBe8AjcEp3Rssozy8HQBM3EOh/iw6HcJ5q2NGBiX+zT98sxngVHozalxA42iZreJToJRZVn1kx5s7aSPh6rX3Q/PgarojhT/apodjZk1sR0ngBPJH1ippRnlumaMmVmfXqbaOSGqFmuoKkd3AHrQ7lhs1zda1J+defD7F7wDZyR2s3Bv6OhzlTYbLMOqTyz9dL1ZJ049Uw0hf358QAcAXsi+P88APm7q9a0Jl43zbx+8szavrvi4lxRxne7+BawPSpzNfm44Gk8Ai45mNKcNP7m3ZwR78QZae7DIs9sJ3gJJiX7vZshznpTR4jH4sHEE5rjTomnOM3OETf1lBTfi0W1XPSpyDtxh6uzx8FrqR0qFMuMW6p2iG4nDtxT71FBDYq28Sk/L7I96YkHyPernfCHAmOib3VTvX5I6hBLtU+1+yfizG8QPUgGXF2lSpUqLQz9Bvydqhni8puZAAAAAElFTkSuQmCC>

[image7]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABIAAAAYCAYAAAD3Va0xAAABE0lEQVR4Xu2SMUsDQRBGx8IiSMCIgqlsJaWNoGAp2NsY8AfY+D8srMQiWKaVlAlqZSOClWgjYhkbQVARLAR9H3MHm8lxnLV58Dhuv73dnbk1m/BXGniAn/iNl3iCq8mcFbzHd7zAdpKN8YIPuBgDmMM+1mJQxA+e4nQYb+HARk9YihbaS96ncAd72EzGS1GfPnAje1cJh5mVysnR8W9xwXx3NfQcZ9JJVdg27886Xpv/nTdcSydV4QgfsYvzeGzeMz3Vq0qorGfzsnLUF/0pLbafjJeya/5B3FmXThf0CmdDVojK0kIRXcw7/MKtkI2hftzgawzMT9gx3+QM66Oxs2m+kyblDnE5y5fwKeRFp57wf/kF2h01tTLTDzYAAAAASUVORK5CYII=>

[image8]: <data:image/png;base64,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>

[image9]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA8AAAAYCAYAAAAlBadpAAAA2UlEQVR4XmNgGAX8QLwGiL8C8R8g3gbEs6F4JVS8FohZYRrQgRYQPwPii0Asiia3gAFiaBaaOBwEMEAULABiRlQphgog/g/EW4CYA00ODNoZIAoy0SWAYDIDRA7kNaxO3wfEb4HYFF0CCE4yQFwVhS4BA48YsPsXBEAaQYGH1VYQAClYB8TSQCwFxbpAPAeIG4CYE64SCwD56QgDIopgOBVZETYAiqbbQCyDLkEMAEUTzpAkBEBRUYwuSAwQAeLTQGyHLoEPODFA0iwooGD4NRAbIisaBUMaAACxTCve6hva1wAAAABJRU5ErkJggg==>

[image10]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABIAAAAYCAYAAAD3Va0xAAABE0lEQVR4XmNgGAWkAkEg7gTij0D8H4iXAzErigoGhlQgPgnEn4B4AxBHoUqjglNA/AKKddHkQIAPiO3RBbGBDiCeygBxVROaHAgoM0BcTxC4AbEVA8SL94BYCVWawRuNjxWAbJIBYk4g3s4AcVUeigoGhnY0PlZgCMQcUDYoIP8A8TEgFoCKcQPxaigbL4hFYksA8WUg/gHEnlAxUPhshKvAA3rR+KDARk4KoPCZiKICCwB5awmaGEgzyBCQYRMYIN7SQlGBBYC8VYMuyABJM6AYBCXCwwwEop4RiGczQKIeHSDH4GQ0ORQAMsQEiC8xYDcIBGAxGIIuAQMgjaBYAdkGw8IoKiAAFIP7GIgIn1Ew4gAAXcgvH1WEvx4AAAAASUVORK5CYII=>

[image11]: <data:image/png;base64,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>

[image12]: <data:image/png;base64,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>

[image13]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAsAAAAYCAYAAAAs7gcTAAAA4ElEQVR4Xu2SMQ5BQRCGRyHRSAiFQucEKh2dWqFwAVcQhTiAViNKCo2OSuUONCJRiEqi0igU/v/tjOzb7AUkvuTLm8zM7pvMeyK/TRsu4QOeglqUKjzDdViIwdtfcBgWYozgEzbDQkgObsWNwXFIEZZhxpqMGryKO8CDc7iBB9j3+hL8eQuwAcfwrbkUnJfNPTjT3ATeYN2aCGe8wL242+6aixKurAWnGnP+vMYJbPJXxtda8wB2NJasuC/mr4zNvKAEF7Ci+STgv7ASd5DweYQ72NXcF67KGg1+jNSsf3w+FoElrSo4UnUAAAAASUVORK5CYII=>