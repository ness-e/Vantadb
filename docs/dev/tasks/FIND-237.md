---
title: "FIND-237: CLI migrate — aceptar el global --db como fallback del target posicional"
kind: task
description: "vanta-cli migrate check --db X falla (target posicional obligatorio) — el fix hace target Option<String> con precedencia positional > --db, unifica la resolución en el dispatch y agrega ejemplo en --help; tests de ambos caminos."
---

# FIND-237: CLI `migrate`: aceptar el global `--db` como fallback del `target` posicional

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 1, F0)
- **Fuente:** validación externa v0.8.0 (2026-10-03) → fila FIND-237 (Backlog DELTA)
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡
- **Tipo:** Rust (CLI — superficie pública)
- **Turns estimados:** 8-12
- **Creado:** 2026-10-04T02:07
- **last-synced:** 2026-10-04T02:50
- **Estado:** ✅ COMPLETED (2026-10-04) — contrato 4/4 + review P2-01 APPROVE; commit local (hash en la recitation MCP).
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps (cierre en este commit)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/bin/vanta-cli.rs` (dispatch `Commands::Migrate`, único call site de producción); `tests/cli_tests.rs` (tests de handler L1441-1459, L1475-1683 + módulo binario `api07_cli_binary` L1773-1927); `src/cli.rs` mod tests (test clap L517-541) |
| Callees | `src/cli_handlers/migrate.rs` (`cmd_migrate_plan` L15, `cmd_migrate_check` L83, `cmd_migrate` L137) → `crate::migration::MigrationEngine`; clap 4.x derive |
| Implicaciones | Solo cambia la superficie clap (`target: String` → `Option<String>`) y la resolución en el dispatch; handlers intactos (siguen recibiendo `&str`). Invocación posicional existente sin cambio de comportamiento; no hay performance/serialización/migración de datos. `--help` de `migrate` gana un bloque de ejemplos (after_help). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/cli.rs` (541L), `src/bin/vanta-cli.rs` (353L), `src/cli_handlers/migrate.rs` (539L, vía codegraph — verbatim), `tests/cli_tests.rs` (helpers L1-46 + secciones migrate L1400-1683 + módulo binario L1750-1927; archivo de 1927L), `.opencode/rules/api-contract.md`, `clean-code-clean-architecture.md` Apéndice V.
- **Archivos referenciados hacia dentro (imports/deps de los editados):** `src/cli.rs` ← `src/bin/vanta-cli.rs` (`use vantadb::cli::{Cli, Commands, ExportFormat}`), `build.rs` (completions), `src/cli_handlers/*` (re-exporta tipos); dispatch usa `cli_handlers::{cmd_migrate_plan, cmd_migrate_check, cmd_migrate}`.
- **Archivos que referencian a los editados (referencias entrantes):** grep `MigrateCommand|cmd_migrate_plan|cmd_migrate_check` = 11 hits en 4 archivos: `src/cli.rs` (def), `src/bin/vanta-cli.rs` (dispatch), `src/cli_handlers/migrate.rs` (handlers), `tests/cli_tests.rs` (tests de handler). Sin otros bins ni crates que construyan `MigrateCommand`.
- **Veredicto impacto:** bajo — 2 archivos de código + 2 de tests; única superficie afectada es el parseo clap de `migrate {plan,run,check}`; sin breaking para la invocación posicional (precedencia positional > `--db`).

## Contrato

"`vanta-cli migrate check --db <db>` → exit 0 (fallback al global) **y** `vanta-cli migrate check <TARGET>` sigue funcionando (positional gana); `--help` muestra un ejemplo de uso; tests CLI verdes."

Verificación exacta (nota: en esta sesión `target/debug/vanta-cli.exe` está lockeado por los MCP servers de memoria → los comandos cargo de abajo corren con `--target-dir target/find237`; en CI/lock-free corren tal cual):
1. `cargo nextest run --profile audit -p vantadb --lib migrate` (parse: fallback + precedencia + help) → 11/11 ✅
2. `cargo nextest run --profile audit -p vantadb --test cli_tests --ignore-default-filter migrate_check` (binario real: ambos caminos; `--ignore-default-filter` es obligatorio — `cli_tests` está excluido por `default-filter` en `.config/nextest.toml`) → 3/3 ✅
3. `cargo fmt --check` ✅ + `cargo clippy -p vantadb --all-targets -- -D warnings` ✅

## Spec (decisiones resueltas por plan + evidencia de código)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Forma del fix | A: `target: Option<String>` + fallback al global vs B: alias `--target` vs C: comando nuevo | A | ✅ Plan FIND-237 §Pre-mortem(1): "target queda `Option<String>` con precedencia positional > `--db`" |
| 2 | Alcance por variante | A: Plan+Run+Check (convención única) vs B: solo Check (mínimo literal) | A | ✅ Evidencia: las 3 variantes comparten el mismo patrón posicional (`cli.rs:423-448`); dejar 2/3 con la trampa contradice "elimina la confusión del único comando con convención distinta" (Gate Justificación) |
| 3 | Precedencia | A: positional > `--db` vs B: `--db` explícito > positional | A | ✅ Plan contrato: "positional gana" |
| 4 | Dónde se resuelve | A: dispatch `vanta-cli.rs` (única regla, handlers intactos) vs B: firma de handlers `Option<&str>` (ripple a tests) | A | ✅ Plan §Pre-mortem(2): "unificar resolución en el handler"; dispatch = handler del subcomando; B rompe firma pública de `cli_handlers` sin ganancia |
| 5 | Ejemplos en help | A: `#[command(after_help)]` en `MigrateCommand` vs B: `long_about` por variante | A | ✅ Plan §Pre-mortem(3): "clap no muestra ejemplos → usar `after_help`" |
| 6 | ¿Gate D (question)? | A: no dispara — fix ya especificado por plan aprobado (tipo/símbolo no nuevos; cambio de campo mandado por el plan) vs B: preguntar | A | ✅ No agrega `pub fn`/tool/endpoint nuevos; el plan F0 full-detail ya decidió el approach y el pre-mortem cubre los 3 riesgos |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `vanta-cli migrate check <TARGET>` y `migrate plan <TARGET>` / `migrate run <TARGET>` siguen funcionando idénticos (scripts existentes) — positional gana.
  2. Los handlers `cli_handlers::cmd_migrate*` conservan su firma `&str` (usados por 20+ tests directos).
  3. Sin cambios de persistencia/formatos on-disk.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb --lib migrate` y `cargo nextest run --profile audit -p vantadb --test cli_tests --ignore-default-filter migrate_check` (ambos verdes).
- **Deuda pendiente:** ninguna.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda. El cambio elimina una inconsistencia de UX (el `target` posicional obligatorio vs `--db` global del resto de comandos); no introduce atajos nuevos.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato ✅ (fallback + positional + help + tests) + fmt/clippy/nextest del área |
| **Commit** | Atómico (~60L), `fix(cli): FIND-237 — ...`, verificación mecánica registrada |
| **Release** | Entrada de changelog (release-plz, patch) — n/a en este commit; queda para 0.9.0 |

## Herramientas necesarias
- Terminal cargo (check, nextest, fmt, clippy) — MCPs de Rust deshabilitados por default
- `codegraph_codegraph_explore` (blast radius de `MigrateCommand`/dispatch)
- `codebase-memory-mcp_check_index_coverage` (cobertura de `src/cli.rs`, `src/bin/vanta-cli.rs`)
- `campaign_verify_cmd` (contrato), `campaign_update_task_state` (recitation)

**Skills cargadas (SDP):** `campaign-executor` · `progreso` · `ponytail` (base, auto) · `source-driven-development` (validar semántica clap 4 global args + after_help) · `test-driven-development` (Prove-It: RED antes del fix) · `git-workflow-and-versioning` (commit atómico local) · `incremental-implementation` (slice único) · `systematic-debugging` (bug-fix: repro/hipótesis) · `documentation-skill` (task file bajo `docs/`).

## Investigation Notes
- **Causa raíz (Phase 1):** `MigrateCommand::Check { target: String }` (`src/cli.rs:444-447`) es posicional **obligatorio**; `--db` es global con `global = true` (`cli.rs:15-22`, env `VANTADB_STORAGE_PATH`, default `./db`) pero clap falla antes de resolverlo porque exige el positional: `error: the following required arguments were not provided: <TARGET>`. El resto de los subcomandos (put/get/search/...) usan solo el global `--db` → `migrate` es la única convención distinta.
- **Patrón existente (Phase 2):** `Commands::Search` usa `Option<String>` posicional con `required_unless_present` y resolución defensiva en dispatch (`vanta-cli.rs:101-105`) — mismo shape de solución.
- **Semántica clap:** un arg `global = true` definido en el root puede pasarse en cualquier nivel del árbol de subcomandos (`migrate check --db X`); con el positional opcional, clap lo acepta. `#[command(after_help = ...)]` a nivel de enum `Subcommand` se muestra en `vanta-cli migrate --help`.
- **Web research:** n/a (semántica clap ya verificada contra el uso existente en el repo + test RED mecánico).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — approach fijado por plan; semántica clap verificada con test RED |
| Pendientes de ejecución (downhill) | 1 — Step 4 (cierre: commit + recitation) |
| % completado | 75% |

## Fase 1 — Evidencia de Debugging (GATE — bug)

- **Repro:** `vanta-cli migrate check --db <db>` → exit ≠0 con `error: the following required arguments were not provided: <TARGET>` (validación externa v0.8.0, 2026-10-03; test RED Step 1 lo reproduce en CI local).
  - Evidencia local (2026-10-04, binario pre-fix `target/debug/vanta-cli.exe`): `EXIT=2` + `error: the following required arguments were not provided: <TARGET>` / `Usage: vanta-cli.exe migrate check --db <DB> <TARGET>`.
- **Hipótesis:** clap exige el positional `target: String` y nunca llega a resolver el global `--db`; la resolución `target`/`--db` no existe en el dispatch.
- **1 variable controlada:** cambiar `target` de `String` a `Option<String>` + fallback en dispatch (nada más; handlers y persistencia intactos).
- **Test RED:** tests unit (parse fallback/precedencia/help) + tests binario real (ambos caminos) escritos y ejecutados ANTES del fix — ver Step 1.
  - Evidencia RED (compilación, `cargo nextest --lib migrate`): `error[E0308]: mismatched types — expected String, found Option<_>` en `src/cli.rs:584,591` + `E0599 no method as_deref` en `:571` → los tests expresan el contrato post-fix y hoy no compilan (RED por razón correcta).
  - **Nota de entorno:** el MCP server de memoria (`vanta-cli server --mcp`, PIDs 9204/12928) mantiene bloqueado `target/debug/vanta-cli.exe` (Access denied al relink) → los builds/tests de esta tarea corren en `CARGO_TARGET_DIR=target/find237` (deps warm-up + incremental). El gate completo `dev-tools/verify.ps1` también está bloqueado por ese lock (su probe `cargo run --bin vanta-cli`); se corre el equivalente scoped en el target alterno.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY — no aplica:** no toca trust boundaries, auth, datos ni dependencias; solo parseo de argumentos CLI (el path sigue validado por los handlers: `path.exists()`).
- [x] **PERFORMANCE — no aplica:** no toca hot paths (parseo clap en startup, una vez por invocación).

## Steps

### Step 1: RED — tests de contrato (parse + binario)
- **Archivos:** `src/cli.rs` (mod tests), `tests/cli_tests.rs` (mod `api07_cli_binary`)
- **Acción:** agregar tests que fijan el contrato: (a) `migrate check --db X` parsea y deja `target: None`; (b) positional gana cuando ambos están; (c) `migrate --help` muestra ejemplo con `--db`; (d) plan/run aceptan el fallback; (e) binario real: `migrate check --db <db>` exit 0 y `migrate check <TARGET> --db <missing>` exit 0 (positional gana).
- **Verify:** `cargo nextest run --profile audit -p vantadb --lib migrate` + `cargo nextest run --profile audit -p vantadb --test cli_tests --ignore-default-filter migrate_check` → FALLAN (RED) por la razón correcta. RED capturado por compile-fail (los tests expresan el contrato post-fix) + repro runtime con el binario pre-fix (`EXIT=2`, clap exige `<TARGET>`).
- **Estado:** ✅ COMPLETED (2026-10-04) — RED capturado: compile-fail E0308/E0599 (tests expresan el contrato post-fix) + repro runtime `EXIT=2` con el binario pre-fix.

### Step 2: GREEN — `Option<String>` + resolución unificada + after_help
- **Archivos:** `src/cli.rs`, `src/bin/vanta-cli.rs`
- **Acción:** `target: Option<String>` en Plan/Run/Check con doc de fallback; `#[command(after_help = "Examples: ...")]` en `MigrateCommand`; en el dispatch resolver `target.as_deref().unwrap_or(&args.db)` por variante (positional > global), handlers intactos.
- **Verify:** mismos comandos del Step 1 → VERDES + `cargo check -p vantadb`.
- **Estado:** ✅ COMPLETED (2026-10-04) — GREEN: `--lib migrate` 11/11 · `--lib cli::tests` 5/5 · E2E `--test cli_tests --ignore-default-filter migrate_check` 3/3 · `cargo check -p vantadb` ✅ · binario directo `FALLBACK_EXIT=0` / `POSITIONAL_EXIT=0` / help con ejemplo ✅. Nota: primer build falló por `legacy_derive_helpers` (`#[command]` antes de `#[derive]`) → corregido a derive-first.

### Step 3: VERIFY full del área
- **Archivos:** —
- **Acción:** correr fmt + clippy del paquete + suite del área (lib completa de `cli::tests` + `cli_tests` completa) vía `campaign_verify_cmd`.
- **Verify:** `cargo fmt --check` ✅ · `cargo clippy -p vantadb --all-targets -- -D warnings` ✅ · `cargo nextest run --profile audit -p vantadb --lib` ✅ · `cargo nextest run --profile audit -p vantadb --test cli_tests --ignore-default-filter` ✅.
- **Estado:** ✅ COMPLETED (2026-10-04) — `cargo fmt --all -- --check` exit 0 · clippy `-p vantadb --lib --bins --tests` exit 0 (reviewer re-corrió `--all-targets` exit 0) · `cli_tests` completo **94/94** · `campaign_verify_cmd` unit (2.7s) y E2E (4.3s) `passed:true`.

### Step 4: CIERRE — OCR + review P2-01 + commit local + recitation
- **Archivos:** `docs/dev/tasks/FIND-237.md`
- **Acción:** `pwsh dev-tools/ocr-review.ps1` (Critical/High bloquean); fork de review a `vanta-review` (agente distinto); commit local `fix(cli): FIND-237 — migrate acepta el global --db como fallback del target`; `campaign_update_task_state` + `skill progreso`.
- **Verify:** OCR sin Critical/High + veredicto review registrado + `git status` limpio de archivos ajenos.
- **Estado:** ✅ COMPLETED (2026-10-04) — OCR delegation sin Critical/High (`target/find237-ocr.json`, rule group `**/*.rs` sobre los 3 archivos); review P2-01 ronda 1 ❌ (solo registro) → fix → ronda 2 ✅ APPROVE (vanta-review, sesión `ses_efa6428c9ffeao26nUvdOu02gt`); commit local `fix(cli): FIND-237 — ...` (solo los 4 archivos de la tarea); recitation MCP + `skill progreso`.

## Dependencias
- F0 wave 0 — sin dependencias (nextTask: FIND-238).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review (subagente P2-01, contexto fresco — 2026-10-04; no participó de la implementación)
- **Enfoque:** ✅ correcto y mínimo. `target: Option<String>` en Plan/Run/Check + resolución única por variante en el dispatch (`target.as_deref().unwrap_or(&args.db)` → precedencia positional > global); handlers `cli_handlers::cmd_migrate*` intactos con firma `&str` (`src/cli_handlers/migrate.rs:15,83,137`). Alternativas evaluadas y descartadas: alias `--target` (superficie extra), `Option<&str>` en handlers (ripple de firma sin ganancia), `required_unless_present` (inútil: `--db` tiene default → siempre presente). Sin scope creep: solo los 3 archivos del diff.
- **Cómo se probó (reproducido por el revisor, no auto-reporte):**
  - `cargo nextest run --target-dir target/find237 --profile audit -p vantadb --lib migrate` → **11/11** ✅
  - `--lib cli::tests` → **5/5** ✅ · `--test cli_tests --ignore-default-filter migrate_check` → **3/3** ✅ (2 E2E nuevos con binario real `CARGO_BIN_EXE_vanta-cli` + `test_migrate_check` existente)
  - Precedencia discriminada: el E2E apunta `--db` a un path inexistente y exige exit 0 → si `--db` ganara, `cmd_migrate_check` falla en `path.exists()` (`src/cli_handlers/migrate.rs:87-93`) ✅
  - `cargo fmt --all -- --check` → exit 0 ✅ · `cargo clippy -p vantadb --all-targets --target-dir target/find237 -- -D warnings` → exit 0 ✅
  - `target/find237/debug/vanta-cli.exe migrate --help` → muestra los 3 ejemplos (`migrate check --db ./my-db  # global --db fallback`) ✅
  - RED pre-fix: `target/debug/vanta-cli.exe migrate check --db ./some-db` (binario pre-fix, mtime 2026-10-03) → EXIT=2 + `error: the following required arguments were not provided: <TARGET>` — coincide exacto con la evidencia del autor ✅
- **Checklist anti-hábitos tóxicos:** ✅ sin salidas inventadas (todas reproducidas) · ✅ clarificación resuelta en spec §3 (Gate D justificado) · ✅ en ronda 1 Steps 2/3 estaban PENDING (sin done declarado sin verificar); luego COMPLETED con evidencia GREEN · ✅ sin fallos ignorados · ✅ pasos conectados al objetivo · ✅ SDP cubre el dominio (bugfix/TDD/git/task-system/docs) · ✅ paradas explícitas (stop conditions del plan)
- **Veredicto:** ✅ APPROVE (ronda 2 — mismo revisor, contexto fresco). Ronda 1: ❌ cambios requeridos, solo registro (comando 2 sin `--ignore-default-filter`; reproducido exit 4 / 0 tests). Cambios aplicados y verificados por lectura en ronda 2 (sin cambios de código → suite no re-ejecutada, por diseño): §Contrato §Verificación exacta (comandos 1-3 con flag + nota `--target-dir`; resultados 11/11 · 3/3 · fmt/clippy) ✓ · §Invariantes (comando con flag) ✓ · Steps 2/3 ✅ COMPLETED con evidencia GREEN real (incl. fix `legacy_derive_helpers` → derive-first, consistente con `src/cli.rs:422-423`) ✓ · Context Save Point sincronizado ✓ · contadores downhill 1 / 75% ✓. `git diff HEAD` de los 3 archivos de código sin cambios vs ronda 1 (20/82/72 · 158+/16-) ✓.
  - **Nits (no bloqueantes, sincronizar en la edición de Step 4):** `docs/dev/tasks/FIND-237.md:120,132` conservan el texto de Verify pre-descubrimiento sin `--ignore-default-filter` (los §canónicos ya están correctos); `FIND-237.md:20` (header Metadata) aún dice "4 steps" vs §P2-03 "1" — alinear al cerrar.

## Notas
- El pre-mortem del plan cubre los 3 riesgos: (1) regresión de invocación → positional-first + test de ambos caminos; (2) ambigüedad target/db → resolución única en dispatch; (3) help stale → after_help con ejemplos.
- `--db` sigue honrando `VANTADB_STORAGE_PATH` (env) y su default `./db` — el fallback hereda esa semántica sin código nuevo.
- `NOTICED BUT NOT TOUCHING:` los doc comments de `cmd_migrate*` (handlers) mencionan `target_path` pero no el fallback — el fallback es responsabilidad del dispatch; sin cambio.

## Context Save Point

- **Última acción:** Steps 1-3 ✅ (RED capturado; GREEN: 11/11 + 5/5 + 3/3 + 94/94, fmt/clippy exit 0, campaign_verify_cmd passed). Review P2-01 ronda 1: ❌ cambios requeridos SOLO de registro → aplicados (comando 2 con `--ignore-default-filter`, Steps 2/3 ✅, esta sección).
- **Próximo step:** Step 4 — commit local `fix(cli): FIND-237 ...` + recitation + `skill progreso`.
- **Archivos en vuelo:** `src/cli.rs`, `src/bin/vanta-cli.rs`, `tests/cli_tests.rs`, `docs/dev/tasks/FIND-237.md` (los 3 primeros a commitear; nada más del working tree compartido).

## RESULTADO §7

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4
PROXIMO_STEP: ninguno
COMMIT_HASH: ba9f38c4 (fix(cli): FIND-237 — migrate acepta el global --db como fallback del target; commit local, sin push)
ARCHIVOS: src/cli.rs, src/bin/vanta-cli.rs, tests/cli_tests.rs, docs/dev/tasks/FIND-237.md
VERIFY_CONTRATO: pasa (nextest lib migrate 11/11 · E2E binario 3/3 · cli_tests 94/94 · fmt/clippy exit 0)
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no | P:plan-fija-fix · D:sin-símbolos-nuevos · V:sin-fallas · C:sin-colaterales
SKILLS_CARGADAS: source-driven-development, test-driven-development, git-workflow-and-versioning, incremental-implementation, systematic-debugging, documentation-skill (+ base auto: campaign-executor, progreso, ponytail)
```
