# FIND-65 — `checked_sub` en test TTL (`vanta-proxy/src/cache.rs`)

> Campaign: `6ab26f3f-cf16-4416-9255-c18cca0bcaf0` · Plan: `docs/plans/2026-09-15-find-correcciones.md`
> Estado: ⬜ PENDING → IN PROGRESS (Wave1, disjunto de FIND-63 desktop y FIND-88 server.rs)
> Appetite: 2h · Esfuerzo: 🟢 · Prioridad: 🟠 · Ruta: vanta-worker
> Branch: `develop` · Commit previsto: `fix: FIND-65 — ...` (diff solo-test)
> nextTask: FIND-88
> Sin símbolos nuevos → sin Spec (fix de test sobre comportamiento existente).

## SDP

`campaign_discover_skills_v2 archivosClave="vanta-proxy/src/cache.rs" phase="BUILD" contractKeywords=["TTL","Instant","checked_sub","cache"]` →
base: campaign-executor, source-driven-development, progreso +
lifecycle: incremental-implementation, test-driven-development, context-engineering,
doubt-driven-development (+ frontend-ui-engineering, api-and-interface-design descartadas por inaplicables).
Cargadas: systematic-debugging, test-driven-development, incremental-implementation, context-engineering.
SKILLS_CARGADAS base sesión: campaign-executor, brainstorming, writing-plans,
planning-and-task-breakdown, progreso, ponytail(full).
`SDP: systematic-debugging + test-driven-development + incremental-implementation + context-engineering (beyond base; frontend-ui/api descartadas)`

## Gate D (question-gates.md)

Blast radius = 1 test fn + suite del crate. Sin símbolos públicos nuevos, sin hot path,
sin API pública, contrato no ambiguo, fix (no feature-add) → Gate D **no disparado**, sin `question`.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos (slice):** `vanta-proxy/src/cache.rs:1-30` (imports, `TTL_DISABLED=ZERO`),
  `:106-109` (prod `is_expired` — solo lectura), `:751-764` (test `ttl_expiry_predicate`);
  reglas `.opencode/rules/server-mcp.md` (proxy: R-1..R-3 no aplican — sin handlers/métricas/versiones);
  `codegraph_explore "vanta-proxy cache is_expired TTL_DISABLED Instant"`.
- **Referencias hacia dentro (lo que el test toca):** `is_expired(inserted_at, ttl)` (privada, mismo archivo),
  `TTL_DISABLED` (`super::TTL_DISABLED`), `std::time::{Duration, Instant}`.
- **Referencias entrantes al test:** ninguna (los tests no tienen callers; `is_expired` prod tiene
  3 callers en `cache.rs`: `lookup`, `lookup_similar`, `lookup_similar_lexical` — ninguno afectado
  porque el fix no toca prod).
- **Veredicto:** impacto = 1 fn de test + suite `cache` del crate. Prod `:107-108` **prohibido editar**.
  `thread.rs:327 is_expired` es otro símbolo homónimo no relacionado (wall-clock, no Instant).

## Root cause (systematic-debugging Fase 1)

`Instant` es monotónico desde boot. `Instant::now() - Duration::from_secs(10_000)` usa el
impl `Sub<Duration>` que **paniquea** si la duración excede el tiempo desde boot. En hosts con
uptime < 10_000 s (~2.7 h) el test paniquea en `:761` ANTES de entrar a `is_expired`.
Prod ya es seguro (`inserted_at.elapsed() >= ttl` nunca paniquea). Bug solo-test.
Hipótesis única: `checked_sub(...).unwrap_or_else(Instant::now)` nunca paniquea y preserva
los 3 asserts (el caso `TTL_DISABLED` da `false` para cualquier `Instant`; el caso "expirada"
se garantiza con `sleep` + TTL diminuto en vez de resta grande).

## Steps

- [x] **Step 1 (único, ~15 líneas):** PLAN→ACT→VERIFY — `ttl_expiry_predicate` reescrito con
  `checked_sub` + `sleep(10ms)` + TTL `1ms` en caso expirada;
  `checked_sub(10_000s).unwrap_or_else(Instant::now)` en caso `TTL_DISABLED`;
  caso viva intacto. ✅
  - `cargo test -p vanta-proxy cache -j 2` → 18/18 ✅ (`ttl_expiry_predicate ok`)
  - `cargo fmt --check -p vanta-proxy` → OK ✅
  - `cargo clippy -p vanta-proxy --all-targets` → 0 warnings en `vanta-proxy` ✅
  - `cargo clippy -p vanta-proxy --all-targets -- -D warnings` → ❌ pre-existente
    (`vantadb` lib `txn.rs:158` `cloned_sole_buffer` dead_code; repro sin fix vía
    `git stash` → mismo error) → colateral → `FIND-93` en Backlog, fuera de scope.
  - `rg` → 2× `checked_sub`, 0 restas `Instant - Duration` en el test ✅
  - Review (code-review-and-quality, self): correctness/readability/architecture/security/performance ✅ → **Approve** (diff 17 líneas solo-test; +10ms en 1 test).
  - DoD: contrato cumplido salvo clippy-`-D` global (bloqueado por FIND-93); sin deuda neta;
    WIP ajeno intacto.

## Contrato

- Test `ttl_expiry_predicate` verde en host joven (o con uptime simulado: `checked_sub` cubre
  el path `None` por construcción — sin resta que paniquee queda demostrado).
- Suite `cache` del crate verde (`cargo test -p vanta-proxy cache -j 2`).
- `cargo clippy -p vanta-proxy --all-targets -- -D warnings` → 0 warnings.
- `cargo fmt --check` limpio; `git diff` = solo `vanta-proxy/src/cache.rs` bloque test.
- Los 3 casos sobreviven: expirada / viva / `TTL_DISABLED`.

## Save Point

- Pre-fix: `rg` confirma 1 ocurrencia `from_secs(10_000)` en `:761`; prod `:107-108` con `elapsed()` OK.
- WIP ajeno intacto (no tocar): `.opencode`, `completions/`, `desktop/src-tauri/Cargo.lock`,
  `docs/pipeline-state.json`, plan file untracked.
