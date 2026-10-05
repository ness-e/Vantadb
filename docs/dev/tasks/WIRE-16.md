---
title: "TASK WIRE-16: Wiring del wrapper vantadb-server (queue + bridge + loop)"
kind: task
description: "T3 WIRE-14→15→16 (ADR-0054): vantadb-server host del scheduler — dep vanta-memory (http-server), bridge HttpCaptureBridge vía ServerHooks, loop spawn_memory_scheduler con factory por pass (FIND-112; sin runner → skip P4), env VANTADB_SCHEDULER_INTERVAL_MS (0 = off; default 60s), shutdown graceful; extensión aditiva del seam (on_storage_ready): el server es dueño del único open"
---

# TASK WIRE-16: Wiring del wrapper vantadb-server (queue + bridge + loop)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 47, F3 — Distribución; bloque F0-expandido L1346-1373)
- **Fuente:** plan Task 47 (L1346-1373) + ADR-0054 T3 (`docs/dev/architecture/adr/ADR-0054-scheduler-host-vantadb-server.md:179`) + handoffs WIRE-14 (`docs/dev/tasks/WIRE-14.md:185`) y WIRE-15 (`docs/dev/tasks/WIRE-15.md:191-194`)
- **Esfuerzo:** 🟠 3-5d | **Appetite:** max 1sem | **Stop (plan L1357):** 5d → entregar wiring HTTP + verificaciones (1) y (3); FIND de restart/sin-runner si faltan
- **Prioridad:** 🟠
- **Tipo:** Rust (`vantadb-server` + extensión aditiva mínima en `src/server/{state,bootstrap}.rs` + e2e)
- **Turns estimados:** 10-14 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `47` en el campaign server)
- **Incógnitas (uphill):** 0 — la única real ("cómo obtiene el host la DB si el server es dueño del único open del proceso") quedó resuelta en DISCOVERY por evidencia (single-writer lock + sharing de `Embedded`; §Spec #1). El plan no la listó, pero sin resolverla el wiring productivo es imposible (segundo open = `DatabaseBusy`).
- **Pendientes (downhill):** 4 steps (1-4)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `47`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Símbolos nuevos (aditivos — sin callers preexistentes):** `vantadb-server`: `scheduler::{SchedulerConfig, scheduler_interval_ms_from_env, ingest_runner_cfg, ingest_runner_factory, wire_memory}`; core: campo `ServerHooks::on_storage_ready` (consumidores: `vantadb-server/src/main.rs` + tests WIRE-16). **Callers de lo existente que NO cambia:** `run(config)` (2 callers + 3 re-exports), `run_with_hooks` (firma intacta), `ServerState` (18 literales intactos — sin campos nuevos), `HttpCaptureBridge`/`run_bridge_pass` (tests conversation_hook), `spawn_memory_scheduler` (tests WIRE-15). |
| Callees | `vanta_memory::services::{conversation_hook::HttpCaptureBridge, scheduler::{spawn_memory_scheduler, MemoryScheduler}, ingest::runner_config::{IngestRunnerCfg, build_ingest_runner, ConcreteRunner}}`, `vanta_memory::utils::{LocalStateBackend, SystemClock}`, `vantadb::cli_server::{ServerHooks, BackgroundService, ConversationTrigger, run_with_hooks}` (WIRE-14), `vantadb::sdk::Embedded`, `vantadb::storage::StorageEngine`. **Dependencia directa nueva:** `vanta-memory` con `http-server` + `llm-driver` (hoy transitiva vía `vantadb-mcp/Cargo.toml:27` con `llm-driver`; `http-server` se activa por primera vez en el grafo default-members → delta de build medido, ADR-0031). |
| Implicaciones | **API pública aditiva** en core (1 campo con `Default` inerte; firma de `run_with_hooks` y comportamiento de `run` idénticos) y en `vantadb-server` (módulo `scheduler`). Sin breaking, sin wire/persistencia, sin migración. Regla 8: el scheduler corre en el proceso writer (HTTP mode); un solo loop por proceso (ADR-0054); `spawn_blocking` para el pass síncrono (lock único del backend, sin guard a través de `.await`). Tests existentes re-corridos; ningún test toca los símbolos nuevos salvo los de esta tarea. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `fc4e994c`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `src/server/state.rs` (:90-179 — `ConversationTrigger` :97, `BackgroundService` :121, `ServerHooks` :135, `ServerState` :146+).
  - `src/server/bootstrap.rs` (:130-419 — `serve_http_or_tls` :145, `run` :287, `run_with_hooks` :298-405 (open :315, state :341-352, ensure :360-367, sweeper :375-388, join :399-402), `shutdown_background_services` :408; tests :436+).
  - `src/server/mod.rs` (:1-59 — re-exports `state`/`bootstrap`; `cli_server` = `pub use crate::server::*`).
  - `vantadb-server/src/main.rs` (:1-143 completo — dispatch `--mcp` vs `run` :69), `lib.rs` (:1-7), `server.rs` (:1-4), `Cargo.toml` (:1-79 completo — deps/features; `vanta-memory` ausente).
  - `vanta-memory/src/services/scheduler.rs` (:1-262 completo — `run_pass` :80, `MemoryScheduler` :138, `impl BackgroundService` :158, `Drop` :171, `spawn_memory_scheduler` :198-262 — factory `Fn() -> Option<R>` por pass).
  - `vanta-memory/src/services/conversation_hook.rs` (:1-107 completo — `HttpCaptureBridge` :36, `trigger` :47-84, doc de wiring :9-20).
  - `vanta-memory/src/ingest/runner_config.rs` (:1-285 — `IngestRunnerCfg` :65, `apply_env` :170, `from_env_toml` :226, `build_runner` :259; `ConcreteRunner::None` = degradado), `vanta-memory/Cargo.toml` (:37-79 — `http-server = ["vantadb/server", "dep:tokio"]` :79), `vanta-memory/src/services/mod.rs` (:1-16 — `conversation_hook` gated `http-server`).
  - `vantadb-mcp/src/wiki.rs` (:265-294, :373-384 — `ingest_toml_path`: `VANTADB_INGEST_CONFIG` > `<data_dir>/vanta-ingest.toml`), `vantadb-mcp/Cargo.toml` (:27 — dep transitiva `vanta-memory` con `llm-driver`), `vantadb-mcp/src/server.rs` (:180-259 — precedente de host que abre storage + arma state/router él mismo).
  - `src/storage/engine/init.rs` (:152-297 — `init_storage`: lock fs2 exclusivo `try_lock_exclusive` :215, `DatabaseBusy` :251, `data_dir = base_path.join("data")` :285), `src/sdk/builder.rs` (:14-59 — `Embedded` Clone :14, `audit: Option<Arc<AuditLogger>>` :18, `from_engine` :51, `open_with_config` :109).
  - `tests/storage/multi_process_lock.rs` (:14-67 — **mismo proceso, segundo writer → `DatabaseBusy`**, evidencia dura del §Spec #1), `vantadb-server/tests/e2e.rs` (:1-58 helpers, :206-266 restart, :336-371 conversation), `vantadb-server/tests/helpers/mod.rs` (:1-49), `vanta-memory/tests/conversation_hook.rs` (:1-180 — ScriptedRunner + L0/L1 asserts), `vanta-memory/tests/scheduler_loop.rs` (:1-120 — `wait_until` bounded), `src/server/telemetry.rs` (:15-80 — `init_telemetry` usa `.init()` → **un solo `run_with_hooks` por proceso de test**), `src/config.rs` (:819-821, :1313-1318 — `VANTADB_TTL_SWEEP_INTERVAL_MS` default 60_000, `0` off).
  - Docs/specs: ADR-0054 (completo — T3 :179, Forma :84-96, restricciones :45-59), plan Tasks 45/46/47 (L1289-1373), `docs/dev/tasks/WIRE-14.md` (completo — handoff :185), `docs/dev/tasks/WIRE-15.md` (completo — handoff :191-196), `docs/dev/tasks/FIND-113-spec.md` (:55-174 — semántica pull-based/restart), `docs/api/VANTA_MEMORY.md` (:243-305 — wiring status), `.opencode/rules/{server-mcp,concurrency-async,api-contract,core-engine}.md`, `.opencode/references/{clean-code-clean-architecture.md (Apéndice V),definition-of-done.md}`.
- **Archivos referenciados hacia dentro (imports/deps):** `crate::server::state::{BackgroundService, ServerHooks, ServerState}` (bootstrap.rs:12), `crate::sdk::Embedded` (bootstrap.rs:343), `crate::storage::StorageEngine` (bootstrap.rs:14), `vantadb::cli_server::*` (main.rs:56-69, `server.rs:1-4`), `vanta_memory::*` (futuro `vantadb-server/src/scheduler.rs`), `vantadb::sdk::Embedded` + `vantadb::cli_server::ConversationTrigger` (conversation_hook.rs:30,47).
- **Referencias entrantes (grep HEAD):** `ServerHooks` = 7 hits (bootstrap tests, mod/routing re-exports; literales solo `::default()` → **campo nuevo aditivo no rompe**). `run_with_hooks` = re-exports `src/server/{mod,routing}.rs` + tests facades; `vantadb-server/src/server.rs` aún NO lo re-exporta (lo hace esta tarea). `on_storage_ready` = **0 hits** (campo nuevo). `vantadb_server::scheduler` = **0 hits** (módulo nuevo). `VANTADB_SCHEDULER_` = **0 hits** (env nueva). `vanta-memory` en `vantadb-server/Cargo.toml` = 0 (dep directa nueva; transitiva ya presente). `init_telemetry` = llamada única por proceso (`.init()` panics en doble init — restricción de test documentada).
- **Veredicto impacto:** **MEDIO-BAJO (aditivo con 1 extensión core mínima)** — 1 campo público con default inerte + reordenamiento interno sin cambio observable (ensure antes de state; mismo handle/config/guard) en bootstrap; módulo nuevo + dep directa + re-export + main wiring en `vantadb-server`; 1 archivo e2e nuevo. 0 firmas cambiadas, 0 campos de `ServerState`, 0 wire, 0 migración. Pre-mortems mitigados: (1) Fast Gate → delta de build medido (vanta-memory ya en el grafo; `http-server` solo compila 2 archivos chicos) + sin feature-gate nuevo por YAGNI si el delta es ~0 [verificado en Step 2]; (2) doble scheduling MCP+HTTP → v1 cablea SOLO HTTP (ADR T3; MCP diferido) + writer = quien abre la DB (HTTP mode = writer o falla); (3) default del intervalo → 60_000 ms espejo `VANTADB_TTL_SWEEP_INTERVAL_MS` (precedente exacto del crate), `0` = off.

## Contrato

"Wiring completo con las 4 verificaciones e2e verdes (plan L1355): (1) `POST /api/v2/conversations` → L0 → pass → `l1/<thread_id>`; (2) restart (cola efímera re-encolable desde L0 persistido); (3) disabled (`VANTADB_SCHEDULER_*` = off, sin loop); (4) sin runner (degrada P4, no bloquea); env config `VANTADB_SCHEDULER_*` documentada (secrets solo env, R-5); shutdown graceful (join). Composición: (a) **extensión aditiva mínima del seam WIRE-14** — `ServerHooks::on_storage_ready: Option<Box<dyn FnOnce(&mut ServerHooks, Embedded) + Send>>` disparado por `run_with_hooks` una vez abierto el storage (y con índices asegurados), antes de construir `ServerState`: el host recibe un clon del `Embedded` del server (single-writer: un segundo open del mismo path es `DatabaseBusy`; un segundo handle partiría audit/purge/supersede locks); `run(config)` y `run_with_hooks(config, hooks)` conservan firma y comportamiento (defaults inertes); (b) **`vantadb-server`**: dep directa `vanta-memory` (`http-server` + `llm-driver`), módulo `scheduler` (`scheduler_interval_ms_from_env` — `VANTADB_SCHEDULER_INTERVAL_MS`, default 60_000, `0` = off; `ingest_runner_cfg` — `VANTADB_INGEST_CONFIG` > `<storage_path>/data/vanta-ingest.toml` + env FIND-112; `ingest_runner_factory` — sin engine real → `None` = skip P4; `wire_memory` — bridge SIEMPRE + loop si intervalo > 0, retorna la cola para observabilidad), `main.rs` cablea vía `on_storage_ready` (solo HTTP mode, solo `!read_only`), `server.rs` re-exporta el seam WIRE-14 + el módulo; (c) **tests**: core (campo default inerte + setteable) + e2e `vantadb-server/tests/scheduler_e2e.rs` (4 verificaciones; la (1) por `run_with_hooks` real — única llamada por proceso por `.init()` de telemetría; (2)(3)(4) por assembly manual `wire_memory` + `app()`); (d) verify: `cargo nextest run --profile audit -p vantadb-server --build-jobs 2` + `cargo nextest run --profile audit -p vantadb --features server --build-jobs 2` + `cargo check -p vantadb` (sin server) + `cargo fmt --check` + `cargo clippy -p vantadb-server --all-targets -- -D warnings` + `cargo clippy -p vantadb --features server --all-targets -- -D warnings`."

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): disparado y resuelto por evidencia con transparencia total.** El contrato del plan sanciona el wiring (Task 47, Gate Result ✅ DO) pero NO resuelve cómo obtiene el host la DB — y el plan de solución toca `src/server/{state,bootstrap}.rs` con 1 campo público nuevo. Evidencia dura: (a) el server abre storage dentro de `run_with_hooks` (bootstrap.rs:315) y un segundo open mismo-proceso falla con `DatabaseBusy` (`tests/storage/multi_process_lock.rs:14-52`, mismo proceso); (b) un segundo handle `Embedded::from_engine` parte audit log (`Option<Arc<AuditLogger>>` por handle, builder.rs:18) y los locks purge/supersede (DUR-03/REVIEW-13); (c) `serve_http_or_tls` es privado (bootstrap.rs:145) → duplicar bootstrap en el host re-implementaría TLS/flush/graceful (~90 líneas) y viola "reutiliza todo lo existente" (ADR :152). La extensión elegida es la completitud natural del seam WIRE-14 para su consumidor designado (worker sin `question` — question-gates §Routing; precedente exacto WIRE-14/15 "Gate D pre-respondido/resuelto-por-evidencia"; el orquestador puede ajustar en review).

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Acceso del host a la DB | A) **campo `ServerHooks::on_storage_ready` (callback FnOnce(&mut ServerHooks, Embedded) disparado post-open)** (pro: un solo open/engine/audit/locks — single-writer intacto; host completa trigger+servicios; additivo con default inerte; contra: 1 campo nuevo en core) / B) host abre la DB + `run_with_hooks` la re-abre (contra: **`DatabaseBusy`** — evidencia tests/storage/multi_process_lock.rs) / C) host abre la DB y sirve él mismo (contra: `serve_http_or_tls` privado → duplica TLS/flush/shutdown; viola ADR "reutiliza") / D) `hooks.storage: Option<Arc<StorageEngine>>` o `run_with_storage` (contra: segundo handle `Embedded` → parte audit + purge/supersede locks, hazards DUR-03/REVIEW-13) | ✅ **A** — decidido-por-evidencia: single-writer (fs2 lock, init.rs:215) + sharing correcto exige el MISMO handle (builder.rs:14-18,51-59) + precedente de forma: el host "construye ServerHooks por campos públicos" (handoff WIRE-14:185) |
| 2 | Punto exacto del disparo | A) **post-open + post-ensure_indexes, pre-state-build** (pro: el scheduler no compite con `ensure_indexes_current`; el trigger debe estar en hooks antes del literal `ServerState`; contra: reordena ensure antes de state — sin cambio observable: mismo handle/config/guard) / B) pre-ensure (contra: primer tick del loop puede pisar el reconcile) / C) post-state (contra: el trigger no entraría al state) | ✅ **A** — decidido-por-evidencia: orden de bootstrap.rs:341-367 + primera pasada inmediata del loop (scheduler.rs:188) |
| 3 | Firma del callback | A) **`FnOnce(&mut ServerHooks, Embedded)`** (pro: el host completa trigger + servicios in-place, sin tipos nuevos; contra: `&mut` en callback) / B) retorna `(Option<Trigger>, Vec<Services>)` (contra: tipos nuevos + merge manual) / C) `FnOnce(Embedded) -> ServerHooks` (contra: semántica de reemplazo ambigua) | ✅ **A** — decidido-por-evidencia: mínimo tipo nuevo (cero), ergonomía del host (literal `hooks.on_storage_ready = Some(Box::new(...))`) |
| 4 | Ubicación del wiring del host | A) **módulo nuevo `vantadb-server/src/scheduler.rs`** (pro: lib testeable + main delgado + espejo `server.rs`; contra: —) / B) todo inline en `main.rs` (contra: bin no testeable por lib tests) | ✅ **A** — decidido-por-evidencia: plan lista `lib.rs`/`server.rs` (re-exports) y e2e; testabilidad TDD (factory fake) |
| 5 | Runner por pass | A) **factory `Fn() -> Option<R>` (WIRE-15) cableada con `ingest_runner_factory(IngestRunnerCfg)`** (pro: reusa FIND-112 `build_ingest_runner`; fake inyectable en e2e; sin engine real → `None` = skip P4 sin quemar tareas; contra: —) / B) `Some(ConcreteRunner::None)` siempre (contra: cada L1 falla NotConfigured → retry ×3 → dead-letter = quema tareas; rechazado por WIRE-15 §Spec #3) | ✅ **A** — decidido-por-evidencia: WIRE-15 contrato ("None → skip observable") + FIND-112 + política P4 |
| 6 | Config del runner | A) **superficie FIND-112 reusada: `VANTADB_INGEST_*`/`VANTADB_LLM_*`/`VANTADB_OPENAI_*` + `VANTADB_INGEST_CONFIG`/`<storage>/data/vanta-ingest.toml`** (pro: cero config nueva; mismo engine que wiki ingest; secrets solo env R-5; contra: —) / B) `VANTADB_SCHEDULER_*` propias para el runner (contra: duplica la superficie FIND-112) | ✅ **A** — decidido-por-evidencia: `IngestRunnerCfg::from_env_toml` + `ingest_toml_path` (vantadb-mcp/src/wiki.rs:373-384) |
| 7 | Knob del scheduler | A) **`VANTADB_SCHEDULER_INTERVAL_MS` (default 60_000; `0` = off)** (pro: espejo exacto `VANTADB_TTL_SWEEP_INTERVAL_MS` (config.rs:819-821,1313-1318); un solo knob; contra: —) / B) enabled-flag separada (contra: 2 knobs para 1 decisión) | ✅ **A** — decidido-por-evidencia: ADR "intervalo (configurable, `0` = off)" + pre-mortem 3 (default seguro) |
| 8 | Semántica disabled | A) **bridge SIEMPRE cableado (L0+enqueue, LLM-free); loop NO spawneado** (pro: "nothing lost" P4 — la captura nunca depende del loop; contra: cola acumula sin procesar) / B) todo off (contra: pierde L0 = viola P4) | ✅ **A** — decidido-por-evidencia: conversation_hook doc :18-20 ("messages stay safely captured in l0/") + contrato (3) |
| 9 | read_only | A) **no cablear nada si `config.read_only`** (pro: espejo guard del sweeper bootstrap.rs:375; sin errores de escritura por POST; contra: —) / B) cablear (contra: cada trigger falla best-effort en log) | ✅ **A** — decidido-por-evidencia: bootstrap.rs:360,375 + P4 (no ruido) |
| 10 | Doble scheduling (MCP+HTTP) | A) **v1 solo HTTP (ADR T3); MCP diferido** (pro: decisión del ADR "V1 cablea el modo HTTP"; writer = quien abre la DB; MCP mode no usa `run_with_hooks` → sin callback → sin scheduler; contra: —) / B) ambos (contra: MCP es session-scoped — rechazado como hogar v1 por ADR) | ✅ **A** — decidido-por-evidencia: ADR :63-66 + MCP-35 (segundo proceso = proxy, no writer) |
| 11 | Tests e2e | A) **4 tests; el (1) por `run_with_hooks` real (única llamada por proceso — `.init()` telemetry), (2)(3)(4) por `wire_memory` directo + `app()`** (pro: cubre el path productivo completo + aislamiento por verificación; contra: (2)(3)(4) no re-testean bootstrap — ya cubierto por (1)) / B) todo por `run_with_hooks` (contra: `.init()` panics en la 2ª llamada mismo proceso → tests flaky/rojos bajo `cargo test`) / C) todo manual (contra: el callback de producción no se testea) | ✅ **A** — decidido-por-evidencia: telemetry.rs:55-76 (`.init()`) + patrón e2e existente (e2e.rs:39-58) |
| 12 | Restart (verificación 2) | A) **L0/L1 persisten (reopen tras abort+drop); cola efímera documentada; nueva captura fluye con queue nueva** (pro: semántica aceptada FIND-113 ("restart = cola vacía"); testable in-process con retry de reopen; contra: no hay re-enqueue automático desde L0 — no existe mecanismo, no se inventa) / B) inventar re-enqueue desde L0 (contra: scope creep sin spec) | ✅ **A** — decidido-por-evidencia: FIND-113-spec.md:218 ("restart = backend vacío (semántica aceptada)") + L0 persistido = dato nunca perdido |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **`run(config)` intacto:** firma y comportamiento idénticos; `run_with_hooks(config, hooks)` firma y semántica intactas (solo se agrega el disparo del callback cuando el host lo setea; default `None` = inerte).
  2. **`ServerState` sin cambios:** 18 literales intactos; `conversation_trigger` ya existía.
  3. **Single-writer:** un solo `StorageEngine` por proceso; el host usa el handle del server (jamás abre uno segundo); un solo loop por DB/proceso.
  4. **P4 (nunca bloquea, nunca pierde):** bridge best-effort (errores tragados por la ruta, MEM-55); sin runner → skip observable y tareas quedan encoladas; la captura L0 no depende del loop.
  5. **Parada graceful:** `MemoryScheduler` se joinea por `BackgroundService` post-run-loop (orden: servicios del host → sweeper); `Drop` best-effort cubre el early-exit (handoff WIRE-14 Low-1).
  6. **Secrets solo env (R-5):** `VANTADB_OPENAI_API_KEY` solo por env; el TOML de ingest nunca lleva key (runner_config.rs:307-308); warn+default nunca panic en config.
  7. **Sin `unwrap`/`expect`/`unsafe` en código nuevo de producción**; sin deps nuevas fuera de `vanta-memory` (ya en el grafo); `src/wal.rs`, `src/vector/`, `src/storage/` **no se tocan**.
  8. **WIP ajeno:** `opencode.jsonc` + master plan modificados en el árbol por otros — NO se stagean; PROHIBIDO tocar `docs/pipeline-state.json`; commit con **pathspec**.
  9. **Un `run_with_hooks` por proceso de test** (`.init()` de telemetría) — restricción documentada del diseño e2e.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb-server --build-jobs 2` · `cargo nextest run --profile audit -p vantadb --features server --build-jobs 2` · `cargo check -p vantadb` (sin server) · `cargo fmt --check` · `cargo clippy -p vantadb-server --all-targets -- -D warnings` · `cargo clippy -p vantadb --features server --all-targets -- -D warnings`.
- **Deuda pendiente:** ninguna del wiring. (Docs = WIRE-17; MCP como segundo host = diferido por ADR; re-enqueue automático desde L0 = no existe mecanismo, semántica FIND-113 aceptada.)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — sin `unsafe`, sin clones en hot path (callback corre 1 vez al arranque; `wire_memory` clona Arc/handles O(1)), sin abstracciones especulativas (1 campo core + 1 módulo host + 1 dep directa que ya estaba en el grafo). El cambio **elimina** la deuda del residual MEM-55 ("el puente existe y no está cableado") y agrega los e2e del wiring. `NOTICED BUT NOT TOUCHING`: `run_bridge_pass` no converge a `run_pass` (WIRE-15 lo declaró); el modo MCP queda sin scheduler (decisión ADR v1); `docs/api/**` → WIRE-17.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: extensión aditiva del seam (`on_storage_ready` inerte por default) + wiring `vantadb-server` (bridge + loop + env + shutdown) + 4 e2e verdes + core tests + suites scoped (vantadb-server + vantadb --features server) + `cargo check -p vantadb` sin server + fmt/clippy verdes |
| **Commit** | Commit atómico conventional `feat(wire):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | Changelog release-plz (feature → minor); verify full scoped documentado (workspace completo no se corre por presupuesto) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius: `run_with_hooks`/`ServerHooks`/`ServerState`/`HttpCaptureBridge`/`spawn_memory_scheduler`) + `codebase-memory-mcp_check_index_coverage` (paths clave, `no_recorded_issue`) + grep puntual
- `cargo nextest` scoped por crate (RED/GREEN TDD) + `campaign_verify_cmd` (verify mecánico)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Gates docs (`node scripts/docs/check-links.mjs && check-docs.mjs && gen-index.mjs --check`) — task file editado

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `test-driven-development` + `systematic-debugging` (pinned) · `source-driven-development` · `incremental-implementation` · `context-engineering` (SDP phase=BUILD) + rol: `rust-write-tests` (e2e del wiring + tests core), `api-and-interface-design` (campo público nuevo del seam + módulo host), `documentation-skill` (task file bajo `docs/`). `security-and-hardening` excluida con justificación (ver Fase SECURITY); `performance-optimization` excluida (no hot path — arranque + pass periódico; Regla 9 N/A).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — evaluación: sin trust boundary nuevo. El cambio no acepta input de red nuevo (la ruta `POST /api/v2/conversations` ya existe con auth/RBAC existentes); el trigger captura contenido de conversación a L0 — comportamiento MEM-55 pre-existente, best-effort, sin exposición nueva; la config lee env con prefijo `VANTADB_*` (R-5, warn+default, sin secrets — el único secret posible `VANTADB_OPENAI_API_KEY` ya es env-only y lo consume el runner FIND-112, nunca TOML). Dep nueva = directa de un crate ya presente en el grafo (sin supply chain nuevo). Sin hallazgos que requieran `security-and-hardening`.
- [ ] **PERFORMANCE** — no aplica: wiring de arranque + pass periódico O(queue) fuera de hot paths; el trabajo del engine corre en `spawn_blocking` (concurrency-async R1); sin serialización/search/ingestión tocadas. Regla 9 no dispara (no hay claim de optimización). Delta de build medido en Step 2 (pre-mortem 1 del plan).

## Steps

### Step 1 — RED: tests que fallan por símbolos ausentes (core + e2e)

- **Archivos:** `src/server/bootstrap.rs` (`#[cfg(test)] mod tests` — 1 test del campo nuevo), `vantadb-server/tests/scheduler_e2e.rs` (nuevo — 4 tests)
- **Acción:** RED — escribir los tests que fallan por símbolos ausentes:
  - Core: `on_storage_ready_defaults_to_none_and_is_host_settable` (default inerte + campo setteable con la firma `FnOnce(&mut ServerHooks, Embedded)`).
  - e2e (patrón `conversation_hook.rs`/`scheduler_loop.rs` — `ScriptedRunner` fake, `wait_until` bounded): (1) `e2e_capture_flows_to_l1_via_run_with_hooks` — `run_with_hooks` real + `on_storage_ready` + `wire_memory(20ms, fake)` + `POST /api/v2/conversations` → L0 → pass → `l1/<thread_id>`; (2) `e2e_restart_persists_l0_and_queue_is_ephemeral` — capture → processed → abort + drop → reopen mismo path → L0/L1 persistidos → nueva queue + nueva captura fluye; (3) `e2e_disabled_interval_keeps_bridge_and_spawns_no_loop` — `interval 0` → `background_services` vacío, POST → L0 + cola intacta, sin L1; (4) `e2e_without_runner_skips_pass_and_keeps_queue` — factory `None` → loop spawneado (services=1) pero skip, cola intacta, sin L1.
- **Verify:** `cargo nextest run --profile audit -p vantadb-server --build-jobs 2` → RED correcto (compile error E0432/E0425/E0560 por `vantadb_server::scheduler`/`on_storage_ready` ausentes, no assertion)
- **Evidencia:** ✅ RED verificado (exit 101): e2e `E0433` ×3 (crate `vanta_memory` ausente) + `E0432` (`vantadb_server::scheduler` ausente) — log `$env:TEMP\wire16-red.log`; core `E0609` ×4 (`no field on_storage_ready`, `bootstrap.rs:672/682/685/688`) — log `$env:TEMP\wire16-red-core.log`. Falla por la razón correcta (símbolos/dep ausentes, no assertion).
- **Estado:** ✅ COMPLETED

### Step 2 — GREEN: extensión core del seam + módulo scheduler + wiring

- **Archivos:** `src/server/state.rs` (+campo `on_storage_ready` con rustdoc), `src/server/bootstrap.rs` (hoist `db` + ensure + disparo callback + destructure con `..`), `vantadb-server/Cargo.toml` (+`vanta-memory` con `http-server`/`llm-driver`), `vantadb-server/src/scheduler.rs` (nuevo), `vantadb-server/src/lib.rs` (+`pub mod scheduler`), `vantadb-server/src/server.rs` (+re-exports del seam), `vantadb-server/src/main.rs` (wiring HTTP mode)
- **Acción:** GREEN mínimo — campo core con doc de single-writer/sharing; bootstrap dispara el callback una vez (post-open, post-ensure, pre-state); módulo host con `scheduler_interval_ms_from_env` (warn+default 60_000, `0` off), `ingest_runner_cfg` (`VANTADB_INGEST_CONFIG` > `<storage>/data/vanta-ingest.toml`), `ingest_runner_factory` (degradado → `None`), `wire_memory` (bridge siempre; loop si >0; retorna `Arc<LocalStateBackend<SystemClock>>`); main cablea solo HTTP mode y `!read_only`. Medir delta de build del pre-mortem 1 (cargo check timing antes/después, documentar).
- **Verify:** `cargo nextest run --profile audit -p vantadb-server --build-jobs 2` → GREEN (4 e2e + suite existente) + `cargo nextest run --profile audit -p vantadb --features server --build-jobs 2` (core: test nuevo + suite) + `cargo check -p vantadb` (sin server)
- **Evidencia:** ✅ GREEN focused: `-p vantadb-server --lib --test scheduler_e2e` → **7/7** (3 unit `parse_interval_ms`/`ingest_runner_factory` + 4 e2e) · ✅ `cargo check -p vantadb-server` exit 0 (tras añadir `tracing` directo — E0433 inicial de R-5 warn) · ✅ core focused `-p vantadb --features server --lib -E "test(~on_storage_ready) or test(~default_server_hooks)"` → **2/2**. ✅ **Delta build (pre-mortem 1):** `cargo clean -p vanta-memory -p vantadb-server` + `cargo check -p vantadb-server` (recompilación completa con `http-server` activo) = **6.9s**; warm incremental 0.8s; build test-profile 32.9-38.9s → **sin feature-gate nuevo** (YAGNI: vanta-memory ya en el grafo vía mcp, tokio/server ya compilados, `http-server` suma 2 módulos chicos; Fast Gate intacto). ⚠️ **Colateral bloqueante resuelto inline** (ver §Notas): HEAD no compilaba `vantadb-mcp` (E0063 `core_search`, MEMG-11) → fix 1 línea + `cargo check -p vantadb-mcp` exit 0.
- **Estado:** ✅ COMPLETED

### Step 3 — VERIFY: suites scoped + fmt + clippy + delta build

- **Archivos:** —
- **Acción:** correr el gate del contrato: `cargo nextest run --profile audit -p vantadb-server --build-jobs 2`, `cargo nextest run --profile audit -p vantadb --features server --build-jobs 2`, `cargo check -p vantadb` (default), `cargo fmt --check`, `cargo clippy -p vantadb-server --all-targets -- -D warnings`, `cargo clippy -p vantadb --features server --all-targets -- -D warnings`. Si algo falla → retry ladder.
- **Verify:** los 6 comandos verdes (evidencia cruda en §Step 3)
- **Evidencia:** ✅ **7 gates verdes:** (1) `cargo nextest run --profile audit -p vantadb-server --build-jobs 2` → **12/12** (el perfil `audit` excluye por config los bins `e2e`/`server`/`mcp_integration`/`benchmarks` de este crate — filtro pre-existente del harness, no relacionado con este diff); (2) bins excluidos explícitos `--test e2e --test server --ignore-default-filter` → **39/39** (no-regresión del bootstrap/state); (3) `cargo nextest run --profile audit -p vantadb --features server --build-jobs 2` → **2678 run: 2678 passed, 4 skipped** (el flake conocido `concurrent_insert_preserves_hnsw_invariants` pasó esta vez, slow 175.5s); (4) `cargo check -p vantadb` (default, sin server) exit 0; (5) `cargo fmt --check` exit 0 (tras `cargo fmt -p vantadb-server` — 4 diffs, todos en archivos propios; core/mcp ya limpios); (6) `cargo clippy -p vantadb-server --all-targets -- -D warnings` exit 0; (7) `cargo clippy -p vantadb --features server --all-targets -- -D warnings` exit 0.
- **Estado:** ✅ COMPLETED

### Step 4 — CIERRE: OCR + Review P2-01 + commit local + campaign

- **Archivos:** `docs/dev/tasks/WIRE-16.md` (§Review + RESULTADO §7)
- **Acción:** `pwsh dev-tools/ocr-review.ps1 -Format json` (advisory; revisar por Rule Group — Critical/High bloquean; Medium → FIND) · clasificar tier HARD-02: paths `vantadb-server/**` + `src/server/**` + `docs/dev/tasks/**` → **Fast** (ninguno matchea los globs adversariales) → verify fast mecánico + **veredicto registrado**; fork `vanta-review` (agente distinto, fresh context) con contrato + diff + evidencia para el veredicto P2-01 (incluye la extensión core del seam) · gates docs (`check-links`/`check-docs`/`gen-index --check`) · commit **LOCAL** `feat(wire):` con pathspec de archivos propios · `campaign_update_task_state(completed, taskId:"47")` con recitation + payload `review` · `skill progreso`.
- **Verify:** veredicto registrado en §Review + `git show --stat HEAD` limitado a archivos propios + `git status` sin WIP ajeno stageado
- **Evidencia:** [a llenar]
- **Estado:** ⬜ PENDING

## Dependencias

- **WIRE-14 ✅** (`b5d294d2` + `4448788b`): seam `ServerHooks`/`BackgroundService`/`run_with_hooks` — releído fresco desde HEAD; handoff Low-1/Low-2 incorporado.
- **WIRE-15 ✅** (`1d5e1697` + `9c881047`): `run_pass`/`MemoryScheduler`/`spawn_memory_scheduler` (factory `Fn() -> Option<R>`) — releído fresco; el fake runner del e2e lo explota.
- **Habilita:** WIRE-17 (docs, post WIRE-16). ADR-0054 T3. Cierra residual MEM-55.
- **Coordinación:** releído fresco desde HEAD `fc4e994c`; cambios mínimos + pathspec; race de staging multi-sesión → `git commit -- <paths>` SIEMPRE; conflicto real → BLOQUEO.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` — P2-01, contexto fresco (no participó de la implementación); tier **Fast** (HARD-02: `vantadb-server/**` + `src/server/**` + `vantadb-mcp/**` + `docs/dev/tasks/**` + `Cargo.lock` + `docs/index.md` + `llms.txt`; ningún path matchea globs adversariales). Sesión reviewer: `ses_ef35294d8ffeEQOowuvrmlAMap`. Review **pre-commit** (changeset sin commitear). Veredicto: ✅ **APPROVE** (0 Critical / 0 High / 0 Medium).
- **Enfoque:** contrato punto por punto (6 puntos: seam core aditivo/firma intacta/orden del callback; wiring host; 4 e2e con outcome real; colateral MCP; concurrencia/lifecycle; coordinación) + alternativas evaluadas (callback vs tuple vs `hooks.storage` — rechazadas con evidencia de hazards audit/purge/supersede) + RBI sobre el lifecycle de parada + verificación de que ningún otro literal exhaustivo de `RecallConfig` quedó roto (22 hits analizados) + WIP ajeno.
- **Cómo se probó:** corridas propias del reviewer (no auto-reporte) — audit `-p vantadb-server` **12/12** · bins excluidos `--test e2e --test server --ignore-default-filter` **39/39** · core focused seam **2/2** · `cargo check -p vantadb` + `-p vantadb-mcp` exit 0 · `cargo fmt --check` exit 0 · clippy ×2 exit 0 · docs gates ×3 exit 0 · RED logs re-leídos (E0433/E0432/E0609×4) · build frío test-profile 41.8s (Fast Gate confirmado). Caveat honesto del reviewer: no re-ejecutó la suite core completa (2678, ~3min) — spot-check con focused + clippy all-targets + check default.
- **Hallazgos + disposición:**
  1. [Low-1] Warnings R-5 invisibles en producción (lecturas env/TOML en `main` antes de `init_telemetry`) → **APLICADO**: ambas lecturas movidas dentro del closure `on_storage_ready` (corre post-telemetría) + `storage_path` clonado; re-verificado (fmt/clippy/focused).
  2. [Low-2] Orden flush→join (pre-existente del seam WIRE-14; con scheduler, un pass en vuelo puede escribir post-flush — ventana ms, exit limpio seguro) → **FIND-291** creada en `docs/dev/Backlog.md` (fila 10-col, severidad 🟢 Baja); no bloquea (lifecycle fijado por WIRE-14/15).
  3. [Optional-1] `ingest_toml_path` sin test → **APLICADO**: helper puro `resolve_ingest_toml_path` + test unitario (override/blank/fallback).
  4. [Optional-2] Restart e2e no aserta un L1 nuevo (dedup puede fusionar) → **DISPENSA**: verificación = drenaje de cola + L0 persistido (semántica FIND-113); caveat documentado en el test.
  5. [NIT-1] Cita FIND-113-spec :52 → **APLICADO** (`:218`).
  6. [NIT-2] Ruido de re-padding en `docs/index.md` → **DISPENSA**: salida canónica de `gen-index` (exit 0).
  7. [NIT-3] Nombres de tests en §Steps desalineados → **APLICADO** (nombres reales).
- **Veredicto:** ✅ **APPROVE** — contrato + DoD Task + gates verdes reproducidos; 0 Critical/High/Medium; Lows/Optional/NITs aplicados o dispensados con motivo; changeset listo para commit local con pathspec (sin push).

## Notas

- **Decisión de diseño clave (incógnita real resuelta):** el host NO puede abrir la DB (segundo open mismo-proceso = `DatabaseBusy`; evidencia `tests/storage/multi_process_lock.rs:14-52`) ni crear un segundo handle `Embedded` (partiría audit + purge/supersede locks). Por eso la completitud del seam WIRE-14 es `ServerHooks::on_storage_ready` — el server comparte su handle (clon) con el host post-open. Alternativas B/C/D rechazadas con evidencia (§Spec #1).
- **Restricción e2e documentada:** `init_telemetry` usa `.init()` (panics en doble init) → **un solo `run_with_hooks` por proceso de test**; la verificación (1) lo usa; (2)(3)(4) usan `wire_memory` + `app()` (patrón e2e existente).
- **Delta de build (pre-mortem 1):** medido en Step 2; sin feature-gate nuevo si ~0 (vanta-memory ya en el grafo; `http-server` compila 2 archivos + tokio ya presente).
- **Restart (semántica FIND-113):** cola efímera (RAM); L0 persistido = dato nunca perdido; no existe re-enqueue automático desde L0 — no se inventa mecanismo (YAGNI, spec ausente).
- **NOTICED BUT NOT TOUCHING:** `run_bridge_pass` (semántica distinta) no converge; MCP como segundo host diferido (ADR); `docs/api/**` → WIRE-17.
- **Coordinación:** WIP ajeno (`opencode.jsonc`, master plan) intacto; commit con pathspec.
- **Colateral bloqueante (resuelto inline, commit separado `fix(mcp):`):** HEAD `fc4e994c` **no compilaba** `vantadb-mcp` — MEMG-11 (`508e211e`) agregó `RecallConfig::core_search` y su task file verificó "únicos literales exhaustivos son la propia definición" (**incorrecto**): `vantadb-mcp/src/handlers/tools.rs:1988` es un literal exhaustivo (los otros sitios usan `..Default::default()`/`..RecallConfig::default()` — verificado: context.rs, vanta-proxy, desktop). Fix mecánico 1 línea `core_search: false` (= path legacy byte-identical; adoptar el path híbrido core = decisión de producto aparte, no de este unblock). Root cause: el verify scoped `-p vanta-memory` no construye `vantadb-mcp` (Regla dura `-p`), el gap quedó latente hasta que WIRE-16 necesitó compilar `vantadb-server`. Lección registrada vía `campaign_memory_write`.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | **0** — acceso del host a la DB resuelto en DISCOVERY (§Spec #1) |
| Pendientes de ejecución (downhill) | **1** step (4 — CIERRE) |
| % completado | 75% (steps 1-3 ejecutados y verificados; cierre en curso) |

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: [a llenar]
STEPS_OK: [a llenar]
PROXIMO_STEP: [a llenar]
COMMIT_HASH: [a llenar]
ARCHIVOS: [a llenar]
VERIFY_CONTRATO: [a llenar]
BLOQUEO: [a llenar]
GATES_EVALUADOS: [a llenar]
SKILLS_CARGADAS: [a llenar]
```
