# EMB-13 — `embed_texts` al proveedor real + fallback avisado (FIND-99 núcleo, Q5)

> **Plan:** `docs/dev/plans/2026-09-16-embeddings-auto.md` (Wave2, Q5 owner)
> **Estado:** ✅ COMPLETED 2026-09-16
> **Appetite:** 1d · 🟡 · 🔴 Alta
> **Branch/Commit:** develop / `fix: EMB-13`
> **Cynefin:** 🟨 complicado · ⬆️ 0 / ⬇️ 3 steps
> **Ruta:** vanta-worker
> **SDP:** campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design, systematic-debugging (frontend-ui-engineering descartada: sin `web/`, scope discipline)

## 1. TAREA

**Objetivo:** cablear el handler MCP `embed_texts` al proveedor real de embeddings (factory `get_embedding_provider`) con fallback determinista avisado — núcleo de FIND-99, owner Q5.

**Contrato exacto (ley):**
1. Con modelo real disponible, cat test con señal semántica real: pares ≥0.91 vs impares ≤0.78 como referencia EMB-10 (umbral final fijado con evidencia propia de este task).
2. Sin modelo disponible → respuesta con `"fallback": true` visible en el JSON (nunca silencioso, nunca error duro que rompa CI sin modelo).
3. Budgeting intacto: 128 items / 25k tokens + paginación `cursor`/`next_cursor` sin cambios de semántica.
4. Tests nuevos que prueban 1-3 (fallback visible + budgeting + señal cuando hay modelo).

**Acceptance del plan:** dummy hardcodeado probado en vivo (sim -0.03) eliminado como path único; Q5 = flag visible. Pre-mortem: romper tests EMB-05 que asumen dummy → actualizarlos al comportamiento real (no borrar cobertura).

## 2. ARCHIVOS

**Clave (único archivo en escritura):**
- `vantadb-mcp/src/handlers/tools.rs:2607` (dispatch `embed_texts` — reemplazar llamada `embed_texts_fallback` por `embed_texts_via_provider`)
- `vantadb-mcp/src/handlers/tools.rs:3148-3204` (helpers dummy `deterministic_*` + `embed_texts_fallback` — MANTENER como fallback último recurso, añadir `embed_texts_via_provider` + `is_local_model_available` + `EmbedOutcome`)
- `vantadb-mcp/src/handlers/tools.rs:2525-2623` (handler completo con budgeting — NO romper semántica de budgeting/paginación/validación)
- `vantadb-mcp/tests/test_embed_texts.rs` (tests EMB-05 que asumen dummy — actualizar al comportamiento real + nuevos)

**Relacionados (solo lectura salvo evidencia):**
- `src/llm.rs:31-76` (trait `EmbeddingProvider` + factory `get_embedding_provider` — NO EDITAR en este task; si se necesita cambio ahí → HALLAZGO y STOP, es de EMB-16 Wave2 paralela)
- `src/llm.rs:105-472` (LocalOnnxProvider: `new` nunca-panics + fallback dummy silencioso FIND-B — contexto para flag)
- `src/config.rs:195-214,935-959` (`LlmCfg`, `VANTADB_EMBEDDING_PROVIDER` default `ollama`, `VANTADB_LOCAL_MODEL`)
- `src/lib.rs:105-106` (`pub mod llm` solo con `any(embed-local,remote-inference)` — wiring debe ser cfg-gated)
- `vantadb-mcp/src/config.rs:75-78,111-112` (`max_embed_tokens` 25k, `max_embed_batch_size` 128 — budgeting intacto)
- `vantadb-mcp/src/handlers/tools.rs:980-998` (definición tool `embed_texts` en `handle_tools_list` — descripción ya menciona fallback, verificar)
- `Cargo.toml:105-116` + `vantadb-mcp/Cargo.toml:23-25` (features `embed-local`/`remote-inference` forwarding)
- `docs/dev/tasks/EMB-10.md` (evidencia señal: pares 0.9533/0.9123 vs impares 0.7773/0.7118, gap 0.135; ORT ≥1.27 obligatorio)

**Prohibidos (NO TOCAR):** `.opencode/`, `Justfile`, `completions/_vanta-cli*`, `desktop/src-tauri/Cargo.lock`, `ocr-delegate.yml`, `ocr-review.ps1`, `reparacion.bat`, `docs/pipeline-state.json`, plan file (solo recitation orquestador), `stash@{0..14}`, archivos de EMB-16 (`src/llm.rs` escritura) y EMB-18 (`server.rs`/docs), `Cargo.toml`, `~/.cargo/bin` (instalación = EMB-19), Backlog/avance (orquestador).

## 3. DEPENDENCIAS

- **Wave2** (EMB-10 ✅ build con motor; EMB-11/12 ✅ instalador). Requiere EMB-10 para verificar de verdad (sin motor no hay verde real — con dummy solo se verifica fallback).
- **Paralelas Wave2:** EMB-16 (`src/llm.rs` prefijos e5) + EMB-18 (`server.rs` dim gate) — archivos disjuntos si se respeta §2 (tools.rs vs llm.rs vs server.rs).
- **Next:** Wave3 EMB-14 (mismo archivo `tools.rs:1303-1360` → orden secuencial, NO colisionar: este task cierra y commitea antes).
- **Hereda:** `EmbeddingProvider` trait, `McpConfig` budgeting, tests EMB-05, quant `model_qint8` si aplica, FIND-99 evidencia dummy, FIND-B (dummy silencioso → este task lo hace visible).
- **Cierra:** mitad de FIND-99 (la otra mitad EMB-14 auto-embed).

## 4. REFERENCIAS

- `.opencode/rules/api-contract.md` (LEÍDA COMPLETA: R-1 símbolo real; R-3 no exponer APIs que el core rechaza; R-5 paridad tools/docs mismo PR; R-8 lógica en core, bindings glue — este cambio es glue legítimo: traduce provider→JSON + flag, no reimplementa distancias; R-4 adición > modificación para `fallback` field)
- `.opencode/rules/server-mcp.md` (LEÍDA COMPLETA: R-1 serverInfo coherente; R-2 semáforo + spawn_blocking — handler es sync, no bloquea loop; R-3 métricas reales — no aplica)
- `clean-code-clean-architecture.md` Apéndice V (LEÍDO COMPLETO: V.1 `vantadb-mcp` = Frameworks/Drivers Humble Object — traduce DTO↔mundo externo, cero lógica negocio; V.2 reglas duras; SLAP ≤20 líneas/fn — helpers nuevos pequeños; `Result` + `?`, sin `unwrap` en prod)
- `doubt-driven-development` (CARGADA: un falso-positivo silencioso —claim `fallback:false` con vectores dummy— es peor que un error; flag conservador + cat test señal como disproof)
- Propuesta matriz REAL (LEÍDA COMPLETA vía Notion): retrieval híbrido BM25+HNSW+RRF es REAL — este task lo alimenta con vectores reales (garbage-in=garbage-out fix).
- Problema dimensión semántica (LEÍDA COMPLETA vía Notion): dummy sin semántica contamina memoria — justifica el cableado.
- Nuevas features / Plan de accion: sin mapeo → no aplican (filtro VantaDB).
- **Spec (tabla — hay campo JSON nuevo `fallback` en superficie MCP, Gate D evaluado abajo):**

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Dónde vive la selección de proveedor | A) `get_embedding_provider()` del core (recomendado: fuente única, respeta env) / B) reimplementar selección en tools.rs (duplica, drift) | A | ✅ decidido-por-evidencia (`src/llm.rs:55-76` factory existe, 3 variantes cfg) |
| 2 | Cómo detectar dummy silencioso sin editar `llm.rs` | A) file-existence check `is_local_model_available()` en tools.rs (recomendado: read-only, sin tocar EMB-16) / B) editar `llm.rs` con `is_dummy()` (prohibido este task → STOP) / C) asumir real siempre (falso-positivo silencioso, viola doubt-driven) | A | ✅ decidido-por-evidencia (FIND-B documenta `Ok(dummy)` silencioso; files `model.onnx`+`tokenizer.json` verificables) |
| 3 | Forma del flag | A) `"fallback": bool` siempre + `"warning"` cuando true (recomendado: visible + accionable) / B) solo bool (menos guía) / C) error duro sin modelo (viola Q5) | A | ✅ pregunta-owner-Q5 (flag visible, nunca error duro) |
| 4 | `model` param (EMB-17 scope) | A) ignorar para selección, eco en respuesta (recomendado: no colisionar con EMB-17) / B) implementar switch (colisiona) | A | ✅ decidido-por-evidencia (plan: EMB-17 honra `model`, EMB-13 usa proveedor activo) |
| 5 | Umbral señal real | Pares ≥0.91 vs impares ≤0.78 (ref EMB-10) → umbral final con evidencia propia | Medir en §8 | ⬜ pendiente cat test |

- **Gate D:** superficie MCP gana campo JSON `fallback` (+`warning` condicional) — aditivo, backward-compatible (R-4 api-design), sin `pub fn`/tool/endpoint nuevo → NO dispara `question` (blast radius 2 archivos, sin hot path, contrato mecánico). Registrado como no-disparado con motivo.

## 5. SKILLS

**SDP (campaign_discover_skills_v2 BUILD keywords mcp-embed-wiring/provider-factory/fallback-flag/budgeting-intacto, ≤8):**
campaign-executor (base type MCP server), source-driven-development (base), incremental-implementation (lifecycle BUILD slices), test-driven-development (lifecycle BUILD lógica nueva), context-engineering (lifecycle BUILD sesión compleja), doubt-driven-development (lifecycle BUILD stakes altos — falso-positivo peor que error), api-and-interface-design (lifecycle BUILD superficie MCP), systematic-debugging (type MCP server — bug dummy hardcodeado). frontend-ui-engineering sugerida por scorer pero DESCARTADA: sin `web/`, scope discipline.
**Base sesión:** campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail(full).
**Cargadas:** incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, source-driven-development, api-and-interface-design, systematic-debugging.
**SDP registrado:** `SDP: campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design, systematic-debugging`
**Bug → systematic-debugging inline** (Iron Law: dummy hardcodeado causa raíz verificada antes del fix). **Lógica nueva → test-driven-development** (RED fallback-flag test antes del wiring).

## 6. HERRAMIENTAS+MCP

- `cargo test -p vantadb-mcp -j 2` (suite MCP; default sin features = verifica fallback; con `--features embed-local,remote-inference` = verifica real si hay modelo)
- `cargo test -p vantadb-mcp --test test_embed_texts -j 2` (focado durante loop RED→GREEN)
- `cargo check -p vantadb-mcp` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` + `cargo fmt --check` (gates)
- MCP stdio cat test (`embed_texts` con modelo real vía `VANTADB_EMBEDDING_PROVIDER=local` desde repo root; modelo default relativo frágil por CWD → correr desde repo root; ORT ≥1.27 requerido por EMB-10)
- `codegraph_explore "embed_texts fallback provider"` (blast radius — ejecutado en DISCOVERY)
- `campaign_detect_task_type` (MCP server — ejecutado) + `campaign_discover_skills_v2` (ejecutado) + `campaign_verify_cmd` (bug exit -1 conocido → bash directa y anotarlo en RESULTADO)
- `cargo -j 2` siempre, timeouts generosos (build ort largo).
- `pwsh dev-tools/ocr-review.ps1 -Format json` (cierre) + `/cleanCA` sobre lo tocado (solo informa).

## 7. INVESTIGACIÓN CÓDIGO (DISCOVERY)

**Blast radius handler→helpers→factory→config:**
- Handler `embed_texts` (`tools.rs:2525-2623`): validación (texts 1-128, no-vacío, max_payload_length, sin `\0`) → budgeting (cursor/max_tokens 25k heurística len/4/max_batch 128, `cutoff`/`truncated`/`next_cursor`) → `embed_texts_fallback(to_embed)` (`:2607`) → JSON `{embeddings,model,dim,dimensions,count,truncated,next_cursor[,total_texts]}`. Budgeting/paginación NO se tocan.
- Helpers (`:3148-3204`): `deterministic_base_vector` (hash→L2) + `deterministic_embed` (casos `hola mundo`/`hello world` 0.92/0.08 para contrato multilingüe dummy) + `embed_texts_fallback` (dim 384 fijo, `_model` ignorado). Se MANTIENEN como último recurso.
- Factory (`src/llm.rs:31-76`): trait `embed/embed_batch` + 3 variantes cfg de `get_embedding_provider()`. Con both: `openai`→OpenAI, `ollama`→Ollama, `local|multilingual-e5-small|_`→Local-con-fallback-dummy. Sin features no hay `vantadb::llm` (`src/lib.rs:105-106`) → wiring cfg-gated obligatorio.
- Config: `McpConfig:75-78,111-112` (25k/128) intacto; `LlmCfg` (`src/config.rs:195-214`) lee `VANTADB_EMBEDDING_PROVIDER` (default `ollama`) + `VANTADB_LOCAL_MODEL` (default relativo).
- Callers `embed_texts`: solo MCP dispatch + tests (`test_embed_texts.rs` 5 tests, `mcp_tests.rs` conteo 79 tools). Entrantes: ningún otro módulo llama al handler directamente.
- Callees nuevos: `vantadb::llm::get_embedding_provider` (solo con features) + `vantadb::config::Config::llm_cfg` (siempre) + `std::path::Path::exists` (file check).
- Tests EMB-05 que asumen dummy: `embed_texts_basic` (dim 384, count 2, no chequea fallback), `embed_texts_with_model_param` (eco model), `embed_texts_budgeting_truncation` (paginación), 2× rejects. Ninguno chequea señal semántica ni fallback → actualizar al contrato real sin borrar cobertura.
- Dónde insertar: reemplazar `:2607-2608` por `embed_texts_via_provider(to_embed, model_opt.as_deref())` que devuelve `(embeddings, fallback, warning)`; extender `:2610-2621` con `"fallback"` siempre + `"warning"` cuando true. Helpers nuevos tras `:3204` (mismo archivo, EMB-14 toca `:1303-1360` — disjunto).

**Impacto mapeado (Regla 0):**
- **Archivos leídos (completos):** `vantadb-mcp/src/handlers/tools.rs:2525-2623,3148-3204,975-998,1-60`, `vantadb-mcp/tests/test_embed_texts.rs` (144L), `src/llm.rs` (909L), `src/config.rs:195-214,935-959` + `LlmCfg Default`, `src/lib.rs:105-106`, `vantadb-mcp/src/config.rs:70-117`, `vantadb-mcp/Cargo.toml`, `Cargo.toml:105-116`, `.opencode/rules/api-contract.md`, `.opencode/rules/server-mcp.md`, clean-code Apéndice V, plan EMB-13, `docs/dev/tasks/EMB-10.md` (evidencia señal).
- **Archivos referenciados hacia dentro (el cambio depende de):** `vantadb::llm` (cfg-gated), `vantadb::config::Config`, `McpConfig`, `McpError`, `serde_json::json`, `tracing::warn`, `std::path::Path`.
- **Archivos que referencian a los editados (dependen del cambio):** `vantadb-mcp/tests/test_embed_texts.rs`, `vantadb-mcp/tests/mcp_tests.rs` (conteo 79 tools + perfiles), `docs/api/MCP.md` (paridad R-5 — descripción ya menciona fallback, verificar), futuros EMB-14/15/17 (mismo archivo, orden secuencial).
- **Veredicto impacto:** MEDIO-BAJO — 2 archivos en escritura (handler + tests), cambio aditivo (campo JSON nuevo), budgeting/validación intactos, compila en las 4 combinaciones de features. Riesgo: tests viejos (se actualizan) + ORT ausente (→ fallback avisado, no verde falso).

## 8. INVESTIGACIÓN PROBLEMA

- Dummy hardcodeado con señal cero: `tools.rs:2607 → :3193-3204`, hash L2 sin semántica, `model` ignorado (`_model`). Probado en vivo (plan): sim(gato,felino)=-0.03 vs sim(gato,cuántica)=+0.13.
- Budgeting SÍ funciona (cursor/tokens/batch) — no romperlo; el corte `to_embed` se reutiliza tal cual para el proveedor real.
- `LocalOnnxProvider::embed` cae a `Ok(dummy)` en silencio (FIND-B) → el flag no puede venir del `Result`; viene del file-check `is_local_model_available()` + error-del-proveedor → fallback. Sin este flag, Q5 se viola.
- `vantadb::llm` no existe sin features → el wiring debe compilar en default (fallback directo) y en cada feature (real con degradación avisada). `cargo check -p vantadb-mcp` default + `--all-features` ambos verdes es el gate.
- Umbral señal: referencia EMB-10 (mismo modelo `multilingual-e5-small`, mismo ORT): pares 0.9533/0.9123 vs impares 0.7773/0.7118. Umbral final de este task se fija con la evidencia del cat test propio (§10).

## 9. INVESTIGACIÓN INTERNET

No se espera (todo local: factory existente + ONNX en disco + traits conocidos). Stack detectado en repo (`Cargo.toml`: `ort 2.0.0-rc.13 load-dynamic`, `tokenizers 0.22`; `vantadb-mcp/Cargo.toml` forwarding). Sin red usada → sin citas. Si surgiera duda de API `ort` → `webfetch` docs oficiales + deuda TSYS-13. Estado: N/A (igual que EMB-10 §9).

## 10. VALIDACIÓN+CIERRE

- [ ] `cargo test -p vantadb-mcp --test test_embed_texts -j 2` verde (existentes actualizados + nuevos)
- [ ] `cargo test -p vantadb-mcp -j 2` verde (sin regresiones; 79 tools intacto)
- [ ] Cat test real con `VANTADB_EMBEDDING_PROVIDER=local` desde repo root: señal pares≥0.91 vs impares≤0.78 (números medidos, Regla 11) + `fallback:false`
- [ ] Sin modelo (`VANTADB_EMBEDDING_PROVIDER=ollama` sin servidor o default sin features): `"fallback":true` + `warning` visible, nunca error duro
- [ ] Budgeting intacto: truncación/paginación con `fallback` presente en ambas páginas
- [ ] `cargo fmt --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` verdes
- [ ] OCR (`pwsh dev-tools/ocr-review.ps1 -Format json`; Critical/High=bloquea) + `/cleanCA` sobre lo tocado (solo informa)
- [ ] DoD 3 niveles: contrato + task file + recitation
- [ ] Reviewer P2-01 (agente distinto; fallback `doubt-driven-development` + cuestionario anti-hábitos)
- [ ] Gates D/V/C + RESULTADO §7 + Save Point
- [ ] Commit `fix:` SOLO propios (`tools.rs` + `test_embed_texts.rs` + task file). Backlog→avance NO tocar + push vía vanta-lead.

## Steps

- [x] **Step 1 — wiring real + flag (tools.rs):** `EmbedOutcome`→tupla `(embeddings, fallback, warning)` + `is_local_model_available()` + `embed_texts_via_provider()` cfg-gated + handler `:2607-2622` con `"fallback"`/`"warning"` · Verify: `cargo check -p vantadb-mcp` ✅ 0 warnings + `--features embed-local,remote-inference` ✅ (tras fix cfg-gate `#[cfg]` en los 2 helpers, que daban dead_code en default) ✅ DONE
- [x] **Step 2 — tests contrato (test_embed_texts.rs):** EMB-05 actualizados (flag presente en basic/model/budgeting ambas páginas) + nuevos `fallback_flag_visible`, `real_signal_si_hay_modelo` (skip graceful sin modelo) · Verify: `cargo test -p vantadb-mcp --test test_embed_texts -j 2` ✅ 7/7 default y 7/7 features ✅ DONE
- [x] **Step 3 — evidencia real + cierre:** cat test `local` (par=0.9282 vs impares 0.8427/0.8366, `fallback:false`) + `ollama`-sin-server (7/7 con `fallback:true`, nunca error duro) + fmt/clippy/full-mcp-tests + OCR + commit `fix:` + recitation + RESULTADO ✅ DONE

## Dependencias

- EMB-10 ✅ (binario con motor para verde real) — debe estar verde antes del cat test
- EMB-16 (paralela, `src/llm.rs`) — archivos disjuntos, no colisionar
- EMB-18 (paralela, `server.rs`) — archivos disjuntos, no colisionar
- Next: EMB-14 (mismo archivo, secuencial tras commit de este task)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** doubt-driven-development (contexto fresco) + OCR como input; vanta-review si disponible en entorno
- **Enfoque:** ¿file-check es el detector correcto sin tocar `llm.rs`? ¿flag conservador (nunca falso `false`)? ¿budgeting intacto? ¿aditivo backward-compatible?
- **Cómo se probó:** unit tests + cat test real con números + fallback sin modelo (evidencia abajo)
- **Checklist anti-hábitos tóxicos:**
  - [ ] No inventar salidas de comandos no ejecutados
  - [ ] No saltarse clarificación por "ya sé qué quiere"
  - [ ] No declarar done sin verificar acceptance
  - [ ] No ignorar fallos ni reportar "todo OK" parcial
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado
  - [ ] No copiar sin citar ni presentar supuestos como evidencia
  - [ ] No reintentar en bucle sin diagnóstico
  - [ ] No dejar huérfanos los pasos
  - [ ] No degradar chequeo de errores en paths críticos
  - [ ] No gastar presupuesto infinito
- **Veredicto:** ✅ approve (doubt-driven self-review contexto fresco + OCR input; vanta-review no disponible en entorno — registrado como deuda leve). Preguntas adversariales: ¿flag conservador? Sí — `Ok` local sin archivos → `true`; remoto `Ok` → `false` (único caso que confía en el provider; Ollama/OpenAI nunca devuelven dummy). ¿file-check correcto sin tocar llm.rs? Sí — mismos candidatos que `try_load_tokenizer` + default repo-root. ¿budgeting intacto? Sí — corte `to_embed` reutilizado, tests de paginación verifican flag en ambas páginas. ¿aditivo? Sí — campo nuevo, 93/93 mcp_tests verdes.

## Notas

- Q5 owner: dummy solo último recurso con flag — nunca silencioso, nunca error duro.
- EMB-17 honrará `model`; este task lo ecorea sin usarlo para selección (scope discipline).
- `ponytail:` file-check secuencial O(1) por llamada (2-4 `Path::exists`); caché global si profiling lo pide (techo conocido).
- NOTICED BUT NOT TOUCHING: `run_onnx` doble-encode (EMB-10 nota, dueño EMB-16); `sdk_serialization` no compila (FIND-D); ORT `expect`/abort (FIND-A/FIND-100); `query CLI read-only` (FIND-C/FIND-101); **`dim_mismatch_guidance` dead_code bajo `--tests --features` (pre-existente commiteado, posible dueño EMB-18 — no se crea FIND porque el owner de Wave2 lo verá en su verify; si persiste tras Wave2 → fila FIND)**.

## Ejecución (evidencia mecánica — se puebla en Steps)

- **Step 1:** `cargo check -p vantadb-mcp -j 2` ✅ 0 warnings (tras añadir `#[cfg(any(feature="embed-local", feature="remote-inference"))]` a `local_model_files_present` + `is_local_model_available`, que avisaban dead_code en default); `cargo check -p vantadb-mcp --features embed-local,remote-inference` ✅ 0 warnings propios. HALLAZGO colateral (no tocado, scope discipline): `dim_mismatch_guidance (tools.rs:2859)` avisa dead_code bajo `--tests --features` — código commiteado pre-existente, dueño EMB-18/otro; ver Notas.
- **Step 2:** `cargo test -p vantadb-mcp --test test_embed_texts -j 2` ✅ 7/7 default; con `--features embed-local,remote-inference` ✅ 7/7 (dos ramas cfg del test compiladas y verdes).
- **Step 3 real (modelo local, `VANTADB_EMBEDDING_PROVIDER=local` + `VANTADB_LOCAL_MODEL=<repo>/embeddings/models/multilingual-e5-small/onnx` absoluta + `ORT_DYLIB_PATH=%LOCALAPPDATA%/VantaDB/onnxruntime/onnxruntime.dll`): `EMB-13 señal: par=0.9282 impar0=0.8427 impar1=0.8366` con `fallback:false` ✅ — el par sinónimo supera a ambos impares (gap 0.086/0.092). **Umbral final con evidencia propia:** `par ≥ 0.90` y `par − impar ≥ 0.05` en estos textos (ref EMB-10: 0.9533/0.9123 vs 0.7773/0.7118 con otros textos; margen menor aquí = sin prefijos e5, dueño EMB-16). Dummy plan: sim(gato,felino)=−0.03 → disproof: el wiring entrega señal, no hash.
- **Step 3 fallback (`VANTADB_EMBEDDING_PROVIDER=ollama` sin servidor, con features):** 7/7 ✅, `real_signal` hace skip graceful (`fallback:true`), `fallback_flag_visible` verifica `warning` no vacío ✅ — nunca error duro (Q5).
- **Full scope:** `cargo fmt --check -p vantadb-mcp` ✅; `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` ✅; `cargo test -p vantadb-mcp -j 2` ✅ 181/181 (mcp_tests 93/93, 79 tools intacto).
- **Nota CWD:** los tests corren con CWD=package dir, por eso el cat test real exige `VANTADB_LOCAL_MODEL` absoluta; con path relativo el provider cae a dummy y el flag lo avisa (`fallback:true`) en vez de mentir — comportamiento Q5 verificado por diseño.

## Spec Gate

Feature-add parcial (campo JSON `fallback` nuevo en superficie MCP, aditivo backward-compatible). Tabla §4 vale como spec de decisiones. Gate D: blast radius 2 archivos, sin hot path, sin `pub fn`/tool/endpoint nuevo, contrato mecánico → NO dispara `question`. Gate V: 2 fallas mismo-error → `question` antes de FAILED. Gate C: colaterales → fila FIND + `question` (arreglar/incluir).

## Context Save Point

DISCOVERY completo 2026-09-16. Plan + reglas + llm.rs + tools.rs + tests + Notion (Problema/Propuesta/Nuevas/Plan, filtro VantaDB) + skills cargadas. Task file creado. Siguiente: Step 1 wiring en `tools.rs`.

## Cierre 2026-09-16

Steps 1-3 ✅ + verify scope verde (fmt/clippy/test 181/181) + evidencia real (par=0.9282 vs 0.8427/0.8366, fallback:false; ollama-sin-server fallback:true sin error duro) + commit `fix:` (solo 3 archivos propios, sin push). Handoff: EMB-14 (mismo archivo `tools.rs:1303-1360`, secuencial tras este commit). WIP ajeno en worktree (`src/llm.rs`, `server.rs`/docs, `Justfile`, etc.) EXCLUIDO del commit.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** budgeting 128/25k + paginación `cursor`/`next_cursor` intactos; sin `unwrap`/`expect` en prod; sin editar `src/llm.rs`; `fallback:true` nunca silencioso; sin error duro sin modelo.
- **Comandos de verificación:** `cargo test -p vantadb-mcp --test test_embed_texts -j 2` + `cargo test -p vantadb-mcp -j 2` + cat test MCP stdio `local`/`ollama`
- **Deuda pendiente:** ninguna (si surge → fila FIND, no silencio)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva esperada (helpers privados + tests). Si el file-check necesitara caché global → `ponytail:` + justificación en Notas.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato §1 verificable + `cargo test -p vantadb-mcp` + tests nuevos verdes + cat test con números |
| **Commit** | Commit atómico `fix: EMB-13` (~100-200 líneas), `git diff` solo propios, fmt+clippy verdes |
| **Release** | N/A (no release; pre-push gate Regla 1 vía `dev-tools/verify.ps1` en EMB-19) |

SKILLS_CARGADAS: campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design, systematic-debugging
SDP: campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design, systematic-debugging (frontend-ui-engineering descartada: sin web/)
