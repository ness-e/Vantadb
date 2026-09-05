# TASK-MEM-64: Skills versionadas + CompactionReport persistido

## Metadata
- **Plan file:** `docs/plans/2026-08-29-full-backlog-parallel.md`
- **Creado:** 2026-08-30T18:45
- **last-synced:** 2026-08-30T19:20
- **Estado:** ✅ COMPLETED

## Blast Radius
- **Archivos clave:** `vanta-memory/src/core/skill/conversation_add/{sink.rs, mod.rs}`, `vanta-memory/src/context_engine/{types.rs, engine.rs, mod.rs}`, `vanta-memory/src/services/pipeline_worker.rs`
- **Callers (IntegratedContext):** 5 sitios — `vanta-memory/src/context_engine/{mod.rs, engine.rs}`, `vanta-memory/src/services/pipeline_worker.rs`, `desktop/src-tauri/src/commands/memory.rs` (externo, no se toca), tests `e2e_flow.rs`
- **Callers (CompactionReport):** 6 sitios en `vanta-memory/src/context_engine/{types.rs, engine.rs, mod.rs, token_estimator.rs}`
- **Callers (sink.rs apply_candidates):** flujo internal de MEM-43 + tests `tests/e2e_flow.rs`
- **Implicaciones:** additive — un nuevo struct `SkillVersion` que se persiste adyacente a `StoredSkill`; un nuevo módulo `report_store.rs` con `record_compaction_report` y `list_compaction_reports`. No rompe signatura pública.

## Contrato
```
Select-String -Path "vanta-memory/src/core/skill/mod.rs" -Pattern "skill_versions|CompactionReport" | Measure-Object | Select-Object Count >= 1
```

Verify mecánico complementario (todos verdes):
- `cargo check -p vanta-memory` → exit 0 ✅
- `cargo clippy -p vanta-memory --all-targets -- -D warnings` → 0 warnings ✅
- `cargo fmt --check -p vanta-memory` → 0 diffs ✅
- `cargo test -p vanta-memory --lib` → 327/327 pass (incluye 1 sink test nuevo `records_history_on_create_and_update` + 2 report_store tests) ✅
- `cargo test -p vanta-memory --lib context_engine` → 31/31 pass ✅
- `cargo test -p vanta-memory --test context_engine` → 9/9 pass ✅

**Contrato principal: Count = 3 (≥1) ✅** (matches `skill_versions`×2 + `CompactionReport`×1 en module-level docstring)

## Spec (gate D — feature-add)
- **SkillVersions (history layer):** historial append-only por `(scope, name)` → cada update empuja a `skills_extract/{scope}/_versions/{name}/{version_seq}` con `{content, content_hash, updated_at_ms, prev_version_seq, created}`. seq autoincrementa desde el último registro; latest pointer queda en `skills_extract/{scope}/{name}` (status quo). Read API: `list_skill_versions(scope, name) -> Vec<SkillVersion>`. **Backward-compat**: el upsert existente sigue funcionando; el versionado es aditivo (parallel writes). Cursor se sigue escribiendo LAST.
- **CompactionReport por sesión:** nuevo módulo `context_engine/report_store.rs` con `PersistedCompactionReport` (envoltorio con `captured_at_ms`, `run_id`, `mmd_injected`, `recall_injected`, `report: CompactionReport`, contadores tokens/msgs). `record_compaction_report(db, session_id, &PersistedCompactionReport)` persiste a `context/{session}/compaction_reports/{run_id}`. `run_id = "<timestamp_ms>-<seq>"` con seq = `list_compaction_reports.len() + 1`. La fn la llama `run_context_assembly` después del `__assembled` write, **best-effort** (warn-on-fail, no fatal — el report es audit, no en critical path).

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:**
  - `vanta-memory/src/core/skill/mod.rs` — exports
  - `vanta-memory/src/core/skill/conversation_add/{mod.rs, sink.rs}` — sink upsert actual
  - `vanta-memory/src/context_engine/mod.rs` — exports
  - `vanta-memory/src/context_engine/types.rs` — CompactionReport/CompactionMode structs
  - `vanta-memory/src/context_engine/engine.rs` (~270L) — assemble, assemble_with_recall, IntegratedContext
  - `vanta-memory/src/services/pipeline_worker.rs` (líneas 380-565 + 596-620) — run_context_assembly + ASSEMBLED_KEY
- **Referencias hacia dentro:** IntegratedContext ya contiene `report: CompactionReport` (engine.rs:182). El "faltante" era persistirlo como registro separado por run, no solo embebido — ya implementado.
- **Referencias salientes (lo que importo):**
  - `vanta-memory/src/core/conversation/l0_recorder.rs` (sanitize_key, sanitize_component)
  - `vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata, VantaValue, VantaMemoryListOptions}`
- **Veredicto:** cambio aditivo puro — agregué tipos, fns de sink, módulo nuevo (report_store.rs), y call site nuevo en worker. No rompí signatura pública. Tests existentes siguen pasando (325 → 327 = +1 sink + +2 report_store, neto +3).

## Steps

### Step 1: Spec table + task file ✅
- **Archivos:** `.opencode/skills/campaign-executor/tasks/MEM-64.md`
- **Acción:** creado task file con Spec + Impacto + Steps
- **Verify:** task file existe + Estado ✅ COMPLETED
- **Estado:** ✅ COMPLETED

### Step 2: Agregar `SkillVersion` struct + version persistence al sink ✅
- **Archivos:**
  - `vanta-memory/src/core/skill/conversation_add/sink.rs` — nuevo `SkillVersion` struct; nueva fn `record_version(...)` que escribe a `skills_extract/{scope}/_versions/{name}/{version_seq}`; `last_version_seq`/`list_skill_versions` para read-back; `apply_candidates` invoca `record_version` en branches create/update
  - `vanta-memory/src/core/skill/conversation_add/mod.rs` — re-export `SkillVersion`
  - `vanta-memory/src/core/skill/mod.rs` — re-export + module-level docstring MEM-64
- **Acción:** versionado append-only
- **Verify:** `cargo check -p vanta-memory` 0 ✅; test `records_history_on_create_and_update` PASS ✅
- **Estado:** ✅ COMPLETED

### Step 3: Persistir `CompactionReport` por run en context_engine ✅
- **Archivos:**
  - `vanta-memory/src/context_engine/report_store.rs` (NUEVO) — `PersistedCompactionReport` + `record_compaction_report` + `list_compaction_reports` + 2 unit tests
  - `vanta-memory/src/context_engine/mod.rs` — declarar `mod report_store` + re-exports
  - `vanta-memory/src/services/pipeline_worker.rs` — import + invocar `record_compaction_report` después de write a `__assembled`, best-effort (warn on fail)
- **Acción:** persiste report adyacente al IntegratedContext para trazabilidad per-run
- **Verify:** `cargo check -p vanta-memory` 0 ✅; tests `round_trips_compaction_report` + `multiple_runs_kept_in_capture_order` PASS ✅
- **Estado:** ✅ COMPLETED

### Step 4: Contrato mecánico + verify full ✅
- **Archivos:** ninguno
- **Acción:** verify chain completo
- **Verify:** `Select-String ... -Pattern "skill_versions|CompactionReport" | Count >= 1` = 3 ✅; clippy 0 ✅; fmt 0 ✅; 327/327 tests PASS ✅
- **Estado:** ✅ COMPLETED

### Step 5: Stage + handoff ⏳
- **Archivos staged (working tree, awaiting vanta-lead commit):**
  - `.opencode/skills/campaign-executor/tasks/MEM-64.md` (task file)
  - `vanta-memory/src/core/skill/mod.rs` (docstring + re-export)
  - `vanta-memory/src/core/skill/conversation_add/mod.rs` (re-export)
  - `vanta-memory/src/core/skill/conversation_add/sink.rs` (SkillVersion + history methods + test)
  - `vanta-memory/src/context_engine/mod.rs` (mod + re-exports)
  - `vanta-memory/src/context_engine/report_store.rs` (NUEVO)
  - `vanta-memory/src/services/pipeline_worker.rs` (call site)
- **Acción:** `git status` muestra solo diffs esperados; vanta-worker NO commitea
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna upstream (MEM-43 ya shipped)

## Pre-mortem (del usuario) — estado
- **Fallo 1 (skill_versions migration rompe records viejos):** ✅ mitigado — additive, no se tocan `skills_extract/{scope}/{name}` existentes; nuevos writes a `_versions/{name}/{seq}`. Reads existentes siguen funcionando (test `same_task_reapplies_as_noop` confirma que el cursor + latest pointer siguen invariantes).
- **Fallo 2 (CompactionReport schema nuevo):** ✅ mitigado — nuevo struct `PersistedCompactionReport` con `#[serde(rename_all = "snake_case")]` implícito via derive; `CompactionReport` interno no cambia (backward compat 100%). Test `round_trips_compaction_report` verifica el serde shape.
- **Fallo 3 (persistir por sesión requiere schema change):** ✅ mitigado — usa namespace `context/{session}/compaction_reports/{run_id}` (sub-namespace separado de `__assembled`); existing `__assembled` sigue siendo la fuente de lectura, `compaction_reports` es aditiva (visible vía `list_compaction_reports`).

## Stop conditions
- >1d → docs-only changelog — no aplicable (terminado en una sesión)

## Notas
- **Write-tool anomaly:** varios intentos de Edit fallaron silenciosamente durante esta sesión (archivo sin cambios visibles en `git diff`). Resolví usando `Write` (full file) en vez de Edit. Posible issue con el flush del editor — anotado para investigación futura.
- **Bloqueo vantadb-mcp context_tests:** pre-existente (verificado con `git stash`), no introducido por MEM-64.

## Context Save Point
- **Fecha:** 2026-08-30T19:20
- **Branch:** develop
- **CI pendiente:** no (vanta-worker NO hace commit; vanta-lead integra)
- **Decisiones:**
  - `SkillVersion` struct (en sink.rs) — append-only history snapshot
  - `PersistedCompactionReport` struct (en context_engine/report_store.rs) — wrapper que aísla el schema del report
  - run_id = `<ms>-<seq>` — determinista, append-only
  - best-effort write del report en pipeline_worker (warn, no fail) — el report es audit, no critical path
- **Problemas conocidos:** ninguno (pre-mortem mitigado)
- **Próxima tarea:** MEM-65 (vanta-worker) — Telemetría por capa + pLimit real