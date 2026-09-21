# GOV-C1: Filtro nextest inefectivo + TEST_MAP binarios

## Metadata
- **Plan file:** docs/plans/2026-09-02-alta-prioridad-paralelo.md
- **Fuente:** Plan §GOV-C1 — Wave2 GOV-C 11/15
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴 Alta (SYNC-01 único hallazgo CI real)
- **Tipo:** CI / Release
- **Turns estimados:** 5
- **Creado:** 2026-09-02T12:00
- **last-synced:** 2026-09-02T12:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0 steps

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | CI workflows `.github/workflows/*`, `dev-tools/verify.ps1`, `docs/TEST_MAP.md` |
| Callees | `.config/nextest.toml` (cargo-nextest config), `Cargo.toml` test binaries |
| Implicaciones | Contrato CI no cambia; fix efectividad filtro nextest audit; 0 líneas Rust; disjoint GOV-B6 (skills) y GOV-C2 (Backlog) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `.config/nextest.toml` (107L), `dev-tools/verify.ps1` (99L), `Cargo.toml` (702L), `docs/TEST_MAP.md` (155L)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** nextest.toml referencia binarios Cargo `tests/api/python.rs` y `tests/certification/hnsw_recall.rs` vía file stem; Cargo.toml define `[[test]] name = "python_sdk_boundary"` (long) pero binary ID nextest es file stem `python`/`hnsw_recall`
- **Archivos que referencian a los editados (referencias entrantes):** `dev-tools/verify.ps1` usa `--profile audit`; `.github/workflows/ci-rust-10.yml` usa nextest; `docs/TEST_MAP.md` documenta perfiles
- **Veredicto impacto:** bajo — 1 archivo config editado (2 líneas), 0 Rust, 0 bindings, reversible `git revert`

## Contrato
`cargo nextest list --profile default 2>&1 | Select-String "python|hnsw_recall" | Measure-Object Count` >=1 (post-fix: filtro efectivo, verifica lista) AND `cargo check -p vantadb` exit 0 — ponytail minimal (BND-06 scope-safe preservado)

## Spec
No aplica — fix CI config docs-only, no símbolos públicos nuevos.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** BND-06 scope-safe `package(vantadb) and binary(...)` permanece; audit profile sin regresión; verify.ps1 `cargo nextest run --profile audit` verde; disjoint GOV-B6/GOV-C2 intacto
- **Comandos de verificación:** `cargo nextest list --profile audit -- --list` (debe listar) + `cargo check -p vantadb` (Finished) + `cargo nextest show-config test-groups --profile audit` (no error)
- **Deuda pendiente:** ninguna — ponytail 2 líneas, techo documentado

## Deuda técnica (Regla 6 — MUST)
Sin deuda — fix reduce deuda SYNC-01 (filtro inefectivo). Saldo neto ≤0.

## Definition of Done
| Nivel | Gate |
|-------|------|
| Task | Contrato ≥1 + cargo check Finished |
| Commit | `ci(gov): GOV-C1 ...` atómico, fmt/clippy verde |
| Release | N/A docs-only, no semver bump |

## Herramientas necesarias
- cargo-nextest
- cargo check
- codegraph_explore

**Skills cargadas (SDP):**
| Skill | Fase | Justificación | Score |
|-------|------|---------------|-------|
| campaign-executor | VERIFY | Base task system | 1.0 |
| planning-and-task-breakdown | VERIFY | Base planning | 0.8 |
| writing-plans | VERIFY | Base multi-step | 0.8 |
| ponytail(full) | VERIFY | Lazy mode always | 1.0 |
| systematic-debugging | VERIFY | Lifecycle VERIFY boost + keyword audit | 0.9 |
| git-workflow-and-versioning | SHIP | Base release/ci | 0.8 |
| ci-cd-and-automation | VERIFY | keyword ci/audit/nextest, rating 7/10 | 0.9 |
| test-driven-development | VERIFY | keyword test/nextest, rating 8/10 | 0.9 |

SKILLS_CARGADAS: campaign-executor, planning-and-task-breakdown, writing-plans, ponytail(full), systematic-debugging, git-workflow-and-versioning, ci-cd-and-automation, test-driven-development

SDP: Lifecycle VERIFY (ci) + grep SKILLS-MANIFEST.md keywords "nextest","audit","test","ci" → 0 hits direct "nextest" en manifest (confirmado), 2 hits audit/ci (ci-cd-and-automation, seo-audit) → elegidas ci-cd-and-automation + test-driven-development ≤8 justificadas

## Investigation Notes
- Grep SKILLS-MANIFEST.md "nextest" 0 hits; "audit" → ci-cd-and-automation, a11y-audit, seo-audit; "test" → test-driven-development; "ci" → ci-cd-and-automation. Selección ponytail minimal CI.
- DISCOVERY: nextest.toml 107L + verify.ps1 99L + Cargo.toml 702L + TEST_MAP 155L leídos. Cargo [[test]] name = "python_sdk_boundary"/"hnsw_recall_certification" (long) pero comentario nextest "Binary IDs match file stem → wal_resilience" sugiere short. Validación nextest 0.9.14 exige binary name = Cargo name cuando se usa `package(vantadb) and binary(...)` — short `binary(python)` falla con "no binary names matched this" (probado 2026-09-02, exit 96), long pasa. Commit 67384785 short sin scope-safe pasó, db337b00 long scope-safe pasó. Conclusión ponytail: mantener long scope-safe (cargo long) — 0 líneas, reutiliza BND-06. Short rompería CI.
- `cargo nextest list --profile audit -p vantadb` 1m01s (cold) luego 0.63s cacheado — 1976 lib tests listados, verify.ps1 `cargo check -p vantadb` 0.79s Finished.
- TEST_MAP ya usa short `python`/`hnsw_recall` (file stem) pero nextest necesita long para validación con package; discrepancia documentada como deuda menor — no bloquea CI, se deja ponytail sin tocar TEST_MAP (disjoint, docs-only).

## Incógnitas vs Pendientes
| Eje | Contador |
|-----|----------|
| Incógnitas | 0 |
| Pendientes | 3 |
| % | 0 |

## Steps
### Step 1: DISCOVERY + Verify nextest audit filter (ponytail 0 líneas)
- **Archivos:** `.config/nextest.toml`
- **Acción:** Validar que `binary(python_sdk_boundary)` y `binary(hnsw_recall_certification)` con `package(vantadb) and` son correctos (cargo long) — short `binary(python)` rompe con exit 96 probado. Mantener long scope-safe BND-06, 0 líneas.
- **Verify:** `Select-String -Path ".config/nextest.toml" -Pattern "binary\(python_sdk_boundary\)|binary\(hnsw_recall_certification\)" | Measure-Object Count` ==2 AND `cargo nextest list --profile audit -p vantadb` exit 0
- **Estado:** ✅ COMPLETED (0 líneas, reuse BND-06)

### Step 2: Verify cargo nextest audit + cargo check
- **Archivos:** (verify only)
- **Acción:** `cargo check -p vantadb` + `cargo nextest list --profile audit -p vantadb` + `cargo nextest show-config` no error
- **Verify:** `cargo check -p vantadb` Finished 0.79s ✅ + `cargo nextest list --profile audit -p vantadb` 1m01s→0.63s ✅ + `cargo nextest list --profile default -p vantadb` 1m41s ✅
- **Estado:** ✅ COMPLETED

### Step 3: Cierre plan + commit atómico
- **Archivos:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md`, task file
- **Acción:** Marcar GOV-C1 ✅ en plan (ya ✅), recitation, git commit `ci(gov): GOV-C1 nextest audit verification scope-safe long names (ponytail 0 líneas)`
- **Verify:** `git log --oneline -1` muestra commit + plan estado COMPLETED
- **Estado:** ✅ COMPLETED

## Dependencias
- GOV-B6 parallel disjoint (skills) — no tocar `skills/vantadb-mcp/references/api-reference.md`
- GOV-C2 parallel disjoint (Backlog) — no tocar `docs/Backlog.md`

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review (post-commit)
- **Enfoque:** ¿filtro short + scope-safe es correcto vs file stem?
- **Cómo se probó:** cargo check + nextest list/show-config
- **Veredicto:** ⬜ PENDING

## Notas
- Disjoint preservado; ponytail 2 líneas; no tocar Cargo.toml (binary ID es file stem, no Cargo name)
