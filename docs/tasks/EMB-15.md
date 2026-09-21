# EMB-15 — embed de query con el MISMO proveedor + prueba sinónimos

> **Plan:** `docs/plans/2026-09-16-embeddings-auto.md` (Wave4, tras EMB-14 ✅; paralela EMB-17 salvo colisión tools.rs → secuencial interno)
> **Estado:** ⏳ IN PROGRESS 2026-09-16
> **Appetite:** 1d · 🟡 · 🟠
> **Branch/Commit:** develop / `feat: EMB-15`
> **Cynefin:** 🟨 complicado · ⬆️ 0 / ⬇️ 3 steps
> **Ruta:** vanta-worker
> **SDP:** campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design, systematic-debugging (frontend-ui-engineering DESCARTADA: sin `web/`, scope discipline)

## 1. TAREA

**Objetivo:** `search_memory`/`memory_search` con texto embeben la query con el proveedor activo (el MISMO que guardó los docs vía EMB-14) y `memory_recall` deja de pasar hook `None` al auto-recall; prueba de sinónimos felino↔gato sin palabras comunes; keyword como fallback solo si no hay proveedor, avisado; ranking RRF intacto.

**Contrato exacto (ley):**
1. `search_memory`/`memory_search`/`search_with_method`/`search_multi` con `text_query` y sin `query_vector` → embeben la query con `embed_query` del proveedor activo (semántica query, prefijo `query:` en e5 vía EMB-16) y buscan híbrido (mismo dual-pool+RRF D38, intacto).
2. `query_vector` provisto se respeta byte-exacto (no re-embed); su dim-check MCP-04/EMB-18 sigue rechazando con `dim_mismatch_guidance` verbatim.
3. `memory_recall` pasa hook de embed real (`embed_query`) a `perform_auto_recall` en vez de `None`; sin proveedor → `None` + `effective_mode: "keyword"` (aviso existente, hoy siempre keyword).
4. Fallo/ausencia de proveedor → keyword honesto + `warn!` en logs (nunca error duro, nunca dummy). En search la forma de respuesta (hits array) NO cambia (R-4/back-compat MCP-39); el aviso es log + documentado.
5. Prueba de sinónimos: query sin palabras comunes con el doc (`felino descansa` vs `el gato duerme en el sofá`) lo recupera con modelo real; sin modelo el test degrada graceful (patrón EMB-13/14).
6. Tests nuevos (RED→GREEN) + suite existente verde. `src/llm.rs` SOLO lectura (usa `embed_query` existente de EMB-16).

**Acceptance del plan:** hoy query MCP pasa hook `None` (`tools.rs:1709`) → recall degrada a keyword (D38); `parse_search_request` nunca embebe → texto-solo es keyword aunque los docs tengan vector (peras con manzanas). Pre-mortem: batch donde aplique (query es 1 solo texto → 1 `embed_query`; no hay batch que aplicar); no tocar ranking RRF.

## 2. ARCHIVOS

**Clave (único archivo prod en escritura):**
- `vantadb-mcp/src/handlers/tools.rs` — hunk A `parse_search_request` (~`:3085-3100`, tras dim-check de provisto: auto-embed de query si vector vacío + texto no-vacío) · hunk B `memory_recall` (`:1709`, `None` → hook real) · hunk C helpers al final (`try_embed_query` + `query_embed_hook`, espejo EMB-14 `try_provider_embed`, cfg-gated)
- `vantadb-mcp/tests/test_query_embed.rs` (NUEVO, patrón `test_auto_embed.rs`: graceful-skip sin modelo, asserts cfg-gated)

**Relacionados (solo lectura):**
- `src/llm.rs:31-53` (trait `embed` doc-semántica / `embed_query` query-semántica + default; factory `get_embedding_provider`), `:539-554` (`LocalOnnxProvider`: `embed_query`→`EmbedKind::Query`)
- `src/sdk/search/mod.rs:108-128` (modo Hybrid default; keyword ignora vector — el fallback es comportamiento core existente), `:139-148` (guard ERR-028 zero-norm — nuestros vectores reales L2-normados no lo disparan)
- `vanta-memory/src/core/record/l1_writer.rs:48` (`EmbedFn = Arc<dyn Fn(&str) -> Option<Vec<f32>>>`; re-export `core::record::EmbedFn`), `l1_reader.rs:189-202` (`cosine_similarity`: `None` en dim-mismatch → pool keyword, sin crash), `auto_recall.rs:356-360` (hook `None`→`None`; filtro empty/all-zero ya existe), `:395` (`semantic_ran` → `effective_mode` honesto)
- `docs/tasks/EMB-14.md` (patrón `try_provider_embed` + `is_local_model_available` + `fallback_warning` a reusar verbatim), `docs/tasks/EMB-16.md` (`embed_query` existe; vector.rs ya embebe queries IQL con él)
- `tools.rs:2903-2922` (`dim_mismatch_guidance` + `index_vector_dim` EMB-18 — reusar, no duplicar)

**Prohibidos (NO TOCAR):** `.opencode/`, `Justfile`, `completions/_vanta-cli*`, `desktop/src-tauri/Cargo.lock`, `ocr-delegate.yml`, `ocr-review.ps1`, `reparacion.bat`, `docs/pipeline-state.json`, plan file (solo recitation orquestador), `stash@{0..14}`, `src/llm.rs` escritura (EMB-16 ✅, solo lectura), zona EMB-17 model param (`tools.rs:2539-2542,3193` + `embed_texts_fallback` — hunks disjuntos; secuencial interno si colisiona), `Cargo.toml`, `~/.cargo/bin`, Backlog/avance (orquestador), ranking RRF (`fusion`/`rrf_merge`/search SDK), `docs/api/MCP.md` (dueño EMB-20).

## 3. DEPENDENCIAS

- **Wave4, tras EMB-14 ✅** (`11de0d42`: puts ya guardan CON vector — hay qué comparar) · **EMB-16 ✅** (`embed_query` existe + prefijos e5) · **EMB-13 ✅** (patrón fallback avisado + helpers) · **EMB-18 ✅** (dim gate a reusar).
- **Paralela EMB-17** (misma tools.rs, hunks disjuntos: 17→`:2539-2542,3193`; 15→`:1709,~3090,final`; secuencial interno si colisiona). **nextTask EMB-17 luego EMB-19.**
- **Hereda:** factory única (`get_embedding_provider`), D38 dual-pool+RRF (intacto), `search_with_method`/`multi` (gratis vía parse compartido), `search_records` filtro empty/zero (defensa en profundidad ya existente).
- **Verifica con margen EMB-16** (asimétrico query:/passage: margen 0.1204 — el recall/search hereda ese margen gratis).

## 4. REFERENCIAS

- `.opencode/rules/api-contract.md` (LEÍDA COMPLETA: R-8 este cambio es glue legítimo — traduce texto→query_vector vía provider, no reimplementa distancias ni ranking; R-4 adición > modificación — shapes de respuesta intactos, aviso por log + campo `effective_mode` existente; R-1 símbolos reales verificados vía Read directo; R-3 cfg-gating como EMB-13/14: sin features → keyword, compila default)
- `.opencode/rules/server-mcp.md` (LEÍDA COMPLETA: R-2 handlers sync — `embed_query` bloqueante ya es el patrón EMB-13/14; sin locks nuevos globales — Regla 8 N/A: sin `dashmap`/`parking_lot`/Tokio nuevos; el `Mutex` vive dentro de `LocalOnnxProvider`, dueño EMB-16)
- `clean-code-clean-architecture.md` Apéndice V (LEÍDO COMPLETO: V.1 `vantadb-mcp` = Frameworks/Drivers Humble Object — helpers ≤20 líneas SLAP, `Result`+`?`/Option sin `unwrap` en prod; V.3 sin stuttering — `try_embed_query`/`query_embed_hook` nombran 1 concepto c/u; V.4 severidades para `/cleanCA`)
- EMB-16 (`embed_query` con `query:` ya existe y vector.rs lo usa — paridad MCP↔IQL) + D38 (dual-pool+RRF intacto: solo se rellena `query_vector`, el ranking no se toca) + EMB-14 (helpers a reusar verbatim).
- **Spec (tabla — sin símbolos públicos nuevos, solo helpers privados + test; Gate D evaluado abajo):**

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Dónde interceptar search | A) `parse_search_request` (recomendado: 1 hunk cubre 4 tools, glue R-8) / B) en cada dispatch (×4 duplicación) / C) core SDK `search` (arrastra dep `llm` al core, blast multi-crate) | A | ✅ decidido-por-evidencia (3 call sites `:1754,1787,3054` verificados) |
| 2 | Semántica del embed de query | A) `embed_query` (recomendado: prefijo `query:` e5, paridad con vector.rs IQL) / B) `embed` doc-semántica (rompe paridad EMB-16, margen menor 0.0812 vs 0.1204) | A | ✅ contrato-plan + EMB-16 |
| 3 | Query dim vs índice | A) mismatch → keyword + `warn!` (recomendado: texto-solo hoy funciona por keyword; rechazar rompería búsquedas que hoy andan) / B) rechazar con guía (rompe compat de texto-solo) | A | ✅ decidido-por-evidencia (SDK keyword-path existente `mod.rs:111`) |
| 4 | Forma del aviso en search | A) `warn!` log (recomendado: shape hits array es back-compat MCP-39, sin slot) / B) envelope con flags (rompe clientes que parsean array) | A | ✅ R-4 aditivo |
| 5 | Hook recall | A) closure `embed_query` + filtro archivos locales (recomendado: no tragar dummies silenciosos, espejo EMB-14) / B) `core_embedding_hook` de vanta-memory (usa `embed` doc-semántica + env `VANTA_*` distinta — peras con manzanas otra vez) | A | ✅ decidido-por-evidencia (`l1_writer.rs:57-60` usa `embed` + otro env) |
| 6 | Build sin features `llm` | A) hook `None` + keyword (recomendado: mirror EMB-14, CI verde sin modelo) | A | ✅ precedente EMB-13/14 |

- **Gate D:** sin `pub fn`/tool/endpoint nuevo (2 helpers privados + test); blast 1 archivo + test; sin hot path (1 embed por query, write-path ya lo hace); contrato mecánico del plan → NO dispara `question`. Registrado con motivo.
- Notion Paso 0c: sin tool Notion en este entorno → se heredan conclusiones EMB-13/14 (Problema dimensión semántica + Propuesta retrieval híbrido REAL; Nuevas/Plan sin mapeo, filtro VantaDB).

## 5. SKILLS

**SDP (campaign_discover_skills_v2 BUILD keywords query-embed/same-provider/synonym-test/keyword-fallback, ≤8):**
campaign-executor — base task-system (1.00) · source-driven-development — base MCP server (1.00) · incremental-implementation — lifecycle BUILD slices verticales (1.00) · test-driven-development — lifecycle BUILD RED→GREEN (1.00) · context-engineering — lifecycle BUILD sesión compleja (1.00) · doubt-driven-development — lifecycle BUILD anti-falso-positivo (1.00) · api-and-interface-design — lifecycle BUILD superficie MCP (1.00). frontend-ui-engineering sugerida pero DESCARTADA: sin `web/`, scope discipline. security-and-hardening (base type MCP en v1) no vino en v2 pero se aplica igual: FASE SECURITY (input usuario + storage) — skill conocida de EMB-14.
**Base sesión:** campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail(full).
**Cargadas:** incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, source-driven-development, api-and-interface-design, systematic-debugging.
**SDP registrado:** `SDP: incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, source-driven-development, api-and-interface-design, systematic-debugging`
**Lógica nueva → test-driven-development (RED primero); bug → systematic-debugging inline.**

## 6. HERRAMIENTAS+MCP

- `cargo test -p vantadb-mcp --test test_query_embed -j 2` (focado RED→GREEN; default y `--features embed-local,remote-inference`)
- `cargo test -p vantadb-mcp -j 2` (suite completa; default + features)
- Cat test real (features + `VANTADB_EMBEDDING_PROVIDER=local` + `VANTADB_LOCAL_MODEL=<absoluta>` + `ORT_DYLIB_PATH=%LOCALAPPDATA%/VantaDB/onnxruntime/onnxruntime.dll`, desde repo root): puts sinónimos → `search_memory` texto-solo sin palabras comunes recupera el par
- MCP stdio sinónimos felino↔gato (si aplica en Step 3)
- `cargo check -p vantadb-mcp` + `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` + `cargo fmt --check` + `git diff --check`; `cargo -j 2` siempre
- MCP: codegraph_explore (blast radius — ejecutado, área vecina) + campaign_detect_task_type → mcp (ejecutado) + campaign_discover_skills_v2 (ejecutado) + campaign_verify_cmd (bug exit -1 conocido → bash directa) + check_index_coverage N/A (Reads directos verificados)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (cierre) + `/cleanCA` scope (solo informa)
- Sin grep-loop si codegraph responde (respondió parcial + Reads directos completaron).

## 7. INVESTIGACIÓN CÓDIGO (DISCOVERY ✅)

**Blast radius query→provider→dual-pool; hook None; ranking intacto:**
- `parse_search_request` (`:3071-3185`): `query_vector` vacío si no hay `query_vector` en args (`:3077-3085`); dim-check solo de provistos (`:3091-3099`); `text_query` pasa tal cual (`:3101`). **Intercepción hunk A:** tras `:3099`, si vector vacío + `text_query` con no-blanco → `try_embed_query` (nuevo, `embed_query` + filtro `is_local_model_available` + `fallback_warning` reusados) → `Ok(v)` + dim vs `index_vector_dim` coincidente → asignar; `Ok` con dim distinta o `Err` → `warn!` + keyword (vector queda vacío = comportamiento exacto de hoy).
- 3 call sites verificados: `dispatch_search_memory` (`:3054`, cubre `search_memory`+`memory_search`), `search_with_method` (`:1754`), `search_multi` (`:1787`). 1 hunk → 4 tools. `search_profile` (`:3159-3169`) intacto: modo Keyword forzado sigue ignorando el vector (el auto-embed no lo burla — `has_vector=false` en SDK).
- `memory_recall` (`:1654-1742`): `perform_auto_recall(&embedded, params, None)` (`:1709`). **Hunk B:** `None` → `query_embed_hook().as_ref()` (nuevo helper: `Some(hook)` con features, `None` sin ellas). Hook = `Arc::new(move |q| provider.embed_query(q).ok().filter(real))` donde `real` = remoto-siempre-Ok / local-solo-si-archivos (espejo EMB-14, nunca dummies). `search_records` (`auto_recall.rs:356-360`) ya filtra empty/all-zero; `cosine_similarity` (`l1_reader.rs:191`) da `None` en dim-mismatch → pool keyword, sin crash; `semantic_ran` → `effective_mode` honesto (`hybrid`/`keyword`).
- Por qué NO `core_embedding_hook`/`local_embedding_hook` de vanta-memory: usan `.embed(text)` (doc-semántica `passage:`) + env `VANTA_EMBEDDING_PROVIDER` distinto al `VANTADB_*` del MCP — sería peras-con-manzanas otra vez (decisión 5).
- RRF intacto: `rrf_merge` + `fusion` + `search_impl` no se tocan; el único cambio observable es `has_vector` false→true con el MISMO proveedor que embebió los docs (paridad de espacios vectoriales por construcción).
- Riesgo mismatch proveedor (query vs docs de otro modelo): mitigado por factory única + dim-check (search) + `cosine None`→keyword (recall) + EMB-18 una-dim-por-base en puts. Riesgo residual: misma dim, distinto modelo (silencioso) — techo conocido, NOTICED (igual que EMB-14 `OllamaProvider` sin override batch).
- ERR-028: vectores reales L2-normados (pooling EMB-16 + norm) → norma ≫ EPSILON, el guard no dispara. Dummies nunca llegan (filtrados antes).

**Impacto mapeado (Regla 0):**
- **Archivos leídos (completos):** tools.rs `:1600-1760,3043-3492,2895-2922`; vector.rs (191L); llm.rs `:1-130,530-610`; sdk/search/mod.rs `:60-170`; auto_recall.rs `:1-105,336-451`; l1_writer.rs `:40-99`; l1_reader.rs `:185-229`; validation.rs `:400-415`; test_auto_embed.rs (267L); api-contract.md; server-mcp.md; clean-code Ap. V (V.1-V.4); plan §EMB-15; EMB-14.md; EMB-16.md.
- **Hacia dentro:** `vantadb::llm::{get_embedding_provider, EmbeddingProvider::embed_query}` (cfg), `vanta_memory::core::record::EmbedFn` + `hooks::{perform_auto_recall, ...}`, `index_vector_dim`/`dim_mismatch_guidance` (reusar), `is_local_model_available`/`fallback_warning` (reusar), `structured_text_content` (forma intacta), `tracing::warn`, `std::sync::Arc`.
- **Entrantes:** `test_query_embed.rs` (nuevo), EMB-17 (hunks disjuntos `:2539-2542,3193`), EMB-19 (sinónimos e2e), MCP.md tabla (EMB-20, no tocar).
- **Veredicto:** BAJO-MEDIO — 1 prod (3 hunks disjuntos) + 1 test nuevo; sin pub symbols; sin hot path nuevo (1 embed/query); default y features compilan.

## 8. INVESTIGACIÓN PROBLEMA

- Peras con manzanas: docs embebidos con modelo M (EMB-14) + query sin embed (keyword) = la query nunca entra al espacio vectorial; aunque hubiera embed con OTRO modelo, cosenos entre espacios distintos son ruido. La fix es paridad por construcción: misma factory, misma `embed_query` que el IQL (`vector.rs`) ya usa.
- Batch vs 1×1: la query es UN texto → 1 `embed_query`; no hay batch que aplicar (pre-mortem "donde aplique" = no aplica aquí; el batch vive en puts EMB-14). Costo: 1 inferencia ONNX por search/recall (~ms, mismo orden que el put).
- Fallback avisado vs silencioso: Q5/EMB-13 (flag visible) — en search no hay slot sin romper back-compat (MCP-39) → `warn!` + docs; en recall el `effective_mode` existente es el aviso (hoy miente por omisión: siempre keyword; tras el fix es honesto).
- Keyword-forzado por profile (`mode: keyword`) sigue ignorando el vector a propósito — el auto-embed no burla una decisión explícita del llamante.
- FASE SECURITY (input usuario + storage, skill base): validación en frontera intacta (`validate_*`, `max_query_length`, `max_top_k`, non-empty/trim antes de embed — no se embebe string vacío); sin secrets; sin deps nuevas; sin `unwrap` en prod; errores dominio → `Ok(error_content)` (MEM-32). Checklist ✅.
- FASE PERFORMANCE: N/A con motivo — 1 embed/query fuera de hot loops indexados; sin claims de perf (Regla 9/11: ningún número sin bench; el margen citado es de EMB-16 con fuente).

## 9. INVESTIGACIÓN INTERNET

No se espera (todo local: factory + `embed_query` + hook existentes, stack verificado `Cargo.toml`: `ort load-dynamic`, forwarding `embed-local`/`remote-inference` en `vantadb-mcp`). Sin red usada → sin citas. Si surgiera duda de API → `webfetch` docs oficiales + deuda TSYS-13. Estado: N/A (igual que EMB-13/14 §9).

## 10. VALIDACIÓN+CIERRE

- [ ] RED: `test_query_embed` sinónimos falla sin el wiring (con features+modelo: query sin palabras comunes NO recupera el par — keyword-only) y pasa tras GREEN
- [ ] `cargo test -p vantadb-mcp --test test_query_embed -j 2` verde (default: graceful-skip + asserts deterministas; features: coherencia flag↔resultados)
- [ ] `cargo test -p vantadb-mcp -j 2` verde (sin regresiones; recall/search existentes intactos)
- [ ] `query_vector` provisto byte-exacto: con vector válido + sin proveedor la búsqueda vectorial sigue funcionando (no re-embed); dim-mismatch provisto sigue rechazando con guía EMB-18 exacta
- [ ] Cat test real: puts → `search_memory` texto-solo recupera sinónimos + `memory_recall` con `effective_mode` hybrid/embedding (evidencia numérica en §Ejecución)
- [ ] `cargo fmt --check` + `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` verdes (default + features)
- [ ] OCR advisory (Critical/High=bloquea) + `/cleanCA` scope (solo informa)
- [ ] DoD 3 niveles + reviewer distinto P2-01 (doubt-driven; vanta-review si disponible — si no, deuda leve como EMB-13/14) + Gates D/V/C + RESULTADO §7 + Save Point
- [ ] Commit `feat:` SOLO propios (tools.rs + test_query_embed.rs + task file). Backlog→avance NO tocar + push vía vanta-lead + coordinación EMB-17 (hunks disjuntos verificados en diff).

## Steps

- [x] **Step 1 — RED tests contrato (`test_query_embed.rs` nuevo):** sinónimos search + sinónimos recall + provisto-se-respeta + fallback-keyword-sin-modelo (+`ensure_indexes_current` MCP-01 en setup) · Verify: default 4/4 (2 señal en skip + 2 deterministas) ✅ · features+local REAL: 2 señal FALLAN por razón correcta (search keys `["d1"]` solo léxico, D0 invisible; recall `effective_mode:"keyword"`) ✅ DONE 2026-09-16
- [x] **Step 2 — GREEN search (hunk A + `try_embed_query`):** `mut query_vector` + auto-embed tras `text_query` (dim-check vs índice, warn+keyword en mismatch/fallo) + helper cfg-gated · Verify: default 4/4 ✅ + features+local 3/4 (sinónimo search PASA con modelo; recall sigue en keyword — esperado, falta hunk B) ✅ DONE 2026-09-16
- [x] **Step 3 — GREEN recall + cierre:** `query_embed_hook()` + hunk B (`None`→hook+warn) + eprintln señal en tests + full default (93+27+9+7+3+7+10+5+7+4+7+3+1+7) + features sin-model-env TODO verde + features+local focado 4/4 con evidencia + fmt/clippy ambas cfgs + OCR advisory + commit + recitation + RESULTADO ✅ DONE 2026-09-16

## Dependencias

- EMB-14 ✅ (puts con vector) · EMB-16 ✅ (`embed_query` + prefijos) · EMB-13 ✅ (fallback avisado) · EMB-18 ✅ (dim gate)
- Next: EMB-17 (hunks disjuntos, secuencial si colisiona) luego EMB-19

## Review (GATE — agente distinto, P2-01) — doubt-driven self-review + OCR input (vanta-review no disponible: deuda leve, igual que EMB-13/14)

- Preguntas adversariales: ¿`embed_query` (no `embed`) en ambos paths? Sí (try_embed_query + hook). ¿dummies al índice/query? No — filtro archivos+empty/zero en ambos (recall además filtra downstream). ¿shapes intactos? Sí — hits array + recall envelope iguales campos. ¿profile Keyword-forzado sigue ignorando vector? Sí (SDK `has_vector=false`; el auto-embed no burla decisión explícita; Vector-forzado ahora resuelve — mejora, no regresión). ¿sin `unwrap` en prod? Sí — match/if-let puros. ¿hunks disjuntos EMB-17 (`:2539-2542,3193`)? Sí — 1709, ~3113, final; `embed_texts_fallback` intacto. ¿TOCTOU env? Patrón heredado EMB-13/14, benigno. ¿locks nuevos (Regla 8)? No — Mutex interno del provider pre-existente. ¿multi-ns multi-dim en search_multi? `index_vector_dim` global — misma semántica que puts EMB-18, techo pre-existente NOTICED.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos no ejecutados
  - [x] No saltarse clarificación por "ya sé qué quiere"
  - [x] No declarar done sin verificar acceptance
  - [x] No ignorar fallos ni reportar "todo OK" parcial (3 fallos model-env investigados con stash-proof, no ocultados)
  - [x] No hacer un solo intento de búsqueda y darlo por saturado
  - [x] No copiar sin citar ni presentar supuestos como evidencia
  - [x] No reintentar en bucle sin diagnóstico
  - [x] No dejar huérfanos los pasos
  - [x] No degradar chequeo de errores en paths críticos
  - [x] No gastar presupuesto infinito
- **Cross-model:** skipped (contexto no-interactivo, sin autorización para CLIs externos).
- **cleanCA (Ap. V, solo informa):** Humble Object ✅ (glue, sin lógica negocio); SLAP 🟡 leve (`try_embed_query` ~25L por ramas cfg — precedente EMB-14 `try_provider_embed` similar, patrón establecido, sin FIND); sin `unwrap`/`unsafe` en prod ✅; sin stuttering ✅; saldo Regla 6 = 0 (sin deuda nueva, sin pago requerido — helpers aditivos + test).
- **Veredicto:** ✅ approve (deuda leve: vanta-review no disponible en entorno).

## Cierre 2026-09-16

Steps 1-3 ✅ + verify scope verde (default todo; features sin-model-env todo; features+local focado 4/4 con evidencia search `["d1","d0","d2"]` + recall `hybrid` con D0) + 3 fallos model-env probados pre-existentes vía stash (HALLAZGO al orquestador, Backlog NO tocado por prohibición) + commit `feat:` (solo 3 archivos propios, sin push). Handoff: EMB-17 (hunks disjuntos verificados en diff) luego EMB-19. WIP ajeno en worktree EXCLUIDO del commit.

## Notas

- `ponytail:` 1 embed por query (sin batch: N=1); file-check O(1) heredado EMB-13/14; sin caché nueva (el `Mutex` de sesión vive en `LocalOnnxProvider`, dueño EMB-16).
- NOTICED BUT NOT TOUCHING: misma-dim-distinto-modelo silencioso (techo, igual que EMB-14 ollama-batch); `core/local_embedding_hook` con env `VANTA_*` divergente (no migrar aquí); `search_profile` Keyword-forzado (intacto a propósito); WIP ajeno en worktree si lo hubiera → excluir del commit.
- Notion Paso 0c: sin tool Notion en este entorno → se heredan conclusiones EMB-13/14.

## Ejecución (evidencia mecánica — se puebla en Steps)

- **Step 1 RED default** (`cargo test -p vantadb-mcp --test test_query_embed -j 2`): 4 passed (2 señal skip sin modelo + 2 deterministas) ✅ (tras fix MCP-01 `ensure_indexes_current` en setup + rename `_storage` cfg-gated)
- **Step 1 RED features+local** (`VANTADB_EMBEDDING_PROVIDER=local` + `VANTADB_LOCAL_MODEL=<repo>/embeddings/models/multilingual-e5-small/onnx` + `ORT_DYLIB_PATH=%LOCALAPPDATA%/VantaDB/onnxruntime/onnxruntime.dll`, `--features embed-local,remote-inference`): 2 passed + 2 FAILED por razón correcta ✅ — search `keys: ["d1"]` (solo léxico, D0 invisible); recall `effective_mode:"keyword"` + solo hit léxico `score:1` (hook `None`)
- **Step 2 GREEN search:** focado default 4/4 ✅ + features+local 3/4 (sinónimo search PASA; recall aún keyword — esperado)
- **Step 3 GREEN recall + evidencia** (mismo env + `--nocapture`, features+local focado 4/4 ✅):
  - `EMB-15 señal search: keys=["d1", "d0", "d2"]` (query `"felino descansando"` sin palabras comunes con D0 → D0 2º por semántica, delante del impar)
  - `EMB-15 señal recall: mode="hybrid" recalled=[d1, D0(gato), d2]` (D0 vía felino↔gato)
- **Full scope default:** 27 lib + 9 code + 7 context + 3 proxy + 93 mcp + 7 scene + 10 skills + 5 auto_embed + 7 embed_texts + 4 query_embed + 7 thread + 3 wiki_async + 1 wiki_e2e + 7 wiki = TODO verde ✅
- **Full scope features SIN model-env** (canónico, como EMB-14): idem TODO verde ✅ (93 mcp incl. concurrent/dims/structured)
- **Full scope features CON model-env local:** 90/93 — 3 fallos (`concurrent_requests` 4-vs-5, `put_validates_vector_dimensions` :1366, `structured_output` :4438). **Root cause probada NO-mi-dif:** `git stash` de tools.rs + mismos 3 tests + mismo env → los 3 FALLAN igual (HALLAZGO al orquestador: tensión pre-existente model-env — puts con auto-embed real 384d vs tests que mezclan dims 3/4 + race concurrente con embeds lentos; dueños: suite mcp/EMB-19; `src/llm.rs` y ranking intactos). Sin model-env (canónico) todo verde.
- **Gates:** `cargo fmt --check -p vantadb-mcp` ✅ (tras `cargo fmt`, diff solo propio) · `cargo clippy -p vantadb-mcp --all-targets` default ✅ y `--features embed-local,remote-inference` ✅ 0 warnings · `git diff --check` ✅
- **Nota CWD:** tests corren con CWD=package dir; la señal real exige `VANTADB_LOCAL_MODEL` absoluta + `ORT_DYLIB_PATH` (igual que EMB-13/14 §Ejecución).

## Spec Gate

Tabla §4 vale como spec de decisiones (sin `pub fn`/tool/endpoint; helpers privados). Gate D: blast 1 archivo + test, sin hot path, contrato mecánico del plan → NO dispara `question`. Gate V: 2 fallas mismo-error → `question` antes de FAILED. Gate C: colaterales → fila FIND + `question`.

## Context Save Point

DISCOVERY completo 2026-09-16. Plan + reglas + llm.rs + tools.rs (search/recall/helpers EMB-13/14/18) + sdk/search + vanta-memory hooks + tests + skills 7 cargadas (frontend descartada). Task file creado. Siguiente: Step 1 RED.

## Invariantes de dominio (handoff — MUST)

- **Preservar:** shapes `search_memory` (hits array) y `memory_recall` (envelope) byte-compatibles; diff EMB-13/14/18 intacto; mensajes EMB-18 exactos; sin `unwrap`/`expect` en prod; sin editar `src/llm.rs`; hunks disjuntos EMB-17 (`:2539-2542,3193`); WIP ajeno excluido del commit; sin push (vía vanta-lead).
- **Comandos:** `cargo test -p vantadb-mcp --test test_query_embed -j 2` + `cargo test -p vantadb-mcp -j 2` + cat test MCP stdio search/recall sinónimos
- **Deuda:** ninguna (si surge → fila FIND, no silencio)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** sin deuda nueva esperada (helpers privados + test). Si surgiera → pagar con P2-5/P2-8 o fila FIND.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato §1 verificable + focado + full suite + cat test sinónimos con evidencia |
| **Commit** | Atómico `feat: EMB-15` (~200-300 líneas), `git diff` solo propios, fmt+clippy verdes |
| **Release** | N/A (pre-push gate Regla 1 vía `dev-tools/verify.ps1` en EMB-19) |

SKILLS_CARGADAS: incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, source-driven-development, api-and-interface-design, systematic-debugging
SDP: incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, source-driven-development, api-and-interface-design, systematic-debugging (frontend-ui-engineering descartada: sin web/)
