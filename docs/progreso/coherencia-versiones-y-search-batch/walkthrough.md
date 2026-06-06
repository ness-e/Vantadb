# Walkthrough: Estabilización de Versiones (T0.3) y Búsqueda por Lotes en Python SDK (T1.4)

Se han completado e integrado con éxito las tareas **T0.3** y **T1.4** en la extensión de Python y el core de VantaDB.

## Cambios Realizados

### Componente: QA / Guardrails de Versión (T0.3)
- **Modificación:** Se expandió el test de integración centralizado [version_coherence.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/version_coherence.rs) para validar los manifiestos de:
  - `vantadb-server/Cargo.toml`
  - `vantadb-mcp/Cargo.toml`
  - `packages/langchain-vantadb/pyproject.toml`
  - `packages/llamaindex-vantadb/pyproject.toml`
- **Garantía:** Se validó no solo que la versión del proyecto coincida con la del Cargo.toml raíz (`0.1.4`), sino también que las integraciones requieran explícitamente `vantadb-py>=0.1.4`, previniendo problemas de drift de versiones en el CI.

### Componente: Python SDK / Búsqueda por Lotes (T1.4)
- **Dependencia:** Se añadió la crate `rayon = "1.12"` al archivo [vantadb-python/Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-python/Cargo.toml).
- **Binding de Python:** Se implementó `search_batch` en [vantadb-python/src/lib.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-python/src/lib.rs):
  - El método libera de forma eager el GIL (`py.allow_threads`).
  - Utiliza paralelismo a nivel de CPU mediante el iterador paralelo `into_par_iter()` de Rayon, procesando el lote de vectores concurrentemente.
- **Suite de Pruebas:** Se añadió un test unitario exhaustivo `test_search_batch` en [test_sdk.py](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-python/tests/test_sdk.py) que asegura que la búsqueda paralela devuelva el mismo orden e idénticas puntuaciones que búsquedas individuales secuenciales.

---

## Verificación y Resultados de Pruebas

### 1. Test de Coherencia de Versiones
El usuario ejecutó de forma manual el comando de validación en Rust:
```powershell
cargo test --test version_coherence
```
**Resultado:**
```text
running 1 test
test public_surfaces_report_same_version ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 2. Tests de Python SDK (incluyendo search_batch)
El usuario activó el entorno virtual y ejecutó la compilación local editable y la batería de tests:
```powershell
maturin develop
pytest tests/test_sdk.py -v
```
**Resultado:**
```text
tests/test_sdk.py::TestVectorSearch::test_search_batch PASSED                                                                                                                                                             [ 50%]
================================================ 18 passed in 4.93s ================================================
```

Ambas tareas están completamente integradas, validadas y sin deuda técnica pendiente.
