# TASK-MEM-67: TokenEstimator auto-detección tiktoken

## Metadata
- **Plan file:** docs/plans/2026-08-29-full-backlog-parallel.md
- **Creado:** 2026-08-30
- **last-synced:** 2026-08-30
- **Estado:** ✅ COMPLETED

## Spec (gate mecánico feature-add)

| Decisión | Elección | Justificación |
|---|---|---|
| ¿Auto-detección en `Default`? | **Sí** — Default usa el backend disponible | D21 amendment + ticket: callers (`TokenEstimator::default()`) no deben conocer el feature flag |
| ¿Conservar `new(chars_per_token)`? | **Sí** | API pública ya en uso (P2-5 no aplica); permite forzar heurístico explícito |
| ¿Eliminar cfg gates? | **No** — conservar para WASM compat (pre-mortem fallo 1) | tiktoken-rs no compila en wasm32; los `#[cfg]` preservan el build |
| ¿Unificar firma `estimate_tokens`? | **Sí** — única firma, dispatch interno | Branch interno en vez de dos `pub fn` gated |
| ¿Backwards compat? | **Sí** — comportamiento idéntico al actual bajo cada feature | Default cambia solo cuándo precise-tokens está on (mejora, no regresión) |

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `vanta-memory/src/context_engine/token_estimator.rs` (323 líneas), `vanta-memory/Cargo.toml` (68 líneas)
- **Referencias hacia dentro:** `crate::context_engine::types::{ChatMessage, ChatRole, CompactionMode, CompactionReport, ContextError}` (no cambian)
- **Referencias entrantes (callers):**
  - `src/context_engine/engine.rs:468` — `TokenEstimator::default()`
  - `src/context_engine/mmd_injector.rs:40,68` — `TokenEstimator::default()`
  - `src/services/pipeline_worker.rs:462` — `TokenEstimator::default()`
  - Tests: `tests/context_engine.rs:20`, `tests/e2e_flow.rs:358,462,531`
- **Veredicto:** API pública se preserva; cambio es interno. Sin breaking change. Sin tocar FFI (no es wasm/python).

## Contrato
`Select-String -Path "vanta-memory/src/context_engine/token_estimator.rs" -Pattern "tiktoken|cfg.*precise" | Measure-Object | Select-Object Count` >= 1

(Ya pasa: Count=8 antes del cambio — mantener o superar.)

## Herramientas
- cargo check, cargo clippy, cargo nextest
- codegraph (no es necesario; cambio confinado)

## Steps

### Step 1: Refactor TokenEstimator a dispatch interno
- **Archivos:** `vanta-memory/src/context_engine/token_estimator.rs`
- **Acción:**
  - Mantener `#[cfg(feature = "precise-tokens")]` para *incluir* tiktoken-rs (WASM compat)
  - Único `pub fn estimate_tokens` sin `#[cfg]` que dispatcha vía `enum TokenBackend { Bpe, Heuristic }` elegido en `Default`
  - `Default` con `#[cfg(feature = "precise-tokens")]` → `Bpe`; sin feature → `Heuristic`
  - `new(chars_per_token)` → siempre `Heuristic` (fuerza explícita)
- **Verify:** `cargo check -p vanta-memory --features precise-tokens` + `cargo check -p vanta-memory` (default)
- **Estado:** ⬜ PENDING

### Step 2: Actualizar tests
- **Archivos:** mismo archivo (sección `mod tests`)
- **Acción:** `estimate_tokens_ascii_unicode_heuristic` ya tiene `#[cfg(not(feature = "precise-tokens"))]` — verificar que sigue compilando con la nueva firma única. `precise_tokens_match_known_cl100k_golden_values` ya tiene `#[cfg(feature = "precise-tokens")]` — verificar dispatch interno.
- **Verify:** `cargo test -p vanta-memory --features precise-tokens --no-run` y sin feature
- **Estado:** ⬜ PENDING

### Step 3: Verify contrato + commit
- **Archivos:** mismos
- **Acción:** correr `Select-String` (mecánico) + clippy + fmt + tests
- **Verify:** comando del contrato pasa con Count >= 8
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna

## Notas
- Pre-mortem 1 (tiktoken en WASM): resuelto conservando `#[cfg]` para el include
- Pre-mortem 2 (default behavior): solo cambia el path cuando precise-tokens está on (mejora precisión, no regresión)
- Pre-mortem 3 (wasm32 target): tiktoken-rs no se compila sin precise-tokens; sin feature sigue con chars/3
- Stop conditions: >1h → docs-only changelog. Estimado: <30min.

## Context Save Point
- **Fecha:** 2026-08-30
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** dispatch interno via enum TokenBackend en vez de duplicar fn signatures
- **Problemas conocidos:** ninguno
- **Próxima tarea:** MEM-65 (paralela en W21) o siguiente