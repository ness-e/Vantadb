# SRV-06: OIDC/JWT authentication (DISCOVERY-first)

## Metadata
- **Plan file:** docs/dev/plans/2026-09-10-code.md (Task 8, Wave2 con PRX-06 + PROV-11, disjuntos)
- **Fuente:** docs/dev/Backlog.md fila SRV-06 + plan Task 8
- **Esfuerzo:** 🔴 2-3d (appetite max 3d; MVP slice ≪ appetite)
- **Prioridad:** 🟡 Media (enterprise)
- **Tipo:** Rust (src/server/ + src/config.rs)
- **Turns estimados:** 15-20
- **Creado:** 2026-09-10
- **Estado:** ✅ COMPLETED (commit a0a3087f, 20 archivos solo-propios)
- **Incógnitas (uphill):** 0 (HS256 gana por evidencia; OIDC DEFER documentado en ADR-039)
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-server` bin, `tests/server_auth_rotation.rs`, `tests/rbac_namespace.rs`, `tests/request_id.rs`, `vanta-memory/tests/conversation_hook.rs` (vía shim `cli_server` re-export) |
| Callees | `jsonwebtoken` (nueva dep opcional, `rust_crypto` backend puro-Rust), `Rbac`, `EntityStore` (L3 intacto), `AuditLogger` |
| Implicaciones | Sin cambio de comportamiento cuando `jwt_secret` unset (fallback inactivo); 401 body idéntico (sin oráculo); struct `ServerState`/`AuthState` ganan 1 campo `Option` aditivo; nueva dep solo bajo feature `server` |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/server/middleware.rs` (auth_middleware:42), `src/server/state.rs` (AuthState:188, ServerState:107), `src/server/mod.rs`, `src/server/router.rs` (AuthState::new:138), `src/server/bootstrap.rs` (validate_auth_config:41, run:284), `src/server/routing.rs` (shim), `src/cli_server.rs` (shim 14L), `src/config.rs` (api_key:319, env:705), `src/server/cli_server_auth_tests.rs` (helper:61), `Cargo.toml` ([dependencies]:25, [features]:103)
- **Archivos referenciados hacia dentro:** `jsonwebtoken` (nuevo, codegraph: 0 matches previos en repo — verificado vía grep); `subtle::ConstantTimeEq` ya usado en state.rs:28
- **Archivos que referencian a los editados:** `AuthState::new` ← router.rs:138 + test helper; `ServerState{}` literales ← bootstrap.rs:318, tests; `VantaConfig` ← cli.rs/cli_handlers (env-only, sin flag CLI en MVP)
- **Veredicto impacto:** bajo — aditivo opt-in; `AuthState::new` es `pub(crate)` (sin superficie pública); `ServerState`/`VantaConfig` ganan campo `Option` con default (no rompe constructores con `..Default::default()`, solo literales exhaustivos de tests que se actualizan)

## Contrato

"DISCOVERY arch registrado en ADR-039 + `cargo test -p vantadb --features server` 0 failed (incl. tests JWT nuevos) + `cargo clippy --workspace --all-targets --all-features -- -D warnings` 0 + `cargo fmt --check` limpio"

## Spec (SDD — feature-add: nuevo módulo `jwt` + campo config + dep)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Mecanismo MVP | HS256 offline con `jsonwebtoken` (sin red, CI-safe; patrón qdrant JWT) / OIDC discovery con JWKS (estándar enterprise weaviate; requiere red en CI +rotación JWKS) | HS256 offline | ✅ decidido-por-evidencia (plan pre-mortem "preferir HS256 offline"; docs/api/HTTP_API.md:633 "No OIDC/SSO yet, SRV-06 delegated"; hardening.md:359 "SRV-06 delegated") |
| 2 | Backend cripto jsonwebtoken v11 | `rust_crypto` (puro-Rust, cross-platform incl. Windows runner) / `aws_lc_rs` (requiere toolchain C/cmake) | `rust_crypto`, `default-features=false` | ✅ decidido-por-evidencia (docs.rs jsonwebtoken 11.0.0 features verificada vía webfetch 2026-09-10) |
| 3 | Identidad resultante del JWT válido | `Transport` (L1-equivalente, reusa path RBAC existente; sin acoplamiento a EntityStore) / `User{sub}` (acopla a colección `user`, migración implícita) | `Transport` | ✅ decidido-por-evidencia (middleware.rs:170-209: Transport sin entrada en token_role_map pasa sin check extra; L3 exige user_key — JWT no lo porta) |
| 4 | Superficie de error | Mismo 401 `{"success":false,"error":"Unauthorized"}` en fallo JWT (sin oráculo api-key-vs-jwt) / error distinto `invalid_token_jwt` | Mismo 401 | ✅ decidido-por-evidencia (security-and-hardening: errores genéricos, sin filtrar internals; middleware.rs:133-141 ya genérico) |
| 5 | Alcance OIDC | DEFER con ADR (JWKS fetch, issuer/aud validation, key rotation) / incluir ahora | DEFER | ✅ decidido-por-evidencia (appetite max 3d; plan stop-condition solo pedía Gate V si DISCOVERY sin ganador — hay ganador) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** Bearer api_key/alt_api_key intacto cuando `jwt_secret` unset; refuse-to-start FIND-07 sin cambios; L2/L3 resolution sin cambios; 401 body byte-idéntico; dev-mode (sin key) sin cambios; feature matrix compila con/sin `server`.
- **Comandos de verificación:** `cargo test -p vantadb --features server jwt` + `cargo test -p vantadb --features server auth` ; `cargo clippy --workspace --all-targets --all-features -- -D warnings` ; `cargo fmt --check`
- **Deuda pendiente:** OIDC discovery (JWKS/issuer) DEFER — ADR-039; `--jwt-secret` flag CLI DEFER (env-only en MVP); `src/cli_server_auth_tests.rs` raíz huérfano (sin `mod`, no compila — pre-existente, NO tocar)

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | Valor |
|------------------------|-------|
| `activeGoal` | SRV-06 OIDC/JWT auth — DISCOVERY-first + MVP HS256 offline |
| `lastAction` | DISCOVERY completo: ganador HS256 offline, ADR-039 + task file creados |
| `result` | PARTIAL (steps 1/6 ✅) |
| `nextAction` | Step 2: Cargo.toml + config.rs + state.rs + jwt.rs |
| `contract` | verificacion: pendiente | evidencia: plan Task 8 + docs.rs jsonwebtoken 11.0.0 | artefactos: docs/dev/tasks/SRV-06.md, docs/dev/architecture/adr/ADR-039-jwt-hs256-offline.md | invariantes: ver arriba | deuda: OIDC DEFER | queda_pendiente: MVP + verify + commit |
| `nextTask` | ninguno (una tarea por invocación) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva sin compensar — la nueva dep `jsonwebtoken` (opcional, solo feature `server`) se compensa documentando el huérfano `src/cli_server_auth_tests.rs` (hallazgo, no fix) y no introduce `unwrap`/`expect` en non-test (solo tests).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable arriba ✅ + fmt/clippy/nextest scoped + tests JWT nuevos verdes |
| **Commit** | Commit atómico solo-archivos-propios, `feat: SRV-06 — ...`, verificación mecánica previa |
| **Release** | N/A justificado en Notas (feature opt-in sin cambio de versión; release-plz decide bump por `feat:`) |

## Herramientas necesarias

- Terminal cargo (check, nextest scoped, clippy, fmt) + codegraph_explore (ya usado)
- `campaign_verify_cmd` (contrato; bug exit -1 → bash directa)

**Skills cargadas (SDP):** api-and-interface-design (contract-first del módulo jwt + Hyrum/error-semantics) · test-driven-development (RED→GREEN tests JWT) · documentation-and-adrs (ADR-039) · security-and-hardening (FASE SECURITY: threat-model auth, sin oráculo, rate-limit intacto) · doubt-driven-development (security-sensitive + 🔴; RED tests como doubt-step para claims de comportamiento) · incremental-implementation (slices ≤100L) · source-driven-development (jsonwebtoken docs.rs verificado) · campaign-executor/progreso/ponytail (base). SDP Paso 0b: `campaign_discover_skills_v2` BUILD (10 candidatos) — `frontend-ui-engineering` y `context-engineering` descartadas (sin UI web; contexto ya mapeado vía codegraph).

## Investigation Notes

- `auth_middleware` 0 matches en shim confirmado: `src/cli_server.rs` es re-export 14L; el middleware vive en `src/server/middleware.rs:42` (re-locate ya hecho por REVIEW-10 — DISCOVERY lo confirma, sin acción).
- Research fila: Backlog SRV-06 "jsonwebtoken HS256 offline vs OIDC discovery"; HTTP_API.md:620 (qdrant JWT RBAC HS256 offline / weaviate OIDC) + :633 ("No OIDC/SSO yet"); hardening.md:24/340/359.
- jsonwebtoken 11.0.0 (docs.rs, verificado 2026-09-10): features `default/use_pem/aws_lc_rs/rust_crypto`; `rust_crypto` = hmac+sha2+... (HS256 sin toolchain C).
- Qdrant: JWT HS256 offline con `iat/exp` + RBAC por claims (referencia de diseño, no dependencia).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 5 (steps 2-6) |
| % completado | 15% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — `security-and-hardening` cargada. Threat model: Spoofing (firma HS256 + `exp` obligatorio; secreto ≥32B recomendado, warn si corto); Tampering (HMAC íntegro; claims tipados, sin `serde_json::Value`); Info-disclosure (401 genérico, `sub` nunca en logs de error — solo en audit ok como user_id); DoS (rate-limiter L1 existente cubre intentos JWT; token size cap vía header limit de axum); EoP (JWT→Transport, sin escalada a L2/L3). Abuse-case test: token firmado con otro secreto → 401; token expirado → 401; `alg:none` → reject (Validation fija HS256).
- [x] **PERFORMANCE** — no aplica (auth fuera de hot path search/ingestión; HMAC ~µs por request; justificado sin bench).

## Steps

### Step 1: DISCOVERY + task file + ADR-039
- **Archivos:** `docs/dev/tasks/SRV-06.md`, `docs/dev/architecture/adr/ADR-039-jwt-hs256-offline.md`
- **Acción:** registrar decisión HS256 vs OIDC con evidencia
- **Verify:** archivos existen + Spec llena
- **Estado:** ✅ COMPLETED

### Step 2: Dep + config + tipos JWT
- **Archivos:** `Cargo.toml`, `src/config.rs`, `src/server/state.rs`, `src/server/jwt.rs` (nuevo), `src/server/mod.rs`
- **Acción:** jsonwebtoken opcional tras `server`; `VantaConfig.jwt_secret` (env `VANTADB_JWT_SECRET`) + builder; `AuthState`/`ServerState.jwt_secret`; módulo `jwt` con `Claims` + `verify_jwt` + `JwtError`
- **Verify:** `cargo check -p vantadb --features server`
- **Estado:** ⬜ PENDING

### Step 3: Middleware + wiring
- **Archivos:** `src/server/middleware.rs`, `src/server/router.rs`, `src/server/bootstrap.rs`, `src/server/cli_server_auth_tests.rs` (helper)
- **Acción:** fallback JWT tras fallo api-key (solo si secret configurado); thread secret config→ServerState→AuthState; helper test + `None`
- **Verify:** `cargo check -p vantadb --features server --tests`
- **Estado:** ⬜ PENDING

### Step 4: Tests JWT (TDD GREEN)
- **Archivos:** `src/server/jwt.rs` (tests), `src/server/cli_server_auth_tests.rs` (2-3 tests HTTP: válido/otro-secreto/expirado)
- **Acción:** RED ya cubierto por diseño (módulo nuevo); tests: válido ✅, expirado 401, secreto ajeno 401, `alg:none` 401, sin-secret ⇒ api-key-only intacto
- **Verify:** `cargo test -p vantadb --features server jwt && cargo test -p vantadb --features server auth`
- **Estado:** ⬜ PENDING

### Step 5: Verify mecánico + docs sync
- **Archivos:** `docs/api/HTTP_API.md` (línea SRV-06), task file
- **Acción:** fmt + clippy workspace all-features + nextest scoped; sync línea "No OIDC/SSO yet" → JWT HS256 nota
- **Verify:** `cargo fmt --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` + `cargo check --no-default-features --features fjall` (matriz mínima)
- **Estado:** ⬜ PENDING

### Step 6: Commit solo-propio + cierre
- **Archivos:** solo tocados por SRV-06
- **Acción:** `git add` selectivo (NO opencode.jsonc/.opencode/Backlog/avance/Investigacion-plan.md) + commit `feat: SRV-06 — ...` + `campaign_update_task_state completed` + RESULTADO
- **Verify:** `git status --short` limpio de ajenos + `git log --oneline -1`
- **Estado:** ✅ COMPLETED (commit a0a3087f; incidente: 2 commits previos arrastraron WIP PRX-06 por `git add` concurrente del agente paralelo — revertidos vía soft-reset + pathspec; WIP ajeno intacto)

## Dependencias
- Wave2 paralelo: PRX-06 (vanta-proxy/src/server.rs) + PROV-11 (providers/) — archivos disjuntos, sin orden requerido.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** doubt-driven-development (degraded, mismo contexto — no hay sub-agente distinto disponible en este runner; cross-model skipped: contexto no-interactivo) + `code-review-and-quality` pre-commit
- **Enfoque:** ¿HS256-offline es el approach correcto vs OIDC? ¿JWT→Transport introduce escalada?
- **Cómo se probó:** tests mecánicos del contrato (paso 4-5), no auto-reporte
- **Checklist anti-hábitos tóxicos:** verificado — sin salidas inventadas (todos los comandos corridos con output real), sin done sin verify (contrato mecánico), sin reintentos en bucle (1 retry con causa raíz: leeway 60s), error-paths de seguridad no degradados (401 genérico, leeway 0, empty-secret fail-closed)
- **Veredicto:** ✅ approve (degraded: mismo contexto; slice aditivo opt-in con 9 tests nuevos + suites auth/rotation/rbac/request_id + vantadb-server 42 tests verdes como evidencia)

## Notas
- Nunca silencio: si verify falla 2× mismo-error → Gate V (STOP con RESULTADO 🟡).
- `campaign_verify_cmd` con bug exit -1 → bash directa.
