# **Convergencia de Cuantización Híbrida en ConnectomeDB v0.5.0: Optimización de la Fidelidad Semántica ante los Axiomas de Hierro**

El panorama contemporáneo de la inteligencia artificial y la gestión de datos a gran escala se enfrenta a un desafío estructural sin precedentes denominado el "muro de la memoria", donde la capacidad de cómputo de los aceleradores de IA ha crecido de forma exponencial mientras que el ancho de banda y la capacidad de la memoria de acceso aleatorio (RAM) no han seguido el mismo ritmo.1 En este contexto, el desarrollo de la fase 31 de la versión 0.5.0 de ConnectomeDB exige una resolución técnica que equilibre la velocidad de búsqueda del índice de mundo pequeño navegable jerárquico (HNSW) con la preservación de la integridad lógica impuesta por los Axiomas de Hierro.1 La arquitectura propuesta para esta fase no debe verse como una elección binaria entre la cuantización de 1 bit (binaria pura) y la de 3 bits (TurboQuant), sino como una integración simbiótica que maximiza la eficiencia en hardware degradado sin comprometer la cordura del sistema.1

## **Fundamentos del Desafío de Memoria en Arquitecturas Cognitivas**

Las bases de datos multimodales inspiradas en la neurociencia, como ConnectomeDB, operan bajo la premisa de unificar vectores, grafos y relaciones en un único binario, emulando la conectividad de un cerebro biológico.1 Esta unificación requiere el almacenamiento de miles de millones de vectores de alta dimensión (embeddings) que consumen recursos masivos. Tradicionalmente, cada dimensión se almacena como un punto flotante de 32 bits, lo que resulta prohibitivo para sistemas que operan en hardware local con límites de 16GB de RAM.1 La cuantización surge como el mecanismo crítico para comprimir estos vectores, reduciendo la precisión de las representaciones numéricas para ahorrar espacio y acelerar las comparaciones matemáticas.4

La cuantización binaria pura (1 bit por dimensión) ofrece una reducción de memoria de hasta 32 veces, transformando vectores complejos en cadenas de bits que pueden compararse mediante instrucciones de hardware ultra rápidas como POPCNT.1 Sin embargo, esta compresión agresiva a menudo degrada la fidelidad del producto interno, introduciendo ruido que puede violar los Axiomas de Hierro.1 Por otro lado, TurboQuant, una innovación de Google Research presentada para ICLR 2026, introduce un esquema de 3 bits que preserva las relaciones geométricas con una distorsión cercana al límite teórico de Shannon, asegurando que el mecanismo de atención y la inferencia lógica permanezcan estables.1

| Característica | Cuantización Binaria (1-bit) | TurboQuant (3-bit) |
| :---- | :---- | :---- |
| Factor de Compresión | 32x respecto a FP32 | \~10.6x respecto a FP32 (6x en KV Cache) |
| Mecanismo de Búsqueda | AND \+ POPCNT | PolarQuant \+ Corrección QJL |
| Fidelidad del Producto Interno | Variable (90-94% de recall) | Cercana a la neutralidad de calidad (\>99%) |
| Dependencia de Hardware | Instrucciones POPCNT/AVX-512 | Kernels de GPU (CUDA/Metal) o SIMD avanzado |
| Entrenamiento / Libros de Código | No requiere (Data-oblivious) | No requiere (Data-oblivious) |
| Propósito Principal | Índices persistentes masivos | Caché transitoria y refinamiento semántico |

1

## **Análisis Técnico de TurboQuant: La Tubería de 3 Bits**

TurboQuant no es simplemente una técnica de compresión incremental; representa un cambio de paradigma en la forma en que los sistemas de IA perciben y almacenan el espacio vectorial.1 Su arquitectura se basa en una tubería matemática de tres etapas diseñada para minimizar el error de reconstrucción cuadrático medio (MSE) y preservar el producto interno fundamental para la lógica de ConnectomeDB.1

### **Fase 1: Rotación Ortogonal Aleatoria y Distribución Beta**

El primer paso crítico es la aplicación de una rotación ortogonal aleatoria, conocida como la transformada de Johnson-Lindenstrauss (JLT).1 Los vectores en los modelos de IA suelen ser "cuasi-dispersos", con componentes de magnitudes muy dispares. Al aplicar una matriz de rotación ortogonal generada de forma determinista, la energía del vector se distribuye uniformemente entre todas las coordenadas.1 Tras esta rotación, cada componente sigue una distribución estadística predecible, generalmente una distribución Beta o Gaussiana, lo que permite utilizar cubetas de cuantización óptimas derivadas del algoritmo de Lloyd-Max.1

### **Fase 2: PolarQuant y Preservación del Radio**

TurboQuant utiliza la técnica PolarQuant para la compresión base. En lugar de cuantizar cada dimensión de forma independiente, agrupa los componentes del vector rotado en pares y los convierte a coordenadas polares ![][image1].1 La magnitud o radio (![][image2]) se almacena con alta precisión (generalmente FP32), mientras que la información direccional o ángulo (![][image3]) se cuantiza a un número reducido de bits (entre 2 y 4).1 Esta distinción es vital porque la lógica de ConnectomeDB es extremadamente sensible a la magnitud de los vectores, ya que esta escala los resultados del producto interno antes de la evaluación axiomática.1

### **Fase 3: Corrección de Residuos mediante QJL**

La tercera fase introduce la Cuantización de Johnson-Lindenstrauss (QJL) para corregir los errores residuales. TurboQuant toma la diferencia entre el vector original y su versión comprimida, lo proyecta a través de una matriz aleatoria y almacena únicamente el bit de signo ![][image4].1 Este "bosquejo" de 1 bit permite realizar una corrección estadística durante el cálculo del producto interno, produciendo un resultado insesgado y con una distorsión controlada.1

## **RaBitQ y la Eficiencia del Índice de 1 Bit**

RaBitQ (SIGMOD 2024\) comparte el principio fundamental de las rotaciones aleatorias pero se especializa en índices persistentes de 1 bit por dimensión.3 Su enfoque es puramente geométrico: proyecta los vectores rotados sobre los vértices de un hipercubo ![][image5].3

La relevancia de RaBitQ para ConnectomeDB reside en su capacidad para operar en hardware antiguo o limitado. En ausencia de unidades de punto flotante potentes, el cálculo de distancias puede reemplazarse por operaciones de bit como POPCNT, disponibles en CPUs x86 desde hace más de una década.1 Para ConnectomeDB v0.5.0, RaBitQ permite que el grafo de navegación HNSW en RAM sea extremadamente ligero, manteniendo la estructura de búsqueda sin agotar los 16GB de memoria disponibles.3

## **Axiomas de Hierro: El Criterio de Verdad y Fidelidad**

El Núcleo Axiomático Inmutable de ConnectomeDB no es una limitación intelectual, sino el sistema operativo de la realidad del proyecto.1 Estos axiomas definen verdades fundamentales, como la imposibilidad de que un dato sea escrito y no escrito simultáneamente o que una transacción ocurra antes de la creación del nodo relacionado.1 La fidelidad del producto interno es el puente entre el mundo probabilístico de los vectores y el mundo determinista de los axiomas.1

### **El Riesgo de la "IA Psicótica"**

Un sistema que opera con fracturas en sus Axiomas de Hierro genera lo que la arquitectura denomina una "IA Psicótica": una entidad cognitiva cuya brújula lógica está desviada, garantizando una degradación exponencial de sus inferencias.1 Si la cuantización binaria pura introduce un error masivo en el cálculo de la similitud entre dos neuronas, el Cortex podría realizar una inferencia basada en una mentira estadística, activando una cadena causal errónea.1

El Protocolo de Emergencia (Panic State) se activa cuando la discrepancia entre la realidad empírica (datos físicos) y los axiomas supera un nivel de seguridad.1 Para evitar el pánico constante, la fase 31 debe asegurar que el motor de búsqueda vectorial proporcione una precisión suficiente para que las validaciones lógicas sean consistentes.1

### **Salud Semántica y Umbral de Tolerancia**

La salud semántica se mide por la consistencia de los punteros y la validez de los resultados de búsqueda. ConnectomeDB debe priorizar la integridad absoluta, entrando en pánico antes de permitir decisiones basadas en el caos.1 TurboQuant, con su fidelidad de 3 bits, actúa como un cortafuegos de la cordura, ofreciendo una base sólida para que el Cortex razone sobre la incertidumbre.1

| Estado de Salud | Acción Técnica | Objetivo de Fidelidad |
| :---- | :---- | :---- |
| 100% (Óptimo) | Operación normal y backups. | Inferencia de alta precisión. |
| 70-99% (Duda) | Cuarentena Semántica. | Verificación mediante vectores originales. |
| \< 70% (Corrupto) | Modo de Rescate (Read-Only). | Prevención de alucinaciones de DB. |
| Axiom Break | panic\! de Rust y volcado de memoria. | Protección de la integridad física. |

1

## **Propuesta de Arquitectura Híbrida para la Fase 31**

La respuesta a la interrogante del usuario no es la exclusión, sino la integración. Se propone una arquitectura de búsqueda de dos fases que utiliza tanto la cuantización binaria de 1 bit como la de 3 bits de TurboQuant, optimizando el rendimiento en RAM sin sacrificar la verdad axiomática.1

### **Fase 1: Navegación de Grano Grueso (1-bit RaBitQ)**

En esta fase, el índice HNSW reside en la RAM utilizando vectores comprimidos a 1 bit por dimensión.1 El objetivo es la velocidad pura. Durante el recorrido del grafo jerárquico, el motor utiliza instrucciones POPCNT para descartar rápidamente millones de nodos irrelevantes.3 Esta fase no requiere una precisión absoluta, solo una dirección semántica correcta para identificar el vecindario del resultado potencial.3

### **Fase 2: Refinamiento de Grano Fino (3-bit TurboQuant)**

Una vez que se ha reducido el conjunto de candidatos a un número manejable (ej. los 100 vecinos más cercanos), el sistema aplica una re-evaluación utilizando vectores comprimidos con TurboQuant de 3 bits.1 Esta fase de refinamiento corrige los errores de la búsqueda binaria y proporciona una estimación del producto interno lo suficientemente precisa para satisfacer los criterios de los Axiomas de Hierro.1

### **Justificación de la Dualidad**

Esta dualidad permite que ConnectomeDB escale más allá de los límites de la RAM física.1 Mientras que el grafo de navegación (ligero) permanece en RAM, los vectores de refinamiento de TurboQuant pueden cargarse dinámicamente o residir en un nivel de caché intermedio.1 El uso de ambas tecnologías permite que el sistema mantenga una latencia sub-milisegundo en las búsquedas mientras garantiza que el 99.5% de los resultados sean idénticos a los obtenidos con precisión completa.6

## **Estrategias de Aceleración y Escalabilidad en Hardware Degradado**

Para que la fase 31 sea exitosa en entornos de recursos limitados, se deben implementar técnicas de bajo nivel que complementen la cuantización.1

### **Tablas de Búsqueda (LUT) y Bit-Slicing**

En hardware que carece de unidades de punto flotante potentes, el cálculo de distancias puede reemplazarse por búsquedas en tablas precomputadas (Lookup Tables).1 Tecnologías como T-MAC y Ladder precalculan las interacciones posibles entre bits de cuantización, permitiendo que la CPU realice solo accesos a memoria y sumas enteras.1

El Bit-Slicing divide los pesos de alta precisión en múltiples celdas de menor precisión, permitiendo operaciones paralelas incluso en hardware que no soporta tipos de datos modernos como FP16 o INT8 de forma nativa.1 Estas técnicas deben integrarse en los hardware adapters de ConnectomeDB para maximizar el uso de instrucciones SIMD antiguas como SSE4.2.1

### **Gestión de Memoria y Shadow Kernel**

La arquitectura debe distinguir entre el cortex\_ram (memoria activa) y el shadow\_kernel (memoria profunda).1 Los vectores cuantizados binarios se mantienen en el cortex\_ram para la búsqueda inmediata, mientras que las versiones de TurboQuant de 3 bits y los vectores originales residen en el almacenamiento persistente gestionado por RocksDB.1

El uso de Column Families en RocksDB permite separar físicamente estos datos, asegurando que los escaneos de rango en el almacenamiento principal sean rápidos y evitando la fragmentación del índice.1 El SleepWorker, actuando como un proceso de mantenimiento circadiano, se encarga de degradar los nodos del cortex\_ram al shadow\_kernel basándose en su relevancia temporal (hits y last\_accessed).1

| Método de Optimización | Beneficio en Hardware Degradado | Tecnología Relacionada |
| :---- | :---- | :---- |
| Cuantización Binaria | Reducción de 32x en uso de RAM. | RaBitQ, POPCNT. |
| Lookup Tables (LUT) | Sustituye multiplicaciones por accesos a caché. | T-MAC, Ladder. |
| Memory-Mapped Files | Escala más allá de la RAM usando disco. | ruvector, Milvus mmap. |
| SIMD (SSE4.2/AVX2) | Paralelismo de datos en CPUs anteriores. | wide crate, Intel IPP. |

1

## **Implementación de la Fase 31 de ConnectomeDB v0.5.0**

La implementación de la fase 31 requiere una reestructuración de los módulos de almacenamiento y búsqueda para soportar el esquema híbrido propuesto. Se detallan los pasos algorítmicos y estructurales necesarios.

### **Paso 1: Refactorización del UnifiedNode y Tipado Cuantizado**

El UnifiedNode debe evolucionar para actuar como un contenedor multiescala de información vectorial.1 Se deben añadir campos dedicados para las representaciones de bajo bit.

Rust

pub enum VectorRepresentations {  
    Full(VectorData),        // FP32 para máxima fidelidad (Disco)  
    Turbo(Box\<\[u8\]\>),        // 3-bit TurboQuant para refinamiento (RAM/Disco)  
    Binary(Box\<\[u64\]\>),      // 1-bit RaBitQ para navegación (RAM)  
}

pub struct UnifiedNode {  
    pub id: u64,  
    pub vector: VectorRepresentations,  
    pub trust\_score: f32,    // Inyectado en Fase 16 para gobernanza  
    pub semantic\_valence: f32, // Impacto Amígdala  
    pub flags: NodeFlags,    // Incluye PINNED para evitar el olvido  
}

1

### **Paso 2: Implementación de la Rotación Walsh-Hadamard (SIMD)**

Tanto TurboQuant como RaBitQ requieren una rotación aleatoria inicial.1 Para evitar el costo ![][image6] de una multiplicación de matriz completa, se debe implementar una Transformada Rápida de Walsh-Hadamard (FWHT) que reduce la complejidad a ![][image7].1 Esta operación debe estar optimizada con la librería wide para utilizar instrucciones AVX-512 o NEON según el hardware detectado en runtime.1

### **Paso 3: Diseño del Executor Híbrido**

El motor de ejecución (executor.rs) debe modificarse para realizar la búsqueda en dos etapas. El planificador de consultas (Cortex) decidirá la ruta de búsqueda basada en la disponibilidad de memoria y la urgencia de la consulta.1

1. **Stage Coarse (HNSW \+ 1-bit):** El executor escanea el grafo en RAM. Utiliza el vector binario para calcular distancias de Hamming aceleradas por POPCNT.  
2. **Stage Fine (TurboQuant Re-ranking):** Se seleccionan los ![][image8] mejores candidatos. El sistema recupera sus vectores de 3 bits (ya sea de RAM o de un buffer de RocksDB) y realiza el cálculo de PolarQuant con corrección QJL.  
3. **Axiom Validation:** El resultado se pasa al DevilsAdvocate. Si el Trust Score del candidato es inferior al del incumbente o si viola una regla de integridad causal, la operación se aborta o se marca para revisión.1

### **Paso 4: Integración del SleepWorker para el Mantenimiento del Índice**

El SleepWorker debe asegurar que las consolidaciones de memoria actualicen correctamente el índice HNSW.1 Un gap detectado en fases anteriores indicaba que los nodos consolidados no siempre actualizaban el índice, arriesgando una divergencia entre disco y memoria.1

En la fase 31, el SleepWorker debe:

1. Identificar neuronas oníricas (hits \< 5).1  
2. Si son elegibles para Neural Summarization (Compresión Cognitiva), invocar a Ollama para generar un resumen denso.1  
3. Actualizar el índice HNSW binario con la nueva neurona de resumen y marcar las originales como AuditableTombstone en el shadow\_kernel.1

## **Análisis Neurobiológico de la Cuantización**

La elección de los niveles de bit no es arbitraria; refleja la organización funcional del cerebro humano. El cortex\_ram actúa como la memoria de trabajo (corteza prefrontal), donde la información es rápida pero volátil y limitada.1 La cuantización de 1 bit emula la señalización binaria de "disparo o no disparo" de las neuronas, permitiendo una conectividad masiva con bajo gasto energético.1

Por otro lado, TurboQuant y su preservación de la magnitud vectorial actúan como el sistema de la Amígdala y el Hipocampo, asegurando que los eventos de alto impacto semántico (valencia emocional) se consoliden con mayor fidelidad.1 Al asignar un semantic\_valence alto a ciertos nodos (ej. alergias del usuario o protocolos de seguridad), el sistema puede aplicar una cuantización menos agresiva o incluso prohibir el olvido bayesiano para esos datos críticos.1

### **Plasticidad Sináptica y Decaimiento de Edges**

En la fase 31, no solo los nodos (neuronas) deben decaer, sino también sus relaciones (edges/synapses).1 Siguiendo el principio de que "las neuronas que disparan juntas, se conectan juntas", el sistema debe implementar la Depresión a Largo Plazo (LTD) para los enlaces.1 Si una relación específica entre dos nodos no se utiliza, su peso cae progresivamente hasta desaparecer, limpiando el ruido del grafo y optimizando el espacio en el índice HNSW cuantizado.1

## **Consideraciones Éticas y de Soberanía del Dato**

La implementación de la cuantización híbrida y los Axiomas de Hierro otorga a ConnectomeDB una soberanía inusual para una base de datos. El sistema deja de ser un espejo pasivo para convertirse en un árbitro de la verdad.1

### **El Modo "Abogado del Diablo" (Devil's Advocate)**

El DevilsAdvocate es el componente que revisa las inconsistencias lógicas. Si una nueva información (Challenger) intenta sobrescribir o relacionarse con una existente (Incumbent) con un Trust Score inferior, el sistema puede rechazar la mutación o moverla al shadow\_kernel para una revisión posterior.1

En un sistema cuantizado, el Devil's Advocate es crucial para detectar si una contradicción detectada es real o un artefacto del error de cuantización.8 Esta estructura permite que el sistema entre en un bucle de "autocrítica" antes de consolidar información en la memoria a largo plazo, mitigando los sesgos de los modelos de lenguaje locales utilizados en la síntesis semántica.8

### **El Dilema de la Honestidad Suprema**

El "pánico" del sistema ante una ruptura axiomática es definido como un acto de honestidad suprema: el sistema prefiere la inexistencia a la mentira.1 Al implementar TurboQuant, se dota al motor de la precisión necesaria para ser honesto. Un sistema basado únicamente en cuantización de 1 bit podría, inadvertidamente, "mentir" debido a colisiones de bits o ruido excesivo, llevando a decisiones catastróficas para el usuario.1

## **Conclusión y Recomendaciones Finales para la Fase 31**

La optimización del índice HNSW en la fase 31 de la versión 0.5.0 debe adoptar un enfoque de **Cuantización en Cascada** que aproveche la eficiencia de RaBitQ y la fidelidad de TurboQuant. Esta solución no solo satisface los requisitos de hardware (16GB RAM) sino que blinda el núcleo lógico contra la entropía semántica.

### **Resumen de Recomendaciones de Implementación**

1. **Dualidad Vectorial:** Implementar vectores de 1 bit para la navegación rápida en el índice HNSW y vectores de 3 bits (TurboQuant) para el refinamiento de candidatos y validación axiomática.1  
2. **Optimización FWHT:** Centralizar la rotación Walsh-Hadamard en un módulo SIMD optimizado para que sea compartida por ambos procesos de cuantización, minimizando la latencia de inserción.1  
3. **Gobernanza de Trust Score:** Utilizar el trust\_score inyectado en la fase 16 como el ponderador final que decide si un resultado de búsqueda cuantizado es aceptable para una inferencia crítica o si requiere una verificación de mayor precisión.1  
4. **Shadow Kernel Activo:** Asegurar que el shadow\_kernel en RocksDB actúe como el subconsciente del sistema, almacenando las versiones de alta fidelidad y las lápidas de los nodos olvidados, permitiendo la "Arqueología Semántica" cuando sea necesaria para resolver paradojas lógicas.1  
5. **Ciclo REM del SleepWorker:** Finalizar la integración del SleepWorker para que la Neural Summarization y la consolidación de nodos respeten el presupuesto de la Amígdala y actualicen el índice vectorial sin divergencias.1

Al seguir este camino, ConnectomeDB v0.5.0 se posiciona no solo como una base de datos vectorial eficiente, sino como un sistema de memoria cognitiva robusto, capaz de preservar la verdad y la lógica formal incluso bajo las restricciones más severas de hardware local. La fase 31 representa el paso definitivo hacia una IA que no solo recuerda, sino que comprende las fronteras de su propia certidumbre.

#### **Obras citadas**

1. Optimización de Bases de Datos Híbridas (1).pdf  
2. TurboQuant: Redefining AI efficiency with extreme compression \- Google Research, fecha de acceso: abril 5, 2026, [https://research.google/blog/turboquant-redefining-ai-efficiency-with-extreme-compression/](https://research.google/blog/turboquant-redefining-ai-efficiency-with-extreme-compression/)  
3. Beyond the TurboQuant-RaBitQ Debate: Why Vector Quantization Matters for AI Infrastructure Costs \- Milvus, fecha de acceso: abril 5, 2026, [https://milvus.io/blog/turboquant-rabitq-vector-database-cost.md](https://milvus.io/blog/turboquant-rabitq-vector-database-cost.md)  
4. Understanding 1-Bit LLMs and How They Differ from Multi-Bit LLM Models, fecha de acceso: abril 5, 2026, [https://metadesignsolutions.com/understanding-1-bit-llms-and-how-they-differ-from-multi-bit-llm-models/](https://metadesignsolutions.com/understanding-1-bit-llms-and-how-they-differ-from-multi-bit-llm-models/)  
5. A Visual Guide to Quantization \- Maarten Grootendorst, fecha de acceso: abril 5, 2026, [https://www.maartengrootendorst.com/blog/quantization/](https://www.maartengrootendorst.com/blog/quantization/)  
6. TurboQuant Explained: 3-Bit KV Cache at 6× Compression \- DecodeTheFuture, fecha de acceso: abril 5, 2026, [https://decodethefuture.org/en/turboquant-vector-quantization-kv-cache/](https://decodethefuture.org/en/turboquant-vector-quantization-kv-cache/)  
7. TurboQuant and RaBitQ: What the Public Story Gets Wrong \- DEV Community, fecha de acceso: abril 5, 2026, [https://dev.to/gaoj0017/turboquant-and-rabitq-what-the-public-story-gets-wrong-1i00](https://dev.to/gaoj0017/turboquant-and-rabitq-what-the-public-story-gets-wrong-1i00)  
8. When All Your AI Agents Are Wrong Together | by Dr. Jerry A. Smith | Medium, fecha de acceso: abril 5, 2026, [https://medium.com/@jsmith0475/when-all-your-ai-agents-are-wrong-together-c719ca9a7f74](https://medium.com/@jsmith0475/when-all-your-ai-agents-are-wrong-together-c719ca9a7f74)  
9. DEBATE: Devil's Advocate-Based Assessment and Text Evaluation \- ACL Anthology, fecha de acceso: abril 5, 2026, [https://aclanthology.org/2024.findings-acl.112.pdf](https://aclanthology.org/2024.findings-acl.112.pdf)

[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACsAAAAXCAYAAACS5bYWAAACT0lEQVR4Xu2WQUiUQRTHX0ihmGKIKBJhHpMuiSQdPHgQunWwSwcP3vTQxVDxIOFF8CbiISJYhBC8iBp0CAIvRpcgCCEsDxYRSF1CMBDq/2fm7c4+Z77dtbXT/uCHfm/225l582ZmRWrUOMVluAJH4QXTVk164Bb8AB/Ciz7eAjfgsH/OZAbmpPDyecBE7MM7sBXuwEdB+224C3uDWJRP8KYNVpkf8H7w/Ay+DZ65ootwEzYE8SKYzVX/97zogmtS3McS/BY8EybsC7xr4nluwAc2WEU0Y+EAGHsuLtshTfC1uJKMMiZuwDE6Yb3/vw62eyuhW1yttgWxZrgND4OYMg8/2iBphK/8X8tj2Cdu9t/FbYBBeAQ7Ch/LhBN9Af/Az4Ff4YlvswzAXzZIdIYWZoGFzqOEHc35+BD8KeUfb5wUs8TJcpXUnLjvnc5/soAm6BR8kSeBhUt+Rdwm4Cw527OgHYe7nt/LZ04gdgIlB8sMvrfBABY728N6qwTtmJtJ0WVOnUDJweruSy0rjxZ2lGovhXasy83vWfaxfv2QITlYwt3HcojBl/hyCtbwsbjNEjv+eLiz9nPirlRu5qcSz6gyAQ9sUGGHsZpk1nklspMUV+E7+FvcIGLcEneN8nOzknE7SeH8fWkbFO7Y2K4kHHA5cLILNhjATOp5nQUnvyfuR06SUhksxSS8Z4NngKXEVbhmG0K4RCM2WCbX4bpUfrNZ9JfYuG2w8HZ6I67jSrgkrlb5/r/AWp2CTyR78xXBGi2ntqoJB8qLghdRjf/KX98+ZXRPXn3JAAAAAElFTkSuQmCC>

[image2]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAkAAAAZCAYAAADjRwSLAAAAnUlEQVR4XmNgGAWkAGYgFgdiQSgfREsAMStMASMQVwPxBiB+BMR1QLwRiLcD8Xog5gQp0gXipQwQk/YB8R8g1gLix0B8B4ilQIoygTgWiGWA+DYDxAQNqIKJDFAruaEMOyD+DMQVIEFcoJgBogikGCsAmbSGAWIdyFqsAOTdm0C8BYg50OTggCj3pADxRyC2QpdABiA3CTFAAnaEAQCCiRe81QZzwwAAAABJRU5ErkJggg==>

[image3]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAYCAYAAADDLGwtAAAA40lEQVR4Xu2RMQ4BQRiFn0JCRKMS0aCTiEJCxwEUFBzCCShUGpVLaCRqiULiEgoKFAoUOo1Cwvv3n9mdlTiAxEu+ZOfNm+z7Z4DfU5RMSIo0yIbkQwkqQvrkaNYZsic9P2FUIVcyMmsbnPoJ6C9n0GDJeAVywkdQNiW0IDHj1ckdH8EBeSHcR77FG1ojQVbGvJCD4UaepG2DWeikUrwMHUKq7KAdpaunJvTk2BrQyR+k43h+abefXPSaJB0PRXImLceTrjVn7SlOlgimy5FusB1WlWzJHDqEPOdXyUWnoa/0l6c3jhwrEtFpyhsAAAAASUVORK5CYII=>

[image4]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAF0AAAAWCAYAAACi7pBsAAACe0lEQVR4Xu2YzUsVURjGX5FA0CKhMKQ/QPpYWEiC0qKFCKFGVEQLEaFd+9SNK/EPEEGQIFwkRtgyQhA0CGoRlSuhj22Q1kKCAsGehzNHZt47c+aMer0fnB/8uON7ZmTmmXfOmXtFAoFA4FhogQtwFDaosUAx7sKX8LQe0EzAp/CEqh8F7fCZLtYwbMpe+AZ2qjHCDOfhXLSdyRd4WRczaIp0cQUuwe/wD9xLDtcsD+FP+A9uw67k8D6X4FfYpwcsvBuL0acPQ3BMFxXn4Q14Bo5L/YTeLebJXRV36HwaZuErPWC5AB/oogOf0ONw33oJnZyU/NDJLTFPRSpPYKsuOihn6OyQQfgN3oMz0TZr1bLA+4Z+Cq5Jynk3w5Xo05dyhs6V/ze8H6txmzWOVQO+oXO6fiFm/wT2bhShXKGfgxvwnSSfPG6z9hG2xeqVwjd0wtdwXlcCLgp8c0mjQ8yrj/Y1fJ9Sp3xr0fiGfhPuirmgeHfYi/wrjreBFLhO6fPL8hFsNIflUjR0ZpzgLPykixE8CR6gHYFTKXXe0bQ3IN/Q+QRxv6zQOcZ9fOFx+hyzLJkCHBw6dPsPSiZ7B+WaXtjF7Oas0Hfg9Vi9UhQNnVN4CdOScjccFAmdN3NSTOg+N/aOmEXzcfQ3j+H2Fuy3O1UYrjH8NvoL9qixOHzyN3XRwg4r0kE+obMD2AkMO65Pd1wVs2awSz7DdXgxsUflsE+tlp2vuQZ/6KKFdyQvxDg+oR8FRefaaoMZ8W0sk7fi8atYBDv1ti4GEjBLZupszg9wWBcDB4ZZMtPc7xWc25+Lf8cHSmF2y3BAD7jgHJr3s20gG2ZXy+tQ/fEfHuqLSHbFd5gAAAAASUVORK5CYII=>

[image5]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAGMAAAAXCAYAAAAfiPFCAAAEQUlEQVR4Xu2ZWahOURTH/8bMIS7hBXlRHiQekEwpQySShChlCKHEgykpeeFB5syZMjwY7gsppWTIEIlcuskUSZGiTOtvncP5lr3P8H3fuYbrV/9y9z7fOfustfZa62xA7aGeaKxoXIlqh/+UTDfRQdHuEtUD6WEA0HkdjFpFL6qNrEHNG6EldDddFk0WTRDNFV0XHQ7mY6E3h4m2i+6I5hRO1zidRAug6yqWFaLNdrAEaBfah3ZKWld30VM7KCwV3bKDUeipo6LXopnB37+b+YGKpQ00Evub8TqiftBodWmMqMGPqwuhXbimt6LK4G8fU0SP7SDUGVV2MMpI0SfRMjvhoaloh6i9nYhhHTSq0nISGl3Fslh0XNTajHPty0X7RV9FF6HvEuqc6KVoItRxLmgn2ot287FRdNqM8X57RPfNeAGMBi6MXktDc9E+aFGKo7Nom+ge9P78TVp2wh+hSVSIbogGw38PpsFXop52AupEGtuXqmknvg/t5oI1ivVirRlvC01Rh8x4AaEzfDe3pHUGo3II9LoszmAETbKDGVgiOoP4wj0AajDXNV1EjwLx35bxiLcXd3Q1tLZEGS564xgvIC9nRMniDEZtlhQYhb+7CU0hDc1cFEY3U4kL7qZj0DXT8JYke4X1omtkjPXlrGgr/Lv1O0nbzpKnMxqJjtjBAO6YeVBnuegIdcQoxL8wf/8A/prE9zsPf+r2pXUGAGvOe2iRfxjogmgg/DWoAF9B8nUeU6EPYOdl53i966FpncGPK+ZsS11RH9E70Uq4n8H0dErUwk4YmCY+QI3ughHNyPbtjKwNTyZY3bmFbKtWjDN8RTOtM1bj15aW62DUrxd9hOZj1668IhoB9/OjsLByPT6Y2+msZ3DvnjDl0G6u+aKgh1nEFooam7k48kpT4feOLZrcFXxpph+u94toUTAehbuimRmzMA2y5aSxfWyCrpddj8+xtBft9hy6ppKhUfmSL0Sj4d76LvJyBiOS1/gMQKZB73UXulvCNbMO8Pf1g799hCmI8lENtUkvMx7CZ/I7hNecQBl3B/kTCjh3QyW0F4+DhpgO3R3XoA5hB8WCm+QIwg7KVXwJo30DNE3GBaavgJeFpL7ZUqwzeEjmO9OZDa0XaagQ3YY6ZBa0dvHeSfCI5Co0RUV7faYu2oD35HlSnCNI6Ayuuez4+mb26bvws0ULxY8htm7VjrkDoiZQ6Kwq6L2tos9iRLKd7R0ZS4Jfx5+hxuUH3uDC6QLYWLD7sWsIxVb0EvSENU3t9NmrLOR68xT0hX5kpTFECP+/4B50d+xF/AdeucnVXrluuxSsQnHHHzwMfAJtv21nlSe0U+7OyKUgJcCCzcJt29k0sIOaAX8dyousDU8mWLCGQgsY2zXmb/s1nhdboId2fwO0C+1DOw0yc7nAU0wWXt9RQbnh0UVNR3ax0C6uU97//Ct8A9NQCmq6gehdAAAAAElFTkSuQmCC>

[image6]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAYCAYAAAC8/X7cAAAC9UlEQVR4Xu2WS6hNURzGP4nI+1VIOsjEY2DgkYEyISkMUBQzAwYmyqOUMjExkcdE6qZEUkqSCElJSIwkj0Qe5VFIGSi+z38vd63/Xnufc+49SN1ffd271+us9X+tBfTRERZT16gX1L3i+79hFnWEGkz1o3ZQb5IRfxFtYAW1xXfUsI76Ti0vvqfBPBEYDjvgjKitlhHUIeo1tYkaC9vYUeoLtbX49iygblNTfIdjILWNukCdpwal3diIvAfWUHepSb4jZj71lLpIjXZ9A6gu6iu1Ku3CGOoW7HCtsoTa7doa1B1qtWsXCrFz1AHfEfOWukyN9B0Fi2BeuIl0zHrYwadGbc3YBztEQOuddm2eZdRL3xhoUB+oua49ZiL1hHpHzYnaFQonYV5qhSEwQ4Vw0DwdaF7x3Sj+ejT+sW8UWuAYdRj5+A6EA8RJJ5Qvm6PvHMNg80ehO1EV//rtg9Qeam0hHSaHxp6BGSBhIfWp+FuHrC7rf0Pqas1VeOVQLsk7SkAVggfUDZgRhPJJ//+I9LHoy6G8KSWyTqyJsk4dSi6Nk8XjsuZDKuY+0ryaTD2E5VJP2AAX5iEetbE6FFpdsHFXYSERUFgpPDyqHD6vVNevU4+itnZYiTR8f21EG2p2gJ2wMSqXKpsxVQdQXmlejCyodZrlTBU6gPSbkBjNDvAKVmZ1V3ieoRyXwbO+LCphFeNap6pc16HNl/agOq5EkstzKBE/I3/BCB8mQiF3wrXrVtdFpUQeh7J3WkFzSs8KeUEVQg8pX0ZnwipI6dQRSsjErQV6ioTqJOMopOTpU7ASOrvoawetkS02Q2Hl8Qrsh6VLsAqzPRqXQxbN1W4ZRsl6HHZ7y9O7qPewtb2xmhEKQOU8xeRSdF8oclX/ZEQeXYC+MgXUNh7pLS0LKpzaReGYe+j1Gi38HM0vwt6yF2aojiOX6pWox1ir76F2mQ4LR1/VOsoE6izqE75dZBzl4H78OeMk6Ed6Ut+rUA4qZyoTt49/yU97xos5VTH3sAAAAABJRU5ErkJggg==>

[image7]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAFIAAAAXCAYAAACYuRhEAAAEj0lEQVR4Xu2XW6hWRRTHV0RUaJaVGBF2rIe0C6JhVzCC6EJEERkV1EPgQyESFCTRk6A++GKoIIQghmgXKCgsiqIiwnoIrZCiNDEyQSMiBKGg/r/WXn7rmz37O8frUfp+8Oecb2b2zOw16zLbbMiQIf8zLpKekT6QvpWWSdP6RvRzhvSktFE6t+jLTJEelB6XVklzUt+I9LC0QFoqTU59J5OJ0n3So9KKpm2+9KZ0QQwajfOl1dJe8xe6WJouvSz9KS0yN1rJTdKX5mMHcaf0ivSb+RpXp757pbelv6QfpUtT38lklrRO2iMdatrOMrfB2ub/gdwo7ZTeky4s+nh4vXRQeqC/6z/v3Wpu5LGAV26XPpEmFX0cEl49noaEc6R3pF2p7Vpz++AMA9lnHspd7jvP3Cs/t/4xj5kvcEVqG0TMQ2jX2GDjb8grzT0SYwYc8hrpXRuQvkbMw21u0Z7hxXjB/dLspi1ObpONweUbnpX+trZnB6eCIUkz7HFx0c6eef+qnTAAOQFr1/JfEIZkARYCchy57qkY1AFrXCKdLb1h7fyY6TIkc9xgXpD42wWHy1qMR3eYF85yvsyZ0lTrPbfcPD+WYRyeijO04KF/bPQq+ZD5uGwEJswemmFDJOdfzfMvcGDMwZpdlIakYv4uPXJ4hEMbfcFz5jn8nub3EukP6XbpVvOoK8FxeC7PgxeyRw68jDJ+0/5R0W4TzPMiDw6CBdebj2OS85r2F63uPfC0tcOD8dmja5SG/Eb6wtoHTds2c0+K98jp6X7z/XblYsDjMH42WDhM1evM9/d92YhBMMxohqSQUMEwAgYKypcOCBEMQAG7LrWTTwkNQqSLck7WzIcX0BbhF7k6GzIMgmfWoGBQOOKKE2B4iiFFsUbsr4+xGjLcnWsO152ARWuGjGRdGiAqIS/eRWnIMgqC2DeeBxQCvOsF613Xdlg9pIF0RFr6IbXh9Xg61zOuaTWqhoyYH2RI7k+/mHtX5LoAA+OplxXthEUtrDh9wptUwaZJ8iWlIXmmy5DZc26Rtkifme/1Y2lG01cjDjtfcaJ4bjTfY+0qyP64A7fAmNzan7d21b5G+s7aBgziThheEcyUfrZeUWFeisVP0uXSE9LCpi+DYTdbf0ogRCkseX/8PSDd3fyGBU0bX068D/OzrxxBGdq/MnckoCCxLgWKQ7nNep+IASmL/NhZLPm+5OQ/NN8Qet/c9Tsvn9b7SqlNzInjra+bF4KV5hWcNV6z9mlH+sjC6+B6809QvvvxiK/NDzlzlXl/OQefnFTm0kmAb32eecv8LnyzeToghZE/Rw6PdHAojB03gyq82F3m9zSEm9dCL8PmuH/WQg/wdk4x+vId72hgHkK+XIvUwoGSNvLc06RXrV30MrxjvhHEnmt5nMOmiNJ/3KFK7jYPh/GCEM4VO0MbfWX6OVJwND6PMeYJAa98yTxcj9bTjhUKIjmZip2jiP1w/aGPMccCeZ2cyr31hMFpUTEpKOMFFZqKTY6j2JBLMSBtZT49UkakT6276B538ICyiJzukCvLnDxkyCnMv8ZfCpGU5lmnAAAAAElFTkSuQmCC>

[image8]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAsAAAAYCAYAAAAs7gcTAAAA4ElEQVR4Xu2SMQ5BQRCGRyHRSAiFQucEKh2dWqFwAVcQhTiAViNKCo2OSuUONCJRiEqi0igU/v/tjOzb7AUkvuTLm8zM7pvMeyK/TRsu4QOeglqUKjzDdViIwdtfcBgWYozgEzbDQkgObsWNwXFIEZZhxpqMGryKO8CDc7iBB9j3+hL8eQuwAcfwrbkUnJfNPTjT3ATeYN2aCGe8wL242+6aixKurAWnGnP+vMYJbPJXxtda8wB2NJasuC/mr4zNvKAEF7Ci+STgv7ASd5DweYQ72NXcF67KGg1+jNSsf3w+FoElrSo4UnUAAAAASUVORK5CYII=>