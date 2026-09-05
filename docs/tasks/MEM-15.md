# MEM-15: F4 Persona first/incremental + triggers + scene navigation

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md` (Task 18, líneas 252-255)
- **Fuente:** plan file Task 18 (MEM-15)
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Tipo:** Rust (crate `vanta-memory`)
- **Turns estimados:** 15-30
- **Creado:** 2026-08-20
- **last-synced:** 2026-08-20
- **Estado:** ✅ COMPLETED (verify 4/4 gates exit 0)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 — 4/4 steps completados

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-memory` (MEM-16 orquestación invocará `generate_persona` + `evaluate_persona_trigger` con el checkpoint manager; MEM-17 skill extract y MEM-19 recall consumen la persona persistida vía `get_persona`) |
| Callees | `core::scene::scene_index::{list_scenes, get_scene, SceneError}` (MEM-12/14 — índice + contenido de escenas), `core::scene::scene_format::SceneBlock`, `core::abstractions::{LlmRunner, LlmRunParams, SceneIndexEntry, PersonaMode, PersonaTriggerPriority}`, `core::prompts::l1_extraction::{PromptMode, epoch_ms_to_rfc3339}`, `core::conversation::{now_ms, sanitize_component}` (pub(crate)), `core::prompts::persona_generation::build_persona_prompt` — todo consumo, cero duplicación |
| Implicaciones | `core/persona/mod.rs` NUEVO (directorio módulo); `core/mod.rs` gana `pub mod persona;` (aditivo); `core/scene/mod.rs` gana `pub mod scene_navigation;` + re-export (aditivo); `core/prompts/mod.rs` gana `pub mod persona_generation;` (aditivo); NO se toca el core `vantadb` ni sus 7 warnings unsafe |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `pipeline-full.md` (247), plan file Task 18 + Principios 1-7 + D19 (líneas 39-86), task files MEM-14.md (111: template + API exacta extractor/soft-delete) / MEM-13.md (referencia template), `vanta-memory/src/core/scene/scene_index.rs` (340: list_scenes/get_scene/patrón namespace+sanitize+read_blocks paginado), `vanta-memory/src/core/scene/scene_format.rs` (150: SceneBlock/index_entry), `vanta-memory/src/core/scene/scene_extractor.rs` (572: patrón genérico `<R: LlmRunner>` + degrade Principio 4 + emptyExtraction), `vanta-memory/src/core/scene/mod.rs` (35), `vanta-memory/src/core/prompts/mod.rs` (24), `vanta-memory/src/core/prompts/scene_extraction.rs` (172: patrón prompt EN inglés + contrato JSON), `vanta-memory/src/core/abstractions/types.rs` (PersonaMode L247, PersonaTriggerPriority L260 con orden P1<P2Cold<P2Rec<P3<P4 ya testeado), `vanta-memory/src/core/abstractions/llm_runner.rs` (248: trait no dyn-compatible, complete_json), `vanta-memory/src/core/conversation/l0_recorder.rs` (sanitize_component/sanitize_key/now_ms), `vanta-memory/src/core/mod.rs` (28), `vanta-memory/Cargo.toml` (33: sin deps nuevas), tests pattern `tests/scene_strategy.rs` (fake runners inline, sin features), TDAM `MC/core/persona/persona-generator.ts` (304: flujo generateLocalPersona — existing strip nav → changed scenes vs checkpoint → mode first/incremental → prompt → escapeXmlTags(strip nav) → límites → append nav → write), TDAM `MC/core/persona/persona-trigger.ts` (136: prioridades P1 request > P2 cold start > P2.5 recovery > P3 first scene > P4 threshold interval), TDAM `MC/core/prompts/persona-generation.ts` (329: system chat persona architect 4 capas ≤2000 chars + work doctrine ≤1200 chars + user prompt stats/changed scenes/existing persona), TDAM `MC/core/scene/scene-navigation.ts` (76: NAV_HEADER, heatEmoji umbrales 50/100/200/500/1000, generateSceneNavigation sort heat desc, stripSceneNavigation), TDAM `MC/utils/sanitize.ts:288-294` (escapeXmlTags: escapa SOLO tags de boundaries de inyección `<\/?(user-persona|relevant-memories|scene-navigation|relevant-scenes|memory-tools-guide|system|assistant)>` case-insensitive)
- **Archivos referenciados hacia dentro:** `core/persona/persona_generator.rs` (nuevo) → scene_index (list_scenes/get_scene), abstractions (LlmRunner/LlmRunParams/PersonaMode/SceneIndexEntry), conversation (now_ms/sanitize_component), prompts l1_extraction (epoch_ms_to_rfc3339/PromptMode), prompts persona_generation (build_persona_prompt/límites), scene_navigation (strip/generate); `core/persona/persona_trigger.rs` (nuevo) → abstractions (PersonaTriggerPriority), std solo; `core/prompts/persona_generation.rs` (nuevo) → abstractions (PersonaMode), prompts l1_extraction (PromptMode); `core/scene/scene_navigation.rs` (nuevo) → abstractions (SceneIndexEntry), std solo
- **Archivos que referencian a los creados (referencias entrantes):** ninguno hoy — MEM-16..19 los consumirán (documentado en Blast Radius); wiring aditivo en 3 mod.rs existentes no rompe nada (solo `pub mod` + re-exports nuevos)
- **Veredicto impacto:** bajo — 4 archivos nuevos + 1 mod.rs nuevo de directorio + 3 wirings aditivos; cero archivos del core `vantadb`; API pública existente intacta; sin migración de datos (namespace nuevo `persona/<session>`)

## Contrato

"`cargo check -p vanta-memory` pasa, `cargo nextest run -p vanta-memory` pasa (incluye tests dedicados de persona D19), `cargo fmt --check` pasa, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` pasa, y el comportamiento específico es: (1) `persona_generator.rs` — generación en 2 modos (`PersonaMode::First` cuando no hay persona previa, `Incremental` cuando existe): lee persona existente (strip navigation), detecta escenas cambiadas (`entry.updated > record.generated_at`, comparación lexicográfica RFC3339 fixed-width; sin persona previa → todas las escenas son cambiadas), skip si no hay cambios y persona existe, firma genérica `<R: LlmRunner>` (trait no dyn-compatible) como MEM-14, post-proceso del output LLM = strip nav + trim + escape_xml_tags + límite de tamaño (chat ≤2000 chars, work ≤1200 chars — output que excede se RECHAZA sin escribir, la persona vieja se preserva), append de navegación fresca, persistencia en namespace `persona/<session>` (Principio 2) como `PersonaRecord {content, mode, generated_at_ms, generated_at}`; runner LLM fallido o output vacío → degrada `success:false` SIN escribir (Principio 4); (2) `persona_trigger.rs` — LLM-free heurístico, `evaluate_persona_trigger(input, trigger_every_n)` pura con prioridades TDAM P1 request explícito > P2ColdStart (scenes>0, sin persona, hay bloques) > P2Recovery (generada antes, body vacío) > P3FirstScene (1 escena + memories>0) > P4MemoryCount (memories >= n), devuelve `TriggerResult {should, priority, reason}` usando el enum `PersonaTriggerPriority` existente; (3) `prompts/persona_generation.rs` — prompts REESCRITOS EN INGLÉS (Principio 7, NO traducir chino): system chat (persona architect 4 capas: base facts/interest graph/interaction protocol/cognitive core; restricciones ≤2000 chars, no over-speculation, solo datos de escenas provistas, no navigation) + work (team operating doctrine ≤1200 chars) + contrato JSON de salida `{"persona": "<documento markdown completo>"}` (divergencia documentada: TDAM usa tool-calls write/edit sobre persona.md; este port emite JSON que la capa determinista persiste — el loop de tools es MEM-16) + user prompt (tiempo, modo, stats, changed scenes content, existing persona, iteration guide); (4) `scene_navigation.rs` — port de scene-navigation.ts: `NAV_HEADER` paridad TDAM, `heat_emoji` umbrales 50/100/200/500/1000, `generate_scene_navigation` (sort heat desc, bloque por escena con nombre/heat/updated/summary — record store: el nombre ES la key, sin paths de filesystem), `strip_scene_navigation` (corta en NAV_HEADER); (5) `escape_xml_tags` — port fiel de sanitize.ts:288: escapa solo los tags de boundaries de inyección (user-persona/relevant-memories/scene-navigation/relevant-scenes/memory-tools-guide/system/assistant, case-insensitive); (6) sin unwrap/expect en producción, sin deps nuevas, errores en inglés, `#[non_exhaustive]` en enums públicos"

## Invariantes de dominio (handoff - MUST)

- **Invariantes a preservar:** (1) LLM opcional (Principio 4) — runner falla u output inválido/vacío/oversized → `success:false` SIN escribir al store (la persona previa nunca se pierde ni corrompe); (2) persistencia SOLO vía store VantaDB (Principio 2) — namespace `persona/<session>` sanitizado con `sanitize_component(s, 128, false)`, key `persona.md` sanitizada con `sanitize_key`; (3) modo determinista: First ⇔ no existe registro; Incremental ⇔ existe; skip cuando persona existe Y 0 escenas cambiadas; (4) detección de cambios por comparación lexicográfica RFC3339 fixed-width (`epoch_ms_to_rfc3339`) — string order == chronological order (invariante ya documentado en scene_index); (5) límites mecánicos: chat ≤2000 chars, work ≤1200 chars sobre el BODY (post strip-nav, pre nav-append) — exceder es rechazo, no truncamiento silencioso; (6) escapeXmlTags SIEMPRE antes de persistir (inyección XML en `<user-persona>` etc.); (7) triggers puros (LLM-free, Principio 4) — orden P1<P2ColdStart<P2Recovery<P3FirstScene<P4MemoryCount ya fijado por el enum de MEM-08b; (8) sin `unwrap()`/`expect()` en producción; (9) sin deps nuevas; (10) errores en inglés (Principio 7); (11) output del LLM tratado como input no confiable (LLM05 — revalidar en boundary)
- **Comandos de verificación:** `cargo check -p vanta-memory`, `cargo nextest run -p vanta-memory`, `cargo fmt --check`, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` — todos exit 0
- **Deuda pendiente:** checkpoint manager (last_persona_at/memories_since_last_persona/scenes_processed) es MEM-16 orquestación — `evaluate_persona_trigger` recibe esos contadores como input puro; el loop agente con tools (write/edit real sobre persona) es MEM-16 — este port usa contrato JSON `{"persona": ...}` ejecutado por capa determinista; sync `docs/api/` del módulo → MEM-38 (docs gate pre-release), anotado

## Recitation (canónico - estructura única)

- **activeGoal:** MEM-15: F4 Persona first/incremental + triggers + scene navigation
- **lastAction:** Implementada la capa L3 completa: `scene_navigation.rs` (NAV_HEADER paridad TDAM, heat_emoji umbrales 50/100/200/500/1000, generate_scene_navigation sort heat desc, strip_scene_navigation), `prompts/persona_generation.rs` (system chat persona architect 4 capas + work doctrine REESCRITOS EN INGLÉS, contrato JSON `{"persona": ...}`, límites MAX_PERSONA_CHARS_CHAT=2000/WORK=1200, user prompt stats/changed scenes/existing persona/iteration guide), `persona_trigger.rs` (evaluate_persona_trigger pura LLM-free, prioridades P1 request > P2ColdStart > P2Recovery > P3FirstScene > P4MemoryCount sobre enum MEM-08b), `persona_generator.rs` (namespace `persona/<session>`, PersonaRecord {content+nav, mode, generated_at_ms/at}, get_persona/has_persona_body/escape_xml_tags port exacto de sanitize.ts:288 (7 boundaries de inyección case-insensitive), generate_persona<R: LlmRunner>: detección de cambios por `entry.updated > generated_at` RFC3339 lexicográfico, skip sin cambios sin llamar al LLM (probado con runner que panica), modo First/Incremental derivado del store, post-proceso strip-nav→trim→escape→límite con RECHAZO (no truncamiento) preservando la persona previa, append nav fresca, persistencia; degrade Principio 4: fallo LLM/output vacío/oversized → success:false SIN escribir); wiring core/mod.rs + scene/mod.rs + prompts/mod.rs; tests D19 tests/persona.rs (14 integration) + unit tests en los 4 módulos
- **result:** ✅ 4/4 gates exit 0: cargo check ✅, nextest 219/219 (177 previos + 42 nuevos) ✅, fmt --check ✅, clippy -D warnings ✅ (7 warnings unsafe pre-existentes del core vantadb, fuera de scope)
- **nextAction:** Ninguna para MEM-15. Siguiente tarea del plan: Task 19 (MEM-16 — F4 Orquestación timers+locks); el lead commitea (`feat:` MEM-15)
- **contract:** cargo check -p vanta-memory exit 0; cargo nextest run -p vanta-memory 219/219 passed; cargo fmt --check exit 0; cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings exit 0
- **nextTask:** Task 19 (MEM-16 — F4 Orquestación timers+locks)

## Deuda técnica (Regla 6 - MUST)

**Saldo neto de deuda por PR:** Sin deuda

> 4 archivos nuevos + 1 mod directorio + 3 wirings aditivos + 1 archivo de tests. Sin dependencias nuevas. Sin ediciones destructivas en módulos existentes.

## Definition of Done (contrato multi-nivel - P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato arriba: cargo check vanta-memory + nextest vanta-memory (tests persona D19) + fmt + clippy con `-D warnings`, todos exit 0 |
| **Commit** | Commit atómico, conventional commit (`feat:` MEM-15), verificación mecánica — commit lo hace vanta-lead, no este worker |
| **Release** | No aplica esta iteración — `vanta-memory` es `publish = false`, aún en construcción MEM-09..18 |

**Gate:** se marca COMPLETED solo si pasa el nivel Task.

## Herramientas necesarias
- cargo (check, nextest, fmt, clippy)
- codegraph_explore (blast radius)
- TDAM clone `C:\Users\Eros\AppData\Local\Temp\opencode\tdam` @ `97f9465`

## Investigation Notes
- **Diseño del port (TDAM → Rust record store, no copia literal):**
  1. **Persistencia persona (Principio 2):** TDAM usa `persona.md` en filesystem + BackupManager. El port usa namespace `persona/<session>` con un único record key `persona.md` (sanitizada), payload JSON `PersonaRecord {content, mode, generated_at_ms, generated_at}` donde `content` = body + navigation appended (paridad TDAM persona.md final). No hay backup rotativo (el store es durable; restore = regenerar).
  2. **Modos first/incremental:** derivados del store — First si `get_persona` devuelve None, Incremental si existe (TDAM `persona-generator.ts:149`). Skip si existe Y ninguna escena cambió desde `generated_at` (TDAM L143-146). Cambios: `entry.updated > generated_at` lexicográfico (ambos RFC3339 fixed-width de `epoch_ms_to_rfc3339`); sin persona previa → todas las escenas vivas cuentan como cambiadas.
  3. **Contrato JSON en vez de tool-calls** (misma divergencia documentada de MEM-14): el LLM emite `{"persona": "<markdown>"}`; la capa determinista hace strip-nav defensivo, trim, escapeXmlTags, límite, append nav fresca, persiste. El agente con tools reales es MEM-16.
  4. **Límites:** chat 2000 / work 1200 chars (de los system prompts TDAM). Enforcement mecánico: rechazar (no truncar) — truncar markdown a mitad de frase corrompe el documento; rechazar preserva la persona previa y permite retry.
  5. **escapeXmlTags:** port EXACTO de sanitize.ts:288-294 — regex `</?(user-persona|relevant-memories|scene-navigation|relevant-scenes|memory-tools-guide|system|assistant)>` case-insensitive, escapando `<`→`&lt;` y `>`→`&gt;` solo en esos matches. Escapar TODO `<`/`>` rompería el markdown legítimo del persona.
  6. **Triggers puros:** TDAM lee checkpoint.json + filesystem; el port toma `PersonaTriggerInput` (contadores/flags) y `trigger_every_n` — la lectura del checkpoint es MEM-16. Recovery (P2Recovery) necesita flag `previously_generated` en el input (equivalente TDAM `last_persona_at > 0 && !hasPersonaBody`).
  7. **scene_navigation:** NAV_HEADER idéntico al TDAM (`---\n## Scene Navigation`) para que strip sea paridad; sin paths absolutos (record store: el nombre de escena es la key bajo `scene/<session>`); footer adaptado (recall/search en vez de read_file).
- **Decisiones de diseño:** error propio `PersonaError {Vanta, Serde, Invalid}` (no se agregan variantes a SceneError); resultado `PersonaGenerationResult {success, updated, mode, changed_scenes, error}` — `updated:false + success:true` = skip legítimo; `has_persona_body(content)` helper público para el trigger recovery path; unit tests inline + integration `tests/persona.rs` con fake runners inline (patrón scene_strategy.rs, sin features).

## Fases explícitas - SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluado: trust boundary (output de LLM = input no confiable, LLM05; inyección XML en secciones delimitadas). Mitigación: (1) escapeXmlTags obligatorio pre-persistencia; (2) límites mecánicos de tamaño con rechazo; (3) strip-nav defensivo del output; (4) validación de contenido vacío; (5) sanitización namespace/key reusada de l0_recorder; (6) sin deps nuevas, sin FFI, sin unwrap/expect. Se delega la auditoría final a vanta-audit (Review gate).
- [x] **PERFORMANCE** — evaluado: O(n) list_scenes + get_scene por escena cambiada por generación; no es hot path (generación puntual L3). Sin profiling requerido.

## Steps

### Step 1 — Discovery + task file
- [x] Leer plan Task 18, MEM-14 (template + API), TDAM persona-generator/persona-trigger/persona-generation/scene-navigation/sanitize.escapeXmlTags, scene_index/scene_format/scene_extractor/mods/abstractions/Cargo.toml, patrón tests
- [x] Verificar task file MEM-15.md no existe → crear (este archivo, Impacto mapeado Regla 0)
- [x] Decidir diseño (Investigation Notes) + verificar blast radius
- **Gate:** ✅ registro en task file antes de tocar código

### Step 2 — scene_navigation.rs + prompts/persona_generation.rs + wiring parcial
- [x] Crear `core/scene/scene_navigation.rs`: NAV_HEADER, heat_emoji, generate_scene_navigation, strip_scene_navigation + unit tests
- [x] Crear `core/prompts/persona_generation.rs`: params/result, build_persona_prompt (chat/work EN inglés, contrato JSON), límites 2000/1200 + unit tests
- [x] Editar `core/scene/mod.rs` + `core/prompts/mod.rs`: pub mod + re-exports
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 3 — core/persona/{mod,persona_trigger,persona_generator}.rs + wiring core
- [x] Crear `core/persona/mod.rs`: pub mod + re-exports
- [x] Crear `core/persona/persona_trigger.rs`: PersonaTriggerInput, TriggerResult, evaluate_persona_trigger (P1-P4) + unit tests
- [x] Crear `core/persona/persona_generator.rs`: persona_namespace, get_persona, has_persona_body, escape_xml_tags, PersonaRecord, PersonaGenerationResult, generate_persona<R: LlmRunner> + unit tests
- [x] Editar `core/mod.rs`: pub mod persona
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 4 — Tests D19 + verify completo + cierre
- [x] Crear `tests/persona.rs`: first generation, incremental, skip sin cambios, degrade runner sin escribir, output vacío rechazado preserva vieja, oversized rechazado, escapeXmlTags aplicado, nav roundtrip, triggers P1-P4
- [x] Verify: cargo check ✅ + nextest 219/219 ✅ + fmt --check ✅ + clippy -D warnings ✅ exit 0
- [x] CIERRE: campaign_update_task_state taskId=18 completed + recitation canónica; respuesta final con bloque RESULTADO §7
- **Gate:** verify todo exit 0 ✅
