# MOD-08+MOD-09: Loop stdio serial + shutdown descarta respuesta in-flight

## Metadata
- **Plan file:** docs/plans/2026-08-24-batch-review-mod-find.md
- **Fuente:** plan file, task MOD-08+MOD-09 (Wave 1)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟠
- **Tipo:** Rust (MCP server)
- **Turns estimados:** 8
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 (resuelta: Send vía remoción de EnteredSpan)
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `run_stdio_server` (vantadb-mcp/src/server.rs:80) → `serve_lines`; `serve_lines` (tests en server.rs) |
| Callees | `dispatch_request` → handlers (initialize/tools/resources/prompts), `McpMetrics`, `RpcResponse`, `write_json`, `ActiveRequestGuard` |
| Implicaciones | Contrato JSON-RPC/MCP no cambia; semántica de tools no cambia. `serve_lines` pasa a despachar en background tasks (concurrencia) — la respuesta in-flight se escribe antes de salir. No toca protocolo ni handlers. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-mcp/src/server.rs` (427L), `vantadb-mcp/src/lib.rs`, `vantadb-mcp/src/protocol.rs`, `vantadb-mcp/src/metrics.rs`, `vantadb-mcp/Cargo.toml`, `.opencode/rules/server-mcp.md`, `.opencode/rules/concurrency-async.md`
- **Archivos referenciados hacia dentro (imports/includes):** `serve_lines`/`dispatch_request` usan `crate::protocol::{RpcRequest,RpcResponse}`, `crate::metrics::{ActiveRequestGuard,McpMetrics}`, `crate::error::McpError`, `crate::handlers::*`, `crate::config::McpConfig`, `vantadb::executor::Executor`, `vantadb::storage::StorageEngine`
- **Archivos que referencian a los editados (referencias entrantes):** `vantadb-mcp/src/lib.rs:43` re-exporta `run_stdio_server`. `grep serve_lines|dispatch_request|run_stdio_server` → solo server.rs + lib.rs + tests/mcp_tests.rs:2154 (comentario). Sin otros callers.
- **Veredicto impacto:** bajo — el cambio está confinado a `serve_lines` + remoción del `EnteredSpan` en `dispatch_request`. Contrato JSON-RPC intacto; la concurrencia es nueva pero serializada al escribir stdout por un único `tokio::sync::Mutex`.

## Contrato
"`cargo nextest run -p vantadb-mcp` pasa y la respuesta in-flight se escribe antes de salir (el loop despacha en background y drena las respuestas pendientes en shutdown)"

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** NO cambiar protocolo JSON-RPC ni semántica de tools MCP (R-1 server-mcp.md). Concurrencia del motor limitada por `tokio::sync::Semaphore` (R-2/R-3 concurrency-async). Trabajo del motor en `spawn_blocking`. Nunca mantener guard de mutex síncrono a través de `.await` (R-2). Escribir stdout siempre vía un único lock (tokio Mutex) para evitar interleaving entre tasks.
- **Comandos de verificación:** `cargo check -p vantadb-mcp`; `cargo nextest run -p vantadb-mcp`; `cargo fmt --check -p vantadb-mcp` (vía `cargo fmt --check`); `cargo clippy -p vantadb-mcp --all-targets -- -D warnings`
- **Deuda pendiente:** ninguna (el `EnteredSpan` de `dispatch_request` se elimina; observabilidad se conserva vía los `debug!`/`warn!` que ya incluyen `method`)

## Fase 1 — Evidencia de Debugging (GATE — tipo Bug)

- **Repro:**
  - MOD-08: client manda un burst de requests pipelined (ej. varios `tools/call`) en una ráfaga. `serve_lines` lee UNA línea y hace `.await` de `dispatch_request` completo (puede tardar por `spawn_blocking`/timeout), no leyendo stdin durante ese tiempo → backpressure/lectura no drenada. El loop es serial.
  - MOD-09: con `running=false` (SIGINT) tras despachar un request, las líneas 201-204 hacen `break` DESPUÉS de construir `response` pero ANTES de escribirlo → la respuesta in-flight se descarta.
- **Hipótesis:** el fix de ambos es un solo cambio en `serve_lines`: (a) despachar cada request en una task de background para que el reader nunca bloquee en un request lento (MOD-08) y (b) drenar las tasks in-flight (JoinSet) antes de retornar, escribiendo cada respuesta bajo un único lock de stdout (MOD-09). Requiere que `dispatch_request` sea `Send` → quitar el `EnteredSpan` (confirmado como `!Send` por el comentario del test en server.rs:351).
- **1 variable controlada:** por intento, solo se cambia el loop de `serve_lines` (+ remoción del span). Sin tocar handlers/protocolo.
- **Test RED:** test `in_flight_response_written_on_shutdown` (running=false + request → la respuesta se escribe igual) — falla en el código actual (el break descarta la respuesta) y pasa con el fix. Test `pipelined_requests_all_answered` (3 requests en un input → 3 respuestas).

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — toca un server stdio con input del cliente y un loop con posible race (multiple tasks escribiendo stdout). Cargada `security-and-hardening`. Hallazgos: se protege stdout con un único `tokio::sync::Mutex` (serializa escrituras, sin interleaving/race entre tasks); no se mantiene guard síncrono a través de `.await`; el JoinSet drena tareas (no se pierden respuestas ni se fugan tasks). Sin input deserializado ejecutado — JSON-RPC parseado y enrutado por método (sin eval). Race de `running` es `AtomicBool` (seguro).
- [x] **PERFORMANCE** — no es hot path de búsqueda/índice; es I/O de server stdio. El cambio mejora latencia bajo carga concurrente (reader no bloquea). No aplica benchmark `canonical_p99`. Nota: se añade un `tokio::sync::Mutex` por request-write (coste despreciable vs. el trabajo del motor).

## Steps

### Step 1: Reescribir serve_lines (MOD-08 + MOD-09) + remover EnteredSpan
- **Archivos:** `vantadb-mcp/src/server.rs`
- **Acción:** (1) `serve_lines`: wrap writer en `Arc<tokio::sync::Mutex>`; leer líneas en loop; por request con id, `inflight.spawn(...)` que despacha y escribe la respuesta bajo el lock; al señalarse shutdown, break del reader y `while inflight.join_next().await.is_some() {}` para drenar respuestas in-flight. (2) `dispatch_request`: eliminar `let _span = span!(...).entered();` y quitar `span`/`Level` del import de `tracing` para que el future sea `Send`. (3) añadir `W: ... + Send + 'static` al bound de `serve_lines`.
- **Verify:** `cargo check -p vantadb-mcp` — ✅ (Finished dev profile, 0 errors)
- **Estado:** ✅ DONE

### Step 2: Tests de regresión (MOD-09 shutdown + MOD-08 pipelined)
- **Archivos:** `vantadb-mcp/src/server.rs` (módulo `tests`)
- **Acción:** añadir `in_flight_response_written_on_shutdown` (running=false + request → respuesta escrita) y `pipelined_requests_all_answered` (3 requests → 3 respuestas). Helper refactorizado a `serve_lines_capture_with(input, running)`.
- **Verify:** `cargo nextest run -p vantadb-mcp` — ✅ 60/60 (incluye los 2 nuevos) + `cargo test -p vantadb-mcp --test mcp_tests` — ✅ 60/60
- **Estado:** ✅ DONE

### Step 3: fmt + clippy
- **Archivos:** `vantadb-mcp/src/server.rs`
- **Acción:** `cargo fmt --check` (1 fix de formato aplicado) y `cargo clippy -p vantadb-mcp --all-targets -- -D warnings`.
- **Verify:** exit 0 ambos — ✅
- **Estado:** ✅ DONE

### Step 4: Verify contrato completo + reporte
- **Acción:** `cargo nextest run -p vantadb-mcp` (60/60) + `cargo test -p vantadb-mcp --test mcp_tests` (60/60). Comando exacto para el lead: `cargo test -p vantadb-mcp --test mcp_tests` (el default-filter de nextest excluye `binary(mcp_tests)` — línea 62 de .config/nextest.toml — así que el verify del contrato usa el runner legacy).
- **Verify:** exit 0 — ✅
- **Estado:** ✅ DONE

## Dependencias
- Ninguna (tarea independiente en Wave 1)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-audit / vanta-review (leaf, no implementa). Auditoría de concurrencia (Regla 8): el cambio introduce tasks + `tokio::sync::Mutex` + JoinSet → delegar deadlock/data-race check a vanta-chaos/vanta-review antes de marcar ✅.
- **Enfoque:** ¿la concurrencia es correcta? ¿el lock de stdout evita races? ¿el drenado del JoinSet garantiza que la respuesta in-flight se escribe?
- **Cómo se probó:** tests RED→GREEN + `cargo nextest run -p vantadb-mcp`.
- **Veredicto:** pendiente

## Notas
- El `EnteredSpan` de `dispatch_request` es la única barrera `!Send` (confirmado por comentario test server.rs:351). Se elimina; la observabilidad se conserva (los `debug!`/`warn!` ya incluyen `method`).
- `serve_lines` pasa de serial a concurrente: el reader drena stdin mientras las tasks procesan; respuestas matched por id (JSON-RPC permite out-of-order). Cada escritura a stdout serializada por un único `tokio::sync::Mutex` (R-2 concurrency-async: tokio Mutex, no síncrono).
- Contrato: "respuesta in-flight se escribe antes de salir" ↔ drenado del JoinSet en shutdown.
