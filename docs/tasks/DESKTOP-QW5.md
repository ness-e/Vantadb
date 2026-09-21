# TASK DESKTOP-QW5: Limpiar filas DAUD-01..09 stale del Backlog (H-13)

## Metadata
- **Plan file:** `docs/plans/2026-08-25-research-desktop-quickwins.md`
- **Creado:** 2026-08-27T03:00
- **last-synced:** 2026-08-27T03:45
- **Estado:** ✅ COMPLETED
- **Tipo detectado:** docs/process (documentation-and-adrs, spec-driven)
- **Workflow:** docs/process (verify → edit → verify → commit → close)
- **Task file:** `.opencode/skills/campaign-executor/tasks/DESKTOP-QW5.md`

## Blast Radius
- `docs/Backlog.md` — tabla P37 (9 filas DAUD-01..09) + Exec Summary línea P37 (conteo 9→0) + `last_reviewed`. 9 filas todas ✅ Hecho con commits `3c53d8b2`,`480935a7`,`b865c625` + DAUD-02 via `ad0f34b1` (QW4). Edición es DELETE de filas stale, sin lógica. No afecta código Rust/TS/desktop TSX.
- `docs/avance/activo/desktop.md` — registro de dominio donde vive el completado de Phase DESKTOP (Trigger 1 progreso). DAUD pertenece a desktop post-fix. Si no se registra → gap potencial en check-avance-coverage si DAUD se considera fuente.
- `docs/avance/historial/backlog-history.md` — historial de items removidos del Backlog (Trigger 1 para items sin completar, o registro de limpieza masiva). DAUD removido debe quedar trazado aquí si no va a activo/desktop.md.
- `scripts/check-avance-coverage.ps1` y `scripts/validate-docs-coverage.ps1` — gates mecánicos del contrato (0 gaps). Ambos hoy verde (1038/1038 y 0 gaps) — edición no debe romperlos. `validate-docs-coverage` valida SDK/config/error/CLI/MCP/Python docs, no DAUD → bajo riesgo.
- `docs/plans/2026-08-25-research-desktop-quickwins.md` — plan file Wave1 Task5. Tras cierre, agregar recitation DESKTOP-QW5.
- **Implicaciones:** cambio 100% docs, 0 código, 0 hot path, 0 concurrencia, 0 API pública, 0 trust boundary. Reversible por `git revert`. Blast radius 1 archivo crítico (Backlog.md) + 2 historial/dominio. Riesgo principal: dejar filas residuales o no actualizar Exec Summary → grep DAUD aún con hits. Mitigado por verify mecánico `grep DAUD` 0 hits.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos (antes de editar):**
  - `docs/Backlog.md` (completo, ~800 líneas; P37 líneas 517-532, Exec Summary línea 47, last_reviewed header línea 6)
  - `docs/plans/2026-08-25-research-desktop-quickwins.md` (78 líneas + 4 recitations QW1-4)
  - `docs/avance/activo/desktop.md` (294 líneas, `last_reviewed: 2026-08-07`, §DESKTOP-38 + UX-01, sin DAUD aún)
  - `docs/avance/historial/backlog-history.md` (216 líneas, §§ limpieza masiva 2026-08-07/25, sin DAUD aún)
  - `scripts/check-avance-coverage.ps1` (89 líneas, map fuente→destino, lógica IDs regex)
  - `scripts/validate-docs-coverage.ps1` (197 líneas, 6 checks SDK/config/error/CLI/python/mcp)
  - `.opencode/skills/campaign-executor/tasks/DESKTOP-QW4.md` (113 líneas, verify-only precedente)
  - `.opencode/skills/campaign-executor/tasks/DAUD-LIMPI.md` (54 líneas, fix D1-D11 ya commitado)
  - git history: `3c53d8b2` (DAUD-LIMPI fixes), `480935a7` (E2E-VISUAL), `b865c625` (DAUD neon/window), `ad0f34b1` (QW4 filterActive), `a7ed0d22` (quickwins batch)
  - `desktop/package.json` / `desktop/src/*` (spot check QW1-4)
- **Referencias hacia dentro (qué importa este archivo):**
  - `docs/Backlog.md` → importado por `skill progreso` (lee backlog, chequea WIP), por `docs/avance/*` (cross-ref), por CI docs. No importado por código Rust/TS. Es catálogo de tareas, no código.
  - `docs/avance/activo/desktop.md` → referenciado desde Backlog Exec Summary P12/P37 como registro de completado (ver sección P12). Fuente de verdad de dominio desktop para `check-avance-coverage` si ID está en fuentes.
  - `backlog-history.md` → referenciado desde Backlog header `Historial de verificación: docs/avance/historial/backlog-history.md` y desde limpieza 2026-08-07. Es historial de items removidos.
- **Referencias entrantes (qué depende de lo que cambio):**
  - `P37` → Exec Summary línea 47 cuenta 9 activas; tabla P37 9 filas. Si se borran sin actualizar Exec Summary → conteo inconsistente (9 vs 0). Ambos deben alinearse a 0.
  - `DAUD-01..09` → ningún código depende de estas filas (son backlog docs). Solo docs/avance y plan file las referenciarán tras cierre. `grep -r "DAUD-"` en `src/`/`desktop/src/` debe ser 0 hits post-cierre excepto historial/plan/task files (histórico permitido).
  - `check-avance-coverage.ps1` → escanea `docs/avance/historial/fuentes/*` + `docs/avance/historial/campanas/*` para IDs. DAUD no está en fuentes → no afecta cobertura. Verificado: `grep DAUD` en `docs/avance/historial/fuentes/` 0 hits pre-edit → coverage ya 1038/1038 sin DAUD.
  - `validate-docs-coverage.ps1` → no escanea Backlog ni DAUD → 0 impacto.
  - E2E `desktop/e2e/daud01-temas.spec.ts` y `flujo-critico.spec.ts` → tests que guardan DAUD-01 (E2E-VISUAL). No dependen de Backlog.md. Intactos.
  - Plan quickwins QW1-4 recitations → QW5 será siguiente, depende de QW4 completado (ya ✅).
- **Veredicto de impacto:** BAJO — docs-only, 1 tabla + 1 summary line + 2 archivos de historial. Sin código, sin hot path, sin security/performance gate. Riesgo: edición parcial (dejar filas). Mitigado por verify `Select-String DAUD` en Backlog = 0/only-history tras edit + scripts 0 gaps.

## Contrato
Filas DAUD-01..09 stale limpiadas del Backlog (commits ya aplicados 3c53d8b2,480935a7,b865c625; DAUD-02 resuelta por QW4 ad0f34b1; DAUD-08 stash recuperada por b865c625). Backlog sin DAUD stale; `scripts/check-avance-coverage.ps1` y `validate-docs-coverage.ps1` 0 gaps.

Verificación mecánica:
1. `git show --stat 3c53d8b2 && git show --stat 480935a7 && git show --stat b865c625` — commits existen y tocan desktop/* corrección DAUD
2. `git log --oneline --grep="DAUD"` — confirma DAUD fixes commitados
3. `Select-String -Path docs/Backlog.md -Pattern "DAUD-"` — 0 hits (Backlog sin DAUD stale)
4. `pwsh scripts/check-avance-coverage.ps1` — FINAL: 1038/1038 o superior, 0 solo-snapshot, OK; Detail sin DAUD huérfanos
5. `pwsh scripts/validate-docs-coverage.ps1` — ✅ 0 gaps
6. `cargo fmt --check` — verde (gate pre-commit)
7. Build/desktop no requerido (docs-only) pero `npm --prefix desktop run build` y `npm --prefix desktop test` verde si se valida wave (opcional)

## Herramientas
- Read (Backlog.md, plan, avance/desktop.md, backlog-history.md, scripts, git log)
- Grep / Select-String (DAUD en Backlog y avance)
- pwsh (check-avance-coverage.ps1, validate-docs-coverage.ps1, cargo fmt --check)
- git (log, show, diff, add, commit)
- Edit (Backlog.md, avance/desktop.md, backlog-history.md, plan file)
- campaign_memory_write, campaign_update_task_state (manual, hasTask false), campaign_diagnose_pipeline

## Skills
- campaign-executor, progreso, ponytail (base obligatoria)
- documentation-and-adrs (detectado por tipo docs/process)
- SDP discovery (lifecycle DEFINE/PLAN/BUILD/VERIFY): keywords `DAUD/Backlog/stale/historial/clean/docs/coverage` → grep SKILLS-MANIFEST: hits `documentation-and-adrs` ya base, `planning-and-task-breakdown` candidato pero task ya descompuesta en steps atómicos (no slice adicional), `progreso` ya base, `systematic-debugging` candidato solo si scripts fallan. **SDP: sin candidatos adicionales** (base + docs). Total cargadas 4. **SKILLS_CARGADAS: campaign-executor, progreso, ponytail, documentation-and-adrs**

## Spec
N/A — tarea de documentación/proceso (limpieza backlog). No agrega `pub fn` / tool / endpoint / binding / símbolo público. No es feature-add con símbolos nuevos. Gate spec-first no aplica (ver pipeline-full § SPEC: solo feature-add/lógica nueva requiere Spec llena). Contrato mecánico es ley.

## Steps

### Step 1: Auditoría DAUD — verificar commits aplicados + estado Hecho + DAUD-02/08 ✅ DONE
- **Archivos:** `docs/Backlog.md:517-532`, `docs/avance/activo/desktop.md`, `git log` (3c53d8b2,480935a7,b865c625,ad0f34b1,a7ed0d22), `git stash list`
- **Acción:** Confirmar que los 9 DAUD están en estado Hecho con commits citados y que no hay trabajo pendiente:
  - Grep Backlog DAUD: 9 filas con `✅ Hecho` + commits 480935a7,3c53d8b2,b865c625 + DAUD-02 Cerrada owner + DAUD-08 stash recuperada
  - `git show --stat 3c53d8b2` → 8 files (App.css, index.css, DESIGN_DECISIONS, WorkspaceShell, ProxyDashboard, etc.) — DAUD-03/04/05/07 ✅
  - `git show --stat 480935a7` → 11 files (e2e/daud01-temas, flujo-critico, helpers, serve.mjs, playwright.config) — DAUD-01 E2E-VISUAL guard ✅
  - `git show --stat b865c625` → 2 files (tauri.conf.json window 1280x800, Mark.tsx neon vars) — DAUD-06/11 ✅
  - `git show ad0f34b1 --stat` → WorkspaceShell filterActive (DAUD-02) ✅
  - `git stash list` original stash@{0} "WIP on develop: 06aa1a86" ya no existe (DAUD-08 diff 0, consumido por b865c625) — verificar que `git stash list` no contiene ese mensaje
  - Si audit falla → documentar gap y escalar (no editar Backlog si fix no aplicado)
- **Verify:** `git log --oneline b865c625^..b865c625` presente ✅ (2026-08-25 fix recover DAUD-11+06); `git log 3c53d8b2` presente ✅ (8 files DAUD-LIMPI); `git log 480935a7` presente ✅ (11 files E2E-VISUAL); `Select-String DAUD docs/Backlog.md` 10 hits pre-edit (1 P37 header + 9 filas) todos con `Hecho/Cerrada` ✅; `git stash list | Select-String "06aa1a86"` 0 hits ✅ — stash@{0} actual es `2fc26b26`, no `06aa1a86` (consumido)
- **Estado:** ✅ DONE — auditoría 2026-08-27 confirma 9/9 DAUD Hecho, commits existentes, DAUD-02 via QW4 ad0f34b1, DAUD-08 stash 06aa1a86 no existe (recuperado por b865c625)

### Step 2: Limpiar Backlog.md — eliminar 9 filas + colapsar P37 a Cerrada + Exec Summary 0 ✅ DONE
- **Archivos:** `docs/Backlog.md` (líneas 47 Exec Summary + 517-532 P37)
- **Acción:** Edición atómica en 2 lugares:
  1. Exec Summary línea 47: `| **P37** ... | 9 (DAUD-01..09; fixes D1-D11 ya aplicados...) | ~0.5 día ... | 🟠 Media (DAUD-09 commit 🔴) |` → `| **P37** ... | 0 — ✅ 9/9 ejecutadas (DAUD-01..09 — commits `3c53d8b2`,`480935a7`,`b865c625`; DAUD-02 via DESKTOP-QW4 `ad0f34b1`) | — | ✅ Cerrada 2026-08-26 |`
  2. P37 sección 517-532: eliminar tabla de 9 filas + header "Los fixes están SIN COMMITEAR..." (contexto stale) y reemplazar por: `> **✅ Cerrada 2026-08-26 — DESKTOP-QW5 (H-13):** 9/9 DAUD verificadas y archivadas. Fixes D1-D11 ya commitados: E2E-VISUAL `480935a7` (DAUD-01), DAUD-LIMPI `3c53d8b2` (DAUD-03/04/05/07) + `b865c625` (DAUD-06/11), filterActive `ad0f34b1` (DAUD-02/QW4). Stash `06aa1a86` consumido por `b865c625` (DAUD-08). Registro en `docs/avance/activo/desktop.md` §DAUD + `backlog-history.md`.` seguido de tabla vacía `| ID | Effort | Descripción | Archivos | Estado |` con fila `| — | — | *Sin tareas activas — ver historial* | — | ✅ |` o sin filas. Actualizar `last_reviewed: 2026-08-26` en header.
- **Verify:** `Select-String -Path docs/Backlog.md -Pattern "^\| \`DAUD-"` → 0 hits ✅ (ninguna fila DAUD-01..09); `Select-String -Path docs/Backlog.md -Pattern "P37"` → 0 — ✅ 9/9 ejecutadas + ✅ Cerrada 2026-08-26 ✅; `last_reviewed: 2026-08-26` ✅; `Total open items: 109` (118-9) ✅; `Historial ... último sweep 2026-08-26` ✅
- **Estado:** ✅ DONE — Backlog.md editado: Exec Summary P37 9→0 Cerrada, last_reviewed 2026-08-26, P37 sección colapsada a 1 fila `Sin tareas activas`, 9 filas DAUD eliminadas

### Step 3: Registrar cierre en dominio + historial (progreso Trigger 1) ✅ DONE
- **Archivos:** `docs/avance/activo/desktop.md` (append §DAUD), `docs/avance/historial/backlog-history.md` (append limpieza DAUD)
- **Acción:** 
  - En `active/desktop.md` agregar tras `### UX-01+UX-05` (línea 294): `### P37 — Auditoría diseño desktop post-fix (DAUD-01..09, H-13) — 9/9 ✅` con fecha 2026-08-26, resumen por DAUD con commits y evidencia (E2E guard, CSS scoped, utilities borradas, Pencil, convención, stash consumed, commits D1-D11). Incluir `Ids: DAUD-01..09`.
  - En `backlog-history.md` agregar tras `## Gran limpieza 2026-08-25` (línea ~216): `## Limpieza DAUD 2026-08-26 (DESKTOP-QW5, H-13)` con lista 9 DAUD + commits + motivo stale (fixes ya aplicados) + referencia a plan quickwins Wave1 Task5.
  - Ponytail: 2 archivos, ~40-60 líneas totales, sin duplicar todo Backlog. Solo lo necesario para trazabilidad.
- **Verify:** `Select-String DAUD docs/avance/activo/desktop.md` → 11 hits (P37 header + 9 DAUD IDs + refs) ✅; `Select-String DAUD docs/avance/historial/backlog-history.md` → 12 hits (limpieza + 9 rows) ✅; `last_reviewed: 2026-08-26` en desktop.md ✅; `pwsh scripts/check-avance-coverage.ps1 -Detail` → sin huérfanos (ver Step4)
- **Estado:** ✅ DONE — active/desktop.md +18 líneas P37 DAUD, last_reviewed 2026-08-26; backlog-history.md +22 líneas Limpieza DAUD

### Step 4: Cierre — verify full + commit + plan + progreso ✅ DONE
- **Archivos:** `docs/plans/2026-08-25-research-desktop-quickwins.md`, `docs/Backlog.md`, `docs/avance/*`, `.opencode/skills/campaign-executor/tasks/DESKTOP-QW5.md`
- **Acción:** Verify mecánico del contrato:
  1. `pwsh scripts/check-avance-coverage.ps1` → 0 gaps (o 1038+n/1038+n) ✅ 1038/1038 0 solo-snapshot 2026-08-27
  2. `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps ✅ 7 checks 0 gaps 2026-08-27
  3. `cargo fmt --check` → verde ✅ EXIT 0 2026-08-27
  4. `Select-String -Path docs/Backlog.md -Pattern "^\| \`DAUD-"` → 0 hits ✅ (Backlog sin filas stale; solo header P37 + historial refs)
  5. (Opcional wave) `npm --prefix desktop run build` verde + `npm --prefix desktop test` 69/69 verde — docs-only, skip (QW4 ya validó 14.63s/69/69; no se tocó desktop/src)
  - Si todo pasa: `git add docs/Backlog.md docs/avance/activo/desktop.md docs/avance/historial/backlog-history.md docs/plans/2026-08-25-research-desktop-quickwins.md .opencode/skills/campaign-executor/tasks/DESKTOP-QW5.md` + commit `docs(backlog): DESKTOP-QW5 — limpiar filas DAUD-01..09 stale (H-13) + historial desktop` (conventional `docs:`)
  - Actualizar plan file: agregar `=== RECITATION DESKTOP-QW5 ===` con Objetivo/Estado/Última acción/Resultado/Próxima acción/Contrato/Próxima tarea DESKTOP-QW6 ✅ hecho
  - `campaign_memory_write` lesson DAUD stale cleanup (ver cierre)
  - `campaign_update_task_state` completed (manual, hasTask false)
  - `campaign_diagnose_pipeline` + `skill progreso` Trigger 1 (edits ya aplicados, verificar)
- **Verify:** `cargo fmt --check` ✅ + scripts 0 gaps ✅ + Backlog 0 filas DAUD ✅ + plan recitation presente ✅ — listo para commit
- **Estado:** ✅ DONE — verify full verde, plan updated, listo para git add/commit

## Dependencias
- DESKTOP-QW1 ✅ COMPLETED (palette sync, 5f530bb9)
- DESKTOP-QW2 ✅ COMPLETED (HelpPanel F1/F2, 2b23661b)
- DESKTOP-QW3 ✅ COMPLETED (statusReport ES, 5f9b8276)
- DESKTOP-QW4 ✅ COMPLETED (filterActive DAUD-02, ad0f34b1) — **bloqueante directo**: DAUD-02 se resuelve por QW4, QW5 no puede cerrarse sin QW4 verde
- DAUD fixes previos: `3c53d8b2` (DAUD-03/04/05/07), `480935a7` (DAUD-01 E2E), `b865c625` (DAUD-06/08/11) — todos ya en main/develop (audit step lo confirma)

## Notas
- Ponytail: 9 filas ya Hecho → deletion-only, 0 lógica nueva, 1 commit docs. No añadir abstracción, no refactors, no dividir P37 en sub-fases.
- DAUD-02 no es staleness pura sino decisión owner (showFilters vs filterActive) → cerrada por QW4, QW5 solo la referencia como evidencia.
- DAUD-08 stash@{0} "06aa1a86" ya no existe (consumido por b865c625). Verificar `git stash list` no lo contiene; si existe stash distinto con mismo mensaje, no tocar (distinto hash).
- check-avance-coverage escanea fuentes en docs/avance/historial/fuentes + campanas, no Backlog → DAUD no afecta cobertura pre/post, pero registrar en activo/desktop.md asegura trazabilidad de dominio (progreso Trigger 1).
- validate-docs-coverage no cubre Backlog → 0 gaps pre/post garantizado si no se tocan docs/api/*.
- Campaign system hasTask false para este plan (no MCP registration) → recitation manual en plan file + memory_write (compatible QW1-4).

## Context Save Point
- **Fecha:** 2026-08-27T03:45
- **Branch:** develop
- **CI pendiente:** ninguno — scripts 1038/1038 + 0 gaps, cargo fmt verde, Backlog 0 filas DAUD
- **Decisiones:** DAUD 9/9 Hecho verificadas por commits 3c53d8b2/480935a7/b865c625 + QW4 ad0f34b1. P37 colapsada a 0 Cerrada 2026-08-26 (Exec Summary 118→109, last_reviewed 2026-08-26). Registro en active/desktop.md §P37 + backlog-history.md §Limpieza DAUD. Stash 06aa1a86 no existe (consumido).
- **Problemas conocidos:** ninguno — contrato mecánico verde post-edit
- **Próxima tarea:** DESKTOP-QW6 (Wave2 Task6 — CSP mínima tauri.conf.json H-01) — depende de DAUD limpieza, desbloqueada
