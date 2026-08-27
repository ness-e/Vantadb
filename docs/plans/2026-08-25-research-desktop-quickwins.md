# Plan — Quick wins INV-desktop-prod (research-desktop-prod-20260825)

> **Origen:** `/research desktop` 2026-08-25 · Informe: `docs/reviews/research-desktop-prod-20260825.md`
> Score global 7.4/10 · 15 hallazgos · Aprobado HITL (Fase D): los 10 ítems de este plan
> fueron seleccionados explícitamente por el owner. Estrategia → Backlog P44.
> **Verificación mecánica del plan:** `cd desktop && npm run build && npm test` verde en cada wave
> (+ `cargo check -p vantadb` si H-04 toca bridge Rust) + `npx playwright test` para H-07.

## Wave 1 — Cosméticos/UX <1h c/u

| # | Tarea | Origen | Archivos clave |
|---|-------|--------|----------------|
| 1 | CommandPalette: sincronizar union completa Surface (verificar `memoria`/`proxy`/`ajustes` presentes) | H-02 | `desktop/src/components/palette/CommandPalette.tsx`, `WorkspaceShell.tsx` |
| 2 | Handler keydown global F1/F2 → HelpPanel | H-03 | `desktop/src/components/layout/HelpPanel.tsx`, `WorkspaceShell.tsx` |
| 3 | `statusReport.ts` markdown EN→ES (consistente con UI) | H-05 | `desktop/src/components/export/statusReport.ts` + tests |
| 4 | Botón FILTROS: activo = reglas >0 (`filterActive`) — decisión owner DAUD-02 cerrada 2026-08-25 | H-14 | `desktop/src/components/layout/WorkspaceShell.tsx` (topbar) |
| 5 | Limpiar filas DAUD-01..09 stale del Backlog (commits ya aplicados: `3c53d8b2`,`480935a7`,`b865c625`; DAUD-02 resuelta por tarea 4; DAUD-08 stash recuperada por `b865c625`) | H-13 | `docs/Backlog.md` P37 |

## Wave 2 — Seguridad + integridad de datos

| # | Tarea | Origen | Archivos clave |
|---|-------|--------|----------------|
| 6 | CSP mínima en tauri.conf.json (`default-src 'self'` + connect-src localhost/remoto según transporte). Fuente: https://v2.tauri.app/security/csp/ — validar app tras cambio (E2E flujo-critico debe pasar) | H-01 🔴 | `desktop/src-tauri/tauri.conf.json:24-26` |
| 7 | Rename namespace preserva `sparse_vector` (copiar campo en el ingestBatch del rename + test que lo fije) | H-04 🟠 | `desktop/src/vanta.ts` (rename flow), `store/connections.test.ts` |

## Wave 3 — Release/medición/E2E

| # | Tarea | Origen | Archivos clave |
|---|-------|--------|----------------|
| 8 | Sincronizar versión desktop con release-plz (o excluirla documentadamente): `package.json:4` + `tauri.conf.json:4` vs tags workspace | H-11 | `desktop/package.json`, `desktop/src-tauri/tauri.conf.json`, release-plz.toml |
| 9 | Baseline medido de recursos del app (startup time, RAM idle) registrado en `docs/operations/BENCHMARKS.md` §Desktop (Regla 11: reemplaza estimación de plataforma DESKTOP-01) | H-15 | `docs/operations/BENCHMARKS.md` |
| 10 | E2E desktop: specs nuevas multi-perfil conexión + proxy dashboard (mock upstream); graph/space quedan smoke visual manual documentado | H-07 | `desktop/e2e/`, `desktop/playwright.config.ts` (config a raíz del paquete — lesson 2026-08-25) |

## Fuera del plan (Backlog P44)

- DESKTOP-40 i18n real ES/EN (H-06) · DESKTOP-41 smoke VM instalador (H-08) · DESKTOP-42 bundles macOS/Linux baja prioridad (H-09) · DESKTOP-43 auto-updater tras firma (H-10)
- H-12 validación manual proxy requiere upstream LLM vivo — ejecutarla como sesión guiada con el owner, no tarea autónoma (queda anotada en P44).

=== RECITATION DESKTOP-QW1 ===
Campaign ID: 39b59c48-a98a-40bd-bc5d-149dd5191263
Objetivo activo: DESKTOP-QW1 — CommandPalette union sync H-02
Estado: completed
Última acción: Steps 1-3 COMPLETED — auditoría unions idénticas (no edición), build 8.04s + tests 69/69 verde, fmt verde, lessons escritas, ready to commit
Resultado: OK
Próxima acción: git add task file + commit feat(desktop): DESKTOP-QW1 — CommandPalette union sync (H-02) + progreso Trigger 1
Contrato: verificacion: npm --prefix desktop run build (8.04s, 2863 modules) ✅ + npm --prefix desktop test (69/69) ✅ + cargo fmt --check ✅; evidencia: PaletteSurface === Surface (12 valores) en CommandPalette.tsx:27-39 y WorkspaceShell.tsx:83 con memoria/proxy/ajustes en Lentes palette (265-287); artefactos: .opencode/skills/campaign-executor/tasks/DESKTOP-QW1.md, desktop/dist/; invariantes: no romper palette grupos (Navegación/Lentes/Favoritos/Historial) ✅; deuda: ninguna
Próxima tarea si completa: DESKTOP-QW2
=== END RECITATION ===
=== RECITATION DESKTOP-QW2 ===
Campaign ID: 39b59c48-a98a-40bd-bc5d-149dd5191263
Objetivo activo: DESKTOP-QW2 — Handler keydown global F1/F2 → HelpPanel (H-03)
Estado: completed
Última acción: Steps 1-4 COMPLETED — auditoría handler existente + tabs F1/F2 implementados (HelpPanel HelpTab + WorkspaceShell helpTab state), build 11.05s (2863 modules) + tests 69/69 (17.31s) + cargo fmt --check verde, lessons escritas
Resultado: OK
Próxima acción: git add task file + HelpPanel/WorkspaceShell + commit feat(desktop): DESKTOP-QW2 — Handler F1/F2 → HelpPanel con tabs contextuales (H-03) + progreso Trigger 1
Contrato: verificacion: npm --prefix desktop run build (11.05s, 2863 modules) ✅ + npm --prefix desktop test (69/69) ✅ + cargo fmt --check ✅; evidencia: WorkspaceShell keydown F1→general / F2→proxy con skip inputs + preventDefault (líneas 347-372) + HelpPanel initialTab prop + tabs UI (HelpPanel.tsx:8-35,146 líneas) con SURFACES 12 (faltaban ACTIVIDAD/MEMORIA/PROXY/AJUSTES) y SHORTCUTS split F1/F2; artefactos: .opencode/skills/campaign-executor/tasks/DESKTOP-QW2.md, desktop/src/components/layout/HelpPanel.tsx, desktop/src/components/layout/WorkspaceShell.tsx, desktop/dist/; invariantes: no romper palette/sidebar/inspector, E2E flujo-critico no regresa ✅; deuda: ninguna
Próxima tarea si completa: DESKTOP-QW3
=== END RECITATION ===
=== RECITATION DESKTOP-QW3 ===
Campaign ID: 2fcc9094-fcac-45ea-a585-8c058b5f8c9d
Objetivo activo: DESKTOP-QW3 — statusReport.ts markdown EN→ES (H-05)
Estado: completed
Última acción: Steps 1-4 COMPLETED — auditoría EN→ES (10 literales, loanwords Namespace/Key preservados) + traducción ya en a7ed0d22 verificada + tests ES actualizados + build 10.29s (2863 modules) + tests 69/69 (20.79s) + cargo fmt --check verde, lessons escritas
Resultado: OK
Próxima acción: git add task file + commit feat(desktop): DESKTOP-QW3 — statusReport.ts markdown EN→ES (H-05) + progreso Trigger 1
Contrato: verificacion: npm --prefix desktop run build (10.29s, 2863 modules) ✅ + npm --prefix desktop test (69/69) ✅ + cargo fmt --check ✅; evidencia: statusReport.ts 10 literales ES (# Reporte de estado VantaDB, Generado:, Registros en vista:, ## Campos de metadata, Sin campos…, | Campo | Tipo |, ## Expiraciones próximas, Ningún registro expira…, | Key | Namespace | Expira | En |) + status-report.test.ts assertions ES (Reporte de estado, Generado, Registros, Campos de metadata, Expiraciones próximas) + loanwords Namespace/Key coherentes con UI (HelpPanel namespaces, DataExplorer Key); artefactos: .opencode/skills/campaign-executor/tasks/DESKTOP-QW3.md, desktop/src/components/export/statusReport.ts (HEAD a7ed0d22), desktop/dist/; invariantes: no romper ExportButtons handleReport, no tocar WAL/vector/storage, E2E flujo-critico no regresa ✅; deuda: ninguna
Próxima tarea si completa: DESKTOP-QW4
=== END RECITATION ===
=== RECITATION DESKTOP-QW4 ===
Campaign ID: 2fcc9094-fcac-45ea-a585-8c058b5f8c9d
Objetivo activo: DESKTOP-QW4 — Botón FILTROS activo = reglas >0 (H-14, DAUD-02)
Estado: completed
Última acción: Steps 1-4 COMPLETED — auditoría filterActive DAUD-02 cerrada 2026-08-25 (reglas>0, no panel) + skip leaf robustez YAGNI + build 14.63s (2863 modules) + tests 69/69 (18.88s) + cargo fmt --check verde, lessons escritas
Resultado: OK
Próxima acción: git add task file + commit feat(desktop): DESKTOP-QW4 — Botón FILTROS activo = reglas >0 (H-14, DAUD-02) + progreso Trigger 1
Contrato: verificacion: npm --prefix desktop run build (14.63s, 2863 modules) ✅ + npm --prefix desktop test (69/69) ✅ + cargo fmt --check ✅; evidencia: WorkspaceShell.tsx:295 filterActive=ruleGroup.rules.length>0 + aria-pressed={filterActive} (744) + FIX-D4 bg-foreground (747) + badge toVantaMemoryFilter(...).length (751) + comentario DAUD-02 (738-739) + showFilters desacoplado (solo panel 803) + grep showFilters?bg 0 hits; artefactos: .opencode/skills/campaign-executor/tasks/DESKTOP-QW4.md, desktop/dist/; invariantes: no romper topbar/visibleResults/runSearch top_k 50, no tocar WAL/vector/storage, E2E 2 specs no regresan ✅; deuda: leaf robustez toVantaMemoryFilter deferred 1 línea (YAGNI, builder validation impide regla vacía) + RetrievalLens bg-neon vs bg-foreground divergencia fuera de scope
Próxima tarea si completa: DESKTOP-QW5
=== END RECITATION ===
=== RECITATION DESKTOP-QW5 ===
Campaign ID: 2fcc9094-fcac-45ea-a585-8c058b5f8c9d
Objetivo activo: DESKTOP-QW5 — Limpiar filas DAUD-01..09 stale del Backlog (H-13)
Estado: completed
Última acción: Steps 1-4 COMPLETED — auditoría 9/9 Hecho (commits 3c53d8b2,480935a7,b865c625 + ad0f34b1 QW4; stash 06aa1a86 0 hits) + Backlog P37 9 filas→0 (Exec Summary 118→109, last_reviewed 2026-08-26, P37 colapsada a Cerrada) + dominio desktop.md §P37 + backlog-history.md §Limpieza DAUD + scripts 0 gaps + cargo fmt verde
Resultado: OK
Próxima acción: git add docs/Backlog.md docs/avance/activo/desktop.md docs/avance/historial/backlog-history.md docs/plans/2026-08-25-research-desktop-quickwins.md .opencode/skills/campaign-executor/tasks/DESKTOP-QW5.md + commit docs(backlog): DESKTOP-QW5 — limpiar filas DAUD-01..09 stale (H-13) + historial desktop
Contrato: verificacion: Select-String "^\| \`DAUD-" 0 hits (Backlog sin filas stale) ✅ + pwsh scripts/check-avance-coverage.ps1 1038/1038 0 gaps ✅ + pwsh scripts/validate-docs-coverage.ps1 0 gaps ✅ + cargo fmt --check ✅; evidencia: Backlog.md P37 0 — ✅ 9/9 ejecutadas (3c53d8b2,480935a7,b865c625; DAUD-02 via ad0f34b1) + last_reviewed 2026-08-26 + Total 109 + P37 Cerrada 2026-08-26; backlog-history.md §Limpieza DAUD 9 rows; desktop.md §P37 9/9 detalle; artefactos: docs/Backlog.md, docs/avance/activo/desktop.md, docs/avance/historial/backlog-history.md, docs/plans/2026-08-25-research-desktop-quickwins.md, .opencode/skills/campaign-executor/tasks/DESKTOP-QW5.md; invariantes: no romper Backlog estructura, no gaps cobertura, no código/docs/api touched ✅; deuda: ninguna
Próxima tarea si completa: DESKTOP-QW6
=== END RECITATION ===
=== RECITATION DESKTOP-QW6 ===
Campaign ID: 2fcc9094-fcac-45ea-a585-8c058b5f8c9d
Objetivo activo: DESKTOP-QW6 — CSP mínima en tauri.conf.json (H-01)
Estado: completed
Última acción: Steps 1-4 COMPLETED — auditoría CSP ya mínima desde a7ed0d22 (default-src 'self' + connect-src ipc+127.0.0.1) + gaps prod localhost/https identificados (ProxyDashboard fetch a localhost/https remoto) + edit 2 líneas connect-src (prod + dev) añadiendo http://localhost:* ws://localhost:* https://* + build 21.18s (2863 modules) + tests 69/69 (18.55s) + cargo check 26.30s + fmt verde, lessons escritas
Resultado: OK
Próxima acción: git add desktop/src-tauri/tauri.conf.json docs/plans/2026-08-25-research-desktop-quickwins.md .opencode/skills/campaign-executor/tasks/DESKTOP-QW6.md + commit feat(desktop): DESKTOP-QW6 — CSP mínima localhost+https remoto (H-01) + progreso Trigger 1
Contrato: verificacion: tauri.conf.json csp.default-src 'self' ✅ + csp.connect-src "ipc: http://ipc.localhost http://127.0.0.1:* ws://127.0.0.1:* http://localhost:* ws://localhost:* https://*" ✅ + devCsp same +https ✅ + npm --prefix desktop run build (21.18s, 2863 modules) ✅ + npm --prefix desktop test (69/69) ✅ + cargo check -p vantadb (26.30s) ✅ + cargo fmt --check ✅; evidencia: desktop/src-tauri/tauri.conf.json:27,33 connect-src ampliado + ProxyDashboard fetchSnapshot solo fetch WebView + ServerClient reqwest Rust no CSP + Tauri docs https://v2.tauri.app/security/csp/ ejemplo ipc: http://ipc.localhost + threat model default-src self bloquea XSS; artefactos: desktop/src-tauri/tauri.conf.json, docs/plans/2026-08-25-research-desktop-quickwins.md, .opencode/skills/campaign-executor/tasks/DESKTOP-QW6.md, desktop/dist/; invariantes: no romper Tauri schema, no tocar WAL/vector/storage, E2E flujo-critico no regresa (fetch proxy no bloqueado) ✅; deuda: ninguna (CSP mínima completa, https://* es mínima para proxyUrl user-controlled)
Próxima tarea si completa: DESKTOP-QW7
=== END RECITATION ===
