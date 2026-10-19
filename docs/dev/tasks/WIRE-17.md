---
title: "TASK WIRE-17: Docs del scheduler (wired status + promoción del rol)"
kind: task
description: "T4 de la cadena WIRE-14→15→16→17 (ADR-0054): documentar el scheduler cableado — VANTA_MEMORY.md §Operational modules (bridge cableado siempre + loop opt-in VANTADB_SCHEDULER_INTERVAL_MS 60s/0=off; claims stale de WIRE-15 corregidos) + EXPERIMENTAL_FEATURES.md (promoción acotada del rol scheduler-host + fila vanta-memory a host resolved); gates docs exit 0"
---

# TASK WIRE-17: Docs del scheduler (wired status + promoción del rol)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 48, F2 — Distribución; bloque L1375-1401)
- **Fuente:** plan Task 48 (L1375-1401) + ADR-0054 T4 (`docs/dev/architecture/adr/ADR-0054-scheduler-host-vantadb-server.md:180`) + handoffs WIRE-15 (`docs/dev/tasks/WIRE-15.md:191-196`) y WIRE-16 (`docs/dev/tasks/WIRE-16.md:185,200-201`)
- **Esfuerzo:** 🟢 1d | **Appetite:** max 1d | **Stop (plan L1386):** —
- **Prioridad:** 🟡
- **Tipo:** Docs (`docs/api/VANTA_MEMORY.md` + `docs/user/operations/EXPERIMENTAL_FEATURES.md`)
- **Turns estimados:** 5-10 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `48` en el campaign server)
- **Incógnitas (uphill):** 0 — no hay decisiones abiertas: ADR-0054 §Scope Budget (:100-104) fija exactamente qué promover (rol "host del scheduler", alcance acotado, no thaw general) y qué filas cambian; el plan Task 48 (Gate Result ✅ DO) fija las filas.
- **Pendientes (downhill):** 4 steps (1-4)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `48`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Links entrantes a las 2 docs (grep HEAD): `docs/api/{COMPATIBILITY,VERSIONING,PYTHON_SDK,BINDINGS_NAMESPACES}.md` (§Exposure triggers), `docs/user/QUICKSTART.md`, `docs/api/EMBEDDINGS.md`, `docs/dev/FASE-A.md`, `docs/index.md`/`llms.txt` (generados). Ninguno depende del contenido editado (links de página, no de ancla). |
| Callees | Código citado (verificado HEAD `7c5ff1b6`): `vantadb-server/src/{scheduler.rs,main.rs}`, `src/server/{state,bootstrap,handlers}.rs`, `vanta-memory/src/services/{scheduler.rs,conversation_hook.rs}`, ADR-0054 §Scope Budget, `docs/dev/Backlog.md` (FIND-289/290). |
| Implicaciones | Solo prosa/estado documental. Sin cambios de código, wire, API, tests. La promoción es del **ROL acotado** (host del scheduler), no un thaw general de labs (ADR-0054 :100-104). Los gates docs (`check-links`/`check-docs`/`gen-index --check`) protegen contra drift de enlaces/kind. Artefacto propio: `docs/dev/tasks/WIRE-17.md` (este task file). |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `7c5ff1b6`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `docs/api/VANTA_MEMORY.md` (:1-60 scope/stability, :200-329 operational modules + audit, :430-574 forgetting/scoring/reflection/contratos) — objetivo de edición.
  - `docs/user/operations/EXPERIMENTAL_FEATURES.md` (:1-183 completo) — objetivo de edición.
  - Código (claims a documentar): `vantadb-server/src/scheduler.rs` (:1-192 completo — `DEFAULT_INTERVAL_MS` :46, `parse_interval_ms` :50, `scheduler_interval_ms_from_env` :68, `ingest_toml_path` :91, `ingest_runner_cfg` :99, `ingest_runner_factory` :108, `wire_memory` :124-146), `vantadb-server/src/main.rs` (:1-166 completo — wiring :69-91), `src/server/state.rs` (:90-199 — `ConversationTrigger` :97, `BackgroundService` :121, `ServerHooks` :140-158, `ServerState` :161-189), `src/server/bootstrap.rs` (codegraph: `run` :287, `run_with_hooks` :302, callback :351-353, `ServerState` :373-384, join :419, `shutdown_background_services` :425), `vanta-memory/src/services/scheduler.rs` (:1-262 completo — `run_pass` :80, `spawn_memory_scheduler` :198), `vanta-memory/src/services/conversation_hook.rs` (:1-107 — leído en WIRE-14/16), `vanta-memory/src/ingest/runner_config.rs` (FIND-112), `src/server/handlers.rs` (trigger call site :1414).
  - Docs/specs: ADR-0054 (completo — T4 :180, Scope Budget :98-104, restricciones :45-59), plan Task 48 (L1375-1401), `docs/dev/tasks/WIRE-14.md` / `WIRE-15.md` / `WIRE-16.md` (completos — handoffs), `docs/dev/Backlog.md` (WIRE-17 fila :208; FIND-289/290/291), `docs/dev/tasks/DEF-07.md` (:157,162,170 — clasificación original), `docs/dev/tasks/WIRE-01.md` (:36,136 — origen del "scheduler host = MEM-55").
- **Archivos referenciados hacia dentro (imports/deps):** n/a (Markdown). Links salientes de las docs editadas: ADR-0054, SPEC.md, VISION.md, tareas WIRE-*, código por path:línea.
- **Referencias entrantes (grep HEAD):** `VANTA_MEMORY.md` = 20+ links (`docs/api/*`, `PYTHON_SDK.md:521,541`, `BINDINGS_NAMESPACES.md:438,467`, `index.md:57`, `api/index.md:33`); `EXPERIMENTAL_FEATURES.md` = `EMBEDDINGS.md:28`, `QUICKSTART.md:318`, `VERSIONING.md:89`, `FASE-A.md:69,94`, `index.md:1634`; `README.md` (boundary espejo :169-184) **no menciona** el scheduler-host → sin drift por la promoción. Ninguna referencia apunta a las líneas editadas (links de página), excepto `BINDINGS_NAMESPACES.md` → `§Exposure triggers` (sección NO editada).
- **Veredicto impacto:** **BAJO** — 2 docs editadas + 1 task file; sin código, sin wire, sin contratos, sin migración. Riesgos mitigados: (1) drift de enlaces/claims → gates docs en el mismo commit (pre-mortem 3); (2) promoción inflada → texto citando ADR-0054 y alcance acotado explícito (pre-mortem 2); (3) orden → dependencia dura WIRE-16 cumplida (`679b3545`, verificado en HEAD `7c5ff1b6`).

## Contrato

"Docs del scheduler cableado (wired status honesto + promoción del rol acotado, links no copias): (a) **`VANTA_MEMORY.md` §Operational modules** — fila + párrafo de `services::conversation_hook` con el wired status real (bridge cableado SIEMPRE por `vantadb-server` HTTP mode `!read_only` vía `ServerHooks::on_storage_ready`; defaults del seam inertes; MCP mode sin scheduler — diferido por ADR) + fila + párrafo nuevos de `services::scheduler` (pass pull-based `run_pass`; loop opt-in `VANTADB_SCHEDULER_INTERVAL_MS` default `60000`, `0` = off; runner FIND-112; sin runner → skip P4; single-writer) + corrección de 2 claims stale de scheduler (:450 'the pull-based service is WIRE-15'; :548 'scheduler wiring are deferred'); (b) **`EXPERIMENTAL_FEATURES.md`** — promoción del rol: fila `vanta-memory` :33 ('host resolved'), fila `vantadb-server` :38 (rol promovido, acotado), watchlist :53 (trigger cumplido), Freeze list :76 (rol landed), fila `vanta-memory` :144 (el claim `conversation_trigger: None` ya no es verdad) y fila dream :170 (host landed; timers dream sin producer). **Gates:** `node scripts/docs/check-links.mjs` + `node scripts/docs/check-docs.mjs` + `node scripts/docs/gen-index.mjs --check` exit 0. **Sin prometer más que el rol acotado** (no thaw general; MCP/proxy sin planner v1; dream sin auto-consume)."

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): no disparado** — tarea 100% docs, sin símbolos públicos nuevos, sin contrato ambiguo: ADR-0054 §Scope Budget (:100-104) fija el alcance exacto ("se promueve el rol 'host del scheduler de `vanta-memory`', alcance acotado; no es un thaw general de labs"; la fila de `vanta-memory` "pasa de 'partial — scheduler host pending' a host resuelto cuando el wiring esté verde") y el plan Task 48 (Gate Result ✅ DO) fija las filas. Worker sin `question` (question-gates §Routing).

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Alcance de la promoción | A) **solo el rol scheduler-host** (pro: mandato del ADR; contra: —) / B) thaw general de `vantadb-server` (contra: viola ADR-0054 + pre-mortem 2) | ✅ A — decidido-por-evidencia: ADR-0054 :100-104 |
| 2 | Fila `vanta-memory` :33 | A) **"host resolved"** (pro: ADR :103-104 "pasa a host resuelto cuando el wiring esté verde" — verde WIRE-16 `679b3545`; contra: —) / B) dejar "partial" (contra: miente post WIRE-16) | ✅ A |
| 3 | Filas extra (:144, :170) | A) **actualizar :144 (claim `conversation_trigger: None` falsa — `main.rs:76-91` cablea el bridge) + :170 (host landed; timers dream sin producer — `scheduler.rs:85-103` despacha solo `l1_idle`)** (pro: honestidad wired-status; contra: 1 fila fuera de la lista explícita del orquestador) / B) solo las 4 listadas (contra: :144 queda falsa; :170 se lee como dependencia cumplida) | ✅ A — decidido-por-evidencia (código HEAD) |
| 4 | Claims stale de VANTA_MEMORY (:450, :548) | A) **corregir los paréntesis que leen a WIRE-15 como futuro** (pro: mismo doc, mismo tema wired-status; evita autocontradicción con la sección nueva; contra: —) / B) dejar (contra: contradicción interna) | ✅ A — decidido-por-evidencia: `run_decay_pass` sin callers de scheduler (grep HEAD); el pass cubre timers+worker+reclaim |
| 5 | Forma de las citas | A) **path:línea al código real (links, no copias)** (pro: contrato "links, no copias"; contra: —) / B) copiar snippets (contra: drift silencioso) | ✅ A |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **Sin promesa de más:** la promoción es SOLO del rol "host del scheduler" (ADR-0054); `vantadb-server` sigue **labs**; nada de "MCP/proxy obtienen el planner" (diferido por ADR) ni "dream auto-consume" (timers sin producer).
  2. **Wired status exacto:** bridge SIEMPRE cableado en HTTP mode `!read_only`; loop opt-in (`VANTADB_SCHEDULER_INTERVAL_MS`, default `60000`, `0` = off); defaults del seam inertes (`ServerHooks::default()` → `None`); MCP mode sin scheduler.
  3. **Claims verificables:** cada afirmación nueva mapea a código HEAD (`vantadb-server/src/scheduler.rs:124-146`, `main.rs:69-91`, `src/server/state.rs:140-158`, `handlers.rs:1414`).
  4. **WIP ajeno:** `opencode.jsonc` + master plan modificados en el árbol por otros — NO se stagean; PROHIBIDO tocar `docs/pipeline-state.json`; commit con **pathspec**.
  5. **Gates docs en el mismo commit** (pre-mortem 3): check-links + check-docs + gen-index --check exit 0.
- **Comandos de verificación:** `node scripts/docs/check-links.mjs` · `node scripts/docs/check-docs.mjs` · `node scripts/docs/gen-index.mjs --check` · `pwsh scripts/validate-docs-coverage.ps1`.
- **Deuda pendiente:** ninguna propia. (WIRE-18 = verificación adversarial del scheduler; MCP como segundo host + dream-timer producer = diferidos por ADR/YAGNI.)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — docs-only; el cambio **elimina** deuda documental (el Scope Budget mentía post WIRE-16: "scheduler host pending" + "not wired in production yet"). Sin código, sin deps. `NOTICED BUT NOT TOUCHING`: `docs/dev/tasks/DEF-07.md` / `WIRE-01.md` son registros históricos (no se reescriben); ADR-0054 inmutable (no se edita un ADR aceptado); `README.md` boundary espejo sin mención del scheduler (sin drift).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: wired status honesto en ambas docs + promoción acotada + gates docs exit 0 + claims verificados contra código HEAD |
| **Commit** | Commit atómico conventional `docs:` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | n/a (docs; sin cambio de versión) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (scheduler wiring: `on_storage_ready`/`wire_memory`/`run_with_hooks`) + `codebase-memory-mcp_check_index_coverage` (6 paths, `no_recorded_issue` ✅) + grep puntual (claims)
- Gates docs (`node scripts/docs/check-links.mjs && check-docs.mjs && gen-index.mjs --check`) + `pwsh scripts/validate-docs-coverage.ps1`
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- fork `vanta-review` (review P2-01 — `docs/api/**` matchea el tier **Adversarial** HARD-02)

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `documentation-and-adrs` + `api-and-interface-design` (pinned) · `writing-guidelines` · `incremental-implementation` · `test-driven-development` (lifecycle BUILD) + rol: `documentation-skill` (docs bajo `docs/` — obligatoria AGENTS.md) · `source-driven-development` (validación de claims contra código). `security-and-hardening`/`performance-optimization` excluidas (docs-only, sin trust boundary ni hot path).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — no aplica: docs-only; sin input/red/FFI/storage/deps nuevas.
- [ ] **PERFORMANCE** — no aplica: sin código; Regla 9 no dispara (sin claim de optimización).

## Steps

### Step 1 — VANTA_MEMORY.md: wired status (sección + filas + claims stale)

- **Archivos:** `docs/api/VANTA_MEMORY.md`
- **Acción:** (a) fila `services::conversation_hook` (:255) → wired by `vantadb-server` HTTP mode (WIRE-16); (b) fila nueva `services::scheduler` (WIRE-15); (c) párrafo `conversation_hook` (:296-305) → wiring status real (bridge siempre + `on_storage_ready` + defaults inertes + MCP diferido; refs actualizadas `state.rs:143`, `handlers.rs:1414`); (d) párrafo nuevo `services::scheduler` (pass + loop + env + runner + P4 + single-writer); (e) corrección del paréntesis stale :450 (WIRE-15 como futuro); (f) corrección :548 ("its wiring into the scheduler").
- **Verify:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs` → exit 0
- **Evidencia:** ✅ 6 ediciones aplicadas: fila `conversation_hook` (:255 → wired by `vantadb-server` HTTP mode + refs `vantadb-server/src/scheduler.rs:124-146`), fila nueva `services::scheduler` (:256), párrafo `Wiring status (WIRE-16)` (:303-311 — `on_storage_ready`, single-writer, defaults inertes, MCP diferido; refs actualizadas `state.rs:143`, `handlers.rs:1414`), párrafo nuevo `services::scheduler` (:313-327), fix stale :450→:472 ("no scheduler calls the decay pass yet (the WIRE-15/16 scheduler pass covers timers + worker + reclaim only)"), fix :548→:570 ("its wiring into the scheduler"). ✅ `check-links` exit 0 · ✅ `check-docs` exit 0 (shell directo + `campaign_verify_cmd` passed:true).
- **Estado:** ✅ COMPLETED

### Step 2 — EXPERIMENTAL_FEATURES.md: promoción del rol + filas

- **Archivos:** `docs/user/operations/EXPERIMENTAL_FEATURES.md`
- **Acción:** filas :33 (`vanta-memory` → host resolved), :38 (`vantadb-server` → rol promovido, acotado, con evidencia), :53 (watchlist → trigger cumplido; queda ICP-02), :76 (Freeze list → rol landed), :144 (claim `conversation_trigger: None` → host landed), :170 (dream: host landed; timers dream sin producer).
- **Verify:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs` → exit 0
- **Evidencia:** ✅ 6 filas + revision note aplicadas (numeración **pre-edit**; post revision-note quedaron +2): :33 (`vanta-memory` → "scheduler host resolved (ADR-0054, WIRE-16)"), :38 (`vantadb-server` → rol promovido acotado, "no general thaw", env citada), :53 (watchlist → rol landed; queda ICP-02), :76 (Freeze list → rol **landed** WIRE-16), :144 (`conversation_trigger: None` eliminado → host landed, evidencia `vantadb-server/src/scheduler.rs`), :170 (dream → "host landed; pass dispatches `l1_idle` timers only — dream timers have no producer yet"), + revision block WIRE-17. ✅ `check-links` exit 0 · ✅ `check-docs` exit 0.
- **Estado:** ✅ COMPLETED

### Step 3 — VERIFY: gates docs completos + coverage

- **Archivos:** —
- **Acción:** `node scripts/docs/check-links.mjs` · `node scripts/docs/check-docs.mjs` · `node scripts/docs/gen-index.mjs --check` · `pwsh scripts/validate-docs-coverage.ps1` · re-lectura de claims contra código (los 6 puntos del contrato). Si algo falla → retry ladder.
- **Verify:** los 4 comandos verdes (evidencia cruda en §Step 3)
- **Evidencia:** ✅ 4 gates: `check-links` exit 0 (0 broken markdown; 4 wikilinks budgeted informativos, budget 20) · `check-docs` exit 0 (GATING all clear) · `gen-index --check` exit 0 tras `--write` (docs/index.md + llms.txt regenerados: 1538 docs, entradas WIRE-17/WIRE-18) · `validate-docs-coverage` exit 0 · markdownlint 0 issues en 3 archivos. Claims re-verificados contra código HEAD (scheduler.rs:124-146, main.rs:69-91, state.rs:143, handlers.rs:1414, scheduler.rs:85-103).
- **Estado:** ✅ COMPLETED

### Step 4 — CIERRE: OCR + Review P2-01 adversarial + commit local + campaign

- **Archivos:** `docs/dev/tasks/WIRE-17.md` (§Review + RESULTADO §7)
- **Acción:** `pwsh dev-tools/ocr-review.ps1 -Format json` (advisory; Critical/High bloquean) · tier HARD-02: `docs/api/**` → **Adversarial** → fork `vanta-review` (agente distinto, contexto fresco) con contrato + diff + evidencia para el veredicto P2-01 · gates docs · commit **LOCAL** `docs:` con pathspec de archivos propios · `campaign_update_task_state(completed, taskId:"48")` con recitation + payload `review` · `skill progreso`.
- **Verify:** veredicto registrado en §Review + `git show --stat HEAD` limitado a archivos propios + `git status` sin WIP ajeno stageado
- **Evidencia:** ✅ OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`): 9 archivos en el workspace, **1 reviewable** (`vanta-memory/tests/scheduler_crash.rs` — WIRE-18, ajeno a este changeset) y **8 excluidos** con `unsupported_ext` (los archivos de este changeset son `.md`) → para un diff docs-only OCR no tiene superficie que revisar (exclusión por diseño del CLI; pasada cognitiva: 0 Critical / 0 High / 0 Medium). ✅ review P2-01 `vanta-review` **APPROVE** (sesión `ses_ef32d1dc9ffeB0W2AjzvwoINs0`; 0C/0H/0Required; Low-1 aplicado → gates re-verdes; Low-2 dispensa+coordinación; NIT-1 aplicado; NIT-2 dispensado; hallazgo out-of-scope → FIND-292). ✅ tier **Adversarial** (HARD-02). ✅ commit **LOCAL** `docs:` **75bd5da8** (6 archivos, pathspec: VANTA_MEMORY + EXPERIMENTAL_FEATURES + WIRE-17 + Backlog FIND-292 + index/llms; pre-commit hook verde) · ⏳ campaign completed + progreso (en curso).
- **Estado:** ⏳ IN PROGRESS (campaign + progreso en curso)

## Dependencias

- **WIRE-16 ✅** (`679b3545` + cierre `7c5ff1b6`): wiring verde — dependencia dura cumplida. WIRE-14/15 ✅ (`b5d294d2`, `1d5e1697`). ADR-0054 T4.
- **Habilita:** WIRE-18 (T5, verificación adversarial del scheduler) — docs ya no bloquean.
- **Coordinación:** releído fresco desde HEAD `7c5ff1b6`; cambios mínimos + pathspec; race de staging multi-sesión → `git commit -- <paths>` SIEMPRE; conflicto real → BLOQUEO.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` — P2-01, contexto fresco (no participó de la implementación); tier **Adversarial** (HARD-02: `docs/api/**` matchea el glob). Sesión reviewer: `ses_ef32d1dc9ffeB0W2AjzvwoINs0`. Review **pre-commit** (changeset sin commitear). Veredicto: ✅ **APPROVE** (0 Critical / 0 High / 0 Required).
- **Enfoque:** contrato punto por punto (wired status exacto vs código HEAD — 12 claims tabulados uno a uno; promoción acotada sin inflación; gates reproducidos independientemente; coherencia interna — grep de claims stale; WIP ajeno fuera del changeset; Regla 11 para las cifras) + alternativas evaluadas + caveats honestos (no reprodujo markdownlint/OCR: no son gates del contrato).
- **Cómo se probó:** corridas propias del reviewer (no auto-reporte) — 4 gates exit 0 (check-links · check-docs · gen-index --check · validate-docs-coverage) · lectura completa de `vantadb-server/src/{scheduler,main}.rs`, `src/server/{state,bootstrap,handlers}.rs`, `vanta-memory/src/services/{scheduler,conversation_hook}.rs`, `Cargo.toml` ×2 · `git diff`/`git status`/`git diff --cached` (nada staged; `pipeline-state.json` intacto) · refs path:línea verificadas una a una · grep de "not wired in production / host pending / conversation_trigger: None" en lo editado (único match: revision note histórica :22).
- **Hallazgos + disposición:**
  1. [Low-1] Claims de wired status omitían el guard `!read_only` (`main.rs:77`) → **APLICADO**: cláusula "writable servers — read-only skips the wiring, mirroring the TTL sweeper guard" (`VANTA_MEMORY.md`) + "on writable servers" (`EXPERIMENTAL_FEATURES.md` :40, :146); gates re-corridos verdes.
  2. [Low-2] `docs/index.md`/`llms.txt` incluyen la entrada WIRE-18 (untracked ajeno) → **DISPENSA con coordinación**: commit LOCAL (sin push); `WIRE-18.md` debe commitearse antes de pushear la ola (nota en §Notas); no modificable sin editar WIP ajeno (correctamente evitado).
  3. [NIT-1] Task file citaba filas con numeración pre-edit → **APLICADO** (anotación "pre-edit" en Step 2).
  4. [NIT-2] Revision note sin link relativo a ADR-0054 → **DISPENSA**: consistente con el estilo existente del archivo (ADR-0031 también plano).
  5. [Out-of-scope, informativo] `docs/dev/strategy/VantaDB-*.md` con claims stale "sin host productivo / `conversation_trigger: None`" → **FIND-292** creada en Backlog (Origen: WIRE-17 · review P2-01; dep GOV-05).
- **Veredicto:** ✅ **APPROVE** — contrato + DoD Task + gates verdes reproducidos; 0 Critical/High/Required; Low-1 aplicado, Low-2 dispensado con coordinación, NIT-1 aplicado, NIT-2 dispensado con motivo; changeset listo para commit local con pathspec (sin push).

## Notas

- **Alcance de la promoción (clave):** ADR-0054 :100-104 — "se promueve el rol 'host del scheduler de `vanta-memory`' (alcance acotado; no es un thaw general de labs)". La fila `vantadb-server` mantiene **labs**; solo cambia la disposición del rol.
- **Wired status (verificado HEAD `7c5ff1b6`):** `main.rs:69-91` cablea en HTTP mode `!read_only`; `wire_memory` (:124-146) cablea el bridge SIEMPRE y spawnea el loop solo si `interval_ms > 0`; env default `60000`, `0` = off, inválido → warn+default; runner FIND-112 degradado → `None` → skip P4; join por `BackgroundService` post-run-loop.
- **Por qué se tocan :144/:450/:548 (fuera de la lista explícita):** son claims de wired-status del MISMO tema que quedan falsos/ambiguos post WIRE-16; dejarlos crearía autocontradicción con la sección nueva (decisión §Spec #3/#4, verificada contra código).
- **NOTICED BUT NOT TOUCHING:** `WIRE-01.md`/`DEF-07.md` (registros históricos); ADR-0054 (inmutable); `README.md` (boundary sin mención del scheduler); `docs/dev/strategy/VantaDB-*.md` (claims stale → **FIND-292**); `docs/dev/tasks/WIRE-18.md` (otra tarea en vuelo).
- **Coordinación:** WIP ajeno (`opencode.jsonc`, master plan, `WIRE-18.md`, `scheduler_crash.rs`) intacto; commit con pathspec. **Low-2 del review:** `docs/index.md`/`llms.txt` regenerados incluyen la entrada de `WIRE-18.md` (untracked ajeno) → commit LOCAL sin push; coordinar que `WIRE-18.md` se commitee antes de pushear la ola (CI `gate-docs-links` corre `gen-index --check`).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | **0** — alcance fijado por ADR-0054 §Scope Budget + plan Task 48 |
| Pendientes de ejecución (downhill) | 0 steps (1-4 ejecutados — hash + campaign al cierre) |
| % completado | 100% (steps 1-4 ejecutados y verificados; hash + campaign al cierre) |

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: 75bd5da8 (docs local, sin push; + commit docs de cierre)
ARCHIVOS: docs/api/VANTA_MEMORY.md, docs/user/operations/EXPERIMENTAL_FEATURES.md, docs/dev/tasks/WIRE-17.md (nuevo), docs/dev/Backlog.md (FIND-292), docs/index.md + llms.txt (gen-index)
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no | P:no disparado (docs; alcance fijado por ADR-0054 §Scope Budget — sin ambigüedad) · D:no disparado (0 símbolos públicos, 0 incógnitas) · V:no disparado (gates verdes; gen-index requirió el --write canónico) · C:no disparado (sin colaterales de código; hallazgo out-of-scope del review → FIND-292 vía routing findings.md, no bloquea)
SKILLS_CARGADAS: campaign-executor, progreso, ponytail (base auto) · documentation-and-adrs, api-and-interface-design (pinned) · writing-guidelines, incremental-implementation, test-driven-development (SDP v3 BUILD) · documentation-skill, source-driven-development (rol)
```
