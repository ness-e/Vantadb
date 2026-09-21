# DESKTOP-35: Slider híbrido cableado a search_profile real — eliminar re-rank client-side

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24T00:00:00
- **Estado:** ✅ COMPLETADA (sin commit — instrucción explícita del orquestador)

## Impacto mapeado (Regla 0)
- **Leídos completos:** `src/sdk/types.rs` (SearchProfileConfig/SearchProfileMode :460-488), `desktop/src-tauri/src/connections/types.rs` (SearchQuery DTO :56-76 + tests), `native.rs` (search_request :411-434), `server.rs` (search :203-224, sin explain/profile), `commands/data.rs`, `desktop/src/vanta.ts` (search :240), `vanta-http-map.ts` (searchToRequest :111-123), `RetrievalLens.tsx`, `retrieval-core.ts`, `ScoreBars.tsx`, `retrieval-core.test.ts`, `selfcheck-retrieval.ts`
- **Referencias entrantes:** RetrievalLens → rerankByWeight/weightFromSlider/computeSegmentsWeighted; ScoreBars → computeSegmentsWeighted(alpha); retrieval-core.test.ts cubre las 4 fns weighted
- **Hallazgo clave DISCOVERY:** el core NO tiene `bm25_weight` — `SearchProfileConfig = {mode: keyword|vector|hybrid, rrf_k?, candidate_k?}`. Pesos intermedios del slider NO expresables server-side → slider pasa a 3 stops discretos (0=keyword, 50=hybrid RRF, 100=vector); se elimina el re-rank client-side completo; gap documentado en código y UI
- **Paridad por construcción:** explain del server y resultados del slider usan el MISMO request (`explain: true` + `search_profile`) vía native.rs → parity garantizada
- **Veredicto:** cambio aditivo en bridge (serde default ⇒ backward compatible), deletions en retrieval-core/ScoreBars/RetrievalLens. Server transport (relational IQL) no soporta profile ni explain hoy — campo ignorado, ya documentado

## Blast Radius
Callers: desktop/src/components/RetrievalLens.tsx, desktop/src/components/retrieval-core.ts
Callees: desktop/src/vanta.ts (search_memory con search_profile), src/sdk/api.rs (MEM-01/02 search profile)
Implicaciones: Resultados del slider == explain del server (RRF real, no aproximación local)

## Spec
N/A — feature search con contrato mecánico

## Contrato
`cd desktop && npm run build`; resultados del slider == explain del server (RRF real)

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: ✅ Verificar search_profile en core + bindings (DISCOVERY — completado)
- **Resultado:** `SearchProfileConfig` existe en `src/sdk/types.rs:478` con `mode`/`rrf_k`/`candidate_k` (serde lowercase; re-export vía `vantadb::`). NO hay `bm25_weight`. Bridge desktop (`SearchQuery` DTO + native.rs) aún no lo pasa.

### Step 2: ✅ Cablear RetrievalLens slider → search_profile
- **Archivos:** `desktop/src-tauri/src/connections/types.rs` (SearchQuery.search_profile + test roundtrip/default), `native.rs` (search_request mapea el campo + test paridad), `desktop/src/vanta.ts` (VantaSearchProfile + SearchQuery.search_profile), `vanta-http-map.ts` (searchToRequest lo reenvía), `retrieval-core.ts` (eliminado re-rank: weightFromSlider/weightedScore/rerankByWeight/computeSegmentsWeighted → fusionModeFromSlider), `RetrievalLens.tsx` (slider discreto step=50, envía search_profile, re-ejecuta search al cambiar, sin re-rank), `ScoreBars.tsx` (sin alpha)
- **Decisión DISCOVERY:** el core NO tiene `bm25_weight` — pesos intermedios no expresables server-side. Slider discreto 0=keyword / 50=hybrid / 100=vector; gap documentado en retrieval-core.ts y UI
- **Verify:** `cargo check --all-targets` ✅ · `npm run build` ✅

### Step 3: ✅ Parity slider == explain server
- **Por construcción:** resultados del slider y explain usan el MISMO request (`explain: true` + `search_profile`) vía native.rs. Test Rust `search_profile_matches_default_and_keyword_mode`: perfil hybrid explícito == baseline sin perfil (mismos hits/scores) + keyword mode nunca produce vector ranks.
- **Verify:** `cargo test` (src-tauri): 79 passed ✅ · `npm test`: 64/64 ✅ · `node --test src/retrieval-core.test.ts`: 5/5 ✅ · selfcheck-retrieval: PASS ✅

## Dependencias
- MEM-01/02 (search profile mode/rrf_k/candidate_k por request) — ya implementados en core+MCP (campaña P27)
- VS-CORE-03 (explain estructurado) — ya completada

## Notas
- DoD: resultados del slider == explain del server (RRF real, no aproximación local)
- Core tenía RRF fijo; MEM-01/02 ya permiten search_profile por request
- Eliminar re-rank client-side en retrieval-core.ts