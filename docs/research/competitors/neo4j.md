# **Ingeniería Inversa de Neo4j: Arquitectura, Mecánica Cognitiva y Fundamentos Estructurales para VantaDB**

La construcción de un sistema de base de datos de nueva generación como VantaDB exige una comprensión profunda de los paradigmas existentes que han definido el procesamiento de grafos y la recuperación de información multidimensional. Neo4j, como pionero en el ámbito de las bases de datos de grafos nativas, ofrece un caso de estudio excepcional para el análisis de ingeniería, permitiendo diseccionar cómo las estructuras de datos físicas, la gestión de la memoria y los protocolos de comunicación convergen para habilitar lo que se denomina Adyacencia Libre de Índices. El presente reporte técnico desglosa los componentes internos de Neo4j mediante un proceso de ingeniería inversa fundamentado en su arquitectura de almacenamiento, lógica de ejecución y comportamiento sistémico, proporcionando una base crítica para la implementación de VantaDB en Rust, integrando vectores y lógica simbólica LISP.

## **Anatomía de la Neurona: Estructura de Datos y Persistencia Nativa**

En la arquitectura de Neo4j, la unidad de información fundamental, el nodo, se comporta como una entidad discreta dentro de una red interconectada, lo que en el contexto de VantaDB se traduce como la neurona. La natividad de Neo4j no reside únicamente en su API, sino en cómo los datos se disponen físicamente en el disco para minimizar el coste de acceso aleatorio. Históricamente, Neo4j ha utilizado un formato de almacenamiento basado en registros de longitud fija, lo que permite el cálculo de direcciones físicas mediante desplazamientos simples o *offsets*.1

### **El Modelo de Almacenamiento de Registros Fijos**

El motor de almacenamiento tradicional de Neo4j descompone el grafo en múltiples archivos de base de datos especializados, cada uno con una responsabilidad estructural definida. El archivo neostore.nodestore.db es el corazón de esta arquitectura. Cada nodo se representa como un registro de 15 bytes. Esta longitud fija es una decisión de diseño crítica: permite que el motor de base de datos localice cualquier nodo en el disco en un tiempo constante ![][image1] si se conoce su identificador interno, simplemente multiplicando el ID del nodo por el tamaño del registro.1

| Componente de Registro (v4.x) | Tamaño en Bytes | Función Técnica |
| :---- | :---- | :---- |
| Bandera de Uso (In-Use) | 1 Byte | Indica si el registro está activo o ha sido eliminado para reutilización. |
| ID de Primera Relación | 4 Bytes | Puntero físico al primer registro de relación en la cadena del nodo. |
| ID de Primera Propiedad | 4 Bytes | Puntero al inicio de la lista enlazada de propiedades del nodo. |
| ID de Grupo de Etiquetas | 4 Bytes | Referencia a la estructura que define las etiquetas asociadas al nodo. |
| Byte de Configuración | 2 Bytes | Reservados para metadatos de formato y expansión futura. |

La gestión de las relaciones en neostore.relationshipstore.db sigue una lógica similar pero más compleja, con registros de 34 bytes. Una relación debe contener punteros directos a su nodo de origen y su nodo de destino, pero también punteros a los registros de relación anterior y posterior para ambos nodos. Esto crea una estructura de lista doblemente enlazada que permite navegar por todas las conexiones de un nodo sin realizar escaneos globales.1

| Atributo de Relación | Tamaño en Bytes | Importancia en el Recorrido del Grafo |
| :---- | :---- | :---- |
| ID de Nodo Origen | 4 Bytes | Referencia directa al punto de partida de la arista. |
| ID de Nodo Destino | 4 Bytes | Referencia directa al punto de llegada de la arista. |
| Tipo de Relación | 4 Bytes | Puntero al almacén de nombres de tipos de relación. |
| Puntero Previo (Origen) | 4 Bytes | Relación anterior en la cadena del nodo origen. |
| Puntero Siguiente (Origen) | 4 Bytes | Siguiente relación en la cadena del nodo origen. |
| Puntero Previo (Destino) | 4 Bytes | Relación anterior en la cadena del nodo destino. |
| Puntero Siguiente (Destino) | 4 Bytes | Siguiente relación en la cadena del nodo destino. |
| ID de Primera Propiedad | 4 Bytes | Inicio de la cadena de propiedades de la relación. |

Este diseño de lista enlazada para las relaciones es lo que fundamenta la Adyacencia Libre de Índices. No se requiere un índice intermedio para saltar de un nodo a su vecino; el motor simplemente sigue el puntero de la relación hacia el ID del nodo destino y luego calcula el *offset* en el archivo de nodos para obtener los datos. Sin embargo, este modelo presenta desafíos cuando un nodo acumula millones de relaciones (supernodos), ya que recorrer la lista enlazada se vuelve secuencial y costoso.3

### **Evolución hacia el Formato de Bloque en Neo4j 5.0**

Con la llegada de Neo4j 5.0, se introdujo el "Block Format" como una evolución generacional del motor de almacenamiento nativo. A diferencia del formato de registro que dispersa los datos en múltiples archivos especializados, el formato de bloque busca la consolidación y la localidad de los datos. El objetivo es reducir las operaciones de entrada/salida (IOPS) al empaquetar nodos y sus propiedades más frecuentes dentro del mismo bloque físico de almacenamiento.5

El formato de bloque utiliza estructuras de datos más complejas, similares a árboles, para manejar datos que crecen dinámicamente. Al alinear los datos con el tamaño de las páginas de la caché de memoria, Neo4j logra que una sola lectura de disco traiga un vecindario completo del grafo a la RAM. Los resultados de ingeniería muestran que el rendimiento mejora sustancialmente a medida que el grafo crece, superando al formato tradicional en un 40% cuando el grafo está totalmente en memoria y hasta en un 70% cuando el sistema está bajo presión de memoria y debe recurrir al disco frecuentemente.6

| Métrica de Almacenamiento | Formato de Registro (Legacy) | Formato de Bloque (v5.0) |
| :---- | :---- | :---- |
| Localidad de Datos | Fragmentada por tipo de entidad | Alta (Propiedades inlining) |
| Eficiencia de Caché | Moderada | Optimizada para CPU y Page Cache |
| Escalabilidad de IDs | Límites fijos (aprox. 34 mil millones) | Escalabilidad masiva (Trillion-scale) |
| Fragilidad ante Supernodos | Alta latencia de recorrido lineal | Estructuras de árbol optimizadas |

Para VantaDB, la lección es clara: el almacenamiento debe estar diseñado para la localidad. En Rust, esto implica el uso de estructuras que minimicen las indirecciones de punteros y aprovechen el diseño de las líneas de caché de la CPU. La transición de Neo4j hacia un modelo de bloques sugiere que el futuro de las bases de datos cognitivas no está en las listas enlazadas simples, sino en estructuras de datos densas y compactas que puedan ser procesadas mediante instrucciones SIMD.3

## **Lógica de Recuperación y Búsqueda: La Convergencia de Grafos y Vectores**

La capacidad de Neo4j para realizar búsquedas no se limita al recorrido topológico; ha integrado capacidades vectoriales que permiten realizar una recuperación semántica sobre los datos interconectados. Esta dualidad es la que VantaDB busca perfeccionar. En Neo4j, el recorrido del grafo y la búsqueda de vecinos más cercanos (ANN) operan en planos lógicos distintos que se encuentran en el optimizador de consultas.7

### **Recorrido de Grafos y Adyacencia Libre de Índices**

La recuperación en un grafo nativo se basa en el principio de que los datos son el índice. En una base de datos relacional tradicional, encontrar conexiones implica realizar una operación de unión (JOIN), que internamente requiere escanear un índice (generalmente un árbol B+) con una complejidad de ![][image2]. Si se realizan ![][image3] saltos, la complejidad acumulada es ![][image4]. Neo4j elimina esta dependencia del logaritmo del tamaño total del dataset. Al utilizar punteros físicos, el costo de atravesar una relación es constante, ![][image1]. Por lo tanto, el tiempo de respuesta de una consulta de grafo es proporcional únicamente a la cantidad del grafo que se recorre, no al tamaño total de la base de datos.7

Esta mecánica de "Pointer Hopping" es extremadamente eficiente en términos de latencia de memoria. Un salto de puntero en RAM toma aproximadamente 100 nanosegundos. En comparación, una consulta a través de una red en una arquitectura distribuida puede tomar 500 microsegundos. Esta diferencia de varios órdenes de magnitud explica por qué las bases de datos de grafos nativas superan a las capas de abstracción de grafos construidas sobre bases de datos NoSQL o relacionales cuando se trata de recorridos profundos de más de tres saltos.3

### **Búsqueda de Vectores y Algoritmo HNSW**

Neo4j utiliza el algoritmo Hierarchical Navigable Small World (HNSW) para sus índices vectoriales. HNSW es una estructura de grafos multicapa donde cada capa es un subconjunto de los vectores, permitiendo una navegación rápida desde conexiones de largo alcance en las capas superiores hasta conexiones de proximidad fina en las capas inferiores. El proceso de búsqueda comienza en un punto de entrada en la capa superior y realiza una búsqueda codiciosa para encontrar los nodos más cercanos al vector de consulta, descendiendo de capa en capa hasta llegar a la base, donde se identifican los k-vecinos más cercanos aproximados.9

Para VantaDB, es crucial analizar los parámetros de configuración de HNSW que Neo4j expone, ya que determinan el equilibrio entre la precisión del recall y la latencia de la consulta.

| Parámetro HNSW | Función en Neo4j | Impacto en Ingeniería |
| :---- | :---- | :---- |
| vector.hnsw.m | Conexiones máximas por nodo | Aumentar ![][image5] mejora el recall pero incrementa el uso de memoria RAM y el tiempo de construcción. |
| ef\_construction | Tamaño de la lista de candidatos durante construcción | Determina la calidad del grafo. Un valor alto produce un índice más robusto. |
| ef\_search | Tamaño de la lista de candidatos durante búsqueda | Parámetro de tiempo de ejecución. Más alto significa mayor precisión a costa de latencia. |
| vector.quantization.enabled | Compresión de vectores | Reduce el tamaño del índice en disco y memoria, permitiendo escalar a más vectores en el mismo hardware. |

La integración de estos vectores con el grafo se realiza mediante el plugin GenAI y la cláusula SEARCH de Cypher. Neo4j permite almacenar embeddings como propiedades de tipo VECTOR, un tipo de dato optimizado que evita la sobrecarga de las listas genéricas de Java. El uso de cuantización en estos vectores permite reducir el espacio ocupado en la caché del sistema operativo hasta en un 60%, lo que es vital para mantener el rendimiento en conjuntos de datos de escala de millones de vectores.14

### **Búsqueda Híbrida y Filtrado Predicativo**

Un aspecto avanzado de la ingeniería de Neo4j es cómo maneja el filtrado de metadatos durante una búsqueda vectorial. Existen tres estrategias principales: pre-filtrado, post-filtrado y filtrado en el índice.

El pre-filtrado utiliza el motor de grafos para identificar un subconjunto de nodos candidatos (por ejemplo, "todos los documentos del autor X") y luego calcula la similitud vectorial sobre ese subconjunto. Aunque garantiza un recall del 100% sobre los candidatos, puede volverse lento si el conjunto de candidatos es muy grande.8 El post-filtrado realiza la búsqueda vectorial primero y luego descarta los resultados que no cumplen los criterios. El problema técnico aquí es que si el filtro es muy restrictivo, se pueden terminar devolviendo cero resultados, lo que requiere técnicas de "over-fetching" (pedir más resultados de los necesarios inicialmente).8

La innovación reciente en Neo4j es el filtrado dentro del índice (In-index filtering). Durante el recorrido del grafo HNSW, el algoritmo consulta un predicado booleano para decidir si un nodo vecino es elegible. Esto permite que la búsqueda vectorial se mantenga dentro de los límites del filtro de metadatos sin perder la eficiencia del algoritmo de "pequeño mundo". Esta técnica utiliza representaciones compactas de metadatos, como mapas de bits o *bitsets*, para realizar comprobaciones rápidas de pertenencia durante el salto entre nodos vectoriales.8

## **Gestión de Memoria y Estado: El Equilibrio entre JVM y Memoria Nativa**

Como sistema basado en la Java Virtual Machine (JVM), Neo4j enfrenta desafíos únicos en la gestión de la memoria, especialmente en lo que respecta a la recolección de basura (Garbage Collection) y el uso de memoria fuera del montón (*off-heap*). Para un arquitecto de VantaDB que trabaja en Rust, entender estas limitaciones es fundamental para diseñar un sistema que evite las pausas de latencia impredecibles de Java.20

### **El Tríptico de Memoria de Neo4j**

La memoria en un servidor Neo4j se divide en tres áreas principales que deben ser configuradas explícitamente para evitar conflictos y degradación del rendimiento.20

1. **JVM Heap:** Aquí residen los objetos de Java, el motor de consultas de Cypher, la gestión de transacciones y los resultados intermedios de las consultas. Si el montón es demasiado pequeño, el sistema sufrirá de "GC trashing", donde la CPU pasa más tiempo limpiando memoria que procesando datos. Si es demasiado grande, las pausas de limpieza ("Stop-the-world") pueden durar segundos, afectando la disponibilidad del sistema.20  
2. **Page Cache:** Esta es la memoria más importante para el rendimiento del grafo. Neo4j gestiona su propio caché de páginas fuera del montón de la JVM para almacenar los archivos de datos y los índices nativos. Al estar fuera del montón, no está sujeto a la recolección de basura de Java. La regla de oro en la ingeniería de Neo4j es asignar suficiente Page Cache para que quepa todo el conjunto de datos de los archivos .db más un margen de crecimiento del 20%.20  
3. **Memoria Nativa / OS:** Incluye la memoria utilizada por las bibliotecas de red (como Netty), los índices de Lucene y los índices vectoriales. Neo4j recomienda dejar al menos 1-2 GB de RAM libre para el sistema operativo para evitar el uso del área de intercambio (*swap*), lo cual es letal para el rendimiento de una base de datos de grafos.20

| Configuración de Memoria | Parámetro Recomendado | Justificación de Ingeniería |
| :---- | :---- | :---- |
| Heap Inicial / Máximo | server.memory.heap.initial\_size \= max\_size | Evita la fragmentación y las pausas de re-asignación durante el crecimiento del montón. |
| Page Cache Size | server.memory.pagecache.size | Debe cubrir el tamaño total del almacén de datos para minimizar lecturas de disco. |
| Max Direct Memory | \-XX:MaxDirectMemorySize | Limita la memoria que Netty y otras bibliotecas nativas pueden solicitar. |
| Gestión de Transacciones | db.memory.transaction.total.max | Evita que una sola consulta masiva agote el montón de la JVM y cause un crash (OOM). |

### **El Problema de la Recolección de Basura (GC)**

La latencia P99 de Neo4j está fuertemente influenciada por el comportamiento del GC. En sistemas de grafos, las consultas suelen generar una gran cantidad de objetos de corta vida (como los iteradores de resultados). Si estos objetos no se limpian rápidamente en la "Young Generation", son promovidos prematuramente a la "Old Generation", lo que eventualmente obliga a una recolección completa. Para optimizar esto, Neo4j sugiere ajustar el tamaño de las generaciones y utilizar recolectores modernos como G1GC o ZGC, buscando que el estado de la transacción nunca llegue a la "Old Generation".22

VantaDB, al ser escrito en Rust, elimina este problema de raíz. Al utilizar un modelo de propiedad (*ownership*) y gestión de memoria determinista sin recolector de basura, VantaDB podrá garantizar latencias consistentes incluso bajo cargas extremas, algo que Neo4j solo puede mitigar mediante un ajuste fino constante.3

## **Análisis de la Documentación y API: El Protocolo Bolt y el Motor de Consultas**

La interacción con Neo4j se basa en dos pilares: el lenguaje de consulta Cypher y el protocolo de transporte binario Bolt. Estos componentes representan la capa de abstracción que convierte las intenciones del usuario en operaciones de bajo nivel sobre el almacenamiento físico.26

### **El Protocolo Bolt y la Serialización PackStream**

Bolt es un protocolo orientado a la conexión y con estado que opera sobre TCP o WebSockets. Su eficiencia se debe a PackStream, un formato de presentación binaria inspirado en MessagePack pero optimizado para los tipos de datos de grafos y el sistema de tipos de Cypher. PackStream utiliza un diseño basado en marcadores de bytes donde el primer byte define tanto el tipo de dato como, en muchos casos, su tamaño o valor directo (para enteros pequeños).27

| Tipo PackStream | Byte de Marcador | Descripción Técnica |
| :---- | :---- | :---- |
| Null | C0 | Representa la ausencia de valor. |
| Boolean | C2 (False) / C3 (True) | Valores lógicos simples. |
| Float | C1 | Punto flotante de 64 bits (IEEE 754). |
| Integer | C8 \- CB | Enteros con signo de 8 a 64 bits. |
| String | 80 \- 8F / D0 \- D2 | Cadenas UTF-8 con longitud variable. |
| Structure | B0 \- BF | Contenedor de tipos para Nodos, Relaciones y Caminos. |

El protocolo Bolt v5.0 introdujo mejoras significativas en la gestión de sesiones y el manejo de identificadores únicos globales (element\_id), superando la dependencia histórica de los IDs de nodo internos que eran volátiles. Para VantaDB, el uso de un protocolo binario similar a Bolt es esencial. Rust ofrece bibliotecas como serde y bincode que podrían proporcionar una serialización aún más rápida que PackStream, aprovechando el diseño de memoria de tipos de Rust para lograr una deserialización de "coste cero".31

### **Ciclo de Vida de una Consulta Cypher**

Cuando una consulta llega a Neo4j, pasa por un pipeline de transformación complejo 26:

1. **Parser y AST:** La cadena de texto se descompone en un Árbol de Sintaxis Abstracta (AST). Neo4j realiza comprobaciones semánticas en este punto, verificando tipos de variables y alcances.33  
2. **Optimizador Lógico:** El AST se convierte en un grafo de consulta que se somete a múltiples pasadas de optimización. Esto incluye la eliminación de operaciones redundantes (WITH innecesarios), el plegado de constantes y la reescritura de predicados para aprovechar índices.33  
3. **Planificador de Consultas (CBO):** El optimizador basado en costos (Cost-based Optimizer) utiliza estadísticas actualizadas de la base de datos para estimar la selectividad de los filtros. Decide si comenzar la búsqueda desde un nodo específico (Index Seek) o realizar un escaneo de etiquetas (Label Scan). La decisión de qué nodo elegir como "punto de anclaje" es vital para el rendimiento.33  
4. **Generación de Plan de Ejecución:** El plan final consiste en una secuencia de operadores (como Expand(All), Filter, Project, ProduceResults). Neo4j ofrece diferentes runtimes: el runtime interpretado (más lento pero compatible con todo), el runtime *slotted* (optimizado para memoria) y el nuevo runtime paralelo para consultas analíticas pesadas.36

## **Inspiración para VantaDB: Neurobiología, Rust y Lógica LISP**

VantaDB aspira a ser más que un almacén de datos; busca ser un motor de inferencia cognitiva. El análisis de Neo4j proporciona el plano de lo que es posible y lo que es necesario mejorar para alcanzar esta visión.

### **Rust como Motor de Alto Rendimiento y Seguridad**

La elección de Rust para VantaDB es una ventaja competitiva directa frente a la arquitectura de Neo4j basada en la JVM. Rust permite el control granular sobre el diseño de la memoria, lo que facilita la implementación de estructuras de datos que imitan la densidad sináptica. Podemos implementar el "Block Format" de Neo4j 5 pero con una gestión de punteros inteligentes y tipos de datos que garanticen la ausencia de condiciones de carrera (*race conditions*) sin necesidad de bloqueos globales pesados. La capacidad de Rust para interactuar directamente con instrucciones de CPU (como AVX-512) permitirá que el recorrido del grafo y las operaciones vectoriales ocurran en el mismo pipeline de ejecución.3

### **Integración de Lógica LISP en el AST**

Neo4j utiliza una estructura interna para Cypher que es esencialmente un lenguaje de árbol. LISP, por su naturaleza, se basa en la manipulación de listas y árboles (S-expressions). En VantaDB, el motor de consultas podría ser un intérprete o compilador LISP que opere directamente sobre la estructura del grafo. Esto permitiría una flexibilidad asombrosa: las "reglas" de inferencia neurobiológica podrían ser funciones LISP almacenadas como nodos en el propio grafo, permitiendo que la base de datos "aprenda" nuevas lógicas de conexión mientras opera.

### **Mapeo Neurobiológico y Plasticidad**

Mientras que Neo4j tiene un esquema flexible pero estructuras físicas rígidas (registros de tamaño fijo), VantaDB puede implementar una verdadera plasticidad sináptica. Podríamos utilizar un modelo de almacenamiento donde las relaciones tengan "pesos" que se actualicen dinámicamente según la frecuencia de acceso, similar a la potenciación a largo plazo en el cerebro. La gestión de memoria en Rust permitiría que estas actualizaciones de pesos sean extremadamente rápidas, utilizando estructuras de datos como matrices de adyacencia comprimidas (CSR) que se reordenan automáticamente para optimizar la localidad de caché.38

## **Puntos Débiles y Limitaciones de Neo4j: Lecciones para el Futuro**

Ningún análisis de ingeniería está completo sin identificar las fallas estructurales que limitan el sistema. Neo4j, a pesar de su madurez, posee vulnerabilidades inherentes a su legado y su entorno de ejecución.10

### **Dependencia de la RAM y el Precipicio de Rendimiento**

El mayor punto débil de Neo4j es su dependencia absoluta de que el conjunto de datos activo quepa en la RAM. Cuando una consulta requiere acceder a datos que han sido expulsados del Page Cache hacia el disco, la latencia aumenta dramáticamente (hasta 10-100 veces). El sistema no maneja bien los conjuntos de datos que son masivamente más grandes que la memoria disponible si el patrón de acceso es altamente aleatorio. Para VantaDB, esto sugiere la necesidad de un motor de almacenamiento que sea "consciente del almacenamiento persistente" desde el primer día, utilizando técnicas de precarga inteligente y estructuras de datos que minimicen el radio de búsqueda en disco.10

### **La Contención de Escritura en Arquitecturas Master-Slave**

Neo4j utiliza una arquitectura donde todas las escrituras deben pasar por un único nodo líder (en un cluster causal). Aunque esto garantiza la consistencia ACID, crea un cuello de botella de escalabilidad vertical. En un grafo altamente dinámico con miles de actualizaciones por segundo, el líder se convierte en el limitante. VantaDB debería investigar arquitecturas de escritura distribuida o modelos de consistencia eventual/causal más granulares para permitir que diferentes "regiones" del conectoma se actualicen en paralelo sin bloquear todo el sistema.10

### **El Riesgo de los Supernodos y el Recorrido Lineal**

Los supernodos siguen siendo la "criptonita" de las bases de datos de grafos nativas. En Neo4j, cuando un nodo tiene millones de relaciones de un mismo tipo, el motor debe iterar secuencialmente a través de la lista enlazada para encontrar una conexión específica, a menos que se use un índice de propiedad de relación (introducido recientemente pero con un alto coste de almacenamiento). VantaDB podría mitigar esto utilizando estructuras de "relaciones indexadas" nativas, donde las aristas de un nodo se almacenen en una estructura de árbol o tabla hash local al nodo, permitiendo búsquedas de vecinos en tiempo ![][image1] incluso para nodos masivamente densos.3

### **Reutilización de IDs y Fragilidad de Referencias**

Neo4j reutiliza los IDs de nodos y relaciones internos. Cuando un nodo es eliminado, su ID queda libre y será asignado al siguiente nodo creado. Esto es extremadamente peligroso si las aplicaciones externas o los registros de auditoría guardan estos IDs. Una referencia antigua podría apuntar repentinamente a un nodo de un tipo completamente diferente. VantaDB debe implementar identificadores inmutables y únicos (como UUIDs o UIDs basados en tiempo) en el nivel más bajo del motor de almacenamiento, sacrificando unos pocos bytes por registro para garantizar la integridad referencial a largo plazo.39

| Punto Débil de Neo4j | Consecuencia Técnica | Oportunidad para VantaDB |
| :---- | :---- | :---- |
| Recolección de Basura (GC) | Latencia impredecible (Jitters) | Gestión manual de memoria en Rust para latencia ultra-baja. |
| Bloqueo de Escritura en Líder | Cuello de botella en ingesta masiva | Diseño de motor de escritura multihilo sin bloqueos (Lock-free). |
| Recorrido Lineal de Relaciones | Degradación por supernodos | Estructuras de adyacencia indexadas localmente (Local Indexing). |
| Reutilización de IDs | Riesgo de corrupción lógica | Identificadores inmutables de 128 bits integrados. |

## **Conclusión y Recomendaciones de Ingeniería**

Neo4j ha definido el estándar de oro para la Adyacencia Libre de Índices, demostrando que la natividad en el almacenamiento es la única forma de lograr un rendimiento de grafos real. Sin embargo, su implementación sobre la JVM y su dependencia de estructuras de datos de registros fijos limitan su capacidad para evolucionar hacia un sistema cognitivo verdaderamente fluido.

Para VantaDB, el camino a seguir implica adoptar la filosofía de punteros físicos de Neo4j pero liberarla de las restricciones del montón de Java. La integración de vectores mediante HNSW con filtrado en el índice es un requisito no negociable para la IA moderna. La verdadera innovación de VantaDB residirá en la simbiosis entre el grafo estructural y la lógica simbólica LISP, permitiendo que las consultas no sean solo búsquedas de datos, sino procesos de razonamiento dinámico que ocurran a la velocidad del hardware nativo.

Al evitar los errores de Neo4j con los supernodos y la reutilización de identificadores, y al aprovechar el rendimiento superior de Rust para el procesamiento paralelo y el acceso a memoria de bajo nivel, VantaDB tiene el potencial de convertirse en la infraestructura definitiva para la computación neurobiológicamente inspirada, superando las limitaciones de los sistemas de grafos tradicionales y abriendo la puerta a una nueva era de bases de datos inteligentes.

#### **Obras citadas**

1. Understanding Neo4j's data on disk \- Knowledge Base, fecha de acceso: abril 7, 2026, [https://neo4j.com/developer/kb/understanding-data-on-disk/](https://neo4j.com/developer/kb/understanding-data-on-disk/)  
2. How is data stored in a graph database? \[duplicate\] \- Stack Overflow, fecha de acceso: abril 7, 2026, [https://stackoverflow.com/questions/48777704/how-is-data-stored-in-a-graph-database](https://stackoverflow.com/questions/48777704/how-is-data-stored-in-a-graph-database)  
3. Cloud-Native Graph Database Architecture: A Deep Dive | by L's Representation | Medium, fecha de acceso: abril 7, 2026, [https://medium.com/@luxianlong/cloud-native-graph-database-architecture-a-deep-dive-350cbdda4750](https://medium.com/@luxianlong/cloud-native-graph-database-architecture-a-deep-dive-350cbdda4750)  
4. Neo4j Super Node Performance Issues \- Justin Boylan-Toomey, fecha de acceso: abril 7, 2026, [https://jboylantoomey.com/post/neo4j-super-node-performance-issues](https://jboylantoomey.com/post/neo4j-super-node-performance-issues)  
5. Store formats \- Operations Manual \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/operations-manual/current/database-internals/store-formats/](https://neo4j.com/docs/operations-manual/current/database-internals/store-formats/)  
6. Try Neo4j's Next-Gen Graph-Native Store Format, fecha de acceso: abril 7, 2026, [https://neo4j.com/blog/developer/neo4j-graph-native-store-format/](https://neo4j.com/blog/developer/neo4j-graph-native-store-format/)  
7. Native vs. Non-Native Graph Database Architecture & Technology, fecha de acceso: abril 7, 2026, [https://neo4j.com/blog/cypher-and-gql/native-vs-non-native-graph-technology/](https://neo4j.com/blog/cypher-and-gql/native-vs-non-native-graph-technology/)  
8. Vector search with filters in Neo4j v2026.01 (Preview), fecha de acceso: abril 7, 2026, [https://neo4j.com/blog/genai/vector-search-with-filters-in-neo4j-v2026-01-preview/](https://neo4j.com/blog/genai/vector-search-with-filters-in-neo4j-v2026-01-preview/)  
9. Neo4j Vector Index and Search \- Developer Guides, fecha de acceso: abril 7, 2026, [https://neo4j.com/developer/genai-ecosystem/vector-search/](https://neo4j.com/developer/genai-ecosystem/vector-search/)  
10. What is Neo4j Architecture? Can anyone explain the Neo4J Architecture with a diagram? \- Quora, fecha de acceso: abril 7, 2026, [https://www.quora.com/What-is-Neo4j-Architecture-Can-anyone-explain-the-Neo4J-Architecture-with-a-diagram](https://www.quora.com/What-is-Neo4j-Architecture-Can-anyone-explain-the-Neo4J-Architecture-with-a-diagram)  
11. The Neighborhood Walk Story. Index-free adjacency is the most… | by Dan McCreary, fecha de acceso: abril 7, 2026, [https://dmccreary.medium.com/how-to-explain-index-free-adjacency-to-your-manager-1a8e68ec664a](https://dmccreary.medium.com/how-to-explain-index-free-adjacency-to-your-manager-1a8e68ec664a)  
12. IVFFlat vs HNSW in pgvector: Which Index Should You Use? \- DEV Community, fecha de acceso: abril 7, 2026, [https://dev.to/philip\_mcclarence\_2ef9475/ivfflat-vs-hnsw-in-pgvector-which-index-should-you-use-305p](https://dev.to/philip_mcclarence_2ef9475/ivfflat-vs-hnsw-in-pgvector-which-index-should-you-use-305p)  
13. Vector Indexes: HNSW vs IVFFLAT vs IVF\_RaBitQ \- Kodesage, fecha de acceso: abril 7, 2026, [https://kodesage.ai/blog/vector-indexes-hnsw-vs-ivfflat-vs-ivf-rabitq](https://kodesage.ai/blog/vector-indexes-hnsw-vs-ivfflat-vs-ivf-rabitq)  
14. Vectors \- Cypher Manual \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/cypher-manual/current/values-and-types/vector/](https://neo4j.com/docs/cypher-manual/current/values-and-types/vector/)  
15. Vector optimization \- Neo4j Aura, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/aura/managing-instances/vector-optimization/](https://neo4j.com/docs/aura/managing-instances/vector-optimization/)  
16. Vector index memory configuration \- Operations Manual \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/operations-manual/current/performance/vector-index-memory-configuration/](https://neo4j.com/docs/operations-manual/current/performance/vector-index-memory-configuration/)  
17. Vector search index pre-filtered query \- Neo4j Graph Platform, fecha de acceso: abril 7, 2026, [https://community.neo4j.com/t/vector-search-index-pre-filtered-query/64465](https://community.neo4j.com/t/vector-search-index-pre-filtered-query/64465)  
18. \[FEA\] Roaring Bitmap support · Issue \#1972 · rapidsai/cuvs \- GitHub, fecha de acceso: abril 7, 2026, [https://github.com/rapidsai/cuvs/issues/1972](https://github.com/rapidsai/cuvs/issues/1972)  
19. Efficient large-scale filtering with bitmap filtering in OpenSearch, fecha de acceso: abril 7, 2026, [https://opensearch.org/blog/introduce-bitmap-filtering-feature/](https://opensearch.org/blog/introduce-bitmap-filtering-feature/)  
20. Memory configuration \- Operations Manual \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/operations-manual/current/performance/memory-configuration/](https://neo4j.com/docs/operations-manual/current/performance/memory-configuration/)  
21. Understanding memory consumption \- Knowledge Base \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/developer/kb/understanding-memory-consumption/](https://neo4j.com/developer/kb/understanding-memory-consumption/)  
22. Tuning of the garbage collector \- Operations Manual \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/operations-manual/current/performance/gc-tuning/](https://neo4j.com/docs/operations-manual/current/performance/gc-tuning/)  
23. Neo4j Guide \- IBM, fecha de acceso: abril 7, 2026, [https://www.ibm.com/docs/en/manta-data-lineage?topic=management-neo4j-guide](https://www.ibm.com/docs/en/manta-data-lineage?topic=management-neo4j-guide)  
24. Neo4j Page Cache \- Ken Wagatsuma's Homepage, fecha de acceso: abril 7, 2026, [https://kenwagatsuma.com/blog/neo4j-page-cache](https://kenwagatsuma.com/blog/neo4j-page-cache)  
25. Disks, RAM and other tips \- Operations Manual \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/operations-manual/current/performance/disks-ram-and-other-tips/](https://neo4j.com/docs/operations-manual/current/performance/disks-ram-and-other-tips/)  
26. Query tuning \- Cypher Manual \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/cypher-manual/current/planning-and-tuning/query-tuning/](https://neo4j.com/docs/cypher-manual/current/planning-and-tuning/query-tuning/)  
27. Bolt Protocol message specification \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/bolt/current/bolt/message/](https://neo4j.com/docs/bolt/current/bolt/message/)  
28. Bolt Protocol \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/bolt/current/bolt/](https://neo4j.com/docs/bolt/current/bolt/)  
29. PackStream \- Bolt Protocol \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/bolt/current/packstream/](https://neo4j.com/docs/bolt/current/packstream/)  
30. Bolt (network protocol) \- Wikipedia, fecha de acceso: abril 7, 2026, [https://en.wikipedia.org/wiki/Bolt\_(network\_protocol)](https://en.wikipedia.org/wiki/Bolt_\(network_protocol\))  
31. Structure Semantics \- Bolt Protocol \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/bolt/current/bolt/structure-semantics/](https://neo4j.com/docs/bolt/current/bolt/structure-semantics/)  
32. Bolt message state transitions in version 5.1 \- Bolt Protocol \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/bolt/current/appendix/version-5/](https://neo4j.com/docs/bolt/current/appendix/version-5/)  
33. Introducing the New Cypher Query Optimizer \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/blog/cypher-and-gql/introducing-new-cypher-query-optimizer/](https://neo4j.com/blog/cypher-and-gql/introducing-new-cypher-query-optimizer/)  
34. Tuning Your Cypher: Tips & Tricks for More Effective Queries \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/blog/cypher-and-gql/tuning-cypher-queries/](https://neo4j.com/blog/cypher-and-gql/tuning-cypher-queries/)  
35. Query Optimization in Neo4j: Four Key Techniques to Supercharge Your Cypher Queries | by Himanshu Jha | Medium, fecha de acceso: abril 7, 2026, [https://medium.com/@jhahimanshu3636/query-optimization-in-neo4j-four-key-techniques-to-supercharge-your-cypher-queries-cf38aa5c7122](https://medium.com/@jhahimanshu3636/query-optimization-in-neo4j-four-key-techniques-to-supercharge-your-cypher-queries-cf38aa5c7122)  
36. Cypher query options \- Neo4j, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/cypher-manual/4.4/query-tuning/query-options/](https://neo4j.com/docs/cypher-manual/4.4/query-tuning/query-options/)  
37. Cypher Performance Improvements in Neo4j 5, fecha de acceso: abril 7, 2026, [https://neo4j.com/blog/developer/cypher-performance-neo4j-5/](https://neo4j.com/blog/developer/cypher-performance-neo4j-5/)  
38. GDS Feature Toggles \- Neo4j Graph Data Science, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/graph-data-science/current/production-deployment/feature-toggles/](https://neo4j.com/docs/graph-data-science/current/production-deployment/feature-toggles/)  
39. Welcome to the Dark Side: Neo4j Worst Practices (& How to Avoid Them), fecha de acceso: abril 7, 2026, [https://neo4j.com/blog/cypher-and-gql/dark-side-neo4j-worst-practices/](https://neo4j.com/blog/cypher-and-gql/dark-side-neo4j-worst-practices/)  
40. How well does the Neo4j scale to tens or hundreds of gigabytes of data and tens or hundreds of millions of nodes/edges? \- Quora, fecha de acceso: abril 7, 2026, [https://www.quora.com/How-well-does-the-Neo4j-scale-to-tens-or-hundreds-of-gigabytes-of-data-and-tens-or-hundreds-of-millions-of-nodes-edges](https://www.quora.com/How-well-does-the-Neo4j-scale-to-tens-or-hundreds-of-gigabytes-of-data-and-tens-or-hundreds-of-millions-of-nodes-edges)  
41. The Production-Ready Neo4j Guide: Performance Tuning and Best Practices | by Alish Satani | Medium, fecha de acceso: abril 7, 2026, [https://medium.com/@satanialish/the-production-ready-neo4j-guide-performance-tuning-and-best-practices-15b78a5fe229](https://medium.com/@satanialish/the-production-ready-neo4j-guide-performance-tuning-and-best-practices-15b78a5fe229)  
42. Parameter tuning \- Neo4j Spark, fecha de acceso: abril 7, 2026, [https://neo4j.com/docs/spark/current/performance/tuning/](https://neo4j.com/docs/spark/current/performance/tuning/)

[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAfCAYAAAClDZ5ZAAAC6klEQVR4Xu2Yy6tPURTHl1BEHtGVKJekPEq6SsSMMjEhRRQxIBkZ8AfI6E6QJCkhkQFJSjK4I4mRIqWUK1IkpQyQx/dz99mcs37nnN85P7/H5PetT669zn6svdZ+/cz66quvXmuc2C5uianOVlWrxD0x3xuqarxYI86JV+JNwvWknEE20zbxVCzwhppquZ1l4rEYFXvFLAuOLRE3xS8LDk5Ovs/TcguOb/IGp2nirDjvDSkxaSfFfasYWSocFj/FFTElax7TRAud/hbHLT8yfHMjgb+9iOhRcdtCX7R1KfNFoxaLt+KgN3gxIBqnUQaaN4CoReK9+CxWOBvaIL6Izd6QaL+FKOwWw1bNEcZ3RjwTA86WEXnI7JBSpFKZiNQDCwM45mx0eFE8ETOdLU/Ur+II2mhhjDu8IYp8/mgh97c6W5HoOG8A88RrcdqVF6mOI7FtJqohpSeICxYaeySmZ825miTuWqgzYtkFyKwxIbtSZWWq40js97mY7WxjOU6u56VJkUgZUoc6Vy07O0fED7E+VVamOo4gIs36W+kNByw0VKfzpeKThXonnI3ofhVDrrxIdR3heyJO5P+KmWRGaYhDb07aWCLShjo06HcmBjQq5rryItV1JPa9JV1Ibo8kBv6tctik18cLa3S+047gQFscWSu+WajjD6cY4a47gmigqiPxxOZ7zpG8Ha7TEclNLRQXe+6W5hQPzZdWfIHryWJHCy3cYZodhussbNMcnKudLa1ubL/smOycDUrPNJeztLj17hPfxUMxmLE2qsqBSAqTerwxuHPhyJ3k/5TP+PdpRnGjKbz+sEi5br+zMODLYo84JT4k7LTgVDPFCJddUWIUiiiKTryicHlkzIViMZM2hywMhMvZoDWp5MSV55qVzNp/iGgz0f7s6pjoqN0dMplEoup9sC3i5UjOFz2sWlF8WPGG6ar40YDHV7OnbhURDZ667ZyYWmr5RwOndrXTspjJnv8c1FdfLeoPfE/F57x+H2EAAAAASUVORK5CYII=>

[image2]: <data:image/png;base64,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>

[image3]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA0AAAAgCAYAAADJ2fKUAAABLElEQVR4Xu2SvSvGURTHv8LiLVJeShllMshkxGhXRpuMymaxKJtRSpRNySJiUxarf8AgGVCKzcvn/M7v6t7r9zw9RvV86lO3e8653XPulZr8oh1ncBl3cBN7k4wK+vECn/ELr7A7yajDorxoOw/UY1deZMUN0Yc3+ITjWawmk/gmL7QDGqKqH5vqfLk3He0XtOCe0n6G8AQ3cB1vcbCMFeT9jOJxuV6QH3aHw6HAiPuZwiN5obEkL7JHt+v+EPp5wDMciYO1+HNRPIQX/MB7XFF2nZh8CD14KD9kLcpLmMBXpZ/UhvGO59gh/9Cr8gMLqh7VHtT27C8as7iPbSHBkvNPakmf8utZXwc4F4KdeImPOBY2oQtP5dO8xi1lQ7EEM6cVB+T92ISb/CO+AauiQTPgpSogAAAAAElFTkSuQmCC>

[image4]: <data:image/png;base64,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>

[image5]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABEAAAAYCAYAAAAcYhYyAAABM0lEQVR4Xu3SvytFYRzH8a9QdFNEGaibsjBYJIvBQFnUdbvDvVlsFpuBZDEpmSg2/gFlUgYbi8x2g9WoSH68P+f7PPc8HZO7up96ndP5nud5znO+PWbttPNv040GjjCLjmASBzhBDb1hvO56Vl3vs2ziEPv4xArOcIkKtkP9FFO4wy7mcQEbwzWGsYxvvGHNfDdKH27NF7rHeKgrM7rUcWw+Ycd8EX2xKx9nA3gwX2QxqStzupSDHlzhI75IMoEX891oV2m20odRPOMRQ+kLsmq+QzU+TfxwMwv4Mm9U+iuKJmuRaqEed9hM7Md6WrS8HxqsSWk2zOdkaaUfJdzgPRZG8GR/64fGabx2mWUarzi3/GzE7Jn3aqlQ1zgdUs3L0mm+so5/MfrVQfu9eEx/sdBSfgAImDyqN/rzqwAAAABJRU5ErkJggg==>
