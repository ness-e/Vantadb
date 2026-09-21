# MEM-38: Docs + ADR gate de cierre F5

## Metadata
- **Plan file:** `docs/plans/2026-08-21-vanta-context-engine.md` (Task 9)
- **Fuente:** plan file Task 9 (MEM-38)
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠 (gate pre-release)
- **Tipo:** Docs (vanta-docs) — NO código Rust, NO commit
- **Creado:** 2026-08-21
- **Estado:** ✅ COMPLETADA
- **Appetite:** max 1d

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** pipeline-full.md; plan file Task 9; task files fuente `MEM-{22,23,24,37,39,40,41,42}.md`; scripts/validate-docs-coverage.ps1 (193L — solo valida src/sdk + config + error + cli + python + MCP tools); docs/api/EMBEDDED_SDK.md (formato frontmatter + secciones por superficie); docs/_templates/adr.md; crate vanta-memory vía codegraph: context_engine/{mod,engine,token_estimator,mmd,mmd_injector}.rs, core/abstractions/llm_runner.rs, core/hooks/auto_recall.rs (RecallScope/RecallConfig), core/memory_generation_log/{mod,store}.rs, seed/mod.rs, bin/vanta-seed.rs; core: src/entity/scene.rs (SceneNodeStore).
- **Referencias hacia dentro:** docs nuevos referencian símbolos públicos de vanta-memory y src/entity/scene.rs. Sin tocar código.
- **Referencias entrantes:** validate-docs-coverage.ps1 NO escanea vanta-memory ni src/entity/scene.rs → los gaps de F5 no son detectados mecánicamente hoy; la documentación es deuda de Regla 3 igual.
- **Veredicto impacto:** bajo — 2 archivos nuevos (ADR-029, docs/api/VANTA_MEMORY.md) + 1 sección en EMBEDDED_SDK.md (scene nodes). Cero código.

## Steps

### Step 1 — Discovery + task file
- [x] Leer fuentes (task files + APIs públicas vía codegraph)
- [x] Crear task file
- **Gate:** ✅ registro antes de editar

### Step 2 — ADR borrador técnico
- [x] `docs/architecture/adr/ADR-029-vanta-memory-context-engine.md` con banner "Borrador técnico para revisión del autor" (Regla 5 forcing function)
- **Gate:** ✅ cubre D21/D22/D23 + trade-offs heat/compresión/MMD/recall-overlap + deuda

### Step 3 — docs/api/VANTA_MEMORY.md
- [x] Superficies F5: context_engine (assemble/assemble_with_recall/emergency_truncate/TokenEstimator/CompactionReport/AssembleConfig), mmd (TaskMemory/save_active/load_active/push_history/list_history/fingerprint/inject_mmd), generation_log (query_session/record_best_effort), seed (vanta-seed CLI), recall_scope (RecallScope D22)
- **Gate:** ✅ formato EMBEDDED_SDK.md (frontmatter + tablas)

### Step 4 — Deuda MEM-12: entity::scene en core
- [x] Sección "Scene Nodes (L2 anchors)" en docs/api/EMBEDDED_SDK.md (SceneNodeStore CRUD)
- **Gate:** ✅

### Step 5 — ROADMAP + verify mecánico + cierre
- [x] ROADMAP: no existe archivo/sección ROADMAP en el repo (root ni docs/) → nada que actualizar; anotado como hallazgo
- [x] Verify: `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps ✅
- [x] Cierre: campaign_update_task_state taskId=9 completed + RESULTADO §7 (commit omitido: orquestador ordenó NO commitear)

## Deuda técnica (Regla 6)

Sin deuda nueva neta. Documentada en ADR-029 §Known Debt (pre-existente): keyword-overlap sin vectores, chars/3 subestima CJK, aggressive→emergency con prefijo protegido over-budget, summary placeholder hasta L1. Gap mecánico: validate-docs-coverage.ps1 no escanea vanta-memory → cobertura de este crate no es enforceada por CI (upgrade futuro del script).

## Recitation (canónico)

=== RECITATION ===
- **activeGoal:** MEM-38 — Docs + ADR gate de cierre F5 (Task 9, plan 2026-08-21-vanta-context-engine)
- **lastAction:** ADR-029 (borrador técnico con banner revisión humana), docs/api/VANTA_MEMORY.md nuevo (context_engine/mmd/generation_log/seed/recall_scope), sección SceneNodeStore en EMBEDDED_SDK.md (deuda MEM-12), verify 0 gaps
- **result:** OK
- **nextAction:** Ninguna para MEM-38. Orquestador: review del autor humano del ADR antes de aprobar (Regla 5) + commit
- **contract:**
  - verificacion: `pwsh scripts/validate-docs-coverage.ps1` ✅ 0 gaps · `cargo doc -p vanta-memory --no-deps` no requerido (sin cambios de código)
  - evidencia:
    - claim: ADR-029 publicado como BORRADOR técnico | evidencia: docs/architecture/adr/ADR-029-vanta-memory-context-engine.md (banner línea 1) | confianza: alta
    - claim: superficies F5 documentadas | evidencia: docs/api/VANTA_MEMORY.md (context_engine, mmd, generation_log, seed, recall_scope) | confianza: alta
    - claim: deuda MEM-12 cerrada | evidencia: docs/api/EMBEDDED_SDK.md §"Scene Nodes" | confianza: alta
    - claim: verify contrato pasa | evidencia: salida validate-docs-coverage.ps1 "0 gaps" | confianza: alta
  - artefactos: docs/architecture/adr/ADR-029-vanta-memory-context-engine.md, docs/api/VANTA_MEMORY.md, docs/api/EMBEDDED_SDK.md (editado), .opencode/skills/campaign-executor/tasks/MEM-38.md
  - invariantes: inglés (Doc Language Split) · sin tocar código Rust · sin commitear · plan file intacto · ADR marcado borrador (autor humano articula decisión final)
  - deuda: ROADMAP inexistente (nada que actualizar); validator no escanea vanta-memory (gap mecánico pre-existente)
  - queda_pendiente: revisión humana del ADR + commit (lead)
- **nextTask:** ninguna (última tarea del plan P29 — CP3)
=== END RECITATION ===
