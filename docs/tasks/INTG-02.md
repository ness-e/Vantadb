# Task INTG-02 — Backend Memory unificada CrewAI (spec-first)

- **Plan:** `docs/plans/2026-09-10-code.md` (Task 14, Wave4)
- **Estado:** ⏳ IN PROGRESS
- **Ruta:** vanta-worker
- **Appetite:** max 2d · **Esfuerzo:** 🟡 1-2d
- **Contrato:** mini-spec (docs.crewai.com) + backend `Memory` + tests ✅
- **Stop conditions:** spec-first gate — sin mini-spec validada no hay ACT.
- **SDP:** spec-driven-development, source-driven-development, test-driven-development, interview-me, idea-refine (DEFINE; keywords: crewai/memory/backend/mini-spec/short-term/long-term)

## Spec (mini-spec validada — gate spec-first ✅)

Fuentes oficiales (source-driven-development, verificadas 2026-09-10):

1. `Memory` unificada — reemplaza `ShortTermMemory`/`LongTermMemory`/`EntityMemory`/`ExternalMemory`
   (PR #4420); API `remember()/recall()/forget()`, scopes jerárquicos, composite scoring,
   `Memory(storage=backend)` para backend custom:
   https://docs.crewai.com/concepts/memory +
   https://github.com/crewAIInc/crewAI/pull/4420
2. `StorageBackend` protocol (`crewai.memory.storage.backend`, 212L): `save/search/delete/update/
   get_record/list_records/get_scope_info/list_scopes/list_categories/count/reset`
   + async `asave/asearch/adelete`; `search` retorna `list[tuple[MemoryRecord, float]]`:
   https://github.com/crewAIInc/crewAI/blob/main/lib/crewai/src/crewai/memory/storage/backend.py
3. `MemoryRecord`/`ScopeInfo`/`MemoryConfig` (`crewai.memory.types`): `MemoryRecord(id, content,
   scope="/", categories, metadata, importance 0.5, created_at, last_accessed, embedding|None,
   source|None, private=False)`; `ScopeInfo(path, record_count, categories, oldest/newest,
   child_scopes)`; `embed_text` callable `list[str] -> list[vec]`:
   https://raw.githubusercontent.com/crewAIInc/crewAI/main/lib/crewai/src/crewai/memory/types.py
4. Versiones pineadas: `crewai>=1.14,<2` (unified Memory + StorageBackend verificados en docs
   v1.14.7/v1.15.17; edge confirma `StorageBackend` + `set_memory_storage_factory`).
   crewai NO instalado en este env (`.venv` ni sistema) → tests con shim local + `importorskip`
   para conformancia de protocolo.
5. SDK verificado por `inspect` (`.venv`, vantadb-py 0.5.0): `put(ns,key,payload,metadata,vector,
   ttl_ms)` upsert; `get_memory(ns,key)` → `VantaMemoryRecord(payload,metadata,vector,key,...)`;
   `search_memory(ns,vec,top_k)` → `VantaSearchHit(score∈[0,1] similaridad, payload, metadata,
   key)` (probe: exacto=1.0, ortogonal=0.0); `list_memory(ns,limit,cursor)` → `(records,
   next_cursor)`; `delete_memory(ns,key)`; `count(ns)`; `list_namespaces()`; namespaces con
   `/` OK (probe `project/alpha`); `import vantadb_py` deprecated → se mantiene por consistencia
   con `vectorstore.py` (migración = follow-up, precedente INTG-01).

| # | Decisión | Opciones | Default (Recomendado) + evidencia |
|---|----------|----------|-----------------------------------|
| 1 | Qué implementar | StorageBackend completo / solo save+search | **completo sync + async thin** — `Memory` llama `list_records/get_scope_info/count/reset` (discovery, TUI, `reset_memories`); parcial = `AttributeError` en runtime (ref: backend.py 212L, unified_memory.py usa storage.search + list + reset) |
| 2 | Mapeo scopes→VantaDB | 1 namespace + scope en metadata / 1 namespace por scope | **1 namespace + scope en metadata** — scopes LLM-creados dinámicos; filtro por `scope_prefix` en Python (verificado `/` nativo existe pero namespaces dinámicos sin bound complican `reset`/`count` globales) |
| 3 | Keys | `record.id` directo / prefijo | **`record.id` directo** — IDs uuid4 ya únicos; upsert nativo `put` verificado por probe |
| 4 | Campos sistema | claves reservadas `__mem_*` / anidar bajo 1 clave | **`__mem_*` planas** (`__mem_scope/categories/importance/created_at/last_accessed/source/private`) — metadata usuario intacta, split trivial al leer; colisión improbable + documentada |
| 5 | Records sin embedding en search | excluir / fallback listado | **excluir** — LanceDB exige embeddings; `search` con query vacío → `[]`; documentado |
| 6 | Tipos crewai ausentes | shim local / dependencia dura | **shim duck-typed** — `try import crewai.memory.types else dataclass local`; backend opera por atributos (id/content/scope/...), `ScopeInfo` real cuando crewai instalado; tests corren sin crewai |
| 7 | Pin crewai | `>=0.100` actual / `>=1.14,<2` | **`>=1.14,<2`** — unified Memory verificada en docs v1.14.7+; `<2` anti-drift (pre-mortem plan) |

Gate D: feature-add con símbolos públicos nuevos → correspondería `question`, sin herramienta disponible en
este runner → defaults Recomendados arriba (contrato del plan + pre-mortem ya aprobados por owner 2026-09-10).

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vantadb_crewai/vectorstore.py` (257L: `VantaDBTool`, put/get/list/delete),
  `__init__.py` (3L), `pyproject.toml` (`crewai>=0.100`), `README.md`, `tests/test_vectorstore.py`
  (107L, patrón `importorskip` + tmpdb por test), `tests/conftest.py`.
- **Referencias hacia dentro (nuevo código usa):** `vantadb.VantaDB.put/get_memory/delete_memory/
  list_memory/search_memory/count` (signatures + semántica verificadas por probe `.venv`);
  `crewai.memory.storage.backend.StorageBackend` (solo import tipado, fallback `object`);
  `crewai.memory.types.MemoryRecord/ScopeInfo` (solo construcción `get_scope_info`, fallback shim).
- **Referencias entrantes:** ninguna (módulo nuevo `memory.py`; solo `__init__.py` re-exporta).
- **Veredicto:** blast radius = `integrations/crewai/` únicamente (Wave4 disjunto con PRX-09-slice2/GOV-TK8;
  codegraph: 6 callers de `VantaDBTool`, ninguno afectado — no se toca `vectorstore.py`).
  Sin hot paths, sin WAL/vector/storage, sin red/auth. Deuda ajena intacta
  (`opencode.jsonc`, `.opencode`, `desktop/src-tauri/Cargo.lock`, `docs/Backlog.md`,
  `Investigacion-plan.md`, plan file untracked — NO tocar ni stagear).

## Steps

- [x] Step 0 — DISCOVERY: mini-spec + task file (este archivo)
- [x] Step 1 — RED: `tests/test_memory.py` (`ModuleNotFoundError: vantadb_crewai.memory`, razón correcta)
- [x] Step 2 — GREEN `memory.py` core: save/search/get/update/delete + 6/6 verdes
- [x] Step 3 — GREEN resto protocolo: list/info/scopes/categories/count/reset + async (11 passed + 1 skip)
- [x] Step 4 — CLOSE: exports + pin `crewai>=1.14,<2` + README + suite verde + commit

## Verify contrato

- `.venv/Scripts/python -m pytest integrations/crewai/tests/ -q` → **11 passed, 2 skipped**
  (12 memory: 11 ✅ + 1 conformancia-protocolo `importorskip` por crewai no instalado;
  vectorstore skip pre-existente — crewai no instalado en este env)
- `from vantadb_crewai import VantaDBTool, VantaDBMemoryBackend` → OK
- WIP ajeno intacto (opencode.jsonc, .opencode, Cargo.lock, Backlog, Investigacion-plan.md, plan untracked)

NOTICED BUT NOT TOUCHING: `import vantadb_py` deprecated (removal 0.6.0) en `memory.py` +
  `vectorstore.py` — migración a `import vantadb` = follow-up (precedente INTG-01, no scope creep);
  conformancia real vs `StorageBackend` (isinstance) pendiente de env con crewai≥1.14 instalado.
