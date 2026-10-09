---
title: "TASK MEMG-15: Portabilidad/interoperabilidad — mapeo de frontera + export con integridad"
kind: task
description: "Mapeo documentado de formatos de frontera 2026 (AGENTS.md, MCP, AAIF/AIMEM/ALF/AMP, W3C CG) + integridad verificable del export JSONL (manifest sidecar sha256, contrato VER-02) + política de versionado/migración; firma criptográfica no sancionada → FIND (vanta-audit)"
---

# TASK MEMG-15: Portabilidad/interoperabilidad — mapeo de frontera + export con integridad

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 61, bloque F0-expandido L1753-1779)
- **Fuente:** Backlog `MEMG-15` (`docs/dev/Backlog.md:141`) + plan Task 61 + base VER-05/VER-06 (importers Mem0/Zep/Letta + `MEMORY_INTERCHANGE_FORMAT.md`) + VER-02 (`src/attestation.rs:731` "no digital signature") + FIND-302(c) (precedente MEMG-17: firma ML-DSA-65 no sancionada → vanta-audit)
- **Esfuerzo:** 🟠 3-5d | **Appetite:** max 1sem | **Stop (plan L1764):** sin decisión de firma en DISCOVERY → checksum + FIND de firma (mínimo viable)
- **Prioridad:** 🟠
- **Tipo:** Rust (crate `vantadb`; SDK + CLI handler + docs) — sin bindings nuevos
- **Turns estimados:** 8-12 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `61` en el campaign server)
- **Incógnitas (uphill):** 0 — resueltas en DISCOVERY (HEAD `b5327d84`, develop):
  - **Mecanismo de firma (pre-mortem #2):** NO se inventa modelo criptográfico. Evidencia: `src/attestation.rs:731` declara "no digital signature: integrity is an sha256 self-hash ... cryptographic signing is a vanta-audit decision"; MEMG-17 (FIND-302 c) dejó la firma ML-DSA-65 fuera de alcance por no estar sancionada (`VER-01 §Diseño`); `rg -i "ml-dsa|dilithium|sign\(" src/sdk src/attestation.rs` = 0. **Decisión: extender el contrato de integridad VER-02 (sha256 + límites declarados + verificación no claim-driven) al archivo de export**; firma criptográfica → FIND delegado a vanta-audit (stop condition L1764).
  - **Forma del artefacto:** sidecar `<path>.manifest.json` — header/footer line rompería el contrato "una línea = un `MemoryExportLine`" (el import contaría la línea como error); campo solo en `ExportReport` no sobrevive el transporte del archivo. Precedente local: MD export escribe `index.json` con hashes por archivo (`src/cli_handlers/export_md.rs:324-339`).
  - **Drafts de frontera 2026:** verificados por fetch (Gate CITAS) — el Backlog citaba `agent-life/agentmemoryprotocol` (404); URL correcta AMP = `github.com/agentmemoryprotocol/agentmemoryprotocol`; ALF = `github.com/agent-life/agent-life-data-format` (repo de la org correcta).
- **Pendientes (downhill):** 5 steps
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `61`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Superficies nuevas (aditivas):** `Embedded::verify_export_integrity`; tipos `ExportManifest`/`ExportIntegrityVerification`; campos `sha256`/`manifest_path` en `ExportReport`; helper crate-interno `write_export_manifest`. **Callers existentes que NO cambian de firma:** `export_namespace`/`export_all` (misma firma, escritura extra de sidecar), `import_file`/`import_records` (intactos, sin enforcement), `cmd_export` (mismo contrato CLI; escribe manifest). |
| Callees | `sha2::Sha256` (ya usado por `src/attestation.rs`; sin dep nueva), `serde_json` (ya), `resolve_export_path` (sandbox WIRE-09 existente), `Error::{Io, serialization, Validation}`. Cero dependencias nuevas. |
| Implicaciones | **API pública aditiva** — sin breaking, sin wire change (el JSONL v2 no cambia: el manifest es sidecar), sin migración de datos. `ExportReport` gana 2 campos con `#[serde(default)]` (consumidores viejos siguen parseando; desktop `native.rs` mapea campos explícitos → compila sin cambio; Python `export_report_to_pydict` se extiende con las 2 claves). Regresión: suites `-p vantadb` (lib + cli_tests + importers) se re-corren. Sin locks, sin concurrencia nueva. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `b5327d84`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `src/sdk/serialization/impl_export.rs` (:1-828 — `export_namespace`/`export_all`/`write_export_file`/`import_*` + tests inline existentes), `src/sdk/importers/mod.rs` (:1-373 — reglas VER-05/ADR-046 §D7, `Conversion::write_jsonl`), `src/sdk/types/record.rs` (:455-574 — `ExportReport`/`ImportReport`/`MemoryExportLine`), `src/sdk/mod.rs` (:14-43 re-exports), `src/sdk/serialization/mod.rs` (:54 `EXPORT_SCHEMA_VERSION=2`; :622 `export_line_from_record`).
  - `docs/api/MEMORY_INTERCHANGE_FORMAT.md` (:1-206 — contrato JSONL v2 completo, mapping tables, determinismo/idempotencia), `src/attestation.rs` (:1-80 scope/integrity/"no digital signature"; :640-799 `WriteReceipt` + `write_declared_limits` con el límite de firma), `src/cli_handlers/export_md.rs` (:100-169 `IndexEntry`/`IndexFile`; :260-399 escritura de `index.json` con hashes — precedente), `src/cli_handlers/data.rs` (:1-160 `cmd_export` JSONL), `src/cli.rs` (:95-164 `Commands::Export`), `src/bin/vanta-cli.rs` (:60-149 dispatch), `tests/cli_tests.rs` (:436-470 `test_cmd_export_and_import`), `tests/importers.rs` (:1-100, :578-628 roundtrips), `vantadb-python/src/lib.rs` (:1630-1739 `export_namespace`/`export_all`/`import_file`), `vantadb-python/src/convert.rs` (:507-514 `export_report_to_pydict`), `vantadb-mcp/src/handlers/tools.rs` (:2795-2854 export inline JSONL — sin sidecar posible), `docs/dev/tasks/MEMG-17.md` (formato canónico + precedente firma), `docs/dev/Backlog.md:141` (evidencia drafts).
  - Fuentes de frontera (fetch verificado 2026-10-05): W3C CG `w3.org/groups/cg/ai-agent-memory-interop/` ✅ · AAIF `datatracker.ietf.org/doc/draft-schemacommons-aaif/` (v00, 2026-06-25, expira 2026-12-27) ✅ · AIMEM `datatracker.ietf.org/doc/draft-vu-aimem-bundle/` (v00, 2026-06-14, expira 2026-12-16) ✅ · AMP `github.com/agentmemoryprotocol/agentmemoryprotocol` ✅ (URL del Backlog = 404, corregida) · ALF `github.com/agent-life/agent-life-data-format` (RC 1.0.0-rc.5) ✅ · AGENTS.md `agents.md` ✅.
- **Archivos referenciados hacia dentro (imports/deps):** `impl_export.rs` → `sha2::Sha256` (nuevo uso del mismo crate), `serde_json`, `super::EXPORT_SCHEMA_VERSION`, tipos en `sdk/types/record.rs`. `data.rs` (CLI) → helper `write_export_manifest` re-exportado crate-interno + `Sha256`. `sdk/mod.rs` re-exporta los 2 tipos nuevos. Docs: `MEMORY_INTERCHANGE_FORMAT.md` (contrato) + task file.
- **Referencias entrantes (grep/CodeGraph HEAD):** callers de `export_namespace`/`export_all` = Python binding (`lib.rs:1654,1695`), desktop `native.rs:717-747`, MCP `export` (no usa el path de archivo), `cmd_export` (CLI, escritura propia), tests (`impl_export.rs` inline, `tests/importers.rs`, `tests/cli_tests.rs`). `rg checksum src/sdk` = **0** (gap confirmado); `rg "manifest" src/sdk` = **0** (gap confirmado); `verify_export_integrity`/`ExportManifest` = 0 hits (símbolos nuevos). Ningún caller existente depende de la ausencia del sidecar (ningún test cuenta archivos del directorio de export).
- **Veredicto impacto:** **BAJO-MEDIO (aditivo)** — 1 archivo core (`impl_export.rs`), 1 archivo de tipos (`record.rs`), 1 re-export (`sdk/mod.rs`), 1 CLI handler (`data.rs`), 1 conversión Python (`convert.rs`), 1 doc de contrato + task file. Sin breaking, sin wire, sin deps, sin locks. Pre-mortem mitigado: (1) drafts mutan → doc con fecha + estado borrador; (2) firma → checksum + FIND (stop L1764); (3) scope multi-formato → slice = mapeo + integridad; conversores AGENTS.md/AMP/ALF → FIND.

## Contrato

"mapeo/adopción de formatos de frontera (AGENTS.md / MCP memory / drafts IETF-W3C) documentado + export con checksum/firma (mecanismo decidido en DISCOVERY: extender VER-01 chain / VER-02 receipt vs firma nueva) + versionado/migración de formatos de memoria; spec + slice mínimo verificable (roundtrip export→import con integridad)" (plan Task 61 L1762).

Verificación: tests RED→GREEN de integridad (inline `impl_export.rs` + `tests/cli_tests.rs`) + suite scoped `cargo nextest run --profile audit -p vantadb --lib --build-jobs 2` + `--test cli_tests` + `--test importers` + `cargo fmt --check` + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` + gates docs (`check-links` / `check-docs` / `gen-index --check` / `validate-docs-coverage.ps1`).

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): pre-respondido por el plan F0** — el contrato Task 61 (Gate Result ✅ DO) sanciona "export con checksum/firma (mecanismo decidido en DISCOVERY...)" y "spec + slice mínimo verificable (roundtrip export→import con integridad)"; el stop L1764 sanciona explícitamente el fallback "checksum + FIND de firma". Símbolos nuevos = realización directa del contrato (manifest + verificación), sin superficie fuera de la sanción. Precedente idéntico: MEMG-17 ("Gate D: pre-respondido por el plan F0").

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Mecanismo de integridad del export | A) **extender el contrato VER-02 (sha256 self-hash + límites declarados + verificación no claim-driven) al archivo JSONL** / B) firma criptográfica nueva (ML-DSA-65/COSE) — 0 hits, no sancionada (`attestation.rs:731`; FIND-302 c) / C) sin integridad (gap) | ✅ **A** — decidido-por-evidencia: VER-02 es el contrato local de integridad ("integrity, not authenticity"); la firma es decisión de vanta-audit (stop L1764: checksum + FIND). Validación de mercado 2026-06 (Major Labs: 0/6 sistemas firman memoria) + direcciones documentadas para FIND-309 (SAIHM ML-DSA-65 / PAM Merkle-DAG) |
| 2 | Forma del artefacto de integridad | A) **sidecar `<path>.manifest.json` (JSON determinista: sin wall-clock, namespaces ordenados — git-friendly VER-06)** / B) header/footer line en el JSONL (rompe "una línea = un registro"; el import la contaría como error) / C) solo campo en `ExportReport` (no sobrevive el transporte del archivo) | ✅ **A** — precedente `export_md.rs` `index.json` (manifest junto al export); determinismo VER-06 |
| 3 | Contenido del manifest | `schema_version` (manifest v1) + `format` + `export_schema_version` (=2) + `records` + `namespaces` (sorted) + `sha256` (hex sobre los bytes exactos del archivo) + `limits` (no-vacío, estilo VER-02: integridad≠autenticidad, alcance del hash, transporte inline sin sidecar) | ✅ — campos mínimos para verificar + declarar límites; sin material de clave |
| 4 | Superficie de verificación | A) **`Embedded::verify_export_integrity(path)` → `ExportIntegrityVerification { status: ok|mismatch|no_manifest, expected_sha256, actual_sha256, records, namespaces, limits }`** (paridad `verify_purge_certificate`; superficie única para bindings futuros) / B) free fn (más honesta — no usa el engine — pero duplica superficie) / C) `import_file` auto-verifica (rechazaría ediciones legítimas post-export; cambia semántica del import) | ✅ **A** — paridad con la verificación VER-02 existente; **import NO cambia** (verificación explícita, documentada) |
| 5 | Firma criptográfica | A) **no implementar; FIND delegado a vanta-audit** (mismo corte que MEMG-17/VER-02; el W3C CG explora ML-DSA-65 pero sin spec sancionada en el repo) / B) inventar esquema (prohibido por el orquestador y por el precedente) | ✅ **A** — stop condition L1764 |
| 6 | Mapeo de formatos de frontera | A) **sección nueva en `docs/api/MEMORY_INTERCHANGE_FORMAT.md`: tabla formato→qué define→relación con JSONL v2→estado, con fecha 2026-10-05 y "borrador/work in progress" por fuente** / B) doc nuevo separado (duplica el contrato; Nygard) / C) transcripción de los drafts (drift) | ✅ **A** — un contrato, un archivo (Regla "one fact, one file"); el slice adopta 1 formato de frontera conceptual (AIMEM: checksum + idempotencia + versionado ya alineados) y mapea el resto |
| 7 | Versionado/migración | A) **documentar la política existente + manifest: JSONL `schema_version` 1..=2 aceptado (normalización v1→v2 en import), aditivo dentro del major, breaking → rechazo (`unsupported memory export schema_version`), manifest v1 independiente** / B) diseñar migrador v3 sin necesidad (YAGNI) | ✅ **A** — la política ya existe en código (`serialization/mod.rs:696`); el doc la hace explícita |
| 8 | CLI `vanta-cli export` | A) **manifest también en el export CLI: hashing al escribir + helper crate-interno `write_export_manifest` (comportamiento preservado: mismo flujo, mismo spinner, mismo skip-si-vacío; + sidecar)** / B) refactor `cmd_export` → `export_namespace`/`export_all` (borra ~60 líneas duplicadas pero cambia UX: barra por registro, crea archivo vacío) | ✅ **A** — slice sin cambio de UX; el refactor del path CLI queda como `NOTICED BUT NOT TOUCHING` (FIND si el owner quiere) |
| 9 | Bindings | A) **Python `export_report_to_pydict` gana `sha256`/`manifest_path` (aditivo, trivial) + docstring; `verify_export_integrity` en bindings/MCP/TS → FIND** / B) exponer todo ahora (scope creep: 3 bindings) | ✅ **A** — consistencia mínima del report; exposición completa a FIND |
| 10 | Tests | A) **inline en `impl_export.rs` (patrón local del módulo) + 1 test CLI en `tests/cli_tests.rs`** / B) integration file nuevo (ceremonia; el módulo ya testea inline) | ✅ **A** — convención local |

## Research — profundización multi-fuente (owner rule 2026-10-06)

> Router `coordinated-web-search` cargado. En esta sesión **metasearch/argus no están conectados** (fallback documentado: `websearch` multi-engine + `webfetch`); toda URL citada fue resuelta con fetch directo (Gate CITAS) salvo indicación explícita.

| # | Fuente (URL) | Fecha | Claim que respalda | Verificación |
|---|--------------|-------|--------------------|--------------|
| 1 | `w3.org/groups/cg/ai-agent-memory-interop/` | consultado 2026-10-05 | W3C CG: celda de memoria cifrada, firma PQC (ML-DSA-65), envelopes DEK, audit anchors, erasure GDPR-17, crosswalks MCP/AAIF | ✅ fetch |
| 2 | `lists.w3.org/Archives/Public/public-ai-agent-memory-interop/2026Jul/0002.html` | 2026-07-16 (charter 2026-06-19) | CG charter v1.0: **normative home = `draft-saihm-memory-protocol`**; el CG NO desarrolla spec normativa competidora (publica Community Group Reports) | ✅ fetch |
| 3 | `datatracker.ietf.org/doc/draft-saihm-memory-protocol/` | v01 2026-05-27 (expira 2026-11-28) | SAIHM: protocolo capa-memoria compañero de MCP (ML-DSA-65, DEK por celda, CBOR, receipts, sharing con revocación, erasure criptográfica, 8 tools MCP) | ✅ fetch |
| 4 | `datatracker.ietf.org/doc/draft-schemacommons-aaif/` | v00 2026-06-25 (expira 2026-12-27) | AAIF: definición portátil de agente + Agent State checkpoints (SHA-256 canonical + firma); memory config | ✅ fetch |
| 5 | `datatracker.ietf.org/doc/draft-vu-aimem-bundle/` | v00 2026-06-14 (expira 2026-12-16) | AIMEM: bundle de memoria con checksum sha256 (JCS) + COSE_Sign1 detached + idempotencia + versionado | ✅ fetch |
| 6 | `github.com/agentmemoryprotocol/agentmemoryprotocol` | spec v0.1 (consultado 2026-10-06) | AMP: markdown-first, graph-native (wiki-links), portabilidad por directorio; **URL del Backlog (`agent-life/agentmemoryprotocol`) = 404 → corregida** | ✅ fetch |
| 7 | `github.com/agent-life/agent-life-data-format` | RC 1.0.0-rc.5 (consultado 2026-10-05) | ALF: ZIP con particiones JSONL + manifest + identity/credentials cifradas; round-trip preserva campos desconocidos | ✅ fetch |
| 8 | `agents.md` | consultado 2026-10-05 | AGENTS.md: formato abierto de instrucciones (Markdown), stewarded por Agentic AI Foundation (Linux Foundation) | ✅ fetch |
| 9 | `modelcontextprotocol.io` | consultado 2026-10-06 | MCP: estándar de conexión (transporte/tools); no define formato de memoria canónico | ✅ fetch |
| 10 | `arxiv.org/abs/2605.11032` | 2026-05-10 | Portable Agent Memory: transferencia con integridad criptográfica (Merkle-DAG + content addressing), capability-scoped disclosure, rehydration anti-inyección, JSON/CBOR | ✅ fetch |
| 11 | `arxiv.org/abs/2606.01138` | v4 2026-08-12 | memorywire: wire format de operaciones (remember/recall/forget/merge/expire) + provenance + HITL; compone con MCP | ✅ fetch |
| 12 | `majorlabs.co/reports/state-of-agent-memory` | 2026-06 | **0/6 sistemas líderes firman memoria o logran portabilidad full-fidelity**; "portable, signed, consent-aware record" = gap de mercado; Mem0 hash ≠ firma | ✅ fetch |
| 13 | `support.claude.com/en/articles/12123587-import-and-export-your-memory-from-claude` | consultado 2026-10-06 | Claude: import/export de memoria entre providers (claim "no lock-in" del mercado) | ⚠️ snippet de búsqueda (no fetch directo) — marcado |

**Learnings aplicados al contrato/spec:**
- El mecanismo elegido (sha256 manifest, contrato VER-02) coincide con la dirección de AIMEM (checksum sobre canonical bytes) y es un **subconjunto honesto** de SAIHM/PAM (que agregan firma/encryption); la validación de mercado (Major Labs 0/6) confirma que el gap es real y que la firma es decisión pendiente — FIND-309 queda con **dos direcciones documentadas** (COSE_Sign1/ML-DSA-65 y Merkle-DAG).
- La tabla de frontera del doc suma SAIHM (home normativo del CG), Portable Agent Memory y memorywire + baseline de industria (Major Labs).
- Hallazgo operativo: la URL del Backlog para AMP estaba rota (404) — corregida en doc + task file.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **Formato JSONL v2 intacto:** ninguna línea cambia; el manifest es sidecar. `schema_version` sigue en 2; el import no cambia de semántica (sin enforcement del sidecar).
  2. **`ExportReport` aditivo:** campos nuevos con `#[serde(default)]`; consumidores existentes (desktop/TS/Python) siguen funcionando sin cambio.
  3. **Verificación no claim-driven:** `verify_export_integrity` lee el archivo + manifest y re-hashea; nunca "confía" en el manifest sin comparar. `no_manifest` es estado honesto (no `ok`).
  4. **Firma nunca reclamada:** los límites declarados del manifest dicen explícitamente "integrity, not authenticity"; sin clave, sin firma, sin claim de autenticidad.
  5. **Sandbox de paths:** el sidecar se escribe junto al path ya resuelto por `resolve_export_path` (WIRE-09); no se introduce resolución de paths nueva.
  6. **Sin `unwrap`/`expect`/`unsafe` en código nuevo de producción; sin deps nuevas** (`sha2` ya presente en el crate).
  7. **WIP ajeno:** `opencode.jsonc` (M) + `dev-tools/heavy-test-lock.ps1` (??) en el árbol por otros — NO se tocan ni stagean; PROHIBIDO tocar `docs/pipeline-state.json`; commit con **pathspec**.
  8. **No tocar:** `src/wal.rs`, `src/storage/**`, `src/vector/**`, `src/engine.rs` (fuera de dominio).
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb --lib --build-jobs 2` · `cargo nextest run --profile audit -p vantadb --test cli_tests --test importers --build-jobs 2` · `cargo fmt --check` · `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` · `node scripts/docs/check-links.mjs` · `node scripts/docs/check-docs.mjs` · `node scripts/docs/gen-index.mjs --check`.
- **Deuda pendiente:** firma criptográfica del export (FIND-309 → vanta-audit); exposición de `verify_export_integrity` en bindings/MCP/TS + refactor `cmd_export`→SDK (FIND-310); conversores de frontera AGENTS.md/AMP/ALF + adopción AIMEM (FIND-311).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — reusa `sha2` (ya en el crate) y el patrón VER-02 (`CertificateIntegrity`/límites declarados); **elimina** la deuda "export JSONL sin checksum/integridad" (gap verificado: `rg checksum src/sdk` = 0) y **agrega tests de contrato** (manifest, tamper, roundtrip). Sin `unsafe`, sin clones en hot paths (export es operación admin), sin abstracciones especulativas (manifest = primitiva pedida por el contrato; tipos espejo de VER-02, no reimplementación). Deps: 0 nuevas.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | (a) export JSONL escribe manifest sha256 (sidecar determinista) + `ExportReport.sha256/manifest_path` RED→GREEN; (b) `verify_export_integrity` con `ok`/`mismatch`/`no_manifest` no claim-driven RED→GREEN; (c) roundtrip export→verify→import con integridad RED→GREEN; (d) CLI export escribe manifest (test); (e) doc: mapeo de frontera 2026 (fecha + borrador por fuente) + integridad + versionado/migración en `MEMORY_INTERCHANGE_FORMAT.md`; (f) FIND de firma registrado; (g) suites scoped + fmt/clippy + gates docs verdes |
| **Commit** | Commit atómico conventional `feat(sdk):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | Changelog release-plz (feature → minor); verify full scoped documentado |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius) + grep puntual (codebase-memory-mcp no conectado en esta sesión — fallback documentado)
- `cargo nextest` scoped por crate (loop TDD) + `campaign_verify_cmd` (verify mecánico)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- `vanta-review` (subagente, review P2-01 fresh-context) al cierre
- Gates docs: `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check`

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `writing-guidelines` · `writing-plans` · `incremental-implementation` · `test-driven-development` · `context-engineering` · `source-driven-development` · `doubt-driven-development` · `api-and-interface-design` (pinned) · `documentation-and-adrs` (pinned) + rol: `rust-write-tests`, `security-and-hardening`, `documentation-skill` (docs). `performance-optimization` excluida (no hot path — Regla 9 no dispara).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — trust boundaries tocados: (1) manifest = JSON sidecar **no confiable** al leer (verify): parseo con errores mapeados a `Error::Serialization` (test O1), sin panics; un manifest corrupto → error (no `ok`); (2) el sha256 es **integridad, no autenticidad** — límite declarado no-vacío en el manifest (contrato VER-02); sin clave/sin firma/sin claim; (3) sidecar escribe/lee junto al path resuelto por `resolve_export_path` (mismo sandbox, sin resolución nueva — verificado por reviewer); (4) sin material sensible en el manifest (solo hash + metadatos del export); (5) sin `unwrap`/`expect`/`unsafe` en producción (grep + clippy ✅). Checklist `security-and-hardening` sobre estos puntos: ✅.
- [x] **PERFORMANCE** — no aplica (export es operación admin fuera de hot paths): hashing streaming 64 KiB en `verify` (fix del techo OOM detectado por OCR — sin materializar el archivo) + buffer de línea reutilizable en el write path SDK/CLI (1 alloc total, no por registro). Regla 9 no dispara. ✅

## Review (P2-01 — fresh context)

**Ronda 1** (`vanta-review`, sesión `ses_ef08f5d6affeWh4yXDsCuQbk3G`): 🔴 REQUEST CHANGES — **sin cambios de código requeridos**; el bloqueo fue el gate Commit (R1: higiene de staging — el tree compartido tenía WIP de MEMG-18 en `Backlog.md`/índices) + 2 Low (O1: faltaban tests de manifest corrupto/archivo ausente; O2: `verify` decidía `ok` solo por digest, sin discriminator de formato) + 2 Nits (N1: sidecar stale en CLI export vacío — honesto, nunca falso ok; N2: docstrings sin mención del sidecar). Verificó adversarialmente: sha256 = bytes exactos (cross-check externo `Get-FileHash`), determinismo byte-idéntico, `no_manifest ≠ ok`, tamper → mismatch, corrupto/ausente → `Err` sin panic, JSON viejo deserializa, sin unwrap/expect/unsafe/panics en producción, 0 deps nuevas, docs↔código, AIMEM spot-check re-fetch ✅. Re-ejecutó: 4/4 + 2/2 + 10/10 + 58/58 + clippy lib + `--no-default-features` + gates docs ✅.

**Fixes ronda 1:** O1 → +3 tests (`verify_export_integrity_rejects_corrupt_manifest` → `Serialization`, `..._rejects_foreign_manifest_format` → `Validation`, `..._missing_file_is_error` → `Io`) · O2 → guard `manifest.format == EXPORT_MANIFEST_FORMAT` antes del veredicto · N2 → docstrings de `export_namespace`/`export_all` mencionan el sidecar · N1 aceptado (mismatch honesto; el path CLI vacío no escribe manifest — el archivo sí se crea/trunca vacío antes del early-return) · R1 → resuelto por los commits de MEMG-18 (`789aeb23`/`c9e94b96`): `Backlog.md` quedó solo con los hunks propios (verificado `git diff -U0`: 2 hunks, ambos MEMG-15); `docs/index.md`/`llms.txt` ya no están modificados → staging pathspec directo, sin hunk-filtering.

**Ronda 2** (misma sesión, re-verificación adversarial): **✅ APPROVE** — staging confirmado (16 archivos exactos, 0 WIP ajeno; `Backlog.md` 2 hunks propios; `docs/index.md`/`llms.txt` sin modificar) + fixes re-verificados con evidencia fresca (13/13 + 2/2, guard O2 con test no vacuo, fmt/clippy/`--no-default-features`/`vantadb_py`/gates docs ✅). 3 Nits no bloqueantes: N1 (frase de doc del path de error → aplicada), N2-a (`schema_version` sin guard — elección consciente, consistente con la política aditiva documentada), N2-b (rationale del N1 aceptado corregido → aplicado).

Revisor: vanta-review (fresh context, P2-01, rondas 1-2) · Enfoque: contrato L1762 + stop L1764 + claims de §Steps, re-ejecutados · Cómo se probó: nextest filtrado 13/13 y 2/2, fmt, clippy lib, `--no-default-features`, `vantadb_py`, gates docs, probes adversariales (hash externo `Get-FileHash` MATCH, corrupto/ausente/ajeno, determinismo, JSON viejo) · Veredicto final: **✅ APPROVE** (ronda 2; 0 Critical/High abiertos; 2 nits aplicados post-approval — solo docs).

## Steps

### Step 1 — RED: tests de integridad del export

- **Archivos:** `src/sdk/serialization/impl_export.rs` (tests inline)
- **Acción:** RED — 4 tests de contrato (patrón local del módulo, `in_memory_db()`): (1) `export_writes_integrity_manifest_sidecar` — export → `report.sha256` hex 64 + `report.manifest_path` existe; manifest parsea con `records`/`namespaces`/`limits` no-vacío; (2) `verify_export_integrity_ok_then_mismatch_on_tamper` — verify → `ok` con `expected==actual`; tamper 1 byte → `mismatch` (no error); (3) `verify_export_integrity_reports_missing_manifest` — export viejo sin sidecar → `no_manifest` (no `ok`); (4) `export_verify_import_roundtrip_is_intact` — export ns → verify ok → import a DB fresca → payload íntegro; re-export determinista (mismo sha256). Verificar que fallan por la razón correcta (símbolos ausentes).
- **Verify:** `cargo nextest run --profile audit -p vantadb --lib -E 'test(export)' --build-jobs 2` → RED correcto.
- **Evidencia:** ✅ RED verificado: `error[E0425]: cannot find type ExportManifest` + `E0609: no field sha256/manifest_path on ExportReport` + `E0599: no method verify_export_integrity` (14 errores, todos por símbolos ausentes — falla por la razón correcta). ✅ GREEN: `4 tests run: 4 passed`.

### Step 2 — GREEN: manifest + verificación (mínimo código)

- **Archivos:** `src/sdk/serialization/impl_export.rs`, `src/sdk/types/record.rs`, `src/sdk/types.rs`, `src/sdk/mod.rs`, `src/attestation.rs` (`hex_lower` → `pub(crate)`), `src/sdk/serialization/mod.rs` (re-export), `tests/sdk_serialization.rs`, `tests/proptest_serialization_roundtrip.rs` (constructores)
- **Acción:** GREEN — `record.rs`: `EXPORT_MANIFEST_SCHEMA_VERSION=1`, `EXPORT_MANIFEST_FORMAT`, `ExportManifest`, `ExportIntegrityVerification`, campos `sha256`/`manifest_path` (`#[serde(default)]`) en `ExportReport`. `impl_export.rs`: `export_manifest_limits()` (no-vacío, estilo VER-02), `manifest_path_for()`, `write_export_manifest()` (determinista: namespaces sorted/dedup, sin wall-clock), `write_export_file` hashea los bytes exactos al escribir (Sha256 streaming) + escribe sidecar; `Embedded::verify_export_integrity` (resolve path → sha256 del archivo → manifest si existe → status). `sdk/mod.rs`: re-export de los tipos nuevos. Sin `unwrap`/`expect`.
- **Verify:** `cargo nextest run --profile audit -p vantadb --lib -E 'test(export)' --build-jobs 2` → GREEN; `cargo fmt --check`; `cargo clippy -p vantadb --lib -- -D warnings`.
- **Evidencia:** ✅ 4/4 tests nuevos verdes · `cargo fmt --check` ✅ (fmt aplicado, solo archivos propios) · `cargo clippy -p vantadb --lib -- -D warnings` ✅. Regresión export/import: `--test memory_export_import --ignore-default-filter` **10/10** ✅ · `--test importers --test sdk_serialization --test proptest_serialization_roundtrip` **58/58** ✅.

### Step 3 — CLI: manifest en `vanta-cli export` + test

- **Archivos:** `src/cli_handlers/data.rs`, `tests/cli_tests.rs`
- **Acción:** `cmd_export` hashea al escribir (Sha256) y llama al helper compartido `write_export_manifest` (re-export crate-interno desde `serialization`); JSON output gana `sha256`/`manifest`; comportamiento preservado (spinner, skip-si-vacío). Test nuevo `test_cmd_export_writes_integrity_manifest`: seed → `cmd_export` → manifest existe + `verify_export_integrity` vía SDK = `ok` + import sigue funcionando (regresión del test existente intacta).
- **Verify:** `cargo nextest run --profile audit -p vantadb --test cli_tests --build-jobs 2`.
- **Evidencia:** ✅ `--test cli_tests -E "test(cmd_export)" --ignore-default-filter` **2/2** (test existente + nuevo) · clippy lib ✅ (helper crate-interno compila en `cli_handlers`).

### Step 4 — Docs + Python report keys

- **Archivos:** `docs/api/MEMORY_INTERCHANGE_FORMAT.md`, `vantadb-python/src/convert.rs` (+ docstring `lib.rs`), task file
- **Acción:** Doc (EN, reference): sección **Integrity manifest** (sidecar spec, verificación, límites declarados, import sin enforcement), sección **Frontier formats (2026-10-05 survey)** (tabla AGENTS.md/MCP/AAIF/AIMEM/ALF/AMP/W3C CG → relación con JSONL v2 → estado "borrador" por fuente; URL corregida AMP), sección **Versioning and migration** (schema 1..=2, aditivo, breaking→rechazo, manifest v1). `convert.rs`: `export_report_to_pydict` gana `sha256`/`manifest_path` + docstring actualizado.
- **Verify:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check`; `cargo check -p vantadb_py` (o `-p vantadb-python`).
- **Evidencia:** ✅ Doc: 3 secciones nuevas + description actualizada; URLs de frontera fetch-verificadas (Gate CITAS: W3C CG ✅ · AAIF ✅ · AIMEM ✅ · AMP ✅ URL corregida · ALF ✅ · AGENTS.md ✅ · MCP ✅ + profundización 2026-10-06: SAIHM ✅ · charter CG ✅ · PAM ✅ · memorywire ✅ · Major Labs ✅). ✅ `cargo check -p vantadb_py` 0 warnings (fix: re-export `write_export_manifest` gateado con `#[cfg(feature = "cli")]` — sin él, `--no-default-features` warn `unused_imports`). ✅ `docs/api/index.md` regenerado (1 línea — mía). Estado final índices globales: `docs/index.md`/`llms.txt` fueron regenerados y commiteados por las sesiones MEMG-14/MEMG-18 (en vuelo concurrente) e incluyen mi description; al cierre NO están modificados → el commit propio no los toca.

### Step 5 — Cierre: FIND firma + gates + review + commit

- **Archivos:** `docs/dev/Backlog.md` (FIND), task file (RESULTADO §7), `docs/dev/avance/activo/operaciones.md`
- **Acción:** registrar FIND de firma (vanta-audit: decisión criptográfica del export — COSE_Sign1/ML-DSA-65, precedente AIMEM §5.3) + FIND de exposición bindings/MCP/TS + FIND conversores de frontera; quitar fila MEMG-15 del Backlog; verify full scoped (nextest lib+cli+importers, fmt, clippy, validate-docs-coverage, gen-index); OCR delegation; review P2-01 (fork `vanta-review`); commit LOCAL `feat(sdk):` con pathspec; campaign close taskId `61`; `skill progreso`.
- **Verify:** comando completo del contrato + `scripts/validate-docs-coverage.ps1`.
- **Evidencia:** ✅ FIND-309/310/311 en Backlog + fila MEMG-15 removida · verify full scoped verde (lib 2363/2363 `--test-threads 2` — 1 timeout por carga concurrente en test HNSW ajeno, re-run aislado 35.9s ✅; cli 95/95; memory_export_import 10/10; importers+sdk_serialization+proptest 58/58; fmt ✅; clippy `--all-targets --all-features -D warnings` ✅; gates docs 4/4) · OCR delegation (13 archivos; 2 mejoras aplicadas: verify streaming + buffer reutilizable) · review P2-01 `vanta-review` R1 changes-required (staging WIP ajeno + O1/O2 Low) → fixes (3 tests + guard de formato + docstrings) → R2 **APPROVE** · commit LOCAL `55b9a532` (16 archivos, pathspec estricto, 0 WIP ajeno — post-commit `git show --stat` verificado) + commit de cierre `docs:` · campaign close taskId `61` ✅ (review payload fresh/approve) · `skill progreso` → `operaciones.md` + gates avance 1034/1034 + coverage 0 gaps. ⛔ sin push.

## RESULTADO (§7)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 5/5 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: 55b9a532 (+ commit de cierre docs:)
ARCHIVOS: docs/api/{MEMORY_INTERCHANGE_FORMAT,index,EMBEDDED_SDK,PYTHON_SDK}.md · docs/dev/Backlog.md · docs/dev/tasks/MEMG-15.md · docs/dev/avance/activo/operaciones.md · src/attestation.rs · src/cli_handlers/data.rs · src/sdk/{mod,types,types/record,serialization/mod,serialization/impl_export}.rs · tests/{cli_tests,sdk_serialization,proptest_serialization_roundtrip}.rs · vantadb-python/src/{convert,lib}.rs
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no (plan F0 pre-respondio) D:no (contrato sanciona slice) V:no (sin fallo repetido) C:no (nit N1 aceptado)
SKILLS_CARGADAS: campaign-executor · progreso · ponytail (base) · writing-guidelines · writing-plans · incremental-implementation · test-driven-development · context-engineering · source-driven-development · doubt-driven-development · api-and-interface-design (pinned) · documentation-and-adrs (pinned) · rust-write-tests · security-and-hardening · documentation-skill · coordinated-web-search (owner rule 2026-10-06)
```
