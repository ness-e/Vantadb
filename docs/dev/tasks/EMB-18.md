# EMB-18 — regla una-dim-por-base (Q4: bloquear+guiar)

> **Plan:** `docs/dev/plans/2026-09-16-embeddings-auto.md` (Wave2, Q4 owner)
> **Estado:** ⏳ IN PROGRESS
> **Appetite:** 4h · 🟢 · 🟡
> **Branch/Commit:** develop / `feat: EMB-18`
> **Cynefin:** 🟦 obvio · ⬆️ 0 / ⬇️ 2 steps
> **Ruta:** vanta-worker
> **SDP:** campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design (frontend-ui-engineering descartada: sin `web/`, scope discipline)

## 1. TAREA

**Objetivo:** cuando la base contiene vectores de dim distinta a la del vector entrante (modelo activo cambiado), bloquear con error claro + comando exacto de regeneración. Nunca auto-reindex silencioso (Q4 owner).

**Contrato exacto (ley):**
1. Base con dim distinta al modelo activo → error que dice dim esperada + obtenida + comando exacto de regeneración (`rebuild_index` MCP / SDK `reindex_hnsw_from_text`).
2. Test que prueba 1 (mismatch bloquea, guía presente, esperado/obtenido presentes).
3. Solo gatear cuando hay vectores: base vacía/mixta-vacía define su dim en el primer put con vector (sin gate) — pre-mortem ya implementado en `index_vector_dim` → `None`, NO romper.
4. Puts sin vector (solo texto) nunca gateados.

**Acceptance del plan:** Q4 owner (error claro + comando sugerido, nunca auto-reindex silencioso). Pre-mortem: falso positivo en base vacía → solo gatear con vectores.

## 2. ARCHIVOS

**Clave (escritura):**
- `vantadb-mcp/src/handlers/tools.rs:1236-1248` (gate `memory_put` — enriquecer mensaje)
- `vantadb-mcp/src/handlers/tools.rs:1336-1350` (gate `memory_put_batch` — idem)
- `vantadb-mcp/src/handlers/tools.rs:1793-1803` (gate search vector `memory_search`-style — idem)
- `vantadb-mcp/src/handlers/tools.rs:3029-3045` (gate `parse_search_request` MCP-04 — idem)
- `vantadb-mcp/src/handlers/tools.rs:2848-2864` (helper privado nuevo `dim_mismatch_guidance` junto a `index_vector_dim` — aditivo, sin `pub` nuevo)
- `vantadb-mcp/tests/mcp_tests.rs` (tests nuevos EMB-18; existentes 1308-1336 y 3316-3363 NO se tocan salvo que fallen por prefijo — prefijo se conserva)

**Relacionados (solo lectura):**
- `src/error.rs:132-138` (`DimensionMismatch { expected, got }` — existe, NO se modifica; mensaje se conserva como prefijo)
- `src/sdk/api/admin.rs:45-109` (`reindex_hnsw_from_text` — comando SDK a sugerir; rebuild real desde vectores almacenados)
- `vantadb-mcp/src/handlers/tools.rs:2204-2210` (tool MCP `rebuild_index` — comando MCP a sugerir)
- `vantadb-mcp/src/validation.rs:478-490` (`error_content` / `error_content_vanta` — helpers existentes, se reutilizan)
- `src/config.rs:295-309` (`LlmCfg`: provider default `ollama`, `local_model_path` — modelo activo es config, no se toca)
- `embeddings/manifest.json` (dims por id 384/512/768/1024/4096 — referencia para el mensaje, no se toca)
- `src/llm.rs` (EMB-16 — PROHIBIDO escritura)

**Prohibidos (NO TOCAR):** `.opencode/`, `Justfile`, `completions/_vanta-cli*`, `desktop/src-tauri/Cargo.lock`, `ocr-delegate.yml`, `ocr-review.ps1`, `reparacion.bat`, `docs/pipeline-state.json`, plan file (solo recitation orquestador), `stash@{0..14}`, `src/llm.rs` escritura (EMB-16), zona embed de `tools.rs` :978/:2604-2618/:3209+ (EMB-13 WIP), `Cargo.toml`, `~/.cargo/bin`, Backlog/avance (orquestador).

## 3. DEPENDENCIAS

- **Wave2** paralela (server.rs/docs vs tools.rs vs llm.rs). BLOQUEANTES: ninguna (EMB-10 ✅ contexto).
- **Paralelas:** EMB-13 (tools.rs zonas embed :978/:2604-2618/:3209+ — WIP no commiteado en worktree; mis hunks :1236/:1336/:1793/:2848/:3034 NO solapan → commit parcial por hunks, solo propios) + EMB-16 (`src/llm.rs` — prohibido escribir).
- **Previa EMB-12 ✅ / nextTask EMB-14.** Protege cambios de modelo (EMB-11/17): el gate es lo que EMB-17 usará al hacer switch por nombre.
- **HALLAZGO DISCOVERY vs plan:** el plan cita `server.rs` como zona de gate; evidencia real: los 4 gates viven en `handlers/tools.rs` (AUD-046/MCP-04), `server.rs` solo hace dispatch. Decisión: gatear en `tools.rs` (donde está el trust boundary validado), sin tocar `server.rs`. `error.rs:132` confirmado existente.

## 4. REFERENCIAS

- `.opencode/rules/server-mcp.md` (LEÍDA COMPLETA: R-1 coherencia tool/doc — sugerir tool real `rebuild_index`, no inventado; R-2 handlers sync vía `spawn_blocking` — mi cambio es sync puro, no bloquea loop; R-3 métricas — no aplica)
- `clean-code-clean-architecture.md` Apéndice V (LEÍDO: V.1 `vantadb-mcp` = Frameworks/Drivers Humble Object — glue legítimo: traduce error core→mensaje accionable, cero lógica de negocio; SLAP ≤20 líneas — helper ~12 líneas; `Result`, sin `unwrap` en prod — `format!` puro)
- `documentation-and-adrs` (mensaje accionable: qué pasó + dim esperada/obtenida + qué hacer + comando exacto)
- `doubt-driven-development` (CARGADA: desconfiar del falso-positivo — base vacía sin gate; y de la mentira útil — `rebuild_index` NO cambia dims, la guía debe decir re-embeder o re-ingerir primero)
- Problema dimensión semántica (LEÍDA COMPLETA vía Notion): mezclar dims corrompe ranking en silencio (garbage-in=garbage-out) → justifica gate explícito.
- Propuesta matriz REAL (LEÍDA COMPLETA vía Notion): retrieval híbrido BM25+HNSW+RRF REAL; `extract_skills`/gobernanza PROPUESTA — fuera de scope.
- Nuevas features / Plan de accion (LEÍDAS, índices): sin mapeo → no aplican (filtro VantaDB).
- **Spec (tabla — sin símbolos públicos nuevos, Gate D evaluado abajo):**

| Decisión | Opción elegida | Alternativa descartada | Por qué |
|---|---|---|---|
| Dónde gatear | `tools.rs` 4 sites existentes | `server.rs` arranque (plan) | Evidencia: trust boundary ya validado ahí (AUD-046/MCP-04); `server.rs` solo despacha |
| Forma del mensaje | Prefijo existente + `Hint:` accionable | Nueva variante de error en `error.rs` | No rompe tests/consumidores que matchean el prefijo; sin `pub` nuevo → Gate D no dispara |
| Comando sugerido | MCP `rebuild_index` + SDK `reindex_hnsw_from_text(ns, page_size)` | Solo uno de los dos | Agente MCP usa tool; operador Rust usa SDK; ambos reales y verificados |
| Honestidad del fix | Re-embeder con dim original O re-ingerir todo + rebuild | "Corre `rebuild_index` y listo" | `rebuild_index` reconstruye desde vectores almacenados — NO cambia dims; sugerirlo solo sería mentira útil |
| Base vacía | Sin gate (`index_vector_dim` → `None` intacto) | Gatear siempre | Pre-mortem plan: falso positivo; primera escritura define la dim |
| Auto-reindex | Nunca (Q4 owner) | Reindex automático al detectar mismatch | Owner eligió bloquear+guiar; auto silencioso = corrupción silenciosa inversa |

- **Gate D (question-gates.md):** NO dispara — blast radius 2 archivos (tools.rs + mcp_tests.rs), sin símbolos `pub` nuevos (helper privado), sin hot path (cold per-request validation), contrato no ambiguo, no feature-add greenfield (wiring sobre comportamiento existente). Sin `question`; se procede.

## 5. SKILLS (SDP v2 phase=BUILD, keywords dim-guard/mismatch-error/reindex-guide/mcp-server)

- `campaign-executor` — base MCP server: state machine PLAN→ACT→VERIFY + RESULTADO.
- `source-driven-development` — base MCP: comando sugerido debe existir de verdad (verificado en código, no memoria).
- `incremental-implementation` — lifecycle BUILD: slices verticales (RED test → GREEN helper+sites → verify → commit parcial).
- `test-driven-development` — lifecycle BUILD: RED→GREEN→REFACTOR; bug sin reproduction test no cuenta.
- `context-engineering` — lifecycle BUILD: context pack Rules→Plan→Source→Error (usado en este file).
- `doubt-driven-development` — lifecycle BUILD: stakes (corrupción silenciosa de índice) → adversarial review del mensaje.
- `api-and-interface-design` — lifecycle BUILD: error como contrato (prefijo estable + Hint aditivo, Hyrum's Law).
- `frontend-ui-engineering` — DESCARTADA (sin `web/`, scope discipline).

## 6. HERRAMIENTAS + MCP

- `cargo test -p vantadb-mcp -j 2 --test mcp_tests dim` (focado RED/GREEN) + `cargo test -p vantadb-mcp -j 2` (suite, antes de cierre)
- MCP stdio provocando mismatch (contrato alterno si hay tiempo: put 4d luego put 2d → error con Hint)
- `git diff --check` (whitespace) + `cargo fmt --check` (o `rustfmt --check` del archivo) + `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` (si tiempo)
- `cargo -j 2` siempre (rustc crash paralelo / OOM)
- MCP usados: codegraph_explore (blast radius — respondió parcial, se completó con grep dirigido), campaign_detect_task_type (mcp), campaign_discover_skills_v2 (8 skills), check_index_coverage (sin issues registrados; freshness metadata_changed → se leyó fuente directa).
- Commit parcial por hunks (solo propios — EMB-13 WIP convive en mismo archivo): `git diff` → filtrar hunks propios → `git apply --cached` → commit. NUNCA `git add` del archivo completo.

## 7. INVESTIGACIÓN CÓDIGO (blast radius — DISCOVERY 2026-09-16)

- **Dónde detectar dim de base:** `index_vector_dim(storage)` (tools.rs:2854) — deriva dim del primer nodo con vector vía `vec_index().all_node_ids()` + `stored_vector(id)?.as_f32_slice()`. Fría por request (no hot loop).
- **Dim del modelo activo:** no se compara directamente en ningún gate — los gates comparan vector ENTRANTE vs dim de base. El caso "modelo activo distinto" se manifiesta como: `embed_texts` devuelve dim nueva → `memory_put` con ese vector → gate dispara. Correcto por construcción.
- **Error existente:** `Error::DimensionMismatch { expected, got }` (error.rs:132) → `"Vector dimension mismatch: expected {expected}, got {got}"`. 4 sites lo formatean con `.to_string()` SIN guía (1239-1245, 1340-1346, 1795-1801, 3036-3042).
- **Comando reindex:** MCP tool `rebuild_index` (tools.rs:2204, SDK `rebuild_index`) + SDK `reindex_hnsw_from_text(ns, page_size)` (admin.rs:45, pagina `list()` + `engine.rebuild_vector_index()`). FIND-79 = tests de este último (solo `test_reindex_hnsw_from_text_no_engine` en api.rs:191 + uso en mcp_tests).
- **Base vacía:** `index_vector_dim` → `None` → skip validación (comentario 2848-2853). Tests 1351-1364 (put sin vector OK) lo cubren parcialmente; mi test añade base vacía + vector cualquier-dim OK.
- **Riesgo falso positivo:** nulo si se conserva el `if let Some(expected)` — el helper solo cambia el TEXTO del error, no la condición.
- **Entrantes a `handle_tools_call`:** `server.rs:dispatch_request` (tools/call, con semáforo + spawn_blocking) + proxy handler + tests. Mi cambio no altera firma ni flujo.

## 8. INVESTIGACIÓN PROBLEMA

Mezclar dims corrompe ranking en silencio: el vector entra al HNSW, `vector_count` sube pero el nodo jamás aparece en search (distancias basura ~0.0) — AUD-046 lo documenta. Tradeoff bloquear vs auto-regenerar: auto-reindex silencioso destruiría la base original sin consentimiento (y `rebuild_index` ni siquiera cambia dims — regeneraría basura con otra forma). Owner eligió bloquear+guiar (Q4). El error debe ser accionable por un agente: qué pasó (dims), qué hacer (2 caminos honestos), comando exacto (tool + SDK).

## 9. INVESTIGACIÓN INTERNET

No se espera (todo local: error existente + tool existente + tests existentes). Sin red para validación externa → [cita NO VERIFICADA — sin red] si se citara algo externo. TSYS-13: sin URLs citadas en evidencia → gate de citas N/A.

## 10. VALIDACIÓN + CIERRE

- Verify contrato: test nuevo RED→GREEN (mismatch lleva esperado+obtenido+`rebuild_index`+`reindex_hnsw_from_text`; base vacía sin gate; put sin vector OK) + `cargo test -p vantadb-mcp -j 2` verde + `git diff --check` + fmt.
- Verify full (pipeline): fmt + clippy + nextest audit + validate-docs + OCR review — adaptado a tiempo; mínimo: tests paquete + fmt + diff-check + OCR si disponible.
- `/cleanCA` scope: `vantadb-mcp/src/handlers/tools.rs` zonas propias (advisory).
- DoD 3 niveles: contrato ✅ + task file + recitation (plan file lo hace el orquestador — prohibido aquí) + commit `feat: EMB-18` parcial (solo hunks propios) + sin push (vía vanta-lead).
- Reviewer P2-01: agente distinto del implementador (orquestador asigna).
- Gates: P no (decisiones Q1-Q5 ya del owner) · D no (ver §4) · V solo si 2 fallas mismo-error · C al cierre (colaterales → FIND-*).
- RESULTADO §7 + Save Point abajo.

## Impacto mapeado (Regla 0)

- **Leídos (zonas exactas):** tools.rs §1-60 (registry), §1225-1424 (put/put_batch/get), §1780-1859 (search vector), §2198-2247 (`rebuild_index`), §2595-2694 (`embed_texts` EMB-13 WIP), §2840-3110 (`index_vector_dim`+`parse_search_request`), validation.rs §380-509 (helpers), error.rs §110-169 + test §539-546, admin.rs §1-109, config.rs §295-309, manifest.json (completo), server-mcp.md (completa), mcp_tests.rs §1295-1364 + §3305-3364.
- **Hacia dentro (mis zonas usan):** `index_vector_dim`, `error_content`, `Error::DimensionMismatch`, `McpError`, `validate_vector`, `parse_memory_input`.
- **Entrantes (dependen de mis zonas):** `dispatch_request`/`mcp_proxy_handler` (server.rs) → `handle_tools_call` → 4 gates; tests mcp_tests (2 tests matchean prefijo — se conserva).
- **Veredicto:** impacto BAJO, aditivo + 4 reemplazos de texto. Sin cambio de firma/flujo/condición. Rollback = revert 1 commit parcial.

## Steps atómicos

- [x] **Step 0 DISCOVERY** — tipo mcp, SDP 7+1, blast radius, Spec, task file (este archivo). Verify: file existe + Gate D evaluado.
- [x] **Step 1 RED** — tests `emb18_dim_mismatch_blocks_with_regen_guidance` (+ `emb18_empty_base_defines_dim_no_gate`) en mcp_tests.rs:4996+; corrió y FALLÓ por razón correcta (`must name the MCP regen tool` — guía ausente, prefijo ya existía). 1 passed / 1 failed.
- [x] **Step 2 GREEN** — helper privado `dim_mismatch_guidance` (tools.rs:2849-2861) + 4 sites (put/batch/search_semantic/parse_search_request); 2/2 emb18 ✅; existentes intactos (`test_mcp_put_validates_vector_dimensions` y batch dim test pasan en suite).
- [x] **Step 3 CIERRE** — suite `cargo test -p vantadb-mcp -j 2` toda verde (93 mcp_tests incl. 2 nuevos) + `cargo fmt --check -p vantadb-mcp` ✅ + `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` ✅ (0 warnings) + `git diff --check` ✅ + commit parcial por hunks (solo propios).

## Context Save Point

- **Objetivo:** EMB-18 Q4 bloquear+guiar (ver §1 contrato).
- **Estado:** Step 0 ✅. Siguiente: Step 1 RED.
- **Archivos:** tools.rs (4 sites + helper @~2854), mcp_tests.rs (tests nuevos al final, patrón `setup_storage`/`default_config`/`handle_tools_call`).
- **Decisiones:** mensaje = prefijo + Hint (ver §4); comando = `rebuild_index` + `reindex_hnsw_from_text`; base vacía intacta; commit parcial por hunks.
- **WIP ajeno:** EMB-13 hunks tools.rs :978/:2604-2618/:3209+ (NO tocar); EMB-16 src/llm.rs (NO tocar); prohibidos ver §2.
- **Comandos:** `cargo test -p vantadb-mcp -j 2 --test mcp_tests emb18` → `cargo test -p vantadb-mcp -j 2` → `git diff --check`.
