# FIND-64 — Fix llamaindex put_batch legacy roto

> **Plan:** `docs/plans/2026-09-07-backlog-triage.md` (Task 1, Wave0)
> **Estado:** ⏳ IN PROGRESS
> **Appetite:** max 1d · **Esfuerzo:** 🟡 2-4h · **Prioridad:** 🟠
> **Branch:** (no commitear — solo vanta-lead commitea; diff listo)
> **SDP:** campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design + systematic-debugging (bug)

## Contrato

- `python -m pytest integrations/llamaindex/tests -q` pasa
- `grep -n put_batch integrations/llamaindex/vantadb_llamaindex/vectorstore.py` muestra kwargs (`keys=`/`vectors=`/`payloads=`/`metadatas=`/`namespace=`)

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `integrations/llamaindex/vantadb_llamaindex/vectorstore.py` (481L), `integrations/llamaindex/tests/test_vectorstore.py`, `integrations/llamaindex/pyproject.toml`, `vantadb-python/src/lib.rs` §470-560 (firma `put_batch`), `integrations/langchain/vantadb_langchain/vectorstore.py` §387-448 (patrón `put` con kwargs — referencia).
- **Referencias hacia dentro (qué usa el call-site):** `vanta.VantaDB.put_batch` (PyO3, `vantadb-python/src/lib.rs:484`, signature `(keys, vectors, payloads=None, metadatas=None, namespace=None, namespaces=None, ttls=None)`); `node.get_embedding()`, `node_to_metadata_dict`.
- **Referencias entrantes (quién llama a `VantaDBVectorStore.add`):** solo framework LlamaIndex (`BasePydanticVectorStore`); ningún otro archivo del repo importa `vantadb_llamaindex` (grep `put_batch` en `integrations/` → único hit `vectorstore.py:134`).
- **Veredicto:** blast radius = 1 archivo + tests existentes. Sin cambio de API pública, sin nuevos símbolos, sin hot path (adapter glue). Riesgo 🟢 mínimo.

## Root cause (systematic-debugging Fase 1)

`vectorstore.py:130` construye 6-tuplas legacy `(namespace, node_id, text, metadata, embedding, None)` y `:134` las pasa como **1 posicional** a `put_batch(entries)`. La firma actual exige `keys: Vec<String>` como primer posicional + `vectors` requerido → TypeError / error de conversión 100% reproducible en `add()`.

## Steps atómicos

| # | Step | Estado | Verify |
|---|------|--------|--------|
| 1 | Repro: `python -m pytest integrations/llamaindex/tests -q` confirma fallo en `add` (TypeError) | ✅ DONE — `TypeError: VantaDB.put_batch() missing 1 required positional argument: 'vectors'` en `vectorstore.py:134` (con `PYTHONPATH=vantadb-python`, py3.11) | output del runner |
| 2 | Fix call-site `:118-134`: columnas directas + `put_batch(keys=, vectors=, payloads=, metadatas=, namespace=)` (sin tupla intermedia, sin `ttls` — siempre None) | ✅ DONE | `campaign_verify_cmd` (grep kwargs) |
| 3 | Verify contrato: pytest suite + grep kwargs | ✅ DONE — 23 passed; grep `:140` muestra kwargs | contrato ✅ |

## Pre-mortem (del plan — verificado)

1. Compat versiones viejas → **ya mitigado**: `pyproject.toml` pinnea `vantadb-py>=0.5.0,<0.6.0`. Sin shim.
2. Mapping 6-tupla → columnas: orden verificado en `:130` = (ns, key, payload, metadata, vector, ttl). Test `test_add_multiple_nodes` + `test_add_and_query` lo cubren como fixture real.
3. Regresión → suite existente en el mismo PR (sin archivo nuevo necesario).

## Herramientas

- Repro/verify: `python -m pytest integrations/llamaindex/tests -q`
- Grep contrato: `Select-String -Pattern "put_batch" -Path integrations/llamaindex/vantadb_llamaindex/vectorstore.py`
- Gate mecánico por step: `campaign_verify_cmd`

## Context Save Point

- 2026-09-07 DISCOVERY: task file creado; firma nueva y call-site roto verificados en código (no re-derivar). Siguiente: Step 1 repro.
- 2026-09-07 CIERRE: 3/3 steps ✅. Contrato verde (23 passed + kwargs). Diff listo, SIN commit (regla: solo vanta-lead commitea). NOTICED BUT NOT TOUCHING: `DeprecationWarning: 'vantadb_py' import name deprecated → use 'import vantadb'` en `vectorstore.py:7` (afecta a todos los adapters; migrar en tarea aparte, fuera de appetite 1d). Plan file NO tocado (waves paralelas MOD-24/FIND-60 en vuelo).
