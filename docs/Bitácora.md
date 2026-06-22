### 2. Riesgos Estructurales y Deuda Técnica Detectada

Bajo un escrutinio estricto de escalabilidad y rendimiento, existen puntos de fricción que requieren atención antes de consolidar la versión `v0.2.0`:

- **Cuello de Botella en la Ingesta (Single-Writer):** Aunque el SDK Python expone un `put_batch` paralelizado vía Rayon, la construcción del grafo HNSW subyacente sigue presentando bloqueos finos (a pesar de la transición de `RwLock` a `DashMap` y `ArcSwap`). En cargas de alta densidad de inserción continua, esto se convierte en el principal factor limitante frente a bases de datos vectoriales servidoras.
    
- **Tokenización BM25 Primitiva por Defecto:** El indexador de texto base (`lowercase-ascii-alnum`) es insuficiente para búsquedas de producción multilingües. Aunque existe la feature `advanced-tokenizer` basada en Tantivy (esquema v4), mantener el tokenizador simple por defecto pospone problemas de _stemming_, _stopwords_ y _Unicode folding_ para los usuarios finales.
    
- **Gestión de Concurrencia de Procesos:** El motor depende de un archivo de bloqueo exclusivo (`.vanta.lock`). Aunque existen pruebas de resiliencia (`file_locking_stress.rs`) ante _stale locks_, en entornos donde agentes concurrentes intenten instanciar el motor desde múltiples procesos Python de manera independiente, se producirán fallos de contención dura (`DatabaseBusy`).
    
- **Inexistencia de Soporte de Recuperación Point-in-Time (PITR):** Fjall no soporta _checkpoints_ nativos como RocksDB. La política de backup actual depende de copias de seguridad lógicas (JSONL) o copias en frío (Cold Copy), lo que añade latencia y complejidad operativa si el volumen de datos de memoria del agente crece significativamente.
  
  ### Direccionalidad Estratégica (Hacia v0.2.0)

El proyecto se encuentra en una etapa óptima para la transición del desarrollo core a la estabilización de distribución. Las prioridades de ejecución se deben alinear así:

1. **Promoción de Search Quality v2:** Transicionar el `advanced-tokenizer` (Tantivy) como la opción por defecto en las compilaciones del release para asegurar paridad semántica en la recuperación léxica. Exponer las capacidades de _snippets_ y _highlighting_ en la API pública de Python.
    
2. **Programa Piloto en Entornos Reales:** Congelar la adición de nuevas características arquitectónicas (LISP/IQL experimental) y centrar la telemetría en el comportamiento real del _heap memory drift_ bajo agentes de IA en la Fase 3.4.
    
3. **Endurecimiento del Pipeline CI/CD:** Completar la certificación SLSA Nivel 2 mediante GitHub Attestations y ejecutar la transición del flujo de TestPyPI hacia el registro de producción de PyPI para habilitar la adopción fricción-cero en la comunidad local-first.