# FIND-113-spec: S6b programador — dueño del backend + superficie mínima (spec-first, cero código)

## Metadata

- **Plan file:** `docs/plans/2026-09-18-cierre-mvp.md` (Task 3, Wave0 tercera en secuencia)
- **Fuente:** `docs/tasks/FIND-113.md` (re-DEFER fundado 2026-09-18: 0 productor, daemon prohibido, MEM-65 intacto) + Gate Justificación del plan (espejo FIND-110-spec; cierra el par S4/S6b en diseño)
- **Esfuerzo:** 🟡 1d (appetite plan) · **Prioridad:** 🟡 Media
- **Tipo:** spec-first docs-only (cero código; auto-detect `docs`, ver Dependencias para el matiz)
- **Creado:** 2026-09-18
- **last-synced:** 2026-09-18
- **Estado:** ⬜ PENDING → 🟡 IN PROGRESS (esta ejecución escribe la spec; el file ES el entregable)
- **Incógnitas (uphill):** 3 iniciales (dueño backend / locks TTL multi-proceso / coherencia-112) → 0 abiertas (resueltas con evidencia, ver Investigación problema)
- **Pendientes (downhill):** 0 (spec-first: nada que ejecutar en código; slices futuros quedan mecánicos, no pendientes de esta tarea)

## 1. TAREA — objetivo + contrato + AC

**Objetivo:** producir el diseño del dueño del scheduler S6b que faltaba en FIND-113, para volver el re-DEFER un slice mecánico futuro. Spec-first, cero código.

**Re-DEFER FIND-113 fundado (no re-derivado, heredado como base):** 0 productor en MCP, daemon/background-thread prohibido, MEM-65 (`TaskKind::Dream` cableado) intacto. Lo que faltaba era el diseño: dueño explícito + superficie mínima + restart + coherencia con FIND-112 §(c). Esta spec lo escribe.

**Coherencia obligada con FIND-112 §(c):** el runner se construye por-llamada (stateless, barato, con secrets); el scheduler debe ser coherente con ese lifecycle, no idéntico a él (ver `## Coherencia con FIND-112 §(c)`).

**Contrato (plan Task 3):** `docs/tasks/FIND-113-spec.md` (este archivo) con (a) dueño del backend (dir estado, quién reclama locks TTL, scope-proceso), (b) superficie mínima (`scheduler_status`/`run_once` o equivalente), (c) gates+tests futuros, (d) coherencia con FIND-112 §(c) escrita + P2-01-spec; **cero código**.

**AC de esta ejecución:**

- (a) ✅ dueño del backend con dir-estado, reclamante de locks y scope explícitos (ver `## Diseño dueño`) — sin dueño no cierra (Stop del plan); hay dueño defendible, la spec cierra
- (b) ✅ superficie mínima como diseño sin código (ver `## Superficie mínima`)
- (c) ✅ gates+tests futuros nombrados y verificables (ver `## Gates y tests futuros`)
- (d) ✅ coherencia con FIND-112 §(c) escrita + qué NO cambia con tradeoff persistencia + P2-01-spec para el orquestador (ver `## Coherencia con FIND-112 §(c)`, `## Qué NO cambia` y `## Review`)

## 2. ARCHIVOS — clave + relacionados + prohibidos

**Entregable (única creación permitida):**

- `docs/tasks/FIND-113-spec.md` (ESTE archivo — ES el entregable)

**Lectura (leídos completos en DISCOVERY, verbo vía `codegraph_explore` + `Read`):**

- `docs/tasks/FIND-113.md` (665L) — re-DEFER fundado con 6 evidencias + Spec Phase 1b + motivo Backlog; base de todo el diseño (no re-derivado, heredado)
- `vanta-memory/src/services/pipeline_worker.rs:1-130` — doc crate (single process MEM-16, sin Prometheus, `claimStaleTasks` portado MEM-66 `:16-18`), `TaskHandler` trait `:49-52`, `WorkerConfig` `:56-73` (max_retries 3, lock_ttl 60s, batch 8), `RunStats` `:82-88`, `LayerTelemetry` MEM-65 `:90-130`
- `vanta-memory/src/services/pipeline_worker.rs:198-335` — `PipelineWorker` (`backend`, `config`, `owner: worker-{pid}` `:199-211`), `with_owner` `:219-222` (2 workers mismo proceso colisionarían sin esto), `run_once` `:232-246` (claim loop hasta `batch_size`), `reclaim_stale` `:251-267`, `run_task` `:272-324` (session lock + handler + settle retry/dead-letter, release SIEMPRE antes de actuar `:295`)
- `vanta-memory/src/services/pipeline_worker.rs:540-565` — `run_dream` (detect_idle + consolidate_session, skip quiet `Ok` si no idle `:559-561`, sin locks nuevos)
- `vanta-memory/src/services/pipeline_worker.rs:804-820` — dispatch `handle` (`L1|Flush→run_l1`, `L2→run_l2`, `Dream→run_dream`, `L3→run_l3+assembly`)
- `vanta-memory/src/utils/local_backend.rs:1-70` — doc in-process (no Redis Principio 7, un mutex, Clock inyectado `:1-10`), `Inner` `:20-34` (buffers/states/timers/queue/`pending`/`locks`), struct `:39-42` ("wrap in `Arc` if several owners need it" `:36-37`)
- `vanta-memory/src/utils/local_backend.rs:228-349` — claims multi-worker MEM-66 (`claim_task` `:241-257`, `complete_task` fencing `:261-270`, `renew_task_lease` `:276-286`, `claim_stale_tasks` `:294-318` con `ponytail: O(n)` `:293`, `requeue_task` `:324-344`), techo multi-proceso explícito `:236-237` ("Multi-process would need a CAS on the lease (documented ceiling, YAGNI here)")
- `vanta-memory/src/utils/local_backend.rs:351-392` — locks TTL (`acquire` `:359-371` con expired colectados primero, `renew` `:374-384`, `release` `:387-392`, todo owner-scoped)
- `vanta-memory/src/core/state/types.rs:38-70` — `TaskKind` `:38-51` (L1/L2/L3/`Dream` que escribe `dream/<s>/<run_id>` sin mutar L1 `:45-48`, `Flush`), `TaskPayload` `:56-70` (id `t_{ms}_{seq}`, priority 0/1/2, attempts)
- `vanta-memory/src/utils/timer_scanner.rs:1-38` — precedente pull-based MEM-16: "the owner calls `run_once` whenever it wants" (sin background interval de TDAM)
- `docs/tasks/FIND-112.md` §(c) (líneas 209-240, lectura SIN editar) — dueño runner por-llamada + 5 puntos lifecycle; referencia de coherencia, no de edición
- `docs/tasks/FIND-110-spec.md` (339L, commit `89b118be`) — hermana S4, coherencia de estilo (slices A+B, tests nombrados, gates G0–G3, condición de ship, restart documentado)

**Referencias normativas (lectura completa, ver `## 4. REFERENCIAS`):**

- `.opencode/rules/core-engine.md` (R-1–R-5) — citada donde aplica
- `.opencode/references/definition-of-done.md` (standing checklist + DoD VantaDB + v1 baseline)
- `.opencode/commands/pipeline.md` (vía `pipeline-full.md` exacto — este prompt)
- `SPEC.md` raíz (§ Alcance cierre-mvp: "S4/S6b diseño primero en FIND-110-spec/FIND-113-spec (cero código); ship solo con dueño defendible, si no re-DEFER honesto")
- `docs/plans/2026-09-18-cierre-mvp.md` Task 3 (solo recitation/contrato — el plan file no se edita desde la tarea)
- **Tabla Spec:** LA SPEC MISMA (este archivo: tabla de decisiones Phase 1b + diseño + gates; no hay tabla por-tool porque no se shippea herramienta alguna en esta tarea)

**Prohibidos (NO tocados — verificación en `git status` al cierre):**

- Todo `src/`, `vanta-memory/src/`, `skills/`, `examples/`, `scripts/`, `reparacion.bat`, `.opencode`, `Justfile`, `ocr-*`, `completions/*`, `desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4, `docs/Backlog.md` (edición del lead), plan file (solo recitation), `C:/Users/Eros/.vantadb*`, WIP ajeno en `git status` (`.opencode`, `SPEC.md`, `completions/*`, `docs/Backlog.md` modificados — intactos, no stageados)

## Blast Radius

Cero código tocado → cero blast radius en código. Mapeo de lectura (qué tocarían los slices futuros, verificado vía `codegraph_explore` en DISCOVERY):

| Dirección | Módulos |
|-----------|---------|
| Callers de `PipelineWorker::run_once` | 5 en `vanta-memory/src/services/conversation_hook.rs` (host in-process) + tests; CERO en `vantadb-mcp` (confirmado por grep FIND-113) |
| Callers de `LocalStateBackend` | 16 en `vanta-memory/src/utils/` + `services/`; CERO en `vantadb-mcp` |
| Callers de `TaskKind` | 3 en `vanta-memory/src/core/state/mod.rs`; tests `e2e_flow.rs`, `pipeline_manager.rs` |
| Dueño futuro (diseño, no código hoy) | `vantadb-mcp/src/server.rs` — contexto del writer (`run_stdio_server`, split writer/proxy MCP-35, `server.rs:377-386,833-907` vía FIND-113) + `vantadb-mcp/src/handlers/tools.rs` (registry + dispatch cuando shippeen las 2 tools) |
| Precedente driver pull-based | `vanta-memory/src/utils/timer_scanner.rs` — `TimerScanner::run_once` (MEM-16, sin intervalo de fondo) |
| Implicaciones | este archivo es docs-only: ningún contrato cambia, ningún test afectado, ningún símbolo nuevo |

## Impacto mapeado (Regla 0) — MUST antes de la primera edición

- **Archivos leídos (completos):** `docs/tasks/FIND-113.md` (665L), `pipeline_worker.rs` (§§1-130, 198-335, 540-565, 804-820), `local_backend.rs` (§§1-70, 228-392), `types.rs` (`:38-70`), `timer_scanner.rs` (38L), `FIND-112.md` §(c) (`:209-240`), `FIND-110-spec.md` (339L), rules `core-engine.md` (40L), `definition-of-done.md`, `SPEC.md` § Alcance cierre-mvp, plan file Task 3 (recitation/contrato)
- **Archivos referenciados hacia dentro:** este spec referencia (no edita) `server.rs` (writer/proxy MCP-35), `handlers/tools.rs` (dispatch stateless), `conversation_hook.rs` (único caller productivo de `run_once`), `pipeline_factory.rs` (trío para hosts in-process), `dreams.rs` (doc "the real merge is MEM-65's pipeline worker") — evidencia heredada de FIND-113, no re-derivada
- **Archivos que referencian a los editados:** NINGUNO — el único archivo creado es este task file (sin referencias entrantes salvo el plan file y la próxima recitation)
- **Veredicto impacto:** NULO en código (docs-only, cero símbolos nuevos, cero ediciones). Si alguien ejecuta los slices futuros, el impacto sería: contexto del server writer (1 sitio: tenencia del `Arc`), `handlers/tools.rs` (registry + dispatch de 2 tools), `docs/api/MCP.md` (R-5 paridad en el mismo PR), tests MCP nuevos — worker, backend, tipos y MEM-65 intactos

## 3. DEPENDENCIAS

- **Wave0 tercera en secuencia.** FIND-98 ✅ (re-DEFER con evidencia, commit `4187eabf`), FIND-110-spec ✅ (spec 339L, commit `89b118be`) — archivos disjuntos (fuera-del-repo · `docs/tasks/110` · `docs/tasks/113`), sin interferencia.
- **Después:** IMPL-112-S1 (orquestador; Wave1; runner local de ingesta — archivos disjuntos `ingest/wiki/mcp`).
- **Coherencia FIND-112 §(c) (leído, no editado):** el scheduler NO puede seguir el patrón por-llamada del runner (ver `## Coherencia`); la coherencia es de lifecycle-opuesto-justificado, no de copia. FIND-112 declara que el scheduler "no lo posee" (`FIND-112.md:240`) y que FIND-113 decide el backend — esta spec es esa decisión en forma de diseño.
- **Stop del plan evaluado:** dueño defendible SÍ existe en diseño (writer-side `Arc` + pull-based explícito + restart documentado, ver `## Diseño dueño`) → la spec cierra; el Stop queda documentado para la EJECUCIÓN futura (si los slices revelan que el productor no puede vivir en el writer → re-DEFER honesto con esta spec como diseño parcial, no forzar).
- **NextTask:** IMPL-112-S1 (orquestador).

## 4. REFERENCIAS

- **Rules (leída completa):** `.opencode/rules/core-engine.md` (40L — R-1 feature-gating `experimental-*` sin default; R-2 internas sin callers no al SDK; R-3 `?`+`Result`, sin unwrap salvo `sync_ext`; R-4 `unsafe`+`SAFETY`; R-5 prefijo único `VANTADB_*` + `parse_env_or`, warn+default nunca panic). Aplicación: R-2 (exponer `run_once` sin dueño/productor MCP = exportar interna sin callers externos — prohibido; por eso el ship se condiciona al Slice B); R-1 (si un slice futuro añade variante con dependencia nueva, va tras flag `experimental-*`, no en default).
- **Refs:** `definition-of-done.md` (standing checklist + DoD VantaDB + v1 baseline + progreso Trigger 1 — ver § DoD 3 niveles).
- **Commands:** `pipeline.md` (vía `pipeline-full.md` exacto — este prompt).
- **SPEC.md raíz** (§ Alcance cierre-mvp 2026-09-18: "S4/S6b diseño primero en FIND-110-spec/FIND-113-spec (cero código); ship solo con dueño defendible, si no re-DEFER honesto" + Open Questions: "productor `submit` S4 y dueño backend S6b (specs 110/113)").
- **Tabla Spec:** LA SPEC MISMA (tabla de decisiones Phase 1b + diseño + gates en este archivo; sin tabla por-tool: no se shippea herramienta alguna en esta tarea).

## 5. SKILLS

**Cargadas (SDP real Paso 0b, `campaign_discover_skills_v2` phase DEFINE, maxSkills 8, keywords scheduler/dueño-backend/locks-TTL/run_once/spec-first/diseño-lifecycle):**

- `spec-driven-development` (núcleo: tabla decisiones Phase 1b + diseño dueño/superficie/gates — también sugerida por el plan)
- `documentation-and-adrs` (spec como documento de decisión + registro de tradeoffs)
- `writing-plans` (estructura del task file por los 10 puntos obligatorios)
- `writing-guidelines` (prosa docs)
- `interview-me` + `idea-refine` (lifecycle DEFINE, base SDP — sin ronda `question`: nada abierto que el repo no responda; decisiones owner-type ya cerradas en FIND-113, re-preguntar sería ruido)
- `doubt-driven-development` (añadida por sugerencia del plan: dueño difuso + pre-mortem locks/daemon; ciclo adversarial degradado anunciado — contexto no-interactivo, P2-01 la hace el orquestador)
- `codebase-memory` (sugerida por el plan: blast radius KG vía `codegraph_explore` + grep workspace; incluye ritual de mantenimiento)

`SDP: spec-driven-development · documentation-and-adrs · writing-plans · writing-guidelines · interview-me · idea-refine (6 SDP) + doubt-driven-development · codebase-memory (2 sugeridas por el plan, justificadas) + base campaign-executor/progreso/ponytail(full).`

`SKILLS_CARGADAS: spec-driven-development, documentation-and-adrs, writing-plans, writing-guidelines, interview-me, idea-refine, doubt-driven-development, codebase-memory (+ base campaign-executor/progreso/ponytail full)`

## 6. HERRAMIENTAS + MCP

- `codegraph_explore "pipeline_worker run_once LocalStateBackend locks TTL TaskKind Dream scheduler"` — blast radius inmediato: 91 símbolos/5 files; `TaskKind` 3 callers, `run_once(pipeline_worker)` 5 callers en `conversation_hook.rs`, `LocalStateBackend` 16 callers en `utils/*`+`services/` — CERO en `vantadb-mcp` (heredado de FIND-113, confirmado por grep)
- `Read` directo (ver § Archivos — paths exactos con líneas)
- `campaign_detect_task_type` → `docs/Documentation` (checks `validate-docs-coverage.ps1`; el contenido es spec de diseño lifecycle Rust-core/MCP — skills SDD+DDD aplicadas por contrato del plan, no por el label; mismo matiz que FIND-110-spec)
- `campaign_discover_skills_v2` phase DEFINE → 8 skills (ver §5)
- `campaign_get_workflow(feature-add)` → perfil spec/implement/verify/review/accept/close (guía; enforcement = C0 genérica; esta tarea vive entera en `spec`)
- `campaign_verify_cmd`: bug exit -1 conocido → N/A con motivo (cero código: no hay clippy/fmt/nextest que correr sobre el diff; sí `git diff --check` por bash directa al cierre, precedente plan Riesgos globales)
- OCR delegation: N/A-justificado (cero código; `.md` excluido `unsupported_ext`, precedente FIND-113 Step 5)
- Internet: no requerida — incógnitas uphill respondidas con evidencia in-repo (backend + worker + FIND-112 §c + precedente pull-based). Cero citas externas, cero deuda TSYS-13

## 7. INVESTIGACIÓN CÓDIGO (DISCOVERY — evidencia de primera mano)

1. **Worker existe y funciona en-proceso:** `PipelineWorker::new` defaultea `owner: worker-{pid}` (`pipeline_worker.rs:207-214`); `with_owner` para multi-worker (`:219-222`); `run_once` consume hasta `batch_size` vía `claim_task(owner, lock_ttl)` (`:232-246`); `run_task` adquiere `pipeline_lock:{session}` con TTL, ejecuta `handler.handle`, libera ANTES de actuar, settle complete/retry/dead-letter (`:272-324`).
2. **Backend es scope-proceso por diseño:** `LocalStateBackend` = `Mutex<Inner>` único (`local_backend.rs:39-42`), Clock inyectado, `pending` + `locks` con leases TTL + fencing por owner; invita `Arc` explícito (`:36-37`: "Cheap to clone-free share: wrap in `Arc` if several owners need it"). Techo documentado `:236-237`: "Multi-process would need a CAS on the lease (documented ceiling, YAGNI here)".
3. **Locks TTL reclamables solo en-proceso:** `claim_task/complete/renew/claim_stale/requeue` (`:241-344`) operan bajo UN mutex en UNA sección crítica — "double-reclaim is impossible in-process". Entre procesos no hay CAS: dos writers reclamarían el mismo lease (el `expire_at` vive en RAM de cada proceso). Scope-proceso o re-DEFER (pre-mortem 1 del plan — resuelto: scope-proceso writer, ver Diseño).
4. **MEM-65 Dream ya cableado — NO duplicar (pre-mortem 2 del plan):** dispatch `handle` matchea `TaskKind::Dream → run_dream` (`:809`); `run_dream` (`:545-565`) valida `detect_idle` con los mismos args que `consolidate_session` re-valida, escribe a `dream/<s>/<run_id>`, nunca muta L1. Reusar, no rediseñar: el scheduler ejecuta `Dream` pasando el `MemoryTaskHandler` existente, sin reimplementar consolidación.
5. **Tipos estables:** `TaskKind` L1/L2/L3/`Dream`/`Flush` (`types.rs:38-51`), `TaskPayload` con id/priority/attempts (`:56-70`). La superficie futura habla estos tipos; no se propone ningún tipo nuevo.
6. **Precedente pull-based sin daemon (MEM-16):** `TimerScanner::run_once` — "the owner calls `run_once` whenever it wants due timers dispatched" (`timer_scanner.rs:1-7`). TDAM corría un intervalo de fondo; el port lo eliminó a propósito. El scheduler sigue el mismo patrón establecido: pass explícito disparado por el dueño, cero threads de fondo. Esto cierra "daemon prohibido" con precedente in-repo, no con opinión.
7. **Coherencia FIND-112 §(c) (leído, no editado):** dueño runner = proceso MCP, registry `ingest_runs()` (`wiki.rs:377-380`), construcción POR LLAMADA porque (a) env mutable, (b) runner barato (Strings, sin conexión), (c) sin secrets en static (`FIND-112.md:214-222`). El scheduler NO puede seguir ese patrón para el backend: el backend debe ser COMPARTIDO entre llamadas para tener tareas; construcción por llamada = cola siempre vacía = mostrador vacío. Pero SÍ lo sigue para el runner de sus passes (ver Coherencia).

## 8. INVESTIGACIÓN PROBLEMA (tradeoffs decididos)

- **Dueño del backend (el punto que decide si la spec cierra):** writer-side del servidor MCP (`Arc<LocalStateBackend>` en el contexto del server). Mismo fundamento evidencial que S4: el split writer/proxy MCP-35 ya resuelve visibilidad multi-cliente si las tools corren writer-side (patrón `mcp_proxy_handler`, FIND-113 §7.6); el lock fs2 single-writer garantiza un solo dueño; el backend invita `Arc` explícito (`local_backend.rs:36-37`); `approval.rs:61-62` ya anticipó el patrón ("wrap in `Arc`"). Dueño explícito, no `static` global implícito.
- **Locks TTL multi-proceso → scope-proceso (pre-mortem 1 del plan, cerrado):** los locks y leases viven en la RAM del writer; solo el writer reclama (`acquire/renew/release` + `claim_stale_tasks` corren writer-side tras el proxy). Otros procesos jamás tocan el backend. El techo CAS (`:236-237`) queda como deuda documentada para un futuro multi-writer, no como bloqueo.
- **Daemon prohibido → pull-based explícito (cerrado con precedente):** ni hilo de fondo ni piggyback en cada `tools/call` (acoplaría la latencia del usuario al trabajo del pipeline: un `run_l1` con LLM real tarda segundos). El driver es la tool explícita `scheduler_run_once` que el host/agente invoca cuando quiere — igual que `TimerScanner::run_once`. Regla 8 N/A: sin threads nuevos, orden de locks intacto (`Mutex` único, release-antes-de-actuar ya documentado `:292-295`).
- **No duplicar MEM-65 (pre-mortem 2 del plan, cerrado):** el pass ejecuta el `MemoryTaskHandler` existente con su dispatch intacto (`:804-820`); `Dream` se reutiliza vía `run_dream` (`:545-565`). Cualquier `run_once` futuro que reimplemente consolidación violaría el contrato.
- **Slice mínimo evaluado honestamente (no escondido):** shippear `status`/`run_once` sin productor = mostrador vacío (`{processed:0}` siempre, prohibido por analogía FIND-110) → el ship se condiciona al Slice B (productor) en el mismo PR, igual que FIND-110-spec condicionó a su Slice B. La spec cierra porque el productor tiene forma concreta (Slice B), no porque se ignore.

## Spec (SDD — Phase 1b, tabla de decisiones)

**Veredicto Phase 1b:** NO es feature-add — cero símbolos públicos nuevos (ningún `pub fn`, tool MCP, endpoint o binding agregado; la spec DISEÑA pero no expone). Tabla resuelta POR EVIDENCIA in-repo (heredada de FIND-113 donde aplica, no re-derivada; sin ronda `question`: nada abierto que el repo no responda y el plan fija Stop honesto en vez de pregunta):

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Dueño del backend en MCP | A) writer-side `Arc` en contexto server (pro: visible vía proxy ya existente, single-writer fs2, invita el propio backend `:36-37`; contra: restart pierde RAM) / B) `static` process-local (contra: diverge entre procesos por MCP-35, dueño global implícito) / C) backend por llamada (contra: cola siempre vacía = mostrador vacío) / D) persistencia on-disk (PROHIBIDA sin ADR + fuera de appetite) / E) sin dueño → Stop + re-DEFER con diseño parcial | ✅ A por evidencia (§7.2 + FIND-113 §7.5-7.6); E era el Stop si A no se sostenía — se sostiene |
| 2 | Driver sin daemon | A) tool explícita `scheduler_run_once` pull-based (pro: precedente `TimerScanner`, cero threads, cero acople de latencia; contra: el host debe invocarla) / B) piggyback en cada `tools/call` (contra: acopla latencia LLM al usuario) / C) hilo de fondo/daemon (PROHIBIDO por contrato + Regla 8) | ✅ A — precedente MEM-16, no opinión |
| 3 | Scope de locks/leases | A) scope-proceso writer (pro: el backend lo es por diseño + techo CAS documentado; contra: multi-writer futuro exige CAS) / B) CAS multi-proceso ya (contra: fuera de appetite, nuevo store) | ✅ A con techo documentado (ver Diseño) |
| 4 | Ship `status`/`run_once` sin productor | A) shippear ya (contra: mostrador vacío, PROHIBIDO) / B) no shippear; ship condicionado a Slice B (productor) en el mismo PR | ✅ B — condición de ship explícita en Gates (espejo FIND-110-spec Decisión 3) |
| 5 | Rediseñar Dream en el scheduler | A) reimplementar consolidación en el pass / B) reusar `TaskKind::Dream` MEM-65 vía handler existente | ✅ B — prohibido por contrato (pre-mortem 2) |
| 6 | Dir de estado | A) ninguno (RAM-only; pro: cero persistencia nueva, coherente con scope-proceso) / B) dir `<db>/scheduler/` (contra: persistencia encubierta sin ADR, fuera de appetite) | ✅ A con motivo escrito (ver Diseño); B solo vía ADR futuro |

**Gate D (question-gates.md):** evaluado ANTES de escribir la spec — blast radius de ESCRITURA = 1 archivo `.md` (cero código); contrato no ambiguo (alternativa re-DEFER de primera clase en el Stop); cero símbolos públicos nuevos; feature-add sin spec N/A (no es feature-add). → **no disparado** (sin `question`, con motivo).

## Diseño dueño (AC-a)

### Dueño explícito (cierra el Stop del plan)

**Dueño: el proceso writer del servidor MCP (`vantadb-mcp`), tenencia `Arc<LocalStateBackend<SystemClock>>` en el contexto del server junto a `StorageEngine`/`Executor`.**

Fundamento (evidencia, no opinión):

1. El modelo de proceso MCP-35 ya resuelve la visibilidad multi-cliente: el proxy ejecuta writer-side — un backend poseído writer-side ES visible a todos los clientes sin diseño adicional de routing (FIND-113 §7.6, mismo fundamento que S4).
2. El backend invita el patrón (`local_backend.rs:36-37`: "wrap in `Arc` if several owners need it") y `PipelineWorker` ya prevé múltiples dueños vía `with_owner` (`:219-222`): el diseño toma esas invitaciones literales — un `Arc`, un writer, owners efímeros por pass.
3. El lock fs2 writer/proxy garantiza un solo writer: la cola tiene un solo dueño por construcción, sin CAS ni coordinación multi-proceso (el techo `:236-237` queda documentado, no bloquea).
4. El driver pull-based tiene precedente (MEM-16 `TimerScanner::run_once`): el dueño dispara passes explícitos, sin threads, sin daemon, sin auditoría Regla 8 nueva.

**Dir de estado: ninguno.** El backend vive solo en RAM del writer (`Mutex<Inner>`). Ningún dir existe ni se propone crearlo en este diseño: crearlo = persistencia nueva on-disk = decisión 6B = ADR futuro fuera de appetite. (Responde al contrato "(dir estado, …)": la respuesta diseñada es "ninguno, con motivo".)

**Quién reclama locks TTL:** el owner efímero de cada pass (`scheduler-{pid}-{seq}`, construido con `with_owner` — sin él, dos passes concurrentes en el writer colisionarían, `:216-218`). Cada pass: `claim_task(owner, lock_ttl)` → `acquire_lock(pipeline_lock:{session}, owner, ttl)` → handler → `release_lock` → settle con fencing (`complete_task` solo para el owner, `:261-270`). Un worker que muere a mitad deja su claim en `pending` hasta que el lease expira; el siguiente pass lo recoge vía `reclaim_stale` con su propio owner (`:251-267`). Todo bajo el `Mutex` único: double-reclaim imposible en-proceso.

**Lo que el dueño hace (contrato de tenencia, 4 deberes):**

1. **Construir una vez** al arrancar el writer (junto al engine): `Arc::new(LocalStateBackend::new(SystemClock))`; compartir por clon de `Arc` a cada dispatch.
2. **Prestar, no copiar:** cada `scheduler_run_once` construye su `PipelineWorker` sobre `&backend` del `Arc` con owner efímero único — todos los clientes (directos y proxeados) ven la misma cola.
3. **Anunciar la semántica efímera:** al arrancar, log `scheduler backend initialized empty (ephemeral: restart drops queue/pending/locks — tasks re-enqueue from persisted sessions)`; al apagar limpio con `pending+queue > 0`, log warn con los counts (observabilidad mínima, ver Gates G1).
4. **No inventar tareas ni decisiones:** las tareas las crea el productor (Slice B); los reintentos los decide el worker (`max_retries`/`dead-letters` existentes, sin política nueva).

### Flujo productor→queue→pass (Slice A + B)

```
[FUTURO productor: enqueue interno writer-side]              (Slice B, mismo proceso writer)
         │  tras write persistido → backend.enqueue_task(kind, session, priority)
         │  sin write → nada (la cola nace vacía y run_once = {processed:0})
         ▼
[BACKEND: Arc<LocalStateBackend> en contexto writer]         (Slice A)
         │  queue ordenada (priority, created_at) + pending con leases + locks TTL
         │  (restart vacía todo → semántica efímera documentada, nunca silenciosa)
         ▼
[PASS explícito vía scheduler_run_once]                      (Slice A, ship bloqueado hasta Slice B)
    claim → session-lock → handler existente (L1/L2/L3/Dream intactos) → settle
    + reclaim_stale acotado del mismo pass
```

**Invariantes del flujo (mecánica existente, no re-diseñada):**

- Tarea con sesión bloqueada se reencola al fondo sin perderse ni doble-procesarse (`:282-290`).
- Lock liberado ANTES de actuar sobre el outcome (`:292-295`): el reintento nunca se deadlockea.
- Fallo con `attempts < max_retries` → requeue + fin del pass (`:314-320`); agotado → dead-letter + complete (`:305-312`).
- `complete/requeue/renew` con fencing por owner (`:261-286, :324-344`): un pass nunca liquida el claim de otro.
- `Dream` quiet-prematuro = `Ok` silencioso (`:559-561`): el productor (timer/hook) re-encola; sin spam de dead-letters.

### Lifecycle restart (cerrado, no silencioso)

1. **Restart = backend vacío.** Sin excepción, sin recovery, sin migración. Nace vacío con cada writer.
2. **No es pérdida de datos de usuario:** las tareas referencian sesiones/trabajo cuya fuente persiste en la DB por el path de escritura directo (igual que S4: dreams-dato-primario-persisten vs scheduler-tarea-derivada). La pérdida se acota a "un pass pendiente no corrido", regenerable: el próximo write/timer del productor re-encola; `Dream` idle re-dispara por timer.
3. **Nunca silencioso:** (i) arranque loguea la semántica efímera; (ii) apagado limpio con cola/pending loguea warn + counts; (iii) la descripción de ambas tools documenta "ephemeral: queued work is lost on server restart; re-enqueued by future writes".
4. **Reapertura con persistencia o multi-writer:** solo vía ADR futuro (CAS en leases `:236-237` + store + migración). Hoy: prohibido por appetite (decisión 6).

## Superficie mínima (AC-b — diseño, sin código)

**Respuesta corta: 2 tools de lectura/ejecución pull-based, 0 tools de escritura pública en v1.** El productor es interno (Slice B), no una tool — exponer `scheduler_enqueue` al LLM sería API pública para mecánica interna (Gate D, diferida con motivo, igual que `capture_submit` en S4).

- **`scheduler_status`** — snapshot read-only del backend del writer. Retorna: `queue_depth` (split por prioridad, vía `queue_depth()` `:217-221`), `pending_count` (`:348-349`), `dead_letters` (count del pass-log, no del backend), `owner` (que el writer expone como `scheduler-{pid}` base), `ephemeral` (nota fija: restart pierde lo encolado). Sin params salvo los de envelope MCP. Nunca falla por cola vacía: `{queue:0,pending:0}` es estado válido, no error.
- **`scheduler_run_once`** — ejecuta UN pass acotado sobre el backend compartido y retorna el `RunStats` (`processed/failed/skipped_locked`, `:82-88`) + `reclaimed` (tareas stale recuperadas en el mismo pass, vía `reclaim_stale` con `limit` = batch). Params: `limit` (default = `batch_size` 8, clamp 1..=32) y `lease_ttl_ms` (default 60_000; clamp documentado). Construye `PipelineWorker::with_owner(scheduler-{pid}-{seq})` por invocación (seq monotónica del writer: sin colisión entre passes concurrentes) y un `MemoryTaskHandler` con runner del Slice C (ver Qué cambia). Errores del handler NUNCA son error de la tool: van a retry/dead-letter del worker (la tool retorna stats, no excepciones de tarea).
- **Lo que NO se expone en v1:** `scheduler_enqueue` pública (el productor es interno Slice B); control de timers (el `TimerScanner` lo maneja el host, no el LLM); `dead_letters` con replay (solo count; replay = ADR futuro); configuración del worker por llamada (defaults `WorkerConfig::default()`; tunable solo vía env `VANTADB_*` si un slice futuro lo exige con motivo R-5).

## Qué cambia (diseño, sin código)

**Respuesta corta: NADA en `pipeline_worker.rs`, `local_backend.rs` ni `types.rs`.** El diseño no exige cambiar ni una firma. Lo que cambia vive en TRES sitios fuera de ellos:

**Cambio 1 — Tenencia (writer context, Slice A mecánico):**

- Un campo `scheduler_backend: Arc<LocalStateBackend<SystemClock>>` en el contexto/holder que ya transporta `StorageEngine`+`Executor` al dispatch (el holder exacto lo nombra el implementador en DISCOVERY del slice; candidatos: struct de contexto del server o parámetro thread-through junto a `&storage`/`&executor` en `handle_tools_call` — decisión de cableado, no de diseño; mismo patrón que S4 Cambio 1).
- Construcción una vez al arrancar el writer; clon de `Arc` por dispatch. Estimación: <30 líneas + 2 logs (arranque/apagado).
- Alternativa descartada con motivo: `static` global (diverge writer/proxy por MCP-35; además es dueño global implícito — viola el invariante "sin dueño global"; el `Arc` explícito en contexto es el dueño explícito que el pre-mortem exigía).

**Cambio 2 — Exposición condicionada (2 tools, Slice A, ship bloqueado hasta Slice B):**

- Registrar `scheduler_status` / `scheduler_run_once` en `handle_tools_list` + dispatch delegando al worker existente (glue → core, coherente con R-8: la lógica vive en `vanta-memory`, el binding solo presta el `Arc` y traduce stats).
- Owner por invocación `scheduler-{pid}-{seq}` (seq en el writer, `with_owner` ya existe para esto).
- Schemas + descripciones: incluir la semántica efímera en cada descripción (R-5: docs en el mismo PR).
- Estimación: <60 líneas + schemas + docs `docs/api/MCP.md` (R-5 paridad mismo PR).

**Cambio 3 — Productor interno (Slice B mecánico, el que habilita el ship):**

- En el futuro path de escritura del writer que produzca trabajo pipelineable: `backend.enqueue_task(...)` tras write persistido — el `enqueue_task` core YA existe (`:191-204`); cablear ≠ diseñar.
- Sin tool pública, sin API nueva, sin Gate D. El implementador nombra el call-site en DISCOVERY del slice (candidatos: post-write writer-side o hook de captura en-proceso; si ningún call-site cabe en ≤40 líneas → re-DEFER honesto con esta spec como diseño parcial, no forzar).
- Estimación: <40 líneas en el call-site + test S6.

**Cambio 4 — Runner de los passes (Slice C mecánico, reuse FIND-112):**

- Cuando el scheduler exista, pide un runner al MISMO constructor de FIND-112 (`build_ingest_runner`-equivalente) con su propio config (ver Coherencia): el runner vive lo que el pass y se dropea con él (la key OpenAI nunca queda en memoria global — misma regla que el thread de ingesta). Default sin modelo ≡ degradado P4 honesto (skip observable, nunca hard error).
- Estimación: wiring <30 líneas (construcción + `MemoryTaskHandler`); sin trait nuevo, sin enum nuevo (reusa `ConcreteRunner` de IMPL-112).

**Suma slices futuros:** A (~90 líneas con docs/tests) + B (~40) + C (~30 wiring) — cada uno ≤100 líneas, orden A→B→C (C puede ir con B si el constructor FIND-112 ya existe), revertibles por separado (aditivos; revert = no registrar tools / no cablear productor).

## Gates y tests futuros (AC-c)

### Tests existentes (contrato fuente, no re-ejecutar para esta spec — cero código)

Mecánica 21/21 verde en `vanta-memory --test pipeline_manager` (FIND-113 § Verificación con output real): queue/priority, capture_atomic, timers, locks owner-scoped + expire por clock, reclaim 2 tests, worker retry/dead-letter, skip-locked sin pérdida, handlers L1/L2/L3, Dream 2/2.

### Tests futuros (nombrados, para los slices; espejo del estilo FIND-112 "tests 1-10")

- S1 `shared_arc_single_view` (Slice A): dos handles (`&backend` del mismo `Arc`) ven la misma tarea tras un `enqueue` — prueba scope-proceso del dueño (un writer, una vista).
- S2 `run_once_processes_enqueued_without_losing` (Slice A+B): con backend del `Arc` + productor de test que encola 3 tareas → un pass `run_once` las procesa (`processed == 3`, cola a 0) — el test captura→pass en-proceso que FIND-113 declaró imposible sin dueño (el dueño lo trae el Slice A).
- S3 `stale_claim_reclaimed_by_next_pass` (Slice A): pass que muere con claim pendiente + avance de clock más allá del lease → siguiente pass con owner distinto lo reclama y procesa (fija `claim_stale_tasks` a nivel dueño, no solo backend).
- S4 `fresh_backend_starts_empty` (Slice A): backend nuevo → `queue_depth == (0,0)` + `pending_count == 0` — fija la semántica restart como test (efímero por construcción).
- S5 `failed_pass_keeps_retry_then_dead_letters` (Slice A): handler que falla siempre → reintentos hasta `max_retries` → dead-letter (fija que la tool nunca convierte fallo de tarea en error MCP).
- S6 `producer_enqueues_after_persisted_write` (Slice B): con productor cableado, un write deja 1 tarea encolada con `session_id` correcto; sin write, cola a 0 — el test productor que cierra el mostrador vacío.
- S7 `status_reports_depth_without_consuming` (Slice A): `status` con 2 encoladas → depth 2 y las 2 siguen en cola (solo lectura, nunca consume).

### Gates de los slices (G0–G3, espejo FIND-112 / FIND-110-spec)

- **G0 (seguridad/higiene):** cero secrets (el runner C usa solo env, patrón FIND-112; grep `api_key` en TOML = fail); `cargo fmt --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` 0 warnings; diff solo-archivos-del-slice.
- **G1 (observable):** con productor (Slice B): write→`status` muestra 1 → `run_once` → `processed == 1` + cola 0; restart → `status` en ceros + log de semántica efímera en arranque; apagado con pendientes loguea warn+counts.
- **G2 (paridad R-5):** `docs/api/MCP.md` actualizado en el MISMO PR + `validate-docs-coverage` 0 gaps; descripciones con semántica efímera.
- **G3 (suites):** `cargo test -p vanta-memory -j 2` (existentes 21/21 + S1–S7) + suite MCP del dispatch verdes, sin regresión en reads/stats; **condición de ship:** las 2 tools solo shippean en el PR que incluya Slice B (productor) — ship de A sin B = mostrador vacío, prohibido. Si el slice añade threads/concurrencia nueva → dispara Regla 8 (`vanta-chaos` stress + P2-01); con el diseño pull-based actual, Regla 8 N/A con motivo.

## Qué NO cambia (AC-d, parte 1)

1. **`vanta-memory/src/services/pipeline_worker.rs`** — ni firmas ni mecánica (`run_once`, `reclaim_stale`, `run_task`, `with_owner`, dispatch `handle`, `run_dream`). El worker ya sabe ser poseído; solo le faltaba el poseedor.
2. **`vanta-memory/src/utils/local_backend.rs`** — ni queue ni pending ni locks ni timers. El `Mutex`+mapas en-RAM es suficiente por diseño a escala scheduler-humano/agente.
3. **`vanta-memory/src/core/state/types.rs`** — `TaskKind`/`TaskPayload` intactos; ningún tipo nuevo.
4. **MEM-65 intacto** — `LayerTelemetry` + dispatch `Dream→run_dream` se reutilizan tal cual; prohibido reimplementar consolidación en el pass.
5. **Dispatch MCP existente** — `handle_tools_call(&params,&executor,&storage,&cfg)` no cambia de firma salvo el thread-through del `Arc` (glue, R-8); las 87 tools ni se enteran.
6. **Sin `static` global, sin threads, sin daemon, sin piggyback** — el dueño es un `Arc` explícito en contexto writer; el driver es pull-based explícito (Regla 8 N/A por construcción).
7. **Sin persistencia de la cola/locks** — tradeoff escrito (decisión 6 + §restart): tareas derivadas re-encolables desde fuente persistida; persistir exigiría store+diseño+ADR fuera de appetite 1d. Solo se reconsidera vía ADR si el scheduler deja de ser best-effort o el writer deja de ser único.
8. **Sin tool `scheduler_enqueue` pública en v1** — `enqueue_task` core (`pub`, ya existe) lo llama el productor interno; exponerlo como tool es API pública nueva (Gate D) y se difiere con motivo (decisión 4, espejo S4 decisión 2).
9. **Runner de ingesta intacto** — FIND-112 construye runners para ingesta; el scheduler pedirá el suyo al mismo constructor (Slice C), no lo duplica ni lo modifica.

## Coherencia con FIND-112 §(c) (AC-d, parte 2 — escrita, no referenciada)

FIND-112 §(c) fija: dueño runner = proceso MCP, registry `ingest_runs()`, construcción POR LLAMADA porque el runner es stateless+barato+con-secrets. El scheduler es coherente con esa decisión sin ser idéntico a ella:

| Eje | Runner (FIND-112 §c) | Scheduler (esta spec) | Coherencia |
|-----|----------------------|----------------------|------------|
| Qué se posee | Nada (se construye y se mueve al thread) | `Arc<LocalStateBackend>` compartido entre llamadas | Opuesta con motivo: el runner no tiene estado que compartir; el backend ES estado compartido |
| Lifecycle | Por llamada (env mutable, barato, sin secrets en static) | Por proceso writer (una construcción, prestado por `Arc`) | Cada uno elige el lifecycle que su estado exige — la coherencia es el criterio, no la forma |
| Secrets | `api_key` vive solo en el stack del thread de ingesta | `api_key` (Slice C) vive solo en el stack del pass | Idéntica regla: ningún secret en estado compartido ni en `static` |
| Multi-proceso | Cada proceso su registry (`Unknown run_id` honesto) | Solo el writer posee; proxy enruta writer-side | Misma honestidad: lo no-compartido se declara, no se finge |
| Quién pide a quién | — | El scheduler (Slice C) pide un runner al constructor FIND-112 con config propia | Sin bloqueo cruzado: el runner por-llamada no necesita scheduler; el scheduler reusará el constructor sin modificarlo |
| Lo que el otro "no posee" | FIND-112: "el scheduler no lo posee" (`:240`, el runner) | Esta spec: el runner no posee el backend; el backend no posee runners | Simetría declarada: cada dueño posee solo su estado |

Esta decisión NO contradice §(c) — la confirma: runner por-llamada vs scheduler compartido son lifecycles opuestos, cada uno correcto para su estado (mismo veredicto que FIND-113 §3-Dependencias, ahora en forma de diseño ejecutable).

## 9. INVESTIGACIÓN INTERNET

No requerida — las 3 incógnitas se cierran con evidencia in-repo (backend + worker + techo CAS documentado + precedente pull-based MEM-16 + FIND-112 §c + hermana S4). Cero citas externas, cero deuda TSYS-13. (Si el ADR futuro multi-writer necesita patrones de leases distribuidos/CAS, ese será el momento de research — no este slice.)

## Invariantes de dominio (handoff — MUST)

- **`TaskKind::Dream` cableado MEM-65 intacto** (no rediseñar, reusar vía handler existente)
- **Dueño explícito writer-side** (`Arc` en contexto server); prohibido `static` global y prohibido backend process-local por cliente o por llamada
- **NO añadir tools scheduler sin productor cableado** (mostrador vacío prohibido — condición de ship G3)
- **NO persistir cola/locks/pending sin ADR** (fuera de appetite S6b)
- **NO daemon/threads/piggyback** (driver pull-based explícito; Regla 8 N/A por construcción)
- **Approve-análogo del worker intacto:** claim→lock→handle→release-antes-de-actuar→settle con fencing; `Dream` quiet = `Ok` sin dead-letter
- **Comandos de verificación:** `git diff --check` (esta spec) · a futuro: `cargo test -p vanta-memory -j 2` (21/21 + S1–S7) + clippy/fmt + `validate-docs-coverage.ps1`
- **Deuda pendiente:** ninguna nueva (saldo Regla 6: 0). La deuda S6b-lifecycle queda CERRADA en diseño; su ejecución futura son los slices A+B+C mecánicos de arriba

## Recitation (canónico)

- `activeGoal`: FIND-113-spec diseño dueño scheduler S6b (spec-first, cero código)
- `lastAction`: DISCOVERY (FIND-113 + worker/backend/locks/MEM-65/dispatch/tipos + timer_scanner pull-based + FIND-112 §c + FIND-110-spec hermana + rules core-engine + SPEC.md + codegraph 91 símbolos + diff-check) + spec escrita en `docs/tasks/FIND-113-spec.md` (dueño writer-side `Arc`, pull-based sin daemon, restart efímero documentado, superficie 2 tools, slices A+B+C, S1–S7, G0–G3, coherencia-112 escrita) + commit `docs:` solo-task-file
- `result`: OK (spec completa a+b+c+d + dueño explícito; Stop no disparado — el dueño se sostiene con evidencia)
- `nextAction`: orquestador — P2-01 sobre LA SPEC (sección Review) + `skill progreso` (Backlog/plan/avance son del lead) + NextTask IMPL-112-S1
- `contract`: verificacion `git diff --check` limpio + `git status` solo-task-file en staging · evidencia: cada claim de diseño cita `file:línea` (worker `:198-324,545-565,804-820`; backend `:36-42,228-237,241-392`; tipos `:38-70`; scanner `:1-7`) · artefactos: `docs/tasks/FIND-113-spec.md` · invariantes: ver sección (Dream intacto, dueño explícito, sin tools sin productor, sin persistencia sin ADR, sin daemon) · deuda: ninguna · queda_pendiente: P2-01 (orquestador) + IMPL-112-S1
- `nextTask`: IMPL-112-S1

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** 0 (docs-only, cero código). `ponytail:` lo más simple que cierra es NO cambiar código hoy — el diseño reutiliza `run_once`/`reclaim_stale`/`with_owner`/`enqueue_task`/dispatch-Dream existentes y añade solo tenencia + cableado futuro. Skipped: persistencia, tool enqueue pública, daemon/piggyback, CAS multi-proceso; add cuando slices A+B+C o ADR futuro.

## 10. VALIDACIÓN + CIERRE — Definition of Done (3 niveles — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Contrato a+b+c+d en este archivo + dueño explícito (Stop no disparado con motivo fundado) + determinista aplicable (diff-check limpio; fmt/clippy/nextest N/A por cero código, justificado) + coherencia-112 escrita | ✅ |
| **Commit** | Atómico (1 archivo nuevo), conventional `docs:`, staging selectivo verificado (`git status` solo-task-file), sin WIP ajeno, NO PUSH | ✅ |
| **Release** | N/A — spec docs sin artefacto shippable; `verify.ps1` completo no aplica (cero código). OCR N/A-justificado (cero código; `unsupported_ext` para `.md`, precedente FIND-113 Step 5) | N/A justificado |

## Herramientas necesarias (resumen §6)

- `codegraph_explore` (blast radius 91 símbolos) · `git diff --check` + `git status` (scope/staging) · `campaign_verify_cmd` no corrido por bug exit -1 NO alcanzado (cero código: bash directa `git diff --check`, precedente plan Riesgos globales) · OCR N/A-justificado (cero código)

**Notion (Paso 0c):** sin tool `fetch`/Notion disponible en este entorno — 4 páginas no consultables; registrado como limitación, no como evidencia (precedente FIND-110-spec/FIND-113).

## Investigation Notes

### Código (DISCOVERY — worker + backend + tipos + precedente pull-based + coherencia)

- `pipeline_worker.rs:207-222` anticipa el dueño (`worker-{pid}` + `with_owner` para no colisionar): el diseño toma esa invitación literal — owners efímeros por pass.
- `pipeline_worker.rs:292-295` fija el orden release-antes-de-actuar: el diseño pull-based concurrente no introduce deadlock nuevo (Regla 8 N/A por construcción, no por suerte).
- `local_backend.rs:36-37` anticipa el `Arc`: el diseño toma esa invitación literal — un `Arc`, un writer.
- `local_backend.rs:236-237` fija el techo CAS multi-proceso como YAGNI documentado: el diseño lo hereda como deuda declarada del ADR futuro, no como bloqueo.
- `timer_scanner.rs:1-7` fija el precedente pull-based MEM-16 (TDAM tenía intervalo de fondo; el port lo eliminó): el driver explícito no es una ocurrencia de esta spec, es el patrón establecido del módulo.
- Re-DEFER FIND-113 (6 evidencias): 0 callers MCP + nada encola + dispatch stateless + split writer/proxy + daemon prohibido + mecánica 21/21 verde. Esta spec responde punto por punto: dueño = writer-side `Arc` con routing proxy ya existente (evidencias 1+3+4), productor = Slice B interno (evidencia 2), daemon = pull-based con precedente (evidencia 5), ship condicionado a A+B (evidencia: mostrador), mecánica intacta + MEM-65 reusado (evidencia 6).
- Coherencia scheduler↔runner (FIND-112 §c): cuando el scheduler exista, pedirá un runner al mismo constructor con su propio config (Slice C); el runner por llamada no lo necesita. Sin bloqueo cruzado.

### Problema (las 3 incógnitas uphill, con evidencia no opinión)

| # | Incógnita | Respuesta + evidencia |
|---|-----------|----------------------|
| 1 | ¿Quién posee el backend (dir estado, reclamante locks)? | Writer MCP (`Arc` en contexto server); dir estado = ninguno (RAM-only con motivo decisión 6); reclama el owner efímero `scheduler-{pid}-{seq}` por pass + `reclaim_stale` del siguiente. Evidencia: backend invita `Arc` (`:36-37`); worker invita owners (`:219-222`); proxy writer-side ya existe (FIND-113 §7.6); `static`/por-llamada/persistencia rechazados en tabla Phase 1b. |
| 2 | ¿Locks TTL multi-proceso? | Scope-proceso writer. Evidencia: `Mutex` único + sección crítica única (`:235-237`); sin CAS entre procesos (techo documentado); single-writer fs2 → un solo reclamante por construcción; multi-writer = ADR futuro. |
| 3 | ¿Coherente con FIND-112 §(c) sin ser idéntico? | Sí, por tabla escrita (ver Coherencia): runner por-llamada (stateless/barato/secrets) vs backend compartido (estado con tareas); misma regla de secrets; Slice C reusará el constructor. Evidencia: `FIND-112.md:214-240` leído sin editar. |

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — dueño / locks-scope / coherencia-112 RESPONDIDAS con evidencia |
| Pendientes de ejecución (downhill) | 0 — slices A+B+C son trabajo futuro mecánico, no pendiente de esta spec |
| % completado | 100% (discovery + diseño + task file + commit) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — N/A con justificación: cero código, cero trust boundaries tocados. Nota para slices: `scheduler_run_once` aceptará `limit`/`lease_ttl_ms` externos → clampear en frontera (patrón `IngestConfig` clamp 1..=20); un futuro productor que acepte `session_id`s externos valida como `dreams.rs` (`validate_identifier`); restart-loss queda como integridad documentada, no silenciosa; `api_key` del Slice C nunca en TOML (precedente FIND-112, G0 lo caza).
- [x] **PERFORMANCE** — N/A con justificación: backend `Mutex<Inner>` + scans O(n) (`pending`, `ponytail` documentado `:293`, tiny por diseño); a escala scheduler-agente es irrelevante; si un ADR futuro persiste el backend o añade driver concurrente, exigir baseline before/after (Regla 9, `canonical_p99`) + auditoría Regla 8 (deadlock/data-race) + stress `vanta-chaos` 10k w/s.

## Steps

### Step 1: DISCOVERY worker + backend + tipos + precedente pull-based + coherencia FIND-112 §c

- **Archivos:** `docs/tasks/FIND-113.md`, `pipeline_worker.rs` (§§1-130, 198-335, 540-565, 804-820), `local_backend.rs` (§§1-70, 228-392), `types.rs` (`:38-70`), `timer_scanner.rs`, `FIND-112.md` §(c), `FIND-110-spec.md`, rules `core-engine.md`, `definition-of-done.md`, `SPEC.md` §cierre-mvp
- **Acción:** leer completos + `codegraph_explore` blast radius + `campaign_discover_skills_v2` DEFINE + `campaign_detect_task_type` + `git diff --check` baseline
- **Verify:** citas worker/backend/tipos/scanner + SDP registrado + type `docs`
- **Estado:** ✅ COMPLETED

### Step 2: Resolver las 3 incógnitas uphill con evidencia

- **Archivos:** FIND-113 §Decisión (6 evidencias) + backend/techo-CAS + scanner pull-based + FIND-112 §(c) (sin re-leer `vantadb-mcp/src/` más allá del scope: evidencia heredada)
- **Acción:** dueño writer-side `Arc` vs `static` vs por-llamada vs persistencia / locks scope-proceso vs CAS / driver pull-based vs piggyback vs daemon; doubt-driven adversarial (¿y si el productor no cabe en el writer? → Stop documentado para slices, re-DEFER con esta spec como parcial; ¿y si la cola crece? → ADR persistencia)
- **Verify:** tabla Phase 1b decisiones 1–6 con opciones+tradeoffs resueltos; Stop evaluado y NO disparado con motivo (dueño se sostiene)
- **Estado:** ✅ COMPLETED

### Step 3: Escribir la spec (a+b+c+d + dueño) en este archivo

- **Archivos:** `docs/tasks/FIND-113-spec.md` (nuevo, único permitido)
- **Acción:** poblar TODO por tarea (10 puntos obligatorios): dueño + dir-estado + reclamante-locks + superficie 2 tools + slices A+B+C + tests S1–S7 + gates G0–G3 + no-cambia + coherencia-112 escrita + DoD + Review P2-01-spec
- **Verify:** secciones Metadata→Verificación completas, dueño explícito presente, cero código
- **Estado:** ✅ COMPLETED

### Step 4: Verify + commit selectivo + RESULTADO

- **Archivos:** `docs/tasks/FIND-113-spec.md` (único en staging)
- **Acción:** `git diff --check` + `git status` (solo-task-file) + `git add docs/tasks/FIND-113-spec.md` + `git commit -m "docs: FIND-113-spec — ..."` (NO PUSH) + `campaign_update_task_state` completed + `campaign_memory_write` + bloque RESULTADO §7
- **Verify:** commit existe (`ae491fdd`, 1 file, 449 insertions); WIP ajeno intacto; OCR N/A-justificado
- **Estado:** ✅ COMPLETED

## Dependencias (resumen §3)

- FIND-112 ✅ (spec cerrada — §(c) como referencia de coherencia sin editarla; esta spec la consume, no la reedita)
- FIND-98 ✅ + FIND-110-spec ✅ (Wave0 hermanas; archivos disjuntos)
- Nota tipo: auto-detect dijo `docs`/`Documentation`; el contenido es spec de diseño lifecycle Rust-core/MCP (skills SDD+DDD+codebase-memory aplicadas por contrato del plan, no por el label). Sin conflicto: el entregable es docs-only.
- NextTask: IMPL-112-S1 (orquestador; Wave1; consume el veredicto "spec S6b cerrada en diseño")
- Stop del plan evaluado: dueño defendible SÍ existe en diseño → la spec cierra; el Stop queda documentado para la EJECUCIÓN futura (si Slice B revela que el productor no puede vivir en el writer → re-DEFER honesto con esta spec como diseño parcial, no forzar)

## Review (GATE — agente distinto, P2-01-spec)

> Lo ejecuta el ORQUESTADOR (no el implementador — instrucción de tarea). P2-01-spec: review SOBRE LA SPEC, no sobre código.

- **Revisor:** pendiente (orquestador asigna vanta-review)
- **Enfoque:** ¿el dueño writer-side `Arc` es defendible o esconde un `static` con otro nombre? ¿restart-efímero es sound (tareas re-encolables desde fuente persistida) o minimiza pérdida real (buffers también en RAM)? ¿la condición de ship (A+B mismo PR) bloquea de verdad el mostrador vacío? ¿el rechazo del piggyback por latencia es correcto o sobredimensionado (un pass `NoLlm` es barato)? ¿algún cambio listado exige en realidad tocar `pipeline_worker.rs`/`local_backend.rs` (rompería el claim "NADA")? ¿la coherencia-112 escrita es fiel a §(c) o la tuerce?
- **Alternativas evaluadas:** ship-2-tools sin productor (mostrador vacío, prohibido) · `scheduler_enqueue` pública (API nueva + Gate D, diferida con motivo) · `static` process-local (diverge MCP-35, prohibido) · backend por llamada (cola siempre vacía, prohibido) · persistencia ya (ADR+store, fuera de appetite) · daemon/hilo/piggyback (prohibido o acopla latencia) · rediseño Dream (prohibido MEM-65) · re-DEFER con diseño parcial (era el Stop; no hizo falta)
- **Cómo se probó:** DISCOVERY con citas `file:línea` de primera mano + codegraph blast radius (91 símbolos) + `git diff --check` limpio; mecánica heredada 21/21 de FIND-113 (no re-ejecutada: cero código que la afecte)
- **OCR:** N/A-justificado — `.md` excluido (`unsupported_ext`, precedente FIND-113 Step 5); cero Critical/High propios, no bloquea
- **Checklist anti-hábitos tóxicos:** sin salidas inventadas (sin test output propio porque cero código; mecánica citada de FIND-113 con output real pegado allá); sin clarificación saltada (3 uphill con evidencia); sin done sin AC (a+b+c+d mapeados 1:1); sin fallos ignorados; sin búsqueda única (Read + codegraph + grep-FIND-113 + rules + SPEC + hermana); supuestos citados como evidencia con `file:línea`; cada step conectado al contrato; N/A dinero/seguridad-ejecución (docs-only); Stop evaluado explícitamente
- **Veredicto:** pendiente revisor distinto

## Notas

- Espejo FIND-110-spec deliberado: slices ≤100L, tests nombrados S1–S7, gates G0–G3, condición de ship explícita, restart documentado — el futuro implementador no diseña, cablea.
- `ponytail:` lo más simple que cierra es NO cambiar código hoy — el diseño reutiliza worker/backend/tipos/dispatch-Dream existentes. Skipped: persistencia, tool enqueue, daemon/piggyback, CAS; add cuando slices A+B+C o ADR futuro.
- Coherencia S4↔S6b (el par cierra en diseño): ambos dueños writer-scoped con colas RAM efímeras; S4 sin locks (Vec simple, humano) vs S6b con locks TTL + leases + reclaim (máquina); coherentes sin ser idénticos — cada estado elige su lifecycle.
- NOTICED BUT NOT TOUCHING: WIP ajeno en `git status` (`.opencode`, `SPEC.md`, `completions/*`, `docs/Backlog.md` modificados; `reparacion.bat` y plan file untracked) — fuera de scope, no se tocan ni se stagean. `docs/Backlog.md` fila FIND-113 y plan recitation los actualiza el orquestador/lead.
- DoD Release N/A justificado: sin artefacto shippable no hay qué certificar con `verify.ps1` completo; `campaign_verify_cmd` no corrido por bug exit -1 no alcanzado (bash directa `git diff --check`, precedente plan Riesgos).
- Doubt-driven (degradado, anunciado): sin `task`/sub-agente reviewer disponible para P2-01 en este contexto (lo hace el orquestador); el ciclo adversarial se ejecutó como auto-revisión contra las 6 decisiones + Stop. Cross-model skipped: contexto no-interactivo.
- `skill progreso`: la ejecuta el orquestador al cerrar (esta ejecución no muta Backlog/plan/avance — prohibido por contrato).

## Verificación

- `git diff --check` — ✅ limpio (ver Step 4)
- `git status --short` — ✅ solo `docs/tasks/FIND-113-spec.md` en staging al commitear; WIP ajeno intacto
- `cargo check -p vantadb` — N/A (cero código tocado)
- `cargo test -p vanta-memory -j 2 --test pipeline_manager` — N/A re-ejecución (mecánica heredada 21/21 de FIND-113 con output real pegado allá; cero código que la afecte)
- `campaign_verify_cmd` — no corrido (cero código; bug exit -1 no alcanzado; bash directa en su lugar)
- OCR delegation — N/A-justificado (cero código; `.md` `unsupported_ext`)
- **Scope discipline:** 1 archivo nuevo (`docs/tasks/FIND-113-spec.md`), 0 ediciones. WIP ajeno intacto.
- **Context health:** output siguió convenciones (citas `file:línea` de primera mano, comandos del stack VantaDB con `-j 2`, sin APIs inventadas — `scheduler_*` citados como propuestos, nunca como existentes)
