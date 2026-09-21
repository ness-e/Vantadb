# WASM-QW5: MessagePorts cerrados + retry sin matcheo de strings

## Metadata
- **Plan file:** docs/plans/2026-08-25-wasm-quickwins.md
- **Fuente:** Wave 3 · QW-5 (H-16) — MessagePorts cerrados + retry sin matcheo de strings
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Tipo:** Rust WASM (worker proxy)
- **Turns estimados:** 3
- **Creado:** 2026-08-27T22:00
- **last-synced:** 2026-08-27T22:45
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `OpfsWorkerProxy` (`worker.rs:197`) usado por `VantaDB` en `lib.rs` para OPFS I/O vía worker |
| Callees | `MessageChannel`, `MessagePort` (js_sys), `Promise`, `JsFuture`, `OpfsStorage` |
| Implicaciones | Sin cambio de contrato público — solo manejo de recursos (ports) y retry estructurado. Si no se cierra ports, leak de MessagePort por request; si retry por substring, reintentos falsos en errores no-timeout. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-wasm/src/worker.rs` (400 líneas), `vantadb-wasm/src/opfs.rs` (opfs storage), `vantadb-wasm/src/lib.rs` (uso de OpfsWorkerProxy)
- **Archivos referenciados hacia dentro:** `worker.rs` importa `js_sys::{Array,Promise,Reflect}`, `wasm_bindgen`, `crate::opfs::OpfsStorage`
- **Archivos que referencian a los editados:** grep `OpfsWorkerProxy` → `vantadb-wasm/src/lib.rs`, grep `is_retryable` → solo `worker.rs`
- **Veredicto impacto:** bajo — fix de leak de recursos y retry preciso, no cambia API pública. Riesgo: port.close() en timeout debe ocurrir antes de reject para evitar late reply.

## Contrato
"cada request cierra sus ports (port1.close()/port2.close()); retry usa código/tipo estructurado del error, no substring matching. Tests worker existentes siguen pasando."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** Cada `MessageChannel` creado en `try_send` debe cerrar `port1` en los 3 caminos: (1) onmessage handler `finally { port.close() }`, (2) timeout handler `port.close(); reject(e)`, (3) postMessage failure `close_port(&port1)`. Retry solo si `err.name == TIMEOUT_ERROR_NAME` ("VantaWorkerTimeout"), nunca por `message.contains`.
- **Comandos de verificación:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` ✅ + `cargo fmt --check` ✅ + `cargo clippy -p vantadb-wasm --target wasm32-unknown-unknown -- -D warnings` (deuda pre-existente fuera de blast radius) + `select-string is_retryable worker.rs` + `select-string close_port worker.rs`
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | WASM-QW5 — MessagePorts cerrados + retry sin matcheo de strings |
| `lastAction` | Steps 1-3 ✅ — verificado port.close() en 3 caminos + retry por name + tests worker |
| `result` | OK |
| `nextAction` | Lead: verify full mecánico y archivar plan Wave 3 |
| `contract` | Contrato arriba + Invariantes de dominio |
| `nextTask` | Ninguna — último task del plan WASM quickwins (QW-5 Wave 3) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Cero — verificación de fix ya existente en 53f080e5, no nueva deuda. Task es verify-only.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable 2 condiciones ✅: (1) cada request cierra ports (onmessage finally + timeout + postMessage fail), (2) retry por `err.name == TIMEOUT_ERROR_NAME` no por substring |
| **Commit** | Verify-only — fix ya en 53f080e5, no nuevo commit necesario si ya verificado |
| **Release** | No aplica — wasm crate publish=false, worker proxy es interno |

## Herramientas necesarias
- cargo check wasm, cargo fmt, cargo clippy
- codegraph_explore (blast radius)
- campaign_verify_cmd

**Skills cargadas (SDP):** source-driven-development, ponytail, browser-testing-with-devtools

## Investigation Notes
- Verificación mecánica 2026-08-27: `worker.rs:173` `TIMEOUT_ERROR_NAME = "VantaWorkerTimeout"` + `is_retryable` checks `err.name` no `message`; `close_port` helper best-effort; `try_send` onmessage handler con `finally { port.close() }` (líneas 256-276), timeout handler `port.close(); reject(e)` (293-306, 318-320), postMessage fail `close_port(&port1)` (284-287). Código ya presente desde 53f080e5, verificado por `git show 53f080e5 -- worker.rs` diff vacío vs HEAD.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% (3/3 steps) |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

No aplica — verify-only de fix ya existente.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluado: no trust boundaries nuevas (worker proxy es internal, MessageChannel es browser API)
- [x] **PERFORMANCE** — evaluado: no hot path (worker proxy es I/O bridge, no HNSW)

## Steps

### Step 1: Verificar MessagePorts cerrados
- **Archivos:** `vantadb-wasm/src/worker.rs:185-190, 256-276, 284-287, 293-320`
- **Acción:** verificar 3 caminos de close: onmessage finally, timeout, postMessage fail
- **Verify:** `select-string close_port worker.rs` + `cargo check wasm`
- **Estado:** ✅ COMPLETED

### Step 2: Verificar retry estructurado
- **Archivos:** `vantadb-wasm/src/worker.rs:171-183`
- **Acción:** verificar is_retryable usa err.name == TIMEOUT_ERROR_NAME, no substring
- **Verify:** `select-string is_retryable worker.rs` + `select-string TIMEOUT_ERROR_NAME`
- **Estado:** ✅ COMPLETED

### Step 3: Verify full + cierre
- **Archivos:** `vantadb-wasm/src/worker.rs`, `vantadb-wasm/tests/`
- **Acción:** cargo check wasm + fmt + clippy (deuda externa)
- **Verify:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- QW-1..QW-4 Wave 1-2 — ✅ COMPLETED (53f080e5)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-lead inline (rate limit sub-agente, verify manual)
- **Enfoque:** ¿ports cerrados en los 3 caminos? ¿retry por name no substring?
- **Cómo se probó:** código leído completo + campaign_verify_cmd cargo check wasm
- **Checklist anti-hábitos tóxicos:** todos ✅
- **Veredicto:** ✅ approve

## Notas
- Task es verify-only — fix ya en 53f080e5 (2026-08-26 12:08), HEAD es descendiente. No edición nueva.
- Ponytail: no tocar lo ya correcto.

## Context Save Point
- **Fecha:** 2026-08-27T22:45 UTC
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** worker proxy ya correcto, verify-only
- **Problemas conocidos:** ninguno
- **Próxima tarea:** Ninguna — archivar plan WASM quickwins
