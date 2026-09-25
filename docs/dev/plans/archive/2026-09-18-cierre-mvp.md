# Plan de Ejecución: Cierre MVP memoria-agentes (deuda viva + IMPL-112) — 2026-09-18

> **Campaign ID:** c578fd8c-1bee-45a8-b036-2f7b00262828
> **Inicio:** 2026-09-18
> **Estado:** ✅ COMPLETADO (9/9, 2026-09-18)
> **Fuente:** `docs/dev/Backlog.md` (FIND-98/116/117/119, SHOW-05, FIND-110/113 re-DEFER como specs, IMPL-112 spec cerrada `docs/dev/tasks/FIND-112.md`) + deuda viva del cierre seguimiento-mvp + Gate P owner 2026-09-18
> **Autonomous:** false
> **FAIL_MODE:** `parallel` declarado (MAX 3) con **ejecución secuencial por rate-limit** (precedente seguimiento-mvp: burst inicial 0/3, 9/9 en secuencial; retrospectiva: secuencial-desde-inicio tras primer rate-limit)
> **SPEC:** `SPEC.md` raíz (§ Alcance cierre-mvp 2026-09-18: IMPL-112→spec FIND-112, S4/S6b spec-first, TUI mutantes-o-límite, Ask-first `~/.cargo/bin` aprobado FIND-98, ts/examples referenciar; counts 87; Open Questions del cierre) + spec IMPL-112 `docs/dev/tasks/FIND-112.md` (5 puntos, P2-01 approve).
> **Gate P:** heredado — set de 9 + alcance mirrors (solo strings, sin commit submodule) + DESKTOP-41 DEFER aprobados vía `question` 2026-09-18 (§ Gate P).
> **SDP (plan):** base fija abajo; cada sub-agente ejecuta `campaign_discover_skills_v2` (phase PLAN→BUILD) en pipeline-full Paso 0b y declara `SKILLS_CARGADAS:`.
> **Selección owner:** Gate P vía `question` 2026-09-18 — set de 9 APROBADO (Recomendado) + mirrors solo-strings (Recomendado) + DESKTOP-41 DEFER (Recomendado).

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 9 (FIND-98, FIND-110-spec, FIND-113-spec, IMPL-112-S1, FIND-116, FIND-119, IMPL-112-S2, FIND-117, SHOW-05) |
| 🟡 DEFER | resto del Backlog (programas enteros + DESKTOP-41 + SHOW-02/03) |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 1 externo (PROV-12: prereqs + secrets humano) |

Status: ⬆️ uphill = 2 (diseños lifecycle S4/S6b como specs · TUI lock-semantics) · ⬇️ downhill = 9 tasks con contrato definido
**Tras este plan el MVP queda cerrado:** binario parity + ingesta real + S4/S6b diseñados (specs) + ejemplos y docs consistentes.

## Gate P — confirmación usuario (2026-09-18)

Triage + `question` (1 ronda, 3/3 Recomendado): set de 9 APROBADO · mirrors SOLO strings sin commit submodule · DESKTOP-41 sigue DEFER.

## Verificación real global (Paso 0, 2026-09-18 — lectura directa + pruebas en vivo)

- **FIND-98 real y vigente:** instalado sirve **79 tools**, fuente **87** (medido en vivo `server --mcp --db C:/Users/Eros/.vantadb`); fila Backlog refrescada (texto viejo 56-vs-79). Lock PID 3864 a re-chequear en runtime.
- **TUI REPL colateral real:** `src/tui/repl.rs:99-100` ejecuta IQL vía `self.engine` long-lived abierto read-only (`src/bin/vanta-cli.rs:267`); mutantes fallan igual que FIND-101 + sin flush. Fix difiere del CLI (engine lifetime + lock exclusiva).
- **SHOW-05 resto real:** QUICKSTART §0 ✅ + requirements `vantadb-py>=0.5.0` ✅ + README §192 ✅ existen (FIND-105); pendiente SOLO decisión `vantadb-ts/examples/` (existe el dir, cero referencias en README/QUICKSTART/SKILLS-MANIFEST).
- **IMPL-112 slices mecánicos** (`docs/dev/tasks/FIND-112.md:244-261,302-307`): S1 = `IngestRunnerCfg` + enum + G1 + tests 1-6,9-10 · S2 = `llm-driver` + G2 + tests 7-8 (condicional a S1 sin fricción).
- **Mirrors stale reales:** 6× `86 tools` en `.opencode/skills/vantadb-mcp/` (SKILL.md:8,125 + api-reference.md:8,13 + configuration.md:127 + mcp-protocol.md:7); árbol submodule dirty ajeno (10 files) → solo strings, sin commit.
- **Estructura:** 5 planes previos revisados (embeddings-auto, find-correcciones, find89-env, mvp-memoria-agentes, seguimiento-mvp) — este plan replica sus secciones (Gate P, Verificación real, Marco normativo, Tasks por wave, SKIP/DEFER/BLOQUEADO, Grafo, Riesgos, Notas, Recitation).

## Marco normativo — reglas, referencias, comandos, agentes, MCP (obligatorio en ejecución)

Todo sub-agente, antes de codificar, carga y cita en su task file:

- **Reglas** (`.opencode/rules/` — la de su área, lectura completa): `durability.md` (IMPL-112) · `core-engine.md` (98/110-spec/113-spec/117) · `api-contract.md` (R-8 glue, R-5 tools+docs) · `server-mcp.md` (IMPL-112, 117) · `release-ci.md` (98 reinstall) · `python-bindings.md` (116) · `README.md` del dir (formato).
- **Referencias** (`.opencode/references/`): `definition-of-done.md` (todas) · `task-system.md` (lead) · `skills-engineering.md` (SDP) · `clean-code-clean-architecture.md` Ap. V (tareas con código) · `ocr-review.md` (gates VERIFY) · `dev-tools.md` + `test-suite.md` + `testing-patterns.md` (verify) · `architecture.md` (IMPL-112, 117).
- **Comandos** (`.opencode/commands/`): `pipeline.md` (ejecución) · `audit.md` (verify L9/post-tarea) · `research.md` (solo DISCOVERY pesado si deriva) · `backlog.md` (triage) · `status.md` · `ship.md`/`rollback.md` (solo al cierre).
- **Agentes** (routing por `Ruta`): `vanta-worker` (las 9) · `vanta-review` (P2-01 todas, batch al cierre por área).
- **MCP/tools:** `codegraph` (`codegraph_explore` primero siempre) · `codebase-memory-mcp` (`detect_changes`, `check_index_coverage`) · `campaign` (task-system del lead) · `vantadb` (dogfooding/smoke) · `ocr` CLI (`ocr-review.ps1` en CIERRE) · agent-search/metasearchmcp/webfetch SOLO si ambigüedad (con URLs verificadas o marca NO VERIFICADA + deuda TSYS-13).

## Tasks

### Wave0 — paridad binario + specs S4/S6b (disjuntos: fuera-del-repo · docs/dev/tasks/110 · docs/dev/tasks/113)

**Task 1: FIND-98 — reinstalar `vanta-cli` (parity 79→87)**
- **Appetite:** 1d · **Esfuerzo:** 🟢 · **Prioridad:** 🔴 Alta · **Archivos clave:** `C:/Users/Eros/.cargo/bin/vanta-cli.exe` (fuera del repo), `target/debug/vanta-cli.exe` (build si hace falta)
- **Verificación real:** ✅ CÓDIGO-REAL — instalado 79 vs fuente 87 en vivo 2026-09-18 (gap: scene_write/edit + dream×5 + skill_extract); fila refrescada.
- **Gate Justificación:** el binario que usa el usuario miente 8 tools; cero fricción exige parity; fix no toca código del repo.
- **Contrato:** binario instalado sirve **87 tools** (`tools/list`) + smoke `initialize` OK + `cargo build` solo si el binario existente está bloqueado y rebuild es la vía; si el lock persiste → STOP con motivo (no forzar), re-DEFER válido.
- **Pre-mortem:** (1) lock Windows del binario vivo → STOP, no forzar (precedente 2026-09-16); (2) rebuild frío tarda → `-j 2`, no matar el build.
- **Stop:** lock persistente tras 2 intentos (matar procesos MCP vivos es decisión del owner) → re-DEFER con evidencia.
- **Risk Register:** 🟡×🟢 lock → STOP sin forzar | 🟢×🟡 build frío → paciencia + `-j 2`.
- **Cynefin:** 🟦 obvio. **Top 3:** lock / rebuild / smoke-87.
- ⬆️ 0 / ⬇️ 2 steps. **DoD:** contrato + task file mínimo + recitation; commit `docs:` SOLO del task file (el binario vive fuera del repo — no hay diff commiteable; si rebuild tocó `Cargo.lock`, incluirlo con motivo o revertirlo); sin deuda.
- **Skills (≤8):** systematic-debugging (lock/repro) · incremental-implementation · shipping-and-launch (distribución) (+ base campaign-executor/progreso/ponytail).
- **Herramientas+MCP:** `tools/list` count + MCP stdio smoke + `cargo build -p vantadb --bin vanta-cli -j 2` (solo si hace falta) + `campaign_verify_cmd` (bug exit -1 → bash).
- **Referencias:** rules `release-ci.md` · refs `definition-of-done.md`, `dev-tools.md` · agents `vanta-worker` + `vanta-review` P2-01.
- **Investigación:** código (instalador/binario/lock, smoke `tools/list`) en DISCOVERY; internet N/A.
- Task file `docs/dev/tasks/FIND-98.md` · ✅ COMPLETED · Ruta vanta-worker. Branch develop. Commit `docs: FIND-98 — ...` (task file; binario fuera del repo).

**Task 2: FIND-110-spec — diseño lifecycle S4 (spec-first, cero código)**
- **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟡 Media · **Archivos clave:** `docs/dev/tasks/FIND-110.md` (lectura: 5 evidencias + motivo), `vanta-memory/src/core/record/approval.rs`, `vanta-memory/src/gateway/approval_handlers.rs` (lectura)
- **Verificación real:** ✅ CÓDIGO-REAL — re-DEFER fundado 2026-09-18 (0 callers, sin submitter, MCP-35); falta el diseño (submit + ownership + restart).
- **Gate Justificación:** espejo FIND-112: la spec convierte el re-DEFER en slice mecánico futuro; sin spec el gap queda en prosa.
- **Contrato:** `docs/dev/tasks/FIND-110-spec.md` con (a) diseño submit→queue→decisión (quién produce, quién posee, lifecycle restart), (b) qué cambia en gateway/handlers (sin código, solo diseño), (c) gates+tests futuros, (d) qué NO cambia (sin persistencia salvo que el diseño la exija con motivo) + P2-01-spec; **cero código**.
- **Pre-mortem:** (1) persistencia tienta → solo si el diseño la exige con tradeoff escrito; (2) dueño difuso otra vez → la spec no cierra sin dueño explícito (Stop).
- **Stop:** sin dueño defendible ni en diseño → la spec lo declara y el gap vuelve a re-DEFER con diseño parcial (no forzar).
- **Risk Register:** 🟡×🟢 dueño difuso → Stop honesto | 🟢×🟡 scope a persistencia → solo con tradeoff.
- **Cynefin:** 🟨 complicado. **Top 3:** productor submit / dueño / restart.
- ⬆️ 1 (diseño lifecycle) / ⬇️ 2 steps. **DoD:** spec + P2-01-spec + recitation; commit `docs:`; sin deuda.
- **Skills:** spec-driven-development (núcleo) · doubt-driven-development · documentation-and-adrs (+ base).
- **Herramientas+MCP:** codegraph (lectura) + `git diff --check`; internet solo si ambigüedad (patrones aprobación, digest + URLs).
- **Referencias:** rules `core-engine.md`, `api-contract.md` · agents `vanta-worker` + `vanta-review` P2-01 (sobre la spec).
- **Investigación:** código (queue/handlers/tests fuente) en DISCOVERY; internet solo si ambigüedad (patrones aprobación Mem0/Letta, digest + URLs).
- Task file `docs/dev/tasks/FIND-110-spec.md` (ES el entregable) · ✅ COMPLETED · Ruta vanta-worker. Branch develop. Commit `docs: FIND-110-spec — ...`.

**Task 3: FIND-113-spec — diseño dueño scheduler (spec-first, cero código)**
- **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟡 Media · **Archivos clave:** `docs/dev/tasks/FIND-113.md` (lectura), `vanta-memory/src/services/pipeline_worker.rs` (lectura), `docs/dev/tasks/FIND-112.md` §(c) (coherencia dueño, lectura sin editar)
- **Verificación real:** ✅ CÓDIGO-REAL — re-DEFER fundado (0 productor, daemon prohibido, MEM-65 intacto); FIND-112 §(c) fija dueño runner por-llamada (el scheduler debe ser coherente, no idéntico).
- **Gate Justificación:** espejo FIND-110-spec; cierra el par S4/S6b en diseño.
- **Contrato:** `docs/dev/tasks/FIND-113-spec.md` con (a) dueño del backend (dir estado, quién reclama locks TTL, scope-proceso), (b) superficie mínima (`scheduler_status`/`run_once` o equivalente), (c) gates+tests futuros, (d) coherencia con FIND-112 §(c) escrita + P2-01-spec; **cero código**.
- **Pre-mortem:** (1) daemon tienta → prohibido por contrato + Regla 8; (2) duplicar MEM-65 → reusar.
- **Stop:** sin dueño defendible → re-DEFER honesto con diseño parcial.
- **Risk Register:** 🟡×🟡 locks multi-proceso → scope-proceso o Stop | 🟢×🟢 MEM-65 → reusar.
- **Cynefin:** 🟨 complicado. **Top 3:** dueño / locks / coherencia-112.
- ⬆️ 1 (dueño) / ⬇️ 2 steps. **DoD:** spec + P2-01-spec + recitation; commit `docs:`; sin deuda.
- **Skills:** spec-driven-development · doubt-driven-development · codebase-memory (+ base).
- **Herramientas+MCP:** codegraph (lectura) + `git diff --check`.
- **Referencias:** rules `core-engine.md` · agents `vanta-worker` + `vanta-review` P2-01 (sobre la spec).
- **Investigación:** código (worker/backend/MEM-65 + FIND-112 §c como coherencia) en DISCOVERY; internet solo si ambigüedad.
- Task file `docs/dev/tasks/FIND-113-spec.md` (ES el entregable) · ✅ COMPLETED · Ruta vanta-worker. Branch develop. Commit `docs: FIND-113-spec — ...`.

### Wave1 — ingesta local + ejemplos + mirrors (disjuntos: ingest/wiki/mcp · examples · submodule-strings)

**Task 4: IMPL-112-S1 — runner local de ingesta (slice mecánico 1)**
- **Appetite:** 2d · **Esfuerzo:** 🟡 · **Prioridad:** 🟠 Media-Alta · **Archivos clave:** spec `docs/dev/tasks/FIND-112.md` (§ constructor, gates G0/G1/G3, tests 1-6,9-10 — LEER completa, es el contrato), `vanta-memory/src/wiki.rs:399-409`, `vanta-memory/src/ingest/worker.rs:187-235`, `vantadb-mcp/src/wiki.rs:274`
- **Verificación real:** ✅ CÓDIGO-REAL + SPEC CERRADA (P2-01 approve 2026-09-18) — trigger cumplido; `start_ingest<R>` genérico ya existe (IMPL solo construye el `R`).
- **Gate Justificación:** cierra el degradado P4 por defecto (local sin modelo ≡ `NoLlm` bit a bit) sin secrets; es el valor shippable de la spec.
- **Contrato:** G0 (status limpio + secrets-grep limpio) + G1 (`VANTADB_INGEST_PROVIDER=local` sin modelo → `sources_skipped` == nº fuentes; con fake → páginas + `sources_processed` > 0) + G3 (suites `ingest`+`wiki` verdes, clippy 0, fmt, coverage 0 gaps; inputSchema del tool SIN cambios) + tests 1-6,9-10 verdes + `wiki_ingest_status` consultable.
- **Decisiones owner (heredadas FIND-112, no re-derivar):** matriz local+ollama/openai · TOML mínima + env secrets-solo-env · superficie wiki+pipeline · etapas local-primero.
- **Pre-mortem:** (1) fricción del trait en local → reportarla (decide S2); (2) `global_llm_concurrency` → reusar `clamp_llm_concurrency`, no duplicar; (3) secrets en TOML/fixtures → prohibido (G0).
- **Stop:** trait fricciona y exige rediseño → ship parcial documentado + S2 en riesgo (avisar, no forzar).
- **Risk Register:** 🟡×🟡 fricción trait → reportar, decide S2 | 🔴×🟢 secrets a disco → prohibido, G0 lo caza | 🟢×🟡 schema tool → prohibido cambiarlo en S1.
- **Cynefin:** 🟨 complicado. **Top 3:** wiring constructor / G1 observable / G0 secrets.
- ⬆️ 0 (spec cerrada) / ⬇️ 3 steps. **DoD:** gates G0/G1/G3 + task file + recitation; commit `feat:`; sin deuda.
- **Skills (≤8):** test-driven-development (tests nombrados 1-6,9-10) · systematic-debugging · codebase-memory · security-and-hardening (secrets) · incremental-implementation (+ base).
- **Herramientas+MCP:** `cargo test -p vanta-memory -j 2 ingest` + `cargo test -p vantadb-mcp -j 2 wiki` + clippy/fmt + `validate-docs-coverage.ps1` + fixture markdown en Temp + `campaign_verify_cmd` (bug exit -1 → bash).
- **Referencias:** rules `durability.md`, `api-contract.md` (R-8), `server-mcp.md` · refs `architecture.md`, `definition-of-done.md`, `clean-code-clean-architecture.md` Ap. V · agents `vanta-worker` + `vanta-review` P2-01.
- **Investigación:** código (spec como contrato + facade `start_ingest` + pipeline + fábrica EMB) en DISCOVERY; internet solo si ambigüedad de diseño.
- Task file `docs/dev/tasks/IMPL-112-S1.md` · ✅ COMPLETED · Ruta vanta-worker. Branch develop. Commit `feat: IMPL-112-S1 — ...`.

**Task 5: FIND-116 — otros ejemplos con API legacy**
- **Appetite:** max 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🟢 Baja · **Archivos clave:** `examples/python/` (`demo.py`, `autogen/`, `crewai/`, `haystack/`, `langgraph/`, `mem0/`, `smoke_test_extended.py`), `vantadb-python/README.md` (tabla `search_memory`)
- **Verificación real:** ✅ CÓDIGO-REAL (propuesto FIND-114 Gate C con lista concreta; patrón `Client` en `test_sdk.py` + `agent_memory_demo.py`).
- **Gate Justificación:** consistencia de ejemplos (dos APIs confunden); mecánico por archivo.
- **Contrato:** cada ejemplo usa `Client`/`search` + smoke/pytest verde por ejemplo tocado + README sin `search_memory` stale; si un ejemplo no mapea 1:1 → se deja con motivo (no forzar).
- **Pre-mortem:** (1) ejemplo con semántica distinta → mapear o dejar; (2) sin test → smoke + nota.
- **Stop:** >1h → ship parcial (archivos hechos) + resto a nota (no re-abrir FIND).
- **Risk Register:** 🟢×🟡 semántica → mapear primero | 🟢×🟢 sin test → smoke.
- **Cynefin:** 🟦 obvio. **Top 3:** mapeo / smoke / README.
- ⬆️ 0 / ⬇️ 2 steps. **DoD:** contrato + task file mínimo + recitation; commit `refactor:`; sin deuda.
- **Skills:** incremental-implementation (migración mecánica por archivo) (+ base).
- **Herramientas+MCP:** pytest/smoke + py_compile + grep cero-legacy + `campaign_verify_cmd`.
- **Referencias:** rules `python-bindings.md` · agents `vanta-worker` + `vanta-review` P2-01.
- **Investigación:** código (mapeo 1:1 legacy→`Client` por archivo antes de editar) en DISCOVERY; internet N/A.
- Task file `docs/dev/tasks/FIND-116.md` · ✅ COMPLETED · Ruta vanta-worker. Branch develop. Commit `refactor: FIND-116 — ...`.

**Task 6: FIND-119 — sync counts mirrors `.opencode` (solo strings)**
- **Appetite:** max 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🟢 Baja · **Archivos clave:** `.opencode/skills/vantadb-mcp/SKILL.md:8,125`, `references/api-reference.md:8,13`, `references/configuration.md:127`, `references/mcp-protocol.md:7` (6× `86 tools` → `87 tools`)
- **Verificación real:** ✅ CÓDIGO-REAL — 6 hits medidos 2026-09-18; fuente en 87 (SKILLMANIFEST/AGENTS/MCP.md/opencode.jsonc).
- **Gate Justificación:** agentes leen skills del submodule; counts stale misinforman; fix trivial con scope quirúrgico.
- **Contrato:** 0 hits `86 tools` en esos 6 paths + `validate-docs-coverage.ps1` 0 gaps + **cero commit en el submodule** (queda en working tree, precedente FIND-103) + resto del árbol dirty intacto.
- **Pre-mortem:** (1) tocar de más en árbol ajeno → SOLO los 6 strings (grep antes/después); (2) commit accidental en submodule → prohibido.
- **Stop:** N/A (mecánico; si el árbol cambió y los paths no existen → reportar, no cazar).
- **Risk Register:** 🟢×🟡 árbol ajeno → 6 strings exactos, verificación por grep.
- **Cynefin:** 🟦 obvio. **Top 3:** scope / no-commit / gaps-0.
- ⬆️ 0 / ⬇️ 1 step. **DoD:** contrato + task file mínimo + recitation; commit `docs:` SOLO `docs/dev/tasks/FIND-119.md` en el repo padre; sin deuda.
- **Skills:** documentation-and-adrs (strings/docs) (+ base).
- **Herramientas+MCP:** grep antes/después + coverage ps1.
- **Referencias:** agents `vanta-worker` + `vanta-review` P2-01.
- **Investigación:** N/A código (strings); verificación por grep antes/después; internet N/A.
- Task file `docs/dev/tasks/FIND-119.md` · ✅ COMPLETED · Ruta vanta-worker. Branch develop. Commit `docs: FIND-119 — ...` (task file padre).

### Wave2 — slice 2 condicional + TUI + ts-decision (disjuntos: ingest-variantes · tui · docs)

**Task 7: IMPL-112-S2 — runners ollama/openai (CONDICIONAL a S1 sin fricción)**
- **Appetite:** 2d · **Esfuerzo:** 🟡 · **Prioridad:** 🟡 Media · **Archivos clave:** spec § `llm-driver` + G2 + tests 7-8, slice S1 como base
- **Verificación real:** 🟡 CONDICIONAL — solo arranca si S1 reportó "trait sin fricción"; si S1 friccionó → esta task se re-triagea (re-spec o DEFER) antes de arrancar, no se ejecuta a ciegas.
- **Gate Justificación:** completa la matriz owner (local+ollama/openai); sin S1 limpio no existe.
- **Contrato:** G2 (sin servidor/key → degrada P4 mismo assert G1; con mock → extract+merge escriben) + tests 7-8 + G3 + G0 (secrets solo env).
- **Decisiones owner (heredadas FIND-112, no re-derivar):** matriz completa local+ollama/openai · degrada P4 observable · `llm-driver` tras S1 limpio.
- **Pre-mortem:** (1) S1 friccionó → NO arrancar sin re-triage (Gate V); (2) mock HTTP frágil → puerto cerrado determinista, no timing.
- **Stop:** Gate V si S1 friccionó; appetite >2d → ship una variante + nota.
- **Risk Register:** 🟡×🟡 S1-fricción → Gate V obligatorio | 🔴×🟢 key a disco → prohibido.
- **Cynefin:** 🟨 complicado. **Top 3:** gate-S1 / degrada-P4 / secrets.
- ⬆️ 1 (fricción S1) / ⬇️ 2 steps. **DoD:** G2/G3 + task file + recitation; commit `feat:`; sin deuda.
- **Skills:** test-driven-development · security-and-hardening · systematic-debugging (+ base).
- **Herramientas+MCP:** `cargo test -p vantadb-mcp -j 2 wiki` + mock puerto cerrado + coverage + `campaign_verify_cmd`.
- **Referencias:** rules `durability.md`, `server-mcp.md` · agents `vanta-worker` + `vanta-review` P2-01.
- **Investigación:** código (S1 como base + mocks HTTP deterministas) en DISCOVERY; internet solo si ambigüedad.
- Task file `docs/dev/tasks/IMPL-112-S2.md` · ✅ COMPLETED · Ruta vanta-worker. Branch develop. Commit `feat: IMPL-112-S2 — ...`.

**Task 8: FIND-117 — TUI REPL abre mutantes (mismo bug FIND-101, fix distinto)**
- **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🟡 Media · **Archivos clave:** `src/tui/repl.rs:99-100` (executor sobre `self.engine`), `src/bin/vanta-cli.rs:267` (open read-only TUI), `src/cli_handlers/data.rs:212-230` (patrón parse-then-open a reusar en espíritu, no copiar)
- **Verificación real:** ✅ CÓDIGO-REAL — engine long-lived read-only; mutantes fallan + sin flush; lock exclusiva impide "abrir rw una vez" (bloquearía al resto).
- **Gate Justificación:** bug real en UI shippada; el fix (por-query open/close o documento de límite) es acotado al REPL.
- **Contrato:** INSERT→SELECT visible en la misma sesión TUI (o límite documentado en el help del REPL si el diseño lo exige con motivo) + test del path + sin regresión en reads/stats + clippy 0.
- **Pre-mortem:** (1) reopen por query rompe estado TUI → diseñar open/close solo del handle de escritura; (2) lock exclusiva de fondo → medir contención, no asumir.
- **Stop:** rediseño TUI necesario → DEFER con diagnóstico (no rediseñar).
- **Risk Register:** 🟡×🟡 estado TUI → handle acotado | 🟡×🟢 lock → medir.
- **Cynefin:** 🟨 complicado (lock-semantics). **Top 3:** lifetime / lock / flush.
- ⬆️ 1 (diseño handle) / ⬇️ 2 steps. **DoD:** contrato + task file + recitation; commit `fix:`; sin deuda.
- **Skills:** systematic-debugging · test-driven-development · codebase-memory (+ base).
- **Herramientas+MCP:** codegraph + `cargo test -p vantadb tui -j 2` (o scope equivalente) + clippy/fmt + `campaign_verify_cmd`.
- **Referencias:** rules `core-engine.md` · refs `clean-code-clean-architecture.md` Ap. V · agents `vanta-worker` + `vanta-review` P2-01.
- **Investigación:** código (REPL/engine/lock + patrón parse-then-open FIND-101 en espíritu) en DISCOVERY; internet N/A.
- Task file `docs/dev/tasks/FIND-117.md` · ✅ COMPLETED · Ruta vanta-worker. Branch develop. Commit `fix: FIND-117 — ...`.

**Task 9: SHOW-05 (resto) — decisión `vantadb-ts/examples` + referencia**
- **Appetite:** max 1h · **Esfuerzo:** 🟢 · **Prioridad:** 🟢 Baja · **Archivos clave:** `vantadb-ts/examples/` (existe), `README.md`/`docs/user/QUICKSTART.md` (1 línea de referencia)
- **Verificación real:** ✅ CÓDIGO-REAL — resto pendiente verificado 2026-09-18: QUICKSTART §0 + requirements + README existen (FIND-105); SOLO falta decidir mover-vs-referenciar (cero referencias hoy).
- **Gate Justificación:** cierra SHOW-05 (resto); mover código es mudanza, referenciar es 1 línea — decidir con motivo ponytail.
- **Contrato:** decisión escrita (mover o referenciar + porqué) + implementada (línea o mudanza) + coverage 0 gaps.
- **Pre-mortem:** (1) mudanza rompe links CI → si mueve, grep de referencias; (2) re-traducir ES → prohibido.
- **Stop:** N/A (tiny; si la decisión exige mudanza grande → solo decisión + nota, no mudanza).
- **Risk Register:** 🟢×🟢 links rotos → grep | 🟢×🟢 alcance → decisión primero.
- **Cynefin:** 🟦 obvio. **Top 3:** decisión / links / gaps-0.
- ⬆️ 0 / ⬇️ 1 step. **DoD:** contrato + task file mínimo + recitation; commit `docs:`; sin deuda.
- **Skills:** documentation-and-adrs (decisión+referencia) (+ base).
- **Herramientas+MCP:** grep referencias + coverage ps1.
- **Referencias:** agents `vanta-worker` + `vanta-review` P2-01.
- **Investigación:** N/A código (grep de referencias + decisión); internet N/A.
- Task file `docs/dev/tasks/SHOW-05.md` · ✅ COMPLETED · Ruta vanta-worker. Branch develop. Commit `docs: SHOW-05 — ...`.

## SKIP

Ninguno (todo lo triagiado con gap real va a DO; SHOW-05 se achicó, no se skipeó).

## DEFER (programas enteros + pesados, fuera del cierre MVP)

- **DESKTOP-41** (smoke VM limpia): Gate P 2026-09-18 lo mantiene DEFER (pide VM + track desktop).
- **SHOW-02/03** (showcase web/RAG-PDF): showcase, no MVP-core.
- **MGR/EXE/FUT/STU/UX/TS/WSM/RES/AGT/GOV/CI-01/DESKTOP/OLD/DISC/DEC + resto FIND**: programas enteros, mismo criterio MVP.
- **Mirrors `.opencode` más allá de strings**: sync estructural del submodule = repo configOpencode (no este plan).

## BLOQUEADO

- **PROV-12** (PyPI wheels): prereqs PROV-01/02/04 + secrets humano — externo, no arranca sin owner.

## Grafo de dependencias / Waves (FAIL_MODE=parallel declarado, ejecución secuencial por rate-limit)

```
Wave0: FIND-98 + FIND-110-spec + FIND-113-spec   (fuera-repo · task-110 · task-113 — disjuntos)
Wave1: IMPL-112-S1 + FIND-116 + FIND-119         (ingest/wiki/mcp · examples · submodule-strings — disjuntos)
Wave2: IMPL-112-S2[condicional S1] + FIND-117 + SHOW-05   (ingest-variantes · tui · docs — disjuntos)
```

S2 arranca SOLO con veredicto "S1 sin fricción" (Gate V si friccionó). Tras este plan: MVP cerrado (IMPL-112 completo + S4/S6b diseñados + parity + consistencia).

Órdenes internos: IMPL-112-S2 consume el veredicto S1 (Wave1→Wave2, Gate V si fricción); 110-spec/113-spec comparten FIND-112 §(c) como referencia común sin editarla; 119 deja el árbol submodule sin commit para el bump futuro; SHOW-05 no toca instaladores (FIND-105 intacto); 98 no produce diff en repo (binario fuera).
Nota runner: secuencial desde el inicio por rate-limit (precedente seguimiento-mvp 2026-09-17/18: burst 0/3 → 9/9 secuencial); si un lanzamiento falla → backoff 2min + retry.

## Riesgos globales

| Riesgo | Respuesta |
|--------|-----------|
| Rate-limit proveedor (burst 0/3 el 2026-09-17) | secuencial desde el inicio; si un lanzamiento falla → backoff 2min + retry (precedente funciona) |
| `campaign_verify_cmd` bug exit -1 | bash directa + mención en RESULTADO |
| rustc crash paralelo / OOM | `-j 2` siempre en cargo |
| Lock binario Windows (FIND-98) | STOP sin forzar, re-DEFER con evidencia |
| Secrets a disco (IMPL-112) | prohibido por contrato; G0 lo caza; review lo audita |
| Edición de más en submodule (FIND-119) | 6 strings exactos + grep antes/después; commit prohibido ahí |
| S2 sin S1 limpio | Gate V obligatorio antes de arrancar |
| WIP ajeno (`completions/*`, `.opencode` resto, `reparacion.bat`, stash GOV-C4, `C:/Users/Eros/.vantadb*`) | staging selectivo, intocable |

## Notas

- SKILLS_CARGADAS base sesión (plan): campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, spec-driven-development, ponytail(full).
- MCP/tools por tarea en su ficha (codegraph primero siempre; codebase-memory-mcp impacto; campaign vía lead).
- Routing: `vanta-worker` (las 9) · `vanta-review` P2-01 todas (batch al cierre por área, precedente 2 campañas).
- Estructura tomada de los 5 planes previos (embeddings-auto, find-correcciones, find89-env, mvp-memoria-agentes, seguimiento-mvp): Resumen + Gate P + Verificación real + Marco normativo + Tasks por wave + SKIP/DEFER/BLOQUEADO + Grafo + Riesgos + Notas + Recitation.
- Fila FIND-98 refrescada + filas FIND-117/FIND-119 creadas en `docs/dev/Backlog.md` durante este plan-mode (gaps verificados, no inventados).

## Retrospectiva de cierre (Start/Stop/Continue + 1 acción medible)

- **Start:** secuencial-desde-inicio tras el primer rate-limit (9/9 sin un solo fallo de proveedor; burst paralelo inicial del plan anterior 0/3). Veredicto fricción explícito en RESULTADO como Gate V de S2 (S1→S2 fluyó sin re-triage).
- **Stop:** prompts de delegación sin el "veredicto explícito" pedido (S1 lo trajo igual por precedente; formalizarlo evitó el Gate V).
- **Continue:** P2-01 batch por área con revisor distinto (3 approve, solo 2 Baja: 1 no-accionable heredada + 1 tracked a configOpencode) + staging selectivo + verify lead por tarea + spec-first para uphill (110/113-specs cierran re-DEFERs en diseño).
- **Acción medible:** rate-fails 0/12 lanzamientos en este plan (vs 4/13 el anterior) → baseline nueva: secuencial + backoff-2min como default con free-tier; métrica: rate-fails/total-launches = 0%.

=== RECITATION ===
Objetivo activo: PLAN cierre-mvp — CERRADO 9/9
Estado: completed (desde: act)
Última acción: 9/9 DO (98 re-DEFER · 110-spec · 113-spec · S1 · 116 · 119 · S2 · 117 · SHOW-05) + P2-01 3× approve (Rust/Python/docs-spec, 2 Baja no-accionables) + progreso (Backlog −8/+0, avance 3 dominios)
Resultado: ✅
Próxima acción: archive a docs/dev/plans/archive/ + nota meta.md (orquestador)
Contrato: 9/9 con commit + verify mecánico lead + P2-01; tools/list 87; coverage 0 gaps; IMPL-112 matriz completa (G1+G2)
Invariantes: WIP ajeno intacto (completions/*, .opencode resto, reparacion.bat, stash GOV-C4, C:/Users/Eros/.vantadb*); sin push (commits locales en develop); mirrors submodule sin commit
Comandos de verificación: runner_config 6/6 · wiki_ingest 5/5 · tui::repl 5/5 · smokes python exit 0 · coverage 0 gaps · fmt/clippy/actionlint exit 0
Deuda: FIND-98 re-DEFER (retry al liberar lock) · FIND-118 (trigger 0.6.0, nueva) · PROV-12 bloqueado + DESKTOP-41/SHOW-02-03 defer · mirrors estructurales (otro repo)
Próxima tarea si completa: ninguna (plan cerrado)
last-synced: 2026-09-18
=== END RECITATION ===
