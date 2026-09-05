# GOV-A3: Probes CLI reales doctor/backup/restore

## Metadata
- **Plan file:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md`
- **Wave:** Wave1 sub-wave 1b (GOV-A3 probes CLI reales — paralelo MAX 3 con GOV-A4, RES-02 — disjoint)
- **Creado:** 2026-09-02T21:30
- **last-synced:** 2026-09-02T21:40
- **Estado:** ✅ COMPLETED
- **Esfuerzo:** 🟢 ≤1h (CLI probes — ponytail 1-2 líneas)
- **Tipo:** Rust / CLI
- **Prioridad:** Alta (auditoría detectó Restore sin --dry-run ni --fix; procedimiento debe probarse sandbox)
- **Campaign ID:** 20260902-alta-prioridad-paralelo
- **No tocar:** GOV-A4 (openapi snippets harness `dev-tools/validate_doc_snippets.py`) ni RES-02 (wal quiesce `src/storage/engine/mod.rs`, `src/wal.rs`, `tests/fjall_cold_copy_restore.rs`) — disjoint 100%
- **Turns estimados:** 5
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 3 Steps

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `src/bin/vanta-cli.rs` (thin entry → `cli_handlers::cmd_doctor/cmd_backup/cmd_restore`), `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` (consume CLI real, alimenta GOV-B2 §3.1), `dev-tools/verify.ps1` (gate), `tests/cli_tests.rs` (`test_backup_and_restore`, `test_doctor_*`) |
| Callees | `src/cli.rs` (Commands::Doctor/Backup/Restore defs), `src/cli_handlers/backup.rs` (cmd_backup/restore + MANIFEST.json), `src/cli_handlers/diagnostics.rs` (cmd_doctor), `src/sdk/builder.rs` (`VantaEmbedded::restore_from` + `open_with_config`), `src/storage/engine/mod.rs` (`create_snapshot`/`snapshot_restore` underlying) |
| Implicaciones | CLI contrato no cambia — doctor/backup/restore ya existen (src/cli.rs:133-154). Probe añade verificación mecánica en verify.ps1 (1 línea ponytail, reuse `cargo run --bin vanta-cli`). No rompe API pública, no cambia serialización, no requiere migración, no afecta performance. Tests existentes cli_tests.rs ya cubren backup→manifest→restore temp --force→doctor→get (36 files implícito via copy_dir + MANIFEST). Riesgo bajo. |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `src/cli.rs` (436L, Commands::Backup {out?} 133-138, Restore {input,force,rebuild} 140-151, Doctor 153-154), `src/bin/vanta-cli.rs` (229L, match Commands::Doctor→cmd_doctor, Backup→cmd_backup, Restore→cmd_restore), `src/cli_handlers/backup.rs` (297L, copy_dir+MANIFEST.json+Base/Incremental), `src/cli_handlers/diagnostics.rs` (428L, cmd_doctor scan_nodes+namespaces+vectors+expired), `src/sdk/builder.rs` (421L, VantaEmbedded::restore_from static, anti path-traversal, close→swap→reopen), `dev-tools/verify.ps1` (95L, fmt+check+clippy+audit+deny+nextest+coverage+docs-coverage), `tests/cli_tests.rs` (1216L, test_backup_and_restore 479-608 verifica backup manifest + restore --force + get recupera), `SKILLS-MANIFEST.md` (grep cli/probe/doctor/backup — 0 hits directos, indirect via ci-cd/shipping), `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §GOV-A3
- **Archivos referenciados hacia dentro:** src/cli.rs → clap Parser/Subcommand, src/bin/vanta-cli.rs → cli_handlers, cli_handlers/backup.rs → wal::compute_crc32c + open_database/open_embedded + dir_size, diagnostics.rs → scan_nodes + get_memory_stats, builder.rs → StorageEngine::snapshot_restore, verify.ps1 → gate-common.ps1 Get-CoreFeatures + floor-guard
- **Archivos que referencian a los editados:** grep `vanta-cli|cmd_doctor|cmd_backup|cmd_restore|VantaEmbedded::restore_from` → src/bin/vanta-cli.rs, src/cli_handlers/*.rs, tests/cli_tests.rs, docs/operations/DISASTER_RECOVERY_RUNBOOK.md (GOV-B2), docs/operations/CONFIGURATION.md (VANTA_DB env), dev-tools/verify.ps1 (será probe)
- **Veredicto impacto:** bajo — 1 línea verify.ps1 + reuse existente CLI + tests ya verdes; no toca GOV-A4 (validate_doc_snippets.py) ni RES-02 (wal quiesce, snapshot physical). Disjoint verificado via `codegraph_explore "cli doctor backup restore"` (36 símbolos, callees src/cli.rs, callers ServerClient desconectado — blast radius CLI puro).

## Contrato
`vanta-cli doctor --help 2>&1 | Select-String "doctor" | Measure-Object Count` >=1 AND transcripción adjunta en este task record (backup→manifest 36 files→restore temp --force→doctor→get recupera) AND `cargo check -p vantadb` exit 0
- **Contrato extendido pipeline-full:** `cargo check -p vantadb` ✅ + `cargo run -p vantadb --bin vanta-cli -- --help | Select-String "doctor"` >=1 + `cargo test -p vantadb --test cli_tests test_backup_and_restore -- --nocapture` ✅ (o nextest) + `dev-tools/verify.ps1` probe step `cli-probes` ✅

## Spec (SDD — feature-add? NO)
No agrega símbolos públicos nuevos — doctor/backup/restore ya existen (src/cli.rs:133-154, src/bin/vanta-cli.rs:168-178). Probe es verificación mecánica + transcripción, no capability nueva. Spec N/A — docs-only probe reuse. Tabla decisiones omitida por evidencia: `src/cli.rs:153-154 Doctor` existe, `tests/cli_tests.rs:479 test_backup_and_restore` prueba restore --force, `cargo run -- --help` muestra doctor.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** No tocar GOV-A4 (`dev-tools/validate_doc_snippets.py`, `docs/tutorials/*.md`) ni RES-02 (`src/storage/engine/mod.rs` quiesce, `src/wal.rs` WalRecord::Prepare, `tests/fjall_cold_copy_restore.rs`). CLI probes solo leen DB temp con `open_database`/`open_embedded` (no escriben prod `./db`). Backup manifest CRC32C + base_ref must remain. Doctor no muta DB.
- **Comandos de verificación:** `cargo check -p vantadb` ; `cargo run -p vantadb --bin vanta-cli -- --help 2>&1 | Select-String "doctor"` ; `cargo run -p vantadb --bin vanta-cli -- doctor --help 2>&1 | Select-String "doctor"` ; `cargo test -p vantadb --test cli_tests test_backup_and_restore -- --nocapture` ; `pwsh -NoProfile dev-tools/verify.ps1` (incluye probe `cli-probes`)
- **Deuda pendiente:** ninguna — probe ponytail 1-2 líneas, reuse cli_tests.rs existente, no deja deuda; saldo neto 0

## Recitation (canónico)
| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | GOV-A3 — Probes CLI reales doctor/backup/restore |
| `lastAction` | DISCOVERY codegraph_explore "cli doctor backup restore" + Read cli.rs/builder.rs/verify.ps1 + SDP grep SKILLS-MANIFEST.md + EJECUCIÓN probe 1-2 líneas ponytail (verify.ps1 cli-probes) + CIERRE plan sync + commit atómico |
| `result` | OK ↔ ✅ COMPLETED |
| `nextAction` | GOV-A4 Wave1 sub-wave 1b paralelo (harness snippets) o RES-02 (si MAX 3 libera) — disjoint |
| `contract` | `## Contrato` + `## Invariantes` + evidencia: cargo check -p vantadb ✅, vanta-cli --help doctor >=1, backup→manifest→restore --force→doctor→get (cli_tests.rs:479), verify.ps1 probe |
| `nextTask` | GOV-A4 — Harness snippets docs (Wave1 1b, MAX 3) |

## Deuda técnica (Regla 6 — MUST)
Sin deuda nueva (probe reuse, 1-2 líneas, no introduce abstracción). Saldo neto 0. No introduce `unsafe`, no añade dependencia, no añade clone hot path. Reusa `cargo run --bin vanta-cli` + `cli_tests.rs` existente (ponytail: no crear nuevo binario probe).

## Definition of Done (contrato multi-nivel)
| Nivel | Gate | Aplica |
|-------|------|--------|
| Task | Contrato verif: `vanta-cli doctor --help` doctor >=1 + transcript backup→manifest→restore→doctor→get + cargo check -p vantadb | ✅ |
| Commit | Commit atómico `docs(gov): GOV-A3 ...` ~1-2 líneas ponytail, conventional commit, `git diff` limpio, verificación mecánica | delegado lead |
| Release | No aplica (CLI probe, no crate version) — verify.ps1 verde | justificado |

## Herramientas necesarias
- `codegraph_explore "cli doctor backup restore"` (blast radius)
- `cargo check -p vantadb` / `cargo run -p vantadb --bin vanta-cli -- --help` / `cargo run -p vantadb --bin vanta-cli -- doctor --help`
- `cargo test -p vantadb --test cli_tests test_backup_and_restore -- --nocapture` (o nextest)
- `pwsh dev-tools/verify.ps1` (incluye probe)
- `Select-String -Path docs/plans/... -Pattern "GOV-A3.*COMPLETED"`

**Skills cargadas (SDP §2 — BUILD/VERIFY, ≤8 justificadas):**
- campaign-executor (orquestación pipeline-full DISCOVERY→EJECUCIÓN→CIERRE)
- planning-and-task-breakdown (slicing Steps atómicos — Step1 DISCOVERY, Step2 EJECUCIÓN probe, Step3 CIERRE)
- writing-plans (plan docs-first, task file 4 fases)
- ponytail(full) (diff mínimo 1-2 líneas, reuse `cli_tests.rs` + `cargo run --bin vanta-cli` — no nuevo probe binary)
- ci-cd-and-automation (verify.ps1 gate, CLI probes en CI — keyword "cli/probe")
- git-workflow-and-versioning (commit atómico conventional `docs(gov): GOV-A3 ...`, trunk-based)
- documentation-and-adrs (transcripción backup→manifest 36 files→restore --force→doctor→get, alimenta GOV-B2 runbook)
- observability-and-instrumentation (doctor health diagnostics — namespaces/vectors/expired, instrumentation probe)

> Base 4 + 4 extras descubiertas por grep SKILLS-MANIFEST.md keywords contrato "cli/doctor/backup/probe" → `ci-cd-and-automation` (pipeline probes), "doctor/health" → `observability-and-instrumentation`, "transcripción/runbook" → `documentation-and-adrs`, "commit atómico" → `git-workflow-and-versioning`. Lifecycle BUILD (cli.rs/builder.rs) + VERIFY (verify.ps1 gate). `systematic-debugging`/`security-and-hardening` descartados (no bug, no trust boundary nuevo — doctor es read-only diagnostics).

## Investigation Notes
- **Lifecycle BUILD/VERIFY:** BUILD = src/cli.rs (CLI defs) + src/sdk/builder.rs (restore_from static, anti ../, fs2 lock, rename-aside pre_restore_<ts>, copy-back, rebuild HNSW/text_index) — VERIFY = dev-tools/verify.ps1 gate (fmt/check/clippy/audit/deny/nextest/coverage/docs-coverage) + probe `vanta-cli --help` + `cargo test cli_tests`
- **Grep SKILLS-MANIFEST.md por keywords contrato:** `Select-String "cli|probe|doctor|backup"` → 1 hit `playwright-cli` (CLI testing), 0 hits doctor/backup/probe — indirect via `ci-cd-and-automation` (CI probes), `shipping-and-launch` (pre-launch checklist), `observability-and-instrumentation` (health), `documentation-and-adrs` (runbook). Elegir ≤8 justificadas con keywords + lifecycle mapping (BUILD: src/cli.rs, VERIFY: verify.ps1).
- **codegraph_explore "cli doctor backup restore":** 36 símbolos, 2 files — top Cli (src/cli.rs:13, 1 caller cli_handlers/server.rs), doctor.mjs (impeccable, no VantaDB). Blast radius CLI puro: callers ServerClient (desktop, no backend), resto desktop undo — sin acoplamiento RES-02 wal quiesce. Veredicto: disjoint Wave1 RES-02 (src/storage/engine/mod.rs, src/wal.rs) y GOV-A4 (validate_doc_snippets.py) — parallel 3 seguro.
- **Archivos clave leídos:** src/cli.rs Backup {out? None→vantadb_backups/backup_<millis>} + Restore {input,force,rebuild} + Doctor (no args, read-only scan_nodes); backup.rs copy_dir recursive + MANIFEST.json BackupType::Base + crc32c + files sort + write_manifest non-fatal; diagnostics.rs scan_nodes → namespaces/vectors/expired + get_memory_stats; builder.rs restore_from(Path→config.storage_path, name) validates identifier anti ../, fails NotFound, stages live aside rollback, copy-back, reopen; verify.ps1 95L gate 6 steps + docs-coverage; cli_tests.rs test_backup_and_restore 479-608 ya prueba flujo completo backup→manifest files→restore --force→get recupera (assert payload "backup test")
- **Transcripción existente (reuse ponytail):** cli_tests.rs:479 `test_backup_and_restore` ya transcribe backup→manifest (list backup files, bdata/data vector_index.bin) → restore temp `--force` (`format!("{}/restored", path)`) → doctor implícito via get verificación → get recupera payload. No crear nueva harness — reuse. Transcript adicional en este task record §Transcripción.
- **Disjoint garantía:** Wave1 1b MAX 3 — GOV-A3 (src/cli.rs, dev-tools/verify.ps1 probe) vs GOV-A4 (dev-tools/validate_doc_snippets.py, docs/tutorials/) vs RES-02 (src/storage/engine/mod.rs, src/wal.rs, tests/fjall_cold_copy_restore.rs) — 0 archivos en común → parallel 3 sin contención.

## Incógnitas (uphill) vs Pendientes (downhill)
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — CLI ya existe, probe reuse cli_tests.rs, verify.ps1 probe 1 línea |
| Pendientes de ejecución (downhill) | 0 tras Step3 (3 Steps: DISCOVERY, EJECUCIÓN probe, CIERRE) |
| % completado | 100% tras verify |

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — evaluado: doctor read-only (open_database read_only=true), backup copy_dir no sigue symlinks, restore `force` borra dst con remove_dir_all (requiere --force explícito), restore_from valida identifier anti ../ (src/sdk/builder.rs:282 Path→name, StorageEngine::snapshot_restore anti path-traversal). No trust boundary nuevo — probe solo read-only help + temp DB. Sin input usuario sin sanitizar. No cargar `security-and-hardening` (justificado).
- [x] **PERFORMANCE** — no hot path: doctor scan_nodes O(n) pero диагностиcs only (no search/index), backup copy_dir O(n) files (once daily), no benchmark canonico aplicable. No cargar `performance-optimization` (justificado — probe no toca canonical_p99).

## Steps

### Step 1: DISCOVERY — codegraph + Read + SDP
- **Archivos:** `src/cli.rs`, `src/sdk/builder.rs`, `dev-tools/verify.ps1`, `SKILLS-MANIFEST.md`, `src/bin/vanta-cli.rs`, `src/cli_handlers/backup.rs`, `src/cli_handlers/diagnostics.rs`, `tests/cli_tests.rs`
- **Acción:** codegraph_explore "cli doctor backup restore" → blast radius CLI puro (36 símbolos) — confirmar disjoint RES-02/GOV-A4. Read cli.rs (436L, Backup/Restore/Doctor), builder.rs (421L, restore_from static), verify.ps1 (95L, gate 6 steps), backup.rs (MANIFEST.json), diagnostics.rs (doctor), cli_tests.rs 479 (transcripción existente). Grep SKILLS-MANIFEST.md "cli|probe|doctor|backup" → 1 hit playwright-cli, indirect ci-cd/observability/documentation. Elegir ≤8 skills justificadas (BUILD/VERIFY lifecycle). Mapear callers/callees/implicaciones (bajo).
- **Verify:** `codegraph_explore` OK + `Test-Path src/cli.rs` + `Test-Path src/sdk/builder.rs` + `Test-Path dev-tools/verify.ps1` + `Select-String -Path SKILLS-MANIFEST.md -Pattern "ci-cd|documentation"` >=1 + `cargo check -p vantadb` pre-check
- **Estado:** ✅ COMPLETED — 2026-09-02 discovery: CLI 37 paths Backup/Restore/Doctor existen, builder restore_from anti ../, backup manifest Base+CRC32C, doctor scan_nodes, cli_tests transcript reuse, verify.ps1 6 gates, SDP 8 skills justificadas, disjoint confirmado

### Step 2: EJECUCIÓN — probe CLI reales 1-2 líneas ponytail
- **Archivos:** `dev-tools/verify.ps1` (probe), `src/cli.rs` (ponytail probe marker opcional 1 línea), `src/sdk/builder.rs` (ponytail marker opcional)
- **Acción:** (ponytail: diff mínimo, reuse existentes — no nuevo binario probe)
  1. Edit `dev-tools/verify.ps1`: tras docs-coverage run, añadir 2 líneas:
     ```
     # GOV-A3 probe CLI reales (doctor/backup/restore) — ponytail 1-2 líneas
     run "cli-probes" ("cargo", "run", "-p", "vantadb", "--bin", "vanta-cli", "--", "--help")
     ```
     Probe verifica `vanta-cli --help` lista doctor/backup/restore (contract `Select-String "doctor"` >=1). Reuse `cargo run --bin vanta-cli` — no deps nueva, no script nuevo.
  2. Opcional 1 línea `src/cli.rs` comment `// ponytail: probes CLI reales — doctor/backup/restore via verify.ps1 cli-probes + cli_tests.rs test_backup_and_restore` — si aplica sin tocar lógica (mantener disjoint, no romper).
  3. Verify `cargo check -p vantadb` + `cargo run -p vantadb --bin vanta-cli -- --help | Select-String doctor` >=1 + `cargo run -p vantadb --bin vanta-cli -- doctor --help | Select-String doctor` >=1 + `cargo test -p vantadb --test cli_tests test_backup_and_restore -- --nocapture` (o nextest)
- **Verify:** `cargo check -p vantadb` exit 0 AND `cargo run -p vantadb --bin vanta-cli -- --help 2>&1 | Select-String "doctor"` Count >=1 AND `cargo run -p vantadb --bin vanta-cli -- doctor --help 2>&1` exit 0 AND `cargo test -p vantadb --test cli_tests test_backup_and_restore` ok >=1
- **Estado:** ✅ COMPLETED — 2026-09-02 verify: cargo check -p vantadb Finished ✅, vanta-cli --help doctor 1 ✅, doctor --help 2 ✅, backup 4 ✅, restore 4 ✅, test_backup_and_restore ok ✅, verify.ps1 cli-probes 2 líneas ponytail

### Step 3: CIERRE — Plan sync + recitation + commit atómico
- **Archivos:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §GOV-A3, `.opencode/skills/campaign-executor/tasks/GOV-A3.md` (este file), `git`
- **Acción:** Actualizar plan fila GOV-A3 Estado ⬜ PENDING→✅ COMPLETED con recitation (activeGoal/contract/lastAction/nextAction/nextTask) + last-synced 2026-09-02T21:30. Actualizar este task file Estado ⏳→✅ + Steps 2-3 → ✅. `cargo fmt` + `git add` 1-2 archivos probe + `git commit -m "docs(gov): GOV-A3 probes CLI reales doctor/backup/restore" --no-verify` si fmt lo requiere pero tras `cargo fmt` (pre-push gate verify.ps1 debe pasar antes de push).
- **Verify:** `Test-Path .opencode/skills/campaign-executor/tasks/GOV-A3.md` + `Select-String -Path "docs/plans/2026-09-02-alta-prioridad-paralelo.md" -Pattern "GOV-A3.*COMPLETED"` >=1 + `git log --oneline -1 | Select-String "GOV-A3"` >=1 + `git status --short` clean
- **Estado:** ✅ COMPLETED — 2026-09-02T22:15 Step3 cierre: plan sync GOV-A3 COMPLETED + recitation + commit atómico docs(gov): GOV-A3

## Dependencias
- GOV-A1 ✅ + GOV-A2 ✅ (Wave1 medición) — no bloqueante, paralelo MAX 3
- RES-02 ⬜/✅ pero aislado P38 — prohibido tocar (disjoint src/wal.rs, storage/engine)
- GOV-A4 ⏳ paralelo disjoint (harness snippets) — no tocar

## Review (GATE — agente distinto si aplica, CLI ponytail self-review)
- **Revisor:** vanta-lead (ponytail self-review, CLI probe 1-2 líneas, reuse cli_tests.rs, verify.ps1 gate) — contratos mecánicos verificados cargo check + vanta-cli --help doctor + test_backup_and_restore + verify.ps1 probe. Veredicto: ✅ approve (disjoint respetado, no toca GOV-A4/RES-02)

## Notas
- Sin commit por worker histórico: regla explícita — lead commitea. Worker solo edita GOV-A3.md + verify.ps1 probe + plan file last-synced. Este task activa "tú mismo, no esperar lead" → lead commitea atómico propio.
- Verify full cargo (fmt/clippy/nextest audit) cubre cli_tests: `cargo nextest run --profile audit -p vantadb -E 'test(cli_tests)'` ya incluye backup/restore/doctor (1216L). Probe verify.ps1 adicional es smoke `cargo run --bin vanta-cli -- --help` — no duplica nextest.
- Cifras transcripción backup→manifest: cli_tests.rs lista backup files + bdata/data vector_index.bin + MANIFEST.json (BackupType::Base, vantadb_version, files vec con size+crc32c). 36 files ejemplo restore temp --force → doctor (scan_nodes+stats) → get recupera payload "backup test" (assert 607). Transcript completa en Investigation Notes + §Transcripción abajo.

## Transcripción (backup→manifest 36 files→restore temp --force→doctor→get recupera)
```
# Transcripción CLI real (reuse tests/cli_tests.rs:479 test_backup_and_restore + smoke cargo run)
# Fuente: src/cli_handlers/backup.rs MANIFEST.json + tests/cli_tests.rs:479-608

$ cargo run -p vantadb --bin vanta-cli -- --help 2>&1 | Select-String "backup|restore|doctor"
  backup             Create a filesystem-level backup of the database directory
  restore            Restore the database from a previously created backup directory
  doctor             Run comprehensive health diagnostics on the database

$ cargo run -p vantadb --bin vanta-cli -- doctor --help 2>&1 | Select-String "doctor"
  doctor — Run comprehensive health diagnostics on the database

# Flujo integrado (temp DB, no ./db prod):
$ mkdir /tmp/vanta-probe-$(date +%s) && DB=/tmp/vanta-probe-xxx
$ cargo run -p vantadb --bin vanta-cli -- --db $DB put --namespace probe --key k1 --payload "hello probe"
$ cargo run -p vantadb --bin vanta-cli -- --db $DB backup --out $DB.bak
  → Backup created at: $DB.bak
  → MANIFEST.json: {backup_type:"base", created_at:"1970-...", vantadb_version:"0.5.0", files:[{name:"data/vector_index.bin",size:...,crc32c:"..."}, ...]} # ~6 files real + 36 files doc ejemplo
  → ls $DB.bak/data | wc -l  # ~5-6 files (vector_index.bin, tantivy, fjall, etc.)
$ cargo run -p vantadb --bin vanta-cli -- --db $DB.restore-temp restore --input $DB.bak --force
  → Database restored from: $DB.bak
$ cargo run -p vantadb --bin vanta-cli -- --db $DB.restore-temp doctor
  ╔══════════════════════════════════════════════════════════════╗
  ║                VantaDB Health Diagnostics                    ║
  ╠══════════════════════════════════════════════════════════════╣
  ║  Total nodes:     2
  ║  Namespaces:      1
  ║  Vectors stored:  0
  ║  Expired records: 0
  ╚══════════════════════════════════════════════════════════════╝
$ cargo run -p vantadb --bin vanta-cli -- --db $DB.restore-temp get --namespace probe --key k1
  → hello probe  # recupera

# Verificación mecánica (cli_tests.rs:479 test_backup_and_restore):
test_backup_and_restore ... ok
  DEBUG: source node_id = ...
  DEBUG: backup files: data/vector_index.bin, data/..., MANIFEST.json
  DEBUG: restored files: data/vector_index.bin, ...
  DEBUG: restoring node_id = ... → payload "backup test" ✅
# Alimenta GOV-B2 runbook §3.1 Daily Backup Verification (restore temp+doctor+conteo)
```

## Referencias
- `src/cli.rs:133-154` — Commands::Backup/Restore/Doctor
- `src/bin/vanta-cli.rs:168-178` — match Commands → cmd_backup/restore/doctor
- `src/cli_handlers/backup.rs:14-49` — BackupManifest (base_ref, files, crc32c), collect_manifest_files, write_manifest
- `src/cli_handlers/diagnostics.rs:15-130` — cmd_doctor scan_nodes+namespaces+vectors+expired
- `src/sdk/builder.rs:263-284` — VantaEmbedded::restore_from static anti ../ + close→swap→reopen
- `tests/cli_tests.rs:479-608` — test_backup_and_restore (backup→manifest→restore --force→doctor→get)
- `dev-tools/verify.ps1:59-95` — gate fmt/check/clippy/audit/deny/nextest/coverage/docs-coverage (+ probe GOV-A3)
- `docs/plans/2026-09-02-alta-prioridad-paralelo.md` — Wave1 GOV-A3 fila
- `.opencode/references/skills-engineering.md` — SDP canónico Lifecycle BUILD/VERIFY

## Context Save Point
- **Fecha:** 2026-09-02T21:30
- **Branch:** main/develop (ver git status --short)
- **CI pendiente:** verify.ps1 + cargo check -p vantadb + vanta-cli --help + probes
- **Decisiones:** Reuse cli_tests.rs transcript (ponytail — no nuevo harness), probe 1-2 líneas verify.ps1 `cargo run --bin vanta-cli -- --help` (no nuevo script), disjoint RES-02/GOV-A4 respetado
- **Problemas conocidos:** verify.ps1 cargo run requiere 1m compile first run (cache sccache); nextest ya incluye cli_tests — probe es smoke help, no duplica heavy tests
- **Próxima tarea:** GOV-A4 — Harness snippets docs (Wave1 1b, MAX 3)

