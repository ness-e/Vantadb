# TASK QW-5: nits agrupados — categorize() eliminada, _normalize_score mem0 documentada, haystack count_documents() cursor paging

## Metadata
- **Plan file:** `docs/plans/2026-08-25-integrations-research-wins.md`
- **Fuente:** Wave 2 QW-5 (H-10 =MOD-50)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** Wave 2 — Limpieza / dedup (nits agrupados)
- **Tipo:** Python
- **Turns estimados:** 5-7
- **Creado:** 2026-08-27T18:00
- **last-synced:** 2026-08-27T18:00
- **Estado:** ✅ COMPLETED
- **Ruta:** vanta-worker
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `integrations/crewai/tests/test_vectorstore.py` (8 tests, ninguno importa categorize — verificado `Get-ChildItem -Recurse` 0 hits); `integrations/crewai/README.md` (no referencia categorize); `integrations/mem0/tests/test_vectorstore.py::test_normalize_score_exact_semantics` (1 test pin semantics); `integrations/haystack/tests/test_vectorstore.py::test_count_documents_many` + `test_count` / `test_delete` / `test_empty_store` (5 tests count path) |
| Callees | `vantadb_py.VantaDB.list_memory` (cursor paginado), `vantadb_py.VantaDB.search_memory` (score cosine distance), `haystack.document_stores.types.DuplicatePolicy`, `mem0.vector_stores.base.VectorStoreBase` (11 abstract methods) |
| Implicaciones | contrato no rompe API pública (thin wrappers); categorize eliminación es breaking solo si consumidores la importaban — era DEPRECATED domain logic no adapter responsibility; _normalize_score doc no cambia runtime salvo `if raw < 0: return 0.0` clamp extra (previo `max(0,1-raw)` ya clamped pero faltaba rama negativa explícita); haystack count_documents pasa de O(1M) materialización a O(n/page_size) cursor pages — sin migración datos ni re-index; tests existentes deben pasar (8 crewai + 20 mem0 + 19 haystack) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `integrations/crewai/vantadb_crewai/vectorstore.py` (257 líneas completas — verifica: `categorize` ausente, `from_dict`/`list` harden QW-1 separado, `VantaDBTool` 257L)
  - `integrations/mem0/vantadb_mem0/vectorstore.py` (307 líneas completas — verifica: `_normalize_score` docstring 14 líneas exact semantics + `if raw < 0` guard)
  - `integrations/haystack/vantadb_haystack/vectorstore.py` (482 líneas completas — verifica: `_COUNT_PAGE_SIZE=1000`, `count_documents` loop cursor `next_cursor`)
  - `integrations/crewai/tests/test_vectorstore.py` (107 líneas)
  - `integrations/mem0/tests/test_vectorstore.py` (168 líneas)
  - `integrations/haystack/tests/test_vectorstore.py` (315 líneas)
  - `docs/plans/2026-08-25-integrations-research-wins.md` (Wave 2 QW-5 contract)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):**
  - `import vantadb_py as vanta` → `vantadb-python` PyO3 (no tocar core `vantadb/src/*` protegido; deprecation warning `vantadb_py`→`vantadb` no bloqueante)
  - `crewai.tools.BaseTool` → fallback object si crewai no instalado (vectorstore.py:9-12)
  - `mem0.vector_stores.base.VectorStoreBase` → 11 abstract methods (OutputData shim)
  - `haystack.dataclasses.Document` + `haystack.document_stores.types.DuplicatePolicy` → DocumentStore protocol
  - `pydantic.PrivateAttr` → `_db` private attr (crewai)
- **Archivos que referencian a los editados (referencias entrantes):**
  - `integrations/crewai/tests` — 8 tests (fixture tool + _put + _run + from_dict roundtrip + cursor chain) — grep `categorize` en tests 0 hits
  - `integrations/mem0/tests` — 20 tests (11 VectorStoreBase + 5 BackwardCompat + 3 OutputData + 1 _normalize_score pin) — grep `categorize` 0 hits
  - `integrations/haystack/tests` — 19 tests (write/filter/count/delete/compound filters/count_documents_many/to_dict/search) — `test_count_documents_many` monkeypatch `_COUNT_PAGE_SIZE=10` con 50 docs ejercita cursor paging
  - `git log 96e143ec` — categorize removal diff `e5857036→e1c853d3` (~65 líneas keyword heuristic) + mem0 doc commit + haystack cursor paging diff
  - `git log 0ae070ca` — QW-5 second patch `if raw < 0: return 0.0` + top_k QW-1 separado
  - Ningún módulo Rust core, web/, u otro adapter depende de estos 3 archivos (grep confirma aislamiento; `Get-ChildItem -Recurse integrations -Filter *.py | Select-String categorize` 0 hits)
- **Veredicto impacto:** bajo — impacto localizado a 3 adapters, seguro para verify. No toca paths multi-índice/dashmap/parking_lot/Tokio → no requiere auditoría concurrencia (Regla 8). No hot path vectorial core → no requiere perf bench (Regla 9). No trust boundary nuevo → no security hardening más allá de thin wrapper validación existente.

## Spec

N/A — cleanup/dedup con contrato mecánico (Wave 2 QW-5). No agrega símbolos públicos nuevos; solo elimina código DEPRECATED, documenta heurística y optimiza conteo por páginas. Tres cambios mecánicos ya pinneados por tests.

Problema:
- `categorize()` en crewai (229-293, ~65 líneas) era lógica de dominio no responsabilidad del adapter — keyword heuristic frágil, DEPRECATED, candidata a eliminación (MOD-50).
- `_normalize_score` mem0 tenía doc breve ("Return a score in [0,1]...") sin semántica exacta distancia→score ni pin de rama negativa; test `test_normalize_score_exact_semantics` requiere doc + clamp explícito.
- `count_documents()` haystack materializaba hasta `_MAX_LIST_LIMIT=1_000_000` records en memoria (`list_memory(limit=1M).records`) — O(1M) riesgo OOM si store crece.

Criterio: `categorize` ausente (`hasattr(..., 'categorize')==False` + grep 0), `_normalize_score` docstring menciona `None`, `[0,1]`, `clamp(1-d,0,1)`, `test_normalize_score_exact_semantics` pasa, `count_documents` itera `while cursor` con `_COUNT_PAGE_SIZE` y `test_count_documents_many` (50 docs, page 10) pasa.

Alcance: `integrations/crewai/vantadb_crewai/vectorstore.py` (categorize removal, commit 96e143ec), `integrations/mem0/vantadb_mem0/vectorstore.py:45-65` (doc + `if raw < 0` guard), `integrations/haystack/vantadb_haystack/vectorstore.py:15+371-394` (_COUNT_PAGE_SIZE + cursor loop).

Decisiones: eliminar categorize (no mover a shared — deletion over addition, ponytail); documentar _normalize_score con semántica exacta vs reemplazar heurística por regla distancia→score (doc elegida — preserva comportamiento + pin test, más segura); cursor paging tamaño 1000 (balance trips vs mem) vs COUNT API server-side (no existe en vantadb_py, paging es única opción client-side).

## Contrato

```
python -m pytest integrations/crewai/tests -q  # 8 passed
python -m pytest integrations/mem0/tests -q  # 20 passed (incl. test_normalize_score_exact_semantics)
python -m pytest integrations/haystack/tests -q  # 19 passed (incl. test_count_documents_many con cursor paging 50 docs page 10)
categorize() eliminada: hasattr(vantadb_crewai.vectorstore,'categorize')==False && grep -r categorize integrations/ 0 hits
_normalize_score documentada: __doc__ contiene "Exact semantics (pinned by" && "clamp(1 - d"
count_documents() cursor paging: _COUNT_PAGE_SIZE=1000 && while cursor: next_cursor loop && test_count_documents_many PASSED
```

Verificación mecánica:
1. `Get-ChildItem -Recurse -Path integrations -Filter "*.py" | Select-String -Pattern "categorize"` → 0 hits ✅ (categorize eliminada en 96e143ec)
2. `python -c "import vantadb_crewai.vectorstore; print(hasattr(vantadb_crewai.vectorstore,'categorize'))"` → False ✅
3. `python -m pytest integrations/mem0/tests/test_vectorstore.py::test_normalize_score_exact_semantics -xvs` → 1 passed ✅ (None→0, [0,1] passthrough, 1.2→0, -0.3→0)
4. `python -c "import sys; sys.path.insert(0,'integrations/mem0'); from vantadb_mem0.vectorstore import _normalize_score; print([_normalize_score(x) for x in [None,0.0,0.3,1.0,1.2,-0.3]])"` → [0.0,0.0,0.3,1.0,0.0,0.0] ✅
5. `python -m pytest integrations/haystack/tests/test_vectorstore.py::test_count_documents_many -xvs` → 1 passed (50 docs, _COUNT_PAGE_SIZE=10) ✅
6. `python -m pytest integrations/crewai/tests integrations/mem0/tests integrations/haystack/tests -q` → 8+20+19=47 passed ✅
7. `grep -n "_COUNT_PAGE_SIZE\|next_cursor" integrations/haystack/vantadb_haystack/vectorstore.py` → page_size 1000 + cursor loop ✅

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `categorize` nunca vuelve (eliminada DEPRECATED, no reintroducir domain logic en adapter); `_normalize_score` semántica pinneada: `None→0`, `[0,1]→passthrough float(raw)`, `d<0→0`, `d≥0→clamp(1-d)` — cambiarla rompe `test_normalize_score_exact_semantics`; `count_documents()` siempre pagina por cursor (`_COUNT_PAGE_SIZE` pages, `next_cursor`) nunca materializa `_MAX_LIST_LIMIT`; adapters siguen thin wrappers (no tocar `vantadb/src/*`); suite 47 tests (8+20+19) pasa.
- **Comandos de verificación:** `python -m pytest integrations/crewai/tests integrations/mem0/tests integrations/haystack/tests -q` → 47 passed; `python -m pytest integrations/mem0/tests/test_vectorstore.py::test_normalize_score_exact_semantics -xvs` → 1 passed; `python -m pytest integrations/haystack/tests/test_vectorstore.py::test_count_documents_many -xvs` → 1 passed; `Get-ChildItem -Recurse -Path integrations -Filter "*.py" | Select-String -Pattern "categorize"` → 0 hits
- **Deuda pendiente:** ninguna — fix ya en HEAD (96e143ec + 0ae070ca), esta tarea re-verifica. Si hay follow-up: migrar `import vantadb_py` → `import vantadb` (deprecation warning, no bloqueante, 3 adapters); considerar COUNT server-side API si vantadb_py expone count (hoy no, paging es workaround client-side).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva. Fix reduce deuda: elimina ~65 líneas DEPRECATED categorize (code-simplification) + documenta heurística críptica + evita O(1M) materialización. No se toca `vantadb/src/*`. Deuda `vantadb_py` deprecation warning permanece pero es externa no introducida aquí.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable cumple: categorize eliminada (0 hits) + _normalize_score doc + clamp + haystack cursor paging + 47 tests pass |
| **Commit** | No aplica en esta iteración (verify-only, fix ya commit 96e143ec + 0ae070ca) — se documenta handoff sin nuevo commit por regla "no commit" del prompt |
| **Release** | No aplica (adapter Python puro, no requiere publish en esta wave — QW-7 separado) |

Gate: task se marca COMPLETED si contrato task pasa + capa determinista (pytest 47 passed) pasa.

## Herramientas necesarias

- **source-driven-development:** verificar docs oficiales Haystack DocumentStore protocol (count_documents), mem0 VectorStoreBase, y vantadb_py list_memory cursor semantics
- **ponytail (full):** ladder mínimo — deletion over addition (categorize ~65L eliminadas), stdlib clamp `min/max`, no añadir deps, thin wrapper paging
- **code-simplification:** reducir complejidad sin cambiar comportamiento — categorize removal es simplificación canónica (menos código, menos deuda)
- **SKILLS_CARGADAS (SDP):** source-driven-development, ponytail, code-simplification (≤3 por encargo, ≤8 total)

Lifecycle mapping BUILD (source-driven, ponytail, code-simplification) + VERIFY (pytest contrato)

Grep SKILLS-MANIFEST.md por `categorize|_normalize_score|haystack|cursor|count_documents|simplification` → `code-simplification` (7/10) match directo "reduce complexity" para categorize removal; `source-driven-development` (8/10) match verify docs before implementing; `ponytail` full ya cargada por cargo. `systematic-debugging` no aplica (nits, no bug runtime). `test-driven-development` ya implícito pero no cargada extra (tests existen pin semantics). Discovery ≤8 → 3 cargadas, justificadas arriba. SDP: sin candidatos adicionales más allá de `vantadb` genérica (rating 8, no añade patrón específico al fix).

## Investigation Notes

- STACK DETECTED (source-driven-development Step 1):
  - `vantadb-py 0.5.0` (integrations/*/pyproject.toml `vantadb-py>=0.5.0,<0.6.0`)
  - `crewai` (optional, fallback object si no instalado)
  - `mem0ai` (VectorStoreBase 11 methods)
  - `haystack-ai` (DocumentStore protocol, Document dataclass, DuplicatePolicy)
  → No fetch externo requerido: contrato verificado via grep + pytest local + git log diff; docs autoritativas ya en repo (tests pin semantics).

- INVESTIGATION QW-5:
  - `git show 96e143ec^:integrations/crewai/vantadb_crewai/vectorstore.py | Select-String -Pattern "def categorize"` → categorize existed pre-96e143ec with DEPRECATED comment (65 líneas keyword heuristic question/technical/greeting)
  - `git show 96e143ec -- integrations/crewai/... | Out-String` → diff shows `-def categorize(text: str) -> str:` + 60 líneas removed, no new code added (deletion over addition)
  - `git show 96e143ec -- integrations/mem0/...` + `0ae070ca` → docstring expanded + `if raw < 0: return 0.0` guard added (previo `max(0,1-raw)` ya handled positive distances but explicit negative branch improves readability + pins test)
  - `git show 96e143ec -- integrations/haystack/...` → `_COUNT_PAGE_SIZE=1000` added + `count_documents` rewritten from `len(list_memory(limit=1M).records)` to `while True: page = list_memory(limit=PAGE,cursor); count+=len(page.records); cursor=page.next_cursor`
  - Current disk verify: `Get-ChildItem -Recurse integrations -Filter *.py | Select-String categorize` → 0 hits (eliminada); `_normalize_score.__doc__` contains "Exact semantics (pinned by"; `_COUNT_PAGE_SIZE` 1000 + `next_cursor` loop present; pytest 8+20+19=47 passed (11.79s+10.39s+21.16s)

- PONYTAIL ladder:
  - categorize: rung 1 (Does this need to exist at all?) → NO, domain logic no adapter responsibility → delete, skipped: mover a shared module / deprecate wrapper
  - _normalize_score: rung 3 (stdlib does it? `min`/`max` clamp) + rung 6 (one line doc) → minimal diff, skipped: reemplazar heurística por fórmula distancia→score con cambio behavior (riesgo regresión)
  - count_documents: rung 3 (reuse existing `list_memory(cursor)` pagination, pattern já en crewai/dspy) → no new dependency, skipped: server-side COUNT API (no existe en vantadb_py)

- CODE-SIMPLIFICATION: categorize removal es simplificación canónica — menos código, menos branches (question/technical/greeting keyword sets), menos tests de dominio frágiles. Verificación: code-simplification skill dice "reducir complejidad sin cambiar comportamiento" — aquí se elimina comportamiento DEPRECATED no usado, verificado por `test_tool_run` / `test_tool_empty` aún pasan.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — approach validado contra git history + grep + pytest, no hay decisiones abiertas |
| Pendientes de ejecución (downhill) | 0 tras verify (4 steps) |
| % completado | 100% (verify-only, fix preexistente) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — No toca trust boundaries (no input usuario sin validar extra, no auth, no FFI nuevo, no deps nuevas). Tres adapters son thin wrappers sobre vantadb_py ya validado. Checklist `security-and-hardening` no aplica más allá de `delete_memory` / `put` ya existentes. Justificación: categorize era pure keyword matching sin IO; _normalize_score es pure function; count_documents solo lectura paginada.
- [x] **PERFORMANCE** — No toca hot path core (vector/index/text_index/engine). Haystack count_documents mejora de O(1M) materialización a O(n/page) cursor pages — mejora memoria (no más 1M records en RAM), latencia extra por N trips pero page 1000 mantiene overhead bajo (50 docs = 5 pages en test monkeypatch 10). No requiere baseline bench (Regla 9 no aplica — no es optimization claim, es OOM avoidance). Justificación: cambio paging evita OOM, no hot path search/ingestión.

## Steps

### Step 1: DISCOVERY — blast radius + Regla 0 + source-driven detect
- **Archivos:** `integrations/crewai/vantadb_crewai/vectorstore.py`, `integrations/mem0/vantadb_mem0/vectorstore.py`, `integrations/haystack/vantadb_haystack/vectorstore.py`, `integrations/*/tests/test_vectorstore.py`
- **Acción:** Confirmar fixes QW-5 (96e143ec + 0ae070ca) ya en disco: categorize ausente (grep 0), _normalize_score doc + guard `if raw < 0`, haystack _COUNT_PAGE_SIZE + cursor loop. Mapear Regla 0 completa arriba. Validar stack versions (vantadb-py 0.5.0, crewai/mem0/haystack optional). Git log diff para evidencia histórica.
- **Verify:** `Get-ChildItem -Recurse integrations -Filter *.py | Select-String -Pattern "categorize"` → 0 hits; `Select-String -Path integrations/mem0/vantadb_mem0/vectorstore.py -Pattern "_normalize_score" -Context 2,10` → doc presente; `Select-String -Path integrations/haystack/vantadb_haystack/vectorstore.py -Pattern "_COUNT_PAGE_SIZE|next_cursor"` → page 1000 + loop; `git show 96e143ec --stat` → QW-5 files
- **Estado:** ✅ DONE (2026-08-27 — ya en HEAD; categorize eliminada verificado 0 hits, _normalize_score doc verificado, haystack paging verificado, Regla 0 mapeada)

### Step 2: ACT — verify fix presente, no edit necesario (harden si hace falta)
- **Archivos:** `integrations/crewai/vantadb_crewai/vectorstore.py` (categorize check), `integrations/mem0/vantadb_mem0/vectorstore.py:45-65` (_normalize_score), `integrations/haystack/vantadb_haystack/vectorstore.py:15,371-394` (count_documents)
- **Acción:** Revisar crewai vectorstore.py no contiene `def categorize` (ya eliminada commit 96e143ec) — si contuviera, eliminar ~65L + DEPRECATED comment (ponytail deletion). Revisar mem0 _normalize_score doc menciona "Exact semantics (pinned by test_normalize_score_exact_semantics)" y guarda `if raw < 0: return 0.0` — si faltara, añadir. Revisar haystack count_documents loop cursor `page.next_cursor` + `_COUNT_PAGE_SIZE` — si faltara, añadir paging (pattern crewai list cursor). Si todo verde, no editar (ponytail: shortest diff wins — no tocar código que ya pasa contrato). Trab-tree diff de crewai pertenece a QW-1 harden, no a QW-5 — no confundir.
- **Verify:** `python -c "import vantadb_crewai.vectorstore; print(hasattr(...))"` → False; `python -c "from vantadb_mem0.vectorstore import _normalize_score; help(_normalize_score)"` → doc; `grep -n "_COUNT_PAGE_SIZE" integrations/haystack/...` → 1000; no edit requerido si todo presente
- **Estado:** ✅ DONE (2026-08-27 — categorize False, _normalize_score doc presente con clamp, haystack paging presente, no edit requerido — verify-only)

### Step 3: VERIFY — suite contract + edge exhaustivo
- **Archivos:** `integrations/crewai/tests/test_vectorstore.py`, `integrations/mem0/tests/test_vectorstore.py`, `integrations/haystack/tests/test_vectorstore.py`
- **Acción:** Ejecutar full suites + casos borde específicos del contrato: crewai 8 tests (fixture tool + roundtrip from_dict + cursor chain), mem0 `test_normalize_score_exact_semantics` (None→0, [0,1] passthrough, 1.2→0, -0.3→0), haystack `test_count_documents_many` (50 docs, page 10, monkeypatch verifies N iterations), haystack `test_count`/`test_empty_store`/`test_delete` count paths. Verificar inline python edge _normalize_score + categorize absent + _COUNT_PAGE_SIZE.
- **Verify:** `python -m pytest integrations/crewai/tests -q` → 8 passed (11.79s) ✅; `python -m pytest integrations/mem0/tests -q` → 20 passed (10.39s) ✅; `python -m pytest integrations/haystack/tests -q` → 19 passed (21.16s) ✅; `python -m pytest integrations/mem0/tests/test_vectorstore.py::test_normalize_score_exact_semantics -xvs` → 1 passed ✅; `python -m pytest integrations/haystack/tests/test_vectorstore.py::test_count_documents_many -xvs` → 1 passed ✅; total 47 passed
- **Estado:** ✅ DONE (2026-08-27 — 47 passed verificado, disk clean, deprecation warnings solo vantadb_py alias)

### Step 4: CIERRE — verify full + recitation + handoff (no commit per prompt)
- **Archivos:** `.opencode/skills/campaign-executor/tasks/QW-5.md`, `docs/plans/2026-08-25-integrations-research-wins.md`
- **Acción:** Actualizar task file estado a ✅ COMPLETED, sync recitation; no commit (prompt dice no commit); producir bloque RESULTADO; handoff a QW-6
- **Verify:** task file steps ✅ 4/4, `git diff HEAD -- integrations/mem0 integrations/haystack` vacío (fix ya en 96e143ec), crewai diff pertenece a QW-1; plan file recitation QW-5 sync si aplica (wave 2 QW-5); RESULTADO bloque parseable con GATES_EVALUADOS y SKILLS_CARGADAS
- **Estado:** ✅ DONE (2026-08-27)

## Dependencias
- Ninguna directa (Wave 2 QW-5 independiente; QW-4 dedup ya en HEAD 96e143ec comparte commit pero no bloquea — ambos fixes ya merged)
- QW-1..QW-4 ya verificados (QW-1 develop diff pendiente commit lead, QW-2/QW-3/QW-4 en HEAD)

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review (leaf, task deny) — verificación mecánica, no implementa
- **Enfoque:** approach categorize deletion (no mover a shared) es correcto vs alternativa mantener DEPRECATED con warning — deletion es ponytail + code-simplification canónica (less code wins, ~65L menos). _normalize_score doc vs reemplazar heurística: doc elegida preserva behavior + pin test (más segura, sin riesgo regresión search scores). Haystack cursor paging vs server COUNT: paging es única opción client-side hoy (vantadb_py no expone count), page 1000 balanceada. Alternativa migración a `count_documents` server API requeriría core Rust change — fuera de scope adapter.
- **Cómo se probó:** evidencia mecánica real (no auto-reporte): `Get-ChildItem -Recurse integrations -Filter *.py | Select-String categorize` 0 hits, `python -c hasattr(...categorize)` False, `typing.get_type_hints` no aplica aquí pero `_normalize_score.__doc__` verificado inline, `python -m pytest integrations/crewai/tests -q` 8 passed (11.79s), `integrations/mem0/tests -q` 20 passed (10.39s) con `test_normalize_score_exact_semantics` 1 passed, `integrations/haystack/tests -q` 19 passed (21.16s) con `test_count_documents_many` 1 passed (50 docs, _COUNT_PAGE_SIZE=10), `git show 96e143ec` diff categorize removal + paging verificados.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ✅ approve — contrato cumple, no se requieren cambios

## Notas

- Fix ya merged en 96e143ec (categorize removal, haystack paging, mem0 doc base) + 0ae070ca (mem0 `if raw < 0` guard) + 2754c783? providers plan no relacionado. Esta tarea pipeline-full re-verifica contrato y documenta evidencia source-driven + code-simplification; cierra como verify-only con evidencia mecánica (ponytail: no editar código que ya pasa).
- Crewai working-tree diff (list cursor empty→None+ValueError + from_dict embedding callable + next_cursor `is not None`) pertenece a QW-1 harden (develop sin commitear), no a QW-5 — QW-5 git diff para mem0/haystack es 0 (ya en HEAD). No tocar ese diff en este commit QW-5.
- `import vantadb_py` deprecation warning (use `import vantadb`) presente en 3 adapters — no bloqueante, deuda menor para QW follow-up batch (no añadir deuda nueva en este PR).
- Disk incident 2026-08-27 lessons.md QW-1 etc. no afecta QW-5 (suites pasan sin StorageFull tras limpieza).
- No commit en esta iteración por instrucción prompt ("no commit, RESULTADO") — task file queda untracked para handoff; el orquestador decide commit (lead).

## Context Save Point
- **Fecha:** 2026-08-27T18:00
- **Branch:** develop
- **CI pendiente:** no — local pytest 47 passed ya verificado (8 crewai 11.79s + 20 mem0 10.39s + 19 haystack 21.16s)
- **Decisiones:** mantener fix actual (categorize eliminada, _normalize_score doc+clamp, haystack cursor paging _COUNT_PAGE_SIZE=1000), no añadir factory/default extra; ponytail ladder: deletion > stdlib > minimal diff; code-simplification: categorize removal es menos código, menos deuda
- **Problemas conocidos:** ninguno — contrato verde
- **Próxima tarea:** QW-6 (decisión letta)

