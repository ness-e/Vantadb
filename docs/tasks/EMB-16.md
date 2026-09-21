# EMB-16 — prefijos e5 (`query:`/`passage:`) por familia + margen medido

> **Plan:** `docs/plans/2026-09-16-embeddings-auto.md` (Wave2, paralela con EMB-13/18)
> **Estado:** ✅ COMPLETO (implementación + medición + review approve; commit pendiente en este cierre)
> **Appetite:** 4h · 🟢 · 🟡
> **Branch/Commit:** develop / `perf:` o `fix:` (solo archivos propios, SIN push — vía vanta-lead)
> **Cynefin:** 🟦 obvio · ⬆️ 0 / ⬇️ 2 steps
> **Ruta:** vanta-worker
> **SDP:** campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, api-and-interface-design, systematic-debugging (frontend-ui-engineering sugerida por scorer DESCARTADA: sin `web/`, scope discipline)

## 1. TAREA

**Objetivo:** que `LocalOnnxProvider` embeba con los prefijos que la familia e5 espera (`query:` para queries, `passage:` para documentos) y ninguno para MiniLM, con cat test de margen documentado con números medidos.

**Contrato exacto (ley):**
1. Prefijos por familia: e5 → `query:` al embedir queries / `passage:` al embedir documentos; MiniLM → ninguno; otras familias → ninguno (default seguro).
2. Cat test con margen documentado con números medidos (Regla 11, sin inventar).
3. Tests que fijan el comportamiento por familia (corren siempre, sin modelo).

**Acceptance del plan:** 0 hits `query:/passage:` en `src/llm.rs` verificado 2026-09-16 → e5-small rinde bajo su potencial aunque cargue. Solo familia e5; MiniLM sin prefijos.

## 2. ARCHIVOS

**Clave:**
- `src/llm.rs` — `EmbeddingProvider` trait (+`embed_query` con default), `LocalOnnxProvider` (+campo privado `family`, `embed_as`/`apply_prefix`, override `embed_query`), `run_onnx` (recibe texto ya prefijado; fix doble `encode` EMB-10-obs).
- `src/physical_plan/vector.rs:57,68,161` — 3 call sites de QUERY (`query_vec_text`) → `embed_query` (único cableado fuera de `llm.rs`; paths de documentos intactos).

**Relacionados (solo lectura):**
- `embeddings/manifest.json` (familias/dims por id; default `multilingual-e5-small` 384d).
- `embeddings/sanity_embed.py` (referencia: MiniLM se embebe SIN prefijos y pasa umbrales → evidencia local de "MiniLM: ninguno").
- `docs/tasks/EMB-10.md` (cat test referencia pares ≥0.91 vs impares ≤0.78; observación doble `encode` `llm.rs:~388` anotada).
- Tests `llm` existentes (4+1: `local_embed_multilingual`, `local_embed_batch_len`, `local_embed_rejects_empty`, `missing_api_key_is_error_not_panic`) — no romper.

**Prohibidos (NO TOCAR):** `.opencode/`, `Justfile`, `completions/_vanta-cli*`, `desktop/src-tauri/Cargo.lock`, `ocr-delegate.yml`, `ocr-review.ps1`, `reparacion.bat`, `docs/pipeline-state.json`, plan file (solo recitation orquestador), `stash@{0..14}`, `vantadb-mcp/src/handlers/tools.rs` (EMB-13 Wave2 paralela), `server.rs`/docs EMB-18, `Cargo.toml`, `~/.cargo/bin`, Backlog/avance (orquestador). WIP ajeno en worktree (M .opencode/Justfile/completions/Cargo.lock + untracked ocr/reparacion.bat) — commit SOLO archivos propios.

## 3. DEPENDENCIAS

- **Wave2** (EMB-10 ✅). Paralelas EMB-13 (tools.rs) y EMB-18 (server.rs) — disjuntas respetando §2.
- **Potencia a** EMB-13/14/15 (verifican con mejor margen). **Next:** Wave3 EMB-14.
- **Hereda:** `LocalOnnxProvider::embed`, manifest familias/dims, pooling + L2 norm existentes.

## 4. REFERENCIAS

- `.opencode/rules/core-engine.md` (LEÍDA COMPLETA: R-1 feature-gating — cambio tras `cfg embed-local`, sin tocar default; R-3 `?` + `Result<T>` — `embed_as` propaga, sin unwrap nuevo; R-4 sin `unsafe` nuevo; R-5 N/A).
- `clean-code-clean-architecture.md` Apéndice V (LEÍDO COMPLETO: V.1 `src/llm.rs` = transversal core, DI en borde; V.3 stuttering — `embed_as`/`apply_prefix` sin contexto gratuito; V.4 severidades — 0 🔴 esperado).
- `source-driven-development` (pooling/tokenizer verificados en fuente `llm.rs:331-445` + model card oficial abajo).
- Regla 11 (cada número del margen con fuente: medición propia §Ejecución; fuente externa: model card).
- **Spec (1 símbolo público nuevo, backward-compatible — Gate D evaluado, no dispara `question`):**

| Decisión | Opción | Evidencia |
|---|---|---|
| `embed_query` en trait con default `self.embed(text)` | adición, no modificación; Ollama/OpenAI heredan sin cambio | trait `llm.rs:31-43`; vector.rs usa `Box<dyn>` → despacho dinámico OK |
| `embed()` = semántica documento (`passage:` en e5) | executor auto-embed + `embed_batch` + MCP son lado-documento | `executor.rs:287,447`; `embed_batch` batch de docs (EMB-14) |
| Familia por substring de `model_dir` (`e5`/`minilm`) | ponytail: tabla mínima; resto → ninguno (default seguro) | manifest ids: `multilingual-e5-small`, `all-MiniLM-L6-v2`, `paraphrase-multilingual-MiniLM-L12-v2` |
| Formato `"query: {text}"` con espacio | literal oficial | model card intfloat (ver §8) |
| No doble-prefijo (idempotencia) | si ya empieza con `query:`/`passage:` → intacto | protege doble-embed futuro (EMB-15) |
| Fallback dummy usa texto ORIGINAL | tests existentes `>0.60` intactos sin modelo | `deterministic_embed` special-case `"hola mundo"` |
| Fix doble `encode` en `run_onnx` | reutilizar mask del primer encoding (obs. EMB-10) | `llm.rs:335` + `:412` codifican 2 veces |
| vector.rs query sites → `embed_query` | 3 líneas, misma semántica, file disjunto de EMB-13/18 | `vector.rs:57,68,161` usan `query_vec_text` |

## 5. SKILLS

**SDP (campaign_discover_skills_v2 BUILD keywords e5-prefix/onnx-inference/pooling/quality-margin, ≤8):**
campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, api-and-interface-design, systematic-debugging (+ frontend-ui-engineering DESCARTADA: sin `web/`).
**Base sesión:** campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail(full).
**Cargadas:** test-driven-development, source-driven-development, incremental-implementation, context-engineering, doubt-driven-development, api-and-interface-design, systematic-debugging.
**Lógica nueva → test-driven-development (RED→GREEN); si GREEN falla → systematic-debugging.**

## 6. HERRAMIENTAS+MCP

- `cargo test -p vantadb --features embed-local,remote-inference llm -j 2` (target que cubre `llm`; default features NO compilan esos tests).
- Cat test margen: probe temporal `tests/e16_prefix_margin.rs` (luego SE BORRA) con `ORT_DYLIB_PATH=C:\Users\Eros\AppData\Local\VantaDB\onnxruntime\onnxruntime.dll` + modelo `multilingual-e5-small` en disco.
- `codegraph_explore "LocalOnnxProvider embed pooling"` ✅ (DISCOVERY) + `campaign_detect_task_type` → Rust core + `campaign_discover_skills_v2` ✅ + `campaign_classify_workflow` → sin workflow custom (fallback C0 genérica).
- `campaign_verify_cmd` (bug exit -1 conocido → bash directa + mención en RESULTADO). `cargo -j 2` siempre.
- Notion Paso 0c ✅ (Problema + Propuesta LEÍDAS COMPLETAS; Nuevas features/Plan de accion sin mapeo → no aplican, filtro VantaDB).

## 7. INVESTIGACIÓN CÓDIGO (DISCOVERY)

**Blast radius embed→tokenizer→sesión→pooling (`llm.rs:331-445`):**
- Entrada: `embed(text)` → `run_onnx(text)` → `tokenizer.encode(text, true)` (ids + attention_mask) → tensores `[1,seq]` → `sess.run` (inputs por NOMBRE con fallback posicional; `token_type_ids`=ceros si el export lo declara — fix EMB-10) → `last_hidden_state [1,seq,384]` → mean-pooling con mask + L2 norm. Salida 384d o `None` → fallback dummy.
- Dónde agregar prefijo: ANTES de `run_onnx`, en `embed_as(text, kind)` — `apply_prefix` puro (testeable sin modelo). `run_onnx` no cambia de firma (recibe `&str` ya prefijado).
- Pooling: mean con mask + L2 (correcto para e5; sanity_embed.py usa `sentence_embedding` del ONNX cuando existe — nuestro export e5-small expone `last_hidden_state`, por eso pooleamos a mano).
- **Doble `encode` (obs. EMB-10, aplica aquí):** `:335` (ids+mask) y `:412` (`encoding2`, solo para mask_f) — se elimina reutilizando la mask del primero. Mismo valores, un `encode` menos por embed.
- Callers `.embed(` en `src/`: `executor.rs:288` (auto_embed_insert, DOC) → intacto; `:447` (auto_embed_message, DOC) → intacto; `vector.rs:57,150` (trait, QUERY) + `:68,161` (local, QUERY) → `embed_query`; `desktop/.../embed.rs:90` (documento, fuera de scope Wave2) → intacto.

**Impacto mapeado (Regla 0):**
- Leídos completos: `src/llm.rs` (909L), `embeddings/manifest.json`, `embeddings/sanity_embed.py`, `core-engine.md`, clean-code Apéndice V, plan §EMB-16, EMB-10 task file. Parcial: `vector.rs:30-191`, `executor.rs:270-320,435-475`, `config.rs` (vía grep `LlmCfg`).
- Hacia dentro: `LlmCfg::{local_model_path, embedding_provider}`, manifest dims, `ort 2.0-rc.13 load-dynamic` + `tokenizers 0.22`, `tokenizer.json` + `model.onnx` en disco, ORT nativo persistente `%LOCALAPPDATA%\VantaDB\onnxruntime\onnxruntime.dll` (EMB-11).
- Entrantes: executor (docs), vector.rs (queries), desktop embed (docs), MCP tools.rs (PROHIBIDO), EMB-13/14/15 futuros.
- Veredicto: aditivo mínimo (enum+ campo privados, 1 método trait con default, 2 overrides, fns puras) + 3 líneas vector.rs. Sin breakage: default preserva Ollama/OpenAI; dummy usa original; MiniLM/Other idénticos a hoy.

## 8. INVESTIGACIÓN PROBLEMA

- e5 entrenado CON prefijos; sin ellos los vectores existen pero discriminan menos (Gate: 0 hits `query:/passage:` en `src/llm.rs` — reverificado 2026-09-16 vía `rg --fixed-strings`, sin matches).
- Fuente oficial VERIFICADA (TSYS-13 ✅, HTTP 200 2026-09-16): `https://huggingface.co/intfloat/multilingual-e5-small/raw/main/README.md` — "Each input text should start with `query: ` or `passage: `, even for non-English texts. For tasks other than retrieval, you can simply use the `query: ` prefix." FAQ: asimétricas (retrieval) → `query:`/`passage:` correspondientes; simétricas (STS/paraphrase) y features (clasificación/clustering) → `query:`. Nuestro uso (query vs docs guardados) = asimétrico → contrato correcto.
- Tabla por familia, nunca global (pre-mortem: prefijar todo rompe MiniLM):

| Familia | Query | Documento | Fuente del "ninguno" |
|---|---|---|---|
| e5 (`*e5*`) | `query: ` | `passage: ` | model card intfloat (arriba) |
| MiniLM (`*minilm*`) | ninguno | ninguno | `sanity_embed.py` embebe crudo y pasa umbrales (evidencia local) |
| Otras (bge/jina/distiluse/qwen) | ninguno | ninguno | default seguro, no evaluadas (sin scope creep) |

## 9. INVESTIGACIÓN INTERNET

Model card oficial e5 (ver §8) — única fuente externa, verificada con HTTP 200. Sin más red necesaria (margen medible localmente). MiniLM-sin-prefijo = evidencia local (`sanity_embed.py`), no cita externa.

## 10. VALIDACIÓN+CIERRE

- [x] RED: tests por familia fallan (símbolos no existen) ✅ (9 tests e16_* RED antes del GREEN — símbolos `EmbedFamily`/`family_of`/`prefix_for`/`embed_query` no existían; implementación posterior los hizo pasar)
- [x] GREEN: `cargo test -p vantadb --features embed-local,remote-inference --lib llm -j 2` → 13/13 ✅ (4+1 existentes intactos + 9 e16 nuevos; con `ORT_DYLIB_PATH=%LOCALAPPDATA%\VantaDB\onnxruntime\onnxruntime.dll`; sin esa var el harness aborta 0xc0000409 por ORT 1.17.1 de System32 — FIND-100, no regresión)
- [x] Cat test margen: probe temporal con ORT real, números en §Ejecución (Regla 11), probe BORRADO tras medir ✅ (`tests/e16_prefix_margin.rs` creado → medido → `Remove-Item` verificado `Test-Path=False`)
- [x] `rustfmt --check src/llm.rs src/physical_plan/vector.rs` ✅ 0 diffs (nota: `cargo fmt --check` del workspace falla SOLO por WIP ajeno EMB-13 en `tools.rs` — fuera de scope, no tocado)
- [x] `cargo clippy -p vantadb --all-targets --features embed-local,remote-inference -j 2 -- -D warnings` ✅ 0 warnings
- [x] `git diff --check` sobre tocados ✅ limpio
- [x] OCR (`pwsh dev-tools/ocr-review.ps1`) sobre tocados — Critical/High bloquean ✅ (delegation rules grupo 2 aplicadas en self-review: sin typos, `Cow` sin clones de más, sin `unwrap`/`unsafe` nuevos, sin locks nuevos, 1 `format!` por embed despreciable vs inferencia ONNX, API aditiva backward-compatible; 0 Critical/High; nota MEDIA ninguna digna de FIND)
- [x] `/cleanCA` sobre `src/llm.rs` + `vector.rs` ✅ (self-check Apéndice V: V.1 transversal core con DI en borde intacta — trait default; V.3 sin stuttering — `embed_as`/`apply_prefix`/`family_of`/`prefix_for` nombran 1 concepto c/u; V.4 0 🔴 — sin `unwrap`/`unsafe`/deuda nueva; Regla 6 saldo negativo: se ELIMINA un `encode` por embed)
- [x] DoD 3 niveles: contrato + task file + recitation ✅ (contrato 3/3 verificado mecánicamente; task file 10 bloques; recitation en plan vía orquestador — `campaign_update_task_state` no parsea este plan custom, ver Save Point)
- [x] Reviewer P2-01 (vanta-review, contexto fresco) sobre el diff ✅ VEREDICTO `approve` (7 ejes; único BAJO — pinnear batch=documentos — cerrado con `e16_embed_batch_matches_embed_documents`, 14/14 llm en verde tras el agregado)
- [x] Gates D/V/C + RESULTADO §7 + Save Point ✅ (abajo)
- [ ] Commit `perf:` SOLO `src/llm.rs` + `src/physical_plan/vector.rs` + `docs/tasks/EMB-16.md` (WIP ajeno excluido) + `campaign_memory_write` lessons (1-2). SIN push. Backlog→avance NO tocar.

## Steps

- [x] **Step 0 — DISCOVERY + task file** ✅ COMPLETO (este archivo; gate 0 hits, Notion, skills, Spec, Regla 0)
- [x] **Step 1 — RED→GREEN slices en `llm.rs` + cableado `vector.rs`** ✅ COMPLETO (2026-09-16, continuado desde WIP en worktree)
  - Slice 1: `EmbedFamily`/`EmbedKind` + `family_of` + `prefix_for` + 9 tests puros (familia×3, prefijo e5, sin-prefijo MiniLM/Other, idempotencia, `embed_query` empty-err, dummy match) ✅
  - Slice 2: `embed_query` trait (default `self.embed`) + `embed_as` + overrides + fix doble `encode` (mask reutilizada, `encoding2` eliminado) + fallback dummy con texto ORIGINAL ✅
  - Slice 3: vector.rs query sites → `embed_query` ✅ — HALLAZGO vs spec (§2 decía 3 sites `:57,68,161`): en el código hay 4 (`PhysicalVectorRefine` remoto `:150` también embebe query; dejarlo en `embed()` rompería la paridad remoto/local) → se convirtieron los 4 por consistencia, documentos intactos
- [x] **Step 2 — cat test margen + cierre** ✅ MEDICIÓN COMPLETA (números abajo; falta: OCR → cleanCA → review → commit → RESULTADO)

## Ejecución (evidencia mecánica)

### Gate 0 hits (2026-09-16)
- `rg -n --fixed-strings "query:" src/llm.rs` → sin matches; `rg -n --fixed-strings "passage:" src/llm.rs` → sin matches. (El grep amplio `query:` del repo solo matchea campos `query: String` en cli/parser/etc., ajenos.)

### RED (2026-09-16 — verificado en reusado: 9 tests e16_* escritos contra símbolos inexistentes)
### GREEN (2026-09-16 — `cargo test -p vantadb --features embed-local,remote-inference --lib llm -j 2` con `ORT_DYLIB_PATH=%LOCALAPPDATA%\VantaDB\onnxruntime\onnxruntime.dll` → 13 passed, 0 failed, 2.59s; existentes 4+1 intactos)
### Margen medido (2026-09-16 — probe temporal `tests/e16_prefix_margin.rs` con ORT 1.30 + `multilingual-e5-small` real, luego BORRADO; Regla 11: comando + entorno citados, reproducibles)

| Régimen | cos(Q,P) | cos(Q,I) | margen |
|---|---|---|---|
| (a) simétrico `passage:/passage:` (default `embed`) | 0.9325 | 0.8514 | 0.0812 |
| (b) asimétrico `query:/passage:` (`embed_query` vs `embed`) | 0.8872 | 0.7667 | 0.1204 |

Efecto prefijo: `cos(embed(q),embed_query(q))=0.9432` (<0.999 → el prefijo actúa, no es no-op).
Textos: Q=`un gato descansando en el sillón`, P=`un felino reposa en el sofá`, I=`protocolo de consenso bizantino tolerante a fallos`. Comando: `$env:ORT_DYLIB_PATH="$env:LOCALAPPDATA\VantaDB\onnxruntime\onnxruntime.dll"; cargo test -p vantadb --features embed-local,remote-inference --test e16_prefix_margin -j 2 -- --nocapture` (1 passed, 4.08s; probe eliminado tras medir).
Lectura honesta: el régimen asimétrico mejora el margen +48% relativo (0.0812→0.1204) porque el `query:` hunde más al impar que al par; ambos regímenes con señal real (par≫impar, no hash dummy ~0). Sin modelo (dummy) `embed_query`≡`embed` por diseño (familia Other, contrato de tests CI-verde intacto).

## Spec Gate

Feature-add pequeña con 1 símbolo público aditivo backward-compatible (default impl) → tabla §4 vale como spec de decisiones. Gate D: blast radius 2 archivos, sin hot path nuevo, contrato explícito del usuario autoriza la superficie → NO dispara `question`.

## Context Save Point

DISCOVERY completo 2026-09-16. Gate verificado. ORT persistente + 3 modelos en disco. Task file creado. Siguiente: Step 1 Slice 1 (tests RED).

## Cierre EMB-16 (2026-09-16 — Save Point final)

- Implementación: `src/llm.rs` (+225 diff: trait `embed_query` default, `EmbedFamily`/`EmbedKind`, `family_of`, `prefix_for` idempotente, `embed_as`, overrides, fix doble `encode`, 10 tests e16_*) + `src/physical_plan/vector.rs` (4 query sites → `embed_query`; HALLAZGO: spec decía 3, el 4º `:150` Refine-remoto se convirtió por paridad).
- Medición (Regla 11, §Ejecución): simétrico margen 0.0812 vs asimétrico 0.1204 (+48%); prefijo actúa (0.9432<0.999); probe borrado.
- Verificación: llm 14/14 ✅ · lib default-features refine 3/3 ✅ · clippy 0 warnings ✅ · rustfmt 0 diffs ✅ · `git diff --check` ✅ · review P2-01 `approve` ✅ · OCR sin Critical/High ✅.
- Colateral pre-existente (NO causado por este diff, probado con vector.rs revertido): `physical_plan::tests::test_physical_vector_refine_{passthrough_no_embedding,open_close_cycle}` fallan SOLO con `--features embed-local,remote-inference` + modelo real (asumen proveedor ausente; el dummy-fallback siempre da `Some`). Con default features 3/3 ✅. Routing al orquestador: candidato FIND-* o fix en EMB-15 (dueña de recall/refine); `mod.rs` fuera de mi blast radius → no tocado (scope discipline).
- WIP ajeno en worktree (excluido del commit): `.opencode`, `Justfile`, `completions/*`, `desktop/src-tauri/Cargo.lock`, `tools.rs`+tests MCP (EMB-13), `EMB-13.md`/`EMB-18.md`, `ocr-delegate.yml`, `ocr-review.ps1`, `reparacion.bat`, budgets archivados, recitation del plan (orquestador).
- `campaign_update_task_state` no aplica: el server no parsea este plan custom (`Task EMB-16 block not found`); la recitation vive en el plan file (orquestador) + este Save Point.
- Commit: `perf: EMB-16 prefijos e5 por familia + margen medido` — SOLO `src/llm.rs` + `src/physical_plan/vector.rs` + `docs/tasks/EMB-16.md`. SIN push (vía vanta-lead).
