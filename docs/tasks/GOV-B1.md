# GOV-B1: case_studies ficticios → archive interno (Show HN bloqueante)

## Metadata
- **Plan file:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md`
- **Wave:** Wave2 (P27 F1 MEM-01..06 + GOV-B1..B6 + GOV-C1..C3, MAX 3 — paralelo MEM-01 docs/case_studies disjoint)
- **Creado:** 2026-09-02T23:55
- **last-synced:** 2026-09-02T23:55
- **Estado:** ⬜ PENDING → ✅ COMPLETED
- **Esfuerzo:** 🟢 1h (docs-only, ponytail minimal — git mv + README disclaimer + stubs)
- **Tipo:** docs / governance / Show HN bloqueante (D6 eliminar, D3 Show HN Sept)
- **Prioridad:** Alta (bloqueante reputación — T0.1 archive interno, D6 owner decision)
- **Campaign ID:** 20260902-alta-prioridad-paralelo
- **No tocar:** `src/planner.rs`, `src/entity/**`, `src/skills.rs`, `src/wal.rs`, `src/storage/**` (MEM-01 engine disjoint) — dominio `docs/case_studies/*`, `docs/archive/case-studies-unverified/*`, `docs/book/src/case_studies/*`, `docs/master-index.md`

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/Backlog.md` filas GOV-B1/D6, `docs/master-index.md` §Case Studies, `docs/book/src/SUMMARY.md:61` Case Studies, `docs/book/src/case_studies/index.md` stub, `web/src/components/vanta/vanta-data.ts:1005` CASE_STUDIES (3 composite anonymized), `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §GOV-B1 |
| Callees | `docs/archive/case-studies-unverified/README.md` (disclaimer D6, 17L), `docs/archive/case-studies-unverified/rag_edge_device.md` (5942L, EdgeSense), `docs/archive/case-studies-unverified/agent_local_memory_ollama.md` (6398L, CodexAgent), `docs/case_studies/` (eliminado — no existe en HEAD), `DISASTER_RECOVERY_RUNBOOK.md` (no tocar — GOV-B2) |
| Implicaciones | Docs-only move + archive. No cambia Rust, no toca nextest, no toca .codegraph. Riesgo: refs públicas residuales a `docs/case_studies/` (web/docs/book/master-index). Mitigación: stubs + archive README + grep global verificado 2026-08-22 (commit 98612db8 sweep 6 archivos). Disjoint 100% con MEM-01 (src/planner.rs) — MAX 3 respetado. |

## Impacto mapeado (Regla 0) — BLAST RADIUS DOCS (codegraph no necesario, docs-only)
- **Archivos leídos (completos):** `docs/archive/case-studies-unverified/README.md` (800L, disclaimer 2026-08-22 D6), `docs/archive/case-studies-unverified/rag_edge_device.md` (5942L frontmatter active, EdgeSense industrial), `docs/archive/case-studies-unverified/agent_local_memory_ollama.md` (6398L CodexAgent), `docs/book/src/case_studies/index.md` (stub 3L archivado), `docs/book/src/SUMMARY.md:61` (link Case Studies → case_studies/index.md), `docs/master-index.md:159` (sección Case Studies archivados), `web/src/components/vanta/vanta-data.ts:1005-1080` (CASE_STUDIES 3 items anonymized Indie AI Studio/Field Robotics/DevTools Startup — sin disclaimer composite, deuda audit 2026-08-19), `DISASTER_RECOVERY_RUNBOOK.md` / `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` (verificado no tocar — GOV-B2), `SKILLS-MANIFEST.md` (601L, grep keywords abajo), `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §GOV-B1 (contrato), `git log --oneline --grep=GOV-B1` (2 commits: a8a21733 retire + 98612db8 residuales)
- **Grep SKILLS-MANIFEST.md keywords "case_studies", "show hn", "runbook", "disaster" (SDP obligatorio — Lifecycle BUILD docs):**
  - `case_studies` → 0 hits (manifest en inglés, expected — no hay skill case_studies; mapea a docs domain)
  - `show hn` → 0 hits (expected — Show HN es contexto GOV reputación, no skill; mapea a documentation-and-adrs + writing-guidelines)
  - `runbook` → 0 hits (expected — runbook es GOV-B2 DISASTER_RECOVERY, no skill dedicada; mapea a documentation-and-adrs)
  - `disaster` → 0 hits (expected — disaster recovery es GOV-B2, no skill; mapea a documentation-and-adrs + observability)
  - **Conclusión SDP mapa:** 0 hits directos confirman que keywords son dominio docs/governance puro — fallback por dominio a skills docs lifecycle BUILD: `documentation-and-adrs` (ADRs, archiving, disclaimer), `writing-guidelines` (voz Show HN), `spec-driven-development` (doc-first archive), `incremental-implementation` (thin slice mv+stub). Keywords no bloquean selección — justifican ≤8 docs-centric.
- **Archivos referenciados hacia dentro:** docs/book/src/SUMMARY.md indexa case_studies/index.md stub; docs/master-index.md referencia archive/; web/vanta-data.ts CASE_STUDIES es composite independiente (no linka docs/case_studies/); docs/archive README es fuente única disclaimer
- **Archivos que referencian a los editados:** grep "case_studies" → 8 hits (archive README, historial, book SUMMARY, plans GOV-B1, audit archive, plan governance) — 0 hits en docs/case_studies/ (eliminado); grep "rag_edge" → solo archive; verificación pre-move 2026-08-22 hizo sweep global (commit 98612db8: docs/README.md, book/*, graphrag/README.md, skills/vantadb/SKILL.md)
- **Veredicto impacto:** bajo — docs-only, 0 líneas Rust, 2 git mv + 1 README + 2 stubs book ya landed (commits a8a21733 + 98612db8). Disjoint 100% con MEM-01 (src/planner.rs engine) y GOV-B2 (DISASTER_RECOVERY_RUNBOOK.md) — sin contención MAX 3. Ponytail: reuse archive existente, no re-mover, solo verify + task file.

## Contrato
`Test-Path docs/archive/case-studies-unverified/rag_edge_device.md` == true AND `Select-String -Path "docs/archive/case-studies-unverified/README.md" -Pattern "no-público|ilustrativos" | Measure-Object Count` >=1
- **Verificación atómica extendida (pipeline-full docs-only no Rust):**
  - `Test-Path docs/archive/case-studies-unverified/rag_edge_device.md` == True (5942L)
  - `Test-Path docs/archive/case-studies-unverified/agent_local_memory_ollama.md` == True (6398L)
  - `Select-String -Path "docs/archive/case-studies-unverified/README.md" -Pattern "no-público|ilustrativos" | Measure-Object Count` >=1 (Count 1, line 15 "ilustrativos NO verificados")
  - `Select-String -Path "docs/archive/case-studies-unverified/README.md" -Pattern "ARCHIVADOS|no-público|CLD-04" | Measure-Object Count` >=2 (disclaimer completo D6)
  - `Test-Path docs/case_studies` == False (no existe — archivado, contract negativo)
  - `Test-Path docs/book/src/case_studies/index.md` == True (stub 3L: "ARCHIVADOS 2026-08-22")
  - `Select-String -Path "docs/master-index.md" -Pattern "archive/case-studies-unverified" | Measure-Object Count` >=1 (master-index actualizado)
  - `cargo check -p vantadb` → Finished dev (docs-only no Rust — verifica que move no rompió workspace)
- **Cifra canónica archivada:** 2 case studies ficticios (rag_edge_device EdgeSense + agent_local_memory_ollama CodexAgent) retirados 2026-08-22 D6, reubicados `docs/archive/case-studies-unverified/` con README disclaimer + stubs book. Case study real vía CLD-04 (enterprise pilot).

## Spec (doc-driven — ponytail minimal)
N/A — docs-only archiving, sin símbolos públicos nuevos. Decisión ya tomada D6 (owner): retirar ficticios antes de Show HN para honest results (PERF-03 / Regla 11). Implementación: `git mv docs/case_studies/*.md → docs/archive/case-studies-unverified/` + README disclaimer (17L) + stubs `docs/book/src/case_studies/index.md` (no borrar carpeta book/case_studies, solo stub) + sweep refs públicas (docs/README.md, master-index.md, graphrag/README.md, skills/vantadb/SKILL.md) — 2 commits ya landed. Esta task Wave2 solo re-verifica guard anti-regresión.

## Invariantes de dominio (handoff - MUST)
- **Invariantes a preservar:** No tocar `src/planner.rs` / `src/entity/**` / `src/skills.rs` (MEM-01 disjoint F1 search profile); No tocar `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` (GOV-B2 runbook fantasma — task paralela); No tocar `src/wal.rs` / `src/storage/**` (RES-01/02 durabilidad SOLO); No re-crear `docs/case_studies/` público; No editar `Cargo.toml` versión (release-plz); No modificar `web/src/components/vanta/vanta-data.ts` CASE_STUDIES en esta task (deuda composite disclaimer es follow-up GOV-F1/F2, no bloquea archive)
- **Comandos de verificación:** `Test-Path docs/archive/case-studies-unverified/rag_edge_device.md` ; `Select-String -Path "docs/archive/case-studies-unverified/README.md" -Pattern "ilustrativos"` ; `Test-Path docs/case_studies` (False) ; `Test-Path docs/book/src/case_studies/index.md` ; `cargo check -p vantadb` ; `Select-String -Path "docs/master-index.md" -Pattern "archive/case-studies-unverified"`
- **Deuda pendiente:** `web/src/components/vanta/vanta-data.ts:1005` CASE_STUDIES 3 historias anonymized sin disclaimer composite (audit 2026-08-19) — no es parte de GOV-B1 archive, trackeado para GOV-F1 (auditoría raíz) como mejora voz/tono writing-guidelines. No bloquear GOV-B1.

## Recitation (canónico - estructura única)
| Campo recitation (MCP) | → fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | GOV-B1 — case_studies ficticios → archive interno (Show HN bloqueante D6) |
| `lastAction` | DISCOVERY Read archive 3 files + book stubs + master-index + web CASE_STUDIES + grep SKILLS-MANIFEST 4 keywords 0 hits → EJECUCIÓN crear GOV-B1.md + ponytail minimal (reuse 2 commits a8a21733+98612db8, 0 líneas nuevas) → verify Select-String + Test-Path + cargo check → CIERRE plan sync |
| `result` | ✅ → ✅ COMPLETED |
| `nextAction` | GOV-B2 (DISASTER_RECOVERY_RUNBOOK ghost commands) + MEM-01 (F1 search profile) parallel MAX 3 disjoint — Wave2 continúa |
| `contract` | `## Contrato` + evidencia: Test-Path rag_edge_device True + README "ilustrativos" Count 1 + case_studies False + book stub True + cargo check Finished |
| `nextTask` | GOV-B2 — Runbook DR sin comandos fantasma (docs/operations/DISASTER_RECOVERY_RUNBOOK.md) |

## Deuda técnica (Regla 6 - MUST)
Sin deuda nueva (docs-only archiving, 0 líneas Rust, 0 líneas docs nuevas netas — reuse commits existentes). Saldo neto 0. Ponytail: 0 files nuevos en este wave (task file solo), reuse archive/book stubs ya landed, no re-mover, no re-escribir DISCLAIMER. Deuda pre-existente no introducida: web CASE_STUDIES composite sin disclaimer (3 items anonymized) — documentada en Invariantes como follow-up GOV-F1, no infla GOV-B1.

## Definition of Done (contrato multi-nivel)
| Nivel | Gate | Aplica |
|-------|------|--------|
| Task | Contrato verificable + verify mecánico | ✅ Test-Path rag_edge_device True + README "ilustrativos" >=1 + cargo check Finished |
| Commit | Conventional commit docs(gov): GOV-B1 | `docs(gov): GOV-B1 case_studies → archive interno — verify guard + task file (Wave2 docs-only)` |
| Release | No aplica (docs governance, no crate bump) | justificado — archive no cambia API, versión 0.5.0 intacta |

## Herramientas necesarias
- PowerShell Test-Path / Select-String / Measure-Object (contrato)
- cargo check -p vantadb (workspace check docs-only, no build Rust nuevo)
- Read/Grep (auditoria docs/archive, book stubs, SKILLS-MANIFEST)
- git log --oneline --grep GOV-B1 (verificar 2 commits previos)

**Skills cargadas (SDP §2 — Lifecycle BUILD docs, ≤8 justificadas, base 6 + extras):**
- campaign-executor (orquestación pipeline-full DISCOVERY→EJECUCIÓN→CIERRE)
- planning-and-task-breakdown (slicing Steps atómicos, MAX 3)
- writing-plans (plan docs-first, archive design)
- ponytail(full) (diff mínimo docs-only, reuse 2 commits, 0 líneas nuevas)
- documentation-and-adrs (ADRs D6, archiving, disclaimer, stubs)
- writing-guidelines (voz Show HN, honest results PERF-03/Regla 11)
- spec-driven-development (doc-driven archive spec, decision D6)
- incremental-implementation (thin slice mv+README+stub, no big rewrite)

## Steps

### Step 1: DISCOVERY — Read case_studies + grep manifest (SDP obligatorio)
- **Archivos:** `docs/archive/case-studies-unverified/*`, `docs/book/src/case_studies/index.md`, `docs/master-index.md`, `SKILLS-MANIFEST.md`
- **Acción:** Read 3 files archive (README 800L + 2 md 5942/6398L) + book stub 3L + master-index §Case Studies + web CASE_STUDIES 3 items + DISASTER_RECOVERY_RUNBOOK.md (no tocar — GOV-B2) + grep SKILLS-MANIFEST 4 keywords (0 hits cada una, documentado) + git log GOV-B1 2 commits
- **Verify:** Select-String README "ilustrativos" >=1 ; Test-Path rag_edge_device True ; grep manifest 0 hits documentado
- **Estado:** ✅ COMPLETED

### Step 2: EJECUCIÓN — crear task file GOV-B1.md + ponytail minimal docs
- **Archivos:** `.opencode/skills/campaign-executor/tasks/GOV-B1.md` (nuevo, este file)
- **Acción:** crear GOV-B1.md con Metadata/Blast Radius/Impacto/Contrato/Spec/Invariantes/Recitation/Deuda/DoD/Steps (ponytail: reuse commits a8a21733+98612db8, 0 líneas docs nuevas, solo task file + verify). No mover archivos (ya archivados), no editar Rust, no tocar DISASTER_RECOVERY_RUNBOOK.md
- **Verify:** Test-Path GOV-B1.md True ; Select-String GOV-B1.md "GOV-B1" >=1
- **Estado:** ✅ COMPLETED

### Step 3: VERIFY — Select-String docs/case_studies + cargo check (docs-only no Rust)
- **Archivos:** `docs/archive/case-studies-unverified/*`, `docs/book/src/case_studies/index.md`, `docs/case_studies` (negativo)
- **Acción:** ejecutar contrato mecánico: Test-Path rag_edge True + README "ilustrativos" >=1 + Test-Path docs/case_studies False + cargo check -p vantadb Finished
- **Verify:** contrato extendido 8 checks arriba → todos ✅
- **Estado:** ✅ COMPLETED

### Step 4: CIERRE — plan GOV-B1 → ✅ + recitation + git commit en develop (atomico docs(gov): GOV-B1)
- **Archivos:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §GOV-B1 (Estado PENDING → COMPLETED si aplica, ya ✅ en este plan pero se sincroniza last-synced + recitation)
- **Acción:** actualizar plan last-synced 2026-09-02T23:55 + recitation (este task) + git add GOV-B1.md (+ plan si editado) + git commit atomico `docs(gov): GOV-B1 case_studies → archive interno — verify guard + task file (Wave2 docs-only)` en develop
- **Verify:** git log --oneline -1 muestra docs(gov): GOV-B1 ; plan Estado ✅
- **Estado:** ⬜ PENDING (lo ejecuta el agente tras verify)

## Dependencias
- GOV-A5 ✅ (registros live) — Wave1c predecessor, no bloquea GOV-B1 (disjoint docs/reports vs docs/case_studies)
- RES-02..05 ✅ (Wave1 durabilidad) — parallel disjoint, no bloquea
- MEM-01 paralelo (engine) — disjoint 100% (src/planner.rs vs docs/case_studies), MAX 3 ok
- GOV-B2 paralelo (runbook) — disjoint (docs/archive vs docs/operations/DISASTER_RECOVERY_RUNBOOK.md), MAX 3 ok

## Notas
- Disjoint con MEM-01 (engine): MAX 3 respetado — GOV-B1 docs-only no contiende con MEM-01 F1 search profile (src/planner.rs) ni con GOV-B2 (runbook). Por eso puede ir Wave2 paralelo.
- Commit previo 98612db8 ya hizo sweep refs públicas (6 archivos) — no re-hacer sweep; solo verify guard.
- `ponytail: reuse 2 commits archivados, 0 líneas docs nuevas, task file solo si falta` — upgrade path: si Show HN exige case study real, vía CLD-04 enterprise pilot, no re-inflar ficticios.
