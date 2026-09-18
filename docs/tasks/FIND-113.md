# FIND-113: S6b programador — dueño del backend + diseño o re-DEFER fundado

## Metadata

- **Plan file:** `docs/plans/2026-09-17-seguimiento-mvp.md` (Task 5, Wave1 segunda en secuencia)
- **Fuente:** plan seguimiento-mvp Task 5 + FIND-107 §7 (S6b) + Backlog FIND-113
- **Esfuerzo:** 🟡 1d (consumido: ~1 slice discovery, cero código)
- **Prioridad:** 🟡 Media
- **Tipo:** Rust core (decisión docs-only, sin símbolos nuevos — `campaign_detect_task_type` → `rust/Rust core`, skills base source-driven/doubt/ponytail)
- **Branch:** develop · **Commit:** `docs: FIND-113 — S6b programador re-DEFER fundado (sin dueño defendible en 1d)` (NO PUSH, staging solo este file)
- **Creado:** 2026-09-17 → ejecutado 2026-09-18
- **last-synced:** 2026-09-18
- **Estado:** ✅ COMPLETED (re-DEFER fundado — cierre válido, no fracaso; espejo de FIND-110)
- **Incógnitas (uphill):** 0 abiertas (1 inicial — ¿quién posee el backend en MCP? — resuelta con evidencia)
- **Pendientes (downhill):** 0 (ship cancelado por decisión; nada que ejecutar)
- **SDP:** ver § Skills (SDP real Paso 0b vía `campaign_discover_skills_v2` phase BUILD)

## 1. TAREA — objetivo + contrato + AC

**Objetivo:** cerrar S6b de FIND-107 — o hay dueño del backend defendible en el
proceso MCP con slice mínimo (`scheduler_status`/`scheduler_run_once` con test)
más docs, o re-DEFER fundado. Espejo exacto de FIND-110 (S4): o hay dueño con
test, o re-DEFER con motivo. PROHIBIDO: rediseñar MEM-65 (`TaskKind::Dream` ya
cableado → reusar) y daemon/background-thread en MCP (fuera de appetite).

**Estado verificado del código (NO re-derivado, confirmado en DISCOVERY):**
`PipelineWorker::run_once` + `LocalStateBackend` + locks TTL existen
(`pipeline_worker.rs:198-324`, `local_backend.rs:36-42,353-392`); falta dueño en
proceso MCP — grep `LocalStateBackend|PipelineWorker|scheduler` en
`vantadb-mcp/src/` = 0 hits de producción (solo mención doc en `dreams.rs:11`).
`TaskKind::Dream` ya cableado MEM-65 (`pipeline_worker.rs:804-820` dispatch +
`run_dream` `:545-565` + 2 tests verdes en suite 21/21).

**Contrato (plan Task 5):** dueño del backend decidido y escrito (dir estado,
quién reclama locks) + `scheduler_status`/`scheduler_run_once` con test o slice
mínimo + docs; o re-DEFER con motivo (NO es fracaso).

**AC de esta ejecución:**

- (a) ✅ decisión dueño escrita (ver `## Decisión dueño (AC-a)` — re-DEFER con 6 evidencias)
- (b) ✅ mecánica en-proceso verde EXISTENTE verificada mecánicamente
  (`cargo test -p vanta-memory -j 2 --test pipeline_manager` → 21/21 ok,
  incluye `handler_dream_task_*` 2/2 + `locks_are_owner_scoped_*` + reclaim 2/2)
  + justificación de re-DEFER en este task file (Backlog lo actualiza el
  orquestador/lead — `docs/Backlog.md` es edición del lead, fuera de mi scope)
- (c) N/A ship (no shippea: sin tabla Spec por tool; ver motivo — shippear
  status/run_once sin dueño = mostrador vacío, prohibido por analogía FIND-110)

## 2. ARCHIVOS — clave + relacionados + prohibidos

**Clave (leídos completos, verbo vía `codegraph_explore` + `Read`):**

- `vanta-memory/src/services/pipeline_worker.rs:1-120` — doc crate (single
  process MEM-16, sin Prometheus, `claimStaleTasks` portado MEM-66 `:16-18`),
  `TaskHandler` trait `:49-52`, `WorkerConfig` `:56-73` (max_retries 3,
  lock_ttl 60s, batch 8), `RunStats` `:82-88`, `LayerTelemetry` MEM-65 `:90-130`
- `vanta-memory/src/services/pipeline_worker.rs:198-335` — `PipelineWorker`
  (`backend`, `config`, `owner: worker-{pid}` `:199-204`), `new` `:207-214`,
  `with_owner` `:219-222` (2 workers mismo proceso colisionarían sin esto),
  `run_once` `:232-246` (claim loop hasta `batch_size`), `reclaim_stale`
  `:251-267`, `run_task` `:272-324` (session lock + handler + settle
  retry/dead-letter, release SIEMPRE antes de actuar `:295`)
- `vanta-memory/src/services/pipeline_worker.rs:342-565,804-820` —
  `MemoryTaskHandler<R: LlmRunner>` `:347-359`, `run_dream` `:545-565`
  (`detect_idle` + `consolidate_session`, skip quiet `Ok` si no idle, sin locks
  nuevos), dispatch `handle` `:804-820` (`L1|Flush→run_l1`, `L2→run_l2`,
  `Dream→run_dream`, `L3→run_l3+assembly`)
- `vanta-memory/src/utils/local_backend.rs` (494L completo) — doc in-process
  (no Redis Principio 7, un mutex, Clock inyectado `:1-10`), `Inner` `:20-34`
  (buffers/states/timers/queue/`pending`/`locks`), struct `:39-42`, techo
  multi-proceso explícito `:228-237` ("Multi-process would need a CAS on the
  lease (documented ceiling, YAGNI here)"), `claim_task` `:241-257`,
  `complete_task` fencing `:261-270`, `renew_task_lease` `:276-286`,
  `claim_stale_tasks` `:294-318` (UNA sección crítica, `ponytail: O(n)`),
  `requeue_task` `:324-344`, locks TTL `acquire/renew/release`
  `:359-392` (expired colectados primero, owner-scoped)
- `vanta-memory/src/core/state/types.rs` (112L) — `TaskKind` `:38-51`
  (L1/L2/L3/`Dream` MEM-61 escribe `dream/<s>/<run_id>`, nunca muta L1/Flush),
  `TaskPayload` `:56-70` (id `t_{ms}_{seq}`, priority 0/1/2, attempts)

**Relacionados (leídos, SOLO lectura salvo ship — no hubo ship):**

- MEM-65 cableado Dream existente (NO rediseñado): dispatch + `run_dream` +
  tests `handler_dream_task_consolidates_session_without_touching_l1` +
  `handler_dream_task_skips_quietly_when_not_idle` (2/2 verdes en suite 21/21)
- `vanta-memory/src/services/conversation_hook.rs` (107L) — único caller
  productivo de `run_once` en el crate (`run_bridge_pass` `:93-107`,
  `trigger_every_n = usize::MAX`, Principio 4: sin runner el host nunca llama)
- `vanta-memory/src/utils/pipeline_factory.rs` (53L) — trío
  backend+manager+worker (`PipelineComponents::assemble`), `build_system()`
  con `SystemClock`
- `vanta-memory/tests/pipeline_manager.rs` — 21 tests (lista verificada con
  `-- --list`): queue/priority, capture_atomic, timers, locks owner-scoped +
  expire por clock, reclaim 2 tests, worker retry/dead-letter, skip-locked sin
  pérdida, L1/L2/L3 handlers, Dream 2 tests
- `vantadb-mcp/src/handlers/` — patrón si shippeara: `dreams.rs` plantilla
  (S2+S3: definiciones + dispatch + validación frontera; 267L leído completo).
  `handlers/mod.rs` (6L) + `handlers/tools.rs:1220-1225`
  (`handle_tools_call(&params,&executor,&storage,&cfg)` — dispatch stateless
  por llamada, sin estado compartido) + `:25-27` registry 87 tools + `:2911,2924`
  dispatch skill/dream. SOLO lectura — cero ediciones (no ship)
- `vantadb-mcp/src/server.rs` — evidencia modelo de proceso MCP:
  `serve_lines` long-lived por cliente (`:377-386`), `dispatch_request`
  `:560-602` (stateless por request, `spawn_blocking(handle_tools_call)`,
  limitación MOD-11 documentada), `dispatch_request_proxy` `:659`,
  `run_stdio_server_auto` `:833-907` (split writer/proxy MCP-35: writer abre DB,
  resto proxean por HTTP; stale cleanup + retry)
- `vantadb-mcp/src/dreams.rs:1-14` — doc: L1 nunca tocado, `promote` preview
  stub, "the real merge is MEM-65's pipeline worker" (prueba de que el worker
  es el dueño lógico del merge, pero sin backend en MCP)
- MEM-65/MEM-66/MEM-16 task files (lectura histórica, sin mutación)
- `docs/tasks/FIND-112.md` §(c) (dueño runner — coherencia dueño-scheduler,
  leído completo 307L, SIN editar — prohibido por contrato de esta tarea)
- `docs/tasks/FIND-107.md:19,121,166` (S6b: `run_once` necesita backend + dir
  estado + quién encola; ningún caller MCP encola hoy → DEFER propuesto,
  fila FIND-113)

**Prohibidos (NO tocados — verificación en `git status` al cierre):**

- Rediseño MEM-65 (`TaskKind::Dream` cableado → reusar, no duplicar)
- Daemon/background-thread en MCP (fuera de appetite, prohibido por contrato)
- `src/wal*.rs` + `src/cli_handlers/` (FIND-109 ✅ `9c881ac5`)
- Approval S4 (FIND-110 ✅ `1b3b3409` — espejo, solo lectura)
- Skill S5 (FIND-111 ✅ `234f0627+e61a15a3` — solo lectura)
- Spec ingesta salvo lectura (FIND-112 ✅ `062300a8` — `docs/tasks/FIND-112.md`
  leído §(c), NO editado)
- `reparacion.bat`, `.opencode` (submodule), `Justfile`, `ocr-*`,
  `completions/*`, `desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4,
  `docs/Backlog.md` (lo toca el orquestador/lead), plan file (solo recitation),
  `C:/Users/Eros/.vantadb*`, WIP ajeno en `git status` (`.opencode`,
  `completions/*`, `docs/Backlog.md` modificados — intactos, no stageados)

## 3. DEPENDENCIAS

- **Wave1 segunda en secuencia.** FIND-112 ✅ spec (`062300a8`,
  recitation plan `:270-278` → NextTask FIND-113). FIND-109 ✅, FIND-110 ✅
  re-DEFER, FIND-111 ✅ SHIP — archivos disjuntos, sin interferencia.
- **Después:** FIND-101 (orquestador; `vanta-cli query` doc-vs-fix, archivos
  disjuntos `src/cli_handlers/data.rs`).
- **Coherencia FIND-112 §(c) (leído, no editado):** dueño runner = proceso MCP,
  registry `ingest_runs()` (`wiki.rs:377-380`), construcción POR LLAMADA (no
  global) porque (a) env mutable, (b) runner barato (Strings, sin conexión),
  (c) sin secrets en static. El scheduler NO puede seguir ese patrón: el
  backend debe ser COMPARTIDO entre llamadas para tener tareas; construcción
  por llamada = cola siempre vacía = mostrador vacío. El único patrón coherente
  sería backend global writer-side — que es exactamente el diseño >1d
  rechazado (ver Decisión). Coherencia verificada, no contradicción: runner
  (stateless por llamada) ≠ scheduler (estado compartido con lifecycle).
- **Stop del plan aplicado:** sin dueño defendible en 1d → re-DEFER con motivo
  (este archivo + RESULTADO; Backlog lo toca el orquestador). No forzado.
- **NextTask:** FIND-101 (orquestador).

## 4. REFERENCIAS

- **Rules (leída completa):** `.opencode/rules/core-engine.md` (40L — R-1
  feature-gating `experimental-*` sin default; R-2 internas sin callers no al
  SDK; R-3 `?`+`Result`, sin unwrap salvo `sync_ext`; R-4 `unsafe`+`SAFETY`;
  R-5 prefijo único `VANTADB_*` + `parse_env_or`, warn+default nunca panic).
  Aplicación: R-2 (exponer `run_once` sin callers MCP = exportar interna al
  SDK/MCP — prohibido); R-8 api-contract (vía FIND-110: glue, no lógica —
  scheduler en MCP sería lógica con estado, no glue).
- **Refs:** `definition-of-done.md` (standing checklist + DoD VantaDB + v1
  baseline + progreso Trigger 1 — ver § DoD 3 niveles) ·
  `clean-code-clean-architecture.md` Ap. V (V.1 mapa capas → repo; V.2 reglas
  duras; V.3 stuttering; V.4 severidades `/cleanCA` — Humble Object: MCP
  handlers traducen DTO ↔ mundo externo, cero lógica de negocio; un scheduler
  con backend propio en `vantadb-mcp` violaría Humble + R-8) ·
  `skills-engineering.md` (SDP).
- **Commands:** `pipeline.md` (vía `pipeline-full.md` exacto — este prompt) ·
  `audit.md` (verify full como referencia; N/A docs-only justificado).
- **SPEC.md raíz:** N/A — no existe SPEC.md raíz versionado; la decisión vive
  en este task file por contrato del plan (igual que FIND-110/112).
- **Tabla Spec:** filas por tool si shippea (FIND-111 patrón) — N/A: no
  shippea, sin tabla (igual que FIND-110 §AC-c). Spec Phase 1b con tabla de
  decisiones POR EVIDENCIA en su lugar (ver § Spec).
- **Notion (Paso 0c):** sin tool `fetch`/Notion disponible en este entorno —
  4 páginas no consultables; registrado como limitación, no como evidencia
  (igual que FIND-110). Filtro VantaDB N/A en consecuencia.

## 5. SKILLS

**Cargadas (SDP Paso 0b, `campaign_discover_skills_v2` phase BUILD,
maxSkills 8, keywords scheduler/dueño-backend/locks-TTL/run_once/…):**

- `spec-driven-development` (núcleo: decisión dueño Specify→Plan, Phase 1b por evidencia)
- `test-driven-development` (mecánica en-proceso como evidencia: 21/21 verde; Prove-It sin reproduction nueva — el gap es lifecycle, no bug)
- `codebase-memory` (blast radius KG: `codegraph_explore` + grep workspace; incluye ritual mantenimiento)
- `incremental-implementation` (slice discovery único + scope discipline + rollback-friendly)
- `source-driven-development` (base Rust: todo claim → `file:línea` real)
- `doubt-driven-development` (base Rust: adversarial ship-vs-defer, 3 alternativas)
- `context-engineering` (lifecycle BUILD: context pack por slice)
- `api-and-interface-design` (evaluar superficie `scheduler_*`, R-8)

Descartada con motivo: `frontend-ui-engineering` (sugerida por SDP lifecycle,
score 1.00) — cero superficie `web/` en S6b; justificado, no cargada (igual
que FIND-110).

`SDP: spec-driven-development · test-driven-development · codebase-memory · incremental-implementation · source-driven-development · doubt-driven-development · context-engineering · api-and-interface-design (8; frontend-ui-engineering descartada: cero web/) + base campaign-executor/progreso/ponytail(full).`

## 6. HERRAMIENTAS + MCP

- `codegraph_explore "pipeline_worker run_once LocalStateBackend TaskKind Dream MemoryTaskHandler scheduler_status scheduler_run_once"` — blast radius inmediato: 29 símbolos/1 file; `TaskKind` 3 callers, `run_once(pipeline_worker)` 5 callers en `conversation_hook.rs`, `MemoryTaskHandler` 1 caller, `LocalStateBackend` 16 callers (utils/*) — CERO en `vantadb-mcp` (confirmado por grep)
- `Read` directo (ver § Archivos — paths exactos con líneas)
- `campaign_detect_task_type` → `rust/Rust core` (checks cargo estándar)
- `campaign_discover_skills_v2` → 8 skills (ver §5)
- `campaign_get_workflow(feature-add)` → perfil spec/implement/verify/review/accept/close (guía; enforcement = C0 genérica)
- `cargo test -p vanta-memory -j 2 --test pipeline_manager` — 21/21 verde (mecánica; Cargo `-j 2` siempre)
- `cargo test -p vanta-memory -j 2 --test pipeline_manager -- --list` — 21 nombres (evidencia Dream/locks/reclaim)
- `campaign_verify_cmd`: bug exit -1 conocido → bash directa con motivo (cero código: no hay clippy/fmt/nextest que correr sobre el diff; sí `git diff --check` + suite mecánica existente al cierre)
- OCR delegation: `pwsh dev-tools/ocr-review.ps1` (advisory) — `FIND-113.md` excluido (`unsupported_ext`, igual que FIND-110); reviewables solo WIP ajeno (fuera de scope, no bloquean)
- Internet: no requerida — incógnita uphill respondida con evidencia in-repo (backend + server + grep exhaustivo). Cero citas externas, cero deuda TSYS-13

## 7. INVESTIGACIÓN CÓDIGO (DISCOVERY — evidencia)

1. **Worker existe y funciona en-proceso:** `PipelineWorker::new` defaultea
   `owner: worker-{pid}` (`pipeline_worker.rs:207-214`); `with_owner` para
   multi-worker (`:219-222`); `run_once` consume hasta `batch_size` vía
   `claim_task(owner, lock_ttl)` (`:232-246`); `run_task` adquiere
   `pipeline_lock:{session}` con TTL, ejecuta `handler.handle`, libera ANTES
   de actuar, settle complete/retry/dead-letter (`:272-324`). 21/21 verde.
2. **Backend es scope-proceso por diseño:** `LocalStateBackend` = `Mutex<Inner>`
   único (`local_backend.rs:39-42`), Clock inyectado (tests deterministas),
   `pending` + `locks` con leases TTL + fencing por owner. Techo documentado
   `:236-237`: "Multi-process would need a CAS on the lease (documented
   ceiling, YAGNI here)". Locks TTL `:359-392` (expired colectados primero,
   owner-scoped acquire/renew/release).
3. **Locks TTL reclamables solo en-proceso:** `claim_task/complete/renew/
   claim_stale/requeue` (`:241-344`) operan bajo UN mutex en UNA sección
   crítica — "double-reclaim is impossible in-process". Entre procesos no hay
   CAS: dos writers reclamarían el mismo lease (el `expire_at` vive en RAM de
   cada proceso). Scope-proceso o DEFER (pre-mortem 1 confirmado).
4. **MEM-65 Dream ya cableado — NO duplicar (pre-mortem 2 confirmado):**
   `handle` matchea `TaskKind::Dream → run_dream` (`:809`); `run_dream`
   (`:545-565`) valida `detect_idle` con los mismos args que
   `consolidate_session` re-valida (sin string-matching, sin dead-letter spam
   por timers prematuros), escribe a `dream/<s>/<run_id>`, nunca muta L1.
   Tests `handler_dream_task_*` 2/2 verdes. Reusar, no rediseñar.
5. **Cero dueño en MCP (el gap):** grep `LocalStateBackend|PipelineWorker|
   scheduler|SystemClock|pipeline` en `vantadb-mcp/` = 0 hits de producción
   (13 matches totales: solo docs `dreams.rs:11`, tests seed-shape, `code.rs`
   pipeline ajeno, `mcp_tests.rs` write-pipeline genérico, server `pipelined`
   requests MOD-08 — ninguno es el scheduler). Ningún caller MCP encola
   (`enqueue_task` 0 hits en `vantadb-mcp`); `run_bridge_pass` vive en
   `vanta-memory` (`conversation_hook.rs:93-107`) y `PipelineFactory` arma el
   trío solo para hosts in-process (`pipeline_factory.rs:41-52`).
6. **El proceso MCP no admite dueño trivial (misma evidencia que FIND-110):**
   `run_stdio_server` long-lived por cliente (`server.rs:377-386`);
   `dispatch_request` stateless por request (`:560-602`:
   `spawn_blocking(handle_tools_call(&params,&executor,&storage,&cfg))`,
   limitación MOD-11 documentada); MCP-35 `run_stdio_server_auto` (`:833-907`)
   divide writer (abre DB) / proxy (HTTP al writer). Un backend `static`
   local divergería entre procesos salvo diseño writer-side + routing proxy
   + semántica restart-loss (pending/locks en RAM → restart = tareas
   silenciosamente perdidas, contradice never-lose-data del pipeline).
   `handle_tools_call` (`tools.rs:1220-1225`) recibe `(&params,&executor,
   &storage,&cfg)` — sin slot para backend compartido; añadirlo es cambiar la
   firma de 87 tools + proxy (diseño >1d).
7. **Coherencia con FIND-112 §(c):** el runner se construye POR LLAMADA porque
   es stateless+barato+con secrets; el scheduler exige estado COMPARTIDO entre
   llamadas (cola con tareas). Son patrones opuestos con justificación opuesta;
   la coherencia es que cada uno elige el lifecycle que su estado exige — y el
   del scheduler no existe en 1d. FIND-112 declara explícitamente que el
   scheduler "no lo posee" (`FIND-112.md:240`) y que FIND-113 decide el backend.

## 8. INVESTIGACIÓN PROBLEMA (tradeoffs decididos)

- **Dueño del backend (el punto que decide si la tarea cierra):** nadie hoy;
  dueño trivial (`static` process-local) diverge por MCP-35; dueño real
  (writer-owned + proxy-routing + restart-semantics + `scheduler_enqueue` path
  porque nada encola hoy) excede appetite 1d y roza persistencia prohibida por
  analogía (locks/pending on-disk = nuevo store). Ver Decisión con 6 evidencias.
- **Slice mínimo evaluado y rechazado (no escondido):** (i) `scheduler_status`
  sobre backend por-llamada = `{queue:0,pending:0,locks:0}` siempre (cola
  siempre vacía) = mostrador vacío prohibido; (ii) `scheduler_status` sobre
  `static` + `scheduler_run_once` con cola vacía = `{processed:0}` siempre +
  nuevo API público (Gate D) + restart-loss sin documentar + divergencia proxy
  = shippear ficción; (iii) test scope-proceso sin dueño (backend construido
  en el test) = probaría lo ya probado 21/21, no el lifecycle (ficción sin
  owner — alternativa rechazada en FIND-110 Review). Los tres violan ponytail
  (lo más simple que funciona es NO shippear — 0 líneas) y R-2 (no exportar
  internas sin callers externos).
- **Locks TTL multi-proceso:** scope-proceso (el backend lo es por diseño +
  techo CAS documentado) o re-DEFER. Elegido re-DEFER porque scope-proceso sin
  dueño ni productor es ficción (ver arriba). Pre-mortem 1 cerrado.
- **No duplicar MEM-65:** Dream reuse confirmado (dispatch + run_dream +
  tests). Cualquier `scheduler_run_once` que reimplemente Dream violaría la
  prohibición del contrato. Pre-mortem 2 cerrado.
- **Daemon/threads:** prohibido por contrato; además el server ya corre tokio
  (`serve_lines` async + `spawn_blocking` por request) — un hilo scheduler de
  fondo con `std::Mutex` backend exigiría auditoría Regla 8 (deadlock/data
  race) + `vanta-chaos` stress + `vanta-review` P2-01, todo >1d. Cerrado por
  prohibición, no por opinión.

## Spec (SDD — Phase 1b)

**Veredicto Phase 1b:** NO es feature-add — cero símbolos públicos nuevos
(ningún `pub fn`, tool MCP, endpoint o binding agregado). La decisión es
docs-only. Tabla de decisiones resuelta POR EVIDENCIA (no hay ronda
`question`: nada abierto que el repo no responda; decisiones owner FIND-112
ya cerradas, no re-derivadas):

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Dueño del backend en MCP stdio | A) backend por llamada (pro: sin estado global; contra: cola siempre vacía = mostrador vacío) / B) `static` process-local writer-side (pro: visible tras proxy si se enruta; contra: restart pierde pending/locks = pérdida silenciosa; divergencia proxy si se posee mal; cambia firma 87 tools) / C) persistencia on-disk (PROHIBIDA por analogía + fuera de appetite) / D) ningún dueño defendible en 1d → re-DEFER | ✅ D por evidencia (§7.5-7.7) |
| 2 | Ship `scheduler_status`/`scheduler_run_once` sin productor | A) shippear (status siempre ceros, run_once siempre `{processed:0}` — mostrador vacío) / B) no shippear | ✅ B — A es el mostrador vacío PROHIBIDO (espejo FIND-110 Decisión 2) |
| 3 | Scope-proceso test-only sin dueño | A) añadir tools + test con backend del test (nuevo API público → Gate D + diseño multi-proceso + restart-loss; excede 1d) / B) diferir | ✅ B — scope fuera de appetite (espejo FIND-110 Decisión 3) |
| 4 | Rediseñar Dream en el scheduler | A) reimplementar consolidación en `scheduler_run_once` / B) reusar `TaskKind::Dream` MEM-65 | ✅ B — prohibido por contrato (pre-mortem 2) |

**Gate D (question-gates.md):** evaluado ANTES del task file — blast radius de
ESCRITURA = 1 archivo `.md` (cero código); contrato no ambiguo (alternativa
re-DEFER de primera clase); cero símbolos públicos nuevos; feature-add sin
spec N/A (no es feature-add). → **no disparado** (sin `question`, con motivo).

## Decisión dueño (AC-a)

**DECISIÓN: re-DEFER fundado. No existe dueño defendible en 1d sin violar el
contrato (mostrador vacío, persistencia encubierta, o daemon prohibido).**

**Dir estado:** no hay ninguno — el backend vive solo en RAM del proceso que
lo construye (`Mutex<Inner>`); ningún dir estado existe ni se propone crearlo
en este slice (crearlo = persistencia nueva on-disk, fuera de appetite).

**Quién reclama locks:** nadie en MCP — los únicos reclamantes son
`run_bridge_pass` (host in-process que posee su `queue: Arc<LocalStateBackend>`)
y los workers del `PipelineFactory` trío. En MCP no hay `Arc` compartido entre
`tools/call` (cada request clona `storage`+`config`, nunca un backend).

Evidencia (toda in-repo, verificable por grep/lectura):

1. **El backend no tiene dueño en ningún proceso MCP.** Grep
   `LocalStateBackend|PipelineWorker` en `vantadb-mcp/src/*.rs` + `handlers/`:
   0 hits de producción. Los 16 callers de `LocalStateBackend` viven en
   `vanta-memory/src/utils/` + `services/` (codegraph blast radius).
2. **Nada encola tareas en MCP.** Grep `enqueue_task` en `vantadb-mcp/`: 0
   hits. Aunque existiera un dueño, la cola nacería vacía y `run_once`
   devolvería `{processed:0}` siempre: ni siquiera hay path captura→cola en
   este proceso (el único entry `HttpCaptureBridge::trigger` vive en
   `vanta-memory` y encola L1 sobre su propia `queue` Arc — otro proceso).
3. **El dispatch MCP es stateless por llamada.** `handle_tools_call(&params,
   &executor, &storage, &cfg)` (`tools.rs:1220-1225`) + `dispatch_request`
   (`server.rs:560-602`, `spawn_blocking` por request) + proxy
   (`:659-780`). Añadir backend compartido cambia la firma del dispatch de las
   87 tools + el proxy HTTP (diseño >1d).
4. **El split writer/proxy impide el dueño trivial.** `run_stdio_server_auto`
   (`server.rs:833-907`, MCP-35): el writer abre la DB, el resto proxean.
   Un `static LocalStateBackend` en cada proceso divergería (cada proxy con su
   cola vacía, locks reclamados por el proceso equivocado). El dueño real sería
   writer-side con routing + semántica restart-loss documentada — diseño con
   nuevo API público (`scheduler_enqueue` implícito, porque sin productor no
   hay qué correr) que excede 1d y roza la persistencia prohibida.
5. **El daemon está prohibido y el hilo de fondo exigiría auditoría Regla 8.**
   Contrato: "daemon/background-thread en MCP (fuera de appetite)". Además el
   worker toca locks por sesión + claim/lease + `reclaim_stale`; exponerlo tras
   `serve_lines` tokio (background tasks MOD-08) exigiría auditoría
   deadlock/data-race + stress 10k w/s + P2-01 por agente distinto — todo >1d.
6. **La mecánica en-proceso YA funciona** (21/21 verde, ver Verificación): el
   gap es 100% ownership/lifecycle, 0% mecánica. El test "mismo proceso corre
   tareas encoladas por MCP" no puede probarse porque no existe el lado
   "encola en MCP" — escribir un test que construye su propio backend probaría
   lo ya probado 21/21, no el lifecycle.

**Motivo re-DEFER (para la fila Backlog — la edita el orquestador/lead):**
"S6b sin dueño defendible: `LocalStateBackend` 0 callers en `vantadb-mcp` +
ningún path encola→cola en MCP + dispatch stateless + split writer/proxy
MCP-35 impide backend process-local trivial (diverge) + daemon prohibido +
dueño real (writer-owned + routing + restart-semantics + productor) excede 1d
y roza persistencia. Shippear status/run_once solo = mostrador vacío
(`{processed:0}` siempre, prohibido). `TaskKind::Dream` MEM-65 intacto y
testeado 2/2 — el gap es lifecycle, no código. Reabrir cuando: (i) exista
productor MCP que encole (`scheduler_enqueue` o hook captura en-proceso), o
(ii) se apruebe diseño writer-owned con semántica restart-loss documentada +
auditoría Regla 8."

**Implicancia SPEC.md F5 / FIND-112:** el scheduler no bloquea IMPL-112 (el
runner se construye por llamada, §3-Dependencias); sueños S2+S3 ✅ y degradado
P4 intactos — S6b era el único pendiente de exposición con gap de lifecycle.

## 9. INVESTIGACIÓN INTERNET

No requerida — incógnita uphill respondida con evidencia in-repo (backend +
server + grep exhaustivo). Cero citas externas, cero deuda TSYS-13. (Si un
futuro diseño writer-owned necesita patrones de liderazgo/leases
distribuidos, ese será el momento de research — no este slice.)

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `TaskKind::Dream` cableado MEM-65 intacto (no
  rediseñar, reusar); `LocalStateBackend` sin dueño global — cualquier futuro
  dueño debe ser explícito (writer-side) con semántica restart-loss
  documentada + CAS multi-proceso (techo `local_backend.rs:236-237`); NO añadir
  tools `scheduler_*` sin productor que encole (mostrador vacío prohibido); NO
  persistir cola/locks sin ADR (fuera de appetite S6b); NO daemon/threads en
  MCP sin auditoría Regla 8 + P2-01; `MemoryTaskHandler` genérico `R: LlmRunner`
  sin cambios (FIND-112 construye runners, no este slice)
- **Comandos de verificación:** `cargo test -p vanta-memory -j 2
  --test pipeline_manager` (21/21 ok) · `cargo fmt --check` N/A (cero Rust
  tocado) · clippy N/A (cero Rust tocado)
- **Deuda pendiente:** ninguna nueva (saldo Regla 6: 0). Deuda conocida que
  ESTE re-defer registra (no crea): S6b-lifecycle sin dueño — vive en la fila
  Backlog FIND-113 con el motivo de arriba

## Recitation (canónico)

- `activeGoal`: FIND-113 S6b programador — dueño backend o re-DEFER
- `lastAction`: DISCOVERY completo (pipeline_worker 872L + local_backend 494L
  + types + hook + factory + server + tools + dreams + FIND-112 §c + FIND-110
  espejo) + grep exhaustivo workspace + mecánica 21/21 verde + decisión
  re-DEFER escrita en `docs/tasks/FIND-113.md` + commit `docs:` solo-task-file
- `result`: OK (re-DEFER fundado = cierre válido del contrato alternativo)
- `nextAction`: orquestador — actualizar fila Backlog FIND-113 con motivo +
  P2-01 sobre la decisión + NextTask FIND-101
- `contract`: verificacion `cargo test -p vanta-memory -j 2
  --test pipeline_manager` → 21 passed / 0 failed · evidencia: cada claim de
  Decisión cita `file:línea` + output del test · artefactos:
  `docs/tasks/FIND-113.md` · invariantes: Dream intacto, sin dueño global, sin
  tools sin productor, sin persistencia sin ADR, sin daemon · deuda: ninguna
  nueva · queda_pendiente: Backlog (lead) + P2-01 (orquestador) + FIND-101
- `nextTask`: FIND-101

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (docs-only, cero código).

## 10. VALIDACIÓN + CIERRE — Definition of Done (3 niveles — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Contrato alternativo cumplido (decisión + mecánica verde existente verificada + motivo) + determinista aplicable (fmt/clippy N/A sin Rust) + tests del cambio N/A (sin cambio; 21/21 existentes como evidencia) | ✅ |
| **Commit** | Atómico (1 archivo nuevo), conventional `docs:`, staging selectivo verificado (`git status`), sin WIP ajeno, NO PUSH | ✅ |
| **Release** | N/A — tarea docs sin artefacto shippable; `dev-tools/verify.ps1` completo no aplica (cero código). Justificado en Notas | N/A justificado |

## Herramientas necesarias (resumen §6)

- `cargo test -p vanta-memory -j 2` (mecánica 21/21) · codegraph_explore (blast
  radius 29 símbolos) · grep workspace (exhaustividad enqueue/owner) ·
  `campaign_verify_cmd` N/A (cero código; bug exit -1 no alcanzado — bash
  directa usada) · OCR delegation (cierre, advisory)

**Notion (Paso 0c):** sin tool disponible — registrado como limitación (§4).

## Investigation Notes

- MEM-68/host-opt-in (analogía FIND-110): el backend está diseñado para hosts
  in-process (`Arc<LocalStateBackend>` poseído por el host:
  `conversation_hook.rs:11-16` ejemplo `Arc::new(LocalStateBackend::new(
  SystemClock))`). El "host" en MCP sería el server — pero el server MCP no
  corre pipeline (sus tools son CRUD memoria/grafos/threads/dreams/wiki, 87
  registradas, cero scheduler). El productor natural (hook captura) vive en
  otro proceso → el scheduler MCP nacería vacío por construcción.
- Contraste S2+S3 (dreams, shippeado FIND-107) + S5 (skill_extract, shippeado
  FIND-111): dreams persisten en `dream/<session>/<run_id>` y skills leen el
  store compartido writer-side sin estado process-local. Scheduler no tiene
  store → no hay plantilla aplicable sin diseñar ownership (igual que approval).
- `dispatch_request` ejecuta `handle_tools_call` writer-side para llamadas
  proxeadas vía `mcp_proxy_handler` (patrón FIND-110): un futuro dueño
  writer-side SÍ sería visible a todos los clientes. Camino técnico existe,
  pero exige (productor + dueño + restart-semantics + Regla 8) = diseño >1d.
  Anotado como reapertura, no como slice escondido.
- Coherencia scheduler↔runner (FIND-112 §c): cuando el scheduler exista,
  pedirá un runner al mismo constructor (`build_ingest_runner`) con su propio
  config — hoy no existe scheduler que lo pida; el runner por llamada no lo
  necesita. Sin bloqueo cruzado.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — "¿quién posee el backend en MCP stdio?" RESPONDIDA: nadie hoy; dueño trivial diverge por MCP-35; dueño real excede appetite; daemon prohibido |
| Pendientes de ejecución (downhill) | 0 — ship cancelado por decisión fundada |
| % completado | 100% (discovery + decisión + task file + commit) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — N/A con justificación: cero código, cero trust
  boundaries tocados. Nota para reapertura: un futuro `scheduler_run_once`
  aceptaría `session_id`s externos → validar en frontera (patrón
  `dreams.rs`: `validate_identifier`) y auditar restart-loss como integridad
  de datos (pending en RAM ≠ durable); `api_key` nunca en TOML (precedente
  FIND-112).
- [x] **PERFORMANCE** — N/A con justificación: backend `Mutex<Inner>` + scan
  O(n) `pending` (techo `ponytail` documentado, tiny por diseño); a escala
  scheduler humano es irrelevante; si la reapertura lo convierte en store
  persistido o hilo de fondo, exigir baseline before/after (Regla 9,
  `canonical_p99`) + auditoría Regla 8 (deadlock/data-race) + stress
  `vanta-chaos` 10k w/s.

## Steps

### Step 1: DISCOVERY worker + backend + tipos fuente ✅ COMPLETED

- **Archivos:** `pipeline_worker.rs:1-120,198-335,342-565,804-820`,
  `local_backend.rs` (494L), `types.rs` (112L)
- **Acción:** leer completos vía codegraph_explore + Read; confirmar run_once
  + LocalStateBackend + locks TTL existen; Dream cableado MEM-65 (no duplicar)
- **Verify:** citas `pipeline_worker.rs:198-324`, `local_backend.rs:228-237,
  353-392`, `types.rs:38-51` confirmadas
- **Estado:** ✅ COMPLETED

### Step 2: Investigación uphill — dueño del backend en MCP stdio ✅ COMPLETED

- **Archivos:** `vantadb-mcp/src/server.rs` (`:377,560-602,659,833-907`),
  `handlers/tools.rs` (`:1220-1225`, registry `:25-27`), `dreams.rs`
  (plantilla 267L), `conversation_hook.rs`, `pipeline_factory.rs`
- **Acción:** responder con evidencia del server (no opinión): loop
  long-lived + split writer/proxy MCP-35 + dispatch stateless + 0 callers MCP
  + 0 enqueue en MCP + daemon prohibido
- **Verify:** citas server + firma dispatch sin estado + grep 0 hits
- **Estado:** ✅ COMPLETED

### Step 3: Exhaustividad productor + coherencia FIND-112 §c + mecánica verde ✅ COMPLETED

- **Archivos:** workspace `*.rs` (grep `enqueue_task|LocalStateBackend|
  PipelineWorker` en `vantadb-mcp/`), `docs/tasks/FIND-112.md` §(c)
- **Acción:** grep exhaustivo; leer FIND-112 §c sin editar; correr
  `cargo test -p vanta-memory -j 2 --test pipeline_manager` (lista + run)
- **Verify:** 0 callers prod en MCP; coherencia runner-vs-scheduler
  documentada; 21/21 tests ok (Dream 2/2, locks, reclaim)
- **Estado:** ✅ COMPLETED

### Step 4: Decisión dueño + task file completo ✅ COMPLETED

- **Archivos:** `docs/tasks/FIND-113.md` (nuevo, este archivo)
- **Acción:** poblar TODO por tarea (10 puntos obligatorios): decisión
  re-DEFER con 6 evidencias + motivo Backlog + Spec Phase 1b + DoD 3 niveles
  + Impacto Regla 0 + Gates
- **Verify:** secciones Metadata→Verificación completas, sin referencias vacías
- **Estado:** ✅ COMPLETED

### Step 5: Verify + OCR + commit selectivo + RESULTADO ✅ COMPLETED

- **Archivos:** `docs/tasks/FIND-113.md` (único en staging)
- **Acción:** `git diff --check` + `git status` selectivo + `pwsh
  dev-tools/ocr-review.ps1` (advisory) + `git add docs/tasks/FIND-113.md` +
  `git commit -m "docs: FIND-113 — ..."` (NO PUSH) + `campaign_update_task_state`
  + `campaign_memory_write` + bloque RESULTADO §7
- **Verify:** `git status` muestra solo el commit nuevo; WIP ajeno intacto
- **Estado:** ✅ COMPLETED (ver § Verificación)
- **OCR:** `pwsh dev-tools/ocr-review.ps1 -Format json` → `docs/tasks/FIND-113.md`
  excluido (`unsupported_ext`, no revisable por OCR — igual que FIND-110);
  únicos reviewables son WIP ajeno (`completions/*`, `.opencode`) — fuera de
  scope, no bloquean este commit. Cero Critical/High propios.

## Impacto mapeado (Regla 0) — MUST antes de la primera edición

- **Archivos leídos (completos):** pipeline_worker.rs (872L: §§1-120,
  198-335, 545-565, 700-820), local_backend.rs (494L), types.rs (112L),
  conversation_hook.rs (107L), pipeline_factory.rs (53L), dreams.rs (267L),
  handlers/tools.rs (cabecera registry + `handle_tools_call:1220-1225` +
  dispatch `:2911,2924`), handlers/mod.rs, server.rs (`:360-459,560-639,
  800-907`), FIND-112.md (307L), FIND-110.md (408L espejo), FIND-107.md §§S6b,
  plan file Task 5, Backlog fila FIND-113, rules core-engine.md (40L),
  definition-of-done.md, clean-code Ap. V (`:646-697`)
- **Archivos referenciados hacia dentro:** pipeline_worker → local_backend
  + state types + checkpoint + L0/L1/L2/L3/dream/context_engine/offload;
  local_backend → state types + managed_timer Clock; conversation_hook →
  pipeline_worker + local_backend + Embedded; factory → los tres; server →
  handlers/tools + StorageEngine + Executor + proxy; dreams → core::dream +
  Embedded
- **Archivos que referencian a los editados:** NINGUNO — esta tarea no edita
  código; el único archivo creado es este task file (sin referencias
  entrantes salvo el plan file recitation)
- **Veredicto impacto:** NULO en código (docs-only). Si alguien shippeara los
  2 tools mañana, el impacto sería: `handlers/tools.rs` (registry + dispatch +
  firma con backend), `server.rs` (estado compartido + proxy routing),
  `docs/api/MCP.md` (R-5 paridad), tests MCP nuevos, posible `scheduler.rs`
  nuevo — y seguiría vacío el mostrador (ver Decisión).

## Dependencias (resumen §3)

- FIND-112 ✅ (`062300a8`) → esta tarea → NextTask FIND-101 (orquestador)
- Stop aplicado: re-DEFER con motivo (este archivo + RESULTADO)

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta el ORQUESTADOR (no el implementador — instrucción de tarea).

- **Revisor:** pendiente (orquestador asigna vanta-review/vanta-audit)
- **Enfoque:** ¿re-DEFER fundado o había dueño shippeable en 1d? Alternativas
  evaluadas: ship-2-tools por llamada (mostrador vacío, prohibido), ship con
  `static` writer-side (nuevo API + restart-loss + firma 87 tools, >1d), test
  scope-proceso sin dueño (ficción, prueba lo ya probado), daemon/hilo
  (prohibido + Regla 8), rediseño Dream (prohibido MEM-65)
- **Cómo se probó:** `cargo test -p vanta-memory -j 2 --test pipeline_manager`
  → 21 passed / 0 failed (mecánica); decisión por grep exhaustivo + lecturas
  citadas (evidencia, no auto-reporte)
- **OCR:** delegation corrida — `FIND-113.md` excluido `unsupported_ext`;
  reviewables solo WIP ajeno (`completions/*`); cero Critical/High propios,
  no bloquea
- **Checklist anti-hábitos tóxicos:** sin salidas inventadas (test output
  real pegado en Verificación); sin clarificación saltada (uphill respondida
  con evidencia); sin done sin AC (AC-a/b/c mapeados); sin fallos ignorados;
  sin búsqueda única (codegraph + grep + lectura directa server.rs + suite
  mecánica); supuestos citados como evidencia con `file:línea`; sin reintentos
  en bucle; cada step conectado al contrato; N/A dinero/seguridad (docs-only);
  presupuesto explícito (stop 1d aplicado como re-DEFER, no como FAILED)
- **Veredicto:** pendiente revisor distinto

## Notas

- Re-DEFER válido ≠ fracaso: el contrato del plan lo lista como cierre
  alternativo de primera clase ("o re-DEFER con motivo (NO es fracaso)"). El
  valor entregado es certeza con evidencia (qué falta exactamente para
  reabrir) en vez de un mostrador vacío (`{processed:0}`) que erosionaría
  confianza (R-3 api-contract, espejo FIND-110).
- `ponytail:` lo más simple que funciona aquí es NO shippear — 0 líneas.
  Skipped: tools/docs/coverage S6b; add cuando exista productor que encole o
  diseño writer-owned aprobado (ver motivo re-DEFER).
- NOTICED BUT NOT TOUCHING: WIP ajeno en `git status` (`.opencode`,
  `completions/*`, `docs/Backlog.md` modificados; `reparacion.bat` y plan file
  untracked) — fuera de scope, no se tocan ni se stagean. `docs/Backlog.md`
  fila FIND-113 la actualiza el orquestador/lead con el motivo de § Decisión.
- DoD Release N/A justificado: sin artefacto shippable no hay qué certificar
  con `verify.ps1` completo; la capa determinista aplicable (nada que
  compilar/lintear) se cumple por vacuidad + suite mecánica verde.
- Doubt-driven (degradado, anunciado): sin `task`/sub-agente reviewer
  disponible para P2-01 en este contexto (lo hace el orquestador); el ciclo
  adversarial se ejecutó como auto-revisión contra las 5 alternativas de
  Review/Enfoque. Cross-model skip: contexto no-interactivo.
- Coherencia FIND-112: esta decisión NO contradice §(c) — la confirma (runner
  por llamada vs scheduler compartido son lifecycles opuestos, cada uno
  correcto para su estado).

## Verificación

- `cargo test -p vanta-memory -j 2 --test pipeline_manager` — ✅ 21 passed,
  0 failed (output real):

```text
running 21 tests
test locks_are_owner_scoped_and_expire_by_clock ... ok
test idle_timer_flushes_quiet_session_via_scanner ... ok
test dead_worker_claim_reclaimed_and_processed_by_new_owner ... ok
test capture_atomic_triggers_at_threshold_and_resets_counter ... ok
test queue_orders_by_priority_then_creation_time ... ok
test set_timer_if_earlier_is_downward_only ... ok
test manager_warmup_doubles_until_cap ... ok
test stale_pending_task_reclaimed_once_by_new_owner ... ok
test timer_scanner_dispatches_expired_entries ... ok
test timers_fire_only_when_expired_and_are_consumed_once ... ok
test worker_retries_then_dead_letters_failing_tasks ... ok
test worker_skips_locked_sessions_without_losing_tasks ... ok
test stateful_manager_persists_states_through_callback ... ok
test handler_dream_task_skips_quietly_when_not_idle ... ok
test full_worker_pass_records_l0_then_runs_l1_noop ... ok
test handler_l1_noop_on_empty_session_and_l3_skips_when_quiet ... ok
test checkpoint_counters_feed_the_persona_trigger ... ok
test handler_dream_task_consolidates_session_without_touching_l1 ... ok
test handler_l3_generates_persona_from_checkpoint_request ... ok
test handler_l1_default_path_still_uses_two_calls ... ok
test handler_l1_batch_flag_fuses_extract_and_dedup_in_one_call ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s
```

- `cargo check -p vantadb` — N/A (cero Rust tocado)
- `cargo check -p vantadb_py` — N/A (cero bindings tocados)
- `campaign_verify_cmd` — no corrido sobre el diff (cero código; bug exit -1
  no alcanzado; bash directa usada para la suite mecánica)
- `git diff --check` — ✅ limpio (verificado al cierre)
- OCR delegation — ver Step 5
- **Scope discipline:** 1 archivo nuevo (`docs/tasks/FIND-113.md`), 0
  ediciones. WIP ajeno intacto.
- **Context health:** output siguió convenciones (citas `file:línea` reales,
  comandos del stack VantaDB con `-j 2`, sin APIs inventadas — `scheduler_*`
  citados como propuestos, nunca como existentes).
