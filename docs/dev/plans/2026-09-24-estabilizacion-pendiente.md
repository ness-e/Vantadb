# Plan: estabilización pendiente pre-0.7.0 (verificado 2026-09-24)

> Estado base: `develop` en `4b5b137e` (pusheado y verificado en origen), 1954 commits sobre `main`.
> PR #222 (`develop → main`) OPEN + BLOCKED + ROJO (4 fails con causa + nombrados abajo).
> Code-scanning: 83 abiertas (82 test-log + #110 test-only) · secrets 0 abiertas (7 resueltas).
> Backlog técnico: 113 filas (fuera de este plan; ver propuesta de arco pendiente).
> Reglas: solo mergear verde y validado · subagentes caídos (todo directo) · Regla 11 (nada fechado sin evidencia)
> · docs fix-directo · P2-01 review antes de cerrar cada fase.

## Tabla

| ID | Frente | Causa verificada | Dueño |
|----|--------|------------------|-------|
| EST-01 | Lurkr pin SHA inválido | `lurkr-informational.yml:38` usa `setup-python@e797f83…` inexistente (entró con FIND-151); el estándar del repo es `5fda3b9… # v7.0.0` (10 usos) | agente |
| EST-02 | Tests ollama con API eliminada | `providers/ollama/tests/test_ollama.py:143` fixture `vanta.VantaDB(path)` + `get_memory/list_memory/delete_memory` (removidos AST-012); mismo `AttributeError` que tumbó el job benchmark | agente |
| EST-03 | ci-gate "Main is green" en falso rojo | `ci-gate.yml:28-58` consulta check-runs del SHA del merge (`GITHUB_SHA` de `refs/pull/222/merge`) donde esos nombres no existen → todo `<not found>` → fail-closed. No mide `main`; el nombre miente | agente (+ owner decide fix) |
| EST-04 | OSV-Scanner fail 11s | Causa aún sin leer (run en progreso al diagnosticar). Observar resultado y leer log | agente |
| EST-05 | Benchmark + API Docs Version | Ya fixeados en `4b5b137e` (bench a `Client`/`search`, headers `openapi.yaml`/`MCP.md` a 0.7.0). Falta verificar verde en el próximo run del PR | CI |
| EST-06 | #110 dream/mod.rs:952 | Test-only: el test pasa `"salt"` literal a `generate_run_id` (helper determinista con `DefaultHasher`, no cripto; prod usa `salt=""`). Explotabilidad cero → dismiss justificado, no fix | agente |
| EST-07 | 82× cleartext-logging en tests | Todos en `vantadb-mcp/tests/` (`mcp_tests.rs` 81 + `thread_tests.rs` 1), severidad warning. Triage: leer 3 muestras; si son logs de debug de tests → dismiss "test-only", si exfiltran secretos → redactar | agente |
| EST-08 | Python 110/115 archivos | 5 archivos sin escanear, causa desconocida. Bajar SARIF del último análisis y listar omitidos | agente |
| EST-09 | Stale configs CodeQL | Owner ya borró `codeql.yml` + `sec-codeql-30.yml` (verificado: sin uploads nuevos salvo `main`, que aún contiene el archivo viejo). Cierre definitivo = merge PR #222 (el archivo sale de `main`) | owner (hecho) + merge |
| EST-10 | API stale en repo (fuera de CI) | `VantaDB(`/`search_memory` vivos en: 4 bench scripts, `skills/vantadb/SKILL.md` (4×), ~20 docs `user/glosario` + operations, `PILOT_PROGRAM`, `REDDIT_POSTS`, `SHOW_HN_PREP`. Mismo patrón ya limpiado en Notion + `local_bench` | agente |
| EST-11 | Cobertura `vantadb-node` en release | Sin verificar si algún workflow `release-*` publica `vantadb-node` (primer publish; antes 404). Bloquea planear la 0.7.0 | agente |
| EST-12 | Puertas owner pre-0.7.0 | FASE-A (diferida por owner), R-05 por sus manos (opcional), merge PR #222 + tag/publish (solo owner, con bypass como pushes previos) | owner |

## EST-01 — Pin setup-python en lurkr

**Contexto:** job `Lurkr capability scan` muere en 3s en `Set up job`: el SHA `e797f83b…` no existe.
**Tarea:** alinear al pin estándar del repo.
**Acciones:**
1. En `.github/workflows/lurkr-informational.yml:38`: `actions/setup-python@e797f83bcb0c7755b38d450d68b9174fb49f1173 # v5` → `actions/setup-python@5fda3b95a4ea91299a34e894583c3862153e4b97 # v7.0.0`.
2. `git commit + push develop`, verificar job verde en el próximo run del PR.
**Pruebas (contrato):** `Select-String -Pattern 'e797f83' .github/` vacío + check `Lurkr capability scan` en verde.
**Riesgos:** nulo (cambio de pin a valor ya usado en 10 jobs).

## EST-02 — Migrar tests ollama a `Client`

**Contexto:** `test_ollama.py:143` fixture usa clase eliminada; `get_memory/list_memory/delete_memory` removidos (AST-012, guard en `test_subclients.py:325`). El `.pyi` confirma reemplazos 1:1 (`db.memory.get/list/delete`, `Record` con `__getitem__`, `ListResult` con `__len__`).
**Tarea:** migrar fixture + 4 tests.
**Acciones:**
1. `s = vanta.VantaDB(path)` → `s = vanta.Client(path)`.
2. `db.get_memory(` → `db.memory.get(` (2×), `db.delete_memory(` → `db.memory.delete(` (1×), `db.list_memory(` → `db.memory.list(` (1×).
3. `len(page["records"])` → `len(page)` (`ListResult` expone `__len__`; `["records"]` no garantizado).
4. Asserts `record["key"/"payload"/"namespace"/"created_at_ms"/"updated_at_ms"]` se quedan (`Record.__getitem__` existe).
5. `python -m py_compile` + correr `pytest providers/ollama/tests/test_ollama.py` si hay intérprete con `vantadb_py` (si no, marca CI como verificador).
**Pruebas (contrato):** `grep 'VantaDB(|_memory(' providers/ollama/tests/` vacío + job `Check provider (ollama)` verde.
**Riesgos:** `record[...]` depende de campos del `Record` real; si CI falla, ajustar a atributos (`.key`/`.payload`) en vez de revertir.

## EST-03 — ci-gate false-red

> ✅ **RESUELTO 2026-09-24** (commit `0c27a960`): mide `main` HEAD vía API + `skipped/neutral` pass + missing tolerado con WARN (decisión owner A). Verificado `ci-gate / Main is green` = pass en PR #222 (run `36084499760`). Detalle: `docs/dev/tasks/EST-03.md` + `docs/dev/avance/activo/ci-cd.md`.

**Contexto:** el gate no mide `main`: interroga check-runs del SHA efímero del merge. Con fail-closed, cualquier PR sin esos 13 nombres exactos muere en 5s con "main CI is red".
**Tarea:** decidir fix con owner (opciones: consultar el head SHA del PR en vez de `GITHUB_SHA`; o renombrar el job a lo que hace; o lista de checks tolerante a `pending`).
**Acciones:**
1. Proponer diff mínimo al owner antes de tocar (es lógica de gate, no typo).
2. Solo tras aprobación: editar + verificar en PR #222.
**Pruebas (contrato):** job verde en PR con CI por correr (no debe exigir checks inexistentes).
**Riesgos:** tocar gates sin aprobación rompe la invariante "solo mergear lo validado". Por eso es propuesta-primero.

## EST-04 — OSV-Scanner

**Contexto:** fail de 11s sin log leído (run en progreso durante el diagnóstico).
**Tarea:** leer log del run `36040482305` (ya terminado a esta hora), clasificar (config vs vuln real) y actuar.
**Pruebas (contrato):** causa escrita + job verde o fila FIND creada si es vuln real.

## EST-05 — Verificar verde post-`4b5b137e`

**Contexto:** mis fixes (bench + headers) entraron tras los runs rojos leídos.
**Tarea:** ninguna edición; leer el próximo run del PR y confirmar `Check API Docs Version` y `benchmark` en verde.
**Pruebas (contrato):** ambos checks verdes en `gh pr checks 222`.

## EST-06 — Dismiss #110 con justificación

**Contexto:** verificado en código (`mod.rs:723-736` + test `:951-956`): `DefaultHasher` no-criptográfico, `"salt"` literal solo en test, prod default `""`.
**Tarea:** dismiss vía API/UI con motivo `test-only / false-positive` + referencia a este plan. Sin cambio de código.
**Pruebas (contrato):** alerta #110 en `dismissed`; contar abiertas 83 → 82.

## EST-07 — Triage 82 cleartext-logging

**Contexto:** 100% en tests, severidad warning. Hipótesis: logs de debug de tests (dismiss masivo justificado).
**Tarea:** leer 3 muestras (`mcp_tests.rs:1368/1349/4855` — las top del listado UI), confirmar ausencia de secretos reales.
**Acciones:** si limpio → dismiss en batch con motivo único; si hay secreto → redactar + commit.
**Pruebas (contrato):** 0 abiertas de esta regla o commit de redacción + re-scan.

## EST-08 — Python 110/115

**Contexto:** UI CodeQL muestra 110/115; conteo local difiere (141 con otro filtro) → comparar peras con peras.
**Tarea:** bajar SARIF del último `sec-codeql.yml:analyze` y listar los 5 omitidos; clasificar (generado/vendored vs gap real).
**Pruebas (contrato):** lista escrita de los 5 + acción (exclusión documentada o inclusión).

## EST-09 — Cierre stale configs (owner hecho, falta merge)

**Contexto:** owner borró ambas configs (verificado: categorías stale solo aparecen en análisis históricos ≤18:27, previos al borrado). `main` aún contiene `sec-codeql-30.yml` (con schedule semanal) → sus runs re-suben esa categoría.
**Tarea:** ninguna acción hasta el merge; post-merge verificar que no reaparece la entrada stale.
**Pruebas (contrato):** tras merge PR #222, `analyses` últimas 20 solo con categoría `sec-codeql.yml:analyze` + banner apagado (confirma owner).

## EST-10 — Barrido API stale fuera de CI

> ✅ **RESUELTO 2026-09-24** (commit `b461c9e8`): ~230 reemplazos en ~75 archivos vivos + imports canónicos (`vantadb_py`→`vantadb`, 42) + fixes reales (NameError latente en `migrate/*.py`, paths C-02 de `validate_doc_snippets.py`, nota de naming falsa en README, tabla Cross-SDK de `vantadb-ts`, sección `Client` de api-reference, sección `search()` node-level de PYTHON_SDK → híbrida, cajas ASCII re-alineadas). Históricos/MCP-scope intactos (lista de exclusión en el task file). Detalle: `docs/dev/tasks/EST-10.md` + `docs/dev/avance/activo/operaciones.md`.

**Contexto:** ~35 archivos con `VantaDB(`/`search_memory` fuera del path de CI (benches no-CI, skill, glosario, operations, strategy). Mismo rename 1:1 ya aplicado 5× hoy.
**Tarea:** bulk idempotente con auditoría de resolución (conteo antes/después por archivo), `py_compile` a los `.py`, commit separado. Archivos runtime ` SKILL.md` espejados a `.opencode/skills/` tras el cambio (mirror FIND-83).
**Pruebas (contrato):** `grep 'vantadb(_py)?\.VantaDB\(|search_memory|get_memory|list_memory|delete_memory' --include='*.py' --include='*.md'` solo con hits intencionales (notas históricas de migración) + `validate-docs-coverage.ps1` 0 gaps.
**Riesgos:** no tocar `docs/dev/research/archive/`, `plans/archive/`, `tasks/` (histórico congelado) ni ejemplos que documenten el "antes".

## EST-11 — Cobertura publish `vantadb-node`

**Contexto:** primer publish (npm 404 hasta hoy); versión ya en 0.7.0 en develop.
**Tarea:** leer `release-npm-61.yml` (+ `release.yml`) y determinar si cubre `vantadb-node` o solo `vantadb`; escribir el veredicto (workflow vs manual + environment a aprobar).
**Pruebas (contrato):** veredicto escrito con nombres de workflow/líneas; si falta, proponer diff sin aplicarlo (decisión owner: publicar vs diferir node).

## EST-12 — Puertas owner (sin acción agente)

FASE-A (`docs/dev/FASE-A.md`), R-05 manos del owner (opcional), merge #222 + tag `v0.7.0` + publish. El agente solo prepara: PR verde + veredicto EST-11 + checklist de release.

## Orden de ejecución

FASE 1 (CI rojo): EST-01 + EST-02 + EST-04 (commits chicos, push, verificar EST-05) → EST-03 (propuesta) → re-leer PR.
FASE 2 (CodeQL): EST-06 + EST-07 + EST-08 → 0 abiertas reales o FINDs creados.
FASE 3 (higiene): EST-10 + EST-11 (veredicto).
FASE 4 (puertas): EST-09 post-merge + EST-12. P2-01 review al cerrar cada fase.
