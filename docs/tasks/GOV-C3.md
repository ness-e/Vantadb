# TASK GOV-C3 — Verify Daily Backup Verification (§3.1 + verify.ps1 daily guard)

## Metadata
- **Plan file:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md` Wave2
- **Wave:** Wave2 14/15 → 15/15 (MEM-01..06 6 + GOV-B1..B6 6 + GOV-C1/C2 2 + GOV-C3 1)
- **Creado:** 2026-09-02T00:00
- **last-synced:** 2026-09-02
- **Estado:** ✅ COMPLETED (verify daily backup — ponytail docs-only)
- **Ruta:** vanta-docs (leaf, docs-only)

## Blast Radius
- `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` §3.1 Daily Backup Verification — ya existe (4 hits), 5 pasos full restore+doctor+count verificados GOV-B2/GOV-A3, no se toca contenido durabilidad
- `dev-tools/verify.ps1` — se AÑADE guard `daily backup verification` (ponytail 2 líneas, valida anchor existe, no heavy restore en fast gate)
- `docs/reports/dora.md` — read-only referencia, no se edita (fuente métricas)
- **No toca:** `.config/nextest.toml` (GOV-C1 disjoint), `docs/master-index.md` (GOV-C4 disjoint), `docs/Backlog.md` (GOV-C2/C3 purga refs ya resuelta como mención textual, no se re-purga aquí), `src/wal.rs`, `src/storage/engine/*`, `src/planner.rs`, `vantadb-mcp/*`

## Contrato (verificable)
```powershell
Select-String -Path "docs/operations/DISASTER_RECOVERY_RUNBOOK.md" -Pattern "Daily Backup Verification" | Measure-Object Count # >=1
Select-String -Path "dev-tools/verify.ps1" -Pattern "Daily Backup Verification" | Measure-Object Count # >=1
cargo check -p vantadb # exit 0
```

## Steps

### Step 1: DISCOVERY — Read runbook + verify
- **Archivos leídos:**
  - `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` 473L §3.1 Daily Backup Verification intacto (4 hits: line 276,278,238,420 + §3.1 30L 5 pasos backup→restore temp→doctor→count/get→cleanup, light variant MANIFEST.json, PENDING note until CLI verify subcommand)
  - `dev-tools/verify.ps1` 99L (fmt, check, clippy, audit, deny, nextest, coverage, docs-coverage, cli-probes, consumo guard) — gap: 0 hits "Daily Backup Verification"
  - `docs/reports/dora.md` 402L (DORA metrics, recovery time, throughput — sin backup verification, no debe tocarse)
  - `SKILLS-MANIFEST.md` grep keywords verify:3 backup:0 daily:0 dora:0 (source-driven-development verify 3 hits)
  - Plan `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §GOV-C3 571-578 (purga refs 10 audit-reports + REPORTE_EVALUACION×2 — ya resuelta como mención textual `Nota GOV-C3 2026-08-22` docs/Backlog.md:200, no re-purga en este task docs(operations) disjoint)
- **Verificación gap:** runbook §3.1 existe ✅, verify.ps1 no tenía guard daily backup ❌ → fix ponytail requerido
- **Verify:** `Select-String runbook 4 >=1` ✅, `Select-String verify.ps1 0` → gap confirmado
- **Estado:** ✅ DONE

### Step 2: EJECUCIÓN — fix verify daily backup ponytail
- **Archivos:** `dev-tools/verify.ps1`
- **Acción:** Añadir guard `daily backup verification` ponytail minimal 2 líneas (reuse pattern `consumo guard`/`cli-probes`): `run "daily backup verification" ("pwsh", "-NoProfile", "-Command", "if ((Select-String -Path 'docs/operations/DISASTER_RECOVERY_RUNBOOK.md' -Pattern 'Daily Backup Verification' | Measure-Object).Count -ge 1) { exit 0 } else { exit 1 }")` + coment `ponytail: validates runbook §3.1 exists (docs-only), no heavy restore in fast gate — Full restore+doctor lives in runbook §3.1, fast gate only checks anchor`
- **Ponytail rationale:** fast gate <5min determinístico offline (CI_POLICY) no puede hacer backup+restore real (requiere FS temporal + `vanta-cli` build); el guard docs-only prueba que procedimiento existe y es discoverable — restore real ya validado GOV-A3 transcription + RES-02 quiesce+flush; upgrade a `cargo run --bin vanta-cli backup/restore temp` si CI heavy gate lo exige
- **Verify:** `Select-String verify.ps1 2 >=1` ✅ + `cargo check -p vantadb` Finished ✅
- **Estado:** ✅ DONE

### Step 3: VERIFY — Select-String + cargo check
- **Comandos:**
  - `Select-String -Path "docs/operations/DISASTER_RECOVERY_RUNBOOK.md" -Pattern "Daily Backup Verification" | Measure-Object Count` → 4 ✅
  - `Select-String -Path "dev-tools/verify.ps1" -Pattern "Daily Backup Verification" | Measure-Object Count` → 2 ✅
  - `cargo check -p vantadb` → Finished ✅
- **Estado:** ✅ DONE

### Step 4: CIERRE — plan sync + commit atómico
- **Acción:** marcar plan GOV-C3 ✅ + recitation + `git commit` atómico `docs(gov): GOV-C3 verify Daily Backup Verification (§3.1 + verify.ps1 guard)` en develop
- **Estado:** ✅ DONE

## Dependencias
- GOV-B2 (runbook sin comandos fantasma) ✅ + GOV-A3 (transcripción backup→restore→doctor) ✅ — prerequisite §3.1 ya verificado 2026-08-22
- RES-02 (quiesce+flush) ✅ — garantiza backup quiesce correctness
- Disjoint GOV-C1 (nextest) / GOV-C2 (Backlog) — preservado 0 archivos en común
- MAX 3 paralelo Wave2 batch preserved

## Notas
- Purga refs muertas audit-reports ya cerrada como mención textual `Nota GOV-C3 2026-08-22` docs/Backlog.md:200 (no `audit-reports/` link roto), método AUD-007 `Select-String audit-reports 2 hits históricos` → 2 hits residuales son históricos intencionales con disclaimer, no links rotos; no re-purga en este task (scope docs/operations only)
- Full backup verification procedure permanece en runbook §3.1 (5 pasos), light variant MANIFEST.json, no `--dry-run` fantasma (GOV-B2 fixed)
- Ponytail ceiling: `ponytail: fast gate docs-only, full restore+doctor if heavy gate` — documentado en verify.ps1

## Context Save Point
- **Fecha:** 2026-09-02
- **Branch:** develop
- **CI pendiente:** no — cargo check + Select-String guards verde
- **Próxima tarea:** Wave3 MEM-07..21 + GOV-C4..C7 + RES-06..07 (MAX 3)
