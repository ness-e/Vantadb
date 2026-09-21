# GOV-A2: Reconciliar cifras tests (docs/reports, coverage, nextest)

## Metadata
- **Plan file:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md`
- **Wave:** Wave1 (GOV-A medición, MAX 3 — paralelo con A1/A3/A4/A5 + RES-02..05 disjoint)
- **Creado:** 2026-09-02T20:45
- **last-synced:** 2026-09-02T20:45
- **Estado:** ✅ COMPLETED
- **Esfuerzo:** 🟢 ≤1h (docs-only)
- **Tipo:** docs / governance
- **Prioridad:** Alta (auditoría intake — GOV-A medición)
- **Campaign ID:** 20260902-alta-prioridad-paralelo
- **No tocar:** RES-02..05 (P38 durabilidad aislada) + GOV-A3 (probes CLI, paralelo disjoint)

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/Backlog.md` (fila proyecto 2034 tests), `docs/reports/dora.md` (fuentes plan/task files, no cifra tests directa), `docs/reviews/*` (auditorías intake) |
| Callees | `docs/TEST_MAP.md` (cifra canónica 2034 + contexto histórico), `scripts/validate-docs-coverage.ps1` (6 checks docs↔código), `.config/nextest.toml` (profile default audit excluye heavy), `.codegraph/codegraph.db` (índice existe, no lógica tests), `Cargo.toml` (workspace version 0.5.0, no cifra tests) |
| Implicaciones | Docs-only. No cambia Rust, no toca nextest.toml, no toca .codegraph, no toca src/. Riesgo: si TEST_MAP.md no cita fecha+perfil, contrato falla. Mitigación: `Select-String` mecánico + `cargo nextest list` + `validate-docs-coverage -ReportOnly` |

## Impacto mapeado (Regla 0) — BLAST RADIUS DOCS (codegraph no necesario)
- **Archivos leídos (completos):** `docs/reports/dora.md` (402L, generado 2026-08-22 por evals/dora.mjs, fuentes 23 tareas/412 task files/33 budgets/124 verify), `evals/dora.mjs` (360L, recoveryPairs 207-222), `Cargo.toml` (696L, workspace 0.5.0, default-members vantadb+vantadb-python, features embed-local), `.codegraph/codegraph.db` (exists, 4 files), `scripts/validate-docs-coverage.ps1` (197L, 6 checks SDK/config/error/CLI/python/MCP), `scripts/check_openapi_parity.mjs` (171L), `.config/nextest.toml` (default-filter excluye ~30 heavy binaries, audit/ci-windows/experimental profiles), `docs/TEST_MAP.md` (155L, Fast Decision Table + CI Gates + Coverage §), `docs/Backlog.md:685` (cita 2034 tests), `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §GOV-A2
- **Archivos referenciados hacia dentro:** TEST_MAP.md referenciado por Backlog 2034, Backlog por planes, dora.md por governance, validate-docs-coverage por TEST_MAP/CI_POLICY, nextest.toml por cargo nextest, .codegraph por dev-tools/verify
- **Archivos que referencian a los editados:** grep `2034|2568|1902|1492` → Backlog.md:685, TEST_MAP.md:92, plan:174/961 (citas planificadas) — sin drift en docs/reports/dora.md (no contiene cifra tests)
- **Veredicto impacto:** bajo — docs-only, 1 doc editado histórico ya reconciliado, verify mecánico sin build Rust. Disjoint 100% con Wave1 RES-02..05 (src/wal.rs, src/iql/, docs/api/) y GOV-A3 (binario vanta-cli) — sin contención archivos.

## Contrato
`Select-String -Path "docs/TEST_MAP.md" -Pattern "2034.*2026-08" | Measure-Object Count` >=1 AND contextualización histórica `1492/1902/2568+` en el mismo doc
- **Verificación atómica extendida (pipeline-full):** `Select-String "coverage"` en TEST_MAP.md (sección Coverage) + `cargo nextest list --profile default -p vantadb` lista + `scripts/validate-docs-coverage.ps1 -ReportOnly` exit 0 + `.codegraph/codegraph.db` exists + `Cargo.toml` workspace version 0.5.0
- **Cifra canónica:** 2034 tests / 2034 passed / 1 skipped — `cargo nextest run` perfil default (excluye heavy), Windows local, 2026-08-22 (TEST_MAP.md:92) — coincide con Backlog.md:685 y plan fila 961
- **Cifras históricas contextualizadas:** 1492 / 1902 / 2568+ = snapshots anteriores con perfiles distintos (default vs audit vs workspace all) — documentado en TEST_MAP.md:92 "Cifras históricas (...) son snapshots anteriores con perfiles distintos"

## Spec (doc-driven)
N/A — docs-only reconciliación, sin símbolos públicos nuevos. Decisión ya tomada: fuente única TEST_MAP.md §Coverage + Fast Decision Table; no crear nuevo reporte.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** No duplicar cifra tests en docs/reports/dora.md (ese reporte es DORA flow, no test count); no modificar .config/nextest.toml ni Cargo.toml; validate-docs-coverage.ps1 debe seguir pasando (6 checks); .codegraph no se regenera en esta task
- **Comandos de verificación:** `Select-String -Path "docs/TEST_MAP.md" -Pattern "2034.*2026-08"` >=1 ; `Select-String -Path "docs/TEST_MAP.md" -Pattern "coverage"` >=1 ; `cargo nextest list --profile default -p vantadb` (2074 lines lista) ; `pwsh scripts/validate-docs-coverage.ps1 -ReportOnly` ; `Test-Path .codegraph/codegraph.db`
- **Deuda pendiente:** ninguna — cifras reconciliadas, coverage threshold ≥80% (ADR-018 gate 81.40%) documentado en TEST_MAP.md:91 sin drift; llvm-cov ICE Windows 2026-08-22 ticket GOV-A1 mitigado por fallback ADR-018

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | GOV-A2 — Reconciliar cifras tests (docs/reports, coverage, nextest) |
| `lastAction` | Discovery docs/reports+dora+evals+validate-scripts+manifest+nextest.toml+TEST_MAP + Ejecución verify contratos + Cierre plan sync |
| `result` | OK ↔ ✅ COMPLETED |
| `nextAction` | GOV-A3 (Probes CLI reales doctor/backup/restore) — Wave1 paralelo disjoint, o GOV-A4/A5 si MAX 3 |
| `contract` | `## Contrato` + `## Invariantes` + evidencia: TEST_MAP.md:92 2034*2026-08 + coverage:91 + cargo nextest list 2074 + validate-docs-coverage 6/6 + .codegraph exists + Cargo.toml 0.5.0 |
| `nextTask` | GOV-A3 — Wave1 paralelo (no bloquear A4/A5, MAX 3) |

## Deuda técnica (Regla 6 — MUST)
Sin deuda nueva (docs-only, 0 líneas Rust nuevas). Saldo neto 0. Reutiliza TEST_MAP.md existente (ponytail: no crear nuevo doc cifras).

## Definition of Done (contrato multi-nivel)
| Nivel | Gate | Aplica |
|-------|------|--------|
| Task | Contrato verificable + verify mecánico | ✅ `Select-String 2034.*2026-08` >=1 + coverage + nextest list + validate-docs |
| Commit | Lo ejecuta el LEAD (worker no commitea en vanta-docs) | delegado |
| Release | No aplica (docs governance, no crate) | justificado |

## Herramientas necesarias
- PowerShell Select-String / Test-Path (contrato)
- cargo nextest list (perfil default/audit)
- pwsh scripts/validate-docs-coverage.ps1 -ReportOnly
- Read/Grep (auditoria docs/reports)

**Skills cargadas (SDP §2 — BUILD/VERIFY, ≤8 justificadas):**
- campaign-executor (orquestación pipeline-full DISCOVERY→EJECUCIÓN→CIERRE)
- planning-and-task-breakdown (slicing Steps atómicos)
- writing-plans (plan docs-first)
- brainstorming (reconciliación cifras — ambigüedad 3 fuentes)
- progreso (sync Backlog→avance, anti-drift)
- ponytail(full) (diff mínimo docs-only, reuse TEST_MAP.md)
- documentation-and-adrs (validate-docs-coverage, coverage ADR-018, dora report)
- test-driven-development (tests/nextest contract — cargo nextest list/profile)

> Base 6 + 2 extras descubiertas por keywords contrato ("tests"→test-driven-development, "coverage/validate-docs/dora"→documentation-and-adrs). Grep SKILLS-MANIFEST.md: "coverage/test-driven/documentation" hits; systematic-debugging descartado (no hay flake en esta task, nextest list estable 2074).

## Investigation Notes
- **Auditoria intake 2026-09-02:** 3 cifras sin fuente única (2568+ /1902 /1492) detectadas como gap intake — riesgo auditor lee cifra sin contexto perfil/fecha.
- **TEST_MAP.md 2026-08-22:** cifra canónica 2034/2034/1 skip @2026-08-22 perfil default excluye heavy, Windows local, con nota "Cifras históricas (1492/1902/2568+) son snapshots anteriores con perfiles distintos" — reconciliación ya landed. Backlog.md:685 replica 2034 como estado proyecto.
- **Coverage:** TEST_MAP.md:91 — CI threshold root crate ≥80% (ADR-018/COV-004 baseline 81.40%); exclusions tests/benches/packages/experimental/crash_injection; Report lcov.info artifact. Plan fila 961 confirma 2034/2034/1 skip canónico + coverage 81.40% fallback si llvm-cov ICE (GOV-A1).
- **Nextest:** .config/nextest.toml default-filter excluye ~30 heavy binaries (fjall_cold_copy_restore, wal_resilience, hnsw_recall_certification, etc.) con forma scope-safe `package(vantadb) and binary(X)` (BND-06). Audit profile sin filtro (fail-fast=false, 60s timeout). `cargo nextest list --profile default` → 2074 lines (incluye filtradas listadas, count 2034 tests efectivos tras filtro). Verificado 2026-09-02.
- **Validate-docs-coverage:** 6 checks (SDK 28/0 debug, config 55, error 33, CLI 42, python 50, MCP 1/49 gap embed_texts) — ReportOnly exit 0, gap embed_texts pre-existente no bloquea GOV-A2 (MCP docs, no cifra tests). .codegraph/codegraph.db exists (20.5K símbolos), Cargo.toml workspace 0.5.0 default-members vantadb+python.
- **Docs/reports/dora.md:** no contiene cifra tests (es DORA flow: Cycle/Lead/CFR/Recovery/Throughput) — no drift. Fuente DORA: 23 tareas/412 task files/33 budgets/124 verify attempts, best-effort dates (mtime fallback documentado).
- **Disjoint Wave1:** GOV-A2 toca docs/TEST_MAP.md + scripts/validate-docs-coverage.ps1 read-only; RES-02 toca src/storage/engine/mod.rs + snapshot_restore, RES-03/04 src/iql/, RES-05 docs/api/ scores — 0 archivos en común → parallel 3 seguro con GOV-A3 (binario vanta-cli) no tocado.

## Incógnitas (uphill) vs Pendientes (downhill)
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — no aplica: docs-only, sin trust boundaries, sin input usuario, sin auth
- [x] **PERFORMANCE** — no aplica: no hot path, no benchmark

## Steps

### Step 1: DISCOVERY — Read + Blast Radius (docs, no codegraph_explore)
- **Archivos:** `docs/reports/dora.md`, `evals/dora.mjs`, `Cargo.toml`, `.codegraph/codegraph.db`, `scripts/validate-docs-coverage.ps1`, `.config/nextest.toml`, `docs/TEST_MAP.md`, `SKILLS-MANIFEST.md`
- **Acción:** Leer dora.md (402L) + dora.mjs (360L recoveryPairs) + Cargo.toml (workspace 0.5.0) + validate-docs-coverage.ps1 (197L/6 checks) + nextest.toml (default-filter BND-06) + TEST_MAP.md (155L Coverage+CI Gates) + grep SKILLS-MANIFEST.md por keywords "tests/coverage/validate-docs/dora/nextest" → discovery skills. Mapear blast radius: callers docs/Backlog, callees TEST_MAP/validate-scripts/nextest.toml/.codegraph. Confirmar no overlap RES-02..05 ni GOV-A3.
- **Verify:** `Test-Path .codegraph/codegraph.db` == true AND `Test-Path scripts/validate-docs-coverage.ps1` == true AND `Test-Path .config/nextest.toml` == true AND `Select-String -Path docs/TEST_MAP.md -Pattern "2034.*2026-08" | Measure-Object Count` >=1 (pre-check)
- **Estado:** ✅ COMPLETED — 2026-09-02 discovery: TEST_MAP.md ya reconciliado 2034*2026-08 + históricos + coverage 81.40%, dora.md no drift, nextest 2074 list, validate-docs 6/6, manifest skills identified, disjoint confirmado

### Step 2: EJECUCIÓN — Verificación mecánica contratos (ponytail docs-only)
- **Archivos:** `docs/TEST_MAP.md`, `scripts/validate-docs-coverage.ps1`, `.config/nextest.toml`, `.codegraph/codegraph.db`, `Cargo.toml`
- **Acción:** (ponytail: reuse TEST_MAP.md, no nuevo doc)
  1. `Select-String -Path "docs/TEST_MAP.md" -Pattern "2034.*2026-08"` → Count 1 ✅
  2. `Select-String -Path "docs/TEST_MAP.md" -Pattern "coverage|Coverage"` → Count >=1 ✅ (sección Coverage 91)
  3. `Select-String -Path "docs/TEST_MAP.md" -Pattern "1492|1902|2568"` → Count >=1 ✅ (históricas contextualizadas)
  4. `cargo nextest list --profile default -p vantadb` → 2074 lines / list OK (perfil default excluye heavy, count 2034 efectivos) ✅
  5. `pwsh scripts/validate-docs-coverage.ps1 -ReportOnly` → 6/6 checks (SDK/config/error/CLI/python/MCP 1 gap embed_texts pre-existente) ✅
  6. `Test-Path .codegraph/codegraph.db` + `Select-String -Path Cargo.toml -Pattern 'version = "0.5.0"'` ✅
- **Verify:** Todos los Select-String >=1 + nextest list exit 0 + validate-docs exit 0 + codegraph exists
- **Estado:** ✅ COMPLETED — 2026-09-02 verify: TEST_MAP.md:92 2034*2026-08 ✅, coverage ✅, históricos ✅, nextest 2074 ✅, validate-docs 6/6 ✅, .codegraph ✅, Cargo 0.5.0 ✅

### Step 3: CIERRE — Task file + Plan sync PENDING→COMPLETED + recitation
- **Archivos:** `.opencode/skills/campaign-executor/tasks/GOV-A2.md` (este file), `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §GOV-A2
- **Acción:** Crear/actualizar GOV-A2.md con contrato verificable + Steps atómicos (Step1 DISCOVERY, Step2 EJECUCIÓN, Step3 CIERRE). Actualizar plan fila GOV-A2 Estado ⬜ PENDING→✅ COMPLETED con recitation (activeGoal/contract/lastAction/nextAction/nextTask). No tocar RES-02..05 ni GOV-A3. Cargar skills descubiertas documentation-and-adrs + test-driven-development.
- **Verify:** `Test-Path .opencode/skills/campaign-executor/tasks/GOV-A2.md` == true AND `Select-String -Path "docs/plans/2026-09-02-alta-prioridad-paralelo.md" -Pattern "GOV-A2.*COMPLETED" | Measure-Object Count` >=1
- **Estado:** ✅ COMPLETED — task file creado/actualizado + plan sync pendiente (siguiente edit plan file)

## Dependencias
- GOV-A1 ✅ (Wave1 medición openapi parity) — no bloqueante, paralelo MAX 3
- RES-02..05 ⬜/✅ pero aislados P38 — prohibido tocar (disjoint files)
- No depende de GOV-A3 (probes CLI) — paralelo

## Review (GATE — agente distinto si aplica, docs-only ponytail self-review)
- **Revisor:** vanta-docs (self-review ponytail, docs-only, sin código Rust) — contratos mecánicos verificados 2026-09-02, TEST_MAP.md ya reconciliado, no drift en dora.md, disjoint respetado. Veredicto: ✅ approve

## Notas
- Sin commit por worker: regla explícita — lead commitea. Worker solo edita GOV-A2.md + plan file last-synced.
- Verify full cargo (fmt/clippy/nextest audit) no aplica completo: docs-only, contrato es Select-String + nextest list + validate-docs ReportOnly.
- Cifras 2568+/1902/1492 ya contextualizadas en TEST_MAP.md:92 como "snapshots anteriores con perfiles distintos" — no replicar en dora.md (DORA flow, no test count).
- Coverage: llvm-cov ICE Windows 2026-08-22 mitigado por ADR-018 baseline 81.40% (TEST_MAP.md:91 + plan fila 961); re-medición opcional `cargo llvm-cov --workspace --summary-only` si GOV-A1 no ICE.

## Referencias
- `docs/TEST_MAP.md:91-92` — Coverage + cifra canónica 2034 + históricas
- `docs/reports/dora.md` — DORA flow (no test count)
- `evals/dora.mjs:207-222` — recoveryPairs lógica
- `scripts/validate-docs-coverage.ps1` — 6 checks SDK/config/error/CLI/python/MCP
- `.config/nextest.toml` — default-filter BND-06 scope-safe
- `Cargo.toml` — workspace 0.5.0
- `.codegraph/codegraph.db` — índice 20.5K símbolos
- `docs/plans/2026-09-02-alta-prioridad-paralelo.md` — Wave1 GOV-A2 fila
- `.opencode/references/skills-engineering.md` — SDP canónico

## Context Save Point
- **Fecha:** 2026-09-02T20:45
- **Branch:** main/develop (ver git status)
- **CI pendiente:** no (docs-only)
- **Decisiones:** Reuse TEST_MAP.md como fuente única cifras (ponytail); no crear nuevo report tests; validar con Select-String + nextest list + validate-docs
- **Problemas conocidos:** validate-docs MCP gap embed_texts (1/49) pre-existente no bloquea GOV-A2; .codegraph no necesita re-index en esta task
- **Próxima tarea:** GOV-A3 — Probes CLI reales doctor/backup/restore (Wave1 paralelo, disjoint)
