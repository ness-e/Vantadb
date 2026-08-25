# FIND-32: tests unitarios rate-limit obsoletos vs burst=rpm (REST-01)

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md (Task 3)
- **Fuente:** hallazgo MOD-13 2026-08-25 (tests stale rate-limit en server.rs, fallan en base)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Tipo:** Rust (fix de TEST — sin lógica de negocio)
- **Creado:** 2026-08-25
- **Estado:** ✅ COMPLETED (implementación worker; commit + review del lead)
- **Incógnitas (uphill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | ninguno — los 2 tests son funciones `#[tokio::test]` standalone; ningún otro código los invoca |
| Callees | `app(state, rpm)` re-export de `vantadb::cli_server::app`; `rate_limit_burst` (src/cli_server.rs:172); `rate_limit_period_ms` (src/cli_server.rs:161); governor (tower_governor, SmartIpKeyExtractor) |
| Implicaciones | Solo cambia el cuerpo de 2 tests. No toca producción, API pública ni bindings. Sin migración. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-server/tests/server.rs` (617L completo: helpers `build_context` :33, `add_addr` :38, `get` :49, `post_query` :60; tests rate-limit :211-250; resto auth/RBAC/concurrency/circuit-breaker/TLS/health sin tocar). `src/cli_server.rs` § rate limit (:160-178 `rate_limit_period_ms` + `rate_limit_burst`; wiring :316-334 governor sobre `protected`; tests governor :4127-4169). `vantadb-server/tests/e2e.rs` `test_e2e_rate_limit_over_http` (:265-309, referencia MOD-14). `vantadb-server/src/server.rs` (re-export, 4L). `.opencode/rules/server-mcp.md` (28L). `MOD-13.md` (contexto hallazgo).
- **Archivos referenciados hacia dentro:** ninguno referencia los 2 tests. El plan file cita `vantadb-server/tests/server.rs:223,235`; `docs/Backlog.md` tiene el FIND-32; `MOD-13.md` recitation lo menciona.
- **Archivos que referencian a los editados:** solo el plan file y el task file. `vantadb-server/tests/helpers/mod.rs` (`build_server_state`, no se modifica).
- **Veredicto impacto:** **bajo** — fix de tests obsoletos. No cambia producción ni API. Los tests quedan alineados al comportamiento REAL del governor (burst=rpm sin auth, REST-01) como ya lo hace el e2e endurecido MOD-14.

## Contrato
`cargo test -p vantadb-server --test server test_rate_limit` pasa (3/3: disabled + enforces_after_burst + health_unaffected) + `cargo check -p vantadb-server` + fmt/clippy del archivo sin warnings nuevos.

## Invariantes de dominio (handoff - MUST)

- **Invariantes a preservar:** (1) no cambiar producción (`src/cli_server.rs` intacto); (2) tests reflejan el comportamiento real del governor — el 6º request (burst=5 + 1) debe ser 429, no debilitar a "acepta todo"; (3) `/health` sigue exento (ruta pública, sin governor); (4) sin timing flaky: requests rápidos dentro de la ventana de replenish de 12s.
- **Comandos de verificación:** `cargo test -p vantadb-server --test server test_rate_limit` ✅ · `cargo check -p vantadb-server` ✅ · `cargo fmt --check` (tests/server.rs) ✅ · `cargo clippy -p vantadb-server --all-targets` ✅
- **Deuda pendiente:** ninguna.

## Decisión de diseño (discovery)

El governor sin auth (dev mode, REST-01) da burst = rpm completo (`rate_limit_burst` :172-178). Con rpm=5 → burst=5, period=12s, key per-IP (`SmartIpKeyExtractor`). `/health` es ruta pública → exenta.

**Opción elegida: A — alinear los tests a burst=rpm sin auth** (no usar auth para forzar burst=1). Razones:
- Consistente con el test e2e endurecido MOD-14 (`test_e2e_rate_limit_over_http`): no-auth + burst=rpm + burst 2x → ≥1 200 y ≥1 429. Es la forma canónica del codebase de probar burst→429.
- Prueba el caso real dev-mode (el que corre sin API key), no el postura fail-closed.
- Sin auth, el test conserva la estructura mínima (misma key de limiter para todos los requests → determinístico).

Forma nueva: `BURST = rpm = 5` requests pasan (200), el request N+1 (6º) → 429. Para `test_rate_limit_health_unaffected`: /health 200 → 5×POST 200 → 6º POST 429 → /health 200 (exento aún con governor tripeado).

## Recitation (canónico - estructura única)

- `activeGoal`: FIND-32 — alinear los 2 tests unitarios rate-limit obsoletos de `vantadb-server/tests/server.rs` (:223, :235) al comportamiento real del governor (burst=rpm sin auth, REST-01).
- `lastAction`: Implementación completa — reescritos `test_rate_limit_enforces_after_burst` y `test_rate_limit_health_unaffected` a burst=rpm sin auth (BURST=5 → 5×200 + 429; /health exento antes/después). Sin tocar producción. Verify: `cargo test -p vantadb-server --test server test_rate_limit` 3/3 ✅; `--test server` completo 19/19 ✅; check ✅; fmt limpio; clippy 0 warnings.
- `result`: `OK` (tarea completa; sin commit — regla sub-agentes)
- `nextAction`: Lead: verifica mecánico y commitea `vantadb-server/tests/server.rs`.
- `contract`:
  - `verificacion`: `cargo test -p vantadb-server --test server test_rate_limit` ✅ 3/3 PASS · `cargo test -p vantadb-server --test server` ✅ 19/19 · `cargo check -p vantadb-server` ✅ · `cargo fmt --check` ✅ · `cargo clippy -p vantadb-server --all-targets -- -D warnings` ✅ 0 warnings
  - `evidencia`: claim: fallaban en base — evidencia: `cargo test -p vantadb-server --test server test_rate_limit` → 1 passed, 2 failed (left 200, right 429) — confianza: alta; claim: burst sin auth = rpm — evidencia: `src/cli_server.rs:172-178` (rate_limit_burst) + test e2e MOD-14 (`vantadb-server/tests/e2e.rs:265`) — confianza: alta; claim: /health exento del governor — evidencia: `src/cli_server.rs:290-336` (public vs protected) — confianza: alta; claim: tests reflejan comportamiento real (6º request 429) — evidencia: 3/3 PASS tras alinear a BURST=5 — confianza: alta
  - `artefactos`: `.opencode/skills/campaign-executor/tasks/FIND-32.md`, `vantadb-server/tests/server.rs`
  - `invariantes`: producción intacta (`src/cli_server.rs` sin cambios); 6º request 429 (no debilitado); /health exento; determinístico (requests rápidos dentro de la ventana de 12s)
  - `deuda`: ninguna
  - `queda_pendiente`: lead verifica y commitea `vantadb-server/tests/server.rs`
- `nextTask`: la que asigne el lead.

## Deuda técnica (Regla 6 - MUST)

**Saldo neto:** cero — fix de tests existentes, sin deuda nueva.

## Steps

1. ✅ **DISCOVERY** — plan file, reglas server-mcp, governor real (cli_server.rs:160-178, 316-334), e2e referencia MOD-14, task file creado con Regla 0. Falla en base confirmada.
2. ✅ **Implementación** — reescritos los 2 tests en `vantadb-server/tests/server.rs` a burst=rpm (BURST=5 → 5×200 + 429, /health exento). Sin tocar producción.
3. ✅ **Verify** — `cargo test -p vantadb-server --test server test_rate_limit` 3/3 ✅ · `--test server` 19/19 ✅ · `cargo check -p vantadb-server` ✅ · `cargo fmt --check` ✅ · `cargo clippy -p vantadb-server --all-targets -- -D warnings` ✅ 0 warnings.
4. ⬜ **Cierre** — SIN commit (regla: sub-agentes no commitean; lead verifica y commitea). Task file actualizado con recitation OK.
