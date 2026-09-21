# FEAT-02 — Superficie Índices/salud (placeholder de VS-03 → real)

> Plan: `docs/plans/2026-08-19-vanta-studio-fase4.md` (Task 16) · Estado: ⏳ PENDING → in-progress al delegar

## Contexto (verify del lead, 2026-08-20)
- W0..W2 completas. FEAT-01 (slider RETRIEVAL) en curso/completa — NO tocar `desktop/src/components/lens/retrieval/` (disjunto con FEAT-02).
- Sidebar ÍNDICES es placeholder (VS-03:75). REST-02/REST-05 alimentan `namespace_stats` y `/api/v2/metrics`.
- E2E standalone pasa (selfcheck-wasm-e2e.ts), 53/53 tests base, builds verdes.

## Contrato (del plan, Task 16)
Surface ÍNDICES real: counts por namespace (REST-05), dims, hnsw_nodes_count, LSM/WAL status (si el core lo expone — verificar; si no, exponer wrapper mínimo), salud (health endpoint); charts simples (reuso de patterns ScoreBars); funciona en desktop (bridge) y web (REST).

## Steps atómicos
1. **DISCOVERY** — verificar qué expone hoy el core: `vanta_metrics` / `namespace_stats` / `/api/v2/metrics` (REST-02/REST-05), `src/sdk/` (hnsw_nodes_count, dims, LSM/WAL — verificar contra código real si existen o no). NO inventar métricas: si el core no la expone → documentar gap + follow-up (patrón "no mentir en UI").
2. UI: reemplazar placeholder ÍNDICES por surface real: counts por namespace, dims, hnsw_nodes_count, LSM/WAL status, salud (health). Charts simples reusando patterns de ScoreBars.
3. Funciona en desktop (bridge) y web (REST). Si una métrica core falta → gap documentado + follow-up.
4. Tests node:test (lógica pura de formateo/agregación si aplica) + builds Tauri y WASM verdes.
5. Smoke: sidebar muestra stats reales ≠ placeholder.

## Verificación (contrato del plan)
- `node --test src/*.test.ts` — todos verdes.
- `npm run build` + `npm run build:wasm` — verdes.
- Smoke: sidebar muestra stats reales ≠ placeholder; si falta métrica core → gap documentado.
- Mecánico del lead post-delegación obligatorio. NO reportar PASS sin verlo.

## Contrato del plan (repetido para el RESULTADO)
- Superficie ÍNDICES real en los 3 modos (Tauri/web/WASM donde aplique).
- Si falta métrica core → gap documentado + follow-up (NUNCA mentir en UI).
- Tests + builds verdes, smoke real.