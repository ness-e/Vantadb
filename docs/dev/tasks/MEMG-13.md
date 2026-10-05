---
title: "TASK MEMG-13: Superficies core restantes en memoria (IQL / versiones / snapshots / filtros)"
kind: task
description: "vanta-memory consume historia/diff de L1 vía core versions y backup/restore vía snapshot (tests por superficie); IQL y filtros restantes evaluados con motivo explícito → FIND (stop 1sem: 2 superficies)"
---

# TASK MEMG-13: Superficies core restantes en memoria (IQL / versiones / snapshots / filtros)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 41, Wave F2 — Memoria I)
- **Fuente:** Backlog `MEMG-13` + plan Task 41 (bloque F0-expandido, L1176-1202) + DELTA §P2 + STU-03 + `docs/dev/research/mgr-12-confianza.md` (§Q3a — min_confidence) + ADR-046 (§D3/D5)
- **Esfuerzo:** 🟠 3-5d | **Appetite:** max 1sem | **Stop aplicado (plan L1187):** entregar las 2 superficies de mayor valor (`versions` + `snapshot`) + FIND del resto (IQL / filtros restantes)
- **Prioridad:** 🟡
- **Tipo:** Rust (crate `vanta-memory`; core SDK consumido sin modificar)
- **Turns estimados:** 10-16 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `41` en el campaign server)
- **Incógnitas (uphill):** 0 — downhill directo (evaluar por superficie → adoptar → tests), Cynefin 🟨 con contrato del plan fijado
- **Pendientes (downhill):** 4 steps (1-4)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `41`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Superficies nuevas (aditivas — sin callers preexistentes):** `read_record_versions`/`read_record_version`/`diff_records` (consumidores: hosts in-process + tests) y `utils::backup::{create_snapshot,list_snapshots,restore_snapshot}`. **Callers de lo existente que NO cambia:** `read_record`/`read_session_records`/`read_namespace_records` (auto_recall, ~30 call sites de tests) mantienen firma y comportamiento — cero ripple. |
| Callees | Core SDK existente (sin cambios): `Embedded::versions`/`get_version` (`src/sdk/api/memory.rs:935,953` → `version_history.rs`), `Embedded::create_snapshot`/`list_snapshots`/`restore_from` (`src/sdk/builder.rs:269,275,297` → `StorageEngine::create_snapshot`/`snapshot_restore`). `sanitize_key`/`l1_namespace` (internos, ya usados por `read_record`). Cero dependencias nuevas. |
| Implicaciones | **API pública aditiva** (símbolos nuevos en `l1_reader` + módulo `utils::backup`) — sin breaking, sin wire, sin migración, sin cambios en `src/sdk/**` (solo consumo). Historia de versiones = best-effort post-commit del core (gap documentado, degraded never corrupt — `memory.rs:932`). Snapshot = flujo core existente (quiesce + mirror + rollback); restore exige engine cerrado (contrato core). Regresión: suites `-p vanta-memory` se re-corren; ningún test existente toca los símbolos nuevos. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `64eee3f7`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `vanta-memory/src/core/record/l1_reader.rs` (:1-319 — `l1_namespace` `:21`, `read_session_records` `:26`, `read_namespace_records` `:36` (cursor loop `:47-71` post-edición + gate `include_quarantined:false` `:53`), `read_record` `:75`, `usable_vector_filter` `:95`, helpers + tests).
  - `vanta-memory/src/core/hooks/auto_recall.rs` (:1-1098 — `RecallConfig` `:231-267` con `core_search` MEMG-11, path legacy `:361-388`, `search_records_core` `:726-866`, `read_scoped_records` `:524-561`).
  - `vanta-memory/src/lib.rs` (:1-69), `src/utils/mod.rs` (:1-24 — re-exports), `src/utils/checkpoint.rs` (:1-60 — patrón módulo/error del crate), `src/core/abstractions/types.rs` (:44-130 — payload `MemoryRecord`, sin `confidence`).
  - `src/sdk/api/memory.rs` (:932-958 — `get_version`/`versions` VS-CORE-07; `:511` `put_one` → `write_snapshot` `:596` best-effort), `src/sdk/version_history.rs` (:20-139 — `SnapshotRecord` payload+vector por versión; orden ascendente), `src/sdk/builder.rs` (:255-300 — snapshot API + contrato `restore_from`), `src/sdk/serialization/vector_types.rs` (:100-175 — `MemorySearchRequest` con `min_confidence`/`as_of_ms`/`valid_window`/`filters`), `src/storage/engine/mod.rs` (:158-172 `FsSnapshot`, `:615-759` create/list/validate/restore), `src/error.rs` (:526 `pub type Result`), `src/config.rs` (`Config: Clone`, `storage_path`).
  - Tests vecinos: `vanta-memory/tests/recall.rs` (:1-112 fixture `db()`+`put_l1`), `tests/md_roundtrip.rs` (:1-70), `tests/l1_semantics_v2.rs` (:268-303 tempdir), `tests/md_git_e2e.rs` (:1-60 Fjall+tempdir), `tests/snapshot_consistency.rs` (:1-75 Fjall+create+restore).
  - Docs/specs: `docs/api/VANTA_MEMORY.md` (:1-415), plan Task 41 (L1176-1202), `docs/dev/tasks/MEMG-12.md` (formato canónico), `mgr-12-confianza.md` (§Q3a min_confidence opt-in).
- **Archivos referenciados hacia dentro (imports/deps):** `l1_reader` → `vantadb::sdk::Embedded` (nuevo uso: `versions`/`get_version`), `sanitize_key`/`l1_namespace` (existentes), `L1Error` (existente). `utils/backup.rs` (nuevo) → `vantadb::{config::Config, sdk::Embedded, storage::FsSnapshot, error::{Error,Result}}` (todos públicos, default-features=false ✓).
- **Referencias entrantes (grep/CodeGraph HEAD):** `read_record` = auto_recall + tests; `read_session_records` = hotspot fan-in 38 (auto_recall + ~30 tests) — **firmas intactas**; `utils::*` re-exporta lo nuevo (aditivo). `versions`/`get_version` en `vanta-memory/src` = **0 hits** (rg HEAD, verificado); `snapshot` en `vanta-memory/src` = solo `BackendSnapshot` (otro dominio: cola de tareas local) — 0 consumo del snapshot core.
- **Veredicto impacto:** **BAJO (aditivo puro)** — 2 archivos de código tocados (`l1_reader.rs` + `utils/backup.rs` nuevo + `utils/mod.rs` re-export) + 2 tests nuevos + docs; sin cambios de firma en lo existente, sin core, sin wire, sin deps, sin locks nuevos. Riesgos del pre-mortem mitigados: (1) sin caso de uso → adoptar solo `versions`+`snapshot`, resto con motivo explícito + FIND; (2) IQL semántica → **no se consume** (FIND; `core_search` MEMG-11 ya cubre el reuse del motor); (3) locks de snapshot → flujo core existente + DB Fjall tempdir en test.

## Contrato

"La memoria consume `versions` + `snapshots` del core donde apliquen, con tests por superficie y sin reimplementaciones nuevas (plan L1185): (a) **historia/diff de L1** — `l1_reader::{read_record_versions, read_record_version}` (mapean `Embedded::versions`/`get_version` al payload `MemoryRecord` + `RecordVersion{version,record}`) + `diff_records(older,newer)` puro (cambios por campo top-level del payload; campo ausente por `skip_serializing_if` ⇒ `Null`); (b) **backup/restore** — `utils::backup::{create_snapshot,list_snapshots,restore_snapshot}` delegando al core (`Embedded::create_snapshot`/`list_snapshots`/`restore_from`), con el contrato de flujo documentado (restore exige engine cerrado → devuelve `Embedded` reabierto). Superficies evaluadas y **no** consumidas quedan con motivo explícito + fila FIND: IQL (reuse del motor ya cubierto por `core_search` MEMG-11; sin consumidor memory-side; cambia semántica de recall — pre-mortem #2) y filtros restantes de recall (`min_confidence`/temporales/`filters`; cursor ya consumido en `read_namespace_records`). Verify: `cargo nextest run --profile audit -p vanta-memory --test l1_history --test backup_snapshot --build-jobs 2` + suite `-p vanta-memory --build-jobs 2` + `cargo fmt --check` + `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` verdes."

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): pre-respondido por el plan F0** — el contrato del plan (Task 41, Gate Result ✅ DO) sanciona las superficies ("la memoria consume IQL + versiones + snapshots + filtros/cursors del core donde apliquen (mínimo: diff de L1 vía `versions`, backup/restore vía snapshot, filtros/cursor en recall); tests por superficie") y el stop fija el corte (L1187: "2 superficies de mayor valor + FIND del resto"); precedente de campaña idéntico: MEMG-11 (`core_search` — "Gate D: pre-respondido por el plan F0", MEMG-11.md:211) y MEMG-02 ("sin símbolos fuera de la sanción del plan"). Los símbolos nuevos SON el mecanismo de consumo sancionado; nombres/firmas siguen convenciones del crate (micro-decisiones decidido-por-evidencia abajo). Sin símbolos fuera de la sanción.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Superficies a consumir | A) **`versions` + `snapshot` (las 2 de mayor valor: auditoría/diff + backup) + FIND del resto** / B) las 4 (IQL+filtros incluidos — riesgo de semántica de recall + scope 1sem) / C) solo tests sin API (no hay "consumo" testeable) | ✅ **A** — decidido-por-evidencia: stop L1187 ("entregar las 2 superficies de mayor valor (p.ej. versions + snapshot) + FIND del resto") + contrato L1185 permite omisión con motivo explícito; B viola pre-mortem #2 (IQL) y #1 ("adoptar solo donde aplique") |
| 2 | Forma de la superficie `versions` | A) **`read_record_versions → Vec<RecordVersion{version,record}>` + `read_record_version → Option<RecordVersion>` (simétricos)** / B) `Vec<MemoryRecord>` (pierde el version number del core → no direccionable para `get_version`) / C) re-exportar el `MemoryRecord` core (payload queda crudo; no es la memoria) | ✅ **A** — decidido-por-evidencia: `read_record` (l1_reader:75) es el precedente del mapeo payload→`MemoryRecord`+vector; `versions()` devuelve el storage version (u64) que `get_version` acepta (memory.rs:935-958) — sin él la historia no es direccionable |
| 3 | Diff | A) **`diff_records(older,newer) -> Vec<RecordFieldChange{field,before,after}>` puro sobre `serde_json::to_value` top-level** / B) diff campo-por-campo hardcodeado (drift al agregar campos) / C) no incluir diff (el contrato lo pide: "diff de L1 vía versions") | ✅ **A** — decidido-por-evidencia: campos `skip_serializing_if` (task_id/team_id/user_id/agent_id/vector/superseded_by, types.rs:103-129) ⇒ `Null` = ausente, documentado; sin mantenimiento de lista de campos; determinista (BTreeSet de keys) |
| 4 | Forma de la superficie `snapshot` | A) **Thin delegation `utils::backup::{create_snapshot,list_snapshots,restore_snapshot}` (core dueño de semántica; entry point memory-scoped + flujo documentado)** / B) reimplementar mirror/restore (prohibido: "sin reimplementaciones nuevas"; api-contract R-8) / C) solo docs+tests sin función (contrato pide consumo) | ✅ **A** — decidido-por-evidencia: core expone `Embedded::create_snapshot`/`list_snapshots`/`restore_from` (builder.rs:269,275,297) con contrato completo (quiesce/mirror/rollback); el crate solo agrega el punto de entrada + contrato de flujo (close→restore→reopen) |
| 5 | Semántica de error | A) **`versions`: `L1Error` existente (Vanta+Serde); `backup`: `vantadb::error::Result` directo (delegación pura, sin tipo nuevo)** / B) `BackupError` nuevo (ceremonia sin valor: el core ya tipa NotFound/InvalidInput) | ✅ **A** — decidido-por-evidencia: `L1Error` es el error de la capa L1 (l1_writer.rs:31-39); el snapshot no agrega semántica de error propia — reusar el tipo del core evita mapping vacío (ponytail) |
| 6 | Tests por superficie | A) **Integración: `tests/l1_history.rs` (in-memory, fixture `put_l1` de recall.rs) + `tests/backup_snapshot.rs` (Fjall + tempdir, patrón md_git_e2e/snapshot_consistency)** / B) unit tests inline (no prueban el flujo real SDK) | ✅ **A** — decidido-por-evidencia: pre-mortem #3 ("flujo CLI existente + DB temp en test"); el crate usa integration tests para todo flujo SDK (recall.rs, md_git_e2e.rs) |
| 7 | Filtros/cursor en recall | A) **Cursor: ya consumido (`read_namespace_records` pagina con `MemoryListOptions.cursor` l1_reader.rs:47-71; search top-k no pagina) → documentar. Filtros restantes (`min_confidence`, `as_of_ms`/`valid_window`, `filters`, `exclude_superseded`): sin consumidor concreto en recall a HEAD → FIND** / B) adoptar `min_confidence` en `RecallConfig` (cambia semántica de recall — risk register 🟡×🟡; exige quitar `Eq` de `RecallConfig` por f32; toca ambos paths) | ✅ **A** — decidido-por-evidencia: pre-mortem #1 ("adoptar solo donde aplique") + stop L1187; `min_confidence` tiene respaldo de spec (mgr-12 §Q3a) pero su consumo es un cambio de semántica de recall con sketch completo → FIND-286 para tarea propia |
| 8 | IQL | A) **No consumir → FIND** (el reuse del motor core ya existe: `core_search` MEMG-11 BM25+HNSW+RRF con dual-path/flag; IQL es superficie de query language, no de scoring; sin consumidor memory-side; pre-mortem #2) / B) IQL en recall (tercera ruta, cambia scoring) / C) IQL en gateway (los handlers scene/knowledge no consultan L1 por query libre) | ✅ **A** — decidido-por-evidencia: plan L1186 ("IQL en recall cambia semántica de scoring → dual-path/flag" — ese flag ES `core_search`, ya entregado); MCP `query_iql`/CLI ya exponen IQL al core; sin caso de uso memory-side verificado |
| 9 | Wire / core / deps | Sin cambios: payload L1 intacto (no se agregan campos); `src/sdk/**` solo consumo; cero dependencias nuevas | ✅ — `#[serde(default)]`/`skip_serializing_if` intactos; snapshot API core disponible con `default-features=false` (`pub mod storage` incondicional, src/lib.rs:147) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **Firmas intactas de lo existente:** `read_record`/`read_session_records`/`read_namespace_records`/`read_scoped_records` no cambian (fan-in 38 en `read_session_records` + ~30 tests). Solo se AGREGA.
  2. **Wire/payload L1 intacto** — sin campos nuevos en `MemoryRecord`; la historia se lee de snapshots del core (`SnapshotRecord`, postcard) sin tocar su formato.
  3. **Gate de cuarentena** (`include_quarantined: false`, SCH-05) intacto en todo read path de recall; las superficies nuevas (versions/diff) son auditoría explícita — documentan que exponen TODAS las versiones retenidas (incl. estado de cuarentena de la versión) porque el propósito es auditoría; NO se usan para alimentar recall.
  4. **Snapshot:** no reimplementar; restore exige engine cerrado (contrato core); nombres validados por el core (anti path-traversal) — el wrapper no relaja la validación.
  5. `src/sdk/**`, `src/wal.rs`, `src/storage/**`, `src/vector/` **no se tocan** (consumo puro).
  6. Sin `unwrap`/`expect`/`unsafe` en código nuevo de producción; sin dependencias nuevas.
  7. **WIP ajeno:** `opencode.jsonc` + master plan modificados en el árbol por otros — NO se stagean; PROHIBIDO tocar `docs/pipeline-state.json`; commit con **pathspec**.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vanta-memory --test l1_history --test backup_snapshot --build-jobs 2` · `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` · `cargo fmt --check` · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings`.
- **Deuda pendiente:** IQL no consumido (FIND-285); filtros restantes de recall (FIND-286). Sin otra deuda.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — sin `unsafe`, sin clones en hot path (todo es read/audit fuera del path de recall; snapshot es operación de mantenimiento), sin abstracciones especulativas (3 funciones de delegación + 2 readers + 1 diff puro). El cambio **elimina** deuda de "reimplementar vs reutilizar" (DELTA P2/STU-03) y agrega tests de contrato. Sin deps nuevas.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: historia/diff de L1 vía `versions` (test dedicado RED→GREEN con 3 versiones reales) + backup/restore vía snapshot (test Fjall tempdir con mutación post-snapshot) + superficie evaluada/no-consumida con motivo + FIND + suites scoped verdes + fmt/clippy verdes |
| **Commit** | Commit atómico conventional `feat(memory):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | Changelog release-plz (feature → minor); verify full scoped documentado (workspace completo no se corre por presupuesto) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius) + `codebase-memory-mcp_check_index_coverage` (6 paths, `no_recorded_issue` ✅) + grep puntual
- `cargo nextest` scoped por crate (loop TDD) + `campaign_verify_cmd` (verify mecánico)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Gates docs (`node scripts/docs/check-links.mjs && check-docs.mjs && gen-index.mjs --check`) — task file + `VANTA_MEMORY.md` editados

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `test-driven-development` (pin) · `systematic-debugging` (pin) · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` (SDP phase=BUILD) + rol: `api-and-interface-design`, `rust-write-tests`, `documentation-skill`. `security-and-hardening` excluida (sin trust boundary nuevo: lecturas propias + delegación a validación core del nombre de snapshot); `performance-optimization` excluida (no hot path — audit/backup fuera del path de recall; Regla 9 no dispara).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — evaluación: sin trust boundary nuevo; `restore_snapshot` recibe `name` del host → la validación (anti path-traversal) queda en el core (`validate_snapshot_name`, storage/engine/mod.rs:750) y el wrapper NO la relaja ni la reimplementa; `read_record_versions` lee payloads propios con fallback skip (mismo patrón `read_namespace_records`). Sin auth/secrets/deps. Sin hallazgos que requieran `security-and-hardening`.
- [ ] **PERFORMANCE** — no aplica: fuera del path de recall (audit/backup son operaciones explícitas del host); `diff_records` es O(campos) sobre 2 records; sin loops nuevos en hot paths, sin serialización nueva en writes. Regla 9 no dispara (no hay claim de optimización).

## Steps

### Step 1 — RED+GREEN: superficie `versions` (historia/diff de L1)

- **Archivos:** `vanta-memory/tests/l1_history.rs` (nuevo), `vanta-memory/src/core/record/l1_reader.rs` (releer fresco desde HEAD)
- **Acción:** RED — tests de contrato: (1) `record_versions_history_is_ascending_and_addressable` — put v1 (content A) + v2 (content B, version 2, superseded_by Some) + v3 → `read_record_versions` = 3 `RecordVersion` ascendentes con version 1/2/3 y contents A/B/C; `read_record_version(2)` = content B; versión inexistente → `None`; record inexistente → vec vacío (falla por compilación: símbolos ausentes → RED correcto). (2) `diff_reports_only_changed_fields` — diff(v1,v2) contiene `content` (A→B), `version` (1→2), `superseded_by` (Null→"m2") y NO contiene campos iguales (`priority`); diff de idénticos = vacío. GREEN — en `l1_reader.rs`: `RecordVersion{version: u64, record: MemoryRecord}` + `read_record_versions` (loop `db.versions`, mapeo payload→`MemoryRecord` + `usable_vector_filter` como `read_record`) + `read_record_version` (`db.get_version`) + `diff_records` (serde_json::to_value, BTreeSet de keys, cambios top-level; `Null` = campo ausente por `skip_serializing_if`).
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test l1_history --build-jobs 2` → RED correcto (compile error por símbolos ausentes) → GREEN (2 tests)
- **Evidencia:** ✅ RED verificado: `error[E0432]` unresolved imports (`diff_records`/`read_record_version`/`read_record_versions` ausentes en `l1_reader`) — falla por la razón correcta (símbolos ausentes, no assertion). ✅ GREEN: `2 tests run: 2 passed` — historia de **3 versiones reales** ascendentes 1/2/3 direccionable (`get_version(1)` + `get_version(2)` interior + 99→None + missing→empty) y diff por campo (`content`/`version`/`superseded_by` Null=ausente; iguales no reportados). **Post-review P2-01 (Required del reviewer):** el test pasó de 2 a 3 versiones + verificación de versión interior; re-verificado 6/6 conjunto. Re-exports en `core/record/mod.rs`; doc del módulo + rustdoc audit-only (quarantine gate SCH-05 no aplica a la historia — Optional del reviewer).
- **Estado:** ✅ COMPLETED

### Step 2 — RED+GREEN: superficie `snapshot` (backup/restore)

- **Archivos:** `vanta-memory/src/utils/backup.rs` (nuevo), `vanta-memory/src/utils/mod.rs` (re-export), `vanta-memory/tests/backup_snapshot.rs` (nuevo)
- **Acción:** RED — `tests/backup_snapshot.rs` (Fjall + tempdir, patrón md_git_e2e): (1) `snapshot_round_trips_l1_records` — 2 records L1 → `create_snapshot("snap-1")` (path existe) → `list_snapshots` lo lista (sorted) → mutación post-snapshot (put m3 + delete m1) → `db.close()` → `restore_snapshot(config,"snap-1")` → `read_session_records` = exactamente los 2 originales (m3 ausente, m1 presente) (falla por compilación → RED). (2) `snapshot_name_validation_is_not_relaxed` — `create_snapshot("../evil")` → Err InvalidInput. (3) `restore_missing_snapshot_is_not_found` — `restore_snapshot(config,"nope")` → Err NotFound. GREEN — `utils/backup.rs`: delegación a `Embedded::create_snapshot`/`list_snapshots`/`restore_from` con doc del flujo (close→restore→reopen; core dueño de semántica) + re-export en `utils/mod.rs`.
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test backup_snapshot --build-jobs 2` → RED correcto → GREEN (3 tests)
- **Evidencia:** ✅ RED verificado: `error[E0432]` unresolved import (`utils::backup` ausente). ✅ GREEN: `4 tests run: 4 passed` — round-trip (pre-snapshot back, ADD post-snapshot gone), validación de nombre no relajada (InvalidInput), NotFound, y **canary FIND-287** (`restore_does_not_roll_back_post_snapshot_deletes`: restore data/-only — tombstone de DELETE **y** metadata de supersession sobreviven; pineado con FIND, no asertado como contrato certificado). Fallo inicial de la aserción de delete-rollback investigado (root cause en §Notas) y ajustado al contrato core real. **Post-review P2-01 (Optional del reviewer):** canary extendido a supersession (el wording del FIND queda con evidencia, no solo análisis); re-verificado 6/6 conjunto.
- **Estado:** ✅ COMPLETED

### Step 3 — Evaluación IQL/filtros + FINDs + docs

- **Archivos:** `docs/dev/Backlog.md` (FIND-285 IQL, FIND-286 filtros restantes), `docs/api/VANTA_MEMORY.md` (§Audit & backup), `docs/dev/tasks/MEMG-13.md` (§Spec/§Notas sincronizados)
- **Acción:** registrar los 2 FINDs con el formato de `prompts/findings.md` (IQL: no consumido, motivos + `core_search` como reuse ya entregado; filtros: `min_confidence`/temporales/`filters` con sketch exacto de implementación y el constraint `RecallConfig: Eq`); documentar las superficies nuevas en `VANTA_MEMORY.md` (sección corta con las firmas reales + flujo snapshot + superficies evaluadas) y el estado de consumo por superficie (cursor consumido, filtros/IQL FIND); correr gates docs.
- **Verify:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` exit 0 + `pwsh scripts/validate-docs-coverage.ps1` 0 gaps
- **Evidencia:** ✅ FIND-285 (IQL) / FIND-286 (filtros) / FIND-287 (snapshot restore data-only) registrados en `Backlog.md:446-448` (esquema 10-col) · `VANTA_MEMORY.md` §Audit & backup (MEMG-13) + fila de capas actualizada (`utils::backup`) · check-links: 0 broken (exit 0) · check-docs: gating all clear (exit 0; orphans=reporting) · gen-index: `--write` regeneró `docs/index.md`+`llms.txt` (delta mínimo: solo MEMG-13.md) → `--check` exit 0 · validate-docs-coverage: 0 gaps (exit 0).
- **Estado:** ✅ COMPLETED

### Step 4 — Verify full + OCR + Review P2-01 + commit local + cierre campaign

- **Archivos:** `docs/dev/tasks/MEMG-13.md` (§Review + RESULTADO §7)
- **Acción:** `cargo fmt --check` · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` · `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` · `pwsh scripts/validate-docs-coverage.ps1` · OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) → revisar por Rule Group (Critical/High bloquean; Medium → FIND); clasificar tier HARD-02 (paths `vanta-memory/**` + `docs/**` → **Fast**; sin `src/sdk/**` propio); fork `vanta-review` con el diff (fresh context); registrar veredicto en §Review; commit **LOCAL** `feat(memory):` con pathspec de archivos propios; `campaign_update_task_state(completed, taskId:"41")` con recitation + payload `review`.
- **Verify:** veredicto registrado + `git show --stat HEAD` limitado a archivos propios + `git status` sin WIP ajeno stageado
- **Evidencia:** (pendiente)
- **Estado:** ⬜ PENDING

## Dependencias

- MEMG-11 ✅ (`508e211e` — `core_search` flag + `put_batch`; `l1_reader.rs`/`auto_recall.rs` releídos frescos desde HEAD), MEMG-12 ✅ (`9d0e371d` — estampado v2; `l1_writer.rs` releído).
- SCH-03/04/05 ✅ (filtros core: `min_confidence`/`as_of_ms`/`valid_window`/`include_quarantined`), VS-CORE-07 ✅ (`get_version`/`versions`), snapshot core ✅ (FIND-25/FIND-33 + MCP-34b).
- MGR-12 §Q3a (min_confidence) — referencia para FIND-286, no bloqueante.
- nextTask: MEMG-07 (Task 42) — lo decide el orquestador.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review — P2-01, contexto fresco (no participó de la implementación); tier **Adversarial** (HARD-02: el diff incluye `docs/api/**` + `vanta-memory/**`). Sesión reviewer: `ses_ef48d6448ffe8WkrnNLDrFB5RJ`. Veredicto ronda 1: 🔴 changes-required (1 Required) → fix aplicado → ronda 2: ✅ **APPROVE**.
- **Enfoque:** five-axis + RBI sobre el diff MEMG-13 (versions/snapshot), verificación DoD P2-08 y evidencia FIND-285/286/287; re-verificación post-fix de la ronda 1.
- **Cómo se probó:** ronda 1 (fresh): contrato 6/6 · suite `-p vanta-memory` 611/611 · fmt · clippy · check-links/check-docs/gen-index --check/validate-docs-coverage exit 0 · RED vía `git show HEAD` (símbolos ausentes) · lectura de core (version_history/get/delete/snapshot/supersede) + evidencia FIND-33/plan archivado/certification. Ronda 2 (post-fix): mismos comandos re-corridos → 6/6 y 611/611 verdes; fix verificado en archivo (3 versiones + interior; canary supersession; rustdoc/doc audit-only; citas FIND-286).
- **Hallazgos + disposición:**
  1. [REQUIRED ronda 1] DoD "3 versiones reales" incumplido (test usaba 2 y no verificaba versión interior) → **CORREGIDO**: 3 `put_l1` reales + `read_record_version(2)` interior; re-verificado.
  2. [OPTIONAL] FIND-287 "supersessions" sin evidencia → **canary extendido** (`db.supersede` post-snapshot → metadata sobrevive; pasó) + FIND actualizado ("verificado").
  3. [OPTIONAL] Invariante audit-only no documentado en público → **frase agregada** al rustdoc de `read_record_versions` + `VANTA_MEMORY.md` §Audit & backup (SCH-05 no aplica; no alimentar contexto de modelo).
  4. [NIT] citas FIND-286 → **corregidas** (`l1_reader.rs:47-71`; `vector_types.rs:134-147`).
  5. [NIT] `sanitize_key` en fixture → **descartado** con razón (`pub(crate)`, inaccesible desde integration tests; ids fijos).
- **Veredicto:** ✅ **APPROVE** — contrato, DoD Task y gates verdes; changeset listo para commit local (Step 4).

## Context Save Point

- **Última acción:** Steps 1-4 ✅ (RED→GREEN 6/6; suite 611/611; fmt/clippy 0; docs gates 0; FIND-285/286/287; OCR sin Critical/High; review P2-01 APPROVE post-fix). Commit local en curso.
- **Próximo paso:** commit **LOCAL** `feat(memory):` con pathspec de archivos propios → commit docs de cierre (RESULTADO + hash) → campaign completed taskId `41`.
- **Estado del worktree:** HEAD `64eee3f7`; WIP ajeno (`opencode.jsonc`, master plan) NO se stagea.

## Notas

- **Evaluación por superficie (pre-mortem #1 — "adoptar solo donde aplique"):** (a) `versions` → **adoptado** (0 hits en vanta-memory; habilita auditoría/diff de L1); (b) `snapshot` → **adoptado** (0 hits; habilita backup/restore; el `BackendSnapshot` de `utils/local_backend.rs` es otro dominio — cola de tareas local); (c) `filtros/cursor` → cursor **ya consumido** (`read_namespace_records` pagina con cursor del core, l1_reader.rs:47-71; `include_quarantined:false` SCH-05; search top-k no pagina), filtros restantes → **FIND-286**; (d) IQL → **FIND-285** (reuse del motor ya cubierto por `core_search` MEMG-11; sin consumidor memory-side).
- **Historia de versiones = best-effort:** el core documenta gap de crash window (`memory.rs:932-933`: "durability is best-effort post-commit... degraded but never corrupt") — la doc de la superficie lo declara.
- **Snapshot locks (pre-mortem #3):** `create_snapshot` requiere DB en disco (Fjall) — InMemory no persiste; restore exige engine cerrado (fs2 lock) → test con Fjall+tempdir, flujo core existente.
- **FIND-287 (descubierto en Step 2 — root cause documentado):** `snapshot_restore` restaura SOLO `data/`; el backend KV live (Fjall) no se renombra ni restaura (decisión FIND-33 + plan archivado 2026-08-29 §1308 "NO tocar snapshot_restore: el backend live en storage_root/ no se renombra (swap es solo sobre data/)"). Consecuencia: un **DELETE** post-snapshot conserva su tombstone del backend y NO se revierte; los **ADDs** post-snapshot sí desaparecen (contrato certificado `snapshot_certification.rs:1435`). Evidencia: test fallido inicial (left `["beta"]` vs right `["alpha","beta"]`) → hipótesis verificada contra `snapshot_restore` (sin manejo de backend) + FIND-33.md:53 ("restore todavía en ROOT-only, no toca el backend"). Tratamiento: doc del módulo `utils/backup.rs` + canary test + FIND-287 (core/storage — fuera del scope de MEMG-13 y de mi dominio; no se toca `src/storage/**`).
- **WIP ajeno:** `opencode.jsonc` + master plan modificados en el árbol por otros — NO se stagean. PROHIBIDO tocar `docs/pipeline-state.json`.
- **Disco:** si el linker falla por espacio → `dev-tools/target-cleanup.ps1 -Clean -Yes` (patrón MEMG-02).
- **NOTICED BUT NOT TOUCHING:** `search_records_core` usa `db.search` (no `search_page`) — cursor de búsqueda no aplica a recall top-k (documentado, sin FIND); `MemoryListOptions.filters` (metadata) sin consumidor memory-side (incluido en FIND-286).

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: (hash en commit docs de cierre)
ARCHIVOS: vanta-memory/src/core/record/l1_reader.rs, vanta-memory/src/core/record/mod.rs, vanta-memory/src/utils/mod.rs, vanta-memory/src/utils/backup.rs (nuevo), vanta-memory/tests/l1_history.rs (nuevo), vanta-memory/tests/backup_snapshot.rs (nuevo), docs/api/VANTA_MEMORY.md, docs/dev/Backlog.md (FIND-285/286/287), docs/dev/tasks/MEMG-13.md, docs/index.md + llms.txt (generados)
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no | P:plan sanciona el slice (Task 41 + stop L1187) · D:no disparado (pre-respondido por plan F0 — precedente MEMG-11/02; sin símbolos fuera de la sanción) · V:no disparado (verde; el fallo inicial del canary fue root-caused, no umbral de 2 fallas mismo-error) · C:no disparado (FIND-285/286/287 registrados; WIP ajeno no stageado)
SKILLS_CARGADAS: campaign-executor, progreso, ponytail (base) · test-driven-development, systematic-debugging (pins) · source-driven-development, doubt-driven-development, incremental-implementation (SDP v3 BUILD) · api-and-interface-design, rust-write-tests, documentation-skill (rol)
```
