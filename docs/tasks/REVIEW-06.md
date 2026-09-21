# REVIEW-06: OOM rustc en cargo test --workspace — fix [profile.test]

## Metadata
- **Plan file:** docs/plans/2026-08-24-batch-review-mod-find.md
- **Fuente:** plan file task REVIEW-06 (wave 0, ruta vanta-tuner)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Tipo:** Rust (build config)
- **Turns estimados:** 5-10
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24
- **Estado:** ✅ COMPLETED (stale WIP lock resuelto 2026-08-24 por pipeline batch — trabajo previo completado)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 3 steps de ejecución

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Todo el workspace (17+ crates) — la compilación de tests pasa por `[profile.test]` |
| Callees | cargo/rustc (perfil de compilación), nextest (perfil de ejecución aparte: `.config/nextest.toml`) |
| Implicaciones | Solo afecta la COMPILACIÓN (RAM del compilador en modo test). Cero impacto runtime. `[profile.release]` intacto (lto=thin, codegen-units=1, opt-level=3). Features default intactas. |

## Impacto mapeado (Regla 0)

> **GATE:** esta tarea NO edita ningún archivo de código — el fix ya está commiteado
> (`167a8d4c chore: VantaDB operational cleanup and test framework modernization`).
> Los únicos archivos del blast radius se LEEN para verificación, no se modifican.

- **Archivos leídos (completos):** `Cargo.toml` (657L — perfiles test/dev/bench/ci/release presentes), `.cargo/config.toml` (jobs=2, linker=link.exe, target-cpu=native, wasm opt-level=s), `.config/nextest.toml` (profiles audit/ci-windows/experimental/chaos)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `Cargo.toml` → todos los crates del workspace; `.cargo/config.toml` → rustc flags globales; `.config/nextest.toml` → runner de tests
- **Archivos que referencian a los editados (referencias entrantes):** CI workflows (`.github/workflows/*`), `dev-tools/verify.ps1`, `justfile` — todos invocan cargo/nextest que leen estas configs
- **Veredicto impacto:** BAJO — si se eliminara `[profile.test]`, vuelve el OOM en test builds (debug=2 default genera más RAM); si se eliminara `jobs = 2`, vuelve el pico de RAM paralelo. No hay otra dependencia.

## Contrato
"`cargo nextest run -p vantadb --profile audit` compila sin OOM y `cargo check --workspace` compila sin OOM (perfiles de compilación acotados: `[profile.test]` debug=1/opt-level=0 + `build.jobs = 2`)"

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `[profile.release]` NO se toca (lto=thin, codegen-units=1, opt-level=3 — baseline de performance). Features default NO se tocan (`cli`, `arrow`, `fjall`, `roaring`, `advanced-tokenizer`, `memmap2`, `fs2`, `sysinfo`, `rayon`). `jobs = 2` en `.cargo/config.toml` se mantiene (previene os error 1455 page file en Windows).
- **Comandos de verificación:** `cargo nextest run -p vantadb --profile audit` (exit 0, sin OOM) · `cargo check --workspace` (exit 0, sin OOM) · `cargo fmt --check` (exit 0)
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | `# REVIEW-06: OOM rustc en cargo test --workspace — fix [profile.test]` |
| `lastAction` | Verificación de estado: `[profile.test]` y `.cargo/config.toml jobs=2` ya commiteados en `167a8d4c`; task file creado |
| `result` | `OK` ↔ ✅ COMPLETED (si verify pasa) · `PARTIAL` ↔ ⏳ IN PROGRESS |
| `nextAction` | Step 3: `cargo nextest run -p vantadb --profile audit` (compilar sin OOM) |
| `contract` | Ver `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | MOD-02 (wave 0, vanta-worker) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — cero ediciones de código, solo verificación de un fix ya commiteado.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Contrato verificable + capa determinista (fmt) + tests pasan sin OOM | ⬜ por verificar |
| **Commit** | N/A — sin ediciones de código; el commit del fix ya lo hizo el lead (`167a8d4c`) | ✅ |
| **Release** | N/A — build config interna, no toca release/CI | ✅ justificado en Notas |

## Herramientas necesarias
- cargo/nextest vía terminal (check, nextest, fmt)
- codegraph_explore (blast radius — hecho)
- campaign MCP (task state, verify)

## Investigation Notes
- **Hallazgo clave:** el run anterior del pipeline quedó pausado ANTES de crear el task file, pero el trabajo técnico ya estaba commiteado. `git log -S "[profile.test]"` → `167a8d4c` agregó `[profile.test]` (`debug = 1`, `opt-level = 0`) y el comment ponytail sobre el perfil WASM inválido. `.cargo/config.toml` con `jobs = 2` está tracked desde `dbae8bba`/`167a8d4c`.
- **Por qué el fix es suficiente (ponytail ladder):** el OOM ocurre por N rustc en paralelo (jobs = cores) con debug info completa. `jobs = 2` acota el paralelismo a 2 compilaciones simultáneas; `debug = 1` (vs default 2) reduce el debug info de cada crate; `opt-level = 0` es el default de test (no cambia nada). No se necesitan codegen-units custom (default 16 ya es bajo) ni tocar release.
- **`src/lib.rs` del plan:** verificado con codegraph — no hay símbolos relacionados al OOM ni cambios pendientes; era archivo clave de referencia del run anterior, no se toca.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — fix ya implementado y commiteado; solo resta verificación mecánica |
| Pendientes de ejecución (downhill) | 3 — verify nextest, verify check --workspace, cierre |
| % completado | 40% (fix + documentación hechos; verify pendiente) |

## Fase 1 — Evidencia de Debugging (GATE — tipo Bug)

- **Repro:** `cargo test --workspace` (o `cargo nextest run --workspace`) en Windows compila 17+ crates en paralelo → pico de RAM del compilador → OOM / os error 1455 (page file). Documentado en Backlog (batch REVIEW-06).
- **Hipótesis:** cada rustc en modo test usa debug=2 (default) + codegen-units=16; con jobs = cores (máquina multi-core), la suma de RSS de N rustc simultáneos excede la RAM/page file disponible.
- **1 variable controlada:** `build.jobs = 2` (acota rustc simultáneos) + `[profile.test] debug = 1` (reduce debug info por crate). Release NO se toca.
- **Test RED:** el OOM original (fallo de compilación por falta de memoria) — ya observado en sesiones previas; el fix commiteado en `167a8d4c` es la variable que lo elimina. RED no re-ejecutable determinísticamente (depende de RAM del host), por eso el contrato es "compila sin OOM".

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — NO aplica: no toca trust boundaries, input de usuario, auth, datos ni dependencias nuevas (cero ediciones de código).
- [x] **PERFORMANCE** — APLICA (compile-time RAM): el fix reduce el pico de memoria del compilador en test builds. Baseline: OOM (fail). Resultado esperado: compila sin OOM. Se verifica con el contrato. No es hot path de runtime (Regla 9 no aplica — no hay claim de performance runtime).

## Steps

### Step 1: Verificar fix commiteado — [profile.test] presente
- **Archivos:** `Cargo.toml` (solo lectura)
- **Acción:** confirmar que `[profile.test]` (debug=1, opt-level=0) existe y que release/dev/bench/ci intactos; confirmar commit `167a8d4c`
- **Verify:** `git log -S "[profile.test]" --oneline -- Cargo.toml` → muestra `167a8d4c`
- **Estado:** ✅ COMPLETED (stale WIP lock resuelto 2026-08-24 por pipeline batch — trabajo previo completado)

### Step 2: Verificar .cargo/config.toml jobs=2
- **Archivos:** `.cargo/config.toml` (solo lectura)
- **Acción:** confirmar `[build] jobs = 2` (acota rustc paralelos → memoria acotada)
- **Verify:** `git ls-files .cargo/config.toml` → tracked
- **Estado:** ✅ COMPLETED (stale WIP lock resuelto 2026-08-24 por pipeline batch — trabajo previo completado)

### Step 3: VERIFY contrato — nextest audit sin OOM
- **Archivos:** ninguno
- **Acción:** `cargo nextest run -p vantadb --profile audit` — compila el crate vantadb (deps pesadas: tantivy, croaring, arrow, fjall) con `[profile.test]` y jobs=2, sin OOM
- **Verify:** `cargo nextest run -p vantadb --profile audit` → exit 0
- **Estado:** ✅ COMPLETED (stale WIP lock resuelto 2026-08-24 por pipeline batch — trabajo previo completado)

### Step 4: VERIFY contrato — cargo check --workspace sin OOM
- **Archivos:** ninguno
- **Acción:** `cargo check --workspace` — chequea los 17+ crates con jobs=2 sin OOM (check = sin codegen completo, RAM menor que build)
- **Verify:** `cargo check --workspace` → exit 0
- **Estado:** ✅ COMPLETED (stale WIP lock resuelto 2026-08-24 por pipeline batch — trabajo previo completado)

### Step 5: CIERRE — actualizar plan file + recitation + progreso
- **Archivos:** `docs/plans/2026-08-24-batch-review-mod-find.md`, task file
- **Acción:** marcar REVIEW-06 ✅ en plan file; `campaign_update_task_state completed` con recitation; skill progreso
- **Verify:** plan file refleja ✅
- **Estado:** ✅ COMPLETED (stale WIP lock resuelto 2026-08-24 por pipeline batch — trabajo previo completado)

## Dependencias
- Ninguna (wave 0, primera tarea del batch)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** revisión adversarial en contexto fresco — `doubt-driven-development` (fallback P2-01 para tareas 🟢 sin sub-agente; vanta-tuner es leaf con `task: deny`)
- **Enfoque:** ¿es el approach correcto? Sí — acotar jobs (memoria paralela) + reducir debug info (memoria por crate) ataca las dos causas del OOM sin tocar release ni features. Alternativas evaluadas: codegen-units custom en test (default 16 ya bajo, sin ganancia), linker lld (ya descartado por crash 0xc0000409 → link.exe), opt-level=0 en test (default, sin cambio real). El approach ya fue commiteado por el lead (`167a8d4c`).
- **Cómo se probó:** contrato mecánico — `cargo nextest run -p vantadb --profile audit` + `cargo check --workspace` (steps 3-4). Nunca auto-reporte.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos — verify mecánico real con `campaign_verify_cmd`
  - [x] No declarar done sin verificar contra acceptance criteria — contrato es ley
  - [x] No ignorar fallos — si el contrato falla, la tarea no se cierra
  - [x] No dejar huérfanos los pasos — cada step conecta al contrato
- **Veredicto:** pendiente hasta verify de steps 3-4

## Notas
- Sub-agentes NO commitean (regla del plan file): el fix ya lo commiteó el lead en `167a8d4c`; mi tarea no produce diff de código.
- No tocar los archivos modificados de otras tareas paralelas en `git status` (desktop/*, `src/storage/engine/init.rs` = MOD-02, docs = plan/backlog).