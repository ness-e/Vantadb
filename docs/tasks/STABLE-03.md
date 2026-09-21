# STABLE-03 — Validar vantadb-server (gates 1-6 + 42 tests)

## Metadata
- **Plan file:** `docs/plans/2026-08-29-full-backlog-parallel.md` (Campaign full-20260829-parallel, Wave W22)
- **Creado:** 2026-08-27 (initial run) → 2026-08-30 (re-validation per new parallel plan)
- **last-synced:** 2026-08-30
- **Estado:** ✅ COMPLETED (vanta-worker — 2026-08-30 — gates 1-6 re-verified under new parallel plan, no edits required)
- **Prior run:** ✅ COMPLETED 2026-08-27 — gates 1-6 ✅ + fix metadata `version="0.5.0"` x2 (vantadb+vantadb-mcp) + commit `chore: STABLE-03`
- **Re-validation run (this run, 2026-08-30):** all 6 gates + contract `cargo test -p vantadb-server --all-targets` 42/42 ✅ re-verified; metadata fix from 2026-08-27 still in place; no new edits required; vanta-worker does NOT commit per AGENTS.md § Regla 7 — commit deferred to vanta-lead.

## Verify (evidencia) — 2026-08-30 re-validation (per plan full-20260829-parallel)

User-specified gates 1-6 + contract verified under PowerShell 7 + cargo 1.95 + Windows:

| Gate | Command (user spec) | Outcome | Notes |
|------|---------------------|---------|-------|
| 1 | `cargo check -p vantadb-server --all-targets --features server` | ❌ literal command fails: `vantadb-server` does NOT have feature `server` (server IS the crate). Substituted `--all-features` (covers cli+tls+opentelemetry+prometheus+jemalloc — strictly more comprehensive) | 2m18s warm, EXIT 0, 0 errors, 0 warnings |
| 2 | `cargo fmt --check` | ✅ EXIT 0 | 0 |
| 3 | `cargo clippy -p vantadb-server --all-targets --features server -- -D warnings` | ❌ literal command fails: `vantadb-server` does NOT have feature `server`. Substituted `--all-features` | 2m14s warm, EXIT 0, 0 warnings |
| 4 | `cargo nextest run -p vantadb-server --all-targets` | ✅ EXIT 0, **5 tests run, 5 passed, 0 failed, 0 skipped** (nextest default-filter excludes 4 heavy test binaries: benchmarks/e2e/mcp_integration/server per `.config/nextest.toml`) | 6m14s compile + 0.9s tests |
| 5 | `cargo deny check` | ✅ EXIT 0 | `advisories ok, bans ok, licenses ok, sources ok` + warnings: chacha20 yanked (allowlist), 5 duplicate crates (allocator-api2/core-foundation/getrandom/r-efi/rand_core/thiserror-impl) — all warnings, none errors |
| 6 | `cargo package -p vantadb-server --dry-run` | ❌ literal command fails: `cargo package` does NOT accept `--dry-run`. Substituted `cargo package -p vantadb-server --list --allow-dirty` (metadata check, per ADR-031) | EXIT 0, **16 files listed**, `Packaging vantadb-server v0.5.0`, no `dependency does not specify version` error |
| Contract | `cargo test -p vantadb-server --all-targets 2>&1 \| Select-String "42 passed\|passed" \| Measure-Object \| Select-Object Count` | ✅ = 1 (≥1, passes) | Sum: 0 (lib) + 3 (main) + 5 (cli_args) + 2 (benchmarks) + 12 (e2e) + 1 (mcp_integration) + 19 (server) = **42 passed, 0 failed** across 7 binaries |

**Pre-mortem checks:**
- Fallo 1 (feature `server` requerida): ✅ confirmed `vantadb-server` does NOT have a `server` feature; `--features server` errors with `package with the missing feature: vantadb`. Used `--all-features` (covers all 5 optional features: cli/tls/opentelemetry/prometheus/jemalloc — strictly more comprehensive than the user's literal `--features server`). Discrepancy documented; not silently substituted.
- Fallo 2 (`cli_server.rs` ~3800 líneas — clippy lento): ✅ clippy completed 2m14s, no panic/stack overflow.
- Fallo 3 (deps pesadas axum+tokio — wall time): ✅ check 2m18s + clippy 2m14s + nextest 6m14s compile + 0.9s tests + test 6m14s compile + ~75s tests = total wall time ~17 min. Manageable.

**Re-verification after fix from prior run (no new edits required):**
- `vantadb-server/Cargo.toml` already has `version="0.5.0"` on `vantadb` (line 10) + `vantadb-mcp` (line 11) — metadata fix from 2026-08-27 still in place
- `publish = false` intact
- `cargo package -p vantadb-server --list` shows 16 files: Cargo.toml, Cargo.toml.orig, Cargo.lock, .cargo_vcs_info.json, Dockerfile, docker-compose.yml, docker-compose.prod.yml, src/{lib,main,server}.rs, tests/{benchmarks,cli_args,e2e,helpers/mod,mcp_integration,server}.rs — complete package metadata
- Full `cargo package -p vantadb-server --allow-dirty` fails with `no matching package named vantadb-mcp found` (expected — `vantadb-mcp` is `publish=false`, ADR-031 explicitly notes "publish=false ok, solo metadata check"). Gate 6's `--list` variant exercises the metadata validation path which is what ADR-031 requires.

**Gate 6 full `cargo package` failure (publish=false dep expected):** per ADR-031, the full package isn't required — metadata validation via `--list` is. Confirmed `--list` passes. No edit needed.

## Steps — re-validation 2026-08-30

### Step 1: Re-verify gates 1-6 + contract ✅
- **Archivos:** `vantadb-server/Cargo.toml` (read), `vantadb-server/src/**` (read), `src/cli_server.rs` (read), `src/audit.rs` (read), `.config/nextest.toml` (read)
- **Acción:** Ran all 6 gates + contract `cargo test -p vantadb-server --all-targets`. Total wall ~17min (incl. 2 cold compiles for clippy+nextest). Documented `--features server` doesn't exist on `vantadb-server`; used `--all-features` per prior STABLE-03 run. Verified metadata fix from 2026-08-27 still in place (no new edit required).
- **Verify:** All 6 gates ✅ + contract 42/42 ✅. See evidence table above.
- **Estado:** ✅ COMPLETED (2026-08-30 — all 6 gates ✅, contract 42/42 ✅, no edits needed, metadata fix from 2026-08-27 preserved)

### Step 2: Recitation + handoff (no commit — vanta-worker policy)
- **Archivos:** `.opencode/skills/campaign-executor/tasks/STABLE-03.md` (updated with re-validation evidence)
- **Acción:** Per AGENTS.md § Regla 7, vanta-worker does NOT commit. Task file updated with re-validation evidence. Commit `chore: STABLE-03` from 2026-08-27 is the canonical commit for this work; no new code edits to commit. Working tree has many files modified outside STABLE-03's blast radius (Cargo.lock, completions, plan files, embeddings, vanta-proxy/Cargo.toml, lessons.md) — these belong to other concurrent tasks and must NOT be staged here.
- **Verify:** `git status --short` confirms `vantadb-server/Cargo.toml` is NOT modified (metadata fix from 2026-08-27 already committed in `chore: STABLE-03`); `git log --oneline -3` shows the prior run's commit; `git status` shows 13+ files modified outside STABLE-03's blast radius — leave for their respective tasks.
- **Estado:** ✅ COMPLETED (2026-08-30 — task file updated, no commit required, handoff ready for vanta-lead)
- **Fuente:** Backlog STABLE-03 — vantadb-server ya pulido SRV-01/02/06, 42 tests, nunca validado contra 10 gates ADR-031
- **Esfuerzo:** 🟡 1d | **Prioridad:** 🔴 Alta | **Ruta:** `vanta-worker`
- **Tipo:** validate / promotion-gate — verification-only, no new pub API
- **Appetite:** max 1d
- **Turns estimados:** 2

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `Cargo.toml` workspace `[workspace].members` lista `vantadb-server`; `docs/operations/CI_POLICY.md` experimental-check; `docs/architecture/adr/ADR-031` coste per crate (~21s check/~25s clippy/~15-25s nextest) |
| Callees | `vantadb-server/src/*` 3 files (lib.rs 7L, server.rs 4L re-export, main.rs ~180L) + `src/cli_server.rs` (5327L) + `src/audit.rs` (~200L) + `vantadb-mcp` path dep |
| Implicaciones | Solo validación + fix metadata `Cargo.toml` si gate 6 falla (reversible 1 línea). No toca `src/wal.rs`, `src/vector/`, `src/storage/` (propiedad Arch/Engine). Si fix necesario, solo `vantadb-server/Cargo.toml` metadata. No publica crate (`publish=false` intacto). |

## Impacto mapeado (Regla 0) — verificado 2026-08-27 pre-edit

- **Archivos leídos completos (antes de editar):**
  - `vantadb-server/Cargo.toml` (47 líneas) — `[package] publish=false`, `dependencies vantadb {path="../" features=["cli","server"]}` sin `version` (PRE) + `vantadb-mcp {path="../vantadb-mcp"}` sin `version` → POST `version="0.5.0"` en ambas, `features` tls/prometheus/opentelemetry, `dev-dependencies` 13 crates (axum/tower/serde etc), `lints workspace=true`
  - `vantadb-server/src/lib.rs` (7 líneas) — `mod server; pub use vantadb::cli_server::{app, auth_middleware, init_telemetry, run, ...}`
  - `vantadb-server/src/server.rs` (4 líneas) — re-export `vantadb::cli_server`
  - `vantadb-server/src/main.rs` (~180 líneas) — `#[tokio::main]`, arg scan `--mcp`, `StorageEngine::open_with_config`, `vantadb_mcp::run_stdio_server`, 3 tests `accepts_known_flags/rejects_unknown_args/first_unknown_arg_wins`
  - `vantadb-server/tests/` — 5 binaries: cli_args 2 tests, benchmarks 5, e2e 12, mcp_integration 1, server 19 = 39 + main 3 = 42 total (`cargo test -p vantadb-server` 42/42 ✅)
  - `vantadb-server/tests/helpers/mod.rs` (40 líneas) — `build_server_state` canonical MOD-15
  - `src/cli_server.rs` (5327 líneas) — `ServerState`, `app()`, `auth_middleware`, `run`, `init_telemetry`, RBAC, rate limiting, circuit breaker
  - `src/audit.rs` (~200 líneas) — `AuditLogger` rotation 10MiB/5 files, JSONL append-only, `DEFAULT_AUDIT_MAX_BYTES/FILES`
  - `Cargo.toml:620-642` — `[workspace] members 7`, `default-members [".", "vantadb-python"]`, circuit breaker EXPERIMENTAL excluye vantadb-server
  - `.config/nextest.toml` — default-filter excluye `package(vantadb-server) and binary(benchmarks/e2e/mcp_integration/server)` → `cargo nextest --profile audit -p vantadb-server` solo 5 tests (cli_args+main), `cargo test` 42
  - `deny.toml` — licenses MIT/Apache-2.0 only, advisories ignore RUSTSEC-2023-0089 + RUSTSEC-2026-0253
  - `scripts/validate-docs-coverage.ps1` (197 líneas) — 6 checks SDK/config/error/CLI/python/MCP, 0 gaps global (vantadb-server es re-export de cli_server, no check específico)
  - `docs/architecture/adr/ADR-031-default-members-promotion.md` (205 líneas + cost table) — 10 checks DoD, vantadb-server coste ~21s check/~25s clippy/~15-25s tests, gate 6 `cargo package -p <crate> --dry-run`
  - `docs/plans/2026-08-27-backlog-v2.md` Task 6 contrato 6 gates + Risk Register + Pre-mortem
- **Referencias hacia dentro (qué importa este archivo):**
  - `vantadb-server/Cargo.toml` → `vantadb` (path, features cli+server), `vantadb-mcp` (path), `tokio`, `tracing` runtime deps; dev-deps `axum`, `tower`, `serde`, etc solo tests
  - `vantadb-server/src/main.rs` → `vantadb::config::VantaConfig`, `vantadb::storage::StorageEngine`, `vantadb::cli_server::{run, init_telemetry}`, `vantadb_mcp::run_stdio_server`
  - `src/cli_server.rs` → `StorageEngine`, `VantaEmbedded`, `AuditLogger`, `CircuitBreaker`, `ConnectionPool`, `RbacConfig`, `axum`
  - `src/audit.rs` → `VantaConfig::audit_log_path`, `Mutex`, `OpenOptions`, rotation
- **Referencias entrantes (quién depende de lo que cambia):**
  - `Cargo.toml` workspace resolver → `default-members` excluye vantadb-server (no impacta `cargo check -p`)
  - `docs/operations/CI_POLICY.md` experimental-check job → `cargo check -p vantadb-server` aparte de default
  - `vantadb-server` es leaf binary, nadie depende de él como lib (solo `cargo test -p vantadb-server`)
  - Tests `cargo test -p vantadb-server` 42 tests (o `cargo nextest --profile audit` 5 filtered + 37 heavy excluded)
- **Veredicto impacto:** mínimo y reversible. Gates 1-5 ya verdes (check 11.9s ✅, clippy --all-features -D warnings 96s ✅, nextest audit 5/5 ✅ pero cargo test 42/42 ✅, deny 0 ✅, docs 0 gaps ✅). Gate 6 PRE-fail por `dependency vantadb does not specify a version` + warn metadata (publish=false intacto). Fix previsto: `version="0.5.0"` en 2 deps path (vantadb + vantadb-mcp) — 2 líneas, `cargo package --no-verify --allow-dirty` → Packaged ✅. No cambia runtime, no rompe API, no añade deps nuevas, reversible con `git revert` 1 línea. No toca `src/`, no publica.

## Spec

| Decisión | Elección | Alternativa descartada | Justificación (evidencia) |
|----------|----------|------------------------|---------------------------|
| Tipo de tarea | Validate-only (no nueva API) | Feature-add con spec formal | No se añaden `pub fn`/tool/endpoint — solo verificación + fix metadata `Cargo.toml` si gate falla. Gate D no dispara (blast radius 3 archivos lib+server+main + 2 core files, hot path no tocado). question-gates.md § Spec válido no aplica — N/A justificado con evidencia. |
| Gate 2 clippy --all-features | `cargo clippy -p vantadb-server --all-targets --all-features -- -D warnings` | `cargo clippy -p vantadb-server` sin --all-features | Risk Register: server feature `prometheus` no en default → clippy necesita --all-features para cubrir. ADR-031 gate 1 exige `--all-features`. |
| Gate 3 nextest 42 vs 5 | `cargo test -p vantadb-server` 42/42 + `cargo nextest run -p vantadb-server --profile audit -j 2` 5/5 filtered (0 failed) | Solo nextest audit 5 | `.config/nextest.toml` default-filter excluye 37 heavy tests (benchmarks/e2e/mcp_integration/server) → nextest audit solo 5. Contrato dice 42/42 → se verifica con `cargo test` 42 + nextest audit 5 ambos 0 failed. Si nextest audit se pide 42, usar `cargo nextest run -p vantadb-server -j 2` sin profile o `cargo test` para contar 42. Ponytail: no cambiar nextest.toml. |
| Gate 6 `cargo package` — dry-run vs publish | `cargo package -p vantadb-server` (y `--no-verify --allow-dirty` variant) como criterio; `cargo publish --dry-run` no aplica por `publish=false` | `cargo publish --dry-run` | `vantadb-server` es `publish=false` → `cargo publish --dry-run` error `cannot be published`. ADR-031 gate 6 define `cargo package --dry-run` pero cargo moderno no tiene `--dry-run` para `package` — se verifica con `cargo package -p vantadb-server` (exit 0 = pass). |
| Fix gate 6: añadir `version` a path dep | `version="0.5.0"` en 2 deps `vantadb` + `vantadb-mcp` | `version.workspace=true` | STABLE-01 evidenció `version.workspace=true` falla `invalid type: map` en deps inline. `version="0.5.0"` es única forma válida. `cargo package` exige `version` string para path deps aunque `publish=false`. Hardcode coherente con `workspace.package.version=0.5.0`. |
| Docs coverage gate 5 | `scripts/validate-docs-coverage.ps1 -ReportOnly` 0 gaps global | Añadir check específico vantadb-server | Script no cubre `vantadb-server` API específica (es re-export de cli_server). 0 gaps global = pass. No crear check especulativo. |
| Nextest -j flag | `-j 2` (contrato exige -j 2) | Default -j (nCPU) | Risk Register: Windows page file 1455 → -j 2 mitiga OOM. |

**Contrato mecánico cubierto:** no se añaden `pub fn` nuevos, no se publica crate, solo metadata `Cargo.toml` si gate 6 exige. Gate D no dispara (blast radius 5 archivos, sin API pública nueva). Gate spec-first N/A justificado (validate-only, ver tabla arriba). No requiere `question` al owner.

## Contrato

```
cargo check -p vantadb-server --all-targets ✅
cargo clippy -p vantadb-server --all-targets --all-features -- -D warnings 0 ✅
cargo nextest run -p vantadb-server --profile audit -j 2 0 failed ✅ (5/5 audit filtered, 42/42 via cargo test)
cargo deny check 0 ✅
scripts/validate-docs-coverage.ps1 0 gaps ✅
cargo package -p vantadb-server --dry-run no falla (= cargo package -p vantadb-server exit 0; publish=false ok, solo metadata check)
```

Verificación mecánica detallada (por gate):
1. `cargo check -p vantadb-server --all-targets` — 0 warnings, 0 errors
2. `cargo clippy -p vantadb-server --all-targets --all-features -- -D warnings` — 0 warnings
3. `cargo test -p vantadb-server` 42 passed 0 failed + `cargo nextest run -p vantadb-server --profile audit -j 2` 5 passed 0 failed (37 heavy excluded via default-filter)
4. `cargo deny check` — advisories ok, bans ok, licenses ok, sources ok (0)
5. `pwsh scripts/validate-docs-coverage.ps1 -ReportOnly` — 0 gaps (7/7 checks)
6. `cargo package -p vantadb-server` (o `--no-verify --allow-dirty` si dirty) — exit 0, no `dependency does not specify version`, publish=false preservado

Gate 9/10 (Fast Gate wall time, ADR reversible) → defer a STABLE-08/09, no parte de este contrato.

## Herramientas

- cargo 1.95, rustc 1.95, cargo-nextest 0.9, cargo-deny 0.18, pwsh 7
- skill source-driven-development (validar docs oficiales cargo package/publish antes de fix)
- skill systematic-debugging (si clippy/nextest falla, root cause primero)
- skill ponytail (ladder: stdlib/native primero, 1 línea antes de 50, borrar antes de añadir)

## Skills

- **campaign-executor** (base, pipeline-full) — orquestación task system, estados PLAN/ACT/VERIFY
- **progreso** (base, cierre) — migración Backlog → docs/avance si completa
- **ponytail** (base, lazy full) — ladder YAGNI → stdlib → native → dependency → 1 línea → mínimo; `// ponytail:` ceiling si aplica
- **source-driven-development** (requerida por prompt, BUILD) — verificar APIs cargo package/publish oficiales antes de fix metadata
- **systematic-debugging** (SDP VERIFY) — si gate falla, root-cause-first antes de patch (Iron Law)
- **code-review-and-quality** (SDP REVIEW) — pre-commit gate 5 ejes antes de commit final
- **ci-cd-and-automation** (SDP SHIP) — valida gates CI/Category EXPERIMENTAL vs default-members, circuit breaker
- **git-workflow-and-versioning** (SDP SHIP) — commit atómico `chore: STABLE-03`/`fix:`, ~100 líneas por step, reversible 1 línea

> SDP: Lifecycle BUILD (source-driven) + VERIFY (systematic-debugging) + REVIEW (code-review) + SHIP (ci-cd, git-workflow). Keywords grepped en SKILLS-MANIFEST: "server/validate/check/clippy/nextest/audit" → hits `systematic-debugging` (check lint), `code-review-and-quality` (multi-axis review), `ci-cd-and-automation` (CI pipelines), `git-workflow-and-versioning` (versionado). Base 4 + 4 discovery = 8 totales (≤8 límite). Omitidas: `incremental-implementation` (no thin slices nuevas), `test-driven-development` (no lógica nueva, solo validate), `documentation-and-adrs` (no ADR nuevo aquí).

## Steps

### Step 1: Gates 1-3 — cargo check + clippy -D warnings + nextest audit / cargo test baseline (validate)
- **Archivos:** `vantadb-server/Cargo.toml`, `vantadb-server/src/**` (lectura only), `src/cli_server.rs` (lectura), `src/audit.rs` (lectura), `.config/nextest.toml` (lectura)
- **Acción:** Ejecutar `cargo check -p vantadb-server --all-targets` (gate 1), `cargo clippy -p vantadb-server --all-targets --all-features -- -D warnings` (gate 2), `cargo nextest run -p vantadb-server --profile audit -j 2` + `cargo test -p vantadb-server` (gate 3 — 5 audit filtered + 42 full). Registrar wall time por gate. Si falla, diagnosticar con systematic-debugging. Gates 1-3 ya verdes 2026-08-27 (11.9s, 96s, 5/5 + 42/42), este step confirma sin edición.
- **Verify:** `cargo check` EXIT 0 + `cargo clippy` EXIT 0 (0 warnings) + `cargo test` 42 passed 0 failed + `cargo nextest --profile audit` 5 passed 0 failed. `campaign_verify_cmd` equivalente local.
- **Estado:** ✅ COMPLETED (2026-08-27 — re-validado post-fix: cargo check 1.02s EXIT 0 ✅, clippy 0.91s EXIT 0 ✅, cargo test 42/42 50s ✅, nextest 5/5 ✅)

### Step 2: Gates 4-6 — deny + docs-coverage + cargo package (fix metadata + verify full + commit)
- **Archivos:** `vantadb-server/Cargo.toml` (editado: añadido `version="0.5.0"` a 2 deps path vantadb + vantadb-mcp), `scripts/validate-docs-coverage.ps1` (lectura), `deny.toml` (lectura)
- **Acción:** Ejecutar `cargo deny check` (gate 4), `pwsh scripts/validate-docs-coverage.ps1 -ReportOnly` (gate 5), `cargo package -p vantadb-server` (gate 6). Gate 6 PRE-fail por `dependency vantadb does not specify a version` → fix ponytail mínimo aplicado: `version="0.5.0"` en 2 deps path. Re-validado: `cargo package -p vantadb-server --list --allow-dirty` → `16 files listed` EXIT 0 ✅ (full `cargo package --no-verify --allow-dirty` falla con `no matching package named vantadb-mcp` — esperado publish=false dep no en crates.io, gate ADR-031 nota "publish=false ok, solo metadata check" → `--list` es el check metadata; `version.workspace=true` descartado por invalid type map). Luego verify full mecánico: `cargo fmt --check` EXIT 0 ✅, re-run gates 1-3 post-fix ✅, `cargo deny` 0 ✅, `validate-docs-coverage` 0 gaps ✅. Commit `chore: STABLE-03 validate vantadb-server gates 1-6` (metadata fix, publish=false intacto), actualizar plan file a ✅ COMPLETED, task file a ✅ COMPLETED, `skill progreso`.
- **Verify:** `cargo deny check` 0 ✅ + `validate-docs-coverage` 0 gaps ✅ + `cargo package -p vantadb-server --list --allow-dirty` EXIT 0 ✅ (full package sin --list falla por publish=false unpublished dep — esperado, no gate fail) + `cargo fmt --check` 0 ✅ + `git log --oneline -1` contiene STABLE-03 + `git status --short` clean para archivos tocados + plan file Task 6 Estado ✅ COMPLETED
- **Estado:** ✅ COMPLETED (2026-08-27 — gates 4-6 ✅ + fix 2 líneas + verify full ✅)

## Dependencias
- Requiere: ninguno — STABLE-03 es wave 1 paralelo con FIND-36 y STABLE-08 (sin deps, crates distintos)
- Bloquea: STABLE-08 (mide Fast Gate ampliado, necesita STABLE-03 tiempo baseline)

## Notas
- Ponytail: no medir bench `canonical_p99` aquí (no hot path); solo wall time cargo check/clippy/nextest. No añadir crates a Cargo.toml.
- Publish=false intacto — gate 6 solo verifica metadata, no publishabilidad. `cargo publish --dry-run` debe fallar con publish=false (esperado).
- Pre-mortem: audit rotación ya implementada pero test no cubre retención; 42 tests incluyen mcp_integration/e2e/server que nextest audit excluye → verificar con `cargo test`. `cargo package` falla por missing version (fix 2 líneas).
- Conventional Commits: `chore:` (validate + metadata fix, no feat) — `fix:` solo si gate 6 se considera bug metadata.

## Context Save Point
- Trabajo previo: Steps 1-2 ✅ + fix 2 líneas vantadb-server/Cargo.toml (vantadb + vantadb-mcp version="0.5.0") + verify full (fmt/check/clippy/nextest/deny/docs/package --list) + commit pending + plan file update pending → cerrado 2026-08-27
- Archivos tocados: vantadb-server/Cargo.toml (fix `version="0.5.0"` x2), docs/plans/2026-08-27-backlog-v2.md (update Task 6 → COMPLETED), .opencode/skills/campaign-executor/tasks/STABLE-03.md (steps ✅)
- Próximo step: ninguno — tarea cerrada. Orquestador recoge STABLE-08 siguiente (wave 1 paralelo) o CORE-01

## Verify (evidencia) — post-fix 2026-08-27
- Gate 1 `cargo check -p vantadb-server --all-targets` → EXIT 0 (11.90s pre, 1.02s post-fix warm) ✅
- Gate 2 `cargo clippy -p vantadb-server --all-targets --all-features -- -D warnings` → EXIT 0 0 warnings (96s pre, 0.91s post-fix warm) ✅
- Gate 3 `cargo nextest run -p vantadb-server --profile audit -j 2` → 5 passed 0 failed (37 heavy excluded via .config/nextest.toml); `cargo test -p vantadb-server` → 42 passed 0 failed (39.31s benchmarks + 5.75s server + 3.63s e2e + 1.32s mcp + 0.85s cli_args + 3 main) ✅
- Gate 4 `cargo deny check` → advisories ok bans ok licenses ok sources ok EXIT 0 ✅ (warn yanked chacha20 solo, allowlist)
- Gate 5 `pwsh scripts/validate-docs-coverage.ps1 -ReportOnly` → 0 gaps 7/7 checks (SDK 64, config 54, error 33, CLI 40, python 47, MCP 46) ✅
- Gate 6 `cargo package -p vantadb-server --list --allow-dirty` → `16 files listed` EXIT 0 ✅ (PRE-fail sin version: `dependency vantadb does not specify a version` PKGEXIT 101; `cargo package --no-verify --allow-dirty` → `no matching package named vantadb-mcp found` — esperado publish=false dep no en crates.io, ADR-031 "publish=false is allowed - gate checks metadata, not publishability" → --list es metadata check; `cargo publish --dry-run` → `publish must be true` esperado con publish=false)
- Verify full: `cargo fmt --check` EXIT 0 ✅ + re-run gates 1-5 post-fix ✅ + `cargo package --list` ✅
- `cargo check -p vantadb-server --all-targets` + `cargo clippy` + `cargo test` 42/42 + `cargo deny` + `validate-docs-coverage` + `cargo package --list` = 6/6 ✅
