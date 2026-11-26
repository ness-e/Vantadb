---
title: "MCP-43: el proceso --mcp hospeda el scheduler de memoria (drena la cola de threads)"
kind: task
status: completed
description: "El `--mcp` writer arranca `vanta_memory::services::scheduler` apuntado a `threads::memory_queue()` y lo detiene antes del flush; el modo proxy no arranca nada y lo declara. Sin backfill L0→cola (techo declarado con fila derivada)."
---

# MCP-43: el `--mcp` writer drena la cola de memoria

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§9.b, ola MCP/MEM)
- **Fuente:** Backlog `MCP-43` — `thread_send` captura L0 y encola L1 (MCP-44), pero nadie drena la cola: la memoria se captura y no se consolida nunca
- **Esfuerzo:** 🟡 2-3d (appetite 1-2d)
- **Tipo:** Rust (feature-add: wiring nuevo en `vantadb-mcp`, sin símbolos públicos nuevos en el core)
- **Estado:** ✅ COMPLETED
- **Creado:** 2026-11-25

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-server/src/main.rs:72` (`--mcp` → `run_stdio_server_auto`), `vantadb-mcp/src/proxy.rs::promote` (MCP-48: proxy que se re-elige writer), `vantadb-mcp/tests/*` |
| Callees | `vanta_memory::services::scheduler::{spawn_memory_scheduler, MemoryScheduler, run_pass}` (WIRE-15), `vanta_memory::ingest::runner_config::{IngestRunnerCfg, ConcreteRunner, build_ingest_runner}`, `vanta_memory::ingest::observability::{note_scheduler_started/stopped}` (MEM-72), `vantadb_mcp::threads::{memory_queue, ingest_runner_cfg}` (MCP-44, `pub(crate)` a propósito) |
| Implicaciones | **Cambia el ciclo de vida del proceso MCP**: un loop Tokio en background que debe parar antes del `flush` y del drop del `DiscoveryFile`. En modo proxy no existe (la cola es del writer remoto). En modo promovido (MCP-48) sí existe, porque ese proceso pasa a ser el writer. Sin migración de datos; la cola sigue siendo in-memory (ADR-0054). Feature `http-server` de `vanta-memory` añadida a `vantadb-mcp` (Tokio ya presente; `vantadb/server` ya activo). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o en la zona editada):** `vantadb-server/src/main.rs` (181L), `vantadb-server/src/scheduler.rs` (192L, el wiring HTTP de referencia), `vantadb-mcp/src/threads.rs` (L1-80, 152-165, 218-377, 435-632), `vantadb-mcp/src/server.rs` (L855-1117: `serve_lines_proxy`, `run_proxy_stdio_server`, `run_stdio_server_auto`), `vantadb-mcp/src/proxy.rs` (L1-200, 240-720: `WriterProxy`, `promote`, `flush_if_promoted`), `vantadb-mcp/src/lib.rs` (76L), `vantadb-mcp/Cargo.toml` (54L), `vanta-memory/Cargo.toml` (104L), `vanta-memory/src/services/scheduler.rs` (399L), `vanta-memory/src/services/mod.rs` (16L), `vanta-memory/src/ingest/observability.rs` (325L), `vanta-memory/src/ingest/runner_config.rs` (L85-304), `vantadb-mcp/tests/thread_memory_pipeline.rs` (114L), `vantadb-mcp/tests/capabilities_ingest.rs` (71L), `docs/api/MCP.md` (L448-500, 585-664), `docs/api/VANTA_MEMORY.md` (L284-394), `docs/dev/tasks/MCP-44.md` (131L), `docs/dev/tasks/MEM-72.md` (§Techo).
- **Referencias hacia dentro:** `run_stdio_server_auto` ← `vantadb-server/src/main.rs:72` (único arranque de `--mcp`); `run_proxy_stdio_server` ← mismo `main.rs` (rama DatabaseBusy) y tests; `WriterProxy::promote` ← `resolve_with_backoff` (L539) y tests de MCP-48.
- **Referencias entrantes de lo editado:** el contrato de `capabilities.ingest` (`docs/api/MCP.md:459-489`, `vantadb-mcp/tests/capabilities_ingest.rs`) y el contrato de `thread_send` (MCP-44). Ninguna firma pública cambia: `start`/`stop` son `pub(crate)`.
- **Veredicto impacto:** MEDIO-ALTO. Un módulo nuevo (`memory_scheduler.rs`, ~120L), dos llamadas de arranque (writer + promoción) y dos de parada (antes de cada `flush`). Pre-mortem: (a) loop que no para y rompe el shutdown → `stop().await` (señal + join) **antes** del flush, y `Drop` de `MemoryScheduler` como red de seguridad; (b) scheduler local en modo proxy → no se llama en la rama proxy, y la promoción sí lo arranca porque ese proceso pasa a ser el writer; (c) backfill que re-encola todo → **no hay backfill** (decisión §Spec-6); (d) binario stale → la verificación es `cargo nextest -p vantadb-mcp` (compila del fuente, no usa binario de `target/`; `FIND-741` no aplica).

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Dónde se arranca el loop | A) en `run_stdio_server_auto` (rama writer) + en `WriterProxy::promote`, con un handle proceso-global idempotente / B) dentro de `run_stdio_server` (se perdería la promoción y el arranque quedaría a un `await` de distancia del open) / C) en el binario `vantadb-server` (no ve la cola de `vantadb-mcp`: `memory_queue()` es `pub(crate)`) | ✅ A — evidencia: `threads::memory_queue` es crate-private a propósito (MCP-44 §Spec-1) y la promoción (MCP-48) convierte a este proceso en writer, así que también debe drenar |
| 2 | Qué cola | A) `threads::memory_queue()` (la MISMA instancia que llena `thread_send`) / B) una cola nueva del scheduler (dos colas: nunca drenaría la real) | ✅ A — es literalmente el contrato de MCP-44 §1a |
| 3 | Runner por pass | A) `IngestRunnerCfg::from_env_toml` + `build_ingest_runner`, mapeando `Some(ConcreteRunner::None)` → `None` (el factory del host HTTP, `vantadb-server/src/scheduler.rs:108-115`) / B) pasar `ConcreteRunner::None` al worker (cada tarea quemaría retry/dead-letter con `NotConfigured`) | ✅ A — P4: sin engine el pass se salta y la cola queda intacta; el skip ya es observable (MEM-72) |
| 4 | Intervalo | A) `VANTADB_SCHEDULER_INTERVAL_MS` (misma variable que el host HTTP), default **5000 ms** para `--mcp` / B) default 60000 ms igual que el HTTP host / C) variable nueva | ✅ A — un turno capturado debe ser recallable en el turno siguiente del agente, no 60 s después; un pass con la cola vacía es un no-op (timer scan + worker vacío + reclaim); `0` sigue significando off, explícito. Variable única: dos nombres para el mismo knob es la clase de deriva que este plan combate |
| 5 | Parada | A) `stop().await` (watch + join) antes de `storage.flush()` en ambas ramas writer y antes del flush del promovido / B) sólo `Drop` (aborta: el pass puede estar a mitad de claim y el `note_scheduler_stopped` del loop se salta) | ✅ A — el orden importa: parar el loop → flush → soltar el lock. `Drop` queda como red de seguridad para caminos de salida tempranos |
| 6 | **Backfill tras restart** | (a) re-escanear `l0/` y re-encolar lo no consolidado — requiere un **marcador de consolidación durable** en el camino L0 (`vanta-memory/src/core/conversation/**`), **fuera del scope de escritura de esta tarea**; (b) no hacer backfill, declararlo como techo + fila derivada; (c) backfill acotado a N/mtime — sin el marcador de (a) degenera en re-encolar sesiones ya consolidadas en **cada** arranque (pre-mortem c: duplicación masiva de llamadas LLM) | ✅ **B** — la cola es in-memory por diseño (ADR-0054) y un backfill correcto exige un marcador durable fuera de scope; (c) sin marcador es peor que nada. Techo declarado: los turnos capturados en L0 que murieron antes de un pass **no** se re-encolan tras un restart; el L0 y el thread store siguen ahí (no se pierde contenido, se difiere la extracción). Fila derivada obligatoria → §Deuda |
| 7 | `scheduler_running` en modo proxy | A) no se arranca nada y el proxy lo declara en el log de arranque (`reason: proxy`), mientras `capabilities` (reenviado al writer) reporta el estado **del writer** / B) un indicador local inventado en el proxy (mentiría: la cola es del writer) | ✅ A — `observability` es proceso-local (MEM-72 §Techo) y `tools/call` se reenvía entero (MCP-48): el cliente siempre lee el proceso que posee la cola |

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | `memory_scheduler.rs`: handle proceso-global idempotente (`start`/`stop`), intervalo por env, factory de runner P4 | ✅ |
| 2 | Arranque en la rama writer de `run_stdio_server_auto` (×2) y en `WriterProxy::promote`; parada antes de cada flush; declaración explícita en la rama proxy | ✅ |
| 3 | RED→GREEN: test del ciclo real turno → tick del timer → `memory_recall` (y `scheduler_running` veraz antes/después de `stop`) | ✅ |
| 4 | Docs: `docs/api/MCP.md` (§ ingest, § Thread writes feed memory) + `docs/api/VANTA_MEMORY.md` (§ scheduler / observability) | ✅ |
| 5 | Verify contrato + OCR + review P2-01 + commit local | ✅ |

## Verification

| Comando | Resultado |
|---|---|
| `cargo nextest run --profile audit -p vantadb-mcp -E 'test(scheduler) or test(mcp_scheduler) or test(thread)' --build-jobs 2` | ✅ 9/9 pass, 191 skipped (8.7 s) |
| `cargo nextest run --profile audit -p vantadb-mcp --build-jobs 2` (suite completa) | ✅ 200/200 pass (105.8 s) |
| `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` | ✅ 747 pass, 2 skipped (15.9 s) |
| `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` | ✅ limpio |
| `cargo clippy -p vanta-memory --all-targets -- -D warnings` | ✅ limpio |
| `cargo fmt -p vantadb-mcp -p vanta-memory -- --check` | ✅ limpio |
| `node scripts/docs/check-links.mjs` / `check-docs.mjs` | ✅ exit 0 (1 wikilink presupuestado en un snapshot histórico, informativo) |
| `node scripts/docs/gen-index.mjs --check` | ❌ **rojo pre-existente de otros carriles** (`docs/index.md` está dirty por `FIND-741`; faltan filas de `FIND-743/744`). No se regenera: arrastraría trabajo ajeno al commit (misma decisión que MCP-44) |
| `npx markdownlint-cli2 --config .github/markdownlint-cli2.yaml docs/api/MCP.md docs/api/VANTA_MEMORY.md` | ✅ 0 issues |
| `pwsh dev-tools/ocr-review.ps1` + `ocr delegate rule` | ✅ aplicado (rule groups Cargo + server-mcp sobre los 5 archivos Rust/manifest; sin Critical/High propios) |
| Repetición anti-flake del test nuevo | ✅ 3 corridas consecutivas verdes (1.7-2.8 s) |

**DoD de la ola (el ciclo real, con número y tiempo):** `memory_scheduler::tests::scheduler_tick_consolidates_a_captured_turn_and_stop_clears_the_flag` — `thread_send` captura (`{captured:true,queued:true}`, profundidad 1) → `memory_recall` devuelve `[]` (la cola no es memoria) → arranca el loop (intervalo 25 ms) → **el tick del intervalo** consolida → `memory_recall` devuelve el turno con «fjall» **sin `memory_put`**; el test completo corre en **1.7-2.8 s** (≈1 tick de 25 ms hasta el efecto) → `capabilities.ingest.scheduler_running == true` → `stop()` → `false`. `run_pass` **no** se invoca a mano en ningún punto: si el ticker no disparara, el test falla.

## Invariantes de dominio (handoff — MUST)

- **Un loop por proceso.** `start()` es idempotente: un segundo arranque (p. ej. promoción tras un arranque writer fallido) no crea un segundo consumidor.
- **El loop para antes del flush** y antes de soltar el lock/discovery file. Nunca un scheduler vivo sobre storage flusheado.
- **Modo proxy: cero scheduler.** La cola es del writer; un consumidor local sería un segundo consumidor sobre datos ajenos.
- **No backfill** (§Spec-6). Re-encolar en cada arranque es duplicación masiva.
- **`scheduler_running` no miente:** `true` sólo con un loop vivo; `stop`/`Drop` lo limpian (MEM-72).
- Sin dependencias nuevas; `unwrap`/`expect` prohibido en código nuevo (tests pueden).

## Deuda

- **Backfill L0→cola tras restart: NO se hace (decisión §Spec-6), con techo declarado.** La cola es in-memory (ADR-0054), así que un turno capturado en `l0/<thread>` que muere antes de un pass **no se re-encola** al arrancar de nuevo; el contenido sobrevive (thread store + L0), lo que se difiere es la extracción automática. Un backfill correcto exige un **marcador durable de consolidación** en el camino L0 (`vanta-memory/src/core/conversation/**`), fuera del scope de escritura de esta tarea; sin ese marcador, la variante acotada por N/mtime re-encola en **cada** arranque turnos ya extraídos (duplicación masiva de llamadas LLM — pre-mortem (c)). **Fila derivada obligatoria** (para `docs/dev/Backlog.md`, que esta tarea no puede tocar; el orquestador asigna el número — propuesto `FIND-745`):

  `| FIND-745 (a confirmar) | Backfill L0→cola L1 tras restart: la cola es in-memory (ADR-0054), así que un turno capturado que muere antes de un pass no se re-extrae nunca; falta un marcador durable de consolidación para re-encolar sólo lo no consolidado | vanta-worker | R | ~1d | matar `--mcp` entre `thread_send` y el pass → un `--mcp` nuevo consolida ese turno y `memory_recall` lo devuelve sin `memory_put`, sin duplicar la extracción de turnos ya consolidados | MCP-43 |`

- **`note_backlog` sigue pendiente (deuda R2 de MCP-44).** `thread_send` publica el backlog con `note_skipped_pass`, que además fija `last_pass = {skipped:true, at_ms}`. Con el loop vivo ese valor se sobrescribe en el siguiente tick (≤ intervalo), así que la ventana de lectura falsa se acorta, pero el productor sigue fabricando un `last_pass`. El fix correcto (`observability::note_backlog(QueueSnapshot)`, actualiza sólo `queue` + el warn) cae en `vanta-memory/src/ingest/**`, que esta tarea podía tocar sólo para un punto de entrada aditivo: se deja para no mezclar contratos.
- **`VANTADB_SCHEDULER_INTERVAL_MS` tiene dos defaults por host** (`60000` en `--http`, `5000` en `--mcp`) leídos por dos crates distintos (`vantadb-server/src/scheduler.rs` y `vantadb-mcp/src/memory_scheduler.rs`) porque `vantadb-server` depende de `vantadb-mcp` y no al revés. Misma duplicación declarada que `ingest_toml_path` (MCP-44): un nombre, un significado, dos lectores que deben mantenerse en sync.
- **`docs/index.md` / `gen-index --check` rojo por otros carriles** (`docs/index.md` dirty con `FIND-741`; faltan `FIND-743/744`). No se regenera para no commitear trabajo ajeno.
- **Corrección de doc incluida:** `docs/api/MCP.md` afirmaba (desde MEM-72) que un proxy ve «su propia cola y `last_pass: null`». Es falso: `dispatch_request_proxy` reenvía `tools/call` entero, así que `capabilities` en un proxy responde con el estado del **writer**. Reescrito en esta tarea (§ ingest); el snapshot sigue siendo proceso-local, lo que cambia es qué proceso contesta.
- **Arranque/parada: techo del join.** Un pass ya entregado a `spawn_blocking` no se puede cancelar (MOD-11), así que `stop()` espera el pass en vuelo — acotado por el timeout del runner, no por el nuestro. Es el mismo techo que el host HTTP (WIRE-14/16) y quedó documentado en el doc-comment de `stop()`; `Drop` sigue siendo la red para salidas tempranas.
- **NOTICED BUT NOT TOUCHING:** `docs/api/MCP.md:629-632` sigue explicando que en un pipeline degradado `last_pass.skipped:true` puede significar «se encoló un turno» en vez de «corrió un tick». Cierto hasta que el loop está vivo; con `--mcp` + loop, el tick sobrescribe. Se deja tal cual porque describe el camino degradado (`runner: none`), que es donde el productor del enqueue sigue siendo el último escritor cuando no hay pass real.
- **NOTICED BUT NOT TOUCHING:** `.opencode/skills/vantadb-mcp/references/api-reference.md` (árbol fuera de scope) no menciona el scheduler; no es falso, es incompleto.

## Review (GATE P2-01 — agente distinto)

- Revisor: `vanta-review` (contexto fresco, dos rondas sobre el mismo diff)
- Enfoque: five-axis + orden de shutdown/single-consumer + veracidad de `scheduler_running` + calidad del test del tick
- Cómo se probó: lectura de `server.rs::run_stdio_server` (`_writer_guard`), `proxy.rs::{promote, flush_if_promoted}`, `vanta-memory/src/services/scheduler.rs` (`Drop`, `note_scheduler_*`), `ingest/observability.rs`; contrato re-corrido tras los fixes (9/9 ×3) + suite completa (200/200)

| Hallazgo | Resolución |
|---|---|
| **[Blocking R1]** `stop()` vivía en el caller: `_writer_guard` (discovery file) dropea al volver de `run_stdio_server`, así que se anunciaba "no writer" con el loop vivo y posiblemente a mitad de pass | `stop().await` movido al **final de `run_stdio_server`** (antes del Drop del guard). Los dos `stop()` del caller eliminados y sustituidos por comentarios de orden. Idempotente: sin loop arrancado es no-op |
| **[Blocking R2]** En `promote()`, `start()` corría antes de `ProxyHandle::for_local_port(...)?`; si ese `?` devolvía `None`, quedaba un loop huérfano con el lock liberado y `link.promoted` sin setear (nadie lo pararía **ni flushearía**) | Reordenado: handle primero, `start` después. Sin handle no hay promoción, ni loop, ni storage que drenar |
| **[Medium R3]** La rama "listener muerto, reiniciarlo" no arrancaba loop (funcionaba sólo por la suposición de que la primera promoción ya lo hizo) | `start` idempotente añadido tras el handle: el invariante queda pinneado, no asumido |
| **[Medium R4]** `stop()` vaciaba el slot antes del await: un `start` concurrente en esa ventana creaba un segundo loop | `Stopping` (`AtomicBool`): se setea antes de tomar el slot, se limpia tras el join; `start_with` lo re-chequea **bajo** el lock del slot |
| **[Low R5]** Los docs no decían que un writer `read_only` no arranca loop | Cláusula añadida en `MCP.md` § ingest y `VANTA_MEMORY.md` § Operational modules, junto con el orden parada→flush→drop del discovery |
| **[Low R6]** Los tests dependen de estado proceso-global (env + cola) | Declarado: el contrato es `cargo nextest` (un proceso por test); bajo `cargo test` en el mismo binario podrían interferir. Sin cambio |
| **[Nit R7]** `timer_pass` sobrevendía lo probado (`TimerScanner`/`l1_idle` no disparan acá) | Renombrado `scheduler_tick_consolidates_...` + doc-comment corregido: lo probado es el **ticker del intervalo** drenando la tarea L1 que encoló `thread_send` |

**Veredicto ronda 1:** `changes-required` (2 blocking + 2 Medium + 2 Low + 1 nit) → **todos resueltos** en esta ronda.
**Re-verdicto ronda 2 (mismo reviewer, sobre el estado final): `approve`.** Verificó el orden end-to-end en los cuatro caminos (writer `server.rs:466-471` → guard `:396` → flush `:1071`; stale-retry `:1121`/`:1125`; promovido vía `flush_if_promoted`; proxy plano sin `promote` exitoso) y que `STOPPING` cierra la ventana de #4 (flag antes del `take`, re-chequeo **bajo** el lock del slot).
Residual declarado (Low, sin acción): dos `stop()` **concurrentes** podrían dejar que un `start` se cuele a mitad del join (hoy inalcanzable: los caminos writer y promovido son mutuamente excluyentes); el `stop` de `run_stdio_server` corre mientras el listener de `spawn_writer_http` puede aceptar requests (lifetime pre-existente de MCP-35, fuera de scope).

## Resultado

El proceso `--mcp` ya no captura y abandona: en modo **writer** hospeda el
scheduler de `vanta-memory` sobre **la misma** cola que llena `thread_send`
(`threads::memory_queue()`), y en modo **proxy** no arranca nada y lo declara.
`vantadb-mcp/src/memory_scheduler.rs` (nuevo, ~150L) es el único driver: `start`
idempotente (`OnceLock<Mutex<Option<MemoryScheduler>>>` + flag `Stopping`),
intervalo por `VANTADB_SCHEDULER_INTERVAL_MS` (default **5000 ms**, `0` = off),
runner por pass desde la misma superficie env/TOML del host HTTP con
`Some(ConcreteRunner::None) → None` (P4: sin engine el pass se salta y la cola
queda intacta), y `stop()` que señala + joinea. Arranca en las dos ramas writer de
`run_stdio_server_auto` (no en `read_only`) y en `WriterProxy::promote` cuando el
proxy se re-elige writer (MCP-48); para al final de `run_stdio_server`, **antes**
de soltar el discovery file y del `flush`, y en `flush_if_promoted` antes del
flush del promovido. `scheduler_running` se setea en `start()` (no en el primer
tick) y se limpia al salir del loop y en `Drop` (MEM-72), así que no miente en
ninguna rama.

Probado con el ciclo real: `thread_send` → **tick del interval ticker** →
`memory_recall` devuelve el turno sin `memory_put` (1.7-2.8 s, 3/3 corridas sin
flake); 200/200 de la suite del crate; 747 de `vanta-memory`. Review P2-01:
ronda 1 `changes-required` (2 blocking sobre el orden de parada y sobre un loop
huérfano en la promoción) → corregidos → ronda 2 `approve`.

**Techo (decisión §Spec-6):** sin backfill L0→cola tras restart — fila derivada
en §Deuda.
