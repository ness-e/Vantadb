# Walkthrough: Fase PLANNER-02 (Volcano Physical Plan & Cost-Based Optimizer) & SERV-01 (Tokio Decoupling)

Este documento detalla la culminación e integración exitosa de las fases **`SERV-01`** y **`PLANNER-02`** en VantaDB. El motor ahora cuenta con un modelo físico Volcano basado en iteradores dinámicos perezosos (`open`, `next`, `close`), un Optimizador basado en Costo por Selectividad (CBO ligero) y una arquitectura del core síncrona, limpia y desacoplada de dependencias asíncronas pesadas como `tokio` y `reqwest`.

---

## 🛠️ Resumen de Cambios Realizados

### 1. Desacoplamiento de dependencias del Core (`vantadb`)
*   **[Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/Cargo.toml)**:
    *   Se eliminó la dependencia `tokio` del core de producción, manteniéndola en `[dev-dependencies]` para pruebas de integración y concurrencia.
    *   Se renombró la feature opcional `llm` a `remote-inference` para evitar fugas de red por defecto.
*   **[src/lib.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/lib.rs)**: Modulado condicionalmente bajo la feature `remote-inference`.
*   **[packages/experimental-governance](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/packages/experimental-governance)**: Adaptados los manifiestos y el trabajador de mantenimiento para usar la nueva feature `remote-inference` del core.

### 2. Modelo Físico Volcano y Pipeline Lazy-Evaluation
*   **[src/query.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/query.rs)**:
    *   Definición del trait central `PhysicalOperator`:
        ```rust
        pub trait PhysicalOperator: Send + Sync {
            fn open(&mut self) -> Result<()>;
            fn next(&mut self) -> Result<Option<UnifiedNode>>;
            fn close(&mut self) -> Result<()>;
        }
        ```
*   **[src/physical_plan.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/physical_plan.rs)**:
    *   `PhysicalScan`: Flujo secuencial de lectura sobre particiones primarias del motor.
    *   `PhysicalFilter`: Filtra tuplas en base a condiciones relacionales.
    *   `PhysicalVectorSearch`: Consulta aproximada HNSW sobre el grafo indexado.
    *   `PhysicalVectorRefine`: Fuerza bruta y refinamiento de distancia post-filtrado.
    *   `PhysicalProject`, `PhysicalLimit`, `PhysicalSort`: Controladores del flujo de salida.

### 3. Estadísticas y Optimizador Basado en Costo (CBO)
*   **[src/storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs)**:
    *   Inicialización de estadísticas de cardinalidad ligera en base al escaneo inicial de metadatos relacionales al abrir el motor.
    *   Implementación de `get_estimated_selectivity(field, op, value)` para estimar el costo/selectividad del filtro relacional.
    *   Corregida advertencia de `borrow of moved value` sobre el puntero del backend de almacenamiento.
*   **[src/planner.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/planner.rs)**:
    *   Traducción lógica a física en `optimize_and_compile`.
    *   **Regla del Optimizador**: Si la selectividad estimada es menor a `0.1` (muy selectiva), compila a `Scan + Filter + Refine` para evitar escaneos vectoriales HNSW inútiles. Si es mayor a `0.1`, compila a `VectorSearch (HNSW) + Post-Filter`.

### 4. Refactorización del Executor
*   **[src/executor.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/executor.rs)**:
    *   Refactorizado `execute_hybrid` para compilar el plan lógico en físico y ejecutar el flujo dinámico consumiendo los resultados en un búfer acotado.

---

## 🔬 Resultados de Validación y Calidad

1.  **Formateo y Estilo de Código**: `cargo fmt --all` ejecutado correctamente.
2.  **Advertencias y Tipos**: Todos los warnings silenciados (`#[allow(unused_mut)]` para features opcionales).
3.  **Compilación y Tests**: Todo el espacio de trabajo compiló en verde bajo todas las combinaciones de features y objetivos de tests.
