# Plan de Implementación: Tipos de Fecha/Hora, Listas Planas y Primitivas DAG

Este plan detalla el diseño y la implementación de tres características técnicas clave en VantaDB:
1. **Soporte Nativo para Tipos de Fecha/Hora (Date/Time Types)** basados en la biblioteca `chrono`.
2. **Soporte Nativo para Listas Planas (Flat Array/List Types)** para almacenar colecciones homogéneas en las propiedades relacionales.
3. **Primitivas de Ejecución de Grafo (DAG)** que incluyen travesías BFS/DFS, detección de ciclos y ordenamiento topológico expuestos en el SDK y bindings de Python.

---

## User Review Required

> [!IMPORTANT]
> **Integración FFI y Cambio de Versión de PyO3:**
> Para permitir la conversión nativa automática de tipos `DateTime` entre Rust y Python, agregaremos la feature `"chrono"` a la dependencia `pyo3` en los archivos `Cargo.toml`. Esto requiere que el compilador tenga acceso a las cabeceras adecuadas.

> [!IMPORTANT]
> **Políticas de Ejecución de Comandos:**
> Siguiendo las reglas del usuario, **no se ejecutarán comandos de compilación o prueba de forma automática**. Al finalizar las implementaciones, se le indicará al usuario exactamente qué comandos ejecutar (p. ej., `cargo test` o `python -m pytest`) para que verifique los resultados.

---

## Proposed Changes

### Core Engine

#### [MODIFY] [Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/Cargo.toml)
* Modificar la dependencia de `pyo3` para habilitar la feature `"chrono"`:
  ```toml
  pyo3 = { version = "0.24.1", features = ["extension-module", "chrono"], optional = true }
  ```

#### [MODIFY] [node.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/node.rs)
* Modificar el enum `FieldValue` para añadir soporte a `DateTime` y listas planas homogéneas:
  ```rust
  pub enum FieldValue {
      String(String),
      Int(i64),
      Float(f64),
      Bool(bool),
      DateTime(chrono::DateTime<chrono::Utc>),
      ListString(Vec<String>),
      ListInt(Vec<i64>),
      ListFloat(Vec<f64>),
      ListBool(Vec<bool>),
      ListDateTime(Vec<chrono::DateTime<chrono::Utc>>),
      Null,
  }
  ```
* Implementar una función auxiliar `to_cardinality_keys(&self) -> Vec<String>` en `FieldValue` para simplificar la indexación y des-indexación de estadísticas de cardinalidad en el motor de almacenamiento.

#### [MODIFY] [storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs)
* Actualizar las funciones `insert`, `delete`, `initialize_cardinality_stats` y `get_estimated_selectivity` para procesar múltiples claves usando `value.to_cardinality_keys()` en lugar de asumir un mapeo `1:1` escalar.

#### [MODIFY] [graph.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/graph.rs)
* Implementar `dfs_traverse` para travesías DFS de profundidad limitada.
* Implementar `topological_sort` mediante un algoritmo DFS con coloreado para detectar ciclos.
* Implementar `is_dag` que devuelve `true` si no hay ciclos.

#### [MODIFY] [sdk.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/sdk.rs)
* Modificar el enum `VantaValue` para reflejar las nuevas variantes añadidas a `FieldValue`.
* Implementar las conversiones `From<VantaValue> for FieldValue` y `From<FieldValue> for VantaValue`.
* Actualizar `encoded_scalar_value` para soportar `VantaValue::DateTime` con formato ordenable (RFC 3339 con microsegundos: `d:<rfc3339>`).
* Actualizar `derived_put_ops` y `derived_delete_ops` para iterar e indexar individualmente cada elemento cuando el valor del campo relacional sea una variante de tipo lista (p. ej., `ListString`), logrando que las búsquedas indexadas de contención funcionen de forma nativa.
* Añadir métodos en `VantaEmbedded` para exponer las primitivas de grafo:
  * `graph_bfs`
  * `graph_dfs`
  * `graph_topological_sort`
  * `graph_is_dag`

---

### Python SDK Bindings

#### [MODIFY] [Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-python/Cargo.toml)
* Modificar la dependencia de `pyo3` para habilitar la feature `"chrono"`:
  ```toml
  pyo3 = { version = "0.24.1", features = ["extension-module", "abi3-py38", "chrono"] }
  ```

#### [MODIFY] [lib.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-python/src/lib.rs)
* Actualizar `py_any_to_value` para soportar:
  * Objetos `datetime.datetime` de Python, convirtiéndolos a `VantaValue::DateTime`.
  * Listas de Python (`PyList`), infiriendo el tipo del primer elemento y mapeándolos a la variante `ListX` correspondiente.
* Actualizar `set_python_value` para inyectar objetos nativos de Python (`datetime`, listas de enteros, flotantes, booleanos, cadenas) en el diccionario de retorno.
* Exponer los métodos de grafo en la clase `VantaEmbedded` de Python.

---

## Verification Plan

### Automated Tests
* El usuario ejecutará las pruebas unitarias e integrales en Rust una vez aplicados los cambios:
  ```bash
  cargo test
  ```
* El usuario ejecutará los tests del SDK de Python:
  ```bash
  pytest vantadb-python/tests/
  ```

### Manual Verification
* Crear un script temporal de prueba en Python para verificar:
  1. Que se puedan guardar y recuperar campos relacionales con `datetime.datetime` actual y listas (como `["vanta", "db"]`).
  2. Que la búsqueda mediante filtros relacionales sobre elementos de listas use el índice correctamente.
  3. Que la ejecución de travesías y ordenamiento topológico devuelva resultados correctos e identifique ciclos si se añaden aristas cíclicas.
