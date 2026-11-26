---
title: "FIND-745: backfill L0→cola L1 tras restart con marcador durable de consolidación"
kind: task
status: completed
description: "Marcador durable `l0_consolidated/<session>` (escrito sólo tras un pass exitoso) y barrido de arranque que re-encola una tarea L1 por sesión con turnos no consolidados. Proxy no hace backfill y lo declara. Observable en `capabilities.ingest.backfill`."
---

# FIND-745: backfill L0→cola L1 tras restart

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md`
- **Fuente:** Backlog `FIND-745` — derivada de `MCP-43` §Spec-6 (decisión b: sin backfill, techo declarado)
- **Esfuerzo:** 🟡 ~1d (appetite 1d)
- **Tipo:** Rust (feature-add: capa L0 + arranque del scheduler MCP). Símbolos `pub` nuevos **aditivos** en `vanta-memory` (`backfill_unconsolidated`, `BackfillReport`, `L0Recorder::{consolidated_ms, mark_consolidated}`, `observability::{BackfillSnapshot, BackfillReason, note_backfill, note_backfill_failed}`) porque los consume `vantadb-mcp`; ninguna firma existente cambia
- **Carril:** R (1 de 2 slots; `FIND-152` en el otro)
- **Estado:** ✅ COMPLETED
- **Creado:** 2026-11-26

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-mcp/src/memory_scheduler.rs::start_with` (único arranque de loop: writer ×2 + promoción MCP-48), `vanta-memory/src/services/pipeline_worker.rs::run_l1_inner` (único punto donde un pass L1 persiste su resultado) |
| Callees | `vanta_memory::core::conversation::consolidation::backfill_unconsolidated`, `L0Recorder::{mark_consolidated, consolidated_ms}`, `vanta_memory::ingest::observability::note_backfill`, `Embedded::list_namespaces`, `LocalStateBackend::{enqueue_task, list_queued_tasks}` |
| Implicaciones | Un namespace nuevo (`l0_consolidated/<session>`), un campo aditivo en `IngestStatus` (`backfill`) y un log aditivo en la rama proxy de `server.rs`. Sin migración: el marcador es derivado y reconstruible (si falta, el primer pass lo escribe). Sin cambios de comportamiento en L1 (el pass sigue leyendo toda la sesión; el marcador sólo decide el backfill). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o en la zona editada):** `vanta-memory/src/core/conversation/l0_recorder.rs` (554L), `vanta-memory/src/core/conversation/mod.rs`, `vanta-memory/src/ingest/observability.rs` (565L), `vanta-memory/src/services/scheduler.rs` (399L), `vanta-memory/src/services/pipeline_worker.rs` (L440-600 + helpers), `vanta-memory/src/utils/local_backend.rs` (L1-250), `vantadb-mcp/src/threads.rs` (637L), `vantadb-mcp/src/memory_scheduler.rs` (393L), `vantadb-mcp/src/server.rs` (L1000-1134), `vantadb-mcp/src/handlers/tools.rs` (L3190-3205), `vantadb-mcp/tests/capabilities_ingest.rs`, `docs/dev/tasks/MCP-43.md`, `docs/api/MCP.md` (§ capabilities + § thread writes), `docs/api/VANTA_MEMORY.md` (§ operational modules + namespaces).
- **Referencias hacia dentro:** `run_l1_inner` ← `MemoryTaskHandler::handle` ← `PipelineWorker::run_once` ← `run_pass` ← `spawn_memory_scheduler`/`run_bridge_pass`; `start_with` ← `memory_scheduler::start` ← `server.rs:1064/1119` + `proxy.rs:590/643`.
- **Referencias entrantes de lo editado:** contrato de `capabilities` (`docs/api/MCP.md:459-489`, `tests/capabilities_ingest.rs` — aserta claves sueltas, no el set exacto: aditivo seguro) y contrato de `thread_send` (MCP-44, intacto). Ninguna firma pública existente cambia; `backfill` es un campo `Option` nuevo.
- **Veredicto impacto:** MEDIO. Un módulo nuevo (~130L + tests), un método de marcador en el recorder, un campo aditivo en observability y una llamada en el L1. Pre-mortem: (a) re-encolar todo → el marcador lo impide y el test lo mide; (b) backfill en proxy → sólo corre dentro de `start_with`, que un proxy no promovido nunca llama, y el log lo nombra; (c) marcador antes de consolidar → se escribe al final del pass (y en los no-ops post-LLM), nunca antes; (d) segundo predicado de salud → `backfill` es un snapshot dentro de `ingest`, no toca `derived_health`.

## Spec

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Dónde vive el marcador | A) namespace propio `l0_consolidated/<session>`, misma forma `{"after_timestamp_ms"}` y mismo `__cursor` que el cursor de captura / B) campo en cada record L0 (reescribir el L0 entero tras cada pass: inmutable-by-design, y una escritura por turno) / C) metadata del thread store en `vantadb::agentic` (`src/**`, prohibido por FIND-152) | ✅ **A** — el cursor de captura (`l0_cursor/<session>`, `l0_recorder.rs:471-484`) es el precedente exacto: mismo `sanitize_component`, mismo `CURSOR_KEY`, mismo JSON. "Capturado" y "consolidado" avanzan por actores distintos en momentos distintos, así que no pueden compartir record |
| 2 | Quién escribe el marcador y cuándo | A) `run_l1_inner` al final del camino exitoso (y tras los no-ops donde el LLM sí corrió) / B) el scheduler MCP después de `run_pass` (no conoce sesiones: `PassStats` es agregado) / C) inferirlo de los records L1 (un turno que no produjo memoria nunca tendría marcador → re-extracción en cada arranque) | ✅ **A** — es el único punto con (sesión, mensajes, resultado) simultáneos. Escribir antes de persistir es el pre-mortem (c): pierde turnos en vez de duplicarlos. `mark_consolidated` es monótono, así que un pass viejo terminando tarde no rebobina |
| 3 | Granularidad del re-encolado | A) una tarea L1 por sesión no consolidada / B) una por turno | ✅ **A** — el pass L1 ya lee la sesión completa (`read_indexable_messages`) y deduplica; N tareas = N extracciones completas. `messages` en el reporte conserva la visibilidad del volumen |
| 4 | Qué cuenta como "pendiente" | A) turnos indexables (`read_indexable_messages`, excluye ruido trivial) / B) todo `read_messages` | ✅ **A** — el ruido trivial nunca se extrae por diseño (NOT-04); con B), una sesión cuyo último turno es "ok" se re-encolaría en **cada** arranque |
| 5 | Cuándo corre el barrido | A) dentro de `start_with`, una vez por proceso, cuando el loop arranca de verdad / B) en la rama proxy (sin cola que drenar: tarea huérfana) / C) con `interval=0` (encolaría un backlog que nadie drena) | ✅ **A** — mismo predicado que MCP-43 para "quién hospeda": writer y promovido sí, proxy no. El proxy lo declara en su log (`backfill = "skipped (proxy)"`) |
| 6 | Observabilidad | A) campo aditivo `ingest.backfill {at_ms, sessions, requeued, messages, reason}` publicado por `note_backfill` / `note_backfill_failed` / B) log solo / C) tool nueva | ✅ **A** — ADR-0065: la superficie es única y el contador encaja dentro de `ingest`, no al lado. `reason ∈ nothing\|requeued\|marker\|failed` hace legible el cero y el fallo. No toca `last_pass` (un barrido no es un pass, D7) ni el predicado de salud |
| 7 | Sesiones a escanear | A) prefijos `l0/`, `l0_cursor/` y `l0_consolidated/` vía `list_namespaces` / B) sólo `l0_cursor/` | ✅ **A** — un crash entre el put del record y el write del cursor dejaría la sesión invisible hasta su próximo turno; el costo es el mismo `list_namespaces` que ya paga `run_decay_maintenance_pass` |

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | RED: tests de backfill (vanta-memory, queue local = restart fiel) + test e2e MCP (L0 huérfano → boot → tick → recall; restart → 0 re-encolados) | ✅ |
| 2 | Green: `L0Recorder::{mark_consolidated, consolidated_ms}` + `core::conversation::consolidation` | ✅ |
| 3 | Wire: marcador al final del pass L1 (`pipeline_worker::run_l1_inner`) | ✅ |
| 4 | Wire: barrido en `memory_scheduler::start_with` + `observability::note_backfill` + log proxy | ✅ |
| 5 | Docs: `docs/api/MCP.md` (§ ingest, § thread writes) + `docs/api/VANTA_MEMORY.md` (§ observability, § namespaces, § L0 modules) | ✅ |
| 6 | Verify contrato + suites + OCR + review P2-01 + cierre (Backlog/avance/commit) | ✅ |

## Verification

| Comando | Resultado |
|---|---|
| `cargo nextest run --profile audit -p vantadb-mcp -E 'test(backfill) or test(l0) or test(thread)' --build-jobs 2` | ✅ 8/8 pass, 194 skipped (pre-fix) |
| `cargo nextest run --profile audit -p vantadb-mcp -E 'test(backfill) or test(capabilities)' --build-jobs 2` (post-review H-1/H-3) | ✅ 3/3 pass, 199 skipped |
| `cargo nextest run --profile audit -p vanta-memory -E 'test(conversation) or test(l0)' --build-jobs 2` | ✅ 17/17 pass, 737 skipped (post-review también) |
| `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` (suite completa, post-review) | ✅ 752/752 pass, 2 skipped (11.4 s) |
| `cargo nextest run --profile audit -p vantadb-mcp --build-jobs 2` (suite completa, post-review) | ✅ 201/201 pass, 1 skipped (93.1 s) |
| `cargo clippy -p vanta-memory -p vantadb-mcp --all-targets -- -D warnings` | ✅ limpio |
| `cargo fmt -p vanta-memory -p vantadb-mcp -- --check` | ✅ limpio |
| `pwsh dev-tools/ocr-review.ps1` | ver §Review |

**DoD medido (el corazón del ticket, no auto-reporte):**

- `memory_scheduler::tests::backfill_revives_a_turn_captured_before_a_kill_and_never_re_enqueues_consolidated_ones` — L0 escrito **antes** de que exista loop (proceso muerto entre `thread_send` y el pass) → `start_with` (25 ms) re-encola **1** sesión / **1** mensaje (el barrido corre en `spawn_blocking`; el test espera el snapshot, no lo asume) → el tick del loop consolida → `memory_recall` lo devuelve **sin `memory_put`** → `capabilities.ingest.backfill.requeued == 1`. `stop()` + `start_with` (restart): `requeued == 0`, `reason == marker`, y tras ~8 ticks vacíos el contador de extracciones **no se movió**. Segundo turno capturado después del restart → `requeued == 1`, `messages == 1`, y el contador sube **exactamente 1** (no 2): el turno ya consolidado no paga nada.
- `core::conversation::consolidation::tests::backfill_requeues_only_unconsolidated_l0_turns_across_restarts` — misma secuencia con `LocalStateBackend` fresco por "boot" (restart fiel, sin estado compartido): turno consolidado → marcador; boot 2 re-encola 0 y el contador de extracciones queda en 1; turno nuevo → 1 re-encolado → contador en 2.

## Invariantes de dominio (handoff — MUST)

- **El marcador no se adelanta a la consolidación.** Sólo se escribe después de que el pass persistió su resultado (o de un no-op donde el LLM corrió). Adelantarlo pierde turnos.
- **Monótono.** `mark_consolidated` nunca rebobina: un pass viejo terminando tarde no puede des-consolidar.
- **Un barrido no es un pass.** No toca `last_pass`, no emite `degraded_warnings`, no participa de `derived_health` (ADR-0065 D7).
- **Una tarea por sesión.** El barrido no duplica backlog: si ya hay una L1 encolada para la sesión, no encola otra.
- **Proxy no promovido: cero backfill.** La cola es del writer (MCP-43); un barrido local encolaría trabajo sin drenador. Un proxy **promovido** (MCP-48) sí barre: ya es writer.
- **Nunca fatal.** Un fallo del barrido/asiento se loguea y deja los turnos en `l0/` (P4): nada se pierde.
- Sin dependencias nuevas; `unwrap`/`expect` sólo en tests.

## Deuda

- **Un LLM que responde algo que no parsea se lee como "no había nada que guardar".** `l1_extractor` marca `success: true` tras `parse_l1_extraction` incluso si el texto no era JSON (`l1_extractor.rs:144-146`); aquí eso avanza el marcador, así que esos turnos no se re-extraen hasta el próximo turno de la sesión. Separarlo exige una señal de parse en `L1ExtractionResult` (`core/record/**`, fuera del scope de escritura de esta fila). Se documenta en el código, no se finge una señal que no existe.
- **El barrido no está cableado en el host HTTP.** `vantadb-server` hospeda loop (WIRE-16) pero no llama `backfill_unconsolidated`; hoy la recuperación tras restart existe sólo en `--mcp`. El sweep es una función del crate, así que cablearlo es una llamada en `vantadb-server/src/scheduler.rs`.
- **Barre aunque el runner sea `none`.** Un `--mcp` degradado encola una L1 por sesión no consolidada y el tick la salta (`note_skipped_pass`): el backlog se lee `broken`, que es la verdad (hay turnos capturados sin consolidar). El costo es cero LLM; se acepta en vez de esconder el estado.
- **Carrera benigna con el primer tick.** El loop se spawnea antes del barrido (que corre en `spawn_blocking`), así que el tick t=0 puede no ver todavía las tareas re-encoladas y procesarlas en el siguiente intervalo. Cuesta a lo sumo un pass; el lock por sesión impide doble procesamiento.
- **Costo del barrido: O(#namespaces) por arranque** (`list_namespaces` + una lectura L0 por sesión capturada). Aceptable (boot-only, mismo costo que el pass de decay) y declarado con `ponytail:` en el módulo; el upgrade path es un índice `l0_cursor/`.
- **Turnos con el mismo milisegundo que el marcador.** El marcador compara `>` sobre `timestamp_ms`; dos mensajes en el mismo ms en los que el segundo llega después del pass comparten timestamp y el barrido no los vería hasta el próximo turno. La ventana es más real de lo que sugiere "teórica": un host programático que apende dos mensajes en el mismo ms la produce, y el propio L0 tiene `tie_break` por exactamente ese caso (`l0_recorder.rs:440-469`). Cerrarla exige un set de keys en el marcador (payload O(n)); no se paga hasta que aparezca un caso real.
- **Sin tope de intentos entre arranques.** Un turno condenado (el runner falla siempre) se re-encola una vez por boot. Es el comportamiento correcto de reintento (nada se pierde), pero no está acotado: si hiciera falta, el `attempts` del `TaskPayload` es el lugar.
- **Los tests del barrido dependen de estado proceso-global** (`observability`, `memory_queue`): correcto bajo nextest (un proceso por test), declarado igual que MEM-72/MCP-43. Bajo `cargo test` en el mismo binario habría que serializarlos.
- **Sesiones compartidas con el host HTTP.** El barrido del `--mcp` enumera `l0/` completo, así que también recupera sesiones capturadas por `vantadb-server` (misma DB, mismo pipeline). Es correcto (una pasada del planner por sesión) pero no está acotado al thread store; queda declarado.
- **`FIND-152` en vuelo** en `src/**`: el marcador evita `vantadb::agentic` y vive entero en `vanta-memory`.
- **NOTICED BUT NOT TOUCHING:** `docs/api/MCP.md` sigue documentando que un `thread_send` en modo degradado fija `last_pass` vía `note_skipped_pass` (deuda R2 de MCP-44, ya declarada en MCP-43 §Deuda). Este ticket no la toca.

## Review (GATE P2-01 — agente distinto)

- Revisor: `vanta-review` (contexto fresco; verificó el contrato por ejecución propia: `-p vantadb-mcp -E 'test(backfill)'` 1/1 y `-p vanta-memory -E 'test(backfill)'` 2/2)
- Enfoque: los 6 pre-mortems del ticket (marcador antes de consolidar, re-encolar en cada arranque, backfill en proxy, segundo predicado de salud), correctitud del e2e, uso del lock, claims de docs

| Hallazgo | Severidad | Resolución |
|---|---|---|
| **H-1** el barrido hacía I/O de `StorageEngine` en el hilo async (regla dura R-7/INV-003, `concurrency-async.md:46-48`) | **Blocking** | `schedule_backfill` corre `publish_backfill` en `Handle::spawn_blocking` (inline sólo sin runtime); los 4 call sites (`server.rs` ×2, `proxy.rs` ×2) no cambian. El test espera el snapshot con `wait_for_backfill` en vez de asumir sincronía |
| **H-2** el marcador avanza cuando el LLM respondió algo que no parsea (`success: true` incondicional tras parse) | Medium | No se puede separar sin señal en `L1ExtractionResult` (`core/record/**`, fuera de scope): se declara en §Deuda y se corrigen los comentarios que afirmaban "the LLM judged" |
| **H-3** un barrido fallido era indistinguible de "no hubo barrido" (`backfill: null`) | Medium | `BackfillReason::Failed` + `note_backfill_failed()`; el `Err` publica `reason: "failed"` antes de loguear |
| **H-4** `VANTA_MEMORY.md` afirmaba que barre "quien hospeda el loop" — falso para el host HTTP | Medium | Texto acotado a `--mcp`; el host HTTP queda en §Deuda |
| **H-5** el gate se declaraba cerrado con §Review vacía y fila de Backlog ya borrada | Medium (proceso) | Esta sección se llena con el dictamen; la fila se quitó del Backlog **antes** del commit de cierre (no del de código) y el estado se sostiene sobre este veredicto |
| **H-6** `turns` contaba mensajes, no turnos, en superficie pública | Low | Renombrado a `messages` en `BackfillReport`, `BackfillSnapshot`, docs, log y asserts |
| **H-7** "el proxy nunca llama `start`" es falso: un proxy **promovido** (MCP-48) sí | Low | Redacción corregida en el módulo, `MCP.md` y `VANTA_MEMORY.md`: "plain (unpromoted) proxy"; el promovido barre porque ya es writer |
| **H-8** barre aunque el runner sea `none` | Low | Aceptado y declarado en §Deuda: el backlog se lee `broken`, que es la verdad; cero costo LLM |
| **H-9** carrera benigna con el tick t=0 del loop | Low | Aceptada y declarada (§Deuda): cuesta a lo sumo un pass; el lock por sesión evita doble procesamiento |
| **H-10** "monotonic by construction" era realmente "bajo el lock de sesión" | Nit | Doc-comment corregido: un rewind sólo duplica una extracción, nunca pierde un turno |
| **H-11** `consolidation.rs` untracked con `mod.rs` ya apuntándolo | Low | `git add` explícito en el commit; verificado con `git show --stat` (`A`, no ausente) |
| **H-12** estado proceso-global en los tests | Low | Declarado (§Deuda), mismo convenio que MEM-72/MCP-43 |
| **H-13** "written only after a pass persisted its result" era incompleto | Nit | `MCP.md` dice "…or deliberately stored nothing" |
| **H-14** metadato "sin símbolos públicos nuevos en el core" | Nit | Metadata corregida: símbolos `pub` aditivos en `vanta-memory` (los consume `vantadb-mcp`) |
| **H-15** ventana same-ms más débil de lo declarado | Nit | §Deuda amplía: caso real de host programático + por qué el set de keys no se paga |
| **H-16** sin tope de intentos entre arranques | Nit | §Deuda: reintento correcto pero no acotado; `TaskPayload::attempts` es el lugar si hiciera falta |

**Veredicto ronda 1:** `changes-required` (1 Blocking + 4 Medium + 7 Low/Nit) → **todos resueltos o declarados** en esta ronda.
**Re-verificación post-fix:** foco `vantadb-mcp` 3/3 (backfill + capabilities), foco `vanta-memory` 17/17, suites completas 752/752 y 201/201, clippy `-D warnings` ✅, fmt ✅.
