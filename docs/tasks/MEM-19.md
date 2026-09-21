# MEM-19: F4 sanitize_text + truncación code-point

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md` (Task 22)
- **Fuente:** plan file Task 22 (MEM-19)
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Tipo:** Rust (crate `vanta-memory`)
- **Creado:** 2026-08-20
- **Estado:** ✅ COMPLETED

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `pipeline-full.md` (247), plan file Task 22, task file `MEM-18.md` (93, plantilla), TDAM `MC/src/utils/sanitize.ts` (405 completo: sanitizeText con 10 reglas de limpieza, stripCodeBlocks, shouldCaptureL0/shouldExtractL1, PROMPT_INJECTION_PATTERNS + looksLikePromptInjection, isFrameworkNoise, pickRecentUnique, escapeXmlTags, sanitizeJsonForParse + escapeControlCharsInJsonStrings), TDAM `MC/src/utils/text-utils.ts` (31 completo: extractWords Latin+CJK bigrams); crate: `utils/mod.rs` (20), `offload/local_llm/parsers/json_utils.rs` (271 completo — extract_json/sanitize_json_for_parse/fix_trailing_commas YA portados en MEM-14), `core/conversation/l0_recorder.rs` (sanitize_component/sanitize_key pub(crate)), `core/hooks/auto_capture.rs` (198 completo — sanitize_content/strip_fenced_code_blocks PRIVADAS), `core/record/l1_extractor.rs` (260 completo — should_extract_l1/is_framework_noise PRIVADAS, ya portan isFrameworkNoise), `core/persona/persona_generator.rs` (escape_xml_tags pub + INJECTION_BOUNDARIES, ya porta escapeXmlTags), `core/hooks/auto_recall.rs` (488 completo — truncate_line PRIVADA ya hace truncación code-point con sufijo), `core/record/l1_reader.rs` (208 — significant_terms cubre la parte Latin de extractWords)
- **Referencias hacia dentro:** nuevos módulos consumen `core::record::l1_extractor::is_framework_noise` (pasa a pub(crate)), `core::hooks::auto_capture::strip_fenced_code_blocks` (pasa a pub(crate)); `auto_recall.rs` delega su `truncate_line` en `utils::text_utils::truncate_with_suffix`
- **Referencias entrantes:** ninguna hoy — módulos nuevos bajo `utils/` + wirings aditivos en `utils/mod.rs` (solo `pub mod`); únicas ediciones a archivos existentes: visibilidad pub(crate) en 3 fns + delegación de truncate_line
- **Veredicto impacto:** bajo — 2 archivos 100% nuevos (`utils/sanitize.rs`, `utils/text_utils.rs`) + wiring aditivo + 3 cambios de visibilidad pub(crate) sin movers de archivo (cero callers rotos); cero archivos del core `vantadb` tocados

## Contrato

"`cargo check -p vanta-memory` pasa; tests dedicados de sanitize (D19) pasan (`cargo nextest run -p vanta-memory`); `cargo fmt --check` pasa; `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` pasa."

## Consolidación vs port nuevo (decisión de diseño)

| Pieza TDAM sanitize.ts | Estado en crate | Acción MEM-19 |
|---|---|---|
| sanitizeText (reglas 1-10) | ❌ NO existe | **NUEVO** en `utils/sanitize.rs` — escaneo manual sin regex (sin deps nuevas) |
| shouldCaptureL0 | parcial (role/min-len en auto_capture; ruido en l1_extractor) | **NUEVO** wrapper que delega en `is_framework_noise` (pub(crate)) |
| looksLikePromptInjection | ❌ NO existe | **NUEVO** subconjunto documentado sin regex |
| stripCodeBlocks | ✅ `strip_fenced_code_blocks` (auto_capture, privada) | pub(crate) + re-export |
| shouldExtractL1/isFrameworkNoise | ✅ `l1_extractor.rs` (privadas) | pub(crate) + re-export |
| escapeXmlTags | ✅ `persona_generator.rs` (pub) | re-export |
| sanitizeJsonForParse/fixTrailingCommas | ✅ `json_utils.rs` (pub, MEM-14) | re-export |
| pickRecentUnique | ❌ NO existe | **NUEVO** en `utils/text_utils.rs` |
| Truncación code-point | ✅ `truncate_line` (auto_recall, privada) | se consolida en `text_utils::truncate_with_suffix`; auto_recall delega |

## Invariantes de dominio (handoff - MUST)

1. Sin deps nuevas: los patrones regex de TDAM se reimplementan con escaneo manual de strings (semántica documentada por regla).
2. Sanitizers nunca panican: bloque sin cierre → se conserva el texto verbatim (nunca se come contenido sin frontera).
3. `sanitize_text` es para capture (L0) y recall (limpieza de query) — NO aplica a contenido ya persistido.
4. Truncación por code points (chars Rust), nunca por bytes — UTF-8 safe.
5. Consolidación por delegación/re-export; NO mover funciones si rompe callers.
6. Sin unwrap/expect en producción; sin deps nuevas; errores tipados cuando aplique.

## Steps

### Step 1 — Discovery + task file
- [x] Leer TDAM (sanitize.ts 405 + text-utils.ts 31 completos) + APIs del crate (codegraph ×3)
- [x] Crear task file (este) con Impacto mapeado Regla 0
- **Gate:** ✅ registro antes de tocar código

### Step 2 — utils/text_utils.rs + consolidación truncate
- [x] `truncate_chars` / `truncate_with_suffix` (code-point safe) + `pick_recent_unique`
- [x] `auto_recall.rs` delega `truncate_line` en text_utils (elimina duplicación)
- [x] Wiring `utils/mod.rs`
- **Gate:** ✅ `cargo check -p vanta-memory` exit 0

### Step 3 — utils/sanitize.rs (consolidado + port nuevo)
- [x] `sanitize_text`: 10 reglas TDAM sin regex (tags de contexto, bloques untrusted, fence JSON session, directivas reply, timestamps, media markers, image-reply, líneas System:, data URIs base64, NUL+whitespace)
- [x] `should_capture_l0` (delega is_framework_noise) + `looks_like_prompt_injection` (subconjunto documentado)
- [x] Re-exports pub(crate) de helpers dispersos + visibilidad pub(crate) en auto_capture (`strip_fenced_code_blocks`) y l1_extractor (`should_extract_l1`, `is_framework_noise`)
- **Gate:** ✅ `cargo check -p vanta-memory` exit 0

### Step 4 — Tests D19 + verify completo + cierre
- [x] Tests inline sanitize (una por regla + casos límite UTF-8) + text_utils (truncación multibyte, dedup) — 27 tests D19 nuevos, suite completa 336/336
- [x] Verify: cargo check + nextest + fmt --check + clippy -D warnings — todos exit 0
- [x] CIERRE: campaign_update_task_state taskId=22 completed + recitation; bloque RESULTADO §7
- **Gate:** ✅ verify todo exit 0

## Bugs encontrados durante implementación
- `strip_paired_tags`: branch de estructura fallida comía el prefijo previo al tag (`out.push_str(&rest[open..])` sin volcar `rest[..open]`) — detectado por test `unclosed_tag_keeps_text_verbatim`, corregido a `push_str(rest)` verbatim.
- Test pre-existente del trabajo parcial: `pick_recent_unique::<&str>(&[], 3)` no compila (`impl Trait` no admite turbofish) → `&[] as &[&str]`; expectativa de `truncate_suffix_is_utf8_safe` era matemáticamente incorrecta (keep = max − suffix_len).

## Deuda técnica (Regla 6)

Sin deuda nueva neta. Desviaciones documentadas: `looks_like_prompt_injection` cubre subconjunto de los 16 patrones TDAM (los de alta señal; sin regex crate el resto sería frágil); `extractWords` CJK-bigrams de text-utils.ts NO portado (significant_terms de l1_reader ya cubre Latin; CJK llega cuando haya caller).

## Recitation (canónico)

- **activeGoal:** MEM-19: F4 sanitize_text + truncación code-point
- **lastAction:** sanitize.rs consolidado + tests D19 (27 nuevos) + verify 4/4 exit 0; task cerrada
- **result:** ✅
- **nextAction:** ninguna — tarea completada; commit pendiente del lead
- **contract:** cargo check -p vanta-memory ✅; cargo nextest run -p vanta-memory ✅ (336/336); cargo fmt --check ✅; cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings ✅
- **invariantes:** sin deps nuevas (escaneo manual sin regex); sanitizers nunca panican (bloque sin cierre → texto verbatim); truncación por code points nunca bytes; consolidación por delegación sin mover funciones
- **deuda:** looks_like_prompt_injection subconjunto de patrones TDAM; extractWords CJK no portado (sin caller)
- **queda_pendiente:** commit por el lead
- **nextTask:** Task 23 (siguiente del plan)
