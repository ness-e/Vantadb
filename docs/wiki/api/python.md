---
title: "API de Python (`vantadb-py`)"
kind: reference
status: active
description: "Binding PyO3 publicado en PyPI: Client, sub-clientes por dominio, AsyncClient, superficie verificada y diferencias frente a otros transportes."
---

# API de Python (`vantadb-py`)

Audiencia: usuarios del SDK Python. Referencia canónica exhaustiva: [PYTHON_SDK](../../api/PYTHON_SDK.md); superficie normativa por método: [BINDINGS_NAMESPACES §Python](../../api/BINDINGS_NAMESPACES.md) (métodos `pyclass` + `connect()` de módulo).

## Distribución e import

`[HECHO]` El paquete PyPI es `vantadb-py` (maturin; `dynamic = ["version"]` resuelta desde el workspace; `requires-python = ">=3.11"`; módulo `vantadb_py`; [`vantadb-python/pyproject.toml`](../../../vantadb-python/pyproject.toml)); publicado a 0.9.0 ([metadata PyPI](https://pypi.org/pypi/vantadb-py/json) verificada 2026-11-06). El import canónico es `import vantadb`; `import vantadb_py` funciona pero emite `DeprecationWarning` (README raíz §Installation). El crate fuente es `vantadb-python/src/{lib,convert,types,vector}.rs`.

## Cliente y sub-clientes

`[HECHO]` `Client` es el handle principal; los métodos de dominio también están agrupados en sub-clientes `db.memory` (`get`/`list`/`delete`/`search`), `db.graph`, `db.system`, `db.wiki` — pyclass delegates que reenvían a `Client` (BINDINGS_NAMESPACES §Sub-Client Design Python). `[HECHO]` AST-012 eliminó los métodos planos `get_memory`/`list_memory`/`delete_memory` sin alias; ops de nodo por id son `insert_node`/`get_node`/`delete_node` (`id: int` = u128 exacto).

```python
import vantadb

db = vantadb.Client("./agent_memory")
db.put(namespace="agent/session-1", key="ctx-001",
       payload="User prefers concise answers",
       metadata={"type": "preference"}, vector=[0.8, 0.1, 0.5])
results = db.search("agent/session-1", query_vector=[0.75, 0.2, 0.6],
                    text_query="concise", top_k=3)
db.memory.get("agent/session-1", "ctx-001")
db.close()
```

Ejemplo ilustrativo **no ejecutado**; las firmas se contrastaron contra la matriz W1/API-02 de BINDINGS_NAMESPACES.

## Capacidades exclusivas de Python

`[HECHO]` Solo en este binding (BINDINGS_NAMESPACES §SDK Surface Differences): `put_batch_raw` (NumPy float32 2D zero-copy), `search_batch`, `search_batch_requests`, `hardware_profile`, `graph_page_rank`, `graph_degree_centrality`, y la capa cognitiva mínima `memory_capture`/`memory_recall` (sync + async, DIST-02). `[HECHO]` No expuestos en Python: `remove_edge`, `sparse_vector` en `put`/`put_batch`, `audit_text_index_deep`, `export_namespace_filtered`, `import_records`, `graph_filtered_traversal` como tal (existe `graph_bfs_filtered`).

`[HECHO]` `AsyncClient`/`AsyncVantaDB` envuelve el engine síncrono en threads con el GIL liberado (`py.detach`); el `drain` de `OpGate` debe llamarse con GIL liberado (`vantadb-ffi-core/src/lib.rs:52-58`).

## Semántica compartida

`[HECHO]` `hit.score` mayor-es-mejor en híbrida; `search_vector` devuelve tuplas `(node_id, distance)` con distancia menor-es-mejor (BINDINGS_NAMESPACES §Score vs Distance). `[HECHO]` Los campos v2 (`valid_at_ms`, `confidence_class`, `quarantined_at_ms`, `as_of_ms`, `min_confidence`, …) cruzan en `search`/`list` como kwargs snake_case (matriz SCH-07). `[HECHO]` La señal de abstinencia (`abstained`/`abstention_reason`) NO viaja en bindings: solo en transportes con página (HTTP/MCP) — declarado, no bug.

## Tests del binding

`[HECHO]` BINDINGS_NAMESPACES cita como evidencia `tests/test_w1_surface.py`, `test_graphrag.py`, `test_sch07.py`, `test_cross_language_parity.py` dentro de `vantadb-python/`; el runner de paridad cross-language emite `target/bindings-parity/python.json` comparado por `dev-tools/parity-compare.mjs`.

## Fuentes

`vantadb-python/` · `docs/api/BINDINGS_NAMESPACES.md` · [PYTHON_SDK](../../api/PYTHON_SDK.md) · [EMBEDDINGS](../../api/EMBEDDINGS.md) · [VANTA_MEMORY](../../api/VANTA_MEMORY.md) · [paridad](parity.md)
