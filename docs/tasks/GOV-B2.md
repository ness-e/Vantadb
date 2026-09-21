# GOV-B2: Runbook DR sin comandos fantasma (DISASTER_RECOVERY_RUNBOOK.md)

## Metadata
- **Plan file:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md`
- **Wave:** Wave2 (GOV-B Show HN bloqueante, MAX 3 paralelo con MEM-01 y GOV-B1)
- **Creado:** 2026-09-02T23:55
- **last-synced:** 2026-09-02T23:55
- **Estado:** ✅ COMPLETED
- **Esfuerzo:** 🟢 ≤1h (docs-only, ponytail)
- **Tipo:** docs / operations / SHIP
- **Prioridad:** Alta (🔴 Addendum 3.4.2 — falla cuando se necesita)
- **Campaign ID:** 20260902-alta-prioridad-paralelo
- **No tocar:** `src/wal.rs`, `src/storage/engine/*`, `src/vector/*` (RES-02 disjoint) — dominio docs/operations + verify

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §GOV-B2, `docs/Backlog.md` filas GOV, `docs/reports/dora.md` §Recovery |
| Callees | `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` (473L), `src/cli.rs:130-154` (Restore/Doctor real), `docs/reports/dora.md` (402L), `SKILLS-MANIFEST.md` (grep SDP) |
| Implicaciones | Docs-only. Riesgo: Select-String "restore --dry-run|doctor --fix" !=0 si notas contienen literal. Mitigación: rephrase notas sin literal ghost-flag. Disjoint 100% con MEM-01 (src/planner.rs) y RES-02 (src/wal.rs). |

## Impacto mapeado (Regla 0) — BLAST RADIUS DOCS
- **Archivos leídos (completos):** `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` (473L, last_reviewed 2026-08-22, Overview SEV-1..4, §2 Recovery, §3 Health Checks Daily Backup Verification con restore temp+doctor+conteo, §3.1 full procedure), `src/cli.rs` (436L, Restore {input,force,rebuild}, Doctor sin --fix, Backup {out}, Migrate dry_run separado), `docs/reports/dora.md` (402L), `SKILLS-MANIFEST.md` (601L)
- **Grep SKILLS-MANIFEST.md keywords "disaster|recovery|runbook|dora" (SDP obligatorio):**
  - `disaster` → 0 hits
  - `recovery` → 1 hit (`incl-cognitive-accessibility-error-prevention-recovery` — diseño de recuperación, KEEP — mapeo manual a documentation-and-adrs + systematic-debugging)
  - `runbook` → 0 hits
  - `dora` → 0 hits
  - `disaster|recovery|runbook` conjunto → 3 hits totales (debugging-and-error-recovery DEPRECATED stub + incl error-prevention-recovery)
  - **Conclusión SDP:** sin skill directa disaster/runbook/dora — fallback por dominio: runbook → `documentation-and-adrs` + `writing-guidelines`, recovery → `systematic-debugging` (verify ghost), dora/report → `unified-review` + `shipping-and-launch`
- **Contrato verificado pre-fix:** `Select-String DISASTER_RECOVERY_RUNBOOK.md "restore --dry-run|doctor --fix"` Count=2 (líneas 146 y 242 — ambas en notas "there is no ..." documentando ausencia, no invocación real; pero contrato exige 0 literal)
- **CLI real validado:** `src/cli.rs:140-154` Restore {input,force,rebuild} — no --dry-run; Doctor unit sin flags; --dry_run solo en Migrate Run (391) — runbook ya usa --input/--force/--rebuild correcto en líneas 65,292,418,462
- **§3.1 Daily Backup Verification:** existe (línea 278) — full procedure 5 pasos (backup --out → restore --input temp → doctor → count/get → cleanup) + light variant MANIFEST.json crc32c + nota PENDING dry-run verify — insumo GOV-A3 verificado 2026-08-22
- **Veredicto impacto:** bajo — docs-only, 2 líneas rephrase, 0 Rust, disjoint Wave2 preservado

## Contrato
`Select-String -Path "docs/operations/DISASTER_RECOVERY_RUNBOOK.md" -Pattern "restore --dry-run|doctor --fix" | Measure-Object Count` ==0
- **Verificación extendida (pipeline-full):** `Test-Path .opencode/skills/campaign-executor/tasks/GOV-B2.md` == true AND `Select-String DISASTER_RECOVERY_RUNBOOK.md "restore --dry-run|doctor --fix" Count==0` AND `Select-String DISASTER_RECOVERY_RUNBOOK.md "restore --input" Count>=1` AND `cargo check -p vantadb` Finished AND plan GOV-B2 → ✅
- **CLI real referencia:** src/cli.rs Restore --input/--force/--rebuild, Doctor sin flags

## Spec (doc-driven)
N/A — docs-only fix runbook SHIP. No símbolos públicos nuevos. Decisiones tomadas: ghost flags eliminados del literal, §3.1 preservado, fuente única CLI src/cli.rs.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** No tocar `src/wal.rs`, `src/storage/engine/*`, `src/vector/*` (RES-02 disjoint Wave1); no tocar `src/planner.rs` (MEM-01 Wave2 paralelo); no modificar `src/cli.rs` (CLI es fuente verdad, runbook se adapta); `cargo check` debe permanecer verde; §3.1 Daily Backup Verification no se borra ni se mueve
- **Comandos de verificación:** `Select-String -Path "docs/operations/DISASTER_RECOVERY_RUNBOOK.md" -Pattern "restore --dry-run|doctor --fix" | Measure-Object Count` ==0 ; `Select-String -Path "docs/operations/DISASTER_RECOVERY_RUNBOOK.md" -Pattern "restore --input" | Measure-Object Count` >=1 ; `cargo check -p vantadb`
- **Deuda pendiente:** ninguna — runbook sin ghost flags, §3.1 intacto, CLI real alineado

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | GOV-B2 — Runbook DR sin comandos fantasma (DISASTER_RECOVERY_RUNBOOK.md) |
| `lastAction` | DISCOVERY Read runbook 473L + cli.rs Restore/Doctor + dora.md + grep SKILLS-MANIFEST disaster/recovery/runbook/dora (3 hits) → EJECUCIÓN crear GOV-B2.md + fix runbook rephrase 2 líneas ghost literal → verify Select-String 0 + cargo check |
| `result` | OK ↔ ✅ COMPLETED |
| `nextAction` | Wave2 paralelo disjoint — MEM-01 search profile / GOV-B1 archive no bloqueados |
| `contract` | `## Contrato` + `## Invariantes` + evidencia: Select-String ghost 0 + restore --input >=1 + cargo check Finished |
| `nextTask` | GOV-B3 — Fix snippets + guard (Wave2 docs) |

## Deuda técnica (Regla 6 — MUST)
Sin deuda nueva (docs-only 2 líneas rephrase, 0 Rust). Saldo neto 0. Ponytail: rephrase notas sin literal ghost-flag vs. borrar notas completas — add full dry-run CLI flag when `Migrate dry_run` patrón se extienda a Restore (ver src/cli.rs:391).

## Definition of Done (contrato multi-nivel)
| Nivel | Gate | Aplica |
|-------|------|--------|
| Task | Contrato verifiable Select-String ghost 0 + restore --input >=1 | ✅ |
| Commit | Lead no — worker commit atómico propio en develop (pipeline-full GOV-B2) | `docs(gov): GOV-B2 runbook DR sin comandos fantasma` |
| Release | No aplica (docs operations, no crate bump) | justificado — runbook SHIP, no API |

## Herramientas necesarias
- PowerShell Select-String / Test-Path / Measure-Object (contrato)
- cargo check -p vantadb (verify Rust no roto)
- Read/Grep (runbook + cli.rs + dora.md + SKILLS-MANIFEST)

**Skills cargadas (SDP §2 — lifecycle SHIP, ≤8 justificadas):**
- campaign-executor (orquestación pipeline-full DISCOVERY→EJECUCIÓN→CIERRE)
- planning-and-task-breakdown (slicing Steps atómicos Wave2)
- writing-plans (plan docs-first SHIP)
- ponytail(full) (diff mínimo docs-only, 2 líneas rephrase)
- documentation-and-adrs (runbook DR, ADR lifecycle)
- systematic-debugging (verify ghost commands, root-cause CLI)
- writing-guidelines (review docs voz/tono operations)
- shipping-and-launch (runbook SHIP pre-launch gate)

> Base 4 + 4 extras descubiertas por grep SKILLS-MANIFEST.md keywords "disaster/recovery/runbook/dora" (recovery→systematic-debugging, runbook/disaster→documentation-and-adrs+writing-guidelines, dora→shipping-and-launch). `debugging-and-error-recovery` descartado (DEPRECATED stub), `incl-error-prevention-recovery` mapeado manual.

## Investigation Notes
- **Plan file GOV-B2:** docs/plans/2026-09-02-alta-prioridad-paralelo.md:418-425 — descripción reescribir runbook a CLI real (restore --input/--force/--rebuild, sin --dry-run/doctor --fix) + §3.1 Daily Backup Verification con restore temp+doctor+conteo insumo GOV-A3, contrato Select-String ghost 0, estado ✅ COMPLETED pero sin task file físico (gap que este pipeline cierra)
- **Runbook actual 473L:** Overview SEV-1..4, §2.1..2.6 recovery, §3 Health Checks Scheduled + Daily Backup Verification (278) con full procedure 5 pasos + light MANIFEST.json, §4 Backup Strategy, §5 Post-Incident, §6 Testing Schedule, §7 Escalation, §8 Appendices A/B/C. Ya usa restore --input/--force/--rebuild en 5 lugares (65,292,402,418,462) — ghost solo en 2 notas explicativas (146,242) que documentan ausencia con literal.
- **CLI real:** src/cli.rs Restore 140-151 (input,force,rebuild) — no --dry-run; Doctor 153-154 unit; --dry_run solo Migrate::Run 376-391. Validación: runbook comandos reales correctos, notas ghost son falso positivo contrato.
- **Grep manifest (SDP):** SKILLS-MANIFEST.md 601L — disaster 0, recovery 1 (incl-cognitive… KEEP), runbook 0, dora 0 → total 3 con DEPRECATED stub. Fallback dominio justifica 4 extras → total 8.
- **Disjoint Wave2:** GOV-B2 toca docs/operations/DISASTER_RECOVERY_RUNBOOK.md; MEM-01 toca src/planner.rs + sdk/serialization + sdk/search; RES-02 toca src/wal.rs + storage/engine — 0 archivos en común → parallel 3 seguro; instrucción Disjoint con MEM-01 respetada.

## Incógnitas (uphill) vs Pendientes (downhill)
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — no aplica: docs-only, sin trust boundaries, sin auth, sin input usuario
- [x] **PERFORMANCE** — no aplica: no hot path, no benchmark, cargo check solo

## Steps

### Step 1: DISCOVERY — Read runbook + cli.rs + dora + grep SKILLS-MANIFEST (SHIP lifecycle)
- **Archivos:** `docs/operations/DISASTER_RECOVERY_RUNBOOK.md`, `src/cli.rs:130-154`, `docs/reports/dora.md`, `SKILLS-MANIFEST.md`, `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §GOV-B2
- **Acción:** Leer runbook 473L completo + cli.rs Restore/Doctor + dora.md 402L + plan GOV-B2. Grep SKILLS-MANIFEST.md por keywords "disaster|recovery|runbook|dora" → discovery skills SHIP. Mapear blast radius callers/callees. Confirmar no overlap MEM-01 (planner) ni RES-02 (wal). Detectar ghost Count=2 falso positivo.
- **Verify:** `Test-Path docs/operations/DISASTER_RECOVERY_RUNBOOK.md` == true AND `Test-Path src/cli.rs` == true AND `Select-String SKILLS-MANIFEST.md "recovery" Count>=1`
- **Estado:** ✅ COMPLETED — 2026-09-02 discovery: runbook 473L + cli.rs 436L + dora.md + grep 3 hits → skills 8, disjoint confirmado, ghost 2 detectado

### Step 2: EJECUCIÓN — Fix runbook ghost literal (ponytail 2 líneas rephrase)
- **Archivos:** `docs/operations/DISASTER_RECOVERY_RUNBOOK.md:146,242`, `.opencode/skills/campaign-executor/tasks/GOV-B2.md` (este file)
- **Acción:** (ponytail: 2 líneas, no borrar notas, rephrase sin literal)
  1. Línea 146: `# Native WAL repair is NOT available yet (there is no `doctor --fix` flag).` → `# Native WAL repair is NOT available yet (there is no doctor fix flag).`
  2. Línea 242: `> **Note:** there is no `vanta-cli restore --dry-run`. To validate...` → `> **Note:** there is no vanta-cli restore dry-run mode. To validate...`
  3. Este task file GOV-B2.md ya creado (contrato secundario Test-Path)
  4. Verify: `Select-String DISASTER_RECOVERY_RUNBOOK.md "restore --dry-run|doctor --fix" Count==0` AND `Select-String DISASTER_RECOVERY_RUNBOOK.md "restore --input" Count>=1` AND `cargo check -p vantadb` Finished
- **Verify:** Select-String ghost 0 + restore --input >=1 + cargo check exit 0
- **Estado:** ✅ COMPLETED — 2026-09-02 fix: rephrase 146/242 sin literal ghost, Select-String ghost 0 ✅ + restore --input 5 >=1 ✅ + cargo check Finished 29s ✅

### Step 3: CIERRE — Plan sync PENDING→COMPLETED + recitation + git commit
- **Archivos:** `.opencode/skills/campaign-executor/tasks/GOV-B2.md` (este file), `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §GOV-B2
- **Acción:** Actualizar task file Steps → ✅, Estado IN PROGRESS→COMPLETED, plan fila GOV-B2 Estado →✅ COMPLETED con recitation (activeGoal/contract/lastAction/nextAction/nextTask). Git commit atómico `docs(gov): GOV-B2 runbook DR sin comandos fantasma` en develop. No tocar MEM-01.
- **Verify:** `Test-Path GOV-B2.md` == true AND `Select-String plan GOV-B2.*COMPLETED Count>=1` AND `git log --oneline -1 | Select-String GOV-B2`
- **Estado:** ✅ COMPLETED — task file Steps 3/3 ✅ + plan sync + commit atómico (este commit)

## Dependencias
- GOV-A3 ✅ (probes CLI reales doctor/backup/restore — insumo §3.1)
- GOV-B1 ✅ (case_studies archive — paralelo disjoint docs/case_studies)
- MEM-01 ✅ (Wave2 paralelo — disjoint src/planner.rs, no bloqueante)
- RES-02 ✅ (WAL v2 landed — no tocar wal/engine, disjoint)

## Review (GATE — docs-only ponytail self-review)
- **Revisor:** vanta-docs (self-review ponytail, docs-only, sin Rust) — contratos mecánicos Select-String ghost 0 + restore --input + cargo check, §3.1 intacto, CLI real alineado. Veredicto: ✅ approve (tras Step2 verify)

## Notas
- Worker commit atómico propio en develop (pipeline-full GOV-B2) — docs(gov): GOV-B2 ... con 2 files (runbook + task file) + plan sync
- Verify cargo check -p vantadb (no --workspace, ponytail quick) — disjoint garantizado
- Disjoint MEM-01 respetado: 0 archivos compartidos, Wave2 parallel 3 seguro

## Referencias
- `docs/plans/2026-09-02-alta-prioridad-paralelo.md` — Wave2 GOV-B2 fila
- `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` — 473L runbook DR
- `src/cli.rs:130-154` — fuente verdad CLI Restore/Doctor
- `docs/reports/dora.md` — DORA flow 402L §Recovery
- `SKILLS-MANIFEST.md` — 601L grep SDP
- `.opencode/references/skills-engineering.md` — SDP canónico
- `docs/operations/BACKUP_POLICY.md` — referenciado por runbook §4

## Context Save Point
- **Fecha:** 2026-09-02T23:55
- **Branch:** develop
- **CI pendiente:** no (docs-only, cargo check local)
- **Decisiones:** Ponytail 2 líneas rephrase ghost literal sin borrar notas; reuse runbook existente 473L + §3.1; no tocar cli.rs
- **Problemas conocidos:** ninguno — ghost 2 falso positivo identificado, fix rephrase, disjoint MEM-01/RES-02 preservado
- **Próxima tarea:** GOV-B3 — Fix snippets + guard (Wave2 docs)
