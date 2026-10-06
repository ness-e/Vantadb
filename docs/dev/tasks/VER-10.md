---
title: "TASK VER-10: Attestation de escritura (extender el certificado de delete a writes)"
kind: task
description: "Recibo de escritura verificable (WriteReceipt v1): put_certified emite content binding sha256 + referencia a la cadena WAL VER-01 + superficies canónicas + límites declarados; verify no claim-driven (schema → integrity → re-scan live); opt-in, delete cert y vanta-cli verify intactos"
---

# TASK VER-10: Attestation de escritura (extender el certificado de delete a writes)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 59, bloque F4-expandido L1693-1719)
- **Fuente:** Backlog `VER-10` (Dep: VER-01 ✅) + VER-02 (certificado de delete ✅) + MEMG-17 (precedente receipt verificable) + plan Task 59
- **Esfuerzo:** 🟡 2-3d | **Appetite:** max 3d | **Stop (plan L1704):** 3d sin contrato → recibo mínimo por `put` + verify + test inválido + FIND de batches/CLI
- **Prioridad:** 🟡
- **Tipo:** Rust (crate `vantadb`; feature-add — símbolos públicos nuevos, aditivos)
- **Turns estimados:** 10-16 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ✅ COMPLETED (2026-10-05) — reservada como taskId `59` en el campaign server
- **Incógnitas (uphill):** 0 — resueltas en DISCOVERY:
  - **Ligar recibo↔frame (pre-mortem #1):** decisión = **evidencia equivalente** (chain reference + content binding + re-scan live), NO frame-level `record_hash`. Evidencia (ver Spec #1): API aditiva y re-escaneo exigen modificar `src/wal.rs`/`src/wal_sharded.rs`/`src/storage/**` (frontera Arch/Engine, prohibida para vanta-worker) y el contrato admite "posición/`record_hash` **o evidencia equivalente**" (plan L1702).
  - **Unidad del recibo (pre-mortem #4):** por operación (`put_certified` = 1 llamada → 1 recibo). Batches quedan como FIND (stop L1704).
- **Pendientes (downhill):** 7 steps
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `59`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Superficie nueva (aditiva):** `Embedded::put_certified(MemoryInput) -> Result<(MemoryRecord, WriteReceipt)>`; `Embedded::verify_write_receipt(receipt_json) -> Result<WriteReceiptVerification>`; tipos públicos `attestation::{WriteReceipt, ContentBinding, WriteReceiptVerification}`. **Callers existentes que NO cambian:** `put`/`put_batch`/`put_record_exact` (firmas intactas), `delete_certified`/`verify_purge_certificate`, `vanta-cli verify`, CLI/MCP delete. |
| Callees | `attestation::{chain_evidence, ChainEvidence, SurfaceReport, CertificateIntegrity}` (reuso VER-02), `sdk::serialization::record_from_node` (re-scan), `StorageEngine::get`, `crate::audit::now_iso`, `sha2::Sha256`, `serde_json`. Cero deps nuevas. |
| Implicaciones | **API pública aditiva** — sin breaking, sin wire, sin migración, sin locks, sin cambios de concurrencia. `put` (hot path) **no cambia** (opt-in: el recibo solo lo paga quien llama `put_certified`). `src/wal.rs`, `src/wal_sharded.rs`, `src/storage/**`, `src/vector/**` **no se tocan**. Regresión: suites `-p vantadb --test certified_delete` + `-p vantadb --lib` (attestation) se re-corren. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `65144964`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `src/attestation.rs` (:1-62 scope/integrity + límites declarados; :63-160 `ChainEvidence`/`CertificateIntegrity`/`chain_evidence`/`declared_limits`; :349-471 `canonical_bytes`/`validate_schema`/`finalize`/`build_certificate`; :473-637 `verify_certificate` no claim-driven; :639-769 tests), `src/sdk/api/memory.rs` (:264-272 `check_read_only`; :635-663 `check_namespace_quota`; :665-806 `put_one` — record → `memory_record_to_node_owned` → `engine.insert` :761; :836-853 `put`; :1140-1170 `get`; :1235-1306 `delete_inner`/`delete_certified`; :1308-1332 `verify_purge_certificate`; :1342-1435 `put_record_exact`), `src/wal.rs` (:23-46 versiones/framing v3; :65-103 `is_chained`/`chain_hash`/`frame_extra_for`; :105-144 `WalRecord`; :398-432 `append` — computa `record_hash`, devuelve `()`; :435-470 `batch_append`; :871-1008 `verify_wal_file` — reporte agregado, sin hashes por frame; :1010-1160 `WalReader::next_record` — **descarta el chain tail** "validated by verify_wal_file, never here"), `src/wal_sharded.rs` (:9-17 `ShardedWal` + `next_shard` round-robin interno), `src/storage/engine/insert.rs` (:155-225 `insert` — `sharded.append(WalRecord::Insert)` dentro de `insert_lock`, antes de `apply_insert`), `src/sdk/serialization/mod.rs` (:130-168 `validate_key`/`validate_metadata` — prefijo reservado rechazado; :331-508 `record_from_node` → `memory_record_from_node_inner` — re-scan/round-trip completo; :511-619 `memory_record_to_node_owned`), `src/sdk/types.rs` (:104-149 `Value`/`Fields = BTreeMap`/`MemoryMetadata = Fields`), `src/sdk/types/record.rs` (:250-327 `MemoryRecord`), `src/node/vector_data.rs` (:23-30 `SparseVector(BTreeMap<u32, f32>)` — serialización determinista), `tests/certified_delete.rs` (:1-375 — patrón de tests de certificados + CLI), `vanta-memory/src/utils/erasure.rs` (:35-106 `ErasureReceipt` — precedente de receipt con tipos VER-02), `docs/api/CERTIFIED_DELETE.md` (:1-128 — contrato doc del delete certificado), `docs/api/WAL_INTEGRITY.md` (esquema de cadena), `Cargo.toml` (serde_json/sha2 ya presentes).
- **Archivos referenciados hacia dentro (imports/deps):** `attestation.rs` (nuevo código) → `sdk::serialization::record_from_node`, `StorageEngine::get`, `sha2`, `serde_json`, `crate::audit::now_iso`. `memory.rs` → `crate::attestation::{build_write_receipt, WriteReceipt, WriteReceiptVerification}`. `tests/write_receipts.rs` (nuevo) → `vantadb::{config::Config, sdk::{Embedded, MemoryInput, MemoryMetadata}, attestation::{WriteReceipt}}`. Doc nueva → `WAL_INTEGRITY.md` + `CERTIFIED_DELETE.md`.
- **Referencias entrantes (grep/CodeGraph HEAD):** `WriteReceipt`/`put_certified`/`write_receipt` = **0 hits** en `src/` (el gap del plan: "solo delete tiene attestation"). Callers de `put`/`put_one`/`put_batch` no cambian de firma. `verify_wal_file` es `pub(crate)` + `#[cfg(any(feature = "cli", test))]` (no reusable sin tocar wal.rs). `chain_evidence` = 1 caller interno (`build_certificate`) + 1 externo (`vanta-memory` erasure, MEMG-17) — sin cambio. `MemoryRecord` round-trip verificado (`put_one` → `record_from_node`: campos persistidos en el nodo; `metadata`/`sparse_vector` son BTreeMap → orden determinista).
- **Veredicto impacto:** **BAJO-MEDIO (aditivo puro)** — 2 archivos de código tocados (`src/attestation.rs` + `src/sdk/api/memory.rs`), 1 test nuevo (`tests/write_receipts.rs`), 1 doc nueva (`docs/api/WRITE_RECEIPTS.md`) + 1 link cruzado. Sin breaking, sin wire, sin deps, sin locks, sin hot path. Pre-mortem mitigado: (1) decisión de ligadura tomada con evidencia y FIND para la exposición del frame (Engine/Arch); (2) reuso total de `ChainEvidence`/`SurfaceReport`/`CertificateIntegrity` (sin fork de schema); (3) opt-in explícito (put intacto); (4) unidad = por operación, batches a FIND.

## Contrato

"**escrituras con attestation verificable extendiendo el chain**: al menos el write path declarado (`put`) emite un recibo con referencia al frame WAL encadenado (posición/`record_hash` **o evidencia equivalente**) + verificación que re-chequee (schema → integridad sha256 → evidencia viva, patrón VER-02) + test válido/inválido (recibo editado/corrupto y payload alterado → detección) + límites declarados (misma honestidad que VER-02: qué NO prueba) + doc (`CERTIFIED_DELETE.md` extendido o doc nuevo de write receipts); certificados de delete y `vanta-cli verify` intactos; attestation opt-in (sin coste por defecto)" (plan Task 59 L1702).

Verificación: tests RED→GREEN (`tests/write_receipts.rs` + unit en `attestation.rs`) + suite scoped `cargo nextest run --profile audit -p vantadb --test write_receipts --test certified_delete --build-jobs 2` + `-p vantadb --lib` + `cargo fmt --check` + `cargo clippy -p vantadb --all-targets -- -D warnings` + gates docs (`check-links`/`check-docs`/`gen-index --check`).

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): pre-respondido por el plan F0** — el contrato Task 59 (Gate Result ✅ DO) sanciona "al menos el write path declarado (p.ej. `put`) emite un recibo… + verificación… + test válido/inválido + límites + doc"; los símbolos nuevos (`put_certified`/`verify_write_receipt`/`WriteReceipt`) son el vehículo mínimo de esa sanción (espejo exacto del precedente VER-02 `delete_certified`/`verify_purge_certificate`). Precedente idéntico: MEMG-17. Sin símbolos fuera de la sanción.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | **Ligadura recibo↔frame WAL (pre-mortem #1)** | A) **API aditiva: `append` devuelve `record_hash`/posición** — requiere `src/wal.rs` + `src/wal_sharded.rs` + plumbing por `engine.insert` → frontera Arch/Engine (prohibida) + blast radius cross-layer / B) **re-escaneo del WAL** — `WalReader::next_record` **descarta el chain tail por diseño** ("validated by `verify_wal_file`, never here", `wal.rs:1082-1085`); `verify_wal_file` devuelve reporte agregado sin hashes por frame; shard = round-robin interno (`wal_sharded.rs`); frames forward-only (sin seek hacia atrás); appends concurrentes interleavan → re-scan no atribuye un frame a UNA escritura; coste O(segmento) / C) **evidencia equivalente (contrato L1702 la admite): `ChainEvidence` VER-01 reusado + content binding sha256 del record + re-scan live** | ✅ **C** — decidido-por-evidencia: A/B requieren tocar wal.rs/storage (fuera de la frontera del worker) y B es además no-confiables bajo concurrencia; C detecta los dos inválidos exigidos (recibo editado → integrity; payload alterado → content hash; record borrado → ausente). FIND-305 registra la exposición del `record_hash` (Engine/Arch) como upgrade |
| 2 | Forma del recibo | A) **`WriteReceipt` nuevo espejando el contrato VER-02: schema version + `out_of_scope` no-vacío + superficies canónicas exactas (`store`, `wal`) + `integrity` sha256 + `ContentBinding` sha256 + `ChainEvidence`; reutiliza `SurfaceReport`/`CertificateIntegrity`/`chain_evidence`** / B) reusar `PurgeCertificate` tal cual (habla de delete/residuos — semántica falsa) / C) firma criptográfica — 0 hits, no sancionada (`attestation.rs:17`) | ✅ **A** — mismo contrato de verificación que los certificados existentes (plan L1700 "solapa con VER-01/VER-02 → reusar, no forkear"); C queda declarado en límites |
| 3 | Qué cubre el content binding | A) **proyección canónica `{namespace, key, node_id, version, payload, metadata, vector, sparse_vector, expires_at_ms}`** — contenido + identidad + versión; B) solo `payload` (mínimo exigido — no detecta metadata/vector alterados); C) record completo JSON (acopla el hash a campos derivados que otras ops legítimas actualizan: confidence MEMG-02, quarantine — rompería recibos válidos) | ✅ **A** — decidido-por-evidencia: B sub-cobertura; C sobre-acoplamiento (los campos derivados no son "lo escrito"); A se documenta en `out_of_scope` (timestamps de sistema y estado derivado NO cubiertos — declarado) |
| 4 | Semántica de verificación al cambiar el record | A) **falla por diseño si el record fue actualizado (re-put) o borrado después: el recibo es attestation point-in-time (espejo de "residues reappeared" del delete)** / B) verificar solo contra la historia WAL (no disponible sin tocar wal.rs — decisión #1) | ✅ **A** — consistente con `verify_certificate` (residuos reaparecidos → fail) y declarado en límites |
| 5 | API del SDK | A) **`put_certified(input) -> Result<(MemoryRecord, WriteReceipt)>` (no pierde el record — el write SÍ lo tiene, a diferencia del delete) + `verify_write_receipt(json)` (espejo del `&str` de `verify_purge_certificate`)** / B) solo recibo (pierde created_at/version — info útil del write) | ✅ **A** — la asimetría con `delete_certified` (solo certificado) está justificada: el delete no tiene record que devolver; el write sí |
| 6 | Momento de emisión | A) **`put` → record → `build_write_receipt(&record)` puro (sin I/O extra); la evidencia viva se re-chequea en VERIFY (patrón VER-02: "referenced, not re-verified per op")** / B) point-read extra en emisión (coste en path opt-in sin valor: el `engine.insert` ya retornó Ok) | ✅ **A** — contrato L1702: "verificación que rechequee… evidencia viva" — la evidencia viva vive en la verificación |
| 7 | Estado del recibo | A) **`status: "recorded"` (único; schema lo valida contra el set conocido)** / B) espejar `purged/residues/not_found` (semántica de delete — falsa para writes) | ✅ **A** — honestidad: un write certificado no tiene estados de residuo; la verificación falla con error tipado |
| 8 | Unidad / superficies | A) **por operación (`put_certified`); superficies `store` (`written`) + `wal` (`frame-recorded`) exactamente una vez; batches/CLI/MCP → FIND-306 (stop L1704)** / B) batches ya (scope creep fuera del appetite) | ✅ **A** — stop del plan; la unidad se declara en el contrato |
| 9 | Tests | A) **integración `tests/write_receipts.rs` (InMemory + tempdir; patrón `certified_delete.rs`) para flujo SDK + unit en `attestation.rs` para schema/content-hash** / B) unit inline (no prueban el flujo) | ✅ **A** — el crate usa integration tests para flujos SDK (precedente VER-02/MEMG-17) |
| 10 | Timestamps / wire | `crate::audit::now_iso()` (ISO 8601 UTC, ya usado por el cert); recibo NO contiene material sensible; sin campos nuevos en `MemoryRecord` | ✅ — verificado en DISCOVERY (`attestation.rs:456`) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **Certificados de delete + `vanta-cli verify` intactos**: `PurgeCertificate`, `build_certificate`, `verify_certificate`, CLI/MCP delete no cambian de comportamiento (suite `certified_delete` verde).
  2. **`put`/`put_batch`/`put_record_exact` intactos** (firmas y semántica) — la attestation es **opt-in**: sin llamada a `put_certified` no hay coste nuevo.
  3. **`src/wal.rs`, `src/wal_sharded.rs`, `src/storage/**`, `src/vector/**` NO se tocan** (frontera Arch/Engine).
  4. **Recibos nunca emitidos estructuralmente inválidos** (`finalize` patrón VER-02); verificación **no claim-driven** (schema → integrity → re-scan; un recibo claimless con hash recomputado se rechaza).
  5. **Sin `unwrap`/`expect`/`unsafe`** en código nuevo de producción; errores vía `Error::Validation`/`Error::serialization`.
  6. **WIP ajeno:** `opencode.jsonc` + master plan + `dev-tools/heavy-test-lock.ps1` (untracked) en el árbol por otros — NO se stagean; PROHIBIDO tocar `docs/pipeline-state.json`; commit con **pathspec**.
  7. **Coordinación MEMG-16** (en vuelo, checker/scopes — otra área): si `src/sdk/api/memory.rs` aparece modificado por la otra sesión al momento de editar → releer fresco antes de editar; conflicto real → BLOQUEO.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb --test write_receipts --test certified_delete --build-jobs 2` · `cargo nextest run --profile audit -p vantadb --lib --build-jobs 2` · `cargo fmt --check` · `cargo clippy -p vantadb --all-targets -- -D warnings` (con **heavy-test-lock** adquirido para las corridas pesadas).
- **Deuda pendiente:** exposición del `record_hash`/posición del frame en `append` (FIND-305 — Engine/Arch); batches/CLI/MCP de write receipts (FIND-306); firma criptográfica (vanta-audit, VER-01 §Diseño).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — sin `unsafe`, sin clones nuevos en hot paths (`put` intacto; el recibo es opt-in y opera O(record)), sin abstracciones especulativas (tipos espejo del contrato VER-02, reuso de `SurfaceReport`/`CertificateIntegrity`/`chain_evidence` — no se reimplementan). El cambio **elimina** la deuda del gap VER-10 ("solo delete es demostrable") y **agrega tests de contrato**. Deps: cero nuevas (sha2/serde_json ya presentes).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | `put_certified` RED→GREEN (recibo válido: shape + content binding + chain) + verify RED→GREEN (válido ok) + inválidos RED→GREEN (recibo editado → integrity; payload/contenido alterado → content mismatch; record borrado → ausente; claimless con hash recomputado → rechazo) + límites declarados no-vacíos + suite scoped verde (`write_receipts` + `certified_delete` + `--lib`) + fmt/clippy verdes + doc `WRITE_RECEIPTS.md` + gates docs verdes |
| **Commit** | Commit atómico conventional `feat(attestation):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | Changelog release-plz (feature → minor); verify full scoped documentado (workspace completo no se corre por presupuesto) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius) + `codebase-memory-mcp_check_index_coverage` (paths clave) + grep puntual
- `cargo nextest` scoped por crate (loop TDD) + `campaign_verify_cmd` (verify mecánico)
- `pwsh dev-tools/heavy-test-lock.ps1 acquire|release` (pruebas pesadas serializadas — regla owner 2026-10-05)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- `vanta-review` (subagente, review P2-01 fresh-context) al cierre
- Gates docs (task file + doc nueva): `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check`

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `test-driven-development` · `api-and-interface-design` · `security-and-hardening` · `rust-write-tests` · `documentation-skill` (pins: api-and-interface-design, test-driven-development; orchestrator: security-and-hardening, rust-write-tests). Excluidas con justificación: `systematic-debugging` (sin bug), `deprecation-and-migration` (API aditiva, sin deprecación), `documentation-and-adrs` (no hay decisión arquitectónica nueva — el contrato ya está sancionado por el plan; la decisión #1 se documenta en el task file), `writing-guidelines`/`writing-plans` (doc técnica EN cubierta por documentation-skill; plan ya existe).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — trust boundaries cubiertos: (1) `receipt_json` parseado con error tipado + schema no claim-driven + `node_id` validado + sin panics; (2) mensajes sin detalles internos de más (patrón VER-02); (3) recibo sin material sensible; (4) **integridad ≠ autenticidad** declarada (sin firma); (5) verificación read-only; (6) [review] floats no finitos: rechazo en emisión + fail-safe en verificación (el binding JSON era ambiguo). Checklist `security-and-hardening` aplicado.
- [x] **PERFORMANCE** — no aplica (Regla 9 no dispara): `put` hot path intacto; recibo opt-in O(record); sin locks/loops nuevos. Coste declarado en el doc.

## Steps

> ~100 líneas por step, un step por turno, reversible. TDD: RED antes de GREEN.

### Step 1: RED — `tests/write_receipts.rs` flujo válido (put_certified + verify) — ✅
- [x] Escribir `tests/write_receipts.rs` (patrón `certified_delete.rs`: `open_temp_db`, fixture `write_input` con metadata + vector): `put_certified_emits_verifiable_receipt` + `write_receipt_verifies_after_flush_and_reopen`.
- [x] Verificar RED (con **heavy-test-lock**): `cargo nextest run --profile audit -p vantadb --test write_receipts --build-jobs 2` → `error[E0599]: no method named put_certified/verify_write_receipt` (4 errores de compilación — razón correcta).

### Step 2: GREEN — schema `WriteReceipt` + builder + unit tests — ✅
- [x] `src/attestation.rs`: `WRITE_RECEIPT_SCHEMA_VERSION = 1`, `WRITE_SURFACES = ["store", "wal"]`, `ContentBinding`, `WriteReceipt`, `WriteReceiptVerification`, `write_declared_limits()` (8 límites honestos), `WriteContentProjection`/`content_sha256` (proyección canónica BTreeMap-safe), `canonical_write_bytes`/`validate_write_schema`/`finalize_write`, `build_write_receipt(&MemoryRecord) -> Result<WriteReceipt>` (`pub(crate)`).
- [x] Unit tests: `write_finalize_computes_stable_hex_digest`, `validate_write_schema_rejects_missing_surface`, `..._empty_declared_limits`, `..._unknown_status`, `content_hash_changes_with_payload`, `..._metadata_and_version`, `content_hash_ignores_system_timestamps_and_derived_state`, `write_declared_limits_are_never_empty` + (review) `first_non_finite_*`.
- [x] `cargo nextest run --profile audit -p vantadb --lib --build-jobs 2` (con lock) — verdes (2356 tests).

### Step 3: GREEN — SDK `put_certified` + `verify_write_receipt` — ✅
- [x] `src/sdk/api/memory.rs`: `Embedded::put_certified(input) -> Result<(MemoryRecord, WriteReceipt)>` (delega en `put` para semántica/audit/quota + `build_write_receipt`; pre-check de finitud ANTES del write — review) y `Embedded::verify_write_receipt(&self, receipt_json: &str) -> Result<WriteReceiptVerification>`.
- [x] `attestation::verify_write_receipt(engine, &receipt)`: schema_version → integrity sha256 → `validate_write_schema` → `content.algorithm` → `node_id` parse → re-scan live (`engine.get` + `record_from_node`: ausente/no-legible → error; ns/key mismatch → error) → fail-safe non-finite → recomputar `content_sha256` → mismatch → error tipado.
- [x] Step 1 verde (con lock): 2/2.

### Step 4: RED→GREEN — tests inválidos + determinismo — ✅
- [x] `verify_rejects_tampered_receipt` (version/node_id sin recomputar → "integrity"); `verify_detects_content_change_after_receipt` (re-put con payload distinto → "content"); `verify_fails_when_record_deleted_after_receipt` (delete → "present"); `verify_rejects_claimless_receipt_even_with_recomputed_hash` (surfaces vaciadas + hash recomputado → "surface"); `write_receipt_is_deterministic_across_equivalent_databases`.
- [x] Correr `--test write_receipts --test certified_delete` (con lock) — verdes (16/16 en ese momento), sin regresión del delete certificado.

### Step 5: REFACTOR + gate de contrato local — ✅
- [x] Limpieza + `rustfmt` (solo archivos propios); `cargo fmt --check` verde (full); `cargo clippy -p vantadb --all-targets -- -D warnings` verde.
- [x] Contrato verificado punto por punto; **review P2-01 ronda 1 = changes-required** (2 fixes: inyectividad de floats no finitos + límites incompletos) → aplicados: pre-check `first_non_finite_input` en `put_certified` ANTES del write + fail-safe `first_non_finite_record` en verify + límites re-enunciados + tests; ronda 2 = **approve**.

### Step 6: Doc — `docs/api/WRITE_RECEIPTS.md` + link cruzado + gates — ✅
- [x] Doc nueva (kind: reference, EN): schema, superficies, content binding (alcance exhaustivo + floats finitos), verificación (orden real), límites, durabilidad/flush, ejemplo JSON + snippet Rust.
- [x] Links cruzados en `CERTIFIED_DELETE.md` + `WAL_INTEGRITY.md`; `check-links` ✅ (0 broken), `check-docs` ✅ (all clear), `gen-index --write` (docs/index.md + docs/api/index.md + llms.txt regenerados).

### Step 7: Cierre — FINDs, verify full scoped, OCR, review P2-01, commit, campaign — ✅
- [x] Rows en `docs/dev/Backlog.md`: **FIND-307** (frame `record_hash`/posición en `append` — Engine/Arch) + **FIND-308** (batches/CLI/MCP de write receipts) — FIND-305/306 fueron tomados por MEMG-16 en paralelo (FIND-306 = el clippy de `merge_tests`, resuelto por este cambio).
- [x] Verify full scoped (con lock): nextest `--test write_receipts --test certified_delete` **20/20** + `--lib` **2356/2356** + fmt + clippy.
- [x] OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) — rule groups aplicados a los archivos propios: sin Critical/High/Medium; 1 Low descartado (NaN→null, resuelto luego por review).
- [x] Review P2-01 adversarial (`vanta-review`, sesión `ses_ef1010d42ffeysPHuxHjBcpF2O`, contexto ≠ autor): ronda 1 changes-required (2 ítems) → ronda 2 **VERDICT: approve** ("the proven hole is closed on both sides; no new hole").
- [x] Commit LOCAL `feat(attestation):` + `fix(test):` (pathspec; ⛔ sin push) — hash en RESULTADO.
- [x] `campaign_update_task_state(taskId="59", "completed", recitation + review payload)`.

## Review (GATE — agente distinto, P2-01)

- **Reviewer:** `vanta-review` (subagente, fresh context) — sesión `ses_ef1010d42ffeysPHuxHjBcpF2O`; autor = sesión `ses_ef131e0b7ffehc0HhftGrHN4Wf`.
- **Ronda 1 (changes-required, sin Critical/High):** (1) la proyección de contenido era no-inyectiva para floats no finitos (NaN/±Inf → `null` en JSON; colisión null↔NaN probada) → fix aplicado en ambos lados (rechazo en emisión ANTES del write + fail-safe en verificación) + tests; (2) límites declarados incompletos (faltaba enumerar validity window/lineage/provenance/last_validated) → re-enunciados "ONLY the projection fields are covered" + límite nuevo de floats finitos; nit del orden de pasos del doc corregido; tests opcionales (sparse + TTL) agregados.
- **Ronda 2 (VERDICT: approve):** evidencia independiente — `--test write_receipts --test certified_delete` 20/20, `--lib -E 'test(attestation)'` 20/20, clippy `--all-targets -D warnings` limpio; "no new hole found; delete certificate and put/put_batch remain untouched. Gate P2-01 (approach review) passes."
- **ACCEPT payload:** `{mode:'fresh', reviewer:'vanta-review', reviewer_context:'ses_ef1010d42ffeysPHuxHjBcpF2O', author_context:'ses_ef131e0b7ffehc0HhftGrHN4Wf', verdict:'approve'}`.

## Context Save Point

- **Estado:** ✅ COMPLETED (2026-10-05) — todos los steps verdes; review P2-01 approve (ronda 2); pendiente solo el hash del commit en RESULTADO.
- **Notas para el próximo agente:** `put_certified` NO modifica `put` (opt-in; solo agrega pre-check de finitud); `verify_write_receipt` recibe JSON (espejo `verify_purge_certificate`); `WRITE_SURFACES` = 2 (`store`, `wal`); el content hash cubre SOLO la proyección declarada (exclusiones enumeradas en límites); floats no finitos rechazados/fall-safe; no se tocó `src/wal.rs`/`src/storage/**`; `src/sdk/merge_tests.rs` cambió por FIND-306 (clippy 1.95), no por VER-10.

## RESULTADO (§7 — se completa al cierre)

```
RESULTADO: <pendiente>
STEPS_OK: 0/7
PROXIMO_STEP: Step 1 — RED tests/write_receipts.rs
COMMIT_HASH: ninguno
ARCHIVOS: docs/dev/tasks/VER-10.md
VERIFY_CONTRATO: no-corrido
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:disparado(pre-respondido por plan) V:no C:no | contrato sancionado por plan F0
SKILLS_CARGADAS: test-driven-development · api-and-interface-design · security-and-hardening · rust-write-tests · documentation-skill (+base auto)
```
