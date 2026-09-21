# MEM-10: F4 L1 extractor (split + 1 call LLM JSON + parse reparación)

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md` (Task 13, líneas 217-222)
- **Fuente:** plan file Task 13 (MEM-10)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴
- **Tipo:** Rust
- **Turns estimados:** 15-30
- **Creado:** 2026-08-20T21:00
- **last-synced:** 2026-08-20T23:00
- **Estado:** ✅ COMPLETED (review P2-01 approbado y hallazgos aplicados)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps pendientes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-memory` (MEM-11 l1-dedup consumirá `extract_l1_memories` + `parse_l1_extraction`; MEM-17 skill extract; orquestación MEM-16) |
| Callees | `core::abstractions::{LlmRunner, LlmRunParams, SceneSegment, ExtractedMemory, MemoryType, L1ExtractionResult}`, `core::conversation::L0Message` (input), `offload::local_llm::parsers::{json_utils, l1_parser}` (internos) |
| Implicaciones | `core/mod.rs` gana `record` + `prompts`; `offload/mod.rs` gana `local_llm`; ningún contrato existente cambia; L1 es LLM-optional (Principio 4 — degrada a `success:false`, nunca pierde datos); `records` queda vacío en este task (el write es MEM-11); tests existentes no se ven afectados |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-memory/src/core/conversation/l0_recorder.rs` (318: L0Message/L0Recorder::read_messages), `vanta-memory/src/core/abstractions/llm_runner.rs` (248: LlmRunner/LlmRunParams/LlmError), `vanta-memory/src/core/abstractions/types.rs` (370: SceneSegment/ExtractedMemory/MemoryType/L1ExtractionResult), `vanta-memory/src/core/mod.rs` (18), `vanta-memory/src/offload/mod.rs` (6), `vanta-memory/src/offload/types.rs`, `vanta-memory/src/adapters/mod.rs` (19: mock feature-gated), `vanta-memory/src/adapters/mock.rs` (96: MockLlmRunner), `vanta-memory/src/lib.rs` (45), `vanta-memory/Cargo.toml` (33), `vanta-memory/tests/l0_capture.rs` (168: patrón de tests D19), TDAM `MC/core/record/l1-extractor.ts` (738), `MC/core/prompts/l1-extraction.ts` (417), `MC/offload/local-llm/parsers/l1-parser.ts` (41), `MC/offload/local-llm/parsers/json-utils.ts` (85), `MC/offload/local-llm/prompts/l1-prompt.ts` (98), `MC/utils/sanitize.ts` L100-169 (shouldExtractL1), `docs/plans/2026-08-18-vanta-memory.md` (L217-222)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `vanta-memory` → `vantadb` (solo tipos de conversación L0); nuevos módulos `core::record`, `core::prompts`, `offload::local_llm::parsers`; dependencias existentes serde/serde_json/thiserror/tracing (sin deps nuevas)
- **Archivos que referencian a los editados (referencias entrantes):** ninguno — `record/`, `prompts/`, `offload/local_llm/` son módulos nuevos; solo se editan `core/mod.rs` y `offload/mod.rs` (agregan `pub mod`)
- **Veredicto impacto:** bajo — solo se agregan módulos nuevos y 2 líneas en 2 archivos `mod.rs`; cero archivos existentes modificados

## Contrato
"`cargo check -p vanta-memory` pasa, `cargo nextest run -p vanta-memory` pasa (incluye tests dedicados de L1), `cargo fmt --check` pasa, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` pasa, y el comportamiento específico es: (1) `extract_l1_memories` con runner LLM exitoso devuelve `success:true` con `extracted_count`, `scene_names`, `last_scene_name` y `records` vacío (el write es MEM-11); (2) el runner que falla degrada a `success:false` sin panic ni pérdida de datos (Principio 4); (3) el split por marcadores separa background (contexto) de new (extraíbles) respetando `max_new_messages`/`max_background_messages`; (4) el parse con reparación tolera code fences, trailing commas y prosa alrededor del JSON, y normaliza tipos legacy (`episode`→episodic, `instruct`→instruction, `preference`→persona) descartando tipos inválidos"

## Invariantes de dominio (handoff - MUST)

- **Invariantes a preservar:** (1) LLM opcional — si `LlmRunner::run` falla, L1 devuelve `L1ExtractionResult { success: false, .. }` sin propagar error ni perder los mensajes L0 (Principio 4); (2) prompts **reescritos en inglés** (Principio 7 — NO traducir el chino de Kenty; riesgo de prompts contaminados); (3) una sola call LLM (task_id `l1-extraction`), JSON array de scene segments; (4) `source_message_ids` solo de los mensajes NEW (nunca background); (5) sin `unwrap()`/`expect()` en código nuevo; (6) tipos normalizados contra `MemoryType` con aliases legacy, tipo inválido se descarta (no rompe el batch); (7) sin deps nuevas (la conversión epoch→RFC3339 es un helper local, sin chrono)
- **Comandos de verificación:** `cargo check -p vanta-memory`, `cargo nextest run -p vanta-memory`, `cargo fmt --check`, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` — todos exit 0
- **Deuda pendiente:** `records`/`stored_count` quedan vacíos por diseño (write + dedup = MEM-11); `should_extract_l1` vive en `l1_extractor.rs` (MEM-19 consolidará sanitize utils si aplica)

## Recitation (canónico - estructura única)

- **activeGoal:** MEM-10: F4 L1 extractor (split + 1 call LLM JSON + parse reparación)
- **lastAction:** Gate P2-01 ejecutado (vanta-audit) → APPROVE con 1 [major] + menores; hallazgos aplicados en el changeset: `repair_priority_scalars` (l1_parser, bare `"priority": sheet`), `sanitize_json_for_parse` (json_utils, control chars en strings), `metadata` default `{}`, noise set real de TDAM `isFrameworkNoise` en `should_extract_l1`; +8 tests (73/73); task file actualizado con Review approbado
- **result:** ✅ COMPLETED
- **nextAction:** vanta-lead commitea (feat: MEM-10). Luego Task 14 (MEM-11 — F4 L1 dedup 2 fases): consumir `extract_l1_memories` + `parse_l1_extraction`, implementar write `MemoryRecord` + dedup store/update/merge/skip
- **contract:** ✅ cumplido: `cargo check -p vanta-memory` exit 0; `cargo nextest run -p vanta-memory` 73/73 exit 0 (9 tests l1_extractor + unit tests json_utils/l1_parser/prompts/l1_extractor); `cargo fmt --check` exit 0; `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` exit 0; review P2-01 APPROVE (hallazgos aplicados)
- **nextTask:** Task 14 (MEM-11 — F4 L1 dedup 2 fases)

## Deuda técnica (Regla 6 - MUST)

**Saldo neto de deuda por PR:** Sin deuda

> No se introduce deuda nueva: 4 archivos nuevos + 2 líneas de `pub mod` en módulos existentes. Sin dependencias nuevas (el helper epoch→RFC3339 evita chrono). El parse tolerante es deliberadamente naive (depth scan para brackets) con techo conocido documentado — mismo trade-off que TDAM json-utils.ts.

## Definition of Done (contrato multi-nivel - P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato arriba: cargo check + nextest (tests L1) + fmt + clippy con `-D warnings`, todos exit 0 |
| **Commit** | Commit atómico (~100 líneas/slice), conventional commit (`feat:` MEM-10), verificación mecánica (no auto-reporte) — commit lo hace vanta-lead, no este worker |
| **Release** | No aplica esta iteración (tarea de feature intermedia, sin release) — justificado: `vanta-memory` es `publish = false`, aún en construcción MEM-09..18 |

**Gate:** se marca COMPLETED solo si pasa el nivel Task (los niveles Commit/Release quedan en manos de vanta-lead).

## Herramientas necesarias
- cargo (check, nextest, fmt, clippy)
- codegraph_explore (blast radius ya ejecutado)
- TDAM clone `C:\Users\Eros\AppData\Local\Temp\opencode\tdam` @ `97f9465`

## Investigation Notes
- **Diseño del port (TDAM l1-extractor.ts → Rust, no copia literal):**
  1. `extract_l1_memories<R: LlmRunner>(runner: &R, messages: &[L0Message], previous_scene_name: Option<&str>, config: &L1ExtractorConfig) -> L1ExtractionResult` — función libre (TDAM es función libre; no hay estado propio → sin struct, YAGNI). **CORRECCIÓN al plan:** `LlmRunner` NO es dyn-compatible (método genérico `complete_json<T>`) → firma genérica `<R: LlmRunner>` en vez de `&dyn LlmRunner` (E0038). `L1ExtractionOptions` eliminado — sus campos (session/isolation) solo los usa el write de MEM-11; en MEM-10 solo se necesita `previous_scene_name` → parámetro directo (YAGNI, se reintroduce en MEM-11 con su contrato de persistencia).
  2. Quality gate `should_extract_l1` (port de sanitize.ts:135-156): vacío/whitespace, ruido de framework, slash-commands, símbolos puros 1-5 chars, solo `?`. Vive en l1_extractor.rs (privado) — MEM-19 consolidará sanitize.
  3. Split: `new = last max_new` (default 10), `bg = hasta max_bg (default 5) inmediatamente anteriores a new`. Solo NEW es extraíble (prompt lo declara explícito + source_message_ids). El gate corre ANTES del split (filter → split), como TDAM.
  4. 1 call LLM: `runner.run(LlmRunParams { prompt: user, system_prompt: Some(system), task_id: "l1-extraction", timeout: 180s })`. NO `complete_json` — el parse tolerante (json_utils) repara lo que el strict deserialize no tolera.
  5. Parse con reparación: `json_utils::extract_json` (direct → fences → brace/bracket scan → trailing commas) + `l1_parser::parse_l1_extraction` (walk `Vec<Value>`, scene_name default "unknown-scene", filtro content vacío, normalize_type con aliases, priority default 50, metadata default null). Los tipos inválidos se descartan por memory (no rompen el batch). `fix_trailing_commas` es string-aware (no corrompe `"x, }"` dentro de strings).
  6. Resultado: `success:true`, `extracted_count`, `records: vec![]` + `stored_count: 0` (write = MEM-11), `scene_names`, `last_scene_name`. LLM falla → `success:false` + vacío (Principio 4, TDAM l1-extractor.ts:203-205). Escenas sin memories → scene_names/last_scene_name igual se reportan (TDAM L232-254).
- **Timestamp en prompt:** TDAM usa `new Date(m.timestamp).toISOString()`. Sin chrono en Cargo.toml → helper local `epoch_ms_to_rfc3339` (~30 líneas, algoritmo civil-from-days de Howard Hinnant, test unitario con valor conocido 1700000000000 → 2023-11-14T22:13:20.000Z). Evita dep nueva (ponytail rung 5). Vive en `core/prompts/l1_extraction.rs` (lo usa format_messages).
- **Prompts reescritos (inglés):** el system prompt de TDAM está en chino (Kenty) — reescribir principios en inglés, NO traducir (Principio 7, riesgo: los prompts chinos son específicos del host OpenClaw y contaminarían la salida). Dos modos: Chat (persona/episodic/instruction) y Code/Work (work_fact/task/method/artifact) — `MemoryPromptMode`-equivalente.
- **`offload/local_llm/parsers/l1_parser.rs`:** TDAM l1-parser.ts parsea OffloadEntry (tool_call pairs, offload MEM-20). En MEM-10 el "parse con reparación" del gate es el de `l1-extractor.ts::parseExtractionResult` → `parse_l1_extraction(raw) -> Vec<SceneSegment>`. El parser de offload (tool pairs) se agrega en MEM-20 si aplica.
- **Corrección del noise set en revisión:** `should_extract_l1` portaba marcadores inventados (`<BOOTSTRAP>`, `SESSION_RESET`) — TDAM usa `isFrameworkNoise` (exact/prefix, no substring). Corregido a los 5 casos reales (sanitize.ts:233-255).
- **Techo conocido (resuelto en revisión):** el port inicial omitía `repairExtractionJson` (bare identifier tras `"priority":` — TDAM l1-extractor.ts:592-598). El gate P2-01 (vanta-audit) lo marcó [major] (pérdida silenciosa de todo el batch). Se implementó `repair_priority_scalars` string-aware en l1_parser (no regex, para no corromper `"priority"` dentro de strings) + `sanitize_json_for_parse` (escape de control chars U+0000–U+001F en strings, TDAM sanitize.ts:316-334) en json_utils. Verificado con 8 tests nuevos (73/73).

## Incógnitas (uphill) vs Pendientes (downhill) - P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — API runner confirmada (llm_runner.rs), tipos confirmados (types.rs), firma L0Message confirmada (l0_recorder.rs); dyn-compat resuelto con firma genérica |
| Pendientes de ejecución (downhill) | 0 — todos los steps completados |
| % completado | 100% |

## Fases explícitas - SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — evaluado: el código nuevo procesa output de LLM (trust boundary — el LLM puede emitir JSON malformado o contenido inyectado). Mitigación: el parse nunca ejecuta contenido; los prompts instruyen a ignorar instrucciones embebidas en mensajes (rewrite del principio "no extraer prompt injection"); `should_extract_l1` es el gate de calidad. No agrega dependencias. No toca FFI. Se delega la auditoría final a vanta-audit (Review gate).
- [ ] **PERFORMANCE** — evaluado: L1 no es hot path del motor (1 call LLM por flush, batch ~10 mensajes). El parse es O(n) single-pass sobre el JSON del LLM. Sin profiling requerido; vanta-tuner podrá revisar si MEM-11 lo exige.

## Steps

### Step 1: Discovery + diseño + task file
- **Archivos:** plan file (solo lectura), l0_recorder.rs, llm_runner.rs, types.rs, TDAM 5 referencias
- **Acción:** verificar firmas (LlmRunner::run, SceneSegment, ExtractedMemory, L0Message), decidir API (función libre con `&dyn LlmRunner`), documentar diseño en Investigation Notes, crear este task file
- **Verify:** codegraph/Read exitosos; task file con formato template
- **Estado:** ✅ COMPLETED

### Step 2: Crear `json_utils.rs` (offload/local_llm/parsers)
- **Archivos:** `vanta-memory/src/offload/local_llm/mod.rs` (crear), `vanta-memory/src/offload/local_llm/parsers/mod.rs` (crear), `vanta-memory/src/offload/local_llm/parsers/json_utils.rs` (crear), `vanta-memory/src/offload/mod.rs` (editar)
- **Acción:** `extract_json<T: DeserializeOwned>(raw) -> Option<T>` (direct → fences → brace/bracket → trailing commas), `fix_trailing_commas(s) -> String` (string-aware); helpers privados `try_parse`/`strip_code_fence`/`slice_between`; 7 tests unitarios. `repair_json` separado no se creó — la reparación vive en las estrategias de `extract_json` (mismo resultado, menos API)
- **Verify:** `cargo check -p vanta-memory`
- **Estado:** ✅ COMPLETED

### Step 3: Crear `l1_parser.rs` (offload/local_llm/parsers)
- **Archivos:** `vanta-memory/src/offload/local_llm/parsers/l1_parser.rs` (crear)
- **Acción:** `parse_l1_extraction(raw: &str) -> Vec<SceneSegment>` (walk `Vec<Value>`: scene_name default "unknown-scene", message_ids strings, memories filtradas por content no vacío, `normalize_type` con aliases legacy episode/instruct/preference, priority default 50 (i64 o f64 truncado), metadata default null); tipo inválido → descarta esa memory (no el batch); 6 tests unitarios
- **Verify:** `cargo check -p vanta-memory`
- **Estado:** ✅ COMPLETED

### Step 4: Crear `l1_extraction.rs` (core/prompts)
- **Archivos:** `vanta-memory/src/core/prompts/mod.rs` (crear), `vanta-memory/src/core/prompts/l1_extraction.rs` (crear), `vanta-memory/src/core/mod.rs` (editar)
- **Acción:** `PromptMode { Chat, Code }`, `extract_memories_system_prompt(mode) -> String` (reescrito en inglés: tarea 1 scene segmentation, tarea 2 memory extraction con bandas de priority, tarea 3 formato JSON), `format_extraction_prompt(new, background, previous_scene_name) -> String` (secciones PREVIOUS SCENE / BACKGROUND (context-only) / NEW MESSAGES), `epoch_ms_to_rfc3339` (helper local civil-from-days); 4 tests unitarios
- **Verify:** `cargo check -p vanta-memory`
- **Estado:** ✅ COMPLETED

### Step 5: Crear `l1_extractor.rs` (core/record)
- **Archivos:** `vanta-memory/src/core/record/mod.rs` (crear), `vanta-memory/src/core/record/l1_extractor.rs` (crear)
- **Acción:** `L1ExtractorConfig { max_new_messages: 10, max_background_messages: 5, max_memories_per_session: 10, prompt_mode: Chat }`, `should_extract_l1(content) -> bool` (gate de calidad privado), `extract_l1_memories<R: LlmRunner>(runner, messages, previous_scene_name, config) -> L1ExtractionResult` (gate → split → prompt → 1 call run → parse → flatten + normalize → limit max_memories → resultado; runner falla → success:false). **CORRECCIÓN al plan:** firma genérica (trait no dyn-compatible) y sin `L1ExtractionOptions` (YAGNI — campos de sesión solo los usa MEM-11); 2 tests unitarios del gate
- **Verify:** `cargo check -p vanta-memory`
- **Estado:** ✅ COMPLETED

### Step 6: Tests D19 + verificación final
- **Archivos:** `vanta-memory/tests/l1_extractor.rs`
- **Acción:** 9 tests de integración con `CapturingRunner` local (fake, sin feature mock): (a) end-to-end success; (b) runner falla → success:false sin panic; (c) split background/new con ventanas chicas (config max_new=2/max_bg=2) + window exclusion; (d) quality gate filtra ruido sin llamar al LLM; (e) parse repara fences + trailing commas + prosa; (f) tipos legacy normalizados + inválidos descartados; (g) truncation max_memories; (h) escenas sin memories se reportan. Test inicial de split corregido (con max_new=10 default todo entra en NEW — se parametrizó la config)
- **Verify:** `cargo nextest run -p vanta-memory` (65/65 ✅) + `cargo fmt --check` (exit 0 ✅) + `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` (exit 0 ✅)
- **Estado:** ✅ COMPLETED

## Dependencias
- Task 11 (MEM-08b): ✅ COMPLETED — trait `LlmRunner` + `LlmRunParams` + tipos `SceneSegment`/`ExtractedMemory`/`MemoryType`/`L1ExtractionResult` ya existen
- Task 12 (MEM-09): ✅ COMPLETED — `L0Message` + `L0Recorder::read_messages` (input de L1) ya existen

## Review (GATE - agente distinto, P2-01)

- **Revisor:** vanta-audit (o fallback `doubt-driven-development` en contexto fresco)
- **Enfoque:** ¿el parse con reparación tolera los casos reales de output LLM (fences, trailing commas, prosa, tipos legacy)? ¿la degradación a `success:false` cumple Principio 4 sin perder datos? ¿el split respeta que solo NEW sea extraíble? ¿la firma genérica `<R: LlmRunner>` es correcta (trait no dyn-compatible)?
- **Cómo se probó:** 9 tests D19 con fake runner local (sin feature mock) + unit tests en los módulos nuevos: `cargo nextest run -p vanta-memory` 73/73 exit 0 (evidencia: `Summary [0.398s] 73 tests run: 73 passed`); `cargo check -p vanta-memory` exit 0; `cargo fmt --check` exit 0; `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` exit 0
- **Checklist anti-hábitos tóxicos:** [ver template — se verifica en el gate]
- **Veredicto:** ✅ APPROVE (vanta-audit, sesión `ses_fdec7a1afffer7nZdTOBfs69O3`) — con 1 [major] + 4 menores. **Los hallazgos se aplicaron en el mismo changeset:**
  - [major] bare `"priority": sheet` / control chars → `repair_priority_scalars` (l1_parser) + `sanitize_json_for_parse` (json_utils), 8 tests nuevos. RESUELTO.
  - [minor] `metadata` default `Null` → `{}` (paridad TDAM). RESUELTO.
  - [minor] noise set `should_extract_l1` → `isFrameworkNoise` real de TDAM (exact/prefix). RESUELTO.
  - [minor] `complete_json` no delega al reparador → delegar en MEM-11 (write), cuando exista el punto de entrada unificado.
  - [nit] ids numéricos descartados (más estricto que TDAM, OK) / `DEFAULT_SCENE` "unknown-scene" vs "未知情境" (consistencia interna prevalece). ACEPTADOS.

## Notas
- **`records` vacío es intencional:** el write de `MemoryRecord` + dedup es MEM-11 (l1_dedup/l1_writer). MEM-10 devuelve `extracted_count` y las memories normalizadas viven en los `SceneSegment` del parse (consumible por MEM-11). Documentado en el docstring de `extract_l1_memories`.
- **Desviaciones del plan (documentadas en Investigation Notes):** (1) firma genérica `<R: LlmRunner>` en vez de `&dyn LlmRunner` — E0038, el trait tiene `complete_json<T>`; (2) sin `L1ExtractionOptions` — campos de sesión solo los consume el write (MEM-11); (3) `repair_json` absorbido por las estrategias de `extract_json`; (4) techo conocido inicial (sin repair de bare identifier) **resuelto en revisión** — ver Review: `repair_priority_scalars` + `sanitize_json_for_parse`.
- **Post-implementación:** el gate P2-01 (vanta-audit) aprobó con hallazgos; se aplicaron todos en el mismo changeset (73/73 tests, +8 nuevos). El diff final incluye el noise set real de TDAM en `should_extract_l1`.
- **Sin deps nuevas:** el helper `epoch_ms_to_rfc3339` evita chrono; el parse tolerante usa serde_json existente.
- **Runner en tests:** un `CapturingRunner` local en el test file (captura prompt + sistema, devuelve respuestas scripted) — evita feature-gatear el test file con `mock` y verifica el contenido del prompt (split).
- **`LlmRunner` sync:** L1 usa el trait sync (D1). El async wrapper (`AsyncLlmRunner`) lo adaptará la capa server (MEM-16) si aplica.