---
title: "SCH-02: Schema v2 (bitemporal + confidence + quarantined + backfill)"
kind: task
description: "schema v2 implementado (validat/invalidat + confianza asserted/derived + quarantined, todo #[serde(default)] compatible v1) Y migración v1→v2 determinista (misma DB → mismo resultado) con backfill (validat=createdat..."
---

# SCH-02: Schema v2 (bitemporal + confidence + quarantined + backfill)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` — Task 27 (F3) · **Origen:** `docs/dev/Backlog.md:939` (+ ADR-0046 accepted 2026-09-28)
- **Fuente del prompt:** sub-agente vanta-worker (orquestador pipeline) — ejecución directa de SCH-02
- **Esfuerzo:** 🔴 3-5d · **Prioridad:** 🔴 · **Tipo:** feature-add (schema v2 + migración)
- **Turns estimados:** — · **Creado:** 2026-09-28 · **last-synced:** 2026-09-28
- **Estado:** ⏳ IN PROGRESS (implementación + fixes review #1 + re-verify ✅; re-review P2-01 + commit = LEAD)
- **Incógnitas (uphill):** 0 · **Pendientes (downhill):** re-review P2-01 + commit (LEAD)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Alcance | `src/sdk/types/record.rs` (ConfidenceClass + 10 campos `MemoryRecord` + 4 `MemoryInput` + 10 `MemoryExportLine` + re-export en `src/sdk/types.rs`) · `src/sdk/version_history.rs` (mirror `SnapshotRecord` + V1-fallback + `From`) · `src/sdk/serialization/mod.rs` (consts `__vanta_*`, mapeo record↔nodo, export v2/import {1,2}) · `src/sdk/serialization/impl_export.rs` (roundtrip) · `src/sdk/api/memory.rs` (materialización V1-V5 + supersede `invalid_at` + bulk projection) · `src/schema.rs` (`CURRENT_SCHEMA_VERSION 1→2`) · `src/migration.rs` (`FormatKind::Records` + backfill) · `src/cli_handlers/migrate.rs` (orden expand→backfill→bump) |
| Callees | `MemoryRecord`/`MemoryInput`/`MemoryExportLine` literales en TODO el workspace (members: `.`, `vantadb-python`, `vantadb-server`, `vantadb-mcp`, `vantadb-wasm`, `vanta-memory`, `vanta-proxy`) + tests root + benches; `UnifiedNode` (sin cambios de struct); `record_from_node`/`memory_record_to_node_owned` (39+35 consumidores); `restore_graph_nodes` (D6: intacto); `eviction.rs`/`llm.rs` (consumidores de `confidence_score`, sin cambios) |
| Implicaciones | Breaking de persistencia del corte 0.8.0 (aditivo en wire por `#[serde(default)]`); los 4 formatos (node KV/WAL, snapshot postcard, export JSONL, header) normalizan v1; migración in-place de snapshots + backfill de nodos determinista; los índices derivados NO se migran (rebuild-on-mismatch, `impl_index.rs:52`) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos / secciones):** `docs/dev/architecture/adr/ADR-0046-schema-v2-migracion-unica.md` (completo, verbatim) · `docs/dev/research/mgr-10-bitemporalidad.md` (§4), `mgr-12-confianza.md` (§2-§10), `mgr-13-cuarentena.md` (§2-§8) · `src/sdk/types/record.rs` (553L, completo) · `src/sdk/version_history.rs` (542L, completo) · `src/sdk/serialization/mod.rs` (1-600) · `src/sdk/serialization/impl_export.rs` (1-200) · `src/sdk/api/memory.rs` (1-620, 885-996) · `src/schema.rs` (272L, completo) · `src/migration.rs` (699L, completo) · `src/cli_handlers/migrate.rs` (415L, completo) · `src/wal.rs` (1-135) · `src/node/unified.rs` (1-120) · `src/storage/ops.rs` (1-200) · `src/storage/engine/insert.rs` (140-240, 414-500, 867-905) · `.opencode/rules/{durability,core-engine,api-contract,release-ci}.md` · `.opencode/references/clean-code-clean-architecture.md` (Apéndice V) · `.opencode/references/definition-of-done.md`
- **Referencias hacia dentro:** ADR-0046 §D2 (campos), §D3 (semántica), §D4 (reglas V1-V5 + tests D4d), §D5/D5d (cuarentena + 30d), §D6 (record↔nodo), §D7 (4 formatos + predicate), §D8 (backfill); plan Task 27 (:703-727); Backlog:939; `tests/{core/snapshot_certification,sdk_serialization,durability_recovery,text_index_recovery}.rs`
- **Referencias entrantes:** SCH-03 (`valid_at`/`invalid_at` en queries), SCH-04 (scores consumibles + `min_confidence`), SCH-05 (cuarentena operativa + T1 + sticky), SCH-06 (doble corrida/crash/chaos), SCH-07 (superficies), SCH-08 (guía 0.8.0); `restore_graph_nodes` (D6 test); eviction/prompt/executor (cambio observable D4c)
- **Veredicto impacto:** alto — API pública + 4 formatos de persistencia + migración; toda la workspace compila contra los structs extendidos

## Contrato
"schema v2 implementado (`valid_at`/`invalid_at` + confianza asserted/derived + `quarantined`, todo `#[serde(default)]` compatible v1) Y migración v1→v2 determinista (misma DB → mismo resultado) con backfill (`valid_at=created_at`, `invalid_at=superseded_at` si existe) Y roundtrip export/import verde (v1 sigue importable) Y reopen/durabilidad verdes"

## Spec (SDD — Phase 1b; decisiones por evidencia, ADR-0046 = spec)

| Decisión | Alternativas | Elegido | Evidencia / por qué |
|---|---|---|---|
| Forma de extensión | `MemoryRecordV2` vs extender record | **Extender record** (One-Version) | ADR-0046 §D2 ("aditivo puro — One-Version: se extiende el record, nunca un `MemoryRecordV2`") |
| `confidence` default | plano 0.0 vs `default_confidence` | **`#[serde(default = "default_confidence")]` (D_a=1.0)** | ADR-0046 §D2 tabla + §D4c; test enumera el default |
| `valid_at_ms` tipo | `Option<u64>` vs `u64` | **`u64` + normalización `ausente ⇒ created_at_ms`** | ADR-0046 §D2/§D8 ("**`u64`** + default de insert `:= created_at_ms`") |
| Rechazo `Derived + Some(confidence)` | clamp vs solo-interno vs rechazo | **Rechazo en boundary** con `Error::Validation{field:"confidence"}` | ADR-0046 §D4b (verbatim) |
| Backfill confidence | 0.5 continuidad vs 1.0 uniforme | **1.0 uniforme (D_a)** | ADR-0046 §D4c (aceptado por owner; deltas documentados) |
| Snapshot decode orden | V1-first vs V2-first | **V2-first → V1-fallback; re-encode siempre V2** | ADR-0046 §D7 tabla #2 (verbatim) |
| Import predicate | exacto ==2 vs `∈{1,2}` | **aceptar ∈{1,2} / rechazar >2** (v1 normaliza; export siempre 2) | ADR-0046 §D7 (verbatim) |
| `records` como formato | sub-paso de `schema` vs `FormatKind::Records` | **`FormatKind::Records`** + orden `all` = backfill ANTES del bump de header | ADR-0046 §Plan (comandos `--format records`); mgr-10:193 permite sub-paso pero los comandos piden `records` |
| Índices derivados | migrar in-place vs rebuild | **NO migrar in-place** (rebuild-on-mismatch existente) | ADR-0046 §D1d rabbit hole + `impl_index.rs:52` |
| `MemoryInput.quarantine` (T1) | SCH-02 vs SCH-05 | **SCH-05** (flag + ops + gates) | mgr-13 §7.3 ("flag T1 y opción de import T1c" en SCH-05); §7.2 SCH-02 = campos + mirrors + migración `None` |
| `include_quarantined`/`min_confidence` wire | SCH-02 vs SCH-04/05 | **SCH-04/05** | mgr-12 §6.1 (SCH-04) + mgr-13 §7.3 (SCH-05) |
| `valid_at_ms = Some(0)` | aceptar 0 vs rechazar | **Rechazar en boundary** (0 ⇒ unset en v1; "salvo valor explícito" no cubre epoch) | ADR §D8; elimina ambigüedad sentinel v1 sin cambio silencioso |

## Invariantes de dominio (handoff — MUST)
- `valid_at_ms <= invalid_at_ms` cuando ambos set (validación en boundaries + test) — ADR §D3-2.
- `supersede()` mantiene `invalid_at == superseded_at` alineados (D3-3) — test.
- Backfill = función pura: sin reloj/aleatoriedad/orden; idempotente (skip nodos con `__vanta_valid_at_ms`) — ADR §Migration.
- `record.confidence == node.confidence_score` post-write (D6) — test de mapeo.
- Enums públicos nuevos `#[non_exhaustive]` (api-contract R-6) + `#[serde(default)]` en TODO campo nuevo (D2).
- No tocar `restore_graph_nodes` (CORE-02 intacto; precedencia D6 documentada + test).
- `MIN_COMPAT_VERSION` queda en 1 (binario v2 lee v1; v1-binary sobre v2 ⇒ TooNew); bump del header AL FINAL.
- Prohibido: migrar índices derivados in-place; segundo breaking (v3); tocar `wal.rs` (struct `UnifiedNode` sin cambios).

## Deuda técnica (Regla 6 — MUST)
**Saldo neto:** 0. La deuda que consume esta task ya está documentada por el ADR (P27 crash-exactitud, cap 32/key, calibración VER-08). Nuevos `FIND` solo si el stop condition se dispara (no disparado).

## Definition of Done (3 niveles)
- **Task:** contrato verbatim ✅ — campos + 4 formatos + backfill determinista + import v1 + tests V1–V5 (D4d) + reopen/durabilidad + review P2-01 (LEAD).
- **Commit:** `feat(schema): v2 — bitemporal + confidence + quarantined + backfill (SCH-02)` — **lo ejecuta el LEAD** (sub-agente sin commit ni self-review).
- **Release:** N/A (el corte 0.8.0 = SCH-08; gate F3 "migración determinista verde").

## Herramientas necesarias
- `codegraph_codegraph_explore` (blast radius) · `cargo nextest run --profile audit -p vantadb -E 'test(migrat) or test(schema) or test(serialization) or test(snapshot) or test(reopen)'` + scoped por archivo · clippy `-D warnings` · `CARGO_BUILD_JOBS=2` · regla dura `-p` · `campaign_verify_cmd` · `pwsh -NoProfile scripts/validate-docs-coverage.ps1`
- **Skills cargadas (SDP v3, BUILD):** base `campaign-executor`+`progreso`+`ponytail` (auto) · pin `deprecation-and-migration` · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` · `test-driven-development` · `rust-write-tests`

## Investigation Notes
- **4 formatos** (ADR §D7): (1) node KV/WAL — campos `__vanta_*` aditivos; WAL `WalRecord::Insert(UnifiedNode)` sin cambios; (2) mirror `SnapshotRecord` — append al final + V2-first→V1-fallback (postcard ignora trailing; V1→V2 falla por faltantes) + re-encode V2; (3) export JSONL — predicate ∈{1,2}; (4) header — bump al final.
- **Bulk (`VDBJSON\n` + 0x01 + `Vec<MemoryInput>`)**: boundary propio (5º); serde-aditivo; test bulk v0x01→v2. `bulk_import_stream` construye nodos a mano → se le agrega proyección v2 (`valid_at`, `class`, `confidence_score`) y rechazo explícito de `Derived` (no hay derivación en el path raw).
- **Determinismo**: `stage_one_node` estampa `last_accessed = now_ms` en el **WAL** (no en `NodeMetadata` KV, verificado `storage/ops.rs:14-23`); el backfill en sí no lee reloj; la doble corrida byte-idéntica con reloj controlado es de SCH-06 (mgr-10:199, riesgo aceptado del engine).
- **`record_from_export_line`**: usado por MCP import/CLI/HTTP (`import_records`) → el predicate nuevo aplica a los 3 paths (D7 nota).
- **Grep verificado**: no existe `quarantine` de contenido ni `confidence` por registro; `valid_at` = 0 hits de dominio.
- **Tests existentes a actualizar:** `tests/core/snapshot_certification.rs:261-286,1107` (export v2), `tests/common/mod.rs:278,330` son `TestMetric` (no tocar); fixtures v1 (`:434-436`) quedan como compat.
- **`export_md.rs`** usa su propio `MD_EXPORT_SCHEMA_VERSION=1` (frontmatter MD) — NO tocar.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 — determinismo (función pura + skip idempotente) y 4 formatos resueltos con tests; review #1 aplicado |
| Pendientes | re-review P2-01 + commit (LEAD) — implementación + fixes + re-verify ✅ |
| % completado | 97% (contenido + fixes + verify ✅; cierre formal LEAD) |

## Recitation
```
=== RECITATION ===
Objetivo activo: SCH-02 — Schema v2 (bitemporal + confidence + quarantined + backfill)
Estado: in-progress (review #1 CHANGES REQUIRED aplicado + re-verify completo; re-review/commit LEAD)
Última acción: Fixes P2-01 (Step 8): R1 (schema sin backfill imposible + dry-run honesto), R2 (validación en put_record_exact), R3 (fixture v1 + tests por path MCP/HTTP/CLI), O1/O2/O3/N1/N2. Re-verify: nextest audit -p vantadb 2403/2403 ✅ · vanta-memory 549/549 ✅ · proxy/mcp/server/wasm/py 437/437 ✅ · e2e server 18/18 ✅ · clippy core+members ✅ · fmt ✅ · docs-coverage 0 gaps ✅
Resultado: PARTIAL
Próxima acción: LEAD — re-review P2-01 adversarial (src/sdk/** + serialization + migrate) + commit `feat(schema): v2 — bitemporal + confidence + quarantined + backfill (SCH-02)` + plan/Backlog/progreso
Contrato: "schema v2 (#[serde(default)] compat v1) + migración v1→v2 determinista con backfill + roundtrip export/import (v1 importable) + reopen/durabilidad" → verificado; fixes R1-R3/O/N con tests (ver §Review P2-01 — fixes aplicados + §Verificación)
Invariantes: un solo breaking (v2/0.8.0); MIN_COMPAT=1; backfill función pura (sin reloj); D4b rechazo; D3-3 alineación (guard future-valid); D6 record↔nodo; restore_graph_nodes intacto; bump imposible sin backfill
Deuda: ninguna nueva (saldo Regla 6 = 0); FIND-184 citado (pre-existente)
Próxima tarea si completa: SCH-03
last-synced: 2026-09-28
=== END RECITATION ===
```

## Fases explícitas — SECURITY | PERFORMANCE
- [ ] **SECURITY** — trust boundary: import/persistencia (dataset v1/v2 hostile), validación de confianza (finito [0,1]), D4b, V1/V3; sin FFI nueva. Checklist `security-and-hardening` al cierre.
- [ ] **PERFORMANCE** — serialización (formatos) y write path (`memory_record_to_node_owned`): sin cambio de complejidad; verificación sin regresión vía tests; hot paths de búsqueda NO tocados (SCH-03/04).

## Steps
### Step 1: Tipos core — ConfidenceClass + campos (record.rs + types.rs)
- **Archivos:** `src/sdk/types/record.rs`, `src/sdk/types.rs`
- **Acción:** enum `#[non_exhaustive] ConfidenceClass` (ADR §D2 verbatim) + `default_confidence()` + `DERIVATION_DISCOUNT` + `MAX_DERIVATION_DEPTH` + 10 campos `MemoryRecord` + 4 `MemoryInput` + 10 `MemoryExportLine` (todos `#[serde(default)]`; `confidence` con `default_confidence`) + `Default` para los 3 tipos (mitiga el blast radius de literales).
- **Verify:** `cargo check -p vantadb` ✅ + unit tests serde defaults ✅ (39/39 scoped)
- **Estado:** ✅ DONE

### Step 2: Mapeo record↔nodo (serialization/mod.rs) + supersede
- **Archivos:** `src/sdk/serialization/mod.rs`, `src/sdk/api/memory.rs`
- **Acción:** 9 consts `FIELD_*` nuevas; write path setea nodo (`__vanta_*` + `confidence_score`, marcador v2 = `FIELD_VALID_AT_MS`); read path con normalización v1 (valid_at⇒created_at, conf D_a, class Asserted; el 0.5 legacy no filtra); `supersede` setea `invalid_at_ms = superseded_at_ms` (D3-3).
- **Verify:** tests `memory_record_v2_fields_roundtrip_through_node` + `v1_node_without_v2_fields_normalizes_on_read` + `supersede_aligns_invalid_at_with_superseded_at` ✅
- **Estado:** ✅ DONE

### Step 3: Materialización de confianza V1–V5 (put/put_batch) + bulk
- **Archivos:** `src/sdk/api/memory.rs`
- **Acción:** validación [0,1]/V1/D4b; derivación `min(padres)×0.9` + V3 (ciclo + profundidad 16); `valid_at` default insert (Some(0) rechazado); carry-forward de cuarentena (I2); bulk: proyección v2 + rechazo explícito de Derived.
- **Verify:** tests V2 (0.45), V4 determinista, V5 re-put, V1/V3, D4b, rango, valid_at, sticky ✅
- **Estado:** ✅ DONE

### Step 4: Snapshot mirror (version_history.rs) + export/import v2
- **Archivos:** `src/sdk/version_history.rs`, `src/sdk/serialization/mod.rs`
- **Acción:** mirror con append (incluye `superseded_*`), V2-first→V1-fallback, re-encode V2 (+ `reencode_snapshot_v2` para migración); `EXPORT_SCHEMA_VERSION=2`; `record_from_export_line` ∈{1,2} + normalización v1 + validaciones locales (rango/V1/ventana).
- **Verify:** test bytes V1 reales + roundtrip v2 + import v1 + rechazos ✅
- **Estado:** ✅ DONE

### Step 5: Migración `records` (migration.rs + migrate.rs) + bump header
- **Archivos:** `src/migration.rs`, `src/cli_handlers/migrate.rs`, `src/schema.rs`
- **Acción:** `FormatKind::Records` (plan + dry-run + real), backfill nodos determinista (batch, skip ya-v2), re-encode de snapshots in-place, `CURRENT_SCHEMA_VERSION=2`; orden backfill→bump en `cmd_migrate`; `MIN_COMPAT_VERSION=1`.
- **Verify:** 5 tests nuevos (idempotencia, dry-run, determinismo 2 copias, snapshots, plan) + `cargo nextest` scoped ✅
- **Estado:** ✅ DONE

### Step 6: Compile-fix workspace + tests de integración (compat/reopen/restore D6)
- **Archivos:** 53 archivos con literales (transform mecánico `..Default::default()`), snapshots insta regenerados (6), `tests/api/public-api.txt` regenerado (134 símbolos v2), tests nuevos en `memory_export_import.rs`/`memory_api.rs`, asserts v2 en `snapshot_certification.rs`.
- **Verify:** `cargo check --workspace --all-targets` ✅ · `vanta-memory` 549/549 ✅ · `proxy+mcp+server+wasm+py` 437/437 ✅ · `query_result_*` con snapshots ✅
- **Estado:** ✅ DONE

### Step 7: Verify full + OCR + docs/avance
- **Archivos:** —
- **Acción:** fmt/clippy/nextest/docs-coverage ✅ (evidencia abajo); OCR pendiente (advisory, LEAD)
- **Estado:** ✅ DONE (con notas)

### Step 8: Fixes review P2-01 (R1-R3 + O1-O3 + N1-N2)
- **Acción:** R1 (orden expand→backfill→bump: `--format schema` auto-ensure del backfill antes del bump + dry-run honrado en ambos branches de schema) · R2 (`put_record_exact` valida confianza/ventana — choke point import tipado HTTP/WASM/SDK) · R3 (fixture `tests/fixtures/export-v1.jsonl` + tests por path MCP/HTTP/CLI) · O1 (tolerancia D3-3 documentada + test) · O2 (supersede rechaza registro future-valid) · O3 (techo de RAM del backfill documentado) · N1 (comentario `decode_snapshot` corregido + test de semántica postcard) · N2 (bulk rechaza `valid_at_ms: Some(0)`, alineado con put).
- **Verify:** 22/22 tests nuevos/fixeados (vantadb) + MCP 2/2 + HTTP e2e 18/18 ✅
- **Estado:** ✅ DONE

### Step 9: Cierre (LEAD)
- **Acción:** re-review P2-01 (tier adversarial: `src/sdk/**` + serialization + migrate) + commit `feat(schema): v2 — ... (SCH-02)` + plan/Backlog/progreso
- **Estado:** ✅ COMPLETED (2026-09-28) — review P2-01 2 rondas: ronda 1 ❌ CHANGES REQUIRED (R1/R2/R3) → fixes → ronda 2 delta ✅ APPROVE. Revisor: `ses_f15b8fdd3ffeG3TM604hQUlNXE` ≠ autor `ses_f16557d2fffeLJdSAlSw0YOeJj`. Gate completo reproducido por el revisor: `-p vantadb` **2403/2403** (431.5s) + scoped R1/R2/R3/N2/migración (5+4+1+1+3+5 ✅) + fmt 0 + clippy core/members 0.

## Review P2-01 — fixes aplicados (2026-09-28)
> Review #1: ❌ CHANGES REQUIRED (3 Required + 3 Optional + 3 Nits). Todos aplicados; re-review = LEAD.

| # | Fix | Archivos | Test |
|---|-----|----------|------|
| **R1** | Bump de header ya no puede ejecutarse sin backfill: en el branch de bump, si Records no fue procesado en la corrida → `migrate_records()` (idempotente) ANTES del bump; `dry_run` honrado en ambos branches del schema (bump y header-less) — `--format all --dry-run` ya no escribe nada | `src/cli_handlers/migrate.rs` | `test_migrate_schema_alone_ensures_backfill_before_bump` (dry→v1 intacto; real→backfill+bump) · `test_migrate_all_dry_run_writes_nothing` (header v1 + nodos intactos + backfill sigue ejecutable) · `test_migrate_run_dry_run_does_not_write_schema` (dir sin header) |
| **R2** | `put_record_exact` valida `validate_confidence_fields` (rango/V1) + ventana `valid_at <= invalid_at` — cubre HTTP `import` records, WASM `import_records`, SDK Rust | `src/sdk/api/memory.rs` | `put_record_exact_rejects_invalid_confidence_and_window` · `import_records_counts_invalid_raw_records_as_errors` (errors=1, no persiste) |
| **R3** | Fixture v1 versionado + test por path de la convergencia del ADR §D7 (MCP/HTTP/CLI → `record_from_export_line`) | `tests/fixtures/export-v1.jsonl` (nuevo) · `tests/cli_tests.rs` · `vantadb-mcp/tests/mcp_tests.rs` · `vantadb-server/tests/e2e.rs` | `test_import_cli_v1_fixture_normalizes` · `test_mcp_import_v1_line_normalizes` · `test_e2e_import_v1_jsonl_path_normalizes` |
| **O1** | Tolerancia D3-3 en import v2 documentada (0.8.0 alinea; setter retroactivo v1.0 puede divergir → se preserva verbatim) | `src/sdk/serialization/mod.rs` | `record_from_export_line_tolerates_d3_divergence` |
| **O2** | `supersede` rechaza registros future-valid (evita `invalid_at < valid_at` y divergencia D3-3) | `src/sdk/api/memory.rs` | `supersede_rejects_record_valid_only_in_the_future` |
| **O3** | Techo de RAM del backfill documentado (`ponytail:` one-shot offline + upgrade path `scan_nodes_page`) | `src/migration.rs` | — (documental) |
| **N1** | Comentario `decode_snapshot` corregido: postcard decodifica structs como tuplas de longitud fija → V2-decode-V1 falla en EOF (los `#[serde(default)]` NO aplican a structs truncados); el fallback V1 es load-bearing | `src/sdk/version_history.rs` | `snapshot_v1_bytes_fail_the_v2_shape_and_use_the_v1_fallback` (guard de semántica postcard) |
| **N2** | Bulk rechaza `valid_at_ms: Some(0)` (alineado con el boundary validado; ya no reinterpreta el sentinel) | `src/sdk/api/memory.rs` | `bulk_import_rejects_explicit_zero_valid_at` |

## Verificación (evidencia 2026-09-28)
- `cargo fmt --all -- --check` → exit 0 (manual; el runner de `campaign_verify_cmd` devuelve -1/spawn 0.3s = flake del runner, no del código)
- `cargo clippy -p vantadb --no-default-features --features "cli,fjall,memmap2,fs2,roaring" --all-targets -- -D warnings` → ✅ exit 0 (canónico verify.ps1)
- `cargo clippy -p vanta-memory -p vanta-proxy -p vantadb-mcp -p vantadb-wasm -p vantadb-server --all-targets -- -D warnings` → ✅ exit 0
- `cargo nextest run --profile audit -p vantadb --build-jobs 2` → ✅ **2403/2403 passed** (manual, 4 slow; el runner de campaign reintentó y el flake conocido `concurrent_insert_preserves_hnsw_invariants` timeoutea bajo ese runner — pasa aislado en 46s y en la corrida manual completa)
- Tests del review: `-E 'test(test_migrate) | test(put_record_exact) | ...'` → 22/22 ✅ · MCP `test_mcp_import_v1_line_normalizes` ✅ · `vantadb-server binary(e2e)` → 18/18 ✅
- `cargo nextest run --profile audit -p vanta-memory` → 549/549 ✅ · `-p vanta-proxy -p vantadb-mcp -p vantadb-server -p vantadb-wasm -p vantadb_py` → 437/437 ✅
- `pwsh -NoProfile scripts/validate-docs-coverage.ps1` → exit 0, 0 gaps ✅
- **Contrato:** determinismo del backfill (test 2 copias + idempotencia) ✅ · roundtrip export/import v1+v2 (3 paths) ✅ · reopen/durabilidad ✅ · compat v1 (node/snapshot/export/bulk) ✅ · orden expand→backfill→bump garantizado en CLI ✅
- **FIND-184 (pre-existente):** `cargo nextest --workspace` falla por unificación de features (`vanta-memory/tests/smoke.rs` E0080 con `llm-driver` de mcp/proxy) — NO es de este diff; gate canónico corre `-p vantadb`; ya registrado en Backlog.

## Dependencias
- **Consume:** SCH-01 ✅ (ADR-0046 `accepted`, firmado 2026-09-28) + MGR-10/12/13 ✅.
- **Destraba:** SCH-03 (`valid_at`/`invalid_at`), SCH-04 (scores), SCH-05 (cuarentena T1/sticky), SCH-06 (suite determinismo).

## Review (GATE P2-01)
- **Review #1:** ❌ CHANGES REQUIRED (3 Required + 3 Optional + 3 Nits) — aplicados en Step 8 (ver §Review P2-01 — fixes aplicados) + re-verify completo ✅.
- ⬜ **Re-review PENDIENTE** — tier **adversarial** (`src/sdk/**` + wire/serialización + migrate); reviewer fresco ≠ autor; no self-review (LEAD).

## Notas
- **No commit / no push / no self-review** (instrucción del orquestador; cierre en LEAD).
- **PROHIBIDOS (ajenos):** `perf-bench.yml`, `opencode.jsonc`, `docs/dev/plans/*`, `Backlog.md` (WIP pre-existente en working tree — no tocar). También apareció `docs/dev/research/docs-strategy/` (untracked, sesión ajena) — intacto.
- `MemoryInput.quarantine` (T1), `include_quarantined`/`min_confidence` wire, sticky completo, gates de inyección → **SCH-04/05** (mgr-12 §6.1, mgr-13 §7.3).
- **docs/api/ (Regla 3):** los símbolos v2 nuevos se documentan en `docs/api/` en el mismo PR del corte → **SCH-07** (contrato propio); `validate-docs-coverage` ya verde (0 gaps).
- Fixtures v1 (`snapshot_certification.rs:434-436`, tests v1 nuevos) quedan como compat: el import v1 normaliza.
- **NOTICED BUT NOT TOUCHING:** `stage_one_node`/`insert` estampan `last_accessed=now` en el WAL (infra pre-existente; la doble corrida byte-idéntica con reloj controlado es de SCH-06, mgr-10:199); `migrate_vector_index`/`migrate_records` abren con `Config::default()` (backend default Fjall; RocksDB opt-in pre-existente, no ampliado acá).
- Snapshots insta regenerados deliberadamente (6 archivos, solo campos v2) + `tests/api/public-api.txt` (+134 líneas de símbolos v2).
