---
title: "Avance — Operaciones & API"
type: domain-log
status: active
tags: [vantadb, avance, ops, api, docs, backup, enterprise]
last_reviewed: 2026-08-07
aliases: []
---

# Avance — Operaciones & API

> Registro consolidado del trabajo en operaciones: API pública/documentación, backup/restore, CLI, telemetría, enterprise. IDs originales conservados.

## API pública & documentación

### REC-010: py.typed
- **Fecha:** 2026-07-31
- **Resultado:** ✅ marcador `py.typed` en paquete Python.

### REC-009: PQ analysis
- **Fecha:** 2026-07-29
- **Resultado:** ✅ Análisis de quantization (PQ) — ver `docs/progreso/2026-07-28-sdk-gap-audit.md`.

### DRV-068: API contract: Python `VantaDB` constructor options (storage_path/embedding_model)
- **Fecha:** 2026-07-12
- **Resultado:** ✅ `with_storage_path`, `with_embedding_model` en Python y en VantaMemoryConfig TS.

### VFY-002 / VFY-003 (API contract)
- VFY-002: validación opciones WASM — ver bindings.
- VFY-003: Paginar reindex_hnsw_from_text — ✅ commit `918df85`.

### DOC-09: docstrings Rust
- **Resultado:** ✅.

### DOC-10: README quickstart
- **Resultado:** ✅.

### DOC-13: Contributing docs
- **Resultado:** ✅.

### DOC-14: Datasets docs
- **Resultado:** ✅.

### DOC-15: Videos y notas de la web
- **Resultado:** ✅.

### DOC-17: README parity (EN/ES)
- **Resultado:** ✅ README-ES.md + parity script.

### DOC-18: Workshop docs
- **Resultado:** ✅.

### DOC-19: Versioned API docs build
- **Resultado:** ✅.

### DOC-20: mdBook full
- **Resultado:** ✅ Docs site mdBook (2026-07-25, P4).

### GH-124: Ejemplos doc-test API pública Rust
- **Resultado:** ✅ 7 doc-tests nuevos; `cargo test --doc -p vantadb` 11/11 pass. (ver core-engine.md)

### API contract sync (vanta-lead)
- **Regla:** cambios de firma pública de Rust se propagan a Python/WASM/TS antes de release; `cargo semver-checks` es gate pre-publish obligatorio.

---

## Backup / Restore / CLI

### REC-001: Foundation types para CLI/SDK
- **Fecha:** 2026-07-29
- **Resultado:** ✅ `Foundation` types en Python/SDK para backup/restore (ver 2026-07-28-sdk-gap-audit).

### REC-008: Backup design
- **Fecha:** 2026-07-29
- **Resultado:** ✅ Diseño de backup — ver `2026-07-28-sdk-gap-audit.md`.

### DRV-126: Paginación keyset
- **Resultado:** ✅ RESUELTO — SearchResults ya implementa paginación keyset + offset-based. (ver core-engine.md)

### COMP-009 (CLI/tools)
- `vanta-cli` dump/import (`.vdbdump`) — ver core-engine.md.

### REC-007: WAL Compaction + Vacuum CLI
- **Resultado:** ✅ `vanta-cli wal compact` / `vanta-cli wal vacuum`. (ver core-engine.md)

### AUD-033: Validación de args CLI + suite de tests en vantadb-server
- **Fecha:** 2026-08-14
- **Resultado:** ✅ `main.rs`: `is_known_flag` (-h/--help/--mcp) + `validate_args`; flag desconocido → error + hint + `exit(2)`; help precedence intacta. `tests/cli_args.rs` nuevo (5 tests, proceso vía `CARGO_BIN_EXE` + `output_with_timeout`). Nextest 5/5. Commit `ef0dfc5c`. (ver docs/progreso/README.md)

---

## Telemetría & monitoreo

### NUEVO-15 (ops): Prometheus metrics endpoint
- **Fecha:** 2026-07-24
- **Resultado:** ✅ `/metrics` en vanta-server; métricas: `vantadb_auto_tune_ef` gauge, `record_vector_index_routing` (COMP-028/OLD-21), core registry (`metrics/core/registry.rs`).

### Audit logging enterprise (TSK-107b)
- **Resultado:** ✅ `src/audit.rs` — ver core-engine.md (jsonl, `VANTADB_AUDIT_LOG_PATH`).

### PERF-36: Config hot-reload
- **Resultado:** ✅ env `VANTADB_*` + `update_config()`.

---

## Enterprise

### TSK-107b: Audit logging
- **Resultado:** ✅ (ver core-engine.md)

### TSK-108/109: (enterprise items) — estado en Backlog/PHASE
- **Estado:** Verificar en `docs/Backlog.md` fase enterprise.

> **Cruce:** config y gobernanza de `VantaConfig` viven en `src/config.rs`; cambios de configuración pública se documentan en `docs/api/`.