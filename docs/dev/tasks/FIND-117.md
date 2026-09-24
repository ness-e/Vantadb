# Task FIND-117 — TUI REPL abre mutantes (mismo bug FIND-101, fix distinto)

> **Plan:** `docs/dev/plans/2026-09-18-cierre-mvp.md` (Task 8, Wave2 segunda en secuencia)
> **Campaign:** c578fd8c-1bee-45a8-b036-2f7b00262828 · **Branch:** develop · **Appetite:** 1d · **Esfuerzo:** 🟡
> **Commit:** `fix: FIND-117 — ...` · **NextTask:** SHOW-05 (la ejecuta el orquestador, no esta ejecución)
> **Estado:** ⬜ PENDING → DISCOVERY completo 2026-09-18 (este file es el entregable de DISCOVERY)

## 1. TAREA — objetivo + contrato + acceptance criteria

**Objetivo:** el TUI REPL (`src/tui/repl.rs:99-100`) ejecuta IQL vía `self.engine` long-lived
abierto **read-only** (`src/bin/vanta-cli.rs:270` — `open_database(&args.db, true)`).
INSERT/UPDATE/DELETE fallan igual que FIND-101 + sin flush. El fix DIFIERE del CLI: no se
puede "abrir rw una vez" sin bloquear al resto (lock exclusiva vs compartida).

**Contrato (del plan, ley):** INSERT→SELECT visible en la misma sesión TUI
(**o** límite documentado en el help del REPL si el diseño lo exige con motivo)
+ test del path + sin regresión en reads/stats + clippy 0.

**Decisión de diseño (DISCOVERY con evidencia, delegada a FIND-117 por el plan):**
rama **límite-documentado adaptativo**:
- (E1) El engine RO rechaza escrituras en `ensure_writable` (`src/storage/engine/stats.rs:27-29`;
  test existente `src/storage/engine/tests/stats.rs:350-357`).
- (E2) Un handle RW acotado por query **mientras el RO vive es imposible**: fs2 shared-vs-exclusive
  sobre `.vanta.lock` (`src/storage/engine/init.rs:199-248` → `DatabaseBusy` tras 1000 ms).
  Medido, no asumido (Step 2).
- (E3) Abrir el TUI RW de entrada (1 línea) es viable pero **cambia el producto**: lock exclusiva
  toda la sesión bloquea hasta a los lectores (`init.rs:234-239`) y voltea el `true` deliberado
  de `vanta-cli.rs:270` — una regresión multi-terminal, no un fix (ponytail: no cambiar un bug
  por una regresión).
- (E4) Swap por query (cerrar RO → abrir RW → exec → cerrar → reabrir RO) churnea el estado TUI
  (`ReplState.engine: Arc` compartido con dashboard/monitor en `src/tui/mod.rs:39-43,81-83`) y deja
  dos handles vivos sobre el mismo backend en el proceso → roza **rediseño TUI** (Stop del plan:
  rediseño → DEFER con diagnóstico, **no rediseñar**).
- Por tanto el diseño exige el límite documentado **con motivo**, adaptativo: si el engine fuese
  writable (mismo handle único), los mutantes corren + flush estilo FIND-101; en RO se reporta el
  límite en vez del error crudo. Cero símbolos públicos nuevos → **Gate D no dispara**.

## 2. ARCHIVOS

**Clave (línea exacta):**
- `src/tui/repl.rs:99-100` — `Executor::new(&self.engine)` + `execute_hybrid` (el bug vive aquí)
- `src/tui/repl.rs:69-80` — `.help` (rama límite-documentado)
- `src/tui/repl.rs:129-133` — brazo `Write` (falta flush FIND-101)
- `src/bin/vanta-cli.rs:270` — `open_database(&args.db, true)` (RO deliberado, NO tocar)
- `src/cli_handlers/data.rs:212-230` — patrón parse-then-open FIND-101 (`de3bd119`, reusar en espíritu)
- `src/cli_handlers/data.rs:326-338` — `query_is_mutating` (privado → `pub(crate)`, 1 línea; NO copiar)

**Relacionados (codegraph_explore + trace_path/grep):**
- `src/storage/engine/stats.rs:16-29` (`guard_write_allowed`/`ensure_writable` — E1)
- `src/storage/engine/init.rs:167-251` (fs2 shared/exclusive + `DatabaseBusy` — E2/E3)
- `src/storage/engine/mod.rs:325` (`pub read_only: bool` — el REPL lo lee, sin getter nuevo)
- `src/storage/engine/maintenance.rs:35-36` (`flush` exige writable — el flush solo corre en Write)
- `src/executor.rs:155-190,244-272` (`execute_hybrid`/`execute_insert` → `Write`)
- `src/tui/mod.rs:32-43` (`run_tui`, `Arc` compartido — E4)
- `tests/cli_tests.rs:480-542` (`test_find101_query_insert_mutates` — patrón de test a seguir)
- Cero tests TUI hoy (grep `ReplState` solo en `tui/`): el test del path es nuevo.

**Prohibidos:** rediseño TUI · `src/wal*.rs` (FIND-109) · `vanta-memory/` · `vantadb-mcp/`
(IMPL-112 ✅) · `examples/` (FIND-116 ✅) · `.opencode/skills/` (FIND-119 ✅) ·
`reparacion.bat`, `.opencode`, `Justfile`, `ocr-*`, `completions/*`,
`desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4, `docs/dev/Backlog.md`, plan file (solo
recitation del orquestador), `C:/Users/Eros/.vantadb*` (datos vivos).

## 3. DEPENDENCIAS

Wave2 segunda en secuencia (IMPL-112-S2 ✅ sin fricción; SHOW-05 después, orquestador).
Sin bloqueantes. **Stop del plan:** rediseño TUI necesario → DEFER con diagnóstico (no rediseñar).
No se alcanzó (E4 lo evita por diseño).

## 4. REFERENCIAS (lectura completa antes de codificar — hecha en DISCOVERY)

- Rules: `.opencode/rules/core-engine.md` (R-3 `?` sin unwrap · R-5 env — el fix no añade env vars)
- Refs: `clean-code-clean-architecture.md` Ap. V (TUI = Frameworks/Drivers → Humble Object;
  V.4 🟡 fn >20 líneas preexistente en `execute()`, no agravar sin scope-creep),
  `definition-of-done.md` (v1 + comandos), `dev-tools.md`, `test-suite.md`
- Commands: `pipeline.md`, `audit.md`
- SPEC.md raíz § Alcance cierre-mvp: "mutantes IQL en sesión **o límite documentado con motivo**
  (decide FIND-117, no re-diseñar el TUI)".
- **Tabla Spec:** N/A — cero símbolos públicos nuevos (cambio privado + `pub(crate)` + strings + tests).

## 5. SKILLS (SDP Paso 0b real — `campaign_discover_skills_v2` phase=BUILD)

`SDP: systematic-debugging, test-driven-development, incremental-implementation,
context-engineering, source-driven-development, doubt-driven-development, codebase-memory`
- **systematic-debugging** — root-cause lock-semantics antes de proponer fix (E1–E4).
- **test-driven-development** — RED (límite documentado) → GREEN → suite sin regresión.
- **incremental-implementation** — 3 steps ≤~100 líneas, repo compilable tras c/u.
- **context-engineering** — Context Pack por slice (rules → spec → source → error).
- **source-driven-development** — fs2/flock lock-semantics verificadas en código, no asumidas.
- **doubt-driven-development** — decisión A-vs-B adversarial (E3: RW-upfront = regresión).
- **codebase-memory** — blast radius + mantenimiento de grafos (cargada; CodeGraph usado primero).
- Descartadas del SDP con motivo: `frontend-ui-engineering` (solo `web/`, el TUI es ratatui),
  `api-and-interface-design` (cero API nueva — re-evaluar si apareciera `pub`), `campaign-executor`/
  `progreso` (auto-cargadas vía MCP). `progreso` Trigger 1 **no se ejecuta**: Backlog/avance/plan
  son del orquestador (SPEC Boundaries + §2 prohibidos).

## 6. HERRAMIENTAS + MCP

- `codegraph_explore` PRIMERO ✅ (REPL → engine → open → lock) · grep/Read para detalle.
- `cargo test -p vantadb --features tui -j 2 tui` (scope REPL; `tui` no está en default features:
  `Cargo.toml:146` `tui = ["dep:ratatui","dep:crossterm","cli"]`).
- `cargo test -p vantadb --features tui,cli -j 2 --test cli_tests` (no-regresión FIND-101).
- `cargo clippy -p vantadb --all-targets --features tui -j 2 -- -D warnings` (clippy 0 incl. tui).
- `cargo fmt --check` · `git diff --check`.
- `campaign_verify_cmd`: bug exit -1 conocido → **bash directa + mención** en RESULTADO.
- Cargo siempre `-j 2`. Internet N/A (código local suficiente; fs2/flock leídos en `init.rs`).
- Notion Paso 0c: sin tool fetch/Notion en este runner → N/A con motivo (la tarea no mapea a
  Problema/Propuesta/features — es bug UI shippada, verificado en código).

## 7. INVESTIGACIÓN CÓDIGO — blast radius (DISCOVERY)

REPL (`repl.rs:52-149` `execute`) → `Executor::execute_hybrid` → `insert/update/delete`
(`executor.rs:244-272` + `storage/engine/insert.rs:185`/`delete.rs:40` `ensure_writable`) → RO
rechaza con `Validation{read_only}` (E1). `flush` (`maintenance.rs:35`) exige writable: en RO el
brazo `Write` es inalcanzable, en RW flushea (FIND-101). Lock: un solo handle por proceso en TUI
(`mod.rs:39`); segundo handle RW choca en `.vanta.lock` (E2). Callers de `execute()`: solo
`handle_repl_key` Enter (`repl.rs:165-173`). `query_is_mutating`: 1 caller (`cmd_query`). Riesgo:
bajo — 2 files + tests, sin hot path, sin API pública, sin concurrencia nueva (Regla 8 N/A).

## 8. INVESTIGACIÓN PROBLEMA — lock-semantics (incógnita uphill, cerrada con evidencia)

¿Handle de escritura acotado por query o límite documentado? **Medición (Step 2)** en vez de
asumir: test tempdir fjall con RO vivo + intento RW → se espera `DatabaseBusy` (~1 s por
`file_lock_timeout_ms:1000` en `config.rs:337`). Si mide busy → rama límite-documentado;
si abre → rama handle-acotado. Decisión preliminar: límite (E2+E3+E4); el test la convierte en
evidencia ejecutable (si el locking cambia algún día, el test avisa y se re-evalúa).

## 9. INVESTIGACIÓN INTERNET

N/A — código local suficiente; sin red → nada que marcar (TSYS-13 sin citas).

## 10. VALIDACIÓN + CIERRE

Verify contrato (límite visible en `.help` + test path + reads/stats verdes + clippy 0) → full
(fmt/clippy/nextest scopes §6) → OCR delegation (`pwsh dev-tools/ocr-review.ps1`;
Critical/High = bloquea) → DoD 3 niveles (standing + v1 determinista + shippable a-e sin deuda
silenciosa) → commit `fix:` staging SELECTIVO (solo `src/tui/repl.rs`,
`src/cli_handlers/data.rs`, tests) → NO PUSH (solo vanta-lead) → RESULTADO §7.
P2-01 y recitation del plan: orquestador. Gates D/V/C: sin tool `question` en este runner;
evaluados abajo con motivo (D: no dispara — sin símbolos públicos; V: solo si 2 fallas
mismo-error; C: colaterales vía findings si aparecen).

## STEPS (atómicos, ~100 líneas, cada uno PLAN → ACT → VERIFY)

- [x] **Step 1 — RED + medición lock (test-only, 0 líneas prod):** 4 tests en `src/tui/repl.rs`
  (2 RED que fallaron por razón correcta — error crudo `Validation read_only` + help sin documentar;
  caracterización reads/stats + medición `DatabaseBusy` que pasaron).
  Verify: `cargo test -p vantadb --features tui -j 2 tui:: --lib` → 2 FAIL(previsto)/2 PASS.
- [x] **Step 2 — GREEN mínimo (`src/tui/repl.rs` + 1 línea `data.rs`):** `query_is_mutating` →
  `pub(crate)`; `execute()` clasifica (parse-then-message) + `run_iql` extraído con `flush()` tras
  `Write` + `.help` documenta el límite. Sin `unwrap` prod, sin `unsafe`, deuda neta ≤ 0
  (`execute()` se acortó por extracción).
  Verify: 4/4 verde + `cargo fmt --check` + clippy tui 0.
- [x] **Step 3 — VERIFY full + cierre:** +1 test round-trip RW (INSERT→SELECT misma sesión, 5/5);
  lib 2099 ✅ + cli_tests 85 ✅ (FIND-101 intacto); `git diff --check` ✅; OCR sin Critical/High
  (revisión cognitiva § rule `**/*.rs`; único Low: doble-parseo por Enter, despreciable en REPL);
  DoD 3 niveles ✅ → commit `fix:` selectivo → RESULTADO.

## Impacto mapeado (Regla 0)

- **Leídos completos:** `src/tui/repl.rs` (327L), `src/tui/mod.rs` (90L),
  `src/cli_handlers/data.rs:205-338`, `src/bin/vanta-cli.rs:230-276`,
  `src/storage/engine/init.rs:155-293`, `src/storage/engine/stats.rs:13-29`,
  `src/executor.rs:153-272`, `src/cli_handlers/db.rs`, SPEC.md § cierre-mvp, Backlog FIND-117.
- **Referencias hacia dentro (lo que el cambio usa):** `Executor::execute_hybrid`,
  `engine.read_only` (`storage/engine/mod.rs:325` pub), `engine.flush()`,
  `query_is_mutating` (se hace `pub(crate)`), `engine.stats()`.
- **Referencias entrantes (quién usa lo cambiado):** `execute()` ← `handle_repl_key` Enter;
  `query_is_mutating` ← `cmd_query`; `.help` ← solo UI. Ningún otro caller.
- **Veredicto:** impacto BAJO y acotado — 2 files prod + tests; sin API pública, sin hot path,
  sin hilos nuevos, sin features nuevas. `vanta-cli.rs:270` NO se toca.

## Context Save Point

Sesión vanta-worker FIND-117 (Wave2). DISCOVERY cerrado: decisión límite-documentado (E1–E4) +
este file. Reanudar: próximo step ⬜ con su comando §6. Recitation MCP al día (`in-progress`).
Orquestador: P2-01 + recitation del plan + Backlog/avance + push (lead).
