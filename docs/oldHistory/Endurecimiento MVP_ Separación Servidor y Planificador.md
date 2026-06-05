# **Diagnóstico de Madurez de Vanta DB y Plan de Acción Detallado para el Endurecimiento del MVP**

La evolución técnica de Vanta DB demuestra una base de ingeniería sólida, caracterizada por una latencia de microsegundos en operaciones transaccionales y capacidades avanzadas de búsqueda híbrida y multidimensional.1 La compilación y ejecución del sistema se sustentan en la versión estable del compilador de Rust 1.94.1 y la herramienta de construcción Cargo 1.94.1, lo cual garantiza el acceso a las últimas optimizaciones de seguridad de memoria y rendimiento del compilador.1 El historial reciente de modificaciones confirma un esfuerzo enfocado en la unificación de contribuciones, la actualización de dependencias de red y asincronía como tokio (versión 1.52.2) y reqwest (versión 0.13.3), y la optimización de primitivas criptográficas y de hardware mediante cpufeatures (versión 0.3.0).1

Las métricas del sistema revelan un rendimiento excepcional, registrando tiempos de ejecución promedio de ![][image1] para operaciones básicas de inserción y lectura de nodos en memoria.1 Sin embargo, para consolidar Vanta DB en su fase de endurecimiento del Producto Mínimo Viable (MVP), es fundamental identificar y subsanar las deficiencias arquitectónicas y los componentes faltantes que impiden su despliegue seguro en entornos de producción de baja latencia.1

## ---

**Diagnóstico de Deficiencias y Componentes Faltantes en la Fase del MVP**

La transición desde un MVP funcional hacia un motor de base de datos de calidad industrial exige un análisis riguroso de las brechas técnicas actuales. A partir de las auditorías de código internas, los planes de trabajo existentes y los registros de limpieza del proyecto, se identifican áreas críticas que comprometen la modularidad, la mantenibilidad y la durabilidad del sistema ante fallos imprevistos.1

### **Acoplamiento de la Capa de Transporte y Protocolo**

El motor central de base de datos mantiene dependencias innecesarias con librerías de red y protocolos específicos.1 La presencia del servidor y del protocolo de contexto de modelos (Model Context Protocol o MCP) dentro del mismo espacio de dependencias limita la portabilidad de Vanta DB.1 Esto impide su uso como base de datos embebida ligera, ya que cualquier cambio en las interfaces web o de comunicación repercute directamente en la estabilidad de la compilación del motor de almacenamiento principal.1

### **Ausencia de una Representación Intermedia en el Planificador de Consultas**

El módulo de planificación de consultas realiza una traducción directa de expresiones lógicas a instrucciones de máquina virtual sin una fase de optimización física estructurada.1 Al carecer de un Árbol de Sintaxis Abstracta (AST) independiente de la ejecución y de una Representación Intermedia (IR), el sistema no puede reorganizar de manera óptima las operaciones de escaneo, la aplicación de proyecciones o la combinación de filtros de texto con vectores de alta dimensionalidad.1 Esto resulta en planes de ejecución rígidos que degradan el rendimiento ante consultas complejas de alta densidad.1

### **Brechas en la Consistencia de Backends Híbridos y Recuperación**

Aunque el motor soporta múltiples backends de persistencia en disco como RocksDB y Fjall, así como almacenamiento volátil en memoria, el mecanismo de sincronización con el registro de escritura previa (Write-Ahead Log o WAL) carece de una coordinación transaccional estricta ante caídas abruptas de energía.1 Las políticas de limpieza descritas en los protocolos de recuperación de mutaciones requieren una sincronización de checkpoints más estricta para evitar la corrupción de índices secundarios y derivados durante los procesos de restauración en frío.1

La siguiente tabla detalla la comparación del estado actual frente al estado objetivo requerido al finalizar el proceso de endurecimiento del MVP:

| Dimensión Técnica | Estado Actual del MVP | Estado Objetivo Endurecido | Impacto en la Estabilidad |
| :---- | :---- | :---- | :---- |
| **Separación de Red** | Servidor y protocolo MCP integrados con dependencias cruzadas en la raíz.1 | Servidor aislado como crate consumidor independiente; motor compilable de forma aislada.1 | Alta reducción de deuda técnica y aislamiento ante fallos de red. |
| **Arquitectura del Planificador** | Traducción directa del parser a bytecode de la VM sin abstracción de plan lógico/físico.1 | Pipeline desacoplado con AST, planes lógicos/físicos y optimizador estático de reglas.1 | Reducción de la latencia en consultas compuestas y extensibilidad del motor de ejecución. |
| **Durabilidad y WAL** | Recuperación del WAL expuesta a escrituras incompletas y asincronía de backends.1 | Sincronización transaccional con CRC32, checkpoints coordinados y GC en segundo plano.1 | Prevención absoluta de corrupción de datos y consistencia determinista ante fallos. |
| **Telemetría de Memoria** | Registro básico de MB y deltas mediante sysinfo::Process sin contención activa.1 | Monitoreo proactivo que estrangula consultas que exceden límites de memoria virtual.1 | Mitigación de caídas por agotamiento de memoria del sistema (*Out-Of-Memory*). |

## ---

**Plan de Acción 1: Separación Arquitectónica y Desacoplamiento del Servidor**

### **Acción Principal y Detallada**

Se debe realizar el aislamiento completo de la infraestructura de red, protocolos de transporte y API de comunicación, encapsulándolos exclusivamente en el subdirectorio de trabajo vantadb-server.1 El motor de almacenamiento y consulta central (vanta-db core) debe purgarse de dependencias de red, de modo que pueda compilarse de manera aislada como una biblioteca nativa reutilizable por otros componentes, tales como interfaces CLI u otras herramientas de automatización.1

### **Desglose de Acciones Secundarias**

El proceso de desacoplamiento se divide en las siguientes etapas secuenciales de desarrollo:

* **Estabilización de las Interfaces SDK y API Internas**: Se requiere rediseñar src/sdk.rs y src/api/mod.rs para que actúen como la única interfaz de entrada pública para el motor, encapsulando las estructuras de datos complejas e impidiendo la exposición de mutexes internos, punteros directos o componentes de gobernanza de baja granularidad al servidor.1  
* **Migración de Protocolos Específicos al Servidor**: Se debe trasladar toda la lógica de serialización y control del protocolo de contexto de modelos (MCP) ubicada en el núcleo hacia vantadb-server/src/mcp.rs, asegurando que este invocador interactúe únicamente con el SDK expuesto por el motor de base de datos.1  
* **Depuración de Manifiestos de Dependencias**: Es indispensable modificar el archivo Cargo.toml raíz para eliminar la importación de frameworks de red, protocolos de transporte y dependencias de serialización específicas del servidor web, reubicándolas de forma exclusiva dentro del archivo de configuración vantadb-server/Cargo.toml.1  
* **Reconfiguración de Puntos de Entrada**: Se debe reescribir la inicialización del sistema en vantadb-server/src/main.rs y la gestión de conexiones en vantadb-server/src/server.rs para instanciar el motor como una dependencia externa, facilitando la inyección de configuraciones de memoria y límites de disco definidos al momento de levantar el servicio.1

### **Archivos Involucrados y Lógica de Negocio**

Esta reorganización arquitectónica requiere la modificación y supervisión de los archivos del proyecto que se describen a continuación:

| Ruta del Archivo | Rol en la Arquitectura | Descripción de la Modificación e Impacto |
| :---- | :---- | :---- |
| .\\Cargo.toml 1 | Configuración del Espacio de Trabajo | Eliminación de crates de transporte como reqwest o características de red pesadas de tokio del núcleo del proyecto.1 |
| .\\vantadb-server\\Cargo.toml 1 | Configuración del Crate del Servidor | Declaración del núcleo de vanta-db como una dependencia local asíncrona e inclusión de dependencias HTTP y gRPC requeridas para la exposición de servicios.1 |
| .\\src\\sdk.rs 1 | Capa de Interfaz del Motor | Ampliación de los métodos públicos para ofrecer firmas asíncronas limpias que acepten tipos básicos de Rust, traduciéndolos a llamadas internas del motor.1 |
| .\\src\\api\\mod.rs 1 | API de Integración | Consolidación de los puntos de contacto lógico para la API estructurada v2 de almacenamiento, sirviendo de puente con el servidor.1 |
| .\\vantadb-server\\src\\server.rs 1 | Servidor de Red | Rediseño de la lógica de hilos y sockets para recibir solicitudes del cliente, parsearlas, invocar al SDK de forma segura y serializar la respuesta.1 |
| .\\vantadb-server\\src\\mcp.rs 1 | Adaptador de Model Context Protocol | Ajuste de las funciones del protocolo MCP para utilizar exclusivamente las estructuras públicas expuestas en src/sdk.rs.1 |

### **Relaciones en Otras Partes del Proyecto**

La separación de la capa del servidor afecta de manera directa al inicializador del motor de ejecución central en src/engine.rs 1, el cual ya no debe invocar hilos de escucha web, limitándose a coordinar la persistencia, la planificación de tareas y la ejecución de consultas. Asimismo, el punto de entrada de la interfaz de consola en src/bin/vanta-cli.rs debe ser adaptado para consumir la base de datos a través del SDK desacoplado, garantizando que el CLI funcione como un cliente embebido ligero sin necesidad de instanciar un servidor TCP de red.1

### **Protocolos de Revisión y Verificación Posterior**

Una vez aplicados los cambios, es mandatorio ejecutar la suite de telemetría de memoria en tests/memory\_telemetry.rs para constatar que el aislamiento del proceso del servidor de red no introduzca fugas de memoria o bloqueos asíncronos en los ciclos de desasignación controlados por el sistema, usando el colector sysinfo::Process para asegurar la estabilidad de la memoria virtual.1 Posteriormente, se deben correr las pruebas lógicas integradas de red ubicadas en vantadb-server/tests/api/server.rs y tests/api/structured\_api\_v2.rs para certificar que el transporte de datos mantenga la misma consistencia funcional previa a la modificación.1

## ---

**Plan de Acción 2: Refactorización y Aislamiento Estructural del Planificador de Consultas**

### **Acción Principal y Detallada**

Se debe llevar a cabo una reestructuración de src/planner.rs con el objetivo de separar la interpretación sintáctica del análisis semántico y la posterior generación de planes físicos de ejecución.1 Se requiere eliminar la dependencia directa del planificador sobre la máquina virtual de expresiones de src/eval/vm.rs y el motor de ejecución en src/executor.rs.1 Esta separación permitirá la introducción de un Optimizador de Consultas capaz de aplicar transformaciones algebraicas y heurísticas antes de compilar el plan de ejecución definitivo.

### **Desglose de Acciones Secundarias**

La transformación de la arquitectura de consultas del motor contempla las siguientes actividades detalladas:

* **Diseño del Árbol de Sintaxis Abstracta (AST)**: Consiste en desacoplar el parser de sintaxis Lisp en src/parser/lisp.rs para que retorne nodos estructurados de un AST fuertemente tipado en lugar de compilar directamente en bytecode de evaluación dinámica de la VM.1  
* **Implementación del Plan Lógico Intermedio**: Se requiere crear representaciones lógicas para operadores de bases de datos tradicionales, incluyendo escaneos, filtrados, proyecciones y búsquedas por similitud, abstrayéndolas de los detalles físicos del almacenamiento.1  
* **Creación de la Fase de Optimización de Planes**: Desarrollo de reglas de optimización estáticas encargadas de optimizar las expresiones de consulta, empujando los predicados restrictivos lo más cerca posible de las fuentes de datos físicas y seleccionando de manera inteligente entre escaneos de bits o búsquedas vectoriales exactas Top-K en src/vector/mod.rs.1  
* **Generación del Plan Físico y Enlace de Ejecución**: Rediseñar la interfaz de entrada del ejecutor en src/executor.rs para aceptar un plan físico resuelto, el cual contendrá las referencias definitivas a los índices de texto en src/text\_index.rs o estructuras columnares en src/columnar.rs.1

### **Archivos Involucrados y Lógica de Negocio**

Los componentes de procesamiento sintáctico e instruccional del motor que requieren cambios se detallan en la siguiente tabla:

| Ruta del Archivo | Rol en el Crate | Modificación Requerida e Impacto en la Lógica |
| :---- | :---- | :---- |
| .\\src\\planner.rs 1 | Planificador del Motor | Creación de las fases parse, analyze, optimize y codegen, rompiendo el flujo monolítico actual de compilación hacia la VM.1 |
| .\\src\\executor.rs 1 | Motor de Ejecución | Modificación para interpretar la estructura del plan físico generado y coordinar la invocación ordenada a los backends de almacenamiento.1 |
| .\\src\\eval\\vm.rs 1 | Máquina Virtual | Reducción del ámbito operativo de la VM, limitando su uso a la evaluación condicional de campos dinámicos individuales y cálculos aritméticos.1 |
| .\\src\\parser\\lisp.rs 1 | Parser de Expresiones | Refactorización para emitir exclusivamente una estructura AST limpia e inmune a las variaciones del motor de ejecución físico.1 |
| .\\src\\query.rs 1 | Definición de Consultas | Ajuste de las estructuras y firmas de consultas que son admitidas por el motor a través del SDK.1 |
| .\\src\\columnar.rs 1 | Representación Columnar | Integración de los accesos rápidos a nivel de columna con el planificador físico para optimizar consultas de agregación masiva. |

### **Relaciones en Otras Partes del Proyecto**

La reestructuración del planificador altera de manera directa el procesamiento de búsquedas vectoriales e indexación híbrida en src/index.rs y src/vector/mod.rs.1 El planificador físico optimizado debe verificar la disponibilidad de índices aproximados (HNSW) en src/vector/quantization.rs y transformaciones de proyección en src/vector/transform.rs antes de emitir instrucciones de escaneo exacto en el ejecutor, garantizando el mejor camino físico para optimizar la eficiencia computacional de las consultas de alta dimensionalidad.1

### **Protocolos de Revisión y Verificación Posterior**

Una vez integrada la nueva lógica del planificador, es crítico realizar la validación de las regresiones lógicas de consultas ejecutando las pruebas unitarias en tests/logic/parser.rs, tests/logic/executor.rs y tests/logic/integration.rs.1 Para descartar caídas en la fidelidad o latencia de la búsqueda híbrida, se deben ejecutar los benchmarks de desempeño en tests/certification/hybrid\_retrieval\_quality.rs y tests/certification/competitive\_bench.rs, certificando que los planes optimizados mantengan la calidad de recuperación y una velocidad de procesamiento inferior al umbral crítico de microsegundos.1

## ---

**Plan de Acción 3: Consolidación de Persistencia, WAL y Robustez ante Fallos de Almacenamiento**

### **Acción Principal y Detallada**

Se debe robustecer la capa de persistencia mediante la unificación de los mecanismos de recuperación del registro de escritura previa (Write-Ahead Log o WAL) y la sincronización estricta de las operaciones mutativas con los motores de almacenamiento RocksDB y Fjall.1 El objetivo primordial es asegurar la consistencia del estado del sistema, evitando que la pérdida repentina de energía deje la base de datos en un estado corrupto o inconsistente entre los índices primarios, los índices derivados y los archivos de registros en disco.1

### **Desglose de Acciones Secundarias**

La implementación del sistema de persistencia y consistencia endurecido se estructura mediante las siguientes fases de desarrollo:

* **Refactorización del Analizador de Recuperación del WAL**: Se requiere robustecer el módulo de arranque de src/wal.rs para parsear secuencialmente los bloques del registro de transacciones, validando la integridad física de cada payload de datos mediante sumas de verificación CRC32 y aislando las escrituras parciales incompletas debidas a cortes de energía para truncar de manera limpia el archivo de registro en la última transacción consistente.1  
* **Implementación de Checkpoints Unificados de Almacenamiento**: Desarrollar en src/storage.rs una coordinación estricta para sincronizar las mutaciones de datos en memoria con los backends físicos activos (src/backends/rocksdb\_backend.rs y src/backends/fjall\_backend.rs).1 El WAL solo podrá ser depurado o rotado cuando todos los backends involucrados notifiquen de manera exitosa la sincronización física de sus respectivas páginas a disco (*fsync*).  
* **Desacoplamiento del Ciclo de Recolección de Basura (GC)**: Separar físicamente la lógica de src/gc.rs de la ruta crítica de escritura del usuario para evitar bloqueos prolongados sobre los índices de la base de datos.1 El inicio de los ciclos de compactación y purga de versiones antiguas u obsoletas debe ser delegado y programado de manera exclusiva por el planificador asíncrono en src/governance/maintenance\_worker.rs.1  
* **Robustecimiento del Filtro de Admisión ante Estrés**: Configurar el módulo src/governance/admission\_filter.rs para monitorear continuamente el estado de las colas de compactación de los backends de almacenamiento, estrangulando momentáneamente las solicitudes de escritura entrantes si el disco presenta demoras que amenacen la integridad del WAL o saturen los deltas de memoria virtual controlados en la gobernanza.1

### **Archivos Involucrados y Lógica de Negocio**

Los archivos críticos que participan en los procesos de escritura física, persistencia duradera y consistencia transaccional se listan a continuación:

| Ruta del Archivo | Rol en el Proyecto | Modificación Técnica e Impacto en el Flujo |
| :---- | :---- | :---- |
| .\\src\\wal.rs 1 | Registro de Escritura Previa | Inclusión de sumas de verificación, alineación de bloques e introducción del modo síncrono configurable de persistencia para entornos críticos de producción.1 |
| .\\src\\storage.rs 1 | Gestor de Almacenamiento | Unificación del flujo de confirmación de escrituras y coordinación del guardado concurrente asíncrono.1 |
| .\\src\\gc.rs 1 | Recolección de Basura | Modificación de los algoritmos de escaneo de marcas de borrado (*tombstones*) para trabajar de forma no bloqueante sobre los índices del motor.1 |
| .\\src\\backends\\rocksdb\_backend.rs 1 | Adaptador de RocksDB | Implementación forzada de opciones de escritura de alta durabilidad (parámetro sync activado durante operaciones clave).1 |
| .\\src\\backends\\fjall\_backend.rs 1 | Adaptador de Fjall | Alineación de las directivas de checkpoint de Fjall con los ciclos de compactación global del motor de gobernanza.1 |
| .\\src\\governance\\maintenance\_worker.rs 1 | Trabajador de Mantenimiento | Programación inteligente de compactaciones en disco e invalidación de índices antiguos en momentos de baja carga operativa.1 |

### **Relaciones en Otras Partes del Proyecto**

La capa de consistencia en el almacenamiento interactúa de forma activa con los componentes lógicos de resolución de conflictos de transacción ubicados en src/governance/conflict\_resolver.rs y con la gestión de invalidación de cachés de búsqueda en src/governance/invalidations.rs.1 Una mutación abortada o declarada en conflicto por la gobernanza de concurrencia debe ser inmediatamente revertida en el almacenamiento físico, impidiendo que sus cambios parciales lleguen a registrarse en el WAL activo o en los motores secundarios.

### **Protocolos de Revisión y Verificación Posterior**

El aseguramiento de la persistencia frente a caídas de tensión eléctrica exige la ejecución exhaustiva de las pruebas de caos e integridad física localizadas en tests/storage/chaos\_integrity.rs y tests/memory\_brutality.rs.1 Adicionalmente, para corroborar el estado limpio de recuperación, se debe correr de forma aislada la suite completa de tests/durability\_recovery.rs, tests/text\_index\_recovery.rs y tests/derived\_index\_recovery.rs, garantizando que todos los registros restaurados reconstruyan fielmente los índices asociados sin pérdida de datos.1 Finalmente, el comportamiento de las réplicas en frío debe verificarse mediante tests/fjall\_cold\_copy\_restore.rs para validar la portabilidad de las bases físicas creadas.1

## ---

**Verificación de Calidad y Suite de Certificación de Endurecimiento**

La validación final de todas las refactorizaciones aplicadas durante el endurecimiento de Vanta DB se articula a través de una serie de suites de certificación que evalúan el rendimiento del motor bajo condiciones severas de concurrencia y límites de hardware.1 Estas verificaciones garantizan que las optimizaciones en el planificador y la separación del servidor mantengan la calidad matemática de los algoritmos de búsqueda y la robustez del consumo de recursos de la plataforma.

La siguiente tabla presenta la configuración técnica de la suite de certificación requerida para aprobar de manera exitosa la fase de endurecimiento del MVP:

| Suite de Certificación | Ámbito de Validación Técnica | Criterio de Aceptación Crítico | Métricas de Rendimiento Esperadas |
| :---- | :---- | :---- | :---- |
| **HNSW Recall & Validation** | Precisión de búsqueda vectorial en índices de aproximación jerárquicos.1 | El planificador refactorizado no debe degradar la precisión geométrica de la búsqueda aproximada.1 | ![][image2] frente a búsquedas exactas Top-K empleando datasets estándar SIFT.1 |
| **Stress Protocol & Hardware Profiles** | Estabilidad del sistema en condiciones de estrés y limitaciones físicas del host.1 | Cero fugas de descriptores de archivos, exclusión mutua de bloqueos y ausencia de pánico en la compilación multihilo de Rust.1 | Soporte continuo de carga de trabajo transaccional pesada durante 4 horas en perfiles de hardware restrictivos.1 |
| **Memory Telemetry Baseline** | Evaluación continua del consumo de memoria en el ciclo de vida del motor.1 | El delta de memoria virtual medido no debe acumular un crecimiento lineal residual tras la liberación de recursos.1 | Mantener una línea base de memoria en reposo estable, medida con precisión instrumental mediante sysinfo::Process.1 |

Para la ejecución coordinada de estas suites y la generación de los informes de auditoría técnica que avalen la transición de Vanta DB a una versión de calidad industrial, se requiere ejecutar de manera automatizada las herramientas y scripts del proyecto descritos en el siguiente cuadro descriptivo:

| Herramienta / Script de Verificación | Propósito de la Ejecución en el Proyecto | Archivos de Resultados Generados |
| :---- | :---- | :---- |
| .\\dev-tools\\smoke\_test.sh 1 | Verificación rápida de la compilación e integridad básica del CLI y el Servidor.1 | Estado de salida de consola exitoso sin errores en el arranque asíncrono.1 |
| .\\dev-tools\\scripts\\test\_runner.sh 1 | Orquestación estructurada de la suite completa de pruebas unitarias, de integración y rendimiento.1 | vanta\_benchmark\_report.json con desglose detallado de microsegundos por comando.1 |
| .\\dev-tools\\scripts\\download\_sift.py 1 | Descarga y preparación local de vectores del dataset de referencia SIFT para validaciones aproximadas.1 | Estructura SIFT local consumida por las pruebas de certificación.1 |
| .\\tests\\certification\\hnsw\_recall.rs 1 | Validación estadística de la convergencia matemática del algoritmo HNSW nativo de la base de datos.1 | vanta\_certification.json con registro de precisión y deltas de cómputo geométrico.1 |

La finalización de estas fases de verificación permitirá cerrar formalmente las tareas pendientes descritas en la hoja de ruta en docs/operations/ROADMAP.md y resolver de forma estructurada los elementos señalados en la auditoría de limpieza en docs/audits/2026-05-04-cleanup-candidates.md, asegurando que Vanta DB alcance un nivel de solidez arquitectónica sin precedentes en su desarrollo.1

#### **Obras citadas**

1. snapshot\_2026-05-18.md

[image1]: <data:image/png;base64,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>

[image2]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAIwAAAAZCAYAAADja8bOAAAEiklEQVR4Xu2aW6hVVRSGf8mgSLMLlJfiYNhVEcFSlNQwe4gIoyLTesoHL+RDBYXhQ+KLIhUkVEQgPkj3KFKMiNj0UGRCPpiCF9CIIqOX0MAga3yOOVtzrbPOXmeD55zN3vOHn7OZY+y5LuOfY4459pEyMjIyMjIyMjIy+gDTjI828B7jhODfzbjd+ITxNeNbxjlhfLz8GdaH8a3Gq4Ito0MsM/5kPG38N/DvMBZ5JozvNy42jrvwze7DWpWf48EwfoXxXRXPcco4JdiacIlxYnUwwzNIS/5CXyybLmC28YTc/py6VzRkkw9VFkzETcZf1ZlgbpAvlC+M89W9zz3qSAXzQtn0P0j32M8ZF1Vs3YRdqhcMIkEsnQgGIBLE8q3xgPEBeebpawxHMHONZ+U+1AndiostmAiEQ6Yl25w0Pm68NHXoJwxHMHcZ/1L/CibFrcaPjb8YnzJeXjb3PoYjmLgl/WO8r2KL4MWtMn5pPGjcrKGDw+okqNQcFNefyL+bvnxWNZntFfl8kM8EbCiMhmAipht3Gv80bjJeWTb3LpoEM8N4VC6Wp1Vf/A0YDxl/kAeZY+6n8lPXw4kfoJj8Xi4UbAQQQZ43fqZCNI/J7+lN41T5NV4PfkMV36MpmAjmQ8h/GLcbry2bew+pYCjs6FfA9+QvGKHsM94R/KuYZPxanqIRV8T1xiNyIV0XxvAlA1EPUUxGIFSunwb07TDWUtELQkyfy4vvBWEsxVgIJoIMg2B4NmqcnkUqmC3yFwpvNr4Rxr+RZ4Y6rJb77NbgVU/QyQj0fAAvEt+9xsuik3xuVulDKua4RV4v3RudAjj6M8ezlXEwVoKJ2xOLZoN6vK5ptyURvI3BRmYgQ6TAjlCwv6/BneJoW6Nyn6Su39MOiGuWynNW7xWMpmB4dk5OZF/6VH1zcmonGJAeqe+v2NLvEsiqYCIHKr5116mCgNBdpnlGlmrJC+mxFgz3xXZKb+ZH43L1WW+mKZCxS1pnZ+WzvdTZqmi6TgqC8rzcl/oo3Q5jvVM3x0gKBlEsldd5iJjPfSWUiKZAxpeNnYBU8ZKKDEOgq0BUsV6hJsF3p+p9I2JW4/chaqkUqWD4AZXVHjESgkEUZBGyCY07tqF2997z6EQwLRUnFo7OC40zjb8bjxlvDLYIXuwOFaeGBfITTvVEBSgUqXHwIeDV6wHmQ2zxXvFLRXwxBUM9wn1Tn9Coa9f/6Ruw8nkR9E940a/Kj8Bpqo1HWexsTWxRgIDFk8oj8uM3WSZtYDG+R0XQCXjcaj4yXpOMr5CLi0BxT79p8PEbgSI2vr/N+KQ8azE/2xZ9HGzr5Md65qIvMk+erSCfGWvKEgjrmfA3Q8UqrmN1JfL/JceDjZqF/y/5Tp5dAC+fPf2IXDg/y8W1zzg5+EQgxpXy4NHYaxkPG99R+RS2RH5N5qMJ+JXxA+Nt8nm5F3o8Ayq2qZSI7W4V2TMlY2nmyhgBsFrvlGeIzRq8pQCEw+pFbE3/qIRwyGb4DhW81Cedj/Gr1ZwlMjIyMroTZLWXVfxU0gnpH/UU/gONHVCMY3hnPgAAAABJRU5ErkJggg==>