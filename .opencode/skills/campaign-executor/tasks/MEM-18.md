# MEM-18: F4 Recall prepend/append + 3 modos

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md` (Task 21)
- **Fuente:** plan file Task 21 (MEM-18)
- **Esfuerzo:** 🔴
- **Prioridad:** 🔴
- **Tipo:** Rust (crate `vanta-memory`)
- **Creado:** 2026-08-20
- **Estado:** ✅ COMPLETED (verify 4/4 gates exit 0)

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `pipeline-full.md` (247), plan file Task 21 + Principios 1-7 + D19, task file `MEM-17.md` (95, plantilla), TDAM `MC/core/hooks/auto-recall.ts` (999 completo: strategies keyword/embedding/hybrid, RRF k=60, applyRecallBudget maxCharsPerMemory/maxTotalRecallChars, truncateRecallLine code-point-safe, formatMemoryLine `- [type|scene] content`, split prependContext (dinámico L1) vs appendSystemContext (estable persona+scene+tools guide), H-15 structured error), TDAM `MC/core/memory-prompt/composer.ts` (41 completo: composeMemorySystemPrompt byte-for-byte passthrough sin custom prompt, escapeClosingTags, GUARDS por layer), TDAM `MC/core/memory-prompt/resolver.ts` (102 completo: cadena candidatos agent→team→instance, resolveMemoryPrompts batch), TDAM `MC/core/memory-prompt/types.ts` (142 completo: MemoryPromptLayer l1/l2/l3, source agent/team/instance/system, ResolvedMemoryPrompt, buildMemoryPromptSettingId sha256-truncado), TDAM `MC/core/profile/profile-sync.ts` (494 completo: ProfileIsolation team+agent, buildProfileIsolationScope `team:{t}|agent:{a}`, MD5 verification skip-write-keep-local, pull/sync/ensure); crate: `auto_capture.rs` (198, patrón hook host-facing), `l1_reader.rs` (208 completo — recall_candidates overlap scoring REUSABLE), `persona_generator.rs` (get_persona/persona_namespace/strip via scene_navigation), `scene_index.rs` (list_scenes heat-desc), `abstractions/types.rs` (370, MemoryRecord), `core/mod.rs`, `hooks/mod.rs`, `Cargo.toml` (sin deps nuevas; sin sha2 → setting id determinista sin crypto hash)
- **Referencias hacia dentro:** nuevos módulos consumen `core::record::l1_reader::{read_session_records, significant_terms, overlap_score}` (estos dos últimos se vuelven `pub(crate)` — única edición a archivo existente), `core::persona::persona_generator::get_persona`, `core::scene::scene_index::list_scenes`, `core::scene::scene_navigation::{generate_scene_navigation, strip_scene_navigation}`, `core::conversation::{sanitize_component, sanitize_key}`, SDK `VantaEmbedded::{get, put}`
- **Referencias entrantes:** ninguna hoy — MEM-19/20/35 consumirán recall; wirings aditivos en `hooks/mod.rs`, `core/mod.rs` (solo `pub mod`)
- **Veredicto impacto:** bajo — archivos 100% nuevos bajo `core/hooks/`, `core/memory_prompt/`, `core/profile/` + 1 wiring aditivo + visibilidad pub(crate) en 2 fns de `l1_reader.rs`; cero archivos del core `vantadb` tocados

## Contrato

"`cargo check -p vanta-memory` pasa; tests dedicados de recall (D19) pasan (`cargo nextest run -p vanta-memory`); `cargo fmt --check` pasa; `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` pasa."

## Simplificaciones documentadas (ponytail — TDAM auto-recall.ts = 999 líneas)

| TDAM | Port Rust | Por qué |
|---|---|---|
| Strategy embedding/hybrid (VectorStore cosine + RRF merge) | Los 3 modos existen (`RecallMode::{Keyword,Embedding,Hybrid}`), pero Embedding/Hybrid degradan a keyword-overlap (mismo techo que `recall_candidates` MEM-11): el crate no tiene embeddings LLM-free ni API vectorial expuesta | Principio 4 (degradar, nunca bloquear); upgrade path idéntico al ya documentado en l1_reader.rs |
| Timeout race Promise.race + RecallErrors taxonomy (H-15) | Sin timeout: todo es sync in-process sobre VantaEmbedded local; errores tipados `#[non_exhaustive]` | No hay I/O remoto que colgue; la taxonomía completa es deuda del stack TDAM (gateway HTTP) |
| MEMORY_TOOLS_GUIDE (guía tools tdai_*) | Constante `MEMORY_TOOLS_GUIDE` en inglés genérico (sin nombres tdai_) | Regla prompts-en-inglés; los tools MCP llegan en MEM-19/20 |
| profile-sync pull/push fs+COS con MD5 y renames atómicos | `profile_sync.rs`: scope team+agent + sync del persona body (nav-stripped) a namespace `profile/{scope}` con content-hash idempotente + lectura scoped para recall | En vanta-memory TODO vive en VantaDB (Principio 2): no hay fs local ni COS que sincronizar; lo ejercitado es scope + idempotencia + lectura |
| buildMemoryPromptSettingId sha256-truncado | Id determinista `mps:{source}:{team}:{agent}:{layer}` sanitizado | Sin dep sha2 nueva (Regla sin-deps); el id no es security-sensitive, solo necesita unicidad+determinismo |
| resolveMemoryPrompts batch Map | `resolve_memory_prompt` single-target loop sobre cadena agent→team→instance | Un solo caller por turno; batch = slop hasta que exista |

## Invariantes de dominio (handoff - MUST)

1. Split prepend/append: L1 memories (dinámico per-turn) → `prepend_context`; persona + scene navigation + tools guide (estable, cacheable) → `append_system_context`.
2. 3 modos declarados: keyword / embedding / hybrid; sin recursos de embedding → degradación a keyword (Principio 4), nunca error.
3. user_text vacío → skip search pero persona/escenas igual se inyectan.
4. Nada que inyectar → `Ok(None)` (no bloque de vacío).
5. Budget recall: truncado per-memory + total, char-boundary-safe (Rust chars = code points).
6. Composer: system prompt intacto byte-for-byte cuando no hay custom prompt resuelto; escape de closing tags anti-inyección.
7. Resolver: prioridad agent > team > instance; solo prompts `status=active` y layer match.
8. Sanitización namespace `[A-Za-z0-9._/-]` ≤128, keys ≤512 sin NUL.
9. Sin unwrap/expect en producción; errores tipados #[non_exhaustive]; sin deps nuevas.

## Steps

### Step 1 — Discovery + task file
- [x] Leer TDAM (auto-recall, memory-prompt/*, profile-sync completos) + APIs del crate
- [x] Crear task file (este) con Impacto mapeado Regla 0
- **Gate:** ✅ registro antes de tocar código

### Step 2 — core/hooks/auto_recall.rs
- [x] RecallMode (3 modos), RecallConfig, RecallResult (prepend/append split)
- [x] keyword search reusando l1_reader overlap + format_memory_line + budget
- [x] perform_auto_recall: search → persona scoped → scene nav → compose
- [x] Wiring hooks/mod.rs + pub(crate) en l1_reader
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 3 — core/memory_prompt/{types,composer,resolver}.rs
- [x] types.rs: Layer/Source/records/ResolvedMemoryPrompt + setting id builder
- [x] composer.rs: compose_memory_system_prompt + guards + escape tags
- [x] resolver.rs: trait MemoryPromptStore + cadena agent→team→instance
- [x] Wiring core/mod.rs
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 4 — core/profile/profile_sync.rs
- [x] ProfileIsolation + build/parse scope `team:{t}|agent:{a}`
- [x] sync_persona_to_scope idempotente (content-hash) + read_scoped_persona
- [x] Wiring core/mod.rs
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 5 — Tests D19 + verify completo + cierre
- [x] Tests inline (budget, formatter, composer, resolver, scope) + integration `tests/recall.rs` (e2e recall con db in-memory, degradación embedding/hybrid, prepend/append split, idempotencia profile sync)
- [x] Verify: cargo check + nextest + fmt --check + clippy -D warnings → 4/4 exit 0, nextest 295/295
- [x] CIERRE: campaign_update_task_state taskId=21 completed + recitation; bloque RESULTADO §7
- **Gate:** verify todo exit 0

## Deuda técnica (Regla 6)

Sin deuda nueva neta. Desviaciones documentadas arriba (embedding/hybrid degradan a keyword — mismo techo documentado de MEM-11; timeout/taxonomía H-15 no portada — sync in-process; setting id sin crypto hash).

## Recitation (canónico)

- **activeGoal:** MEM-18: F4 Recall prepend/append + 3 modos
- **lastAction:** Implementado recall completo: `hooks/auto_recall.rs` (RecallMode keyword/embedding/hybrid con degradación a keyword-overlap documentada — Principio 4; RecallResult con split prepend_context (L1 dinámico per-turn) vs append_system_context (persona+scene nav+tools guide, cacheable); search_keyword reusa significant_terms/overlap_score de l1_reader (pub(crate)); format_memory_line `- [type|scene] content (activity time: ...)` TDAM-parity; apply_recall_budget per-memory + total char-boundary-safe; user_text vacío → skip search pero inyecta persona/escenas; nada → Ok(None)); `memory_prompt/types.rs` (Layer l1/l2/l3, Source agent/team/instance/system, records, setting id determinista sanitizado sin crypto hash); `memory_prompt/composer.rs` (compose_memory_system_prompt passthrough byte-for-byte sin custom prompt, escape_closing_tags case-insensitive `&lt;/...&gt;`, guards por layer en inglés); `memory_prompt/resolver.rs` (trait MemoryPromptStore + cadena agent→team→instance, solo prompts active + layer match); `profile/profile_sync.rs` (ProfileIsolation team+agent, scope `team:{t}|agent:{a}` build/parse, sync_persona_to_scope idempotente por igualdad de contenido (nav-stripped) a ns `profile/{scope}`, read_scoped_persona para recall). 27 tests D19 nuevos (13 unit inline + 14 integration tests/recall.rs). 4/4 gates exit 0.
- **result:** OK — 4/4 gates exit 0: cargo check ✅, nextest 295/295 (268 previos + 27 nuevos) ✅, fmt --check ✅, clippy -p vanta-memory --all-targets --no-deps -D warnings ✅ (7 warnings unsafe pre-existentes del core vantadb, fuera de scope)
- **nextAction:** Ninguna para MEM-18. Siguiente tarea del plan: Task 22 (MEM-19 — sanitize_text + truncación code-point); el lead commitea (`feat:` MEM-18)
- **contract:** cargo check -p vanta-memory exit 0; cargo nextest run -p vanta-memory 295/295 passed; cargo fmt --check exit 0; cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings exit 0
- **invariantes:** split prepend (dinámico L1) / append (estable persona+nav+guide); 3 modos declarados con degradación a keyword (Principio 4); user_text vacío → persona/escenas igual se inyectan; nada que inyectar → Ok(None); composer passthrough byte-for-byte sin custom prompt + escape closing tags anti-inyección; resolver prioridad agent>team>instance solo active+layer match; sanitización namespace [A-Za-z0-9._/-] ≤128 / keys ≤512 sin NUL; sin unwrap/expect producción; sin deps nuevas
- **deuda:** desviaciones ponytail documentadas (embedding/hybrid degradan a keyword — mismo techo MEM-11, upgrade al exponer API vectorial; timeout/taxonomía H-15 no portada — sync in-process; setting id sin crypto hash; profile-sync fs/COS no portado — todo vive en VantaDB Principio 2)
- **queda_pendiente:** commit por el lead
- **nextTask:** Task 22 (MEM-19 — F4 sanitize_text + truncación code-point)
