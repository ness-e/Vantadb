---
title: "TASK MEMG-12: Semántica v2 write-side en el pipeline (confidence/valid_at)"
kind: task
description: "L1 escribe semántica v2 real en el punto único put_record (extracción + promoción dream): valid_at_ms = nacimiento del contenido + confianza Asserted/D_a explícitas; round-trip test put→export→import; TTL y derived/T1b quarantine diferidos a FIND (stop condition 1.5sem)"
---

# TASK MEMG-12: Semántica v2 write-side en el pipeline (`confidence`/`valid_at`/TTL)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 40, Wave F2)
- **Fuente:** Backlog `MEMG-12` (fila :147) + plan Task 40 (bloque F0-expandido) + `docs/dev/research/mgr-10-bitemporalidad.md` (§D7/D8, §1.4) + `docs/dev/research/mgr-12-confianza.md` (§3.4) + `docs/dev/research/mgr-13-cuarentena.md` (§3.2 T1b) + DELTA §P1 (2026-09-30)
- **Esfuerzo:** 🟠 1-1.5sem | **Appetite:** max 2sem | **Stop aplicado (plan L1158):** entregar `confidence`+`valid_at` en L1 (los dos sitios del pipeline) + FIND de TTL/dream-promote restante
- **Prioridad:** 🟠
- **Tipo:** Rust (crate `vanta-memory`; core SDK consumido sin modificar)
- **Turns estimados:** 12-20 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `40` en el campaign server)
- **Incógnitas (uphill):** 1 → política de TTL/valid_at por tipo (MGR-09 sin research-doc) → **resuelta como política mínima declarada + `[a verificar]`** (pre-mortem #1 del plan; NO se inventa la fórmula completa)
- **Pendientes (downhill):** 4 steps (1-4)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `40`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `put_record` (pub(crate), 4 call sites HEAD): `write_memory` (`l1_writer.rs:170,250` — extracción store/update/merge), `mark_contradicted_targets` (`:327`), `promote_dream_run` (`dream/mod.rs:914`). `write_memory` ← `apply_dedup_batch`, `run_l1_dedup`, `pipeline_worker`, tests (`generation_log`, `l1_dedup`, `l1_contradiction`, `capture_approval`, `pipeline_manager`). `promote_dream_run` ← MCP `vantadb-mcp/src/dreams.rs:275` + `dreaming.rs` tests (firma sin cambios → callers intactos). |
| Callees | Core SDK existente (sin cambios): `MemoryInput` v2 (`valid_at_ms`/`confidence_class`/`confidence`), `materialize_valid_at`/`materialize_confidence` (`src/sdk/api/memory.rs:203-318`), `default_confidence`/`ConfidenceClass` (`src/sdk/types/record.rs:74-120`), `put_one`. `chrono` (dep ya usada por `seed/md_import.rs:270`), `epoch_ms_to_rfc3339` (prompts). Cero dependencias nuevas. |
| Implicaciones | Semántica de escritura L1: `valid_at_ms` explícito = nacimiento del contenido (merge = earliest target) y confianza `Asserted`/D_a explícitas en cada write del pipeline. **Wire intacto** (campos v2 ya existen, `#[serde(default)]` sin tocar); payload L1 (vanta-memory `MemoryRecord`) intacto; sin símbolos públicos nuevos; sin migración. Regresión: tests de export (md_git_e2e asserts de presencia) y suites de dedup/contradiction/dream se re-corren. **Coordinación MEMG-11:** su Step 1 refactoriza builders (`record_input`) en el MISMO archivo → diff mínimo y localizado; no se tocan `apply_dedup_batch`/`l1_batch`/`auto_recall` (paths de MEMG-11). |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `29beacff`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `vanta-memory/src/core/record/l1_writer.rs` (:1-440 — `put_record` `:396-423` con `ttl_ms: None` + `..Default::default()`, `write_memory` `:125-278`, `mark_contradicted_targets` `:297-330`).
  - `vanta-memory/src/core/dream/mod.rs` (:150-300 tipos `DreamRun`/`Dreamer`; `:440-569` store layer + `write_dream_run` `ttl_ms: None` `:537`; `:700-935` `plan_promotion`/`build_promotion`/`promote_dream_run` — apply vía `put_record` `:914`).
  - `vanta-memory/src/core/abstractions/types.rs` (399L — L1 payload `MemoryRecord` `:71-130`: `created_at` ISO, `timestamps`, sin campos v2).
  - `vanta-memory/src/core/record/l1_reader.rs` (:1-100 — `l1_namespace`, `read_record`, gate `include_quarantined: false` `:53`).
  - `vanta-memory/src/core/conversation/l0_recorder.rs` (:180-299 — site `ttl_ms: None` `:243`, fuera del stop scope).
  - `vanta-memory/src/services/pipeline_worker.rs` (:404-548 `run_l1`/`run_l1_batch` → ambos funnel a `l1_writer`; `:690-829` site `ttl_ms: None` `:775` = assembled-context, fuera del stop scope).
  - `src/sdk/types/record.rs` (:1-327 — `default_confidence` D_a=1.0 `:74`, `ConfidenceClass` `:87-120`, `MemoryInput` v2 `:176-247`).
  - `src/sdk/api/memory.rs` (:60-568 — `enter_quarantine` `:98-109`, `materialize_confidence` `:203-265`, `materialize_valid_at` `:306-318` (None→created_at; `Some(0)` rechazado), `put_one` `:457-561`).
  - `src/sdk/serialization/impl_export.rs` (:170-429 — `export_namespace`, `import_file(quarantine)`, `import_records` → `put_record_exact`).
  - `vanta-memory/src/seed/mod.rs` (:120-199 — espejo export v2), `seed/md_import.rs` (:250-294 parse RFC3339 con chrono; `:344` `import_records(changed, false)`), `tests/generation_log.rs` (fixture `test_db` + `write_memory`), `tests/md_roundtrip.rs` (round-trip v2 existente), `tests/recall.rs:674-725` (gate cuarentena).
  - Specs: `mgr-10-bitemporalidad.md` §D7/D8/§1.4, `mgr-12-confianza.md` §3.1-3.4, `mgr-13-cuarentena.md` §2.3/§3.2-3.4, ADR-046 (§D5e import default, §D4c D_a=1.0).
- **Archivos referenciados hacia dentro (imports/deps):** `l1_writer` → `vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata, Value}`, `l1_reader` (`read_record`, `l1_namespace`), `lifecycle` (`mark_contradiction`), `prompts` (`epoch_ms_to_rfc3339`), `profile`. `dream/mod.rs` → `l1_writer::put_record` (mismo crate). Nuevo: `chrono` (ya en el crate), `vantadb::sdk::{default_confidence, ConfidenceClass}`.
- **Referencias entrantes (grep/CodeGraph HEAD):** `put_record` 4 call sites (arriba); `write_memory` = `pipeline_manager.rs` + 5 test files; `promote_dream_run` = MCP `dreams.rs:275` + `dreaming.rs` (firma intacta). Ningún caller cambia.
- **Veredicto impacto:** **BAJO-MEDIO controlado** — cambio localizado en 1 función (`put_record`) + 1 helper privado + 1 archivo de test nuevo; sin API pública nueva, sin wire, sin deps, sin locks. Riesgos del pre-mortem mitigados: (1) TTL sin spec → política mínima declarada + FIND (no se inventa); (2) doble escritura de confianza → punto único `put_record` (valor intrínseco al record aguas arriba; derived futuro extenderá con stamp explícito); (3) cuarentena solapada → NO se toca (T1b → FIND, sticky I2 intacto).

## Contrato

"`put_record` (punto único de escritura L1 — pre-mortem #2) estampa semántica v2 real en los **dos sitios del pipeline que escriben L1** — extracción (`write_memory` store/update/merge) y promoción dream (`promote_dream_run`) — sin cambios de wire ni símbolos públicos: `valid_at_ms` = nacimiento del contenido (record `created_at` ISO→ms; merge = earliest target; fallback al default del core si el parse falla o es 0) y confianza `Asserted` + D_a explícitas (MGR-12 §3.4: default uniforme asserted/D_a para toda escritura directa). Round-trip test (pipeline put → `export_namespace` → `import_file` → mismos campos v2). Invariantes MGR-10 §1.4 preservados (`valid_at <= invalid_at`; backfill puro — sin migración). Política declarada en §Spec: **sin TTL** hasta MGR-09 (FIND) y **derived/T1b quarantine → FIND** (stop condition 1.5sem: requieren parent linkage en `Dreamer` + write options `reason`/`by` que no existen; NO se inventan). Verify: `cargo nextest run --profile audit -p vanta-memory --test l1_semantics_v2 --build-jobs 2` + suite `-p vanta-memory --build-jobs 2` + `cargo fmt --check` + `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` verdes."

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): no disparado** — el plan (Task 40, Gate Result ✅ DO) sanciona la semántica y los sitios; el slice no agrega símbolos públicos nuevos (`put_record` es `pub(crate)`) ni cambia contratos; micro-decisiones decididas-por-evidencia abajo. Gate mecánico spec-first: esta sección llena.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Sitios de escritura | A) **Los 2 sitios L1 del pipeline: extracción (`write_memory`) + promoción (`promote_dream_run`), ambos vía `put_record`** / B) solo extracción / C) L0 también (fuera del stop: "en L1") | ✅ **A** — decidido-por-evidencia: plan L1158 ("confidence+valid_at en L1 (los dos sitios)"); los únicos dos writers de `l1/<session>` son `write_memory` y `promote_dream_run` (grep HEAD: `put_record` = 4 call sites, todos L1) |
| 2 | Mecanismo de estampado | A) **Auto-estampado en `put_record` a partir del record (created_at + política asserted/D_a)** / B) parámetro `WriteV2` en la firma (4 call sites) / C) campos v2 en el payload L1 (duplica estado → divergencia, MGR-13 §2.3) | ✅ **A** — decidido-por-evidencia: pre-mortem #2 ("un solo punto (`l1_writer`)"); el valor es intrínseco al record (su `created_at` ES el nacimiento); B queda para derived (FIND) cuando exista valor aguas arriba; C rechazado por MGR-13 §2.3 (doble fuente) |
| 3 | `valid_at_ms` L1 | A) **Nacimiento del contenido: `record.created_at` ISO→ms; merge/update = earliest target (ya computado en el record); fallback None (default core) si parse falla/0** / B) now_ms de la escritura (= default actual) / C) timestamps de mensajes (heurística — NO inventar) | ✅ **A** — decidido-por-evidencia: MGR-10 §D7 (normalización v1 "valid_at = created_at") + §D8 (None→created_at_ms); el merge HOY diverge (storage created=now vs L1 created=earliest) → A lo alinea; C prohibido por pre-mortem #1 |
| 4 | `confidence` L1 asserted | A) **Clase `Asserted` + score D_a explícitos (`Some`) en cada write** / B) dejar `None` (default implícito) | ✅ **A** — decidido-por-evidencia: contrato ("escribe... reales (no defaults)"); MGR-12 §3.4 ("default uniforme asserted/D_a para toda escritura directa") + Q1/D4c (D_a=1.0); el write DECLARA su semántica (V5: clase por escritura) |
| 5 | Derived class + T1b quarantine (dream) | A) **FIND** (parent linkage no existe en `Dreamer::consolidate` → `Vec<MemoryRecord>` sin `derived_from`; T1b exige `reason=derived_promotion`/`by=system:dream_promote` que `MemoryInput.quarantine: bool` no expresa) / B) heurística de parents / C) extender `MemoryInput` + `Dreamer` (símbolos públicos nuevos → Gate D; fuera del stop) | ✅ **A** — decidido-por-evidencia: plan L1158 ("FIND de ... dream-promote restante"); inventar parents violaría V1/V4 (MGR-12 §2.3); C = trabajo restante documentado en FIND-280 |
| 6 | TTL semántico | A) **Sin TTL (política mínima declarada) + FIND** — MGR-09 (retention) sin research-doc; L1 se gobierna por heat/lifecycle (MEM-60), no TTL; artifacts (`__assembled`, `run.json`) se sobreescriben/auditan / B) TTL por tipo inventado | ✅ **A** — decidido-por-evidencia: pre-mortem #1 ("política mínima declarada + [a verificar] — NO inventes"); plan L1158 permite FIND; `[a verificar]` contra MGR-09 |
| 7 | Round-trip | A) **`export_namespace` → `import_file` sobre el record escrito por el pipeline; comparar campos v2** / B) reusar solo md_roundtrip (no cubre pipeline) | ✅ **A** — decidido-por-evidencia: contrato ("put → export → import → mismos campos"); APIs core existentes (SCH-02 ya preserva v2 en import) |
| 8 | Wire / payload | Sin cambios: campos v2 ya existen en `MemoryInput`/`MemoryRecord` core; payload L1 (vanta-memory) intacto | ✅ — `#[serde(default)]` ya presente; sin migración (backfill puro MGR-10 §1.4) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. Wire v2 y payload L1 intactos — sin campos nuevos, sin migración; `#[serde(default)]` sin tocar.
  2. `valid_at <= invalid_at` (aquí `invalid_at = None`) y `valid_at > 0`: `Some(0)` nunca se emite (fallback `None` → default core `created_at_ms`).
  3. `put_record` = **punto único** de estampado (pre-mortem #2): ningún otro sitio calcula/escribe confianza en el pipeline L1.
  4. Cuarentena sticky (I2) y gates de lectura intactos — este slice NO toca quarantine.
  5. `src/sdk/**` sin cambios (solo consumo); `src/wal.rs`, `src/storage/`, `src/vector/` no se tocan.
  6. Sin `unwrap`/`expect`/`unsafe` en código nuevo de producción (Regla 2/4); sin dependencias nuevas.
  7. **MEMG-11 (en vuelo, mismo worktree):** no tocar `apply_dedup_batch`/`l1_batch.rs`/`auto_recall.rs`; releer `l1_writer.rs` fresco desde HEAD antes de editar; commit con **pathspec** (no stagear sus untracked ni WIP ajeno).
- **Comandos de verificación:** `cargo nextest run --profile audit -p vanta-memory --test l1_semantics_v2 --build-jobs 2` · `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` · `cargo fmt --check` · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings`.
- **Deuda pendiente:** TTL semántico por tipo (FIND-279, MGR-09); derived class + T1b quarantine de dream + T1c plumbing de import (FIND-280); calibración/decay = VER-08 (ajeno).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — sin `unsafe` nuevo, sin clones en hot path, sin abstracciones especulativas; el cambio **elimina** una dependencia implícita en defaults (el write ahora declara su semántica) y agrega tests de contrato. Sin deps nuevas.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: `valid_at_ms` de nacimiento + `Asserted`/D_a explícitas en ambos sitios L1, con test dedicado RED→GREEN + round-trip + suites scoped verdes + fmt/clippy verdes + política declarada (§Spec) |
| **Commit** | Commit atómico conventional `feat(memory):` + pathspec solo de archivos propios (sin WIP ajeno ni untracked de MEMG-11) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | Changelog release-plz (feature → minor); verify full scoped documentado (workspace completo no se corre por presupuesto) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius `put_record`/`write_memory`/`promote_dream_run`) + `codebase-memory-mcp_check_index_coverage` (7 paths, `no_recorded_issue` ✅) + grep puntual (tests/docs no indexados)
- `cargo nextest` scoped por crate (loop TDD) + `campaign_verify_cmd` (verify mecánico)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Gates docs (`node scripts/docs/check-links.mjs && check-docs.mjs && gen-index.mjs --check`) — task file nuevo

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` · `test-driven-development` · `context-engineering` (SDP phase=BUILD) + rol: `rust-write-tests`, `documentation-and-adrs`. `security-and-hardening` excluida (sin trust boundary nuevo — solo campos existentes en writes internos); `performance-optimization` excluida (no hot path — 2 campos extra en un `put` ya existente; Regla 9 no dispara).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluación: no introduce trust boundary nuevo; escribe campos ya validados por el core (`materialize_valid_at`/`materialize_confidence`); el parse RFC3339 es input interno (records propios) con fallback seguro; sin auth/secrets/deps. Sin hallazgos que requieran `security-and-hardening`.
- [x] **PERFORMANCE** — no aplica: el cambio agrega 2 campos al `MemoryInput` de un `put` ya existente + 1 parse de string por record; sin loops nuevos, sin serialización nueva, fuera de hot paths de search. Regla 9 no dispara (no hay claim de optimización).

## Steps

### Step 1 — RED: tests dedicados de la semántica v2 write-side

- **Archivos:** `vanta-memory/tests/l1_semantics_v2.rs` (nuevo)
- **Acción:** escribir los tests que fijan el contrato ANTES de implementar: (1) `store_write_stamps_valid_at_with_record_birth` — `write_memory` Store con `now_ms` sintético T0 → storage record: `valid_at_ms == T0` (hoy: real-now ≠ T0 → RED), `confidence_class == Asserted`, `confidence == 1.0`, `derived_from` vacío; (2) `merge_write_stamps_valid_at_with_earliest_target_birth` — store T0/T1 + Merge en T2 → merged: `valid_at_ms == T0` (earliest; hoy: real-now → RED); (3) `dream_promotion_stamps_valid_at_with_content_birth` — `DreamRun` con record consolidado id nuevo `created_at=T0` → `promote_dream_run` → storage: `valid_at_ms == T0` (hoy: real-now → RED); (4) `pipeline_write_roundtrips_v2_fields_via_export_import` — pipeline put → `export_namespace` → `import_file` en db fresca → mismos campos v2 (incl. `valid_at_ms == T0`). Correr → RED esperado: fallas de aserción en `valid_at_ms` (el resto de campos ya coinciden por default).
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test l1_semantics_v2 --build-jobs 2` → RED por la razón correcta (assertion `valid_at_ms`, no error de compilación)
- **Evidencia:** ✅ RED verificado: `4 tests run: 4 FAILED` — aserciones `valid_at_ms` con `left: 1791185790571` (reloj real) vs `right: 1700000000000` (T0 sintético) en store/merge/promotion/round-trip; fallo por la razón correcta (el default write-time, no error de compilación).
- **Estado:** ✅ COMPLETED

### Step 2 — GREEN: estampado v2 en el punto único (`put_record`)

- **Archivos:** `vanta-memory/src/core/record/l1_writer.rs` (releer fresco desde HEAD — coordinación MEMG-11)
- **Acción:** en `put_record` (mínimo diff): `valid_at_ms = rfc3339_to_epoch_ms(&record.created_at).filter(|ms| *ms > 0)` + `confidence_class: Some(ConfidenceClass::Asserted)` + `confidence: Some(default_confidence())` en el `MemoryInput`; helper privado `rfc3339_to_epoch_ms` (chrono, inverso de `epoch_ms_to_rfc3339`) con doc de la semántica (nacimiento del contenido, MGR-10 §D7); actualizar el doc-comment de `put_record` (semántica v2 write-side + single write point). NO tocar `apply_dedup_batch`/`write_memory`/`l1_batch` (paths de MEMG-11).
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test l1_semantics_v2 --build-jobs 2` → GREEN · suites existentes: `--test l1_dedup --test l1_contradiction --test capture_approval --test dreaming --test generation_log --test md_roundtrip --test md_git_e2e` verdes
- **Evidencia:** ✅ GREEN: `5 tests run: 5 passed` (incluye `batch_write_stamps_v2_fields_via_shared_builder` agregado tras la integración) · regresión: 8 suites `53 tests run: 53 passed, 1 skipped` · **integración MEMG-11 (coordinación):** su refactor en vuelo extrajo `record_input` como builder compartido y absorbió el estampado v2 en él (secuencial `put_record` + batch `apply_dedup_batch`→`put_batch`); `persist_planned` sigue delegando a `put_record`. El punto único quedó más fuerte que el diseño inicial (cubre ambos mecanismos).
- **Estado:** ✅ COMPLETED

### Step 3 — FINDs + verify full scoped + OCR delegation

- **Archivos:** `docs/dev/Backlog.md` (FIND-279 TTL, FIND-280 dream-promote restante), `docs/dev/tasks/MEMG-12.md` (§Spec/§Notas sincronizados)
- **Acción:** registrar los 2 FINDs con formato de `prompts/findings.md` (TTL semántico por tipo requiere MGR-09; derived class + T1b quarantine + T1c plumbing requieren parent linkage en `Dreamer` + write options `reason`/`by`); correr `cargo fmt --check` · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` · `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` · `pwsh scripts/validate-docs-coverage.ps1` · gates docs; OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) → revisar archivos por Rule Group (Critical/High bloquean; Medium → FIND).
- **Verify:** los comandos exit 0 + OCR sin Critical/High
- **Evidencia:** ✅ FIND-279/280 registrados (Backlog:442-443) · `cargo fmt --all -- --check` 0 · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` 0 · `cargo clippy -p vantadb -j 2 -- -D warnings` 0 (hook) · suite `-p vanta-memory`: `602 tests run: 602 passed, 2 skipped` · `validate-docs-coverage.ps1` 0 gaps · gates docs (check-links/check-docs/gen-index --check) exit 0 · OCR delegation ejecutado (2 Rule Groups): revisión contra group 2 sin Critical/High en el diff propio (sin unwrap/unsafe en prod; parse con fallback; sin API pública nueva).
- **Estado:** ✅ COMPLETED

### Step 4 — Review P2-01 + commit local + cierre campaign

- **Archivos:** `docs/dev/tasks/MEMG-12.md` (§Review + RESULTADO §7)
- **Acción:** clasificar tier HARD-02 del diff (paths `vanta-memory/**` + `docs/dev/**` → **Fast**; sin `src/sdk/**`); fork `vanta-review` con el diff (fresh context) si disponible; registrar veredicto en §Review; commit **LOCAL** `feat(memory):` con pathspec de archivos propios; `campaign_update_task_state(completed, taskId:"40")` con recitation + payload `review`.
- **Verify:** veredicto registrado + `git show --stat HEAD` limitado a archivos propios + `git status` sin WIP ajeno stageado
- **Estado:** ⬜ PENDING

## Dependencias

- SCH-02/04/05 ✅ (campos v2 + validación + cuarentena read-side), MEMG-01 ✅ (`10b35b41`), MEMG-02 ✅ (`fcc17ca7`) — archivos frescos desde HEAD antes de tocar.
- MGR-09 ❌ (retention sin research-doc) — no bloqueante: TTL → FIND (política mínima declarada).
- MEMG-11 ⏳ (en vuelo, mismo worktree) — coordinación de paths documentada en §Notas; no solapamiento de funciones.
- nextTask: MEMG-13 (Task 41) — lo decide el orquestador.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review — P2-01, contexto fresco (no participó de la implementación); tier **Fast** (HARD-02: `vanta-memory/**` + `docs/dev/**`; sin `src/sdk/**` propio). Sesión reviewer: `ses_ef4f06c1bffeupwI1MGt7CCS6t`. Veredicto: ✅ **APPROVE** (sin Critical/High; 3 hallazgos Low, ninguno bloquea commit).
- **Enfoque:** contrato re-verificado comando por comando por el reviewer; RED creíble comprobado (`git show HEAD:...l1_writer.rs` sin `valid_at_ms`/`confidence` → default write-time real vs T0 2023); ambos sitios L1 pasan por el builder único (`put_record`←`persist_planned` y `promote_dream_run`); batch vía `record_input`+`put_batch` (test batch lo prueba); fallback seguro (parse fail/pre-1970/0 → `None` → default core, nunca `Some(0)`); wire/payload intactos; símbolos nuevos privados; sin unwrap/unsafe en prod; `chrono` ya era dep directa; FIND-279/280 correctos sin sobre-declarar; política defendible contra el stop de Task 40 (plan L1158).
- **Cómo se probó:** reviewer re-corrió `cargo nextest run --profile audit -p vanta-memory --test l1_semantics_v2 --build-jobs 2` (5/5 en el momento del review) · `cargo fmt --all -- --check` 0 · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` 0 · `git show HEAD:` para el RED.
- **Hallazgos (Low) + disposición:**
  1. Doc fusionado en `l1_writer.rs:478-485` (co-autoría MEMG-11: el bloque describe ambos paths) → **documentado** en §Notas (suficiente; no se toca el archivo co-autoreado por cosmética).
  2. Test gap: fallback de birth inválido + re-persist de contradicción sin cobertura → **cerrado**: 2 tests agregados post-approve (`unparseable_birth_falls_back_to_core_default`, `contradiction_re_persist_keeps_old_record_birth`); re-verificado 7/7.
  3. Re-persist de contradicción resetea a D_a una confidence decayada por MEMG-02 (pre-existente, byte-idéntico pre/post MEMG-12) → **FIND-281** registrado.
- **Post-approve micro-additions (dentro de la dirección aprobada):** 2 tests + FIND-281; re-verificados: `l1_semantics_v2` **7/7** · `cargo fmt --all -- --check` 0.
- **Veredicto:** ✅ **APPROVE** — contrato sostenido con evidencia reproducible; listo para commit local (Step 4, sin push).

## Context Save Point

- **Última acción:** Steps 1-3 ✅ (RED 4/4 → GREEN 5/5; regresión 53/53; suite 602/602; fmt/clippy/docs/coverage 0; FIND-279/280; OCR sin Critical/High). Step 4 en curso (review P2-01 → commit).
- **Próximo paso:** fork `vanta-review` (tier Fast) → §Review → commit LOCAL `feat(memory):` con pathspec (`vanta-memory/src/core/record/l1_writer.rs`, `vanta-memory/tests/l1_semantics_v2.rs`, `docs/dev/tasks/MEMG-12.md`, `docs/dev/Backlog.md`) → campaign completed taskId `40`.
- **Estado del worktree:** cambios propios + co-autoría MEMG-11 en `l1_writer.rs` (integrada y verde); WIP ajeno NO se stagea (`opencode.jsonc`, plan maestro, `src/sdk/api/memory.rs`, `auto_recall.rs`, tests untracked de MEMG-11).

## Notas

- **Política declarada (pre-mortem #1 — MGR-09 ausente):** `[a verificar]` — TTL semántico por tipo NO se implementa hasta que exista el research-doc de retention (MGR-09). Política mínima vigente: L1 sin TTL (gobierno por heat/lifecycle MEM-60); artifacts de pipeline (`__assembled` `pipeline_worker.rs:775`, `run.json` `dream/mod.rs:537`, L0 `l0_recorder.rs:243`) sin TTL — se sobreescriben por key o son auditables. FIND-279 registra el gap.
- **Doble escritura de confianza (pre-mortem #2):** resuelto por diseño — `put_record` es el único sitio que estampa; extracción y promoción heredan la política asserted/D_a. Cuando derived aterrice (FIND-280), el stamp vendrá como valor explícito aguas arriba (firma extendida), nunca un segundo punto de escritura.
- **Cuarentena (pre-mortem #3):** solape MGR-06/MEMG-01 evitado — este slice NO marca cuarentena; T1b (derived_promotion) queda en FIND-280 con la spec exacta (MGR-13 §3.2) y el requisito técnico identificado.
- **Coordinación MEMG-11 (mismo worktree, en vuelo) — EVENTO REAL DOCUMENTADO:** durante el Step 2, MEMG-11 editó `l1_writer.rs` en paralelo (su Step 1: `plan_write`/`record_input`/`apply_dedup_batch`→`put_batch`) y su save intermedio rompió el compile en dos ocasiones (`record_input` y `search_records_core` ausentes, ~1-2 min cada uno); se esperó acotadamente y se reintentó (sin tocar su WIP). Al completar su save, **integró el estampado v2 dentro de su builder compartido `record_input`** (que ahora usan `put_record` y el path batch): el punto único quedó más fuerte que el diseño inicial y ambos paths escriben la semántica v2. Verificado verde con su test (`l1_batch_write`) + suite completa. **Consecuencia para el commit:** `l1_writer.rs` es co-autoreado — el commit de MEMG-12 lo incluye en su estado integrado (documentado en el body del commit); MEMG-11 conserva `memory.rs`/`auto_recall.rs`/sus tests para su commit propio.
- **WIP ajeno:** `opencode.jsonc` + master plan modificados en el árbol por otros — NO se stagean. PROHIBIDO tocar `docs/pipeline-state.json`.
- **Disco:** si el linker falla por espacio → `dev-tools/target-cleanup.ps1 -Clean -Yes` (patrón MEMG-02).
- **NOTICED BUT NOT TOUCHING:** los otros 3 sitios `ttl_ms: None` (L0/assembled/run.json) quedan como están (política mínima declarada); `MemoryInput.quarantine: bool` no expresa `reason`/`by` (FIND-280); el doc-block de `record_input` quedó fusionado (describe ambos paths — suficiente).

## RESULTADO §7 (contrato de retorno — pipeline-full)

> Se completa al cierre (Step 4).
