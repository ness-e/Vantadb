# FIND-114: Migrar `examples/python/agent_memory.py` de import legacy `vantadb_py.VantaDB` a `Client`

## Metadata
- **Plan file:** docs/dev/plans/2026-09-17-seguimiento-mvp.md (Wave2, Task 8)
- **Fuente:** plan file Wave2 Task 8 + Gate Justificación (ejemplo legacy confunde; migración mecánica al patrón SHOW-04)
- **Esfuerzo:** 🟢 1h (max 1h appetite)
- **Prioridad:** 🟢 Baja
- **Tipo:** Python (ejemplo; refactor mecánico 1:1, sin lógica nueva)
- **Turns estimados:** 5-10
- **Creado:** 2026-09-18
- **last-synced:** 2026-09-18
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0

## 1. TAREA
- **Objetivo:** migrar `examples/python/agent_memory.py` del API legacy (`import vantadb_py as vantadb` + `vantadb.VantaDB(...)`) al patrón canónico `Client` (SHOW-04: `from vantadb import Client` como en `examples/agent_memory_cli/agent_memory_demo.py` y `vantadb-python/tests/test_sdk.py` con `vanta.Client`).
- **Motivo:** el ejemplo usa API legacy y confunde (dos APIs visibles); `VantaDB` ya no existe como clase (solo `Client` se registra en `vantadb-python/src/lib.rs:2395`).
- **Contrato:** ejemplo usa `Client` + corre verde (pytest del ejemplo si existe, o smoke del script).
- **AC:**
  1. El ejemplo importa `Client` (patrón SHOW-04), cero referencias a `VantaDB`/`search_memory` legacy.
  2. Comportamiento idéntico (mismos namespaces, keys, payloads, vectores, prints, métricas, flush/close, cleanup).
  3. Smoke del script verde (o pytest verde si existiera test del ejemplo).

## 2. ARCHIVOS
- **Clave:** `examples/python/agent_memory.py:5,13` (import + constructor legacy).
- **Relacionados (solo lectura):**
  - `vantadb-python/tests/test_sdk.py` (patrón `vanta.Client`, `db.put`, `db.search`).
  - `examples/agent_memory_cli/agent_memory_demo.py` (patrón SHOW-04: `from vantadb import Client`, `put`/`memory.get`/`flush`/`close`).
  - `vantadb-python/src/lib.rs` (lectura: `Client::put:1018`, `Client::search:1243`, `operational_metrics:1553`, `flush:1769`, `close:1886`, registro módulo `2395`).
  - `vantadb-python/src/types.rs` (lectura: `SearchHit.key/payload/metadata/score` verificados).
  - `vantadb-python/vantadb/__init__.py` + `vantadb_py/__init__.py` (lectura: import canónico vs legacy deprecado PY-03).
- **PROHIBIDOS (no tocar):** cambiar comportamiento del ejemplo; resto de `examples/` (otros ejemplos legacy como `demo.py`, `autogen_memory.py`, `smoke_test_extended.py` quedan intactos — scope discipline); `vantadb-python/src/` (bindings — solo lectura); `reparacion.bat`; `.opencode`; `Justfile`; `ocr-*`; `completions/*`; `desktop/src-tauri/Cargo.lock`; `stash@{0}` GOV-C4; `docs/dev/Backlog.md`; plan file (solo recitation); `C:/Users/Eros/.vantadb*`.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | ninguno (ejemplo standalone; grep `agent_memory.py` solo lo referencia docs/README, sin imports) |
| Callees | `vantadb.Client` (`put`/`search`/`operational_metrics`/`flush`/`close`), `SearchHit` getters |
| Implicaciones | contrato público no cambia; comportamiento idéntico; sin impacto performance/memoria/serialización; sin migración de datos; ningún test existente afectado |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `examples/python/agent_memory.py` (50L); `examples/agent_memory_cli/agent_memory_demo.py` (94L); `vantadb-python/vantadb/__init__.py` (28L); `vantadb-python/vantadb_py/__init__.py` (parcial, header+imports); `vantadb-python/src/lib.rs` (secciones Client:52-171, put:1018-1044, search:1195-1290, registro:2391-2416); `vantadb-python/src/types.rs` (Record getters 59-166, SearchHit getters 285-360); `vantadb-python/tests/test_sdk.py` (1-120); `vantadb-python/smoke_test_extended.py` (69L, stale — confirma legacy roto); `.opencode/rules/python-bindings.md`; `.opencode/references/definition-of-done.md`.
- **Archivos referenciados hacia dentro (imports/dependencias del ejemplo):** `vantadb_py` (legacy, deprecado PY-03 con DeprecationWarning) → tras migración `vantadb.Client`; stdlib `os`, `shutil` (intactos).
- **Archivos que referencian al editado (referencias entrantes):** grep `agent_memory.py` en repo: solo menciones docs (ningún import/dependencia de código). `codegraph_explore` confirma: sin callers de código (solo ruido web/EXAMPLES).
- **Veredicto impacto:** BAJO — 1 archivo ejemplo standalone, migración mecánica renombre; si el editado desapareciera, nada de código se rompe (solo docs lo mencionan).

## Contrato
"examples/python/agent_memory.py importa `Client` (cero `VantaDB`/`search_memory`), comportamiento idéntico, y smoke `python examples/python/agent_memory.py` exit 0 (o pytest verde si aplica)"

## Spec (SDD)
No aplica: refactor mecánico de ejemplo, cero símbolos públicos nuevos (Phase 1b: ninguna señal — no se agrega `pub fn`, tool, endpoint ni método de binding). Migración 1:1 verificada en DISCOVERY.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** comportamiento del ejemplo idéntico (namespaces/keys/payloads/vectores/prints/métricas/cleanup); no tocar resto de `examples/` ni `vantadb-python/src/`; WIP ajeno intocable; secrets nunca a disco.
- **Comandos de verificación:** `python examples/python/agent_memory.py` (smoke exit 0); `python -m pytest vantadb-python/tests/test_sdk.py -x -q` (opcional, contrato no lo exige); `campaign_verify_cmd` para gates si aplica (ejemplo .py — clippy/nextest no aplican; bug exit -1 → bash directa).
- **Deuda pendiente:** ninguna (otros ejemplos legacy quedan como NOTICED BUT NOT TOUCHING).

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | Valor |
|------------------------|-------|
| `activeGoal` | FIND-114 migrar agent_memory.py a Client |
| `lastAction` | Step 1 DISCOVERY: task file creado + mapeo 1:1 legacy→Client verificado |
| `result` | PARTIAL (Steps 2-3 pendientes) |
| `nextAction` | Step 2 ACT: editar agent_memory.py (import+ctor+search) + smoke |
| `contract` | verificacion: task file creado; evidencia: claim mapeo 1:1 verificado en código real / evidencia lib.rs:2395+1243+1018, types.rs:298-329, test_subclients.py:316-335 / confianza alta; artefactos: docs/dev/tasks/FIND-114.md; invariantes: comportamiento idéntico, resto examples intacto; deuda: Steps 2-3 pendientes; queda_pendiente: edición + smoke + commit + FIND-115 |
| `nextTask` | FIND-115 (orquestador) |

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda — refactor mecánico sin atajos; otros ejemplos legacy (demo.py, autogen/crewai/haystack/langgraph/mem0, smoke_test_extended.py, README vantadb-python con `search_memory`) NOTICED BUT NOT TOUCHING (fuera de scope; ¿crear task? → orquestador decide).

## Definition of Done (contrato multi-nivel — P2-08)
| Nivel | Gate |
|-------|------|
| **Task** | AC 1-3 ✅ + smoke verde + `git diff` solo `examples/python/agent_memory.py` |
| **Commit** | commit atómico `refactor: FIND-114 — ...`, staging selectivo, NO PUSH |
| **Release** | N/A (ejemplo; sin cambio user-visible versionable ni changelog) |

## Herramientas necesarias
- system python o venv auditoría (`python -m pytest` / smoke del script)
- `campaign_verify_cmd` (bug exit -1 → bash directa)
- codegraph (ubicar `Client` si hace falta — ya usado)

**Skills cargadas (SDP):** `SDP: incremental-implementation` (slice único mecánico: Implement→Test→Verify→Commit) · `systematic-debugging` (mapeo 1:1 + Prove-It si semántica diverge; sugerida por plan) · base sesión (campaign-executor, progreso, ponytail full). Keywords: migrar/ejemplo/Client/legacy/smoke/pytest. `frontend-ui-engineering`/`doubt-driven-development` del scoring descartadas (sin UI, sin trust boundary).

## Investigation Notes
### INVESTIGACIÓN CÓDIGO (DISCOVERY — mapeo 1:1 legacy→Client antes de cambiar)
| # | Legacy (`agent_memory.py`) | Canónico (`Client`) | Evidencia | Veredicto |
|---|---------------------------|---------------------|-----------|-----------|
| 1 | `import vantadb_py as vantadb` | `from vantadb import Client` (patrón SHOW-04) | `vantadb/__init__.py:1-16` (canónico), `vantadb_py/__init__.py:1-21` (legacy deprecado PY-03, warning), `agent_memory_demo.py:27` | 1:1 ✅ |
| 2 | `vantadb.VantaDB(DB_PATH, memory_limit_bytes=...)` | `Client(DB_PATH, memory_limit_bytes=...)` | `lib.rs:2395` solo registra `Client` (no existe clase `VantaDB`); `lib.rs:570-586` ctor `Client(db_path, ...)`; `demo.py:90` y `smoke_test_extended.py:16` confirman `VantaDB` era el nombre viejo | 1:1 ✅ |
| 3 | `db.put(ns, key, payload, metadata=..., vector=...)` | idéntico (shared-name AST-012) | `lib.rs:1018-1044` misma firma; `agent_memory_demo.py:49` mismo uso | sin cambio ✅ |
| 4 | `db.search_memory(ns, query_vector=..., text_query=..., top_k=...)` | `db.search(ns, query_vector, text_query=..., top_k=...)` | `lib.rs:1241-1255` `search(namespace, query_vector, filters=None, text_query=None, top_k=10, ...)`; `test_subclients.py:316-335` guard AST-012: `search_memory` ELIMINADO (assert not hasattr); README `vantadb-python/README.md:70` stale (menciona `search_memory` — NOT touching) | rename 1:1 ✅ (kwargs `query_vector=`/`text_query=`/`top_k=` válidos por firma PyO3) |
| 5 | `hit.key / hit.score / hit.payload / dict(hit.metadata)` | idéntico | `types.rs:298-329` SearchHit expone `key/payload/metadata/score/node_id`; `metadata` devuelve `PyDict` → `dict()` válido | sin cambio ✅ |
| 6 | `db.operational_metrics()/flush()/close()` | idéntico | `lib.rs:1553,1769,1886` existen en `Client` | sin cambio ✅ |
### INVESTIGACIÓN PROBLEMA
¿Semántica legacy sin equivalente? NO — todo el API usado tiene equivalente 1:1 en `Client`. Stop rule (semántica no-mapeable en 1h → DEFER) no dispara. Appetite intacto.
### INVESTIGACIÓN INTERNET
N/A (todo verificado en código local; sin APIs externas).
### Pre-mortem (plan) verificado
(1) semántica distinta → mapeada 1:1 arriba, sin divergencia; (2) sin test del ejemplo → smoke manual del script + nota en RESULTADO.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 2 (Step 2 edición+smoke, Step 3 commit+cierre) |
| % completado | 30% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — no aplica: ejemplo local sin trust boundary (sin input usuario, auth, deps nuevas, red, FFI propio). Sin `security-and-hardening`.
- [x] **PERFORMANCE** — no aplica: sin hot path (ejemplo 3 vectores 3-d demo). Sin `performance-optimization`, sin baseline.

## Steps
### Step 1: DISCOVERY (mapeo 1:1 + task file)
- **Archivos:** `docs/dev/tasks/FIND-114.md` (este file)
- **Acción:** mapear legacy→Client en código real + crear task file con las 10 secciones
- **Verify:** task file existe con Impacto Regla 0 poblado
- **Estado:** ✅ COMPLETED

### Step 2: ACT (migración mecánica + smoke)
- **Archivos:** `examples/python/agent_memory.py`
- **Acción:** (a) import → `from vantadb import Client`; (b) ctor → `Client(...)`; (c) `search_memory` → `search`; resto intacto. Luego smoke `python examples/python/agent_memory.py` (exit 0). Si `vantadb` no importable en este env → diagnosticar (¿binding compilado? ¿venv auditoría?) sin instalar nada fuera de scope; fallback: verificación estática del mapeo + nota.
- **Verify:** `grep -n "VantaDB\|search_memory\|vantadb_py" examples/python/agent_memory.py` vacío + smoke exit 0
- **Estado:** ✅ COMPLETED (3 líneas: import→`from vantadb import Client`, ctor→`Client(...)`, `search_memory`→`search`; resto idéntico; grep cero-legacy salvo docstring producto línea 2; smoke `PYTHONUTF8=1 python examples/python/agent_memory.py` exit 0 con 3 hits + métricas + cleanup)

### Step 3: VERIFY + commit + cierre
- **Archivos:** `docs/dev/tasks/FIND-114.md` (sync), plan file (recitation FIND-114)
- **Acción:** DoD 3 niveles + OCR delegation (`pwsh dev-tools/ocr-review.ps1`) + commit `refactor: FIND-114 — ...` (staging selectivo, NO PUSH) + recitation + RESULTADO §7
- **Verify:** `git log --oneline -1` + `git status --short` limpio en ejemplo
- **Estado:** ✅ COMPLETED (py_compile OK vía bash directa — `campaign_verify_cmd` bug exit -1 documentado en plan; OCR preview sin Critical/High en `agent_memory.py` — cambio mecánico 3 líneas, grupo python sin hallazgos; resto grupos = WIP ajeno no tocado)

## Dependencias
- FIND-102 ✅ (Wave2 primera; check tests verde — completada, recitation plan).
- Stop: semántica no-mapeable en 1h → DEFER con diagnóstico (no disparado).
- NextTask: FIND-115 (orquestador; sync one-liner README_ES + docs-view).

## 3. DEPENDENCIAS (resumen)
Wave2 segunda en secuencia. NextTask: FIND-115.

## 4. REFERENCIAS
- Rules (lectura completa ✅): `.opencode/rules/python-bindings.md` (R-1 batch/GIL, R-2 closures — no aplican a ejemplo, sin batch nuevo; verificado).
- Refs: `.opencode/references/definition-of-done.md` (DoD 3 niveles arriba); commands `pipeline.md` (este run); SPEC.md raíz (N/A — refactor ejemplo, tabla Spec N/A).
- Workflow: `refactor` (audit→migrate→cleanup→verify→review→accept→close); este task: audit ✅ (Step 1) → migrate (Step 2) → verify+close (Step 3); cleanup N/A (sin dead code).

## 5. SKILLS
Base en sesión (campaign-executor, progreso, ponytail full) + Sugeridas plan (incremental-implementation ✅ cargada, systematic-debugging ✅ cargada). SDP real Paso 0b ejecutado (`campaign_discover_skills_v2` phase BUILD): 8 candidatos, 2 aplicables cargadas, resto descartado con motivo. `SKILLS_CARGADAS:` en RESULTADO §7.

## 6. HERRAMIENTAS+MCP
system python o venv auditoría + `campaign_verify_cmd` (bug exit -1 → bash directa). codegraph usado en DISCOVERY. Internet N/A.

## 7. INVESTIGACIÓN CÓDIGO
Ver Investigation Notes (tabla 6 filas). En DISCOVERY ✅.

## 8. INVESTIGACIÓN PROBLEMA
Ver Investigation Notes. Sin semántica huérfana ✅.

## 9. INVESTIGACIÓN INTERNET
N/A.

## 10. VALIDACIÓN+CIERRE
Verify contrato (Step 3) + OCR delegation + DoD 3 niveles + P2-01 orquestador (review lo hace el orquestador/lead al recibir RESULTADO — como worker dejo evidencia + diff mínimo) + Gates D/V/C vía `question` (D: no dispara — 1 archivo ejemplo; V: solo si smoke falla 2× mismo error; C: colaterales = otros ejemplos legacy → propongo FIND nuevo, no toco) + RESULTADO §7.

## Review (GATE — agente distinto, P2-01)
- **Revisor:** orquestador/lead (P2-01; worker no se auto-aprueba — pendiente veredicto del orquestador)
- **Enfoque:** ¿migración 1:1 fiel? Tabla 6 filas en Investigation Notes dice sí; diff 3 líneas, sin lógica tocada.
- **Cómo se probó:** smoke real `PYTHONUTF8=1 python examples/python/agent_memory.py` exit 0 (3 hits ctx-002/001/003 + RSS 27.63MB + HNSW 3 nodos + cleanup) en `C:/Users/Eros/AppData/Local/Temp/opencode` (DB temporal fuera del repo, sin residuos); `py_compile` OK; grep cero-legacy (solo docstring producto L2).
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos (outputs reales pegados arriba).
  - [x] No saltarse clarificación (mapeo verificado en código antes de editar).
  - [x] No declarar done sin verificar AC (AC 1-3 con evidencia).
  - [x] No ignorar fallos (UnicodeEncodeError inicial diagnosticado como codepage consola, re-run UTF-8 verde; sin tocar el ejemplo).
  - [x] No un solo intento de búsqueda (codegraph + grep + lectura directa lib.rs/types.rs/tests).
  - [x] No copiar sin citar (toda afirmación con file:línea).
  - [x] No reintentar en bucle (1 retry con diagnóstico).
  - [x] No dejar huérfanos (cada step → contrato).
  - [x] No degradar errores (N/A, ejemplo sin paths dinero/seguridad).
  - [x] No gastar presupuesto infinito (3 líneas, 1 smoke re-run, stop rules no disparadas).
- **Veredicto:** pendiente orquestador (worker deja evidencia + diff mínimo)

## Notas
- Gate D: NO dispara (1 archivo ejemplo, sin símbolos públicos nuevos, sin hot path, contrato mecánico claro).
- Gate P: heredado del plan (set de 9 aprobado por owner).
- NOTICED BUT NOT TOUCHING: `examples/demo/demo.py:90,190`, `examples/python/{autogen,crewai,haystack,langgraph,mem0}_*.py`, `examples/colab/*.ipynb`, `vantadb-python/smoke_test_extended.py`, `vantadb-python/README.md` (tabla `search_memory`) — todos legacy; ¿nuevo FIND de sync? → orquestador decide.
- Branch: develop. Commit: `refactor: FIND-114 — ...`. NO PUSH.
