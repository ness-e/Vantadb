---
title: "FIND-748: queue.depth es un probe vivo y la salud del ingest es de cuatro estados"
kind: task
status: completed
description: "Implementar ADR-0065: capabilities.ingest.queue se lee en el momento de responder (o null), last_pass.queue conserva la muestra del pass, y health {unknown|idle|draining|broken} reemplaza la condición que mentía en dirección optimista"
---

# FIND-748: `queue.depth` es un probe vivo y la salud del ingest es de cuatro estados

## Metadata
- **Decisión:** [ADR-0065](../architecture/adr/ADR-0065-ingest-queue-live-probe-health.md)
  (`status: accepted`, `vanta-arch`, 2026-11-25)
- **Origen:** `MEM-73` (commit `c3e9c098e`), hallazgo 4 · fila `docs/dev/Backlog.md:836`
- **Tipo:** corrección de contrato en `vanta-memory` + `vantadb-mcp` + docs de API
- **Estado:** ✅ Completado (2026-11-25)
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

## Implementación (vanta-worker, 2026-11-25)

D1-D7 implementadas **sin desviaciones**, y el código no reveló ningún hecho que
las contradiga. **D8 no se entrega en el ejemplo que enuncia** — el código
contradice su prosa, y se devuelve a `vanta-arch` en vez de decidirlo aquí (punto
6, abajo).

### Cambios

| Archivo | Qué entró |
|---|---|
| `vanta-memory/src/ingest/observability.rs` | `IngestHealth` + `DrainPath` (`#[non_exhaustive]`, snake_case, `#[default] Unknown`/`None`); `PassSnapshot.queue`; `IngestStatus.queue: Option<QueueSnapshot>` + `scheduler_ever_started` + `health` + `drain_path`; `derived_drain_path`/`derived_health` (precedencia literal del ADR); `status_with_queue` (escribe queue → drain_path → health → notice) y `status()` delegando; `is_broken() == health == Broken`; `note_scheduler_started` pone `running` **y** `ever_started`; campo de estado `queue` borrado; `note_backlog` (sólo latch) y `note_skipped_pass` = `last_pass` + `note_backlog`; `notice()` con ramas broken-degradado / broken-sin-drainer / unknown nombrando causa |
| `vanta-memory/src/utils/local_backend.rs` | `queue_occupancy() -> (usize, usize)` en **una** adquisición, con `ponytail:` del techo |
| `vanta-memory/src/services/scheduler.rs` | `:138-146` y `:363-367` usan `queue_occupancy()` (sin lectura desgarrada) |
| `vantadb-mcp/src/threads.rs` | `queue_snapshot()`; `publish_backlog_if_degraded` → `note_backlog` (un enqueue no escribe `last_pass`) |
| `vantadb-mcp/src/handlers/tools.rs:3198` | `status_with_queue(Some(threads::queue_snapshot()))` |
| `vanta-memory/tests/ingest_observability.rs` | un solo `#[test]`, fases F1→(MEM-72)→F2→F3(a/b)→F4→F5→F6 en el orden obligatorio |
| `vantadb-mcp/tests/thread_memory_pipeline.rs` | asserts por el wire: `health == "broken"`, `queue.depth == 1` vivo, `drain_path == "none"`, **`last_pass == null`** tras `thread_send` (D7); se fue el `observability::status().is_broken()` |
| `vantadb-mcp/tests/capabilities_ingest.rs` | reescrito (no borrado): su valor único es el contrato aditivo + «degradado con cola vacía es `idle`». La aserción `last_pass.skipped == true` — que afirmaba la verruga que D7 elimina — se fue |
| `vantadb-mcp/tests/l0_l3_cycle_e2e.rs:351-354` | comentario actualizado + `health == "idle"` tras el drenaje |
| `docs/api/MCP.md` | ejemplo con `queue`/`health`/`drain_path`/`last_pass.queue`, tabla de precedencia de `health` (con `unknown` acotado a «sin evidencia de hospedar»), `:644` corregido, párrafo «two semantics / last_pass on a degraded pipeline» **borrado**, one-shot/`mcp-call` con lo que cada hijo responde de verdad (`:504-507`), y dos frases del plugin que todavía describían el predicado viejo |
| `docs/api/VANTA_MEMORY.md` | `:286`, `:391` (muestra del pass vs. lectura viva), `:399-406` reescrito a `health`/`drain_path` |

### 1. El punto que el contrato dejaba a criterio: dónde se deriva

`with_live_queue(status, queue)` es la única función que escribe el triple
derivado en el orden del ADR; `status_with_queue` es su llamada con el clon del
registro, y el `#[cfg(test)]` usa la misma función. Así el test unitario de
`notice` no puede construir un `IngestStatus` inconsistente a mano (ADR-0065
§ Negative consequences: «tests must go through `status_with_queue`») sin tocar
el registro proceso-global, que es justo lo que obliga a que la integración sea un
solo `#[test]`.

### 2. Mutaciones de control (obligatorias — «roja antes de verde»)

**Antes** — probe temporal (borrado tras capturarlo) con el código de `HEAD`
(`note_runner(Ollama)`, 3 tareas reales encoladas en un `LocalStateBackend`, F1
expresada contra la API de hoy):

```text
running 1 test
live depth = 3, reported queue.depth = 0, is_broken() = false
thread 'f1_against_todays_code_reports_not_broken_with_depth_3' panicked at
vanta-memory\tests\find748_before_probe.rs:34:5:
F1 red on today's code: 3 queued tasks with a healthy runner read NOT broken
test result: FAILED. 0 passed; 1 failed
```

**Después** — `handlers/tools.rs` revertido a `status()` (con el resto del cambio
en su sitio):

```text
thread 'thread_send_queues_ingest_once_per_turn_and_degraded_pipeline_reads_broken'
panicked at vantadb-mcp\tests\thread_memory_pipeline.rs:76:5:
assertion `left == right` failed: one task, read live:
{... "ingest":{"degraded":true,"health":"unknown","last_pass":null,
 "notice":"cannot see a queue here: this process does not host the ingest
 pipeline, so its health is unknown (not idle)...","queue":null,
 "runner":"none","scheduler_ever_started":false,"scheduler_running":false} ...}
  left: Null
 right: Number(1)
test result: FAILED. 0 passed; 1 failed
```

Ambas corridas quedan registradas aquí y en el mensaje del commit; el archivo de
probe no se comitea.

### 3. Verificación

```text
cargo nextest run --profile audit -p vanta-memory -E 'test(ingest_observability) or test(scheduler) or test(pass) or binary(ingest_observability)' --build-jobs 2
  → Summary [1.758s] 24 tests run: 24 passed, 725 skipped
cargo nextest run --profile audit -p vantadb-mcp -E 'test(capabilities) or test(thread_memory) or test(mcp_scheduler) or binary(thread_memory_pipeline) or binary(capabilities_ingest)' --build-jobs 2
  → Summary [10.190s] 3 tests run: 3 passed, 198 skipped
cargo nextest run --profile audit -p vantadb-mcp -E 'test(scheduler_tick_consolidates)'
  → 1 passed (loop real: thread_send → tick → recall → stop limpia el flag)
cargo clippy -p vanta-memory -p vantadb-mcp --all-targets -- -D warnings  → limpio
cargo fmt --check -p vanta-memory -p vantadb-mcp                          → FMT OK
node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs     → GATING: all clear
```

Nota de filtro: `-E 'test(ingest_observability)'` no matchea nada porque nextest
filtra por **nombre de test**, no por binario; el comando del contrato se corrió
tal cual y se le añadió `binary(...)` para que efectivamente cubra el archivo.
`l0_l3_cycle_e2e` es `#[ignore]` (lane `dev-tools/e2e-memory-cycle.ps1`); su
aserción nueva (`health == "idle"`) es consecuencia de la que ya existía
(`queue.depth == 0` + lectura viva ⇒ `Idle`).

### 4. Consumidores de `capabilities.ingest`

`rg` sobre el repo: **uno solo en el wire** (el tool MCP mismo). El único cliente
que lo parsea es el plugin de OpenCode, **fuera de este repo**
(`.opencode/plugins/vanta-memory-capture.ts`, `FIND-749`, base `2296ccd`), que ya
guarda `ingest.queue?.depth` — el cambio de forma no lo rompe, pero sigue
recomputando el predicado viejo (deuda registrada, no tocada aquí).
`vantadb-server` no publica `ingest`; `evals/`, `vantadb-python`, `vantadb-wasm`,
`vantadb-node` y `vantadb-ts` no lo leen. Los tests del repo se actualizaron en
esta PR.

### 5. Naturaleza del cambio

**Aditivo en el wire salvo un caso**, declarado en el ADR: `queue` pasa de objeto
a `objeto | null` en el tipo. En el wire de `capabilities` el campo sigue siendo
siempre un objeto (el handler siempre lee su propia cola); el `null` aparece
llamando `status()` desde librería. `scheduler_ever_started`, `health` y
`drain_path` son llaves nuevas; `last_pass.queue` es una llave nueva dentro de un
objeto existente. Un cliente que hacía `ingest.queue.depth` sin guard hoy no
rompe por `null` en el wire, pero la regla documentada es guardar de todas formas.
En Rust, `IngestStatus.queue` cambia de tipo y `PassSnapshot` gana un campo
(ambos `pub`, pero `vanta-memory` es pre-1.0 y la superficie es de diagnóstico).

### 6. Hallazgo devuelto a `vanta-arch` (no decidido aquí)

`ADR-0065 § D8` afirma: «in one-shot `mcp-call` every invocation is a fresh
process whose empty queue is not the writer's, which is why that case now reads
`unknown` instead of `idle`». **El código no produce eso en el caso general.**

Ruta verificada: `src/cli_handlers/mcp_call.rs:260-266` lanza `<server> --mcp`
con `VANTADB_STORAGE_PATH`; `vantadb-mcp/src/server.rs:1057-1065` abre el writer
y, si no es read-only, llama `memory_scheduler::start()`, que ejecuta
`note_scheduler_started()` **antes** de servir la primera petición
(`memory_scheduler.rs:134`). En ese momento `runner` **sigue siendo `Unknown`** (el
runner se construye dentro del factory por tick, `memory_scheduler.rs:94-100`); lo
que derrota al guard de «sin evidencia» no es el runner sino
`scheduler_running == true` (`observability.rs:365`). Con eso, `derived_health` da
**`idle`** (cola `{0,0}`) y `derived_drain_path` da **`loop`**.
Cuando otro writer ya tiene la base (`DatabaseBusy`, `server.rs:1078+`), el hijo
**no responde**: reenvía el `tools/call` entero al writer (`proxy.rs:197-200` →
`server.rs:806-825,127-166`), así que el cliente lee la cola y la salud **del
writer**, no un `unknown` del proxy.

Consecuencia: `queue: null` es **inalcanzable en el wire** (el handler siempre
pasa `Some`), y el caso que D8 describe como `unknown` (el one-shot) responde
`idle` — o la salud del writer si el hijo acabó en proxy. El contrato C4 pedía
exactamente ese `Some(...)` incondicional, así que no se tocó el handler: aplicar
la prosa de D8 exigiría pasar `None` cuando no hay evidencia de hospedar, y eso es
una decisión de arquitectura, no de implementación. La documentación publicada se
corrigió a lo que el código hace (hijo writer → `idle`, su propia cola; hijo proxy
→ la respuesta del writer; `unknown` cuando el proceso no tiene evidencia de
hospedar, p. ej. writer read-only), y queda **pendiente** que `vanta-arch` decida entre (a) aceptar
esa redacción y enmendar D8, o (b) cambiar el handler para pasar `None` sin
evidencia de hospedaje (lo que además haría aparecer `queue: null` en el wire).

## Review

### Ronda 1 — `changes-required`

- **Revisor:** `vanta-review` (sub-agente, contexto fresco, sin haber escrito el código).
- **Enfoque:** adversarial sobre el diff completo (11 archivos) contra `ADR-0065`
  D1-D8, el contrato de `FIND-748` y las rutas que lo consumen
  (`server.rs:1044-1149`, `mcp_call.rs:234-303`, `memory_scheduler.rs:60-165`).
- **Cómo se probó:** lectura de los diffs + `rg`/lectura de los call sites;
  verificación mecánica de la precedencia de `derived_health`/`derived_drain_path`
  contra el ADR token por token, de que ningún otro lector del campo de estado
  `queue` sobrevive, del orden queue→drain_path→health→notice, de la disciplina de
  lock en `latch_backlog` (el `warn!` nunca corre con el mutex tomado), y de que la
  aserción nueva del test unitario ejerce la **segunda** regla de `unknown`.
- **Veredicto:** `changes-required`, 2 bloqueantes + 7 no bloqueantes.
  1. **Bloqueante (wire falso).** La documentación afirmaba que un `mcp-call`
  one-shot lee `queue: null` / `health: "unknown"`. Falso: el handler pasa siempre
  `Some`, y el hijo one-shot que gana el writer arranca su loop
  (`server.rs:1057-1065` → `memory_scheduler.rs:134`) y lee `idle`/`loop`.
  2. **Bloqueante (proceso).** El § Review era auto-atestado: afirmaba un veredicto
  que no existía. Reemplazado por este registro.
  - No bloqueantes aplicados: comentario del enum, línea en blanco espuria, la
    frase del `warn!` (quedaba colgada del bucket no-degradado), y el hueco de
    cobertura de la rama «hay lectura de cola pero sin evidencia de hospedar»
    (ahora en el test unitario).
  - No aplicados (declarados): ADR `:15` «Not yet implemented» (el ADR está fuera
    del alcance de esta tarea), fila del Backlog (es del lead), e2e `#[ignore]`, y
    un accesor `degraded()` para no formatear un `String` por enqueue (una
    asignación por turno en un camino que ya toma el lock de la cola y escribe L0
    no justifica superficie pública nueva).

### Ronda 2 — `changes-required`

- **Revisor:** el mismo, segunda ronda sobre lo cambiado.
- **Enfoque:** ¿queda alguna frase que afirme un comportamiento que el código no
  produce?
- **Cómo se probó:** siguió el reenvío del proxy hasta el handler
  (`proxy.rs:197-200` → `server.rs:806-825` → `mcp_proxy_handler:127-166`) y
  comprobó que el proceso en modo proxy **no responde** `capabilities`: lo reenvía.
- **Veredicto:** `changes-required`, 1 bloqueante nuevo + 3 no bloqueantes: el
  ejemplo «proxy → `unknown`» (introducido en la ronda 1) seguía siendo falso y
  además contradecía un párrafo correcto diez líneas antes. Aplicado: el proxy
  sale de todos los ejemplos de `unknown` (los honestos son writer read-only y
  writer con loop deshabilitado antes del primer pass/enqueue) y la rama proxy
  pasa a decir que la respuesta es la del writer. También corregida la premisa de
  §6: el `idle` del hijo writer se apoya en `scheduler_running == true`, no en el
  runner (que sigue `Unknown` hasta el primer tick).

### Ronda 3 — `approve`

- **Revisor:** el mismo, tercera ronda acotada a las cuatro superficies cambiadas.
- **Enfoque:** ¿es verdad cada frase que queda, en los cuatro estados (hijo writer,
  hijo proxy, writer read-only, writer con loop deshabilitado)?
- **Cómo se probó:** lectura de los cuatro archivos + `rg` de `queue: null`,
  `queue?.depth`, `one-shot` y `proxy` sobre `docs/api/**` y el módulo, contrastado
  con `tools.rs:3199-3201` (único publicador, siempre `Some`), `server.rs:1056,1063`
  (read-only no arranca loop) y `memory_scheduler.rs:111-116` (intervalo `0` retorna
  antes de `note_scheduler_started`).
- **Veredicto:** `approve`, sin bloqueantes. Condición de cierre (cumplida): este
  registro existe antes del commit y la tarea no se marca `completed` sin él.
  Deuda declarada que queda para `vanta-arch`: `tasks/FIND-748.md:279` («one-shot
  reporta `unknown`») contradice §6 — se corrige junto con la enmienda de D8, no
  como edición suelta.

### Reglas transversales

- **Regla 0:** todos los archivos del alcance leídos completos antes de editar
  (`observability.rs`, `local_backend.rs`, `scheduler.rs:100-189/330-399`,
  `threads.rs:230-399`, `tools.rs:3160-3209`, los tres tests, `MCP.md:440-740`,
  `VANTA_MEMORY.md:275-314/380-419`).
- **Scope discipline:** no se tocó `.opencode/**`, `vantadb-server`, ni
  `gen-index.mjs` (índice stale por otro carril). Fuera del alcance detectado y
  **no tocado**: `.github/workflows/nightly.yml`, `docs/dev/operations/CI_POLICY.md`,
  `docs/dev/tasks/MEM-73.md`, `src/wal.rs`, `src/wal_sharded.rs` (otro carril en
  el mismo worktree).
