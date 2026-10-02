---
title: "ADR-0054: Hogar del planificador de memoria L0→L3 (vantadb-server)"
kind: adr
status: accepted
description: "El planificador L0→L3 (dedup → escenas → persona + dream) no corre solo: los servicios son pull-based y ningún proceso los dispara. Decisión: el hogar es el wrapper vantadb-server"
tags: [vantadb, architecture, adr, vanta-memory, server]
created: "2026-10-01"
---

# ADR-0054: Hogar del planificador de memoria L0→L3 (vantadb-server)

> **Aceptado por delegación del owner (2026-10-01):** "Que vanta-arch decida con un ADR corto".
> Este ADR fija el host; las tareas derivadas (§ Tareas derivadas) las agenda el lead.

## Context

WIRE-01 cerró el loop de memoria del proxy (captura → recall, commit `679c75a9`) pero dejó
explícito un pendiente: **el planificador L0→L3** — el mecanismo que hace correr dedup →
escenas (L2) → persona (L3) + dream/consolidación — **no corre solo**; ningún proceso lo
dispara hoy ([WIRE-01](../../tasks/WIRE-01.md):36,136 — "scheduler host = MEM-55, fuera de scope").

Estado del código que encuadra la decisión (verificado en HEAD):

- Los servicios del pipeline son **pull-based por diseño** (MEM-16): `TimerScanner::run_once` —
  "the owner calls `run_once` whenever it wants due timers dispatched"
  (`vanta-memory/src/utils/timer_scanner.rs:1-7`); `PipelineWorker::run_once` / `reclaim_stale`
  (`vanta-memory/src/services/pipeline_worker.rs:232,251`); `AutoSyncScheduler::tick`
  (`vanta-memory/src/ingest/auto_sync.rs:4`). Cero drivers de fondo y cero callers productivos
  fuera de `vanta-memory` (evidencia consolidada en [FIND-113-spec](../../tasks/FIND-113-spec.md)).
- El puente de captura del server **existe y no está cableado**: `HttpCaptureBridge`
  (captura L0 + enqueue L1) + `run_bridge_pass`
  (`vanta-memory/src/services/conversation_hook.rs:36-107`), gancho `conversation_trigger`
  (`src/server/state.rs:96,134`), y `conversation_trigger: None` en el arranque
  (`src/server/bootstrap.rs:332`). Residual [MEM-55](../../tasks/complete/MEM-55.md):
  "el wiring (con runner configurado) es decisión de host post-release".
- El Scope Budget (DEF-07) ya listó al server como host prospecto: "Only core-adjacent role:
  prospective host of the `vanta-memory` scheduler (`src/server/bootstrap.rs:332`, WIRE-01) —
  promote that role via the inversion rule if it lands there"
  ([EXPERIMENTAL_FEATURES.md](../../../user/operations/EXPERIMENTAL_FEATURES.md):38); watchlist
  de promoción: "`vantadb-server`: the WIRE-01 scheduler-host decision" (`:52`); y `vanta-memory`
  figura como "Accelerate (partial — scheduler host pending, WIRE-01)" (`:33`).
- Identidad fijada por el owner (2026-10-01): **"Motor + plataforma completa"** — la plataforma
  (server) es parte del producto, no solo un lab congelado.

Restricciones técnicas que deciden:

1. **Single-writer por DB.** El lock fs2 + el split writer/proxy MCP-35 garantizan un solo
   proceso dueño de la DB; los demás proxyan. El scheduler debe correr en el **writer**.
2. **Estado del pipeline en RAM.** Cola, timers y locks viven en `LocalStateBackend`
   (`Mutex<Inner>`; invita `Arc` explícito — `vanta-memory/src/utils/local_backend.rs:36-42`);
   multi-proceso exigiría CAS sobre leases (techo documentado, `:236-237`). Un dueño por
   proceso; restart = cola vacía (semántica aceptada en FIND-113-spec).
3. **Ciclo Cargo.** `vanta-memory → vantadb` está prohibido (`conversation_hook.rs:4-7`):
   el core **no puede** alojar el pipeline, y `vanta-cli server` (bin del crate core,
   `src/cli.rs:337-347`) tampoco. El host debe ser un crate/binario externo que dependa de
   `vanta-memory`.
4. **Los hooks del terminal son one-shot.** `vanta-cli mcp-call` spawnea `vantadb-server --mcp`,
   hace un roundtrip JSON-RPC y muere ([WIRE-10](../../tasks/WIRE-10.md) Steps 3-4): no hay
   proceso vivo donde colgar timers/idle.

## Decision

**El hogar del planificador L0→L3 es `vantadb-server`** — el wrapper externo de plataforma.
V1 cablea el **modo HTTP** (el daemon, con el puente de conversaciones y el precedente de loop
de fondo); el **modo MCP del mismo binario** queda como segundo host natural, diferido — misma
decisión, misma crate, sin ADR nuevo.

### Por qué (criterio técnico)

1. **Es el único entry point de server que puede depender legalmente de `vanta-memory`**
   (restricción 3). Además ya la construye transitivamente:
   `vantadb-server → vantadb-mcp → vanta-memory/llm-driver` (`vantadb-server/Cargo.toml:27` ·
   `vantadb-mcp/Cargo.toml:27`) — el delta de build es ~0.
2. **Ya tiene runtime de daemon + precedente exacto de loop con shutdown graceful**: el TTL
   sweeper (`src/gc.rs:142-176`) spawneado en bootstrap (`src/server/bootstrap.rs:350-369`) y
   joineado al final (`:380-382`). El scheduler replica ese patrón (interval + `spawn_blocking`
   + watch/join).
3. **Ya tiene el productor**: el gancho `conversation_trigger` + `HttpCaptureBridge`
   (captura L0 + enqueue L1) — el residual MEM-55 se cierra con el mismo cambio.
4. **Es el proceso largo de la plataforma** — no session-scoped como el MCP, no opt-in como el
   proxy, no transitorio como el terminal — y es el host prospecto documentado con ruta de
   promoción (DEF-07).

### Forma (sin daemon nuevo en `vanta-memory`)

- La lógica sigue **pull-based** donde vive hoy: un *pass* = `TimerScanner::run_once`
  (dispara timers vencidos → encola L1/Flush/Dream) + `PipelineWorker::run_once` +
  `reclaim_stale`. Cero reimplementación (MEM-65/Dream se reusa tal cual).
- `vantadb-server` aporta el **driver**: loop de intervalo (configurable, `0` = off),
  `spawn_blocking` para el pass síncrono, shutdown graceful (join) — espejo del TTL sweeper.
- **Productor**: cablear el trigger en bootstrap con `HttpCaptureBridge` (captura L0 + enqueue
  L1); `run_bridge_pass` se generaliza al pass del driver.
- **Runner**: construido por pass desde env/TOML (patrón FIND-112 `build_ingest_runner`); sin
  runner → degrada P4 (skip observable, nunca bloquea). Secretos solo por env (R-5).
- **Regla de writer**: el scheduler corre en el proceso que posee la DB; el no-writer no lo
  ejecuta (sin doble scheduling).

### Interacción con el Scope Budget

La decisión **es** el trigger de promoción listado en la watchlist (DEF-07:52). Al aterrizar el
wiring: cambiar la fila de `vantadb-server` en EXPERIMENTAL_FEATURES.md — se promueve el
**rol "host del scheduler de `vanta-memory`"** (alcance acotado; no es un thaw general de labs).
La fila de `vanta-memory` pasa de "partial — scheduler host pending" a host resuelto cuando el
wiring esté verde.

## Alternatives Considered

### Proxy (`vanta-proxy`) — rechazada

- Pros: ya captura (`auto_capture` + `capture.rs`), tiene tokio y `vanta-memory`/`llm-driver`;
  el North Star se instrumenta ahí (DEF-05).
- Cons: **opt-in** — solo corre si el usuario rutea tráfico LLM por el gateway; **labs
  congelado** — el carve-out cubre capture/inject, no L2/L3/dream (un scheduler excede el
  carve-out y requeriría promoción sobre evidencia más débil); su camino de memoria es paralelo
  al del core (atar la consolidación al gateway duplicaría o bifurcaría el pipeline).
- Rechazada: el planner quedaría condicionado al uso del gateway; la política de scope no lo cubre.

### Terminal (`vanta-cli` / hooks) — rechazada

- Pros: los triggers de captura ya viven ahí (WIRE-10, 4 clientes).
- Cons: **procesos one-shot** (spawn → roundtrip → exit): no hay proceso vivo donde colgar
  timers/idle/dream; ejecutar el pipeline dentro del hook **bloquearía el turno del agente**
  (los hooks son síncronos en varios clientes); requeriría un daemon nuevo (prohibido por el
  precedente pull-based MEM-16 y FIND-113).
- Rechazada: la forma de ejecución es incompatible con un planificador por timers.

### MCP server (`vantadb-mcp` / `vantadb-server --mcp`) — rechazada como hogar v1; diferida como segundo host

- Pros: core-promise (ICP-01), es el **writer** en la topología MCP-35, ya depende de
  `vanta-memory`, y ahí viven las ops de memoria del agente (hooks/dream/escenas/gateway).
- Cons: **vida = sesión del agente** — muere al cerrar el IDE: sin consolidación idle/overnight;
  WIRE-01 ya defaulteó **NO** la tool MCP del scheduler ("scheduler host = MEM-55"); el diseño
  FIND-113 pull-based (tool invocada) reintroduce intervención manual.
- Rechazada como hogar v1. No contradice la decisión: es el **mismo binario** (`vantadb-server
  --mcp`), segundo host natural cuando exista demanda de consolidación intra-sesión — se decide
  con evidencia, no ahora.

### Daemon propio dentro de `vanta-memory` — rechazada

- Pros: un solo sitio.
- Cons: contradice el precedente pull-based MEM-16 ("el owner llama `run_once`") y el "daemon
  prohibido" de FIND-113; duplicaría el driver en cada embedder (desktop/proxy/MCP) y
  multiplicaría dueños de la cola.
- Rechazada: el driver pertenece al host, no al crate de lógica.

## Consequences

### Positivas

- El pipeline L0→L3 corre solo en el proceso de plataforma: cierra el residual MEM-55 y el
  "scheduler host pending" del Scope Budget.
- Reutiliza todo lo existente: puente MEM-55, pass pull-based MEM-16, patrón loop/shutdown del
  TTL sweeper, runner FIND-112 — **cero lógica de pipeline nueva**.
- Un solo dueño por DB (writer): sin doble procesamiento; cola efímera con semántica ya aceptada
  (re-encolable desde capturas persistidas).
- Cero threads nuevos en `vanta-memory` (sigue pull-based); el driver vive donde ya hay tokio.

### Negativas / deuda asumida

- `vantadb-server` es **labs**: el rol se promueve (row change en EXPERIMENTAL_FEATURES.md); el
  thaw es acotado al rol, no general.
- **MCP-only (ICP-01) y proxy-only no obtienen el planner en v1** — limitación declarada
  (segundo host diferido / carve-out). El recall no se bloquea; la consolidación queda pendiente
  para esos deployments.
- **Asimetría entre entry points de server**: `vanta-cli server --http` (bin core) no puede
  alojarlo (ciclo Cargo) → se documenta; el host canónico es `vantadb-server`.
- Single-writer: si corren server + MCP, solo el writer ejecuta el scheduler (el otro ya no abre
  la DB — comportamiento existente, a documentar).
- Build/Fast Gate: `vantadb-server` está en default-members; el delta de build es ~0
  (`vanta-memory` ya está en el grafo) pero el feature-gate nuevo debe cuidar el presupuesto
  <5min (ADR-0031).

## Tareas derivadas (propuestas — el lead crea las filas)

| # | Alcance | Crate | Verificación |
|---|---------|-------|--------------|
| T1 | Seam de host aditivo en el arranque: inyección de `conversation_trigger` + servicio de fondo (trait + punto de wiring en bootstrap), arranque/parada graceful | `vantadb` (core) | `cargo test -p vantadb --features server` + test de servicio fake spawneado/joineado |
| T2 | Servicio scheduler: `run_pass` (timers + worker + reclaim) + loop helper con shutdown (espejo `MemoryTtlSweeper`), feature-gated; runner por pass (FIND-112), degrada P4 | `vanta-memory` | `cargo test -p vanta-memory --features http-server` (pass procesa, timers disparan, shutdown joinea, sin runner degrada) |
| T3 | Wiring del wrapper: dep `vanta-memory`, construcción queue + bridge + loop, env config (`VANTADB_SCHEDULER_*`, R-5), shutdown | `vantadb-server` | e2e `POST /conversation/add` → L0 → pass → `l1/<session>`; restart; disabled; sin runner |
| T4 | Docs: `VANTA_MEMORY.md` §Operational modules (wired status), `EXPERIMENTAL_FEATURES.md` (promoción del rol + fila `vanta-memory`) | `docs/` | `node scripts/docs/check-docs.mjs` + `check-links.mjs` |
| T5 | Verificación adversarial: crash mid-pass + restart (cola efímera), review P2-01 | `vanta-chaos` / `vanta-review` | chaos test + verdict |

## References

- Decisión delegada: owner 2026-10-01 ("Que vanta-arch decida con un ADR corto").
- Tarea origen: [WIRE-01](../../tasks/WIRE-01.md) (commit `679c75a9`) · residual
  [MEM-55](../../tasks/complete/MEM-55.md) · diseño previo del owner-scheduler MCP:
  [FIND-113-spec](../../tasks/FIND-113-spec.md).
- Scope Budget / identidad: [EXPERIMENTAL_FEATURES.md](../../../user/operations/EXPERIMENTAL_FEATURES.md):22-52
  · master roadmap `docs/dev/plans/2026-09-26-master-roadmap.md`.
- Contrato del crate: [VANTA_MEMORY.md](../../../api/VANTA_MEMORY.md) §Operational modules.
- Precedentes: [ADR-0029](./ADR-0029-vanta-memory-context-engine.md) (context engine) ·
  [ADR-0040](./ADR-0040-auto-consolidate-local-first.md) (auto-consolidación local-first) ·
  [ADR-0031](./ADR-0031-default-members-promotion.md) (labs fuera de Fast Gate).
- Verificados sin interacción con esta decisión: [ADR-0046](./ADR-0046-schema-v2-migracion-unica.md)
  (schema v2 — no toca el host del pipeline) · [ADR-0039](./ADR-0039-jwt-hs256-offline.md)
  (auth del server — ortogonal).
- Código citado: `vanta-memory/src/utils/timer_scanner.rs:1-7` ·
  `vanta-memory/src/services/pipeline_worker.rs:232,251` ·
  `vanta-memory/src/utils/local_backend.rs:36-42,236-237` ·
  `vanta-memory/src/services/conversation_hook.rs:36-107` · `src/server/state.rs:96,134` ·
  `src/server/bootstrap.rs:332,350-369,380-382` · `src/gc.rs:142-176` ·
  `vantadb-server/Cargo.toml:27` · `vantadb-mcp/Cargo.toml:27` · `src/cli.rs:337-347`.
