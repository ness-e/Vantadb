# Task INTG-01 — Adapter LangGraph (spec-first)

- **Plan:** `docs/plans/2026-09-10-code.md` (Task 11, Wave3)
- **Estado:** ⏳ IN PROGRESS
- **Ruta:** vanta-worker
- **Appetite:** max 3d · **Esfuerzo:** 🔴 2-3d
- **Contrato:** mini-spec (fuentes oficiales ref) + checkpointer + `BaseStore` KV jerárquico + tests ✅
- **Stop conditions:** spec-first gate — sin mini-spec validada no hay ACT.
- **SDP:** spec-driven-development, source-driven-development, test-driven-development, interview-me, idea-refine (DEFINE; keywords: langgraph/checkpointer/basestore/mini-spec/kv-jerárquico)

## Spec (mini-spec validada — gate spec-first ✅)

Fuentes oficiales (source-driven-development, verificadas 2026-09-10):

1. `BaseCheckpointSaver` — métodos requeridos `put/put_writes/get_tuple/list/delete_thread` (+ variantes `a*`):
   https://github.com/langchain-ai/langgraph/blob/5931a5f0/libs/checkpoint/README.md +
   https://reference.langchain.com/python/langgraph.checkpoint/base/BaseCheckpointSaver
2. `BaseStore` — abstractos SOLO `batch/abatch`; ops `GetOp/PutOp(value=None=delete)/SearchOp/ListNamespacesOp`
   (verificado en instalada v4.2.0: `BaseStore.__abstractmethods__ == {'batch','abatch'}`):
   https://docs.langchain.com/oss/python/langgraph/stores ("Build a custom store") +
   `libs/checkpoint/langgraph/store/base/__init__.py`
3. Namespaces = tuplas jerárquicas con prefix matching; `Item(namespace,key,value,created_at,updated_at)`,
   `SearchItem` + `score`; `MatchCondition(match_type, path)`:
   https://docs.langchain.com/oss/python/langchain/long-term-memory
4. Referencia de implementación: `InMemorySaver` (misma semántica put/get_tuple/list/put_writes/delete_thread,
   async = thin wrappers) — instalada en `site-packages/langgraph/checkpoint/memory/__init__.py`.
5. Versiones pineadas: `langgraph-checkpoint>=2,<5` (instalada 4.2.0 verificada), `langchain-core>=0.3`
   (instalada 1.6.2). No existe paquete `langgraph-store` en PyPI (store vive en `langgraph-checkpoint`).

| # | Decisión | Opciones | Default (Recomendado) + evidencia |
|---|----------|----------|-----------------------------------|
| 1 | Alcance checkpointer | full blobs split / inline | **inline** — channel_values inline en payload; semánticamente equivalente, sin split necesario (ref: InMemorySaver usa blobs como optimización, no requisito) |
| 2 | Mapeo namespaces→VantaDB | join `/` / plano con prefijo | **join `/`** — preserva jerarquía + prefix matching nativo; partes con `/` o vacías → `InvalidNamespaceError` |
| 3 | Keys checkpointer | string compuesto / JSON array | **JSON array** `[thread_id, ns, id]` — reversible, sin escaping |
| 4 | Semantic search store | sin query / con embeddings opcional | **embeddings opcional** — `query` sin embeddings → `ValueError` documentado; con embeddings → `search_memory` por namespace + merge por score |
| 5 | TTL / index args | soportar / rechazar | **rechazar TTL** (`NotImplementedError`, `supports_ttl=False`); `index` aceptado e ignorado (documentado) |
| 6 | Import name | `vantadb` nuevo / `vantadb_py` actual | **`vantadb_py`** — consistente con `vectorstore.py` existente; migración a `vantadb` = follow-up (no scope creep) |

Gate D: feature-add con símbolos públicos nuevos → correspondería `question`, sin herramienta disponible en
este runner → defaults Recomendados arriba (contrato del plan + pre-mortem ya aprobados por owner 2026-09-10).

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vantadb_langchain/vectorstore.py` (1-573 parcial: init/put/search/delete),
  `__init__.py`, `pyproject.toml`, `tests/test_vectorstore.py`, `tests/conftest.py`, `README.md`,
  `langgraph/checkpoint/memory/__init__.py` (InMemorySaver, referencia externa instalada).
- **Referencias hacia dentro (nuevo código usa):** `vantadb.VantaDB.put/get_memory/delete_memory/list_memory/
  list_namespaces/search_memory` (signatures verificadas por `inspect`); `langgraph.checkpoint.base.*`;
  `langgraph.store.base.*`.
- **Referencias entrantes:** ninguna (módulos nuevos `store.py`, `checkpointer.py`; solo `__init__.py` re-exporta).
- **Veredicto:** blast radius = `integrations/langchain/` únicamente (Wave3 disjunto con PRX-07/DESKTOP-45).
  Sin hot paths, sin WAL/vector/storage, sin red/auth. Deuda ajena intacta
  (`opencode.jsonc`, `.opencode`, `Investigacion-plan.md` — NO tocar ni stagear).

## Steps

- [x] Step 0 — DISCOVERY: mini-spec + task file (este archivo)
- [x] Step 1 — RED: `tests/test_store.py` + `tests/test_checkpointer.py` (ImportError VantaDBStore, razón correcta)
- [x] Step 2 — GREEN `store.py`: `VantaDBStore(BaseStore)` batch/abatch + 12/12 verdes
  (2 fallos intermedios → root cause: `_put` sin vector/metadata; fix en puente, no en tests)
- [x] Step 3 — GREEN `checkpointer.py`: inline (sin blob split) + 7/7 verdes
  (1 fallo: `:` prohibido en namespaces VantaDB → `langgraph.checkpoints`/`.writes`)
- [x] Step 4 — CLOSE: exports + `langgraph-checkpoint>=2,<5` + README + 47/47 + e2e StateGraph real

## Verify contrato

- `python -m pytest integrations/langchain/tests/ -q` → **47 passed** (27 vectorstore + 12 store + 8 checkpointer)
- e2e: `StateGraph` compilado + `VantaDBCheckpointer` — invoke×2 + `get_state` ✅
- `python -c "from vantadb_langchain import VantaDBStore, VantaDBCheckpointer"` → OK
- Review 5 ejes (code-review-and-quality): Approve — sin Critical/Required pendientes
- WIP ajeno intacto (PRX-07, Cargo.lock, .opencode, opencode.jsonc, Backlog, Investigacion-plan.md)

NOTICED BUT NOT TOUCHING: `import vantadb_py` deprecated (removal 0.6.0) en vectorstore.py +
  nuevos módulos — migración a `import vantadb` = follow-up; `pytest-asyncio` instalado global
  (necesario para suite async, no pineado en pyproject del adapter).

## Verify contrato

- `python -m pytest integrations/langchain/tests/ -q` → 0 failed
- `python -c "import vantadb_langchain"` → expone `VantaDBStore`, `VantaDBCheckpointer`
