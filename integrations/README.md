# VantaDB integrations (9 adapters)

Python adapters standalone. Cada uno vive en su carpeta con `pyproject.toml`,
tests propios y README. Ninguno se publica por separado a mano: el release
es el workflow `release-adapters-62.yml` (tag manual `adapters-v*`).

## Estado PyPI: Alpha, no publicados

Los 9 paquetes estan en `Development Status :: 3 - Alpha` (classifier en
cada `pyproject.toml`) y **ninguno esta en PyPI** ([cita registry NO
VERIFICADA — sin red]: sin acceso a red en esta sesion no se pudo consultar
el indice; la evidencia local es la ausencia de tags `adapters-v*` en el
repo + la nota "Not on PyPI yet" en los 9 READMEs). Publicar sin mantenedor
por adapter no es salida valida (Stop del plan) → documentar Alpha SI es la
salida. Instalacion hasta el primer release: `cd integrations/<adapter> &&
pip install .`

## Pins (FIND-84)

Toda dependencia de framework declara upper-bound next-major. Gate:
`python -m pytest integrations/test_pins.py` (10 tests parametrizados ×9
adapters; `vantadb-py>=0.5.0,<0.6.0` exenta por traer techo propio).

| Adapter | Paquete | Framework pin |
|---------|---------|---------------|
| crewai | `vantadb-crewai` | `crewai>=1.14,<2` (ya tenia techo) |
| dspy | `vantadb-dspy` | `dspy>=2.6,<3` |
| haystack | `vantadb-haystack` | `haystack-ai>=2.0,<3` |
| langchain | `vantadb-langchain` | `langchain-core>=0.3,<1`, `langgraph-checkpoint>=2,<5` (ya tenia techo) |
| letta | `vantadb-letta` | `letta-client>=1.0.0,<2` |
| llamaindex | `vantadb-llamaindex` | `llama-index-core>=0.12,<1` |
| mem0 | `vantadb-mem0` | `mem0ai>=0.1.0,<1` |
| ollama | `vantadb-ollama` | `ollama>=0.4,<1` |
| openai | `vantadb-openai` | `openai>=1.0,<2` |

Techos elegidos contra la matriz de compatibilidad
(`.github/workflows/adapters-compat.yml`, que fija minors: dspy 2.6,
haystack 2.10, openai 1.40…): next-major es el techo seguro sin evidencia
de breaking. NOTICED (fuera de scope, no fix): compat fija `ollama 0.3`
< pin `>=0.4`, y `letta 0.2` (paquete `letta`) vs pin `letta-client>=1.0.0`
(paquete distinto).

## Fixtures (FIND-84)

Todas las fixtures usan `tmp_path` (auto-cleanup de pytest): cero
`tempfile.mkdtemp()` sin liberar, cero subdirectorios inexistentes, cero
escrituras a disco fuera de tmpdir. Los adapters ignoran `db_path` bajo
test via el shim (in-memory).

## `dist/`: decision (FIND-84)

**NO commitear.** Cada adapter gitignora `dist/` en su `.gitignore`; `git
ls-files integrations | grep dist` = vacio (artefactos locales de builds
manuales, p.ej. `crewai/dist/*.whl`, ya ignorados). El workflow de release
regenera `dist/` en CI y lo sube como artefacto (`path:
integrations/<adapter>/dist/`). Limpieza local si molesta:
`rm -rf integrations/*/dist` (nunca commitear el borrado: no hay nada
trackeado que borrar).

## Suites + shim temporal (FIND-84, pendiente FIND-94)

`python -m pytest integrations/<adapter>/tests/` **por adapter separado**
(un solo proceso para varios adapters colisiona: cada `tests/` trae
`conftest.py` + `__init__.py` con el mismo nombre de modulo).
Los 9 adapters construyen `vanta.VantaDB(...)` (API pre-0.5.0) pero el SDK
0.5.0 solo expone `Client`/`connect` → todas las suites fallaban con
`AttributeError` (FIND-94, Backlog :236, NO absorbido: migrar 12 call-sites
prod excede appetite y toca `dspy/vectorstore.py` prohibido por FIND-69).
Hasta FIND-94: `integrations/vantadb_test_shim.py` (`_FakeVantaDB`
in-memory + fallback `dspy.Prediction` contra el shadowing del directorio
local) importado por los 9 `tests/conftest.py`. **Borrar shim + 9 imports
cuando FIND-94 migre el prod.**

Estado verificado 2026-09-15 (mocks, sin red): crewai 11 passed/2 skipped,
dspy 8 passed, haystack skipped (sin SDK), langchain 47 passed, letta 17
passed, llamaindex skipped (sin SDK), mem0 skipped (sin SDK), ollama 9
passed, openai 9 passed, pins 10 passed. Skips = `importorskip` por SDK de
framework ausente (salida valida, no rojo).

`twine check` verificado 2026-09-15 sobre `dist/` locales (twine 7.0.0,
offline): 12/12 PASSED en crewai/dspy/langchain/llamaindex/mem0/openai.
haystack/letta/ollama no tienen `dist/` local y no se pudo buildear offline
(sin hatchling, sin red) → sus READMEs no afirman `twine check`.

HALLAZGO → FIND-94 (convencion `score`): el shim emite similitud coseno
(mayor = mejor) porque el test crewai `test_search_returns_scored_match`
(`score > 0.5` en match exacto) lo exige; pero los comentarios de
langchain (`store.py:254`, `vectorstore.py:270,349`), llamaindex
(`vectorstore.py:210,264`) y mem0 (`vectorstore.py:46`) describen el backend
como distancia (menor = mejor). Evidencia de motor no dirimente (engine
ordena DESC + `>= min_score`). FIND-94 debe fijar la convencion verdadera
contra el backend real y ajustar el test crewai o los 3 adapters.
