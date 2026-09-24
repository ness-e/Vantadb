# Plan de Ejecución: Seguimiento MVP (DEFER + pendientes) — 2026-09-17

> **Campaign ID:** 64985e0b-0570-431c-a1e9-1d0d551ad54e
> **Inicio:** 2026-09-17
> **Estado:** ✅ COMPLETADO (9/9, 2026-09-18)
> **Fuente:** `docs/dev/Backlog.md` (FIND-101/102/109/110/111/112/113/114/115) + decisiones owner FIND-112 vía `question` 2026-09-17
> **Autonomous:** false
> **FAIL_MODE:** `parallel` (MAX 3; secuencial interno si colisionan archivos)
> **Gate P:** heredado — owner aprobó set de 9 + alcance FIND-112 (spec, matriz local+ollama/openai, TOML+env, wiki+pipeline) en esta sesión.
> **SDP (plan):** base fija abajo; cada sub-agente ejecuta `campaign_discover_skills_v2` (phase PLAN→BUILD) en pipeline-full Paso 0b y declara `SKILLS_CARGADAS:`.

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 9 (FIND-101/102/109/110/111/112/113/114/115) |
| 🟡 DEFER | resto del Backlog (programas enteros, fuera de este seguimiento) |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 (FIND-112 era el candidato; resuelto como spec) |

Status: ⬆️ uphill = 3 (diseño lifecycle S4 · dueño backend S6b · alcance salvage 109) · ⬇️ downhill = 9 tasks con contrato definido
**Implementación IMPL-112 (runner real por etapas) explícitamente FUERA de este plan** — ver § siguiente plan.

## Implementación siguiente plan (placeholder, NO es tarea de este plan)

- **IMPL-112 — runner real de ingesta por etapas.** Trigger: FIND-112 spec cerrada + P2-01 sobre la spec. Etapas: (1) runner local (cero secrets, cierra degradado P4), (2) Ollama/OpenAI si el trait los absorbe sin fricción. Si la spec muestra que ni el caso local tiene consumidor → la spec queda como decisión registrada y la implementación espera demanda (cierre válido, ponytail).

## Tasks

### Wave0 — durabilidad + exposición diferida (disjuntos: wal/storage/cli · approval · skill)

**Task 1: FIND-109 — `vanta-cli wal salvage` + error tipado en MCP**
- **Appetite:** 2d · **Esfuerzo:** 🟡 · **Prioridad:** 🔴 Alta · **Archivos clave:** `src/wal_sharded.rs:69-94,281`, `src/storage/engine/init.rs:413,486-490`, `src/cli_handlers/wal.rs`, `src/cli_handlers/server.rs:253`
- **Verificación real:** ✅ CÓDIGO-REAL — `verify_shard_counts` aborta open read-write (shard 1: 39 vs 40 requeridos, caida real 2026-09-17 `C:/Users/Eros/.vantadb`); solo `compact`/`vacuum` existen y ambos abren read-write; solo-lectura (`doctor`, `export`) sí abre. Fixture real disponible: `.vantadb-corrupt-20260917` + `.vantadb.bak-20260917` (fuera del repo, NO tocar sin backup).
- **Gate Justificación:** MCP hard-down sin vía de reparación es el riesgo Alta más barato de cerrar; fix acotado a wal+cli+mcp-error.
- **Contrato:** fixture WAL truncado en Temp → `wal salvage` replaya lo coherente + reporte explícito de descartados (cero skip silencioso) + exit 0; MCP devuelve error tipado en vez de exit 1 (o documentado por qué no con evidencia); suite `wal` verde + clippy `-D warnings` 0.
- **Pre-mortem:** (1) replay coherente ambiguo en colas round-robin → definir "coherente" en la spec del slice antes de codificar; (2) tocar ERR-011 debilita el guard → el guard sigue abortando por defecto, salvage es comando explícito opt-in.
- **Stop:** appetite >2d → ship solo-lectura + DEFER resto; premisa invalidada (fixture no reproduce) → re-evaluar gate.
- **Risk Register:** 🟡×🔴 debilitar ERR-011 → salvage opt-in, guard intacto | 🟡×🟡 fixture no reproduce → usar copia del backup real en Temp.
- **Cynefin:** 🟨 complicado (WAL/round-robin). **Top 3:** definición de coherente / no debilitar guard / fixture fiel.
- ⬆️ 1 (alcance exacto salvage) / ⬇️ 3 steps. **DoD:** contrato + task file sync + recitation; commit `feat:`; sin deuda.
- **Skills (≤8):** systematic-debugging (repro con fixture) · test-driven-development (RED fixture→GREEN salvage) · codebase-memory (blast radius wal) · doubt-driven-development (guard ERR-011) · incremental-implementation (+ base campaign-executor/progreso/ponytail).
- **Herramientas+MCP:** codegraph + codebase-memory-mcp + `cargo test -p vantadb wal -j 2` + clippy + `campaign_verify_cmd` (bug exit -1 → bash) + `vanta-cli wal salvage` smoke en Temp.
- **Referencias:** rules `durability.md` + `core-engine.md` · refs `definition-of-done.md`, `clean-code-clean-architecture.md` Ap. V, `dev-tools.md`, `test-suite.md` · commands `pipeline.md`, `audit.md` · agents `vanta-worker` + `vanta-review` P2-01.
- Task file `docs/dev/tasks/FIND-109.md` · ⬜ PENDING · Ruta vanta-worker. Branch develop. Commit `feat: FIND-109 — ...`.

**Task 2: FIND-110 — S4 bandeja de aprobación (diseño lifecycle + ship o re-DEFER)**
- **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟡 Media · **Archivos clave:** `vanta-memory/src/core/record/approval.rs:9,64-72`, `vanta-memory/src/gateway/approval_handlers.rs:90-118`
- **Verificación real:** ✅ CÓDIGO-REAL — queue in-memory por proceso; MCP stdio stateless → mostrador vacío entre procesos (evidencia FIND-107 §7).
- **Gate Justificación:** cierra S4 de FIND-107: o hay lifecycle con test en-proceso, o re-DEFER fundado (no mostrador vacío).
- **Contrato:** decisión lifecycle escrita (dueño del queue en proceso MCP o persistencia) + test en-proceso verde que prueba captura→decisión en mismo proceso + tools/docs si shippea; si no cierra → fila Backlog actualizada con motivo (re-DEFER válido).
- **Pre-mortem:** (1) persistencia tienta → fuera de appetite, solo lifecycle-proceso o re-DEFER; (2) test en-proceso flaky → determinista sin timing.
- **Stop:** sin lifecycle defendible en 1d → re-DEFER con motivo (no forzar mostrador vacío).
- **Risk Register:** 🟡×🟢 scope-creep a persistencia → prohibido por contrato | 🟢×🟡 flaky → asserts contenido.
- **Cynefin:** 🟨 complicado. **Top 3:** dueño del queue / test determinista / no-mostardor-vacío.
- ⬆️ 1 (diseño lifecycle) / ⬇️ 2 steps. **DoD:** contrato + task file + recitation; commit `feat:` o `docs:` (si re-DEFER, commit del task file); sin deuda.
- **Skills:** spec-driven-development (decisión lifecycle) · test-driven-development · doubt-driven-development · codebase-memory (+ base).
- **Herramientas+MCP:** codegraph + `cargo test -p vanta-memory -j 2` + `campaign_verify_cmd`.
- **Referencias:** rules `core-engine.md`, `api-contract.md` (R-8: glue, no lógica) · refs `definition-of-done.md` · agents `vanta-worker` + `vanta-review` P2-01.
- Task file `docs/dev/tasks/FIND-110.md` · ⬜ PENDING · Ruta vanta-worker. Branch develop. Commit `feat: FIND-110 — ...`.

**Task 3: FIND-111 — S5 `skill_extract` solo-candidatos (o re-DEFER)**
- **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟡 Media · **Archivos clave:** `vanta-memory/src/core/skill/` (`skill_extractor.rs:209`), `vantadb-mcp/src/handlers/`
- **Verificación real:** ✅ CÓDIGO-REAL — `run_skill_extract_once`/`extract_skills_with_llm` exigen `LlmRunner` sin path degradado (evidencia FIND-107 §7).
- **Gate Justificación:** exponer como solo-candidatos read-only es seguro y cierra S5 sin runner; si exige dependencia → re-DEFER.
- **Contrato:** tool `skill_extract` read-only (candidatos, sin sink de escritura) + degrada documentada sin runner (`{success:false, candidates:[], error}`) + test + fila MCP.md + coverage 0 gaps; o re-DEFER con motivo.
- **Pre-mortem:** (1) sink con escritura tienta → slice separado, prohibido acá; (2) runner genérico sin degradar → DEFER, no forzar.
- **Stop:** sin path degradado limpio → re-DEFER (no bloquea).
- **Risk Register:** 🟡×🟢 escritura accidental → read-only por contrato | 🟢×🟡 runner → DEFER puntual.
- **Cynefin:** 🟨 complicado. **Top 3:** read-only real / degrada honesta / scope (sin sink).
- ⬆️ 0 / ⬇️ 2 steps. **DoD:** contrato + task file + recitation; commit `feat:`; sin deuda.
- **Skills:** api-and-interface-design (schema) · test-driven-development · mcp-builder (+ base).
- **Herramientas+MCP:** codegraph + `cargo test -p vantadb-mcp -j 2` + MCP stdio smoke + coverage ps1.
- **Referencias:** rules `api-contract.md`, `server-mcp.md` · agents `vanta-worker` + `vanta-review` P2-01.
- Task file `docs/dev/tasks/FIND-111.md` · ⬜ PENDING · Ruta vanta-worker. Branch develop. Commit `feat: FIND-111 — ...`.

### Wave1 — spec ingesta + scheduler + CLI (disjuntos: docs/tasks · services · cli)

**Task 4: FIND-112 — spec + diseño del runner real de ingesta (SIN código)**
- **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟡 Media · **Archivos clave:** `vantadb-mcp/src/wiki.rs:274`, `vanta-memory/src/ingest/` (`worker.rs`, `mod.rs`), `vanta-memory/src/wiki.rs:399-467` (lectura)
- **Verificación real:** ✅ CÓDIGO-REAL — `start_ingest::<NoLlm>` fijo = sources skipped (P4); pipeline serial en `ingest/worker.rs` (evidencia FIND-107 §7).
- **Gate Justificación:** decisiones owner cerradas vía `question` 2026-09-17; la spec convierte uphill en slice mecánico para IMPL-112.
- **Contrato:** `docs/dev/tasks/FIND-112.md` spec con 5 puntos (trait runner + matriz / schema TOML+env / lifecycle-dueño / gates+tests futuros / qué sigue degradado) + P2-01 sobre la spec; **cero archivos de código tocados** (solo task file; Backlog ya refleja decisiones).
- **Decisiones owner (cerradas, no re-derivar):** spec sin código · matriz local+ollama/openai (reusar EMB) · TOML mínima + env (secrets solo env) · superficie wiki_ingest + pipeline general.
- **Pre-mortem:** (1) matriz ×3 explota diseño → tabla variante×(transporte/config/secret/test) obligatoria; (2) dueño del runner difuso → la spec no cierra sin dueño explícito.
- **Stop:** desacuerdo en dueño tras 2 rondas → Gate V; spec que no cierra en 1d → DEFER con lo avanzado.
- **Risk Register:** 🟡×🟡 matriz explota → tabla obligatoria | 🟡×🟢 lifecycle difuso → dueño explícito o no cierra.
- **Cynefin:** 🟨 complicado. **Top 3:** tabla matriz / dueño / qué-queda-degradado.
- ⬆️ 1 (dueño del runner) / ⬇️ 2 steps. **DoD:** spec 5 puntos + P2-01-spec + recitation; commit `docs:`; sin deuda.
- **Skills:** spec-driven-development (núcleo) · documentation-and-adrs · doubt-driven-development · api-and-interface-design (schema config) (+ base).
- **Herramientas+MCP:** codegraph (lectura) + notion (fetch dim memoria si mapea, filtro VantaDB); internet solo si ambigüedad de diseño (patrones Mem0/Letta).
- **Referencias:** rules `core-engine.md`, `api-contract.md` · refs `definition-of-done.md`, `architecture.md` · agents `vanta-worker` + `vanta-review` P2-01 (sobre la spec).
- Task file `docs/dev/tasks/FIND-112.md` · ⬜ PENDING · Ruta vanta-worker. Branch develop. Commit `docs: FIND-112 — ...`.

**Task 5: FIND-113 — S6b programador (dueño del backend + diseño o re-DEFER)**
- **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟡 Media · **Archivos clave:** `vanta-memory/src/services/pipeline_worker.rs:1-120` (`run_once`, `TaskKind::Dream`)
- **Verificación real:** ✅ CÓDIGO-REAL — `run_once` + `LocalStateBackend` + locks TTL existen; sin dueño en proceso MCP (evidencia FIND-107 §7).
- **Gate Justificación:** espejo de FIND-110: o hay dueño con test, o re-DEFER fundado.
- **Contrato:** dueño del backend decidido y escrito (dir estado, quién reclama locks) + `scheduler_status`/`scheduler_run_once` con test o slice mínimo + docs; o re-DEFER con motivo.
- **Pre-mortem:** (1) locks TTL en MCP multi-proceso → definir scope-proceso o re-DEFER; (2) `TaskKind::Dream` ya cableado MEM-65 → no duplicar.
- **Stop:** sin dueño defendible en 1d → re-DEFER (no forzar).
- **Risk Register:** 🟡×🟡 locks multi-proceso → scope-proceso o DEFER | 🟢×🟢 duplicar MEM-65 → reusar.
- **Cynefin:** 🟨 complicado. **Top 3:** dueño / locks / no-duplicar.
- ⬆️ 1 (dueño) / ⬇️ 2 steps. **DoD:** contrato + task file + recitation; commit `feat:`; sin deuda.
- **Skills:** spec-driven-development · test-driven-development · codebase-memory (+ base).
- **Herramientas+MCP:** codegraph + `cargo test -p vanta-memory -j 2`.
- **Referencias:** rules `core-engine.md` · agents `vanta-worker` + `vanta-review` P2-01.
- Task file `docs/dev/tasks/FIND-113.md` · ⬜ PENDING · Ruta vanta-worker. Branch develop. Commit `feat: FIND-113 — ...`.

**Task 6: FIND-101 — `vanta-cli query` doc-vs-fix (INSERT/UPDATE/DELETE)**
- **Appetite:** max 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🟢 Baja · **Archivos clave:** `src/cli_handlers/data.rs:212`, `src/bin/vanta-cli.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `query` abre read-only (evidencia EMB-10 `docs/dev/tasks/EMB-10.md:173`).
- **Gate Justificación:** tiny con decisión pendiente; se decide y se hace en el mismo slice.
- **Contrato:** doc-vs-fix decidido (documentar límite o abrir read-write para IQL mutante) + implementado + `--help` coherente.
- **Pre-mortem:** (1) abrir read-write rompe invariante CLI → si hay invariante, gana doc; (2) scope-creep a IQL completo → prohibido.
- **Stop:** implica rediseño CLI → DEFER con motivo (no rediseñar en 1h).
- **Risk Register:** 🟢×🟡 invariante read-only → doc gana | 🟢×🟢 creep → prohibido.
- **Cynefin:** 🟦 obvio. **Top 3:** decisión doc-vs-fix / help coherente / sin creep.
- ⬆️ 0 / ⬇️ 1 step. **DoD:** contrato + task file mínimo + recitation; commit `fix:`/`docs:`; sin deuda.
- **Skills:** documentation-and-adrs (+ base).
- **Herramientas+MCP:** `vanta-cli query --help` + `campaign_verify_cmd`.
- **Referencias:** rules `core-engine.md` · agents `vanta-worker` + `vanta-review` P2-01.
- Task file `docs/dev/tasks/FIND-101.md` · ⬜ PENDING · Ruta vanta-worker. Branch develop. Commit `fix: FIND-101 — ...`.

### Wave2 — quick wins (disjuntos: tests · examples · docs/web)

**Task 7: FIND-102 — `tests/sdk_serialization.rs` compila**
- **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟢 Baja · **Archivos clave:** `tests/sdk_serialization.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — 39 errores, `QueryResult` no declarado (evidencia EMB-10 `:174`); archivo existe (verificado 2026-09-17).
- **Gate Justificación:** test roto pre-existente ensucia `cargo check --tests`; fix mecánico probable (import/tipo).
- **Contrato:** `cargo check -p vantadb --tests` verde (o el scope mínimo que incluya ese archivo) + sin cambiar asserts existentes salvo que estén mal (y entonces se documenta por qué).
- **Pre-mortem:** (1) 39 errores esconden rediseño → si no es mecánico en 1d, DEFER con diagnóstico; (2) "arreglar" borrando tests → prohibido (Regla 2).
- **Stop:** no-mecánico → DEFER con diagnóstico (no reescribir suite).
- **Risk Register:** 🟡×🟡 rediseño escondido → DEFER | 🔴×🟢 borrar tests → prohibido.
- **Cynefin:** 🟨 complicado (hasta ver los 39). **Top 3:** mecánico-vs-rediseño / no-borrar / check verde.
- ⬆️ 1 (naturaleza de los 39) / ⬇️ 1 step. **DoD:** contrato + task file mínimo + recitation; commit `test:`; sin deuda.
- **Skills:** systematic-debugging · test-driven-development (+ base).
- **Herramientas+MCP:** `cargo check -p vantadb --tests -j 2`.
- **Referencias:** refs `test-suite.md` · agents `vanta-worker` + `vanta-review` P2-01.
- Task file `docs/dev/tasks/FIND-102.md` · ⬜ PENDING · Ruta vanta-worker. Branch develop. Commit `test: FIND-102 — ...`.

**Task 8: FIND-114 — migrar `agent_memory.py` a `Client`**
- **Appetite:** max 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🟢 Baja · **Archivos clave:** `examples/python/agent_memory.py:5,13`
- **Verificación real:** ✅ CÓDIGO-RESTA-VERIFICAR — `import vantadb_py as vantadb` + `vantadb.VantaDB(...)` (verificado 2026-09-17); patrón `Client` en `test_sdk.py` + `agent_memory_demo.py`.
- **Gate Justificación:** ejemplo legacy confunde (dos APIs); migración mecánica al patrón SHOW-04.
- **Contrato:** ejemplo usa `Client` + corre verde (pytest del ejemplo si existe, o smoke del script).
- **Pre-mortem:** (1) API legacy con semántica distinta → mapear 1:1 antes de cambiar; (2) sin test del ejemplo → smoke manual + nota.
- **Stop:** semántica no-mapeable en 1h → DEFER con diagnóstico.
- **Risk Register:** 🟢×🟡 semántica → mapear primero | 🟢×🟢 sin test → smoke.
- **Cynefin:** 🟦 obvio. **Top 3:** mapeo 1:1 / smoke verde / sin cambiar comportamiento.
- ⬆️ 0 / ⬇️ 1 step. **DoD:** contrato + task file mínimo + recitation; commit `refactor:`; sin deuda.
- **Skills:** incremental-implementation (+ base).
- **Herramientas+MCP:** pytest (venv auditoría o system) + `campaign_verify_cmd`.
- **Referencias:** rules `python-bindings.md` · agents `vanta-worker` + `vanta-review` P2-01.
- Task file `docs/dev/tasks/FIND-114.md` · ⬜ PENDING · Ruta vanta-worker. Branch develop. Commit `refactor: FIND-114 — ...`.

**Task 9: FIND-115 — sync one-liner viejo (README_ES + docs-view)**
- **Appetite:** max 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🟢 Baja · **Archivos clave:** `README_ES.md:250,256`, `web/` docs-view (`docs-view.tsx:263,270` por verificar path exacto en DISCOVERY)
- **Verificación real:** 🟡 VERIFICAR — README_ES confirmado con one-liner viejo (2026-09-17); path web por confirmar en DISCOVERY (puede haber renombrado → SKIP parcial de esa superficie).
- **Gate Justificación:** tres superficies con instrucciones distintas rompen confianza en instalación; fix = unificar o referenciar.
- **Contrato:** las superficies existentes muestran el one-liner FIND-105 o referencian a la fuente única + `validate-docs-coverage.ps1` 0 gaps.
- **Pre-mortem:** (1) path web renombrado → SKIP parcial con evidencia, no cazar fantasma; (2) traducciones ES desincronizadas → solo el one-liner, no re-traducir.
- **Stop:** superficie inexistente → SKIP parcial (no cazar).
- **Risk Register:** 🟢×🟢 path fantasma → SKIP parcial | 🟢×🟢 re-traducción → prohibida.
- **Cynefin:** 🟦 obvio. **Top 3:** fuente única / path real / coverage 0 gaps.
- ⬆️ 0 / ⬇️ 1 step. **DoD:** contrato + task file mínimo + recitation; commit `docs:`; sin deuda.
- **Skills:** documentation-and-adrs (+ base).
- **Herramientas+MCP:** `validate-docs-coverage.ps1` + `campaign_verify_cmd`.
- **Referencias:** agents `vanta-worker` + `vanta-review` P2-01.
- Task file `docs/dev/tasks/FIND-115.md` · ⬜ PENDING · Ruta vanta-worker. Branch develop. Commit `docs: FIND-115 — ...`.

## SKIP

Ninguno (las 9 tienen gap real verificado; el resto del Backlog no se triagió en este plan enfocado).

## DEFER (fuera de este seguimiento)

Programas enteros del Backlog no incluidos (mismo criterio que el plan MVP): MGR/EXE/FUT/STU/UX/TS/WSM/RES/AGT/GOV/CI-01/SHOW-02-03/DESKTOP/OLD/DISC/DEC + resto FIND (ninguno: todos los FIND-MVP están en este plan o cerrados).

## BLOQUEADO

Nada (FIND-112 era el candidato; resuelto como spec con decisiones owner cerradas).

## Grafo de dependencias / Waves (FAIL_MODE=parallel, MAX 3)

```
Wave0: FIND-109 + FIND-110 + FIND-111   (wal · approval · skill — disjuntos)
Wave1: FIND-112 + FIND-113 + FIND-101   (spec-doc · scheduler · cli-query — disjuntos)
Wave2: FIND-102 + FIND-114 + FIND-115   (tests · examples · docs/web — disjuntos)
```

IMPL-112 vive en el plan subsiguiente (trigger: spec FIND-112 cerrada). FIND-110/111/113 pueden re-DEFER puntual con motivo (stop rules) sin bloquear su wave.

## Riesgos globales

| Riesgo | Respuesta |
|--------|-----------|
| Mismo archivo en una wave | waves separan por área (wal vs approval vs skill; spec vs scheduler vs cli; tests vs examples vs docs); si colisiona → secuencial interno |
| `campaign_verify_cmd` bug exit -1 | bash directa + mención en RESULTADO |
| rustc crash paralelo / OOM | `-j 2` siempre en cargo |
| Secrets a disco (112/impl futura) | prohibido por contrato; review lo audita |
| Fixture 109 no reproduce | copia del backup real en Temp (fuera del repo) |
| `NoLlm` tocado antes de la spec | prohibido: 112 es spec sin código |

## Notas

- SKILLS_CARGADAS base sesión: campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, spec-driven-development, ponytail(full) + por tarea según matriz.
- MCP/tools por tarea en su ficha (codegraph primero siempre; codebase-memory-mcp impacto; campaign vía lead).
- Routing: `vanta-worker` (las 9) · `vanta-review` P2-01 todas.
- Gate P de este plan: set de 9 + alcance FIND-112 aprobados por owner en esta sesión (pregunta de decisión 4/4 respondida).

=== RECITATION ===
Objetivo activo: PLAN seguimiento-mvp — CERRADO 9/9
Estado: completed (desde: act)
Última acción: 9/9 DO (109 ship · 110 re-DEFER · 111 ship 87 tools · 112 spec · 113 re-DEFER · 101 fix · 102 verify-only · 114 refactor · 115 docs) + P2-01 3× approve (Rust/MCP+Py/docs-spec) + follow-ups c1eacebd + progreso (Backlog −9/+1 FIND-116, avance 3 dominios)
Resultado: ✅
Próxima acción: archive a docs/dev/plans/archive/ + nota meta.md (orquestador)
Contrato: 9/9 con commit + verify mecánico lead + P2-01; tools/list 87; coverage 0 gaps
Invariantes: WIP ajeno intacto (completions/*, .opencode, reparacion.bat, stash GOV-C4); sin push (commits locales en develop); mirrors .opencode en 86 = deuda submodule ajeno (no tocado)
Comandos de verificación: wal 69/69 · capture_approval 4/4 · skills 14/14 · pipeline_manager 21/21 · cli 85/85 · sdk_serialization 16/16 · test-hooks n/a · coverage 0 gaps · fmt/clippy/actionlint exit 0
Deuda: FIND-116 (otros ejemplos legacy, Baja) + FIND-110/113 re-DEFER (diseño futuro) + IMPL-112 (trigger: spec cerrada) + mirrors submodule
Próxima tarea si completa: ninguna (plan cerrado)
last-synced: 2026-09-18
=== END RECITATION ===

## Retrospectiva de cierre (Start/Stop/Continue + 1 acción medible)

- **Start:** prompts DETALLE 10 bloques + ejecución secuencial ante rate-limit (9/9 sin un solo fallo de proveedor tras el cambio; el burst paralelo inicial 0/3).
- **Stop:** waves paralelas por defecto cuando el proveedor free-tier ya mostró rate-limit (dos tormentas en este plan); secuencial desde el inicio si hubo fallos recientes.
- **Continue:** P2-01 batch por área con revisor distinto (3 approve, 4 Baja reales, 1 changes evitado por diseño) + staging selectivo + verify lead por tarea.
- **Acción medible:** fallos de proveedor por plan 4/13 lanzamientos (31%) → con secuencial-desde-inicio tras primer rate-limit, objetivo 0% en el próximo plan; métrica: rate-fails/total-launches.

=== RECITATION FIND-109 ===
Campaign ID: 64985e0b-0570-431c-a1e9-1d0d551ad54e
Objetivo activo: FIND-109 wal salvage opt-in + MCP error tipado
Estado: completed
Última acción: subagent vanta-worker 4/4 slices, commit 9c881ac5 (8 files); lead verify wal 69/69 EXIT=0
Resultado: ✅
Próxima acción: FIND-110 S4 lifecycle (secuencial)
Contrato: salvage fixture Temp exit 0 + reporte explicito + suite wal verde + clippy 0
Próxima tarea si completa: FIND-110
=== END RECITATION ===

=== RECITATION FIND-110 ===
Campaign ID: 64985e0b-0570-431c-a1e9-1d0d551ad54e
Objetivo activo: FIND-110 S4 lifecycle (ship o re-DEFER)
Estado: completed
Última acción: subagent vanta-worker: re-DEFER fundado (sin dueno defendible, 0 callers should_gate), commit 1b3b3409 docs-only; lead verify capture_approval 4/4 bash directa (verify_cmd bug -1)
Resultado: ✅
Próxima acción: FIND-111 S5 skill_extract
Contrato: decision lifecycle escrita + test en-proceso o fila actualizada con motivo
Próxima tarea si completa: FIND-111
=== END RECITATION ===

=== RECITATION FIND-111 ===
Campaign ID: 64985e0b-0570-431c-a1e9-1d0d551ad54e
Objetivo activo: FIND-111 skill_extract solo-candidatos
Estado: completed
Última acción: subagent vanta-worker SHIP 86->87, commits 234f0627+e61a15a3; lead verify skills_tests 14/14
Resultado: ✅
Próxima acción: Wave1: FIND-112 spec (secuencial)
Contrato: tool read-only + degrada honesta + test + docs + coverage 0 gaps
Próxima tarea si completa: FIND-112
=== END RECITATION ===

=== RECITATION FIND-111 ===
Campaign ID: 64985e0b-0570-431c-a1e9-1d0d551ad54e
Objetivo activo: FIND-111 S5 skill_extract solo-candidatos read-only
Estado: in-progress
Última acción: Step 1 DISCOVERY: task file FIND-111.md creado con Spec 10 filas + Impacto Regla 0 + decision extender skills.rs
Resultado: ✅
Próxima acción: Step 2 RED: test skill_extract en skills_tests.rs que falla (tool no existe)
Contrato: verificacion: task file docs/dev/tasks/FIND-111.md creado (Step 1 DISCOVERY); evidencia: claim: path degradado limpio confirmado en core | evidencia: vanta-memory/src/core/skill/skill_extractor.rs:245-253 + vanta-memory/tests/skill_extract.rs:128-135 | confianza: alta; artefactos: docs/dev/tasks/FIND-111.md; invariantes: cero codigo tocado (zero-code step); sin sink; WIP ajeno intocable; deuda: Steps 2-5 pendientes; queda_pendiente: RED test (Step 2)
Próxima tarea si completa: FIND-111
=== END RECITATION ===

=== RECITATION FIND-112 ===
Campaign ID: 64985e0b-0570-431c-a1e9-1d0d551ad54e
Objetivo activo: FIND-112 spec runner real ingesta (SIN codigo)
Estado: completed
Última acción: subagent vanta-worker spec 307L, commit 062300a8 docs-only; lead verify diff-check 0 + stat 1 file
Resultado: ✅
Próxima acción: FIND-113 S6b dueno backend
Contrato: spec 5 puntos + tabla matriz + dueno explicito + P2-01-spec; cero codigo
Próxima tarea si completa: FIND-113
=== END RECITATION ===

=== RECITATION FIND-113 ===
Campaign ID: 64985e0b-0570-431c-a1e9-1d0d551ad54e
Objetivo activo: FIND-113 S6b dueno backend (ship o re-DEFER)
Estado: completed
Última acción: subagent vanta-worker: re-DEFER fundado (0 callers/productor, daemon prohibido, MEM-65 intacto), commit b2b20073 docs-only; lead verify pipeline_manager 21/21
Resultado: ✅
Próxima acción: FIND-101 query doc-vs-fix
Contrato: dueno decidido + slice minimo con test o re-DEFER con motivo
Próxima tarea si completa: FIND-101
=== END RECITATION ===

=== RECITATION FIND-101 ===
Campaign ID: 64985e0b-0570-431c-a1e9-1d0d551ad54e
Objetivo activo: FIND-101 query doc-vs-fix
Estado: completed
Última acción: subagent vanta-worker FIX parse-then-open, commit de3bd119; lead verify cli_tests 85/85; avance core-engine sin commitear (progreso al cierre) + colateral TUI propuesto como FIND nuevo
Resultado: ✅
Próxima acción: Wave2: FIND-102
Contrato: decision doc-vs-fix + implementado + help coherente
Próxima tarea si completa: FIND-102
=== END RECITATION ===

=== RECITATION FIND-102 ===
Campaign ID: 64985e0b-0570-431c-a1e9-1d0d551ad54e
Objetivo activo: FIND-102 sdk_serialization no compila
Estado: completed
Última acción: subagent vanta-worker: verify-only (drift autocorregido), commit 12db7d84->amend bb33903e (subject con backslash corregido); lead verify 16/16
Resultado: ✅
Próxima acción: FIND-114 migrar agent_memory.py
Contrato: check tests verde + asserts intactos
Próxima tarea si completa: FIND-114
=== END RECITATION ===

=== RECITATION FIND-114 ===
Campaign ID: 64985e0b-0570-431c-a1e9-1d0d551ad54e
Objetivo activo: FIND-114 migrar agent_memory.py a Client
Estado: completed
Última acción: subagent vanta-worker migracion 3-lineas 1:1, commit ff05d663; lead verify py-compile + grep cero-legacy; Gate C propone FIND-116 otros ejemplos legacy (decision al cierre)
Resultado: ✅
Próxima acción: FIND-115 sync one-liner
Contrato: ejemplo usa Client + smoke verde
Próxima tarea si completa: FIND-115
=== END RECITATION ===

=== RECITATION FIND-115 ===
Campaign ID: 64985e0b-0570-431c-a1e9-1d0d551ad54e
Objetivo activo: FIND-115 sync one-liner
Estado: completed
Última acción: subagent vanta-worker docs-sync 2 superficies, commit c1e72d88; lead verify coverage 0 gaps; SPEC.md placeholders propuestos como FIND (decision cierre)
Resultado: ✅
Próxima acción: cierre: P2-01 batch + progreso + archive
Contrato: superficies muestran one-liner vigente o referencian + coverage 0 gaps
Próxima tarea si completa: cierre-plan
=== END RECITATION ===

=== RECITATION FIND-115 ===
Campaign ID: 64985e0b-0570-431c-a1e9-1d0d551ad54e
Objetivo activo: FIND-115 sync one-liner viejo con FIND-105 (fuente unica)
Estado: in-progress
Última acción: Step 1 DISCOVERY: task file FIND-115.md creado con 10 secciones + Impacto Regla 0 + Spec decisiones; path web confirmado (no fantasma)
Resultado: ✅
Próxima acción: Step 2 ACT: editar README_ES.md (nota referencia) + docs-view.tsx (parrafo Trust/wizard)
Contrato: verificacion: DISCOVERY completo (glob+grep+git show 1349e63c) | evidencia: claim: comandos bare identicos en 4 superficies, delta=contexto FIND-105 | evidencia: README_ES.md:250,256 + docs-view.tsx:263,270 + README.md:192-221 + QUICKSTART:19-53 | confianza: alta; artefactos: docs/dev/tasks/FIND-115.md; invariantes: WIP ajeno intocable; re-traduccion ES prohibida; develop; NO PUSH; deuda: Steps 2-5 pendientes; queda_pendiente: ACT edits + VERIFY + COMMIT + CLOSE
Próxima tarea si completa: ninguna (ultima del plan; cierre del orquestador)
=== END RECITATION ===
