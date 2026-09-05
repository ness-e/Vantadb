# TASK QW-1: crewai from_dict + cursor

## Metadata
- **Plan file:** `docs/plans/2026-08-25-integrations-research-wins.md`
- **Creado:** 2026-08-27T00:00
- **last-synced:** 2026-08-27T16:30
- **Estado:** ✅ COMPLETED
- **Ruta:** vanta-worker
- **Prioridad:** Wave 1 — Fixes de bugs de contrato

## Blast Radius

Callers | Callees | Implicaciones

- `VantaDBTool.from_dict` (crewai vectorstore.py:212) — 1 caller en tests `test_from_dict_roundtrip_no_typeerror`; era `embedding=data.get("embedding_model")` pasando string como callable → TypeError en `_run`/`_put`. Fix ignora string, fallback a listing.
- `VantaDBTool.to_dict` (crewai vectorstore.py:194) — serializa `embedding_model` como type name; roundtrip no debe re-inyectarlo.
- `VantaDBTool.list` (crewai vectorstore.py:168) — cursor str desde páginas serializadas → `list_memory` espera int; patrón dspy.
- `VantaDBTool._run` (crewai vectorstore.py:75) — usa `self.embedding(query)` si existe; sin embedding fallback a `list_memory`.
- `VantaDBTool.__init__` + `top_k` field — persitencia `k` en dict, reconstrucción en `from_dict`.
- **Implicaciones:** thin wrapper, sin tocar `vantadb/src/*` (protegido). Blast radius pequeño (<10 archivos), solo `integrations/crewai/`.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos (antes de editar):**
  - `integrations/crewai/vantadb_crewai/vectorstore.py` (236 líneas completas)
  - `integrations/crewai/tests/test_vectorstore.py` (107 líneas completas)
  - `integrations/crewai/pyproject.toml` (36 líneas completas)
  - `integrations/crewai/vantadb_crewai/__init__.py` (3 líneas)
  - `integrations/dspy/vantadb_dspy/vectorstore.py:161-173` (patrón cursor dspy)
  - `docs/plans/2026-08-25-integrations-research-wins.md` (Wave 1 QW-1)
- **Referencias hacia dentro (qué importa este archivo):**
  - `import vantadb_py as vanta` → `vantadb-python` (PyO3, no tocar core)
  - `crewai.tools.BaseTool` → fallback object si no instalado
  - `pydantic.PrivateAttr` → `_db` private attr
- **Referencias entrantes (qué depende de lo que cambio):**
  - `tests/test_vectorstore.py` — 8 tests usan VantaDBTool (from_dict, cursor, _run, _put)
  - `integrations/crewai/README.md` → importa VantaDBTool
  - Ningún módulo Rust core depende de este adapter (aislado)
- **Veredicto:** impacto localizado, seguro para edit. No toca paths multi-índice/dashmap/parking_lot/Tokio → no requiere auditoría concurrencia (Regla 8). No hot path vectorial → no requiere perf bench (Regla 9).

## Spec

N/A — bug-fix con contrato mecánico (Wave 1 QW-1). No agrega símbolos públicos nuevos; solo corrige contrato existente.

Problema: `from_dict` pasaba `embedding_model` string como callable → TypeError en roundtrip; `list(cursor=...)` no convertía str→int.
Criterio: roundtrip `to_dict→from_dict→_run` sin TypeError; `list(cursor=str)` funciona; 8 tests crewai pasan.
Alcance: `integrations/crewai/vantadb_crewai/vectorstore.py:164-176,217-218` + `top_k` persistencia.
Decisiones: embedding_model es solo type-name, no reconstruible → se ignora (documentado), fallback a listing; cursor siempre int(cursor) si no None; top_k campo Pydantic.

## Contrato

```
python -m pytest integrations/crewai/tests -q  # 8 passed
roundtrip to_dict→from_dict→_run(query) no lanza TypeError; from_dict reconstruye embedding callable (ignora string); list(cursor=...) str→int
```

Verificación mecánica:
1. `python -m pytest integrations/crewai/tests/test_vectorstore.py::test_from_dict_roundtrip_no_typeerror -q` ✅
2. `python -m pytest integrations/crewai/tests/test_vectorstore.py::test_list_cursor_string -q` ✅
3. `python -m pytest integrations/crewai/tests -q` → 8 passed ✅

## Herramientas

Skills cargadas: `source-driven-development`, `ponytail`, `systematic-debugging`, `test-driven-development` (≤4 por SDP, resto sin candidatos)

- **source-driven-development:** verificar APIs externas (crewai BaseTool, vantadb_py list_memory cursor tipo)
- **ponytail (full):** ladder mínimo — reusar stdlib, no añadir deps, thin wrapper
- **systematic-debugging:** root cause TypeError en from_dict → string-as-callable
- **test-driven-development:** contrato ya en tests existentes, verify rojo→verde
- **SKILLS_CARGADAS (SDP):** source-driven-development, ponytail, systematic-debugging, test-driven-development + SDP sin candidatos adicionales (keywords: crewai, vectorstore, cursor, from_dict, embedding)

Lifecycle mapping BUILD (source-driven, ponytail, TDD) + VERIFY (systematic-debugging)
Grep SKILLS-MANIFEST.md por `crewai|vector|embedding|cursor` → sin skill directa; `vantadb` genérica no añade valor específico al fix.
Discovery ≤8 skills → 4 cargadas, justificadas arriba.

## Steps

### Step 1: DISCOVERY — verificar fixes ya aplicados y blast radius
- **Archivos:** `integrations/crewai/vantadb_crewai/vectorstore.py`, `integrations/crewai/tests/test_vectorstore.py`
- **Acción:** Confirmar que fixes QW-1 (96e143ec + 0ae070ca) ya están en disco: from_dict ignora embedding_model, top_k persiste, cursor str→int. Mapear Regla 0 completa arriba. Validar que 8 tests existentes cubren contrato.
- **Verify:** `python -m pytest integrations/crewai/tests -q` → 8 passed; `codegraph_explore` blast radius
- **Estado:** ✅ DONE (2026-08-27 — ya en HEAD; 8 passed verificado)

### Step 2: ACT — harden from_dict + list edge cases (si hace falta) + validar roundtrip
- **Archivos:** `integrations/crewai/vantadb_crewai/vectorstore.py:168-192,212-233`
- **Acción:** Revisar `from_dict` edge: si `data` trae `embedding` callable directo (no solo `embedding_model` string) debe respetarlo; validar que `list(cursor="")` / `cursor="0"` / `cursor=None` no rompe. Añadir manejo robusto: `cursor` vacío → None, int() con try/except ValueError → raise accionable. Verificar `to_dict`/`from_dict` roundtrip sin DB lock (paths distintos). Si todo ya verde, no editar — validar.
- **Verify:** scripts python inline: roundtrip con/without embedding, list con cursor str/int/None; pytest 8 passed
- **Estado:** ✅ DONE (2026-08-27 — edits: list str→int robusto + next_cursor fix + from_dict embedding callable + top_k fallback; inline edge tests 5 casos + pytest 8 passed)

### Step 3: VERIFY — suite contract + edge cursor exhaustivo
- **Archivos:** `integrations/crewai/tests/test_vectorstore.py`
- **Acción:** Ejecutar full suite + casos borde: `from_dict({"embedding_model": "function"})` → no TypeError; `_run` tras from_dict sin embedding debe fallback; `list(limit=2)` paginado serializado str cursor chain.
- **Verify:** `python -m pytest integrations/crewai/tests -v` 8 passed; manual `campaign_verify_cmd` equivalente
- **Estado:** ✅ DONE (2026-08-27 — 8 passed 43.82s + paginated cursor chain 2→4 verified + invalid cursor ValueError accionable)

### Step 4: CIERRE — verify full + commit + recitation + progreso
- **Archivos:** `.opencode/skills/campaign-executor/tasks/QW-1.md`, `docs/plans/2026-08-25-integrations-research-wins.md`
- **Acción:** `cargo fmt --check` (no aplica, py only) + `cargo clippy` skip; pytest gate; git add solo `integrations/crewai/` si hubo cambios; commit conventional `fix(integrations): crewai from_dict + cursor (QW-1)`; update plan recitation; skill progreso; handoff
- **Verify:** `campaign_verify_cmd` + git log
- **Estado:** ✅ DONE (2026-08-27 — verify: pytest 8 passed, cursor empty→None, cursor str→int, from_dict embedding callable; sin commit por regla lead commitea; plan sync abajo)

## Dependencias
- Ninguna (Wave 1 QW-1 independiente; QW-2/QW-3 disjuntos)

## Notas
- Fix ya merged en 96e143ec (cursor + from_dict) y 0ae070ca (top_k). Esta tarea pipeline-full re-verifica contrato y hardeniza edges; si no hace falta código, cierra como verify-only con evidencia.
- `embedding_model` string no reconstruible — docstring explícito: pasar callable explícitamente si se necesita semantic search tras roundtrip.
- Cursor pattern tomado de `integrations/dspy/vantadb_dspy/vectorstore.py:172` (str→int).

## Context Save Point
- **Fecha:** 2026-08-27T16:30
- **Branch:** develop
- **CI pendiente:** no — local pytest 8 passed 43.82s (2026-08-27) + edge cursor chain verified
- **Decisiones:** harden minimal ponytail: list empty→None + ValueError accionable + next_cursor fix (`next_cursor`/`cursor` fallback, `is not None`); from_dict embedding callable (`embedding` if callable, fallback `embedding_model` callable, else None) + `k`/`top_k` compat
- **Problemas conocidos:** ninguno
- **Próxima tarea:** QW-2
