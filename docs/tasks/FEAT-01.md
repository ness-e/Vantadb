# FEAT-01 — Slider de pesos híbridos BM25/vector en RETRIEVAL

> Plan: `docs/plans/2026-08-19-vanta-studio-fase4.md` (Task 15) · Estado: ⏳ PENDING → in-progress al delegar

## Contexto (verify del lead, 2026-08-20)
- W0..W2 completas. `vanta_search` ya mapeado en 3 transports (Tauri/HTTP/WASM).
- Barra RETRIEVAL: `desktop/src/components/lens/retrieval/` (RetrievalLens.tsx, ScoreBars.tsx) — texto + vector-picker + filtros existen; falta SOLO el slider de pesos (SYNTHESIS §4 RETRIEVAL:123).
- El E2E standalone pasa (selfcheck-wasm-e2e.ts, 53/53 tests, builds verdes).

## Contrato (del plan, Task 15)
Verificar si `search`/`hybrid_search` acepta `alpha`/pesos (BM25 vs vector); si no, exponerlo (core aditivo si es trivial, sino REST wrapper). UI: slider (0=BM25 puro, 1=vector puro, default=RRF/50) en barra RETRIEVAL; ScoreBars reflejan el peso activo; tests + build verde.

## Steps atómicos
1. **DISCOVERY** — leer `RetrievalLens.tsx`, `ScoreBars.tsx`, `desktop/src/vanta.ts` (vanta_search), `desktop/src/vanta-http-map.ts` + `desktop/src/vanta-wasm-map.ts` (search), `desktop/src-tauri/src/connections/` (search / hybrid). Verificar si el core (`src/sdk/api.rs` search/hybrid_search) acepta alpha/weights HOY — si no, verificar cuánto costaría exponerlo (Regla 0 del lead: validar contra docs/fuente real; NO inventar parámetros que el core no soporta).
2. Si el core NO acepta pesos: NO tocar core a menos que el cambio sea trivial y aditivo. Prioridad: wrapper/REST si el server ya expone algo; si ni eso, **documentar el gap** en el task file + follow-up, y aun así entregar la UI (slider que mapea a RRF/búsqueda existente con el valor documentado como "peso planificado" NO mentir en UI — ver FEAT-02 pattern "no mentir en UI").
3. UI: slider en barra RETRIEVAL (0=BM25, 1=vector, default=50=RRF); ScoreBars reflejan el peso activo (tooltip/label con el peso actual); los resultados se re-buscan al cambiar el slider (debounce razonable).
4. Tests node:test para la lógica pura (mapeo slider→parámetro, si aplica) + build Tauri y WASM verdes.
5. Smoke manual: slider cambia resultados visiblemente en DB temp (o documentar por qué no puede).

## Verificación (contrato del plan)
- `cargo test` core verde (si toca core).
- `node --test src/*.test.ts` — todos verdes.
- `npm run build` + `npm run build:wasm` — verdes.
- Smoke: slider cambia resultados visiblemente (o gap documentado).
- Mecánico del lead post-delegación obligatorio. NO reportar PASS sin verlo.

## Contrato del plan (repetido para el RESULTADO)
- Slider funcional en la barra RETRIEVAL en los 3 modos (Tauri/web/WASM).
- Si el core no soporta pesos → gap documentado + follow-up (NUNCA mentir en UI).
- Tests + builds verdes, smoke real.