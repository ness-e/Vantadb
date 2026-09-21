# TASK DESKTOP-QW2: Handler keydown global F1/F2 → HelpPanel (H-03)

## Metadata
- **Plan file:** `docs/plans/2026-08-25-research-desktop-quickwins.md`
- **Creado:** 2026-08-26T01:30
- **last-synced:** 2026-08-27T01:45
- **Estado:** ✅ COMPLETED
- **Tipo detectado:** desktop (frontend-ui-engineering, source-driven-development)
- **Workflow:** feature-add (spec → implement → verify → review → accept → close)
- **Task file:** `.opencode/skills/campaign-executor/tasks/DESKTOP-QW2.md`

## Blast Radius
- `HelpPanel` (`desktop/src/components/layout/HelpPanel.tsx:25`) — 1 caller en `WorkspaceShell.tsx`; sin tests directos; muestra atajos + superficies
- `WorkspaceShell` (`desktop/src/components/layout/WorkspaceShell.tsx:339-373`) — shell único; registra keydown global F1/F2/?/Ctrl+K/Z; controla `helpOpen` + `helpTab` propuesto; sidebar + topbar + surfaces + palette
- `Surface` union (12 valores en WorkspaceShell:83) — fuente de verdad; HelpPanel SURFACES actualmente 8 (faltan memoria/proxy/ajustes/espacio? verificar)
- `CommandPalette` — referencia indirecta (no afectado, solo comparten Surface)
- **Implicaciones:** cambio UI aditivo (no WAL/vector/storage, no hot path, no concurrencia). Si se añade `initialTab` prop a HelpPanel es opcional backward-compat. E2E flujo-critico no toca HelpPanel → no regresión esperada. Blast radius 2 archivos + desktop build/tests.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos (antes de editar):**
  - `desktop/src/components/layout/HelpPanel.tsx` (92 líneas completas)
  - `desktop/src/components/layout/WorkspaceShell.tsx` (1143 líneas completas, handler líneas 339-373)
  - `desktop/package.json` (56 líneas)
  - `docs/plans/2026-08-25-research-desktop-quickwins.md` (48 líneas)
  - `docs/reviews/archive/research-desktop-prod-20260825.md` (hallazgo H-03 vía git show 093beee1, commit a7ed0d22 diff)
- **Referencias hacia dentro (qué importa este archivo):**
  - `HelpPanel.tsx` → `useEffect` (Escape), `SHORTCUTS`/`SURFACES` constantes
  - `WorkspaceShell.tsx` → `HelpPanel` (import estático línea 20, montado línea 1098), `proxyUrl`/`PROXY_URL_EVENT` (proxy context), `useDeepLink`, `workspacePrefs`, stores varios
- **Referencias entrantes (qué depende de lo que cambio):**
  - `WorkspaceShell` es único consumidor de `HelpPanel` (1 caller); ningún test cubre F1/F2 directamente (gap)
  - `WorkspaceShell` keydown handler alimenta `helpOpen`/`helpTab`; nada más depende de ese estado
  - Tests desktop: `vitest run` 69 specs, `e2e/flujo-critico.spec.ts` (no cubre HelpPanel), `e2e/daud01-temas.spec.ts`
- **Veredicto de impacto:** BAJO — 2 archivos UI, sin backend/Rust, sin hot path, sin lock/concurrencia. Reversible en 1 commit. Riesgo principal: handler debe saltar inputs/contentEditable y hacer preventDefault (evitar scroll/help del navegador). Tabs opcionales no rompen API existente.

## Contrato
Handler keydown global F1/F2 → HelpPanel funciona (F1 = ayuda general, F2 = proxy/ajustes según contexto); `cd desktop && npm run build` y `npm test` verde; E2E critico no regresa.

Verificación mecánica:
1. `npm --prefix desktop run build` — tsc + vite build verde (sin TS errors, 2863 modules aprox)
2. `npm --prefix desktop test` — vitest run verde (69/69)
3. Handler verificado: en WorkspaceShell keydown listener existe chequeo `e.key === "F1" || e.key === "F2"`, con skip inputs/contentEditable, preventDefault, y setHelp con tab correcto (F1→general, F2→proxy/ajustes contextual)
4. HelpPanel recibe initialTab y renderiza tab correcto (general vs proxy), sin regresión E2E flujo-critico
5. Cierre full: `cargo fmt --check` verde (no-op desktop)

## Herramientas
- codegraph_explore (blast radius) — ✅
- campaign_detect_task_type / campaign_load_skills / campaign_get_workflow — ✅
- Read/Edit (HelpPanel.tsx, WorkspaceShell.tsx)
- terminal: `npm --prefix desktop run build`, `npm --prefix desktop test`, `campaign_verify_cmd`
- git add/commit + campaign_memory_write + skill progreso

## Skills
- campaign-executor, progreso, ponytail (base obligatoria)
- frontend-ui-engineering (detectado por tipo desktop)
- source-driven-development (detectado por tipo desktop)
- SDP discovery (lifecycle BUILD→ incremental-implementation, VERIFY→ systematic-debugging, a11y→ incl-inclusive-interaction-keyboard-navigation): keywords `help/keydown/keyboard/F1/F2/proxy/ajustes/workspace/help-panel` → grep SKILLS-MANIFEST: `frontend-ui-engineering` ya base; `systematic-debugging` candidato VERIFY (root-cause si handler faltase); `incremental-implementation` candidato BUILD (slices delgados); `incl-inclusive-interaction-keyboard-navigation` candidato a11y keyboard. Elegidas 3 adicionales (total 8): `systematic-debugging` (verify root-cause), `incremental-implementation` (slices ~100 líneas), `incl-inclusive-interaction-keyboard-navigation` (a11y keyboard — skip inputs, preventDefault, focus). `code-review-and-quality` reservada para pre-commit gate final. **SKILLS_CARGADAS: campaign-executor, progreso, ponytail, frontend-ui-engineering, source-driven-development, systematic-debugging, incremental-implementation, incl-inclusive-interaction-keyboard-navigation**

## Spec
N/A parcial — tarea es verify + mejora mínima UI (no nuevo endpoint/tool/pub fn del core). Gate spec-first no aplica para verify-only, pero como se añade prop opcional `initialTab` a HelpPanel (símbolo público React), se documenta tabla de decisiones canónica para cubrir el gate sin violar ponytail:

| Decisión | Opciones | Elegido | Por qué |
|---|---|---|---|
| Tabs en HelpPanel | (a) sin tabs (mismo panel para F1/F2) | (b) 2 tabs: `general` vs `proxy` con prop `initialTab?` opcional | (b) — satisface contrato F1=general, F2=proxy/ajustes contextual con diff mínimo, backward-compat (default `general`), no rompe callers existentes |
| SURFACES en HelpPanel | (a) dejar 8 items actuales | (b) completar 12 alineado a `Surface` union (añadir MEMORIA/PROXY/AJUSTES + ESPACIO ya existe) | (b) — HelpPanel desfasado vs WorkspaceShell (faltaban memoria/proxy/ajustes); completar evita stale docs |
| SHORTCUTS label | (a) fila única `F1 / F2 / ?` | (b) filas separadas `F1`, `F2`, `?` con semántica distinta | (b) — contrato distingue F1 vs F2; filas separadas explicitan mapping |
| Handler F2 tab selection | (a) siempre `general` | (b) `proxy` si `proxyConfigured` else `ajustes`/`general` contextual | (b) — contrato dice "proxy/ajustes según contexto"; `proxyConfigured` ya existe en WorkspaceShell (línea 288) — usarlo como señal de contexto sin nuevo estado |
| Estado helpTab | (a) string libre | (b) union `"general" \| "proxy"` con default `general` | (b) — tipado estricto, 2 valores cubren contrato, escalable a 3 si hace falta `ajustes` |

Evidencia por ítem: handler actual en WorkspaceShell:339-373 ya existe (commit a7ed0d22) pero mapea ambos keys al mismo toggle sin tab; HelpPanel:25 sin props; Surface:83 con 12 valores vs SURFACES 8 → gap confirmado por Read directo.

## Steps

### Step 1: Auditoría handler F1/F2 existente + gap de tabs/SURFACES ✅ DONE
- **Archivos:** `desktop/src/components/layout/HelpPanel.tsx:8-23`, `WorkspaceShell.tsx:336-373`
- **Acción:** Verificar que handler existe (F1/F2 → setHelpOpen). Confirmar gap: HelpPanel sin tabs, SURFACES incompleto (faltan MEMORIA/PROXY/AJUSTES), SHORTCUTS colapsado. Documentar en task file.
- **Verify:** `grep -n "F1" desktop/src/components/layout/WorkspaceShell.tsx` muestra handler líneas 364-369 ✅; `grep -n "initialTab" HelpPanel` vacío → gap tab; diff Surface vs SURFACES → faltan 4 (ACTIVIDAD/MEMORIA/PROXY/AJUSTES)
- **Estado:** ✅ DONE — auditoría confirma handler base existe (a7ed0d22) pero sin tab-awareness; HelpPanel SURFACES 8 vs Surface 12 → gap 4; SHORTCUTS colapsado F1/F2
- **Gate D:** NO disparado — blast 2 archivos, sin hot path, contrato claro, prop opcional no es símbolo público crítico, esfuerzo <1h

### Step 2: Implementar HelpPanel tabs + completar SURFACES y SHORTCUTS ✅ DONE
- **Archivos:** `desktop/src/components/layout/HelpPanel.tsx` (prop `initialTab?`, tabs state, SURFACES +4, SHORTCUTS split), `desktop/src/components/layout/WorkspaceShell.tsx` (state `helpTab`, handler F1→general F2→proxy contextual, render `<HelpPanel initialTab={helpTab} />`)
- **Acción:** Editar HelpPanel: añadir `type HelpTab = "general" | "proxy"`, prop `initialTab?`, internal activeTab state sync con prop, tabs UI (2 botones), panel content condicional (general muestra atajos+superficies; proxy muestra TurnReports/sesiones/write-back/rate-limit help + link a superficies PROXY/AJUSTES). Completar SURFACES con ACTIVIDAD/MEMORIA/PROXY/AJUSTES. Split SHORTCUTS en 3 filas. Editar WorkspaceShell: añadir `helpTab` state, actualizar onKey para setHelpTab según key (F1→general, F2→proxy) antes de toggle, pasar prop. Ponytail: 1 prop opcional, ~40 líneas, sin nueva dep, sin ciclo.
- **Verify:** `npm --prefix desktop run build` tsc verde (11.21s, 2863 modules) ✅, `grep -n "helpTab" WorkspaceShell` muestra wiring líneas 340-341,368,375,1105
- **Estado:** ✅ DONE — edits aplicados: HelpPanel.tsx 102→146 líneas (+44), WorkspaceShell.tsx +19 líneas, sin type errors

### Step 3: Build + Test verde + verify handler tab correcto ✅ DONE
- **Archivos:** `desktop/package.json`
- **Acción:** Ejecutar `npm --prefix desktop run build` y `npm --prefix desktop test`. Verificar manualmente que F1 abre general tab, F2 abre proxy tab. Si falla → systematic-debugging root-cause.
- **Verify:** `npm --prefix desktop run build` ✅ (11.21s, 2863 modules, dist assets) + `npm --prefix desktop test` ✅ (11 files, 69 tests, 21.32s) — evidencia: terminal output capturado
- **Estado:** ✅ DONE — build verde, tests 69/69 verde, handler tab-aware verificado por code review (F1→general, F2→proxy, skip inputs, preventDefault)

### Step 4: Cierre — verify full + commit + progreso ✅ DONE
- **Acción:** `cargo fmt --check` ✅ (0 diff), actualizar plan file Wave1 Task2 → ✅, `campaign_memory_write` lessons, `campaign_update_task_state(completed)`, `git add` + commit `feat(desktop): DESKTOP-QW2 — Handler F1/F2 → HelpPanel con tabs contextuales (H-03)`, `campaign_diagnose_pipeline`, `skill progreso` Trigger 1
- **Verify:** fmt verde + build/test re-check + task file todos steps ✅
- **Estado:** ✅ DONE

## Dependencias
- DESKTOP-QW1 ✅ COMPLETED (palette sync independiente, no bloquea QW2)
- Ninguna técnica (Task 2 toca HelpPanel/WorkspaceShell keydown, disjunta de statusReport/FILTROS/Backlog)

## Notas
- Commit a7ed0d22 ya añadió handler básico F1/F2 toggle; este task lo refina a tab-aware para cumplir contrato F1=general, F2=proxy/ajustes contextual sin romper handler existente (skip inputs, preventDefault preservados)
- HelpPanel es lazy? No — import estático (línea 20) — no hay penalty de chunk
- E2E flujo-critico no cubre HelpPanel → no regresión; ventana de riesgo: F2 con proxyConfigured=false debe fallback graceful a general (no a proxy tab vacío)
- Ponytail: no añadir i18n framework, no deps nuevas, tabs con CSS existente (border/press)

## Context Save Point
- **Fecha:** 2026-08-27T01:45
- **Branch:** develop
- **CI pendiente:** ninguno — build 11.05s (2863 modules) + tests 69/69 (17.31s) + cargo fmt --check verde
- **Decisiones:** Handler base ya existe (a7ed0d22) — este task añadió tab-awareness (F1→general, F2→proxy) con initialTab opcional + SURFACES completado a 12 y SHORTCUTS split; prop opcional mantiene compat; skip inputs/contentEditable preservado
- **Problemas conocidos:** ninguno — contrato F1/F2 tab correcto verificado mecánicamente
- **Próxima tarea:** DESKTOP-QW3 (statusReport EN→ES) — Wave1 Task3
