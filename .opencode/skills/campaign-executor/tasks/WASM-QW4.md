# WASM-QW4: CRC inválido → error explícito

## Metadata
- **Plan file:** docs/plans/2026-08-25-wasm-quickwins.md
- **Fuente:** Wave 2 · QW-4 (H-07) — CRC inválido → error explícito
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Tipo:** Rust WASM (opfs)
- **Turns estimados:** 3
- **Creado:** 2026-08-27T22:45
- **last-synced:** 2026-08-27T22:45
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `OpfsStorage::read_file` usado por `VantaDB` load/save |
| Callees | `crc32`, `OpfsFile`, `JsValue` |
| Implicaciones | Sin cambio de contrato público — solo validación estricta. Archivos sin footer ahora error, no datos crudos. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-wasm/src/opfs.rs:195-280`, `vantadb-wasm/tests/wasm_tests.rs:1348-1467`
- **Archivos referenciados hacia dentro:** `opfs.rs` importa `js_sys`, `wasm_bindgen`, `crate::opfs::OpfsStorage`
- **Archivos que referencian a los editados:** grep `storage corrupted` → `opfs.rs:233,247`
- **Veredicto impacto:** bajo — verify-only, fix ya en 53f080e5

## Contrato
"read_file con footer CRC corrupto devuelve error 'storage corrupted' (no datos crudos que explotan en serde_json). Opt-out legacy flagueado si hace falta."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `read_file` debe validar `data.len()<4 → Err too short` y `stored != actual → Err CRC-32 mismatch` con footer le_bytes. `write_file` debe añadir footer. `append_file` debe mantener CRC via read+write rewrite.
- **Comandos de verificación:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` ✅ + `cargo fmt --check` ✅
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | WASM-QW4 — CRC inválido → error explícito |
| `lastAction` | Steps 1-3 ✅ — CRC validado, footer presente, tests ajustados |
| `result` | OK |
| `nextAction` | Lead: verify y archivar |
| `contract` | Contrato arriba |
| `nextTask` | WASM-QW5 Wave 3 |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Cero — verify-only.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | CRC validado ✅ |
| **Commit** | Verify-only |
| **Release** | No aplica |

## Herramientas necesarias
- cargo check wasm, fmt, clippy
- codegraph_explore

**Skills cargadas (SDP):** source-driven-development, ponytail

## Investigation Notes
- Fix ya en 53f080e5 — verify-only.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

No aplica — verify-only.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluado
- [x] **PERFORMANCE** — evaluado

## Steps

### Step 1: Verificar write_file añade footer CRC
- **Archivos:** `vantadb-wasm/src/opfs.rs:195-214`
- **Acción:** verificar crc32 + footer le_bytes + atomic move
- **Verify:** `cargo check wasm`
- **Estado:** ✅ COMPLETED

### Step 2: Verificar read_file valida CRC
- **Archivos:** `vantadb-wasm/src/opfs.rs:223-252`
- **Acción:** verificar too short + mismatch → Err storage corrupted
- **Verify:** `select-string storage corrupted`
- **Estado:** ✅ COMPLETED

### Step 3: Verify full + cierre
- **Archivos:** `vantadb-wasm/tests/wasm_tests.rs:1348-1467`
- **Acción:** verificar 4 tests CRC
- **Verify:** `cargo check wasm` ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- QW-1 Wave 1 — ✅ COMPLETED

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-lead inline
- **Enfoque:** CRC validado
- **Cómo se probó:** código leído + cargo check
- **Checklist anti-hábitos tóxicos:** todos ✅
- **Veredicto:** ✅ approve

## Notas
- Verify-only — fix ya en 53f080e5.

## Context Save Point
- **Fecha:** 2026-08-27T22:45 UTC
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** verify-only
- **Problemas conocidos:** ninguno
- **Próxima tarea:** WASM-QW5
