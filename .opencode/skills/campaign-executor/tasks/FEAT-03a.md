# FEAT-03a — Consolidación asistida: UI (candidatos + diff visible + superseded_by)

> Plan: `docs/plans/2026-08-19-vanta-studio-fase4.md` (Task 17) · Estado: ⏳ PENDING → in-progress al delegar
> D16 (usuario 2026-08-20): **"Quiero todo"** — (a) UI + (b) core decay. Este task file cubre (a) UI-only.
> (b) core decay corre en paralelo por vanta-arch (task file FEAT-03b.md) — NO tocar core Rust.

## Contexto (verify del lead, 2026-08-20)
- W0..W3 parcial: FEAT-01 (retrieval slider) y FEAT-02 (Índices) commiteados. E2E standalone pasa.
- `desktop/src/components/` patrones: ImportPaste/ImportDrop (ingest), DataExplorer (grid MEMORIAS), IndicesLens (surface stats), RetrievalLens (search).
- `vanta_search` mapeado en 3 transports con explain (rrf_text_rank/rrf_vector_rank) — fuente de candidatos por similitud.
- `vanta_put`/`vanta_ingest` aceptan `metadata` arbitraria (serde_json::Value) → `superseded_by` es metadata de usuario.

## Contrato (plan Task 17, alcance (a) D16)
Nueva surface/lente de consolidación: detectar candidatos duplicados/superados por similitud (search kNN), diff visible entre pares, sugerencia "superado por" → escribir metadata `superseded_by` en el record superado (put/ingest). MVP textual primero (matriz de duplicados por embedding = iteración, no esta fase).

## Steps atómicos
1. **DISCOVERY** — leer `DataExplorer.tsx` (grid, fetchFirst/listPage), `RetrievalLens.tsx` (search + explain), `vanta.ts` (vanta_search/vanta_put/vanta_get), `vanta-wasm-map.ts`/`vanta-http-map.ts` (search/put/get). Verificar cómo el grid obtiene records (listPage por namespace).
2. Nueva surface/lente (patrón IndicesLens/RetrievalLens): "CONSOLIDAR" — selector de namespace (o usa el activo), detección: para cada record, search kNN sobre su text con los demás (top_k modesto) → pares candidatos con score de similitud.
3. Diff visible entre pares: texto completo de ambos lados + metadata; marcar visualmente similitud (p.ej. score %) — patrón visual del proyecto (font-tech, borders, neon).
4. Acción "marcar superado por": escribe `metadata.superseded_by = <id del record vigente>` en el record superado (vanta_put). El record vigente queda visible; el superado se marca en la UI (badge "superado por X").
5. Tests node:test (lógica pura de detección/diff/merge de metadata — NUNCA tocar core en este task file) + builds Tauri y WASM verdes.
6. Smoke: DB temp con duplicados → surface los marca con diff.

## Verificación (contrato del plan)
- `node --test src/*.test.ts` — todos verdes.
- `npm run build` + `npm run build:wasm` — verdes.
- Smoke: DB temp con duplicados → surface los marca con diff visible.
- Mecánico del lead post-delegación obligatorio. NO reportar PASS sin verlo.

## Contrato del plan (repetido para el RESULTADO)
- Surface CONSOLIDAR real en los 3 modos (Tauri/web/WASM donde aplique).
- Candidatos por similitud (search kNN) + diff visible + `superseded_by` escrito.
- MVP textual; matriz por embedding documentada como iteración (no esta fase).
- Tests + builds verdes, smoke real.