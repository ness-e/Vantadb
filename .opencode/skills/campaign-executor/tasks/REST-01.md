# REST-01 — Calibrar rate limiter para ráfagas UI (429 en consola)

> **Plan:** `docs/plans/2026-08-19-vanta-studio-fase4.md` — Wave 1
> **Estado:** ✅ COMPLETO (worktree listo, sin commit — verify/commit del lead)
> **Fecha:** 2026-08-19

## Contrato

- Default del rate limiter revisado para ráfagas UI locales: rpm 600 (antes 100) con burst = rpm completo sin auth (loopback/dev), burst = rpm/10 con auth
- Env var `VANTADB_RATE_LIMIT_RPM` documentada (config.rs docstring, SECURITY.md, HTTP_API.md, .env.tokens.example)
- Respuesta 429 con header `Retry-After` y body `{success:false,error}` (consistente con `vanta_error_response`)
- NO relajar si `require_auth` activo (fail-closed AUD-021): `require_auth` ⇒ `api_key` seteado en startup (validate_startup_config L1418-1430) ⇒ burst conservador
- Tests de ráfaga: 20 requests consecutivas sin 429 en loopback sin auth
- E2E sin `VANTADB_RATE_LIMIT_RPM=0` (se eliminó el escape del harness)

## Impacto mapeado (Regla 0)

- **Leídos completos:** `src/cli_server.rs` (rate_limit_period_ms/burst L127-183, app_with_cors governor L260-280, tests L3020-3105, validate_startup_config L1418-1430, run_server L1536), `src/config.rs` (HotReloadConfig default L148, docstring L297-303, parse_env_or L658, test L1323), `docs/api/HTTP_API.md` (Rate Limiting L139/L171), `docs/operations/SECURITY.md` (L135-141), `.env.tokens.example` (L85), `desktop/scripts/selfcheck-web-e2e.ts`
- **Referencias entrantes:** `run_server` (src/cli_server.rs L1536) consume `config.rate_limit_rpm` → `app(state, rpm)`; E2E `selfcheck-web-e2e.ts` arranca el server y hace ráfagas UI reales
- **Referencias salientes:** `tower_governor` (`GovernorConfigBuilder`, `GovernorLayer`, `GovernorError`), helpers `rate_limit_period_ms`/`rate_limit_burst`/`rate_limit_error_response` (solo usados en cli_server.rs)
- **Veredicto:** cambio acotado al rate limiter + docs + E2E harness; no toca endpoints existentes, no toca `src/wal.rs`/`src/vector/`/`src/storage/`

## Steps

- [x] 1. Verificar cómo se lee `VANTADB_RATE_LIMIT_RPM` hoy: `parse_env_or("VANTADB_RATE_LIMIT_RPM", 600)` en `VantaConfig::default()` (src/config.rs L658) — leído en startup, default ahora 600
- [x] 2. Default rpm 100 → 600 (config.rs: HotReloadConfig L148, parse_env_or L658, test L1323) + docs (SECURITY.md, .env.tokens.example)
- [x] 3. Burst: `rate_limit_burst(rpm, auth_active)` — sin auth burst = rpm completo (ráfaga consola ~12 reqs nunca 429), con auth burst = rpm/10 (AUD-021 fail-closed)
- [x] 4. `rate_limit_error_response`: 429 con headers de tower_governor (incluye `retry-after`) + body `{success:false,error}` — shape consistente con `vanta_error_response`
- [x] 5. Governor layer con `error_handler(rate_limit_error_response)` (antes default de tower_governor)
- [x] 6. Tests: `governor_config_always_builds_for_positive_rpm` (rpm 1..=10000 × auth/no-auth), `rate_limiter_allows_ui_burst_without_auth` (20 reqs → 0×429), `rate_limiter_stays_conservative_with_auth` (429 + Retry-After + shape JSON)
- [x] 7. E2E `selfcheck-web-e2e.ts`: eliminar escape `VANTADB_RATE_LIMIT_RPM=0` + comentario stale
- [x] 8. Docs HTTP_API.md: default 600 + burst behavior documentado (dos secciones Rate Limiting stale corregidas)
- [x] 9. Verify: `cargo test --features server` verde
- [x] 10. Verify: build bin vanta-cli con server feature + smoke 20 GETs → 0×429
- [x] 11. Verify: E2E sin `VANTADB_RATE_LIMIT_RPM=0` pasa

## Verify

- `cargo test --features server cli_server` — ✅ EXIT=0, 28/28 (incluye los 3 tests REST-01)
- `cargo test --features server --lib` — ✅ EXIT=0, 1831 passed / 0 failed (incluye config default-600)
- `cargo fmt --check -p vantadb` — ✅ EXIT=0 (5 diffs de formato del worktree aplicados con `cargo fmt`)
- Smoke: 20 GETs secuenciales a `/api/v2/health` (server local, DB temp, default 600 rpm) → 20×200, 0×429 — ✅
- `node scripts/selfcheck-web-e2e.ts` (desktop/) sin `VANTADB_RATE_LIMIT_RPM` override — ✅ EXIT=0, 11/11 checks PASS
- Límite: el comando literal `cargo test --features server` (workspace completo) incluye el tier Heavy de certificación (benchmark_internal_10k, prefetch_benchmark, memory_brutality, fuzz_proptest…) que excede timeouts de sesión (CI_POLICY: hasta 2 hr, manual/scheduled). La superficie REST-01 (cli_server + config) está cubierta por los dos runs verdes de arriba; el lead puede correr el suite completo antes del commit.

## Notas

- NO commit — vanta-lead hace verify + commit (Regla del plan: sub-agentes no commitean)
- El worktree ya contenía el trabajo parcial de REST-01..03 de un intento previo; REST-01 se validó contra contrato, se completaron los gaps (E2E escape + docs HTTP_API.md) y se corrió verify
- Invariante fail-closed: `require_auth` force `api_key` en startup (`validate_startup_config`) → el burst relajado solo aplica sin auth (loopback/dev)
- 429 mantiene headers de tower_governor (`retry-after`/`x-ratelimit-after`) verbatim + body JSON propio