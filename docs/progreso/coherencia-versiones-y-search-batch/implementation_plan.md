# Plan de Implementación: Estabilización de Versiones (T0.3) y Búsqueda por Lotes en Python SDK (T1.4)

Este plan detalla los cambios necesarios para robustecer el control de versiones en todo el espacio de trabajo y agregar el entry point de búsqueda por lotes (`search_batch()`) en el SDK de Python con paralelismo a nivel de CPU (Rayon) y liberación del GIL.

## User Review Required

> [!IMPORTANT]
> - De acuerdo a las políticas de ejecución, el agente **no** compilará ni ejecutará tests directamente en el entorno de desarrollo del usuario. Se proporcionarán las instrucciones exactas de consola para que el usuario ejecute de forma manual e informe los resultados.
> - Se requiere que el entorno local de Python tenga instalado `maturin` y `pytest` para la compilación y prueba de la extensión nativa Python.

## Proposed Changes

---

### Componente: QA / Guardrails de Versión (T0.3)

#### [MODIFY] [version_coherence.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/version_coherence.rs)
- Se ampliará la cobertura del suite de prueba `public_surfaces_report_same_version` para verificar:
  - `vantadb-server/Cargo.toml`
  - `packages/langchain-vantadb/pyproject.toml` (versión del paquete y versión requerida de `vantadb-py`).
  - `packages/llamaindex-vantadb/pyproject.toml` (versión del paquete y versión requerida de `vantadb-py`).

---

### Componente: Python SDK / Búsqueda por Lotes (T1.4)

#### [MODIFY] [Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-python/Cargo.toml)
- Añadir `rayon = "1.12"` a las dependencias del binding.

#### [MODIFY] [lib.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-python/src/lib.rs)
- Implementar el método `search_batch` expuesto a Python dentro del bloque `#[pymethods]` de `VantaDB`.
- Lógica de implementación:
  - Liberar el GIL usando `py.allow_threads`.
  - Usar un iterador paralelo (`into_par_iter()`) de Rayon sobre el lote de vectores de entrada.
  - Ejecutar `engine.search_vector(&vector, top_k)` en paralelo para cada vector.
  - Mapear y recolectar los hits convirtiendo las estructuras internas a tipos Rust primitivos, y devolver `Vec<Vec<(u64, f32)>>`.

#### [MODIFY] [test_sdk.py](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-python/tests/test_sdk.py)
- Añadir el caso de prueba `test_search_batch` en la clase `TestVectorSearch`:
  - Insertar un conjunto controlado de vectores.
  - Llamar a `db.search_batch()` con múltiples vectores de consulta.
  - Validar que el formato de salida sea una lista de listas de tuplas `(node_id, distance)` y que los resultados coincidan con consultas individuales secuenciales.

## Verification Plan

### Pruebas Automatizadas (Propuestas para Ejecución Manual)

El usuario deberá ejecutar los siguientes comandos en la raíz del proyecto para validar los cambios:

1. **Compilar y ejecutar test de guardrail de versiones en Rust**:
   ```powershell
   cargo test --test version_coherence
   ```

2. **Compilar y reinstalar localmente el SDK de Python**:
   ```powershell
   cd vantadb-python
   # Activar entorno virtual si corresponde (ej. .venv\Scripts\Activate.ps1)
   maturin develop
   ```

3. **Ejecutar suite de tests del SDK de Python (incluyendo el nuevo test de batch)**:
   ```powershell
   pytest tests/test_sdk.py -v
   ```
