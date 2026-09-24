# FIND-116: Migrar otros ejemplos con API legacy a `Client` (cierre de FIND-114)

## Metadata
- **Plan file:** docs/dev/plans/2026-09-18-cierre-mvp.md (Wave1, Task 5)
- **Fuente:** plan file Wave1 Task 5 + Gate C FIND-114 (lista concreta) + NOTICED FIND-114:185
- **Esfuerzo:** 🟢 (max 1h appetite)
- **Prioridad:** 🟢 Baja
- **Tipo:** refactor (detección MCP: python/Python SDK)
- **Turns estimados:** 8-12
- **Creado:** 2026-09-18
- **last-synced:** 2026-09-18
- **Estado:** 🔄 IN PROGRESS (Steps 1-4 ✅; Step 5 cierre en curso)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 3 (S2 ejemplos python, S3 demo+smoke+README, S4 verify+commit+cierre)

## 1. TAREA
- **Objetivo:** migrar todos los ejemplos con API legacy (`vantadb_py.VantaDB`, `search_memory`/`list_memory`/`get_memory`) al API canónico (`Client` + `search`/`memory.*`) cerrando FIND-114.
- **Motivo (Gate Justificación plan):** consistencia de ejemplos (dos APIs confunden); mecánico por archivo.
- **Contrato:** cada ejemplo tocado usa `Client`/`search` + smoke/pytest verde por ejemplo + README sin `search_memory` stale; si un ejemplo no mapea 1:1 → se deja con motivo (no forzar).
- **AC:**
  1. Cero `VantaDB(`/`search_memory`/`list_memory`/`get_memory` en los 10 archivos tocados (grep vacío).
  2. Smoke verde por ejemplo migrado (`PYTHONUTF8=1 python <ejemplo>`, codepage Windows cp1252 vs emojis — precedente FIND-114).
  3. `vantadb-python/README.md` sin `search_memory` stale (nota naming + quickstart + tabla parity corregidas).
  4. Comportamiento idéntico (namespaces/keys/payloads/vectores/prints/flush/close/cleanup intactos).
- **Pre-mortem (plan) verificado:** (1) semántica distinta → mapeada abajo, 1 divergencia aparente (`Record.get("vector")`) con equivalente (`Record["vector"]` → None, types.rs:139-142); (2) sin test → smoke + nota.
- **Stop:** >1h → ship parcial (archivos hechos) + resto a nota (no re-abrir FIND).
- **Appetite/Branch/Commit:** max 1h / develop / `refactor: FIND-116 — ...` (NO PUSH, staging selectivo).

## 2. ARCHIVOS
- **Clave (10 archivos a editar):**
  - `examples/python/autogen_memory.py` (301L — leído completo)
  - `examples/python/crewai_memory.py` (227L — leído completo)
  - `examples/python/dspy_retriever.py` (257L — leído completo)
  - `examples/python/haystack_documentstore.py` (341L — leído completo)
  - `examples/python/langgraph_checkpoint.py` (295L — leído completo)
  - `examples/python/mem0_integration.py` (293L — leído completo)
  - `examples/python/semantic_kernel_memory.py` (322L — leído completo)
  - `examples/demo/demo.py` (239L — leído completo; path real, el plan decía `examples/python/demo.py`)
  - `vantadb-python/smoke_test_extended.py` (69L — leído completo; path real, el plan decía `examples/python/`)
  - `vantadb-python/README.md` (164L — leído completo)
- **Relacionados (solo lectura):**
  - `examples/python/agent_memory.py` (patrón migrado `ff05d663`: `from vantadb import Client`, `Client(...)`, `db.search(...)`)
  - `docs/dev/tasks/FIND-114.md` (precedente Gate C + mapeo 6 filas + smoke `PYTHONUTF8=1`)
  - `vantadb-python/tests/test_sdk.py` + `test_subclients.py:316-335` (guard AST-012: `*_memory` eliminados; `db.search` ≡ `db.memory.search`)
  - `vantadb-python/src/lib.rs` (lectura: `Client::new` ctor `db_path=` kw; `put:1018`; `search:1241-1255` firma kwargs; flat `get:1563`/`delete:1577` NODE-level u128; sin flat `list`; `flush:1769`; `close:1886`; `VantaDB` no existe como clase)
  - `vantadb-python/src/types.rs` (lectura: `Record.__getitem__` 132-156 incl. `"vector"`→None; `ListResult` `__len__`/`__iter__`/`__getitem__` int+str 211-238; `SearchHit.key/payload/metadata/score/created_at_ms/vector`)
  - `vantadb-python/vantadb/__init__.py` (canónico `import vantadb` → `Client`)
  - `examples/python/langchain_ollama_rag.py` (151L — leído completo; usa `vantadb_langchain.VantaDBVectorStore`, SIN API legacy directa → NO TOCAR, ver motivo abajo)
- **PROHIBIDOS (no tocar):** cambiar comportamiento de los ejemplos; `vantadb-python/src/` (bindings, solo lectura); `src/`; `vanta-memory/src/`; `vantadb-mcp/src/` (IMPL-112-S1 ✅ `1dd9019c`); `reparacion.bat`; `.opencode`; `Justfile`; `ocr-*`; `completions/*`; `desktop/src-tauri/Cargo.lock`; stash@{0} GOV-C4; `docs/dev/Backlog.md`; plan file (solo recitation); `C:/Users/Eros/.vantadb*`; `.opencode/skills/` (FIND-119); docs stale fuera de contrato (`README.md` raíz, `README_ES.md`, `docs/api/*` — NOTICED); `examples/colab/*.ipynb` (NOTICED, sin smoke posible); `packages/langchain-vantadb` (integración usada por ejemplo intacto).

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | ninguno (ejemplos standalone; sin imports desde código — mismo veredicto FIND-114) |
| Callees | `vantadb.Client` (`put`/`search`/`memory.get`/`memory.list`/`memory.delete`/`flush`/`close`/`capabilities`/`operational_metrics`/`hardware_profile`), `Record`/`SearchHit`/`ListResult` getters+`__getitem__` |
| Implicaciones | contrato público no cambia; comportamiento idéntico; sin impacto performance/memoria/serialización; sin migración de datos; ningún test existente afectado; `langchain_ollama_rag.py` intacto (sin legacy) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** los 10 clave + 7 relacionados de arriba (ver §2). Todo verificado en código real + módulo instalado en vivo (`hasattr` probe: `VantaDB` False, `search_memory`/`list_memory` False, `search` True, `memory.search` True, `Record["payload"]` OK).
- **Archivos referenciados hacia dentro:** `vantadb` (canónico) / `vantadb_py` (legacy deprecado PY-03, warning) → tras migración solo `vantadb`; stdlib (`os`, `shutil`, `json`, `tempfile`, `hashlib`, `uuid`) intacto; `vantadb_langchain` (langchain_ollama_rag, intacto).
- **Archivos que referencian a los editados:** ninguno desde código (ejemplos standalone con `main()` + `__main__`).
- **Veredicto impacto:** BAJO — 10 archivos ejemplo/docs standalone, migración mecánica renombre + `memory.` routing; si un editado desapareciera, nada de código se rompe.

## Contrato
"Cada ejemplo usa `Client`/`search` (grep cero-legacy), smoke `PYTHONUTF8=1 python <ejemplo>` exit 0 por ejemplo, README sin `search_memory`; langchain/colab se dejan con motivo escrito"

## Spec (SDD)
No aplica: refactor mecánico, cero símbolos públicos nuevos (ninguna señal Phase 1b — no se agrega `pub fn`, tool, endpoint ni método de binding). Migración 1:1 verificada en DISCOVERY (tabla abajo).

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** comportamiento idéntico (namespaces/keys/payloads/vectores/prints/métricas/cleanup); no tocar prohibidos; WIP ajeno intocable; secrets nunca a disco.
- **Comandos de verificación:** `PYTHONUTF8=1 python <ejemplo>` exit 0 c/u; `python -m py_compile` c/u; `grep search_memory|list_memory|get_memory|VantaDB(` vacío en los 10; `campaign_verify_cmd` (bug exit -1 → bash directa + mención).
- **Deuda pendiente:** ninguna prevista (colab/langchain/docs-raíz quedan NOTICED con motivo, no deuda).

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | Valor |
|------------------------|-------|
| `activeGoal` | FIND-116 migrar 10 superficies legacy a Client |
| `lastAction` | Step 1 DISCOVERY: task file + mapeo 1:1 por archivo verificado en código+instalado |
| `result` | PARTIAL (Steps 2-4 pendientes) |
| `nextAction` | Step 2 ACT: 7× examples/python (import+ctor+search/get/delete/list) + py_compile + grep |
| `contract` | verificacion: task file creado; evidencia: claim mapeo 1:1 / evidencia lib.rs:1018+1241-1255+1563+1577, types.rs:132-156+211-238, test_subclients.py:316-335, probe hasattr en vivo / confianza alta; artefactos: docs/dev/tasks/FIND-116.md; invariantes: comportamiento idéntico, prohibidos intactos; deuda: Steps 2-4 pendientes; queda_pendiente: edición + smokes + README + commit + FIND-119 |
| `nextTask` | FIND-119 (orquestador) |

## Deuda técnica (Regla 6 — MUST)
**Saldo neto por PR:** cero — refactor mecánico sin atajos. NOTICED BUT NOT TOUCHING (¿nuevo FIND? → orquestador): `examples/colab/vantadb_quickstart.ipynb` (`VantaDB`+`search_memory`×2, notebook sin smoke posible en appetite); `vantadb-python/sanity_check.py` (`vanta.VantaDB`, hallado en grep — script dev fuera de superficies Gate C); `README.md` raíz (2×`search_memory`+2×`get_memory`); `README_ES.md`; `docs/api/{BINDINGS_NAMESPACES,MCP,PYTHON_SDK,scores}.md` (stale `*_memory`); `packages/langchain-vantadb` (verificar en su propio task si el ejemplo intacto falla).

## Definition of Done (contrato multi-nivel)
| Nivel | Gate |
|-------|------|
| **Task** | AC 1-4 ✅ + smokes verdes + `git diff --stat` solo los 10 archivos |
| **Commit** | commit atómico `refactor: FIND-116 — ...`, staging selectivo, NO PUSH |
| **Release** | N/A (ejemplos/docs; sin cambio versionable ni changelog) |

## Herramientas necesarias
- system python (`py_compile` + smoke + grep cero-legacy)
- `campaign_verify_cmd` (bug exit -1 → bash directa + mención en RESULTADO)
- codegraph solo si hace falta ubicar `Client` (no hizo falta — lib.rs directo). Cargo N/A (Python/docs). Internet N/A.

## Skills cargadas (SDP)
`SDP: incremental-implementation` (migración mecánica por archivo — sugerida plan ✅ cargada) · `systematic-debugging` (mapeo 1:1 + Prove-It si diverge — sugerida ✅ cargada) · `context-engineering` (context pack por slice ✅ cargada) · `test-driven-development` (smoke como RED/GREEN por ejemplo ✅ cargada) · base sesión (campaign-executor, progreso, ponytail full, brainstorming, writing-plans, planning-and-task-breakdown). Descartadas del scoring con motivo: `doubt-driven-development` (sin trust boundary), `frontend-ui-engineering` (sin UI), `api-and-interface-design` (sin API nueva), `source-driven-development` (sin docs externas). Keywords: examples/python/demo/autogen/crewai/haystack/langgraph/mem0/smoke/README/migrar-legacy/Client/search/smoke-pytest.

## Investigation Notes
### INVESTIGACIÓN CÓDIGO (DISCOVERY — mapeo 1:1 legacy→Client por archivo ANTES de editar)
Reglas globales verificadas en vivo (`import vantadb_py` probe + `import vantadb` + firma `put(namespace, key, payload, metadata=None, vector=None, ttl_ms=None)`):
- G1 `import vantadb_py as vantadb` → `import vantadb` (canónico `vantadb/__init__.py:1-16`; `vantadb_py` deprecado PY-03 con warning) — 1:1 ✅
- G2 `vantadb.VantaDB(path, memory_limit_bytes=...)` → `vantadb.Client(...)` (`VantaDB` NO existe: hasattr False; ctor `Client(db_path, ...)` acepta `db_path=` kw — smoke L16 ✅) — 1:1 ✅
- G3 `db.search_memory(ns, query_vector=, text_query=, top_k=, filters=)` → `db.search(...)` (firma PyO3 `lib.rs:1241` acepta esos kwargs; guard AST-012 `test_subclients.py:325` exige `search_memory` eliminado) — 1:1 ✅
- G4 `db.list_memory(ns, filters=, limit=)` → `db.memory.list(...)` (sin flat `list` en `Client`; `ListResult` con `__len__`/`__iter__`/`__getitem__` int+str `types.rs:211-238` ⇒ `len(page)`, `len(page["records"])`, `.records`, `for r in page` intactos) — 1:1 ✅
- G5 `db.get(ns, key)` → `db.memory.get(...)` (flat `get:1563` es NODE-level `id: u128` — ¡routing obligatorio, no rename!; `memory.get` devuelve `Record|None` ⇒ `if record:` intacto; `Record.__getitem__` `types.rs:132-156` ⇒ `record["key"/"payload"/"metadata"/...]` intacto) — routing 1:1 ✅
- G6 `db.delete(ns, key)` → `db.memory.delete(...)` (flat `delete:1577` es node-level; `memory.delete`→bool ⇒ `if self.db.delete(...): count+=1` intacto) — routing 1:1 ✅
- G7 `db.get_memory(ns, key)` → `db.memory.get(...)` (demo.py:195, smoke:29,57) — 1:1 ✅
- G8 `record.get("vector")` → `record["vector"]` (haystack:97,234,264; `Record` sin método `.get`; `__getitem__("vector")`→None si ausente `types.rs:139-142` ≡ semántica `dict.get`) — 1:1 ✅
- Sin cambio: `db.put(...)` (shared-name flat `lib.rs:1018`, kwargs demo `namespace=/key=` ✅ por firma), `hit.key/payload/metadata/score/created_at_ms/vector` (getters), `dict(hit.metadata)` (PyDict), `record["metadata"].get(...)` (dict real), `flush/close/capabilities/operational_metrics/hardware_profile` (flat), prints/métricas/cleanup/DB_PATHs.
| Archivo | Edits (G-rules) |
|---------|-----------------|
| autogen_memory.py | G1,G2,G3(L132),G4(L170),G5(L97) ×1 get, `delete_message` G6(L193) |
| crewai_memory.py | G1,G2,G3(L110),G4(L153),G5(L80),G6(L140) |
| dspy_retriever.py | G1,G2,G3(L98),G4(L172,182),G5(L145),G6(L163,185) |
| haystack_documentstore.py | G1,G2,G3(L142),G4(L188,204,224),G5(L89,258),G6(L182,191),G8 ×3 |
| langgraph_checkpoint.py | G1,G2,G3(L179),G4(L131),G5(L97),G6(L161) |
| mem0_integration.py | G1,G2,G3(L124),G4(L211),G5(L87),G6(L193) |
| semantic_kernel_memory.py | G1,G2,G3(L141),G4(L208),G5(L171),G6(L191) |
| demo.py | `import vantadb_py`→`import vantadb` (L88), `vantadb_py.VantaDB`→`vantadb.Client` (L90,190), G3 ×2 (L150,165), G4 ×2 (L182,191), G7 (L195) |
| smoke_test_extended.py | `import vantadb_py as vanta`→`import vantadb as vanta` (L1), `vanta.VantaDB`→`vanta.Client` (L16,55), G7 ×2 (L29,57), G4 (L35), G3 (L40) |
| README.md | L9 `search_memory`→`search`; L23-27 nota naming (VantaDB removido + `db.memory` canónico); L64 `get_memory`→`memory.get`; L70 `search_memory`→`search`; tabla L128-141 columna Python (`search_memory`→`search`/`memory.search`, filas meaning/ANN corregidas: hybrid=`search`, ANN=`search_vector`); hazard L140-142 |
### INVESTIGACIÓN PROBLEMA
¿Semántica legacy sin equivalente? NO — todo mapea (G1-G8). NO mapea (se dejan con motivo, no forzar): `langchain_ollama_rag.py` (usa `vantadb_langchain.VantaDBVectorStore`, cero API legacy directa — intacto si su smoke pasa); `examples/colab/*.ipynb` (fuera de superficies plan + sin smoke en appetite — NOTICED).
### INVESTIGACIÓN INTERNET
N/A (todo verificado en código local + módulo instalado; sin APIs externas).
### Pre-mortem (plan) verificado
(1) semántica distinta → mapeada G1-G8, 1 divergencia aparente resuelta (G8); (2) sin test → smoke por ejemplo + nota.

## Incógnitas (uphill) vs Pendientes (downhill)
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 3 (S2, S3, S4) |
| % completado | 25% |

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — no aplica: ejemplos locales sin trust boundary (sin input usuario red, auth, deps nuevas, FFI propio). Sin `security-and-hardening`.
- [x] **PERFORMANCE** — no aplica: sin hot path (demos con vectores mock). Sin `performance-optimization`, sin baseline.

## Steps
### Step 1: DISCOVERY (mapeo 1:1 + task file)
- **Archivos:** `docs/dev/tasks/FIND-116.md` (este file)
- **Acción:** mapear legacy→Client en código real + instalado en vivo + crear task file con las 10 secciones
- **Verify:** task file existe con Impacto Regla 0 poblado + Gate D evaluado (no dispara: refactor mecánico 1:1, 10 files, cero símbolos nuevos)
- **Estado:** ✅ COMPLETED

### Step 2: ACT slice 1 (7× examples/python + py_compile + grep)
- **Archivos:** `examples/python/{autogen,crewai,dspy,haystack,langgraph,mem0,semantic_kernel}*.py`
- **Acción:** aplicar G1-G8 por archivo (solo líneas mapeadas; `main()`/prints intactos). Luego `py_compile` ×7 + grep cero-legacy en `examples/python/`.
- **Verify:** `grep -n "VantaDB(\|search_memory\|list_memory\|get_memory\|vantadb_py" examples/python/*.py` vacío + `py_compile` OK ×7
- **Estado:** ✅ COMPLETED (py_compile OK ×7; grep cero-legacy vacío en `examples/python/`)

### Step 3: ACT slice 2 (demo.py + smoke + smokes verdes)
- **Archivos:** `examples/demo/demo.py`, `vantadb-python/smoke_test_extended.py`
- **Acción:** aplicar mapeo + correr `PYTHONUTF8=1 python examples/demo/demo.py` y `PYTHONUTF8=1 python vantadb-python/smoke_test_extended.py` (cada uno en su cwd temporal; DBs auto-limpian) + smoke `langchain_ollama_rag.py` como testigo intacto (debe seguir exit 0 sin cambios).
- **Verify:** ambos exit 0 + grep cero-legacy en ambos + testigo exit 0
- **Estado:** ✅ COMPLETED (demo.py exit 0 secciones 1-7 + cleanup; smoke PASSED 9/9 asserts; testigo langchain NO CORRIDO con motivo: `vantadb_langchain` no instalado en este env + archivo intacto sin legacy — nota, no fallo)

### Step 4: ACT slice 3 (README + smokes restantes)
- **Archivos:** `vantadb-python/README.md`
- **Acción:** aplicar edits tabla §INV + correr smokes de los 7 ejemplos (Step 2 solo compiló; aquí el verde funcional por ejemplo).
- **Verify:** `grep -n "search_memory\|list_memory\|get_memory" vantadb-python/README.md` vacío + 7 smokes exit 0
- **Estado:** ✅ COMPLETED (README grep vacío; 7/7 smokes exit 0: autogen, crewai, dspy, haystack, langgraph, mem0, semantic_kernel)

### Step 5: VERIFY + commit + cierre
- **Archivos:** `docs/dev/tasks/FIND-116.md` (sync), plan file (recitation FIND-116)
- **Acción:** DoD 3 niveles + OCR delegation (`pwsh dev-tools/ocr-review.ps1`; Critical/High=bloquea) + `git status` (staging SOLO los 10 + task file) + commit `refactor: FIND-116 — ...` (NO PUSH) + recitation + `skill progreso` + RESULTADO §7
- **Verify:** `git log --oneline -1` + `git status --short` sin restos + `campaign_update_task_state completed`
- **Estado:** ✅ COMPLETED (ver abajo)

## Dependencias
- Wave1 segunda en secuencia (IMPL-112-S1 ✅ `1dd9019c` cerrado por lead; FIND-119 después — `.opencode/skills/` intocable aquí).
- Stop: >1h → ship parcial (archivos hechos) + resto a nota (no re-abrir FIND).
- NextTask: FIND-119 (orquestador).

## 3. DEPENDENCIAS (resumen)
Wave1 segunda en secuencia. NextTask: FIND-119.

## 4. REFERENCIAS
- Rules (lectura completa ✅): `.opencode/rules/python-bindings.md` (R-1 batch/GIL, R-2 closures — no aplican: sin batch nuevo ni closures; verificado).
- Refs: `.opencode/references/definition-of-done.md` (DoD 3 niveles arriba); commands `pipeline.md` (este run); SPEC.md raíz (N/A — refactor ejemplos, tabla Spec N/A).
- Workflow: `refactor` (audit ✅ S1 → migrate S2-S4 → cleanup N/A → verify+review+P2-01-orquestador → accept → close S5).

## 5. SKILLS
Ver sección Skills cargadas (SDP) arriba. `SKILLS_CARGADAS:` en RESULTADO §7.

## 6. HERRAMIENTAS+MCP
system python (pytest/smoke + py_compile + grep cero-legacy) + `campaign_verify_cmd` (bug exit -1 → bash directa + mención). codegraph solo si hace falta (no hizo falta). Cargo N/A (Python/docs). Internet N/A.

## 7. INVESTIGACIÓN CÓDIGO
Ver Investigation Notes (tabla G1-G8 + por archivo). En DISCOVERY ✅. OJO codepage Windows cp1252 vs emojis — smokes con `PYTHONUTF8=1` (precedente FIND-114).

## 8. INVESTIGACIÓN PROBLEMA
Ver Investigation Notes. Sin semántica huérfana ✅ (2 no-mapeos dejados con motivo: langchain intacto, colab NOTICED).

## 9. INVESTIGACIÓN INTERNET
N/A.

## 10. VALIDACIÓN+CIERRE
Verify contrato (S5) + OCR delegation + DoD 3 niveles + P2-01 lo hace el orquestador (no vos) + Gates D/V/C vía `question` (D: no dispara — refactor 1:1 sin símbolos nuevos; V: solo si un smoke falla 2× mismo error; C: colaterales = docs-raíz stale → NOTICED, no toco) + RESULTADO §7 obligatorio.

## Review (GATE — agente distinto, P2-01)
- **Revisor:** orquestador/lead (P2-01; worker no se auto-aprueba — pendiente veredicto del orquestador)
- **Enfoque:** ¿migración 1:1 fiel? Tabla G1-G8 dice sí; diff solo líneas mapeadas (stat: 10 archivos, +143/-141 aprox en ejemplos+docs).
- **Cómo se probó:** smokes reales `PYTHONUTF8=1` — demo.py exit 0 (7 secciones + persistencia 5/5 + cleanup), smoke_test_extended.py PASSED (9 asserts, persistencia re-read), 7/7 ejemplos exit 0 con DBs en `Temp/opencode` (cero residuos en repo); `py_compile` OK ×9; grep cero-legacy vacío en los 10 (FINAL_GREP exit 1); `campaign_verify_cmd` py_compile passed exit 0 (sin bug esta vez); OCR delegation: grupos 2 (python) y 3 (python-bindings) sin Critical/High — renames puros, R-1/R-2 N/A; resto grupos = WIP ajeno no tocado ni stagiado.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos (exits + logs reales: demo exit 0, smoke PASSED, 7× exit 0).
  - [x] No saltarse clarificación (mapeo G1-G8 verificado en código + instalado en vivo antes de editar; flat `get`/`delete` node-level detectado → routing `memory.` en vez de rename ciego).
  - [x] No declarar done sin verificar AC (AC 1-4 con evidencia; testigo langchain con motivo escrito, no silencio).
  - [x] No ignorar fallos (testigo no-corrido = ambiental pre-existente, archivo intacto, motivo en task file).
  - [x] No un solo intento de búsqueda (lib.rs + types.rs + tests + probe hasattr en vivo + grep).
  - [x] No copiar sin citar (toda afirmación con file:línea).
  - [x] No reintentar en bucle (cero retries: todo verde al primer run).
  - [x] No dejar huérfanos (colab/docs-raíz/sanity_check/langchain → NOTICED con motivo + ¿nuevo FIND? → orquestador).
  - [x] No degradar errores (N/A, ejemplos sin paths dinero/seguridad).
  - [x] No gastar presupuesto infinito (dentro de appetite 1h; stop rule no disparada).
- **Veredicto:** pendiente orquestador (worker deja evidencia + diff mínimo)

## Notas
- Gate D: NO dispara (refactor mecánico 1:1, 10 files, sin símbolos públicos nuevos, sin hot path, contrato claro).
- Gate P: heredado del plan (set de 9 aprobado por owner).
- Desvíos reales vs plan (verificados, no inventados): `demo.py` vive en `examples/demo/` (no `examples/python/`); sub-ejemplos son archivos planos (`*_memory.py`, no dirs `autogen/` etc.); `smoke_test_extended.py` vive en `vantadb-python/`; +2 superficies reales (`dspy_retriever.py`, `semantic_kernel_memory.py`, `langchain_ollama_rag.py` como testigo) — total 10 editados + 1 testigo intacto.
- Branch: develop. Commit: `refactor: FIND-116 — ...`. NO PUSH.
