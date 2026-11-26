---
title: "FIND-748: queue.depth es un probe vivo y la salud del ingest es de cuatro estados"
kind: task
status: pending
description: "Implementar ADR-0065: capabilities.ingest.queue se lee en el momento de responder (o null), last_pass.queue conserva la muestra del pass, y health {unknown|idle|draining|broken} reemplaza la condición que mentía en dirección optimista"
---

# FIND-748: `queue.depth` es un probe vivo y la salud del ingest es de cuatro estados

## Metadata
- **Decisión:** [ADR-0065](../architecture/adr/ADR-0065-ingest-queue-live-probe-health.md)
  (`status: accepted`, `vanta-arch`, 2026-11-25)
- **Origen:** `MEM-73` (commit `c3e9c098e`), hallazgo 4 · fila `docs/dev/Backlog.md:836`
- **Tipo:** corrección de contrato en `vanta-memory` + `vantadb-mcp` + docs de API
- **Estado:** 🟡 Pendiente
- **Dueño de la implementación:** `vanta-worker` (no `vanta-arch`: esta tarea no diseña)
- **Review:** `vanta-review` (P2-01) antes de `COMPLETED`

## El defecto, en una frase

`capabilities.ingest.queue.depth` se escribe **sólo** desde resultados de pass
(`observability.rs:184,202`), así que en el camino sano es la foto del pass anterior;
`is_broken()` (`:131-133`) se define sobre esa foto, y por eso un pipeline con trabajo
encolado puede reportar `false` en tres situaciones distintas: (1) turno recién
encolado, (2) scheduler nunca arrancado o ya muerto, (3) proceso que no ve ninguna cola.

## Alcance de archivos (cerrado)

| Archivo | Cambio |
|---|---|
| `vanta-memory/src/ingest/observability.rs` | D1-D7: tipos, derivación, `status_with_queue`, `note_backlog` |
| `vanta-memory/src/utils/local_backend.rs` | `queue_occupancy()` — una adquisición, sin lectura desgarrada |
| `vantadb-mcp/src/threads.rs` | `publish_backlog_if_degraded` → `note_backlog`; helper de snapshot |
| `vantadb-mcp/src/handlers/tools.rs` | `out["ingest"]` usa `status_with_queue(Some(...))` |
| `vantadb-mcp/tests/*`, `vanta-memory/tests/ingest_observability.rs` | contrato mecánico abajo |
| `docs/api/MCP.md`, `docs/api/VANTA_MEMORY.md` | misma PR, regla 3 de `AGENTS.md` |

**Fuera de alcance (declarado):** `.opencode/plugins/vanta-memory-capture.ts` (repo
aparte, base `2296ccd`), el host HTTP (`vantadb-server` no publica `ingest` hoy:
`rg observability vantadb-server/src` = 0 hits), y la vista multi-proceso (ADR-0065 § D8).

## Contrato mecánico

### C1. `observability.rs` — tipos

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IngestHealth { #[default] Unknown, Idle, Draining, Broken }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DrainPath { #[default] None, Loop, OnDemand }   // "none" | "loop" | "on_demand"

// PassSnapshot gana la muestra del pass; la foto vive donde su `at_ms` la fecha.
pub struct PassSnapshot { pub at_ms: u64, pub skipped: bool, pub processed: usize,
                          pub failed: usize, pub timers_fired: usize,
                          pub queue: QueueSnapshot }      // NUEVO

pub struct IngestStatus {
    pub runner: RunnerState,
    pub degraded: bool,
    pub scheduler_running: bool,
    pub scheduler_ever_started: bool,        // NUEVO: proceso-vida
    pub queue: Option<QueueSnapshot>,        // CAMBIA: vivo o null, nunca viejo
    pub last_pass: Option<PassSnapshot>,
    pub health: IngestHealth,                // NUEVO: derivado al publicar
    pub drain_path: DrainPath,               // NUEVO: derivado al publicar
    pub degraded_warnings: u32,
    pub notice: Option<String>,
}
```

`QueueSnapshot { depth, claimed }` no cambia. Los dos enums nuevos llevan
`#[non_exhaustive]`; `IngestStatus` **no** (evita romper construcción por literales
fuera del crate más de lo necesario).

### C2. `observability.rs` — derivación (privada) y API

```rust
fn derived_drain_path(s: &IngestStatus) -> DrainPath {
    if s.degraded { return DrainPath::None; }
    if s.scheduler_running { return DrainPath::Loop; }
    if !s.scheduler_ever_started && s.last_pass.is_some() { return DrainPath::OnDemand; }
    DrainPath::None
}

fn derived_health(s: &IngestStatus) -> IngestHealth {
    let Some(q) = s.queue else { return IngestHealth::Unknown };       // 1. sin lectura viva
    if s.runner == RunnerState::Unknown && !s.scheduler_running
        && s.last_pass.is_none() { return IngestHealth::Unknown }      // 2. sin evidencia de hospedar
    if q.depth > 0 && s.drain_path == DrainPath::None { return IngestHealth::Broken }
    if q.depth > 0 { return IngestHealth::Draining }
    IngestHealth::Idle
}

pub fn status() -> IngestStatus { status_with_queue(None) }            // firma intacta
pub fn status_with_queue(queue: Option<QueueSnapshot>) -> IngestStatus  // NUEVA

impl IngestStatus {
    pub fn is_broken(&self) -> bool { self.health == IngestHealth::Broken }
}
```

Reglas de implementación:

- `status_with_queue` escribe `queue`, luego `drain_path`, luego `health`, luego
  `notice` — en ese orden, sobre el clon (igual que `notice` hoy, `:241-243`).
- `note_scheduler_started()` pone `scheduler_running = true` **y**
  `scheduler_ever_started = true`. `note_scheduler_stopped()` sólo baja el primero.
- El campo `State.status.queue` se **elimina** (no se deprecia): `note_pass` y
  `note_skipped_pass` guardan lo que reciben en `last_pass.queue`.
- `pub fn note_backlog(queue: QueueSnapshot)` — sólo el latch de warn («una vez por
  episodio», resetea con `depth == 0`). `note_skipped_pass` = grabar `last_pass`
  + `note_backlog`. Ningún write de `last_pass` desde un enqueue (ADR-0065 § D7).
- `notice()` gana dos ramas y **no** puede decir "runner is none" cuando el runner
  está sano: `broken && degraded` → texto actual; `broken && !degraded` → menciona
  que hay trabajo encolado y que ningún scheduler lo drena (`drain_path: none`);
  `unknown` → nombra cuál de las dos causas (sin lectura viva / sin evidencia de
  hospedar el pipeline). Mantener `degraded`-idle y `runner: unknown` como hoy.

### C3. `local_backend.rs` — una lectura, no dos

```rust
/// `(queued, in_flight)` en una sola adquisición: el par no puede desgarrarse.
pub fn queue_occupancy(&self) -> (usize, usize) {
    let inner = self.lock();
    (inner.queue.len(), inner.pending.len())
}
```

Reemplaza las secuencias `queue_depth()` + `pending_count()` de los tres call sites
(`scheduler.rs:138-146`, `scheduler.rs:363-367`, `threads.rs:373-377`).
`queue_depth()` sigue existiendo (lo usan los tests de prioridad) pero **no** se usa
para publicar el snapshot: separa por prioridad con un scan O(n) y se lee en dos
pasos. Nota `ponytail:` obligatoria en el método nuevo: *"one lock, O(1) by
`Vec::len`/`HashMap::len`; if a profile ever shows it, this is already the cheap version."*

### C4. `vantadb-mcp` — leer la cola que el proceso ya tiene

```rust
// threads.rs: junto a memory_queue()
pub(crate) fn queue_snapshot() -> QueueSnapshot {
    let (depth, claimed) = memory_queue().queue_occupancy();
    QueueSnapshot { depth, claimed }
}

// threads.rs: publish_backlog_if_degraded -> sin fabricar un pass
fn publish_backlog_if_degraded(queue: &LocalStateBackend<SystemClock>) {
    if !observability::status().degraded { return; }
    let (depth, claimed) = queue.queue_occupancy();
    observability::note_backlog(QueueSnapshot { depth, claimed });
}

// handlers/tools.rs:3198
out["ingest"] = json!(observability::status_with_queue(Some(crate::threads::queue_snapshot())));
```

## Aserciones exactas (el test ES el contrato)

`vanta-memory/tests/ingest_observability.rs` sigue siendo **un solo** `#[test]`
(el registro es proceso-global; `observability.rs:38-40`). Las fases nuevas van en
este orden **obligatorio**, porque el estado se acumula:

**F1 — scheduler nunca arrancado, trabajo encolado, runner sano** (el agujero de
`FIND-748`; hoy da `is_broken() == false`):

```rust
observability::note_runner(RunnerState::Ollama);
// ninguna llamada a note_scheduler_started ni note_pass en esta fase
let live = QueueSnapshot { depth: 3, claimed: 0 };
let s = observability::status_with_queue(Some(live));
assert_eq!(s.queue, Some(live), "queue es el valor vivo, no una foto");
assert_eq!(s.last_pass, None, "un enqueue no inventa un pass");
assert_eq!(s.drain_path, DrainPath::None);
assert_eq!(s.health, IngestHealth::Broken);
assert!(s.is_broken());
assert!(s.notice.as_deref().unwrap_or("").contains('3'));
```

**F2 — tras un pass real** (misma fase 6 que ya existe, `:104-128`):
`run_pass(&queue, db, &NoMemories)` → `status_with_queue(Some(colas_vacía))`:
`queue == Some({0,0})`, `health == Idle`, `last_pass.queue.depth == 0`,
`last_pass.skipped == false`, `notice == None`, `!is_broken()`.

**F3 — `is_broken()` lee la cola viva y nunca `last_pass.queue`** (las dos
direcciones; esto es «cuál de los dos lee `is_broken()`»):

```rust
// (a) la foto dice 5, la cola viva dice 0 -> NO roto
observability::note_pass(0, 0, 0, QueueSnapshot { depth: 5, claimed: 0 });
let s = observability::status_with_queue(Some(QueueSnapshot { depth: 0, claimed: 0 }));
assert_eq!(s.last_pass.as_ref().unwrap().queue.depth, 5, "la foto vive en last_pass");
assert!(!s.is_broken(), "is_broken() no puede depender de la foto");

// (b) la foto dice 0, la cola viva dice 2 y el runner es none -> roto
observability::note_runner(RunnerState::None);
observability::note_pass(0, 0, 0, QueueSnapshot { depth: 0, claimed: 0 });
let s = observability::status_with_queue(Some(QueueSnapshot { depth: 2, claimed: 0 }));
assert_eq!(s.last_pass.as_ref().unwrap().queue.depth, 0);
assert_eq!(s.health, IngestHealth::Broken);
assert!(s.is_broken());
```

**F4 — sin lectura viva no se responde `idle`** (`status()`):

```rust
let s = observability::status();
assert_eq!(s.queue, None);
assert_eq!(s.health, IngestHealth::Unknown);
assert!(!s.is_broken(), "false = «no probado roto», nunca «sano»");
assert!(s.notice.is_some());
assert_eq!(serde_json::to_value(&s).unwrap()["queue"], serde_json::json!(null));
```

**F5 — leer no mueve el latch:** llamar `status_with_queue(...)` tres veces deja
`degraded_warnings` igual; sólo `note_skipped_pass`/`note_backlog` con `depth > 0`
lo incrementan, una vez por episodio (`:58-68` sigue verde).

**F6 — loop que arrancó y murió:** `note_scheduler_started()` →
`note_scheduler_stopped()` con `depth > 0` y runner sano → `drain_path == None`,
`health == Broken`. (Clase `MEM-72`, ahora en la fórmula.)

`vantadb-mcp` (integración; `memory_queue()` es `pub(crate)`, así que la cola se
puebla por la API pública `thread_send`):

- `tests/thread_memory_pipeline.rs:76-85`: `caps.ingest.queue.depth == 1` sigue
  valiendo (ahora por lectura viva); `caps.ingest.health == "broken"`; y la
  aserción `observability::status().is_broken()` pasa a
  `caps["ingest"]["health"] == json!("broken")` — `status()` ya no ve cola.
- `tests/capabilities_ingest.rs`: **reescribir o borrar**. Su premisa era
  `note_skipped_pass(backlog)` + `queue.depth == 5`, que ahora exige cola real;
  y su aserción `last_pass.skipped == true` **afirmaba la verruga** que D7 borra.
  Si tras reescribirlo queda subconjunto de `thread_memory_pipeline.rs`, borrarlo
  (deleción antes que duplicado).
- `tests/l0_l3_cycle_e2e.rs:351-354`: actualizar el comentario (ya no es «snapshot
  del último pass»); `queue.depth == 0` y `notice == null` tras el pass siguen
  valiendo; añadir `health == "idle"`.

## Verificación (comando único)

```bash
cargo nextest run --profile audit -p vanta-memory \
  -E 'test(ingest_observability) or test(scheduler) or test(pass)' --build-jobs 2
cargo nextest run --profile audit -p vantadb-mcp \
  -E 'test(capabilities) or test(thread_memory) or test(mcp_scheduler)' --build-jobs 2
cargo clippy -p vanta-memory -p vantadb-mcp --all-targets -- -D warnings
cargo fmt --check -p vanta-memory -p vantadb-mcp
node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs
```

**Mutación de control (obligatoria, «roja antes de verde»):** con el código de hoy,
F1 **tiene** que fallar. Cómo demostrarlo sin implementar: en la rama del test,
sustituir `status_with_queue(Some(live))` por el `status()` actual y comprobar que
`is_broken()` da `false` con `depth: 3`; y, tras implementar, revertir la línea de
`handlers/tools.rs` a `status()` — el test de `thread_memory_pipeline` debe caer.

`node scripts/docs/gen-index.mjs --write` **no** corre en esta tarea: a la fecha de
esta decisión el índice ya está stale por otro carril (`MEMG-27`/`MEMG-28`,
medido con `gen-index.mjs --check`). El carril que lo regenere lo hace en su commit.

## Fila de Backlog propuesta (para que el lead la agregue; no editada aquí)

> `FIND-7xx` | 🟡 Media | **El plugin de OpenCode decide `isBroken` con la fórmula
> vieja (`runner === "none" && depth > 0`) y no consume `ingest.health`** |
> `.opencode/plugins/vanta-memory-capture.ts` | 🟢 30-60 min | 🟡 Pendiente |
> Con `ADR-0065` la superficie publica `health`/`drain_path`, pero el plugin
> (`:137-138`, `:144`) sigue recomputando el predicado: queda ciego al caso «loop que
> arrancó y murió con trabajo encolado» y trata `queue: null` como `0`. Deuda
> consciente de `FIND-748`: arreglar el cliente exige tocar el repo de `.opencode`
> (base `2296ccd`), fuera del alcance de la implementación del core. | Origen:
> `FIND-748` (2026-11-25) | Ver: `ADR-0065`, `MCP-47`

## Qué NO resuelve (declarado)

- **Vista multi-proceso.** `ingest` describe el proceso que respondió. Proxy
  reenvía a writer (`MCP.md:498-507`); one-shot reporta `unknown` en vez de `idle`.
  Sigue siendo deuda `MCP-43`.
- **Duplicación del predicado en el cliente** (fila de arriba).
- **Leases varadas.** Un `claimed > 0` con `drain_path: none` no se marca `broken`
  (evita el falso positivo de la primera pasada de un host on-demand). Se ve en el
  campo `queue.claimed`, no en `health`.
