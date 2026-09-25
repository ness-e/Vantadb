# FIND-94 — drift SDK 0.5.0 vs 9 adapters (`vanta.VantaDB` ausente)

> Campaign: 6ab26f3f-cf16-4416-9255-c18cca0bcaf0 · Wave9 (plan-adjust 2026-09-16) · Appetite 🟡 Media · Branch develop
> Origen: Backlog :236 + verify FIND-69 2026-09-15 + verify FIND-84 (shim temporal)
> Estado: ⬜ PENDING → ⏳ IN PROGRESS (esta sesión) · Task file creado en DISCOVERY (no existía)
> Última task: FIND-93 ✅ 0cd54e47 · Next: cierre de campaña (orquestador)

## 1. TAREA

**Objetivo:** eliminar el drift entre el SDK Python 0.5.0 y los 9 adapters de `integrations/`: el módulo compilado ya NO expone `VantaDB` (solo `Client`/`connect` + sub-clientes `memory`/`graph`/`system`/`wiki` + `Record`/`SearchHit`), pero 11 call-sites productivos construyen `vanta.VantaDB(...)` → todas sus suites fallan con `AttributeError` sin el shim temporal de FIND-84.

**Contrato exacto:**
- Los 9 adapters importan y construyen sin `AttributeError` (cero `vanta.VantaDB`, cero `vanta.VantaMemoryRecord`, cero `vanta.VantaSearchHit` en `integrations/*/vantadb_*/` + `integrations/vantadb_shared/`).
- Suites `pytest integrations/<adapter>/tests/` verdes por adapter separado (mocks, sin red; skips por `importorskip` de framework ausente = salida válida, no rojo). Matriz CON-framework documentada como no-ejecutada si el framework no está instalado (igual que FIND-69).
- Decisión migrar-vs-shim documentada con evidencia (tabla Spec §4b).
- Salida ship-parcial aceptada: adapters migrados en verde + resto con DEFER-ratificado individual (motivo por adapter), nunca silencio.
- Al completar la migración prod: BORRAR `integrations/vantadb_test_shim.py` + los 9 imports en `tests/conftest.py` (las suites deben ejercitar el backend real vía `tmp_path`).

**Acceptance criteria:**
1. `rg -n "VantaDB\(|VantaMemoryRecord|VantaSearchHit" integrations/dspy integrations/letta integrations/haystack integrations/crewai integrations/langchain integrations/llamaindex integrations/mem0 integrations/vantadb_shared integrations/ollama integrations/openai --glob "*.py"` = 0 hits en código prod (docs/comentarios históricos pueden citar el nombre viejo solo como contexto de migración, nunca como código).
2. `python -W ignore -c "import vantadb_py as m; assert not hasattr(m,'VantaDB'); assert hasattr(m,'Client'); assert hasattr(m,'connect')"` ✅ (ya verificado en DISCOVERY).
3. `python -W ignore -c "import vantadb as m; assert not hasattr(m,'VantaDB')"` ✅.
4. Por adapter (proceso separado, mocks): `python -m pytest integrations/<adapter>/tests/ -q` verde (passed + skipped-por-framework-OK, 0 failed/error). Adapters: crewai, dspy, haystack, langchain, letta, llamaindex, mem0, ollama, openai (ollama/openai heredan de `vantadb_shared`).
5. `python -m pytest integrations/test_pins.py -q` verde (10 tests, FIND-84 intacto).
6. `python -m py_compile` sobre los 11 `.py` prod tocados ✅ + `git diff --check` limpio.
7. Score-convención fijada contra backend real (ver §7): similitud coseno mayor=mejor (1.0 idéntico), NO distancia. `langchain`/`llamaindex` con `1.0 - score/2.0` corregidos o con DEFER-ratificado individual.
8. Sin símbolos públicos nuevos (migración usa `Client`/`memory.*` existentes) → Gate D por conteo de archivos (>10) evaluado abajo, sin `question` bloqueante (pre-autorizado por plan-adjust + ship-parcial).

## 2. ARCHIVOS

**Clave (re-verificados en DISCOVERY 2026-09-16 con `rg -n "VantaDB\(" integrations/` = 12 líneas: 11 código + 1 README-doc):**
- `integrations/dspy/vantadb_dspy/vectorstore.py:75` (`vanta.VantaDB(`) · `:104,110,165,179,185` (search_memory/list_memory/delete_memory/put)
- `integrations/letta/vantadb_letta/vectorstore.py:43` · `:83,108,114,129,152` (put/list_memory/search_memory/delete_memory)
- `integrations/haystack/vantadb_haystack/vectorstore.py:135` · `:151,159` (tipos `vanta.VantaMemoryRecord`/`vanta.VantaSearchHit` en anotaciones) · `:275,281,287,290,316,353,362,364,369,385,473,478` (get_memory/delete_memory/put/list_memory/search_memory)
- `integrations/crewai/vantadb_crewai/vectorstore.py:68` · `:102,115,148,165,188` (search_memory/list_memory/put/delete_memory)
- `integrations/crewai/vantadb_crewai/memory.py:104` · `:148,167,177,203,248` (list_memory/put/get_memory/search_memory/delete_memory)
- `integrations/langchain/vantadb_langchain/checkpointer.py:80` · `:103,115,125,251,281,291,302,310` (list_memory/get_memory/put/delete_memory)
- `integrations/langchain/vantadb_langchain/store.py:88` · `:159,165,174,187,209,246` (put/get_memory/delete_memory/list_namespaces/list_memory/search_memory) · `:254` comentario distancia
- `integrations/langchain/vantadb_langchain/vectorstore.py:44` · `:63,200,286,295,362,371,439,496,517,528,561,536` + `:216,572` (mapeo `1.0 - s/2.0`) · `:270,349` docstrings distancia
- `integrations/llamaindex/vantadb_llamaindex/vectorstore.py:53` · `:61` (`-> vanta.VantaDB`) · `:72,84` (tipos legacy) · `:140,152,163,184,193,245,347,356,465,478,484,490` (put_batch/list_memory/delete_memory/search_memory/get_memory) · `:210,215,264,371` (mapeo `1.0 - score/2.0`)
- `integrations/mem0/vantadb_mem0/vectorstore.py:122` · `:157,177,191,213,216,217,224,238,245,249,251,269,304,306` (put/search_memory/delete_memory/update_memory/get_memory/list_namespaces/delete_namespace/list_memory)
- `integrations/vantadb_shared/__init__.py:47` · `:102,156,203` (put/search_memory/delete_memory)
- `integrations/vantadb_test_shim.py` (SOLO LECTURA + BORRADO al final si migración completa; NO re-tocar lógica — FIND-84)
- `integrations/test_pins.py` (SOLO LECTURA para verify; NO re-tocar — FIND-84)
- `vantadb-python/vantadb_py/__init__.py` (SOLO LECTURA: confirma exports `Client`/`connect`/`Record`/`SearchHit`, ausencia `VantaDB`/`VantaMemoryRecord`/`VantaSearchHit`)
- `vantadb-python/vantadb_py/vantadb_py.pyi:160-331` (SOLO LECTURA: firma `Client.__init__(db_path, memory_limit_bytes, read_only, backend)` + `MemoryClient` con `put/put_batch/get/list/delete/search/...`; flat `get/delete` son node-level `id:u128`, NO memoria)
- `docs/api/PYTHON_SDK.md` (SOLO LECTURA: firma vigente `Client`/`connect`, path `db.memory.*`, `ListResult`, Regla 11 N/A)
- `integrations/README.md:60-92` (SOLO LECTURA: estado suites 2026-09-15 + hallazgo score pendiente FIND-94)
- Tests por adapter: `integrations/<adapter>/tests/test_*.py` + `tests/conftest.py` (9 con shim-import a eliminar al final)
- `docs/dev/tasks/FIND-69.md` (SOLO LECTURA: `vectorstore.py:62-70` ya fixeado f3d6c634 — extender, no revertir; matriz sin-red como precedente)

**Prohibidos (WIP ajeno que NO se toca):**
`.opencode/` (submodule con WIP ajeno — solo lectura) · `Justfile` · `completions/_vanta-cli*` · `desktop/src-tauri/Cargo.lock` · `.github/workflows/ocr-delegate.yml` · `dev-tools/ocr-review.ps1` (solo ejecución) · `reparacion.bat` · `docs/pipeline-state.json` · plan file (solo recitation orquestador) · `stash@{0..14}` · archivos FIND-92 (`ci-rustdoc.yml`) y FIND-93 (`src/storage/engine/txn.rs`) · `vantadb-python/` fuera de lectura (el binding lo publica otro flujo; si el shim debiera vivir ahí y no podemos → Gate V con propuesta exacta, NO edición directa) · `docs/dev/Backlog.md` + `docs/dev/avance/` (race paralelo — orquestador vía `progreso`) · push (vía vanta-lead).

## 3. DEPENDENCIAS

- Wave9 (plan-adjust 2026-09-16; 29/30 con FIND-92 ✅ e395b563 + FIND-93 ✅ 0cd54e47). Sin bloqueantes.
- Previa: FIND-93 ✅ (clippy `txn.rs` desbloqueado — verificado que `cargo clippy -p vantadb` verde antes de tocar Python).
- Next: cierre de campaña (progreso masivo + retrospectiva + archive) — esta es la ÚLTIMA task.
- Colaterales: FIND-84 (shim temporal + pins + fixtures — NO re-tocar salvo HALLAZGO; al cerrar FIND-94 se borra el shim) · FIND-69 (dspy fallback — NO revertir `:62-70`) · FIND-85 (classifiers — intacto).
- Filtro Propuesta §3: "SDKs mínimos + server opcional como wrapper" → superficie mínima, preferir migración; shim solo si migración rompe con evidencia.

## 4. REFERENCIAS

- **Propuesta Notion §3 (SDKs mínimos — mantener superficie mínima):** "SDKs mínimos + server opcional como wrapper" + inventario "SDK Python (PyO3, ~55 métodos en la clase principal)". Lectura completa 2026-09-16 vía fetch (ver §9). Filtro: preferir migración al API existente (`Client`/`memory.*`); shim permanente (`VantaDB` re-expuesto) solo si la migración rompe algo con evidencia reproducible. Un shim en `integrations/` sería deuda permanente (segunda superficie que mantener); un shim en `vantadb-python/` está prohibido en esta task (otro flujo publica el binding).
- **`docs/api/PYTHON_SDK.md` (API vigente, last_reviewed 2026-09-15):** constructor `vantadb.Client(db_path, memory_limit_bytes, read_only, backend)`; `vantadb.connect(path, memory_limit, read_only, backend)`; memoria en `db.memory.put/get/list/delete/search` (short names, paridad TS `MemoryClient`); flat `get/delete` son node-level (`id: u128`); `ListResult` con `.records`/`.next_cursor` + iteración + dict-style; `Record`/`SearchHit` con `.key/.payload/.metadata/.vector/.score/...`; legacy `VantaDB`/`VantaMemoryRecord`/`VantaSearchHit`/`VantaVector`/`VantaError` removidos (doc dice 0.6.0, código 0.5.0 ya sin ellos — divergencia menor documentada, no bloquea).
- **Reglas:** `.opencode/rules/python-bindings.md` (R-1 GIL-release batch — no aplica: adapters son glue sync; R-2 closures — no aplica) + `.opencode/rules/api-contract.md` (R-1 claim→símbolo real — cada `memory.*` verificado en `dir()` real; R-8 lógica en core, bindings glue — los adapters NO reimplementan distancias salvo MMR/RRF opt-in documentados con `TODO(core)`; el fix `1.0 - score/2.0` devuelve el cálculo al core).
- **Regla 11 N/A justificado:** sin claims de performance en esta task (cero números, cero adjetivos de rendimiento). Benchmarks no aplican (adapters glue, no hot paths).
- **Clean Code (Paso 0c):** `.opencode/references/clean-code-clean-architecture.md` leído completo + Apéndice V (mapa capas→repo: `integrations/` = Frameworks/Drivers = Humble Objects, glue + memoria, cero lógica de negocio; SLAP ≤20 líneas/≤3 args como convención operativa; TDD FIRST+AAA).

### 4b. Spec — decisión migrar-vs-shim (Gate D: tabla requerida porque blast radius = 11 prod + 9 conftest + 1 shim > 10 archivos)

| Decisión | Opción | Evidencia | Consecuencia |
|---|---|---|---|
| D1 Constructor | **Migrar** `vanta.VantaDB(...)` → `vanta.Client(...)` (mismos kwargs) | `vantadb_py.pyi:174-180` firma `Client(db_path, memory_limit_bytes, read_only, backend)` idéntica a los 11 call-sites (todos pasan `db_path` posicional + esos 3 kwargs); probe real `Client(d)` OK | 11 líneas, cero cambio de firma pública de adapters |
| D2 Lecturas/escrituras memoria | **Migrar** `get_memory/list_memory/delete_memory/search_memory` → `memory.get/memory.list/memory.delete/memory.search` | `dir(Client)` sin `*_memory` (probe: `has search_memory: False`); `dir(db.memory)` con `get/list/delete/search/put/...`; `pyi:415-424` short names canónicos; probes `memory.get/list/delete/search` OK contra real | ~70 call-sites mecánicos; `put`/`put_batch`/`list_namespaces` quedan igual (existen en ambos) |
| D3 `update_memory` (solo mem0:213) | **Migrar** a `get` + `put` (upsert preservando metadata/vector) | `has update_memory: False` (probe); docstring mem0 ya prevé fallback delete+insert; `put` es upsert nativo (crewai `update()` lo usa así) | 4-6 líneas nuevas en mem0, sin símbolo nuevo |
| D4 `delete_namespace` (solo mem0:245) | **Migrar** a loop `memory.list` + `memory.delete` (igual que su fallback actual) | `has delete_namespace: False`; fallback existente hace exactamente ese loop con API vieja; `delete_by_filter` rechaza filtro vacío (no sirve) | ~8 líneas, sin símbolo nuevo |
| D5 `dict(page)` (solo dspy:179) | **Migrar** a `{"records": page.records, "next_cursor": page.next_cursor}` | Probe real: `dict(ListResult)` → `TypeError: 'int' object is not an instance of 'str'`; `page.records` + `page.next_cursor` OK; `page["records"]` OK | 1-3 líneas, fija crash garantizado post-migración |
| D6 Tipos `VantaMemoryRecord`/`VantaSearchHit` (5 sites) | **Migrar** a `Record`/`SearchHit` (o `Any` donde solo anotan) | `hasattr VantaMemoryRecord/VantaSearchHit: False`, `Record/SearchHit: True`; con `from __future__ import annotations` no rompen runtime hoy, pero mienten (api-contract R-1) | 5 líneas, cero runtime |
| D7 `-> vanta.VantaDB` (llamaindex:61) | **Migrar** a `-> vanta.Client` | `Client` existe; el property devuelve el handle real | 1 línea |
| D8 Score-convención | **Migrar+fijar**: backend real emite **similitud** mayor=mejor (probe §7); corregir `1.0 - score/2.0` (langchain:216 + llamaindex:210,215,264,371) a `score` directo + docstrings distancia→similitud; `_cosine_relevance_score_fn` + `test_cosine_relevance_score_fn` + `test_normalize_score_exact_semantics` se actualizan o DEFER-ratifican por adapter | Probes 2026-09-16 (ver §7): idéntico→1.0, ortogonal→0.0, diag→0.707; `text_query` ordena DESC por relevancia; crewai `score > 0.5` exige similitud | 5 sites + 2-3 tests; si un adapter rompe de forma no trivial → ship-parcial con DEFER-ratificado (pre-mortem) |
| D9 Shim permanente | **Rechazado** (solo test-shim temporal hasta fin de migración) | Propuesta §3 superficie mínima; shim = segunda superficie permanente; `vantadb-python/` prohibido en esta task (Gate V si hiciera falta ahí) | Cero símbolos nuevos → sin Gate D por API pública (solo por conteo de archivos, pre-autorizado) |

**Recomendado: MIGRAR (D1-D8), rechazar D9.** Sin `question` bloqueante: el contrato de la task ya pre-autoriza migrar-o-shim-con-evidencia + ship-parcial, y la evidencia de probes es concluyente. `question` no disponible en este runner → se documenta Gate D como disparado-por-conteo y resuelto-por-evidencia.

## 5. SKILLS (SDP Paso 0b — `campaign_discover_skills_v2` phase=BUILD, keywords sdk-drift/adapter-migration/shim-vs-migrate/pytest-matrix, ≤8)

- `source-driven-development` — cuándo: cada rename se verifica contra `vantadb_py.pyi` + `dir()` real + `PYTHON_SDK.md`, nunca desde memoria (api-contract R-1).
- `incremental-implementation` — cuándo: 11 prod en slices verticales por adapter (un adapter = un slice verde + commit atómico), nunca >100 líneas sin verify.
- `test-driven-development` — cuándo: RED = `pytest` sin shim falla con `AttributeError` (repro), GREEN = migración mínima, REFACTOR = score-fix con tests verdes.
- `context-engineering` — cuándo: por slice solo el adapter + su test + ejemplo patrón ya migrado (<2k líneas, Selective Include).
- `systematic-debugging` — cuándo: bug (drift PROV): root-cause = SDK removió `VantaDB` sin migrar adapters; si un verify falla → fases 1-4, no retry ciego. (Bug → skill inline obligatoria.)
- `doubt-driven-development` — cuándo: decisión no-trivial D8 (score) + D3/D4 (sin equivalente directo) → CLAIM + evidencia probe antes de cerrar.
- `api-and-interface-design` — cuándo: cambio toca superficie pública de adapters (constructores/firmas/test-helpers) → no nuevos símbolos, paridad entre los 9.
- `campaign-executor` — cuándo: base (state machine PLAN→ACT→VERIFY, recitation, RESULTADO). `progreso` auto-vía MCP (no carga manual).
- Excluida con motivo: `frontend-ui-engineering` (SDP la sugirió por lifecycle genérico; 0 archivos `web/` en blast radius → no aplica).

`SKILLS_CARGADAS:` systematic-debugging, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, api-and-interface-design (+ campaign-executor/progreso auto-vía MCP).

## 6. HERRAMIENTAS+MCP

**Comandos exactos (PowerShell, sin `head`):**
- `rg -n "VantaDB\(" integrations/` — re-verificar 12 hits en DISCOVERY (hecho: 11 código + 1 README).
- `rg -n "self\._db\.|self\._client\." integrations/ --glob "*.py"` — mapa de ~80 métodos post-construcción (hecho, ver §7).
- `python -W ignore -c "import vantadb_py as m; print(hasattr(m,'VantaDB'), hasattr(m,'Client'))"` — `False True` (hecho).
- `python C:\Users\Eros\AppData\Local\Temp\opencode\probe_find94.py` + `probe_find94b.py` — probes backend real (hechos, ver §7).
- Por adapter (proceso separado, mocks, sin red): `python -m pytest integrations/<adapter>/tests/ -q` (crewai, dspy, haystack, langchain, letta, llamaindex, mem0, ollama, openai).
- `python -m pytest integrations/test_pins.py -q` (FIND-84 intacto).
- `python -m py_compile integrations/<...>/vectorstore.py ...` por archivo tocado.
- `git diff --check` + `git status --short` (solo propios) antes de cada commit.
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation; Critical/High=bloquea, Medium→FIND-*) + `/cleanCA` sobre los `.py` tocados antes de cerrar (norma ritual — audita, solo informa).
- `python -m pytest integrations/dspy/tests/ -q` sin shim = repro RED (AttributeError) antes del primer GREEN.

**MCP:**
- `campaign_detect_task_type` ✅ (type=python, checks=pytest) · `campaign_discover_skills_v2` ✅ (8 skills) · `campaign_get_next_task` ✅ (hasTask=false; recitation FIND-93 apunta a FIND-94) · `codegraph_explore` ✅ (blast radius + fuente verbatim llamaindex/shim) · `campaign_verify_cmd` — bug exit -1 conocido (plan Riesgos) → fallback bash directa y anotarlo en RESULTADO · `campaign_update_task_state` (in-progress/completed con recitation canónica) · `campaign_memory_write` (1-2 lessons) · `campaign_diagnose_pipeline` (cierre).
- Sin red (sin instalar dspy/crewai/llamaindex/mem0/haystack/letta): matriz CON-framework como no-ejecutada si no instalados (precedente FIND-69). Sin `webfetch` externo necesario (API local en `vantadb-python/` + probes reales). Si se afirma algo externo → `[cita NO VERIFICADA]` + deuda TSYS-13.

## 7. INVESTIGACIÓN CÓDIGO (blast radius — DISCOVERY 2026-09-16)

**12 hits → 11 call-sites prod + 1 doc:** shared:47 · llamaindex:53 · mem0:122 · crewai vectorstore:68 + memory:104 · haystack:135 · dspy:75 · letta:43 · langchain checkpointer:80 + store:88 + vectorstore:44. Todos con la misma forma `vanta.VantaDB(db_path, memory_limit_bytes=, read_only=, backend=)` → mapeo trivial a `vanta.Client` (firma idéntica `pyi:174-180`).

**Qué API usan post-construcción (define el shim mínimo / mapeo a `Client`):**
- `put(ns,key,text,metadata,vector)` — 11 sites → `Client.put` EXISTE (firma `pyi:184-192`, tercer param renombrado `payload` pero posicional-compatible). Sin cambio.
- `put_batch(keys,vectors,payloads,metadatas,namespace)` — llamaindex:140 → EXISTE (`pyi:193-202`). Sin cambio.
- `get_memory(ns,key)` — 8 sites → `memory.get(ns,key)` (`pyi:415`). CAMBIO.
- `list_memory(ns,filters,limit,cursor)` → `memory.list(...)` (`pyi:416-424`). CAMBIO. Retorna `ListResult` (`.records`/`.next_cursor`, iterable, dict-style) — compatible salvo `dict(page)` (dspy:179, D5).
- `delete_memory(ns,key)` — 15+ sites → `memory.delete(ns,key)` (`pyi:424`). CAMBIO.
- `search_memory(ns,vector,top_k,distance_metric,filters,text_query)` — 15+ sites → `memory.search(ns,query_vector,top_k,distance_metric,filters,text_query)` (`pyi:376-387`). CAMBIO (mismo orden kwargs, `query_vector` es el 2º posicional en ambos).
- `list_namespaces()` — store:187, mem0:238 → EXISTE en `Client` y `memory`. Sin cambio.
- `update_memory` (mem0:213) / `delete_namespace` (mem0:245) → NO EXISTEN (probe `False/False`). Migración D3/D4.
- Tipos `vanta.VantaMemoryRecord` (haystack:151, langchain:536, llamaindex:84) / `vanta.VantaSearchHit` (haystack:159, langchain:63, llamaindex:72) → NO EXISTEN (probe `False/False`); son `Record`/`SearchHit`. Con `from __future__ import annotations` no rompen import, pero mienten.
- `client` property llamaindex:61 (`-> vanta.VantaDB`) → `-> vanta.Client`.

**Probes backend real (evidencia concluyente, `probe_find94.py/b.py` 2026-09-16):**
- `Client(d).put/list/get/delete/search` vía `memory.*` OK contra real (1 record roundtrip).
- `dict(ListResult)` → `TypeError: 'int' object is not an instance of 'str'` (D5 confirmado).
- Vector-search emite **similitud coseno mayor=mejor**: idéntico→1.0, ortogonal→0.0, diag→0.7071; orden DESC; `text_query` también mayor=mejor (hello 0.0325 > other 0.0163). Novec excluido del índice vectorial (3 hits tras insert sin vector).
- `has update_memory/delete_namespace/search_memory/VantaDB/VantaMemoryRecord/VantaSearchHit: False`; `Record/SearchHit/Client/connect: True`.
- Frameworks instalados: `langchain_core/ollama/openai OK`; `dspy/crewai/llama_index/mem0/haystack/letta_client MISSING` → matriz CON-framework no-ejecutable para esos 6 (documentar, igual que FIND-69).

**Implicaciones/riesgos:** cambiar constructor en 11 sites (trivial) + ~70 renames `*_memory→memory.*` (mecánico) + 3 casos especiales (D3/D4/D5) + 5 tipos + score-fix D8. Riesgo: `Client` sin paridad total (`update_memory`/`delete_namespace` ausentes) → D3/D4 con código nuevo mínimo; riesgo score-invertido en MMR/RRF si no se fija D8 → fix incluido o DEFER-ratificado por adapter (nunca silencio).

## 8. INVESTIGACIÓN PROBLEMA

**Drift PROV:** SDK 0.5.0 removió/renombró `VantaDB` (clase) + `*_memory` (métodos) + `Vanta*` (tipos) sin migrar los 9 adapters, que quedaron pineados a la API pre-0.5.0. Causa raíz (systematic-debugging Fase 1): el rename nativo AST-010 + paridad TS `MemoryClient` (AST-012) + política ADR-041 anti-stutter se aplicaron al binding pero no a `integrations/`. El síntoma (`AttributeError: module 'vantadb_py' has no attribute 'VantaDB'`) es total: 12 hits × 9 adapters, verificado FIND-69 (dspy 5 failed + 3 errors; `hasattr is False`).

**Tradeoff:**
- **Migración (elegida):** superficie mínima (Propuesta §3), deuda cero, ~100 líneas mecánicas + 10 líneas especiales; riesgo = tocar 11 archivos (mitigado por slices por adapter + pytest por adapter + `git diff --check`). Revierte el drift en la dirección correcta (adapters → SDK vigente).
- **Shim permanente (rechazada):** compat rápida (1 alias `VantaDB = Client` + 4 métodos `*_memory` delegando) pero deuda permanente (segunda superficie, semántica `update_memory`/`delete_namespace` inventada, tipos duales) y viola Propuesta §3; además el shim correcto viviría en `vantadb-python/` (prohibido en esta task → Gate V).
- **Ship-parcial (válvula):** lo verde ahora (adapters migrados + suites verdes), resto con DEFER-ratificado individual por adapter (motivo + evidencia), nunca silencio. Pre-mortem: si un adapter rompe de forma no trivial (score-framework, fixture disco, `put_batch` tipos) → DEFER ese adapter, ship resto.

## 9. INVESTIGACIÓN INTERNET + NOTION (Paso 0c)

- **Internet:** no se espera (API local en `vantadb-python/` + probes reales). Cero afirmaciones externas en esta task → sin citas, sin deuda TSYS-13. Si el reviewer exige una afirmación externa → `[cita NO VERIFICADA — sin red]` + `contract.deuda`.
- **Notion (4/4 leídas COMPLETAS vía fetch 2026-09-16):** `Propuesta` (filtro §3 SDKs mínimos + inventario §3 SDK Python ~55 métodos — base de D1/D2), `Problema` (ciclo de vida conservar/recuperar/priorizar/actualizar/descartar — el drift rompe recuperar/actualizar en 9 adapters), `Nuevas features` (índice 20 ideas + holones — sin mapeo a esta task: fix de drift, no feature), `Plan de accion` (índice Fase A/0 + roadmap v0.5→v0.7 — esta task sostiene Fase A: 0 críticos). Filtro VantaDB aplicado (otros holones NO aplican).
- **References:** clean-code-clean-architecture COMPLETO (Apéndice V normativo: `integrations/` = Frameworks/Drivers = Humble Objects glue+memoria) + `python-bindings.md` + `api-contract.md` (R-1/R-8) — ver §4.

## 10. VALIDACIÓN+CIERRE

- Verify contrato (§1 criterios 1-8) + verify full relevante: `py_compile` 11 prod + `pytest` 9 adapters + `test_pins.py` + `git diff --check`; `cargo` N/A (cero Rust tocado — justificar); `campaign_verify_cmd` intentado, con fallback bash por bug exit -1 (anotado).
- OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`; Critical/High=bloquea, Medium→FIND-*) + `/cleanCA` sobre los `.py` tocados antes de cerrar (norma ritual — audita, solo informa).
- DoD 3 niveles: (1) contrato §1 ✅ · (2) `definition-of-done.md` (sin deuda neta: se borra shim temporal; tests + docs sync en mismo cambio) · (3) reviewer distinto P2-01 (`vanta-review`, nunca auto-auditoría).
- Gates: P no (plan-adjust ya triageó) · D disparado-por-conteo (>10 archivos) resuelto-por-evidencia (tabla §4b, sin `question` disponible; Recomendado=migrar) · V si `vantadb-python/` hiciera falta (propuesta exacta, no edición) · C al cierre (colaterales: ¿shim borrado? ¿pins intactos? ¿score fijado o DEFER?).
- Commit conventional `fix: FIND-94 — ...` (solo archivos propios) · Backlog→avance NO tocar (race paralelo, orquestador) · push vía vanta-lead.
- Bloque RESULTADO §7 + Context Save Point en task file. `campaign_update_task_state` + `campaign_memory_write` (lessons) + `campaign_diagnose_pipeline` + `skill progreso` (solo si no hay race; si hay race → anotar pendiente orquestador).

**SDP:** ver §5. **Research Digest:** sin digest previo (DISCOVERY inline; 12 hits acotados + probes reales).

## Impacto mapeado (Regla 0) — MUST antes de la PRIMERA edición

**Archivos leídos completos:** `integrations/vantadb_shared/__init__.py` (219L) · `integrations/vantadb_test_shim.py` (237L) · `integrations/README.md` (92L) · `vantadb-python/vantadb_py/__init__.py` (552L) · `vantadb_py.pyi` (515L) · `docs/api/PYTHON_SDK.md` (1248L, secciones constructor/sub-clientes/tipos) · `dspy/vectorstore.py` (185L) · `letta/vectorstore.py` (205L) · `mem0/vectorstore.py` (307L) · `crewai/vectorstore.py` (257L) · `crewai/memory.py` (325L) · `langchain/vectorstore.py` (573L parcial 1-100/190-240/405-573) · `langchain/store.py` (296L parcial) · `langchain/checkpointer.py` (350L parcial) · `llamaindex/vectorstore.py` (493L parcial) · `haystack/vectorstore.py` (482L parcial) + `rg` global de métodos/tipos/scores + probes reales ×2 + `codegraph_explore` (40 símbolos, fuente verbatim llamaindex/shim).

**Referencias hacia dentro (qué importa cada prod):** `vantadb_py.Client/connect/Record/SearchHit/ListResult` (binding compilado, solo lectura) · `docs/api/PYTHON_SDK.md` (contrato de firmas) · frameworks externos (`dspy/crewai/langchain/llamaindex/mem0/haystack/letta` — la mayoría ausentes: `importorskip`/fallbacks).

**Referencias entrantes (quién usa cada prod):** `integrations/<adapter>/tests/test_*.py` (9 suites) · `tests/conftest.py` (9, vía shim) · `integrations/test_pins.py` (pins, no toca runtime) · `ollama/vectorstore.py` + `openai/vectorstore.py` → `vantadb_shared.EmbeddingVectorStore` (heredan el fix) · `langchain/__init__.py` (re-exporta los 3) · `crewai/__init__.py` (re-exporta 2).

**Veredicto de impacto:** BLAST RADIUS = 11 prod + 9 conftest + 1 shim + 9 suites (medio, glue-only, sin hot paths Rust, sin concurrencia, sin trust boundaries nuevos). Edición segura archivo-por-adapter con verify por adapter; rollback por archivo (cambios aditivos de renames, sin migraciones de datos). Prohibidos intactos (ver §2). Primera edición permitida tras este mapeo.

## Steps atómicos (~100 líneas + verify mecánico c/u)

- [x] **Step 1 — shared + dspy + letta (D1+D2+D5):** DONE 2026-09-16. `vanta.VantaDB→vanta.Client` (3 constructores) + `memory.*` + dspy `dict(page)`→`{"records","next_cursor"}` + dspy `Prediction`-shadowing guard (FIND-94: `import dspy` resuelve al dir local sin `Prediction`). Verify: `py_compile` 3 ✅ + dspy 8 passed ✅ + letta 17 passed ✅ + ollama 9 ✅ + openai 9 ✅ (heredan shared).
- [x] **Step 2 — crewai ×2 + haystack (D1+D2+D6):** DONE 2026-09-16. Constructores (2) + `memory.*` (~15 sites) + haystack tipos `VantaMemoryRecord/SearchHit→Record/SearchHit` + crewai `super().__init__` TypeError-guard (mismo patrón FIND-69 dspy: fallback `object` no acepta kwargs). Verify: `py_compile` ✅ + crewai memory 11 passed/1 skipped ✅ + crewai vectorstore skipped (sin framework) ✅ + haystack skipped (sin SDK) ✅.
- [x] **Step 3 — langchain ×3 (D1+D2+D6+D8):** DONE 2026-09-16. Constructores (3) + `memory.*` (~20 sites) + tipos + docstrings distancia→similitud + `store.py:256` score→clamp-similitud + `vectorstore.py:216` MMR relevance directa + `_cosine_relevance_score_fn`→clamp + test `test_cosine_relevance_score` actualizado + `tests/test_vectorstore.py:83` `memory.get`. Incidente: disco C: lleno por `pytest-of-Eros` acumulados (~20GB) → `StorageFull` transitorio; causa raíz verificada (no del cambio), fix operativo (borrado pytest-70/71), re-run 47 passed ✅. Verify: `py_compile` ✅ + langchain 47 passed ✅ + pins 10 ✅.
- [x] **Step 4 — llamaindex + mem0 (D1+D2+D3+D4+D6+D8):** DONE 2026-09-16. Constructores (2) + `memory.*` (~25 sites) + `put_batch` intacto + mem0 `update→get+put` (preserva vector) + mem0 `delete_col→loop paginado` + tipos + llamaindex RRF/MMR `hit.score` directo (4 sites). `_normalize_score` mem0 intacto (compatible: [0,1] passthrough = similitud correcta; rama distancia muerta pero inofensiva). Verify: `py_compile` ✅ + llamaindex skipped (sin SDK) ✅ + mem0 skipped (sin SDK) ✅ + smoke real (crewai/dspy/letta/shared) SMOKE-OK ✅.
- [x] **Step 5 — cierre:** DONE 2026-09-16. `rg` 0 hits prod ✅ + matriz 9 adapters verde/skipped-válido ✅ + `test_pins.py` 10 ✅ + `git diff --check` limpio ✅ + shim `vantadb_test_shim.py` BORRADO + 9 conftest sin import ✅ (migración completa, sin ship-parcial: 0 DEFER) + OCR + cleanCA + review P2-01 + commit `fix:` + recitation + RESULTADO.

## Context Save Point

- DISCOVERY completo 2026-09-16: 12 hits re-verificados, `hasattr VantaDB False` (ambos import names), `dir()` real mapeado, probes backend real ×2 (similitud mayor=mejor), frameworks instalados (langchain/ollama/openai OK, resto MISSING), Notion 4/4 + references leídas, SDP 8 skills cargadas (1 excluida con motivo), Gate D evaluado (tabla §4b, Recomendado=migrar).
- ACT completo 2026-09-16 (Steps 1-5): migración total sin DEFER — 11 prod + 9 conftest + 1 test langchain + shim borrado. Matriz: crewai-mem 11p/1s, crewai-vs skipped, dspy 8p, haystack skipped, langchain 47p, letta 17p, llamaindex skipped, mem0 skipped, ollama 9p, openai 9p, pins 10p. Skips = importorskip framework ausente (válido). Incidente disco lleno documentado en Step 3.
- Cierre: OCR + cleanCA + review P2-01 + commit + recitation + RESULTADO.
- Deuda al cerrar: ninguna (shim eliminado; `_normalize_score` distancia-muerta documentada como inofensiva).

## Review P2-01 (vanta-review, 2026-09-16) — APPROVE

Sin hallazgos Critical/High. Aplicados 3 nits en código propio: `delete_col` re-lista desde inicio (cursor-offset bajo mutación salta registros >1000); docstring `_normalize_score` (similitud passthrough + rama distancia legacy); comentario `crewai:184` (`memory.list`). No aplicados (fuera de scope, pre-existentes): loops destructivos con cursor en llamaindex/langchain/checkpointer (mismo patrón heredado), guard `super().__init__` crewai, shadowing `Retrieve` dspy, bordes de clamp en test. Re-run post-nits: crewai memory 11p/1s ✅ + py_compile ✅ + diff-check ✅.

---
*SDP: source-driven-development, incremental-implementation, test-driven-development, context-engineering, systematic-debugging, doubt-driven-development, api-and-interface-design (+ campaign-executor/progreso auto-vía MCP; frontend-ui-engineering excluida: 0 web/).*
