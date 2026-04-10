# **Análisis de Ingeniería Inversa de TigerGraph: Arquitectura Nativa Paralela y Fundamentos para ConnectomeDB**

El panorama de la computación de grafos ha sido transformado por la aparición de sistemas que no solo almacenan relaciones, sino que las ejecutan como unidades de procesamiento. TigerGraph se define como una plataforma de grafos nativa y paralela (Native Parallel Graph, NPG), diseñada para abordar analíticas a escala web en tiempo real mediante una arquitectura que co-localiza el almacenamiento y el cómputo.1 Para el desarrollo de ConnectomeDB, un sistema inspirado en la neurobiología y escrito en Rust, TigerGraph ofrece un caso de estudio crítico sobre cómo la implementación en C++ y el diseño orientado a mensajes pueden alcanzar un rendimiento de millones de travesías por segundo.1 Esta investigación desglosa los componentes internos de TigerGraph, desde la estructura física de sus datos hasta las sutilezas de su gestión de memoria y búsqueda vectorial.

## **Anatomía de la "Neurona": Estructura de Datos y el Motor GSE**

En el núcleo de TigerGraph, la representación de la información se aleja de las tablas relacionales y los documentos NoSQL para adoptar un modelo donde cada vértice y arista actúa como una unidad de almacenamiento y computación simultánea.2 Esta filosofía, que la empresa compara explícitamente con el comportamiento de las neuronas en el cerebro humano, es el pilar sobre el cual se construye el Graph Storage Engine (GSE).1

### **El Motor de Almacenamiento de Grafos (GSE) y el Formato Nativo**

El GSE es el componente responsable de gestionar el "Graph Store", el almacén físico y lógico de los datos.1 A diferencia de los sistemas que utilizan una capa de abstracción sobre bases de datos de claves-valores, TigerGraph utiliza un formato de almacenamiento propietario diseñado para la compresión y la velocidad.2 La compresión en TigerGraph no es solo una medida de ahorro de espacio, sino una estrategia para maximizar el uso de la caché de la CPU y minimizar los fallos de página al procesar grandes volúmenes de datos.3

La arquitectura física organiza los datos para aprovechar la localidad, tanto en disco (SSD/HDD) como en memoria RAM y caché.2 Los valores de los datos se almacenan en formatos codificados que permiten factores de compresión de entre 2x y 10x, dependiendo de la estructura del grafo.3 Un aspecto técnico fundamental es que esta codificación es a menudo homomórfica, lo que permite realizar comparaciones y ciertos cálculos internos sin necesidad de descomprimir los datos, reduciendo así la latencia de CPU.3

### **El Servicio de Identidad (IDS) y la Adyacencia Libre de Índices**

Un subcomponente crítico dentro del GSE es el ID Service (IDS).1 Su función es la traducción entre los identificadores externos (IDs proporcionados por el usuario, frecuentemente cadenas) y los identificadores internos del almacén de grafos.1 Esta traducción es esencial para implementar la adyacencia libre de índices (index-free adjacency), donde cada nodo mantiene referencias físicas directas a sus vecinos en lugar de depender de búsquedas en tablas de índices globales.4

En términos de ingeniería, la adyacencia libre de índices permite que el costo de navegar de un nodo a otro sea constante (![][image1]), independientemente del tamaño total del grafo.6 Esto se logra porque las aristas se almacenan como punteros directos a las direcciones de memoria de los nodos adyacentes.4 Cuando un nodo es recuperado y cargado en la caché, sus nodos relacionados son fácilmente accesibles, lo que acelera dramáticamente las consultas de múltiples saltos (multi-hop).4

| Componente del GSE | Función Técnica | Impacto en el Rendimiento |
| :---- | :---- | :---- |
| **IDS (ID Service)** | Mapeo de IDs externos a internos | Permite el uso de tipos de datos compactos para punteros internos.1 |
| **Proprietary Encoding** | Compresión homomórfica de atributos | Reduce el ancho de banda de memoria y mejora el uso de la caché.3 |
| **IFA Implementation** | Almacenamiento de aristas como punteros directos | Elimina la necesidad de "joins" y búsquedas en índices durante la travesía.4 |
| **Localidad de Datos** | Co-ubicación de vértices y sus aristas de salida | Optimiza las lecturas secuenciales y el paralelismo de hilos.2 |

### **Representación Lógica y Tipado**

TigerGraph implementa el modelo de grafo de propiedades etiquetado (Labeled Property Graph, LPG).2 Los vértices y aristas pueden tener atributos de diversos tipos, incluyendo tipos atómicos, tuplas, contenedores y acumuladores especiales.9 La rigidez del esquema en TigerGraph, aunque requiere una definición previa mediante DDL, permite optimizaciones de bajo nivel en el motor de ejecución que los sistemas sin esquema (schema-less) no pueden alcanzar fácilmente.11

## **Lógica de Recuperación y Búsqueda: El Motor GPE y GSQL**

Si el GSE es el cuerpo de la neurona, el Graph Processing Engine (GPE) es el potencial de acción que permite la comunicación y el procesamiento.1 El GPE es el motor de procesamiento paralelo masivo (MPP) que ejecuta las consultas y algoritmos sobre el grafo.1

### **Procesamiento Masivo Paralelo (MPP) y Diseño Orientado a Mensajes**

La arquitectura de TigerGraph se basa en un diseño de paso de mensajes para coordinar las actividades.1 Cada vértice y arista funciona como una unidad de cómputo independiente. Durante la ejecución de una consulta, los vértices pueden enviar mensajes a través de sus aristas a otros vértices, permitiendo que el grafo funcione como un motor de computación distribuida.2

Este enfoque MPP permite que TigerGraph escale horizontalmente. El sistema particiona automáticamente los datos entre los nodos del clúster utilizando índices hash para determinar la ubicación de los datos.3 En el modo de consulta distribuida, la computación se mueve hacia los datos; cada servidor procesa su parte del grafo y los resultados parciales se agregan para la respuesta final.3

### **GSQL: Un Lenguaje Compilado para Grafos**

A diferencia de los lenguajes interpretados, GSQL se diseña para ser de alto nivel pero extremadamente expresivo, permitiendo especificar algoritmos iterativos complejos.3 GSQL se basa en bloques SELECT-FROM-WHERE, pero introduce la capacidad de manipular el estado del grafo a través de acumuladores.3

Una característica distintiva de GSQL es que sus consultas pueden ser instaladas. Al instalar una consulta, el sistema la compila en código C++ optimizado, el cual se enlaza dinámicamente con el motor GPE.12 Esto elimina el overhead de interpretación durante la ejecución y permite que el optimizador de consultas realice transformaciones agresivas basadas en el esquema del grafo.12

### **Acumuladores: El Estado Dinámico de la Búsqueda**

Los acumuladores son variables especiales en GSQL que permiten recolectar información durante la travesía.10 Son fundamentales para el paralelismo, ya que actúan como variables con exclusión mutua (mutex) que pueden ser actualizadas simultáneamente por múltiples hilos de ejecución.10

| Tipo de Acumulador | Ámbito de Uso | Ejemplo de Operación Logica |
| :---- | :---- | :---- |
| **Global (@@)** | Global para toda la consulta | Sumar el total de transacciones en un grafo de fraude.10 |
| **Local (@)** | Adjunto a cada vértice | Almacenar el puntaje de PageRank intermedio de un nodo.10 |
| **Collection** | Estructuras de datos (List, Set, Map) | Agrupar los IDs de todos los vecinos visitados en un salto.10 |
| **Scalar** | Valores numéricos o booleanos | Encontrar el valor máximo de una propiedad entre los vecinos.10 |

El uso de acumuladores permite que TigerGraph realice análisis complejos, como la detección de comunidades o el cálculo de centralidad, de manera eficiente dentro de la base de datos, reduciendo la necesidad de mover datos a sistemas externos.15

## **Integración de Vectores y Búsqueda Híbrida**

TigerGraph ha evolucionado para soportar capacidades de base de datos vectorial, integrando vectores como atributos de especialidad en los vértices.17 Esto permite realizar búsquedas de similitud aproximada (ANN) dentro del mismo contexto del grafo.17

### **Implementación de HNSW Distribuido**

Para la búsqueda vectorial, TigerGraph utiliza el algoritmo Hierarchical Navigable Small World (HNSW).17 El sistema construye automáticamente índices HNSW a medida que se cargan los vectores, y estos índices se distribuyen a través del clúster para mantener la escalabilidad.17 El rendimiento de la búsqueda vectorial se puede ajustar mediante parámetros como el factor de exploración (![][image2]), donde un valor más alto mejora la precisión (recall) pero aumenta la latencia.17

### **Lógica de Consulta Híbrida Grafo \+ Vector**

La verdadera potencia de TigerGraph reside en la capacidad de combinar la búsqueda vectorial con la travesía de grafos en una sola unidad lógica a través de GSQL.18 Esto permite arquitecturas de búsqueda híbrida avanzadas:

1. **Recuperación Vectorial**: Se utiliza vectorSearch para encontrar los ![][image3] nodos más similares en un espacio de incrustaciones (embeddings).17  
2. **Filtrado por Grafo**: Los resultados del paso anterior se utilizan como "semillas" para una travesía de múltiples saltos que aplica lógica de negocio o filtros estructurales (por ejemplo, "encontrar productos similares comprados por amigos del usuario").18

Este enfoque supera las limitaciones de las bases de datos vectoriales puras, que a menudo requieren un post-filtrado costoso o carecen de la capacidad de navegar relaciones complejas entre los vectores recuperados.18

## **Gestión de Memoria y Estado: El Desafío del "Memory-First"**

TigerGraph es un sistema diseñado para priorizar la memoria RAM, lo que le permite alcanzar latencias bajas, pero también impone desafíos significativos en la gestión de recursos.21

### **Control de Concurrencia y Aislamiento de Transacciones**

Para gestionar las actualizaciones concurrentes, TigerGraph utiliza el nivel de aislamiento de lectura confirmada (Read-Committed).21 Internamente, implementa el control de concurrencia de versiones múltiples (MVCC) para permitir que las operaciones de lectura y escritura ocurran simultáneamente sin bloqueos excesivos.21

La consistencia se mantiene de forma estricta en entornos distribuidos. Una transacción de actualización solo se considera completa cuando todas las réplicas han terminado la actualización en el mismo orden, garantizando la durabilidad mediante el uso de logs de escritura anticipada (Write-Ahead Logging, WAL) que se persisten en disco.21

### **Estados de Memoria y Protección del Sistema**

El GPE monitorea constantemente la memoria libre y utiliza umbrales para entrar en diferentes estados de operación para evitar fallos por falta de memoria (OOM) 21:

| Estado de Memoria | Umbral Típico (Memoria Libre) | Comportamiento del Sistema |
| :---- | :---- | :---- |
| **Healthy** | **![][image4]** | Operación normal sin restricciones.21 |
| **Alert** | **![][image5]** | El sistema mueve datos a disco y limita los hilos de procesamiento.21 |
| **Critical** | **![][image6]** | El sistema aborta consultas activas para preservar la estabilidad del nodo.21 |

Además, TigerGraph implementa un mecanismo de desbordamiento a disco para transacciones grandes. Si una transacción supera un límite de memoria (por defecto 4 MB), el binario de la transacción se convierte de memoria a archivo antes de ser procesado, lo que previene que ráfagas de escrituras masivas saturen la RAM.21

## **Análisis de la Documentación y la API**

La interacción con TigerGraph se facilita a través de varios componentes que exponen la funcionalidad del sistema a los desarrolladores y administradores.1

### **RESTPP: El Servidor REST Mejorado**

El centro de la gestión de tareas es RESTPP, un servidor RESTful especializado que maneja la comunicación entre las interfaces de usuario y los servicios internos.1 RESTPP actúa como una pasarela que recibe solicitudes HTTP, las valida y las enruta al GPE o GSE según corresponda.1

Las aplicaciones empresariales suelen interactuar con TigerGraph a través de endpoints REST para ejecutar consultas pre-instaladas, lo que minimiza la latencia de red y permite una integración sencilla con microservicios.1 La gestión del sistema se realiza mediante gAdmin, una utilidad de administración que permite configurar parámetros, monitorizar servicios y gestionar la seguridad del clúster.1

### **Pipeline de Carga y Conectores**

TigerGraph ofrece un cargador de datos flexible capaz de ingerir datos tabulares o semiestructurados en tiempo real.1 Los trabajos de carga (Loading Jobs) son declarativos y permiten mapear columnas de archivos (como CSV, JSON o Avro) directamente a atributos de vértices y aristas.22 El sistema soporta fuentes de datos modernas como Amazon S3, Google Cloud Storage, Kafka y data warehouses como Snowflake y BigQuery.22

## **Inspiración para ConnectomeDB: Features para Extraer**

Como arquitecto de ConnectomeDB, el análisis de TigerGraph proporciona lecciones valiosas sobre qué características son esenciales para un motor de grafos de alto rendimiento escrito en Rust.

### **1\. El Concepto de "Neurona Computacional"**

La idea de tratar cada vértice como una unidad de procesamiento es fundamental para ConnectomeDB. En Rust, esto puede implementarse de manera eficiente utilizando el modelo de actores o sistemas de paso de mensajes extremadamente ligeros (como los proporcionados por el ecosistema de tokio o actix). La clave es permitir que el estado del vértice sea local y que la computación se distribuya de forma natural sobre la estructura del grafo.

### **2\. Acumuladores Seguros en Rust**

La implementación de acumuladores en TigerGraph requiere el uso de mutex pesados en C++.10 En ConnectomeDB, se puede aprovechar el sistema de tipos de Rust y las operaciones atómicas (std::sync::atomic) para crear acumuladores con mucho menos overhead de sincronización. Las "variables mutables protegidas por mutex" de TigerGraph pueden evolucionar a estructuras de datos Concurrentes Libres de Bloqueos (Lock-Free), lo que escalaría mucho mejor en procesadores multi-núcleo modernos.

### **3\. Compilación JIT de Consultas**

La compilación de GSQL a C++ es poderosa pero lenta en términos de despliegue.14 Para ConnectomeDB, el uso de lógica LISP permite una ventaja estratégica. Se podría implementar un compilador Just-In-Time (JIT) utilizando LLVM o Cranelift para transformar las expresiones LISP y las travesías de grafos directamente en código máquina nativo en tiempo de ejecución, logrando la velocidad de TigerGraph con una flexibilidad mucho mayor.

### **4\. Zero-Copy y Formatos de Datos Eficientes**

TigerGraph sufre con la construcción de grandes respuestas JSON.26 ConnectomeDB debería adoptar el principio de "Zero-Copy" desde el diseño, utilizando formatos como Apache Arrow para el intercambio de datos entre el motor de base de datos y los clientes.27 Esto reduciría drásticamente el uso de CPU y la latencia de red al evitar la serialización y deserialización constante de datos.

### **5\. Integración Nativa de HNSW**

En lugar de ver los vectores como atributos adicionales, ConnectomeDB puede tratarlos como ciudadanos de primera clase. La integración del algoritmo HNSW directamente en el motor de almacenamiento (GSE) permitiría que las búsquedas vectoriales y las travesías estructurales compartan las mismas optimizaciones de caché y localidad de datos.

## **Puntos Débiles: La Oportunidad de Mercado para ConnectomeDB**

A pesar de su liderazgo tecnológico, TigerGraph presenta vulnerabilidades arquitectónicas y operativas que ConnectomeDB puede capitalizar.

### **1\. El Costo de la Serialización JSON**

Un cuello de botella documentado en TigerGraph es la generación de respuestas JSON masivas. Cuando el tamaño de la respuesta es grande, el sistema puede pasar más tiempo componiendo el JSON que recorriendo el grafo, lo que lleva a un uso ineficiente de la CPU.26

* **Oportunidad**: ConnectomeDB puede diferenciarse ofreciendo interfaces binarias nativas (como gRPC o Arrow Flight) que eliminan por completo la necesidad de formateo de texto pesado.

### **2\. Rigidez del Esquema y Gestión de Memoria**

TigerGraph requiere un esquema predefinido y sufre de problemas de memoria si hay demasiados tipos de vértices o si los IDs de las cadenas son muy largos.12 Además, la gestión de memoria en C++ es propensa a fragmentación y requiere ajustes manuales complejos de parámetros como SysAlertFreePct.21

* **Oportunidad**: Al ser escrito en Rust, ConnectomeDB ofrece una gestión de memoria mucho más segura y predecible sin los riesgos de seguridad asociados a C++. La flexibilidad de LISP permitiría un esquema más dinámico y adaptable, vital para aplicaciones de IA cognitiva que necesitan evolucionar su conocimiento.

### **3\. Latencia en la Actualización de Índices Vectoriales**

El índice HNSW en TigerGraph puede presentar retrasos (lag) respecto a las actualizaciones de datos debido al tiempo que consumen los cálculos de similitud.17

* **Oportunidad**: ConnectomeDB puede implementar técnicas de indexación vectorial más granulares y concurrentes que minimicen este lag, asegurando que los resultados de la búsqueda vectorial reflejen los datos más recientes casi en tiempo real.

### **4\. Limitaciones en la Lógica de Filtrado Híbrido**

Aunque TigerGraph soporta búsqueda híbrida, la implementación de pre-filtrado y post-filtrado puede ser compleja de optimizar para el usuario común.20 La falta de soporte para ciertas funciones en el modo de consulta distribuida también limita la flexibilidad de los desarrolladores.13

* **Oportunidad**: Un motor basado en LISP permitiría un optimizador de consultas mucho más potente que pueda decidir automáticamente la mejor estrategia de filtrado (pre vs post) basada en estadísticas en tiempo real, simplificando enormemente el desarrollo de aplicaciones RAG complejas.

## **Síntesis Técnica: De TigerGraph a ConnectomeDB**

La investigación sobre TigerGraph revela que el éxito en el mundo de los Big Graphs depende de la capacidad de paralelizar el cómputo al nivel más bajo posible y de reducir la fricción entre el almacenamiento y el procesamiento.2 La arquitectura Native Parallel Graph es el estándar de oro actual, pero su implementación sobre tecnologías heredadas como C++ y protocolos como REST/JSON deja espacio para la innovación.

Para ConnectomeDB, el camino a seguir implica:

* Adoptar la **adyacencia libre de índices** y el **almacenamiento nativo** como base innegociable.4  
* Mejorar el modelo de **acumuladores** mediante las garantías de concurrencia de Rust.10  
* Sustituir la complejidad de GSQL por la elegancia y potencia de **LISP**.3  
* Integrar la **búsqueda vectorial HNSW** de forma profunda y distribuida.17  
* Optimizar el transporte de datos mediante **Zero-Copy y formatos binarios**.26

Al atacar los puntos donde TigerGraph muestra debilidad—específicamente la eficiencia de la serialización, la seguridad de la memoria y la flexibilidad del lenguaje de consulta—ConnectomeDB puede posicionarse no solo como una alternativa de alto rendimiento, sino como la evolución necesaria hacia una base de datos verdaderamente cognitiva y preparada para la era de la IA generativa y el razonamiento complejo sobre redes de conocimiento masivas.

#### **Fuentes citadas**

1. Internal Architecture :: TigerGraph DB, acceso: abril 7, 2026, [https://docs.tigergraph.com/tigergraph-server/4.2/intro/internal-architecture](https://docs.tigergraph.com/tigergraph-server/4.2/intro/internal-architecture)  
2. Native Graph Database Engine \- TigerGraph, acceso: abril 7, 2026, [https://www.tigergraph.com/tigergraph-db/](https://www.tigergraph.com/tigergraph-db/)  
3. TigerGraph: A Native MPP Graph Database \- arXiv, acceso: abril 7, 2026, [https://arxiv.org/pdf/1901.08248](https://arxiv.org/pdf/1901.08248)  
4. Graph database \- Wikipedia, acceso: abril 7, 2026, [https://en.wikipedia.org/wiki/Graph\_database](https://en.wikipedia.org/wiki/Graph_database)  
5. TigerGraph vs Neo4j: How to Choose for Your Workload \- PuppyGraph, acceso: abril 7, 2026, [https://www.puppygraph.com/blog/tigergraph-vs-neo4j](https://www.puppygraph.com/blog/tigergraph-vs-neo4j)  
6. When are adjacency lists or matrices the better choice?, acceso: abril 7, 2026, [https://cs.stackexchange.com/questions/79322/when-are-adjacency-lists-or-matrices-the-better-choice](https://cs.stackexchange.com/questions/79322/when-are-adjacency-lists-or-matrices-the-better-choice)  
7. Graph Database \- TigerGraph, acceso: abril 7, 2026, [https://www.tigergraph.com/glossary/graph-database-2/](https://www.tigergraph.com/glossary/graph-database-2/)  
8. GraphAr: An Efficient Storage Scheme for Graph Data in Data Lakes \- arXiv, acceso: abril 7, 2026, [https://arxiv.org/html/2312.09577v4](https://arxiv.org/html/2312.09577v4)  
9. TigerGraph Overview. A comprehensive overview of TigerGraph… | by Asma Zgolli, PhD | DataNess.AI | Medium, acceso: abril 7, 2026, [https://medium.com/dataness-ai/tigergraph-overview-50c949272a5d](https://medium.com/dataness-ai/tigergraph-overview-50c949272a5d)  
10. Accumulators :: GSQL Language Reference \- TigerGraph Documentation, acceso: abril 7, 2026, [https://docs.tigergraph.com/gsql-ref/4.2/querying/accumulators](https://docs.tigergraph.com/gsql-ref/4.2/querying/accumulators)  
11. Graph Database Performance \- TigerGraph, acceso: abril 7, 2026, [https://www.tigergraph.com/glossary/graph-database-performance/](https://www.tigergraph.com/glossary/graph-database-performance/)  
12. Workload Management :: TigerGraph DB, acceso: abril 7, 2026, [https://docs.tigergraph.com/tigergraph-server/4.2/system-management/workload-management](https://docs.tigergraph.com/tigergraph-server/4.2/system-management/workload-management)  
13. Distributed Query Mode :: GSQL Language Reference \- TigerGraph Documentation, acceso: abril 7, 2026, [https://docs.tigergraph.com/gsql-ref/4.2/querying/distributed-query-mode](https://docs.tigergraph.com/gsql-ref/4.2/querying/distributed-query-mode)  
14. GSQL Query Language \- TigerGraph, acceso: abril 7, 2026, [https://www.tigergraph.com/glossary/gsql/](https://www.tigergraph.com/glossary/gsql/)  
15. Using Accumulators in GSQL for Complex Graph Analytics \- TigerGraph, acceso: abril 7, 2026, [https://info.tigergraph.com/graph-gurus-11](https://info.tigergraph.com/graph-gurus-11)  
16. Real-Time Graph Analytics \- TigerGraph, acceso: abril 7, 2026, [https://www.tigergraph.com/glossary/real-time-data-analytics/](https://www.tigergraph.com/glossary/real-time-data-analytics/)  
17. Vector Database Operations :: GSQL Language Reference \- TigerGraph Documentation, acceso: abril 7, 2026, [https://docs.tigergraph.com/gsql-ref/4.2/vector/](https://docs.tigergraph.com/gsql-ref/4.2/vector/)  
18. Vector Databases \- TigerGraph, acceso: abril 7, 2026, [https://www.tigergraph.com/glossary/vector-databases/](https://www.tigergraph.com/glossary/vector-databases/)  
19. Next Generation Hybrid Search (Graph \+ Vector) to Power AI at Scale \- TigerGraph, acceso: abril 7, 2026, [https://www.tigergraph.com/vector-database-integration/](https://www.tigergraph.com/vector-database-integration/)  
20. Optimizing Filtered Vector Search in MyScale \- Medium, acceso: abril 7, 2026, [https://medium.com/@myscale/optimizing-filtered-vector-search-in-myscale-77675aaa849c](https://medium.com/@myscale/optimizing-filtered-vector-search-in-myscale-77675aaa849c)  
21. Memory management :: TigerGraph DB, acceso: abril 7, 2026, [https://docs.tigergraph.com/tigergraph-server/4.2/system-management/memory-management](https://docs.tigergraph.com/tigergraph-server/4.2/system-management/memory-management)  
22. Transaction Processing and ACID Support :: TigerGraph DB, acceso: abril 7, 2026, [https://docs.tigergraph.com/tigergraph-server/4.2/intro/transaction-and-acid](https://docs.tigergraph.com/tigergraph-server/4.2/intro/transaction-and-acid)  
23. Load from \- TigerGraph Documentation, acceso: abril 7, 2026, [https://docs.tigergraph.com/tigergraph-server/4.2/data-loading/load-template](https://docs.tigergraph.com/tigergraph-server/4.2/data-loading/load-template)  
24. Map Data To Graph :: GraphStudio and Admin Portal \- TigerGraph Documentation, acceso: abril 7, 2026, [https://docs.tigergraph.com/gui/4.2/graphstudio/map-data-to-graph](https://docs.tigergraph.com/gui/4.2/graphstudio/map-data-to-graph)  
25. TigerGraph Savanna Architecture, acceso: abril 7, 2026, [https://docs.tigergraph.com/savanna/main/overview/architecture](https://docs.tigergraph.com/savanna/main/overview/architecture)  
26. Troubleshooting Guide :: TigerGraph DB, acceso: abril 7, 2026, [https://docs.tigergraph.com/tigergraph-server/4.2/troubleshooting/troubleshooting-guide](https://docs.tigergraph.com/tigergraph-server/4.2/troubleshooting/troubleshooting-guide)  
27. Towards Designing Future-Proof Data Processing Systems \- TUM, acceso: abril 7, 2026, [https://db.in.tum.de/\~jungmair/papers/p2473-jungmair.pdf](https://db.in.tum.de/~jungmair/papers/p2473-jungmair.pdf)  
28. Pre and Post Filtering in Vector Search with Metadata and RAG Pipelines \- DEV Community, acceso: abril 7, 2026, [https://dev.to/volland/pre-and-post-filtering-in-vector-search-with-metadata-and-rag-pipelines-2hji](https://dev.to/volland/pre-and-post-filtering-in-vector-search-with-metadata-and-rag-pipelines-2hji)

[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAfCAYAAAClDZ5ZAAAC6klEQVR4Xu2Yy6tPURTHl1BEHtGVKJekPEq6SsSMMjEhRRQxIBkZ8AfI6E6QJCkhkQFJSjK4I4mRIqWUK1IkpQyQx/dz99mcs37nnN85P7/H5PetT669zn6svdZ+/cz66quvXmuc2C5uianOVlWrxD0x3xuqarxYI86JV+JNwvWknEE20zbxVCzwhppquZ1l4rEYFXvFLAuOLRE3xS8LDk5Ovs/TcguOb/IGp2nirDjvDSkxaSfFfasYWSocFj/FFTElax7TRAud/hbHLT8yfHMjgb+9iOhRcdtCX7R1KfNFoxaLt+KgN3gxIBqnUQaaN4CoReK9+CxWOBvaIL6Izd6QaL+FKOwWw1bNEcZ3RjwTA86WEXnI7JBSpFKZiNQDCwM45mx0eFE8ETOdLU/Ur+II2mhhjDu8IYp8/mgh97c6W5HoOG8A88RrcdqVF6mOI7FtJqohpSeICxYaeySmZ825miTuWqgzYtkFyKwxIbtSZWWq40js97mY7WxjOU6u56VJkUgZUoc6Vy07O0fED7E+VVamOo4gIs36W+kNByw0VKfzpeKThXonnI3ofhVDrrxIdR3heyJO5P+KmWRGaYhDb07aWCLShjo06HcmBjQq5rryItV1JPa9JV1Ibo8kBv6tctik18cLa3S+047gQFscWSu+WajjD6cY4a47gmigqiPxxOZ7zpG8Ha7TEclNLRQXe+6W5hQPzZdWfIHryWJHCy3cYZodhussbNMcnKudLa1ubL/smOycDUrPNJeztLj17hPfxUMxmLE2qsqBSAqTerwxuHPhyJ3k/5TP+PdpRnGjKbz+sEi5br+zMODLYo84JT4k7LTgVDPFCJddUWIUiiiKTryicHlkzIViMZM2hywMhMvZoDWp5MSV55qVzNp/iGgz0f7s6pjoqN0dMplEoup9sC3i5UjOFz2sWlF8WPGG6ar40YDHV7OnbhURDZ667ZyYWmr5RwOndrXTspjJnv8c1FdfLeoPfE/F57x+H2EAAAAASUVORK5CYII=>

[image2]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACUAAAAfCAYAAABgfwTIAAACO0lEQVR4Xu2WO2gVQRSGf4mCiRY+AklQMAiCgiASUhijVRqLqIWdtdhYCoJVQK0FrYSAJE0aUwVRUoiIXbqgjSCE4AMElYgKIj7+37Oze3Yyu/dusMt+8MHdee6cOTN7gZaWTUo/PU8vNFDt1c9zIKtr4hjdos4x5+gq/UT/ZOq3yrzvXL08o86Oq7B2X1G0+ZCVeX39A7pVnauYoL/pNzoS1QV66TTq21yGTbhC95Wr/qHIjNPPdCaqW8c12GBLdHdU5xmlL+j+uAI24X10joLKVa85KwmNNJgiUYci9Bzrc0poMVqUxlHE6lCULsaFHoV5BTZY3PAYvUf7sme91DzdkbcoUJ229idsizynYYkd0OKVMpVoAA30BfYSniv0lnsehG1hCi1IC3uJciS1E3MoXkLbrDH25C0SpPJpG2x1b+nZrKyOunw6SZeRzsMk2+lDlI+79z09mLeuRpFRhNRHxz6+SjSH5uqKVD4pSiqfpU/pzqy8Dp9Piny4IO/SX/R60bQz4X76SI9EdRrc55O25BIddmWBcD+9pgOuXAtaRIekjknlU+AmPeWej9JnKE8q6vJJ2/oIG8ynO1FdjCa+QaeicuHzKb6femCLTX7jUujt36CcT1Ucp69g0YrR8f6O9P3UmLp88uhu0rbprkl9OkI+xfdTI3bRIXobNpi+ZYezMu8h2KlZoz/oCXXO0HbshUV7ATbOE9ghUN9uTmxOOLr+LurGx7B/CYHJRBvvVN6ypaWlpeX/8xfOVKS//9MaaAAAAABJRU5ErkJggg==>

[image3]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA0AAAAgCAYAAADJ2fKUAAABLElEQVR4Xu2SvSvGURTHv8LiLVJeShllMshkxGhXRpuMymaxKJtRSpRNySJiUxarf8AgGVCKzcvn/M7v6t7r9zw9RvV86lO3e8653XPulZr8oh1ncBl3cBN7k4wK+vECn/ELr7A7yajDorxoOw/UY1deZMUN0Yc3+ITjWawmk/gmL7QDGqKqH5vqfLk3He0XtOCe0n6G8AQ3cB1vcbCMFeT9jOJxuV6QH3aHw6HAiPuZwiN5obEkL7JHt+v+EPp5wDMciYO1+HNRPIQX/MB7XFF2nZh8CD14KD9kLcpLmMBXpZ/UhvGO59gh/9Cr8gMLqh7VHtT27C8as7iPbSHBkvNPakmf8utZXwc4F4KdeImPOBY2oQtP5dO8xi1lQ7EEM6cVB+T92ISb/CO+AauiQTPgpSogAAAAAElFTkSuQmCC>

[image4]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAD0AAAAZCAYAAACCXybJAAAAqUlEQVR4XmNgGAWjYBSMglEwCkbBKBiEQAyIDwJxPhBzo8kNa8AHxDVA/BKIu4FYGFV6eANQTINiHOT5uUCsiCo9vAEnECcB8TMgXgfE6qjSwxswA7E/EF8F4uNAbA7EjCgqhjEAed4JiM9DMYgNEhsRABTLoNgGxToo9u1RpYcnGFGeHlHJG7kg28UwzAuyEVVljajGyYhrhoI6HKcYIJ4GeX4UjIJhAAAh6Bwh1NDOigAAAABJRU5ErkJggg==>

[image5]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAGgAAAAZCAYAAADdYmvFAAAC70lEQVR4Xu2Z24uNURjGH6EIORaKuJByLDkl1ERCToUL5Q+QEkU55kJSuBIpuTIXklMucCNlRFwoUlxJUSIprojk8Dy937LXXnuv75uRLWbeX/3a8613TTOt91vvOmzAcRzHcbo/A+gx+om+pIvqww30ojvpoTTg/Hn60tP0Jh1MJ9DHdDftH/UL9KbbYH3GJTGHTKYP6fI0kDCf3qc/6Dd6G/a7KfPoF7o2attA39NX9AzdSNfTI/Q1/UrXhc49Hb3Fq+hR+gg24FJtOTR4H2Fvut54uYd+Rn0ixF5YaZsdtU2iF+kwOguWHNlG99FTsJnnkCH0MCxBa1CbFbkEjaXP6BXUD6J+Vpti6hNohyVzZtQ2mnYUnzEz6F14acsyEDZwZQnaBItvTwNkFyymkhXIJUhr0sioTX/7Gry0lVKVoD70EvJxlTfFzsJ2YkKJ1JqyIHSCJesybHcX2AEvbZVUJWgofYB8XG2K3aGDirap9APdUjwrcQejZ+GlrZNUJUilSeeYXDwkSH3C+qKEaHa8oythJVLlbXgR99LWBVqRIKEkzaXH6X46Ioo1K206L2l7rk8nolUJypGWNiVSpe8tPU9f0BVFzMHfTZBmx3XUl7YlsPVqYfE8hd5C/W4vxzTUzlNdcTH+o41JVYI0UM+Rj4cEqU/ZoGqm6K4tLm3aIZ6jT1Ergep3AnbzUIUnCLYt1gKfi4cEqU+8hU6Zg8Zdm5Ki5HTA/o+AzlR+aVpQlSChhV7xsoOq+uRoVtpEKJ8dqE+Q/o/26LlH05kELaPf0TwJatPFaVvSHmhW2gI+gzJo0HQm0Rs8nT6BJWhz0Sb7/eptl6tXYevM+KhdXyPoJvoCGgc/oNKmQ+yYNIDaLcXvrkHdlnjW5Exnkwb4HqwkbS18A1t7RkX9YkJpW50GItJdnJJ+A+UbDieDvmLQndoBWNmaiNr9WzN0ID0Jmyk54nOQ7vN0M55+feG0CM2Csp1djEru0uLTcf5NfgKSjsKfwUGpagAAAABJRU5ErkJggg==>

[image6]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAD0AAAAZCAYAAACCXybJAAAAuUlEQVR4Xu2XrQ0CQRCF50IQhAqQKBwWTwMg0dcAAkcXCPTlGrgCaACJwqNw9MB7WRA7oQHezJd8ySWzZvL2Z84sSZIkSZJEhAXs4ckX1GjgCl7hHW7gqFohBBtbw9tHfss2O4Y7+IAXKykzbUkmsIVPOFg5v7JM4R6+YAfndVkLJnu00uwZzuqyJjy7BwvW9JdQ29sT6iLzcNtvrQwifLKWJvxkeUINJx6mzLSZuvwY+oswPxzJv/IGl/AcRgO1LyUAAAAASUVORK5CYII=>