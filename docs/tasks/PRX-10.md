# PRX-10 — Guardrails y MCP governance (tras PRX-03)

> **Plan:** docs/plans/2026-09-10-code.md Task 18 · **Estado:** ✅ COMPLETO
> **Ruta:** vanta-worker · **Appetite:** max 1d · **Wave:** W6 (secuencial interno con PRX-11: primero PRX-10)
> **SDP:** test-driven-development, incremental-implementation, context-engineering, security-and-hardening, api-and-interface-design, doubt-driven-development (keywords: guardrails, allowlists, virtual keys, moderation, governance)

## Gate P/D

- Gate P: owner aprobó plan 19 (Recomendado) 2026-09-10 — contrato "diseño previo + allowlists por key" ya aprobado.
- Gate D: no disparado — blast radius 4 archivos mismo crate (nuevo `guardrails.rs` + `config.rs` campo + `error.rs` variante + `server.rs` gate 1c); `pub` nuevos solo en `vanta-proxy` (no API pública core); precedente PRX-07/PRX-13 mismo patrón sin question.
- Stop condition plan: keys PRX-03 insuficientes → BLOQUEADO. **Verificado al inicio: NO BLOQUEADO** — `VirtualKey { id, budget_usd, enforce }` (cost.rs:125) + comentario "Extensible toward PRX-10 allowlists" (cost.rs:123) + `CostConfig` comment "Per-key budgets (PRX-10 allowlists) override this" (config.rs:158-161). `model: &str` ya resuelto en `process_inner` (server.rs:278).

## Spec (diseño previo — contrato lo exige)

| Decisión | Opción elegida | Evidencia / por qué |
|---|---|---|
| Alcance slice | Solo allowlist de modelos por key; moderación-provider = follow-up | Appetite 1d; "moderación conectable" queda como punto de extensión documentado en gate 1c, no implementado (scope discipline) |
| Dónde va el gate | `process_inner` gate **1c**: tras budget 1b, antes rate-limit 2 | 1b ya resuelve `user_id` autenticado; allowlist nunca autoriza (D34 intacto); corre antes de session/inject para rechazar barato |
| Default | `enabled=false` → transparente; key ausente o lista vacía → todo permitido | Paridad PRX-07/PRX-13 (transparent proxy); aditivo, TOML legacy intacto |
| Deny shape | 403 `guardrail_blocked` con `{key, model}` (identidad user_id, nunca secreto) | Paridad `BudgetExceeded` (solo user_id en 429); 403 = autorizado pero no permitido sobre ese modelo |
| Config TOML | `[guardrails] enabled + allowed_models = { "key-1" = ["gpt-4o-mini"] }` | `HashMap<String, Vec<String>>` serde directo |
| Fail mode | Fail-closed solo cuando `enabled=true` (política explícita del operador) | Distinto de cost fail-open: allowlist es policy opt-in, no telemetría |

## Impacto mapeado (Regla 0)

- **Leídos completos:** `vanta-proxy/src/cost.rs` (VirtualKey/check_budget), `config.rs` (ProxyConfig/CostConfig), `server.rs` (gates 1b/5a/5b, AppState), `error.rs` (BudgetExceeded/RedactionBlocked shapes), `lib.rs` (módulos).
- **Referencias hacia dentro (lo que toco):** `ProxyConfig` (nuevo campo `guardrails`), `ProxyError` (nueva variante), `AppState::process_inner` (nuevo gate 1c), `lib.rs` (`pub mod guardrails`).
- **Referencias entrantes:** `process_inner` llamado por handlers (chat/completions, messages, responses); `ProxyConfig::load` TOML; `AppState::from_engine` construye redactor/optimizer (gate 1c lee `self.config.guardrails` directo, sin estado — sin construcción extra).
- **Veredicto:** aditivo puro; ningún path existente cambia con `enabled=false` (default). Sin `dashmap`/`parking_lot`/Tokio nuevo → Regla 8 no dispara. Sin nuevas deps → `cargo deny` intacto.

## Steps

- [x] **S1:** RED `vanta-proxy/tests/prx10_guardrails.rs` (unit `check()` + error 403 shape + TOML compat) → falla E0432
- [x] **S2:** GREEN `guardrails.rs` + `ProxyError::GuardrailBlocked` + `config.guardrails` + `lib.rs`
- [x] **S3:** Gate 1c en `server.rs` + fix 15 inicializadores `ProxyConfig` en 13 tests + `cargo test -p vanta-proxy` 0 failed + clippy 0 + fmt limpio
- [x] **S4:** REVIEW 3 ejes + verify full + commit solo-propio + lessons + plan inline

## Cierre (2026-09-10)

- Gate 1c verifica el **modelo pedido** (pre-routing PRX-06): key con allowlist que pide modelo fuera de política → 403 aunque el fork lo hubiera reescrito a uno barato. Deny conservador = safe default documentado.
- `authenticate()` extra en 1c = paridad con patrón 1b (lookup barato, sin estado nuevo).
- Incidente git: `git stash pop` accidental sobre stash ajeno `wip-antes-deps-2026-09-03` → reparado (`checkout HEAD` del archivo en conflicto + borrados 2 `.budget.json` restaurados por el pop). Lección: nunca `stash` en worktree con WIP ajeno; verificar con `git stash list` antes.

## Contrato verify

`cargo test -p vanta-proxy` 0 failed + `cargo clippy -p vanta-proxy --all-targets --all-features -- -D warnings` 0 + `cargo fmt --check` scoped limpio.
