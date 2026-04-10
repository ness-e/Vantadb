# **Ingeniería Inversa de ArangoDB: Implicaciones Arquitectónicas para el Desarrollo de ConnectomeDB**

La convergencia de modelos de datos en una única plataforma de almacenamiento representa uno de los mayores desafíos de la ingeniería de sistemas contemporánea. ArangoDB ha emergido como una solución predominante en el espacio multimodelo, integrando documentos, grafos y capacidades de búsqueda vectorial. Para el diseño de ConnectomeDB, una base de datos cognitiva inspirada en la neurobiología y escrita en Rust, el análisis de ArangoDB ofrece una hoja de ruta crítica sobre los compromisos de rendimiento, la gestión de la consistencia y la eficiencia de la serialización en disco. Este informe desglosa la infraestructura interna de ArangoDB, desde su sustrato de almacenamiento basado en RocksDB hasta sus algoritmos de navegación de grafos, con el fin de extraer lecciones aplicables a una arquitectura de alto rendimiento y baja latencia.

## **Anatomía de la "Neurona": Estructura de Datos e Infraestructura de Almacenamiento**

En el contexto de una base de datos cognitiva, la "neurona" es la unidad fundamental de información. En ArangoDB, esta unidad se manifiesta como un documento JSON persistido en un formato binario propietario. A diferencia de los sistemas puramente columnares o de clave-valor, ArangoDB adopta un enfoque híbrido que prioriza la flexibilidad del esquema sin sacrificar excesivamente el rendimiento de acceso a sub-objetos.

### **El Sustrato de Almacenamiento: RocksDB y Árboles LSM**

ArangoDB utiliza RocksDB como su motor de almacenamiento por defecto en las versiones modernas, abandonando el antiguo motor MMFiles basado en archivos mapeados en memoria.1 RocksDB es un motor de almacenamiento de clave-valor integrable, optimizado para unidades de estado sólido (SSD) y memoria persistente, que implementa una estructura de datos de árbol de mezcla estructurado por registros (LSM-Tree).3

La arquitectura de RocksDB organiza los datos en múltiples niveles, lo que influye directamente en cómo ArangoDB gestiona la persistencia de los documentos. Cuando se realiza una escritura, la información se registra primero en un Registro de Escritura Anticipada (WAL) para garantizar la durabilidad y simultáneamente se inserta en una estructura en memoria denominada MemTable.3 Una vez que la MemTable alcanza un umbral de tamaño, se convierte en inmutable y se vuelca al disco como un archivo de Tabla de Cadenas Ordenadas (SSTable) en el Nivel 0 (L0).3

| Componente de Almacenamiento | Función Principal | Impacto en Latencia |
| :---- | :---- | :---- |
| **Write-Ahead Log (WAL)** | Garantizar durabilidad ante fallos del sistema. | Alta (I/O secuencial).3 |
| **MemTable** | Buffer de escritura en RAM para acceso rápido. | Muy baja (acceso a memoria).3 |
| **Block Cache** | Almacenar bloques de datos SST descomprimidos. | Baja (evita lecturas de disco).3 |
| **SSTables (Nivel 0-N)** | Almacenamiento persistente y organizado por niveles. | Variable (depende de la compactación).3 |

El proceso de compactación es el mecanismo mediante el cual RocksDB combina y purga datos obsoletos, moviendo los registros hacia niveles superiores (L1, L2, etc.).3 Para ConnectomeDB, este comportamiento es análogo a la consolidación de la memoria a largo plazo, donde los datos se reorganizan para optimizar la lectura a expensas de un procesamiento de fondo intensivo.

### **Gestión de Metadatos y el Identificador de Objeto**

Cada objeto o "neurona" en ArangoDB posee metadatos intrínsecos que permiten su localización y control de versiones. Estos metadatos no son meros atributos adicionales, sino componentes fundamentales del motor de ejecución.

1. **\_key**: Un identificador único dentro de una colección, proporcionado por el usuario o generado automáticamente.6  
2. **\_id**: Un identificador global que combina el nombre de la colección y la clave (coleccion/clave), permitiendo el direccionamiento único en todo el espacio de nombres de la base de datos.6  
3. **\_rev**: Un identificador de revisión utilizado para el Control de Concurrencia Multiversión (MVCC).6 Este campo es crítico para detectar conflictos de escritura-escritura sin necesidad de bloqueos globales.

Los metadatos se almacenan junto con el cuerpo del documento en el formato de serialización, pero se indexan por separado para permitir búsquedas primarias instantáneas. ArangoDB mantiene un índice primario de tabla hash para cada colección, vinculando la \_key con la ubicación física (u offset) del documento en el motor RocksDB.1

### **Serialización: El Formato VelocyPack (VPack)**

Uno de los logros de ingeniería más significativos de ArangoDB es VelocyPack (VPack), su formato de serialización binaria schemaless.9 VPack fue diseñado para superar las ineficiencias de JSON (que requiere parseo completo), BSON (que es ineficiente en espacio para arreglos) y MessagePack (que carece de acceso rápido a sub-objetos).9

VPack permite que los datos se utilicen en la misma secuencia de bytes para el transporte, el almacenamiento y el trabajo en memoria.9 La característica más innovadora de VPack es su capacidad de acceso aleatorio a los atributos de un objeto sin necesidad de deserializar todo el documento. Esto se logra mediante una tabla de offsets interna al final del blob binario, que actúa como un índice local para las claves del objeto.9

| Formato | Acceso a Sub-objetos | Compactación | Soporte de Tipos |
| :---- | :---- | :---- | :---- |
| **JSON** | No (requiere parseo) | Baja | Limitado (solo strings, números, null) |
| **BSON** | Limitado | Media | Extenso |
| **MessagePack** | No | Alta | Bueno |
| **VelocyPack** | **Sí (mediante offsets)** | **Alta** | **Total (incluye fechas y precisión arbitraria)**.9 |

Para una arquitectura en Rust como ConnectomeDB, la lógica de VPack es una referencia esencial. Permite implementar un sistema de "copia cero" parcial, donde el motor de consulta puede inspeccionar campos específicos (como el peso de una sinapsis o una etiqueta lógica) simplemente saltando a un offset específico en un buffer de memoria, lo cual es fundamental para el procesamiento de grafos a gran escala.

## **Lógica de Recuperación y Búsqueda: De Vectores a Grafos**

La capacidad de ConnectomeDB para funcionar como una base de datos cognitiva depende de la eficiencia con la que puede recuperar patrones similares (vectores) y navegar relaciones complejas (grafos). ArangoDB resuelve esto integrando ambos paradigmas en su lenguaje de consulta AQL y utilizando estructuras de indexación especializadas.

### **Indexación Vectorial: HNSW y Compromisos de Rendimiento**

ArangoDB ha integrado capacidades de búsqueda vectorial mediante el algoritmo de Mundo Pequeño Navegable Jerárquico (HNSW).11 HNSW es un algoritmo de Vecinos Más Cercanos Aproximados (ANN) que construye una estructura de grafo multicapa sobre los vectores de alta dimensionalidad.12

El rendimiento de la búsqueda vectorial en ArangoDB está gobernado por la relación entre el "Recall" (la fracción de vecinos verdaderos encontrados) y la latencia de la consulta.12 Durante la fase de construcción del índice, ArangoDB permite configurar parámetros críticos que determinan la densidad del grafo:

* **M**: El número máximo de conexiones para cada elemento en cada capa del grafo. Un valor de ![][image1] más alto mejora el recall en grafos con alta dimensionalidad, pero aumenta significativamente el consumo de RAM y el tiempo de indexación.12  
* **efConstruction**: Define el tamaño de la lista de candidatos explorados durante la construcción del índice. Un valor mayor resulta en un grafo de mayor calidad pero a un costo computacional elevado.12  
* **efSearch**: Un parámetro de tiempo de consulta que determina cuántos nodos se exploran. Aumentar efSearch permite recuperar precisión perdida durante la fase de construcción, incrementando la latencia de manera lineal o superlineal.12

La complejidad de búsqueda en HNSW es aproximadamente ![][image2], lo que permite escalar a millones de vectores.13 Sin embargo, a medida que el corpus crece, la presión sobre la caché de la CPU aumenta debido a los saltos aleatorios de memoria necesarios para navegar el grafo de vectores, lo que puede degradar el rendimiento si el índice no cabe completamente en RAM.14

### **Resolución de Saltos en Grafos: ¿Index-free Adjacency o Edge Index?**

Una de las discusiones más profundas en la arquitectura de bases de datos de grafos es el uso de Adyacencia Libre de Índices (IFA) frente a índices de bordes (Edge Indexes). Los sistemas nativos como Neo4j utilizan IFA, donde cada nodo contiene punteros físicos a sus vecinos.15 ArangoDB, sin embargo, emplea un enfoque de "Índice de Bordes Híbrido".17

En ArangoDB, los bordes son documentos completos almacenados en colecciones especiales que contienen los atributos obligatorios \_from y \_to.7 El motor mantiene un índice hash optimizado sobre estos atributos, lo que permite localizar todos los bordes entrantes o salientes de un nodo en tiempo constante ![][image3].10

Aunque puristas del grafo argumentan que un índice hash no es "nativamente" libre de índices, la realidad técnica en sistemas distribuidos es que IFA presenta dificultades extremas para el escalado horizontal, ya que los punteros de memoria física no son válidos a través de los límites de la red.17 El enfoque de ArangoDB permite que el grafo se distribuya en múltiples fragmentos (shards) mientras mantiene un rendimiento de salto de nodo extremadamente competitivo, facilitado por técnicas como los SmartGraphs que co-localizan vértices relacionados en el mismo servidor físico.2

### **Filtrado Híbrido y Búsqueda Semántica**

ArangoDB implementa el filtrado híbrido a través de su motor ArangoSearch, que combina capacidades de recuperación de información (IR) con modelos de datos existentes.11 Este motor utiliza índices invertidos para búsquedas de texto y puede integrarse con búsquedas vectoriales en una única ejecución de AQL.11

La lógica de ejecución para una consulta híbrida (ej. "buscar vectores similares pero que contengan la palabra 'neurobiología'") sigue un plan de ejecución optimizado donde el optimizador de AQL decide si aplicar primero el filtro de texto o la búsqueda vectorial basándose en la selectividad estimada de los índices.11 Si se utiliza el modo SearchType.HYBRID, ArangoDB emplea la Fusión de Rango Recíproco (RRF) para combinar las puntuaciones de relevancia del motor de texto y las puntuaciones de distancia del motor vectorial, proporcionando un resultado unificado.23

## **Gestión de Memoria y Estado: El Desafío de la Concurrencia**

Para ConnectomeDB, el manejo de la memoria en Rust debe ser superior al modelo de C++ de ArangoDB, que a menudo sufre de un consumo de RAM excesivo y picos de carga impredecibles.

### **Estrategias de Caché y Compatibilidad Zero-Copy**

ArangoDB es una base de datos de tipo "mostly-memory", lo que significa que el rendimiento óptimo se alcanza cuando el conjunto de datos de trabajo reside en la memoria principal.1 El sistema gestiona múltiples capas de caché:

1. **Caché de Bloques de RocksDB**: Almacena datos comprimidos y descomprimidos leídos del disco.3  
2. **Caché de Consultas de AQL**: Almacena resultados de consultas deterministas para evitar re-ejecuciones.25  
3. **Caché de Sistema Operativo**: Al apoyarse en archivos, ArangoDB se beneficia de la caché de páginas del kernel para acelerar los accesos repetidos a los archivos SST.3

En cuanto a la arquitectura "Zero-Copy", ArangoDB no es compatible de forma nativa con formatos como Apache Arrow para el almacenamiento interno, aunque su formato VelocyPack persigue objetivos similares al minimizar las copias durante el acceso a sub-objetos.9 Sin embargo, la falta de una integración nativa con Arrow limita su eficiencia en flujos de trabajo de análisis masivo (OLAP) donde el intercambio de buffers entre procesos es vital.27 ConnectomeDB, al ser desarrollado en Rust, tiene la oportunidad de utilizar Apache Arrow o FlatBuffers como su representación interna de memoria de primera clase, logrando una eficiencia de "copia cero" real que ArangoDB solo emula parcialmente.

### **Concurrencia y Bloqueos en Escrituras Masivas**

La gestión de la concurrencia en ArangoDB depende del motor de almacenamiento seleccionado. En el motor RocksDB, se utiliza un modelo de bloqueo optimista a nivel de documento.1 Las transacciones no bloquean la lectura de otros hilos, ya que cada transacción opera sobre un "Snapshot" (instantánea) consistente del estado de la base de datos en un punto en el tiempo determinado por un número de secuencia.8

| Escenario de Concurrencia | Mecanismo de ArangoDB | Riesgo Asociado |
| :---- | :---- | :---- |
| **Lectura-Lectura** | Libre de bloqueos (Shared snapshots). | Ninguno. |
| **Lectura-Escritura** | No bloqueante (Lectura de versión antigua). | Lecturas ligeramente desactualizadas.8 |
| **Escritura-Escritura** | Detección de conflictos en commit (RocksDB). | Aborto de transacción por conflicto de revisión.1 |
| **Escrituras Masivas** | Compactación de fondo y WAL. | "Thread explosion" y aumento de latencia de I/O.33 |

Un problema reportado comúnmente en entornos de alta concurrencia es la proliferación de hilos (thread spikes) cuando el sistema intenta procesar miles de peticiones HTTP simultáneas, lo que puede llevar al agotamiento de los descriptores de archivos y a la inestabilidad del sistema.33 ConnectomeDB puede mitigar esto mediante el uso de modelos de concurrencia basados en actores o tareas asíncronas (Tokio en Rust) con un pool de hilos controlado.

### **El Mecanismo de "Olvido": TTL y Compactación Inteligente**

Desde una perspectiva cognitiva, el olvido es tan importante como la memorización. ArangoDB implementa esto mediante:

* **Índices TTL (Time-To-Live)**: Permiten marcar documentos para su eliminación automática después de un periodo de tiempo determinado.34  
* **Compactación de RocksDB**: Es el proceso físico de "olvido" donde los registros marcados para eliminación (tombstones) son finalmente eliminados de los archivos SST.3

ArangoDB permite configurar estrategias de compactación personalizadas que pueden ser vistas como una forma de "mantenimiento sináptico". Por ejemplo, se puede priorizar la compactación de ciertos niveles para liberar espacio en disco más rápido, aunque esto incremente la amplificación de escritura (![][image4]).3

## **Análisis de la Documentación y API: La Experiencia del Desarrollador**

El éxito de una base de datos depende tanto de su núcleo técnico como de su interfaz con el mundo exterior. AQL y el ecosistema de SDKs de ArangoDB definen su usabilidad.

### **AQL: Un Lenguaje Declarativo para la Multimodalidad**

AQL es un lenguaje de consulta puramente de manipulación de datos (DML).38 Su sintaxis se aleja del estándar SQL para adoptar una estructura más cercana a la programación funcional o imperativa moderna, utilizando cláusulas FOR, FILTER, LET, COLLECT y RETURN.18

Su flexibilidad radica en la capacidad de combinar modelos en una sola sentencia. Un ejemplo de su potencia es la navegación de grafos con filtrado dinámico:

Fragmento de código

FOR v, e, p IN 1..3 OUTBOUND 'neurons/source' GRAPH 'connectome'  
  FILTER v.activation \> 0.8  
  SEARCH p.edges\[\*\].type \== 'excitatory'  
  RETURN { neuron: v.id, path: p }

Esta sintaxis es excepcionalmente expresiva para modelar circuitos neuronales, permitiendo definir profundidad de búsqueda y condiciones de poda (PRUNE) que detienen la exploración de ramas irrelevantes, optimizando drásticamente el rendimiento en grafos densos.40

### **Innovaciones en el SDK: Foxx y Microservicios en la Base de Datos**

Quizás la característica más innovadora del ecosistema de ArangoDB es el framework **Foxx**.43 Foxx permite escribir lógica de negocio en JavaScript que se ejecuta directamente dentro del proceso de la base de datos, sobre el motor V8.43

Esto elimina la latencia de red en operaciones complejas de "leer-modificar-escribir", permitiendo que la base de datos actúe como un servidor de aplicaciones integrado. Para ConnectomeDB, esto sugiere la implementación de un sistema similar de "procedimientos almacenados" pero utilizando WebAssembly (Wasm), lo que permitiría ejecutar lógica en Rust o LISP a velocidad nativa dentro del motor de la base de datos sin los riesgos de seguridad y rendimiento de un motor de JS completo.

### **Fracasos y Errores Comunes en el Mundo Real**

El análisis de foros y reportes de errores revela debilidades estructurales en ArangoDB cuando se enfrenta a escalas extremas:

1. **Corrupción de Índices en Compactación**: Se han reportado casos raros donde la compactación de RocksDB encuentra claves fuera de orden, resultando en un estado de "solo lectura" forzado que requiere purgas manuales de archivos de datos.45  
2. **Consumo de RAM Impredecible**: La combinación de la caché de RocksDB, el motor V8 para Foxx y los buffers de red puede llevar a situaciones de Out-Of-Memory (OOM) si no se limitan estrictamente los parámetros de memoria.16  
3. **Latencia en la Interfaz Web (Aardvark)**: A medida que el número de colecciones y fragmentos crece, la consola de administración se vuelve lenta, consumiendo recursos significativos del servidor solo para renderizar estadísticas.25  
4. **Degradación del Índice Vectorial**: Al insertar datos de forma masiva en índices HNSW, la precisión del recall puede caer si no se reconstruye o reequilibra el índice periódicamente, un proceso que es costoso en CPU.13

## **Inspiración para ConnectomeDB: Funcionalidades Críticas a Extraer**

Basándose en la ingeniería inversa de ArangoDB, ConnectomeDB debe implementar las siguientes tres lógicas para asegurar su competitividad y superioridad técnica en el dominio cognitivo.

### **1\. Lógica de Poda de Grafos Dinámica (PRUNE)**

La cláusula PRUNE en AQL es una de las herramientas más potentes para manejar la explosión combinatoria en grafos.40 Permite detener una búsqueda en una rama específica tan pronto como se cumple una condición, sin descartar los resultados obtenidos hasta ese punto.

**Adaptación a ConnectomeDB (Rust)**:

En Rust, podemos implementar PRUNE utilizando iteradores perezosos (*lazy iterators*) y cierres (*closures*) que evalúan el estado de la ruta en tiempo real. Dado que ConnectomeDB usará LISP para su lógica, podríamos permitir que el usuario defina predicados LISP que el motor de ejecución en Rust compile a bytecode de Wasm para una evaluación ultrarrápida durante la travesía del grafo. Esto permitiría emular la inhibición sináptica biológica: si una ruta de señales es "demasiado débil", el motor de búsqueda aborta esa rama instantáneamente.

### **2\. SmartGraphs y Co-localización de Datos Relacionados**

ArangoDB Enterprise utiliza SmartGraphs para asegurar que los vértices conectados residan en el mismo fragmento, minimizando los "saltos de red" que destruyen el rendimiento de los grafos distribuidos.2

**Adaptación a ConnectomeDB (Rust)**:

ConnectomeDB debe adoptar un esquema de sharding basado en "Comunidades Lógicas". Utilizando algoritmos de detección de comunidades (como Louvain) durante el proceso de ingestión, ConnectomeDB puede asignar fragmentos basándose en la densidad de relaciones en lugar de simplemente aplicar un hash a la clave primaria. En Rust, esto se puede gestionar mediante un servicio de orquestación ligero que utilice Raft para mantener la topología de los fragmentos, asegurando que las "áreas funcionales" del cerebro cognitivo (grupos de datos relacionados) se procesen siempre en el mismo nodo físico para maximizar la localidad de la caché L3.

### **3\. Serialización de Acceso Aleatorio Tipo VPack con Seguridad de Memoria**

La capacidad de VPack para inspeccionar sub-objetos sin parseo es vital.9 Sin embargo, VPack es un formato de C++.

**Adaptación a ConnectomeDB (Rust)**:

ConnectomeDB debería utilizar una estructura de datos interna inspirada en rkyv o FlatBuffers, que permita mapear bytes de disco directamente a estructuras de Rust sin fase de deserialización. Esto proporcionaría un rendimiento de "copia cero" total. Al integrar la lógica LISP, los "S-expressions" podrían almacenarse en este formato binario, permitiendo que el intérprete LISP ejecute código directamente sobre la memoria mapeada. Esto reduciría drásticamente el uso de CPU y RAM, superando la implementación de ArangoDB que todavía requiere copias intermedias para pasar datos al motor V8.

## **Puntos Débiles: La Oportunidad de Mercado para ConnectomeDB**

ArangoDB, a pesar de su potencia, presenta flancos vulnerables que ConnectomeDB puede explotar para posicionarse como una solución superior.

### **El Problema de la "Gula de Memoria" y el Motor V8**

ArangoDB integra el motor JavaScript V8 para sus consultas y microservicios.43 Si bien esto ofrece flexibilidad, el costo en RAM es astronómico. V8 tiene su propio recolector de basura (GC), que compite con el motor de la base de datos y el sistema operativo por los recursos.16

**Oportunidad para ConnectomeDB**: Al eliminar JavaScript y optar por un núcleo puro en Rust con un intérprete LISP liviano o compilación a Wasm, ConnectomeDB puede funcionar con una fracción de la memoria requerida por ArangoDB. ConnectomeDB puede garantizar una gestión de memoria determinista, evitando las pausas del GC y permitiendo despliegues en hardware mucho más modesto o en el "edge" (dispositivos IoT, robótica), donde ArangoDB es simplemente demasiado pesado.47

### **Complejidad de Configuración y el "Ajuste de RocksDB"**

Configurar ArangoDB para que sea performante requiere ajustar cientos de parámetros de RocksDB (tamaño de MemTable, niveles de compactación, filtros de Bloom, etc.).3 Esta complejidad crea una barrera de entrada y un costo operativo alto.

**Oportunidad para ConnectomeDB**:

ConnectomeDB puede diferenciarse mediante la "Arquitectura Auto-Ajustable". En lugar de exponer parámetros de bajo nivel, ConnectomeDB puede utilizar su propia lógica cognitiva para monitorear patrones de acceso y ajustar dinámicamente sus estructuras de datos internas. Por ejemplo, si detecta una alta frecuencia de lecturas en una sub-red de grafos, podría decidir automáticamente elevar esos datos de una estructura LSM en disco a una estructura de adyacencia directa en RAM.

### **Dependencia de Licencias y el Límite de la Comunidad**

El cambio de ArangoDB a la licencia BSL 1.1 limita la libertad de los desarrolladores y establece techos artificiales (como el límite de 100GB en la edición comunitaria).43

**Oportunidad para ConnectomeDB**: Al ser un proyecto nuevo escrito en Rust, ConnectomeDB tiene la oportunidad de construir una comunidad desde cero con una licencia verdaderamente abierta (Apache 2.0). Ofreciendo nativamente características que en ArangoDB son "Enterprise" (como SmartGraphs o backups en caliente), ConnectomeDB puede atraer a la base de usuarios descontentos con la dirección comercial de ArangoDB, posicionándose como la alternativa de alto rendimiento y libre de restricciones.43

## **Conclusiones Técnicas y Estratégicas**

ArangoDB es un triunfo de la ingeniería multimodelo, pero su arquitectura arrastra el peso de decisiones tomadas hace una década. Su dependencia de RocksDB le otorga una base sólida pero rígida, y su integración con V8 le da flexibilidad a un costo prohibitivo de recursos.

Para ConnectomeDB, el camino hacia la competitividad radica en la "Evolución por Simplificación". Al adoptar Rust, ConnectomeDB elimina las ineficiencias de la gestión de memoria de C++ y el GC de JavaScript. Al integrar LISP como motor lógico nativo sobre un almacenamiento binario de acceso aleatorio, ConnectomeDB puede ofrecer una base de datos que no solo almacena información, sino que la procesa con una eficiencia que emula la arquitectura del cerebro humano.

La clave del éxito será mantener la expresividad de AQL pero con la velocidad de ejecución de un kernel de Rust, transformando el "Edge Index" de ArangoDB en una "Sinapsis Computacional" de alto rendimiento que pueda escalar billones de bordes en hardware convencional. ArangoDB ha demostrado que el modelo multimodelo es el futuro; ConnectomeDB debe demostrar que ese futuro es más ligero, más rápido y profundamente más inteligente.

#### **Obras citadas**

1. ArangoDB \- Revision \#12, fecha de acceso: abril 8, 2026, [https://dbdb.io/db/arangodb/revisions/12](https://dbdb.io/db/arangodb/revisions/12)  
2. Dgraph vs ArangoDB: Architecture and Consistency \- PuppyGraph, fecha de acceso: abril 8, 2026, [https://www.puppygraph.com/blog/arangodb-vs-dgraph](https://www.puppygraph.com/blog/arangodb-vs-dgraph)  
3. RocksDB Architecture \- Mintlify, fecha de acceso: abril 8, 2026, [https://mintlify.com/facebook/rocksdb/concepts/architecture](https://mintlify.com/facebook/rocksdb/concepts/architecture)  
4. The Fundamentals of RocksDB \- GetStream.io, fecha de acceso: abril 8, 2026, [https://getstream.io/blog/rocksdb-fundamentals/](https://getstream.io/blog/rocksdb-fundamentals/)  
5. Compaction Options · ArangoDB v3.3.3 Documentation \- Huihoo, fecha de acceso: abril 8, 2026, [https://docs.huihoo.com/arangodb/3.3/Manual/Administration/Configuration/Compaction.html](https://docs.huihoo.com/arangodb/3.3/Manual/Administration/Configuration/Compaction.html)  
6. python-arango \- aioarango, fecha de acceso: abril 8, 2026, [https://aioarango.readthedocs.io/\_/downloads/en/stable/pdf/](https://aioarango.readthedocs.io/_/downloads/en/stable/pdf/)  
7. aioarangodb Documentation, fecha de acceso: abril 8, 2026, [https://aioarangodb.readthedocs.io/\_/downloads/en/latest/pdf/](https://aioarangodb.readthedocs.io/_/downloads/en/latest/pdf/)  
8. Multiversion concurrency control \- Wikipedia, fecha de acceso: abril 8, 2026, [https://en.wikipedia.org/wiki/Multiversion\_concurrency\_control](https://en.wikipedia.org/wiki/Multiversion_concurrency_control)  
9. GitHub \- arangodb/velocypack: A fast and compact format for serialization and storage, fecha de acceso: abril 8, 2026, [https://github.com/arangodb/velocypack](https://github.com/arangodb/velocypack)  
10. Applying a Label Propagation Algorithm to Detect Communities in Graph Databases (Andi Ferhati) \- ResearchGate, fecha de acceso: abril 8, 2026, [https://www.researchgate.net/publication/364933016\_Clustering\_Graphs\_-\_Applying\_a\_Label\_Propagation\_Algorithm\_to\_Detect\_Communities\_in\_Graph\_Databases/fulltext/635f3e6a8d4484154a4cb5ae/Clustering-Graphs-Applying-a-Label-Propagation-Algorithm-to-Detect-Communities-in-Graph-Databases.pdf](https://www.researchgate.net/publication/364933016_Clustering_Graphs_-_Applying_a_Label_Propagation_Algorithm_to_Detect_Communities_in_Graph_Databases/fulltext/635f3e6a8d4484154a4cb5ae/Clustering-Graphs-Applying-a-Label-Propagation-Algorithm-to-Detect-Communities-in-Graph-Databases.pdf)  
11. Vector Search in ArangoDB: Practical Insights and Hands-On Examples, fecha de acceso: abril 8, 2026, [https://arango.ai/blog/vector-search-in-arangodb-practical-insights-and-hands-on-examples/](https://arango.ai/blog/vector-search-in-arangodb-practical-insights-and-hands-on-examples/)  
12. Vector Search: Navigating Recall and Performance \- OpenSource Connections, fecha de acceso: abril 8, 2026, [https://opensourceconnections.com/blog/2025/02/27/vector-search-navigating-recall-and-performance/](https://opensourceconnections.com/blog/2025/02/27/vector-search-navigating-recall-and-performance/)  
13. HNSW at Scale: Why Your RAG System Gets Worse as the Vector Database Grows, fecha de acceso: abril 8, 2026, [https://towardsdatascience.com/hnsw-at-scale-why-your-rag-system-gets-worse-as-the-vector-database-grows/](https://towardsdatascience.com/hnsw-at-scale-why-your-rag-system-gets-worse-as-the-vector-database-grows/)  
14. HNSW at Scale: Why Adding More Documents to Your Database Breaks RAG | by Gowtham Boyina | Feb, 2026 | Level Up Coding, fecha de acceso: abril 8, 2026, [https://levelup.gitconnected.com/hnsw-at-scale-why-adding-more-documents-to-your-database-breaks-rag-0cca7008107d](https://levelup.gitconnected.com/hnsw-at-scale-why-adding-more-documents-to-your-database-breaks-rag-0cca7008107d)  
15. Graph database \- Wikipedia, fecha de acceso: abril 8, 2026, [https://en.wikipedia.org/wiki/Graph\_database](https://en.wikipedia.org/wiki/Graph_database)  
16. 7 Best Graph Databases in 2026 \- PuppyGraph, fecha de acceso: abril 8, 2026, [https://www.puppygraph.com/blog/best-graph-databases](https://www.puppygraph.com/blog/best-graph-databases)  
17. Graph Done Right • Arango, fecha de acceso: abril 8, 2026, [https://arango.ai/resources/graph-done-right/](https://arango.ai/resources/graph-done-right/)  
18. ArangoDB – A Graph Database \- StatusNeo, fecha de acceso: abril 8, 2026, [https://statusneo.com/arangodb-a-graph-database/](https://statusneo.com/arangodb-a-graph-database/)  
19. Graph databases and their application to the Italian Business Register for efficient search of relationships among companies \- Padua Thesis and Dissertation Archive, fecha de acceso: abril 8, 2026, [https://thesis.unipd.it/retrieve/9f005b0e-feab-4d7e-8489-6c51ae428212/luca\_sinico\_tesi.pdf](https://thesis.unipd.it/retrieve/9f005b0e-feab-4d7e-8489-6c51ae428212/luca_sinico_tesi.pdf)  
20. Arangodb vs Janusgraph : Know The Difference \- PuppyGraph, fecha de acceso: abril 8, 2026, [https://www.puppygraph.com/blog/arangodb-vs-janusgraph](https://www.puppygraph.com/blog/arangodb-vs-janusgraph)  
21. Three Ways to Scale your Graph \- ArangoDB, fecha de acceso: abril 8, 2026, [https://arango.ai/blog/three-ways-to-scale-your-graph/](https://arango.ai/blog/three-ways-to-scale-your-graph/)  
22. arangodb/README.md at devel \- GitHub, fecha de acceso: abril 8, 2026, [https://github.com/arangodb/arangodb/blob/devel/README.md](https://github.com/arangodb/arangodb/blob/devel/README.md)  
23. Vector Stores — langchain-arangodb documentation, fecha de acceso: abril 8, 2026, [https://langchain-arangodb.readthedocs.io/en/latest/vectorstores.html](https://langchain-arangodb.readthedocs.io/en/latest/vectorstores.html)  
24. Methods to decrease Memory consumption in ArangoDB \- Stack Overflow, fecha de acceso: abril 8, 2026, [https://stackoverflow.com/questions/40401515/methods-to-decrease-memory-consumption-in-arangodb](https://stackoverflow.com/questions/40401515/methods-to-decrease-memory-consumption-in-arangodb)  
25. Issues · arangodb/arangodb \- GitHub, fecha de acceso: abril 8, 2026, [https://github.com/arangodb/arangodb/issues](https://github.com/arangodb/arangodb/issues)  
26. Zero-Copy Data Processing in Python Using Apache Arrow | by Majidbasharat | Medium, fecha de acceso: abril 8, 2026, [https://medium.com/@majidbasharat21/zero-copy-data-processing-in-python-using-apache-arrow-831beb90c59d](https://medium.com/@majidbasharat21/zero-copy-data-processing-in-python-using-apache-arrow-831beb90c59d)  
27. Zero-copy, zero contest \- Columnar Blog, fecha de acceso: abril 8, 2026, [https://columnar.tech/blog/zero-copy-zero-contest/](https://columnar.tech/blog/zero-copy-zero-contest/)  
28. How to Integrate OTel Arrow with Apache Arrow-Native Backends Like ClickHouse, fecha de acceso: abril 8, 2026, [https://oneuptime.com/blog/post/2026-02-06-otel-arrow-clickhouse-zero-copy/view](https://oneuptime.com/blog/post/2026-02-06-otel-arrow-clickhouse-zero-copy/view)  
29. Apache Arrow | Apache Arrow, fecha de acceso: abril 8, 2026, [https://arrow.apache.org/](https://arrow.apache.org/)  
30. MVCC Concept: The No-Lock Approach for High-Concurrency Systems \- DEV Community, fecha de acceso: abril 8, 2026, [https://dev.to/markliu2013/mvcc-concept-the-no-lock-approach-for-high-concurrency-systems-ooj](https://dev.to/markliu2013/mvcc-concept-the-no-lock-approach-for-high-concurrency-systems-ooj)  
31. How MySQL MVCC (Multi-Version Concurrency Control) Works \- OneUptime, fecha de acceso: abril 8, 2026, [https://oneuptime.com/blog/post/2026-03-31-mysql-mvcc-multi-version-concurrency-control/view](https://oneuptime.com/blog/post/2026-03-31-mysql-mvcc-multi-version-concurrency-control/view)  
32. multi-thread access ArangoDB very slow · Issue \#92 · arangodb/python-arango \- GitHub, fecha de acceso: abril 8, 2026, [https://github.com/arangodb/python-arango/issues/92](https://github.com/arangodb/python-arango/issues/92)  
33. High number of Threads they wouldn't close · Issue \#4604 \- GitHub, fecha de acceso: abril 8, 2026, [https://github.com/arangodb/arangodb/issues/4604](https://github.com/arangodb/arangodb/issues/4604)  
34. From Virtual Threads to Vector Databases: Why Java 25 is the Foundational LTS for the AI Enterprise \- JAVAPRO, fecha de acceso: abril 8, 2026, [https://javapro.io/wp-content/uploads/2025/12/JAVAPRO\_05-2025.pdf](https://javapro.io/wp-content/uploads/2025/12/JAVAPRO_05-2025.pdf)  
35. Database interfaces — list of Rust libraries/crates // Lib.rs, fecha de acceso: abril 8, 2026, [https://lib.rs/database](https://lib.rs/database)  
36. Artificial General Intelligence: 13th International Conference, AGI 2020, St. Petersburg, Russia, September 16–19, 2020, Proceedings \[1st ed.\] 9783030521516, 9783030521523 \- DOKUMEN.PUB, fecha de acceso: abril 8, 2026, [https://dokumen.pub/artificial-general-intelligence-13th-international-conference-agi-2020-st-petersburg-russia-september-1619-2020-proceedings-1st-ed-9783030521516-9783030521523.html](https://dokumen.pub/artificial-general-intelligence-13th-international-conference-agi-2020-st-petersburg-russia-september-1619-2020-proceedings-1st-ed-9783030521516-9783030521523.html)  
37. fecha de acceso: abril 8, 2026, [https://raw.githubusercontent.com/arangodb/arangodb/3.4/CHANGELOG](https://raw.githubusercontent.com/arangodb/arangodb/3.4/CHANGELOG)  
38. Exploiting AQL Injection Vulnerabilities in ArangoDB \- Anvil Secure, fecha de acceso: abril 8, 2026, [https://www.anvilsecure.com/blog/exploiting-aql-injection-vulnerabilities-in-arangodb.html](https://www.anvilsecure.com/blog/exploiting-aql-injection-vulnerabilities-in-arangodb.html)  
39. ArangoDB v3.3.1 AQL Documentation, fecha de acceso: abril 8, 2026, [https://download.arangodb.com/arangodb33/doc/ArangoDB\_AQL\_3.3.1.pdf](https://download.arangodb.com/arangodb33/doc/ArangoDB_AQL_3.3.1.pdf)  
40. Traversal · ArangoDB v3.2.0 AQL Documentation \- Huihoo, fecha de acceso: abril 8, 2026, [https://docs.huihoo.com/arangodb/3.2/AQL/Graphs/Traversals.html](https://docs.huihoo.com/arangodb/3.2/AQL/Graphs/Traversals.html)  
41. Using Custom Visitors in AQL Graph Traversals \- J@ArangoDB, fecha de acceso: abril 8, 2026, [http://jsteemann.github.io/blog/2015/01/28/using-custom-visitors-in-aql-graph-traversals/](http://jsteemann.github.io/blog/2015/01/28/using-custom-visitors-in-aql-graph-traversals/)  
42. AQL: serious flaw: failure to prune paths · Issue \#3979 \- GitHub, fecha de acceso: abril 8, 2026, [https://github.com/arangodb/arangodb/issues/3979](https://github.com/arangodb/arangodb/issues/3979)  
43. Neo4j Alternatives in 2026: A Fair Look at the Open-Source Options \- ArcadeDB, fecha de acceso: abril 8, 2026, [https://arcadedb.com/blog/neo4j-alternatives-in-2026-a-fair-look-at-the-open-source-options/](https://arcadedb.com/blog/neo4j-alternatives-in-2026-a-fair-look-at-the-open-source-options/)  
44. ArangoDB Features Review \- by Ali Bayat \- Medium, fecha de acceso: abril 8, 2026, [https://medium.com/@ali.bayat/arangodb-features-review-7db4d72824b3](https://medium.com/@ali.bayat/arangodb-features-review-7db4d72824b3)  
45. A while after upgrade to v3.12.0, unable to create documents: "Corruption: Compaction sees out-of-order keys" \#20841 \- GitHub, fecha de acceso: abril 8, 2026, [https://github.com/arangodb/arangodb/issues/20841](https://github.com/arangodb/arangodb/issues/20841)  
46. ArangoDB corruption issue \- MetaDefender Aether(Sandbox) \- OPSWAT, fecha de acceso: abril 8, 2026, [https://www.opswat.com/docs/filescan/troubleshooting/arangodb-corruption-issue](https://www.opswat.com/docs/filescan/troubleshooting/arangodb-corruption-issue)  
47. high system load, especially with clustering enabled · Issue \#837 \- GitHub, fecha de acceso: abril 8, 2026, [https://github.com/arangodb/arangodb/issues/837](https://github.com/arangodb/arangodb/issues/837)  
48. ArangoDB reviews 2026 | FitGap, fecha de acceso: abril 8, 2026, [https://us.fitgap.com/products/015423/arangodb](https://us.fitgap.com/products/015423/arangodb)

[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABoAAAAfCAYAAAD5h919AAABzElEQVR4Xu2UvyuFURjHvxIR5WekDJLFJEkMTJQMKNn8Af4BZbGaxayQQTYlFtItBmUysRhIZLAoBvLj++15j3vOfb3XdS2G91ufXOc85z3nOd/nOUCqVHlUTybIlMcQKfODElRKBhCu1bf0zZj6yAW5JR8RN6TVD0rQOHmDrXkh1+SQtHkxMfWSS3JPnkhPOB1TE9kmZ7CNpsPpZClwi+zBFo6G04FKyDyZJXfkkXQFEXm0QmbIOmwj/U6SrnsJdph3ckrqgogEKWgfdl1zsI0Wgoisqska6YDFKFaHLEhK+4g0wq5Qi5XZd1KmooocoAh/VmF3Pwy7jgzs9L6UxVo03o4/+CPp+lR1qsDmrwjrK/kifyT5o2yK8kdqIVew0+rUTuoZVZqylpw/y18RP8j3R9LGOuUzrLck9cxm9Ff6sz9SBdmFfWQsGlcmysjJ+fNAOr3xvPL9cXK9pFIfJIsI3z5t+mt/9Da5K3JyvbQBy7YtmC2if1QAx8j64zQJ+5DKPNcDlXYGBfpTCysCeaEK6ycNyPqkp/+V7JDKaEzeqSLVZyp/FctINKa5mGSeTNSJfE5IjRdzTrqj/8thL3XuGke+BzhVqlT/XZ8zZm/GD9L33wAAAABJRU5ErkJggg==>

[image2]: <data:image/png;base64,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>

[image3]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAfCAYAAAClDZ5ZAAAC6klEQVR4Xu2Yy6tPURTHl1BEHtGVKJekPEq6SsSMMjEhRRQxIBkZ8AfI6E6QJCkhkQFJSjK4I4mRIqWUK1IkpQyQx/dz99mcs37nnN85P7/H5PetT669zn6svdZ+/cz66quvXmuc2C5uianOVlWrxD0x3xuqarxYI86JV+JNwvWknEE20zbxVCzwhppquZ1l4rEYFXvFLAuOLRE3xS8LDk5Ovs/TcguOb/IGp2nirDjvDSkxaSfFfasYWSocFj/FFTElax7TRAud/hbHLT8yfHMjgb+9iOhRcdtCX7R1KfNFoxaLt+KgN3gxIBqnUQaaN4CoReK9+CxWOBvaIL6Izd6QaL+FKOwWw1bNEcZ3RjwTA86WEXnI7JBSpFKZiNQDCwM45mx0eFE8ETOdLU/Ur+II2mhhjDu8IYp8/mgh97c6W5HoOG8A88RrcdqVF6mOI7FtJqohpSeICxYaeySmZ825miTuWqgzYtkFyKwxIbtSZWWq40js97mY7WxjOU6u56VJkUgZUoc6Vy07O0fED7E+VVamOo4gIs36W+kNByw0VKfzpeKThXonnI3ofhVDrrxIdR3heyJO5P+KmWRGaYhDb07aWCLShjo06HcmBjQq5rryItV1JPa9JV1Ibo8kBv6tctik18cLa3S+047gQFscWSu+WajjD6cY4a47gmigqiPxxOZ7zpG8Ha7TEclNLRQXe+6W5hQPzZdWfIHryWJHCy3cYZodhussbNMcnKudLa1ubL/smOycDUrPNJeztLj17hPfxUMxmLE2qsqBSAqTerwxuHPhyJ3k/5TP+PdpRnGjKbz+sEi5br+zMODLYo84JT4k7LTgVDPFCJddUWIUiiiKTryicHlkzIViMZM2hywMhMvZoDWp5MSV55qVzNp/iGgz0f7s6pjoqN0dMplEoup9sC3i5UjOFz2sWlF8WPGG6ar40YDHV7OnbhURDZ667ZyYWmr5RwOndrXTspjJnv8c1FdfLeoPfE/F57x+H2EAAAAASUVORK5CYII=>

[image4]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAD8AAAAfCAYAAABQ8xXpAAADlUlEQVR4Xu2XS6iNURTHl1BEeV3kUW6SkgwkKa8MGEhKUhQTKZSZlBgRY4kyUpckeRUhhXQzEUbEUKGLUhIhjzzW766zztlnn72/893D4A6+X/2Lu/be395rr8c+IhUVFRWDgxGqCaohseF/M0+1IaEVqmE18e/YjhZJgy7Vusi+RjUqGFOG4aoe1SPVuMgWM0Na99ROi6XmVA52QvVG9aemX6o+1SmxGxiruq56F4z5oXqu2iwNlqleB2M+q+6qJgVjyrBKbA8vVVMiW8we1Suxb/l32Sd/CxXaL4mdu85oVW/NeCw0BMxUvRUbszayOXj0rOqkamRkK8MY1T2xb7xXzWk2Z9khNueFalqzqR/2tVT1QXU6svV7Ao+wQIuxBrfAbTBmfWRzZqluysBv29mp+qj6rfqiWtBsTsLBeiRzqwF+xr2xATh00eG3SCN0UguwiSNi4zoBxz1WbVN9EnPAyqYRaagL1Af2RQQUwdnCVK1DuLNAr1gahHCTD1TfJH94bumitM4tA447qjokFrYeYbn0CuG7RMlPsdAOWS5W4BzSMelQDpQ7PLbD0qgLLBJCfp8TK3qdwLz7Yk7uUj2TcjcJ3CRjmcNchzBnT35YHLxQNb4+IsAPHy8yV3VbNVmsmKVSgxpA18jlWxE47rJqY+3/YfFNRVhIUb4vUT1RTQ/+loUQY5GwxXgr9I15XQg/xGPkmpiTOoEacVUa3YF3wR1JR1hMGCW0M29t3rpviLXrtvjhaWe0NSAcCR3fGJuJU4MKfVA6e41xK4R7XNXbFV8nzHeixB8yx8XeCvsbQ4uhWLCIt5hUHsd1oVvs1kuFVgTOosCdUU0VizYXhZPvEAFFL0Tv7zy4SEuHvd2STHFL4V70w6fy2A9PalCV/6W1zReLsj7Jv8h6pbX4OkX5Tjrw3ih9KX54Ftsq6TzGIX540uSK5DdXBO/382Ipk2K3tNafmKKuMFSs/5dOxfAFx2Njn7RO9rrwXSzUeId3Ak4kLHOO8+8QFbnbo219lXR/HzDh4Z9K+onqdYExF8RucKCwLi+yIsf54YueuJ7vcWvuiPCZmMtjTw1+IJCzZSGCaInMfyg2H0fGbQhnUrg2iTkZ4QgcRij7OkQDacle+eXYLXZ5uUhqiz8uisKRX1n82qJKxylRhM9js6HI7RDP9ZRWSyMicjogHUK13K6aHRsCcMou1cTYUFFRUVFRUVExWPgLbvsDv+Zxa6gAAAAASUVORK5CYII=>