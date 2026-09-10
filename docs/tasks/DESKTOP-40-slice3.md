# DESKTOP-40-slice3 — i18n lentes/paneles desktop

- **Plan:** `docs/plans/2026-09-10-code.md` (Task 3 DEFER) · **Backlog:** `docs/Backlog.md` fila `DESKTOP-40-slice3`
- **Estado:** ⏳ IN PROGRESS · **Rama:** `develop` (verificada paso 0)
- **Contrato:** lentes + paneles restantes migrados a `tt()/tp()` + `npm run build` desktop 0 errores + `npx tsc --noEmit` exit 0 + `npx vitest run src/i18n` verdes
- **Appetite:** max 2d · **Commit:** conventional con task ID (`feat(desktop): ... (DESKTOP-40-slice3)`)
- **SDP:** frontend-ui-engineering, incremental-implementation, test-driven-development, context-engineering (+ campaign-executor/progreso/ponytail auto-MCP; source-driven-development no aplica — sin API externa; doubt-driven/api-and-interface-design descartados — sin trust boundary ni API pública nueva)

## Spec

Patrón slices 1-2 (trusted, b64cbb30/bbdeae17): `tt(lang, key, fallback-ES)` + `tp(lang, key, fallback, params)` con catálogo simétrico ES/EN (`Record<keyof typeof es, string>`). Sin símbolos públicos nuevos salvo claves de catálogo.

| # | Decisión | Evidencia / por qué |
|---|----------|---------------------|
| 1 | `lang` por prop opcional con default `connectionPrefs.get().lang ?? "es"` (patrón HelpPanel) — NO tocar WorkspaceShell | HelpPanel.tsx:40, NamespaceDialog.tsx:38; WorkspaceShell re-renderiza hijos al cambiar `lang` (state + LANG_EVENT 300-304) → el default se re-evalúa por render, reactivo sin wiring |
| 2 | Fallback = literal ES inline | dictionaries.ts:330-332; UI nunca crashea si falta clave |
| 3 | Literales técnicos/marca quedan literales intencionalmente | Precedente slice 1-2: "VantaDB Studio", "ESPAÑOL/ENGLISH", jerga técnica (BM25/HNSW/RRF/WAL/JSONL/vantaPut/Ctrl+Z), labels de superficie (PAPELERA/RESUMEN) son identidad |
| 4 | Tests: extender `i18n.test.ts` con claves nuevas (resolución ES/EN + simetría existente + `tp()` interpolación) | Contrato exige "vitest i18n verdes"; pirámide: unit small |
| 5 | `relTime` TrashLens ("now/5m ago") y formatos técnicos se traducen vía claves con params | Son UI visible; params `{m}/{h}/{d}` vía `tp()` |
| 6 | Mark/mark-studio = copy de mascota/brand → literal intencional, NO migrar | Igual que SplashScreen tagline (slice 2 decisión 29) |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `desktop/src/i18n/dictionaries.ts` (344L), `desktop/src/i18n/index.ts` (3L barrel — NOTA: contexto decía store `lang` + `LANG_EVENT` en index.ts, el código lo contradice: viven en `store/connections.ts:99,126,129`), `desktop/src/i18n/i18n.test.ts` (58L), `store/connections.ts` (vía grep: LANG_EVENT + connectionPrefs), `WorkspaceShell.tsx` (vía grep: montaje lentes 1009-1192, `lang` state 300-304 — lentes NO reciben `lang` hoy), `TrashLens.tsx` (138L), `SelectionBar.tsx` (104L), `ConfirmDiscard.tsx` (136L), `SpaceLens.tsx` + `GraphLens.tsx` (verbatim vía codegraph_explore).
- **Referencias hacia dentro:** `../../i18n` (tt/tp/DesktopLang), `../../store/connections` (connectionPrefs default). Lentes reciben `onNotice/onError` ya traducidos desde shell (firma intacta — se añade solo prop opcional `lang?`).
- **Referencias entrantes:** WorkspaceShell → todas las lentes/paneles (props existentes intactas; `lang?` opcional no rompe callers). Tests: `MemoryLens.test.tsx`, `HomeOverview.test.tsx`, `ProxyDashboard.test.tsx`, `ImportConfirm.test.tsx`, `AppContextMenu.test.tsx` — verificar que no aserten strings migrados.
- **Veredicto:** impacto 🟢 LOCAL por archivo; sin cambios API/store/transporte; rollback por archivo. WorkspaceShell NO se toca.

## Gate D

Blast radius por slice ≤3 archivos, sin hot path, sin API pública nueva, contrato explícito → **no disparado**, se procede a ACT. Estrategia pantalla-por-pantalla + stop condition (shippear verdes; si appetite >2d → INCOMPLETO con próximo step exacto).

## Auditoría pantalla→strings→claves

| Pantalla | Strings ES usuario-visible | Claves |
|----------|---------------------------|--------|
| TrashLens | aria "Papelera", meta `{n} tombstone(s) · Ctrl+Z deshace`, hint restore, "papelera vacía", notices restaurado/eliminado `{id}`, title restore, "↩ RESTORE", "¿BORRAR?", aria cancelar, title descarta, "BORRAR DEF.", relTime now/{m}m/{h}h/{d}d | `trash.*` |
| SpaceLens | notice proyección lista `{n}`, notice exportados `{n}`, notice papelera lote, "proyectando…", aria namespace, "todos los namespaces", title re-proyectar, "⤒ proyectar", hint interacción, aria scatterplot `{n}`, empty/loading/error states | `space.*` |
| SelectionBar | aria grupo, `{n} seleccionado(s)`, titles export/borrar/limpiar, "⭳ exportar (n)", "¿BORRAR {n}?", aria cancelar, "✕ eliminar (n)", "✕ limpiar" | `space.sel.*` |
| GraphLens | badge resaltados `{n}`, "expandiendo…", aviso tope, titles iql/fit/reset/labels, hint interacción, aria grafo `{nodos,aristas,ns}`, fallback "cargando escena 3D…", aria lista nodos | `graph.*` |
| IqlConsole | notice contexto obsoleto, header título/resultado/acciones | `graph.console.*` |
| RetrievalLens | notice query vacía, meta/desglose, placeholder, títulos/aria filtros, empty states, "vecino semántico", errores audit | `retrieval.*` |
| ConsolidateLens | notices sin backend/sin marcados, aria sección, empty states, títulos batch | `consolidate.*` |
| ConfirmDiscard | título eliminar `{n}`, radios papelera/permanente, hint escribí `{expected}`, mismatch, botones | `consolidate.confirm.*` |
| IndicesLens | meta `{backend} · poll 4s`, error métricas, aria secciones, gaps | `indices.*` |
| MemoryLens | TBD (leer) | `memory.*` |
| ProxyDashboard | TBD (leer) | `proxy.*` |
| Inspector + tabs | TBD (leer) | `inspector.*` |
| CommandPalette | TBD (leer) | `palette.*` |
| HomeOverview | TBD (leer) | `home.*` |
| ActivityPanel/Timeline | TBD (leer) | `activity.*` |
| DataExplorer/ResultsList | TBD (leer) | `data.*` |
| IngestForm/ImportPaste/ImportDrop | TBD (leer) | `ingest.*` |
| ConnectionPanel/MetricsGrid/KpiCards/SopPanel/ExportPanel/ExportButtons | TBD (leer) | `panels.*` |
| FiltersBuilder | TBD (leer, 1 match) | `filters.*` |
| ScoreBars/PayloadTab/GraphScene/etc. | 0 strings → no-op | — |
| Mark/mark-studio | brand copy → literal intencional | — |

## Steps

- [x] **Step 0 — DISCOVERY + task file:** gap real (grep `i18n` = solo shell chrome), patrón `lang` por prop, Gate D no dispara.
- [x] **Step 1 — catálogo lens.* + tests RED→GREEN** (`dictionaries.ts`, `i18n.test.ts`): claves `trash.*`/`space.*`/`graph.*`/`retrieval.*`/`consolidate.*`/`indices.*`/`memory.*`/`proxy.*`/`inspector.*`/`palette.*`/`home.*`/`activity.*`/`data.*`/`ingest.*`/`panels.*`/`export.*`/`ctxmenu.*` ES/EN + tests. Verify: `npx vitest run src/i18n` → 11/11 ✅.
- [x] **Step 2 — SpaceLens + SelectionBar**: migrados → tsc (lang por prop, `useProjection(lang)`, clave nueva `space.noVectors` ES/EN para error sin-vectores).
- [x] **Step 3 — GraphLens + IqlConsole**: migrados → tsc (role/aria-* intactos, lista sr-only preservada).
- [x] **Step 4 — RetrievalLens**: migrado → tsc.
- [x] **Step 5 — ConsolidateLens + ConfirmDiscard**: migrados → tsc.
- [x] **Step 6 — TrashLens**: migrado → tsc (`relTime` vía claves con params).
- [x] **Step 7 — IndicesLens + MemoryLens + ProxyDashboard**: migrados → tsc.
- [x] **Step 8 — Inspector + tabs + CommandPalette + FiltersBuilder + ResultsList**: migrados → tsc.
- [x] **Step 9 — HomeOverview + ActivityPanel/Timeline + DataExplorer**: migrados → tsc.
- [x] **Step 10 — IngestForm/ImportPaste/ImportDrop + ConnectionPanel/MetricsGrid/KpiCards/SopPanel/ExportPanel/ExportButtons**: migrados → tsc.
- [x] **Step 11 — VERIFY contrato + commit solo-propios + sync**: `npm run build` 0 errores (19.98s, solo warnings chunk-size pre-existentes) + tsc 0 + vitest i18n 11/11 + commit + recitation.

## Iteraciones (RETRY 2026-09-10)

| # | Acción | Resultado | Herramienta |
|---|--------|-----------|-------------|
| 0 | Paso 0 rama: `git branch --show-current` → develop ✅; SDP `campaign_discover_skills_v2` (8 skills) + 4 skills cargadas | ✅ | bash/mcp/skill |
| 1 | Hallazgo: 2 intentos previos SÍ dejaron trabajo (40 archivos desktop + catálogo 800+ claves + tests en worktree sin commitear) — se continúa, no se rehace | ✅ | git diff |
| 2 | `codegraph_explore` blast radius (sync DISABLED por lock — se leyó fuente directa) + caza ES: lentes 0 matches, paneles solo comentarios | ✅ | codegraph/bash |
| 3 | Fix tsc: `useProjection(lang)` sin usar → clave nueva `space.noVectors` ES/EN + `tt()` en error + `SpaceLens` pasa `lang` | ✅ tsc 0 | edit/bash |
| 4 | Fix vitest: expectativa errónea `export.copiedRecords` ES ("records"→"registros") + asserts `space.noVectors` ES/EN | ✅ 11/11 | edit/bash |
| 5 | VERIFY cierre: tsc 0 + vitest i18n 11/11 + build 19.98s 0 errores; full suite 73/86 (13 fails pre-existentes `localStorage is not available` — entorno Node, ajenos al slice) | ✅ | bash |

## Contrato de verificación

- `cd desktop && npm run build` → 0 errores (solo warnings pre-existentes documentados)
- `cd desktop && npx tsc --noEmit` → exit 0
- `cd desktop && npx vitest run src/i18n` → verde
- `campaign_verify_cmd` con bug exit -1 conocido → bash directa (reportado)

## Deuda / WIP ajeno (NO TOCAR)

`M opencode.jsonc`, `M .opencode`, `M desktop/src-tauri/Cargo.lock`, `D docs/plans/2026-09-10-code.md`, `?? Investigacion-plan.md` — ajenos. Commit SOLO archivos del slice.
