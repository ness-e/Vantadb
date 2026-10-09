---
title: "TASK MEMG-17: Rollback + verificabilidad + erasure criptográfica"
kind: task
description: "Rollback semántico de L1 a versión/snapshot con linaje declarado + recibos verificables de erasure (contrato VER-02, sin firma — no sancionada) + erasure criptográfica por destrucción de DEK (registry 32B CSPRNG wrapped con master Cipher) con scope declarado; tests por pieza"
---

# TASK MEMG-17: Rollback + verificabilidad + erasure criptográfica

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 55, bloque F0-expandido L1576-1602)
- **Fuente:** Backlog `MEMG-17` + plan Task 55 + MEMG-13 (versions/snapshot ✅) + VER-01 (hash-chain ✅) + VER-02 (certificados ✅) + FIND-287 (restore data-only) + FIND-194 (rotación master key)
- **Esfuerzo:** 🟠 3-5d | **Appetite:** max 1sem | **Stop (plan L1587):** 5d sin las 3 piezas → entregar (a) rollback semántico con linaje + test sobre versions/snapshot + FIND de recibos/erasure
- **Prioridad:** 🟠
- **Tipo:** Rust (crate `vanta-memory`; core consumido — 1 cambio de visibilidad `pub` + 1 firma crate-interna)
- **Turns estimados:** 12-18 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `55` en el campaign server)
- **Incógnitas (uphill):** 0 — resueltas en DISCOVERY:
  - **Crypto (pre-mortem #1):** el modelo existente es master key (`VANTADB_ENCRYPTION_KEY`, raw 32B o passphrase) + `Cipher::derive_namespace` HKDF-SHA256 POR NAMESPACE (`src/crypto.rs:216`, VER-03). **Clave HKDF-derivada NO es destruible** (se re-deriva del master) → un DEK registry con claves aleatorias por scope **wrapped** con el master es la pieza genuinamente faltante; no se inventa un modelo cripto completo (se compone con `Cipher` existente).
  - **Firma ML-DSA-65:** 0 hits de ML-DSA/Dilithium/sign en el repo; `src/attestation.rs:17` delega firma a `vanta-audit` (VER-01 §Diseño). **No sancionada por spec → NO se firma**: el recibo usa el contrato VER-02 (schema + sha256 self-hash + re-scan no claim-driven + chain VER-01 referenciada), declarado en `out_of_scope`.
- **Pendientes (downhill):** 5 steps
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `55`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Superficies nuevas (aditivas):** `core::record::rollback::{rollback_record, RollbackReport}`; `utils::backup::rollback_snapshot` (+`SnapshotRollbackReport`); `utils::erasure::{create_scope, seal, open, erase_scope, verify_erasure_receipt, ErasureReceipt, ErasureVerification, ErasureError}`. **Callers existentes que NO cambian de firma:** `read_record*`/`diff_records` (l1_reader), `write_memory`/`apply_dedup_batch` (l1_writer), `utils::backup::{create_snapshot,list_snapshots,restore_snapshot}` (MEMG-13), `Embedded::delete/put/versions` (core, solo consumo). |
| Callees | Core existente: `Embedded::{put, get, delete, versions, get_version}` (`src/sdk/api/memory.rs:597,1108,1142,1148`), `version_history::purge_key` (vía delete), `crypto::Cipher` (`src/crypto.rs:161-310`), `attestation::{ChainEvidence, CertificateIntegrity, SurfaceReport}` (tipos públicos VER-02), `wal::WAL_FORMAT_VERSION` (re-export público). `put_record` (crate) reutilizado por el rollback. Cero dependencias nuevas fuera del lockfile: `sha2 0.11` y `rand 0.9` (mismas versiones que el core; ya resueltas en `Cargo.lock`). |
| Implicaciones | **API pública aditiva** en `vanta-memory` — sin breaking, sin wire, sin migración de datos. **2 cambios en el core, mínimos:** (1) `attestation::chain_evidence()` privado → `pub` (builder de `ChainEvidence`; additive, single source of truth para los recibos); (2) `l1_writer::put_record` pasa de `Result<(), L1Error>` a `Result<MemoryRecord, L1Error>` (crate-interno; sus 3 callers — `persist_planned`, `mark_contradicted_targets`, `dream::promote` — descartan el Ok con `?;`, compilan sin cambio). Erasure borra la DEK vía `Embedded::delete` (el delete del core ya purga version history — VS-CORE-07). Sin locks nuevos, sin cambios de concurrencia. Regresión: suites `-p vanta-memory` + `-p vantadb` (attestation) se re-corren. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `c5a7bc62`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `vanta-memory/src/core/record/l1_reader.rs` (:113-224 — `RecordVersion`/`read_record_versions`/`read_record_version`/`diff_records`; :24-26 `l1_namespace`), `l1_writer.rs` (:1-39 `L1Error`; :127-291 `write_memory`/`plan_write`/`persist_planned`; :385-453 `apply_dedup_batch`; :473-533 `record_input`/`put_record`; :540-557 helpers), `core/record/mod.rs` (re-exports), `core/abstractions/types.rs` (:44-130 `MemoryRecord`), `core/dream/mod.rs` (:900-920 caller de `put_record`), `core/conversation/l0_recorder.rs` (:140-160 `sanitize_component`/`sanitize_key`).
  - `vanta-memory/src/utils/backup.rs` (:1-57 contrato MEMG-13 + FIND-287), `utils/mod.rs` (re-exports), `tests/l1_history.rs` (:1-64 fixture `record`/`put_l1`), `tests/backup_snapshot.rs` (:1-45 Fjall+tempdir), `Cargo.toml` (deps/features), `src/lib.rs`.
  - Core: `src/attestation.rs` (:1-62 scope/integrity + `SurfaceReport`; :63-157 `ChainEvidence`/`CertificateIntegrity`/`PurgeCertificate`/`declared_limits`/`chain_evidence`; :346-468 `canonical_bytes`/`validate_schema`/`finalize`/`build_certificate`; :497-634 `verify_certificate` no claim-driven), `src/crypto.rs` (:141-234 `Cipher`/`from_env`/`derive_namespace`; :257-373 encrypt/decrypt framing), `src/sdk/api/memory.rs` (:560-593 purge_expired, :595-679 `put_one` — versión core = existing+1 + snapshot best-effort; :1095-1113 `versions`; :1141-1174 `delete_inner` — purga version history; :1176-1243 `delete_certified`/`verify_purge_certificate`), `src/sdk/version_history.rs` (:37-417 `version_prefix`/`get_version`/`versions`/`purge_key`/`evict_overflow`), `src/sdk/builder.rs` (:269-300 snapshot API), `src/storage/engine/mod.rs` (:644-763 create/list/validate/restore), `src/error.rs` (:226-241 `NotFound{kind,id}`/`Validation{field,reason}`; :526 `Result`), `src/wal.rs:69` (`is_chained` pub(crate)), `Cargo.toml` (workspace: `rand=0.9` L85, `sha2=0.11` L136).
- **Archivos referenciados hacia dentro (imports/deps):** `rollback.rs` (nuevo) → `l1_reader::{read_record, read_record_version, diff_records, l1_namespace, RecordFieldChange}`, `l1_writer::put_record` (crate), `l1_extraction::epoch_ms_to_rfc3339` (crate), `vantadb::error::Error`. `erasure.rs` (nuevo) → `vantadb::{crypto::Cipher, sdk::Embedded, attestation::{ChainEvidence, CertificateIntegrity, SurfaceReport}}`, `l0_recorder::sanitize_key` (crate), `sha2`/`rand`/`chrono`. `backup.rs` (+`rollback_snapshot`) → `Embedded::restore_from` (ya importado).
- **Referencias entrantes (grep/CodeGraph HEAD):** `rollback`/`erasure`/`DEK` = **0 hits** en el path de memoria (re-verificado en DISCOVERY; confirma el gap). Callers de `put_record` = 3 (todos `?;`, descartan Ok). `chain_evidence` = 1 caller interno (`build_certificate`). `utils::*` re-exporta lo nuevo (aditivo). Ningún export existente cambia de nombre.
- **Veredicto impacto:** **MEDIO-BAJO (aditivo con 2 micro-cambios core)** — 3 archivos nuevos de código en `vanta-memory` (`rollback.rs`, `erasure.rs`, `tests/{rollback,erasure}.rs`), 4 archivos tocados mínimamente (`l1_writer.rs` firma + `mod.rs`/`utils/mod.rs` re-export + `backup.rs` función + `Cargo.toml` deps + `src/attestation.rs` visibilidad). Sin breaking, sin wire, sin deps nuevas fuera del lockfile, sin locks. Pre-mortem mitigado: (1) crypto verificado → DEK registry compuesto con `Cipher`; (2) FIND-287 declarado explícito en `SnapshotRollbackReport.not_reverted`; (3) orden rollback → erasure → recibos con stop.

## Contrato

"**(a) rollback semántico** a versión/snapshot con linaje (reusa versions/snapshot; declara explícitamente qué revierte y qué no — caveat FIND-287: el restore de snapshot es data-only, el KV live conserva tombstones) + **(b)** recibos verificables extendiendo hash-chain VER-01 con el contrato de verificación de los certificados existentes (firma ML-DSA-65 solo si la spec lo sanciona → [a verificar] = NO sancionada, 0 hits) + **(c)** erasure criptográfica (destrucción DEK + tombstone; scope declarado) — **cada pieza con test propio; sin romper certificados/verify existentes (suites verdes)**" (plan Task 55 L1585).

Verificación: tests RED→GREEN por pieza (`tests/rollback.rs`, `tests/erasure.rs`) + suites scoped `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` + `-p vantadb --test attestation` (o el binario que cubra `attestation`) + `cargo fmt --check` + `cargo clippy -p vanta-memory -p vantadb --all-targets -- -D warnings`.

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): pre-respondido por el plan F0** — el contrato Task 55 (Gate Result ✅ DO) sanciona las 3 piezas y sus superficies ("(a) rollback semántico…; (b) recibos verificables…; (c) erasure criptográfica… cada pieza con test propio"); el stop (L1587) fija el corte. Precedente idéntico: MEMG-13 ("Gate D: pre-respondido por el plan F0"). Sin símbolos fuera de la sanción.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Alcance de las 3 piezas | A) **las 3, en orden rollback → erasure → recibos, con stop a la primera incompleta** / B) solo (a) + FIND | ✅ **A** — contrato L1585 las pide; stop L1587 autoriza degradar a (a)+FIND si no llegan |
| 2 | Forma del rollback de versión | A) **append-only: escribe el estado de la versión target como versión NUEVA (linaje visible en `versions()`), report con diff `live→target` + `from/to/new_version`** / B) overwrite/borrado de versiones (pierde el linaje — viola el propósito de auditoría) / C) solo `get_version` (ya existe — no hay "rollback") | ✅ **A** — decidido-por-evidencia: `put_one` auto-incrementa versión core y snapshotea la previa (`memory.rs:615-618`); `diff_records` (MEMG-13) provee el diff; el linaje es el punto del contrato |
| 3 | Semántica de supersession en rollback | A) **restaura el payload target verbatim (incl. `superseded_by`) y lo declara en `out_of_scope`** / B) limpia `superseded_by` (revive) — cambio semántico no sancionado | ✅ **A** — "declara explícitamente qué revierte y qué NO" (L1585); time-travel fiel; B decidiría semántica de lifecycle sin spec |
| 4 | Rollback de snapshot | A) **`rollback_snapshot` = `restore_snapshot` (MEMG-13) + `SnapshotRollbackReport` con `reverted`/`not_reverted`/`caveats` (FIND-287 explícito)** / B) reimplementar en `vanta-memory` el restore del backend KV (FIND-287 es de Arch/Engine; frontera de dominio) | ✅ **A** — el restore existe y está certificado; lo que falta es la **declaración explícita** de alcance (caveat del plan) |
| 5 | Modelo crypto para erasure | A) **registry de DEK aleatorios (32B CSPRNG) por scope, wrapped con master `Cipher` (AES-256-GCM) en namespace reservado `erasure/dek`; destrucción = `delete` (tombstone + purge versions) + recibo** / B) reusar `derive_namespace` HKDF (VER-03) — **no destruible** (se re-deriva del master) / C) capa cripto completa integrada al write path L1 (scope inviable en 1sem) | ✅ **A** — pre-mortem #1 ("diseñar en el slice, no inventar modelo completo"): A se **compone** con `Cipher` existente (unchanged) sin tocar el write path; la integración L1 (envelope por registro) queda como deuda evaluable (FIND) |
| 6 | Forma del recibo (b) | A) **`ErasureReceipt` espejando el contrato VER-02: schema version + `out_of_scope` no-vacío + superficies canónicas exactas + sha256 canonical + `ChainEvidence` VER-01 + verificación no claim-driven (re-scan live); reutiliza los tipos públicos `ChainEvidence`/`CertificateIntegrity`/`SurfaceReport`** / B) firma ML-DSA-65 — **0 hits en repo, sin spec que la sancione** (`attestation.rs:17` delega a vanta-audit) / C) reusar `PurgeCertificate` tal cual (habla de node/namespace de memoria, no de DEK/scope) | ✅ **A** — L1585: "con el contrato de verificación de los certificados existentes; firma solo si la spec lo sanciona" — no lo hace |
| 7 | `chain_evidence()` en core | A) **`pub` (1 línea, additive; single source para scheme/format/version)** / B) duplicar el builder en `vanta-memory` (drift si el WAL cambia) | ✅ **A** — `is_chained` es `pub(crate)` (`wal.rs:69`) → B requeriría más superficie; A replica exactamente lo que `build_certificate` emite |
| 8 | Scope del erasure | A) **declarado: erasure criptográfica de payloads sellados por `seal` bajo el DEK del scope; blobs caller-held; integración al write path L1 → FIND** / B) erasure "de todo el namespace L1" sin cifrado real (claim falso) | ✅ **A** — honestidad VER-02 ("logical purge, not secure erase" + límites explícitos); B sería claim-driven |
| 9 | Semántica de errores | A) **`L1Error` existente para rollback (`Error::NotFound{kind,id}` para record/versión ausente); `ErasureError` nuevo (thiserror) para el vault/recibo** / B) tipos nuevos para todo (ceremonia) | ✅ **A** — decisión #5 de MEMG-13 (L1Error es el error de la capa L1); el erasure tiene semántica propia (DEK ausente/corrupto) que no mapea a L1 |
| 10 | Tests por pieza | A) **integración: `tests/rollback.rs` (InMemory para versión + Fjall/tempdir para snapshot) y `tests/erasure.rs` (InMemory); RED→GREEN por step** / B) unit inline (no prueban el flujo SDK) | ✅ **A** — MEMG-13 decisión #6: el crate usa integration tests para todo flujo SDK |
| 11 | Timestamps / wire | `chrono` (ya dep) RFC3339 UTC; recibo NO contiene material de clave (solo scope/estado/evidencia); payload L1 intacto | ✅ — verificado: `chrono` en `vanta-memory/Cargo.toml`; sin campos nuevos en `MemoryRecord` |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **Firmas intactas de lo existente:** `read_record*`/`diff_records`/`write_memory`/`apply_dedup_batch`/`backup::{create_snapshot,list_snapshots,restore_snapshot}` no cambian. Solo se AGREGA.
  2. **Wire/payload L1 intacto** — sin campos nuevos en `MemoryRecord`; la historia se lee de snapshots del core sin tocar su formato.
  3. **Gate de cuarentena** (SCH-05) intacto en recall: el rollback es operación de auditoría/admin explícita (como `versions`) — declara que escribe vía `put_record` (no pasa por recall).
  4. **Snapshot:** no reimplementar; el restore existente es autoritativo (close→restore→reopen); el report solo declara alcance (FIND-287).
  5. `src/wal.rs`, `src/storage/**`, `src/vector/**` **no se tocan**; `src/attestation.rs` solo cambia `chain_evidence` a `pub` (additive). `src/sdk/**` solo consumo.
  6. Sin `unwrap`/`expect`/`unsafe` en código nuevo de producción; deps nuevas SOLO `sha2 0.11` + `rand 0.9` (mismas versiones del core, ya en `Cargo.lock`).
  7. **Recibos nunca emitidos estructuralmente inválidos** (`finalize` patrón VER-02); verificación **nunca claim-driven** (schema primero, re-scan después).
  8. **WIP ajeno:** `opencode.jsonc` + master plan + archivos de MEMG-10 (`vanta-proxy/**`, `vanta-memory/src/core/hooks/*`, `vanta-memory/tests/memg10_trust_gate.rs`, `vantadb-mcp/src/config.rs`) en el árbol por otros — NO se stagean; PROHIBIDO tocar `docs/pipeline-state.json`; commit con **pathspec**.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vanta-memory --test rollback --test erasure --build-jobs 2` · `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` · `cargo nextest run --profile audit -p vantadb --lib` · `cargo fmt --check` · `cargo clippy -p vanta-memory -p vantadb --all-targets -- -D warnings`.
- **Deuda pendiente:** integración envelope/DEK al write path L1 (FIND); firma criptográfica ML-DSA-65 (vanta-audit, VER-01 §Diseño); restore de backend KV en snapshot (FIND-287, Arch/Engine).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — no hay `unsafe`, no hay clones en hot paths (operaciones admin/auditoría), no hay abstracciones especulativas (DEK registry = primitiva pedida; se compone con `Cipher` y tipos VER-02 existentes, no los reimplementa). El cambio **elimina** deuda de "no hay rollback/erasure/recibos" (gap Backlog MEMG-17) y **agrega tests de contrato** por pieza. Deps: `sha2`/`rand` ya presentes en el lockfile (misma versión que core) — sin peso nuevo de supply chain.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | (a) rollback de versión RED→GREEN con linaje (`from/to/new` + diff + `out_of_scope`) + rollback de snapshot con report declarado (FIND-287 explícito) + (c) erasure: DEK registry + `seal`/`open` + `erase_scope` (tombstone + purge) RED→GREEN + (b) recibo verificable (schema + integrity + re-scan no claim-driven; tamper y re-creación detectados) RED→GREEN + suites scoped verdes + fmt/clippy verdes |
| **Commit** | Commit atómico conventional `feat(memory):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | Changelog release-plz (feature → minor); verify full scoped documentado (workspace completo no se corre por presupuesto) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius) + `codebase-memory-mcp_check_index_coverage` (7 paths, `no_recorded_issue` ✅) + grep puntual
- `cargo nextest` scoped por crate (loop TDD) + `campaign_verify_cmd` (verify mecánico)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- `vanta-review` (subagente, review P2-01 fresh-context) al cierre
- Gates docs (task file + `docs/api/VANTA_MEMORY.md` editados): `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check`

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` · `test-driven-development` · `context-engineering` (SDP phase=BUILD) + rol: `security-and-hardening`, `rust-write-tests`, `documentation-and-adrs`, `documentation-skill` (docs). `performance-optimization` excluida (no hot path — Regla 9 no dispara).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — trust boundaries nuevos: (1) `scope`/`reason` strings del caller → sanitizados (`sanitize_key`, 512B) antes de tocar store; (2) DEK: 32B CSPRNG (`rand::rng()`), **nunca** logueado, **nunca** en el recibo (el recibo solo declara estado/scope/evidencia); wrapping AES-256-GCM vía `Cipher` existente (no se implementa primitiva cripto nueva); (3) `seal`/`open` mapean `CryptoError` sin filtrar detalles de clave; (4) rollback: `record_id`/`session_key` por helpers existentes; `target_version` validado por lookup (ausente → NotFound). Checklist `security-and-hardening` sobre estos puntos al cierre.
- [ ] **PERFORMANCE** — no aplica: operaciones admin/auditoría explícitas fuera del path de recall; sin loops nuevos en hot paths; `diff_records` O(campos); erasure O(1) store + scan de prefix corto. Regla 9 no dispara.

## Review (P2-01 — fresh context)

**Ronda 1** (vanta-review, sesión `ses_ef20ad433ffefWeG1953lUTVFi`): 🔴 changes-required — 1 Critical (`tests/erasure.rs` sin `#![cfg(feature = "erasure")]`: el build default/CI sin feature no compilaba — reproducido por el reviewer con el comando del contrato) + 2 Low (evidence stale; `declared_limits` sin los derived indexes ni la nota de durabilidad). Probes adversariales sin hallazgos en las 3 piezas (linaje append-only real, verificación no claim-driven con paridad VER-02, DEK destruida + panic-free, sin key material).

**Fixes ronda 1:** gating del test (1 línea) + evidencia actualizada + nota de durabilidad cross-process en `erase_scope` (paridad `delete_certified`) + línea de derived indexes (text/sparse no re-escaneados) en `declared_limits`.

**Ronda 2** (mismo reviewer, re-verificación adversarial): **✅ APPROVE** — los comandos exactos que fallaron ahora pasan (default **701/701**, feature **712/712**, erasure 11/11, attestation 10/10); matrix fmt/clippy/docs re-ejecutada ✅; nota informativa no-gate: 3 links intra-doc a símbolos privados en rustdoc (jobs de rustdoc verificados en verde por el reviewer — sin fix requerido).

Revisor: vanta-review (fresh context, P2-01, rondas 1-2) · Enfoque: contrato L1585 + las 3 piezas, evidencia re-ejecutada por el revisor · Cómo se probó: nextest default/feature/attestation + fmt/clippy/docs re-run · Veredicto: ✅ APPROVE (0 Critical abiertos).

## Steps

### Step 1 — RED+GREEN: (a) rollback de versión con linaje

- **Archivos:** `vanta-memory/tests/rollback.rs` (nuevo), `vanta-memory/src/core/record/rollback.rs` (nuevo), `vanta-memory/src/core/record/mod.rs` (re-export), `vanta-memory/src/core/record/l1_writer.rs` (`put_record` devuelve `MemoryRecord`)
- **Acción:** RED — tests de contrato (InMemory, fixture `record`/`put_l1` de `l1_history.rs`): (1) `rollback_restores_target_content_as_new_version_with_lineage` — v1 A + v2 B + v3 C → `rollback_record(…, 1)` → live content A, `read_record_versions` = 4 versiones ascendentes (v4 = target + bookkeeping), report `{from:3, to:1, new:4, changes: content C→A, out_of_scope no-vacío}`; (2) `rollback_missing_version_is_not_found` — target 99 → Err NotFound; (3) `rollback_deleted_record_is_not_found` — `db.delete` (purga versions) → Err NotFound (no resucita); (4) `rollback_to_superseded_target_declares_supersession` — target con `superseded_by=Some` → payload restaurado lo conserva y `out_of_scope` lo declara; (5) `rollback_missing_record_is_not_found`. GREEN — `rollback.rs`: `RollbackReport` + `rollback_record(db, session_key, record_id, target_version, now_ms)`: lee live (`read_record`) y target (`read_record_version`), clona target con `version = live.version+1`, `updated_at = now`, `timestamps` target ∪ {now} (BTreeSet), escribe vía `put_record` (vector target aparte), lee el nuevo tip (`read_record_versions().last()`) y arma report (`changes = diff_records(live, target)`, `out_of_scope` declarado). `l1_writer::put_record` → `Result<MemoryRecord, L1Error>` (3 callers `?;` intactos).
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test rollback --build-jobs 2` → RED correcto (compile error por símbolos ausentes) → GREEN.
- **Evidencia:** ✅ RED verificado: `error[E0432]: unresolved import vanta_memory::core::record::rollback` (falla por la razón correcta — símbolo ausente). ✅ GREEN: `5 tests run: 5 passed` — linaje append-only (3→4 versiones, v4 = target + bookkeeping, `from:3/to:1/new:4`), delta de store `content C→A`, supersession verbatim declarada, 3 casos NotFound (versión ausente / registro ausente / registro borrado — no resucita). `cargo fmt --check` ✅ · `cargo clippy -p vanta-memory --lib -D warnings` ✅.

### Step 2 — RED+GREEN: (a) rollback de snapshot declarado (FIND-287)

- **Archivos:** `vanta-memory/tests/rollback.rs` (extiende), `vanta-memory/src/utils/backup.rs` (`rollback_snapshot` + `SnapshotRollbackReport`)
- **Acción:** RED — (6) `snapshot_rollback_declares_data_only_scope` — Fjall+tempdir: snapshot → mutación post-snapshot (add m3 + delete m1 + supersession) → `db.close()` → `rollback_snapshot(config, "snap-1")` → report: `snapshot=="snap-1"`, `reverted` menciona `data/`, `not_reverted` menciona backend KV/tombstones (FIND-287) y version history, `caveats` no-vacío; DB reabierta = estado pre-snapshot para `data/` (m1 presente, m3 ausente). (7) `snapshot_rollback_missing_is_not_found`. GREEN — wrapper sobre `restore_snapshot` que arma el report estático declarado (constantes), sin reimplementar restore.
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test rollback --build-jobs 2` → RED → GREEN.
- **Evidencia:** ✅ RED verificado: `error[E0432]: no rollback_snapshot in utils::backup`. ✅ GREEN: `7 tests run: 7 passed` — `SnapshotRollbackReport` declara `reverted` (swap `data/` + rebuild), `not_reverted` (backend KV tombstones + version history + audit/WAL, **FIND-287 citado**) y `caveats`; comportamiento FIND-287 pinneado por la superficie nueva (m1 sigue borrado, m3 desaparece, m2 vuelve → `["beta"]`), NotFound para snapshot ausente.

### Step 3 — RED+GREEN: (c) erasure — DEK vault + seal/open + erase

- **Archivos:** `vanta-memory/tests/erasure.rs` (nuevo), `vanta-memory/src/utils/erasure.rs` (nuevo), `vanta-memory/src/utils/mod.rs` (re-export), `vanta-memory/Cargo.toml` (`sha2`+`rand`), `src/attestation.rs` (`chain_evidence` → `pub`)
- **Acción:** RED — (1) `seal_open_roundtrip_binds_scope` — create_scope("agent-1") + seal/open roundtrip; open con scope distinto → `DekNotFound`; tamper → `DecryptFailed`; (2) `create_scope_twice_is_rejected`; (3) `seal_without_scope_is_not_found`; (4) `erase_scope_destroys_dek_and_tombstones` — erase → `status=="erased"`, `db.get(dek)` = None, `db.versions(dek)` = 0, `open` → `DekNotFound`; (5) `erase_without_dek_is_not_found` (recibo not_found). GREEN — `erasure.rs`: `ERASURE_DEK_NAMESPACE="erasure/dek"`, `ErasureError`, `create_scope` (DEK 32B `rand::rng()` → wrap `master.encrypt` → record hex), `seal`/`open` (`Cipher::new(&dek)`), `erase_scope` (`db.delete` + recibo). Receipt mínimo (sin verify aún) con superficies canónicas `["dek_record","version_history","wal"]`, `out_of_scope` declarado, `finalize`. `chain_evidence()` pub en core.
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test erasure --build-jobs 2` → RED → GREEN.
- **Evidencia:** ✅ RED verificado: `error[E0432]: unresolved import ... erasure` + **descubrimiento**: `vantadb::crypto` está tras feature `encryption` del core → el módulo quedó feature-gated (`erasure = ["vantadb/encryption", "dep:rand", "dep:sha2"]`, opt-in; build default intacto). ✅ GREEN: `5 tests run: 5 passed` — seal/open bindeado al scope (scope ajeno → `DecryptFailed`; tamper AEAD → `DecryptFailed`), `ScopeExists` (no rekey), `DekNotFound`, erase destruye la DEK (record live ausente + version history vacía vía purge del delete) y el blob queda irrecuperable; recibo `not_found` con límites declarados para scope ausente. **Post-OCR (fix):** +1 test de regresión `corrupted_dek_payload_is_malformed_not_panic` — la revisión OCR detectó panic potencial en `decode_hex` (slicing de `&str` sobre payload corrupto multibyte); fix byte-wise (`chunks_exact` + `char::to_digit`, nunca paniquea) + test (`6 tests` del slice c).

### Step 4 — RED+GREEN: (b) recibo verificable (contrato VER-02, no claim-driven)

- **Archivos:** `vanta-memory/tests/erasure.rs` (extiende), `vanta-memory/src/utils/erasure.rs`
- **Acción:** RED — (6) `receipt_verifies_against_live_state` — erase → `verify_erasure_receipt(ok)` (schema+integrity+re-scan, `residues_now=0`); (7) `receipt_integrity_detects_edits` — mutar `status`/`surfaces` en el JSON → Err integrity; (8) `receipt_rejects_unknown_status_and_partial_surfaces` (schema no claim-driven: status desconocido / superficie faltante / `out_of_scope` vacío con hash recomputado → rechazo); (9) `receipt_detects_recreated_scope` — erase → verify ok → re-create scope (mismo nombre) → verify → Err "residues reappeared". GREEN — `verify_erasure_receipt` espejando `verify_certificate` (:497-634): schema_version → canonical sha256 → `validate_schema` → re-scan (`dek_record` via `db.get`, `version_history` via `db.versions`) → verdict.
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test erasure --build-jobs 2` → RED → GREEN.
- **Evidencia:** ✅ RED verificado (rename temporal del símbolo): `error[E0432]: unresolved import erasure::verify_erasure_receipt`. ✅ GREEN: `10 tests run: 10 passed` (5+5) — verificación no claim-driven: verify ok contra estado live (`residues_now=0`, superficies re-escaneadas), integridad detecta edición, schema rechaza superficie parcial y status desconocido **aun con hash recomputado** (paridad atacante), y re-creación del scope rompe el claim (`residues reappeared`). **Post-review P2-01 (ronda 1) — fixes aplicados:** (1) **CRITICAL**: `tests/erasure.rs` sin `#![cfg(feature = "erasure")]` rompía el build default/CI (E0432 sin feature) → gating de un archivo + re-verificado: build **default 701/701** y con feature **712/712** (46 binarios, 2 skipped); (2) evidence actualizada (este addendum); (3) `erase_scope` doc gana la nota de durabilidad cross-process (paridad `delete_certified`) y `declared_limits` declara los derived indexes (text/sparse) no re-escaneados por el recibo. `-p vantadb --lib` attestation 10/10 · `cargo check -p vantadb` ✅ · clippy `--all-targets --all-features -D warnings` ✅ · fmt ✅.

### Step 5 — Cierre: docs + verify full scoped + OCR + review P2-01 + commit

- **Archivos:** `docs/api/VANTA_MEMORY.md` (sección nueva), `docs/dev/tasks/MEMG-17.md` (evidencia), (+ Backlog: fila FIND si hay diferidos)
- **Acción:** docs (EN, sección "Rollback, erasure & receipts (MEMG-17)" + feature flags n/a + debts); verify scoped completo (`-p vanta-memory` suite + `-p vantadb --lib` + fmt + clippy); OCR delegation; review P2-01 fork `vanta-review`; commit local pathspec `feat(memory):`; campaign close taskId `55`; skill progreso.
- **Verify:** commands del contrato + `dev-tools/verify_changed.ps1` si aplica.
- **Evidencia:** ✅ Docs (VANTA_MEMORY.md: sección "Rollback, erasure & receipts (MEMG-17)" + feature flag `erasure` + namespace `erasure/dek`; Backlog: FIND-302; task file + index/llms regenerados — gates ✅). Verify full scoped: **default 701/701** · **--features erasure 712/712** · rollback 7/7 · erasure 11/11 · attestation VER-02 10/10 · `cargo fmt --check` ✅ · clippy `--all-targets --all-features -D warnings` ✅ · `scripts/validate-docs-coverage.ps1` 0 gaps ✅ · `gen-index --check` ✅. OCR delegation ejecutada (hallazgo real corregido: `decode_hex` panic-free + test). Review P2-01 ronda 1→2 **APPROVE**. Commits **LOCALES**: `80cb84b0` (feat) + `docs(task)` de esta sección (⛔ sin push).

## RESULTADO (sección 7 - contrato de retorno)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 5/5 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: 80cb84b0 (+ docs(task) de esta sección)
ARCHIVOS: vanta-memory/src/core/record/{rollback.rs, l1_writer.rs, mod.rs} · vanta-memory/src/utils/{backup.rs, erasure.rs, mod.rs} · vanta-memory/Cargo.toml · Cargo.lock · vanta-memory/tests/{rollback.rs, erasure.rs} · src/attestation.rs · docs/api/VANTA_MEMORY.md · docs/dev/Backlog.md (FIND-302) · docs/dev/tasks/MEMG-17.md · docs/index.md + llms.txt (regenerados)
VERIFY_CONTRATO: pasa (RED→GREEN por pieza (a)/(b)/(c) + suites default 701/701 y feature 712/712 + attestation 10/10 + fmt/clippy + docs gates + coverage 0-gaps + OCR + review P2-01)
BLOQUEO: ninguno
GATES_EVALUADOS: P:no(familia aprobada F0) D:no(pre-respondido por plan F0 — contrato sanciona las 3 piezas) V:no C:no | + coordinación: código WIP de MEMG-10 NO stageado; index/llms regenerados incluyen filas de MEMG-10 (generado del árbol — declarado en commit)
SKILLS_CARGADAS: campaign-executor · progreso · ponytail (base auto) · source-driven-development · doubt-driven-development · incremental-implementation · test-driven-development · context-engineering (SDP phase=BUILD) · security-and-hardening · rust-write-tests · documentation-and-adrs (rol) · documentation-skill (docs)
```

**Review P2-01:** APPROVE (fresh, reviewer `vanta-review`, contexto `ses_ef20ad433ffefWeG1953lUTVFi` ≠ autor — rondas 1-2 en §Review).

## Notas (coordinación + shared files)

- **MEMG-10 en vuelo (misma área):** releído fresco en DISCOVERY; código suyo (`vanta-proxy/**`, `src/server/**`, `src/audit.rs`, hooks, `vantadb-mcp/**`, `tests/rbac_namespace.rs`, `tests/memg10_trust_gate.rs`, `docs/dev/research/mgr-04-policy-engine.md`) **NO stageado**. Conflicto real: ninguno (paths disjuntos). Compartidos: `Backlog.md` (su FIND-301 intacto + mi FIND-302), `docs/index.md`/`llms.txt` (regenerados; incluyen sus filas en vuelo — generado del árbol, declarado en el commit `80cb84b0`).
- **Incidente de entorno:** server restart a mitad de sesión (subagent review interrumpido → re-fork, misma sesión de reviewer); 2 crashes transitorios del `clippy-driver` (STATUS_STACK_BUFFER_OVERRUN bajo builds concurrentes con MEMG-10) → re-run limpio ✅.
- **PROHIBIDO y respetado:** `opencode.jsonc`, master plan, `docs/pipeline-state.json` no tocados/stageados.
- **Context Save Point (si el run se interrumpe):** estado en §Steps (5/5 ✅) + recitation; el slice es aditivo y ya está commiteado; nada pendiente salvo el cierre campaign (taskId `55`) y `skill progreso`.
