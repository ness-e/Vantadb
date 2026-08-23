# Review de Módulo — `vantadb-python/`

| Campo | Valor |
|---|---|
| **Módulo** | `vantadb-python/` — binding PyO3 (maturin) del core VantaDB |
| **Alcance** | `src/lib.rs`, `src/types.rs`, `src/convert.rs`, `src/vector.rs`, `vantadb_py/__init__.py`, stubs `.pyi`, `tests/`, `pyproject.toml`, `Cargo.toml` |
| **Revisor** | ox-alpha (segunda opinión, contexto fresco — no participó en la implementación) |
| **Fecha** | 2026-08-22 |
| **Veredicto** | 🔴 **Cambios requeridos** |
| **Score** | **6.5 / 10** |

---

## 1. Resumen Ejecutivo

El binding PyO3 está bien construido en su núcleo: el fix SEC-01 (UAF por
`__array_interface__` con puntero crudo) está correctamente verificado en ambos
sites (`vector.rs:70-83`, `types.rs:398-416`) y **no encontré otros paths que
alien memoria con NumPy**. Los docstrings Rust son ejemplares (Google style,
con ejemplos ejecutables). El gate de durabilidad `OpGate` es un patrón sano.

Sin embargo, la revisión con contexto fresco detectó:

1. **🔴 La suite completa por defecto (`pytest`) está rota**: 66 tests fallan
   cuando se corren todos los archivos juntos, aunque `test_sdk.py` aislado
   pasa 70/70. El auto-reporte "pytest 70 passed" corresponde solo a un archivo.
2. **🔴 Riesgo de deadlock en `close()` concurrente**: `OpGate::drain()` espera
   el condvar **sosteniendo el GIL**, mientras las operaciones in-flight lo
   necesitan de vuelta tras `py.detach`.
3. **🟡 Stubs `.pyi` significativamente desactualizados** (dos copias que
   derivan por separado) y wrapper async con parámetros perdidos.

## 2. Verificación de Contrato

| Verificación | Comando | Resultado |
|---|---|---|
| Suite SDK (claim del implementador) | `.\.venv\Scripts\python.exe -m pytest -q tests/test_sdk.py` | ✅ **70 passed** en 143s |
| test_sdk + subclients | `... -m pytest -q tests/test_sdk.py tests/test_subclients.py` | ✅ 88 passed |
| + perf | `... + tests/test_perf_15_16.py` | ✅ 97 passed |
| **Gate completo por defecto** (`addopts = "-m 'not slow'"`, pyproject.toml:49) | `... -m pytest -q` | ❌ **66 failed, 43 passed, 4 deselected** |

Error dominante en el run completo:
```
RuntimeError: Resource limit exceeded: Memory pressure: 180375552 bytes used
(134% of 134217728 limit, threshold 80%)
```
Causa raíz: los archivos `test_async_smoke.py` / `test_perf_15_16.py` abren DBs
con `memory_limit_bytes=128MB`; como muchos tests anteriores **no cierran sus
DBs** (sin teardown), el RSS del proceso acumulado dispara el guard de presión
de memoria del core para cualquier test posterior. La suite es
dependiente-del-orden y el gate por defecto (`pytest`) es rojo. El claim
"70 passed" es verdadero pero parcial — el contrato real del módulo
(`pytest` según pyproject) no pasa.

## 3. Tabla de Hallazgos

| # | Severidad | Hallazgo | Evidencia |
|---|---|---|---|
| H1 | 🔴 | Suite completa por defecto falla (66 failed) por interferencia entre archivos: RSS acumulado de DBs sin cerrar + guard de memoria. Gate CI rojo u orden-dependiente. | Run `pytest -q` completo (ver §2); `tests/test_async_smoke.py:35` fija `memory_limit_bytes=128MB`; tests sin fixture teardown que cierre DBs |
| H2 | 🔴 | Deadlock potencial: `close()` llama `OpGate::drain()` **con GIL tomado**; un op in-flight que terminó su `py.detach` necesita re-adquirir el GIL para retornar → bloqueo mutuo. Escenario realista con `AsyncVantaDB` (`asyncio.to_thread`). | `lib.rs:1711-1717` (`drain()` antes del `py.detach`), `lib.rs:132-139` (`cvar.wait` sin liberar GIL), `lib.rs:880` (patrón detach→re-adquiere GIL). Sin test de close concurrente |
| H3 | 🟡 | Stubs duplicados y desactualizados: `put_batch`/`put_batch_raw` declaran `-> list[dict]` pero retornan `list[VantaMemoryRecord]`; faltan `exclude_superseded` (search/list), `created_at_ms` en `add_edge`, `superseded_by/at_ms` en record, `search_batch_requests`, `bulk_import*`, `reindex_hnsw_from_text`, `supersede` en varios stubs | `vantadb_py/__init__.pyi:56,65,80,88,111-117,163-302` vs `src/lib.rs:1079,2104` |
| H4 | 🟡 | `AsyncVantaDB.graph_bfs/graph_dfs` pierden el parámetro `direction` ("Forward"/"Reverse"/"Both") que el sync soporta — API async menos expresiva sin razón documentada | `vantadb/__init__.py:332-340` vs `lib.rs:1735,1757` |
| H5 | 🟡 | Sin jerarquía de excepciones propia: todo mapea a builtins genéricos; el catch-all es `RuntimeError`. Un usuario no puede distinguir error-de-engine de bug interno sin parsear strings | `src/convert.rs:659-684` (`map_vanta_error`) |
| H6 | 🟡 | Política de validación inconsistente: `backend` desconocido → `ValueError` (AUD-037), pero `distance_metric` y `method` desconocidos → warning silencioso + fallback a default. Mismo tipo de error de usuario, dos comportamientos | `lib.rs:168-177` vs `lib.rs:1095-1105,2115-2130` |
| H7 | 🟡 | `search_batch_requests` ignora `exclude_superseded`: `parse_search_request` lo hardcodea a `false` (`lib.rs:2104`), mientras `search_memory` sí lo expone. Resultados inconsistentes entre batch y single | `lib.rs:2094-2106` vs `lib.rs:1079` |
| H8 | 🟡 | `query()` IQL retorna string formateado, no datos estructurados — pérdida de información para consumo programático (el core retorna `VantaQueryResult` tipado) | `lib.rs:1578-1589`, `convert.rs:303-334` |
| H9 | 🟡 | ~30% de la API core no expuesta (ver §5): `count`, `delete_by_filter`, `similar_to_key`, `namespace_stats`, `versions/get_version`, `remove_edge`, `vacuum`, `search_multi/search_all`, graph filtrado, snapshots, threads agentic | Comparación verificada contra `src/sdk/api.rs`, `graph.rs`, `builder.rs`, `search/multi.rs` |
| H10 | 🟡 | Metadatos no soportan `bytes` (`py_any_to_value` lo rechaza) ni ints > i64 (OverflowError); datetime pierde tzinfo (se normaliza a UTC sin documentarlo en docstring de `put`) | `convert.rs:36-147` |
| H11 | 🟡 | Tests: sin cobertura de edge cases críticos — close concurrente (H2), arrays F-order/no contiguos en `put_batch_raw` (camino fallback nunca ejercido), paginación cursor hasta `next_cursor=None`, roundtrip metadata con datetime/listas, mapeo de errores (Timeout/FileNotFound), ids u128 > 2^64 | `tests/test_sdk.py` (70 tests, revisión de nombres); `.coverage` presente pero % no medido en esta review |
| H12 | 🟢 | Artefactos stale en repo: wheels 0.1.5/0.4.0 en `dist/` vs versión actual 0.5.0; `.pyd`/`.pdb` commiteados dentro de `vantadb_py/`; `test_vanta_db/`, `test_path_dummy/` (fixtures runtime) en el árbol | `dist/*.whl`, `vantadb_py/*.pyd`, glob del módulo |
| H13 | 🟢 | `MAX_K` clampea `top_k` silenciosamente (1000) en vez de validar — un usuario pidiendo k=5000 recibe 1000 sin aviso | `lib.rs:43,1114,1453` |
| H14 | 🟢 | Cache LRU de metadatos por-repr (CODE-014): correcto para tipos soportados (repr determinista), pero añade complejidad para un win acotado — candidata a ponytail-review si el profiling no justifica | `convert.rs:23-34,585-640` |
| H15 | 🟢 | `connect()` no acepta `read_only`/`backend` mientras `VantaDB.__init__` sí — asimetría menor entre los dos entry points | `lib.rs:2139-2148` vs `lib.rs:366-382` |

### Fix SEC-01 — verificación puntual ✅

Ambos sites de `__array_interface__` entregan ahora un `PyBytes` *owned*
(copia little-endian f32); NumPy copia ese buffer al ndarray y nunca aliasa la
memoria del pyclass:
- `src/vector.rs:70-83` (`VantaVector.get_array_interface`)
- `src/types.rs:398-416` (`VantaSearchHit.get_search_hit_array_interface`)

Otros paths auditados que tocan buffers:
- `FlatBufferView` (`types.rs:18-40`): préstamo scoped dentro de una sola
  llamada, convertido a `Vec<f32>` por fila — seguro.
- `extract_vector` (`convert.rs:177-221`): `PyBuffer::to_vec`/slice copia — seguro.
- `try_numpy_array` (`convert.rs:153-167`): usa `numpy.array(vv)` → copia — seguro.
- `node_to_pydict` vector: clonado a lista Python — seguro.

No hay otros paths que expongan punteros crudos. **SEC-01/AUDIT-01 cerrados
correctamente**, con tests de regresión específicos (`test_asarray_*`,
`test_search_hit_array_interface_does_not_aliase_pyclass`).

## 4. Flujo de Uso Real (DX)

Recorrido `pip install vantadb-py` → `VantaDB("./data")` → put/get/list/search → close:

1. **Instalación**: wheel abi3-py311 único cubre 3.11+ ✅; `requires-python>=3.11`
   consistente; `py.typed` incluido ✅. Riesgo: solo hay wheels win_amd64
   locales; la matriz multi-plataforma de CI no fue verificada en esta review.
2. **Apertura**: docstring excelente; `":memory:"` y `""` documentados;
   backend inválido da `ValueError` claro ✅.
3. **put/get**: coherentes; `get_memory` retorna objeto tipado con getters +
   `__getitem__` dual ✅. Pero `get(id)` (grafo) retorna **dict plano** —
   dos convenciones de retorno distintas bajo el mismo handle (objetos memory,
   dicts graph). Confuso pero documentado parcialmente en BINDINGS_NAMESPACES.md.
4. **list/search**: paginación por cursor bien diseñada. `search_memory` acepta
   numpy/list/array.array ✅. `put_batch` posicional deprecada pero sigue
   siendo el primer argumento — emite warning en cada llamada (ruido).
5. **close**: correcto en mono-hilo; riesgo de deadlock concurrente (H2).
   No hay `__del__`/context-manager en el sync `VantaDB` — el usuario debe
   recordar cerrar o confiar en GC del Arc (fugas de file handles en scripts).
6. **Async**: wrapper completo y bien pensado (semáforo + `to_thread`),
   salvo H4 y ausencia de sub-clients (`db.memory.*` etc.) en async.

**Dónde se rompe/confunde**: mezcla de convenciones objeto-vs-dict, `query()`
que devuelve texto formateado (H8), y la primera llamada a `put_batch`
posicional que escupe un DeprecationWarning en producción.

## 5. API Coverage vs Core (verificado)

Expuestos en Python (flat, 42 métodos + `connect`): insert, get, delete,
put, put_batch, put_batch_raw, get_memory, delete_memory, list_memory,
search_memory, search, search_batch, search_batch_requests,
explain_memory_search, supersede, generate_snippet, purge_expired,
list_namespaces, capabilities, hardware_profile, operational_metrics, query,
flush, compact_wal, compact_layout, rebuild_index, reindex_hnsw_from_text,
repair_text_index, audit_text_index (+deep vía flag), export_namespace,
export_all, import_file, bulk_import, bulk_import_bytes, close, add_edge,
graph_bfs, graph_dfs, graph_topological_sort, graph_is_dag, graph_page_rank,
graph_degree_centrality, recover_archived_nodes.

**NO expuestos** (verificado contra `src/sdk/`):

| Método core | Ubicación | Impacto |
|---|---|---|
| `count(namespace, filter)` | api.rs:1412 | Alto — conteo requiere listar todo |
| `delete_by_filter(namespace, filter)` | api.rs:1343 | Alto — borrado masivo requiere loop cliente |
| `similar_to_key(...)` | api.rs:1520 | Alto — caso de uso central de memoria |
| `namespace_stats(...)` | api.rs:1474 | Medio — observabilidad |
| `versions(ns,key)` / `get_version(...)` | api.rs:451,469 | Alto — historial de versiones inaccesible |
| `remove_edge(source,target,label)` | api.rs:1218 | Alto — grafo sin forma de quitar aristas |
| `vacuum()` | api.rs:81 | Medio — mantenimiento |
| `pipeline()/optimizer_config()/set_optimizer_config()` | api.rs:90-106 | Medio — tuning |
| `search_multi` / `search_all` | search/multi.rs:20,76 | Medio — búsqueda multi-namespace |
| `graph_bfs_filtered` / `graph_dfs_filtered` | sdk/graph.rs:89,113 | Medio |
| `graphrag_search(...)` | builder.rs:146 | Alto si es feature pública |
| Threads agentic: `create_thread`, `send_message`, `get_thread`, `list_threads`, `delete_thread`, `purge_expired_threads` | builder.rs:161-202 | Depende de si son públicos |
| `create_snapshot` / `list_snapshots` | builder.rs:243,249 | Medio — backup/restore |

Campos de request no surfaced: `VantaMemoryListOptions.filter_ops`,
`VantaMemorySearchRequest.query_sparse` (sparse vectors) y `search_profile`.

Cobertura aproximada: **~29 de ~42 métodos públicos del SDK embebible ≈ 69%**.
Los gaps de mayor impacto son `similar_to_key`, `count`, `delete_by_filter`,
`versions` y `remove_edge` — todos casos de uso naturales desde Python hoy
imposibles o O(n) del lado cliente.

## 6. Ponytail Review (complejidad)

- `convert.rs:23-34,585-640`: `yagni:` LRU cache keyed-by-repr para dictitos de
  metadata ≤4 entradas. Si el profiling no muestra win medible, eliminar deja
  el código ~50 líneas más corto y elimina la clase de bugs sutiles de cache.
- `lib.rs:469-540` + firma deprecated: `shrink:` mantener la API de tuples como
  primer parámetro deprecado obliga a doble camino de parsing permanente.
  Cortar el camino viejo en la próxima minor simplifica ~60 líneas.
- Duplicación de stubs (`vantadb_py.pyi` + `__init__.pyi` casi idénticos):
  `delete:` consolidar en un stub; la deriva H3 es consecuencia directa de la
  duplicación.
- `forward_to_db!` (lib.rs:256-274): buena decisión — macro delegante, cero
  lógica nueva. 👍

net: ~−120 líneas posibles sin pérdida funcional.

## 7. Alternativas Evaluadas (brainstorm)

- **Excepciones jerárquicas custom vs builtins**: una clase base `VantaError`
  (subclasando RuntimeError) con subtipos Storage/Validation/Timeout permitiría
  `except VantaError` fino sin romper el mapeo actual. Recomendado — costo bajo,
  se agrega sin tocar el mapeo existente (solo envolver).
- **Stubs generados automáticamente** (p.ej. generar `.pyi` desde los
  `#[pymethods]` en build, o al menos un test que compare firmas) vs
  mantenimiento manual: el manual ya derivó (H3). Recomendado un test de
  consistencia barato antes que tooling nuevo.
- **Teardown de tests**: fixture autouse que cierre DBs creados resolvería H1
  sin tocar el core.

## 8. Recomendaciones Priorizadas (iterate)

1. **(bloquea release)** Arreglar H1: fixture de teardown que cierre todas las
   DBs de test + aislar `VANTADB_MEMORY_LIMIT`/límites por test; meta: `pytest`
   completo verde en una pasada.
2. **(bloqueante de seguridad-concurrencia)** H2: mover `drain()` dentro de
   `py.detach` (el condvar no necesita GIL) y añadir test de estrés
   ops-concurrentes-vs-close. Verificar idempotencia de doble `close()`.
3. Regenerar/sincronizar stubs `.pyi` (H3) y añadir `direction` a
   `AsyncVantaDB.graph_bfs/dfs` (H4); idealmente un test de drift stubs↔runtime.
4. Exponer el top-5 de gaps de API: `similar_to_key`, `count`,
   `delete_by_filter`, `versions`, `remove_edge` (H9).
5. Introducir jerarquía `VantaError(VantaStorageError, VantaValidationError,
   VantaTimeoutError)` sobre el mapeo actual (H5).
6. Homogeneizar validación: `distance_metric`/`method` inválidos deberían
   raising `ValueError` como `backend` (H6) — breaking minor, anunciarlo.
7. Retornar estructura (dict/list) de `query()` manteniendo el formato actual
   como `repr` (H8).
8. Limpiar artefactos del repo: wheels stale, `.pdb/.pyd`, fixtures runtime
   (H12) — moverlos a `.gitignore`.

## 9. DoD Multi-nivel

| Nivel | Estado |
|---|---|
| Task | ⚠️ Parcial: suite completa roja (H1); claim "70 passed" cierto pero solo para `test_sdk.py` |
| Commit | ✅ Conventional commits, workspace versioning consistentes (0.5.0 pyproject = Cargo workspace) |
| Release | ⚠️ abi3 wheel correcto, pero matriz multi-plataforma no verificada; artefactos stale en dist |

---

### Dictamen

- **Veredicto:** 🔴 Cambios requeridos
- **Contrato:** NO pasó — `pytest -q` (gate por defecto del pyproject) = 66 failed /
  43 passed; `test_sdk.py` aislado = 70 passed ✅ (claim verificado)
- **DoD:** Task pendiente de H1/H2; Release condicionado a suite verde y matriz de wheels
