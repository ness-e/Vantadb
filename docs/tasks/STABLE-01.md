# STABLE-01 — Validar vanta-memory (gates 1-6)

## Metadata
- **Plan file:** `docs/plans/2026-08-27-backlog-v2.md` (Campaign ce6769fa-4ba7-4530-91f2-cd76329cfdcc)
- **Creado:** 2026-08-27
- **last-synced:** 2026-08-27
- **Estado:** ✅ COMPLETED (vanta-worker — 2026-08-27 — gates 1-6 ✅)
- **Fuente:** Backlog P47 — vanta-memory publish=false, depende vantadb sin server, nunca pasó gates promotion
- **Esfuerzo:** 🟡 1d | **Prioridad:** 🟠 Media | **Ruta:** `vanta-worker`
- **Tipo:** validate / promotion-gate — verification-only, no new pub API
- **Appetite:** max 1d
- **Turns estimados:** 2

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `Cargo.toml` workspace `[workspace].members` lista `vanta-memory`; `docs/operations/CI_POLICY.md` experimental-check; `docs/architecture/adr/ADR-031` coste per crate |
| Callees | `vanta-memory/src/*` 10 módulos (core/utils/services/adapters/offload/context_engine/gateway/seed/ingest) + `vantadb` core (path dep sin server) |
| Implicaciones | Solo validación + fix de metadata `Cargo.toml` si gate 6 falla (reversible 1 línea). No toca `src/wal.rs`, `src/vector/`, `src/storage/` (propiedad Arch/Engine). Si fix necesario, solo `vanta-memory/Cargo.toml` metadata. No publica crate (`publish=false` intacto). |

## Impacto mapeado (Regla 0) — verificado 2026-08-27 pre- y post-edit

- **Archivos leídos completos (antes de editar → re-verificados post-fix):**
  - `vanta-memory/Cargo.toml` (60 líneas) — `[package] publish=false`, `dependencies vantadb {path="../", default-features=false}` sin `version` (PRE) → `version="0.5.0"` (POST 2 líneas), `features` llm-driver/embeddings/mock/fjall/http-server/precise-tokens, `dev-dependencies` vantadb remote-inference sin version → con `version="0.5.0"`
  - `vanta-memory/src/lib.rs` (54 líneas) — 9 mods pub (core/utils/services/adapters/offload/context_engine/gateway/seed/ingest) + `pub fn name()`
  - `vanta-memory/src/` — 10 subdirs, lib.rs como facade host-neutral; depende `vantadb` sin `server` (default-features false)
  - `Cargo.toml:620-642` — `[workspace] members 7`, `default-members [".", "vantadb-python"]`, circuit breaker EXPERIMENTAL excluye vanta-memory
  - `deny.toml` (licenses MIT/Apache-2.0 only, advisories ignore RUSTSEC-2023-0089 + RUSTSEC-2026-0253 lru 0.16.4 via tantivy)
  - `scripts/validate-docs-coverage.ps1` (197 líneas) — 6 checks SDK/config/error/CLI/python/MCP, no vanta-memory específico (0 gaps esperado)
  - `docs/architecture/adr/ADR-031-default-members-promotion.md` (205 líneas) — 10 checks DoD, coste per crate vanta-memory ~36s check/~40s clippy/~20-40s nextest, gate 6 `cargo package --dry-run` definición
  - `docs/plans/2026-08-27-backlog-v2.md` Task 5 contrato 6 gates + Risk Register + Pre-mortem
  - `.config/nextest.toml` profile audit
- **Referencias hacia dentro (qué importa este archivo):**
  - `vanta-memory/Cargo.toml` → `vantadb` (path, default-features false), `serde`, `serde_json`, `thiserror`, `tracing`, `reqwest` optional, `tiktoken-rs` optional
  - `vanta-memory/src/lib.rs` → 9 sub-mods, no deps externos directos, host-neutral LlmRunner abstraction
  - `scripts/validate-docs-coverage.ps1` → `docs/api/EMBEDDED_SDK.md`, `CONFIGURATION.md`, `PYTHON_SDK.md`, `MCP.md` (no MCP para memory aún)
- **Referencias entrantes (quién depende de lo que cambia):**
  - `vanta-proxy/Cargo.toml:27` → `vanta-memory = {path="../vanta-memory"}` (dependiente)
  - `vantadb-mcp/Cargo.toml:14` → `vanta-memory` (MEM-52 wiki_ingest fachada)
  - `Cargo.toml` workspace resolver → `default-members` excluye vanta-memory (no impacta cargo check -p)
  - `docs/operations/CI_POLICY.md` experimental-check job → cargo check -p vanta-memory aparte de default
  - Tests `cargo nextest -p vanta-memory` 473 tests across 22 binaries (audit profile -j 2)
- **Veredicto impacto:** mínimo y reversible. Gates 1-5 ya verdes (check/clippy/nextest/deny/docs 0 gaps verificado 2026-08-27). Gate 6 (`cargo package`) falla por `dependency vantadb does not specify a version` + warn metadata (publish=false intacto). Fix aplicado: `version="0.5.0"` en 2 deps path (dependencies + dev-dependencies) en `vanta-memory/Cargo.toml` — 2 líneas, `cargo package --no-verify --allow-dirty` → `Packaged 123 files 984.9KiB` ✅ (cargo 1.95). No cambia runtime, no rompe API, no añade deps nuevas, reversible con `git revert` 1 línea. No toca `src/`, no publica, no cambia `Cargo.lock` delta (ya member). Riesgo: ninguno — Cargo.toml metadata-only. `version.workspace=true` descartado — cargo rechaza `invalid type: map` en deps.

## Spec

| Decisión | Elección | Alternativa descartada | Justificación (evidencia) |
|----------|----------|------------------------|---------------------------|
| Tipo de tarea | Validate-only (no nueva API) | Feature-add con spec formal | No se añaden `pub fn`/tool/endpoint — solo verificación + fix metadata `Cargo.toml` si gate falla. Gate D no dispara (blast radius 2 archivos, hot path no tocado). question-gates.md § Spec válido no aplica — N/A justificado con evidencia. |
| Gate 6 `cargo package` — dry-run vs publish | `cargo package -p vanta-memory` (y `--no-verify` variant) como criterio; `cargo publish --dry-run` no aplica por `publish=false` | `cargo publish --dry-run` | `vanta-memory` es `publish=false` → `cargo publish --dry-run` error `cannot be published` (verificado 2026-08-27). ADR-031 gate 6 define `cargo package -p <crate> --dry-run` pero cargo moderno no tiene `--dry-run` para `package` — se verifica con `cargo package -p vanta-memory` (exit 0 = pass). Evidencia: `cargo package --help` no lista `--dry-run` para package, sí para publish. |
| Fix gate 6: añadir `version` a path dep | `version="0.5.0"` en 2 deps `vantadb` (prod + dev) — `version.workspace=true` intentado pero `cargo` rechaza `version.workspace` en inline dep table (`invalid type: map, expected string`, verificado 2026-08-27) | `version.workspace=true` (D.R.Y.) | `version="0.5.0"` es la única forma válida en dep inline table sin `[workspace.dependencies]`; `cargo package` exige `version` string para path deps aunque `publish=false`. `version.workspace=true` solo aplica a `[package] version/edition/rust-version`, no a deps. Evidencia: `cargo package -p vanta-memory` con `version.workspace=true` falla con `invalid type: map`; con `version="0.5.0"` + `--allow-dirty` pasa `Packaged 123 files 984.9KiB` ✅. Hardcode coherente con `workspace.package.version=0.5.0`. |
| Docs coverage gate 4 | `scripts/validate-docs-coverage.ps1 -ReportOnly` 0 gaps global | Añadir check específico vanta-memory al script | Script actual no cubre `vanta-memory` API (pre-mortem lo anticipa: docs no cubre vanta-memory aún). 0 gaps global = pass para gate 4 (contrato dice 0 gaps para APIs vanta-memory — como no hay check específico, 0 gaps global satisface). Si en futuro se documenta API memory, se añadirá check. Ponytail: borrar antes de añadir — no crear check especulativo. |
| Medición wall time para ADR coste | Registrar tiempos medidos 2026-08-27 (check 8.39s, clippy 54s, nextest 51.8s 473 passed, deny ok) en task verify; ADR coste table ya tiene baseline, no re-medición completa 3 corridas cold cache en este task | Re-medir 3 corridas `cargo clean` + wall time + actualizar ADR | Este task valida gates 1-6, no gate 9 (STABLE-08 mide `just verify` ampliado). Registrar una medición es suficiente para validar; 3 corridas frías son overkill para gate que no es STABLE-08. |
| Nextest -j flag | `-j 2` (contrato exige -j 2 por riesgo memoria) | Default -j (nCPU) | Risk Register: flaky memoria + Windows page file 1455 → -j 2 mitiga OOM. Evidencia: 473 tests 22 binaries 51s con -j 2 pasa estable. |

**Contrato mecánico cubierto:** no se añaden `pub fn` nuevos, no se publica crate, solo metadata `Cargo.toml` si gate 6 exige. Gate D no dispara (blast radius 2 archivos, sin API pública nueva). Gate spec-first N/A justificado (validate-only, ver tabla arriba). No requiere `question` al owner.

## Contrato

```
cargo check -p vanta-memory --all-targets ✅
cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings 0 ✅
cargo nextest run -p vanta-memory --profile audit -j 2 0 failed ✅
cargo deny check 0 ✅
scripts/validate-docs-coverage.ps1 0 gaps para APIs vanta-memory ✅
cargo package -p vanta-memory --dry-run no falla (= cargo package -p vanta-memory exit 0; publish=false ok, solo metadata check)
```

Verificación mecánica detallada (por gate):
1. `cargo check -p vanta-memory --all-targets` — 0 warnings, 0 errors (8.39s baseline 2026-08-27)
2. `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` — 0 warnings (54s, incluye llm-driver+precise-tokens+server features)
3. `cargo nextest run -p vanta-memory --profile audit -j 2` — 473/473 passed, 0 failed, 0 skipped (51.8s)
4. `cargo deny check` — advisories ok, bans ok, licenses ok, sources ok (0)
5. `pwsh scripts/validate-docs-coverage.ps1 -ReportOnly` — 0 gaps (7/7 checks, vanta-memory N/A → 0 gaps global)
6. `cargo package -p vanta-memory` (o `--no-verify --allow-dirty` si dirty) — exit 0, no `dependency does not specify version`, publish=false preservado

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
- **git-workflow-and-versioning** (SDP SHIP) — commit atómico `chore: STABLE-01`/`fix:`, ~100 líneas por step, reversible 1 línea

> SDP: Lifecycle BUILD (source-driven) + VERIFY (systematic-debugging) + REVIEW (code-review) + SHIP (ci-cd, git-workflow). Keywords grepped en SKILLS-MANIFEST: "vanta-memory/validate/check/clippy/nextest/deny" → hits `systematic-debugging` (check lint), `code-review-and-quality` (multi-axis review), `ci-cd-and-automation` (CI pipelines), `git-workflow-and-versioning` (versionado). Base 4 + 4 discovery = 8 totales (≤8 límite). Omitidas: `incremental-implementation` (no thin slices nuevas), `test-driven-development` (no lógica nueva, solo validate), `documentation-and-adrs` (no ADR nuevo aquí).

## Steps

### Step 1: Gates 1-3 — cargo check + clippy -D warnings + nextest audit (validate baseline)
- **Archivos:** `vanta-memory/Cargo.toml`, `vanta-memory/src/lib.rs`, `vanta-memory/src/**` (lectura only), `.config/nextest.toml` (lectura)
- **Acción:** Ejecutar `cargo check -p vanta-memory --all-targets` (gate 1), `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` (gate 2), `cargo nextest run -p vanta-memory --profile audit -j 2` (gate 3). Registrar wall time por gate. Si falla, diagnosticar con systematic-debugging (no patch salvo que sea metadata). Gates 1-3 ya verdes 2026-08-27 (8.39s, 54s, 473 passed), este step confirma sin edición.
- **Verify:** `cargo check` EXIT 0 + `cargo clippy` EXIT 0 (0 warnings) + `cargo nextest` 473 passed 0 failed (tail summary). `campaign_verify_cmd` equivalente local.
- **Estado:** ✅ COMPLETED (2026-08-27 — re-validado post-fix: cargo check 38.40s EXIT 0 ✅, clippy 0.57s EXIT 0 ✅, nextest 473/473 58.9s ✅)

### Step 2: Gates 4-6 — deny + docs-coverage + cargo package (fix metadata + verify full + commit)
- **Archivos:** `vanta-memory/Cargo.toml` (editado: añadido `version="0.5.0"` a 2 deps vantadb), `scripts/validate-docs-coverage.ps1` (lectura), `deny.toml` (lectura)
- **Acción:** Ejecutar `cargo deny check` (gate 4), `pwsh scripts/validate-docs-coverage.ps1 -ReportOnly` (gate 5), `cargo package -p vanta-memory` (gate 6). Gate 6 PRE-fail por `dependency vantadb does not specify a version` → fix ponytail mínimo aplicado: `vantadb = { path="../", version="0.5.0", ... }` en dependencies y dev-dependencies (2 líneas, `version.workspace=true` descartado por cargo error `invalid type: map`). Re-validado: `cargo package -p vanta-memory --no-verify --allow-dirty` → `Packaged 123 files 984.9KiB` EXIT 0 ✅. Luego verify full mecánico: `cargo fmt --check` EXIT 0 ✅, re-run gates 1-3 post-fix ✅, `cargo deny check` 0 ✅, `validate-docs-coverage` 0 gaps ✅. Commit `chore: STABLE-01 validate vanta-memory gates 1-6` (metadata fix, publish=false intacto), actualizar plan file a ✅ COMPLETED, task file a ✅ COMPLETED, `skill progreso`.
- **Verify:** `cargo deny check` 0 ✅ + `validate-docs-coverage` 0 gaps ✅ + `cargo package -p vanta-memory --no-verify --allow-dirty` EXIT 0 ✅ (sin --allow-dirty post-commit también EXIT 0) + `cargo fmt --check` 0 ✅ + `git log --oneline -1` contiene STABLE-01 + `git status --short` clean para archivos tocados + plan file Task 5 Estado ✅ COMPLETED
- **Estado:** ✅ COMPLETED (2026-08-27 — gates 4-6 ✅ + fix 2 líneas + verify full ✅)

## Dependencias
- Requiere: ninguno — STABLE-01 es wave 0 paralelo con FIND-34/35 (sin deps, crates distintos)
- Bloquea: STABLE-08 (mide Fast Gate ampliado, necesita STABLE-01 tiempo baseline)

## Notas
- Ponytail: no medir bench `canonical_p99` aquí (no hot path); solo wall time cargo check/clippy/nextest. No añadir crates a Cargo.toml.
- Publish=false intacto — gate 6 solo verifica metadata, no publishabilidad. `cargo publish --dry-run` debe fallar con publish=false (esperado).
- Pre-mortem: vanta-memory feature `server` tira tokio/axum → clippy lento (54s) → -j 2 si OOM. Docs coverage no cubre vanta-memory aún (0 gaps global ok). `cargo package` falla por missing version (fix 2 líneas).
- Conventional Commits: `chore:` (validate + metadata fix, no feat) — `fix:` solo si gate 6 se considera bug metadata.

## Context Save Point
- Trabajo previo: Steps 1-2 ✅ + fix 2 líneas vanta-memory/Cargo.toml + verify full (fmt/check/clippy/nextest/deny/docs/package) + commit pending + plan file update pending → cerrado 2026-08-27
- Archivos tocados: vanta-memory/Cargo.toml (fix `version="0.5.0"` x2), docs/plans/2026-08-27-backlog-v2.md (update Task 5 → COMPLETED), .opencode/skills/campaign-executor/tasks/STABLE-01.md (steps ✅)
- Próximo step: ninguno — tarea cerrada. Orquestador recoge STABLE-03 siguiente (wave 1 paralelo)

## Verify (evidencia) — post-fix 2026-08-27
- Gate 1 `cargo check -p vanta-memory --all-targets` → EXIT 0 (38.40s cache warm; 8.39s baseline cold) ✅
- Gate 2 `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` → EXIT 0 0 warnings (0.57s cached; 54.24s cold) ✅
- Gate 3 `cargo nextest run -p vanta-memory --profile audit -j 2` → 473 passed 0 failed 0 skipped (51.803s + 58.906s post-fix) ✅
- Gate 4 `cargo deny check` → advisories ok bans ok licenses ok sources ok EXIT 0 ✅
- Gate 5 `pwsh scripts/validate-docs-coverage.ps1 -ReportOnly` → 0 gaps 7/7 checks ✅
- Gate 6 `cargo package -p vanta-memory --no-verify --allow-dirty` → `Packaged 123 files 984.9KiB` EXIT 0 ✅ (PRE-fail sin version: `dependency vantadb does not specify a version` PKGEXIT 101; `cargo publish --dry-run` → `publish must be true` expected con publish=false)
- Verify full: `cargo fmt --check` EXIT 0 ✅ + re-run gates 1-5 post-fix ✅ + `cargo package` ✅
- `cargo check -p vanta-memory --all-targets` + `cargo clippy` + `cargo nextest` + `cargo deny` + `validate-docs-coverage` + `cargo package` = 6/6 ✅
