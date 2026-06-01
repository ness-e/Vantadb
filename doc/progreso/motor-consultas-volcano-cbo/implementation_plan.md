# Plan de Implementación: Fase PLANNER-02 (Query Engine: AST, Plan Físico Volcano y Optimizador por Selectividad)

Este plan detalla el diseño e implementación del nuevo motor de ejecución de consultas y optimización de VantaDB. Se introduce un motor físico Volcano-style basado en iteradores dinámicos (`open`, `next`, `close`), un AST estructurado enriquecido para consultas complejas, y un Optimizador por Selectividad (CBO simple) que analiza estadísticas de almacenamiento para decidir el orden de escaneo y filtrado más eficiente entre HNSW, BM25 y metadatos relacionales.

---

## User Review Required

> [!IMPORTANT]
> **Transición de Ejecución Lineal a Volcano Iterators:** El executor actual (`src/executor.rs`) recorre lineal y secuencialmente un vector simple de operadores. Migraremos a un **modelo físico de tipo Volcano**, lo que permitirá componer iteradores perezosos (lazy evaluation) zero-copy y flujos de datos eficientes en memoria.
> 
> **Optimizador por Selectividad de Índices:** Se implementará lógica dinamicamente evaluada en el planificador para decidir la estrategia de búsqueda híbrida. Si un filtro relacional es altamente selectivo, se aplicará primero el filtro y luego similitud vectorial por fuerza bruta (refinement). Si no es selectivo, se consultará HNSW primero y se aplicará post-filtrado.

---

## Open Questions

> [!WARNING]
> **Medición de Selectividad Estática vs Dinámica:** Para calcular la selectividad de metadatos, requerimos estadísticas aproximadas. Proponemos añadir un contador básico de cardinalidad por campo en `StorageEngine` (ej. histograma ligero de llaves/valores) para no penalizar el rendimiento de escritura. ¿Estás de acuerdo con este enfoque simplificado de recuento de cardinalidad?

---

## Proposed Changes

### [Component: Query Engine & AST]

#### [MODIFY] [src/query.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/query.rs)
*   Enriquecer `Condition` y `LogicalOperator` para soportar combinaciones complejas de proyección, ordenamiento y límites en subconsultas.
*   Introducir el contrato del **Plan Físico Volcano**:
    ```rust
    pub trait PhysicalOperator: Send + Sync {
        fn open(&mut self) -> crate::error::Result<()>;
        fn next(&mut self) -> crate::error::Result<Option<crate::node::UnifiedNode>>;
        fn close(&mut self) -> crate::error::Result<()>;
    }
    ```

#### [NEW] [src/planner/physical_plan.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/planner/physical_plan.rs)
*   Implementar las estructuras concretas que conforman los operadores del stream Volcano:
    *   `PhysicalScan`: Escaneo secuencial o de índice relacional primario sobre particiones de almacenamiento.
    *   `PhysicalFilter`: Operador que evalúa condiciones booleanas/relacionales sobre el flujo de nodos procedentes de su operador hijo.
    *   `PhysicalVectorSearch`: Consulta aproximada en el índice HNSW para extraer los vecinos más cercanos.
    *   `PhysicalProject`: Limita los campos expuestos en el resultado final (zero-copy).
    *   `PhysicalLimit` / `PhysicalSort`: Operadores de agregación y corte de flujo.

---

### [Component: Cost-Based Optimizer (CBO)]

#### [MODIFY] [src/storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs)
*   Añadir el método `get_estimated_selectivity(field: &str, op: &RelOp, value: &FieldValue) -> f32` que retorne un float aproximado entre `0.0` y `1.0` sobre la porción de datos que cumplen la condición.
*   Integrar estadísticas básicas basadas en el tamaño de las particiones activas en RocksDB/Fjall.

#### [MODIFY] [src/planner.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/planner.rs)
*   Implementar la lógica del optimizador por selectividad:
    *   Analizar las condiciones relacionales de la consulta.
    *   Si la selectividad estimada de los metadatos es `< 0.1` (alta selectividad, ej. filtro por un ID único o rol muy restringido), el plan físico priorizará:
        `PhysicalFilter` (hijo: `PhysicalScan`) → `Vector Similarity Check (Brute Force Refinement)`.
    *   Si la selectividad es `> 0.1` o no hay filtros relacionales, se prioriza:
        `Post-Filter` (hijo: `PhysicalVectorSearch` vía HNSW).

---

### [Component: Query Executor]

#### [MODIFY] [src/executor.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/executor.rs)
*   Refactorizar el método `execute_hybrid` para:
    1.  Traducir el `LogicalPlan` a un `PhysicalPlan` optimizado a través del CBO.
    2.  Llamar a `.open()` en el operador físico raíz.
    3.  Iterar dinámicamente llamando a `.next()` en bucle para recolectar, formatear y retornar el conjunto de resultados, asegurando una huella de memoria constante (pipeline processing).
    4.  Llamar a `.close()` para liberar descriptores de recursos.

---

## Verification Plan

### Automated Tests (Solicitado al Usuario)

Para verificar la excelencia del nuevo motor de consultas y su correcto comportamiento ante diferentes selectividades, te solicitaré ejecutar los siguientes tests:

1.  **Ejecución de Pruebas Unitarias del Planner y AST:**
    `cargo test --lib planner`
    *Verifica la traducción y optimización lógica del CBO.*

2.  **Pruebas de Integración de Ejecución de Consultas:**
    `cargo test --test executor`
    *Garantiza que el executor Volcano devuelva los mismos resultados funcionales con un consumo de recursos altamente optimizado.*

3.  **Benchmarks Comparativos de Selectividad:**
    `cargo bench --bench hybrid_queries`
    *Demostrará la reducción de page faults y latencias p99 al alternar inteligentemente entre escaneos relacionales y búsquedas HNSW.*
