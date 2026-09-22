# VULN-B3: js-yaml node/web + nanoid desktop (bumps mínimos, solo lockfiles)

## Metadata
- **Plan file:** docs/plans/2026-09-22-vulnerabilities.md (Wave B, gap sin cobertura)
- **Fuente:** plan Wave B §VULN-B3 + triage vanta-audit (alertas #42 node, #48 web-nested, #43-parcial desktop)
- **Esfuerzo:** 🟢 1h (3 slices lockfile-only, sin código)
- **Prioridad:** 🟠 HIGH (js-yaml CVE-2026-84375 7.5 + nanoid CVE-2026-67213 8.2, transitivas build-time)
- **Tipo:** JS lockfile (dependencias npm, cero código)
- **Turns estimados:** 8
- **Creado:** 2026-09-22T00:00
- **last-synced:** 2026-09-22
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps (3/3 slices ✅ + cierre)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Ninguno en runtime: js-yaml/nanoid son transitivas build-time (tooling). js-yaml ← @napi-rs/cli (node) / @eslint/eslintrc←eslint (web); nanoid ← postcss←vite (desktop). No hay `import` propio de estos paquetes en `*/src/`. |
| Callees | Registro npm (js-yaml≥4.3.2, nanoid≥3.3.18). Sin cambio de API pública. |
| Implicaciones | Contrato no cambia; comportamiento público idéntico (patch bumps); sin impacto perf/memoria/serialización; sin migración de datos; tests existentes no afectados (solo lockfiles). Riesgo principal: `overrides` puede forzar duplicados si el rango del padre no lo admite — se verifica con `npm ls` por slice. |

## Impacto mapeado (Regla 0)

> GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0).

- **Archivos leídos (completos):** `vantadb-node/package.json` (59L, sin deps directas js-yaml), `web/package.json` (72L, eslint ^9 en devDeps), `desktop/package.json` (56L, sin dep directa nanoid), `.opencode/rules/js-ecosystem.md` (34L completa), `.opencode/references/definition-of-done.md` (144L completa), `docs/plans/2026-09-22-vulnerabilities.md` (90L).
- **Lockfiles leídos (ventanas):** `vantadb-node/package-lock.json:2401-2412` (js-yaml 4.3.1, dev:true), `:430` (`"js-yaml": "^4.2.0"` rango padre admite 4.3.2 sin override forzoso — verificar con npm); `web/package-lock.json:465-502` (padre `@eslint/eslintrc` exige `^4.3.0` + nested `node_modules/@eslint/eslintrc/node_modules/js-yaml@4.3.1`); `desktop/package-lock.json:5409-5427` (nanoid 3.3.17) + `:5603` (`"nanoid": "^3.3.17"` rango padre admite 3.3.18/3.3.19).
- **Archivos referenciados hacia dentro:** `vantadb-node/package-lock.json` ← `vantadb-node/package.json` (`@napi-rs/cli ^3.8.2`); `web/package-lock.json` ← `web/package.json` (`eslint ^9`, `eslint-config-next ^16.3.4`); `desktop/package-lock.json` ← `desktop/package.json` (`vite ^7.0.4` → `postcss 8.5.26` → `nanoid ^3.3.17`).
- **Archivos que referencian a los editados:** Ningún `*/src/` importa js-yaml/nanoid (verificado: `codegraph_explore` no devuelve símbolos npm; `detect_changes develop` → 1 changed file solo plan Campaign-ID, impacted 0). `vantadb-ts/` NO se toca (VULN-B1/B2 en paralelo).
- **Veredicto impacto:** BAJO. Solo 6 archivos permitidos (`*/package.json` + `*/package-lock.json` × 3 carpetas) + este task file. Cero código, cero API pública, rollback = `git revert` del commit.

## Contrato
"`npm ls js-yaml` en vantadb-node y web muestra ≥4.3.2 sin 4.3.1 residual; `npm ls nanoid` en desktop muestra ≥3.3.18 sin 3.3.17 residual; `npx tsc --noEmit` verde en web (y donde aplique); cero cambios en `*/src/`; commit selectivo en rama `fix/vuln-node-web-desktop` SIN PUSH."

## Spec (SDD)
N/A — tarea 100% lockfile, sin símbolos públicos nuevos (Phase 1b: ninguna señal feature-add). Tabla Spec del contrato: N/A.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** no tocar `vantadb-ts/`, `src/` Rust, `.github/workflows/`, `docs/` salvo este task file, secretos, WIP ajeno (`docs/plans/...` M preexistente por Campaign-ID se deja sin commitear); no publicar nada (`npm publish` prohibido); no pushear (solo commit local).
- **Comandos de verificación:** `npm ls js-yaml` (node/web), `npm ls nanoid` (desktop), `npx tsc --noEmit` (web), `git status --short` (solo 7 paths).
- **Deuda pendiente:** ninguna al cerrar (si `tsc` tarda >10min → reportar y seguir, deuda escrita en Notas).

## Recitation (canónica)
- activeGoal: VULN-B3 — cerrar gaps #42/#48/#43-parcial con bumps mínimos
- lastAction: DISCOVERY completo + task file creado
- result: PARTIAL
- nextAction: Slice 1 node (npm ls → bump → verify)
- contract: ver ## Contrato + evidencia `npm ls` por slice + artefactos 6 lockfiles + task file
- nextTask: lead pushea + abre PRs (no el worker)

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda. `overrides` (si se usa en web) se justifica en Notas como bump mínimo sin romper build; no introduce deuda nueva (pin transitivo documentado, revertible con `git revert`).

## Definition of Done (contrato multi-nivel — P2-08)
- Task: contrato verificable (npm ls + tsc) ✅ + capa determinista adaptada (fmt/clippy/nextest N/A para lockfiles JS — se sustituye por `npm ls` + `tsc`/`lint` donde aplique) + sin regresiones.
- Commit: atómico (~solo lockfiles), conventional `fix: VULN-B3 — ...`, `git diff` limpio salvo WIP ajeno preexistente, verificación mecánica (nunca auto-reporte).
- Release: N/A (no publica; lead abre PRs).

## Herramientas necesarias
- npm por carpeta (`npm ls/update/install`), `npx tsc --noEmit` (web), `campaign_verify_cmd` (con fallback bash si BUG exit -1), `codegraph_explore` (blast radius), `campaign_detect_changes`/`check_index_coverage` (evidencia).

**Skills cargadas (SDP):** campaign-executor (base type TS SDK) · source-driven-development (base type) · incremental-implementation (lifecycle BUILD, slices verticales) · test-driven-development (lifecycle BUILD; adaptado: RED=versión vulnerable probada con `npm ls`, GREEN=bump, sin TDD clásico por cero código) · context-engineering (sesión nueva, jerarquía Rules→Spec→Source) · doubt-driven-development (stakes seguridad, verificación adversarial) · frontend-ui-engineering (lifecycle BUILD toca web/) · api-and-interface-design (boundaries de módulos) — 8/8, phase=BUILD, keywords [npm, lockfile, js-yaml, nanoid, overrides, eslint].
- SDP: campaign_discover_skills_v2 2026-09-22 (8 skills, todas cargadas).

## Investigation Notes
- `npm ls` evidencia (2026-09-22): node `vantadb-node@0.5.0 └─ @napi-rs/cli@3.8.2 └─ js-yaml@4.3.1`; web `vantadb-web@0.2.1 └─ eslint@9.39.5 └─ @eslint/eslintrc@3.3.6 └─ js-yaml@4.3.1`; desktop `desktop@0.1.0 └─ vite@7.3.6 └─ postcss@8.5.26 └─ nanoid@3.3.17`. npm 11.6.0 / node v26.8.1.
- `detect_changes develop`: 1 changed (plan Campaign-ID), impacted 0. `check_index_coverage`: 3 package.json sin issue registrado. `codegraph_explore`: sin símbolos npm propios (blast radius Rust irrelevante para lockfiles).
- GHSA/CVE del plan: js-yaml GHSA-2883-xcg3-v3hh (CVE-2026-84375, fix 4.3.2); nanoid GHSA-2v37-7h3g-55p8 (CVE-2026-67213, floor 3.3.18). Internet N/A (sin web research; fuentes del plan bastan).
- Estrategia por slice: intentar `npm update <pkg>` (mínimo); si el rango padre lo admite (`^4.2.0`/`^4.3.0`/`^3.3.17` admiten el floor) el update basta; si queda nested 4.3.1 (web) → `overrides` en `web/package.json` justificado (bump directo de `eslint`/`@eslint/eslintrc` sería major-riesgoso y rompe Scope Discipline).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — cadena de dependencia probada con `npm ls`; floors del plan; estrategia update→overrides definida |
| Pendientes de ejecución (downhill) | 3 — Slice 1 node, Slice 2 web, Slice 3 desktop |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — toca dependencias (bump). Skill aplicable: `security-and-hardening` (checklist: solo bumps patch de advisories verificados, sin nuevas deps, sin secretos, `npm ls` como evidencia; `cargo audit` N/A — stack JS). Sin checklist adicional: no hay input de usuario/auth/FFI/red en el cambio.
- [x] **PERFORMANCE** — N/A justificado: lockfiles patch no tocan hot paths (vector/engine/search/serialización). Sin baseline necesario.

## Steps

### Step 1: Slice 1 — vantadb-node js-yaml →≥4.3.2
- **Archivos:** `vantadb-node/package.json`, `vantadb-node/package-lock.json`
- **Acción:** `npm ls js-yaml` (RED: 4.3.1) → `npm update js-yaml` (o pin/override mínimo si el rango padre no resuelve) → `npm ls js-yaml` (GREEN: ≥4.3.2, sin 4.3.1)
- **Verify:** `npm ls js-yaml` + `git diff --stat` (solo 2 paths del slice)
- **Estado:** ✅ COMPLETED (js-yaml@4.3.2 vía `npm update js-yaml --save-dev`; diff: solo versión+integrity+prune `@emnapi/runtime` opcional + normalización `libc`, ninguna otra versión cambiada)

### Step 2: Slice 2 — web nested js-yaml →≥4.3.2
- **Archivos:** `web/package.json`, `web/package-lock.json`
- **Acción:** `npm ls js-yaml` (RED: nested 4.3.1 vía @eslint/eslintrc) → `npm update js-yaml` / bump mínimo; si persiste nested → `overrides: {"js-yaml": ">=4.3.2"}` + `npm install` (justificado: bump directo de eslint sería riesgoso) → `npm ls js-yaml` (GREEN)
- **Verify:** `npm ls js-yaml` + `npx tsc --noEmit` (si >10min → reportar y seguir) + `git diff --stat`
- **Estado:** ✅ COMPLETED (nested `@eslint/eslintrc/node_modules/js-yaml@4.3.2` vía `npm update js-yaml` directo — `overrides` NO fue necesario; tsc exit 0; eslint exit 0)

### Step 3: Slice 3 — desktop nanoid →≥3.3.18
- **Archivos:** `desktop/package.json`, `desktop/package-lock.json`
- **Acción:** `npm ls nanoid` (RED: 3.3.17 vía postcss) → `npm update nanoid` (o override mínimo) → `npm ls nanoid` (GREEN: ≥3.3.18)
- **Verify:** `npm ls nanoid` + `git diff --stat`
- **Estado:** ✅ COMPLETED (nanoid@3.3.19 vía `npm update nanoid`, `changed 1 package`; diff 3/3 líneas versión+resolved+integrity — excede floor 3.3.18, igual que Wave A #199)

### Step 4: Cierre — rama + commit selectivo SIN PUSH + RESULTADO
- **Archivos:** los 6 lockfiles + este task file
- **Acción:** rama `fix/vuln-node-web-desktop` desde `develop`; `git add` selectivo (7 paths); commit `fix: VULN-B3 — js-yaml node/web + nanoid desktop`; NO PUSH; rellenar RESULTADO §7 + Gates D/V/C
- **Verify:** `git status --short` + `git log --oneline -1`
- **Estado:** ✅ COMPLETED (commit pathspec selectivo; plan file + `vantadb-ts/*` + `VULN-B1-B2.md` del agente paralelo excluidos; SIN PUSH)

## Dependencias
- Wave B paralela: VULN-B1/B2 disjuntos (no tocar `vantadb-ts/`). NextTask: lead pushea + abre PRs (no el worker).

## Review (GATE — agente distinto, P2-01)
- **Revisor:** doubt-driven-development (contexto fresco; vanta-audit/review no invocable desde worker — se registra como deuda de proceso para el lead).
- **Enfoque:** ¿overrides justificados? ¿bumps mínimos sin código? ¿alcance respetado?
- **Cómo se probó:** `npm ls` por slice + `tsc` web + `git diff --stat` (evidencia mecánica, no auto-reporte).
- **Checklist anti-hábitos tóxicos:**
  - [ ] No inventar salidas de comandos no ejecutados.
  - [ ] No saltarse clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar contrato.
  - [ ] No ignorar fallos ni reportar "todo OK" con fallo parcial.
  - [ ] No hacer un solo intento y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos.
  - [ ] No degradar chequeo de errores en seguridad.
  - [ ] No gastar presupuesto infinito.
- **Veredicto:** ⏳ pendiente-lead-PR (motivo: worker no spawnea review dedicado para bump lockfile-only con evidencia mecánica completa; el PR del lead + CI verde es el review de agente distinto efectivo — P2-01 diferido al PR, sin bloqueo del commit local)

## Notas
- Ponytail full: ladder existe→stdlib→dep→mínimo; `npm update` antes que `overrides`; `overrides` solo si el update deja nested vulnerable. Cero código por contrato.
- WIP ajeno: `M docs/plans/2026-09-22-vulnerabilities.md` (Campaign-ID) preexistente — NO commitear, NO revertir.
- `campaign_verify_cmd` SIN BUG en esta corrida (exit 0, diff --stat de los 3 lockfiles) — no hizo falta fallback bash.
- `overrides` en web NO hizo falta: `npm update js-yaml` levantó el nested a 4.3.2 (el rango padre `^4.3.0` lo admite). Decisión documentada como exige el contrato.
- nanoid resolvió a 3.3.19 (no 3.3.18 exacto): `npm update` toma el máximo del rango `^3.3.17`; excede el floor igual que #199 en Wave A — aceptado.
- `skill progreso` OMITIDA por scope (tocaría `docs/Backlog.md`+`docs/avance/`, prohibidos en el contrato) — el lead la ejecuta al abrir PRs.
- Plan file NO tocado (WIP paralelo VULN-B1/B2 activo en el mismo worktree: `vantadb-ts/*` + `docs/tasks/VULN-B1-B2.md` + recitation del plan) — handoff vía `campaign_update_task_state` (traceId) en vez de edición de archivo.
- Gates: P no-disparado (lockfiles, sin símbolos públicos nuevos, sin spec) · D no-disparado (blast radius 0, contrato cerrado, 0 incógnitas) · V no-disparado (cero fallas verify) · C no-disparado (sin colaterales propios; WIP ajeno excluido del commit).
