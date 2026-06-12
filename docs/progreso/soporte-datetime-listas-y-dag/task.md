# Tareas de Implementación: Tipos de Fecha/Hora, Listas Planas y Primitivas DAG

## 1. Configuración de dependencias (Cargo)
- [x] Modificar `Cargo.toml` raíz para añadir la feature `"chrono"` a `pyo3`.
- [x] Modificar `vantadb-python/Cargo.toml` para añadir la feature `"chrono"` a `pyo3`.

## 2. Tipos de datos en el Core Engine
- [x] Modificar `src/node.rs` para agregar `DateTime` y variantes de listas planas homogéneas a `FieldValue`.
- [x] Implementar `to_cardinality_keys(&self)` en `FieldValue` para manejo de cardinalidades complejas y listas.
- [x] Modificar `src/storage.rs` para adaptar el cálculo de cardinalidad (`insert`, `delete`, `initialize_cardinality_stats`, `get_estimated_selectivity`) usando `to_cardinality_keys()`.

## 3. Primitivas de Grafo (DAG) en Core Engine
- [x] Modificar `src/graph.rs` para implementar `dfs_traverse`, `topological_sort` (con detección de ciclos DFS coloreados) y `is_dag`.

## 4. SDK de Rust e Indexación
- [x] Modificar `src/sdk.rs` para reflejar las nuevas variantes en `VantaValue`.
- [x] Implementar conversiones `From` bidireccionales entre `VantaValue` y `FieldValue`.
- [x] Adaptar `encoded_scalar_value` para soportar `DateTime` formateado a RFC 3339.
- [x] Modificar `derived_put_ops` and `derived_delete_ops` para indexar individualmente los elementos de listas.
- [x] Exponer los métodos de grafos (`graph_bfs`, `graph_dfs`, `graph_topological_sort`, `graph_is_dag`) en la estructura `VantaEmbedded`.

## 5. Bindings de Python (PyO3)
- [x] Modificar `vantadb-python/src/lib.rs`:
  - [x] Adaptar `py_any_to_value` para aceptar objetos `datetime` de Python y listas homogéneas.
  - [x] Adaptar `set_python_value` para retornar objetos nativos de Python correspondientes a fecha/hora y listas.
  - [x] Exponer métodos de grafo en la clase `VantaEmbedded` de Python.

## 6. Verificación y Commits
- [x] Solicitar al usuario que compile y ejecute las pruebas en Rust (`cargo test`).
- [x] Solicitar al usuario que ejecute las pruebas en Python (`pytest`).
- [x] Crear script temporal de verificación del FFI y primitivas.
- [x] Crear los commits Git correspondientes a la implementación.
