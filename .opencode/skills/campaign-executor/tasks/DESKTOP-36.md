# DESKTOP-36: Bridge Tauri vanta-memory — exponer scene/persona/skill/genlog read-only

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24T00:00:00
- **Estado:** ✅ COMPLETED (sin commit — regla de la tarea)

## Impacto mapeado (Regla 0)
- **Leídos completos:** desktop/src-tauri/src/commands/memory.rs (787L, MEM-53/58), lib.rs, Cargo.toml (src-tauri), vanta-memory/src/gateway/knowledge_handlers.rs (1-178 + 179-273), core/memory_generation_log/{mod,store}.rs, core/skill/conversation_add/sink.rs, seed/{mod,input}.rs, bin/vanta-seed.rs, desktop/src/vanta.ts (540-610 + grep), vanta.test.ts
- **Referencias entrantes:** lib.rs registra commands::memory::* (generate_handler); vanta.ts consume `vanta_context_assemble`; vanta.test.ts mockea invoke
- **Referencias salientes:** gateway::{scene_read,scene_query} (firmas verificadas), memory_generation_log::query_session(db, session, Option<GenerationLayer>), seed::import_seed_str
- **Veredicto:** dep vanta-memory YA en src-tauri Cargo.toml (MEM-53). persona_get/scenes_list/skills_list YA existen — reutilizar, no duplicar. Agregar SOLO scene_read/scene_query/genlog_query + bindings TS. Sin writes. skill_versions/skill_restore/compaction_report SIN backing API en vanta-memory → deuda documentada (no inventar).

## Blast Radius
Callers: desktop/src/components/memory/* (nuevos), desktop/src/vanta.ts
Callees: vanta-memory/src/gateway/knowledge_handlers.rs (scene_read/list/query, persona/<session>, skills versionadas, genlog L1/L2/L3, compaction reports), desktop/src-tauri/src/commands/memory.rs (nuevo)
Implicaciones: vanta.ts expone scene_list/persona_get/skill_list/genlog_query probados contra seed de vanta-seed

## Spec
N/A — bridge read-only con contrato mecánico

## Contrato
`cargo check -p vantadb && cd desktop && npm run build`; `vanta.ts` expone scene_list/persona_get/skill_list/genlog_query probados contra seed de `vanta-seed`

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: Crear comandos Tauri para vanta-memory (read-only) — ✅
- Implementado: `vanta_scene_read`, `vanta_scene_query`, `vanta_genlog_query` en `desktop/src-tauri/src/commands/memory.rs` + registro en `lib.rs`. Reutilizados los existentes `vanta_persona_get`/`vanta_scenes_list`/`vanta_skills_list` (MEM-53 ya los cubría — no duplicar). Dep `vanta-memory` ya estaba en Cargo.toml (MEM-53).
- NO implementado (sin backing API en vanta-memory, no inventar): `skill_versions`/`skill_restore` (skills son content-hash upsert sin versiones; restore es WRITE → prohibido v1), `compaction_report(session)` (CompactionReport es output in-memory del assemble, ya expuesto vía `vanta_context_assemble`, no persiste por sesión). Deuda documentada abajo.

### Step 2: Extender vanta.ts con bindings TypeScript — ✅
- Tipos + wrappers en `desktop/src/vanta.ts` (inline, patrón existente del archivo — NO se creó types/memory.ts, desviación menor documentada): `memorySceneList/SceneRead/SceneQuery/PersonaGet/SkillList/GenlogQuery`. camelCase default del IPC para comandos MEM-53; snake_case para los nuevos (`rename_all = "snake_case"`).

### Step 3: Test contra seed vanta-seed — ✅ (vía path real de import)
- Rust: `seeded_store_answers_skills_list_persona_and_genlog` ejecuta `import_seed_str` (el MISMO código del binario `vanta-seed`) contra DB embedded + genlog L3 + skills content-hash + persona snapshot. Más tests: scene_read NotFound soft-delete, scene_query top_k/overlap, genlog filter L2 + limit + orden ts.
- TS: `vanta.test.ts` valida contrato wire exacto (nombres de comando + claves args snake/camel según comando). E2E contra app corriendo con DB seedeada no ejecutable en vitest → deuda documentada.
- Verify: cargo check ✅ · cargo test --lib 77/77 ✅ · npm run build ✅ · npm test 50/50 ✅

## Dependencias
- P27 Vanta Memory Engine (vanta-memory crate + handlers) — en ejecución
- DESKTOP-26 (tests frontend) — para test integración

## Notas
- DoD: `vanta.ts` expone scene_list/persona_get/skill_list/genlog_query probados contra seed de `vanta-seed`
- vanta-memory NO tiene transporte MCP/REST aún — bridge Tauri usa crate directo (patrón core inlined `../..`)
- Sin writes en v1 (observabilidad primero)