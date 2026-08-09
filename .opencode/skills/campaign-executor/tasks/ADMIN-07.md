# ADMIN-07: Data Explorer — panel de exploración de registros con paginación

## Metadata
- **Plan file:** docs/plans/2026-08-08-admin-console.md
- **Creado:** 2026-08-08
- **last-synced:** 2026-08-08
- **Estado:** ✅ COMPLETED

## Blast Radius
- `desktop/src/components/DataExplorer.tsx` — NUEVO: browse (`vanta_list`) + semantic search (`vanta_search` con score) sobre la conexión activa, tabla con id/ns/text/score, "Load more".
- `desktop/src/App.tsx` — `<DataExplorer active busy runError />` montado full-width entre el grid y el footer.
- `desktop/src/App.css` — estilos `.explorer-table`/`.explorer-text`/`.explorer-more` (tokens ADMIN-03: paper, borde 2px/3px ink, radius 0, sombra dura).
- Core Rust: **NO tocado** — verificada la firma real de `vanta_list` (data.rs:65): `(namespace, limit)` sin offset/cursor.

## Contrato
"`npm run build` en `desktop/` ✅; Data Explorer lista records de la conexión activa con paginación (limit creciente) y muestra score en modo search."

## Pasos
### Step 1: Leer fuentes — ✅
- `App.tsx` (65 líneas original), `App.css` (tokens cream/ink/neon/paper, `.panel` 3px + sombra 6px, `.results`/`.score` existentes), `vanta.ts` (bridge con `list({namespace, limit})` + `search`), `commands/data.rs` (`vanta_list`). `manager.rs:192` (`list_records(namespace, limit.unwrap_or(100))`).
- `SopPanel.tsx` apareció como WIP concurrente (otra línea, commit ADMIN-06).

### Step 2: Zero-code planning — ✅
1. Core pagination: NO soporta offset/cursor (trait `VantaConnection::list` + adapters native/server + manager solo aceptan `namespace`+`limit`) → no tocar Rust (≫5 líneas).
2. DataExplorer: input query → vacío = browse (`vanta_list`), texto = search (`vanta_search` con score). Paginación: "Load more" crece el limit 50→100→… reemplazando la lista (sin offsets, sin duplicados).
3. Montar full-width en App.tsx; CSS tabla con estilo del design system.

### Step 3: Implementar — ✅
- **DataExplorer.tsx** (146 líneas): estado `query/rows/limit/loading/mode`; `fetchRows(kind, q, lim)` normaliza `MemoryRecord`/`SearchResult` a `Row {id, namespace, text, score|null}`; `useEffect` browse al montar/cuando aparece conexión; submit → search si query no vacío; `Load more (+50)` crece limit; estados vacío/error/no-conexión.
- **App.tsx**: `import DataExplorer` + `<DataExplorer active={!!state.active} busy={state.busy} runError={reportError} />`.
- **App.css**: `.explorer-table` (collapse, borde 2px ink, th uppercase paper), `.explorer-text` (ellipsis 420px), `.explorer-more`.

### Step 4: Verify — ✅
- `npx tsc --noEmit --strict ... DataExplorer.tsx` → EXIT 0 (validación propia sin SopPanel).
- `npm run build` (workdir desktop) → `tsc && vite build` EXIT 0, **built in 1.87s**, 41 modules, sin errores TS. (El primer intento falló por (a) mi `r.score` nullable, (b) SopPanel.tsx roto; ambos resueltos — SopPanel lo arregló la sesión concurrente.)

### Step 5: Commit — ✅
- Commit `7a19a9f5` — `feat(ADMIN-07): data explorer with pagination for active connection` — 1 archivo, 146 insertions: `desktop/src/components/DataExplorer.tsx`.
- Pre-commit hook: sin Rust staged → cargo skip; actionlint ok.
- App.tsx/App.css ya estaban en HEAD vía commit concurrente ADMIN-06 `f20d67a4` (barrió mis ediciones junto con su SopPanel al commitear antes que yo — mismo patrón documentado en ADMIN-05). Mi commit contiene SOLO el componente nuevo, que es el que completaba el árbol.

## Dependencias
- ADMIN-03 (design system light) — estilos reutilizados.
- ADMIN-05 (KpiCards) — patrón de componente autónomo.
- ADMIN-06 (SopPanel) — convivió en App.tsx concurrentemente.

## Notas
- **Decisión de paginación (documentada):** el core NO soporta offset/cursor — `vanta_list`/`list_records`/`VantaConnection::list` solo aceptan `name+limit`. Real paginación requeriría cambiar el trait de conexión, los 2 adapters y el engine (≫5 líneas, fuera de alcance). Se usa **limit creciente con "Load more"**: 50→100→200…, cada fetch reemplaza la lista — muestra más records sin duplicados ni offsets inventados. `ponytail:` comment en el componente: "Replace with real offset/cursor when the core exposes one."
- **Colisión con sesión paralela ADMIN-06:** otra sesión trabajaba los mismos archivos (App.tsx/App.css/SopPanel.tsx). Su commit `f20d67a4` incluyó mis ediciones de App.tsx/App.css (estaban en el working tree). Mi commit final solo contiene `DataExplorer.tsx`. El estado del árbol queda COHERENTE: HEAD importa+renderiza DataExplorer y `DataExplorer.tsx` existe en HEAD.
- **Score:** `vanta_list` no devuelve score (solo `vanta_search`). La columna score aparece solo en modo search. En browse, `"—"` no se renderiza.
- Concurrencia: sesión concurrente activa sobre el mismo plan (admin-console). No tocar SopPanel.tsx una vez mergeado.

## Context Save Point
- **Fecha:** 2026-08-08
- **Branch:** develop
- **Commit:** 7a19a9f544ffe48a2295b78af3a19ddd7bb8a6fd
- **CI pendiente:** no (npm run build local ✅; sin Rust en el commit)
- **Decisiones:** paginación = limit creciente + "Load more" (sin offset porque el core no lo expone; documentado en el task file y en un comentario `ponytail:` del componente). Modo browse (list) vs search (score) según query vacía/no. Score solo en modo search (list no lo devuelve).
- **Problemas conocidos:** sesiones paralelas mutando App.tsx/App.css — commit ADMIN-06 incluyó mis cambios de montaje; si se reworkea App.tsx, re-verificar el montaje de DataExplorer. El estado del plan file lo actualiza la sesión orquestadora.
- **Próxima tarea:** siguiente ADMIN en docs/plans/2026-08-08-admin-console.md (verificar con `campaign_get_next_task`).