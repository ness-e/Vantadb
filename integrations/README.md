# VantaDB integrations (7 adapters + 2 superseded twins)

Python adapters standalone. Cada uno vive en su carpeta con `pyproject.toml`,
tests propios y README. Los 7 adapters publicables se liberan juntos via el
workflow `release-adapters.yml` (tag manual `adapters-v*`); los twins
`openai`/`ollama` quedan **source-only** (superseded por los providers Rust —
ver FIND-273 abajo).

## Estado PyPI: Alpha, no publicados

Los 7 paquetes publicables estan en `Development Status :: 3 - Alpha`
(classifier en cada `pyproject.toml`) y **ninguno esta en PyPI** (verificado
live 2026-10-06: `GET https://pypi.org/pypi/<nombre>/json` = 404 ×7, y lo
mismo en test.pypi.org; `vantadb-py` = 0.8.0). Publicar sin mantenedor por
adapter no es salida valida (Stop del plan) → documentar Alpha SI es la
salida. Instalacion hasta el primer release: `cd integrations/<adapter> &&
pip install .`

> **FIND-273 (resuelto 2026-10-06):** los twins `openai` y `ollama` fueron
> **retirados del release** (decisión owner): sus nombres/módulos PyPI
> pertenecen a los providers Rust canónicos (`vantadb-openai`/`vantadb-ollama`)
> y quedan en repo como **source-only** (sus READMEs lo indican). Ver
> [MKT-20 §F6-2](../docs/dev/tasks/MKT-20.md).

## Pins (FIND-84)

Toda dependencia de framework declara upper-bound next-major. Gate:
`python -m pytest integrations/test_pins.py` (1 test de matriz + 9
parametrizados ×9 dirs; `vantadb-py>=0.6.1,<0.9.0` exenta por traer techo propio).

> Nota (2026-10-06): pin `vantadb-py` bumpeado a `>=0.6.1,<0.9.0` (decisión
> owner) — piso real: 0.5.0 no tiene la API `Client`; el techo permite el
> core live 0.8.0. Evidencia en [MKT-20 §F6-3](../docs/dev/tasks/MKT-20.md).

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
escrituras a disco fuera de tmpdir. Las fixtures crean la DB bajo `tmp_path`
(backend real; sin shim — FIND-94).

## `dist/`: decision (FIND-84)

**NO commitear.** Cada adapter gitignora `dist/` en su `.gitignore`; `git
ls-files integrations | grep dist` = vacio (artefactos locales de builds
manuales, p.ej. `crewai/dist/*.whl`, ya ignorados). El workflow de release
regenera `dist/` en CI y lo sube como artefacto (`path:
integrations/<adapter>/dist/`). Limpieza local si molesta:
`rm -rf integrations/*/dist` (nunca commitear el borrado: no hay nada
trackeado que borrar).

## Suites (FIND-84; migración FIND-94 completada)

`python -m pytest integrations/<adapter>/tests/` **por adapter separado**
(un solo proceso para varios adapters colisiona: cada `tests/` trae
`conftest.py` + `__init__.py` con el mismo nombre de modulo).
FIND-94 (2026-09-16) migró los adapters de `vanta.VantaDB(...)` (API
pre-0.5.0) a `Client`/`memory.*` y eliminó el shim in-memory
(`integrations/vantadb_test_shim.py` ya no existe); las suites corren contra
el backend real (`tmp_path`). Cubren los 9 dirs (7 publicables + 2 twins
source-only).

Estado verificado 2026-09-15 (pre-migración, mocks — histórico): crewai 11 passed/2 skipped,
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
ordena DESC + `>= min_score`). Resuelto por FIND-94 (2026-09-16):
`score`-similitud contra el backend real.
