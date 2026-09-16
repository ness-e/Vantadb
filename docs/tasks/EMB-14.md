# EMB-14 — auto-embed en `memory_put`/`put_batch` (cierra FIND-99 con EMB-13)

> **Plan:** `docs/plans/2026-09-16-embeddings-auto.md` (Wave3, secuencial tras EMB-13)
> **Estado:** ⏳ IN PROGRESS 2026-09-16
> **Appetite:** 1d · 🟡 · 🔴 Alta
> **Branch/Commit:** develop / `feat: EMB-14`
> **Cynefin:** 🟨 complicado · ⬆️ 0 / ⬇️ 3 steps
> **Ruta:** vanta-worker
> **SDP:** campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design, systematic-debugging, security-and-hardening (frontend-ui-engineering descartada: sin `web/`, scope discipline)

## 1. TAREA

**Objetivo:** el MCP deje de guardar sin vector siempre: `memory_put`/`memory_put_batch` sin `vector` embeben el payload con el proveedor activo vía UN `embed_batch` (no 1×1); vector provisto se respeta byte-exacto (no re-embed); fallo del proveedor → guarda sin vector + aviso explícito (flag + warning, nunca error duro, nunca vectores dummy almacenados). Cierra FIND-99 junto con EMB-13.

**Contrato exacto (ley):**
1. Put sin vector → guarda CON vector del proveedor activo (get lo muestra con `vector` no-nulo). En build sin features `llm` o sin modelo → guarda sin vector + `fallback:true` + `warning` (Q5 heredado).
2. Vector provisto se respeta: round-trip byte-exacto, sin re-embed, sin warning espurio (`fallback:false`).
3. Fallo proveedor → guarda sin vector + aviso explícito (`fallback:true` + `warning` no-vacío + `warn!` en logs). Nunca error duro, nunca dummy hasheado persistido (el dummy no tiene señal: plan sim −0.03).
4. Batch: UN solo `embed_batch` para todos los faltantes; dim check de provistos PRIMERO (preserva mensajes EMB-18 exactos), luego auto, luego dim check de auto-vectores (reusa `dim_mismatch_guidance`, no duplicar).
5. Tests nuevos (RED→GREEN) + suite existente verde (1 re-point quirúrgico permitido: batch round-trip parse).

**Acceptance del plan:** MCP guarda sin vector siempre hoy (0 llamadas a provider en `src/sdk/`); evidencia GET `"vector":null`. Pre-mortem: latencia batch → `embed_batch` + budgeting existente.

## 2. ARCHIVOS

**Clave (único archivo prod en escritura):**
- `vantadb-mcp/src/handlers/tools.rs:1208-1302` (`memory_put`: vector `let`→`mut`, auto-embed tras check AUD-046, dim check auto-vectores, respuesta flat+flags, 2 líneas description en `:73-74`)
- `vantadb-mcp/src/handlers/tools.rs:1308-1348` (`memory_put_batch`: auto-embed faltantes tras check provistos `:1330-1341`, dim check auto-vectores, envelope `{records,fallback,warning?}`, 1 línea description `:102`)
- `vantadb-mcp/src/handlers/tools.rs:3208-3311` (zona EMB-13 — SOLO REUSAR `is_local_model_available`, `fallback_warning`; NO pisar su diff, commit `ad8af1d1` ya cerrado)
- `vantadb-mcp/tests/test_auto_embed.rs` (NUEVO, patrón `test_embed_texts.rs`)
- `vantadb-mcp/tests/mcp_tests.rs:3245` (1 re-point: `records` ← `parsed["records"]`, resto intacto)

**Relacionados (solo lectura):**
- `src/llm.rs:26-53` (trait + `embed_batch` default), `:57-111` (factory 3 variantes cfg), `:537-555` (override local secuencial)
- `vantadb-mcp/src/handlers/tools.rs:2844-2863` (`dim_mismatch_guidance` + `index_vector_dim` EMB-18 — reusar, no duplicar)
- `vantadb-mcp/src/validation.rs:386-415` (`text_content_structured`/`structured_text_content`), `vantadb-mcp/src/config.rs:75-78` (budgeting intacto)
- `docs/tasks/EMB-13.md` (patrón fallback avisado), `docs/tasks/EMB-16.md` (prefijos e5 se heredan gratis vía provider), EMB-18 tests `:4968-5075` (mensajes exactos a preservar)

**Prohibidos (NO TOCAR):** `.opencode/`, `Justfile`, `completions/_vanta-cli*`, `desktop/src-tauri/Cargo.lock`, `ocr-delegate.yml`, `ocr-review.ps1`, `reparacion.bat`, `docs/pipeline-state.json`, plan file (solo recitation orquestador), `stash@{0..14}`, `src/llm.rs` escritura (EMB-16), `src/physical_plan/vector.rs` (EMB-16), zona EMB-18 gates, `Cargo.toml`, `~/.cargo/bin`, Backlog/avance (orquestador), `docs/api/MCP.md` tabla (dueño EMB-20 — NOTICED, no tocar), WIP ajeno en worktree.

## 3. DEPENDENCIAS

- **Wave3, sola, secuencial tras EMB-13 ✅** (`ad8af1d1`, mismo archivo — orden cumplido, diff EMB-13 intacto verificado en DISCOVERY).
- **Previa EMB-18 ✅** (`43405be3` dim gate — se reusa, no se duplica) · **nextTask EMB-15** (recall con mismo proveedor; lee tras este commit).
- **Alimenta a EMB-15/19.** EMB-16 ✅ (prefijos e5 heredados gratis). Secuencial interno, NO paralelo con nadie.
- **Hereda:** `Embedded::put/put_batch`, trait provider + `embed_batch`, budgeting MCP, suite mcp verde, FIND-99 segunda mitad.

## 4. REFERENCIAS

- `.opencode/rules/api-contract.md` (LEÍDA COMPLETA: R-8 este cambio es glue legítimo — traduce provider→input, no reimplementa distancias ni ranking; R-4 adición > modificación — flags aditivos, batch envelope con `records` preservado; R-1 símbolos reales verificados vía codegraph/Read; R-3 el wiring es cfg-gated como EMB-13)
- `.opencode/rules/server-mcp.md` (LEÍDA COMPLETA: R-2 handlers sync, sin bloqueo nuevo — `embed_batch` bloqueante ya es el patrón EMB-13; sin locks nuevos globales — Regla 8 N/A: sin `dashmap`/`parking_lot`/Tokio nuevos)
- `clean-code-clean-architecture.md` Apéndice V (LEÍDO COMPLETO: V.1 `vantadb-mcp` = Frameworks/Drivers Humble Object — cero lógica negocio, helpers ≤20 líneas SLAP, `Result`+`?` sin `unwrap` en prod; V.4 severidades para `/cleanCA`)
- EMB-16 (prefijos e5 ya activos en `LocalOnnxProvider::embed_as` — el auto-embed los hereda gratis, sin código extra) + EMB-18 (dim gate protege el put; orden provided-check→auto→auto-check preserva sus mensajes exactos)
- **Spec (tabla — hay campos JSON nuevos `fallback`/`warning` en superficie MCP + 1 re-point de test, Gate D evaluado abajo):**

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Dónde interceptar | A) handlers MCP tools.rs (recomendado: glue R-8, blast 1 archivo, cfg-gating EMB-13) / B) core SDK `put` (arrastra dep `llm` al core, afecta CLI/proxy, blast multi-crate) | A | ✅ decidido-por-evidencia (factory ya cableada en tools.rs `:3261`; `Embedded::put` tiene N callers) |
| 2 | Transporte batch | A) UN `embed_batch` para faltantes (recomendado: pre-mortem plan) / B) 1×1 por record (latencia N×, viola pre-mortem) | A | ✅ contrato-plan (nota: `OllamaProvider` usa default loop — techo conocido, NOTICED no touching) |
| 3 | Forma del aviso | A) single flat+flags `{...record,fallback,warning?}` + batch envelope `{records,fallback,warning?}` (recomendado: 0 tests rotos salvo 1 re-point, R-4 aditivo) / B) envelopes ambos (rompe 4 asserts AUD-045/structured) / C) 2º content-block (sin flag machine-readable, inconsistente) | A | ✅ decidido-por-evidencia (AUD-045 `:2695,2756` + structured `:4421` exigen record plano) |
| 4 | Fallo proveedor | A) guardar sin vector + flag+warning+`warn!` (recomendado: contrato explícito) / B) dummy hasheado (sin señal, peor que null — plan sim −0.03) / C) error duro (viola Q5) | A | ✅ contrato-plan |
| 5 | Auto-vector vs dim base | A) reusar `dim_mismatch_guidance` y bloquear sin guardar (recomendado: Q4/EMB-18, no duplicar guía) / B) guardar sin vector (silencia cambio de modelo) / C) auto-reindex (prohibido Q4) | A | ✅ pregunta-owner-Q4 |
| 6 | Build sin features `llm` | A) guardar sin vector + `fallback:true` "unconfigured" (recomendado: mirror EMB-13 `:3298-3310`, CI verde sin red/modelo) / B) no compilar put (rompe default) | A | ✅ decidido-por-evidencia (EMB-13 precedente) |

- **Gate D:** sin `pub fn`/tool/endpoint nuevo (helpers privados + campos JSON aditivos); blast 1 archivo + tests; sin hot path; contrato mecánico → NO dispara `question`. Registrado con motivo.
- Si símbolo público nuevo → no aplica (no hay); tabla Spec arriba vale como spec de decisiones + Gate D no-disparado.

## 5. SKILLS

**SDP (campaign_discover_skills_v2 BUILD keywords auto-embed/put-batch/batch-embed/fallback-aviso, ≤8):**
campaign-executor — base task-system (score 1.00) · source-driven-development — base MCP server (1.00) · incremental-implementation — lifecycle BUILD slices verticales (1.00) · test-driven-development — lifecycle BUILD RED→GREEN (1.00) · context-engineering — lifecycle BUILD sesión compleja (1.00) · doubt-driven-development — lifecycle BUILD stakes altos, anti-falso-positivo (1.00) · api-and-interface-design — lifecycle BUILD superficie MCP (1.00) · systematic-debugging — wiring provider con fallos reales (type MCP) · security-and-hardening — base MCP + FASE SECURITY (input usuario + storage). frontend-ui-engineering sugerida pero DESCARTADA: sin `web/`, scope discipline.
**Base sesión:** campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail(full).
**Cargadas:** incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, source-driven-development, api-and-interface-design, systematic-debugging, security-and-hardening.
**SDP registrado:** `SDP: incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, source-driven-development, api-and-interface-design, systematic-debugging, security-and-hardening`
**Bug → systematic-debugging inline. Lógica nueva → test-driven-development (RED primero).**

## 6. HERRAMIENTAS+MCP

- `cargo test -p vantadb-mcp --test test_auto_embed -j 2` (focado RED→GREEN)
- `cargo test -p vantadb-mcp -j 2` (suite completa; default + `--features embed-local,remote-inference`)
- `cargo check -p vantadb-mcp` + `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` + `cargo fmt --check` (gates)
- MCP stdio put→get con vector (cat test si hay modelo: `VANTADB_EMBEDDING_PROVIDER=local` + `VANTADB_LOCAL_MODEL` absoluta + `ORT_DYLIB_PATH`, desde repo root)
- `git diff --check`, `cargo -j 2` siempre, timeouts generosos
- MCP: codegraph_explore + trace_path (blast radius — ejecutados), campaign_detect_task_type (mcp — ejecutado), campaign_discover_skills_v2 (ejecutado), campaign_verify_cmd (bug exit -1 conocido → bash directa), check_index_coverage (sin issues, metadata_changed → fuente Read directo)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (cierre) + `/cleanCA` sobre lo tocado (solo informa)
- Sin grep-loop si codegraph responde (respondió + Reads directos).

## 7. INVESTIGACIÓN CÓDIGO (DISCOVERY ✅)

**Blast radius put→Embedded→storage; batch path; doble-embed:**
- `memory_put` (`:1208-1302`): valida ns/key/payload → vector opcional (`validate_vector`, `max_vector_dim`) → check AUD-046 dim provisto vs `index_vector_dim` (`:1236-1242`) → sparse/ttl/metadata → `MemoryInput` (`:1287-1295`) → `embedded.put` (`:1298`). **Intercepción:** `let vector` → `mut`, tras `:1242` rellenar si `None`; tras rellenar, dim check del auto-vector vs índice (mismo `dim_mismatch_guidance`); respuesta flat+flags vía `structured_text_content`.
- `memory_put_batch` (`:1308-1348`): parse por item (`parse_memory_input` `:2868`, mismas validaciones) → check provistos `:1330-1341` (PRIMERO, preserva mensaje EMB-18 "expected 4, got 2") → `embedded.put_batch` (`:1344`). **Intercepción:** tras `:1341`, UN `embed_batch` con payloads faltantes, zip-back por índice; dim check auto-vectores; envelope `{records,fallback,warning?}`.
- `embed_texts_via_provider` (`:3252-3310`): llama `get_embedding_provider().embed_batch` (cfg) — el auto-embed llama al MISMO provider pero NUNCA usa `embed_texts_fallback` (dummies no se persisten). Reusa `is_local_model_available` + `fallback_warning` verbatim.
- Latencia: single = 1 embed; batch = 1 `embed_batch` (local = loop secuencial `:547-554`; ollama = default loop 1×1 HTTP — techo conocido, NOTICED). Sin caps nuevas (scope; budgeting `max_embed_*` es de `embed_texts`, intacto).
- Doble-embed imposible por construcción: solo `vector.is_none()` se rellena; test byte-exacto lo prueba en toda config.
- Callers `handle_tools_call("memory_put"/"memory_put_batch")`: tests + proxy HTTP (solo chequea visibilidad writer, no shape) + clientes externos (shape single 100% aditiva; batch envelope evoluciona 1 parse de test propio).
- Respuesta single: `to_value(&record)` + `["fallback"]=bool` + `["warning"]` si true (flat, R-4). Batch: `json!({records, fallback, warning?})`.

**Impacto mapeado (Regla 0):**
- **Archivos leídos (completos):** tools.rs `:1-130,1090-1410,2525-2615,2840-2933,3148-3311`; validation.rs `:380-510`; config.rs; llm.rs `:1-120,530-555`; test_embed_texts.rs; mcp_tests `:465-534,1257-1360,2660-2770,3224-3360,4406-4455,4968-5075`; mcp_fallback_proxy `:57-97`; api-contract.md; server-mcp.md; clean-code Ap. V; plan EMB-14; EMB-13.md.
- **Hacia dentro:** `vantadb::llm` (cfg), `Embedded`, `McpConfig`, `validate_*`/`parse_memory_input`, `index_vector_dim`/`dim_mismatch_guidance`, `is_local_model_available`/`fallback_warning`, `structured_text_content`, `serde_json`, `tracing::warn`.
- **Entrantes:** `test_auto_embed.rs` (nuevo), mcp_tests `:3245` (1 re-point), EMB-15 (lee tras commit), MCP.md tabla (EMB-20, no tocar).
- **Veredicto:** BAJO-MEDIO — 1 prod + 1 test nuevo + 1 re-point + 2 descriptions; sin pub symbols; sin hot path; default y features compilan.

## 8. INVESTIGACIÓN PROBLEMA

- Guardar sin vector = memoria no buscable por significado (FIND-99 segunda mitad): el vector HNSW falta → recall semántico ciego; solo keyword/BM25. El auto-embed lo cierra sin pasos extra del agente.
- `embed_batch` vs 1×1: un round-trip lógico al provider (local: loop interno; remoto: N HTTP en default impl — techo documentado, no de este task). Alternativa 1×1 en handlers = N× overhead + N× file-checks; descartada por pre-mortem.
- Aviso vs silencio: Q5 (EMB-13) exige flag visible; un put que guarda vectorless sin decirlo es el mismo falso-positivo silencioso que el dummy (doubt-driven: peor que error). Por eso flag+warning+`warn!`.
- Nunca persistir dummies: hash L2 determinista contamina el índice con vecinos falsos — estrictamente peor que `vector:null` (keyword-only honesto).
- FASE SECURITY (input usuario + storage, skill cargada): validación en frontera intacta (`validate_*`/`parse_memory_input` no se tocan); sin secrets; sin deps nuevas; sin `unwrap` en prod; errores dominio → `Ok(error_content)` (MEM-32), nunca pánico. Checklist ✅.
- FASE PERFORMANCE: N/A con motivo — write path, no hot loop search/ingestión; sin claims de perf (Regla 9/11: ningún número sin bench).

## 9. INVESTIGACIÓN INTERNET

No se espera (todo local: factory + trait + helpers existentes, stack verificado en `Cargo.toml`: `ort load-dynamic`, `vantadb-mcp` forwarding `embed-local`/`remote-inference`). Sin red usada → sin citas. Si surgiera duda de API → `webfetch` docs oficiales + deuda TSYS-13. Estado: N/A (igual que EMB-13 §9).

## 10. VALIDACIÓN+CIERRE

- [ ] RED: `test_auto_embed` falla por razón correcta (sin campo `fallback`) antes del wiring
- [ ] `cargo test -p vantadb-mcp --test test_auto_embed -j 2` verde (default y features)
- [ ] `cargo test -p vantadb-mcp -j 2` verde (91/91 sin regresiones; 1 re-point batch intacto en cobertura)
- [ ] Vector provisto byte-exacto en TODA config (no re-embed determinista)
- [ ] Cat test put→get: con modelo local vector no-nulo; sin modelo `fallback:true`+warning, nunca error duro
- [ ] `cargo fmt --check` + `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` verdes
- [ ] OCR advisory (Critical/High=bloquea) + `/cleanCA` scope (solo informa)
- [ ] DoD 3 niveles + reviewer P2-01 (doubt-driven self-review contexto fresco; vanta-review no disponible — deuda leve como EMB-13) + Gates D/V/C + RESULTADO §7 + Save Point
- [ ] Commit `feat:` SOLO propios (tools.rs + test_auto_embed.rs + mcp_tests re-point + task file). Backlog→avance NO tocar + push vía vanta-lead.

## Steps

- [x] **Step 1 — RED tests contrato (test_auto_embed.rs nuevo):** 5 tests (§10) + Verify: focado 4 fallan por `fallback` ausente (Null vs bool) + 1 pasa por skip graceful sin modelo ✅ DONE 2026-09-16
- [x] **Step 2 — GREEN single put + helpers:** `try_provider_embed()` + `auto_embed_one()` + wiring `memory_put` (flat+flags, dim-check ordenado) + 2 descriptions · Verify: focado 5/5 default ✅ DONE 2026-09-16
- [x] **Step 3 — GREEN batch + cierre:** `auto_embed_missing()` (1 embed_batch) + envelope batch + re-point `:3245` + colateral rápido EMB-13 (`if_same_then_else`, EMB-13 tests re-verificados 7/7) + full default (93 mcp + 5 auto + 7 embed) + features (93 + 7 + 5) + señal real par=0.9282 vs 0.8427/0.8366 + fmt/clippy ambas cfgs + OCR advisory + commit + recitation + RESULTADO ✅ DONE 2026-09-16

## Dependencias

- EMB-13 ✅ (mismo archivo, orden cumplido) · EMB-18 ✅ (guía reusada) · EMB-16 ✅ (prefijos gratis)
- Next: EMB-15 (mismo proveedor en recall; secuencial tras commit)

## Review (GATE — agente distinto, P2-01) — veredicto abajo en §Ejecución

- **Revisor:** doubt-driven-development (contexto fresco) + OCR como input; vanta-review si disponible
- **Enfoque:** ¿flat+flags single preserva AUD-045/structured? ¿batch 1 re-point conserva cobertura? ¿dummies nunca persistidos? ¿dim-check ordenado preserva EMB-18 exacto? ¿sin `unwrap` en prod?
- **Cómo se probó:** unit tests cfg-agnósticos + cat test con/sin modelo (evidencia abajo)

## Notas

- `ponytail:` file-check O(1) heredado de EMB-13; `embed_batch` único por llamada put/batch; sin caché nueva.
- NOTICED BUT NOT TOUCHING: `OllamaProvider` sin override `embed_batch` (default 1×1 HTTP) → futuro; MCP.md tabla puts (EMB-20); `sdk_serialization` (FIND-102); ORT expect/abort (FIND-100); query CLI read-only (FIND-101); `dim_mismatch_guidance` dead_code bajo `--tests --features` (dueño EMB-18/Wave2).
- Notion Paso 0c: sin tool Notion en este entorno → se heredan conclusiones EMB-13 (Problema dimensión semántica + Propuesta retrieval híbrido REAL; Nuevas/Plan sin mapeo, filtro VantaDB).

## Ejecución (evidencia mecánica — se puebla en Steps)

- **Step 1 RED:** `cargo test -p vantadb-mcp --test test_auto_embed -j 2` → 4 fallan por `fallback` ausente (`left: Null`), 1 pasa por skip graceful sin modelo (razón correcta) ✅
- **Step 2 GREEN:** focado 5/5 default ✅
- **Step 3 real (modelo local, `VANTADB_EMBEDDING_PROVIDER=local` + `VANTADB_LOCAL_MODEL=<repo>/embeddings/models/multilingual-e5-small/onnx` absoluta + `ORT_DYLIB_PATH=%LOCALAPPDATA%/VantaDB/onnxruntime/onnxruntime.dll`, features `embed-local,remote-inference`): `EMB-14 señal: par=0.9282 impar0=0.8427 impar1=0.8366` con `fallback:false` ✅ — put sin vector guarda CON vector semántico (get lo muestra); gap 0.086/0.092, umbral plan (par≥0.90, par−impar≥0.05) cumplido.
- **Step 3 fallback (features, `ollama` sin servidor):** 5/5 ✅ flag↔warning coherentes, nunca error duro.
- **Full scope default:** mcp_tests 93/93 + auto_embed 5/5 + embed_texts 7/7, 0 failed ✅ · **features:** mcp_tests 93/93 + embed_texts 7/7 + auto_embed 5/5 ✅
- **Gates:** `cargo fmt --check -p vantadb-mcp` ✅ · `cargo clippy -p vantadb-mcp --all-targets` default ✅ y `--features embed-local,remote-inference` ✅ (tras colateral: colapsar ramas idénticas EMB-13 `if_same_then_else`) · `git diff --check` ✅
- **Nota CWD:** tests corren con CWD=package dir; el cat test real exige `VANTADB_LOCAL_MODEL` absoluta (igual que EMB-13 §Ejecución).
- **Colateral rápido (inline, no FIND):** `embed_texts_via_provider` tenía ramas `if/else if` idénticas que clippy-features rechaza (pre-existente `ad8af1d1`, invisible en clippy-default de EMB-13) → colapsadas a `||`, semántica intacta, EMB-13 re-verificado 7/7 ambas cfgs.

## Review (GATE — agente distinto, P2-01) — doubt-driven self-review + OCR input (vanta-review no disponible: deuda leve, igual que EMB-13)

- Preguntas adversariales: ¿flag conservador? Sí — `false` solo con vectores reales asignados o provistos; local-`Ok`-sin-archivos → `true` (dummies descartados). ¿dummies persistidos? No — `embed_texts_fallback` nunca llamado en paths put (verificable por grep). ¿EMB-18 exacto? Sí — bloques provistos byte-idénticos + suite verde ambas cfgs. ¿dim auto-vectores? Bloquea con guía sin guardar (Q4). ¿`unwrap` en prod? No — `if-let`/`get`/indexación con invariante comentado. ¿TOCTOU dim-check→put? Pre-existente AUD-046, sin locks nuevos (Regla 8 N/A).
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos no ejecutados
  - [x] No saltarse clarificación por "ya sé qué quiere"
  - [x] No declarar done sin verificar acceptance
  - [x] No ignorar fallos ni reportar "todo OK" parcial
  - [x] No hacer un solo intento de búsqueda y darlo por saturado
  - [x] No copiar sin citar ni presentar supuestos como evidencia
  - [x] No reintentar en bucle sin diagnóstico
  - [x] No dejar huérfanos los pasos
  - [x] No degradar chequeo de errores en paths críticos
  - [x] No gastar presupuesto infinito
- **Cross-model:** skipped (contexto no-interactivo, sin autorización para CLIs externos).
- **cleanCA (Ap. V, solo informa):** Humble Object ✅ (glue, sin lógica negocio); SLAP 🟡 leve (`try_provider_embed`/`auto_embed_missing` >20L por ramas cfg — precedente EMB-13 similar); sin `unwrap` en prod ✅; saldo Regla 6 ≤0 vía colateral clippy fix ✅.
- **Veredicto:** ✅ approve (deuda leve: vanta-review no disponible en entorno).

## Cierre 2026-09-16

Steps 1-3 ✅ + verify scope verde (default 93+5+7, features 93+7+5) + evidencia real (par=0.9282 vs 0.8427/0.8366, fallback:false; ollama-sin-server fallback:true sin error duro) + commit `feat:` (solo 4 archivos propios, sin push). Handoff: EMB-15 (recall con mismo proveedor, secuencial tras este commit). WIP ajeno en worktree EXCLUIDO del commit. FIND-99 se cierra con EMB-13+EMB-14 (marca el orquestador).

## Spec Gate

Tabla §4 vale como spec de decisiones (adición JSON aditiva, sin `pub fn`/tool/endpoint). Gate D: blast 1 archivo + tests, sin hot path, contrato mecánico → NO dispara `question`. Gate V: 2 fallas mismo-error → `question` antes de FAILED. Gate C: colaterales → fila FIND + `question`.

## Context Save Point

DISCOVERY completo 2026-09-16. Plan + reglas + llm.rs + tools.rs (put/batch/embed_texts/helpers) + validation/config + tests + skills 8 cargadas. Task file creado. Siguiente: Step 1 RED.

## Invariantes de dominio (handoff — MUST)

- **Preservar:** budgeting `embed_texts` intacto; diff EMB-13 (`:2592-2614,3208-3310`) intacto; mensajes EMB-18 exactos; sin `unwrap`/`expect` en prod; sin editar `src/llm.rs`; WIP ajeno excluido del commit; sin push (vía vanta-lead).
- **Comandos:** `cargo test -p vantadb-mcp --test test_auto_embed -j 2` + `cargo test -p vantadb-mcp -j 2` + cat test MCP stdio put→get
- **Deuda:** ninguna (si surge → fila FIND, no silencio)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** sin deuda nueva esperada (helpers privados + tests). Si surgiera → pagar con P2-5/P2-8 o fila FIND.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato §1 verificable + focado + full suite + cat test con evidencia |
| **Commit** | Atómico `feat: EMB-14` (~150-250 líneas), `git diff` solo propios, fmt+clippy verdes |
| **Release** | N/A (pre-push gate Regla 1 vía `dev-tools/verify.ps1` en EMB-19) |

SKILLS_CARGADAS: incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, source-driven-development, api-and-interface-design, systematic-debugging, security-and-hardening
SDP: incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, source-driven-development, api-and-interface-design, systematic-debugging, security-and-hardening (frontend-ui-engineering descartada: sin web/)
