# INV-008 — Batch Queries Python SDK (diseño)

- **Estado:** DISEÑO — parcialmente implementado (ver Gate 2026-08-03)
- **Dominio:** vantadb-python (bindings PyO3), core SDK (`src/sdk/`)
- **Alcance:** NO implementación. Diseño de la API de batch queries con SearchRequest completo (namespace, text_query, filters, distance_metric, explain), paralelismo con GIL release, y veredicto YAGNI sobre dónde vive el nuevo código.

---

## 1. Estado actual verificado (2026-08-03)

`search_batch` vector-only **YA EXISTE** y funciona end-to-end:

| Componente | Referencia | Detalle |
|---|---|---|
| `search_batch` PyO3 (sync) | `vantadb-python/src/lib.rs:1181-1209` | Firma `(vectors, top_k=10) -> Vec<Vec<(u64, f32)>>` |
| GIL policy eager | `vantadb-python/src/lib.rs:1187` | Comentario `PERF-24`: extracción de vectores con GIL (`extract_vector`) ANTES de detach |
| Ejecución paralela | `vantadb-python/src/lib.rs:1193-1208` | `py.detach(move || ...)` + Rayon `into_par_iter()` sobre `engine.search_vector(&vector, top_k)`; `collect::<Result<_,_>>()` fail-fast |
| Wrapper async | `vantadb-python/vantadb_py/__init__.py:214-217` | `async def search_batch(self, vectors, top_k=10)` → `_run` → `asyncio.to_thread` |
| Stub sync `.pyi` | `vantadb-python/vantadb_py/__init__.pyi:91-93` | `def search_batch(self, vectors: list[Any], top_k: int = 10) -> list[list[tuple[int, float]]]` |
| Stub async `.pyi` | `vantadb-python/vantadb_py/__init__.pyi:203` | `async def search_batch(...)` |
| Core KNN single | `src/sdk/api.rs:1081` | `pub fn search_vector(&self, vector: &[f32], top_k: usize) -> Result<Vec<VantaSearchHit>>` |
| Core search completo | `src/sdk/search/mod.rs:74` | `pub fn search(&self, request: VantaMemorySearchRequest) -> Result<Vec<VantaMemorySearchHit>>` — soporta namespace, text_query, filters, hybrid (`fuse_rrf_with_report`, `validate_metadata`) |
| Core search multi | `src/sdk/search/mod.rs:1383` | `search_multi(...)` — batch a nivel core, ajeno al binding actual |
| Core search all | `src/sdk/search/mod.rs:1439` | `search_all(...)` — barrido completo |
| Tests | `vantadb-python/tests/test_sdk.py` | Cobertura de `search_batch` vector-only |
| Bench | `benchmarks/batch_vs_sequential_bench.py` | Mide batch vs secuencial (baseline para el target de §3) |

### El gap real (1 hueco concreto)

`search_batch` actual **solo acepta vectores + top_k** y devuelve tuplas crudas `(u64, f32)`:
- No acepta un SearchRequest completo (namespace, text_query, filters, distance_metric, explain) → **no sirve para batch híbrido/filtrado**.
- No devuelve payload/metadata, solo `(node_id, distance)`.
- El wrapper Python de `search` single (`__init__.py:211-212`) tampoco expone filters/hybrid: solo `(vector, top_k)`. Es decir, el gap no se puede cerrar desde Python puro sin tocar el binding.

---

## 2. Propuesta de API

### Firma Python (target)

```python
# dataclass de request — mapeo directo a VantaMemorySearchRequest
@dataclass
class SearchRequest:
    vector: list[float]              # obligatorio para KNN; en hybrid se combina con text_query
    top_k: int = 10
    namespace: str | None = None     # partición del índice
    text_query: str | None = None    # activa hybrid (fuse RRF) si hay índice de texto
    filters: dict | None = None      # metadata filters (validate_metadata en core)
    distance_metric: str | None = None  # override del metric del índice
    explain: bool = False            # devuelve explicación del scoring por hit

@dataclass
class SearchResult:
    id: int
    score: float
    payload: dict | None = None      # metadata del nodo (no expuesto hoy por search_batch)
    explanation: dict | None = None  # solo si explain=True

async def search_batch(self, queries: list[SearchRequest]) -> list[SearchResult]:
    """Parallel search over a list of full search requests."""
    return await self._run(self._sync.search_batch_requests, queries)
```

### Firma Rust (nuevo método en el binding)

```rust
/// GIL Policy: RELEASED eager — igual que search_batch (lib.rs:1187).
#[pyo3(signature = (queries,))]
fn search_batch_requests(
    &self,
    py: Python,
    queries: Vec<SearchRequestPy>,
) -> PyResult<Vec<Vec<SearchResultPy>>> { ... }
```

Donde `SearchRequestPy` es un `#[pyclass]` (o `#[derive(FromPyObject)]` sobre dicts) con los mismos campos, y la conversión a `VantaMemorySearchRequest` ocurre **eager con GIL** (misma policy que `PERF-24`): todo dict/objeto Python se valida y convierte antes de `py.detach`.

### Conversión eager (obligatoria)

1. Con GIL: cada `SearchRequest` Python → `VantaMemorySearchRequest` (validar tipos, dimensión del vector, `filters` → formato que `validate_metadata` espera, `text_query`/`namespace` copiados a String).
2. `py.detach` + Rayon (ver §3): búsqueda pura Rust.
3. Re-adquirir GIL en el return del `#[pyfunction]`: construir `SearchResult` Python desde las hits (incluye `payload` y `explanation`).

Esto garantiza **cero objetos Python tocados sin GIL** — el mismo invariante que el patrón actual.

### Manejo de errores parciales

| Opción | Semántica | Costo |
|---|---|---|
| **A. Fail-fast (recomendada v1)** | Si una query del lote es inválida (dimensión equivocada, filtro mal formado), falla TODO el batch con `PyErr` descriptivo. Consistente con `search_batch` actual (`collect::<Result<_,_>>()` en lib.rs:1207). | Cero código nuevo de error |
| B. Por-query | `list[SearchResult | SearchError]` por índice, tolerante. | Tipo de error nuevo + API de resultado más compleja |

**Decisión YAGNI:** Opción A para v1. No hay caso de uso verificado de lotes heterogéneos válido+inválido. Si aparece, se agrega `on_error="raise"|"skip"` sin romper la API actual.

---

## 3. Diseño de ejecución

**Reusar el patrón existente al pie de la letra** (`vantadb-python/src/lib.rs:1187-1208`):

```
1. Extraer y validar TODAS las queries con GIL (eager)     ← ya probado, PERF-24
2. let engine = self.engine.clone()                        ← Arc clonado
3. py.detach(move || {
       queries.into_par_iter()
           .map(|q| engine.search(q.to_core_request()))
           .collect::<Result<Vec<_>, _>>()                 ← fail-fast
   })
4. Convertir hits → SearchResult Python (GIL re-adquirido)
```

Razones por las que este patrón es el correcto (no inventar otro):
- **GIL liberado** durante la parte costosa (traversal de grafos HNSW) — otras threads Python corren.
- **Rayon ya está en el árbol de deps** del binding (lo usa `search_batch`); cero dependencias nuevas.
- **Lectura concurrente del engine ya es thread-safe** (`Arc<VantaEmbedded>` clonado, sin locks nuevos en el hot path).
- Cero `unsafe`, cero `Python::with_gil` anidado.

### Target de performance

> **Batch de 10 queries < 3× el tiempo de 10 `search` single secuenciales.**

- Racional: 10 traversals paralelos con Rayon → speedup esperado 3-5× según cores; 3× es un umbral conservador que no depende del hardware.
- El bench existente `benchmarks/batch_vs_sequential_bench.py` ya mide exactamente este gap con `search_batch` vector-only; al implementar, extenderlo con el método nuevo (mismo harness, caso `queries=10`).
- Riesgo conocido (límite): en un solo core el batch no gana nada vs secuencial — el target asume ≥4 cores, que es el mínimo soportado por el bench actual.

---

## 4. Veredicto: método nuevo vs wrapper sobre `search`

### Opción 1 — Wrapper Python sobre `search` existente (cero Rust)

```python
async def search_batch(self, queries):
    return [await self.search(q.vector, q.top_k) for q in queries]
```

Descartada: **no expresa el gap**. `search` Python solo toma `(vector, top_k)` (`__init__.py:211-212`) — no tiene filters, text_query, namespace ni explain. Un wrapper puro no puede hacer batch híbrido/filtrado sin extender `search` primero, y aun así serializaría N round-trips FFI + N threads `to_thread` (GIL por llamada), perdiendo el paralelismo Rayon. Es más código Python para peor resultado.

### Opción 2 — Método nuevo `search_batch_requests` en el binding (recomendada)

- **Único método nuevo** en `vantadb-python/src/lib.rs`, reutilizando el patrón de `search_batch` (que ya está escrito y testeado).
- **Un dataclass** `SearchRequest` + `SearchResult` en Python (`vantadb_py/__init__.py` + stubs `.pyi`).
- **Un wrapper async** (`_run` → `to_thread`), idéntico a los existentes.
- El core (`src/sdk/search/mod.rs:74`, `engine.search(VantaMemorySearchRequest)`) **ya hace todo** el trabajo pesado — no se toca engine, storage, ni serialización.

### Veredicto final

**Opción 2, recortada al mínimo:** `search_batch_requests` es el mínimo viable porque el requisito (batch híbrido/filtrado) no se puede satisfacer desde el binding actual, pero el core ya lo soporta completo. No se agrega: tipos de error por-query (§2 Opción B), `search_all`/`search_multi` al binding, ni refactor de `search` single (compatibilidad atrás). Todo eso se agrega cuando haya caso de uso real.

---

## 5. Orden de implementación sugerido (para el backlog, fuera de alcance de este doc)

1. `SearchRequestPy`/`SearchResultPy` (o `FromPyObject` sobre dicts) + `search_batch_requests` en `vantadb-python/src/lib.rs` (copiar patrón lib.rs:1187-1208). *(BINDING)*
2. Dataclass `SearchRequest` + `SearchResult` y wrapper async `search_batch(queries)` en `vantadb_py/__init__.py` + stubs `.pyi`. *(PYTHON)*
3. Tests en `vantadb-python/tests/test_sdk.py`: batch de 10 con filters, batch hybrid (vector+text_query), error fail-fast con vector de dimensión inválida. *(TEST)*
4. Extender `benchmarks/batch_vs_sequential_bench.py` con el método nuevo y verificar el target de §3. *(PERF)*

Estimación: 1-2 días, 0 dependencias nuevas, 0 cambios de core/engine/storage.

---

## Gate 2026-08-03

**Estado: PARCIALMENTE IMPLEMENTADO.**

- ✅ **Ya implementado (verificado en código, NO es diseño):** `search_batch` vector-only con GIL release eager + Rayon (`vantadb-python/src/lib.rs:1181-1209`), wrapper async (`__init__.py:214-217`), stubs `.pyi` (`:91`, `:203`), tests (`test_sdk.py`), bench (`batch_vs_sequential_bench.py`).
- ⚠️ **No implementado (diseño pendiente de backlog):** `search_batch_requests` con SearchRequest completo (namespace, text_query, filters, distance_metric, explain), `SearchRequest`/`SearchResult` Python, manejo de errores parciales (v1 = fail-fast), bench extendido con el target 10 < 3× single.
- ✅ **Decisión tomada:** método nuevo en el binding con el patrón existente (§4 Opción 2); sin wrapper Python puro (no expresa el gap), sin tipos de error por-query, sin tocar core/engine.
- **Recomendación al próximo ejecutor:** arrancar por §5 paso 1 — copiar el patrón de `search_batch` (`lib.rs:1187-1208`) que ya está probado; el core `engine.search(VantaMemorySearchRequest)` ya cubre hybrid/filters.

---

*Generado por vanta-worker — INV-008, 2026-08-03.*
