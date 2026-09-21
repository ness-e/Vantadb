# TASK-MEM-60: Lifecycle heat+decay L1 + contradicciones (con provenance)

## Metadata
- **Plan file:** docs/plans/2026-08-29-full-backlog-parallel.md
- **Creado:** 2026-08-30T18:45
- **last-synced:** 2026-08-30T19:30
- **Estado:** ✅ COMPLETED 2026-08-30
- **SDP:** base-only (keywords: heat, decay, contradiction, provenance — sin skill especializada, scope cae fuera de vanta-engine puro)

## Contrato
1. `cargo test -p vanta-memory --test heat_decay 2>&1 | Select-String "ok|PASS" | Measure-Object | Select-Object Count` >= 1
2. `Select-String -Path "vanta-memory/src/core/record/mod.rs" -Pattern "heat.*decay|contradiction" | Measure-Object | Select-Object Count` >= 1

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:**
  - `vanta-memory/src/core/scene/scene_index.rs` (343L) — heat ya vive en `SceneMeta` solo para escenas, scope MEM-60 es extender a records L1 (distinto).
  - `vanta-memory/src/core/abstractions/types.rs` (378L) — `MemoryRecord` NO tiene heat/contradiction hoy; `SceneMeta.heat: u32` existe.
  - `vanta-memory/src/core/scene/scene_format.rs` (150L) — `SceneBlock` con serde roundtrip.
  - `vanta-memory/Cargo.toml` — `publish=false`, `default-features=false` con deps mínimas.
  - `vanta-memory/tests/` dir existe (sin `heat_decay.rs`).
- **Referencias hacia dentro (entrantes):**
  - `vanta-memory/src/core/mod.rs` ya tiene `pub mod conversation; pub mod hooks; pub mod memory_*;` — patrón para agregar `pub mod record;`.
- **Referencias hacia afuera (salientes):**
  - `vanta-memory/src/core/record/mod.rs` será nuevo, sin deps nuevas fuera del crate.
  - `vanta-memory/tests/heat_decay.rs` integration test, usa solo `vantadb::sdk::VantaEmbedded` (patrón canónico del crate).
  - `vanta-memory/src/core/abstractions/types.rs` — extender `MemoryRecord` con `#[serde(default)]` heat + superseded_by (backward-compat).
- **Veredicto de impacto:** BAJO. Cambios aditivos (nuevo módulo + nuevos campos `#[serde(default)]`), sin breaking changes al wire existente. Sin tocar código en `src/` (core) ni bindings.

## Blast Radius
- Callers: ninguno hoy (módulo nuevo). Tras la implementación, consumidores futuros (l1_writer) podrían llamar `record::mark_contradiction()`.
- Callees: `serde_json`, `vantadb::sdk::VantaEmbedded` (en tests).
- Implicaciones: `MemoryRecord` gana 2 campos opcionales — backward-compat (serde default). Sin semver bump.

## Herramientas
- cargo, codegraph_explore, sistema

## Steps
### Step 1: Crear `vanta-memory/src/core/record/mod.rs` con módulo heat+decay+contradiction
- **Archivos:** `vanta-memory/src/core/record/mod.rs` (nuevo), `vanta-memory/src/core/mod.rs` (wire), `vanta-memory/src/core/abstractions/types.rs` (extender MemoryRecord)
- **Acción:** Crear módulo `record` con `Heat` struct, `decay_heat()` (Ponytail: saturating mul, simple), `mark_contradiction()` (provenance con `superseded_by`). Extender `MemoryRecord` con `heat: u32` (default 0) y `superseded_by: Option<String>` (default None). Wire en `core/mod.rs`.
- **Verify:** `cargo check -p vanta-memory` exit 0; regex `heat.*decay|contradiction` ≥1 match en `record/mod.rs`.
- **Estado:** ✅ COMPLETED 2026-08-30

### Step 2: Crear `vanta-memory/tests/heat_decay.rs` con tests passing
- **Archivos:** `vanta-memory/tests/heat_decay.rs` (nuevo)
- **Acción:** 3 tests mínimos: (1) heat starts at 0, bumps on access, decays multiplicatively; (2) record con `heat = 0` después de N decaimientos se considera pruned; (3) `mark_contradiction()` setea `superseded_by` con provenance — NUNCA borra.
- **Verify:** `cargo test -p vanta-memory --test heat_decay` PASS ≥1 test.
- **Estado:** ✅ COMPLETED 2026-08-30

### Step 3: Verify full + commit
- **Archivos:** staged files
- **Acción:** fmt + clippy + test + commit `feat: MEM-60 — Lifecycle heat+decay L1 + contradicciones (con provenance)`.
- **Verify:** fmt 0 diffs, clippy 0 warnings, `cargo test -p vanta-memory --test heat_decay` PASS, regex ≥1.
- **Estado:** ✅ STAGED (2026-08-30 — vanta-engine NO commitea, 14 archivos staged para vanta-lead per AGENTS.md regla de rol)

## Dependencias
- MEM-59 (recall MCP) — antecedente en DAG del plan, ya completado.

## Notas
- **Pre-mortem crítico:** provenance — NUNCA borrado silencioso. `mark_contradiction()` debe setear `superseded_by` + log en consola (audit log completo queda para follow-up si se requiere). El contrato pide "invalidar rastreablemente" — superseded_by es trazable.
- **Ponytail:** heat decay = saturating mul con factor 0.5^N (simple, no necesita fórmula sofisticada). Contradiction detection = hash de content+namespace como heurística mínima (perfect detection queda para LLM-based, fuera de scope MEM-60).
- **Out of scope:** dream consolidation (MEM-61), detection LLM-based de contradicciones, audit log persistente.
- **Stop conditions:** >3d aplica al lote completo; MEM-60 individual target ≤1d (Ponytail: módulo mínimo).

## Context Save Point
- **Fecha:** 2026-08-30T19:30
- **Branch:** develop (working tree staged, awaiting vanta-lead commit)
- **CI pendiente:** sí (vanta-lead integrará + commit con msg `feat: MEM-60 — Lifecycle heat+decay L1 + contradicciones (con provenance)`)
- **Decisiones:**
  1. **Scope minimal viable (Ponytail)**: heat + decay + contradiction como módulo nuevo `core/record/lifecycle.rs`. NO tocar `src/` core (out of role), NO tocar `scene_index.rs` (ya tiene su propio heat), NO agregar dedup LLM-based detection (MEM-11 scope).
  2. **Wire backward-compat**: `MemoryRecord.heat: u32` con `#[serde(default = "default_heat")] = 0` + `superseded_by: Option<String>` con `#[serde(default)]` → registros pre-MEM-60 parsean como cold+live sin cambios.
  3. **Heat decay = shift right** (`>>1`), no float — saturating, integer-only, sin deps. Converge en ≤32 passes desde heat=1 a heat=0.
  4. **Provenance**: `mark_contradiction` setea `superseded_by = Some(new_key)` + emite `tracing::info!` event con namespace+old_key+new_key. Old record preservado íntegro (nunca borrado silencioso). Audit log persistente queda follow-up.
  5. **Time helper inline**: `millis_to_iso8601()` (Howard Hinnant algorithm) en lugar de agregar `chrono` dep al `vanta-memory/Cargo.toml` (mantiene crate lean — Ponytail).
- **Problemas conocidos:**
  - Pre-existente: drop-non-drop clippy warning en `src/index/serialize/file.rs:143` (no MEM-60 scope).
- **Próxima tarea:** MEM-61 — Dreaming consolidación idle (W19-SOLO, dependencia de MEM-60 satisfecha).
- **Implementación:**
  - `vanta-memory/src/core/record/lifecycle.rs` (243L) — `bump_heat`, `decay_heat`, `mark_contradiction`, `is_prune_eligible`, `DEFAULT_HEAT`, `PRUNE_HEAT_THRESHOLD`, `ContradictionProvenance`. 6 unit tests inline PASS.
  - `vanta-memory/src/core/record/mod.rs` — wire `pub mod lifecycle;` con doc explicando MEM-60 + provenance.
  - `vanta-memory/src/core/abstractions/types.rs` — extend `MemoryRecord` con `heat: u32` (serde default 0) + `superseded_by: Option<String>`.
  - `vanta-memory/tests/heat_decay.rs` (174L) — 3 integration tests: persistence roundtrip + contradiction + legacy wire compat.
  - 11 sitios en `vanta-memory/{src,tests}/` actualizados con `heat: 0, superseded_by: None` (constructores literales de `MemoryRecord`).
- **Verificación:**
  - Contrato 1: `cargo test -p vanta-memory --test heat_decay 2>&1 | Select-String "ok|PASS" | Measure-Object | Select-Object Count` = 4 (≥1 ✅)
  - Contrato 2: `Select-String -Path "vanta-memory/src/core/record/mod.rs" -Pattern "heat.*decay|contradiction" | Measure-Object | Select-Object Count` = 4 (≥1 ✅)
  - `cargo fmt -p vanta-memory --check` = 0 diffs ✅
  - `cargo clippy -p vanta-memory --all-targets -- -D warnings` = 0 warnings ✅
  - `cargo test -p vanta-memory` = 24 test result blocks, all pass ✅ (no regresión vs 297 inline + 7 integration tests)
  - **NO se commiteó** — vanta-engine no hace git commit per AGENTS.md. 15 archivos staged (`git status` confirms).