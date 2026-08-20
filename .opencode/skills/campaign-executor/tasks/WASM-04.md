# WASM-04 — Drag&drop `.vdbdump`/JSONL (import de archivos reales)

> Plan: `docs/plans/2026-08-19-vanta-studio-fase4.md` (Task 14) · Estado: ⏳ PENDING → in-progress al delegar

## Contexto (verify del lead, 2026-08-20)
- OP-01 textarea pegado existe: `desktop/src/components/ingest/ImportPaste.tsx` — `parseImport(paste, ns)` → preview → `runImport(rows, ingestBatch)` → `onImported(count)`.
- `parseImport.ts`: `parseJson` + `parseCsv`, `MAX_IMPORT`, `runImport` con `CHUNK_SIZE` (50).
- `vanta_ingest_batch` ya mapeado en los 3 transports (Tauri `commands/data.rs`, HTTP `vanta-http-map.ts`, WASM `vanta-wasm-map.ts`) — **no inventar endpoint nuevo**: reusar `ingestBatch` de `desktop/src/vanta.ts`.
- El grid (MEMORIAS) se refresca vía `onImported`/`onRefresh` en WorkspaceShell (import individual NO remonta — solo ImportPaste usa `gridKey++`).
- E2E standalone: `desktop/scripts/selfcheck-wasm-e2e.ts` (node:http server + Playwright Edge). Ya existe (WASM-03).

## Contrato (del plan, Task 14)
Drop zone en MEMORIAS/import: arrastrar `.vdbdump`/`.jsonl`/`.csv` → parse (reuso parser OP-01) → preview → ingest. Modo WASM: leer File via File API (`FileReader`/`arrayBuffer`) → persistir (el transport ya persiste tras cada mutación). Modo server/web: el mismo `ingestBatch` (multipart/base64 NO necesario — no existe endpoint import dedicado).

## Steps atómicos
1. **DISCOVERY** — leer `ImportPaste.tsx`, `parseImport.ts`, `WorkspaceShell.tsx` (línea ~733 onNotice / 734-742 DataExplorer gridKey / 837 gridKey++), `vanta.ts` (ingestBatch), `vanta-wasm-map.ts` (vanta_ingest_batch). Verificar qué formatos soporta `parseImport` hoy (CSV/JSON/NDJSON) y si `.vdbdump` necesita un parser propio (Qdrant-style `{id, vector, payload}` → mapear a `IngestItem`).
2. Crear `desktop/src/components/ingest/ImportDrop.tsx` reusando `parseImport`:
   - Drop zone (drag over/leave/drop + `onDrop` con `dataTransfer.files`) + input file `<input type="file" accept=".csv,.json,.jsonl,.vdbdump">` como fallback accesible.
   - Leer archivo: `file.text()` (File API — suficiente para CSV/JSONL; no requiere File System Access ni FileReader en el flujo WASM).
   - Extensiones: `.json`/`.jsonl`/`.vdbdump` → parseJson (NDJSON ya soportado si parseJson maneja array o NDJSON); `.csv` → parseCsv. `.vdbdump` = NDJSON VantaDB o Qdrant-style → mapear.
   - Preview + reporte: reusar el mismo patrón de `ImportPaste` (misma tabla, mismos botones, mismo `runImport(rows, ingestBatch)` + `onImported`).
   - Nombre del archivo visible en la zona; re-drop permite re-importar.
3. Integrar en `WorkspaceShell.tsx`: botón "IMPORT" junto al existente (o el mismo modal con tabs Pegar/Archivo — decidir el menor cambio; el plan pide "drop zone en MEMORIAS/import", no un modal nuevo obligatorio). El grid refresca con `onImported` existente.
4. Tests node:test: parser reusado + `ImportDrop`-logic (parse de un `.vdbdump`/`.jsonl`/`.csv` de ejemplo → mismos rows que paste; `runImport` verde). No testear DOM (sin jsdom en el proyecto) — testear la lógica pura.
5. E2E standalone: extender `selfcheck-wasm-e2e.ts` o script nuevo — drop file real (Playwright `setInputFiles` sobre el input file) → records en grid. Verificar también en modo server web si el contrato lo exige (prioridad: WASM).
6. Docs: `docs/api/WASM_STANDALONE.md` — añadir drag&drop a "What works" (si aplica a modo wasm).

## Verificación (contrato del plan)
- E2E: drop file real → records en grid.
- `node --test src/*.test.ts` — todos verdes (41 + nuevos).
- `npm run build` (Tauri) y `npx vite build --mode wasm` — verdes.
- Mecánico del lead post-delegación obligatorio.

## Contrato del plan (repetido para el RESULTADO)
- Drop zone funcional en los 3 modos (Tauri/web/WASM) con el mismo `ingestBatch`.
- `.vdbdump`/`.jsonl`/`.csv` parseados con reuso de OP-01; fallos de parse NUNCA silenciosos (patrón ImportPaste: alert + filas marcadas).
- E2E PASS real (NO reportar PASS sin verlo — el lead verifica el script y el exit code).