---
title: "SCH-06: Tests — migración determinista, time-travel, roundtrip export/import y chaos (slice 0.8.0)"
kind: task
description: Matriz de cierre — doble corrida byte-idéntica, AS OF valid-time con fechas de referencia, roundtrip v1↔v2 y recuperación tras crash mid-migración
---

# SCH-06: Tests — migración determinista, time-travel, roundtrip export/import y chaos (slice 0.8.0)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` — Task 31 (F3) · **Origen:** plan L813-837; [ADR-0046](../architecture/adr/ADR-0046-schema-v2-migracion-unica.md) (§D3, §D4d, §D5, §D7, §Migration); [MGR-13](../research/mgr-13-cuarentena.md) §3/§7; [MGR-10](../research/mgr-10-bitemporalidad.md) §4
- **Fuente del prompt:** sub-agente vanta-chaos (orquestador pipeline) — wave F3.4 (única en vuelo); branch `develop`
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴 · **Tipo:** tests/chaos (suite de certificación)
- **Creado:** 2026-09-29 · **last-synced:** 2026-09-29
- **Estado:** ⏳ IN PROGRESS — implementación + verify mecánico ✅ (7/7 steps); pendiente **review P2-01 + commit (LEAD)**
- **Incógnitas (uphill):** 0 (resueltas en DISCOVERY/probe) · **Pendientes (downhill):** 0 steps de ejecución

## Contrato (verbatim del prompt de tarea)
> "suite verde: migración v1→v2 determinista (misma DB → mismo resultado; doble corrida byte-idéntica sobre copia) Y time-travel `AS OF` con fechas de referencia (eje valid/transaction según ADR SCH-01) Y roundtrip export/import v1↔v2 (v1 sigue importable; campos nuevos sobreviven ida y vuelta) Y chaos: crash durante migración → recuperación íntegra (failpoint + reopen + integridad de índices) Y bordes (TTL+quarantine, supersede+invalid) — todo corriendo en CI vía chaos.yml + nextest scoped"

**Cláusulas a verificar (matriz de cierre):**

| # | Cláusula | Superficie | Evidencia esperada |
|---|----------|-----------|--------------------|
| C1 | Migración v1→v2 determinista: misma DB ⇒ mismo resultado; doble corrida sobre copia byte-idéntica | `src/migration.rs` (backfill puro) vía `tests/schema_v2_migration.rs` | `migration_v1_to_v2_is_byte_identical_across_copies` (+ dry-run no escribe, + idempotencia) |
| C2 | Time-travel `AS OF` con fechas de referencia; eje **valid** (ADR-0046 §D3) + eje transaction por key | `MemorySearchRequest.as_of_ms/valid_window`, `MemoryListOptions`, IQL `AS OF`, `versions()` | `tests/time_travel.rs` (boundaries inclusive/exclusive, reopen, IQL≡SDK, eje transaction separado) |
| C3 | Roundtrip export/import v1↔v2: v1 sigue importable; campos nuevos sobreviven ida y vuelta | `record_from_export_line` (≤2 / >2), `export_all`/`import_file` | `tests/memory_export_import.rs` (fixture v1 → normalización → re-export v2; wire v2 full-field roundtrip) |
| C4 | Chaos: crash durante migración ⇒ recuperación íntegra (failpoint + reopen + integridad de índices) | `MigrationEngine::migrate_records` + failpoints existentes (`storage_insert_fail`, `wal_append_fail`) | `tests/chaos_migration.rs` (abort mid-batches ⇒ header v1 intacto, 0 pérdida, re-run completa, lectura/índices OK) |
| C5 | Bordes: TTL+quarantine, supersede+invalid (I3 ortogonalidad) | `purge_expired`, `supersede`, default-exclude | `tests/quarantine_containment.rs` (+2 tests) |
| C6 | Todo corriendo en CI: chaos vía `chaos.yml` + perfil `chaos`; suite scoped nextest | `.config/nextest.toml`, `.github/workflows/chaos.yml`, `Cargo.toml` | Wiring verificado localmente con el comando exacto de chaos.yml; exclusiones del perfil `default` para el binario failpoints-only |

**Alcance explícito:** suite de tests (3 binarios nuevos + 3 extensiones) + wiring CI/perfil + corpus fuzz. **PROHIBIDO:** producción `src/**` (los failpoints existentes cubren el write path de la migración — F2 resuelto sin tocar el engine), `docs/**` salvo este task file, `Backlog.md`, `perf-bench.yml`, `opencode.jsonc`, plan file (LEAD).

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| **Crea** | `tests/schema_v2_migration.rs` · `tests/time_travel.rs` · `tests/chaos_migration.rs` · `fuzz/corpus/fuzz_parser/seed_as_of*` (3 seeds) |
| **Extiende** | `tests/memory_export_import.rs` (roundtrip v1↔v2 full-field) · `tests/quarantine_containment.rs` (bordes TTL+quarantine, supersede+invalid) · `.config/nextest.toml` (default-filter + perfil chaos) · `Cargo.toml` (`[[test]]` ×3, `required-features=["failpoints"]` para chaos_migration) · `.github/workflows/chaos.yml` (solo si el wiring del perfil no alcanza) |
| **Callers** | Ninguno en producción — archivos de test consumen superficie pública: `vantadb::{Embedded, MemoryInput, MemoryRecord, MemoryListOptions, MemorySearchRequest, ValidWindow, MigrationEngine?}` (verificado: `pub mod migration`, `pub mod sdk`) |
| **Callees** | `MigrationEngine::migrate_records`/`plan_all` · `StorageHeader` · `StorageEngine::{open_with_config, get, insert, flush}` · `Embedded::{put, get, list, search, search_page, supersede, versions, purge_expired, quarantine_apply/promote/reject, export_all, export_namespace, import_file}` · `fail::{cfg, remove}` (feature `failpoints`) |
| **Implicaciones** | Cero cambios de producción; cero cambios de wire. El runtime de tests crece (~3 binarios scoped; 1 failpoints-only). El perfil `default`/`audit` gana 2 binarios livianos (migración con fixture chico) y sigue excluyendo el de failpoints (como `chaos_integrity`/`crash_injection`). `chaos.yml` gana los tests nuevos vía filtro del perfil `chaos` (no por step nuevo). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/migration.rs` (1164L) · `src/cli_handlers/migrate.rs` (538L) · `.config/nextest.toml` (107L) · `.github/workflows/chaos.yml` (54L) · `tests/common/mod.rs` (679L) · `tests/memory_export_import.rs` (390L) · `tests/schema_evolution.rs` (137L) · `tests/durability_recovery.rs` (:330-449 incl. failpoint `snapshot_serialize_fail`) · `tests/storage/crash_injection.rs` (:1-120) · `fuzz/fuzz_targets/fuzz_parser.rs` (23L) · `tests/quarantine_containment.rs` (:1-304, :235-304, :536-660) · `tests/cli_tests.rs` (:1440-1649, `seed_v1_database` + `cmd_migrate`) · `docs/dev/tasks/SCH-05.md` (plantilla)
- **Archivos leídos (rangos clave):** `src/sdk/types/record.rs` (:40-399 — `ValidWindow`, `MemoryInput`, `MemoryRecord`, `MemoryListOptions`) · `src/sdk/search/page.rs` (:120-250 fingerprint) · `src/sdk/search/tests.rs` (:2295-2514 — precedente semántica AS OF/valid_window) · `src/sdk/api.rs` (:1000-1089 — list AS OF) · `src/sdk/api/memory.rs` (:975-1157 — ops cuarentena + purge) · `src/sdk/version_history.rs` (:20-79 key format; :294 `reencode_snapshot_v2`) · `src/sdk/serialization/impl_export.rs` (:290-409 — import con `quarantine`) · `src/storage/engine/insert.rs` (:150-225 insert failpoint; :395-484 batch prelude failpoint) · `src/storage/ops.rs` (:60-226 vstore write) · `src/storage/engine/init.rs` (:15-84 open) · `src/schema.rs` (:1-200 header/compat) · `src/executor.rs` (:205-300 filtro valid-time; :690-829 tests IQL AS OF) · `src/lib.rs` (:57-225 exports) · `Cargo.toml` (:122-168 features; :235-260 dev-deps; :350-676 bins/tests)
- **Referencias hacia dentro:** ADR-0046 §D3 (predicado `[valid, invalid)`, AS OF = valid; transaction por key = `versions`), §D4d (tests V4 `backfill_is_deterministic` → SCH-06 doble corrida), §D5/D5d (deadline 30d, I1-I3), §D7 (import ≤2/>2, normalización v1), §Migration (expand→backfill→bump; orden invariable); MGR-10 §4 (backfill función pura sin reloj); MGR-13 §7-4 (bordes/chaos SCH-06: `cargo nextest run --profile audit -p vantadb --test quarantine_containment`); SCH-03 (commits `932b1211` — semántica AS OF fijada).
- **Referencias entrantes:** SCH-07 (superficies consumen la suite verde como precondición), SCH-08 (corte 0.8.0 cita esta evidencia de determinismo). El gate F3 del plan (`:35`) = "migración determinista verde" → esta suite ES el gate.
- **Veredicto impacto:** bajo en producción (cero cambios), medio en CI (nuevos binarios + filtro del perfil `chaos`). Riesgo concentrado en la fragilidad del determinismo byte-exacto (ver §Spec decisión 1) y en el costo CI del fixture con 2 batches (1.1k nodos — acotado).

## Spec (decisiones por evidencia — contrato verbatim + ADR-0046)

| # | Decisión | Alternativas | Elegido | Evidencia |
|---|----------|--------------|---------|-----------|
| 1 | Scope del "byte-idéntica sobre copia" | (a) directorio completo incluyendo WAL/journal internos · (b) artefactos de datos de la migración (export JSONL byte-exacto + `vector_store.vanta` + `.vanta.schema`) | (b), con probe empírico documentado | El write path del engine sella `last_accessed = now_ms` en el WAL (`insert.rs:211`, `get.rs:190`) — reloj del engine, no output del backfill (`migration.rs:563-582` es función pura). **Probe (Step 1):** los ÚNICOS archivos volátiles son `data/vanta.shard{0..3}.wal` + `data/vanta.wal.shards`; vstore, `keyspaces/**`, header y `index.bin` resultaron **byte-idénticos entre copias** (el filtro del test los compara estricto). |
| 2 | Fixture v1 | (a) nodos armados a mano (helper `seed_v1_database`, precedente `cli_tests:1526`) · (b) DB v2 "degradada" | (a) | Precedente directo + fiel a `is_memory_record_node` (namespace/key/payload); los 4 campos v2 se omiten por construcción; `__vanta_superseded_by`/`_at_ms` por literal (no exportados, precedente `memory_api.rs:624`) |
| 3 | Failpoint del chaos (F2 pre-mortem) | (a) instrumentar `migrate_records` con `fail` + abort-antes-de-swap · (b) reusar failpoints del write path (`storage_insert_fail` en `batch_prelude`, `wal_append_fail`) | (b) | (a) exige tocar `src/` (stop condition: engine >1d); (b) da crash mid-migración real (2º chunk falla con `1*off->return` — sintaxis verificada en docs.rs/fail 0.5.1) sin tocar producción, y el "abort-before-swap" ya es estructural (el bump del header vive en el CLI, después del backfill) |
| 4 | Semántica de fechas de referencia en time-travel | (a) `now()` + sleeps · (b) fechas sintéticas fijas + timestamps del propio registro | (b) | ADR §D3 predicate; unit tests SCH-03 usan T fijos (`search/tests.rs:2303`); `supersede()` fija `invalid_at := now` → los boundaries de fin se anclan al timestamp del record (`superseded_at_ms`), nunca a sleeps |
| 5 | Eje transaction en la suite | (a) `AS OF` transaction cross-key (no existe, v1.0) · (b) `versions()`/`get_version` por key | (b) | ADR §D3-4 verbatim: "el time-travel de transaction en 0.8.0 es por key"; test de ortogonalidad valid vs transaction |
| 6 | Home de los bordes (TTL+quarantine, supersede+invalid) | (a) archivo nuevo · (b) extender `tests/quarantine_containment.rs` | (b) | MGR-13 §7-4 lista esos bordes con la suite de contención; reuso de helpers `listed_keys`/`quarantined_input`/`in_memory_db` |
| 7 | Fuzz de `AS OF` | (a) target nuevo · (b) seeds de corpus | (b) | Stop condition: "fuzz targets nuevos de migración → NO". `fuzz_parser` ya llama `parse_query`/`parse_statement` (AS OF incluido); 2 seeds dirigen el corpus al grammar nuevo |
| 8 | Wiring CI | (a) step nuevo en `chaos.yml` · (b) extender `default-filter` del perfil `chaos` | (b) | chaos.yml ya corre `--profile chaos --features failpoints -p vantadb`; el filtro es el único punto a tocar (y el perfil `default` excluye el binario failpoints-only, patrón `chaos_integrity`/`crash_injection`) |

## Invariantes de dominio (handoff — MUST)
- **Cero cambios de producción:** `git diff --name-only` no debe incluir `src/**` ni `vanta-memory/**` ni bindings. Los failpoints existentes son la instrumentación.
- **Determinismo del backfill:** función pura de valores almacenados, sin reloj/aleatoriedad/orden (ADR §Migration; `migration.rs:147-234`). Los tests no promueven reloj: no `sleep`, no `now()` en assertions.
- **Predicado D3 verbatim:** `valid_at <= T < invalid_at` (inclusive start / exclusive end); `AS OF` = eje valid; `versions()` = eje transaction.
- **I1/I2/I3 (MGR-13):** nunca auto-promoción por TTL/deadline; sticky; ortogonalidad TTL/supersede/quarantine.
- **D7:** import acepta `schema_version ∈ {1,2}`, rechaza `>2`; export siempre emite 2; v1 se normaliza.
- **PROHIBIDOS:** `docs/**` (salvo este task file), `Backlog.md`, `perf-bench.yml`, `opencode.jsonc`, plan file (LEAD). No commit, no self-review (LEAD).
- Regla dura `-p` + `CARGO_BUILD_JOBS=2`; nunca nextest sin `-p` desde raíz.

## Deuda técnica (Regla 6 — MUST)
**Saldo neto: 0.** Cero `unsafe` nuevo, cero dependencias nuevas, cero hot path. Deuda diferida citada: failpoint dedicado en `migrate_records` (abort-antes-de-swap nativo) → FIND si los failpoints del write path dejan un hueco no cubierto; `AS OF` transaction cross-key → v1.0 (ADR §D1d); mirror `SnapshotRecord` v1→v2 a nivel integración (requiere acceso a particiones, no público) queda en cobertura unitaria de SCH-02 → nota en task file.

## Definition of Done
- **Task:** contrato ✅ (matriz C1-C6) + fmt/clippy/nextest verdes + tests del cambio + wiring CI verificado con el comando exacto de chaos.yml.
- **Commit:** N/A — **LEAD** (el prompt de tarea prohíbe commit/self-review).
- **Release:** N/A (wave de tests; corte lo hace SCH-08/release-plz).

## Steps

### Step 1: RED/GREEN — `tests/schema_v2_migration.rs` (C1)
- **Archivos:** `tests/schema_v2_migration.rs` (nuevo), `Cargo.toml`
- **PLAN:** fixture v1 (32 records: plain + superseded) → copia ×2 → `migrate_records()` + bump en ambas → compare byte-scope (probe primero: listar archivos que difieren para fijar el scope con evidencia) → dry-run no escribe → idempotencia.
- **Estado:** ✅ **4/4** — byte-identidad entre copias (volátiles = shards WAL, documentado), valores de referencia ADR por record, dry-run (sin backfill + header/vstore intactos) + re-run skip, normalización D7 en lectura v1. RED capturó 2 fallos reales en el primer run: (a) `assert_eq!` de mapas completos volcaba bytes (reemplazado por diff conciso), (b) `Embedded::get` sobre fixture raw no usa el namespace index — el boundary correcto para D7 es `vantadb::sdk::record_from_node` (index rebuild es de recovery, no de lecturas crudas).

### Step 2: GREEN — `tests/time_travel.rs` (C2)
- **Archivos:** `tests/time_travel.rs` (nuevo), `Cargo.toml`
- **PLAN:** boundaries con fechas de referencia (search + list), valid_window overlap, error de ventana vacía, reopen, IQL `AS OF` ≡ SDK, eje transaction (`versions`) separado del valid.
- **Estado:** ✅ **4/4** — inclusive start/exclusive end (T=end exacto excluye; sucesor dueño del instante), overlap `[from,to)`, ventana vacía/invertida rechazada en search y list, reopen + IQL `SELECT * FROM tt AS OF 500` ≡ SDK, eje transaction (`versions`/`get_version`) ortogonal al valid (valid_at=500 vs created_at real).

### Step 3: GREEN — roundtrip v1↔v2 (C3)
- **Archivos:** `tests/memory_export_import.rs` (extender)
- **PLAN:** fixture `tests/fixtures/export-v1.jsonl` → import → export v2 → re-import → igualdad; wire v2 full-field (todos los campos nuevos) → export → import → equality de `MemoryRecord` completo.
- **Estado:** ✅ **10/10** — fixture v1 importa (2/2), re-export siempre `schema_version=2` con normalización materializada (`invalid_at=1500`, `confidence=1.0`), re-import lossless; wire v2 con los 10 campos nuevos + `derived` (clase+score+padres) sobrevive **dos** ciclos export→import con igualdad de struct completo.

### Step 4: CHAOS — `tests/chaos_migration.rs` (C4)
- **Archivos:** `tests/chaos_migration.rs` (nuevo), `Cargo.toml` (`required-features=["failpoints"]`)
- **PLAN:** v1 fixture 1.1k records (2 chunks) → `storage_insert_fail` `1*off->return` → migración falla mid-B2 → header v1 intacto + 0 pérdida + 1000 backfilled → reopen/lecturas/índices OK → re-run completa → bump → todo v2. Segundo test con `wal_append_fail`.
- **Estado:** ✅ **1/1 (3 escenarios)** — S1 chunk 2 abortado: header v1, 0 pérdida, 1000 backfilled, re-run completa, `rebuild_index` + list 1100 + text search + `audit_text_index` ✅. S2 `wal_append_fail` primer grupo: WAL coherente, reopen OK, re-run completa. S3 (witness) falla **entre** grupos de shard ⇒ guard ERR-011 **rechaza** la recuperación con mensaje explícito (fail-loud, salvage FIND-109) — ver §Hallazgos.

### Step 5: BORDES — extender `tests/quarantine_containment.rs` (C5)
- **Archivos:** `tests/quarantine_containment.rs`
- **PLAN:** TTL+quarantine (purge físico sin promoción, poll acotado por deadline) y supersede+invalid (I3: ambos ejes independientes; `invalid_at == superseded_at`).
- **Estado:** ✅ **27/27** — TTL vivo no purga + aislado; TTL vencido purga físicamente sin promover ni dejar fantasma; supersede preserva cuarentena y alinea `invalid_at`; AS OF del eje valid respeta el boundary aún cuarentenado (opt-in); promote no limpia supersede.

### Step 6: Wiring CI/perfil + fuzz seeds (C6)
- **Archivos:** `.config/nextest.toml`, `Cargo.toml`, `fuzz/corpus/fuzz_parser/seed_as_of{,_where,_duplicate}`, `.github/workflows/chaos.yml` (sin cambios — no hizo falta)
- **PLAN:** `default-filter` excluye `chaos_migration`; perfil `chaos` lo incluye; verificación local con el comando exacto de chaos.yml.
- **Estado:** ✅ — `nextest list --profile audit -p vantadb` lista los 8 tests nuevos (schema_v2_migration×4 + time_travel×4) y **excluye** `chaos_migration`; `nextest list --profile audit --features failpoints` tampoco lo lista (filtro OK); **comando CI exacto** `cargo nextest run --profile chaos --features failpoints -p vantadb` → **2/2** (chaos_integrity + chaos_migration) en 21s. chaos.yml no requirió edición (el perfil es el único punto). Seeds AS OF en el corpus del parser (target sin cambios: ya llama ambos parsers).

### Step 7: Verify full + cierre
- **PLAN:** `nextest --profile audit -p vantadb` (nuevos binarios incluidos) + `--features failpoints` scoped + chaos profile + fmt + clippy + validate-docs-coverage + OCR.
- **Estado:** ✅ mecánico (ver §Verificación final) — **pendiente review P2-01 + commit (LEAD)**.

## Progreso

| Step | Estado | Evidencia |
|------|--------|-----------|
| 1 schema_v2_migration | ✅ | `schema_v2_migration` 4/4 (`nextest --profile audit --test schema_v2_migration`) |
| 2 time_travel | ✅ | `time_travel` 4/4 |
| 3 roundtrip v1↔v2 | ✅ | `memory_export_import` 10/10 (8 previos + 2 nuevos) |
| 4 chaos_migration | ✅ | `chaos_migration` 1/1 (3 escenarios) con `--features failpoints` |
| 5 bordes quarantine | ✅ | `quarantine_containment` 27/27 (24 previos + 3 nuevos) |
| 6 wiring CI + seeds | ✅ | `nextest --profile chaos --features failpoints -p vantadb` → 2/2; list audit excluye chaos_migration; seeds AS OF |
| 7 verify full | ✅ | fmt 0 diffs · clippy workspace all-targets all-features `-D warnings` exit 0 · core audit **2475/2475** (2 skipped, 275s) |

## Verificación final (evidencia mecánica)

| Comando | Resultado |
|---|---|
| `cargo nextest run --profile audit -p vantadb --test schema_v2_migration` | ✅ 4/4 (3.9s) |
| `cargo nextest run --profile audit -p vantadb --test time_travel` | ✅ 4/4 (2.8s) |
| `cargo nextest run --features failpoints -p vantadb --test chaos_migration` | ✅ 1/1 (42s pre-refactor) |
| `cargo nextest run --profile chaos --features failpoints -p vantadb` (comando CI exacto) | ✅ **2/2** (chaos_integrity + chaos_migration, 17s; sin SLOW) |
| `cargo nextest run --profile audit -p vantadb --build-jobs 2 --no-fail-fast` | ✅ **2475/2475** (2 skipped; 275s) — incluye los 8 tests nuevos |
| `cargo nextest run --profile audit -p vantadb --test memory_export_import --ignore-default-filter` | ✅ 10/10 |
| `cargo nextest run --profile audit -p vantadb --test quarantine_containment` | ✅ 27/27 |
| `cargo fmt --all -- --check` | ✅ 0 diffs |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ✅ exit 0 |
| `pwsh scripts/validate-docs-coverage.ps1` | ⚠️ exit 1 — **2 gaps heredados de SCH-05** (`confidence_threshold`, `quarantine_review_default_days` en `docs/api/CONFIGURATION.md`; diferidos a SCH-07 por WIP ajeno). Cero gaps nuevos: el diff no toca `src/**`. |
| OCR advisory (`dev-tools/ocr-review.ps1 -Format json`) | ✅ spec generado para los 5 archivos de test; pase cognitivo acotado: sin Critical/High (unwrap/expect solo en tests con allow explícito; sin unsafe/FFI/concurrencia; fixtures inertes; poll TTL acotado por deadline) |

**Contrato (matriz):** C1 ✅ · C2 ✅ · C3 ✅ · C4 ✅ · C5 ✅ · C6 ✅.

## Hallazgos (chaos)

| # | Severidad | Hallazgo | Repro | Estado |
|---|-----------|----------|-------|--------|
| H1 | 🟡 FIND candidato (storage/WAL, pre-existente — no del diff) | Un fallo transitorio (o crash real) **entre grupos de shard** de un `batch_append` deja los counts del WAL shardeado incoherentes; el guard ERR-011 (`src/wal_sharded.rs:69-94`) **rechaza la recuperación completa** ("aborting recovery instead of silently dropping data") hasta correr el salvage opt-in (`vanta-cli wal salvage`, FIND-109). Fail-loud, nunca drop silencioso; pero "reabrir tras crash mid-batch" requiere salvage. Aplica a cualquier batch write, no solo a la migración. | `tests/chaos_migration.rs` escenario 3: fixture v1 chico + `fail::cfg("wal_append_fail", "1*off->return")` + `migrate_records()` ⇒ `open_engine` devuelve `Wal(...)` con `"WAL shard 1 is truncated: ... aborting recovery"`. | Witness permanente en el test; **FIND para LEAD** (dueño sugerido: storage/WAL; decidir auto-salvage vs mensaje accionable) |
| H2 | 🟢 Nota de transparencia | `tests/common/mod.rs:278,330` (`TestMetric.schema_version: 1`) es el **schema del reporte de certificación**, no el schema de storage — no debía bumpearse con v2 (el plan lo listaba como "gap" por homonimia). No se tocó. | Lectura directa del harness | Documentado; sin acción |
| H3 | 🟢 Nota de cobertura | `snapshot_certification.rs`/`sdk_serialization.rs`/`wal_rollback.rs`/`crash_injection.rs`/`schema_evolution.rs` ya cubren sus invariantes v2 (SCH-02 los actualizó: `schema_version=2` en el snapshot de export) o no aplican al contrato SCH-06; el mirror `SnapshotRecord` v1→v2 a nivel **integración** requiere acceso a `BackendPartition` (pub(crate)) — cobertura unitaria queda en `src/migration.rs` (`test_records_backfill_rewrites_v1_snapshots`). No se extendieron para no duplicar cobertura. | grep de suites + lectura | Documentado |

## Pendientes (§Pendientes)
- **gen-index:** `node scripts/docs/gen-index.mjs --check` falla (`docs/index.md` + `llms.txt` stale; la entrada de este task file aún no está en el índice). **No se regeneró** porque `docs/**` está prohibido en esta task → **LEAD:** `node scripts/docs/gen-index.mjs --write` y commitear los 2 generados. `check-links` ✅ (exit 0, deuda ajena en presupuesto) y `check-docs` ✅ (exit 0).
- **H1 (ERR-011/salvage):** proponer fila FIND en `docs/dev/Backlog.md` (LEAD; el task no puede tocar Backlog).
- **docs/api (2 gaps heredados de SCH-05):** `confidence_threshold`, `quarantine_review_default_days` → SCH-07.
- **Failpoint dedicado en `migrate_records`** (abort-antes-de-swap nativo): no fue necesario (los failpoints del write path cubren el contrato); si storage/WAL decide instrumentar, este suite es el consumidor natural.

## Dependencias
- **Consume:** SCH-02 ✅ (`7af34366`; backfill determinista + normalización) · SCH-03 ✅ (`932b1211`; AS OF valid-time + fingerprint) · SCH-04 ✅ (`932b1211`; `min_confidence`) · SCH-05 ✅ (`83d65518`; cuarentena/abstención/ops) · ADR-0046 `accepted` ✅ · MGR-10/MGR-13 ✅.
- **Bloquea:** SCH-07 (superficies + docs; consume esta suite como precondición). **nextTask:** SCH-07.

## Herramientas
- nextest scoped `-p vantadb` + profiles `audit`/`chaos` (timeout 900; flake HNSW → aislado) · `CARGO_BUILD_JOBS=2` · codegraph antes de grep · docs.rs/fail (sintaxis de acciones contadas, verificado por webfetch) · `dev-tools/verify_changed.ps1` (rápido) / verify full en cierre.
- **SDP (v3, BUILD):** base `campaign-executor`+`progreso` (auto) · pins `test-driven-development` · `systematic-debugging` · `rust-write-tests` (pin de rol chaos) · `deprecation-and-migration` (pin storage/schema, SDP) · `doubt-driven-development` · `documentation-skill` (task file). 

## Review (GATE — agente distinto, P2-01; tier **fast** por paths del diff: tests/CI/config)
> **Revisor:** `ses_f1307bf0bffeYyAu7Y72mD4zY9` (fresco ≠ autor `ses_f13392e30ffeqe5vRP62kIJgWE`) — **✅ APPROVE** (tier fast; C1-C6 reproducidos: comando CI exacto 2/2, suites 8/8 + 10/10 + 27/27, byte-identity 4/4, fmt/clippy exit 0). 0 Critical/Required; nit de conteo de seeds corregido. H1 → FIND-186 registrado (`70196101`).
> **Pendiente (LEAD):** review P2-01 + commit. Diff toca `tests/**`, `.config/nextest.toml`, `Cargo.toml`, corpus fuzz — **sin** `src/**` → tier fast (verify mecánico + veredicto registrado; no adversarial). Insumos para el revisor: §Verificación final (comandos → resultado), §Hallazgos (H1-FIND), y el wiring del perfil `chaos` verificado con el comando CI exacto.

## Context Save Point
- **Discovery ✅ (2026-09-29):** infra verificada (failpoints `storage_insert_fail` en `batch_prelude` + `wal_append_fail`; `1*off->return` = falla el 2º chunk — docs.rs/fail 0.5.1); precedentes `seed_v1_database`/`copy_dir_all`/failpoint tests; gap real = suite inexistente. Scope byte = probe empírico documentado (riesgo F1: WAL sella `last_accessed=now_ms` en el write path del engine — no es output del backfill).
- **Implementación ✅ (2026-09-29):** Steps 1-7 con evidencia por step (3 binarios nuevos + 2 suites extendidas; **cero cambios de producción** — `git status` no muestra `src/**`). RED capturó 3 fallos reales corregidos en el proceso (assert de mapas volcando bytes; `Embedded::get` sin namespace index en fixtures raw; fixture multi-shard del witness ERR-011). Refactor de runtime en chaos (fixture chico para S2/S3): 51s → 17-20s, margen CI (terminate-after 3 = 90s).
- **Pendiente (LEAD):** review P2-01 (fast) + commit local (nada de push). FIND candidato H1 (ERR-011/salvage) para fila en Backlog (prohibido editar Backlog en esta task). `nextTask`: SCH-07.
