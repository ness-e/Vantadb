# DESKTOP-40-slice2 — i18n resto UI desktop (shell chrome)

- **Plan:** `docs/dev/plans/2026-09-10-code.md` (Task 3) · **Campaign:** 2c3d4e5f-6a7b-8c9d-0e1f-2a3b4c5d6e01
- **Estado:** ⏳ IN PROGRESS · **Rama:** `develop` (verificada al inicio)
- **Contrato:** pantallas migradas a `tt()` + `npm run build` 0 + tsc 0 + vitest i18n verdes
- **Scope slice2 (acotado pantalla-por-pantalla):** shell chrome ONLY — `WorkspaceShell` (sidebar/topbar/notices/filtros),
  `TitleBar`, `SplashScreen`, `HelpPanel`, `NamespaceDialog`, `App` fallback, `dictionaries` + `i18n.test`.
  `LensShell` es props-driven (sin strings propios → verify no-op).
- **DEFER → DESKTOP-40-slice3:** lentes y paneles (Retrieval/Indices/Consolidate/Graph/Space/Memory/Proxy/Trash/
  Inspector/Ingest/DataExplorer/ConnectionPanel/Metrics/Kpi/Sop/Export/ResultsList/CommandPalette/Import).
- **SDP:** campaign-executor, frontend-ui-engineering, source-driven-development, incremental-implementation,
  test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design
  (via `campaign_discover_skills_v2` phase BUILD; cargadas: incremental-implementation, test-driven-development,
  context-engineering, frontend-ui-engineering)

## Spec

Patrón slice 1 (b64cbb30, trusted): `tt(lang, key, fallback-ES)` + `tp(lang, key, fallback, params)` con
`lang = connectionPrefs.get().lang ?? "es"`. Catálogo simétrico ES/EN (`Record<keyof typeof es, string>`).
Sin símbolos públicos nuevos salvo claves de catálogo (no `pub fn`/endpoint/binding nuevo).

| Decisión | Opción elegida | Evidencia / por qué |
|---|---|---|
| Fuente de `lang` en shell | `connectionPrefs.get().lang ?? "es"` por render | Mismo que `Settings.tsx:39`; store ya persiste `lang` (connections.ts:68) |
| Fallback | Literal ES inline (patrón `createTt` web) | `dictionaries.ts:67-69`; si falta clave la UI sigue en ES, nunca crashea |
| Interpolación | `tp()` solo donde hay `{name}`/conteos | `Settings.tsx:60`; resto `tt()` simple |
| HelpPanel SHORTCUTS/SURFACES | Tablas se mantienen; se traducen descripciones vía claves `help.*`, nombres de superficie quedan como labels propios (RESUMEN/MEMORIAS/…) | Labels son identidad de navegación (igual que Settings deja "ESPAÑOL/ENGLISH" literales); traducir descripciones sí, renombrar superficies no |
| TitleBar aria-labels | `tt()` (Minimizar/Maximizar/Cerrar) | Son UI visible a AT; "VantaDB Studio" es marca → no se traduce |
| SplashScreen | `aria-label` + hint via `tt()`; "VantaDB Studio" + tagline técnica EN quedan | Marca + tagline son identidad (igual que brand en WorkspaceShell sidebar) |
| WorkspaceShell notices dinámicos | `tp()` con params `{name}`/`{n}`/`{count}` | Pre-mortem: strings dinámicos sin clave → cada template literal con variable gana clave `shell.notice.*` |
| LensShell | Sin cambios (props-driven) | No contiene strings propios; las lentes que lo usan se migran en slice3 |
| Tests | Extender `i18n.test.ts` (claves nuevas ES/EN + simetría + `tp()` interpolación) | Contrato exige "vitest i18n verdes"; pirámide: unit small |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `desktop/src/i18n/dictionaries.ts` (81L), `desktop/src/i18n/index.ts` (3L),
  `desktop/src/i18n/i18n.test.ts` (22L), `desktop/src/pages/Settings.tsx` (229L, patrón),
  `desktop/src/components/layout/WorkspaceShell.tsx` (~1160L), `TitleBar.tsx` (67L),
  `SplashScreen.tsx` (79L), `HelpPanel.tsx` (149L), `NamespaceDialog.tsx` (158L),
  `LensShell.tsx` (37L), `desktop/src/App.tsx` (63L), `desktop/src/store/connections.ts` (128L).
- **Referencias hacia dentro (qué importa el slice):** `../i18n` (tt/tp/DesktopLang),
  `../store/connections` (connectionPrefs.lang), componentes layout entre sí vía `WorkspaceShell`
  (Settings, HelpPanel, NamespaceDialog, AppContextMenu, CommandPalette lazy).
- **Referencias entrantes (quién usa lo tocado):** `Settings` consumido por `CommandPalette.tsx` (1 caller);
  `WorkspaceShell` consumido por `App.tsx`; `TitleBar`/`SplashScreen` por `App.tsx`;
  `HelpPanel`/`NamespaceDialog` por `WorkspaceShell`; `tt/tp` solo por `Settings.tsx` hoy (grep `from.*i18n` = 1 match).
  Ningún test cubre `WorkspaceShell`/`Settings` (codegraph: "no covering tests found").
- **Veredicto:** impacto LOCAL al shell chrome; sin cambios de API/store/transporte; lentes reciben
  `onNotice(string)` ya traducido desde el shell (firma intacta). Riesgo bajo; rollback por archivo.

## Gate D

Blast radius slice2 acotado = 7 archivos editados (dictionaries, i18n.test, TitleBar, SplashScreen,
HelpPanel, NamespaceDialog, WorkspaceShell + App 1-liner) — por debajo del umbral >10.
Sin símbolos públicos nuevos, contrato no ambiguo, con Spec llena arriba → **Gate D: no disparado**,
se procede a ACT. Scope total `desktop/src/` queda cubierto por estrategia pantalla-por-pantalla
+ stop condition (shippear verdes + DEFER resto en slice3).

## Steps

- [x] **Step 1 — catálogo shell.* + tests RED→GREEN** (`dictionaries.ts`, `i18n.test.ts`): añadir claves
  `shell.*`/`layout.*`/`help.*`/`ns.*`/`splash.*`/`titlebar.*` ES/EN + tests (resolución ES/EN, fallback,
  simetría, `tp()` interpolación). Verify: `npx vitest run src/i18n` en `desktop/` → 8/8 ✅
  (+11 claves post-discovery: shell.loadingViewer/imported/menu* + help.tr*Label).
- [x] **Step 2 — TitleBar + SplashScreen + App fallback** (`TitleBar.tsx`, `SplashScreen.tsx`, `App.tsx`):
  aria-labels e hint vía `tt()`; `reportError` fallback vía `tt()`. Verify: `npx tsc --noEmit` → 0 ✅.
- [x] **Step 3 — HelpPanel** (`HelpPanel.tsx`): SHORTCUTS/SURFACES descripciones + chrome (título, tabs,
  headings, tip, aria) vía `tt()`; `lang` por prop con default `connectionPrefs`. Verify: tsc 0 ✅.
- [x] **Step 4 — NamespaceDialog** (`NamespaceDialog.tsx`): títulos/placeholders/avisos/botones vía
  `tt()`/`tp()`; `lang` por prop. Verify: tsc 0 ✅.
- [x] **Step 5 — WorkspaceShell** (`WorkspaceShell.tsx`): sidebar/topbar/notices/filtros/resultados/
  resúmenes/menú vía `tt()`/`tp()` con `lang` reactivo (LANG_EVENT); `lang={lang}` a HelpPanel/NamespaceDialog.
  Verify: tsc 0 ✅. (No hizo falta partir: slices 5a/5b secuenciales en una sesión.)
- [x] **Step 6 — verify full + commit**: `npm run build` 0 ✅ + `npx tsc --noEmit` 0 ✅ + `npx vitest run src/i18n`
  8/8 ✅ + spot-check grep 0 hardcodeados (solo fallbacks/marca) ✅; commit `feat: DESKTOP-40-slice2 — i18n shell chrome`.

## Contrato de verificación

- `cd desktop && npm run build` → 0 errores
- `cd desktop && npx tsc --noEmit` → 0 errores
- `cd desktop && npx vitest run src/i18n` → verde
- `grep ES-hardcodeado → 0` en archivos migrados (spot-check, no en comentarios/marca)

## Deuda / WIP ajeno (NO TOCAR)

`M opencode.jsonc`, `M .opencode`, `M desktop/src-tauri/Cargo.lock`, `?? Investigacion-plan.md` — ajenos.
`campaign_verify_cmd` con bug exit -1 → bash directa. Commit SOLO archivos del slice.
