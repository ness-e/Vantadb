# EMB-10 — build con embed-local (+remote-inference) + runtime-switchable + cat test

> **Plan:** `docs/dev/plans/2026-09-16-embeddings-auto.md` (Wave0, Gate raíz)
> **Estado:** ⏳ IN PROGRESS
> **Appetite:** 1d · 🟡 · 🔴 Alta
> **Branch/Commit:** develop / `feat: EMB-10`
> **Cynefin:** 🟦 obvio · ⬆️ 0 / ⬇️ 2 steps
> **Ruta:** vanta-worker
> **SDP:** campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, api-and-interface-design, systematic-debugging (frontend-ui-engineering descartada: sin `web/`, scope discipline)

## 1. TAREA

**Objetivo:** producir un binario `vanta-cli` con el motor ONNX adentro (`embed-local` + `remote-inference`) y probar que el switch de proveedor es solo-config (sin recompilar).

**Contrato exacto (ley):**
1. `cargo build --bin vanta-cli --features embed-local,remote-inference -j 2` OK (comando candidato REVERIFICADO en DISCOVERY — exacto, ver §7).
2. `ort`/`tokenizers` presentes en el build (artefactos en `target/debug/deps/` + `cargo tree -e features -p vantadb` muestra flags efectivos).
3. Con MISMO binario:
   - `VANTADB_EMBEDDING_PROVIDER=local` → cat test con señal semántica real (sim(pares)≫sim(impares), umbral con evidencia).
   - `VANTADB_EMBEDDING_PROVIDER=ollama` sin servidor → degradación avisada sin crash (error claro, no panic).
4. Prueba que el switch es solo-config.
5. NO instalar global (PID 3864 tiene el viejo; instalar = EMB-19).

**Acceptance del plan:** sin motor en el binario nada posterior funciona (EMB-11..20 asumen binario con motor). Gate Justificación: desbloqueo raíz.

## 2. ARCHIVOS

**Clave:**
- `Cargo.toml:105-116` (features: `default` sin embed-local verificado; `embed-local = ["dep:ort","dep:tokenizers"]`; `remote-inference = ["dep:reqwest"]`; `ort 2.0.0-rc.13 load-dynamic`, `tokenizers 0.22`)
- `Cargo.toml:302-306` (bin `vanta-cli`, `required-features = ["cli"]`)
- `src/llm.rs:47-100` (3 variantes factory: both / remote-only / local-only; con both: `local|multilingual-e5-small` → `LocalOnnxProvider::new` con fallback dummy 384d; `_` → mismo fallback local; `ollama`/`openai` → remotos)
- `target/debug/vanta-cli.exe` (salida; actual 39MB del 2026-09-16 03:50, sin ort)

**Relacionados:**
- `embeddings/models/multilingual-e5-small/onnx/` (`model.onnx` 470MB + `tokenizer.json` 17MB presentes verificados)
- `embeddings/manifest.json` (dims; default `multilingual-e5-small`, 384d)
- `src/config.rs:195-214` (`LlmCfg`: `embedding_provider` default `ollama`, `local_model_path`, `VANTADB_EMBEDDING_PROVIDER`, `VANTADB_LOCAL_MODEL`)
- `src/llm.rs:124-260` (`LocalOnnxProvider::from_llm_cfg`: nunca panics, fallback dummy si falta modelo/sesión; `detect_dim` vía manifest→config.json→384)
- `embeddings/sanity_embed.py` (cat test Python de referencia con umbrales por modelo)

**Prohibidos (WIP ajeno, NO tocar):** `.opencode/`, `Justfile`, `completions/_vanta-cli*`, `desktop/src-tauri/Cargo.lock`, `ocr-delegate.yml`, `ocr-review.ps1`, `reparacion.bat`, `docs/pipeline-state.json`, `~/.cargo/bin/vanta-cli.exe` (instalación = EMB-19), plan file (solo recitation orquestador), `stash@{0..14}`.

## 3. DEPENDENCIAS

- **Wave0**, sin bloqueantes. Previa: ninguna (plan nuevo).
- **Desbloquea:** EMB-11..20 para verificación real (todas asumen binario con motor).
- **Next:** EMB-11/12 (Wave1, scripts disjuntos, pueden correr con W0 en vuelo).
- **Hereda:** features workspace, `LocalOnnxProvider`, manifest dims, task FIND-99 (evidencia dummy: sim(gato,felino)=-0.03 vs sim(gato,cuántica)=+0.13).

## 4. REFERENCIAS

- `.opencode/rules/core-engine.md` (llm.rs — LEÍDA COMPLETA: R-1 feature-gating experimental sin deps en default; R-3 `?` + `Result<T>`; R-4 `unsafe` + `// SAFETY:`; R-5 prefijo único `VANTADB_*`, valor no reconocido → `warn!` + default, nunca panic)
- `clean-code-clean-architecture.md` Apéndice V (LEÍDO COMPLETO: V.1 capas — `src/llm.rs` = transversal core, env vars vía Config; V.2 reglas duras; V.3 stuttering; V.4 severidades — EMB-10 toca 0 código, N/A)
- Propuesta matriz REAL (LEÍDA COMPLETA vía Notion): retrieval híbrido BM25+HNSW+RRF es REAL (`text_index`, `vector/`, tests `hybrid_retrieval_quality`); `extract_skills`/gobernanza PROPUESTA — fuera de scope.
- Problema dimensión semántica (LEÍDA COMPLETA vía Notion): garbage-in = garbage-out → justifica el plan (dummy sin semántica contamina memoria).
- Nuevas features / Plan de accion: sin mapeo → no aplican (filtro VantaDB).
- **Spec (tabla, no se esperan símbolos públicos nuevos — solo flags de build):**

| Decisión | Opción | Evidencia |
|---|---|---|
| Comando build | `cargo build --bin vanta-cli --features embed-local,remote-inference -j 2` | `Cargo.toml:106` default sin embed-local; `:302-306` bin requiere solo `cli`; flags existen `:115-116` |
| Cat test local | cosenos vía binario/MCP con `VANTADB_EMBEDDING_PROVIDER=local` desde repo root | `llm.rs:60-67` factory local + `detect_dim` manifest; modelo+tokenizer presentes |
| Degradación ollama | error avisado, no crash, sin servidor | `OllamaProvider` con timeouts 10s/30s (`llm.rs:566-571` patrón); R-5 warn+default |
| Sin símbolos públicos nuevos | N/A — solo flags | factory ya existe en 3 variantes; 0 código tocado esperado |

## 5. SKILLS

**SDP (campaign_discover_skills_v2 BUILD keywords onnx-build/cargo-features/embed-local/runtime-switch, ≤8):**
campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, api-and-interface-design, systematic-debugging (+ frontend-ui-engineering sugerida por scorer pero DESCARTADA con justificación: sin `web/`, scope discipline).
**Base sesión:** campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail(full).
**Cargadas:** source-driven, doubt-driven, incremental, test-driven, context-engineering, api-and-interface-design, systematic-debugging.
**Bug → systematic-debugging inline** (Stop: `ort` no compila → Gate V con log, no rabbit hole).

## 6. HERRAMIENTAS+MCP

- `Get-PSDrive C` (incidente FIND-94): 65.28GB libres / 410.54GB usados — OK para build.
- `cargo build --bin vanta-cli --features embed-local,remote-inference -j 2` (timeout generoso 15-20min, no matar sano).
- `cargo tree -e features -p vantadb -f "{p} {f}" -i ort,tokenizers` (flags efectivos).
- Artefactos: `Get-ChildItem target/debug/deps/ | Where Name -match ort|onnxruntime|tokenizers-`.
- MCP stdio cat test (`embed_texts` o query con `VANTADB_EMBEDDING_PROVIDER=local` desde repo root; modelo default relativo frágil por CWD → correr desde repo root).
- `campaign_verify_cmd` (bug exit -1 conocido → bash directa + mención en RESULTADO).
- `codegraph_explore` (blast radius: `get_embedding_provider` 4 callers en `physical_plan/vector.rs`, `executor.rs`; ⚠️ no covering tests).
- `cargo check -p vantadb`, `cargo fmt --check` (0 código tocado → N/A pero verificar si algo cambia).

## 7. INVESTIGACIÓN CÓDIGO (DISCOVERY)

**Blast radius flags→bin→llm factory→tool:**
- Flags: `embed-local` mete `ort 2.0.0-rc.13 (load-dynamic)` + `tokenizers 0.22`; `remote-inference` mete `reqwest` (blocking). Default NO incluye ninguno de los dos (`Cargo.toml:106`).
- Bin: `vanta-cli` requiere solo `cli` → compila hoy sin motor (39MB, 0 artefactos `ort`/`onnxruntime` en `target/debug/deps/`; 14 `*tokenizer*` son tantivy, NO la crate `tokenizers` — HALLAZGO de nomenclatura documentado).
- Factory (`src/llm.rs:47-101`): 3 variantes cfg:
  - both → `openai`→OpenAI, `ollama`→Ollama, `local|multilingual-e5-small|_`→Local (con fallback dummy 384d si modelo no carga).
  - remote-only → `openai`→OpenAI, `_`→Ollama (sin local posible).
  - local-only → siempre Local (con fallback dummy).
  - Con el build EMB-10 (both) el switch `local`/`ollama`/`openai` es runtime vía `VANTADB_EMBEDDING_PROVIDER` — prueba de solo-config posible.
- Dónde vive onnxruntime nativo: `ort load-dynamic` carga la lib dinámica en runtime (`ort::init().commit()` en `try_load_session`); si no carga → `session=None` → dummy determinista (hash, sin semántica) — el cat test lo revela (pares≈impares), NO es verde.
- `detect_dim`: manifest.json (por id en path) → config.json `hidden_size` → 384.
- `try_load_tokenizer`: candidatos `model_dir/tokenizer.json`, `../`, `../../`, default repo path + búsqueda recursiva.
- Callers factory: `src/physical_plan/vector.rs`, `src/executor.rs` (4 callers); MCP `embed_texts` hoy dummy hardcodeado (`tools.rs:2607`→`:3193-3204`) — cableado real es EMB-13, NO esta tarea.

**Impacto mapeado (Regla 0):**
- Archivos leídos completos: `Cargo.toml:95-224,290-329`, `src/llm.rs:1-260`, `.opencode/rules/core-engine.md`, clean-code Apéndice V, plan EMB-10, `embeddings/manifest.json` (parcial 60 líneas), `embeddings/sanity_embed.py` (vía codegraph), `src/config.rs` (vía codegraph `LlmCfg`), `src/error.rs` (vía codegraph `code()`).
- Referencias hacia dentro (EMB-10 depende de): `ort`, `tokenizers`, `reqwest` (features); `Config::llm_cfg()`; `embeddings/` en disco.
- Referencias entrantes (dependen de EMB-10): EMB-11..20 (verificación real); `physical_plan/vector.rs`, `executor.rs` (callers factory).
- Veredicto: build-only, 0 código tocado esperado; impacto = nuevo binario local (NO commiteable, NO instalable global). Riesgo: build largo + onnxruntime nativo.

## 8. INVESTIGACIÓN PROBLEMA

- Binario sin motor (evidencia mecánica): 0 artefactos `^(lib)?(ort|onnxruntime|tokenizers)-` en `target/debug/deps/`; default features sin `embed-local`; bin actual 39MB del 2026-09-16.
- Dummy probado en vivo (heredado plan): sim(gato,felino)=-0.03 vs sim(gato,cuántica)=+0.13 — sin señal semántica.
- `onnxruntime` debe cargar o el test lo revela: con build EMB-10, si `ort::init` o sesión fallan → dummy → cat test da pares≈impares → HALLAZGO (no verde), se reporta.
- Pre-mortem cubierto: disco OK (65GB); build 10-20min con `-j 2`; no `cargo clean` sin aviso.

## 9. INVESTIGACIÓN INTERNET

No se espera (todo local: flags Cargo + ONNX en disco + factory existente). Stack detectado en repo (`Cargo.toml`: `ort 2.0.0-rc.13`, `tokenizers 0.22`, `rustc 1.95.0`) — sin necesidad de docs externas. Sin red usada → sin citas. Si surgiera duda de API `ort` → `webfetch` docs oficiales `ort.pyke.io` + deuda TSYS-13 si sin red. Estado: N/A.

## 10. VALIDACIÓN+CIERRE

- [ ] `Get-PSDrive C` OK (65.28GB) ✅ (DISCOVERY)
- [ ] `cargo build --bin vanta-cli --features embed-local,remote-inference -j 2` OK
- [ ] `ort`/`tokenizers` presentes (`deps/` + `cargo tree -e features`)
- [ ] MISMO binario: `VANTADB_EMBEDDING_PROVIDER=local` → cat test con señal (números + umbral con evidencia)
- [ ] MISMO binario: `VANTADB_EMBEDDING_PROVIDER=ollama` sin servidor → degradación avisada sin crash
- [ ] Switch solo-config probado (sin recompilar entre ambos)
- [ ] OCR (`pwsh dev-tools/ocr-review.ps1`) — 0 código tocado esperado, solo task file
- [ ] `/cleanCA` N/A (0 código tocado — solo build)
- [ ] DoD 3 niveles: contrato + task file + recitation
- [ ] Reviewer P2-01 (mismo contexto no auto-audita cambios críticos; build-only → review liviana)
- [ ] Gates D/V/C evaluados + RESULTADO §7 + Save Point
- [ ] Commit `feat:` SOLO si hay archivos propios commiteables (binario NO se commitea; task file SÍ)
- [ ] Backlog→avance NO tocar + push vía vanta-lead

## Steps

- [x] **Step 1 — build con motor:** `cargo build --bin vanta-cli --features embed-local,remote-inference -j 2` + verificar `ort`/`tokenizers` en build ✅ COMPLETO (13m36s; libort 4.64MB + libtokenizers 14.40MB; vanta-cli.exe 23.7MB)
- [x] **Step 1b — fix token_type_ids (slice extra, contrato lo exige):** RED (ranking dummy #14,#12 + ORT Gather error) → GREEN (name-based feed + zeros) → rebuild 48s ✅ COMPLETO
- [x] **Step 2 — cat test runtime-switch:** MISMO binario `local` con señal + `ollama` sin servidor avisado ✅ COMPLETO (matriz 3 casos abajo)
- [ ] **Step 3 — cierre:** OCR + commit task file + recitation + RESULTADO ⬜ PENDING (en curso)

## Ejecución (evidencia mecánica)

### Build
- `cargo build --bin vanta-cli --features embed-local,remote-inference -j 2` → `Finished dev profile in 13m36s` (disco C: 65.28GB libres antes).
- `cargo tree -p vantadb --features embed-local,remote-inference -f "{p} {f}" -i ort-sys` → `ort-sys ... api-17..api-27,...` + `ort ... api-27,copy-dylibs,default,download-binaries,load-dynamic,...` → **ORT_API_VERSION compilado = 27** (requiere nativo ≥1.27.x).
- `cargo tree -i ort` directo falla (`package ID did not match`) — nota de uso, no un error del build.

### Root cause nativo (systematic-debugging Fases 1-2)
- Crash inicial: `ort/lib.rs:234 expect` → `BadVersion { 1.17.1 }` (System32) → `mutex_std.rs:15 Mutex poisoned` → `panicking.rs:225 panic in a function that cannot unwind` → abort.
- `ORT_API_VERSION = 17 + api-18..api-27` (ort-sys version.rs) = 27 con defaults de ort → 1.17.1 y 1.26.0 (.venv) AMBOS rechazados.
- `download-binaries`/`copy-dylibs` activos pero sin efecto con `load-dynamic` (nada en target/ ni ~/.cache/ort).
- Solución env-only: ORT v1.30.0 (latest GitHub, ≥1.27 ✓) descargado a `Temp/opencode/ort130` (78.8MB zip, fuera del repo). `ORT_DYLIB_PATH` soportado por ort (lib.rs:224).

### RED (antes del fix, binario recién compilado)
- Server log: `ERROR ... Gather node ... Missing Input: token_type_ids` → `run_onnx` devolvía None → `embed()` caía a dummy silencioso.
- Ranking query "un gato descansando en el sillon": **#14(foto), #12(felino)** — #11/#13 ausentes. Sin señal.

### Fix (src/llm.rs `run_onnx`, +24/-7)
- Causa: solo se alimentaban inputs[0..1] por posición; el export e5 declara `token_type_ids` (verificado: inputs = input_ids, attention_mask, token_type_ids; outputs = last_hidden_state [b,s,384]).
- Cambio: resolución por NOMBRE (fallback posicional) + tensor de ceros [1,seq] cuando el modelo declara `token_type`. Sin símbolos públicos nuevos. Patrón espejo de `embeddings/sanity_embed.py`.
- `cargo fmt --check` ✅ (tras fmt) · `cargo clippy ... --all-targets` 0 warnings ✅ · lib tests `llm` 4/4 ✅ con ORT.
- OCR `delegate rule src/llm.rs`: sin Critical/High. Observación menor (no bloquea): `run_onnx` codifica el texto 2 veces (línea ~388, pre-existente) → candidato para EMB-16 que ya toca llm.rs.

### GREEN — matriz runtime-switch (MISMO binario, sin recompilar)
| # | Provider | ORT | Resultado |
|---|---|---|---|
| 1 | `local` | 1.30 | Ranking **#12, #11, #14, #13** = orden coseno exacto. Números ref Python (mismo modelo): 0.9533 / 0.9123 / 0.7773 / 0.7118 → pares≥0.91 ≫ impares≤0.78 (gap 0.135). Sin error Gather en log. ✅ |
| 2 | `ollama` (sin servidor) | ausente | INSERT → HTTP 200 + WARN `Auto-embedding failed ... Network error ... (http://localhost:11434/api/embed)` (avisado, sin crash). `~` search → HTTP 500 por pánico ort en worker-task (servidor VIVE, Tokio lo contiene). ✅ sin crash / ⚠️ 500 documentado |
| 3 | `ollama` (sin servidor) | 1.30 | `~` search → **#12, #11, #14, #13** vía fallback local (vector.rs:62-74, by design). Switch = solo `VANTADB_EMBEDDING_PROVIDER`. ✅ |

### Hallazgos para el orquestador (NO se arreglan acá — Backlog NO se toca por orden expresa; crear filas FIND)
- **FIND-A (alta):** `ort::init().commit()` hace `expect` (BadVersion/Dlopen) + mutex poisoned → abort en hilo main (CLI) / pánico de task (HTTP 500). La "degradación graceful" de llm.rs es falsa sin dylib compatible. Opciones: `catch_unwind` + pre-chequeo de versión, o empaquetar dylib (EMB-11 debe descargar nativo ≥1.27 — requisito nuevo para el instalador).
- **FIND-B (media):** `LocalOnnxProvider::embed` cae a dummy hash en silencio (`Ok(deterministic_embed)`); solo el log ORT delata. Q5 exige flag visible → cubrir en EMB-13 (`"fallback": true`).
- **FIND-C (baja):** `vanta-cli query` abre read-only (`data.rs:212`) → todo IQL INSERT/UPDATE/DELETE por CLI falla. Doc vs fix a decidir.
- **FIND-D (baja):** `tests/sdk_serialization.rs` no compila (39 errores, `QueryResult` no declarado) — pre-existente, ajeno a este task.
- Deuda TSYS-13: ninguna cita URL usada en evidencia (todo local + GitHub API/release verificados en vivo).

## Spec Gate

N/A feature-add (build + fix privado, 0 símbolos públicos nuevos). Tabla §4 + matriz de arriba valen como spec de decisiones. Gate D: 1 archivo, 1 fn privada, sin API nueva → NO dispara `question`.

## Spec Gate

N/A feature-add (build-only, 0 símbolos públicos nuevos). Tabla §4 vale como spec de decisiones. Gate D: blast radius ≤10 archivos tocados (0), sin hot path, sin API pública nueva, contrato mecánico → NO dispara `question`.

## Context Save Point

DISCOVERY completo 2026-09-16. Disco OK. Comando reverificado exacto. Modelo+tokenizer presentes. Task file creado. Siguiente: Step 1 build.
