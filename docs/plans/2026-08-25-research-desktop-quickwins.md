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
