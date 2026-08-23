# Review de Módulo — `integrations/`

**Fecha:** 2026-08-23 · **Revisor:** ox-alpha (worker) · **Alcance:** los 9 adapters Python de `integrations/`
**Método:** lectura completa del código fuente de los 9 adapters + `pyproject.toml` + `__init__.py` de cada uno; tests leídos en detalle solo para langchain; verificación puntual contra la API real de `vantadb-python` (codegraph + grep) y PyPI en vivo.

---

## Resumen

`integrations/` contiene **9 adapters Python puros** (el backlog MKT-18f hablaba de 5 — está desactualizado): `langchain`, `llamaindex`, `mem0`, `crewai`, `dspy`, `haystack`, `letta`, `ollama`, `openai`. Todos son thin wrappers sobre el SDK `vantadb-py`, todos versionados 0.5.0 con hatchling, y **ninguno está publicado en PyPI** (verificado hoy: `pypi.org/pypi/vantadb-langchain/json` y `.../vantadb-openai/json` → 404). **MKT-18f sigue vigente.**

## Arquitectura

```
Framework (LangChain / LlamaIndex / mem0 / CrewAI / DSPy / Haystack)
   ▼
Adapter (integrations/*/vantdab_*/vectorstore.py)  ← implementa la clase base del framework
   ▼
vantadb_py.VantaDB (PyO3) → core Rust
```

Patrón común y consistente en todos: constructor con `db_path`/`namespace`/`memory_limit_bytes`/`read_only`/`backend`, embedding opcional inyectable como callable, operaciones sobre un namespace dedicado.

## Contratos por adapter

| Adapter | Clase base del framework implementada | Contrato respetado | Tests | Metadata publicable | Publicado |
|---|---|---|---|---|---|
| langchain | `langchain_core.vectorstores.VectorStore` (+MMR, from_texts, get_by_ids) | ✅ Completo | ✅ Extensos (~30 tests) | ✅ | ❌ |
| llamaindex | `BasePydanticVectorStore` (add/delete/query/get_nodes/clear) | ⚠️ Sí, con bugs latentes | ✅ Presentes | ✅ | ❌ |
| mem0 | `mem0.vector_stores.base.VectorStoreBase` (11 métodos abstractos) | ⚠️ Sí, con fallbacks por APIs faltantes | ✅ Presentes | ✅ | ❌ |
| crewai | `crewai.tools.BaseTool` (`_run`) | ⚠️ Sí, con bug en `from_dict` | ✅ Presentes | ✅ | ❌ |
| dspy | `dspy.Retrieve` (`forward` → `Prediction(passages)`) | ✅ Sí | ✅ Presentes | ✅ | ❌ |
| haystack | Protocolo `DocumentStore` Haystack 2.x (write/filter/count/delete/to_dict) | ✅ El más completo | ✅ Presentes | ✅ | ❌ |
| letta | **Ninguno** — clase planilla propia | ❌ Letta no tiene API pública de vector-store; es un adapter "conveniencia" | ✅ Presentes | ✅ | ❌ |
| ollama | Ninguno oficial — API estilo LangChain (add_texts/similarity_search/a*) | ⚠️ Parcial | ✅ Presentes | ✅ | ❌ |
| openai | Ninguno oficial — gemelo casi idéntico al de ollama | ⚠️ Parcial | ✅ Presentes | ✅ | ❌ |

## Fortalezas

1. **Consistencia transversal real**: mismo constructor, mismas convenciones de namespace y embedding-callable en los 9.
2. **Dependencias correctas**: `vantadb-py>=0.5.0,<0.6.0` acotado; framework como dependencia declarada (salvo letta, que no necesita ninguna).
3. **Los imports contra la API real de vantadb-py son correctos**: verificado que `put`, `get_memory`, `search_memory(namespace, vector, ...)`, `list_memory`, `delete_memory`, `put_batch`, `list_namespaces` existen con esas firmas. Los adapters usan `search_memory` con namespace posicional — coincide.
4. **Manejo honesto de filtros complejos**: llamaindex y haystack traducen EQ nativo y post-filtran OR/NOT/comparaciones client-side, documentado con comentarios.
5. Docstrings exhaustivos estilo Sphinx en langchain/haystack/mem0.

## Hallazgos

| # | Severidad | Adapter | Archivo | Hallazgo |
|---|-----------|---------|---------|----------|
| I1 | **High** | todos | `pyproject.toml` × 9 | **MKT-18f confirmado**: ningún paquete publicado a PyPI (404 verificado para 2 muestras). La metadata ES publicable (classifiers, URLs, licencia, wheel target correcto), así que es un gap de publicación, no de calidad. |
| I2 | **High** | crewai | `vectorstore.py:217-218` | `from_dict()` pasa `data.get("embedding_model")` — **un string** — como `embedding`. Luego `_run()` llama `self.embedding(query)` → `TypeError: 'str' object is not callable`. Round-trip to_dict→from_dict roto. |
| I3 | **Medium** | crewai | `vectorstore.py:164-176` | `list(cursor: Optional[str])` pasa el cursor str directo a `list_memory(cursor=...)`, cuyo parámetro PyO3 es `Option<usize>` → `TypeError` si alguien pagina. dspy lo hace bien (convierte a int); crewai no. |
| I4 | **Medium** | langchain | `vectorstore.py:470` | `add_documents()`: `ids = [doc.id for doc in documents if doc.id is not None] or None` — si SOLO ALGUNOS docs tienen id, la lista filtrada es más corta que `texts` → `ValueError` engañoso en `add_texts`. Caso mixto sin manejar. |
| I5 | **Medium** | llamaindex | `vectorstore.py:362` | `List[MetadataFilter]` usa un nombre **no importado** (solo se importa `MetadataFilters`). Salvado por `from __future__ import annotations` (lazy), pero revienta con cualquier `get_type_hints()`/generación de schema. |
| I6 | **Medium** | llamaindex | `vectorstore.py:42-50` | Asigna `self._namespace`/`self._client` sin declararlos como `PrivateAttr` en el cuerpo de la clase pydantic. LlamaIndex declara este patrón explícitamente para atributos privados; según versión de pydantic puede lanzar ValidationError. Requiere prueba runtime (no ejecutada). |
| I7 | **Medium** | mem0 | `vectorstore.py:203, 235` | `update_memory` y `delete_namespace` **no existen en vantadb-py** (verificado hoy por grep). El adapter lo sabe (try/except AttributeError con fallback delete+insert), pero significa que `update()` siempre pierde el vector nuevo cuando solo cambia payload, y `delete_col()` siempre hace el path lento N-deletes. Deuda de API, no de adapter. |
| I8 | **Medium** | ollama/openai (int.) | ambos `vectorstore.py` | **Gemelos ~95% idénticos** (~220 líneas duplicadas c/u), incluida una re-definición local de `Document` que sombrea el concepto. Además `asimilarity_search` es fake-async (delega síncrono bloqueando el event loop) mientras `aadd_texts` sí usa executor — inconsistente entre sí. |
| I9 | **Low** | mem0 | `vectorstore.py:45-55` | `_normalize_score`: heurística ambigua ("si está en [0,1] pasa, si no invierto") — un score de distancia 0.8 (bastante similar) se vuelve 0.2, pero uno de 1.2 se vuelve -0.2→0. Semántica frágil, marcada como tal. |
| I10 | **Low** | letta | todo el adapter | Sin contrato de framework que verificar — riesgo de drift silencioso si Letta define API en el futuro. Documentar como experimental o retirar. |
| I11 | **Low** | crewai | `vectorstore.py:229-293` | `categorize()` — lógica de dominio keyword-matching muerta en un adapter, ya marcada DEPRECATED. Candidato a borrado inmediato (ponytail: delete). |
| I12 | **Low** | haystack | `vectorstore.py:14, 370-382` | `count_documents()` lista hasta 1M records para contar → O(n) memoria. Con cursor pagination disponible, debería contar por páginas. |

### Ponytail-audit (solo complejidad)

- `delete:` `categorize()` completa (~65 líneas) — dominio, no adapter. [crewai]
- `shrink:` factorizar `Document` dataclass + add_texts/delete/a* compartidos de openai/ollama en un módulo común interno → ~200 líneas menos y una sola fuente de verdad. [ollama, openai]
- `yagni:` fallbacks try/ImportError de crewai/dspy cuando el paquete igual depende del framework en pyproject — el fallback solo disfraza installs rotos. [crewai, dspy]

## Flujo de uso real de un dev (hoy)

```bash
pip install vantadb-langchain        # ❌ FALLA — no existe en PyPI
pip install git+https://github.com/ness-e/Vantadb#subdirectory=integrations/langchain   # workaround
```

Tras instalar (hoy: solo vía repo), el flujo funciona bien:

```python
from vantadb_langchain import VantaDBVectorStore
store = VantaDBVectorStore(my_embeddings, db_path="./data", namespace="docs")
store.add_documents(docs)               # ✅
store.similarity_search("query", k=5)   # ✅
```

**El bloqueador real del flujo es el paso 1**: sin publicación, cada framework-integrado exige install-from-git, lo que mata la adopción. La metadata ya está lista; falta el pipeline de publicación (release-plz cubre crates.io/npm — PyPI de integrations quedó fuera).

## Incompletudes

- Tests de crewai/dspy/haystack/letta/mem0/ollama/openai **no leídos en detalle** (solo langchain); sé que existen (`tests/test_vectorstore.py` en cada uno) pero no evalué su cobertura real.
- No ejecuté ninguna suite (review estática; además I7/I2/I5 se detectaron por lectura, ejecutar suites es el paso siguiente natural).
- Contratos de mem0/CrewAI/DSPy/Haystack verificados contra conocimiento del patrón + estructura del código, **no contra docs oficiales fetcheadas** en esta sesión (source-driven incompleto aquí).
- Compatibilidad de versiones mínimas declaradas (`langchain-core>=0.3`, `llama-index-core>=0.12`, `dspy>=2.6`) no verificada contra releases actuales.

## Propuestas (priorizadas)

1. **Cerrar MKT-18f**: agregar publicación PyPI de los 9 paquetes al release pipeline (o decidir formalmente no publicarlos y quitar metadata "publicable" engañosa). Es el mayor multiplicador de valor del directorio entero.
2. Fix I2 (crewai from_dict) e I3 (cursor str) — bugs de contrato propios, baratos.
3. Fix I4 (langchain ids mixtos) — validar longitud ANTES de filtrar, o generar ids faltantes.
4. Importar `MetadataFilter` en llamaindex (I5) y declarar `PrivateAttr`s (I6) tras prueba runtime.
5. Fusionar ollama/openai-integration en un módulo compartido (I8) — o aceptar la duplicación consciente y documentarla.
6. Pedir a core: `update_memory` + `delete_namespace` en vantadb-py para que mem0 deje de vivir de fallbacks (I7).
7. Borrar `categorize()` (I11) y decidir el futuro de letta (I10).

## Score

**6.5 / 10**

El mejor directorio de los dos revisados: contratos mayormente respetados, tests presentes en los 9, imports correctos contra la API real, metadata seria. Pierde puntos por: cero presencia en PyPI (el hallazgo que motiva MKT-18f sigue siendo cierto hoy), 4 bugs latentes de contrato (crewai×2, langchain×1, llamaindex×2 menores), y ~450 líneas de duplicación openai/ollama que ya generaron divergencia interna.

## No verificado

- Contenido de `tests/test_vectorstore.py` de crewai, dspy, haystack, letta, mem0, ollama y openai (existencia confirmada; contenido no leído — sesión anterior cortó por OOM).
- READMEs de los 9 adapters (no leídos).
- Ejecución runtime de las suites ni validación de I6/I5 en caliente.
- Docs oficiales de mem0/crewai/dspy/haystack para validar el contrato al 100% (fuente-driven pendiente).

---

## Trazabilidad Backlog

Derivado a la fase **P32** de `docs/Backlog.md` (2026-08-23):

| Hallazgo | Tarea |
|---|---|
| I2/I3 — crewai: `from_dict` pasa string como embedding (`TypeError`) + cursor str a `Option<usize>` | **MOD-46** |
| I4 — langchain: ids parciales en `add_documents()` producen `ValueError` engañoso | **MOD-47** |
| I5/I6 — llamaindex: `MetadataFilter` no importado + atributos privados sin `PrivateAttr` | **MOD-48** |
| I8 — Gemelos openai/ollama ~95% idénticos (async inconsistente entre sí) | **MOD-49** |
| I7, I9–I12 — nits (fallbacks mem0 por APIs faltantes, heurística `_normalize_score`, letta sin contrato, `categorize()` muerta, `count_documents()` O(n)) | **MOD-50** |

Los hallazgos ya trackeados previamente que este reporte menciona (**MKT-18f** — hallazgo I1: 0 paquetes publicados en PyPI) → referenciados en su fila existente en `docs/Backlog.md`, no duplicados aquí.
