---
title: "Avance — Operaciones & API"
type: domain-log
status: active
tags: [vantadb, avance, ops, api, docs, backup, enterprise]
last_reviewed: 2026-08-29
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

### AUD-044: CLI search en DB fresca (2026-08-18)
- **Resultado:** ✅ handlers `search`/`similar-to-key`/`search-multi`/`search-all`/`count` abren engine read-write → SDK corre `ensure_indexes_current` (idempotente) — adiós `NotFound { text_index bm25 }` en DB nueva sin rebuild manual. Test regresión + manual `put`+`search` OK (score 0.2877). Commit `a1d92f03`. (ver docs/progreso/README.md)

### AUD-051: CLI `put --metadata` + filtros `__vanta_*` (2026-08-18)
- **Resultado:** ✅ flag `--metadata '<json>'` (object root, rechaza `__vanta_*`, paridad `validate_metadata`); docs aclaran que filtros aplican solo a metadata de usuario (`__vanta_*` nunca matchean); completions regenerados 4 shells. cli_tests 79/79. Commit `626dcc00`. (ver docs/progreso/README.md)

### MOD-12: `ensure_indexes_current` en arranque del server HTTP (2026-08-23)
- **Resultado:** ✅ `cli_server::run` corre `ensure_indexes_current` tras abrir el engine (guard `read_only`; twin del fix MCP-01 stdio) — búsqueda textual/híbrida funcional vía `/api/v2/*` en DB fresca sin rebuild manual (antes: 404 `text_index not found: bm25`). Test e2e `test_e2e_text_search_fresh_db` RED(404)→GREEN; e2e 12/12, clippy/fmt ✅. HTTP_API.md: advertencias obsoletas de rebuild removidas. Commit `5623e41f`.

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

### FND-07: Regla de observabilidad real (prometheus) + probe endpoint — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ `/metrics` responde con feed real de latencia de queries (prometheus) + regla R-3 en `.opencode/rules/server-mcp.md` (todo endpoint nuevo expone métricas reales, no placeholders). Commit `8820bdaf`.

### FND-22: CONTRIBUTING.md + triage de issues (post-launch) — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ `CONTRIBUTING.md` (commit convention, PR flow, gates) + guía de triage en `.github/`. Commit `d9beaa9a`.

### GOV-B4: openapi.yaml completo (~29 paths desde cli_server.rs) + gate paridad — migrado 2026-08-22 (ver docs/progreso/README.md)
- **Resultado:** ✅ openapi.yaml regenerado (35 paths / 40 ops, 29 `/api/v2/*`, version sincronizada); `scripts/check_openapi_parity.mjs` (stdlib-only) + step en gate-docs-21.yml. Commit pendiente del lead.
### MEM-54: Skills CRUD en server HTTP (P33 Task 4, H5) - 2026-08-22
- **Resultado:** OK — POST /api/v2/skills (create idempotente content-hash) + PUT/PATCH/DELETE /api/v2/skills/{skill_id} con query params owner_agent+expected_version (lock optimista MEM-06; stale = 409). Owner-mismatch devuelve el mismo 404 que missing (anti-enumeracion). Tests D19 x2 en cli_server (--features server), openapi.yaml parity OK (37 paths / 44 ops), HTTP_API.md actualizado.

### FIND-46: Doc drift semver-checks — Documentar cargo semver-checks en pre-release gate
- **Fecha:** 2026-08-29
- **Objetivo:** Resolver doc drift entre CI (gate semver-checks existe en ci-rust-10.yml:88-118) y docs de operations (no documentado).
- **Resultado:** ✅ Documentado el gate en docs/operations/ci-cd-guide.md (job table + sección dedicada con install local y scope antadb-only) + cross-ref en docs/operations/CI_POLICY.md (job table §1) + docs/operations/master-index.md last_reviewed actualizado. Contrato del plan verificado: cargo semver-checks --help → 92 líneas (path 1 ✅) + 6 matches de semver-checks en docs/operations/ (path 2 ✅).
- **Archivos tocados:** docs/operations/ci-cd-guide.md, docs/operations/CI_POLICY.md, docs/operations/master-index.md
- **Commit:** staged para vanta-lead (vanta-docs no hace commit)
- **Origen:** codegraph-20260827 Fase 11 + plan 2026-08-29-full-backlog-parallel.md W0-2
- **Pre-mortem cerrado:** install local documentado (cargo install cargo-semver-checks --locked); CI ya está automatizado vía taiki-e/install-action.

### MCP-40: Registro en el ecosistema MCP — `server.json` + listings
- **Fecha:** 2026-08-29
- **Objetivo:** Publicar VantaDB MCP en el Official MCP Registry (`io.github.ness-e/vantadb`) + listings secundarios (glama/smithery, passive). Cierra gap §6 P1-F del research mcp-research-20260825.
- **Resultado:** ✅ (vanta-worker no commitea — staged, await vanta-lead) `server.json` creado en raíz conforme al schema oficial 2025-12-11 (verificado vía `webfetch https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json` — `name`/`description`/`version` son los únicos required, todo lo demás opcional). `docs/operations/MCP_REGISTRY.md` documenta el manifest, submission state (pending, TBD PR), pre-mortem (schema bump, namespace verification), aggregator strategy (glama/smithery auto-scraping, sin manifests paralelos), y release-time update procedure. `docs/api/MCP.md` extendido con sub-sección "Registry manifest" linkeando al server.json y al doc nuevo. `docs/operations/master-index.md` actualizado (GOV-C5). Sin tocar código fuente. Blast radius: 0 archivos de código.
- **Contrato verificado:** `Test-Path server.json` = True; `Select-String -Path server.json -Pattern "modelcontextprotocol"` count = 2 (≥1 ✅); `python json.load` parsea OK (8 keys); 8 secciones `##` en `MCP_REGISTRY.md` (≥5 ✅); 3 hits de `server.json` en `MCP_REGISTRY.md` + `MCP.md`; 1 hit de `MCP_REGISTRY` en `master-index.md` (GOV-C5 ✅).
- **Archivos tocados:** `server.json` (nuevo), `docs/operations/MCP_REGISTRY.md` (nuevo), `docs/api/MCP.md` (extendida), `docs/operations/master-index.md` (GOV-C5), `.opencode/skills/campaign-executor/tasks/MCP-40.md` (task file).
- **Staged:** 5 archivos. vanta-worker no commitea (regla AGENTS.md §"Límites de herramientas por rol") — **BLOQUEO para vanta-lead**: `git commit -m "docs: MCP-40 — Registry manifest + ecosystem listings"`.
- **Origen:** plan 2026-08-29-full-backlog-parallel.md W0-1 (parallel 3 con FIND-46 y PROV-08).
- **Deuda abierta (no bloqueante):** `packages[]`/`remotes[]` ausentes — install via `cargo install --git`. Cuando se publique binario en `ghcr.io/ness-e/vantadb-mcp` (release firmado), regenerar server.json con OCI entry (FIND-49 propuesto). Submission PR al registry es manual (TBD).
