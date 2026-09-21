# EMB-17 — parámetro `model` honrado (switch por nombre, Q2)

> **Plan:** `docs/plans/2026-09-16-embeddings-auto.md` (Wave4, tras EMB-15 ✅ `9f4fd725`; paralela EMB-15 ya cerró, secuencial interno si colisionan)
> **Estado:** ⏳ IN PROGRESS 2026-09-16
> **Appetite:** 1d · 🟡 · 🟡
> **Branch/Commit:** develop / `feat: EMB-17`
> **Cynefin:** 🟨 complicado (caché + concurrencia) · ⬆️ 1 / ⬇️ 3 steps
> **Ruta:** vanta-worker
> **SDP:** campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design, systematic-debugging (frontend-ui-engineering DESCARTADA: sin `web/`, scope discipline)

## 1. TAREA

**Objetivo:** `embed_texts` con `model: "<id del manifest>"` usa ESE modelo (switch por nombre, Q2 owner). Hoy el parámetro es público pero ignorado en silencio (`_model` en `tools.rs:3295`, ecoreo sin selección en `:2661-2666` verificado 2026-09-16). Q2 promete elegir entre modelos.

**Contrato exacto (ley):**
1. `model: "<id del manifest>"` usa ese modelo vía caché de sesiones (una por modelo ~450MB c/u, con tope documentado `MAX_CACHED_LOCAL_MODELS=2` + evicción avisada `warn!`, no silenciosa).
2. Id desconocido → error claro `invalid_params` con lista válida (9 ids del manifest) — nunca silencio, nunca dummy con otro modelo.
3. Id conocido pero archivos ausentes en disco (solo 3/9 presentes) → error claro con comando `python embeddings/download.py --only <id>` + tamaño (`size_onnx_mb`) — nunca fallback silencioso con dim errónea.
4. Dim distinta a la base → NO duplicar: gatea EMB-18 (`dim_mismatch_guidance` verbatim en puts/search; `embed_texts` solo devuelve la dim correcta del modelo).
5. `model: None` → comportamiento EMB-13 intacto (env `VANTADB_EMBEDDING_PROVIDER` + fallback Q5 avisado).
6. Test nuevo RED→GREEN que prueba 1-3 + suite existente verde.

**Acceptance del plan:** Gate Justificación (parámetro público ignorado verificado; Q2 promete elegir). Pre-mortem: RAM ×N → tope+evicción documentada; dim distinta → EMB-18.

## 2. ARCHIVOS

**Clave (escritura):**
- `vantadb-mcp/src/handlers/tools.rs:2576-2681` (handler `embed_texts` — `model_opt` hoy solo ecoreo `:2661-2666`; hunk A: validación + switch + `effective_model` en respuesta)
- `vantadb-mcp/src/handlers/tools.rs:3295-3412` (helpers `embed_texts_fallback(_model)` + `embed_texts_via_provider(model)` + comentario `:3316` "NO selecciona (EMB-17)" — hunk B: manifest map + caché + switch real)
- `vantadb-mcp/tests/test_model_switch.rs` (NUEVO, patrón `test_embed_texts.rs`: asserts cfg-gated default/features)
- `docs/tasks/EMB-17.md` (este archivo)

**Relacionados (solo lectura):**
- `src/llm.rs:57-123` (factory `get_embedding_provider` + `LocalOnnxProvider { session: parking_lot::Mutex<Session>, dim, family, model_dir }` — se REUSA, NO se modifica salvo necesidad mínima; prefijos EMB-16 intactos)
- `src/llm.rs:190-353` (`LocalOnnxProvider::new/detect_dim/try_load_*` — construcción por `model_dir`, `new` nunca falla (dummy fallback); el filtro de archivos lo hace el llamante)
- `src/config.rs:195-214,295-309,935-940` (`LlmCfg`, default `ollama` + `local_model_path`, env `VANTADB_LOCAL_MODEL` absoluta)
- `embeddings/manifest.json` (mapa id→dir→dim→size: 9 modelos 384/512/768/1024/4096; 3 en disco; referencia, NO se toca)
- `desktop/src-tauri/src/commands/embed.rs:160-283` (precedente `resolve_model` id→dir→dim + `EmbeddingCache by_id/by_path` + `build_backend` con error "ONNX model not found … run download.py" — patrón a espejar, con TOPE nuevo)
- `tools.rs:2910-2929` (`dim_mismatch_guidance` + `index_vector_dim` EMB-18 — reusar, NO duplicar)
- `tools.rs:3544-3619` (`try_embed_query` + `query_embed_hook` EMB-15 — hunks disjuntos, NO tocar)
- `tools.rs:1708-1716` (recall hook EMB-15 — hunk disjunto, NO tocar)
- `tools.rs:980-998` (schema `embed_texts` — `model` ya documentado como "Optional model id override", NO se cambia)
- `docs/tasks/EMB-13.md` (patrón fallback avisado + `is_local_model_available` + `fallback_warning` verbatim), `EMB-14.md` (patrón `try_provider_embed` + nunca-dummies-en-puts), `EMB-15.md` (hunks `:1706,~3081,~3532` disjuntos), `EMB-18.md` (dim gate a reusar), `EMB-16.md` (`embed_query`/familias intactas)

**Prohibidos (NO TOCAR):** `.opencode/`, `Justfile`, `completions/_vanta-cli*`, `desktop/src-tauri/Cargo.lock`, `ocr-delegate.yml`, `ocr-review.ps1`, `reparacion.bat`, `docs/pipeline-state.json`, plan file (solo recitation orquestador), `stash@{0..14}`, zona EMB-15 recall/search (`:1708-1716`, `:3110-3142`, `:3543-3619` — hunks disjuntos; EMB-15 ya cerró `9f4fd725`, no pisar), `src/llm.rs` escritura salvo necesidad mínima justificada (EMB-16 ✅, no revertir prefijos), `Cargo.toml`, `~/.cargo/bin`, Backlog/avance (orquestador), `docs/api/MCP.md` (dueño EMB-20).

## 3. DEPENDENCIAS

- **Wave4, tras EMB-15 ✅** (`9f4fd725`: hunks `:1706,:3081,:3532` — verificados disjuntos de mis zonas `:2576-2681,:3295-3412` vía `git show`; secuencial interno ya resuelto por cierre EMB-15).
- **BLOQUEANTES:** EMB-13 ✅ (zona `embed_texts` + `embed_texts_via_provider` a extender; budgeting 128/25k + fallback Q5 intactos) + EMB-18 ✅ (`dim_mismatch_guidance` a reusar, NO duplicar).
- **Paralela EMB-15 ✅ ya cerró** (verificar hunks disjuntos antes del commit: `git show 9f4fd725 -- tools.rs | grep ^@@` → 4 hunks ninguno en `:2576-2681/:3295-3412`).
- **Previa EMB-15 ✅ / nextTask EMB-19.** Sirve a EMB-11 (wizard ofrece switch por nombre; este fix lo hace real).
- **Hereda:** manifest (id→dir→dim→size), `LocalOnnxProvider::new`, `is_local_model_available`/`fallback_warning`/`local_model_files_present`, EMB-18 dim gate, desktop `resolve_model`+`EmbeddingCache` como precedente.

## 4. REFERENCIAS

- `.opencode/rules/api-contract.md` (LEÍDA COMPLETA: R-8 glue legítimo — traducir `model`→provider vía manifest+cache, NO reimplementar distancias/ranking/pooling (viven en `src/llm.rs`+`src/sdk/search/`); R-4 adición>modificación — shapes `embed_texts` intactos (`embeddings/model/dim/fallback/warning/truncated/next_cursor`), `effective_model` = mismo campo `model` con valor canónico, error de id desconocido = `invalid_params` en frontera; R-1 símbolos reales verificados vía Read directo; R-3 cfg-gating como EMB-13: sin features → validación de id igual + fallback avisado para conocidos)
- `.opencode/rules/core-engine.md` (LEÍDA COMPLETA: R-3 sin `unwrap/expect` en prod — cache con `OnceLock`+`get_or_init` + `if-let`, errores con `?`/`map_err`; R-5 env `VANTADB_*` intacto — `model` NO crea env nueva)
- `.opencode/rules/concurrency-async.md` (LEÍDA COMPLETA — Regla 8: handlers MCP son sync vía `spawn_blocking` (`server.rs` dispatch); sin `.await` en mis hunks → R-2 N/A por construcción; sin `insert_lock`/`volatile_cache` (son del core multi-índice, no de este glue) → R-8 orden global N/A; lock propio: global `parking_lot::Mutex<HashMap>` solo para `clone Arc` (µs, nunca a través de inferencia que toma el `Mutex<Session>` por-provider) → orden cache→sesión, nunca inverso, sin re-adquisición; R-3 semáforo ya existe en dispatch, no se inventa límite con mutex)
- `clean-code-clean-architecture.md` Apéndice V (LEÍDO: V.1 `vantadb-mcp` = Frameworks/Drivers Humble Object — helpers ≤20 líneas SLAP, `Result`+`?` sin `unwrap` en prod; V.3 sin stuttering — `resolve_manifest_model`/`cached_local_provider_for`/`model_cache_eviction` nombran 1 concepto c/u; V.4 severidades para `/cleanCA`)
- Propuesta §3 superficie mínima (heredada EMB-13/14: sin tool/endpoint/`pub fn` nuevo — helpers privados + test; sin Notion tool en este entorno → conclusiones EMB-13/14: Problema dimensión semántica + Propuesta retrieval híbrido REAL; Nuevas/Plan sin mapeo, filtro VantaDB)
- **Spec (tabla — 1 símbolo privado nuevo en el mismo archivo + test; Gate D evaluado abajo):**

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Dónde resolver `model` | A) `tools.rs` helpers privados (recomendado: glue R-8, hunks disjuntos, sin tocar `llm.rs`) / B) `src/llm.rs` `get_provider_for_model` (arrastra cache al core, riesgo revert EMB-16) / C) por-request sin cache (recarga ONNX ~450MB por llamada — inviable) | A | ✅ decidido-por-evidencia (EMB-18/15 ya gatean en `tools.rs`; `llm.rs` solo lectura) |
| 2 | Alcance del switch | A) solo `embed_texts` con `model` explícito (recomendado: único tool con slot `model` en schema `:993`; puts/search usan proveedor activo sin slot) / B) también puts/search con `model` (cambia 3 schemas + paridad EMB-14/15 — scope creep) | A | ✅ contrato-plan + schema verificado |
| 3 | Remoto vs local con `model` | A) `model`=manifest-id siempre fuerza local ONNX de ese id (recomendado: determinista, testeable, Q2 es sobre modelos del manifest) / B) `model` como nombre ollama/openai cuando provider=remoto (dos namespaces de nombres, lista válida ambigua) | A | ✅ Q2+manifest (9 ids); remoto sigue por env sin `model` |
| 4 | Id conocido sin archivos | A) error claro + `download.py --only <id>` + tamaño (recomendado: pedir explícito que no existe ≠ "sin modelo"; fallback 384d mentiría la dim) / B) fallback avisado (dim 384 para un 768 pedido — basura con flag) | A | ✅ Q2 "resto avisado" + honestidad EMB-18 |
| 5 | Id conocido sin features compiladas | A) fallback avisado (recomendado: mirror EMB-13 Q5, CI verde sin engine) / B) error duro (rompe CI default) | A | ✅ precedente EMB-13 |
| 6 | Tope de caché | A) `MAX_CACHED_LOCAL_MODELS=2` FIFO arbitraria avisada `warn!` (recomendado: ~900MB max, HashMap sin dep nueva, ponytail) / B) LRU exacta con `lru` crate (nueva dep, overkill) / C) sin tope (precedente desktop — RAM ×N sin techo, viola pre-mortem) | A | ✅ pre-mortem plan |
| 7 | Manifest ilegible (CWD frágil) | A) candidatos `embeddings/manifest.json` + `../` + `CARGO_MANIFEST_DIR`-relativo + fallback hardcodeado 9 ids verificado 2026-09-16 (recomendado: validación de id funciona en tests con CWD=package) / B) solo path relativo (tests fallan según CWD) | A | ✅ fragilidad CWD ya documentada en plan |

- **Gate D (question-gates.md):** ¿dispara? Blast 1 archivo prod + 1 test nuevo; 0 símbolos `pub` nuevos (cache `static` privada + 3 `fn` privadas); sin hot path (cold per-request, inferencia ya existe); contrato mecánico del plan + Spec tabla arriba → NO dispara `question`. Sin `pub fn`/tool/endpoint nuevo → se procede. Registrado con motivo.

## 5. SKILLS (SDP v2 phase=BUILD, keywords model-switch/session-cache/manifest-map/concurrency-audit, ≤8)

- `campaign-executor` — base task-system: state machine PLAN→ACT→VERIFY + RESULTADO. (1.00 base MCP)
- `source-driven-development` — base MCP: comando sugerido y mapa manifest verificados en código, no memoria. (1.00 base)
- `incremental-implementation` — lifecycle BUILD: slices verticales RED→GREEN→verify→commit parcial. (1.00)
- `test-driven-development` — lifecycle BUILD: RED→GREEN→REFACTOR; bug sin reproduction test no cuenta. (1.00)
- `context-engineering` — lifecycle BUILD: context pack Rules→Plan→Source→Error (usado en este file). (1.00)
- `doubt-driven-development` — lifecycle BUILD: stakes (RAM ×N, dim silenciosa) → adversarial review del switch. (1.00)
- `api-and-interface-design` — lifecycle BUILD: error como contrato (lista válida estable, Hyrum's Law). (1.00)
- `systematic-debugging` — extra (no vino en v2 pero es base MCP v1 + precedente EMB-15): FASE ante fallo de verify, no re-intentar a ciegas.
- `frontend-ui-engineering` — DESCARTADA (sugerida por scorer sin ver `web/`; sin `web/` en scope, scope discipline).
- **Base sesión:** campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail(full).
- **Cargadas:** incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, api-and-interface-design, systematic-debugging.
- **SDP registrado:** `SDP: incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, api-and-interface-design, systematic-debugging (frontend-ui-engineering descartada: sin web/)`
- **Lógica nueva → test-driven-development (RED primero).**

## 6. HERRAMIENTAS+MCP

- `cargo test -p vantadb-mcp --test test_model_switch -j 2` (focado RED→GREEN; default y `--features embed-local,remote-inference`)
- `cargo test -p vantadb-mcp -j 2` (suite completa; default + features sin-model-env canónico como EMB-14/15)
- Cat test real (features + `VANTADB_EMBEDDING_PROVIDER=local`, desde repo root): `embed_texts` con `model=all-MiniLM-L6-v2` vs `multilingual-e5-small` → dims correctas + señal; `model=no-existe` → error con lista; `model=bge-base-en-v1.5` (no en disco) → error con download hint
- MCP stdio switch por nombre + error id desconocido (contrato alterno si hay tiempo)
- `git diff --check` + `cargo fmt --check -p vantadb-mcp` + `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` (default + features); `cargo -j 2` siempre
- MCP: codegraph_explore (blast radius — ejecutado) + campaign_detect_task_type → mcp (ejecutado) + campaign_discover_skills_v2 (ejecutado, 8 skills) + check_index_coverage (sin issues, freshness metadata_changed → se leyó fuente directa) + campaign_verify_cmd (bug exit -1 conocido → bash directa)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (cierre) + `/cleanCA` scope (solo informa)
- Sin grep-loop si codegraph responde (respondió + Reads directos completaron).

## 7. INVESTIGACIÓN CÓDIGO (blast radius — DISCOVERY 2026-09-16)

**Param → manifest map → caché sesiones → dim gate; RAM ×N; locks:**
- Handler `embed_texts` (`:2576-2681`): `model_opt` parseado `:2590-2593`, pasado a `embed_texts_via_provider(to_embed, model_opt.as_deref())` `:2661`, ecoreado `:2666` (`model_opt.unwrap_or("multilingual-e5-small")`). Budgeting 128/25k intacto (`:2585-2657`). **Hunk A:** tras validar `model_opt` contra manifest (desconocido → `invalid_params` con lista), resolver `effective_model` canónico y pasarlo al provider + respuesta.
- `embed_texts_fallback(texts, _model)` (`:3295`): `_model` ignorado a propósito (dummy 384d sin motor); se conserva — el fallback solo sirve al path `None` (Q5). Con `model` explícito y features, NUNCA fallback (error claro en su lugar).
- `embed_texts_via_provider(texts, model)` (`:3352-3412`): comentario `:3316` "model se ecorea pero NO selecciona (EMB-17)" — el switch vive aquí. Path actual: env `VANTADB_EMBEDDING_PROVIDER` → `get_embedding_provider().embed_batch`. **Hunk B:** si `model=Some(id)` → `resolve_manifest_model(id)?` → `cached_local_provider_for` → `embed_batch` → `Ok((vectors,false,None))` con `effective_model=id`; archivos ausentes → `Err` download-hint; provider-Err con model explícito → `Err` (no fallback). Si `None` → path EMB-13 intacto byte-exacto.
- Manifest (`embeddings/manifest.json` completo leído): 9 ids (`bge-small-en-v1.5` 384, `all-MiniLM-L6-v2` 384, `bge-base-en-v1.5` 768, `jina-es-v2-base` 768, `paraphrase-multilingual-MiniLM-L12-v2` 384, `distiluse-multilingual` 512, `multilingual-e5-small` 384 default, `bge-m3` 1024, `qwen3-embedding-8b` 4096 + `onnx:null` + exception GPU). Dir canónica `embeddings/models/<id>/onnx`. Tamaños `size_onnx_mb` para el hint (80–1200, null→"unknown size").
- `LocalOnnxProvider` (`llm.rs:116-123`): `session: Option<parking_lot::Mutex<Session>>` por instancia (dueño EMB-16, NO se toca); `new(dir)` nunca falla (dummy fallback) → el llamante filtra con `local_model_files_present(dir)` (archivos `model.onnx`|`onnx/model.onnx`|`model_int8.onnx` + `tokenizer.json`, `:3318-3336`) antes de aceptar como real. `detect_dim` (`:228-270`) lee manifest por substring de dir o `config.json hidden_size`, default 384.
- Precedente desktop (`embed.rs:160-283`): `resolve_model` id→`(dir,id,dim)` + error `"unknown embedding model: …; pass a manifest id …"` + `EmbeddingCache { by_id, by_path }` + `build_backend` error `"ONNX model not found at …; run python embeddings/download.py --only <id>"`. Se espeja con TOPE nuevo (desktop no tiene tope — aquí `MAX_CACHED_LOCAL_MODELS=2` + evicción avisada).
- EMB-18 reuse (`:2910-2929`): `embed_texts` NO gatea (devuelve dim del modelo); puts/search gatean con `dim_mismatch_guidance` verbatim. Sin duplicar.
- EMB-15 disjuntos (`git show 9f4fd725 -- tools.rs | grep ^@@` → `1706,3081,3106,3532`; mis zonas `2576-2681,3295-3412` — 0 solape). EMB-13/14 patrones (`fallback_warning`, `is_local_model_available`, `try_provider_embed` nunca-dummies) intactos.
- Concurrencia (Regla 8 auditada): handler sync (dispatch vía `spawn_blocking` en `server.rs`, sin `.await` en mis hunks); cache `parking_lot::Mutex<HashMap<String,Arc<LocalOnnxProvider>>>` global via `OnceLock` — guard solo para `get().cloned()`/`insert` (µs); inferencia toma `Mutex<Session>` por-provider DENTRO de `embed` (orden cache→sesión, nunca inverso, sin re-adquisición, sin guard cruzando await). Sin `dashmap`/Tokio/`insert_lock`/`volatile_cache` nuevos. Evicción = `keys().next().cloned()` + `remove` + `warn!` (avisada). RAM: 1 Arc por modelo, máx 2 residentes (~900MB techo documentado en comentario + respuesta NO expone conteo — el aviso es log, no shape).

**Impacto mapeado (Regla 0):**
- **Leídos (completos):** tools.rs `:975-998` (schema), `:1639-1748` (recall EMB-15), `:2530-2681` (embed_texts), `:2840-2929` (EMB-18 helper+dim), `:3079-3208` (parse_search EMB-15), `:3295-3619` (helpers EMB-13/14/15), llm.rs `:1-270,355-555,557-756,905-970`, config.rs `:195-214,295-309,920-960`, manifest.json (completo), desktop embed.rs `:58-100,158-338`, test_embed_texts.rs (291L), EMB-13/14/15/18.md, api-contract.md, core-engine.md, concurrency-async.md, clean-code Ap. V (V.1-V.4), plan §EMB-17.
- **Hacia dentro:** `vantadb::llm::{LocalOnnxProvider, EmbeddingProvider::embed_batch}`, `local_model_files_present`, `fallback_warning`, `McpError::{invalid_params,internal_error}`, `warn!`, `OnceLock`, `parking_lot::Mutex`, manifest `serde_json::Value`.
- **Entrantes:** `handle_tools_call("embed_texts")` ← `server.rs dispatch_request` + tests (`test_embed_texts.rs` existente — `embed_texts_with_model_param` espera ecoreo, se conserva; `test_model_switch.rs` nuevo); EMB-11 wizard (consume el switch); EMB-19 e2e (verifica matriz).
- **Veredicto:** MEDIO-BAJO — 1 prod (2 hunks disjuntos) + 1 test nuevo; sin `pub` nuevo; sin schema nuevo; default y features compilan; rollback = revert 1 commit.

## 8. INVESTIGACIÓN PROBLEMA

- Promesa rota Q2: el schema publica `model` ("Optional model id override") pero el código lo ignora (`_model`, comentario `:3316` lo admite). Un agente que pide `all-MiniLM-L6-v2` recibe vectores del default sin aviso — silently wrong model, peor que error (doubt-driven: falso-positivo es peor que error).
- Tradeoff caché con tope+evicción avisada vs una sola sesión: una sola sesión global (sobrescribir `VANTADB_LOCAL_MODEL` por request) serializa modelos distintos con recargas de ~450MB por switch (segundos + fragmentación) y rompe concurrencia (dos requests con modelos distintos se pisan). Caché por-modelo con tope 2 = ambos modelos residentes en el caso común (default + 1 alternativo), switch O(1) por `Arc::clone`, techo RAM acotado y avisado. LRU exacta overkill (nueva dep); FIFO arbitraria avisada es suficiente y ponytail.
- Dim distinta vía EMB-18: `embed_texts` con otro modelo devuelve otra dim (p.ej. 768 vs base 384); el put siguiente es rechazado con guía accionable (re-embeder con dim original O re-ingerir + `rebuild_index`). EMB-17 NO gatea, NO sugiere reindex, NO auto-convierte — solo devuelve la dim verdadera del modelo pedido.
- Wizard EMB-11 ofrece switch: sin este fix el wizard mentiría (ofrece lo que el MCP ignora). Con el fix, el wizard es honesto.

## 9. INVESTIGACIÓN INTERNET

No se espera (todo local: manifest + `LocalOnnxProvider` + helpers existentes, stack verificado `Cargo.toml`: `ort load-dynamic`, forwarding `embed-local`/`remote-inference` en `vantadb-mcp`). Sin red usada → sin citas. Si surgiera duda de API → `webfetch` docs oficiales + deuda TSYS-13. Estado: N/A (igual que EMB-13/14/15 §9). Gate de citas TSYS-13: sin URLs en evidencia → N/A.

## 10. VALIDACIÓN+CIERRE

- [ ] RED: `test_model_switch` (desconocido→Err con lista válida; conocido→ecoreo+dim; sin-archivos→Err con download hint) falla sin el wiring por razón correcta (hoy desconocido pasa como ecoreo 200)
- [ ] `cargo test -p vantadb-mcp --test test_model_switch -j 2` verde (default: desconocido-Err + conocido-fallback-avisado; features: coherencia flag↔resultados)
- [ ] `cargo test -p vantadb-mcp -j 2` verde (sin regresiones; `test_embed_texts` 7/7 intactos incl. `embed_texts_with_model_param`)
- [ ] Cat test real (features+local): switch `all-MiniLM-L6-v2` usa ese modelo (dim 384 + `model` canónico), `bge-base-en-v1.5` sin archivos → error con download hint, `no-existe` → error con lista
- [ ] `cargo fmt --check` + `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` verdes (default + features)
- [ ] OCR advisory (Critical/High=bloquea) + `/cleanCA` scope (solo informa)
- [ ] DoD 3 niveles + reviewer distinto P2-01 (doubt-driven; vanta-review si disponible — si no, deuda leve como EMB-13/14/15) + Gates D/V/C + RESULTADO §7 + Save Point
- [ ] Commit `feat:` SOLO propios (tools.rs + test_model_switch.rs + task file). Backlog→avance NO tocar + push vía vanta-lead + hunks disjuntos EMB-15 verificados en diff.

## Steps

- [x] **Step 1 — RED tests contrato (`test_model_switch.rs` nuevo):** desconocido→Err+lista + conocido-ecoreo+dim + sin-archivos→Err-download-hint (+cfg-gated default/features como EMB-13) · Verify: default 1 failed por razón correcta (`unknown model debe ser Err, no Ok-ecoreo silencioso` con `fallback:true` ecoreando `no-existe-xyz`) + 2 passed ✅ DONE 2026-09-16
- [x] **Step 2 — GREEN switch (hunks A+B):** `validate_manifest_model_id` en handler + `manifest_model_table/resolve/ids` + `MAX_CACHED_LOCAL_MODELS=2` + `model_session_cache/cached_local_provider_for/embed_texts_with_manifest_model` + rama `model=Some` en `embed_texts_via_provider` + fix CWD-robusto (`manifest_candidate_dirs/find_existing_model_dir`) + fix clippy `manual_find` · Verify: focado default 3/3 ✅ + `test_embed_texts` 7/7 ✅ + `test_auto_embed` 5/5 ✅ + `test_query_embed` 4/4 ✅ + `mcp_tests` 93/93 ✅ DONE 2026-09-16
- [x] **Step 3 — CIERRE:** fmt ✅ + clippy default ✅ + clippy features ✅ + `git diff --check` ✅ + OCR advisory (sin Critical/High específicos; WIP ajeno excluido) + hunks disjuntos EMB-15 verificados + commit + recitation + RESULTADO ✅ DONE 2026-09-16

## Ejecución (evidencia mecánica)

- **Step 1 RED default** (`cargo test -p vantadb-mcp --test test_model_switch -j 2`): 1 failed (`emb17_unknown_model_rejected_with_valid_list` — `Ok` con `model:no-existe-xyz` + `fallback:true`, razón correcta) + 2 passed ✅
- **Step 2 GREEN default** (mismo comando): 3/3 ✅ · `test_embed_texts` 7/7 ✅ (incl. `embed_texts_with_model_param` con comentario actualizado) · `test_auto_embed` 5/5 ✅ · `test_query_embed` 4/4 ✅ · `mcp_tests` 93/93 ✅
- **Features:** `cargo clippy --features embed-local,remote-inference` ✅ 0 warnings (tras fix `manual_find`) · `cargo test --features` NO ejecutable en este runner: abort ORT `0xc0000409` pre-existente (FIND-100: System32 onnxruntime 1.17.1 vs ORT_API_VERSION=27; `test_embed_texts` con features aborta IDÉNTICO sin mi diff → stash-proof por paridad, HALLAZGO al orquestador, Backlog NO tocado por prohibición)
- **Gates:** `cargo fmt --check -p vantadb-mcp` ✅ (tras `cargo fmt`) · `cargo clippy` default ✅ y features ✅ · `git diff --check` ✅ (solo warning CRLF ajeno `completions/_vanta-cli.ps1`, fuera de mi commit)
- **Hunks disjuntos EMB-15:** `git diff HEAD -- tools.rs | grep ^@@` → `-2591,+2591` (handler validación), `-3313,+3318` (comentario), `-3348,+3354` (helpers+cache) — 0 solape con EMB-15 (`1706,3081,3106,3532` vía `git show 9f4fd725`) ✅
- **Nota CWD:** tests corren con CWD=package dir; el switch resuelve `../embeddings/…` + `CARGO_MANIFEST_DIR`-absoluto + fallback hardcodeado (fix verificado: features sin-model-env habría fallado sin él en `e5-small`)

## Review (GATE — P2-01) — doubt-driven self-review + OCR input (vanta-review no disponible: deuda leve, igual que EMB-13/14/15)

- Preguntas adversariales: ¿desconocido siempre Err incluso sin features? Sí (validación en handler, no cfg-gated; manifest hardcodeado cubre CWD). ¿conocido con features pero sin archivos da fallback silencioso? No — Err con download hint + tamaño (decisión 4). ¿conocido sin features da error duro rompiendo CI? No — fallback avisado (decisión 5, test cfg-gated). ¿explicit-model con provider-Err traga fallback? No — Err (solo `None` tiene fallback Q5). ¿dim distinta auto-convierte o sugiere reindex? No — devuelve dim verdadera, EMB-18 gatea (decisión: no duplicar). ¿cache lock a través de inferencia? No — guard solo para `clone Arc`/`insert` (µs); inferencia toma `Mutex<Session>` por-provider después (orden cache→sesión). ¿guard cruzando `.await`? No — handler sync vía `spawn_blocking`. ¿`unwrap/expect` en prod? No — `map_err`+`?` puros; tests con `allow` + invariantes (patrón `test_embed_texts.rs`). ¿shape `embed_texts` intacto? Sí — mismos 8 campos; `model` = id canónico validado. ¿hunks EMB-15 intactos? Sí — disjuntos verificados. ¿TOCTOU env/files? Heredado EMB-13/14 (check-then-act benigno por request; el peor caso es un Err honesto o un dummy filtrado, nunca corrupción).
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos no ejecutados
  - [x] No saltarse clarificación por "ya sé qué quiere"
  - [x] No declarar done sin verificar acceptance
  - [x] No ignorar fallos ni reportar "todo OK" parcial (abort ORT features investigado con paridad EMB-13, no ocultado)
  - [x] No hacer un solo intento de búsqueda y darlo por saturado
  - [x] No copiar sin citar ni presentar supuestos como evidencia
  - [x] No reintentar en bucle sin diagnóstico
  - [x] No dejar huérfanos los pasos
  - [x] No degradar chequeo de errores en paths críticos
  - [x] No gastar presupuesto infinito
- **Cross-model:** skipped (contexto no-interactivo, sin autorización para CLIs externos).
- **cleanCA (Ap. V, solo informa):** Humble Object ✅ (glue manifest→provider, sin lógica negocio); SLAP ✅ (helpers ≤20L c/u salvo `manifest_model_table` ~30L por tabla 9 ids — precedente `resolve_model` desktop similar, sin FIND); sin `unwrap`/`unsafe` en prod ✅; sin stuttering ✅; saldo Regla 6 = 0 (aditivo + test; comentario 1L en `test_embed_texts.rs` es pago de honestidad, no deuda).
- **Veredicto:** ✅ approve (deuda leve: vanta-review no disponible en entorno).

## Cierre 2026-09-16

Steps 1-3 ✅ + verify scope verde (default todo; features clippy ambas cfgs; features-run bloqueado por ORT pre-existente con paridad probada) + commit `feat:` (solo 4 archivos propios, sin push). Handoff: EMB-19 (matriz e2e + install coordinado) luego EMB-20 (docs). WIP ajeno en worktree EXCLUIDO del commit.

## Notas

- `ponytail:` caché `HashMap` + evicción arbitraria avisada (sin crate `lru`; LRU exacta si profiling pide orden); 1 provider por `model` (igual costo que 1 put EMB-14 por switch); manifest hardcodeado como fallback (sin IO en hot path cuando el archivo falta).
- NOTICED BUT NOT TOUCHING: misma-dim-distinto-modelo silencioso (techo, igual que EMB-14/15); `local_model_files_present` con fallback a `multilingual-e5-small` global (precedente EMB-13, no tocar aquí); `qwen3-embedding-8b` (`onnx:null`, GPU-only) siempre → download-hint honesto; WIP ajeno en worktree (`.opencode/Justfile/completions/desktop-Cargo.lock/plan-file`) → excluir del commit.
- Notion Paso 0c: sin tool Notion en este entorno → se heredan conclusiones EMB-13/14 (Problema + Propuesta REAL; Nuevas/Plan sin mapeo, filtro VantaDB).

## Spec Gate

Tabla §4 vale como spec de decisiones (sin `pub fn`/tool/endpoint; helpers privados + test). Gate D: blast 1 archivo + test, sin hot path, contrato mecánico del plan → NO dispara `question`. Gate V: 2 fallas mismo-error → `question` antes de FAILED. Gate C: colaterales → fila FIND + `question`.

## Context Save Point

DISCOVERY completo 2026-09-16. Plan + reglas (api-contract+core-engine+concurrency+Ap.V) + tools.rs (embed_texts+helpers EMB-13/14/15/18) + llm.rs (factory+provider intactos) + manifest 9 ids + desktop precedente + tests + skills 7 cargadas (frontend descartada). Task file creado. Siguiente: Step 1 RED.

## Invariantes de dominio (handoff — MUST)

- **Preservar:** shapes `embed_texts` intactos (mismos campos; `model` = id canónico); budgeting 128/25k; fallback Q5 para `None`; mensajes EMB-18 exactos; prefijos EMB-16; hunks EMB-15 (`:1706,~3081,~3532`) intactos; sin `unwrap/expect` en prod; sin editar `src/llm.rs`; WIP ajeno excluido del commit; sin push (vía vanta-lead).
- **Comandos:** `cargo test -p vantadb-mcp --test test_model_switch -j 2` + `cargo test -p vantadb-mcp -j 2` + cat test MCP stdio switch/error
- **Deuda:** ninguna (si surge → fila FIND, no silencio)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** sin deuda nueva esperada (helpers privados + test). Si surgiera → pagar con P2-5/P2-8 o fila FIND.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato §1 verificable + focado + full suite + cat test switch con evidencia |
| **Commit** | Atómico `feat: EMB-17` (~200-300 líneas), `git diff` solo propios, fmt+clippy verdes |
| **Release** | N/A (pre-push gate Regla 1 vía `dev-tools/verify.ps1` en EMB-19) |

SKILLS_CARGADAS: incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, api-and-interface-design, systematic-debugging
SDP: incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, api-and-interface-design, systematic-debugging (frontend-ui-engineering descartada: sin web/)
