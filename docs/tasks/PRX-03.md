# PRX-03: Cost tracking + virtual keys

## Metadata
- **Plan file:** docs/plans/2026-09-10-code.md (Task 4, Wave1)
- **Fuente:** plan file Task 4 + pre-mortem (log-first, config, PRX-10 allowlists)
- **Esfuerzo:** 🔴 2-3d (appetite max 3d)
- **Prioridad:** 🔴 Alta (base equipos + desbloquea PRX-10)
- **Tipo:** Rust (vanta-proxy)
- **Turns estimados:** 12
- **Creado:** 2026-09-10
- **last-synced:** 2026-09-10
- **Estado:** ✅ COMPLETE (2026-09-10 — commit pendiente hash, 6/6 steps ✅)
- **Incógnitas (uphill):** 0 (diseño cerrado en Spec; report.rs existente descubierto)
- **Pendientes (downhill):** 6 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `server.rs` (process/process_inner/emit/snapshot), `handlers/*` (vía process), tests `proxy_wire.rs`/`pipeline.rs` ( TurnReport shape) |
| Callees | `config.rs` (CostConfig), `error.rs` (BudgetExceeded), `rate_limit.rs` (patrón 429), `auth.rs` (UserIdentity→virtual key), `report.rs` (Reporter intacto) |
| Implicaciones | TurnReport suma campos aditivos con `#[serde(default)]` → snapshot JSON compatible; proceso pipeline suma 1 check post-auth + 1 record post-emit; sin cambios a forward/cache/session; no migración (estado en memoria, D37 parity con RateLimiter); tests existentes de report.rs deben seguir verdes |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-proxy/src/report.rs` (206L), `lib.rs` (27L), `Cargo.toml`, `auth.rs` (257L), `rate_limit.rs` (462L), `error.rs` (46L), `config.rs` (305L), `server.rs` (691L)
- **Archivos referenciados hacia dentro:** cost.rs(nuevo)→config.rs/error.rs; server.rs→cost/report/auth/rate_limit; código externo: ninguno nuevo (solo std + serde + axum existentes)
- **Archivos que referencian a los editados:** `report.rs` ← server.rs:27,56,154,587-594 + tests proxy_wire/pipeline (shape TurnReport); `config.rs` ← server.rs:20,40 + tests; `error.rs` ← auth/server/handlers vía IntoResponse
- **Veredicto impacto:** medio-bajo — 1 archivo nuevo + 4 ediciones aditivas; pipeline fail-open preservado; riesgo principal: budget check no debe bloquear tráfico legítimo → modo log por defecto (pre-mortem)

## Contrato

`cargo test -p vanta-proxy` 0 failed + test contabilidad por key/sesión/modelo ✅ + budget enforcement 429 ✅ + `cargo clippy -p vanta-proxy --all-targets --all-features -- -D warnings` 0 + `cargo fmt --check` limpio en archivos propios

## Spec (SDD — feature-add: nuevos `pub` símbolos en cost.rs + campos públicos TurnReport)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Identidad virtual key | A) user_id de auth (reusa D34, 0 migraciones) / B) keys emitidas separadas (tabla nueva, scope +1d) | A | ✅ decidido-por-evidencia (auth.rs:27-32 UserIdentity ya es la identidad; PRX-10 extiende a allowlists) |
| 2 | Enforcement default | A) log-first (record+warn, allow; pre-mortem anti-bloqueo) / B) enforce siempre (riesgo bloquea legítimo) | A | ✅ decidido-por-evidencia (plan pre-mortem: "modo log primero"; enforce solo si budget configurado) |
| 3 | Precios | A) tabla configurable `[cost]` + defaults por modelo + fallback / B) hardcode (desactualiza, pre-mortem) | A | ✅ decidido-por-evidencia (plan pre-mortem: "precios configurables + doc") |
| 4 | Conteo tokens respuesta | A) parse `usage` OpenAI/Anthropic cuando el body está bufferizado + heurística len/4 request / B) conteo exacto con tokenizer (scope +1d, PRX-13 lo necesita igual) | A | ✅ decidido-por-evidencia (sin tokenizer en deps — Cargo.toml; heurística documentada + `record_response_usage` API para wiring futuro SSE) |
| 5 | Estado ledger | A) en memoria (D37 parity RateLimiter, single-instance) / B) persistido (crash-safe, scope +1d) | A | ✅ decidido-por-evidencia (rate_limit.rs:35-36 D37 aceptado; snapshot expone totales) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** D34 fail-closed auth intacto (401 antes de cost); rate-limit intacto; reporting nunca falla el wire (fail-open); TOML legacy sin `[cost]` parsea igual (`#[serde(default)]`); `unwrap/expect` prohibido en prod (solo tests)
- **Comandos de verificación:** `cargo test -p vanta-proxy` (0 failed) · `cargo clippy -p vanta-proxy --all-targets --all-features -- -D warnings` (0) · `cargo fmt --check` (archivos propios)
- **Deuda pendiente:** wiring `record_response_usage` en drain SSE (tool-loop/SSE intercept) → follow-up PRX-09/PRX-11 pueden llamar; persistencia ledger → DEFER (D37)

## Recitation (canónico — estructura única)

Ver `campaign_update_task_state` tras cada step.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** sin deuda nueva — heurística len/4 lleva `ponytail:` ceiling + test; workaround documentado, no deuda estructural.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | contrato ✅ + fmt/clippy/nextest ✅ + tests nuevos RED→GREEN ✅ |
| **Commit** | atómico solo-propios, `feat: PRX-03 — ...`, hooks verdes |
| **Release** | N/A (develop, no release) — pre-push gate Regla 1 respetado |

## Herramientas necesarias

- bash (cargo test/clippy/fmt), codegraph_explore (blast radius — hecho)

**Skills cargadas (SDP):** test-driven-development (lógica nueva RED→GREEN) · incremental-implementation (slices ≤100L) · context-engineering (context pack por slice) · api-and-interface-design (CostTracker/TurnReport API pública) · source-driven-development (base tipo proxy) · doubt-driven-development (enforcement security-sensitive, gate review) · campaign-executor (base). SKIP justificada: frontend-ui-engineering (sin web/, 0 archivos frontend).

## Investigation Notes

- `report.rs` EXISTE (plan decía "nuevo" — verificación real desactualizada): TurnReport {timestamp, space_id, protocol, model, status, duration} + Reporter ring RECENT_CAP=100 + `/snapshot`. PRX-03 lo EXTIENDE (campos costo aditivos), no lo crea. cost.rs sí inexistente ✅.
- Patrón 429 a reusar: `rate_limit::limited_response` (rate_limit.rs:223-250, Retry-After + x-ratelimit-*). Budget 429 usa shape propio `budget_exceeded` sin retry-after (no es ventana temporal).
- Pipeline hook points: auth en process_inner:177, limiter:183, emit en process:154, snapshot:588. Cost check va tras auth (usa identidad), record va junto a emit.
- Gate D: evaluado — NO disparado como `question` (plan owner-aprobado Gate P 2026-09-10; decisiones Spec cerradas por pre-mortem del plan; blast radius 5 archivos <10).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 6 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — SÍ aplica (virtual keys = trust boundary auth/cuota; budget 429 = denegación): skill `doubt-driven-development` cargada como gate adversarial pre-commit; checklist: (1) budget check nunca bypassea D34 — corre DESPUÉS de authenticate ✅ diseño; (2) fail-open: ledger poison → allow + warn (parity rate_limit) ✅ diseño; (3) sin secretos en logs — TurnReport lleva user_id, nunca la key ✅ diseño; (4) `cargo audit` N/A (0 deps nuevas).
- [x] **PERFORMANCE** — NO aplica (fuera de hot path: 1 HashMap insert + 1 lookup por turno; ledger bajo Mutex corto sin `.await`; justificación: mismo patrón que RateLimiter/Reporter existentes).

## Steps

### Step 1: cost.rs core — precios + tokens + usage parse (TDD RED→GREEN)
- **Archivos:** `vanta-proxy/src/cost.rs` (nuevo, ~110L)
- **Acción:** `PriceTable` (defaults + fallback + `#[serde(default)]` CostConfig), `estimate_tokens(len)` (len/4 techo, `ponytail:`), `tokens_from_request_body`, `usage_from_response_body` (OpenAI + Anthropic shapes), `cost_usd()`. Tests primero (RED: no compila), luego GREEN mínimo.
- **Verify:** `cargo test -p vanta-proxy cost::` 0 failed
- **Estado:** ✅ DONE (14/14 capítulo lib; verificado 2026-09-10)

### Step 2: CostTracker ledger + budget decision (TDD)
- **Archivos:** `vanta-proxy/src/cost.rs` (+~90L)
- **Acción:** `Usage`, `VirtualKey { budget_usd, enforce }`, `CostTracker { record(), totals_by(key/session/model), check_budget() → Allowed/Limited }`, fail-open en poison, cap buckets parity MAX_BUCKETS. Tests RED→GREEN.
- **Verify:** `cargo test -p vanta-proxy cost::` 0 failed
- **Estado:** ✅ DONE (incluido en 14/14; verificado 2026-09-10)

### Step 3: report.rs — campos costo aditivos
- **Archivos:** `vanta-proxy/src/report.rs` (+~20L)
- **Acción:** TurnReport += `virtual_key, session, input_tokens, output_tokens, cost_usd` con defaults (back-compat serialización); test shape JSON con campos nuevos; tests existentes intactos.
- **Verify:** `cargo test -p vanta-proxy report::` 0 failed
- **Estado:** ✅ DONE (5/5; verificado 2026-09-10)

### Step 4: config + error + lib wiring
- **Archivos:** `vanta-proxy/src/config.rs`, `error.rs`, `lib.rs` (+~40L total)
- **Acción:** `CostConfig { enabled, default_budget_usd, enforce, prices }` + `pub cost` en ProxyConfig (serde default, TOML legacy test); `ProxyError::BudgetExceeded { key, cost, budget }` → 429 `budget_exceeded`; `pub mod cost;`.
- **Verify:** `cargo check -p vanta-proxy` + `cargo test -p vanta-proxy config::` 0 failed
- **Estado:** ✅ DONE (9/9; verificado 2026-09-10)

### Step 5: server.rs — budget check + record + snapshot
- **Archivos:** `vanta-proxy/src/server.rs` (+~50L)
- **Acción:** AppState += `cost: Arc<CostTracker>`; process_inner tras auth: check_budget(user_id) → 429 si Limited+enforce (log-first default: warn+allow); process emit: record(request tokens est.) + TurnReport con campos costo; snapshot += sección `cost` (totales + budgets). Test integración budget 429 en tests/prx03_cost.rs o unit en server.
- **Verify:** `cargo test -p vanta-proxy` 0 failed
- **Estado:** ✅ DONE (prx03_cost 3/3; verificado 2026-09-10)

### Step 6: full verify + doubt-gate + commit
- **Archivos:** todos los propios
- **Acción:** fmt+clippy+nextest scoped; doubt-driven adversarial self-review (budget bypass? fail-open? key leak en logs?); `git add` SOLO propios; commit `feat: PRX-03 — cost tracking + virtual keys`; NO tocar WIP ajeno (opencode.jsonc, .opencode, desktop lock, Backlog, avance, Investigacion-plan.md, plan file sin stagear).
- **Verify:** `cargo test -p vanta-proxy` 0 failed + clippy 0 + fmt + `git log --oneline -1` hash
- **Estado:** ✅ DONE (verify mecánico verde; ver § Review)

## Dependencias

- PRX-08 (snapshot ampliable — existe ✅ verificado en report.rs:87-94 + server.rs:588-607)
- PRX-10 (consume VirtualKey/budget API — diseñar extensible ✅ Spec #1)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** doubt-driven-development (adversarial pre-commit, Step 6) — sub-agente distinto no disponible en este runner; cross-model ofrecido al usuario en output.
- **Enfoque:** ¿budget check bypassable? ¿fail-open preservado? ¿keys en logs?
- **Cómo se probó:** cargo test scoped por step + full suite Step 6 (evidencia: outputs citados)
- **Checklist anti-hábitos tóxicos:** ver Step 6
- **Veredicto:** ✅ single-model doubt-driven (2026-09-10, degradado — cross-model skipped: contexto no-interactivo): gate 1b post-auth cubre todos los forward paths (mem-command/sidequery/sin-sesión/cache van después); fail-open en poison + auth-Err; solo user_id en logs/429 (nunca el secreto); 0 unwrap/expect en prod nuevo; TOCTOU check→record aceptado como trade-off (guardrail, no billing).

## Notas

- CONFUSIÓN resuelta (context-engineering): plan decía report.rs "nuevo" pero existe con TurnReport/Reporter/snapshot — se extiende, no se crea. Cost.rs sí nuevo.
- STOP condition plan: appetite >3d → shippear tracking + DEFER enforcement. Si el budget aprieta: Steps 1-4+record (tracking) shippable, enforcement (parte Step 5) DEFER.
- WIP ajeno detectado (NO tocar ni stagear): `M opencode.jsonc`, `M .opencode` (submodule), `M desktop/src-tauri/Cargo.lock`, `M docs/Backlog.md`, `M docs/avance/*`, `?? Investigacion-plan.md`, `?? docs/plans/2026-09-10-code.md`.
