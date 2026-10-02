---
title: "VER-02: Borrado certificado — delete-path shred→GC→WAL + attestation por superficie"
kind: task
description: "Cablea la purga del shredded store en el delete-path (delete/delete_batch/purge_permanent) + certificado de purga JSON v1 por superficie (CLI/MCP, verificable, determinista); diseño explícito para review de vanta-audit."
---

# VER-02: Borrado certificado — delete-path shred→GC→WAL + attestation

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` — Task 38 (F4) · **Origen:** plan L995-1019 (bloque completo leído); Backlog:924; consumidor ICP-02 (F5, plan L1085).
- **Fuente del prompt:** sub-agente `vanta-worker` (orquestador pipeline) — wave F4.2 (co-batch VER-06 ‖ VER-03); branch `develop`; commits = LEAD.
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴 · **Tipo:** feature-add (Rust core + CLI + MCP) — wiring 3 capas + schema público nuevo.
- **Creado:** 2026-09-29 · **last-synced:** 2026-09-29
- **Estado:** ✅ COMPLETO (carril worker) — steps 8/8; pendiente SOLO carril LEAD: commit local + review P2-01 fresco (`vanta-audit` para el diseño del certificado) — NO ejecutados por diseño de la wave.
- **Gate D (question-gates):** DISPARADO por contrato (símbolos públicos nuevos: `Embedded::delete_certified`, `Embedded::verify_purge_certificate`, `PurgeCertificate`, `Commands::Certificate` + param `attest` en MCP `memory_delete`) — **pre-respondido por el orquestador**: el bloque F4 Task 38 ya pasó su completado de fase (contrato verbatim + stop conditions + Risk Register en el plan) y la instrucción de ejecución es explícita ("ejecutá UNA TAREA COMPLETA"). Superficie mínima requerida por el contrato: `delete --attest` (emisión CLI), `certificate verify --file` (consulta), `memory_delete {attest:true}` (emisión MCP). Sin símbolos extra.
- **Gate V (question-gates):** no disparado (0 fallas de verify mismo-error).
- **SDP:** base `campaign-executor`+`progreso`+`ponytail` (auto) · pins/obligatorias del bloque: `security-and-hardening`, `source-driven-development`, `test-driven-development`, `rust-write-tests`, `doubt-driven-development` · `documentation-skill` (crea `docs/api/CERTIFIED_DELETE.md`). Sin candidatos extra tras discovery.

## Contrato (verbatim del plan, L1005)
> "delete-path completo y verificable: `delete`/`delete_batch`/purga cablean el shred store (`ShreddedRowStore::delete`), remueven de índices derivados (HNSW/text/derived) y dejan rastro WAL verificable Y certificado de purga (JSON con timestamp, key/namespace, superficies barridas y evidencia) emitido por CLI y MCP, consultable y determinista Y test E2E: record borrado → 0 residuos en store/índices/shred + certificado válido (con firma/chain de VER-01 si está; si no, campo reservado) Y doc (alcance: purga lógica/física; NO unlearning paramétrico)"

**Cláusulas a verificar (matriz de cierre):**

| # | Cláusula | Superficie | Evidencia |
|---|----------|-----------|-----------|
| C1 | `delete`/`delete_batch`/purga cablean `ShreddedRowStore::delete`; índices derivados (HNSW/text/derived) removidos | `src/shred/mod.rs`, `src/storage/engine/delete.rs` | tests unit (4 capas) + E2E |
| C2 | Rastro WAL verificable (tombstone en WAL encadenado VER-01) | `src/storage/engine/delete.rs:44` + `src/wal.rs` (v3) | unit: replay encuentra `WalRecord::Delete` + `chain.status` en certificado |
| C3 | Certificado JSON (timestamp, key/namespace, superficies, evidencia) emitido por CLI y MCP, consultable y determinista | `src/attestation.rs`, `src/cli_handlers/`, `vantadb-mcp/src/handlers/tools.rs` | unit attestation + E2E + test MCP + smoke CLI |
| C4 | Test E2E: 0 residuos store/índices/shred + certificado válido (≠válido si se manipula) + doc de alcance | `tests/certified_delete.rs`, `docs/api/CERTIFIED_DELETE.md` | tests dedicados + `validate-docs-coverage` |

## Re-baseline (verificado 2026-09-29, branch `develop`)

- **`src/shred/` = JSON Shredding** (columnar typed storage de metadata, `mod.rs:4-13`) — **NO secure-delete**. El delete-path estaba sin cablear (`mod.rs:47-48` "Not yet wired — shredded entries survive node deletion until garbage collection (Phase 2)"). `delete()` existe y está testeado: **`:284`** (`#[allow(dead_code)]`, el plan citaba `:362-363` = líneas del test — re-baseline corregido).
- **Delete-path verificado (leído completo):** `delete(id,_reason)` (`delete.rs:19`) escribe tombstone WAL ANTES de store I/O (`:44`, `WalRecord::Delete`), aplica `apply_delete_inner` (tombstone de vector store `FLAG_TOMBSTONE` + `remove_hnsw_entry` + cache) y `backend.delete(Default, id)` (`:48`). `_reason` se ignora hoy. `delete_batch` (`:160`) hace lo mismo en batch (WAL `:220`, backend `write_batch` `:249-260`). `purge_permanent` (`:274`) borra Default + TombstoneStorage + Tombstones — **no** InternalMetadata (donde vive `shred::{id}`). `is_deleted` (`:294`) lee la partición Tombstones (solo la escribe la vía transaccional `txn.rs:394` — irrelevante acá).
- **Ningún backend partition `InternalMetadata`/shred se toca en delete** → gap confirmado (`rg 'ShreddedRowStore::delete'` en código no-test = 0 usos).
- **GC:** `GcWorker::sweep` (`gc.rs:38-81`) llama `storage.delete(id, "GC TTL Expired")` → hereda el wiring; `purge_ttl_for_deleted` (`:88`) solo limpia el TTL map. Sweeper de producción (`:130-176`) → `Embedded::purge_expired` (`memory.rs:1102`, `engine.delete` en `:1201`) → hereda el wiring. `delete_by_filter` (`namespaces.rs:251`) → `self.delete` → hereda.
- **SDK delete** (`memory.rs:778`): `engine.delete` + `replace_derived_indexes(engine, Some(&existing), None)` (`impl_index.rs:154`; derived NamespaceIndex/PayloadIndex + text postings/stats + sparse ops vía `write_backend_batch`) + `version_history::purge_key` (best-effort, `version_history.rs:342`) + `AuditEvent "delete"` con `reason` (`audit.rs:33`).
- **Attestation NO existe:** `rg 'attestation'` en `src/` = 0 (solo TLS `certificate`). `AuditEvent` (JSONL opt-in) es el modelo de evento, no un certificado.
- **VER-01 ✅ (commit `0cc14247`):** WAL framing v3 encadenado (`WAL_FORMAT_VERSION` = 3; `pub`), `verify_wal_file`/`verify_shards` `#[cfg(any(feature = "cli", test))]` (no invocables incondicionalmente desde lib), CLI `Commands::Verify` existe (`cli.rs:201`), `WAL_FORMAT_VERSION` re-exportado (`lib.rs:206`). Los consumidores encadenados (MGR-13 §8) son VER-02/VER-04.
- **Accesores disponibles para el scan de residuos (verificado):** `engine.backend` (`pub(crate)`), `get_from_partition`/`scan_partition`/`scan_partition_prefix` (`storage/engine/partition.rs:26/:35/:56`), `scalar_lookup_int_le`, `engine.hnsw.load()` (usado desde `sdk/api/search.rs:22`), `derived_delete_ops`/`text_index_ops_for_replace`/`sparse_index_ops_for_replace` (ops exactas del delete), `version_prefix` (`version_history.rs:39`). `sha2 = "0.11"` ya es dep directa (promovida por VER-01, `Cargo.toml:136`).
- **MCP:** `memory_delete` definido en `tools.rs:168` (schema {namespace,key}) y handler en `:1548`; el meta-test `test_mcp_tool_annotations_coverage` (`mcp_tests.rs:4603`) exige title + 4 bools por tool — agregar un **param opcional** no altera conteos ni hints. `vantadb-mcp/tests/mcp_tests.rs` corre el server in-process.
- **CLI:** `Commands::Delete {namespace,key}` (`cli.rs:230`) → `cmd_delete` (`crud.rs:402`); patrón subcommand-enum: `Wal(WalCommand)` (`cli.rs:440`), `Commands::Verify` con exit code (`cli.rs:201`, handler `wal.rs` `cmd_verify`). `open_embedded(path, read_only)` (`db.rs:18`), `print_json` (`fmt.rs:93`).
- **Co-batch F4.2 (NO tocar):** `src/cli_handlers/export_md.rs`, `src/cli_handlers/index.rs`, `vanta-memory/src/seed/**` (VER-06); `vanta-proxy/**`, `src/crypto.rs` (VER-03).

## Blast Radius

| Dirección | Archivos |
|-----------|----------|
| **Edita** | `src/shred/mod.rs` (helper `store_key`/`delete_op` + wire) · `src/storage/engine/delete.rs` (purga shred en delete/delete_batch/purge_permanent) · `src/attestation.rs` (**nuevo** — certificado + scan de residuos + verify) · `src/lib.rs` (`pub mod attestation`) · `src/sdk/api/memory.rs` (`delete_inner` refactor + `delete_certified` + `verify_purge_certificate`) · `src/cli.rs` (`Delete --attest [--out]` + `CertificateCommand::Verify`) · `src/bin/vanta-cli.rs` (dispatch) · `src/cli_handlers/crud.rs` (attest en delete: `cmd_delete_certified` + `cmd_certificate_verify` — I-1: NO existe `cli_handlers/attest.rs`) · `vantadb-mcp/src/handlers/tools.rs` (param `attest` + handler) · `tests/certified_delete.rs` (**nuevo**) · `tests/api/public-api.txt` (snapshot — símbolos nuevos) · `docs/api/CERTIFIED_DELETE.md` (**nuevo**) · `docs/user/operations/CONFIGURATION.md` (tabla CLI, quirúrgico) · este task file |
| **NO toca (prohibido/wave)** | `docs/dev/Backlog.md` · `perf-bench.yml` · `CONSTRAINTS.md` · `desktop/**` · `opencode.jsonc` · plan file (LEAD) · **regiones co-batch F4.2:** `src/cli_handlers/export_md.rs`, `src/cli_handlers/index.rs`, `vanta-memory/src/seed/**` (VER-06), `vanta-proxy/**`, `src/crypto.rs` (VER-03) · `src/wal*.rs` (VER-01 revisado; solo se **cita** el chain, no se edita) |
| **Referencias hacia dentro** | `GcWorker::sweep` → `StorageEngine::delete`; `purge_expired`/`delete_by_filter` → `Embedded::delete`; `apply_delete_inner` es compartido con `commit_transaction` (txn) — la purga shred NO entra ahí (deletes transaccionales conservan el stamp MVCC) |
| **Referencias entrantes** | SDK/MCP/CLI delete (3 canales) + GC + TTL sweeper consumen el wiring; el certificado lo consumen ICP-02 (F5) y `vanta-audit` (diseño); erratas del plan: `src/shred/mod.rs:362-363` → real `:284`; `delete.rs:160/:220` correctos |
| **Implicaciones** | +1 op de backend por delete (point-delete en InternalMetadata); hot path `delete` intacto (best-effort, nunca bloquea el delete core; batch = misma batch atómica); API pública nueva (`Embedded::delete_certified`/`verify_purge_certificate`, `attestation::*`, `Commands::Delete.out`, `Commands::Certificate`) → snapshot `public-api.txt` regenerado (**+165 líneas netas reales**, es 100% VER-02 (0 ítems públicos ajenos — verificado en review) — I-1 corrige el "+5 símbolos" inicial); certificado introduce `schema_version` propio (v1) — formato nuevo, sin breaking de wire |

## Impacto mapeado (Regla 0)

- **Leídos completos:** `src/shred/mod.rs` (703L) · `src/storage/engine/delete.rs` (301L) · `src/gc.rs` (501L) · `src/audit.rs` (381L) · `docs/dev/tasks/VER-01.md` (183L) · `.opencode/rules/durability.md` · `src/cli.rs` (rangos 190-229/425-484: `Commands`, `WalCommand`) · `src/cli_handlers/crud.rs` (:380-459) · `src/cli_handlers/wal.rs` (:268-360 `cmd_verify`) · `vantadb-mcp/src/handlers/tools.rs` (:150-219 defs, :1530-1591 handlers) · `src/sdk/serialization/mod.rs` (:160-275 key builders).
- **Leídos rangos clave:** `src/sdk/api/memory.rs` (:300-679 put/put_batch + shred wire; :1090-1209 purge_expired) · `src/sdk/api/namespaces.rs` (:240-359 delete_by_filter) · `src/sdk/serialization/impl_index.rs` (:100-280 replace_derived_indexes + state adjust) · `src/sdk/version_history.rs` (:330-389 purge_key/versions) · `src/storage/engine/mod.rs` (campos `pub(crate)`: backend/hnsw/wal/cache/…) · `src/storage/wal.rs` (25L) · `src/wal.rs` (chain: `WAL_CHAIN_VERSION`:31, `chain_hash`:90, `WalVerifyReport`:840, `verify_wal_file` gate :863) · `src/wal_sharded.rs` (`verify_shards` gate :333) · `src/cli_handlers/db.rs` (:18) · `tests/cli_tests.rs` (:1-45) · `mcp_tests.rs` (:4592-4692).
- **Referencias hacia dentro:** `StorageEngine::delete` ← `gc.rs` (2 callers), `sdk/api/memory.rs`, tests; `delete_batch` ← 9 callers; `purge_permanent` ← 4 callers (tests/ops); `ShreddedRowStore::delete` ← 0 callers no-test (el gap).
- **Referencias salientes:** `attestation.rs` → `shred`, `sdk::serialization` (key builders/ops), `sdk::version_history` (prefix), `wal` (`WAL_FORMAT_VERSION`), `storage::engine` (getters pub(crate)). Sin duplicar layouts de keys (reusa `derived_delete_ops`/`posting_delete_ops`/`sparse_index_ops_for_replace`).
- **Veredicto impacto:** **medio-alto** — sin cambios de wire ni de formato on-disk (WAL v3 intacto; shred key format intacto, solo se borra). Hot path delete +1 delete best-effort (clase de durabilidad de `put` shred, `memory.rs:390`). `apply_delete_inner` NO se toca (txn comparte). Ninguna región co-batch tocada.

## Spec (feature-add con decisiones por evidencia)

| # | Decisión | Elegido | Evidencia |
|---|----------|---------|-----------|
| 1 | Alcance del wiring shred | `delete` (best-effort post-commit, `let _ =`) + `delete_batch` (op en la batch atómica) + `purge_permanent` (op en su batch) | Pre-mortem del plan F2 ("shred purge best-effort post-commit, misma clase de durabilidad que put :390, nunca bloquea el delete core"). Single-delete no puede fallar el delete por un purge secundario (WAL ya es commit point, `delete.rs:13-17`). Batch ya es todo-o-nada (`write_batch`), +1 op no cambia la clase de fallo. `apply_delete_inner` compartido con txn (`txn.rs` commit) → NO se toca (MVCC). |
| 2 | Vía de emisión | `Embedded::delete_certified(ns,key)` (refactor de `delete` a `delete_inner` compartido) | CLI y MCP ya pasan por `Embedded::delete` (`crud.rs:430`, `tools.rs:1561`); refactor mínimo sin duplicar lógica en bindings (regla del rol §2.6). |
| 3 | Superficies del certificado | Fijas, en orden: `store`, `shred`, `vector_index`, `vector_store`, `derived_index`, `text_index`, `sparse_index`, `version_history`, `wal` | Inventario del plan ("store+índices+shred+WAL") + pre-mortem F1 ("inventario por superficie con estado explícito — nunca silencio"). Cada una con `action`, `residues`, `evidence`. |
| 4 | Cómo se prueban los residuos de índices derivados/text | Reusar las **ops exactas del delete** (`derived_delete_ops`, `posting_delete_ops`, `sparse_index_ops_for_replace`) y re-leer cada key Delete con `get_from_partition` | Cero duplicación del layout de keys (`serialization/mod.rs:171/:252`, `text_index.rs:764`); las ops ya son la fuente de verdad del barrido. `residues` = keys aún presentes. |
| 5 | `vector_store` (bytes físicos) | `action: "tombstoned"` (si el record tenía vector); evidencia receipt + límite declarado (bytes físicos persisten hasta compactación de segmento) | `apply_delete_inner` escribe `FLAG_TOMBSTONE` (`delete.rs:118-128`); no hay secure-erase de bytes (fuera de alcance: "purga lógica/física", stop condition unlearning). Sin overselling. |
| 6 | `wal` surface | `action: "tombstone-recorded"`, `residues: 0` (la WAL es historia append-only por diseño: no se borra) + `chain` con referencia VER-01 | `delete.rs:44` appendea `WalRecord::Delete` antes del I/O; VER-01 (v3 encadenado) es el verificador (`vanta-cli verify`). El certificado **referencia** el chain — no lo re-verifica por delete (gate de feature `cli` de `verify_wal_file` + coste O(WAL); firma/ancla externa = deuda v1.0 ya prevista por el plan L941/VER-01 §Diseño). |
| 7 | Integridad del certificado | `integrity.sha256` = SHA-256 hex sobre el JSON canónico del certificado con `integrity.sha256=""` (determinista; detecta edición/corrupción, **no autentica** — sin clave) | Pre-mortem F3 ("manipular el certificado rompe la verificación"). `sha2` ya es dep directa. Autenticidad (firma con clave del motor) = review `vanta-audit` (plan: "Firma/attestation criptográfica se decide con vanta-audit"). |
| 8 | `out_of_scope` declarado | Array fijo en el certificado: media física/secure-erase, backups/snapshots externos, segmentos WAL archivados, unlearning paramétrico, exports (gobierno WIRE-09) | Contrato "doc (alcance: purga lógica/física; NO unlearning paramétrico)" + stop conditions (backups → FIND documentado, no silencioso). Coordinación WIRE-09: los exports son copias fuera del perímetro del store. |
| 9 | Superficie CLI | `delete --attest` (emisión) + `CertificateCommand::Verify { file }` (consulta/validación, exit ≠0 si inválido) | Contrato "emitido por CLI y MCP, consultable". Patrón subcommand-enum existente (`WalCommand` `cli.rs:440`); `cmd_verify` (VER-01) como precedente de exit code. |
| 10 | Superficie MCP | Param opcional `attest: bool` (default `false`) en `memory_delete`; respuesta gana `certificate` solo con attest | Default seguro (Regla 4): sin flag, comportamiento idéntico. NO se agrega tool nuevo (mantiene conteos del meta-test `mcp_tests.rs:4603` y hints intactos). |
| 11 | Determinismo | Orden fijo de superficies, sin maps en el JSON (Vec), strings estáticos; único campo variable = `timestamp` | Test de doble emisión sobre DBs equivalentes (normalizando timestamp). |

## Diseño del certificado (explícito para revisión `vanta-audit` — P2-01)

**Schema v1 (`PurgeCertificate`, `src/attestation.rs`):**

```json
{
  "schema_version": 1,
  "timestamp": "2026-09-29T12:00:00Z",
  "namespace": "persona", "key": "alice", "node_id": "1234…",
  "reason": "memory delete",
  "status": "purged | residues | not_found",
  "surfaces": [
    {"surface":"store","action":"deleted","residues":0,"evidence":"Point-read of backend partition Default is absent"},
    {"surface":"shred","action":"deleted","residues":0,"evidence":"InternalMetadata key shred::<node_id> absent (JSON Shredding column store)"},
    {"surface":"vector_index","action":"deleted","residues":0,"evidence":"HNSW has no entry for the node"},
    {"surface":"vector_store","action":"tombstoned|none-needed","residues":0,"evidence":"Vector header FLAG_TOMBSTONE set at delete; physical bytes remain until segment compaction (declared limit)"},
    {"surface":"derived_index","action":"deleted","residues":0,"evidence":"NamespaceIndex + PayloadIndex keys reconstructed from the deleted record are absent"},
    {"surface":"text_index","action":"deleted","residues":0,"evidence":"Posting keys for the record terms are absent (stats recomputed by replace)"},
    {"surface":"sparse_index","action":"deleted|none-needed","residues":0,"evidence":"Sparse posting keys absent"},
    {"surface":"version_history","action":"deleted","residues":0,"evidence":"Versions partition prefix scan is empty"},
    {"surface":"wal","action":"tombstone-recorded","residues":0,"evidence":"WalRecord::Delete appended before store I/O (append-only log keeps the trace by design)"}
  ],
  "chain": {"scheme":"sha256-prev-hash","format_version":3,"chained":true,"status":"referenced","verify_command":"vanta-cli verify"},
  "out_of_scope": ["physical media / secure erase", "backups and snapshots outside the live DB dir", "archived WAL segments", "parametric unlearning", "exports (WIRE-09 governs them)"],
  "integrity": {"algorithm":"sha256","sha256":"<hex over canonical JSON with this field empty>"}
}
```

**Qué garantiza (y qué NO):**
- Garantiza: (a) el barrido corrió y las superficies enumeradas no tienen entradas **alcanzables** del registro; (b) el delete dejó tombstone en la WAL encadenada v3 (VER-01); (c) el JSON no fue editado sin cambiar el hash (integridad, no autenticidad).
- NO garantiza (declarado): bytes físicos (segments/page-cache/SSD remap), copias fuera del dir (backups/snapshots/exports/segmentos archivados), ni unlearning paramétrico. Sin ancla externa no hay firma → **review de `vanta-audit` decide** si se agrega firma con clave del motor (plan L941; VER-01 §Diseño lo dejó como follow-up).
- Verificación (`Embedded::verify_purge_certificate`): recomputa hash + re-escanea las superficies **re-verificables sin el record** (`store`, `shred`, `vector_index`, `version_history`, key exacta de `NamespaceIndex`); si alguna claim `0` falla → `Err`. Las superficies que dependen de las ops construidas al delete (payload/text/sparse) quedan cubiertas por el hash de integridad — límite documentado en `docs/api/CERTIFIED_DELETE.md`.

**Determinismo:** doble corrida sobre DBs equivalentes → cuerpos idénticos salvo `timestamp` y su `integrity.sha256` derivado. Superficies en orden fijo; sin `HashMap` serializado; evidencia = strings estáticos.

**Durabilidad del certificado (hallazgo del smoke cross-proceso):** `certificate verify` abre la DB **read-only** — y un open read-only no re-ejecuta el WAL (ERR-050b). Por eso el handler `cmd_delete_certified` **cierra (flush) la DB antes de retornar**: sin ese close, el HNSW reabierto resucita la entrada del nodo y el verify reporta `vector_index` con residuos (detectado en el smoke E2E y corregido — `db.close()?` tras el delete). El verify es, por diseño, un chequeo fuerte: exige que la purga esté persistida, no solo aplicada en memoria.

**Encoding del archivo (`--out`):** capturar el `--json` de stdout desde PowerShell mangla UTF-8 no-ASCII (`—` → `ΓÇö`), rompiendo el hash. `delete --attest --out <file>` escribe el archivo desde Rust (UTF-8 correcto) y `certificate verify` acepta tanto el certificado crudo como el envelope `{deleted, certificate}` del CLI. Uso recomendado en Windows: `--out`.

## Invariantes de dominio (handoff — MUST)
- El delete core NUNCA falla por la purga shred en `delete()` single (best-effort, WAL = commit point). En `delete_batch`/`purge_permanent` la op va en la batch (misma clase de fallo que el resto).
- `apply_delete_inner`/`stamp_deleted_in_backend` (txn/MVCC) NO se modifican.
- Formato on-disk intacto: `shred::{node_id}` (solo se borra), WAL v3 intacto (solo se cita), sin cambios de wire/record v2 (SCH-02).
- `WalRecord::Delete` sigue escribiéndose ANTES del store I/O (`delete.rs:44`) — el certificado no reordena el path.
- Certificado: nunca declarar `purged` con residuos > 0 (status honesto `residues`/`not_found`); `out_of_scope` siempre presente.
- Prohibido tocar regiones co-batch F4.2 (export_md/index/seed/proxy/crypto) y `src/wal*.rs`.

## Deuda técnica (Regla 6 — MUST)
**Saldo neto: ≥0.** Paga: elimina el gap "shred no purga entradas huérfanas" (Backlog:924) + cierra la promesa "forget certificable" sin overselling. Añade: +1 op backend por delete (best-effort) + módulo nuevo + superficie pública nueva (snapshot `public-api.txt` regenerado, +165 líneas netas) — sin deps nuevas (`sha2` ya directa).

**FIND candidatos (registrar por el LEAD/orquestador; `Backlog.md` prohibido en esta wave):**
1. Firma criptográfica del certificado con clave del motor (ancla externa) — decisión de `vanta-audit` (plan L941; VER-01 §Diseño lo difirió igual).
2. Purga de backups/snapshots externos y de segmentos WAL archivados (`wal_shipping`) — stop condition del plan ("declarar alcance store+índices+shred+WAL y FIND para backups").
3. Snapshot `public-api.txt` co-batch: regenerado incluirá símbolos en vuelo de VER-06/VER-03 si están en el tree — re-regenerar post-merge de la wave si hace falta (precedente VER-01 §Deuda 4).
4. Rotación del audit log (`audit.rs` `.1..N`) puede retener eventos históricos de delete — el certificado no cubre audit logs rotados (declarar en `out_of_scope` si el reviewer lo considera material).
5. **`cmd_delete` (no-attest) no cierra la DB** (pre-existente, no VER-02): mismo latente que el punto de durabilidad de arriba — un open read-only posterior puede ver HNSW stale; los opens read-write lo corrigen vía replay del WAL. Decidir con LEAD si uniformar `db.close()` en los handlers CLI (cmd_put/cmd_delete/cmd_delete_by_filter).
6. Coste del scan del certificado: ~9 point-reads + reconstrucción de keys por delete attestado (despreciable vs delete; solo corre con `--attest`/`attest:true`). Sin acción salvo que el uso attestado se vuelva masivo (entonces batch/async). Dueño sugerido: `vanta-tuner` si aplica.

## Definition of Done (3 niveles)
- **Task:** C1–C4 con evidencia (§Verificación); gates `cargo fmt --check` + clippy `-D warnings` + nextest scoped + workspace audit (excl. co-batch si hace falta); `validate-docs-coverage` ✅ para los comandos nuevos.
- **Commit (LEAD):** `feat(core): VER-02 — borrado certificado (shred→GC→WAL purga + attestation por superficie)` — lo ejecuta el LEAD.
- **Review:** P2-01 con contexto fresco — **diseño del certificado revisado por `vanta-audit`** (§Diseño = ARTIFACT/CONTRACT del review). NO self-review.

## Herramientas necesarias
- `cargo check -p vantadb` · `cargo clippy --workspace --all-targets --all-features -- -D warnings` (o `-p vantadb --lib --bins` si el co-batch tiene el `--all-targets` bloqueado) · `cargo fmt --check` · `cargo nextest run --profile audit -p vantadb --test certified_delete` + `--lib` scoped (shred/storage::engine::/gc/attestation) + full `-p vantadb` · `CARGO_BUILD_JOBS=2`.
- Snapshot API: `VANTADB_PUBLIC_API_UPDATE=1 cargo nextest run -p vantadb --test public_api` (nightly ✅).
- MCP: `cargo test -p vantadb-mcp --test mcp_tests test_memory_delete_attest_emits_purge_certificate -- --exact` (I-1: el binario `mcp_tests` está excluido del default-filter de nextest → se corre con `cargo test` scoped; precedente `heavy-certification.yml`).
- Docs: `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --write` + `pwsh scripts/validate-docs-coverage.ps1`.
- FIND-173 (MCP verify cmd): usar `pwsh -Command "& ./script; exit $LASTEXITCODE"` si se valida por MCP. FIND-177: no relinkear `vantadb-server.exe` (no aplica acá; `vanta-cli.exe` sí es reemplazable).

## Steps

| # | Step | Estado | Evidencia |
|---|------|--------|-----------|
| 0 | DISCOVERY (plan Task 38 entero, delete path, gc, sdk, wal chain, cli, mcp, rules, skills) | ✅ | §Re-baseline + §Impacto mapeado + §Spec |
| 1 | RED wiring: tests unit (delete → shred; delete_batch → shred; purge_permanent → shred; GC sweep → shred) | ✅ | RED real: **5 tests fallaron** antes del wiring (4 engine + 1 gc) + **2 tests shred** (`delete_missing_entry_is_noop`, `delete_op_targets_internal_metadata` — añadidos con el GREEN) = **7 tests de wiring** (I-1 corrige el "5 tests" inicial) |
| 2 | GREEN wiring: `shred/mod.rs` (store_key/delete_op) + `delete.rs` (3 capas) | ✅ | 19/19 `shredded` + 388/388 `storage::engine` + 13/13 `gc::tests` |
| 3 | RED attestation: unit + integración `tests/certified_delete.rs` | ✅ | RED real: 10 errores E0599 (`delete_certified`/`verify_purge_certificate` no existían); 2 bugs de test propios encontrados y corregidos (call a `text_index_ops_for_replace` post-delete re-decrementaba stats → fix a ops de claves puras; WAL shard naming `vanta.shardN.wal` + flush antes del scan) |
| 4 | GREEN attestation: `src/attestation.rs` + `Embedded::delete_certified`/`verify_purge_certificate` (refactor `delete_inner`) | ✅ | 4/4 unit attestation + 5/5 E2E SDK (purged/0 residuos, determinismo, tamper, residuos reaparecidos, not_found honesto) |
| 5 | CLI/MCP: `Delete --attest [--out]`, `CertificateCommand::Verify`, handler, dispatch, MCP param `attest` + handler | ✅ | `cargo check --bins` + clippy lib/bins ✅ · MCP `--all-targets` clippy ✅ |
| 6 | Tests CLI/MCP + E2E completo (3 canales) | ✅ | `certified_delete` 8/8 (2 CLI) · MCP `test_memory_delete_attest_emits_purge_certificate` 1/1 + meta-test annotations + CRUD flow ✅ |
| 7 | Docs (CERTIFIED_DELETE.md + CONFIGURATION.md quirúrgico + gate docs) | ✅ | `validate-docs-coverage` **0 gaps** (46 CLI cmds) · check-links within budget · gen-index --write · (check-docs GATING ❌ por `VER-06.md` co-batch sin frontmatter — NO mío) |
| 8 | Verify full + snapshot API + cierre task file + RESULTADO | ✅ | `-p vantadb` completo **2535/2535** · lib **2293/2293** · clippy lib/bins/test + mcp all-targets ✅ · fmt ✅ · snapshot regenerado + PASS · smoke CLI ✅ (válido exit 0 / tamper exit 1) · docs 0 gaps · recitation/RESULTADO §7 (sin commit ni self-review — LEAD) |

## Pendientes — batch de fixes del review `vanta-audit` (2026-09-29, ❌ CHANGES REQUIRED → resuelto)

| # | Sev | Hallazgo | Fix aplicado |
|---|-----|----------|--------------|
| H-1 | High | `verify_certificate` claim-driven: cert con `surfaces: []` (o sin la superficie viva) + hash recomputado evade el re-scan y "verifica" con residuos vivos | **Schema validation** en `verify_certificate` + `finalize`: 9 superficies canónicas exactamente-una-vez, `status ∈ {purged,residues,not_found}`, `out_of_scope` no vacío (`validate_schema`, `src/attestation.rs`). **Veredicto residues-aware**: `Err` si `residues_now > 0 && status != "not_found"`. `residues_now` ahora también en la salida **humana** del CLI. Test nuevo: claimless + hash recomputado + fichero vivo ⇒ verify falla (además del clásico "residuos reaparecidos"). 6 unit tests de schema + 1 E2E nuevo |
| M-1 | Medium | Requisito flush/close no documentado; SDK/MCP no flushean → verificación cross-proceso puede dar falso negativo | Sección **Durability** en `docs/api/CERTIFIED_DELETE.md` + `# Durability` en el docstring de `Embedded::delete_certified` + nota en `verify_purge_certificate` (read-only no replay WAL; CLI cierra; SDK/MCP documentados; fallo seguro) |
| L-1 | Low | `not_found` no remedia huérfanos pre-VER-02 (shred viejo → `shred.residues=1`) | Documentado en `CERTIFIED_DELETE.md` §Scope: **reportar, no purgar in-place** (borrar la fila `shred::` de un nodo ausente podría destruir datos no pedidos); los reportes `not_found` verifican con `residues_now` visible |
| L-2 | Low | Certificado no ligado a una instancia de DB | Documentado en §Scope + docstring de `verify_purge_certificate`: verifica por ns/key/node_id contra cualquier DB; emparejar con path/backup identity en registros propios |
| I-1 | Info | Inexactitudes del task file | Corregidas: `cli_handlers/attest.rs` NO existía → handlers en `crud.rs`; "+5 símbolos" → +165 líneas netas reales del snapshot; comando MCP → `cargo test -p vantadb-mcp --test mcp_tests test_memory_delete_attest_emits_purge_certificate -- --exact`; "5 tests RED" → 7 tests de wiring (4 engine + 1 gc + 2 shred) |
| I-2 | Info | Retención de ns/key (y rotados) del audit log | Añadida a `out_of_scope` del certificado (`declared_limits()`) + lista de la doc + assert unit |

## Verificación (batch re-review — comandos y resultados)

| Comando | Resultado |
|---|---|
| `cargo nextest run --profile audit -p vantadb --lib attestation` | ✅ **10/10** (4 previos + 6 nuevos de `validate_schema`/`finalize`) |
| `cargo nextest run --profile audit -p vantadb --test certified_delete` | ✅ **9/9** (8 previos + `verify_rejects_claimless_certificate_even_with_recomputed_hash`) |
| Regresión completa `cargo nextest run --profile audit -p vantadb` | ✅ **2542/2542** (+7 vs ronda previa: 6 schema + 1 E2E H-1) |
| MCP `cargo test -p vantadb-mcp --test mcp_tests test_memory_delete_attest_emits_purge_certificate -- --exact` | ✅ 1/1 |
| Spot wiring: `--lib shredded` + `--lib storage::engine` + `--lib gc::tests` | ✅ 19/19 · 388/388 · 13/13 |
| Smoke CLI (binario real, DB temporal): honesto → `verify` exit **0** (`ok:true`, `residues_now:0`) | ✅ |
| Smoke CLI: tamper (residues 0→9) → exit **1** (integrity mismatch) | ✅ |
| Smoke CLI: resurrección (re-put tras delete certificado) → exit **1** (residuos) | ✅ |
| Smoke CLI: **PoC H-1** (cert claimless + hash recomputado, fichero vivo) → **exit 1** (schema: superficies faltantes) | ✅ (antes: exit 0) |
| `cargo clippy -p vantadb --lib --bins -- -D warnings` + `--test certified_delete` + `-p vantadb-mcp --all-targets` | ✅ |
| `cargo fmt --check` | ✅ |
| `cargo nextest run -p vantadb --test public_api` (sin env) | ✅ snapshot al día (sin cambios de firma en este batch) |
| `pwsh scripts/validate-docs-coverage.ps1` | ✅ **0 gaps** (46 CLI cmds) |

## Recitation (worker → LEAD)

- **Objetivo:** VER-02 — borrado certificado (shred→GC→WAL + attestation por superficie), incl. batch de fixes del review.
- **Última acción:** fixes H-1 (schema + veredicto residues-aware + `residues_now` humano + test PoC), M-1 (Durability doc/docstrings), L-1/L-2/I-2 (docs + `out_of_scope` audit logs), I-1 (task file) + re-verify completo.
- **Resultado:** OK (carril worker). Pendiente SOLO carril LEAD: commit local + review P2-01 fresco (`vanta-audit` — delta de H-1/M-1).
- **Invariantes que se mantienen:** delete core sin cambio de semántica; `apply_delete_inner`/txn intactos; formato on-disk intacto; el certificado nunca declara `purged` con residuos.
- **Deuda:** FINDs 1–6 del task file (firma externa = `vanta-audit`; backups/segmentos archivados; snapshot co-batch post-merge; audit-log rotado ya declarado en `out_of_scope`; `cmd_delete` sin close — LEAD decide uniformar).
- **Próxima tarea si completa:** VER-04.

## Cierre
- **Commit:** NO (LEAD). **Review:** NO self-review — P2-01 fresco; `vanta-audit` revisa el §Diseño del certificado + el wiring del shred (señales: best-effort single-delete, límites sin ancla externa, integridad ≠ autenticidad, superficies no re-verificables sin record).
- **Handoff:** consumer ICP-02 (F5, métrica "0 PII"); VER-04 cita el mismo chain; docs lift a `docs/api/` hecho acá (wave permite docs).
