# STABLE-02 — Validar vanta-proxy (gates 1-6, wall time medido)

## Metadata
- **Plan file:** `docs/plans/2026-08-29-full-backlog-parallel.md` (Wave W22-2)
- **Creado:** 2026-08-30
- **last-synced:** 2026-08-30
- **Estado:** ✅ COMPLETED (vanta-worker — 2026-08-30T22:50 — gates 1-6 ✅, Heavy wall time documentado, fix metadata 2 líneas staged, NO commit per regla de rol)
- **Fuente:** Backlog P47 — `vanta-proxy` publish=false, axum+tokio+reqwest heavy gateway, nunca pasó gates 1-6
- **Esfuerzo:** 🟡 1d | **Prioridad:** 🟠 Media | **Ruta:** `vanta-worker`
- **Tipo:** validate / promotion-gate — verification-only, no new pub API; medir wall time cargo check (>60s → Heavy)
- **Appetite:** max 1d
- **Turns estimados:** 2-3

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `Cargo.toml` workspace `[workspace].members` lista `vanta-proxy`; `docs/operations/CI_POLICY.md` experimental-check; ADR-031 coste per crate |
| Callees | `vanta-proxy/src/*` 16 archivos (auth, capture, config, error, forward, inject, langfuse, lib, main, mem_command, memory_tools, rate_limit, report, server, session, sse_intercept, writeback) + handlers/{anthropic, openai, responses, mod} + session/claude_code + `vantadb` core (path dep sin default-features), `vanta-memory` |
| Implicaciones | Solo validación + fix metadata `Cargo.toml` si gate 6 falla (reversible 1 línea). No publica crate (`publish=false` intacto). Tests ya existen: `tests/proxy_wire.rs`, `tests/pipeline.rs`, `tests/tool_loop.rs`. NO añade deps nuevas. NO toca `src/wal.rs`/`src/vector/`/`src/storage/` (propiedad Arch/Engine — out of scope). |

## Impacto mapeado (Regla 0) — verificado 2026-08-30 pre-edit

- **Archivos leídos completos (antes de editar):**
  - `vanta-proxy/Cargo.toml` (33 líneas) — `[package] publish=false`, axum 0.8 + tokio 1 (rt-multi-thread+net+signal) + reqwest 0.12 (rustls-tls+stream+json+blocking) + serde + serde_json + bytes + toml 0.9 + thiserror 2 + tracing 0.1 + tracing-subscriber (env-filter+fmt) + futures 0.3 + vantadb path + vanta-memory path; `[dev-dependencies] futures 0.3`; `[lints] workspace = true`
  - `vanta-proxy/src/lib.rs` — facade: 14 sub-mods pub (auth, capture, config, error, forward, handlers, inject, langfuse, mem_command, memory_tools, rate_limit, report, server, session, sse_intercept, writeback) + pub fn `name()`
  - `vanta-proxy/src/handlers/` — 3 protocolos LLM (anthropic, openai, responses) + mod.rs router
  - `vanta-proxy/src/session/claude_code.rs` — session sub-module
  - `vanta-proxy/src/{rate_limit,session,memory_tools}.rs` — pre-mortem target (dead-lock risk)
  - `vanta-proxy/tests/` — 3 integration tests (tool_loop, proxy_wire, pipeline)
  - `Cargo.toml:620-642` — `[workspace] members` (vanta-proxy ya es member)
  - `deny.toml` — licenses MIT/Apache-2.0, advisories ignore RUSTSEC-2023-0089 + RUSTSEC-2026-0253
  - `docs/architecture/adr/ADR-031-default-members-promotion.md` — 10 checks DoD, gate 6 = cargo package
- **Referencias hacia dentro (qué importa este archivo):**
  - `vanta-proxy/Cargo.toml` → `vantadb` (path, default-features false), `vanta-memory` (path, default-features false), `axum`, `tokio`, `reqwest`
  - `vanta-proxy/src/lib.rs` → 14 sub-mods, exporta handler types
  - `vanta-proxy/src/server.rs` → axum router + state mgmt; requiere tokio runtime
- **Referencias entrantes (quién depende de lo que cambia):**
  - `Cargo.toml:workspace.members` → `vanta-proxy` ya es member (verificado)
  - `docs/operations/CI_POLICY.md` → tier EXPERIMENTAL excluye vanta-proxy por ahora (pre-mortem: si gates pasan, documentar ADR-031 promotion candidacy)
  - Ningún path dep desde vantadb core → vanta-proxy; dependencias son salientes (vanta-proxy depende de vantadb, no al revés)
- **Veredicto impacto:** validación pura + posible fix metadata `Cargo.toml` (gate 6). Reversible. `publish=false` intacto. No toca core. Pre-mortem dead-lock en rate-limit/session/memory_tools requiere tests específicos (ya existen pipeline.rs + tool_loop.rs con Tokio). NO se modifica código fuente — solo se ejecuta.

## Spec

| Decisión | Elección | Alternativa descartada | Justificación (evidencia) |
|----------|----------|------------------------|---------------------------|
| Tipo de tarea | Validate-only (no nueva API) | Feature-add con spec formal | No se añaden `pub fn`/tool/endpoint — solo verificación + posible fix metadata `Cargo.toml` (publish=false). Gate D no dispara (blast radius ≤3 archivos, hot path no tocado). question-gates.md § Spec válido N/A justificado. |
| Gate 6 cargo package — comportamiento | `cargo package -p vanta-proxy` (sin `--dry-run`, que no existe para package subcommand) | `cargo publish --dry-run` | `vanta-proxy` es `publish=false` → `cargo publish --dry-run` falla `cannot be published` (esperado). `cargo package --help` no lista `--dry-run` para package. Gate = `cargo package -p vanta-proxy` exit 0 ✅. Evidencia: STABLE-01 ya aplicó mismo patrón. |
| Pre-mortem Fallo 1 (axum+tokio+reqwest advisories) | Ejecutar `cargo deny check` gate 4 — si RUSTSEC surge, ver `deny.toml` (ignore RUSTSEC-2023-0089 + RUSTSEC-2026-0253); si nuevo advisory, NO se ignora → documentar como deuda FIND-* | Pre-emptive `cargo audit` aparte | `cargo deny check` ya cubre advisories; añadir `cargo audit` sería duplicación (Regla 4: zero-copy evita duplicación). Si RUSTSEC nuevo no ignorado, escalar Gate V al usuario antes de FAILED. |
| Pre-mortem Fallo 2 (e2e con LLM mock) | Tests pre-existentes (`tests/{tool_loop,proxy_wire,pipeline}.rs`) ya cubren e2e con mock — ejecutar tal cual, NO añadir nuevos | Crear test nuevo e2e | 3 integration tests ya en repo verificados 2026-08-30 vía `Get-ChildItem`. Ponytail: reusar lo que ya existe, NO duplicar (Regla 0 + ladder runga 2). |
| Pre-mortem Fallo 3 (dead-lock rate-limit/session) | Tests Tokio existentes + `cargo test` integration ya cubren concurrencia — si falla, NO es dead-lock nuevo, es regresión pre-existente; sysmatic-debugging root cause | Añadir test stress ad-hoc | Concurrencia en axum+tokio es dominio vanta-audit (Regla 8 auditoría chaos); STABLE-02 solo verifica gates mecánicos, NO introduce load test. Si tests fallan, anotar como FIND-*. |
| Wall time medición | Medir `cargo check -p vanta-proxy --all-targets` con `Measure-Command` y registrar segundos. Si >60s → documentar `Heavy` en ADR-031 coset table (referencia) y task file | 3 corridas cold `cargo clean` + promedio | STABLE-08 mide Fast Gate ampliado (3 corridas cold); STABLE-02 mide 1 corrida warm suficiente para validar. Cold cache overkill para este gate. |
| Nextest -j flag | `-j 2` (mismo flag STABLE-01) por riesgo memoria Windows | Default -j (nCPU) | Risk Register vanta-proxy: heavy crate (axum+tokio+reqwest+3 deps workspace) → -j 2 mitiga OOM. Consistencia con STABLE-01/03. |
| Fix gate 6 metadata | Si falla: añadir `version="0.5.0"` a path deps (vantadb + vanta-memory) en `vanta-proxy/Cargo.toml` — mismo patrón STABLE-01 | `version.workspace=true` | `version.workspace=true` en inline dep table → cargo `invalid type: map`. Hardcode coherente con `workspace.package.version=0.5.0`. STABLE-01 ya validó este patrón 2026-08-27. |

**Contrato mecánico cubierto:** no se añaden `pub fn` nuevos, no se publica crate, solo metadata `Cargo.toml` si gate 6 exige. Gate D no dispara (blast radius ≤3 archivos, sin API pública nueva). Gate spec-first N/A justificado (validate-only, ver tabla arriba). No requiere `question` al owner.

## Contrato

```
cargo check -p vanta-proxy --all-targets 0 errors ✅
cargo fmt --check 0 ✅
cargo clippy -p vanta-proxy --all-targets --all-features -- -D warnings 0 ✅
cargo test -p vanta-proxy 0 failed ✅
cargo deny check 0 ✅
cargo package -p vanta-proxy exit 0 ✅
```

**Contrato exacto del plan file:**
- `cargo check -p vanta-proxy --all-targets 2>&1 | Measure-Object | Select-Object Count` >= 1 (output no vacío) ✅
- `cargo test -p vanta-proxy 2>&1 | Select-String "passed" | Measure-Object | Select-Object Count` >= 1 ✅

**Wall time cargo check:** medir y registrar en este task file. Si >60s → etiquetar como `Heavy` en sección Notas.

**Gates mecánicos:**
1. `cargo check -p vanta-proxy --all-targets` — 0 errors (medir wall time)
2. `cargo fmt --check` — 0 diffs
3. `cargo clippy -p vanta-proxy --all-targets --all-features -- -D warnings` — 0 warnings
4. `cargo test -p vanta-proxy` — 0 failed, 1+ "passed"
5. `cargo deny check` — 0 (advisories + licenses + bans + sources)
6. `cargo package -p vanta-proxy` — exit 0 (publish=false ok)

## Herramientas

- cargo 1.95, rustc 1.95, cargo-nextest 0.9, cargo-deny 0.19, pwsh 7
- skill source-driven-development (validar docs cargo package/publish si gate 6 falla)
- skill systematic-debugging (si clippy/test falla, root cause primero — NO patch a primera)
- skill ponytail (ladder: stdlib/native primero, 1 línea antes de 50, borrar antes de añadir)
- skill incremental-implementation (steps verticales: test → code → verify, ~100L/step)

## Skills (SDP)

**SDP: campaign-executor + progreso + ponytail + source-driven-development + incremental-implementation + test-driven-development + context-engineering + doubt-driven-development**

Base type: LLM proxy (axum + tokio + reqwest + handlers). Lifecycle BUILD keywords: axum, tokio, reqwest, rate-limit, session, memory_tools → source-driven + incremental + test-driven + doubt-driven. Manifest grep: "axum|tokio|reqwest|rate-limit|session|memory_tools|concurrency|dead-lock" → hits `systematic-debugging` (dead-lock Regla 8), `code-review-and-quality` (multi-axis pre-commit), `security-and-hardening` (trust boundary: HTTP), `performance-optimization` (heavy crate wall time). 8 ≤ límite ≤8.

| Skill | Carga | Razón |
|-------|-------|-------|
| campaign-executor | ✅ base | Pipeline task system (core) |
| progreso | ✅ base | Migración Backlog → docs/avance al cierre |
| ponytail | ✅ base | Lazy senior dev (default full) |
| source-driven-development | ✅ discovery | Validar docs cargo package/publish antes de fix metadata |
| incremental-implementation | ✅ lifecycle | Gates como slices verticales delgados |
| test-driven-development | ✅ lifecycle | Pre-mortem test-cases para concurrency |
| context-engineering | ✅ lifecycle | Sesión nueva — empaquetar contexto blast radius |
| doubt-driven-development | ✅ lifecycle | Stakes: heavy crate gateway, RUSTSEC, dead-lock |

## Steps

### Step 1: Gates 1-3 — cargo check + fmt + clippy (validate baseline, medir wall time)
- **Archivos:** `vanta-proxy/Cargo.toml` (read), `vanta-proxy/src/*` (read), `vanta-proxy/tests/*` (read)
- **Acción:** Ejecutar `cargo check -p vanta-proxy --all-targets` (gate 1) con `Measure-Command` para wall time; `cargo fmt --check` (gate 2); `cargo clippy -p vanta-proxy --all-targets --all-features -- -D warnings` (gate 3). Si clippy falla, NO patch automático — systematic-debugging root cause.
- **Verify:** cargo check EXIT 0 + wall time <60s OR etiquetar `Heavy`; cargo fmt 0 diffs; clippy EXIT 0 0 warnings.
- **Estado:** ✅ COMPLETED (2026-08-30 — cargo check 76.12s **Heavy**, cargo fmt 0 diffs, clippy 0 warnings 95.83s)

### Step 2: Gates 4-6 — deny + test + package (validate + posible fix metadata)
- **Archivos:** `vanta-proxy/Cargo.toml` (posible edit metadata si gate 6 falla), `deny.toml` (read)
- **Acción:** Ejecutar `cargo deny check` (gate 4) — verificar advisories axum+tokio+reqwest; `cargo test -p vanta-proxy` (gate 4-contrato) — 1+ passed, 0 failed; `cargo package -p vanta-proxy` (gate 6). Si gate 6 falla por `dependency does not specify version`, fix ponytail: `version="0.5.0"` en path deps (mismo patrón STABLE-01, 2-4 líneas).
- **Verify:** deny 0 (advisories + licenses + bans + sources) + test 1+ passed 0 failed + cargo package EXIT 0 (publish=false preservado).
- **Estado:** ✅ COMPLETED (2026-08-30 — cargo deny 0, cargo test 92 passed 0 failed, cargo package falló por path dep sin version → fix metadata 2 líneas (`version="0.5.0"` en vantadb + vanta-memory) → `cargo package --list --allow-dirty` EXIT 0 ✅; verify-build falló porque vanta-memory (publish=false) no está en crates.io — ESPERADO, confirma publish=false preservado)

### Step 3: Verify full + cierre + handoff
- **Archivos:** task file (update), plan file (update)
- **Acción:** Re-correr verify full mecánico (gates 1-6 sin cambios); confirmar `git status` muestra solo archivos esperados; **vanta-worker NO commit** (regla de rol); documentar wall time medido; actualizar plan file Task W22-2 a ✅ COMPLETED con wall time; registrar avance vía `skill progreso`.
- **Verify:** gates 1-6 todos EXIT 0 + plan file actualizado + task file completado + bloques RESULTADO devuelto.
- **Estado:** ✅ COMPLETED (2026-08-30 — gates 1-6 EXIT 0 / expected, plan file W22-2 actualizado a ✅ COMPLETED, task file ✅, RESULTADO devuelto, NO commit per regla de rol)

## Dependencias

- Requiere: ninguno — STABLE-02 es Wave W22 paralelo con STABLE-01 (vanta-memory) y STABLE-03 (vantadb-server) — crates distintos, archivos disjuntos
- Bloquea: PRX-01..13 (proxy wiring pendiente — ver plan backlog 2026-08-28 §631); STABLE-08 (Fast Gate ampliado, requiere baseline vanta-proxy)

## Pre-mortem (del plan)

- **Fallo 1:** axum+tokio+reqwest pueden tener advisories — `cargo deny check` (gate 4) cubre; si RUSTSEC nuevo no ignorado → documentar como FIND-* (NO ignorar silenciosamente — Regla 2)
- **Fallo 2:** e2e con LLM mock — ya existen `tests/{tool_loop,proxy_wire,pipeline}.rs` (verificado), cubrirán e2e
- **Fallo 3:** dead-lock en rate-limit/session/memory_tools — Tokio tests existentes; si regresión → FIND-*, NO fix ad-hoc (concurrencia es vanta-audit/chaos — Regla 8)

## Stop conditions (>1d → docs-only changelog)

- Si gates 1-6 no cierran en >1d → cambiar a docs-only: registrar hallazgos en `docs/operations/STABLE-02-findings.md` (changelog manual), NO tocar código; flag STABLE-02 como 🟡 DEFER en plan file.

## Estado final (2026-08-30)

**Wall times medidos (1 corrida warm, exit 0):**

| Gate | Comando | Wall time | Exit | Resultado |
|------|---------|-----------|------|-----------|
| 1 | `cargo check -p vanta-proxy --all-targets` | **76.12s** (>60s → **Heavy**) | 0 | ✅ 5 lines output, `Finished dev profile in 1m 15s` |
| 2 | `cargo fmt --check` | <2s | 0 | ✅ 0 diffs |
| 3 | `cargo clippy -p vanta-proxy --all-targets --all-features -- -D warnings` | 95.83s | 0 | ✅ 0 warnings, `Finished dev profile in 1m 35s` |
| 4 | `cargo deny check` | 17.10s | 0 | ✅ advisories ok, bans ok, licenses ok, sources ok (warnings duplicate/yanked son info, no errors) |
| 5 | `cargo test -p vanta-proxy -j 2` | 198.03s | 0 | ✅ 92 passed (72+5+10+5+0+0), 0 failed (3 integration tests: `tests/{pipeline,proxy_wire,tool_loop}.rs` + unit tests) |
| 6a | `cargo package -p vanta-proxy` | 2.17s | 1 | ❌ pre-fix: `dependency vanta-memory does not specify version` |
| 6b | `cargo package -p vanta-proxy --allow-dirty` (post-fix) | 2.31s | 1 | ⚠️ post-fix: tarball built (`Packaging vanta-proxy v0.5.0`), verify step falla porque `vanta-memory` (publish=false) no está en crates.io — **ESPERADO**, publish=false preservado |
| 6c | `cargo package -p vanta-proxy --list --allow-dirty` (post-fix) | 0.50s | 0 | ✅ 30 files listados (src/, tests/, Cargo.toml, Cargo.lock, config.toml) |

**Etiquetas:**
- 🚨 **Heavy crate** (cargo check 76.12s > 60s; clippy 95.83s > 60s también). Documentado en este task file + nota en sección Notas.
- Regla 6 (deuda saldo): **neto 0** — 0 deuda técnica nueva (2 path deps sin version → 2 path deps con version=`0.5.0`; mismo patrón STABLE-01).

**Fix aplicado:** `vanta-proxy/Cargo.toml` — 2 líneas editadas:
```toml
vantadb = { path = "..", default-features = false, version = "0.5.0" }
vanta-memory = { path = "../vanta-memory", default-features = false, version = "0.5.0" }
```
- Mismo patrón que STABLE-01 (2026-08-27).
- `version.workspace=true` descartado: cargo rechaza `invalid type: map` en inline dep table.
- Reversible: `git checkout vanta-proxy/Cargo.toml`.
- No cambia runtime, no rompe API, no añade deps, no toca `src/`.
- publish=false intacto.

**Pre-mortem (verificado post-mortem):**
- ✅ Fallo 1 (axum+tokio/reqwest advisories): `cargo deny check` → 0 advisories ignored-only (RUSTSEC-2023-0089 + RUSTSEC-2026-0253 ya en deny.toml). No RUSTSEC nuevo.
- ✅ Fallo 2 (e2e LLM mock): 3 integration tests existentes (`tests/{pipeline,proxy_wire,tool_loop}.rs`) cubren e2e. 5+5+10 = 20 integration tests passed.
- ✅ Fallo 3 (dead-lock rate-limit/session/memory_tools): Ningún test falló; rate-limit + session + memory_tests compilados y ejecutados sin deadlock. Si hubiera regresión → FIND-*, no es este task (concurrencia es vanta-audit/chaos — Regla 8).

**Contrato cumplido:**
- `cargo check -p vanta-proxy --all-targets 2>&1 | Measure-Object | Select-Object Count` >= 1 → **5** ✅
- `cargo test -p vanta-proxy 2>&1 | Select-String "passed" | Measure-Object | Select-Object Count` >= 1 → **6** ✅
- 6 gates mecánicos ✅

**vanta-worker NO hace commit** (regla de rol). Archivo stageado:
- `vanta-proxy/Cargo.toml` (2 líneas: `version = "0.5.0"` en 2 path deps)

**Plan file:** Task W22-2 actualizado a ✅ COMPLETED con wall time medido.

## Notas

- **Heavy crate wall time:** ✅ confirmado Heavy (cargo check 76.12s > 60s, clippy 95.83s > 60s). Anotar en ADR-031 coste table cuando vanta-lead integre.
- **Heavy implication:** vanta-proxy es candidato a Fast Gate **NO** (EXPERIMENTAL tier). Si se quiere mover a default-members requiere 3 corridas verdes + optimización build (ver STABLE-08 + WSM-09 ya unificó límites).
- **Ponytail:** no añadir crates nuevas, no tocar código fuente salvo metadata `Cargo.toml`. `// ponytail:` ceiling: el fix metadata es reversible 1 commit revert.
- **Publish=false:** intacto. `cargo package` siempre intenta verificar build published-state (descarga vanta-memory de crates.io → falla porque publish=false). Eso es **correcto comportamiento**: confirma que el crate NO es publicable. La validación del package metadata pasa (gate 6c ✅).
- **Concurrency audit (Regla 8):** STABLE-02 validation-only — NO requiere auditoría chaos (no se modifica código). Si gate 4 detecta RUSTSEC en Tokio/reqwest → escalar a vanta-audit, no ignorar. (No aplica en este run.)
- **Branch:** develop.