# FIND-21 — Menú contextual + atajos globales desktop

- **Estado:** ✅ COMPLETED
- **Plan:** `docs/plans/2026-09-10-fixes.md` (Task 4, Wave1)
- **Contrato:** `npm run build` 0 + right-click muestra menú propio + atajos documentados en guía + tsc 0
- **Appetite / Branch:** max 1d / develop
- **SDP:** frontend-ui-engineering, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design, campaign-executor, progreso (discover_skills_v2 BUILD, score 1.00)

## Decisión de diseño (DISCOVERY 2026-09-10)

- Stop condition del plan: "API Tauri no disponible → atajos in-app + DEFER nativo".
- `desktop/src-tauri/Cargo.toml` NO tiene `tauri-plugin-global-shortcut` ni `@tauri-apps/plugin-global-shortcut`
  en `package.json`. Agregar dep nativa = riesgo red/offline + colisión con SO (pre-mortem F1).
- **Decisión:** menú contextual propio in-app (React `onContextMenu`, funciona desktop+web) +
  atajos in-app extendidos. Shortcut nativo global (plugin Tauri) → **DEFER documentado** en guía.
- Gate D: no dispara (blast radius 2 archivos nuevos + wiring shell + HelpPanel + GUIDE delta;
  sin API pública nueva, sin hot path, sin símbolos `pub`).
- **Verify contrato (2026-09-10):** `npx tsc --noEmit` exit 0 · vitest AppContextMenu 3/3 PASS ·
  `npm run build` exit 0 (29.12s, warning preexistente chunk >500kB) · right-click → menú propio
  (código + test; smoke con app viva pendiente, sesión manual).

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `desktop/src/components/layout/WorkspaceShell.tsx` (1147L),
  `HelpPanel.tsx` (147L), `App.tsx` (63L), `main.tsx`, `package.json`, `vitest.config.ts`,
  `src-tauri/Cargo.toml`, `tauri.conf.json`, `docs/desktop/GUIDE.md` (92L).
- **Referencias hacia dentro (lo que toco usa):** `WorkspaceShell` usa `setPaletteOpen`,
  `setHelpOpen/setHelpTab`, `setSurface`, `onToggleTheme`, `onNotice`; `HelpPanel` exporta
  `HelpTab` + `SHORTCUTS`; stores `searchHistory`, `workspacePrefs` (no se tocan).
- **Referencias entrantes (quién usa lo que toco):** `App.tsx` renderiza `WorkspaceShell`
  (props intactas — solo se agregan hijos internos); `HelpPanel` solo lo monta `WorkspaceShell:1102`;
  `GUIDE.md` linkeado desde `README/ARCHITECTURE` desktop (apéndice, sin reordenar).
- **Veredicto:** impacto LOCAL al shell + 2 archivos nuevos autocontenidos. Sin cambios de props,
  stores, bridge Rust ni deps. Rollback = revert 5 archivos.

## Steps (~100 líneas c/u)

- [ ] **Step 1 — `AppContextMenu.tsx` + test RTL:** componente menú propio (role=menu, items con
  atajo visible, Esc/click-fuera cierra, clamp a viewport). Test: render items + click dispara
  acción + Esc cierra. ✅ DONE (3/3 PASS)
- [ ] **Step 2 — wiring `WorkspaceShell` + atajos nuevos:** `onContextMenu` en root (suppress
  nativo), estado `menuAt`, items (Paleta Ctrl+K, Guía ?, Tema Alt+T, Ajustes Ctrl+,);
  extender handler `onKey` (Alt+T tema, Ctrl+, ajustes, skip inputs). ✅ DONE
- [ ] **Step 3 — docs + verify + commit:** `HelpPanel.SHORTCUTS` (+2 filas), `GUIDE.md` sección
  delta (menú + atajos + DEFER nativo), `npm run build` 0 + `npx tsc --noEmit` 0 + vitest
  focado verde, commit solo archivos propios. ✅ DONE

## Notas

- WIP ajeno intacto: `M .opencode`, `M opencode.jsonc`, `M docs/Backlog.md`,
  `M desktop/src-tauri/Cargo.lock`, `?? Investigacion-plan.md` — NO tocar ni stagear.
- `campaign_verify_cmd` con bug exit -1 → bash directa si falla.
