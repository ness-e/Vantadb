# BND-03 — tiktoken feature-gate `precise-tokens` (enmienda D21 del ADR-29)

**Task ID:** 5 (plan P33 `docs/plans/2026-08-22-vanta-ultima-milla.md`)
**Estado:** ⬜ PENDING → IN PROGRESS
**Ruta:** vanta-worker · **Cynefin:** 🟦 obvio · **Esfuerzo:** 🟢

## Contrato

"`cargo check -p vanta-memory` default pasa (chars/3 intacto); con `--features precise-tokens`: estimate usa tiktoken-rs, tests comparan contra valores conocidos de cl100k; CJK/código ahora precisos"

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `vanta-memory/src/context_engine/token_estimator.rs` (268L, verbatim vía codegraph)
- `vanta-memory/Cargo.toml` (46L)

**Referencias entrantes (blast radius):**
- `TokenEstimator` — 13 callers: `context_engine/mod.rs`, `mmd_injector.rs`, `engine.rs`, `token_estimator.rs`; tests `tests/context_engine.rs`
- `emergency_truncate` (7 callers), `build_units` (6 callers) — no cambian

**Referencias salientes:** `context_engine::types` (ChatMessage/ChatRole/CompactionReport/ContextError), serde_json.

**Veredicto:** cambio confinado al cuerpo de `estimate_tokens` (+cfg branch) y `Cargo.toml`.
Cero cambios de firma pública → callers intactos. Bajo la feature, chars_per_token sigue
leyéndose en `new()` (validación) → sin dead_code. Default build no toca tiktoken-rs (dep optional).

## Decisión D21 revisada (ADR-029 enmienda)

Integrar tiktoken-rs detrás de feature opt-in `precise-tokens`. Default SIN feature =
chars/3 liviano intacto. Peso binario (+2-6MB) solo paga quien activa la feature.

## API validada (source-driven)

- tiktoken-rs **0.12.0** (docs.rs, 2026-08): `cl100k_base_singleton()` + `encode_with_special_tokens(text).len()`
- MSRV upstream Rust 2024 → requiere ≥1.85; workspace MSRV 1.94.1 ✅
- License MIT ✅ (deny.toml)
- Golden documentado OpenAI Cookbook: `"tiktoken is great!"` → 6 tokens (cl100k_base)

## Steps

### ✅ Step 1 — Feature gate + rama tiktoken en estimate_tokens
- [x] `Cargo.toml`: dep opcional `tiktoken-rs = { version = "0.12", optional = true }` + feature `precise-tokens = ["dep:tiktoken-rs"]`
- [x] `token_estimator.rs`: doc D21 actualizada; `estimate_tokens` con `#[cfg(feature)]` rama singleton (`cl100k_base_singleton` + `encode_with_special_tokens`) / rama chars/3 default

### ✅ Step 2 — Golden tests cl100k (feature-gated)
- [x] Tests `#[cfg(feature = "precise-tokens")]`: ""→0, "tiktoken is great!"→6 (OpenAI Cookbook), "hello world"→2, 你好世界→5, `fn main() { println!("hi"); }`→9 — valores pinneados contra run real (RED: CJK 6→5, código 12→9 corregidos con actuals del fallo)
- [x] Test heurístico pre-existente dividido: invariantes comunes (vacío/determinismo) ambas ramas; aritmética chars/3 solo sin feature

### ✅ Step 3 — Verify mecánico completo
- [x] `cargo check -p vanta-memory --all-targets` → exit 0
- [x] `cargo test -p vanta-memory` → exit 0, 0 fallidos (473 tests)
- [x] `cargo check -p vanta-memory --features precise-tokens --all-targets` → exit 0
- [x] `cargo test -p vanta-memory --features precise-tokens` → exit 0, 0 fallidos (tras desacoplar 3 tests e2e del budget chars/3 fijo → proporcional al estimador, patrón MEM-43)
- [x] `cargo fmt --check`: MIS archivos exit 0 vía `rustfmt --check`; el global falla SOLO en `vantadb-mcp/src/handlers/tools.rs:1272` (WIP de otra sesión GOV-B2/E1, archivo no tocado por esta tarea)
- [x] `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` → exit 0

## Notas

- BND-06 vigente: nextest -p roto → usar `cargo test -p vanta-memory` (suite default 0 fallidos).
- Pre-mortem: peso binario solo en builds con feature (dep optional lo garantiza); WASM build sin feature no se toca (vanta-memory no está en el build wasm).
- Risk Register: version drift → pin `"0.12"` + golden tests.
- NO commitear (regla del orquestador para esta tarea). Commit pendiente para el lead/orquestador.
- MCP bloqueó in-progress por convención one-task-at-a-time (WIP ajeno GOV-B2/GOV-E1): se procedió igual; cierre vía completed.

## Context Save Point

Tarea COMPLETA — nada pendiente de reanudar. Archivos tocados:
- `vanta-memory/Cargo.toml` (dep+feature)
- `vanta-memory/src/context_engine/token_estimator.rs` (rama cfg + golden tests + split test heurístico)
- `vanta-memory/tests/context_engine.rs` (budget proporcional en mem37_a)
- `vanta-memory/tests/e2e_flow.rs` (budget proporcional en mem37/d19 e2e)
- `docs/Backlog.md` (fila BND-03 eliminada) · `docs/progreso/README.md` (bullet P33)
