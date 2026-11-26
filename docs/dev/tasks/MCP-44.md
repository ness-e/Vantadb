---
title: "MCP-44: thread_send alimenta el pipeline de memoria (L0 + enqueue L1)"
kind: task
status: completed
description: "`thread_send` deja de ser solo storage: captura el turno en L0 y encola UNA tarea L1 en la cola del proceso; con runner `none` lo publica en `capabilities.ingest` (MEM-72) — la ejecución del pass queda para el scheduler (MCP-43)."
---

# MCP-44: `thread_send` alimenta el pipeline de memoria

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§9.b, ola MCP/MEM)
- **Fuente:** Backlog `MCP-44` — `thread_send` debe alimentar el pipeline (captura L0 + enqueue L1), no solo escribir storage
- **Esfuerzo:** 🟡 1-2d
- **Tipo:** Rust (feature-add: comportamiento nuevo + campos aditivos en la respuesta del tool)
- **Estado:** ✅ COMPLETED
- **Creado:** 2026-11-25

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-mcp/src/handlers/tools.rs:3807` (`handle_thread_tool` ← `tools/call`, y vía proxy HTTP `mcp_proxy_handler` en el writer), hosts MCP (brain/agent/full), `vantadb-mcp/tests/thread_tests.rs` |
| Callees | `vanta_memory::core::conversation::{L0Recorder, L0Capture, L0Message, L0Role}` (captura L0, LLM-free), `vanta_memory::utils::{LocalStateBackend, SystemClock}` (cola proceso-local, ADR-0054), `vanta_memory::ingest::observability::{status, note_skipped_pass, QueueSnapshot}` (MEM-72), `vanta_memory::core::state::{TaskKind, TaskPayload}` |
| Implicaciones | Solo aditivo. El contrato `{ok:true}` se mantiene; se añade `memory:{captured,queued}`. La cola es **proceso-local** (ADR-0054 / MEM-72 §Techo): en modo proxy el handler no corre en este proceso — el proxy reenvía al writer (MCP-48) y **allí** se encola, en el proceso que tiene el lock. Sin migración de datos. Una tarea L1 por turno nuevo (no por mensaje de storage, no por replay del cursor L0). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o en la zona editada):** `vantadb-mcp/src/threads.rs` (282L), `vantadb-mcp/src/handlers/tools.rs` (L2338-2515 recall, L3780-3830 dispatch), `vantadb-mcp/src/server.rs` (L1-1248: writer HTTP + proxy dispatch), `vantadb-mcp/src/proxy.rs` (L560-649 promote), `vantadb-mcp/tests/thread_tests.rs` (179L), `vantadb-mcp/tests/capabilities_ingest.rs` (71L), `vanta-memory/src/ingest/observability.rs` (325L), `vanta-memory/src/ingest/worker.rs`, `vanta-memory/src/ingest/mod.rs`, `vanta-memory/src/ingest/runner_config.rs` (438L), `vanta-memory/src/services/scheduler.rs` (394L), `vanta-memory/src/services/conversation_hook.rs` (107L), `vanta-memory/src/services/pipeline_worker.rs` (`handle`/`run_l1`), `vanta-memory/src/utils/local_backend.rs` (L1-420), `vanta-memory/src/utils/pipeline_manager.rs` (147L), `vanta-memory/src/core/conversation/l0_recorder.rs` (554L), `vanta-memory/src/core/hooks/auto_capture.rs`, `vanta-memory/src/core/hooks/auto_recall.rs` (`read_scoped_records`, `InjectionPolicy`), `vanta-memory/tests/scheduler.rs`, `docs/api/MCP.md` (threads), `docs/dev/tasks/MEM-72.md`.
- **Referencias hacia dentro:** `handle_thread_tool` ← `handlers/tools.rs:3807` (único punto de dispatch); `threads::handle_thread_tool` no se exporta fuera del crate (`mod threads;` privado en `lib.rs`).
- **Referencias entrantes de lo editado:** el contrato del tool `thread_send` (`docs/api/MCP.md:587`, `handlers/resources.rs` family map, `handlers/tools.rs` annotations) y los tests `thread_tests.rs`.
- **Veredicto impacto:** MEDIO. Una función nueva (`capture_and_enqueue`) + un `OnceLock` de cola + un campo aditivo en la respuesta + tests. Pre-mortem: (a) doble encolado → una única llamada a `enqueue_task` por turno **nuevo** (`recorded_count > 0`); el test cuenta tareas por turno; (b) encolar en el proxy → el handler solo corre en el writer (proxy reenvía, MCP-48), y el test lo deja documentado en la frontera; (c) dar por hecho que el pass corre solo con runner `none` → el test verifica `capabilities.ingest.is_broken()` y que el scheduler NO está en este proceso.

## Contrato

```
cargo nextest run --profile audit -p vantadb-mcp -E 'test(thread)' --build-jobs 2
```

más un test que: envía un turno con contenido recuperable, ejecuta el pass L1 con un runner scripted, y demuestra que `memory_recall` lo devuelve (sin `memory_put`). La mitad honesta: con runner `none`, `capabilities.ingest.is_broken()` es `true` tras encolar (encoló y nada puede extraerlo).

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Dónde vive la cola L1 del proceso MCP | A) `OnceLock<Arc<LocalStateBackend<SystemClock>>>` en `threads.rs`, reutilizable por MCP-43 / B) una cola nueva por llamada (muere con la llamada: nadie la drena) / C) tocar `server.rs` para inyectarla en el dispatch | ✅ A — evidencia: ADR-0054 "one loop per process"; la cola es in-process y MCP-43 debe tomar la MISMA instancia |
| 2 | Quién captura L0 | A) `L0Recorder` directo (mismo camino que `HttpCaptureBridge::trigger`) / B) `AutoCaptureHook` (filtra por cursor de sesión: pensado para replay de transcripciones, aquí perdemos turnos) | ✅ A — evidencia: `conversation_hook.rs:59-73` es el patrón del host para un turno suelto |
| 3 | Rol no conversacional (`system`, `tool`, …) | A) escribir el thread y saltar memoria (`captured:false`) / B) error del tool | ✅ A — los roles del thread son libres (MCP-32); romper el write por un rol de memoria no aplica |
| 4 | Encolar cuando el turno es replay del cursor L0 | A) no encolar (`recorded_count == 0`) / B) encolar igual (extracción duplicada) | ✅ A — evidencia `l0_recorder.rs:290-316` (el cursor filtra replays) |
| 5 | Visibilidad del backlog cuando el runner es `none` | A) publicar profundidad con `note_skipped_pass` (MEM-72 ya trata "sin runner + cola" como pipeline roto y avisa una vez por episodio) / B) señal nueva en `capabilities` / C) `note_backlog` nuevo en `vanta-memory` (archivo prohibido en esta tarea) | ✅ A — "no inventes otra señal" (MCP-44 §1); C queda propuesto en §Incógnitas. Coste aceptado y documentado: la publicación fija `last_pass.skipped` (ver `docs/api/MCP.md` § Thread writes feed memory) |
| 7 | Cómo se entera el proceso MCP de su runner (review R1) | A) anotarlo en el arranque del host (`server.rs`, **prohibido** por MCP-44) / B) anotarlo en el primer enqueue con el mismo mapeo config→runner (`IngestRunnerCfg::from_env_toml` + `build_runner`; respeta un runner ya declarado) / C) documentar que `is_broken()` no aparece hasta un pass | ✅ B — el estado pasa a ser veraz hoy sin tocar `server.rs`; una sola lectura de config por proceso |
| 6 | ¿`thread_send` ejecuta el pass? | A) NO: encola y deja la ejecución al scheduler (MCP-43) / B) correr `run_pass` en el handler (bloquea el turno y duplica el scheduler) | ✅ A — §8 del prompt: el scheduler es MCP-43 |

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | `threads.rs`: cola proceso-local + `capture_and_enqueue` (L0 + 1 tarea L1) + publicar backlog degradado | ✅ |
| 2 | `thread_send`: respuesta aditiva `memory{captured,queued}` + descripción del tool | ✅ |
| 3 | Tests: e2e L0→pass→`memory_recall` (lib) + degradado/`capabilities` y "una tarea por turno" (`tests/`) | ✅ |
| 4 | Docs: `docs/api/MCP.md` (thread_send + frontera proxy/scheduler) | ✅ |
| 5 | Verify contrato + OCR + review P2-01 + commit local | ✅ |

## Verification

| Comando | Resultado |
|---|---|
| `cargo nextest run --profile audit -p vantadb-mcp -E 'test(thread)' --build-jobs 2` | ✅ 7/7 pass (191 skipped) — corrido dos veces: antes y después de los fixes del review |
| `cargo nextest run --profile audit -p vantadb-mcp --test thread_tests --test thread_memory_pipeline --build-jobs 2` | ✅ 8/8 pass |
| `cargo nextest run --profile audit -p vantadb-mcp --build-jobs 2` (suite completa) | ✅ 198/198 pass (primera corrida). Segunda corrida: 1 fallo **flake** ajeno — `mcp_proxy_reconnect::live_writer_lock_blocks_promotion` ("bounded retry violated: 5.05s", test de timing de MCP-48 bajo contención del gate concurrente); re-corrido del binario: 4/4 pass |
| `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` | ✅ limpio |
| `cargo fmt -p vantadb-mcp -- --check` | ✅ limpio |
| `node scripts/docs/check-links.mjs` / `check-docs.mjs` | ✅ 0 broken links; exit 0 |
| `npx markdownlint-cli2 --config .github/markdownlint-cli2.yaml docs/api/MCP.md` | ✅ 0 issues |
| `pwsh dev-tools/ocr-review.ps1 -Format json` | ✅ aplicado (rule groups Rust + server-mcp sobre `threads.rs`; sin Critical/High propios) |

## Invariantes de dominio (handoff — MUST)

- **Una tarea L1 por turno nuevo:** nada de doble encolado; el replay del cursor L0 no encola.
- **Frontera de proceso:** la cola es del proceso que tiene el lock (escritor). El proxy NO encola localmente: reenvía (MCP-48).
- **P4:** si L0 falla, el mensaje del thread ya está persistido y el tool responde `ok` con `memory.captured=false` — nunca se pierde ni se bloquea el turno.
- **MEM-72 es la única superficie de diagnóstico:** no se añaden señales nuevas; se alimenta `queue`/`notice`.
- **Sin dependencias nuevas.** `unwrap`/`expect` prohibido en código nuevo (tests pueden).

## Deuda

- **`note_backlog` (review R2).** Publicar el backlog desde un enqueue usa `note_skipped_pass`, que además fija `last_pass = {skipped:true, at_ms}`. `queue`, `notice` y el warn son honestos; `last_pass.at_ms` pasa a significar "último enqueue" en un pipeline degradado. El fix correcto es un `observability::note_backlog(QueueSnapshot)` que actualice sólo `queue` + el warn (reutilizando la fn privada del warn, mismas señales): requiere editar `vanta-memory/src/ingest/**`, **prohibido** en esta tarea (recién tocado por MEM-72). Mientras tanto está documentado en `docs/api/MCP.md` § Thread writes feed memory. `docs/api/VANTA_MEMORY.md` (§ Operational modules) dice `last_pass: null` = "no pass ran"; necesita la misma nota cuando el carril MEM vuelva a tocar el archivo (hoy está modificado por otro carril en el árbol compartido).
- **Backlog `MCP-44` vs el contrato del plan (deltas declarados).** El contrato de esta tarea (plan §9.b + prompt del orquestador) es "turno con contenido → `memory_recall` sin `memory_put`"; la fila del Backlog pide además (a) *opt-in por env* y (b) un test de **doble entrega MCP + HTTP** con un solo registro L1. (a) **descartado a propósito**: el gate de `v0.10.0` es que un turno capturado por MCP sea memoria por defecto; un flag `off` por defecto reproduciría el bug. (b) cubierto **parcialmente** por idempotencia: el cursor L0 descarta la entrega repetida en la misma ventana de ms y el dedup L1 absorbe la de otro ms (review O1), pero **no hay test dedicado MCP+HTTP** → candidato a `FIND-*` / `MEM-73`.
- **Cascada de borrado (review R3).** `thread_delete`/`thread_purge_expired` borran el thread store, no `l0/<thread_id>` ni las tareas L1 encoladas: lo capturado sobrevive como memoria (documentado; cascada no implementada) → candidato a `FIND-*` / tarea de cascada.
- **Ventana de replay (review O1).** El cursor L0 filtra replays por `timestamp_ms`; un retry en otro milisegundo es un turno nuevo, y dos sends concurrentes idénticos en el mismo ms pueden encolar 2 tareas (el dedup L1 lo absorbe; L0 no pierde datos). Sin lock extra (ponytail).
- **Cola in-memory (review O2).** Sin backfill L0→cola tras un restart: las tareas encoladas viven lo que el proceso. Decisión de MCP-43.
- `thread_send` no registra el turno en L0 cuando el storage es read-only (el `send_message` ya falla antes): el orden de operaciones lo cubre.
- **Índice de docs:** `node scripts/docs/gen-index.mjs --check` ya estaba rojo antes de esta tarea (faltan en `docs/index.md` las filas de `FIND-733/734/736`, de otros carriles). Se revirtió la regeneración para no commitear entradas de task files que aún no están en el árbol; el índice se regenerará cuando esos carriles cierren.
- **NOTICED BUT NOT TOUCHING:** `docs/api/MCP.md` documenta `thread_get` como `{thread:{...}}` pero el código devuelve el thread pelado (`thread_to_json`). Drift pre-existente de MCP-32; fuera de scope → candidato a `FIND-*`.
- **NOTICED BUT NOT TOUCHING:** `.opencode/skills/vantadb-mcp/references/api-reference.md:184` sigue con `thread_send → {ok:true}` (incompleto, no falso). Ese árbol no está en el scope de escritura de MCP-44.

## Incógnitas (para el orquestador)

- **`observability::note_backlog(QueueSnapshot)`** — propuesto para `vanta-memory/src/ingest/observability.rs` (fuera de scope por la prohibición sobre `vanta-memory/src/ingest/**`): actualiza `queue` + emite el warn una vez por episodio, sin tocar `last_pass`. Borra la deuda R2 y deja `note_skipped_pass` significando lo suyo.
- **Arranque del runner en el host MCP** — para MCP-43: si el scheduler se hospeda en `--mcp`, anotar el runner al arrancar (`IngestRunnerCfg::from_env_toml` + `build_runner`) es preferible a la anotación perezosa del primer enqueue (`threads.rs::note_runner_state_once`), que hoy cubre el hueco sin tocar `server.rs`.

## Review (GATE P2-01 — agente distinto)

- Revisor: `vanta-review` (contexto fresco, una ronda)
- Enfoque: five-axis + RBI; verificación del contrato exacto y del camino de producción
- Cómo se probó: `cargo nextest run --profile audit -p vantadb-mcp -E 'test(thread)' --build-jobs 2` → 7/7 passed, 191 skipped; lectura de `observability`/`local_backend`/`l0_recorder`/`auto_recall`; `cargo fmt --check` OK
- Veredicto inicial: 🔴 `changes-required` — R1 (degradado no cableado en `--mcp`), R2 (`last_pass` fabricado), R3 (delete deja L0+cola), R4 (cierre DoD); O1–O4 opcionales

| Hallazgo | Resolución |
|---|---|
| **[Required R1]** Una sesión `--mcp` que sólo usa threads reportaba `runner:"unknown"` + `depth:0` con N turnos encolados (el verde inyectaba el estado) | `note_runner_state_once`: en el primer enqueue anota el runner con el mismo mapeo config→runner del host (`IngestRunnerCfg::from_env_toml` + `build_runner`, respeta un runner ya declarado). `--mcp` pasa a reportar `runner:"none"`, `depth>0`, `is_broken()` sin tocar `server.rs` |
| **[Required R2]** `note_skipped_pass` desde un enqueue fabricaba `last_pass` | Sin `note_backlog` disponible (archivo prohibido) → se acepta el cambio semántico y se documenta en MCP.md (más §Deuda/§Incógnitas con la propuesta de `note_backlog`) |
| **[Required R3]** `thread_delete`/`thread_purge_expired` no borran L0/L1 | Documentado en MCP.md (§ Thread writes feed memory + fila de `thread_delete`) + fila de deuda con la cascada como tarea |
| **[Required R4]** Task file sin cierre DoD | Este bloque: status `completed`, Steps ✅, §Verification con salidas exactas, §Review |
| **[Optional O1]** "un retry no puede duplicar memoria" sobrevendido | Comentario corregido: dentro de la ventana del cursor no encola; un retry en otro ms es un turno nuevo y el dedup L1 lo absorbe (+ deuda) |
| **[Optional O2]** "stay queued" no decía que la cola es in-memory | MCP.md: "for the lifetime of that process", sin backfill L0→cola |
| **[Optional O3]** No se probaba `degraded_warnings == 1` desde el nuevo writer | Asertado en ambos tests + `memory:{captured:false,queued:false}` para rol `system` |
| **[Optional O4]** `skills/vantadb-mcp/references/api-reference.md:184` stale | Fuera del scope de escritura → NOTICED BUT NOT TOUCHING |

## Resultado

`thread_send` dejó de ser sólo storage. Cada turno `user`/`assistant` se captura
en L0 (`l0/<thread_id>`, LLM-free, idempotente por cursor) y encola **una**
tarea L1 en la cola proceso-local del writer; `thread_delete` sigue siendo sólo
thread store (documentado). La respuesta es aditiva
(`{ok:true, memory:{captured,queued}}`), el contrato `{ok:true}` de MCP-32 sigue
intacto y los roles no conversacionales se persisten sin alimentar memoria. Un
pass (`run_pass`; MCP-43 en `--mcp`) convierte el turno en `l1/*` y
`memory_recall` lo devuelve sin `memory_put` — probado end-to-end. Con runner
`none` y cola no vacía, `capabilities.ingest` publica `degraded`, `queue.depth`
y `notice` (y `is_broken()` es `true`), y el proceso aprende su runner en el
primer enqueue: el trabajo encolado nunca se lee como "idle".
