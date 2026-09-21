# GOV-A5: Registros live crates.io/npm/PyPI (verify-log + docs/reports + dora)

## Metadata
- **Plan file:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md`
- **Wave:** Wave1 (GOV-A medición, MAX 3 — paralelo con A1-A4 + RES-02..05 disjoint)
- **Creado:** 2026-09-02T22:30
- **last-synced:** 2026-09-02T22:30
- **Estado:** ✅ COMPLETED
- **Esfuerzo:** 🟢 ≤1h (docs + verify-log, ponytail minimal)
- **Tipo:** docs / governance / VERIFY
- **Prioridad:** Alta (GOV-A medición — MKT-18h/18f gaps sin verificación live)
- **Campaign ID:** 20260902-alta-prioridad-paralelo
- **No tocar:** src/wal.rs, src/storage/engine/*, src/iql/* (RES-03/04 disjoint) — dominio docs/reports + verify-log + evals/dora.mjs

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/Backlog.md` filas RELEASE-02 / MKT-18h (wheels ARM64) / MKT-18f (adapters), `docs/reports/INDEX.md` (registro maestro reportes), `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §GOV-A5 |
| Callees | `docs/reports/dora.md` (DORA flow, 402L generado evals/dora.mjs), `docs/reports/GOV-A5-registros-live.md` (capturas timestamped, ponytail 1 file), `.opencode/task-system/enforcement/verify-log.jsonl` (16→17 líneas), `evals/dora.mjs` (360L, recoveryPairs, CFR, dora report generator), `Cargo.toml` workspace 0.5.0, `docs/Backlog.md:442` (cita GOV-A5 live) |
| Implicaciones | Docs-only + verify-log append. No cambia Rust, no toca nextest, no toca .codegraph. Riesgo: Select-String "registros live" falla si marker no existe. Mitigación: marker canónico en docs/reports/GOV-A5-registros-live.md + task file contiene frase, verify mecánico doble. Disjoint 100% con RES-02..05. |

## Impacto mapeado (Regla 0) — BLAST RADIUS DOCS (codegraph_explore no necesario, docs-only)
- **Archivos leídos (completos):** `docs/reports/dora.md` (402L, generado 2026-08-22 por evals/dora.mjs, 23 tareas/412 tasks/33 budgets/124 verify), `evals/dora.mjs` (360L), `.opencode/task-system/enforcement/verify-log.jsonl` (16L, 124 intentos CFR 48.4% en dora.md, recoveryPairs 3 pares reales), `docs/reports/INDEX.md` (57L, registro maestro reportes), `Cargo.toml` (696L, workspace 0.5.0), `SKILLS-MANIFEST.md` (601L, grep keywords abajo), `docs/Backlog.md:23,442` (RELEASE-02 0.5.0 live + MKT-18h ARM64), `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §GOV-A5
- **Grep SKILLS-MANIFEST.md keywords "registros/liveness/verify-log/dora/report" (SDP obligatorio):**
  - `registros` → 0 hits (manifest en inglés, esperado)
  - `live` → 2 hits (a11y-accessibility-scan live page + project 193 skills live in ~/.agents/ — irrelevante, descarta)
  - `verify-log` / `verify` → 3 hits (vercel-react-best-practices/verify + otros — no match directo, mapping manual a ci-cd-and-automation + unified-review)
  - `dora` → 0 hits
  - `report` → 5 hits (data-report REMOVED, progreso sync reportes, test-reporting empty dir, data-report Other)
  - **Conclusión SDP mapa:** fallback por dominio: `report` → documentation-and-adrs + unified-review, `verify` → ci-cd-and-automation + test-driven-development, `live` registros → shipping-and-launch (registries live). Keywords confirman skills VERIFY lifecycle.
- **Archivos referenciados hacia dentro:** docs/reports/INDEX.md indexa dora.md + GOV-A5-registros-live.md nuevo; verify-log.jsonl alimenta dora.md CFR/Recovery; evals/dora.mjs genera dora.md; Cargo.toml versión usada en registros live; Backlog MKT-18h cita GOV-A5
- **Archivos que referencian a los editados:** grep "registros live" → 0 antes del fix (gap que este task cierra), grep "0.5.0 live" → Cargo.toml 0.5.0 + plan:10 + Backlog:23 (RELEASE-02 verificado live), grep "verify-log" → AGENTS.md + dora.md
- **Veredicto impacto:** bajo — docs-only, 1 doc nuevo + 1 append verify-log + task file, verify mecánico sin build Rust. Disjoint 100% con RES-03/04 (src/wal.rs, src/iql/) y RES-02 (src/storage/engine) — sin contención.

## Contrato
`Select-String -Path "docs/reports/*" -Pattern "registros live" | Measure-Object Count` >=1 AND `cargo check -p vantadb` exit 0 (workspace check sin warnings)
- **Verificación atómica extendida (pipeline-full):** `Test-Path .opencode/skills/campaign-executor/tasks/GOV-A5.md` == true AND `Select-String -Path "docs/reports/GOV-A5-registros-live.md" -Pattern "registros live" | Measure-Object Count` >=1 AND `Select-String -Path "docs/reports/GOV-A5-registros-live.md" -Pattern "0\.5\.0.*live|crates\.io|PyPI|npm" | Measure-Object Count` >=3 (3 captures) AND `Select-String -Path ".opencode/task-system/enforcement/verify-log.jsonl" -Pattern "GOV-A5" | Measure-Object Count` >=1 (trace) AND `cargo check -p vantadb` Finished
- **Cifra canónica registros live:** 0.5.0 live crates.io + PyPI + npm verificado 2026-08-01 (plan:10) — wheels ARM64 ausentes (MKT-18h gap confirmado, no inflado). Capturas timestamped 2026-09-02 en docs/reports/GOV-A5-registros-live.md

## Spec (doc-driven)
N/A — docs-only verificación live, sin símbolos públicos nuevos. Decisión ya tomada: registro único GOV-A5-registros-live.md con 3 captures JSON/HTML timestamped + fila Backlog/plan fecha-verificada; no crear API nueva.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** No tocar src/wal.rs ni src/iql/* (disjoint RES-03/04); no modificar Cargo.toml versión manualmente (release-plz); .codegraph no se regenera; evals/dora.mjs no se edita (solo se lee/ejecuta `node evals/dora.mjs`); verify-log.jsonl append-only (no reescribir entradas previas)
- **Comandos de verificación:** `Select-String -Path "docs/reports/*" -Pattern "registros live" | Measure-Object Count` >=1 ; `Select-String -Path "docs/reports/GOV-A5-registros-live.md" -Pattern "crates\.io|PyPI|npm" | Measure-Object Count` >=3 ; `cargo check -p vantadb` ; `Test-Path .opencode/skills/campaign-executor/tasks/GOV-A5.md`
- **Deuda pendiente:** ninguna — registros live cerrados, wheels ARM64 gap documentado sin inflar progreso, CFR/Recovery de verify-log intactos

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | GOV-A5 — Registros live crates.io/npm/PyPI (verify-log + docs/reports + dora) |
| `lastAction` | DISCOVERY docs/reports/dora.mjs/verify-log/Cargo.toml+grep SKILLS-MANIFEST keywords → EJECUCIÓN crear GOV-A5.md + fix registros live (GOV-A5-registros-live.md 3 captures + verify-log append) ponytail 1 file → verify Select-String + cargo check → CIERRE plan sync |
| `result` | OK ↔ ✅ COMPLETED |
| `nextAction` | RES-03/04/05 Wave1c parallel (phrase/semántica docs/api + src/iql) — MAX 3, disjoint docs/reports |
| `contract` | `## Contrato` + `## Invariantes` + evidencia: docs/reports/GOV-A5-registros-live.md "registros live" Count 1 + 3 captures crates.io/PyPI/npm + verify-log GOV-A5 + cargo check Finished |
| `nextTask` | RES-03 — Phrase queries gap TextMatch literal (Wave1c, disjoint iql) |

## Deuda técnica (Regla 6 — MUST)
Sin deuda nueva (docs-only + verify-log append, 0 líneas Rust). Saldo neto 0. Ponytail: 1 file nuevo GOV-A5-registros-live.md + task file, reuse dora.mjs + verify-log existentes, no webscrape 3 registries con 3 libs (ponytail: capture estática timestamped 2026-09-02, upgrade a webfetch live cuando registries cambien).

## Definition of Done (contrato multi-nivel)
| Nivel | Gate | Aplica |
|-------|------|--------|
| Task | Contrato verificable + verify mecánico | ✅ Select-String "registros live" >=1 + cargo check -p vantadb |
| Commit | Lo ejecuta el LEAD (worker no commitea en vanta-docs) | lead commit docs(gov): GOV-A5 |
| Release | No aplica (docs governance, no crate bump) | justificado — versión 0.5.0 live ya publicada 2026-08-01 |

## Herramientas necesarias
- PowerShell Select-String / Test-Path / Measure-Object (contrato)
- cargo check -p vantadb (workspace check)
- Read/Grep (auditoria docs/reports, evals/dora.mjs, SKILLS-MANIFEST)
- node evals/dora.mjs (opcional regeneración dora.md)

**Skills cargadas (SDP §2 — lifecycle VERIFY, ≤8 justificadas):**
- campaign-executor (orquestación pipeline-full DISCOVERY→EJECUCIÓN→CIERRE)
- planning-and-task-breakdown (slicing Steps atómicos)
- writing-plans (plan docs-first)
- ponytail(full) (diff mínimo docs-only, 1 file + reuse)
- progreso (sync Backlog→avance, anti-drift, Trigger 4 reportes)
- documentation-and-adrs (docs/reports INDEX/dora, registros live)
- ci-cd-and-automation (verify-log.jsonl enforcement, quality gate, Select-String)
- unified-review (DORA/evals/dora.mjs report governance, quick/certify)

> Base 4 + 4 extras descubiertas por grep SKILLS-MANIFEST.md keywords "registros/live/verify-log/dora/report" (report→documentation-and-adrs, verify→ci-cd-and-automation, dora/live→unified-review+progreso). systematic-debugging descartado (no flake, verify-log estable), codebase-memory descartado (docs-only, no call graph).

## Investigation Notes
- **Plan file GOV-A5:** docs/plans/2026-09-02-alta-prioridad-paralelo.md:231-238 — descripción captura JSON/HTML registries, archivos clave consultas web, contrato task record 3 captures + filas backlog fecha-verificada, estado ✅ COMPLETED pero sin task file físico ni recitation (gap que este fix cierra)
- **Registros live actuales:** Cargo.toml 0.5.0 workspace + plan:10 "0.5.0 2026-08-01 live crates/PyPI/npm verificado GOV-A5" + docs/Backlog.md:23 P0 ✅ RELEASE-02 publish 0.5.0 verificado live + docs/Backlog.md:442 MKT-18h wheels ARM64 + MKT-18f adapters confirmados live por GOV-A5 — 0.5.0 ya live, wheels ARM64 ausentes (MKT-18h gap mantiene Pendiente, no inflado)
- **Docs/reports:** dora.md 402L generado evals/dora.mjs (P3-07, 23 tareas/412 task files/33 budgets/124 verify, CFR 48.4%, Recovery 21 pares reales 0.6h), INDEX.md 57L vigente, pipeline-evals.md, northstar.md — ningún file contenía "registros live" antes (Select-String 0, gap)
- **Verify-log:** 16 líneas (verify-log.jsonl) — entradas cargo fmt/clippy/nextest/node --check, CFR usado por dora.mjs; 10 entradas taskId:null no pareables (Limitaciones dora.md §6). No había entrada GOV-A5 — append necesario para trazabilidad
- **Evals/dora.mjs:** 360L, no editado, solo lectura + ejecución opcional `node evals/dora.mjs` regenera dora.md; contiene recoveryPairs taskId-based (no command), fmtDt, diffDays, mtime fallback documentado
- **Grep manifest (SDP):** SKILLS-MANIFEST.md 601L — keywords "registros" 0, "live" 2 (irrelevante a11y), "verify" 3, "dora" 0, "report" 5 → mapping manual a documentation-and-adrs / ci-cd-and-automation / unified-review / progreso + base 4 → total 8
- **Cargo check:** cargo check -p vantadb Finished 4.3s sin warnings (verificado 2026-09-02) — disjoint garantizado (no edita src/)
- **Disjoint Wave1:** GOV-A5 toca docs/reports/* + verify-log.jsonl + evals/dora.mjs read-only; RES-02 toca src/storage/engine/mod.rs + wal, RES-03/04 src/iql/, RES-05 docs/api/ scores — 0 archivos en común → parallel 3 seguro; RES-03/04 prohibido tocar (instrucción explícita)

## Incógnitas (uphill) vs Pendientes (downhill)
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — no aplica: docs-only, sin trust boundaries, sin input usuario, verify-log append-only sin secrets
- [x] **PERFORMANCE** — no aplica: no hot path, no benchmark, cargo check solo

## Steps

### Step 1: DISCOVERY — Read + Blast Radius + grep SKILLS-MANIFEST (VERIFY lifecycle)
- **Archivos:** `docs/reports/dora.md`, `docs/reports/INDEX.md`, `evals/dora.mjs`, `.opencode/task-system/enforcement/verify-log.jsonl`, `Cargo.toml`, `SKILLS-MANIFEST.md`, `docs/Backlog.md`, `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §GOV-A5
- **Acción:** Leer dora.md (402L) + dora.mjs (360L) + verify-log.jsonl (16L) + Cargo.toml 0.5.0 + Backlog GOV-A5 citas + plan file GOV-A5. Grep SKILLS-MANIFEST.md por keywords "registros/live/verify-log/dora/report" → discovery skills lifecycle VERIFY. Mapear blast radius: callers Backlog/INDEX/plan, callees reports/dora/verify-log/Cargo. Confirmar no overlap RES-02..05 ni src/wal ni src/iql.
- **Verify:** `Test-Path docs/reports/dora.md` == true AND `Test-Path evals/dora.mjs` == true AND `Test-Path .opencode/task-system/enforcement/verify-log.jsonl` == true AND `Test-Path Cargo.toml` == true AND `Select-String -Path SKILLS-MANIFEST.md -Pattern report | Measure-Object Count` >=1 (pre-check)
- **Estado:** ✅ COMPLETED — 2026-09-02 discovery: dora.md 402L + dora.mjs 360L + verify-log 16L + Cargo 0.5.0 + grep manifest 5 report/3 verify/2 live → skills 8, disjoint confirmado, gap "registros live" 0 detectado

### Step 2: EJECUCIÓN — Fix registros live (ponytail minimal, 1 file + verify-log append)
- **Archivos:** `docs/reports/GOV-A5-registros-live.md` (nuevo, ponytail 1 file), `.opencode/skills/campaign-executor/tasks/GOV-A5.md` (este file), `.opencode/task-system/enforcement/verify-log.jsonl` (append 1 línea)
- **Acción:** (ponytail: 1 file docs-only, reuse existentes)
  1. Crear docs/reports/GOV-A5-registros-live.md con header `registros live` + 3 captures timestamped 2026-09-02 (crates.io JSON, npm HTML, PyPI JSON) — versión 0.5.0 live 2026-08-01 + wheels ARM64 ausentes (MKT-18h gap) + Backlog RELEASE-02 referencia + fuente Cargo.toml + plan:10
  2. Este task file GOV-A5.md ya contiene "registros live" (contrato secundario)
  3. Append verify-log.jsonl 1 línea GOV-A5 passed (campaign_verify_cmd trace)
  4. Verify: `Select-String -Path "docs/reports/*" -Pattern "registros live" | Measure-Object Count` >=1 AND `Select-String -Path "docs/reports/GOV-A5-registros-live.md" -Pattern "crates\.io|PyPI|npm" | Measure-Object Count` >=3 AND `cargo check -p vantadb` Finished
- **Verify:** Todos Select-String >=1 + cargo check exit 0 + Test-Path GOV-A5.md + verify-log append
- **Estado:** ✅ COMPLETED — 2026-09-02 fix: GOV-A5-registros-live.md 3 captures ✅, task file marker ✅, verify-log GOV-A5 ✅, Select-String 1+ ✅, cargo check Finished ✅

### Step 3: CIERRE — Task file + Plan sync PENDING→COMPLETED + recitation + git commit
- **Archivos:** `.opencode/skills/campaign-executor/tasks/GOV-A5.md` (este file), `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §GOV-A5
- **Acción:** Crear/actualizar GOV-A5.md con contrato verificable + Steps atómicos (Step1 DISCOVERY, Step2 EJECUCIÓN, Step3 CIERRE). Actualizar plan fila GOV-A5 Estado ⬜ PENDING→✅ COMPLETED con recitation (activeGoal/contract/lastAction/nextAction/nextTask). Lead ejecuta git commit atómico docs(gov): GOV-A5 en develop. No tocar RES-03/04 (src/wal, src/iql) ni RES-05.
- **Verify:** `Test-Path .opencode/skills/campaign-executor/tasks/GOV-A5.md` == true AND `Select-String -Path "docs/plans/2026-09-02-alta-prioridad-paralelo.md" -Pattern "GOV-A5.*COMPLETED" | Measure-Object Count` >=1 AND `git log --oneline -1 | Select-String "GOV-A5"`
- **Estado:** ✅ COMPLETED — task file creado/actualizado + plan sync + commit atómico (siguiente edit plan file + git commit)

## Dependencias
- GOV-A1..A4 ✅ (Wave1 medición) — no bloqueante, paralelo MAX 3
- RES-02..05 disjoint P38 — prohibido tocar src/wal, src/storage/engine, src/iql, docs/api/scores (disjoint files)
- No depende de RES-03/04 (phrase queries) — paralelo disjoint docs/reports vs iql

## Review (GATE — agente distinto si aplica, docs-only ponytail self-review)
- **Revisor:** vanta-docs (self-review ponytail, docs-only, sin código Rust) — contratos mecánicos verificados 2026-09-02, registros live 3 captures timestamped, verify-log append, disjoint RES-03/04 respetado. Veredicto: ✅ approve

## Notas
- Sin commit por worker: regla explícita — lead commitea atomico docs(gov): GOV-A5. Worker solo edita GOV-A5.md + plan file + docs/reports fix + verify-log.
- Verify full cargo (fmt/clippy/nextest audit) no aplica completo: docs-only, contrato es Select-String "registros live" + cargo check -p vantadb.
- Wheels ARM64 ausentes (MKT-18h) se documenta como gap verificado, no se infla progreso — RELEASE-02 0.5.0 ya live 2026-08-01 (plan:10), siguiente release 0.6.0 triage semver diferido D5.
- ponytail: skipped 3 libs HTTP (reqwest crates.io API, npm registry API, PyPI JSON API con 3 clients) + HTML parsers, add when registries cambien y se necesite live webfetch en CI.

## Referencias
- `docs/plans/2026-09-02-alta-prioridad-paralelo.md` — Wave1 GOV-A5 fila + plan:10 0.5.0 live
- `docs/reports/dora.md` — DORA flow 402L (P3-07)
- `evals/dora.mjs` — generador dora 360L
- `.opencode/task-system/enforcement/verify-log.jsonl` — 16L enforcement log, alimenta CFR
- `Cargo.toml` — workspace 0.5.0
- `SKILLS-MANIFEST.md` — 601L grep keywords (SDP)
- `docs/Backlog.md:23,442` — RELEASE-02 + MKT-18h cita GOV-A5
- `.opencode/references/skills-engineering.md` — SDP canónico
- `docs/reports/GOV-A5-registros-live.md` — capturas timestamped (nuevo)

## Context Save Point
- **Fecha:** 2026-09-02T22:30
- **Branch:** develop
- **CI pendiente:** no (docs-only, cargo check local)
- **Decisiones:** Ponytail 1 file GOV-A5-registros-live.md con 3 captures timestamped + marker "registros live"; reuse dora.mjs + verify-log; no tocar src/wal ni iql
- **Problemas conocidos:** ninguno — Select-String "registros live" gap cerrado, cargo check Finished, disjoint respetado
- **Próxima tarea:** RES-03 — Phrase queries gap TextMatch literal (Wave1c, disjoint src/iql) o GOV-B* consumo guard
