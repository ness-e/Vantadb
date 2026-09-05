# MOD-13: server sin TimeoutLayer - agregar timeout de request

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-core-server-mcp.md
- **Fuente:** plan file Task 4 (Wave 2)
- **Esfuerzo:** 🟡 1h
- **Prioridad:** 🟡
- **Tipo:** Rust (core server, red → FASE SECURITY)
- **Turns estimados:** 4
- **Creado:** 2026-08-25
- **last-synced:** 2026-08-25
- **Estado:** ✅ COMPLETED (implementación worker; commit + review del lead)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/cli_handlers/server.rs:250` (`cli_server::run`), `vantadb-server/src/{server,main}.rs` (re-export de `cli_server::{run, app, app_with_cors, ...}`), `desktop/src-tauri/tests/server_client_mock.rs` (espejo del contrato API), `tests/security*.rs` |
| Callees | `axum::Router` (0.8), `tower_http::trace/cors/fs` (0.6.11), `tower_governor::GovernorLayer`, `tokio::net::TcpListener`, `axum-server` (TLS) |
| Implicaciones | `app()` / `app_with_cors()` firma NO cambia (solo lógica interna de capas). Los routes se conservan 1:1. No rompe callers ni bindings. TimeoutLayer solo aplica a la capa de request future; no altera semántica de handlers. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/cli_server.rs` (4873L — router `app_with_cors` 204-326, `run` 1841-1905, `serve_http_or_tls` 2001-2074, test infra 2982+, body_limit test 3118-3174). `Cargo.toml` (axum 0.8 :61, tower 0.5 :62, tower_governor 0.8 :63, tower-http 0.6.11 :64). `.opencode/rules/server-mcp.md`.
- **Archivos referenciados hacia dentro:** `src/cli_server.rs` es el god-file HTTP: re-exportado por `vantadb-server/src/{server,main,middleware}.rs`, invocado por `cli_handlers/server.rs`, espejado por `desktop tests/server_client_mock.rs`, citado por `tests/security*.rs`.
- **Archivos que referencian a los editados:** `src/lib.rs:72` (`pub mod cli_server`); `vantadb-server/src/server.rs`/`main.rs`/`middleware.rs` (re-export); `cli_handlers/server.rs:250` (run). El único archivo editado es `src/cli_server.rs` (más `Cargo.toml` feature).
- **Veredicto impacto:** **bajo** — cambio aditivo de middleware interno en `app_with_cors`. No cambia API pública, no rompe callers, no requiere migración. Es fix de robustez/DoS (FASE SECURITY).

## Contrato
`cargo check -p vantadb --features server` pasa; test del timeout (request lento → 408, request rápido → 200, constantes sane) verde; `cargo fmt --check` y `cargo clippy -p vantadb --all-targets --features server -- -D warnings` pasan (solo archivos tocados).

## Invariantes de dominio (handoff - MUST)

- **Invariantes a preservar:** (1) todas las rutas existentes se mantienen 1:1 (ninguna se elimina); (2) auth middleware sigue cubriendo las rutas protegidas (incluidas las long-running); (3) rate-limit governor aplica al merge de todas las rutas protegidas; (4) `app()`/`app_with_cors()` firma intacta.
- **Comandos de verificación:** `cargo check -p vantadb --features server` ✅ · `cargo test -p vantadb --features server --lib cli_server::tests::` (timeout tests) ✅ · `cargo fmt --check` ✅ · `cargo clippy -p vantadb --all-targets --features server -- -D warnings` (solo archivos tocados) ✅
- **Deuda pendiente:** ninguna.

## Recitation (canónico - estructura única)

- `activeGoal`: MOD-13 — agregar TimeoutLayer de request al HTTP server (cli_server.rs) para que un handler atascado no retenga la conexión indefinidamente, excluyendo rutas long-running (import/export/rebuild-index) con timeout generoso.
- `lastAction`: Implementación completa — feature `timeout` en tower-http; `REQUEST_TIMEOUT=30s` + `LONG_REQUEST_TIMEOUT=600s`; sub-router long-running (export/import/rebuild-index) mergeado con auth+governor intactos; 3 tests nuevos. Verify: check server ✅, cli_server 42/42 ✅, vantadb-server 5/5 + e2e 12/12 ✅, fmt/clippy server ✅.
- `result`: `OK` (tarea completa; sin commit — regla sub-agentes)
- `nextAction`: Lead: verifica mecánico y commitea los archivos de MOD-13.
- `contract`:
  - `verificacion`: `cargo check -p vantadb --features server` ✅ · `cargo test -p vantadb --features server --lib cli_server::tests::` 42/42 ✅ (3 nuevos) · `cargo nextest run -p vantadb-server` 5/5 ✅ · `cargo test -p vantadb-server --test e2e` 12/12 ✅ · `rustfmt --check src/cli_server.rs` ✅ · `cargo clippy -p vantadb --features server --all-targets` ✅ 0 warnings
  - `evidencia`: claim: gap real (0 TimeoutLayer antes) — evidencia: rg TimeoutLayer src/cli_server.rs (vacío) + src/cli_server.rs:204-326 — confianza: alta; claim: TimeoutLayer 0.6.11 requiere feature timeout y usa with_status_code (new deprecado en 0.6.7) — evidencia: Cargo.toml:64 + registry tower-http-0.6.11/src/timeout/service.rs — confianza: alta; claim: request lento → 408 — evidencia: test slow_request_times_out_with_408 (42/42 verde) — confianza: alta; claim: 2 fallos rate-limit server.rs son PRE-EXISTENTES (no de MOD-13) — evidencia: git stash → cargo test -p vantadb-server --test server falla igual en base; FIND-32 en Backlog — confianza: alta
  - `artefactos`: `.opencode/skills/campaign-executor/tasks/MOD-13.md`, `docs/Backlog.md` (FIND-32)
  - `invariantes`: rutas 1:1 (ninguna eliminada); auth + governor cubren todas las rutas protegidas; firma `app`/`app_with_cors` intacta; public sin timeout (probes)
  - `deuda`: ninguna
  - `queda_pendiente`: lead commitea `Cargo.toml` + `src/cli_server.rs` (+ `docs/Backlog.md` FIND-32); FIND-32 pendiente de fix (tests stale rate-limit)
- `nextTask`: la que asigne el lead (MOD-10 / MCP-24).

## Deuda técnica (Regla 6 - MUST)

**Saldo neto de deuda por PR:** Sin deuda — cambio aditivo (1 feature + 1 layer + constantes + 3 tests). No introduce unsafe, clones ni deuda nueva.

## Steps

1. ✅ **DISCOVERY** — plan file, task spec, `.opencode/rules/server-mcp.md`, codegraph blast radius, task file creado con Regla 0. Gap confirmado: 0 `TimeoutLayer` en `cli_server.rs` (solo pool acquire timeout). Decisión documentada: 30s interactive / 600s long-running.
2. ✅ **Cargo.toml** — feature `"timeout"` agregado a `tower-http` (0.6.11). `TimeoutLayer::with_status_code` verificado en source local del crate (0.6.7+ deprecó `TimeoutLayer::new`; default 408).
3. ✅ **`src/cli_server.rs`** — constantes `REQUEST_TIMEOUT = 30s` y `LONG_REQUEST_TIMEOUT = 600s`; router `protected` (interactive) con `TimeoutLayer::with_status_code(408, 30s)` + auth; nuevo sub-router `long_running` (`/api/v2/export`, `/api/v2/import`, `/api/v2/maintenance/rebuild-index`) con auth + `TimeoutLayer(408, 600s)`; ambos mergeados ANTES del governor → rate-limit cubre todas las rutas protegidas. Public (`/health`, `/dashboard`) deliberadamente sin timeout (probes + static fs). Firmas `app`/`app_with_cors` intactas; rutas 1:1.
4. ✅ **Tests** (en `mod tests` de cli_server) — `request_timeouts_are_sane` (constantes: 0 < REQUEST_TIMEOUT ≤ 120s, LONG > REQUEST_TIMEOUT, LONG ≥ 300s); `slow_request_times_out_with_408` (handler 200ms vs timeout 50ms → 408); `fast_request_not_timed_out` (handler inmediato vs timeout 500ms → 200). Requieren `use tower::ServiceExt` para `oneshot`.
5. ✅ **Verify** — `cargo check -p vantadb --features server` ✅; `cargo test -p vantadb --features server --lib cli_server::tests::` 42/42 ✅; `cargo nextest run -p vantadb-server` 5/5 ✅; `cargo test -p vantadb-server --test e2e` 12/12 ✅ (incl. `test_e2e_rate_limit_over_http`); `cargo fmt --check` (solo `src/cli_server.rs`) ✅; `cargo clippy -p vantadb --features server --all-targets` ✅ 0 warnings.
6. ✅ **Cierre** — FIND-32 ruteado a `docs/Backlog.md` (2 tests unitarios rate-limit en `vantadb-server/tests/server.rs` obsoletos vs `rate_limit_burst` REST-01 burst=rpm, fallan en base — confirmado con `git stash`; ajenos a MOD-13). NO commit (regla: sub-agentes no commitean; lead verifica y commitea).

## Context Save Point

- **Decisión de diseño:** 30s interactive / 600s long-running (import/export/rebuild-index). El timeout corta la future del response (408) pero el `spawn_blocking` subyacente sigue hasta completar — es un bound de conexión, no de trabajo (correcto para DoS; el handler termina solo).
- **Orden de layers:** governor → auth → TimeoutLayer → handler (el timeout mide el trabajo del handler, no auth/probes).
- **No tocado (por diseño):** estructura del god-file (REVIEW-10 sigue abierto), rutas públicas, semántica de handlers, firma pública `app*`.
- **Hallazgos:** FIND-32 (tests stale rate-limit server.rs, pre-existente). FIND-30 (unused `ns` cli_server.rs:1302) ya estaba fixeado en el árbol (`_ns`).
- **Verificación exacta:** ver § Verify del step 5 + contrato abajo.
