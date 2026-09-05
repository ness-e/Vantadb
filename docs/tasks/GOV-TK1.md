# GOV-TK1: `vanta-cli doctor --fix` con dry-run seguro

## Metadata
- **Plan file:** `docs/plans/2026-09-03-quality-gtm-wave.md` (Wave0 — Task 2)
- **Fuente:** plan Task 2 + Backlog fila GOV-TK1 + auditoría 2026-08-21 §D4 (flags fantasma)
- **Esfuerzo:** 🟡 ~3h
- **Prioridad:** 🟡 Media (DR runbook depende conceptualmente)
- **Tipo:** Rust CLI feature-add (superficie CLI: nuevo flag `--fix` + `--force`)
- **Turns estimados:** 4
- **Creado:** 2026-09-03
- **last-synced:** 2026-09-03
- **Estado:** ✅ COMPLETED 2026-09-03
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0 steps
- **Campaign ID:** 2026-09-03-quality-gtm-wave
- **Tipo auto-detectado:** `docs` por MCP (regex) → override manual a `feature-add` (nuevo flag CLI `pub` en `cli.rs` + handler). Gate D satisfecho por plan (scope+contrato explícitos).
- **SDP:** campaign-executor, progreso, ponytail (full), test-driven-development, incremental-implementation, context-engineering, code-review-and-quality, observability-and-instrumentation (keywords: cli, doctor, fix, dry-run, runbook; manifest grep sin candidatos extra → base + lifecycle + pedidas por prompt)
- **Research Digest:** `dry_run: bool` existe SOLO en `Migrate::Run` (cli.rs:380) — NO en `Restore` (restore --dry-run NO existe; runbook §2.6 lo dice honesto). `Doctor` es unit variant (cli.rs:154) sin flags → `--fix` hoy falla exit 2 `unexpected argument '--fix'`. Regla 8 no dispara (sin dashmap/parking_lot/Tokio nuevos); si el fix tocara paths multi-índice → exigir auditoría deadlock (no aplica: solo fs create_dir_all + open read-only).

## Gate D (question-gates.md) — evaluado, SATISFECHO POR PLAN (no `question`)
- Blast radius: 4 archivos (`src/cli.rs`, `src/bin/vanta-cli.rs`, `src/cli_handlers/diagnostics.rs`, `tests/cli_tests.rs`) + 1 doc (`DISASTER_RECOVERY_RUNBOOK.md` §2.3/§3). Sin hot path (diagnostics abre read-only + fs metadata), sin API pública nueva (solo CLI surface), SÍ agrega símbolos `pub` nuevos (`Doctor { fix, force }` fields + `cmd_doctor` params) → feature-add de superficie CLI.
- Plan declara explícito: "Gate D ya satisfecho por este plan: scope y contrato explícitos" + pre-mortem + stop conditions + risk register (dry-run default + confirm).
- Veredicto: proceder sin `question` al usuario. Scope congelado: crear dirs faltantes + report permisos; NUNCA reparación destructiva.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/bin/vanta-cli.rs:185` (`Commands::Doctor` dispatch) — único caller de `cmd_doctor` |
| Callees | `open_database` (read-only), `engine.scan_nodes`, `engine.get_memory_stats`, `std::fs::create_dir_all`, `print_success/warning/info` |
| Implicaciones | CLI surface cambia (help muestra `--fix/--force`); `cmd_doctor` firma cambia (db_path, fix, force, verbose) → actualizar todos los callers (bin + tests). Sin cambio en engine/storage/wal. Sin perf (fs ops O(1)). |

## Impacto mapeado (Regla 0) — OBLIGATORIO antes de cualquier edición

- **Archivos leídos (completos):**
  - `src/cli.rs` (436L — `Doctor` unit variant :154, `Migrate::Run.dry_run` :380 como patrón a seguir)
  - `src/bin/vanta-cli.rs` (236L — dispatch `Commands::Doctor => cmd_doctor(&args.db, verbose)` :185)
  - `src/cli_handlers/diagnostics.rs` (428L — `cmd_doctor(db_path, verbose)` :17, report-only, early-return si dir no existe)
  - `src/cli_handlers/fmt.rs` (81L — `print_success/info/warning`, `confirm_action` disponible)
  - `src/cli_handlers/backup.rs` (`cmd_restore` sin dry_run — NOTICED BUT NOT TOUCHING)
  - `src/cli_handlers/mod.rs`, `src/cli_handlers/db.rs` (open helpers)
  - `src/storage/engine/init.rs:159-315` (layout: base_path + `.vanta.lock` + `data/` via create_dir_all; schema `.vanta.schema` via load_or_create)
  - `src/schema.rs:165-194` (load_or_create vs check_compat)
  - `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` (473L — §2.3:146 "there is no doctor fix flag" HONESTO hoy; §2.6:242 "no restore dry-run" HONESTO; §3 health checks)
  - `tests/cli_tests.rs` (1219L — `test_doctor_*` 3 tests llaman `cmd_doctor(path, bool)`)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):**
  - `diagnostics.rs` → `crate::cli_handlers::{create_spinner, human_readable_size, memory_node_id, open_database, print_info, print_warning, FIELD_*}` + `fmt::{header_style, info_style}` + `error::{ChainedError, Result}` + `node::{FieldValue, NodeFlags, VectorRepresentations}`
  - `vanta-cli.rs` → `vantadb::cli::{Cli, Commands, ExportFormat}` + `cli_handlers`
  - `cli.rs` → `clap::{Parser, Subcommand, ValueEnum}`
- **Archivos que referencian a los editados (referencias entrantes):**
  - `rg "cmd_doctor" src/ tests/` → `src/cli_handlers/diagnostics.rs` (def), `src/bin/vanta-cli.rs:185` (único caller prod), `tests/cli_tests.rs:645,657,665` (3 callers test)
  - `rg "Commands::Doctor" src/` → solo `src/cli.rs` (def) + `src/bin/vanta-cli.rs` (match)
  - `rg "doctor --fix" docs/` → 0 en operations (post GOV-B2 Count==0 honesto); 13 hits solo en tracking (Backlog/planes/reviews históricos, no pasos de usuario)
- **Veredicto impacto:** BAJO. 2 archivos prod (cli.rs enum + diagnostics handler + 1 línea dispatch) + tests. Sin storage/engine/wal/config. Reversible (`git revert`). Sin concurrencia nueva.

## Contrato (ley)
- `rg -n -e "fix" src/cli.rs | rg -c "doctor|Fix|--fix"` ≥1 (flag existe en superficie CLI)
- `vanta-cli doctor --fix` sobre DB temporal exit 0 con salida listando reparaciones (o "nothing to fix")
- `rg -n "doctor --fix" docs/operations/DISASTER_RECOVERY_RUNBOOK.md` ≥1 y es VERDAD ahora (actualizar §2.3 honesto: flag existe, scope seguro, dry-run default)
- `cargo clippy --workspace --all-targets -- -D warnings` exit 0
- NO tocar `docs/operations/BENCHMARKS.md` ni `src/config.rs` (otros waves). NO reparación destructiva (stop → hallazgo).

## Spec (SDD — feature-add superficie CLI)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Forma del flag | A) `Doctor { fix: bool, force: bool }` (sigue patrón Restore force/migrate dry_run+force; dry-run default sin prompt, `--force` aplica) / B) `Doctor { fix: bool, dry_run: bool }` (dry_run default true — confuso con clap bool) / C) `--fix` interactivo con `confirm_action` (bloquea tests no-tty) | A | ✅ decidido-por-evidencia (ref: `src/cli.rs:145-150` Restore force, `src/cli.rs:379-383` Migrate dry_run+force, `src/cli_handlers/fmt.rs:74` confirm_action existe pero tests son no-interactivos) |
| 2 | Qué repara `--fix` (scope seguro) | A) crear `db_path` + `db_path/data/` faltantes (aditivo, reversible, O(1)) / B) + fix permisos/chown (no-portable win/unix, riesgo) / C) + borrar `.vanta.lock` stale / tocar WAL/datos (DESTRUCTIVO — prohibido por stop condition) | A (B solo report, C solo report "left alone — manual review") | ✅ decidido-por-evidencia (ref: `src/storage/engine/init.rs:180,311` engine ya hace create_dir_all en writable open; doctor abre read-only → no crea nada hoy; lock es process-exclusive `init.rs:178-261` → borrarlo con otro proceso vivo = data race) |
| 3 | Semántica sin `--force` | A) dry-run puro: lista WOULD-FIX + "re-run with --force", exit 0, cero mutación / B) aplica directo (viola risk register dry-run default) | A | ✅ plan lo exige ("dry-run por default + confirmación"; Risk Register 🟡×🔴 fix destructivo silencioso) |
| 4 | Salida listando reparaciones | A) `print_success("Fixed: ...")` / `print_warning("Would fix: ...")` / `print_success("nothing to fix")` (reusa fmt helpers, testeable por exit code + fs state) / B) JSON `--json` (scope creep, sin precedente en doctor) | A | ✅ ponytail ladder (rung 2: reusar `fmt.rs` existente) |
| 5 | Runbook honesto | A) §2.3 reescribe "NOT available" → documenta `doctor --fix [--force]` con scope seguro + ejemplo dry-run; §3 tabla añade fila fix / B) dejar runbook intacto (contrato exige ≥1 hit VERDAD → imposible) | A | ✅ contrato lo exige |

**Código real — callback helper error/string (patrón del handler):**
```rust
// diagnostics.rs — errores propagados con Result, sin unwrap/expect (Regla worker #1).
// Mensajes humanos vía print_* helpers (no anyhow en lib; anyhow solo en bin).
pub fn cmd_doctor(db_path: &str, fix: bool, force: bool, verbose: bool) -> Result<()> {
    // dry-run default: sin --force no hay mutación; lista WOULD-FIX y sale 0.
    // Con --force: create_dir_all(db_path) + create_dir_all(db_path/data) — aditivo.
    // .vanta.lock stale / permisos / WAL / datos → solo report ("left alone"), nunca borrar.
}
```

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** (1) `--fix` sin `--force` NUNCA muta el filesystem; (2) `--fix --force` solo crea dirs (`create_dir_all`), nunca borra/sobrescribe WAL, datos, `.vanta.lock`, `.vanta.schema`; (3) `doctor` sin `--fix` mantiene salida actual byte-compatible (no romper scripts que parsean); (4) sin `dashmap/parking_lot/Tokio` nuevos.
- **Comandos de verificación:** `cargo run --quiet --bin vanta-cli -- doctor --fix --db <tmp>` exit 0 + `cargo clippy --workspace --all-targets -- -D warnings` exit 0 + `cargo nextest run --profile audit -p vantadb --test cli_tests` 0 failed
- **Deuda pendiente:** ninguna (si aparece reparación destructiva necesaria → STOP + hallazgo FIND-*, no implementar)

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | # GOV-TK1: `vanta-cli doctor --fix` con dry-run seguro |
| `lastAction` | DISCOVERY completo + repro `--fix` exit 2 (ver Notas) |
| `result` | `PARTIAL` (DISCOVERY ✅, Steps 1-4 ⬜) |
| `nextAction` | Step 1: `src/cli.rs` Doctor struct + dispatch (archivo + `cargo check -p vantadb`) |
| `contract` | `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | Wave0 resto (RES-07, GOV-TK9 ✅ done por otro agente) |

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda. No se introduce `unsafe`, `unwrap/expect` (prohibidos), ni clon en hot path. `confirm_action` no se usa (evita bloqueo tty) → sin deuda interactiva.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato (4 ANDs) + `cargo nextest cli_tests` verde + runbook honesto |
| **Commit** | Atómico, `feat(cli): doctor --fix con dry-run seguro (GOV-TK1)`, solo archivos del blast radius |
| **Release** | `dev-tools/verify.ps1` (o `just verify`) + CHANGELOG si user-visible (CLI docs) |

## Herramientas necesarias
- read/grep/edit/bash/codegraph_explore/campaign_verify_cmd/campaign_update_task_state

**Skills cargadas (SDP):** campaign-executor (base task-system), progreso (cierre avance), ponytail full (ladder YAGNI), test-driven-development (RED→GREEN), incremental-implementation (slices verticales), context-engineering (hierarchy Rules→Spec→Source), code-review-and-quality (pre-commit gate), observability-and-instrumentation (mensajes CLI observables: dry-run lista reparaciones, exit codes)

## Investigation Notes
- **Claim:** `doctor --fix` no existe hoy (exit 2 unexpected argument)
  **Evidencia:** `cargo run --quiet --bin vanta-cli -- doctor --fix` → `error: unexpected argument '--fix found` exit 2 (ejecutado hoy)
  **Confianza:** alta
- **Claim:** `restore --dry-run` tampoco existe (research digest del prompt confunde con `Migrate::Run.dry_run`)
  **Evidencia:** `src/cli.rs:141-151` Restore solo input/force/rebuild; `cargo run --quiet --bin vanta-cli -- restore --help` sin `--dry-run`; `src/cli.rs:380` es `Migrate::Run.dry_run`
  **Confianza:** alta → NOTICED BUT NOT TOUCHING (fuera de scope GOV-TK1; no tocar restore)
- **Claim:** runbook es honesto hoy (0 hits `doctor --fix` en operations tras GOV-B2)
  **Evidencia:** `rg "doctor --fix" docs/` → 0 en operations; `DISASTER_RECOVERY_RUNBOOK.md:146` "there is no doctor fix flag", `:242` "no restore dry-run mode"
  **Confianza:** alta → tras implementar, §2.3 debe pasar a documentar el flag real (contrato exige ≥1 VERDAD)

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — bug-flag-fantasma)
- **Repro:** `cargo run --quiet --bin vanta-cli -- doctor --fix --db C:/Temp/vanta-doctor-repro` → `error: unexpected argument '--fix' found` exit 2. `doctor --help` no lista `--fix`.
- **Hipótesis:** `Commands::Doctor` es unit variant sin campos (cli.rs:154) + dispatch ignora flags (vanta-cli.rs:185) → clap rechaza `--fix` antes de llegar al handler.
- **1 variable controlada:** agregar campos `fix/force` al variant + pasarlos al handler (nada más en este intento).
- **Test RED:** `cargo nextest run -p vantadb --test cli_tests doctor_fix` debe FALLAR antes del fix (nuevo test `doctor_fix_dry_run_lists_repairs` llama `cmd_doctor(path, true, false, false)` — no compila hoy por firma de 2 args) + repro CLI exit 2.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — trust boundary: `db_path` es input usuario → `ops::prevent_path_traversal` ya lo cubre en `init_storage`; `--fix --force` solo `create_dir_all` (aditivo, sin overwrite, sin chown, sin borrar lock/WAL). Sin auth/sesiones/deps nuevas/FFI/red. No se carga `security-and-hardening` (sin trust boundary nuevo; justificado).
- [x] **PERFORMANCE** — no toca hot path (diagnostics abre read-only + scan existente; fix añade 2× `create_dir_all` + metadata checks O(1)). Sin bench (Regla 9 no dispara: no es optimización). No se carga `performance-optimization` (justificado).

## Steps
### Step 1: CLI surface — `Doctor { fix, force }` + dispatch (RED→GREEN slice 1)
- **Archivos:** `src/cli.rs:153-154`, `src/bin/vanta-cli.rs:185`
- **Acción:** `Doctor` pasa de unit a struct variant `{ #[arg(long)] fix: bool, #[arg(long)] force: bool }` con docs (dry-run default + `--force` aplica). Dispatch: `Commands::Doctor { fix, force } => cmd_doctor(&args.db, fix, force, verbose)`. Nada más.
- **Verify:** `cargo check -p vantadb` ✅ + `cargo run --quiet --bin vanta-cli -- doctor --help` lista `--fix/--force` ✅ + `cargo run --quiet --bin vanta-cli -- doctor --fix --db <tmp>` exit 0 ✅ (ejecutado hoy, ver Notas)
- **Estado:** ✅ COMPLETED

### Step 2: Handler safe-fix — dry-run default + `create_dir_all` (slice 2)
- **Archivos:** `src/cli_handlers/diagnostics.rs:17`
- **Acción:** `cmd_doctor(db_path, fix, force, verbose)`:
  1. Sin `--fix`: comportamiento actual idéntico (early-return si no existe + diagnostics).
  2. Con `--fix` sin `--force` (dry-run): computa reparaciones pendientes (db dir faltante, data/ faltante, permisos no-writable → report) e imprime `Would fix: ...` + `re-run with --force to apply` + `nothing to fix` si cero; CERO mutación; luego sigue con diagnostics si la DB abre (o sale 0 si no existe).
  3. Con `--fix --force`: aplica solo (1) `create_dir_all(db_path)`, (2) `create_dir_all(db_path/data)`; imprime `Fixed: ...`; lock/permisos/WAL/datos → `Left alone: <id>: <reason>` (nunca borrar).
  4. Sin `unwrap/expect`, sin `unsafe`, reusa `print_success/warning/info`.
- **Verify:** `cargo check -p vantadb` + `cargo clippy -p vantadb --all-targets -- -D warnings` ✅ (workspace clippy ✅)
- **Estado:** ✅ COMPLETED

### Step 3: Tests CLI — `doctor --fix` dry-run + force (TDD GREEN)
- **Archivos:** `tests/cli_tests.rs` (append; actualizar 3 callers `cmd_doctor(path, bool)` → `(path, false, false, bool)`)
- **Acción:** nuevos tests: `doctor_fix_dry_run_no_mutation` (dir fantasma: dry-run exit ok + dir NO creado), `doctor_fix_force_creates_dirs` (force crea db+data/), `doctor_fix_nothing_to_fix` (db existente sana → ok). Actualizar `test_doctor_*` existentes a nueva firma con `fix=false`.
- **Verify:** `cargo nextest run --profile audit -p vantadb --test cli_tests` 0 failed → ejecutado vía `cargo test -p vantadb --test cli_tests doctor` 6/6 ✅ + file completo `--test-threads=1` 81/82 (1 fallo ambiental disco lleno en `test_backup_and_restore`, causa OS 112, sin relación)
- **Estado:** ✅ COMPLETED

### Step 4: Runbook honesto + verify full + commit + progreso (cierre)
- **Archivos:** `docs/operations/DISASTER_RECOVERY_RUNBOOK.md` (§2.3 + §3 tabla), `docs/plans/2026-09-03-quality-gtm-wave.md` (Task 2 → ✅), commit
- **Acción:** §2.3: reemplazar "NOT available (no doctor fix flag)" por doc real `doctor --fix` (dry-run default, `--force` aplica, scope seguro: crea dirs; lock/WAL/datos = manual) con ejemplo. §3: fila `doctor --fix` en checks. Verify full (fmt+clippy+nextest+deny/docs). Commit `feat(cli): doctor --fix con dry-run seguro (GOV-TK1)` solo blast radius. `skill progreso`.
- **Verify:** contrato completo (4 ANDs) + `rg -n "doctor --fix" docs/operations/DISASTER_RECOVERY_RUNBOOK.md` 3 hits VERDAD ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (Wave0 paralelo con RES-07 y GOV-TK9 — disjuntos: config.rs/BENCHMARKS y checklist vs cli.rs/handlers).

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-review (scope discipline + no-destructivo) — chaos NO (Regla 8 no dispara).
- **Enfoque:** ¿`--fix --force` puede borrar/sobrescribir datos en algún path? ¿dry-run default realmente no muta?
- **Cómo se probó:** `cargo nextest cli_tests` + repro CLI binario sobre DB temporal + `rg` contrato.
- **Checklist anti-hábitos tóxicos:**
  - [ ] No inventar salidas de comandos no ejecutados.
  - [ ] No saltarse clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar acceptance criteria.
  - [ ] No ignorar fallos ni reportar "todo OK" con fallo parcial.
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos.
  - [ ] No degradar chequeo de errores en paths de dinero/seguridad.
  - [ ] No gastar presupuesto infinito.
- **Veredicto:** ⬜ pendiente (tras Step 4, pre-commit `code-review-and-quality`)

## Notas (cierre 2026-09-03)
- **Evidencia de verificación ejecutada hoy:**
  - RED: `doctor --fix` → `error: unexpected argument '--fix' found` exit 2 (pre-fix).
  - GREEN dry-run missing dir: `Would fix: create missing database directory` + `Would fix: create missing data subdirectory` + `dry-run: re-run with --force` + `(empty)` warning, exit 0, dir NO creado.
  - GREEN force: `Fixed: ...` ×2, exit 0, `data/` existe.
  - GREEN healthy seeded DB: `doctor --fix: nothing to fix` + diagnostics completos, exit 0.
  - GREEN empty-initialised (dirs sin schema): `nothing to fix` + `Database is empty/uninitialised (lock_file not found...); nothing further to diagnose`, exit 0.
  - `cargo fmt --check` 0 · `cargo clippy --workspace --all-targets -- -D warnings` 0 · `cargo test cli_tests doctor` 6/6 · `rg fix cli.rs | rg doctor|Fix|--fix` ≥1 (cli.rs:157,159) · `rg doctor --fix runbook` 3 hits VERDAD.
  - `validate-docs-coverage.ps1`: cli.rs 42 ok, 0 gaps nuevos; único gap `embed_texts`/MCP ya trackeado en FIND-53 (colateral con ticket → sin acción).
  - Full workspace nextest: NO corre en esta máquina (link LNK1140 PDB limit + disco C: 958MB libres — ambiental, binarios de `vantadb-mcp` no tocados). `cargo test cli_tests --test-threads=1`: 81/82, único fallo `test_backup_and_restore` por OS error 112 disco lleno (ambiental, pre-existente por diseño del test que restaura dentro del propio source dir).
- **Review (self, pre-commit — revisor distinto pendiente en `/audit certify`):** correctitud ✅ (spec tabla 5/5), simplicidad ✅ (ladder rung 2: reusa fmt helpers + patrón force), arquitectura ✅ (precedente Restore/Migrate), seguridad ✅ (solo create_dir_all, sin overwrite/borrado, traversal ya cubierto por init_storage), performance ✅ (O(1) fs checks).
- **Backlog fila GOV-TK1 (línea 431): NO eliminada** — cubre también `Restore --dry-run`/`verify` (mitad pendiente, runbook §2.6 honesto "PENDING"). Solo la mitad `doctor --fix` cierra aquí; avance registra el parcial. Orquestador decide si splitea la fila.
- NOTICED BUT NOT TOUCHING: `restore --dry-run` no existe (research digest del prompt era inexacto; es `migrate --dry-run`); `src/config.rs`, `BENCHMARKS.md` (otros waves); `discovery file stale` no existe en el repo (scope reducido a dirs + report).
- NOTICED BUT NOT TOUCHING: `restore --dry-run` no existe (research digest del prompt es inexacto; es `migrate --dry-run`). Fuera de scope → ¿crear FIND-*? No: GOV-TK1 fila Backlog ya lo menciona como contexto; no abrir ticket nuevo sin evidencia de que el runbook lo exija hoy (runbook §2.6 es honesto). `src/config.rs`, `BENCHMARKS.md` → otros waves, prohibido tocar.
- `discovery file stale` del prompt no existe como tal en el repo (grep sin hit de discovery-file). Interpretación segura: no hay archivo discovery que normalizar; el scope se reduce a dirs + report. Si aparece un archivo stale real durante implement → report-only, no borrar.
- Question Gates D/V/C: D satisfecho por plan (ver arriba); V no dispara aún (0 fallas verify); C al cierre (colaterales: restore --dry-run ausente → no se arregla inline, ya documentado aquí).

## Referencias
- `.opencode/references/definition-of-done.md` — standing quality bar
- `.opencode/references/skills-engineering.md` — SDP lifecycle mapping
- `SKILLS-MANIFEST.md` — catálogo de skills disponibles

## Context Save Point
- **Fecha:** 2026-09-03
- **Branch:** develop
- **CI pendiente:** no (aún sin edits; tras Steps 1-3: verify full pendiente)
- **Decisiones:** `fix+force` sobre `fix+dry_run` (evidencia: patrón Restore/Migrate + tests no-tty); scope = create dirs + report (evidencia: init.rs layout + lock exclusivo); runbook §2.3 se reescribe (contrato exige VERDAD).
- **Problemas conocidos:** campaign MCP usa IDs numéricos (esta tarea = plan Task 2); task file canónico es `GOV-TK1.md` (nombre del prompt).
- **Próxima tarea:** Step 1 (esta sesión continúa: PLAN→ACT→VERIFY por step).

## Segunda mitad — restore --dry-run (2026-09-03, re-escalada Backlog: "queda el verificador de restore")

> NO rehacer Steps 1-4 (doctor --fix landed `adf03752`). Continuar desde aquí.

### Impacto mapeado restore --dry-run (Regla 0) — ANTES de editar código

- **Archivos leídos (completos):**
  - `src/cli.rs:140-151` (`Restore { input, force, rebuild }` sin `dry_run`; patrón `Migrate::Run.dry_run` :387-388 a seguir)
  - `src/cli_handlers/backup.rs:225-297` (`cmd_restore(db_path, input, force, rebuild, verbose)` — copia hoy: exists check :235, force gate :244, `remove_dir_all` :253, `create_dir_all` :260, `copy_dir` :264, rebuild opcional :270-278)
  - `src/bin/vanta-cli.rs:179-183` (dispatch `Commands::Restore => cmd_restore(...)` — único caller prod)
  - `src/cli_handlers/fmt.rs` (`print_success/info/warning` a reusar, patrón doctor)
  - `src/cli_handlers/util.rs` (`dir_size`, `human_readable_size` a reusar)
  - `tests/cli_tests.rs:482-635` (`test_backup_and_restore` + `test_restore_missing_backup` llaman `cmd_restore(..., true, false, bool)` 5 args; `setup_temp_db` + `seed_record` patrón para dry-run)
  - `docs/operations/DISASTER_RECOVERY_RUNBOOK.md:249-251,324-325` ("no restore dry-run" honesto hoy; §3.1 full-restore procedure)
- **Referencias entrantes:** `rg "cmd_restore" src/ tests/` → def `backup.rs:227`, caller `vanta-cli.rs:183`, tests `cli_tests.rs:556,573,630` (3 sites). `rg "Commands::Restore"` → `cli.rs` def + `vanta-cli.rs` match.
- **Referencias salientes:** `backup.rs` → `cli_handlers::{create_spinner, dir_size, human_readable_size, open_embedded, print_info/success/warning}` + `error::{ChainedError, Result}` + `walkdir_flat/copy_dir` locales.
- **Veredicto:** BAJO. 3 archivos prod (cli.rs flag + backup.rs early-return + 1 línea dispatch) + tests. Sin storage/engine/wal/config. Sin `dashmap/parking_lot`/Tokio nuevos → Regla 8 no dispara. Reversible. Sin flag = comportamiento actual intacto.

### Contrato segunda mitad (ley)
- `rg "dry_run" src/cli.rs` dentro de `struct Restore` ≥1 (`--dry-run` flag existe)
- `cmd_restore` maneja `dry_run`: valida input (existe, es dir, no vacío, MANIFEST si presente) + tamaño + conflictos target, LISTA qué restauraría, CERO mutación; sin flag = comportamiento actual intacto
- Tests nuevos `restore_dry_run_*`: inexistente → error claro; válido + target existente → dry-run Ok y target UNTOUCHED (assert antes/después idéntico)
- `vanta-cli restore --input <tmpbackup> -d <tmp> --dry-run` exit 0 sin mutar
- Runbook: si menciona `restore --dry-run` es VERDAD (comando real, scope dry-run)
- `cargo clippy --workspace --all-targets -- -D warnings` 0 + `cargo fmt --check` 0 + `cargo test -p vantadb --lib cli` 0 failed
- NO tocar `src/ingestion.rs` (FIND-57) ni `vantadb-server/` (FIND-56). NO stagear `completions/`, `Cargo.lock`, `.opencode`. NO tocar `stash@{0}`.

### Spec restore --dry-run (decisiones por evidencia)
| # | Decisión | Resuelto |
|---|----------|----------|
| 1 | Flag explícito `dry_run: bool` (`#[arg(long)]`, patrón `Migrate::Run.dry_run`) — NO `fix/force` estilo doctor (restore ya tiene `force` con semántica overwrite; `--dry-run` es ortogonal y auto-documentado) | ✅ evidencia `cli.rs:387` |
| 2 | Firma `cmd_restore(db, input, force, rebuild, dry_run, verbose)` (`dry_run` antes de `verbose`, `verbose` último como en resto de handlers) | ✅ convención handlers |
| 3 | Dry-run valida read-only: `!exists`→Err claro; `!is_dir`→Err; vacío→Err; `MANIFEST.json` presente→parse valida (Err si corrupto), ausente→warning legacy; `dir_size` + lista `walkdir_flat` relativos; conflicto target→warning (NO error: preview sin `--force` debe salir 0); `--rebuild`→informa "would rebuild" | ✅ ponytail rung 2 (reusar `walkdir_flat/dir_size/print_*`) |
| 4 | CERO mutación en dry-run: early-return antes de spinner/remove/create/copy/rebuild/open; sin flag = código actual byte-idéntico | ✅ invariante |
| 5 | Runbook §2.6 + §3: reemplazar "no restore dry-run" por doc real `restore --dry-run` con ejemplo + límites (no valida CRCs por archivo, no abre DB) | ✅ contrato VERDAD |

### Steps segunda mitad
- [x] Step 5 (RED): tests `restore_dry_run_*` nuevos con firma 6 args → FALLARON (E0061 6 vs 5, razón correcta) ✅
- [x] Step 6 (GREEN): `cli.rs` flag + `backup.rs` early-return dry-run + dispatch `vanta-cli.rs` + actualizar 3 callers viejos a `false` ✅ (`cargo check --tests` 0)
- [x] Step 7 (VERIFY+Cierre): clippy/fmt/test + binario real `--dry-run` exit 0 sin mutar + runbook honesto + Backlog fila ELIMINADA + avance + memory + commit `feat(cli): restore --dry-run validación sin mutar (GOV-TK1)` ✅ (`ed938c86`, 7 files, pre-commit fmt/clippy/actionlint ok; stash@{0} intacto; completions/Cargo.lock/.opencode NO stageados)
