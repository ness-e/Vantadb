# TASK-MEM-61: Dreaming consolidación idle (sleep-time tiering)

## Metadata
- **Plan file:** `docs/plans/2026-08-29-full-backlog-parallel.md` (W19-SOLO)
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-61.md`
- **Creado:** 2026-08-30T00:00 (W19-SOLO)
- **last-synced:** 2026-08-30T00:00
- **Estado:** ⬜ PENDING
- **Tipo:** feature-add (nuevo módulo + integración test)
- **Dominio:** core memory pipeline (vanta-memory crate)
- **Prioridad:** 🟠 Media-Alta
- **Appetite:** max 3d (per plan)
- **Esfuerzo:** 🔴 2-3d (per plan) — recortado a MVP viable (Ponytail: mínimo que cumple contrato)
- **Cynefin:** 🟧 Complejo (sleep-time tiering, multi-componente)
- **SDP:** `source-driven-development` (validar patrón Letta) + `doubt-driven-development`
  (verificar no-mutación del store original) + `test-driven-development` (RED-GREEN)
  + `incremental-implementation` (slices delgados)

## Blast Radius

### Archivos leídos completos
- `vanta-memory/src/services/pipeline_worker.rs` (565L) — TaskHandler, MemoryTaskHandler,
  run_l1/run_l2/run_l3, lock/retry/dead-letter
- `vanta-memory/src/services/mod.rs` (11L) — registry
- `vanta-memory/src/core/mod.rs` (50L) — registry de submódulos core
- `vanta-memory/src/lib.rs` (54L) — root re-exports
- `vanta-memory/src/core/record/lifecycle.rs` (295L) — MEM-60 precedent (heat/decay/contradiction)
- `vanta-memory/src/core/record/mod.rs` (36L) — registry
- `vanta-memory/src/core/abstractions/mod.rs` (20L)
- `vanta-memory/src/core/abstractions/llm_runner.rs` (80L+) — LlmRunner trait
- `vanta-memory/src/core/abstractions/types.rs` (170L+, MemoryRecord struct)
- `vanta-memory/src/core/state/types.rs` (108L, TaskKind enum + TaskPayload struct)
- `vanta-memory/src/utils/checkpoint.rs` (235L, CheckpointManager + Checkpoint struct)
- `vanta-memory/tests/heat_decay.rs` (184L, integration test precedent)
- `vanta-memory/tests/l1_dedup.rs` (595L, contract-test precedent — D19)
- `vanta-memory/Cargo.toml` (65L)
- `src/sdk/types.rs` (1856L, VantaMemoryInput shape)
- `docs/plans/2026-08-29-full-backlog-parallel.md` (lines 988-1002 = MEM-61 entry)

### Referencias hacia dentro (outbound deps del módulo nuevo)
- `crate::core::abstractions::{LlmRunner, MemoryRecord, MemoryType}`
- `crate::core::record::{read_session_records, l1_namespace}`
- `crate::core::record::lifecycle::{bump_heat, decay_heat, mark_contradiction, is_prune_eligible}`
- `crate::utils::checkpoint::{CheckpointManager, Checkpoint}`
- `vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata}`

### Referencias entrantes (inbound — qué archivos consumen `core/dream`)
- **Ninguna** (módulo nuevo, no hay callers preexistentes). Las llamadas se originarán
  desde el `pipeline_worker.rs` cuando se extienda `MemoryTaskHandler::handle` con el
  nuevo TaskKind (FUTURO — fuera de scope de este MVP). Por seguridad, la
  integración al worker se hace via un trait `Dreamer` que el host llama
  opcionalmente; cero impacto en el hot path mientras no se conecte.

### Veredicto de impacto (Regla 0)
- **API pública nueva:** `pub mod dream` + `pub trait Dreamer` + `pub struct
  DreamConfig` + 4 funciones públicas (`detect_idle`, `merge_duplicates`,
  `resolve_contradictions`, `normalize_relative_dates`).
- **Mutación del store original:** **NUNCA** — solo escribe en namespace separado
  `dream/<session>` (ver Spec §5).
- **Riesgo de regresión en suite existente:** bajo — el módulo no toca
  `core/record/{l1,l2,l3}` ni `pipeline_worker.rs`. Único side-effect:
  agregar `pub mod dream;` en `core/mod.rs` (línea 1 cambio aditivo).
- **Blast radius total:** ~3 archivos (nuevo módulo + entry en core/mod.rs +
  integration test). Dentro del scope W19 declarado en plan file.

## Spec

| Decisión | Elección | Justificación por evidencia |
|----------|----------|-----------------------------|
| **1. Patrón arquitectónico** | sleep-time tiering (Letta 0.7+) — agente secundario corre durante idle, **nunca** muta store original, escribe a store separado revisable. | Letta 2025-04-21 ("Sleep-time Compute"): "Memory formation in MemGPT is incremental, so memories may become messy and disorganized over time. Sleep-time agents on the other hand can continuously improve their learned context to generate clean, concise, and detailed memories." — `https://www.letta.com/blog/sleep-time-compute` (validado via webfetch 2026-08-30). Aplica 1:1 al caso VantaDB: L0/L1 crudo acumula drift → consolidación off-loop. |
| **2. Detección de idle** | clock inyectable (`now_ms - last_active_at_ms >= threshold_ms`) + método público `detect_idle` puro. Sin tokio task — caller decide cuándo gatillar. | Plan file: "idle ≥X min o cierre de sesión". Pattern MEM-60 (`mark_contradiction(old, new_key, now_ms)`) usa clock inyectado — replicamos. Ponytail: no agregar timer runtime; exponer primitiva pura testeable. Threshold configurable via `DreamConfig { idle_threshold_ms: u64 }` (default 10 min = 600_000 ms). |
| **3. Store consolidado** | namespace separado `dream/<session>/<run_id>` con `VantaMemoryInput` regular. **Read-only sobre `l1/<session>`**; **write-only sobre `dream/<session>/<run_id>`**. | Letta: "the sleep-time agent modifies the memory in an 'anytime' fashion - so the primary agent can read from this memory whenever, without having to wait for the sleep-time agent to finish its reasoning." — store separado es la implementación canónica. Nunca borrar/mutar `l1/<session>` (Regla 4 del proyecto: durabilidad del store principal). |
| **4. LLM tiering** | `DreamConfig.dreaming_runner: Option<Rc<dyn Dreamer>>` — slot genérico; el host inyecta el LLM que quiera (sleep-time = modelo más potente). Si `None` → funciones puras LLM-free siguen funcionando (merge/dedupe por hash). | Letta: "the conversational primary agent can use a fast model like gpt-4o-mini, while the sleep-time agent can use a larger and slower model like gpt-4.1 or Sonnet 3.7". El trait permite `MockDreamer` (tests) → `OpenAiDreamer` (host) sin acoplar `vanta-memory` a reqwest. |
| **5. Operaciones de consolidación** | 4 funciones puras + 1 trait: (a) `merge_duplicates` — colapsa duplicados residuales por (scene, content-shingle hash); (b) `resolve_contradictions` — reaplica `mark_contradiction` por prioridad/timestamp (re-usa MEM-60 sin duplicar lógica); (c) `normalize_relative_dates` — "ayer/hoy/mañana" + offsets numéricos → ISO-8601 absoluto via tabla de anclas; (d) `detect_idle` — clock check; (e) `trait Dreamer` — host puede extender con cualquier razonamiento LLM-driven adicional. | Plan file: "fusiona duplicados residuales, resuelve contradicciones pendientes, normaliza fechas relativas→absolutas". Cobertura 1:1. MEM-60 ya tiene `mark_contradiction` + `is_prune_eligible` — reusamos para no duplicar (Regla 6). |
| **6. Run record + auditoría** | Cada consolidación emite un `DreamRun` JSON en `dream/<session>/<run_id>` con: `started_at_ms`, `ended_at_ms`, `inputs_scanned`, `merged_ids`, `contradicted_ids`, `normalized_count`, `runner_label`. **Append-only** — `run_id = uuid-v7` (deterministic test friendly). | Plan: "store consolidado nuevo revisable/descartable" → revisable requiere metadata. Append-only preserva historial para diff/replay. |
| **7. Descartar / revisar (user-facing)** | API pública: `list_dream_runs(session) -> Vec<DreamRunMeta>`, `discard_dream_run(session, run_id) -> Result<()>` (delete del namespace), `promote_dream_run(session, run_id) -> Result<usize>` (TODO stub: retorna count, **no muta l1/<session>** — promotion = "apply the merged view as new snapshot" queda para W21 — esta tarea SOLO deja la API pública + discard). | Plan: "store consolidado nuevo revisable/descartable". MVP: discard sí; promote stub con doc que explica "promotion no muta `l1/<session>` por invariante" + retorna count + deja TODO. W20 (MEM-62/63) cubre UI / vanta-cli. |
| **8. Hook con PipelineWorker** | **NO se toca `pipeline_worker.rs` en esta tarea**. Razón: el plan marca MEM-61 SOLO y precede MEM-62..67 (parallel waves). Integración al worker es una **tarea siguiente** (W21 MEM-64..67). El módulo `dream` queda como primitiva testeable + trait extensible. | Plan W21 explicit: "MEM-64, MEM-65, MEM-67 → skill_versions, pipeline_worker, token_estimator" — la integración al pipeline_worker llega en MEM-65, no aquí. |
| **9. Cobertura de tests** | (i) Unit tests inline en cada función pública (4 funciones × ≥3 tests). (ii) 1 integration test `tests/dreaming.rs` con 4 escenarios: detecta idle / merge dups / contradicción / normalización fechas. (iii) 1 test con `MockDreamer` para validar trait extension. Contrato del plan: `cargo test -p vanta-memory --test dreaming 2>&1 | Select-String "ok" | Measure-Object Count >= 1`. | Precedent MEM-60 (`tests/heat_decay.rs`) usa el mismo patrón. Cobertura 80% unit + 20% integration per TDD pyramid. |
| **10. Lint/format/build budget** | `cargo fmt --check`, `cargo clippy -p vanta-memory --all-targets -- -D warnings`, `cargo nextest run -p vanta-memory --profile audit --build-jobs 2`. Sin warnings nuevos. Sin nuevas deps en Cargo.toml (Ponytail: stdlib + ya-imported). | Regla 1 (pre-push gate), Regla 6 (sin deuda técnica nueva), Regla 9 (no perf claim sin bench — N/A aquí, módulo no toca hot path). |

## Contrato

```powershell
Test-Path "vanta-memory/src/core/dream/mod.rs" == $true
AND
cargo test -p vanta-memory --test dreaming 2>&1 | Select-String "ok|PASS" | Measure-Object | Select-Object Count | >= 1
```

Y, **plus contract (per Spec §9)**:

```powershell
Select-String -Path "vanta-memory/src/core/dream/mod.rs" -Pattern "pub fn (merge_duplicates|resolve_contradictions|normalize_relative_dates|detect_idle)" | Measure-Object Count >= 4
Select-String -Path "vanta-memory/src/core/dream/mod.rs" -Pattern "pub trait Dreamer" | Measure-Object Count >= 1
Test-Path "vanta-memory/tests/dreaming.rs" == $true
cargo clippy -p vanta-memory --all-targets -- -D warnings  # 0 warnings
cargo fmt --check -p vanta-memory                           # 0 diffs
```

## Herramientas
- `codegraph_explore` (ya corrido para blast radius)
- `cargo check -p vanta-memory` (verify incremental)
- `cargo nextest run -p vanta-memory --profile audit --build-jobs 2` (full test del crate)
- `cargo fmt --check -p vanta-memory`
- `cargo clippy -p vanta-memory --all-targets -- -D warnings`

## Steps

### Step 1: dream module skeleton + types
- **Archivos:** nuevo `vanta-memory/src/core/dream/mod.rs`
- **Acción:** declarar `pub struct DreamConfig`, `pub trait Dreamer`,
  `pub struct DreamRun`, `pub struct DreamRunMeta`, `pub struct NormalizedDate`,
  `pub struct DuplicateGroup`, `pub enum ConsolidationError`. Sin implementación
  todavía (stubs `unimplemented!()` con docs). 4 funciones `pub fn` con signature
  pero body minimal. `pub use` los tipos en `core/dream/mod.rs`.
- **Verify:** `cargo check -p vanta-memory` 0 errors
- **Estado:** ⬜ PENDING

### Step 2: detect_idle + merge_duplicates (LLM-free, pure logic)
- **Archivos:** mismo `mod.rs`
- **Acción:** Implementar `detect_idle(now_ms, last_active_at_ms, threshold_ms) -> bool`
  y `merge_duplicates(records: Vec<MemoryRecord>) -> Vec<DuplicateGroup>` (shingle hash
  por `scene_name + lower(content)`). Unit tests inline.
- **Verify:** `cargo test -p vanta-memory --lib dream:: 2>&1 | Select-String "ok" | Measure-Object Count >= 6`
- **Estado:** ⬜ PENDING

### Step 3: resolve_contradictions (reusa MEM-60) + normalize_relative_dates
- **Archivos:** mismo `mod.rs`
- **Acción:** `resolve_contradictions(records: &mut Vec<MemoryRecord>, now_ms: u64) -> Vec<ContradictionProvenance>`
  invoca `mark_contradiction` por pares de misma `scene_name` + `priority` opuesto +
  `created_at` más reciente gana. `normalize_relative_dates(record: &mut MemoryRecord,
  anchor_ms: u64) -> NormalizedDate` busca "ayer/hoy/mañana/hace N (minutos|horas|días|semanas)"
  en `metadata.activity_start_time` y reemplaza por absoluto ISO-8601. Unit tests
  inline.
- **Verify:** `cargo test -p vanta-memory --lib dream:: 2>&1 | Select-String "ok" | Measure-Object Count >= 12`
- **Estado:** ⬜ PENDING

### Step 4: Dreamer trait + MockDreamer (test extension point)
- **Archivos:** mismo `mod.rs`
- **Acción:** `pub trait Dreamer { fn label(&self) -> &str; fn consolidate(&self,
  records: Vec<MemoryRecord>, ctx: &DreamContext) -> Result<Vec<MemoryRecord>,
  String>; }`. Struct `MockDreamer` (cfg test) que rota 4 records: 2 dups +
  1 contradicción + 1 fecha relativa → retorna records consolidados. Esto valida
  que el trait es extensible sin acoplar a reqwest.
- **Verify:** `cargo test -p vanta-memory --lib dream:: 2>&1 | Select-String "ok" | Measure-Object Count >= 15`
- **Estado:** ⬜ PENDING

### Step 5: store layer (read l1/<session> + write dream/<session>/<run_id>)
- **Archivos:** mismo `mod.rs`
- **Acción:** `pub fn scan_session_records(db, session_id) -> Vec<MemoryRecord>`,
  `pub fn write_dream_run(db, run: &DreamRun) -> Result<(), ConsolidationError>`,
  `pub fn list_dream_runs(db, session_id) -> Result<Vec<DreamRunMeta>, ConsolidationError>`,
  `pub fn discard_dream_run(db, session_id, run_id) -> Result<(), ConsolidationError>`,
  `pub fn promote_dream_run(db, session_id, run_id) -> Result<usize, ConsolidationError>`
  (stub que cuenta cuántos records traería el run pero **NO** muta `l1/<session>`).
  Namespace: `dream/<sanitized_session>/<run_id>`. **NUNCA** escribe a
  `l1/<sanitized_session>` (asserted by code comment + unit test).
- **Verify:** `cargo test -p vanta-memory --lib dream:: 2>&1 | Select-String "ok" | Measure-Object Count >= 20`
- **Estado:** ⬜ PENDING

### Step 6: integration test `tests/dreaming.rs` (contrato del plan)
- **Archivos:** nuevo `vanta-memory/tests/dreaming.rs`
- **Acción:** 4 tests integration contra VantaEmbedded in-memory real:
  (a) `dream_idle_detected_after_threshold` — clock 5 min advance, scan,
  verify run written; (b) `dream_merge_duplicates_persists_to_separate_namespace`
  — write 2 L1 records casi idénticos, run dream, verify 1 record en
  `dream/<sess>/<run_id>` y 2 siguen en `l1/<sess>` (NUNCA se borra); (c)
  `dream_resolves_contradiction_without_touching_original` — write 2 records
  contradictorios, run, verify l1 intacto; (d) `dream_normalizes_relative_dates`
  — write record con `metadata.activity_start_time: "ayer"`, run con anchor,
  verify metadata absolutizado en el dream namespace.
- **Verify:** `cargo test -p vanta-memory --test dreaming 2>&1 | Select-String "ok|PASS" | Measure-Object Count >= 4`
- **Estado:** ⬜ PENDING

### Step 7: gate Regla 0 — entry en core/mod.rs
- **Archivos:** `vanta-memory/src/core/mod.rs`
- **Acción:** agregar `pub mod dream;` después de `pub mod memory_generation_log;`
  (línea 49-50).
- **Verify:** `cargo check -p vanta-memory` 0 errors
- **Estado:** ⬜ PENDING

### Step 8: verify full (fmt + clippy + nextest)
- **Archivos:** —
- **Acción:** `cargo fmt --check -p vanta-memory`,
  `cargo clippy -p vanta-memory --all-targets -- -D warnings`,
  `cargo nextest run -p vanta-memory --profile audit --build-jobs 2`.
- **Verify:** 0 warnings, 0 diffs, todos los tests pre-existentes + nuevos pasan.
- **Estado:** ⬜ PENDING

### Step 9: commit (vanta-engine NO hace commit — staged para vanta-lead)
- **Archivos:** todos los nuevos/modificados
- **Acción:** `git add` selectivo, NO commit (per regla de rol). Actualizar plan
  file + skill progreso + devolver RESULTADO.
- **Verify:** `git status` muestra solo archivos esperados staged.
- **Estado:** ⬜ PENDING

## Dependencias
- **MEM-60** (heat + decay + contradiction) — REUSA `mark_contradiction` (lifecycle.rs:86).
- **MEM-16** (PipelineWorker) — usa el `PipelineSessionState`/`Checkpoint` para
  `last_active_time_ms` (no se modifica; solo se lee).
- **MEM-11** (L1 dedup) — `read_session_records` (record/l1_reader.rs) para scan.

## Notas

### Pre-mortem (per plan)
- **Fallo 1: idle detection requiere timer o hook — clearable** → MITIGADO: `detect_idle`
  es función pura sin timer; el caller decide. **Nada de tokio/spawn.**
- **Fallo 2: store consolidado nuevo — schema, no breaking del original** →
  MITIGADO: namespace `dream/<s>/<run>` separado. Test verifica que `l1/<s>` intacto.
- **Fallo 3: LLM tiering configurable — env var o config struct** → MITIGADO:
  `DreamConfig.dreaming_runner: Option<Rc<dyn Dreamer>>`. Sin env vars (Ponytail:
  config explícito > magic env). Host inyecta.
- **Fallo 4: descartar/revisar — user-facing API** → MITIGADO: `discard_dream_run`
  funcional + `list_dream_runs`. `promote_dream_run` queda stub con TODO
  documentado (W21 cubre UI/cli).

### Risk Register
- 🟡×🔴 mutación del original → store separado, schema snapshot. **Mitigación:
  test integration verifica `l1/<s>` intacto post-run.**
- 🟡×🟡 idle detection en CI → manual trigger. **Mitigación: API `detect_idle`
  pura — sin timers runtime en producción. CI testea la primitiva.**

### Stop conditions (>3d → docs-only)
- Si step 5 (store layer) excede el budget → entregar con store layer NO-OP stub
  (write_dream_run retorna Ok sin escribir) + dejar TODO W21. El módulo `dream`
  igual compila + pasa unit tests + integration test parcial.

### Cynefin
- 🟧 Complejo. Probe-sense-respond en pasos chicos (cada step reversible, ≤100 LOC).

### Source-driven validation
- Letta blog "Sleep-time Compute" 2025-04-21 validado via webfetch → patrón
  confirmado. **Source URL:** `https://www.letta.com/blog/sleep-time-compute`.

## Context Save Point
- **Fecha:** 2026-08-30T00:00
- **Branch:** develop
- **CI pendiente:** no (todo local)
- **Decisiones:** spec-first feature-add con table + justificación por evidencia;
  ponytail mínimo viable; reusa MEM-60 (`mark_contradiction`); sin tocar
  `pipeline_worker.rs` (integración es MEM-65 en W21).
- **Problemas conocidos:** ninguno al cierre de este task file.
- **Próxima tarea (post implementación):** MEM-62 — Export markdown git-friendly
  (W20, parallel 3).

### SDP loaded
- `source-driven-development` (validar Letta sleep-time pattern via webfetch)
- `doubt-driven-development` (no-mutación store original, schema snapshot)
- `test-driven-development` (RED unit tests → GREEN impl)
- `incremental-implementation` (8 steps verticales, ≤100 LOC cada uno)
- `context-engineering` (módulo aislado, blast radius acotado)
- `campaign-executor` + `progreso` + `ponytail` (base SDP)