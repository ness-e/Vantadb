# ADMIN-09: Snapshot export + persistencia

## Metadata
- **Plan file:** docs/plans/2026-08-08-admin-console.md
- **Creado:** 2026-08-08T00:00
- **last-synced:** 2026-08-08T00:00
- **Estado:** ✅ COMPLETED

## Blast Radius
- Callers: App.tsx (render), ninguna otra línea depende de ExportPanel.
- Callees: vanta.ts `metrics()` → comando Tauri `vanta_metrics` (ADMIN-01).
- Implicaciones: localStorage key `vanta.last_snapshot` (frontend-only, sin escritura nativa).

## Contrato
"`npm run build` en desktop/ pasa; commit solo de desktop/src/; persistence en localStorage; carga al montar si el poll live aún no respondió."

## Herramientas
- bash (npm run build, git), read, write, edit.

## Steps
### Step 1: Leer contexto
- Leídos App.tsx, vanta.ts, package.json (no hay plugin-dialog/plugin-fs → blob download decide), KpiCards/SopPanel (patrón poll), App.css (tokens).
- **Estado:** ✅

### Step 2: Implementar
- `desktop/src/components/ExportPanel.tsx` (nuevo): fetch `metrics()` on mount → persiste `{at, snapshot}` en localStorage (`vanta.last_snapshot`); init state desde localStorage (fallback si el poll live aún no respondió); botón Export blob-download con filename timestamp ISO.
- `desktop/src/App.tsx`: import + render `<ExportPanel />` tras KpiCards.
- `desktop/src/App.css`: `.export`, `.export-row`, `.export-saved` (tokens existentes paper/ink/hard shadow).
- **Estado:** ✅

### Step 3: Verify
- `npm run build` → tsc + vite ✅ (1 fix: `useState<Stored | null>`).
- Commit: `e0e8ff3a` — solo 3 archivos desktop/src.
- **Estado:** ✅

## Dependencias
- ADMIN-01 (vanta_metrics), ADMIN-04/05 (poll grid), ADMIN-06 (patrón panel).

## Notas
- Snapshot exportado via `URL.createObjectURL` + `<a download>` porque package.json no instala `@tauri-apps/plugin-dialog` ni `plugin-fs`; el contrato permite frontend blob download — no se agregaron plugins.
- El TS interface `OperationalMetrics` declara el subset que lee la UI; el objeto en runtime trae todos los campos del wire Rust (JSON.stringify no pierde campos extra) → no se tocó vanta.ts.

## Context Save Point
- **Fecha:** 2026-08-08
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** Blob download en vez de plugins Tauri porque no están instalados y el contrato lo permite; localStorage (no archivo nativo) para persistence del último snapshot.
- **Problemas conocidos:** Ninguno.
- **Próxima tarea:** TBD