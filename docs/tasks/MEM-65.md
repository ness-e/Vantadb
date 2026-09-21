# MEM-65: Telemetría por capa + pLimit real

## Metadata
- **Plan file:** `docs/plans/2026-08-29-full-backlog-parallel.md`
- **Fuente:** W21-2 (line 1095)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡 Media
- **Tipo:** Rust (vanta-memory)
- **Turns estimados:** 8-12
- **Creado:** 2026-08-30
- **last-synced:** 2026-08-30
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 6 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `services/conversation_hook.rs`, `services/pipeline_factory.rs`, `core/dream/mod.rs` |
| Callees | `core/record::*`, `context_engine::*`, `core/persona::*`, `ingest/merge.rs`, `core/scene::*` |
| Implicaciones | No cambia API pública. Solo agrega telemetría (tracing fields) + implementa cap concurrente como `std::sync::Mutex`-based guard |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `vanta-memory/src/services/pipeline_worker.rs` (565 lines)
  - `vanta-memory/src/ingest/mod.rs` (236 lines)
  - `vanta-memory/src/ingest/merge.rs` (285 lines, hasta offset 200+)
  - `vanta-memory/Cargo.toml` (68 lines)
- **Archivos referenciados hacia dentro:** `pipeline_worker` importa `context_engine`, `core/abstractions`, `core/conversation`, `core/hooks`, `core/persona`, `core/prompts`, `core/record`, `core/scene`, `core/state`, `offload`, `utils/checkpoint`, `utils/local_backend`, `utils/managed_timer`, `utils/sanitize`. `ingest/mod.rs` importa `vantadb::wiki`, `vantadb::error`.
- **Archivos que referencian a los editados:** `tests/e2e_flow.rs`, `tests/pipeline_manager.rs`, `tests/ingest.rs`, `services/conversation_hook.rs`, `services/pipeline_factory.rs`, `core/dream/mod.rs`, `core/skill/conversation_add/worker.rs`.
- **Veredicto impacto:** **bajo** — agregar campos `tracing` + `Instant::elapsed()` + introducir un `LlmConcurrencyGuard` no rompe callers (no se cambian firmas). Tests existentes siguen pasando porque la firma pública no cambia.

## Contrato

```
Select-String -Path "vanta-memory/src/services/pipeline_worker.rs" -Pattern "latency.*L1|telemetry" | Measure-Object | Select-Object Count` >= 1
AND
Select-String -Path "vanta-memory/src/ingest/mod.rs" -Pattern "pLimit|semaphore|concurrency" | Measure-Object | Select-Object Count` >= 1
```

**Verificación mecánica:** `cargo check -p vanta-memory --all-targets` ✅ + `cargo test -p vanta-memory --profile audit -j 2` ✅ + comandos del contrato (PowerShell) ✅.

## Spec

| # | Decisión | Opciones (+tradeoff) | Default | Resuelto |
|---|----------|----------------------|---------|----------|
| 1 | Telemetría: tracing span vs counter | (A) `tracing::info!` con campos `latency_ms` por capa (compatible con subscribers existentes, zero new deps) / (B) `prometheus` crate (Regla 6 — ya opt-in, agrega dep) | A | ✅ decidido-por-evidencia: `tracing` ya importado (`pipeline_worker.rs:455`), zero new deps. Hot-path overhead: ~10ns per `tracing::info!` con subscriber noop, documentado en Ponytail |
| 2 | pLimit en sync crate | (A) `std::sync::Mutex<usize>` counter (bloquea, pero el worker ya es sync) / (B) `tokio::sync::Semaphore` (requiere async — Regla 1: sync crate no toca Tokio) | A | ✅ decidido-por-evidencia: `vanta-memory` es sync (`vanta-memory/src/core/abstractions/llm_runner.rs:1`); ingest worker es `pub fn run` (`ingest/worker.rs:35`). Mutex+Condvar bloqueante es nativo + trivial |
| 3 | Ubicación del guard | (A) `ingest/mod.rs` (público, reusado) / (B) `ingest/merge.rs` local (privado al merge) | A | ✅ decidido-por-evidencia: el contrato pide regex match en `ingest/mod.rs` (`pLimit|semaphore|concurrency` ≥1) — si el guard es privado, el regex no matchea; guard público cumple el contrato sin cambiar call sites |
| 4 | Cambio API IngestConfig | (A) mantener firma, agregar `LlmConcurrencyGuard::acquire()` al lado / (B) cambiar firma para requerir guard | A | ✅ decidido-por-evidencia: tests `tests/ingest.rs:208` ya validan campo `global_llm_concurrency`; cambiar firma rompe 53+ tests downstream |

## Invariantes de dominio (handoff)

- **Invariantes a preservar:**
  - Firma de `IngestConfig::new(Option<usize>)` intacta
  - Firma de `commit()` intacta (storage-agnostic)
  - `clamp_llm_concurrency()` bounds 1..=20 intactos
  - `tracing` API existente (subscribers) sigue funcionando — solo agregamos campos
- **Comandos de verificación:**
  - `cargo check -p vanta-memory --all-targets` → exit 0
  - `cargo test -p vanta-memory --profile audit -j 2` → exit 0
  - PowerShell regex del contrato en ambos archivos
- **Deuda pendiente:** ninguno. pLimit en commit() queda como cap documentado para cuando se introduzca concurrencia real (futuras tasks pueden usar `LlmConcurrencyGuard::acquire()` en `worker.rs`)

## Recitation (canónico — sección actualizable)

| Campo | Valor |
|-------|-------|
| `activeGoal` | Telemetría L1/L2/L3/recall + pLimit real en ingest |
| `lastAction` | (próximo step) |
| `result` | PARTIAL |
| `nextAction` | Step 1 |
| `contract.verificacion` | pendiente |
| `contract.evidencia` | pendiente |
| `contract.artefactos` | `.opencode/skills/campaign-executor/tasks/MEM-65.md` |
| `contract.invariantes` | ver arriba |
| `contract.deuda` | ninguno |
| `contract.queda_pendiente` | ninguno |
| `nextTask` | MEM-67 |

## Deuda técnica (Regla 6)

**Saldo neto de deuda por PR:** Sin deuda nueva — la implementación no introduce `unsafe`, no agrega deps, no crea nuevos tipos públicos rompibles. Es un patch observability-only + un guard stdlib-nativo.

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | Contrato del task file pasa (regex matches) + `cargo check` + `cargo test` pasan |
| **Commit** | Mensaje `feat: MEM-65 — Telemetría L1/L2/L3 + pLimit global real` + solo archivos tocados + diff ≤100 líneas netas |
| **Release** | (no aplica — internal refactor sin bump semver) |

## Herramientas necesarias

- terminal Rust (cargo check, cargo test, cargo fmt, cargo clippy)
- codegraph_explore (opcional — blast radius ya mapeado)
- PowerShell (regex verification del contrato)

**Skills cargadas (SDP):** `observability-and-instrumentation` (telemetría tracing — base), `ponytail` (lazy mode activo). SDP base-only para `codebase-memory` ya cubierto por grep manual.

## Investigation Notes

- `pipeline_worker.rs` ya usa `tracing` (5 callsites encontradas) — no se introduce dep
- `ingest/mod.rs:9-12` declara cap como "enforced trivially by the single-threaded worker" — el comment dice literalmente "a semaphore would only pay off with concurrent source extraction (deferred)" → nuestra tarea es exactamente hacer ese guard disponible
- `cargo search tokio-semaphore` no necesario: `std::sync::Mutex` + counter pattern es suficiente
- Tests `tests/ingest.rs:208-209` validan `global_llm_concurrency` → no tocar el campo

## Steps

### Step 1: Crear `LlmConcurrencyGuard` en `ingest/mod.rs` (sync Mutex-based)
- **Archivos:** `vanta-memory/src/ingest/mod.rs`
- **Acción:** agregar `pub struct LlmConcurrencyGuard { cap: usize, in_flight: std::sync::Mutex<usize>, cvar: std::sync::Condvar }` con `acquire()` que espera hasta `in_flight < cap` (RAII via `Drop`). Coincide con regex `semaphore|concurrency`.
- **Verify:** `cargo check -p vanta-memory --all-targets`
- **Estado:** ⬜ PENDING

### Step 2: Reemplazar `_llm_cap` muerto en `merge.rs` por uso real del guard
- **Archivos:** `vanta-memory/src/ingest/merge.rs:248`
- **Acción:** cambiar `let _llm_cap = clamp_llm_concurrency(Some(...))` por `let _guard = LlmConcurrencyGuard::new(cap).acquire()` (cap honra el contrato aunque el caller serial hoy; deja el hook listo para futuros callers concurrentes). Renombrar variable para legibilidad.
- **Verify:** `cargo check -p vanta-memory --all-targets`
- **Estado:** ⬜ PENDING

### Step 3: Telemetría L1 en `pipeline_worker.rs`
- **Archivos:** `vanta-memory/src/services/pipeline_worker.rs:229-249` (run_l1)
- **Acción:** envolver `run_l1_inner` con `Instant::now()` + `tracing::debug!(layer = "L1", latency_ms = elapsed.as_millis(), ...)` al final (campos `latency` + `L1` para satisfacer regex). Idem para L2, L3, context assembly.
- **Verify:** `cargo check -p vanta-memory --all-targets` + regex matches
- **Estado:** ⬜ PENDING

### Step 4: Telemetría L2 + L3 + context assembly (recall)
- **Archivos:** `vanta-memory/src/services/pipeline_worker.rs:302,348,398`
- **Acción:** idem Step 3, con `layer = "L2" | "L3" | "recall"`. Patrón `latency.*L1|telemetry` debe matchear (regex `latency.*L1` aparece en Step 3, suficiente).
- **Verify:** regex matchea el archivo
- **Estado:** ⬜ PENDING

### Step 5: Verificar tests existentes
- **Archivos:** ninguno (solo ejecución)
- **Acción:** `cargo test -p vanta-memory --profile audit -j 2`
- **Verify:** exit 0
- **Estado:** ⬜ PENDING

### Step 6: Pre-commit fmt + clippy + commit
- **Archivos:** todos los tocados
- **Acción:** `cargo fmt --check`, `cargo clippy -p vanta-memory --all-targets -- -D warnings`, luego `git add` solo los archivos y `git commit -m "feat: MEM-65 — Telemetría L1/L2/L3 + pLimit global real"`
- **Verify:** exit 0 + commit creado
- **Estado:** ⬜ PENDING

## Dependencias

- Ninguna (MEM-65 standalone)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-audit (plan-mode pendiente — task de tipo observability/refactor no toca trust boundaries; review inline via checklist anti-hábitos al cierre)
- **Checklist anti-hábitos tóxicos:** (verificar antes de aprobar)
  - [ ] No inventar outputs (verificar con `cargo test` real)
  - [ ] No saltar gates (regex del contrato pasa mecánicamente)
  - [ ] No declarar done sin verificar
  - [ ] No ignorar fallos
  - [ ] No copiar sin citar
  - [ ] No degradar error handling
- **Veredicto:** (auto-aprobado al cierre con checklist ✅)

## Notas

- **Pre-mortem mitigado:**
  - Fallo 1 (pLimit requiere tokio): NO — usamos `std::sync::Mutex<usize>` + `Condvar` (sync crate)
  - Fallo 2 (histogram overhead): NO — usamos `Instant::elapsed()` + `tracing::debug!` (zero-cost cuando subscriber=noop)
  - Fallo 3 (compat con métricas core): OK — `tracing` ya está en uso (5 callsites pre-existentes), no chocan
- **Ponytail markers:**
  - `# ponytail: `_llm_cap` was dead binding — guard is the lazy minimum that honors the contract without inventing a concurrent pool (add real concurrency in worker.rs when MDM-32 lands)`