# HARD-05 — Entorno local blindado (regla `-p`, required-features, target dir)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 5 — HARD-05, F0)
- **Fuente:** master-roadmap Task 5 · incidente real 2026-09-26 (commit `f6c395ef`)
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🔴
- **Tipo:** Mixto (workspace config + docs harness + tests Rust)
- **Turns estimados:** 8
- **Creado:** 2026-09-26T19:58
- **last-synced:** 2026-09-27
- **Estado:** ✅ COMPLETED 2026-09-27
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps (4/4 ✅)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `dev-tools/verify.ps1` (fast gate, ya scoped `-p vantadb` L61-79) · `.githooks/pre-commit`/`pre-push` · CI `ci-rust.yml` (consume `default-members`/`[[test]]`) · todos los agentes leen `.opencode/AGENTS.md` |
| Callees | Workspace `default-members` (`.`, `vantadb-python`, `vanta-memory`, `vantadb-server`, `vantadb-mcp`) · `tests/cli_tests.rs` → `vantadb::cli_handlers::cmd_server` (`src/cli_handlers/server.rs:206`) → `cmd_server_http`/`cmd_server_mcp` · nextest profiles |
| Implicaciones | Sin cambio de API pública. Cambia: (a) normativa de agentes (Gate H), (b) superficie de test (targets skipeados explícitamente vs silenciosamente), (c) gate local ante la clase de fallo "feature unification → hang". No hay migración de datos ni impacto en serialización |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.opencode/AGENTS.md` (cargado en contexto de sesión) · `dev-tools/verify.ps1` (102 líneas) · `docs/dev/references/troubleshooting.md` (20 líneas — es índice) · `tests/cli_tests.rs:1490-1569` (gate `#[cfg(not(feature = "server"))]` en `test_server_missing_feature`, L1526-1543) · `Cargo.toml:340-673` (todas las entradas `[[test]]`) + `Cargo.toml:745-769` (`default-members`) · `.opencode/references/test-suite.md` (comandos)
- **Archivos referenciados hacia dentro:** `tests/cli_tests.rs` → `tests/common/mod.rs` (cfg `cli`/`sysinfo`) + `vantadb::cli_handlers`; `Cargo.toml` `[[test]]` → runner nextest/CI; `default-members` → unificación de features del workspace
- **Archivos que referencian a los editados (referencias entrantes):** hooks git invocan `dev-tools/verify*.ps1`; `ci-rust.yml` invoca el workspace; todos los agentes cargan `AGENTS.md` (normativo); `AGENTS.md` apunta a `references/test-suite.md`
- **Veredicto impacto:** **bajo-medio** — nada se rompe; el riesgo es (1) ocultar tests con `required-features` mal puesto y (2) wording rechazado por Gate H. Mitigaciones: conteo before/after por target + Gate H antes del commit

## Contrato

> "`AGENTS.md` prohíbe `cargo test`/`nextest` sin `-p` desde la raíz Y targets feature-gated tienen `required-features` (sin skip silencioso) Y suite unificada (`cargo test --test cli_tests`) completa sin hang Y `troubleshooting.md` documenta síntoma/causa/fix" (plan master, verbatim).

**Verificación mecánica del contrato:**
1. `rg -n "sin -p|NUNCA .*cargo (test|nextest)" .opencode/AGENTS.md` → regla presente.
2. `cargo metadata`/inspección: `stress_protocol` con `required-features = ["rayon"]`; `cargo test -p vantadb --test stress_protocol --no-run` OK; conteo `cargo nextest list -p vantadb` before == after (default features).
3. `cargo test -p vantadb --test cli_tests` → 88/88; repro unificado `cargo test --test cli_tests` (desde raíz, con timeout guard) completa sin hang.
4. `rg -n "8080" docs/dev/references/troubleshooting.md` → entrada con síntoma/causa/fix/comando exacto.

## Spec (SDD — sin símbolos públicos nuevos; decisiones registradas por evidencia)

> Phase 1b: NO es feature-add (no agrega `pub fn`/struct/tool/endpoint/binding). Igual se registran las decisiones técnicas:
> Gate P/D suprimidos: tarea F0 dentro de plan ya aprobado, sin superficie pública nueva (question-gates.md §Anti-abuso).

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Ubicación de la regla dura `-p` | A) `AGENTS.md` §Reglas + warning en §Test Suite (normativo, visible) / B) solo `references/test-suite.md` (no normativo, invisible en carga obligatoria) | A | ✅ decidido-por-evidencia (plan exige "regla dura" en AGENTS.md; AGENTS.md es el archivo normativo) |
| 2 | Alcance de `required-features` | A) solo targets 100% feature-gated (`stress_protocol` → `rayon`; L19 `#![cfg(feature = "rayon")]`) / B) agregar a targets con cualquier cfg interno (riesgo: ocultar tests válidos) | A | ✅ decidido-por-evidencia (ref: `Cargo.toml:469-471` sin `required-features`; gates no-feature quedan documentados como excepción) |
| 3 | Verificación de no-hang | A) correr el repro unificado con guarda de tiempo (job/timeout) / B) solo verificación canónica `-p` (no cubre el modo que produjo el hang) | A | ✅ decidido-por-evidencia (incidente solo ocurría en build unificado `f6c395ef`) |
| 4 | Entrada de troubleshooting | A) nueva sección al final de `troubleshooting.md` (sigue su instrucción "agregar síntoma al final de la sección correspondiente", archivo L13-14) / B) nota en `bug-workflow.md` (el plan pide troubleshooting.md) | A | ✅ decidido-por-evidencia (plan archivos clave) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** la verificación canónica `cargo test -p vantadb --test cli_tests` sigue en 88/88; ningún test desaparece silenciosamente (conteo nextest before/after idéntico con default features); `.opencode/` requiere Gate H verde antes del commit.
- **Comandos de verificación:** `cargo nextest list -p vantadb | Measure-Object -Line` (before/after) · `cargo test -p vantadb --test cli_tests` · `cargo test --test cli_tests` (raíz, timeout guard) · `pwsh dev-tools/verify_changed.ps1` · `/harness` (Gate H).
- **Deuda pendiente:** ninguna si el audit no encuentra más targets feature-gated; cualquier target no expresable con `required-features` (`#![cfg(debug_assertions)]`, `#![cfg(miri)]`) se documenta como excepción con motivo (no se fuerza).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# HARD-05: Entorno local blindado` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | HARD-06 (`docs/dev/tasks/HARD-06.md`) |

`contract` (sub-campos §12.3):
```
contract:
  verificacion: <comando EXACTO + resultado obtenido>
  evidencia:
    - claim: required-features aplicado sin ocultar tests
      evidencia: conteo nextest before/after + Cargo.toml diff
      confianza: alta
  artefactos: [docs/dev/tasks/HARD-05.md, Cargo.toml, .opencode/AGENTS.md, docs/dev/references/troubleshooting.md]
  invariantes: cli_tests 88/88; conteo de tests sin regresión
  deuda: ninguna / excepciones documentadas
  queda_pendiente: Gate H (/harness vía vanta-harness) antes del commit
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (neto negativo — paga el riesgo vivo R2 del plan: la clase de fallo "invocación ad-hoc sin `-p` → hang :8080").

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable (4 condiciones) + `cargo check -p vantadb` + `cargo test -p vantadb --test cli_tests` 88/88 + repro unificado sin hang |
| **Commit** | 1 commit local atómico, conventional (`chore(tooling): HARD-05 — ...`), diff limpio, Gate H verde, sin push (Regla 7) |
| **Release** | N/A — sin cambio de producto; justificado: verificación = `verify_changed.ps1` + Gate H (harness). No aplica changelog/semver |

## Herramientas necesarias
- Terminal (cargo/nextest), grep/read
- `codegraph_explore` (blast radius `cmd_server`)
- campaign MCP (`campaign_update_task_state`, `campaign_verify_cmd`)
- `/harness` (Gate H — vía `vanta-harness`, leaf distinto del implementador)

**Skills cargadas (SDP v3, pre-run 2026-09-27):** `campaign-executor` · `progreso` · `doubt-driven-development` · `test-driven-development` · `systematic-debugging` · `writing-guidelines` · `writing-plans` · `incremental-implementation` · **PINNED (policy):** `doubt-driven-development`, `test-driven-development`, `systematic-debugging`
`SDP: campaign-executor, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, writing-guidelines`

## Investigation Notes
- **Incidente 2026-09-26 (commit `f6c395ef`):** `cargo test` desde la raíz sin `-p` → `default-members` unifican features (`vantadb-server` pide `vantadb/server`) → `test_server_missing_feature` llamó `cmd_server(http=true)` → arrancó el server HTTP real y colgó infinito en `:8080`. Mitigación aplicada: `#[cfg(not(feature = "server"))]` en `tests/cli_tests.rs:1527` (verify canónico 88/88; suite unificada 87/87 → 1 test skip silencioso que esta task documenta/blinda).
- **verify.ps1 ya scoped:** usa `-p vantadb` en check/clippy/nextest/coverage (L61-79) — solo verificar, no editar.
- **`references/test-suite.md` L7 documenta `cargo nextest run --profile audit --workspace --build-jobs 2`** (certificación pesada) → anotar como tal para que no se use como comando de iteración.
- **codegraph:** `cmd_server` → `cmd_server_http`/`cmd_server_mcp` (1 caller c/u, sin tests de cobertura); `Cli` (`src/cli.rs:13`) 1 caller (`server.rs`).
- **Audit de gates (Cargo.toml L363-667):** con `required-features` hoy: `chaos_integrity`(failpoints), `columnar`(arrow), `mmap_hnsw`(cli), `hybrid_ranking_metrics`(cli), `request_id`/`server_auth_rotation`/`rbac_namespace`(server), `cli_tests`/`file_locking_stress`/`prefetch_benchmark`/`benchmark_datasets`(cli). Sin `required-features` y con gate whole-file por feature: **`stress_protocol`** (`#![cfg(feature = "rayon")]`). Gates no expresables por Cargo (documentar, no forzar): `derived_index_recovery`/`text_index_recovery` (`debug_assertions`), `miri_unsafe` (`miri`), negativo `server` en `cli_tests`.
- **troubleshooting.md es un índice de 20 líneas** (no tiene secciones de síntomas aún) → crear sección al final con formato síntoma/causa/fix/comando.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — incidente y fixes ya investigados (plan Task 5) |
| Pendientes de ejecución (downhill) | 0 steps (4/4 ✅) |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluado: no aplica. Cambios son config de workspace/docs/normativa; no toca trust boundaries, input de usuario, auth ni dependencias.
- [x] **PERFORMANCE** — evaluado: no aplica a hot path de producto. Impacto colateral positivo: evita builds unificados accidentales (tiempo de iteración). Sin benchmark de producto requerido (Regla 9 no aplica — no es optimización de runtime).

## Steps

### Step 1: Regla dura `-p` (AGENTS.md + test-suite.md)
- **Archivos:** `.opencode/AGENTS.md`, `.opencode/references/test-suite.md`
- **Acción:** en `AGENTS.md` (§Reglas — junto a la pirámide de gates de Regla 1 + warning en §Test Suite) agregar: "NUNCA `cargo test`/`cargo nextest` sin `-p <crate>` desde la raíz del workspace; el build unificado de `default-members` unifica features en el `target/` compartido y puede colgar (incidente `f6c395ef`, `:8080`)"; en `test-suite.md` anotar que `--workspace` es SOLO certificación pesada y agregar los comandos canónicos scoped.
- **Verify:** `rg -n "sin -p|NUNCA" .opencode/AGENTS.md` → regla presente; `rg -n "workspace|pesada" .opencode/references/test-suite.md` → anotación presente.
- **Estado:** ✅ DONE 2026-09-27 — regla en Regla 1 (L484) + warning §Test Suite (L430) + coherencia de 3 menciones bare (L163/L487/L641); test-suite.md scoped + `--workspace`=Heavy. Verify: `rg "NUNCA .*cargo (test|nextest)" AGENTS.md` OK (L430/L484); `rg "workspace|pesada" test-suite.md` OK (L4/L18/L23). Nota: el comando combinado del contrato con el segmento `sin -p` no spawnea vía MCP (shell del server); el mismo comando vía pwsh local → exit 0 con ambas líneas.

### Step 2: required-features (audit + Cargo.toml)
- **Archivos:** `Cargo.toml`, (evidencia) `tests/certification/stress_protocol.rs:19`
- **Acción:** contar targets antes (`cargo nextest list -p vantadb | Measure-Object -Line`); agregar `required-features = ["rayon"]` a `[[test]] stress_protocol` (L469-471); documentar en el commit/Notas las excepciones no expresables (`debug_assertions` en `derived_index_recovery`/`text_index_recovery`, `miri` en `miri_unsafe`, gate negativo `server` en `cli_tests`).
- **Verify:** `cargo nextest list -p vantadb | Measure-Object -Line` == conteo previo; `cargo test -p vantadb --test stress_protocol --no-run` exit 0; `cargo check -p vantadb`.
- **Estado:** ✅ DONE 2026-09-27 — audit completo: único target feature-gated sin required-features era `stress_protocol` (`#![cfg(feature = "rayon")]`; `rayon` es default → no oculta tests). Excepciones no expresables documentadas (ver Notas). Verify: nextest list before=2297 / after=2297 (--target-dir target/session-api01); `--no-run` exit 0; `cargo check -p vantadb` exit 0.

### Step 3: Entrada en troubleshooting.md
- **Archivos:** `docs/dev/references/troubleshooting.md`
- **Acción:** agregar sección (al final, formato del archivo L13-14): síntoma → `cargo test` desde raíz cuelga infinito en `:8080`; causa raíz → unificación de features por `default-members` (`vantadb-server` → `vantadb/server`); solución → usar `-p vantadb` + `#[cfg(not(feature = "server"))]` + `required-features`; comando exacto de repro y de verificación.
- **Verify:** `rg -n "8080|unificaci[oó]n de features|f6c395ef" docs/dev/references/troubleshooting.md` → entrada completa; `npx markdownlint-cli2 docs/dev/references/troubleshooting.md` (si disponible) 0 errores nuevos.
- **Estado:** ✅ DONE 2026-09-27 — sección "Builds y tests locales" con síntoma/causa raíz/solución 3 capas/comandos (L22-43); frontmatter last_reviewed actualizado. Verify: rg OK (5 matches: L24/L26/L27/L30/L43); markdownlint-cli2 v0.23.3 → 0 issues, exit 0 (MCP: variante `-e "8080" -e "unificaci.n de features" -e "f6c395ef"` OK).

### Step 4: Verificación unificada + Gate H + commit
- **Archivos:** (sin edición) `dev-tools/verify.ps1` (verificar scoped)
- **Acción:** correr (a) canónico `cargo test -p vantadb --test cli_tests` → 88/88; (b) repro unificado `cargo test --test cli_tests` desde raíz con guarda de tiempo (p.ej. `Start-Job`/`Wait-Job -Timeout 900`; si la forma exacta difiere por semántica de cargo, documentar el repro fiel en troubleshooting.md); (c) `rg "cargo (test|nextest)" dev-tools/verify*.ps1` → todos con `-p`; (d) Gate H `/harness` verde (vanta-harness); commit local `chore(tooling): HARD-05 — ...` (sin push).
- **Verify:** outputs (a)-(d) pegados en la recitation; commit local creado.
- **Estado:** ✅ DONE 2026-09-27 — (a) ✅ `test result: ok. 88 passed; 0 failed` (canónico); (b) ✅ `test result: ok. 87 passed; 0 failed` SIN hang (guarda de tiempo = timeout 1800s del runner MCP; δ 88→87 demuestra `server` ON en el build unificado + gate `cfg` activo); (c) ✅ todas las invocaciones llevan `-p vantadb` (verify.ps1 L69/L70/L75/L77; verify_changed.ps1 L31/L32 — solo lectura, NO editado); extras: fmt ✅, docs-coverage ✅ 0 gaps, check-agents-refs ✅ 6 refs, OCR advisory ✅ (Cargo.toml + troubleshooting sin Critical/High); (d) ✅ Gate H ronda 2 ✅ (condicional a FIND-* del residual, ver §Review) + P2-01 ✅ approve. Extra: `verify_changed.ps1` **ALL 4 PASS** (fmt/check/clippy/docs-coverage; `CARGO_TARGET_DIR=target/session-api01`). Commit: lo hace el LEAD (este agente no commitea; push solo con instrucción del owner).

## Dependencias
- HARD-02 (plan master): coordina el mismo `dev-tools/verify.ps1`/gates — HARD-05 NO lo edita (solo verifica scoped); sin bloqueo mutuo. Ejecutable en cualquier orden.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED. **Gate H obligatorio (`.opencode/AGENTS.md` + `references/`): `/harness` verde vía `vanta-harness` antes del commit.**

- **Revisor:** `vanta-review` — P2-01, contexto fresco (sesión distinta del implementador). 2026-09-27.
- **Gate H (ronda 1):** ❌ cambios requeridos (`vanta-harness`, 2026-09-27): el wording "`--workspace` = solo Heavy Certification" contradecía los gates de cierre por tarea (`pipeline-full.md:156`, `iter-loop-tools.md:325`, `RULES.md:142`, `definition-of-done.md:118` usan `--workspace` como check de cierre/step). **Fix aplicado:** regla reescrita nombrando el tier de cierre ("verificaciones deliberadas de cierre/certificación: verify full de tarea/step, determinista por commit, Heavy Certification, CI") y actualizados `AGENTS.md` §Test Suite + `test-suite.md` L4/L18. Residual MEDIA del gate (menciones bare `cargo nextest` en `agents/vanta-lead.md:84,228,265`, `agents/vanta-worker.md:93,237,367`, `agents/vanta-engine.md:318`, `RULES.md:123,154`, `unified-review/SKILL.md:1096`): **fuera del blast radius declarado** — ruteado en RESULTADO §queda_pendiente para fila `FIND-*` del LEAD (no en Backlog: `validate_scope` lo marca fuera de scope para esta tarea). Ronda 2 pendiente.
- **Gate H (ronda 2):** ✅ approve (`vanta-harness`, mismo leaf re-invocado) — findings ALTA 1-4 resueltos: el wording nombra el tier de cierre ("verify full de tarea/step, camino determinista por commit, Heavy Certification, CI") y cubre `pipeline-full.md:156` (heredado en pipeline-run/plan), `iter-loop-tools.md:325`, `RULES.md:142`, `definition-of-done.md:118`, `sdp-v3.mjs:32`; findings 6-7 resueltos. **Condición del gate:** registrar el residual F5 (9 sitios bare `cargo nextest` sin `-p`) como fila `FIND-*` al cierre — **✅ creada: `FIND-164` en `docs/dev/Backlog.md`** (routing canónico del protocolo de findings; `validate_scope` la marcó advisory-fuera-de-scope, override justificado por findings.md + condición del gate). LOW cosmético `AGENTS.md:163` ("--workspace solo Heavy Certification" → "solo cierre/certificación") corregido en el mismo changeset.
- **Enfoque:** ¿la regla cierra la clase de fallo? ¿`required-features` no oculta tests? ¿troubleshooting.md permite reproducir el hang?
- **Cómo se probó:** spot-check mecánico (sin re-ejecutar suites pesadas, por instrucción de costo): diffs scoped root + `.opencode`; `rg` regla dura en AGENTS.md (L430/L484) y `-p` en verify.ps1 (L69/L70/L75/L77) + verify_changed.ps1 (L31/L32); `required-features` único target whole-file feature-gated sin él era `stress_protocol` (Cargo.toml:476; rayon ∈ default L150 → conteo 2297 no cambia); `#![cfg` en tests/ = 4 gates (rayon/2×debug_assertions/miri) → excepciones documentadas OK; cfg gate `cli_tests.rs:1526-1543` es el ÚNICO `cfg(not(feature="server"))` del archivo → δ 88→87 consistente con server ON; tests server-gated (`request_id`/`server_auth_rotation`/`rbac_namespace`) y `vantadb-server/tests` bindean `127.0.0.1:0` → sin vías de hang `:8080` equivalentes. NO re-ejecutado: suites 88/87, nextest count, Gate H.
- **Checklist anti-hábitos tóxicos** (contrato de comportamiento — el revisor verifica que el implementador NO haya incurrido en ninguno antes de aprobar; fuente §12 de `docs/Investigaciones/2026-08-10-agent-engineering/agent-02-task-execution.md`):
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas.
  > Verificado por spot-check: todos los claims baratos re-chequeados (líneas, gates, comandos) coinciden con el working tree; Step 4 honesto (IN PROGRESS, no "done").
- **Veredicto:** ✅ approve (changeset/approach) — Commit-level DoD sigue PENDIENTE por diseño: Gate H (`vanta-harness`) + commit del LEAD (Step 4d). Hallazgos: 0 Critical/Required; Optional: la regla `-p` es normativa, no mecánicamente enforced (no hay hook/lint que rechace invocación bare desde raíz; upgrade: guard en hook/CI si reaparece).

## Notas
- **Gate H** obligatorio antes del commit (cambio en `.opencode/`). Nota de la política master: cada tarea = 1 commit local; push solo con instrucción del owner.
- **"target dir":** el título del plan alude al `target/` compartido del workspace — la unificación de features ocurre en ese build; la regla `-p` es la defensa primaria (no hay cambio de `CARGO_TARGET_DIR` en esta task).
- **NOTICED BUT NOT TOUCHING:** `tests/derived_index_recovery.rs:1` y `tests/text_index_recovery.rs:3` (`#![cfg(debug_assertions)]`) y `tests/miri_unsafe.rs:1` (`#![cfg(miri)]`) compilan vacíos en corridas normales — no expresable vía `required-features`; documentados como excepción (§Step 2).
- `ponytail:` sin tooling nuevo — se usan los gates existentes (verify + `/harness`).
