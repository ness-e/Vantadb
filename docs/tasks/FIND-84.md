# FIND-84 — pins + fixtures + dist/PyPI integrations (9 adapters)

## Metadata
- **Plan file:** `docs/plans/2026-09-15-find-correcciones.md` (Task 22, Wave7)
- **Creado:** 2026-09-15
- **Estado:** ⏳ IN PROGRESS
- **Appetite:** 1d · 🟡 · 🟡 · Ruta vanta-worker
- **SDP:** campaign-executor, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, api-and-interface-design (SDP v2 phase=BUILD, keywords integrations-pins/pytest-fixtures/dist-decision/pypi-alpha; `frontend-ui-engineering` devuelta por lifecycle pero DESCARTADA — sin `web/` en scope, justificación registrada)
- **Task file previo:** NO EXISTÍA → DISCOVERY completo según pipeline-full.md

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** 9× `integrations/*/pyproject.toml` (dependencies), 9× `tests/conftest.py` (87B, solo sys.path), `tests/test_*.py` de los 9 adapters (patrón fixtures), `integrations/vantadb_shared/__init__.py:1-70` (packaging note force-include + `vanta.VantaDB` en `:47`), `vantadb-python/vantadb_py/__init__.py:1-60` (exporta `Client`, NO `VantaDB`), `Client` en `vantadb_py.pyi:160-230` (métodos `put/search/delete`, SIN `search_memory/list_memory`), `.github/workflows/adapters-compat.yml` (matriz versiones), `release-adapters-62.yml:29,68` (9 adapters, tag `adapters-v*`), 9× `README.md` (cabeceras PyPI), 9× `.gitignore` (`dist/` ignorado), `docs/Backlog.md:227` (FIND-84) + `:236` (FIND-94).
- **Referencias hacia dentro (qué usa lo que toco):** `release-adapters-62.yml` buildea `integrations/<adapter>/dist/` (artefacto CI, no commiteado); `adapters-compat.yml` instala `pip install -e .` por adapter + fija `framework_version` explícito (uppers no lo rompen); `ci-examples-12.yml:124` instala `integrations/langchain`; `web/src/app/integrations/page.tsx` (matriz marketing, fuera de scope, no se toca).
- **Referencias entrantes (quién me usa):** ninguna parte del runtime importa `integrations/*` (adapters standalone, publicados por separado).
- **Veredicto de impacto:** BAJO-MEDIO. Cambios: metadata (pyproject), tests-only (fixtures + shim), docs (READMEs). Cero código prod de adapters (ningún `vectorstore.py` tocado — incluye prohibición FIND-69 `dspy/vectorstore.py`). Riesgo mayor: pin rompe adapter → mitigado con test central parametrizado (Step 1). Riesgo pytest: bloqueado por FIND-94 → shim test-only acotado (ver HALLAZGO).

## Contrato
"upper-bounds en 7/9 adapters + fixtures sin subdir inexistente/disco + decisión `dist/` documentada + PyPI/Alpha documentada + pytest verde (mocks)."
Verify: `python -m pytest integrations/<adapter>/tests/` por adapter (mocks, sin red) + `python -m py_compile` + `git diff --check` + `git status --short` (solo propios).

## Gate D — evaluado, NO disparado
Blast radius >10 archivos pero alcance AUTORIZADO explícito en invocación (ARCHIVOS clave listados + plan Wave7) → motivo: alcance pre-autorizado por owner.

## HALLAZGO FIND-94 — decisión explícita (sin scope-creep silencioso)
- **Evidencia:** `rg vanta.VantaDB integrations/` = 12 hits en 7 prod files + `vantadb_shared/__init__.py:47`; SDK 0.5.0 expone `Client/connect`, NO `VantaDB` (`vantadb_py/__init__.py`, `hasattr False`); métodos viejos (`search_memory/list_memory/delete_memory/get_memory/update_memory/delete_namespace`) NO existen en `Client` (pyi:160-230) → alias simple NO viable. Baseline: `pytest integrations/dspy/tests/` = 5 failed + 3 errors `AttributeError: module 'vantadb_py' has no attribute 'VantaDB'`; `ollama/` = 9 errors idénticos.
- **Decisión: NO se absorbe FIND-94.** Migración prod (12 call-sites + cambio de firma/métodos, incluye `dspy/vectorstore.py` PROHIBIDO por FIND-69 DONE `f3d6c634`) excede appetite 1d y viola scope discipline. FIND-94 queda ⬜ PENDIENTE (dueño: orquestador/wave futura).
- **Absorción acotada EXPLÍCITA (dentro del contrato "pytest verde (mocks)"):** shim test-only `FakeVantaDB` en `integrations/conftest.py` (implementa las 8 operaciones viejas in-memory, ~80 líneas) + fixtures migran a `tmp_path`. Cero archivos prod tocados. Documentado como temporal pendiente de FIND-94 en el propio conftest + README central. Si el reviewer lo considera creep → alternativa B (revertir shim, pytest queda bloqueado documentado).

## Spec (decisiones — tabla, Regla "nuevo símbolo público → tabla")
| # | Decisión | Opción elegida | Evidencia / por qué |
|---|----------|----------------|---------------------|
| S1 | Upper bounds | next-major por framework (`dspy<3`, `haystack-ai<3`, `langchain-core<1`, `letta-client<2`, `llama-index-core<1`, `mem0ai<1`, `ollama<1`, `openai<2`; crewai `<2` ya ✅, langgraph `<5` ya ✅) | compat-matrix fija minors (dspy 2.6, haystack 2.10, openai 1.40…); next-major es el techo seguro sin evidencia de breaking |
| S2 | `FakeVantaDB` test-only | 1 clase en `integrations/conftest.py`, NO símbolo público (import solo bajo `pytest`, nombre `_FakeVantaDB`) | evita tabla Spec pública; prod intacto |
| S3 | Fixtures | `tempfile.mkdtemp()` + subdir inexistente → `tmp_path` nativo (auto-cleanup) | langchain checkpointer/store ya usan `tmp_path` (precedente en-repo) |
| S4 | `dist/` | NO commitear (ya gitignored por adapter); documentar + NO borrar discos ajenos (solo documentar limpieza local `rm -rf`) | `git ls-files integrations|grep dist` = 0; `check-ignore` confirma `dist/` ignorado; release workflow lo regenera |
| S5 | PyPI/Alpha | Los 9 = Alpha, no publicados (unificar 4 READMEs que implican `pip install` sin nota) | classifiers `Development Status :: 3 - Alpha` en 9/9; 5 READMEs declaran "Not on PyPI yet"; publish solo vía tag manual `adapters-v*` (sin evidencia de tag en repo, `git tag|grep adapter` = 0); sin red no se verifica registry → [cita registry NO VERIFICADA — sin red] |
| S6 | Drift compat-matrix | NOTICED, no fix: `ollama` compat fija 0.3 < pyproject `>=0.4`; `letta` compat fija `letta 0.2` vs pyproject `letta-client>=1.0.0` (paquetes distintos) | fuera de scope (CI ajeno, posible FIND nuevo del orquestador) |

## Steps
### Step 1: pins + test central (RED→GREEN)
- **Archivos:** 8× `integrations/*/pyproject.toml` (crewai ya ✅, se verifica), `integrations/test_pins.py` (NUEVO, parametrizado ×9)
- **Acción:** RED = test que parsea los 9 pyprojects y exige upper-bound en cada dep de framework; GREEN = añadir los 8 uppers (S1)
- **Verify:** `python -m pytest integrations/test_pins.py -q` + `python -m py_compile` de los pyprojects (tomllib) + `git diff --check`
- **Estado:** ✅ DONE (RED 8 failed/2 passed → GREEN 10 passed; diff-check limpio)

### Step 2: fixtures a tmp_path
- **Archivos:** `tests/test_*.py` de crewai/dspy/haystack/letta/llamaindex/mem0/ollama/openai/langchain-vectorstore (patrón `mkdtemp()` → `tmp_path`)
- **Acción:** migrar fixtures con `tmp_path` (precedente langchain store/checkpointer); sin cambiar asserts
- **Verify:** `python -m py_compile` todos + `pytest --collect-only` por adapter
- **Estado:** ✅ DONE (script one-shot: 10 files, 29 firmas + 29 paths; `rg mkdtemp|import tempfile` = 0; py_compile OK; diff-check limpio)

### Step 3: shim test-only + pytest verde
- **Archivos:** `integrations/vantadb_test_shim.py` (NUEVO, `_FakeVantaDB` + fixture autouse + fallback `dspy.Prediction`), 9× `tests/conftest.py` (+1 import c/u), 9 suites
- **Acción:** fake in-memory con las 8 ops + `created_at_ms/updated_at_ms/version/node_id` + coseno-similitud en vector-query (excluye sin vector) + respeta `filters`/`text_query`; correr las 9 suites con mocks (por separado: `tests.conftest` colisiona en un solo proceso)
- **Verify:** `python -m pytest integrations/<adapter>/tests/ -q` ×9 verdes (skips por SDK ausente = OK documentado)
- **Estado:** ✅ DONE (crewai 11p/2s, dspy 8p, haystack 1s, langchain 47p, letta 17p, llamaindex 1s, mem0 1s, ollama 9p, openai 9p, pins 10p. Hallazgos de sesion: confcutdir=rootdir-adapter obliga shim por-conftest; shadowing `dspy/` requiere fallback Prediction; score=similitud satisface crewai+mem0; `node_id` requerido por langchain)

### Step 4: dist decisión + READMEs + cierre
- **Archivos:** `integrations/README.md` (NUEVO central: matriz pins/PyPI-Alpha/dist), 4 READMEs (haystack/letta/ollama/openai: nota Alpha), commit `fix:`/`docs:` solo propios, review vanta-review (P2-01)
- **Acción:** documentar S4/S5; commit solo archivos propios (prohibidos intactos: `.opencode/`, `completions/`, tauri lock, `Justfile`, ocr-*, `reparacion.bat`, FIND-85/80 files, `pipeline-state.json`, `dspy/vectorstore.py`)
- **Verify:** `git diff --check` + `git status --short` (solo propios) + reviewer approve
- **Estado:** ✅ DONE
- **Reconcile P2-01 (vanta-review, CHANGES-REQUESTED → atendido):**
  - Requerido `twine` sin fuente → VERIFICADO mecánicamente: `twine check` 12/12 PASSED (twine 7.0.0, offline) en los 6 dists locales; haystack/letta/ollama (sin dist/, sin hatchling/red para buildear) con wording honesto sin claim.
  - Crítico score similitud-vs-distancia → NO se cambia el shim: evidencia mixta (engine ordena DESC + `>= min_score` = mayor-mejor; test crewai exige similitud y corrió en CI contra backend real; 3 comentarios dicen distancia). Cambiar el shim rompería el test crewai para satisfacer comentarios; ajustar el test al mock sería test-fitting. Divergencia documentada en shim + README central como HALLAZGO → FIND-94 (dueño con backend real).

## Dependencias
- Wave7 paralela FIND-85/FIND-80 (archivos disjuntos, verificado). Previa Wave6 DONE. Next Wave8.
- Bloqueante parcial: FIND-94 (mitigado con shim test-only; prod queda pendiente).

## Notas
- Ponytail full: pins mínimos next-major, mocks/tmpdir, sin publish forzado (Stop: Alpha documentado es salida válida).
- NO tocar Backlog→avance (race paralelo, orquestador). Push vía vanta-lead.
- `campaign_verify_cmd` bug exit -1 conocido → fallback bash directa documentada.

## Context Save Point
- **Fecha:** 2026-09-15
- **Branch:** develop
- **CI pendiente:** no (solo pytest local + diff checks; CI la corre release-adapters en tag)
- **Decisiones:** S1–S6 arriba; FIND-94 NO absorbido (shim test-only acotado explícito)
- **Problemas conocidos:** WIP ajeno en worktree (prohibidos modificados/untracked — no tocar ni commitear)
- **Próxima tarea:** Wave8 (orquestador decide)
