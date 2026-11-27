---
title: "Avance — Operaciones & API"
kind: review
status: active
description: "existía (VECTORSEARCH(...) → <field> ~ \"texto\", min = 0.88), endpoint de embeddings de"
tags: [vantadb, avance, ops, api, docs, backup, enterprise]
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
- **Estado:** Verificar en `docs/dev/Backlog.md` fase enterprise.

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
- **Resultado:** ✅ Documentado el gate en docs/dev/operations/ci-cd-guide.md (job table + sección dedicada con install local y scope antadb-only) + cross-ref en docs/dev/operations/CI_POLICY.md (job table §1) + docs/user/operations/master-index.md last_reviewed actualizado. Contrato del plan verificado: cargo semver-checks --help → 92 líneas (path 1 ✅) + 6 matches de semver-checks en docs/user/operations/ (path 2 ✅).
- **Archivos tocados:** docs/dev/operations/ci-cd-guide.md, docs/dev/operations/CI_POLICY.md, docs/user/operations/master-index.md
- **Commit:** staged para vanta-lead (vanta-docs no hace commit)
- **Origen:** codegraph-20260827 Fase 11 + plan 2026-08-29-full-backlog-parallel.md W0-2
- **Pre-mortem cerrado:** install local documentado (cargo install cargo-semver-checks --locked); CI ya está automatizado vía taiki-e/install-action.

### MCP-40: Registro en el ecosistema MCP — `server.json` + listings
- **Fecha:** 2026-08-29
- **Objetivo:** Publicar VantaDB MCP en el Official MCP Registry (`io.github.ness-e/vantadb`) + listings secundarios (glama/smithery, passive). Cierra gap §6 P1-F del research mcp-research-20260825.
- **Resultado:** ✅ (vanta-worker no commitea — staged, await vanta-lead) `server.json` creado en raíz conforme al schema oficial 2025-12-11 (verificado vía `webfetch https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json` — `name`/`description`/`version` son los únicos required, todo lo demás opcional). `docs/user/operations/MCP_REGISTRY.md` documenta el manifest, submission state (pending, TBD PR), pre-mortem (schema bump, namespace verification), aggregator strategy (glama/smithery auto-scraping, sin manifests paralelos), y release-time update procedure. `docs/api/MCP.md` extendido con sub-sección "Registry manifest" linkeando al server.json y al doc nuevo. `docs/user/operations/master-index.md` actualizado (GOV-C5). Sin tocar código fuente. Blast radius: 0 archivos de código.
- **Contrato verificado:** `Test-Path server.json` = True; `Select-String -Path server.json -Pattern "modelcontextprotocol"` count = 2 (≥1 ✅); `python json.load` parsea OK (8 keys); 8 secciones `##` en `MCP_REGISTRY.md` (≥5 ✅); 3 hits de `server.json` en `MCP_REGISTRY.md` + `MCP.md`; 1 hit de `MCP_REGISTRY` en `master-index.md` (GOV-C5 ✅).
- **Archivos tocados:** `server.json` (nuevo), `docs/user/operations/MCP_REGISTRY.md` (nuevo), `docs/api/MCP.md` (extendida), `docs/user/operations/master-index.md` (GOV-C5), `.opencode/skills/campaign-executor/tasks/MCP-40.md` (task file).
- **Staged:** 5 archivos. vanta-worker no commitea (regla AGENTS.md §"Límites de herramientas por rol") — **BLOQUEO para vanta-lead**: `git commit -m "docs: MCP-40 — Registry manifest + ecosystem listings"`.
- **Origen:** plan 2026-08-29-full-backlog-parallel.md W0-1 (parallel 3 con FIND-46 y PROV-08).
- **Deuda abierta (no bloqueante):** `packages[]`/`remotes[]` ausentes — install via `cargo install --git`. Cuando se publique binario en `ghcr.io/ness-e/vantadb-mcp` (release firmado), regenerar server.json con OCI entry (FIND-49 propuesto). Submission PR al registry es manual (TBD).

### GOV-TK9: Verificar URL `vantadb-examples` del checklist (Wave0 Task 3)
- **Fecha:** 2026-09-03
- **Objetivo:** el checklist piloto (doc de venta enterprise) tenía un paso clone a la org `vantadb` que no existe (FIND-17/ADR-0030: owner real `ness-e/*`).
- **Resultado:** ✅ ambas URLs verificadas live 404 (`github.com/vantadb/vantadb-examples` y `github.com/ness-e/vantadb-examples`); ningún repo existe en ninguna org → rama TODO-humano del contrato (no crear repos desde el agente). `docs/dev/operations/pilot-onboarding-checklist.md:51` ahora TODO explícito en inglés; `rg "vantadb/vantadb-examples" docs/user/operations/ docs/api/` = 0. Fila Backlog eliminada (con ella muere la cita a la ruta vieja `pilot-onboarding-checklist.md` — la canónica es `docs/user/operations/...`).
- **Archivos tocados:** `docs/dev/operations/pilot-onboarding-checklist.md`, `docs/dev/Backlog.md` (fila removida), `docs/dev/plans/2026-09-03-quality-gtm-wave.md` (Task 3 → ✅)
- **Deuda abierta:** crear el repo `vantadb-examples` (o apuntar a ejemplos reales) es acción humana externa.

### MKT-21 (ex MKT-18i): Compose demo multi-servicio VantaDB + Ollama (re-escalado: sin AnythingLLM)
- **Fecha:** 2026-09-03
- **Objetivo:** demo copypaste local-first (r/LocalLLaMA): `docker compose up -d` orquesta VantaDB + Ollama + AnythingLLM.
- **Resultado:** ✅ compose raíz con 2 servicios, tags explícitos: `ollama/ollama:0.33.2` (pin verificado vía Docker Hub API: digest de `latest` == `0.33.2`, multi-arch) + `build: .` con `image: vantadb/server:0.5.0` (== workspace/Dockerfile APP_VERSION). Cabecera: quickstart, RAM mínima ~4 GB, nota GPU CPU-default (docs.ollama.com/docker), volumen de modelos persistente. **Re-escalado por stop condition:** AnythingLLM NO soporta VantaDB como vector backend — evidencia: `server/.env.example` master (github.com/Mintplex-Labs/anything-llm): `VECTOR_DB` ∈ lancedb/chroma/chromacloud/pinecone/astra/pgvector/weaviate/qdrant/milvus/zilliz; sin glue inventado. Enlaces: `docs/user/operations/DEPLOYMENT_GUIDE.md` §3 + `docs/user/tutorials/02-local-rag-pipeline.md` (README no tiene bloque docker — verificado).
- **Contrato verificado:** `rg -ci "ollama|anythingllm|anything-llm" docker-compose.yml` = 14 (≥2 ✅); sin docker CLI en host → `docker compose config -q` reemplazado por parse PyYAML equivalente OK + assert de tags explícitos (sin `:latest`); run-time `up -d` diferido (sin daemon). `docker-compose.dev.yml` intacto.
- **Archivos tocados:** `docker-compose.yml`, `docs/user/operations/DEPLOYMENT_GUIDE.md`, `docs/user/tutorials/02-local-rag-pipeline.md`, `docs/dev/Backlog.md` (fila re-escalada), plan Task 6 → ✅.
- **Commit:** `abb6594c` (develop).
- **Deuda abierta:** upstream feature request a AnythingLLM (backend VantaDB) = acción humana; run-time verificación `up -d` pendiente en host con daemon; si SRV-07 publica imagen oficial, swap `build: .` → `image:` publicada (nota inline en el compose).


### SRV-07: Dockerfile unprivileged + wiring release (quality-gtm wave1)

> **Retirado 2026-10-02:** Docker eliminado del repo por decisión del owner — job `docker-image` borrado de `release-binaries.yml`, Dockerfiles/compose `git rm`, docs de deploy sin sección Docker (ver FIND-229).

- **Fecha:** 2026-09-03
- **Objetivo:** imagen reproducible y ejecutable sin root con uid arbitrario (patrón qdrant) + wiring honesto al pipeline RELEASE; sin decisión de registry (es de marca).
- **Resultado:** ✅ Builder del `Dockerfile` raíz reescrito: la capa "skeleton sources" era irrecuperable (cargo valida los 73 `[[test]]` + `[[bin]]` explícitos del root `Cargo.toml` al cargar el manifiesto; y el `COPY --from=builder /build/target/...` apuntaba a un cache-mount que nunca se commitea en la imagen → el build jamás pudo pasar). Ahora: `COPY . .` + cache mounts de BuildKit (registry+target) con `cp` del binario a path commiteado. Runtime: `chmod 777 /var/lib/vantadb` (data dir) → `docker run --user <uid>:<gid>` arbitrario funciona sin rebuild; `ARG VANTA_RUNAS_UID=1001` para override en build; `USER vantadb` no-root preservado. `.dockerignore`: `tests/` y `benches/` dejan de excluirse (requisito de validación de manifiesto); `data/` agregado (guard de contexto). Wiring: job `docker-image` (build-no-push) en `release-binaries-63.yml` — build + smoke unprivileged (`--user 10001:10001`: write-test en data dir + `vantadb-server --help` vía entrypoint real) + export `docker save` como asset del release. Docs: §"Run unprivileged (arbitrary UID)" en `DEPLOYMENT_GUIDE.md` §3; sección "Docker Image Publishing (SRV-07)" en `CI_POLICY.md` (por qué NO push: ghcr vs Docker Hub = decisión de marca + credenciales inexistentes).
- **Contrato verificado:** `rg -n "^USER|runas|RUNAS" Dockerfile` = 5 hits ✅; `rg -ci docker release-binaries-63.yml` = 9 ✅; `actionlint` exit 0 ✅; continuation-lint del Dockerfile OK (la clase de bug que rompía el build: 2 `&&` sin `\`) ✅. Sin daemon Docker local → `docker build/run` diferidos al job `docker-image` de CI (gate real en el próximo tag/dispatch, documentado en CI_POLICY, no fake). `docker-compose.yml` NO tocado: `/var/lib/vantadb` sigue el path del volumen → named volume hereda modo 0777, compose sin `user:` usa uid 1001 → no rompe; `docker compose config -q` sin CLI local, diferido a CI.
- **Archivos tocados:** `Dockerfile`, `.dockerignore`, `.github/workflows/release-binaries-63.yml`, `docs/user/operations/DEPLOYMENT_GUIDE.md`, `docs/dev/operations/CI_POLICY.md`, `docs/dev/Backlog.md` (fila SRV-07 eliminada), `docs/dev/plans/2026-09-03-quality-gtm-wave.md` (Task 5 ✅ + recitation), `docs/dev/avance/activo/operaciones.md`.
- **Deuda abierta:** (1) verificación end-to-end del build en CI al primer dispatch (pre-mortem del plan) — sigue vigente y ahora cubre también esta deprecación; (2) ~~`vantadb-server/Dockerfile` alternativo roto~~ ✅ RESUELTA por FIND-56 abajo (la deuda histórica de esta línea se conserva como registro); (3) push a registry cuando haya decisión de marca + credenciales.

### FIND-56: Deprecar `vantadb-server/Dockerfile` roto a favor de la imagen raíz
- **Fecha:** 2026-09-03
- **Objetivo:** eliminar el Dockerfile alternativo in-construible + dejar `hardening.md` §5 honesto (el doc lo presentaba como FUNCIONAL con `--target unprivileged` inalcanzable).
- **Resultado:** ✅ opción (b) DEPRECATE (Ponytail: borra 112L sin perder capacidad). Triple-bug verificado: (1) `COPY vantadb/Cargo.toml` — no existe `vantadb/` (`Test-Path`=False, crate raíz en `.`); (2) `cargo build --package vantadb-server` produce `vantadb-server` pero el stage copiaba `target/release/vanta-cli` (bin del paquete raíz); el entrypoint `vanta-cli server --http…` existe (`src/cli.rs:314`) pero para el binario equivocado — el real solo entiende `--mcp/--help` + env; (3) stage `release-binary` descargaba `vantadb-server-<V>-<ARCH>-…tar.gz` cuando el release publica `vantadb-<target>.tar.gz` (404 garantizado; además el release ya publica la imagen docker como asset). Capacidades preservadas: build estándar → `Dockerfile` raíz (cache mounts, OCI labels, smoke en CI job `docker-image` que buildea la raíz — 0 workflows tocaban el archivo eliminado); unprivileged → flags runtime (read-only/cap-drop/no-new-privs + tmpfs, ya en compose y DEPLOYMENT_GUIDE §3); release-binary → asset imagen del release. Ningún workflow referenciaba el archivo (`rg` en `.github/` = 0 paths).
- **Contrato verificado:** `Test-Path vantadb-server/Dockerfile`=False ✅; `rg "COPY vantadb/" vantadb-server/`=0 ✅; `rg -l "vantadb-server/Dockerfile"` ex-archives = solo Backlog (fila eliminada en este cierre) + esta deuda histórica SRV-07 (registro fechado, exenta) ✅; PyYAML `safe_load` OK ×2 con `build={context: .., dockerfile: Dockerfile}` ✅; `rg "release-binary|target:"` en composes=0 ✅; `rg "vantadb-server/Dockerfile|--target"` en hardening+GUIDE=0 ✅. Sin daemon docker en host → build/smoke diferidos al job `docker-image` de CI (precedente SRV-07/MKT-21 (ex MKT-18i), NUNCA fakes). actionlint N/A (0 workflows), cargo fmt/clippy/nextest N/A (0 Rust).
- **Archivos tocados:** `vantadb-server/Dockerfile` (baja trackeada ya registrada en `0a54a545`, 112 deletions; la copia del worktree — resurrección no-trackeada — eliminada en esta sesión vía `git rm`; estado final consistente worktree+index+HEAD), `vantadb-server/docker-compose.yml` (2 builds → raíz, hardening runtime intacto), `vantadb-server/docker-compose.prod.yml` (build → raíz, sin `target release-binary`/`args` muertos; env prod + resources intactos), `docs/user/operations/hardening.md` (§5 reescrito sobre imagen canónica + nota de deprecación; tabla comparativa corregida UID-mecanismo), `docs/user/operations/DEPLOYMENT_GUIDE.md` (1 línea: puntero vivo al archivo eliminado), `docs/dev/Backlog.md` (fila FIND-56 eliminada).
- **Deuda abierta:** verificación end-to-end en CI al primer dispatch/tag (job `docker-image`: build raíz + smoke `--user 10001:10001` + export como asset).

- **Origen:** Backlog FIND-56 (L219) + deuda SRV-07 (2) de esta misma página.

- GOV-TK3 (docs): drift yaml-real x3 - Resultado: doc-fix x3 (codigo verificado correcto); parity 5/5, parser 117/117, docs-coverage 0 gaps. Commit b3be4176 (2026-09-05).

### PRX-04: cache-preserving injection (plan 2026-09-08-backlog Wave2)
- **Fecha:** 2026-09-09
- **Objetivo:** inyección D29 preserva prompt caching (re-inyectar mismo body = no-op byte-a-byte por provider).
- **Resultado:** ✅ test 125 passed/0 failed (7 prx04_* nuevos) + clippy 0 + fmt 0. RESUME tras rate-limit sin pérdida.
- **Commit:** 7ecaff5f

### PRX-08: higiene y ceilings proxy (plan 2026-09-08-backlog Wave1)
- **Fecha:** 2026-09-09
- **Objetivo:** auth O(1) + self-loop fail-fast + caps sesión/buckets + writeback JSONL append-only + filtro mixed-tools.
- **Resultado:** ✅ test 118/118 + clippy 0 + fmt 0 (clippy final vía RESUME tras aterrizar MEM-66 6ad16fbf).
- **Commit:** 4bbc2804 + 3f5dab48 (cierre)

### PRX-09: cache exacto slice 1 (plan 2026-09-09 Wave1)
- **Fecha:** 2026-09-09
- **Objetivo:** `ExactCache` FIFO opt-in + hook session-path post-inyección; semántico = slice 2 DEFER.
- **Resultado:** ✅ test 142/0 (8 nuevos) + PRX-04 sin regresión + clippy/fmt 0; cap 4MB con 502 tipado.
- **Commit:** 50b40228

### PRX-02: fallback multi-upstream + retries (plan 2026-09-10-code Wave0)
- **Fecha:** 2026-09-10
- **Objetivo:** config `[upstreams]` + failover 429/5xx + backoff exponencial + health pasivo.
- **Resultado:** ✅ test 161/0 (prx02_failover 4/4) + clippy/fmt 0; solo reintentos idempotentes; commit tras RESUME (hook fmt bloqueado por WIP MEM-69, resuelto).
- **Commit:** 7272ce87

### PRX-03: cost tracking + virtual keys (plan 2026-09-10-code Wave1)
- **Fecha:** 2026-09-10
- **Objetivo:** `cost.rs` (PriceTable, CostTracker, ledger key×sesión×modelo) + enforcement 429 + `/snapshot` cost.
- **Resultado:** ✅ test 0 failed (lib 134 + prx03_cost 3/3) + clippy/fmt 0; fail-open; modo log-first; SSE wiring futuro.
- **Commit:** eb79dcd2

### PRX-06: tier routing (plan 2026-09-10-code Wave2)
- **Fecha:** 2026-09-10
- **Objetivo:** slots haiku/sonnet/opus + `/v1/responses` en tool-loop.
- **Resultado:** ✅ test 0 failed (134+11 suites, prx06 12/12 e2e) + clippy/fmt 0; Shadow default; D34 intacto; failover PRX-02 gratis.
- **Commit:** ebb17de3

### PRX-07: PII redaction egress (plan 2026-09-10-code Wave3)
- **Fecha:** 2026-09-10
- **Objetivo:** `redact.rs` + hook 5a + block/mask/log.
- **Resultado:** ✅ test 0 failed (12 nuevos) + clippy/fmt 0; fail-open; deny advisories pre-existente (SRV-06, no tocado).
- **Commit:** d4536d89

### PRX-09-slice2: semantic caching + TTL + LRU (plan 2026-09-10-code Wave4)
- **Fecha:** 2026-09-10
- **Objetivo:** similitud léxica local + TTL por entrada + LRU touch (embeddings reales DEFER).
- **Resultado:** ✅ test 216/0 (semántico + TTL/LRU) + sin regresión exact/PRX-04 + clippy 0.
- **Commit:** e8419ce4

### PRX-09-embeddings slice 3: EmbedProvider + embed-path en cache (sin plan file)
- **Fecha:** 2026-09-10
- **Objetivo:** similitud por embeddings reales: trait mínimo + stub offline + provider Ollama opcional.
- **Resultado:** ✅ lib 155/0 (4 tests embed nuevos: paráfrasis deploy/release HIT donde léxico MISS, miss no-relacionado, fallback en fallo, budget latencia ~16µs/lookup) + prx09 4/4 + 13 suites ✅ + clippy all-targets/all-features 0 + fmt propio 0; sin regresión exact/PRX-04; server.rs/config.rs/Cargo intactos. Resto: wiring `with_embedder` en AppState (server.rs owned por PRX-11) → re-registrar fila PRX-09 en Backlog.
- **Commit:** 15157d39 (+77585a4f docs cierre)

### PRX-10: guardrails por virtual key (plan 2026-09-10-code Wave6)
- **Fecha:** 2026-09-10
- **Objetivo:** allowlists de modelos por key (gate 1c + 403 guardrail_blocked).
- **Resultado:** ✅ test 0 failed (149+8 nuevos) + clippy/fmt 0; D34 intacto; retomó parcial S1/S2 del abort.
- **Commit:** 950df333

### PRX-11-slice1: translate lib pura (plan 2026-09-10-code Wave6)
- **Fecha:** 2026-09-10
- **Objetivo:** `translate.rs` request↔response + SSE mínimo + sanitize + guards, sin wiring.
- **Resultado:** ✅ test 0 failed (9/9 nuevos) + clippy/fmt 0; server intacto; slice 2 (wiring + thinking fina) DEFER.
- **Commit:** 5c75c16b

### PRX-11-slice2: thinking fidelity + contract (lib-complete, wiring DEFER)
- **Fecha:** 2026-09-10
- **Objetivo:** thinking↔reasoning por variante + `TranslateConfig`/`should_translate`.
- **Resultado:** ✅ 16/16 tests + clippy/fmt 0 + fuente protocolo citada; hook server-side DEFER (config.rs wave-owned, diff en 6 pasos en task file).
- **Commit:** c7361f94

### PRX-11-slice3: wiring opt-in (lead-inline, subagentes abortados ×3)
- **Fecha:** 2026-09-10
- **Objetivo:** hook 5d + `ProxyConfig.translate` + map-back con content-length fijo.
- **Resultado:** ✅ lib 163 + 15 suites 0 failed (8 tests hook nuevos) + clippy/fmt 0; default verbatim intacto; sin campo AppState (lee config).
- **Commit:** e6a12230

### GOV-TK8: BENCHMARKS §18 run sintético (plan 2026-09-10-code Wave4)
- **Fecha:** 2026-09-10
- **Objetivo:** curar evidencia cruda en §18 con comando reproducible + Regla 11.
- **Resultado:** ✅ R11 0 hits + append-only + hooks; benches no re-corridos (entorno contaminado documentado).
- **Commit:** f6950738

### PRX-13: contexto en tránsito (plan 2026-09-10-code Wave5)
- **Fecha:** 2026-09-10
- **Objetivo:** `context.rs` trim por budget (Performance/Balanced/Economy) + override por key.
- **Resultado:** ✅ test 0 failed (14/14 nuevos) + clippy/fmt 0; default disabled; safe-mode a nivel mensaje.
- **Commit:** 462488e9

### SRV-06: JWT HS256 offline (plan 2026-09-10-code Wave2)
- **Fecha:** 2026-09-10
- **Objetivo:** DISCOVERY arch + MVP auth (OIDC discovery DEFER vía ADR-0039).
- **Resultado:** ✅ server 58/58 + auth 3/3 + rotation/rbac 13/13 + server 42/42 + clippy/fmt 0; race git-add paralela revertida.
- **Commit:** a0a3087f (+ADR-0039)

### PRX-09-wiring: verificada ya-implementada (sin cambios)
- **Fecha:** 2026-09-10
- **Objetivo:** confirmar wiring `with_embedder` tras PRX-11-slice3.
- **Resultado:** ✅ hunk intacto en server.rs:127-138 + prx09 6/6 + suites 0 failed + clippy/fmt 0; cero ediciones.
- **Commit:** 2c97d911 (pre-existente)

### FIND-65: checked_sub en test TTL de vanta-proxy (solo test)
- **Fecha:** 2026-09-15
- **Objetivo:** test ttl_expiry_predicate paniqueaba con uptime <2.7h (Instant - 10_000s); fix con checked_sub + sleep/TTL diminuto, prod elapsed() intacto.
- **Resultado:** ✅ suite cache 18/18 + fmt OK + clippy vanta-proxy 0 warnings (clippy -D global bloqueado por dead_code pre-existente txn.rs:158 → FIND-93).
- **Commit:** 432eee45 (3 files: cache.rs test + task file + Backlog)

### FIND-71: embeddings peso + verify smoke + sizes re-medidos
- **Fecha:** 2026-09-15
- **Objetivo:** ALLOW_PATTERNS global traía *.bin+*.safetensors (3-4× lo declarado); verify_model dummy sin documentar; sizes README por re-medir; lock sin validación repo/rev.
- **Resultado:** ✅ base recortada (sin *.bin) + MODEL_PATTERNS/get_allow_patterns por modelo; --check --only 9/9 verde + tamper-lock detectado; smoke documentado (docstring + verify.log + README); sizes = manifest (253/170/878/2200/941/1079/691/3470/16000MB); download --check + verify --check + py_compile + diff-check verdes offline; review vanta-review approve.
- **Commit:** 94c94f25 (4 files: download.py, verify.py, README.md, docs/dev/tasks/FIND-71.md)

### FIND-67: QUICKSTART a 0.5.0 (+ legacy `WANTA_*`)
- **Fecha:** 2026-09-15
- **Objetivo:** puerta de entrada veraz ×4 (boundary 0.5.0, wheel `./dist/vantadb_py-0.5.0-*.whl`, `VANTADB_*`, last_reviewed) + HALLAZGOS inline (`db.search`, nota audit/rebuild).
- **Resultado:** ✅ 23L/6 hunks + revalidación literal (put/get/list/search/export/audit/verify) + coverage 0 gaps; review trio approve.
- **Commit:** b5d9b3f8

### FIND-72: CLI argparse + pins + WinError32 en benches py
- **Fecha:** 2026-09-16
- **Objetivo:** `--help` exit 0 sin correr benches (defaults exactos preservados) + floors 7 deps + 8 `rmtree` con `ignore_errors` + notas WinError32.
- **Resultado:** ✅ py_compile 0 + diff-check + OCR 0 Critical/High + `/cleanCA` 0 🔴; review approve.
- **Commit:** 107550d8

### FIND-74: requirements + enlaces + decisión TS
- **Fecha:** 2026-09-15
- **Objetivo:** `vantadb-py>=0.5.0` + 1 línea QUICKSTART→examples + 1 fila README→demo/colab + decisión TS (referenciar `vantadb-ts/examples/`, no mover); hunks FIND-67 intactos.
- **Resultado:** ✅ diff-check + coverage 0 gaps + OCR sin Critical/High; review P2-01 approve.
- **Commit:** 5e428aea

### FIND-81: higiene `vantadb-server/` (artefactos + README)
- **Fecha:** 2026-09-15
- **Objetivo:** resto probado (writer-only + mtimes stale + exe vivo con `--db` distinto) → borrados locales + README 5L; `.gitignore` ya cubría (no editado, ponytail).
- **Resultado:** ✅ check server 0 warnings + diff-check + hooks; review trio approve.
- **Commit:** 9713eb54

### EMB-11: asistente de instalación embeddings con defaults + ORT nativo ≥1.27
- **Fecha:** 2026-09-16
- **Objetivo:** wizard `setup-embeddings.ps1` (`-NonInteractive` todo por default: local + e5-small; interactivo Enter=auto) + ORT 1.30 en store persistente + `ORT_DYLIB_PATH` en sesión; secrets nunca a disco.
- **Resultado:** ✅ NI exit 0 + `download.py --check` verde + rerun idempotente + secrets audit 0 findings + OCR sin Critical/High; review P2-01 approve.
- **Commit:** a3c4d903

### EMB-12: script lanzador MCP con env explícito + smoke dim==384
- **Fecha:** 2026-09-16
- **Objetivo:** `vanta-mcp-local.ps1` (params `-DbPath` mandatory + env sesión + ORT autodetect + arranque `server --mcp --db`).
- **Resultado:** ✅ smoke vivo `initialize` OK + `tools/list` 79 + `embed_texts` dim==384 + stdout stdio puro (fix Write-Host→stderr) + secrets audit CLAIM sostenido; review P2-01 approve.
- **Commit:** f80b9886

### CODEX-131: compose con loopback por defecto + auth documentada
- **Fecha:** 2026-09-19
- **Objetivo:** `docker-compose.yml` bind no-loopback sin key = contenedor muere antes del healthcheck (Codex P1).
- **Resultado:** `VANTADB_HOST=${...:-127.0.0.1}` + auth documentada sin secretos (footgun `Some("")` verificado, sin key por defecto); P2-01 approve.
- **Commit:** e0e74673 (rebase de 5b871993)

### EXE-03-prep: kit gate Fase A listo
- **Fecha:** 2026-09-19
- **Objetivo:** checklist + plantilla usuario-01 + README honesto (desbloqueador del anuncio, ejecución humana owner-side).
- **Resultado:** `docs/FASE-A.md` (checklist 5 familias + usuario-01 + auditoría README + handoff); coverage 0 gaps; P2-01 lead-docs approve (verificación abajo).
- **Commit:** 0a33a73e

### Notion N-01/N-02/N-04/N-05/N-06 (sincronización con código 0.6.1)
- **Fecha:** 2026-09-24
- **Objetivo:** backlog-notion N-01/02/04/05/06 aprobados por owner (N-03 sigue bloqueado por gate Fase A; N-07..N-11 diferidos).
- **Resultado:** ✅ SDKs-Plan00 (0 refs vieja API; nota migración corregida contra test_subclients AST-012) + api-reference (título v0.6.1) + Plan03 (tweets intactos, PROPUESTA intacto) + Roadmap (changelogs a hitos/criterios, v0.6.1 real, v0.5.1 no-publicada, sin ✅ sin release) + Propuesta (MCP 87, Python 48 en Client, skill_extract REAL vs extract_skills PROPUESTA) + Benchmarks (13/13 secciones redactadas, 0 pendientes, links verificados labelstud/Awesome-GraphMemory).
- **Filas removidas de backlog-notion.md:** N-01, N-02, N-04, N-05, N-06.

### Pipeline RUN plan estabilizacion-pendiente (FASE 1+2, 2026-09-24)
- **Plan:** docs/dev/plans/2026-09-24-estabilizacion-pendiente.md (EST-01..12).
- **Ejecutado:** EST-01 pin setup-python v7.0.0 + EST-02 ollama tests a Client/memory.* + EST-04 GOTOOLCHAIN go1.27.0 + lurkr skip sin .opencode + EST-03 ci-gate head-SHA (aprobado owner) + EST-06 dismiss #110 + EST-07 dismiss 82 test-log + EST-08 SARIF 404/time-box + EST-11 veredicto (release-npm-node.yml tags node-v*, OIDC).
- **Resultado:** code-scanning 83 OPEN -> 0; commits bcd62115 + 5128c2bc + 9705b434.
- **Pendiente:** EST-05 (verde en proximo run PR), EST-09 post-merge, EST-10 barrido, EST-12 puertas owner.
- **Actualización 2026-09-25:** EST-05 ✅ + EST-09 ✅ + EST-10 ✅ + merge #222/tag `v0.7.0`/publish ✅ → pendiente solo EST-12 (FASE-A + R-05, owner). Detalle: `docs/dev/avance/activo/ci-cd.md`.
- **Nota:** commit bcd62115 arrastro plan ajeno api-estandarizacion.md (paralelo, se deja intacto); su task API-STD-01.md queda untracked ajeno.

### EST-10: barrido API stale en superficies vivas (pre-0.5.0 → canónica)
- **Fecha:** 2026-09-24
- **Objetivo:** Migrar referencias a la API Python pre-0.5.0 (`VantaDB(`, `search_memory`, `get_memory`/`list_memory`/`delete_memory`, `AsyncVantaDB`) en superficies vivas: docs/user (glosario/operations/blog/learning), READMEs, `docs/api/PYTHON_SDK.md`, docs/dev/{vision,graphrag,architecture,operations,strategy}, skills, benches no-CI, evals, dev-tools.
- **Resultado:** ✅ ~230 reemplazos en ~75 archivos vivos → canónica (`Client`, `db.search`, `db.memory.get/list/delete`, `search_vector`, `AsyncClient`) + imports `vantadb_py`→`vantadb` (42) + fixes reales: `migrate/chroma.py`|`lancedb.py` `VantaDB(`→`Client(` (NameError latente), `validate_doc_snippets.py` (import + paths C-02), nota de naming falsa en README, tabla Cross-SDK en `vantadb-ts/README.md`, sección `Client` en api-reference, sección `search()` node-level de PYTHON_SDK → híbrida, cajas ASCII re-alineadas. `py_compile` 0 errores; coverage 0 gaps; skills mirror hash-SAME; históricos/MCP-scope intactos (lista de exclusión en el task file); 15 archivos legacy CRLF normalizados a LF (gitattributes). Verificación: `git grep` final solo con hits intencionales (wheels `vantadb_py-*.whl`, paths `vantadb_py.migrate`, notas de migración, guard AST-012).
- **Commit:** b461c9e8

### EST-05: verificación benchmark verde post-fix (perf-bench + API Docs) + review P2-01
- **Fecha:** 2026-09-24
- **Objetivo:** Confirmar `benchmark` (perf-bench) y `Check API Docs Version` verdes tras los fixes de la ola EST.
- **Resultado:** ✅ `benchmark` success en runs `36040468608` (EN el commit del fix `4b5b137e`) y `36086858245` (head `b461c9e8`); `Check API Docs Version` pass sobre PR head `3bbfbe45`. **Hallazgo del review:** el gate de regresión de perf-bench está **INERTE** (baseline vacío desde 2026-08-12 → compare warning+exit 0) → `FIND-153` creado (activar con `workflow_dispatch update_baseline=true`). Review P2-01 por `vanta-review`: ronda 1 ❌ (1C+1R+1O+~4N) → ronda 2 ❌ (FIND-153 sin registrar) → **ronda 3 ✅ APPROVE**. `campaign_verify_cmd` passed=true.
- **Commit:** (este commit)

### C-10: cierre README (paridad ES/EN + badges)
- **Fecha:** 2026-09-25
- **Objetivo:** Cierre total de paridad README tras C-02.
- **Resultado:** ✅ 4 fixes en `README_ES.md` (colab `blob/main`→`blob/develop`; fila "Ejecutar ejemplos ejecutables"; nota ADR-0047; **Trust block traducido** + nota "Fuente vigente" reubicada a §2 para paridad estructural con EN); verify v3 `campaign_verify_cmd` passed=true (0 missing links md+HTML, 0 badge drift shields.io, paridad ES/EN excl. switcher, colab positivo+negativo, Trust OK). Review P2-01 ✅ ronda 3 (ronda 1: badge-check muerto + Trust ausente; ronda 2: falso positivo del switcher → fix v3).
- **Commit:** ac46911d (+02e7d32a docs)

### WIRE-01: Loop de memoria del proxy + cost real + presupuesto de inyección
- **Fecha:** 2026-09-25
- **Objetivo:** cerrar loop inerte (namespaces disjuntos) + output_tokens real + tope del bloque `<vanta-memory>`
- **Resultado:** ✅ dual-write L1 + cost en path buffered + `InjectionConfig.max_tokens` (2000, 0=off); pivot A→B por evidencia (3 fallos PRX-09 por claves inestables); lead verify independiente; P2-01 vanta-review approve
- **Commit:** 679c75a9

### WIRE-10: Distribución P0 — install.sh macOS, Colab a Client, mcp-call + hooks sin pwsh
- **Fecha:** 2026-09-25
- **Objetivo:** promesa "1 comando" rota en macOS + puerta Colab rota + hooks mudos sin pwsh (C5/C9/C12)
- **Resultado:** ✅ install.sh fallback `command -v sha256sum || shasum -a 256` + normalización formato asset (fail-closed); notebook a `from vantadb import Client` + `db.search`/`db.memory.*` ejecutado e2e; `vanta-cli mcp-call` one-shot (JSON-RPC stdio, exit 0/1/2, placeholders `{{}}` stdin) + templates hooks 1.1.0 (memory_recall verbatim, reminders estáticos); P2-01 changes-required→aplicado (truncate UTF-8, reap zombie)
- **Commit:** 8e55e853

### WIRE-11: `llm-driver` always-on en MCP/proxy
- **Fecha:** 2026-09-25
- **Objetivo:** provider ollama/openai decorativo en silencio → funciona o falla con mensaje claro
- **Resultado:** ✅ 2 líneas Cargo (MCP + proxy) + test `llm_driver_fails_loud_on_unreachable_endpoint`; reqwest v0.12.28 única (dual TLS = patrón workspace); lead verify 1/1; P2-01 vanta-review approve
- **Commit:** 84cb2d19

### Notion N-17: sync páginas VantaDB Docs (2026-09-30)
- **Fecha:** 2026-09-30
- **Objetivo:** aplicar los borradores de `docs/dev/strategy/NOTION-SYNC-2026-09-24.md` (10 páginas + higiene) — aprobación owner via question.
- **Resultado:** ✅ 9 páginas actualizadas (append, sin borrados): Problema · Propuesta · Roadmap · SDKs · Benchmarks · Seguridad · Observabilidad · Gobernanza · Casos de uso + `Definición oficial` creada (`3ebd0445-9756-81bb-a6f2-c3b67157b866`) + higiene (3 duplicados archivados) + verificación reviewer + link fix. **Sync #2 (mismo día):** validación externa del marco — 6 páginas (áreas/Problema/dims/Propuesta/ámbitos/PI) con notas + 2 calificaciones inline, verificadas 6/6 + 2/2 (`NOTION-SYNC-2026-09-24.md` §14).
- **Filas removidas de backlog-notion.md:** N-17.
- **Commits:** `bc4958cd` (N-17) · `b9d1e459` (Sync #2).

### Auditoria de backlogs + rescates OLD (2026-09-30)
- **Movidas a Backlog-negocio:** STRAT-01/02/03/07, DX-10, FASE-A/R-05 (consolida EXE-03+EST-12) + BIZ-14/15.
- **Promociones a Backlog:** FUT-03/FUT-07 -> P28; FUT-17..23 anadidos a backlog-futuro; FIND-208..212; MKT-19; RES-16.
- **Rescates OLD:** DECISIONS-NOT-TAKEN (registro de descartes) + 7 archivos historicos (archive/) + tutorial 06 LM Studio + 5 tipos de contexto.
- **Commits:** 3ff933a5, d7861c3a, 3b8d9727 (+ este).

### Migración de formato de backlogs + auditoría PDFs OLD (2026-09-30)
- **Fecha:** 2026-09-30
- **Objetivo:** unificar los 4 backlogs al esquema canónico de 10 columnas (`ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias`) + eliminar filas completadas + registrar los 15 candidatos de la auditoría PDFs OLD + actualizar los consumidores del formato en `.opencode/` (task-system, comandos, skills).
- **Resultado:** ✅ esquema 10-col en `Backlog.md` / `Backlog-negocio.md` / `backlog-futuro.md` / `backlog-notion.md` (spec canónica: `.opencode/references/backlog-format.md`); filas completadas eliminadas (campaña F0–F6 + previas); secciones cerradas → notas de cierre; FIND-213..220 + BIZ-16/17 + FUT-24/25 registrados; nota de procedencia de los PDFs en `docs/dev/archive/PDFS-OLD-AUDITORIA-2026-09-30.md`; `.opencode/` actualizado (Gate H: cambios requeridos → aplicados).
- **Commits:** (+ este) · `.opencode`: commit propio (repo separado).

### Verificación de realidad del backlog (2026-10-01)
- **Fecha:** 2026-10-01
- **Objetivo:** validar cada ítem del backlog contra el código HEAD (7 sub-agentes en paralelo, read-only): campos, referencias file:line, vigencia del gap, estado.
- **Resultado:** ✅ ~276 ítems verificados (DELTA 49 · FIND 62 · P24–P59 ~60 · negocio 34 · futuro 22 · notion 26). **8 filas cerradas** (stale/implementadas): `DX-11` · `FIND-138` · `FIND-170` · `FIND-171` · `UX-08` · `UX-11` · `SHOW-03` · `HIG-02`. **~40 correcciones aplicadas** (evidencia con números/líneas actualizadas, rutas `web/`→`ness-e/Vantadb-web`, residuales re-scopeados). Hallazgos para owner (resueltos 2026-10-01): `WIRE-01/09` ✅ verificados (commits `679c75a9`/`6a0f6934`; registrados en avance — sin fila por diseño); `GOV-TK5` → renombrado `GOV-05`.
- **Commits:** (+ este)

### FIND-176: wizard ORT_DYLIB_PATH al archivo (4 sitios) + test de regresión
- **Fecha:** 2026-10-01
- **Objetivo:** El wizard dejaba `ORT_DYLIB_PATH` apuntando al DIRECTORIO del store (`ort` carga un archivo → embeddings en dummy). Fix en 4 sitios + normalización del path pre-seteado.
- **Resultado:** ✅ RED→GREEN 4→8 PASS (script `dev-tools/scripts/test-find176-wizard-ort-dylib.ps1`) · LoadLibraryExW A/B (126 vs 0) · gates docs 0.
- **Commit:** 3ae64a7d (local, sin push)

### Lista de congelados hasta 1.0 — aprobada (owner 2026-10-01)
- **Fecha:** 2026-10-01
- **Objetivo:** cerrar el alcance hasta 1.0 con lista cerrada (no criterio): superficies activas vs congeladas, whitelist de features nuevas, umbrales técnicos y de uso.
- **Resultado:** ✅ publicada en `docs/user/operations/EXPERIMENTAL_FEATURES.md` §Freeze List until 1.0 (34 componentes aprobados 1×1 por el owner; ajustes: proxy congelado sin publicar hasta 1.0.0 · server = host del scheduler + starter para embedders · Studio reducido a visor · 3 conectores activos).
- **Commits:** (+ este)

### Case studies no verificados — registro eliminado (C5, owner 2026-10-01)
- **Fecha:** 2026-10-01
- **Objetivo:** cerrar C5: destino final de los 2 case studies ficticios (documentos ya purgados en `00ee1ba2`; quedaba solo el README del archivo).
- **Resultado:** ✅ decisión owner: eliminar también el registro (`docs/dev/archive/case-studies-unverified/`) + refs actualizadas. El caso real llegará vía CLD-04 (enterprise pilot).
- **Commits:** (+ este)

### DX-12: Instalador/wizard como selector de módulos (enable/disable por componente)
- **Fecha:** 2026-10-04
- **Objetivo:** UX de entrada del producto: selección por componente (7 módulos con estado installable/frozen) + ambos caminos (interactivo/no-interactive) + idempotencia.
- **Resultado:** ✅ Wizard (`setup-embeddings.ps1`) con tabla de módulos + `-Modules <csv>`/`none`; gating real por módulo (MCP, TOML proxy, modelo/ORT, providers); frozen no finge soporte; disable = no re-escribir, nunca borrar. `install.ps1`/`install.sh` con pass-through + fix de honestidad (exit code del wizard ya no se enmascara como "✅"). Smoke completo: ambos caminos, `-Modules mcp,server`/`none`/desconocidos, idempotencia (hash-estable), dry-runs PS7+PS5.1+sh. Review P2-01: ronda 1 changes-required (High: coerción `[string]$Modules` en sesión) → fix → APPROVE.
- **Commit:** 9e211d10 (local, sin push)

### WIRE-14: Seam de host aditivo en el arranque (`run_with_hooks` + `BackgroundService`)
- **Fecha:** 2026-10-05
- **Objetivo:** ADR-0054 T1 — seam aditivo en el core que permite al host (`vantadb-server`) inyectar `ConversationTrigger` + servicios de fondo con parada graceful; prerequisito de WIRE-15/16 (el core no puede depender de `vanta-memory`).
- **Resultado:** ✅ `ServerHooks { conversation_trigger, background_services }` + `Default` inerte; `run_with_hooks(config, hooks)` aditivo (`run` delega con defaults, firma intacta); trait `BackgroundService` dyn-safe (futuro boxeado) + impl `MemoryTtlSweeper` (el sweeper se joinea por la vía genérica); parada graceful post-run-loop (espejo TTL sweeper, Drop best-effort documentado); tests: fake spawn/join determinista (watch + join) + facades + defaults pinneados. Verify: suite `-p vantadb --features server` 2675/2676 (único timeout = flake de carga pre-existente `concurrent_insert_preserves_hnsw_invariants`, re-corrido aislado ✅) + clippy/fmt/check-sin-server verdes. Review P2-01 `vanta-review` APPROVE (0C/0H/0M). Habilita WIRE-15/16.
- **Commit:** b5d294d2 (feat, local sin push) + 4448788b (docs task)

### WIRE-15: Servicio scheduler en `vanta-memory` (`run_pass` + loop con shutdown)
- **Fecha:** 2026-10-05
- **Objetivo:** ADR-0054 T2 — el crate de lógica aporta el pass pull-based (`run_pass` = timers + worker + reclaim, cero lógica de pipeline nueva) + loop helper feature-gated con parada graceful; WIRE-16 solo cablea el driver.
- **Resultado:** ✅ `services/scheduler.rs` nuevo: `run_pass` (TimerScanner → `l1_idle:` a L1 + `PipelineWorker::run_once` + `reclaim_stale`; reusa `MemoryTaskHandler` con defaults y cadencia L3 50) + `MemoryScheduler`/`spawn_memory_scheduler` (feature `http-server`; espejo `MemoryTtlSweeper`: watch + join + Drop best-effort — handoff WIRE-14 Low-1) + impl `BackgroundService` (seam WIRE-14, UFCS al inherent); factory `Fn() -> Option<R>` por pass (FIND-112) con skip observable sin runner (P4: tareas quedan encoladas, nunca bloquea); pass en `spawn_blocking`; single-writer documentado. Verify: suite `-p vanta-memory --features http-server` 676/676 (8 tests nuevos) + default 668/668 + `cargo check` default (sin uso de Tokio) + fmt/clippy 0. Review P2-01 `vanta-review` APPROVE (0C/0H/0M; Lows/NITs aplicados). Habilita WIRE-16.
- **Commit:** 1d5e1697 (feat, local sin push) + 9c881047 (docs task)

### WIRE-16: Wiring del wrapper `vantadb-server` (queue + bridge + loop)
- **Fecha:** 2026-10-05
- **Objetivo:** ADR-0054 T3 — `vantadb-server` como host del scheduler de memoria: cierra el residual MEM-55 (el puente existía y no estaba cableado); consume las superficies WIRE-14/15 (seam + servicio).
- **Resultado:** ✅ **Seam core aditivo**: `ServerHooks::on_storage_ready` (clon del `Embedded` del server post-open — single-writer: un segundo open del mismo path es `DatabaseBusy` y un segundo handle partiría audit/purge/supersede locks; evidencia dura en DISCOVERY); `run`/`run_with_hooks` firma+comportamiento intactos, defaults inertes, `ServerState` sin cambios. **Host**: dep directa `vanta-memory` (`http-server`+`llm-driver`); módulo `scheduler` (`VANTADB_SCHEDULER_INTERVAL_MS` default 60s/0=off; superficie runner FIND-112; degradado → `None` = skip P4 sin quemar tareas; bridge siempre cableado); `main` cablea solo HTTP mode (writer) con guard `read_only`; seam WIRE-14 re-exportado. **Tests**: 4 e2e (POST→L0→pass→l1 vía `run_with_hooks`; restart L0/L1 + cola efímera; disabled sin loop; sin runner skip) + pins core + unit. Verify: `-p vantadb-server` audit 12/12 + bins excluidos 39/39 + `-p vantadb --features server` 2678/2678 + check default + fmt/clippy×2 + delta build 6.9s. Review P2-01 `vanta-review` APPROVE (0C/0H/0M; Low-1/Optional-1/NITs aplicados; Low-2 → FIND-291). **Colateral bloqueante resuelto**: HEAD no compilaba `vantadb-mcp` (E0063 `core_search`, gap verify scoped de MEMG-11) → `c05cd77d`. Habilita WIRE-17.
- **Commit:** 679b3545 (feat, local sin push) + c05cd77d (fix mcp colateral) + ac012bd6 (docs task)

### WIRE-17: Docs del scheduler (wired status + promoción del rol)
- **Fecha:** 2026-10-05
- **Objetivo:** ADR-0054 T4 — documentar el scheduler cableado (post WIRE-16): wired status en `VANTA_MEMORY.md` §Operational modules + promoción acotada del rol en `EXPERIMENTAL_FEATURES.md` (Scope Budget); dependencia dura WIRE-16 cumplida.
- **Resultado:** ✅ `VANTA_MEMORY.md`: fila `conversation_hook` → wired por `vantadb-server` HTTP mode (writable — read-only skips el wiring) + fila/párrafo nuevos `services::scheduler` (pass pull-based `run_pass` + loop opt-in `VANTADB_SCHEDULER_INTERVAL_MS` 60000/0=off; runner FIND-112; skip P4; single-writer; MCP diferido) + 2 claims stale corregidos (decay/reflection leían WIRE-15 como futuro). `EXPERIMENTAL_FEATURES.md`: promoción del rol (filas :33 host resolved / :38 rol promovido acotado — **labs se mantiene, no thaw general** / :53 watchlist / :76 Freeze list / :144 claim `conversation_trigger: None` eliminado / :170 dream sin producer) + revision note. Gates: `check-links`/`check-docs`/`gen-index --check`/`validate-docs-coverage` exit 0 + markdownlint 0. Review P2-01 **adversarial** (HARD-02: `docs/api/**`) `vanta-review` APPROVE (0C/0H/0R; Low-1 aplicado; Low-2 dispensa+coordinación; NIT-1 aplicado; NIT-2 dispensado; out-of-scope → FIND-292). Cierra T4 de ADR-0054 (cadena WIRE-14→17).
- **Commit:** 75bd5da8 (docs, local sin push) + 20199096 (docs task)

### WIRE-18: Verificación adversarial del scheduler (crash mid-pass + restart)
- **Fecha:** 2026-10-05
- **Objetivo:** ADR-0054 T5 — cerrar la cadena WIRE-14→17 con la verificación adversarial del scheduler: crash mid-pass determinista + restart → DB íntegra, sin doble procesamiento corrupto, cola re-encolable desde L0 persistido; review P2-01 por agente distinto.
- **Resultado:** ✅ `vanta-memory/tests/scheduler_crash.rs` (test-only, 0 producción): (1) crash mid-pass con hook determinista (panic del runner, no timing — pre-mortem 1) sobre Fjall real + drop sin close → reopen: L0/L1 pre-crash record-idénticos (full `MemoryRecord`), cola efímera (nueva vacía), re-encolado desde L0 → crash-b completa y la re-entrega de crash-a NO duplica (judge dedup skip); segunda reconstrucción no-op; (2) reclaim in-process: claim huérfano tras panic se reclama al expirar el lease (MEM-66) sin pérdida. RED-equivalente por mutación (skip→store → FAIL left:2/right:1, revertido). Verify: focused 2/2 ×6 + suites `-p vanta-memory` 670/670 y `--features http-server` 678/678 + fmt/clippy×2/check. Review P2-01 `vanta-review` APPROVE (0C/0H; Medium de cierre → FIND-293; Low-1 full-record aplicado; Low-2 coordinación; NITs aplicados). Cierra T5 (cadena WIRE-14→18).
- **Commit:** 95cdb2ce (test, local sin push) + b65ae6d0 (docs task)

### MEMG-14: Marco 2.0 — núcleo + extensiones (reformulación del marco 8/6/10/PI)
- **Fecha:** 2026-10-05
- **Objetivo:** reformular el marco 8/6/10/PI como "núcleo + extensiones" tras la validación externa 2026-09-30 (7 sub-decisiones a-g del Backlog); respaldo del marco en repo + registro de validaciones + cadencia anual + draft de sync Notion (contrato plan Task 60 L1734, F5).
- **Resultado:** ✅ research-doc `docs/dev/research/memg-14-marco-2.0.md`: §1 respaldo núcleo (D1-D8/C1-C6/AM1-AM10/PI; anclas `SPEC.md:9` + `Backlog.md:776` + IDs Notion + marcas `[a verificar — Notion]`); §2 reformulación (a)-(g) mapeable 1:1 a `Backlog.md:140`; §3 registro de validaciones externas (dream/bitemporal/grafo/erasure/interop); §4 cadencia anual; §5 draft de sync (6 páginas, pendiente lane owner); §6 límites. Fuentes 11/11 fetch-verificadas (TSYS-13; arXiv 2604.16548 confirma 6 fases con Share y Forget/Rollback; W3C CG charter v1.0; DAMA-DMBOK). Gates docs 4/4 verdes (check-links 0 / check-docs clear / gen-index / coverage 0 gaps). Review P2-01 vanta-review tier Fast: ronda 1 changes-required (R-1 transitorio de índice por WIP ajeno MEMG-15 declarado; R-2 ancla corregida) → ronda 2 APPROVE. OCR delegation: N/A (diff docs-only; los archivos code reviewables del árbol son WIP ajeno). **Deuda declarada:** aplicación del sync Notion (lane owner) + R-1 pre-push (secuenciar MEMG-15).
- **Commit:** 21939b6f (docs research, local sin push) + commit de cierre docs(avance)

### MEMG-18: Multimodalidad — decisión + spec (extensión transversal vs 9ª dimensión) (master plan 0.9.0 F5, Task 62)
- **Fecha:** 2026-10-06
- **Objetivo:** documentar la decisión multimodal (extensión de modalidad transversal vs 9ª dimensión) + spec mínima (contenido multimodal en episódica/semántica + embeddings multimodales) + trigger de reevaluación refinado con evidencia 2026; sin implementación (contrato plan Task 62 L1790, F5).
- **Resultado:** ✅ research-doc `docs/dev/research/memg-18-multimodalidad.md`: §1 decisión (A extensión transversal elegida / B 9ª dim rechazada / C almacén dedicado ya descartado; alcance v1.0 intacto); §2 evidencia 2026 multi-fuente (2.1 surveys arXiv 2602.06052/2512.13564 + MIRIX 2507.07957; 2.2 competidores Mem0/Cognee/Letta — patrón transversal; 2.3 embeddings jina-clip-v2/Voyage); §3 spec mínima anclada en `MemoryInput`/`MemoryRecord`/`MemoryExportLine` + trait `Ingestor`; §4 trigger refinado (MGR-25 + caso + prerrequisito de embedder); §5 límites. Fuentes 9/9 fetch-verificadas (TSYS-13) + re-verificadas por el reviewer. Gates docs 4/4 verdes (check-links 0 / check-docs clear / gen-index / coverage 0 gaps). Review P2-01 vanta-review tier Fast: ronda 1 APPROVE (O-1/N-1/N-3) + delta re-review (profundización multi-fuente; O-2/N-2 aplicados) → APPROVE final. OCR delegation: N/A (diff docs-only; reviewables = WIP ajeno MEMG-15). Trigger refinado propagado a FUT-15 (`backlog-futuro.md:28` + `Backlog.md:520`) + pointer en `memg-14-marco-2.0.md` §2.2. **Deuda declarada:** ninguna nueva (el alcance v1.0 sigue cerrado; implementación espera al trigger).
- **Commit:** 789aeb23 (docs research, local sin push) + commit de cierre docs(avance)

### MEMG-15: Portabilidad/interoperabilidad — mapeo de frontera + export con integridad verificable (master plan 0.9.0 F5, Task 61)
- **Fecha:** 2026-10-06
- **Objetivo:** mapeo/adopción de formatos de frontera (AGENTS.md / MCP memory / drafts IETF-W3C) documentado + export con checksum/firma (mecanismo decidido en DISCOVERY) + versionado/migración de formatos; spec + slice mínimo verificable (roundtrip export→import con integridad) (contrato plan Task 61 L1762; stop L1764).
- **Resultado:** ✅ **Slice (SDK+CLI):** export JSONL escribe manifest sidecar determinista `<path>.manifest.json` (sha256 de los bytes exactos + records/namespaces + límites declarados estilo VER-02) + `ExportReport.{sha256,manifest_path}` (aditivos `#[serde(default)]`) + `Embedded::verify_export_integrity` (no claim-driven: `ok`/`mismatch`/`no_manifest`/`Err`; hashing streaming 64 KiB; guard de formato) + `vanta-cli export` con el mismo manifest vía helper compartido + Python expone las 2 claves del report. **Docs:** `MEMORY_INTERCHANGE_FORMAT.md` §Integrity manifest + §Versioning and migration + §Frontier formats (2026-10-05 survey: AGENTS.md/MCP/AAIF/AIMEM/ALF/AMP/W3C CG/SAIHM/Portable Agent Memory/memorywire + baseline de industria Major Labs 0/6) — profundización multi-fuente (owner rule 2026-10-06): 13 fuentes fechadas fetch-verificadas (AMP con URL corregida; SAIHM = home normativo del CG). **Firma criptográfica NO sancionada → FIND-309** (dos direcciones: COSE_Sign1/ML-DSA-65 y Merkle-DAG) + FIND-310 (exposición bindings/MCP/TS + refactor `cmd_export`) + FIND-311 (conversores AGENTS.md/AMP/ALF). Tests 7/7 nuevos (RED→GREEN; corrupto/ajeno/ausente → Err sin panic); lib 2363/2363 (`--test-threads 2`; 1 timeout por carga en test HNSW ajeno, re-run aislado ✅), cli 95/95, export suites 68/68, memory_export_import 10/10; clippy all-targets all-features ✅; gates docs 4/4. Review P2-01 vanta-review (tier adversarial): R1 changes-required (staging WIP ajeno + O1/O2 Low) → fixes → R2 **APPROVE**. OCR delegation: 13 archivos, 2 mejoras aplicadas (verify streaming, buffer reutilizable). **Deuda declarada:** FIND-309/310/311.
- **Commit:** 55b9a532 (feat sdk, local sin push) + commit de cierre docs(avance)

### MEMG-19: Prospectiva + descartes documentados (sensorial/emocional) (master plan 0.9.0 F5, Task 63)
- **Fecha:** 2026-10-06
- **Objetivo:** documentar (a) la prospectiva/intencional como patrón de uso (working+temporal+procedimental; **no almacén**) con benchmarks de referencia (PM-Bench/TriggerBench) y (b) los descartes sensorial-como-almacén y emocional/motivacional con respaldos fetch-verificados + criterios de revisión (emocional: re-evaluar en 6-12m) — contrato plan Task 63 L1818, F5; grep del repo > 0 para ambos.
- **Resultado:** ✅ research-doc `docs/dev/research/memg-19-prospectiva-descartes.md`: §1 prospectiva como patrón de uso (definición PM-Bench + decisión no-almacén con fundamento 4 puntos + 4 capas ancladas en código — working `mmd.rs`/`task_checkpoint.rs`, temporal `record.rs` SCH-02/03, procedimental `MemoryType::{Instruction,WorkTask,WorkMethod}`, disparo `auto_recall`/scheduler — + 3 benchmarks con cifras: PM-Bench 65.1% F1 máx, TriggerBench PM≠RM, Proactive Memory Agent +8.3pp/+6.8pp); §2 descartes (sensorial: Mem0 raw input buffer/survey 2602.06052/MIRIX/MEMG-18 §2.1; emocional: MemEmo "ningún sistema robusto"/PsychoAgent sin significancia/survey 2511.20657; registro auditable §2.3 con criterios — ventana emocional 2027-03-30→2027-09-30); §3 límites; §4 fuentes. Fuentes 9/9 fetch-verificadas (TSYS-13; 8 arXiv + Mem0) + re-verificadas en vivo por el reviewer. Gates docs 4/4 verdes (check-links 0 broken / check-docs GATING clear / gen-index --check / coverage 0 gaps); grep contrato: sensorial=10 · emocional=10 (>0). Review P2-01 vanta-review tier Fast: **APPROVE** (0 Required; O-1/N-1/N-2 aplicados; N-3 informativo; N-4 cumplido al cierre). OCR delegation: N/A (diff docs-only; reviewables del árbol = WIP ajeno). Pointers: memg-18 §1.1(C)/§3.8 + memg-14 §2.2/§2.7 (descartes + estado). **Deuda declarada:** ninguna nueva (descartes documentados; prospectiva no normativa, sin wiring especulativo).
- **Commit:** (docs research, local sin push) + commit de cierre docs(avance)

### STRAT-06: Research licencias (Khoj/Jan/Reor/OpenWebUI/Letta) + ACV OSS DBs (master plan 0.9.0 F0, Task 66)
- **Fecha:** 2026-10-06
- **Objetivo:** research doc con licencias comparadas del grupo local-first AI (Khoj/Jan/Reor/OpenWebUI/Letta) + OSS DBs relevantes y ACV/bands de OSS DBs con fuente por número (o marcado "modelado") + implicaciones para el modelo open-core/OEM; fuentes citadas (contrato plan Task 66 L1895); sin implementación.
- **Resultado:** ✅ research-doc `docs/dev/research/strat-06-licencias-acv-oss.md`: §1 licencias fetch-verificadas 2026-10-06 (Khoj AGPL-3.0 + **Khoj Cloud sunset 2026-04-15** — aviso oficial con razones de secuencia cloud-first; Jan Apache-2.0, 6.8M+ downloads; Reor AGPL-3.0 **archivado** 2025; OpenWebUI "Open WebUI License" — BSD-3 + branding clause v0.6.6, no-OSI, enterprise license, CLA; Letta Apache-2.0, $20 Pro; OSS DBs: ClickHouse/Qdrant Apache-2.0, Turso MIT, **Weaviate split BSD-3 + `wl/` enterprise (cambio detectado vs precedente)**, Neo4j GPL-3.0, MongoDB SSPL, Elastic/Redis AGPL); §2 ACV con fuente por número + columna `tipo` (dato/reportado/modelado): ClickHouse >$350M ARR (reportado, ago-2026), Supabase $70M→~$170M (est. Sacra), MongoDB 2,999 ≥$100k / 402 ≥$1M (SEC 10-Q, jul-2026), Elastic 1,800+ ≥$100k ACV (earnings Q1 FY27), Neo4j >$200M (oficial nov-2024), Redis >$300M / 12k pagos / 50+ >$1M (oficial ene-2026) + bandas `modelado (derivado)` con derivación visible (self-serve → $1M+); §3 implicaciones open-core/OEM sin re-litigar la decisión Apache-2.0+CLA: 3 vías de protección sin cerrar el motor (marca/enterprise-license OpenWebUI, directorio EE Weaviate/PostHog, dual license Neo4j $200M = ancla OEM), el ACV grande vive en enterprise ≥$100k, advertencia Khoj para el diseño de fase 2. Fuentes 32 fetch-verificadas (TSYS-13) + 11/11 spot-checks del reviewer. Gates docs 4/4 verdes (check-links 0 / check-docs GATING clear / gen-index --check / coverage 0 gaps). Review P2-01 vanta-review tier Fast: ronda 1 changes-required (F1 higiene de commit — fila STRAT-04 en `docs/index.md` generado → índice excluido del pathspec; F2 Jan 6.8M+; F3 claim "0 hits" corregido; F4 tipo Redis; F5 permalinks — sub-claim Neo4j rechazado con evidencia) → fixes → delta r2 **APPROVE**. OCR delegation: N/A (diff docs-only; reviewables del árbol = WIP ajeno). **Deuda declarada:** `docs/index.md` regenerado sin commitear (contiene fila STRAT-04 — regeneración conjunta al cierre de STRAT-04); datos de revenue/pricing con ventana de validez 6-18m (re-verificar antes de decisiones de pricing/OEM).
- **Commit:** (docs research, local sin push) + commit de cierre docs(avance)

### SRV-10: Cifrado en reposo del server (HTTP) — re-scope (b): superficie honesta (master plan 0.9.0 F5, Task 70)
- **Fecha:** 2026-10-06
- **Objetivo:** Re-scope obligatorio en DISCOVERY (FIND-249 diferido por decisión owner): contrato reducido a superficie de config/key mgmt + docs honestas (sin claim de protección) + test. Motivo real: el aviso ENC-01 es feature-gated y el build default del server no compila `encryption` → `VANTADB_ENCRYPTION_KEY` era silenciosamente ignorada (falsa seguridad, pre-mortem #3).
- **Resultado:** ✅ Contrato reducido completo — `vantadb-server` gana feature opt-in `encryption = ["vantadb/encryption"]` (RED→GREEN: "does not contain this feature" → check verde; con ella el aviso ENC-01 cubre la ruta server vía `open_with_config`, `bootstrap.rs:314`) · aviso nuevo `encryption_key_ignored_notice` + call en `run_with_hooks` (cubre `vanta-cli server --http` + `vantadb-server`; excluye InMemory; NO loggea la key; RED→GREEN 3/3) · docs honestas: `HTTP_API.md` (sin "SRV-10 on the roadmap"; ahora aviso + FIND-249), `CONFIGURATION.md:74,499`, `hardening.md:280` · verify en worktree aislado (MCP server lockea bins — FIND-177; sin matar procesos): suite lib+server **2509/2509** · clippy scoped + workspace all-features · fmt · gates docs (check-links/check-docs/gen-index/coverage) · OCR 0 Critical/High/Medium · review P2-01 adversarial vanta-review **APPROVE** (H2/H3/H5/H6 aplicados post-verdict; H1 gen-index stale por IQL.md de WIRE-13 → re-regen al commit). Frontera respetada: sin tocar write paths de storage/WAL/HNSW/crypto.rs/vfile (cifrado real = FIND-249). Commit creado en worktree de verificación (hook completo verde; 9/9 SHA256-idéntico) + `reset --soft` a develop; `--no-verify` NO usado.
- **Commit:** `ca3f00da` (local, sin push)
### FIND-445: 21 hallazgos P2/P3 del árbol de docs + 5 P1 adyacentes
- **Fecha:** 2026-10-20
- **Resultado:** ✅ Corregidos contra el código real: sintaxis IQL de vector search que no
  existía (`VECTOR_SEARCH(...)` → `<field> ~ "texto", min = 0.88`), endpoint de embeddings de
  Ollama, conteos de tools y version del registro MCP, el claim "every error response carries
  `error.data.code`" (las 5 factorías JSON-RPC lo omiten), 9 responses de `openapi.yaml` con
  envoltorio `{success,error,data}` inexistente, `/dashboard` documentado como público de
  loopback, `x-vanta-user-key` como security scheme, la ruta viva `POST /api/v2/mcp/proxy`,
  el flag inexistente `--allow-dummy-embeddings`, `VANTA_EMBEDDING_PROVIDER` → `VANTADB_*`,
  7 citas `archivo:línea` obsoletas en `docs/api/scores.md` y 1 afirmación de comportamiento
  falsa, `sync_mode` con su env var, `heavy-certification.yml` → `certification.yml` y el
  Recall@10 atribuido a 50K. ADR-0011 → `superseded` por ADR-0058 y ADR-0012 → `superseded`
  por ADR-0059 (los dos quedan `proposed`: los ratifica el owner).
  Gates: check-docs (GATING all clear), validate-docs-coverage (0 gaps), check_openapi_parity
  (37 paths = 37), check-links (0 rotos), check-doc-examples (OK), gen-index --check (al día).
  43 citas `archivo:línea` verificadas contra el símbolo correcto (0 mismatch).
  PR: `docs/docs-tree-and-skills` (#270).
### D-02: el tipo público de configuración es `Config`, no `VantaConfig`
- **Fecha:** 2026-10-20
- **Resultado:** ✅ `src/lib.rs:177` exporta `config::{Config, ...}`; el tipo `VantaConfig` no
  existe en `src/` (0 ocurrencias) y la referencia normativa de configuración lo usaba en 21
  sitios, incluidos snippets Rust que no compilaban. `docs/user/operations/CONFIGURATION.md`
  usa ahora `Config` en todo el documento. Hallazgo de la auditoría profunda v0.9.0 (P1).
### D-10: `max_blocking_threads` = 2 × `available_parallelism()` (16 es el fallback)
- **Fecha:** 2026-10-20
- **Resultado:** ✅ El default documentado era `16`; el real es
  `available_parallelism().map(|n| n.get() * 2).unwrap_or(16)` (`src/config.rs:1073-1075`).
  Corregido en `docs/user/operations/CONFIGURATION.md`. Hallazgo de la auditoría profunda
  v0.9.0 (P1).
### D-11: `wal_buffer_size` = `None` (no 65536)
- **Fecha:** 2026-10-20
- **Resultado:** ✅ El default real es `None` (`src/config.rs:347`); la fila del doc se
  contradecía a sí misma (la descripción ya decía "`None` = OS default") y el ejemplo JSON
  traía `65536`. Ambos corregidos en `docs/user/operations/CONFIGURATION.md`. Hallazgo de la
  auditoría profunda v0.9.0 (P1).
### D-15: 6 env vars leídas por el código y ausentes de las tablas de configuración
- **Fecha:** 2026-10-20
- **Resultado:** ✅ `VANTADB_QUIET` y `VANTADB_SILENT` (`src/hardware/mod.rs`),
  `VANTADB_PROFILE_PATH` (`src/hardware/mod.rs`, con su cadena de resolución),
  `VANTADB_SYNC_MODE` (`src/cli_handlers/config.rs`, default `periodic`, leída por la capa
  `vanta config` y no por `Config::from_env()`), `VANTADB_SCHEDULER_INTERVAL_MS` (default 60 s,
  `0` desactiva, `vantadb-server/src/scheduler.rs`) y `VANTADB_INGEST_CONFIG` (default
  `<storage_path>/data/vanta-ingest.toml`, `vantadb-server/src/scheduler.rs`) documentadas en
  la tabla de runtime de `docs/user/operations/CONFIGURATION.md`. Hallazgo de la auditoría
  profunda v0.9.0 (P1).

### Paso 22 — Entrega documental y herramientas de migración

- **Fecha del commit:** 2026-11-06.
- **Alcance implementado:** once artículos públicos en inglés; CLI y helper ChromaDB, tutorial oficial, adaptador Mem0 de alcance declarado, plantilla de issues, política de contribuciones asistidas por IA y anexo forense interno.
- **Evidencia:** suite focalizada de 36 tests aprobados con una advertencia de deprecación preexistente; el caso de error de CLI se revalidó después del último cambio de mensaje. Ruff sobre cinco archivos Python y Markdownlint sobre catorce documentos sin incidencias. Enlaces internos resueltos y manifiesto de superficies reconciliado.
- **Commits:** `094f53bf8f214b526aa942242d30f3b0295f5870` (entregables) y `f6ed51d058467226b073af71eea045fca4fa875e` (formato del README Python, con AST y comentarios de los ejemplos conservados).
- **Publicación remota:** `git push origin develop` aceptado, exit 0. HEAD local, `origin/develop` y `git ls-remote --heads origin develop` coinciden en `f6ed51d058467226b073af71eea045fca4fa875e`.
- **Barrera local:** pre-push `ALL 10 PASS`; Ruff del paquete Python y Markdownlint del alcance completo del hook aprobados. El primer intento fue bloqueado por formato preexistente del README, corregido en el segundo commit sin alterar reglas.
- **Checks remotos:** GitHub aceptó el push indicando bypass de diez checks requeridos todavía esperados. Esto no certifica CI remota en verde; no se modificaron las reglas de protección.
- **Seguridad remota:** GitHub anunció 31 alertas en la rama predeterminada (8 altas, 19 moderadas y 4 bajas); no se auditaron en este paso.
- **Límites:** no se certifican compatibilidad completa con todas las versiones de Mem0, extracción LLM, latencias menores de 1 ms, migraciones menores de 5 s ni inmunidad ante crash. Los criterios ampliados del Backlog se conservan; esta entrada registra los artefactos entregados, no cierra esos criterios sin evidencia.

### Paso 23 — Demos ejecutables, evidencia visual y benchmark agéntico

- **Fecha:** 2026-11-06.
- **Objetivo:** entregar el alcance del prompt del Paso 23 con bases temporales, datos ficticios y resultados reproducibles.
- **Resultado técnico:** implementados y publicados los ejemplos Python de supersession y borrado certificado, la ejecución hero verificada y su reproducción congelada, los ejemplos Node/Bun/OPFS y la comparación serial de tres motores. Revisión independiente aprobada y Gate H sano para la sincronización de la skill MCP.
- **Visual:** `assets/demo.mp4` de 60 segundos, H.264, 1280×800, 942964 bytes y sin audio; `assets/demo.gif` de 3838932 bytes; póster estático y transcripción completa. Revisión visual de las cinco etapas exportadas. Playwright registra únicamente una página de un contexto nuevo; no se ejecutó la captura nativa de Recordly después de comprobar el origen monitor+recorte en su fuente oficial. VHS conserva una reproducción de datos congelados, no renderizada en este host por falta de ttyd.
- **Validación focalizada:** 34 tests Python aprobados, incluidos 20 de las demos, 4 de la hero y 10 contratos del arnés; 7 tests Node aprobados y repetidos después del formato. Bun y persistencia OPFS después de recarga comprobados. Ruff limpio. `verify_changed.ps1`: `ALL 4 PASS`; `verify_staged.ps1`: `ALL 1 PASS`. Markdownlint ampliado: 234 archivos, cero incidencias; cobertura documental sin gaps y diez pares de la skill sincronizados.
- **Mediciones:** 2000 operaciones por motor y 6000 muestras; resultados y límites en `docs/user/operations/BENCHMARKS.md`, sección 24, y `benchmarks/agent_memory_results.json`. No se alcanzaron las metas de menos de 2 ms ni 25 MB para el worker Python. La prueba separada de cuatro hilos no demuestra ausencia de contención ni soporte multiproceso.
- **Verificación de superficies:** backend 0.9.0; perfiles medidos `brain=15`, `full=84`, `agent=39`, `dev=38`, `memory=22`. Escena L2 escrita mediante MCP y recuperada tras reabrir en otro proceso; es una proyección de la aplicación, no consolidación automática con LLM. El certificado muestra integridad SHA-256 por registro, no firma ni prueba independiente de borrado físico de WAL, backups, exportaciones o pesos de modelos.
- **Contrato MCP:** smokes autoritativos del perfil predeterminado y de `full` con 5/5 cada uno, cuatro recursos y cuatro prompts. La deuda del manual embebido que aún anuncia doce herramientas queda registrada en Backlog; no se modifica código Rust ni se afirma saneamiento global del catálogo.
- **Criterios ampliados:** no se eliminan filas del Backlog ni se archiva el plan maestro. Permanecen sin certificar sus requisitos adicionales de instalación/failover multiagente, supersession automática/AS OF, comprobación física contra WAL, ejecución Node NAPI-RS, integración CI de los seis proveedores y benchmark de 10000–50000 turnos.
- **Evidencia local:** `C:\Users\Eros\paso23-evidence\delivery-gates\`, respuestas MCP en `hero-verified-l2-run\`, perfiles adicionales en `mcp-other-profiles-verified.json` y cinco PNG exportados. Los datos del benchmark y de la hero no se regeneraron durante la producción visual.
- **Integridad de artefactos:** el primer intento de commit se detuvo porque Prettier reformateó JSON y JavaScript. Se aceptó el formato sin omitir hooks; se verificó igualdad completa de los valores JSON frente a la captura original y se conservó esta en `paso23-evidence/benchmark-original-before-prettier.json`. El checksum publicado se recalculó sobre el blob staged y se comprobó después del commit: `abcf700ff6bcc893d1b0a8a472aa43fdfe945a35d55d149c7d73b897003a04dd`. No se repitieron las mediciones.
- **Publicación:** commit `632f49187ef8fce8e21abea78fba7ecbb8c5c463`, 46 archivos; push a `origin/develop` aceptado con exit 0. HEAD local, referencia de seguimiento y consulta directa del remoto coinciden; árbol público limpio.
- **Barrera pre-push:** `ALL 10 PASS`, más 77 tests del binding Node nativo y 361 del SDK TypeScript aprobados; Ruff del paquete Python y Markdownlint de 219 documentos aprobados. El guard `canonical_p99` solo compila con `--no-run`; la cobertura Rust se difiere al nightly por configuración existente, sin nueva medición de cobertura en este cierre.
- **Estado remoto:** GitHub informó bypass de los diez checks requeridos todavía esperados y 31 alertas en la rama predeterminada (8 altas, 19 moderadas, 4 bajas), no auditadas en este paso. No se certifica CI remota en verde ni preparación para release; no se modificaron protecciones.

### FIND-291: Drenar `shutdown_background_services` antes del `flush()` final en `serve_http_or_tls`
- **Fecha:** 2026-11-18
- **Objetivo:** Invertir el orden de apagado en `src/server/bootstrap.rs` para que los servicios de fondo (scheduler L1, TTL sweeper) se detengan y drenen sus últimas escrituras antes de ejecutar el `flush()` final del `StorageEngine`.
- **Resultado:** ✅ `drain_services_then_flush` (`src/server/bootstrap.rs:489-564`) ejecuta `shutdown_background_services` → `flush_on_shutdown_async` en HTTP, TLS y rutas de error; cada servicio se aísla en `tokio::spawn` con `SERVICE_SHUTDOWN_TIMEOUT = 30s` (de modo que un panic o timeout se loguea sin abortar el flush final) y `shutdown_telemetry()` se invoca una única vez; review P2-01 `vanta-review 2047a71b` APPROVE y auditoría `#20` PASS.
- **Commit:** `eca33817b`, `a51dfcc1f`


## Migración de backlog — pase 2 campaña `consolidation-2026-11-07` (2026-11-19)

### ACT-04: Reconciliación Enterprise vs. Freeze-List y Limpieza de Features
- **Fecha:** 2026-11-07
- **Objetivo:** Reconciliación Enterprise vs. freeze-list y limpieza de features experimentales en docs.
- **Resultado:** ✅
- **Commit:** `6eb3a00e6`

### TEC-01: Purga Exhaustiva de `InMemoryEngine` en Documentación
- **Fecha:** 2026-11-06
- **Objetivo:** Purga exhaustiva de referencias a `InMemoryEngine` en la documentación.
- **Resultado:** ✅
- **Commit:** `08c6dc603`, `255db8208`

### TEC-09: Sistema de Telemetría Pasiva (Cron) y Activa (Opt-In) (`D6`)
- **Fecha:** 2026-11-07
- **Objetivo:** Sistema de telemetría pasiva (cron) y activa (opt-in) con colector de adopción (D6).
- **Resultado:** ✅
- **Commit:** `6eb3a00e6`

### DOC-01: Paridad Total y Rigurosa de `README_ES.md`
- **Fecha:** 2026-11-06
- **Objetivo:** Paridad total y rigurosa de `README_ES.md` con `README.md`.
- **Resultado:** ✅
- **Commit:** `0512116e6`, `632f49187`

### DOC-02: Purga de IDs Internos Residuales en `docs/user/`
- **Fecha:** 2026-11-06
- **Objetivo:** Purga de IDs internos residuales en `docs/user/` (higiene de documentación pública).
- **Resultado:** ✅
- **Commit:** `08c6dc603`

### DOC-03: Plantilla `GOOD FIRST ISSUE` y Política de PRs de IA
- **Fecha:** 2026-11-06
- **Objetivo:** Plantilla `GOOD FIRST ISSUE` y política de PRs de IA en las guías de contribución.
- **Resultado:** ✅
- **Commit:** `094f53bf8`

### DOC-04: Regla en `pre-commit` contra `.DS_Store` y Archivos Binarios
- **Fecha:** 2026-11-06
- **Objetivo:** Regla `pre-commit` contra `.DS_Store` y archivos binarios (anti-junk).
- **Resultado:** ✅
- **Commit:** `08c6dc603`

### DOC-05: Tabla Quickstart 60s por IDE en Cabecera de `README.md`
- **Fecha:** 2026-11-06
- **Objetivo:** Tabla quickstart de 60s por IDE en la cabecera de `README.md`.
- **Resultado:** ✅
- **Commit:** `0512116e6`, `c80a0ae23`

### DOC-06: Auditoría, Limpieza y Alineación a v0.9.0+ de los 9 Artículos Rescatados (`docs/user/blog/`)
- **Fecha:** 2026-11-06
- **Objetivo:** Auditoría, limpieza y alineación a v0.9.0+ de los 9 artículos rescatados del blog.
- **Resultado:** ✅
- **Commit:** `87d2006c0`

### DOC-07: Matriz `grep` vs VantaDB (`Q23`), Nota Toolchain `roaring` (`Q6`) y Promoción `search_batch` (`Q12`)
- **Fecha:** 2026-11-06
- **Objetivo:** Matriz `grep` vs VantaDB (Q23), nota toolchain `roaring` (Q6) y promoción `search_by_metadata`.
- **Resultado:** ✅
- **Commit:** `094f53bf8`

### DOC-08: Script de Migración desde ChromaDB y Tutorial Oficial (`Q22 Opción B`)
- **Fecha:** 2026-11-06
- **Objetivo:** Script de migración desde ChromaDB y tutorial oficial (Q22 Opción B).
- **Resultado:** ✅
- **Commit:** `094f53bf8`

### PRD-04: Soak Testing 1,000 Turnos contra Fuga de Contexto (`Q26 Opción B`)
- **Fecha:** 2026-11-07
- **Objetivo:** Arnés de soak testing de 1.000 turnos contra fuga de contexto (Q26 Opción B).
- **Resultado:** ✅
- **Commit:** `88022de96`

### PRD-07: Mantenibilidad Estable de `vantadb-server` como Puente HTTP (`Q7 Opción B`)
- **Fecha:** 2026-11-07
- **Objetivo:** Mantenibilidad estable de `vantadb-server` como puente HTTP (Q7 Opción B).
- **Resultado:** ✅
- **Commit:** `0a77dfb71`

### DEM-01: Hero Demo Grabada en Terminal (VHS Tape de 60s)
- **Fecha:** 2026-11-06
- **Objetivo:** Hero demo grabada en terminal (VHS tape de 60s) restaurada y publicada.
- **Resultado:** ✅
- **Commit:** `87d2006c0`, `632f49187`

### DEM-02: Demo Interactiva de Deduplicación y Supersession en Vivo
- **Fecha:** 2026-11-06
- **Objetivo:** Demo interactiva de deduplicación y supersession en vivo.
- **Resultado:** ✅
- **Commit:** `632f49187`

### DEM-03: Demo de Borrado Certificado GDPR Ejecutivo
- **Fecha:** 2026-11-06
- **Objetivo:** Demo de borrado certificado GDPR ejecutivo.
- **Resultado:** ✅
- **Commit:** `632f49187`

### DEM-04: Suite Canónica de Ejemplos en TypeScript (`examples/typescript/`) y Fix `FIND-538`
- **Fecha:** 2026-11-06
- **Objetivo:** Suite canónica de ejemplos en TypeScript (`examples/typescript/`) y fix FIND-538.
- **Resultado:** ✅
- **Commit:** `632f49187`

### DEM-05: Benchmark de Memoria Agéntica frente a `sqlite-vec` y `LanceDB` (`Q21 Opción B`)
- **Fecha:** 2026-11-06
- **Objetivo:** Benchmark de memoria agéntica frente a `sqlite-vec` y `LanceDB` (Q21 Opción B).
- **Resultado:** ✅
- **Commit:** `632f49187`

### BNCH-01: Arnés de Estrés Sintético `Database Sequential-NIAH` (`evals/bench_sequential_niah.py`) (Fase 5)
- **Fecha:** 2026-11-07
- **Objetivo:** Arnés de estrés sintético `Database Sequential-NIAH` (`evals/bench_sequential_niah.py`).
- **Resultado:** ✅
- **Commit:** `c56115825`

### EVAL-02: Blindaje Metodológico de LongMemEval con Partición Holdout Inviolable
- **Fecha:** 2026-11-07
- **Objetivo:** Blindaje metodológico de LongMemEval con partición holdout inviolable (guard SHA-256).
- **Resultado:** ✅
- **Commit:** `211d9d6bd`

### GTM-01: Renderizado del GIF/MP4 desde `vhs/demo.tape` e Incrustación en README
- **Fecha:** 2026-11-06
- **Objetivo:** Renderizado del GIF/MP4 desde `vhs/demo.tape` e incrustación en README.
- **Resultado:** ✅
- **Commit:** `632f49187`, `0512116e6`

### GTM-02: Tabla Comparativa "David vs. Goliat" en `README.md` y `README_ES.md`
- **Fecha:** 2026-11-06
- **Objetivo:** Tabla comparativa "David vs. Goliat" en `README.md` y `README_ES.md`.
- **Resultado:** ✅
- **Commit:** `0512116e6`

### GTM-03: Comando `vantadb mcp init` Auto-Configurador de IDEs en 30 Segundos
- **Fecha:** 2026-11-07
- **Objetivo:** Comando `vanta-cli mcp init` autoconfigurador de IDEs en 30 segundos.
- **Resultado:** ✅
- **Commit:** `e61bc3f4a`

### GTM-04: Preparación de Manifiestos para los 5 Registros MCP Oficiales
- **Fecha:** 2026-11-07
- **Objetivo:** Manifiestos para los registros MCP oficiales (`glama.json`, `smithery.yaml`).
- **Resultado:** ✅
- **Commit:** `c56115825`

### GTM-10: Envío Escalonado de PRs a las 4 *Awesome Lists* (`Q28 Opción B`)
- **Fecha:** 2026-11-07
- **Objetivo:** Kit operativo de PRs a las 4 Awesome Lists formalizado en `AWESOME_LISTS_AND_MCP_REGISTRIES_PR_KIT.md` (repo-ready; envío externo del owner tras FASE-A).
- **Resultado:** ✅
- **Commit:** `c56115825`

### DX-10: Pre-validación Automatizada — 5 Instalaciones Limpias End-to-End
- **Fecha:** 2026-11-06
- **Objetivo:** Arnés de pre-validación automatizada: 5 instalaciones limpias end-to-end por canal.
- **Resultado:** ✅
- **Commit:** `7d1dd86c2`


### FIND-197: Runner LoCoMo opt-in (licencia CC BY-NC 4.0)
- **Fecha:** 2026-11-22
- **Objetivo:** Proveer runner opt-in del benchmark LoCoMo con fetch del dataset por el usuario (dataset no vendado en el repo por licencia CC BY-NC 4.0).
- **Resultado:** ✅ — verificado en HEAD 2026-11-22: `evals/eval_locomo.py` implementa `--dataset-path` (fetch del usuario) + fallback sintético determinista `--self-test`/`--smoke` (seed=42), métricas recall@k/MRR/NDCG/F1 + comparativa Mem0.
- **Commit:** `09116b060` (MEM0-05)

### DOC-24: Saneamiento de `Backlog.md`, reconciliación estratégica/benchmarks y corrección `setup-embeddings.ps1`
- **Fecha:** 2026-11-22
- **Objetivo:** Reconciliar `docs/user/operations/BENCHMARKS.md` (§19.3/§23 alineados con §23.3 ACT-02) y `docs/dev/strategy/{GO_TO_MARKET,ROADMAP}.md` locales con `v0.10.0`; corregir `Get-RepoRoot` en `scripts/setup-embeddings.ps1` (8/8 `test_supply_chain.py` y 0 errores AST); migrar cierres a `avance/`, archivar exclusiones en `activo/wontfix-archive.md` y reparar filas malformadas y SHAs en el plan archivado `2026-11-07`.
- **Resultado:** ✅ Verificado en `HEAD` mediante gates de documentación (`check-avance-coverage.ps1`, `validate-docs-coverage.ps1`, `check-docs.mjs`).
- **Commit:** `5a45a1f8c` (`setup-embeddings.ps1`), `35330f283`, `8b4ff7fbc`, `4487d5097` (`BENCHMARKS.md`)

### PRD-03, FIND-627, FIND-628 y cierres ops/embeddings campaña `2026-11-07` (`FIND-224`, `384`, `402`, `531`, `641`, `645`)
- **Fecha:** 2026-11-22
- **Objetivo:** (`PRD-03`) Certificar cumplimiento MCP spec `2025-06-18` (`CancellationRegistry` + `notifications/cancelled` + handshake stdio en `vantadb-mcp/src/server.rs`, tests `43..49` verdes); (`FIND-628`) actualizar referencias rotas de `src/storage.rs` a `src/storage/engine/init.rs:262-305` / `maintenance.rs` en `DURABILITY_GUARANTEES.md` y documentar colisión `.vanta.lock` en `QUICKSTART.md` y `MCP.md`; (`FIND-627`) documentar receta `@cloudflare/codemode` con `VANTADB_MCP_PROFILE=full` y límite de Anthropic PTC en `MCP.md`; migrar además `FIND-224`, `384`, `402`, `531`, `641` y `645` ya verificados en `HEAD` (`CONFIGURATION.md:479`, `embeddings/manifest.json` 1034 MB, `50c358b01`, `0e6c76d96`, `884363b5a`, `5d6ac8eb4`).
- **Resultado:** ✅ Gates de documentación y tests `vantadb-mcp --lib` (49/49) en verde.
- **Commit:** `884363b5a`, `0e6c76d96`, `50c358b01`, `5d6ac8eb4`, `bb2af63a9`, `b432f0219`

### `ACT-02` y `FIND-655`: Certificación LongMemEval-S 500q (`hybrid_rrf = 0.8872` / `0.9787 PASS`) + apertura diferida `FIND-656`
- **Fecha:** 2026-11-22
- **Objetivo:** Completar el estudio de ablación `--ablation all` de 500 preguntas en LongMemEval-S (`BAAI/bge-small-en-v1.5` 384d, `--strict-embeddings`, `157 926` vectores únicos en caché `.npz`, 0 tokens LLM), certificar el path nativo híbrido del motor frente al umbral `>= 0.85`, resolver el bloqueo `FIND-655` del harness Python y documentar resultados y trabajo futuro (`FIND-656`).
- **Resultado:** ✅ Certificado y publicado:
  - **`hybrid_rrf` (HNSW + BM25 + RRF nativo):** `recall_all@5 = 0.8872`, `recall_any@5 = 0.9787`, `F1 = 0.66` → **`GATE PASS` (`>= 0.85`)**.
  - **`vector_only` (HNSW + `query_embed`):** `recall_all@5 = 0.8830`, `recall_any@5 = 0.9660`, `F1 = 0.67` → **`PASS`**.
  - **`bm25_only` (línea base léxica):** `recall_all@5 = 0.8000`, `recall_any@5 = 0.9596`, `F1 = 0.66`.
  - **`hybrid_graphrag` (heurística experimental regex del harness Python):** `recall_all@5 = 0.8489`, `recall_any@5 = 0.9340`, `F1 = 0.52`; mejora `multi-session` (`0.7934 -> 0.8347`) y `knowledge-update` (`0.9722 -> 0.9861`), pero degrada `single-session-assistant` (`1.0000 -> 0.6071`, −22/56 preguntas). Relegado a ablación experimental documentada (Opción A + C futura → **`FIND-656`** diferida post-`v0.10.0`).
  - Documentación y evidencia actualizadas en `evals/eval_longmemeval.py`, `evals/README.md`, `evals/results/longmemeval_ablation_all_fixed.json`, `docs/user/operations/BENCHMARKS.md` (§19/§23), `README.md`, `README_ES.md` y `docs/user/COMPARISON.md` (§2.5).
  - Verificación de gates locales: `check-links`, `check-docs`, `gen-index --check`, `markdownlint-cli2`, `eval_longmemeval.py --self-test` y `cargo test -p vantadb --test version_coherence` (`2/2 PASS`).
- **Commit:** `4b8f72cd5`

### `FIND-662` y `FIND-663`: Saneamiento documental integral de benchmarks y registro de los 59 ejecutables en `BENCHMARKS.md`
- **Fecha:** 2026-11-22
- **Objetivo:** Resolver las 7 contradicciones factuales y enlaces rotos detectados en la auditoría de los 59 benchmarks (`FIND-662`) y registrar en `docs/user/operations/BENCHMARKS.md` todos los benches Criterion, suites de certificación, WASM multi-worker, HTTP server, GraphRAG y soak tests que estaban huérfanos fuera de la documentación canónica (`FIND-663`).
- **Resultado:** ✅ Completado y verificado con gates documentales en verde:
  - Eliminados enlaces rotos a `docs/user/benchmarks/COMPETITIVE_SDK_BENCH.md` en `benchmarks/README.md` y `BENCHMARKS.md` §18.
  - Sincronizados `docs/user/COMPARISON.md` (§2.2 y §2.4), `skills/vantadb/SKILL.md`, `README.md` y `README_ES.md` (`Bloque 1 = 50K`, slice `100K` de SIFT-1M), `benchmarks/WASM_BENCHMARK_REPORT.md` (bundle `1.77 MB` / `728 KB` gzip + banner de fuente única), `docs/user/blog/graphrag-benchmark.md` (nota del fix `2026-10-29` a 3 000 nodos) y `evals/runners/README.md` (estado denso actual de `VantaAdapter`).
  - Añadidas secciones **§27** (`soak_test.py` 1 000 turnos + alerta honesta de deriva lineal), **§28** (`wasm_bench.mjs` + `wasm_threads_bench.mjs` 400K `SharedArrayBuffer`), **§29** (`graphrag_bench.rs` + `vantadb-server/tests/benchmarks.rs`) y **§30** (catálogo completo de benches Criterion en CI, suites `tests/certification/*` y `evals/agent/run_eval.py`), además de registrar `batch_txn_overhead` en **§26** y `docs/wiki/modules/evals.md`, y precisar los pseudo-embeddings deterministas 64d en **§25**.
  - Las 17 tareas restantes de ingeniería, CI, optimización y nuevos benchmarks estándar (`FIND-664..680`) quedan activas en `docs/dev/Backlog.md`.
- **Commit:** `01f49309c`

### FIND-686: Higiene pre-release — `html_root_url` obsoleto (`0.8.0`), doc falsa en `src/api/mod.rs` y cifras de competidores sin fuente en `README*` (Regla 11)
- **Fecha:** 2026-11-23
- **Objetivo:** Limpiar tres desajustes visibles ante auditoría pública pre-Show HN: (1) `src/lib.rs:1` declaraba `html_root_url = "https://docs.rs/vantadb/0.8.0"` (innecesario en rustdoc moderno y desactualizado); (2) `src/api/mod.rs:1` afirmaba contener `"HTTP API handlers and route definitions"` cuando solo expone `pub mod scores`; (3) la tabla comparativa principal en `README.md:262-274` y `README_ES.md:264-276` incluía QPS específicos para Zep (`8.1`) y Letta (`0.9`) sin benchmark reproducible propio en el repo (violando Regla 11 y `COMPARISON.md` §3).
- **Resultado:** ✅ — los tres puntos corregidos sin alterar la API pública.
- **Commit:** `383c5aadc`

### FIND-688: Reconciliación integral del hub `VantaDB Docs` en Notion contra `HEAD` `v0.10.0` (Ola 3)
- **Fecha:** 2026-11-23
- **Objetivo:** Corregir en Notion `VantaDB Docs` (`3d4d0445-9756-80f2-a5fc-e53ad196a8ba`) y 10 subpáginas los errores bloqueantes frente al producto real: (1) Quickstarts con API inexistente (`db.embed(...)` en Python/TS, `Client.open` en Python, `await Client.open` en TS como persistente — el WASM es no-durable y lo persistente en Node es `await connect` — y `db.forget(..., key=...)` afirmando certificado WORM sin `mode="certified"`); (2) `Observabilidad, benchmarks y métricas` publicando `0.874` (proyección retractada) y 4 rutas de archivos inexistentes (`evals/long_mem_eval.py`, `benches/fast_gate_bench.rs`, `tests/storage/purge_attest.rs`, `crates/vanta-memory/benches/embed.rs`); (3) 7 páginas con afirmaciones falsas ("un solo archivo `.vanta`" — es un directorio; "modelos ONNX integrados sin descarga" — requieren `setup-embeddings.ps1`; "`group_commit: true` por defecto" — es opt-in; `@vantadb/wasm` — real: `vantadb-wasm`; servidor `gRPC` inexistente; Studio/Team Sync en presente — son *Coming Soon*); (4) 3 enlaces a subpáginas borradas (`System Architecture`, `Storage Engine`, `Roadmap`) y omisión de `ADR-0061` (`T54`), `ADR-0057` (cifrado), los 7 adapters PyPI y `ACT-02` (`0.8872`).
- **Resultado:** ✅ Ola 3 — páginas Notion actualizadas; enlaces a las 3 subpáginas borradas retirados/marcados; faltantes incorporados (`ADR-0061`, `ADR-0057`, 7 adapters PyPI, `ACT-02` `0.8872`). Trabajo externo en Notion (sin commit de repo).
- **Filas removidas de backlog-notion.md:** `N-12`, `N-13`, `N-14`, `N-15`, `N-16`, `N-18`, `N-19`, `N-20`, `N-21`, `N-22`, `N-23`, `N-24` (absorbidas por esta fila, declarado en `docs/dev/Backlog.md`).

### FIND-694: `scripts/setup-embeddings.ps1` moría en pwsh 7 — `Get-DefaultDbPath` asignaba `$home = $HOME` sobre variable `AllScope` read-only
- **Fecha:** 2026-11-23
- **Objetivo:** Desbloquear el wizard de embeddings en PowerShell 7: `$HOME` es automática `ReadOnly, AllScope` y case-insensitive, por lo que `$home = $HOME` en `scripts/setup-embeddings.ps1:294-298` lanzaba `WriteError` fatal bajo `Set-StrictMode` + `ErrorActionPreference=Stop`, bloqueando el wizard completo (ruta por defecto de `-DbPath`).
- **Resultado:** ✅ — variable renombrada a `$homeDir`. Verificado: `pwsh scripts/setup-embeddings.ps1 -NonInteractive -Modules embeddings` exit 0 con modelo ya presente y ORT 1.30.
- **Commit:** `3d2c7d138`

### FIND-681: Fuga de embeddings sintéticos con `strict` y ONNX Runtime ausente o inválido (Scopegrain `VB-6`)
- **Fecha:** 2026-11-23
- **Objetivo:** Verificar en vivo, contra el binario reconstruido con `--features embed-local`, que `VANTADB_STRICT_EMBEDDINGS=true` falla cerrado cuando la DLL de ONNX Runtime falta o es inválida, y que con runtime válido las embeddings son reales y distintas entre textos (sin vectores sintéticos silenciosos).
- **Resultado:** ✅ — `node .scopegrain/research/2026-11-16-vantadb-embedding-probe.mjs <projectRoot>` → `passed-functional-embeddings`, `controlIssues: []`, `positivePassed: true`. Dos corridas: artefacto `target/release` (sha256 `fe0433dc…`) y binario instalado (sha256 `b2f3b48dbb5ad39f68e8c51e4d9e766a19f844f70a7dbd20b818680dd226a145`). Evidencia: `.tasksystem/verification/2026-11-16-native-embedding-83b942a0-4981-44cc-b874-757e9561761c.json`. El spec quedó re-pinneado al binario realmente probado (antes apuntaba al 0.9.0 pre-fix, `67b55078…`).
- **Commit:** `01a38f353` (fix) · verificación 2026-11-23

### FIND-695: `vantadb-server` no exponía `--version`/`-V`
- **Fecha:** 2026-11-23
- **Objetivo:** Dar al binario una forma estándar de autodeclarar su build, para soporte y para el pinning de artefactos del probe de Scopegrain.
- **Resultado:** ✅ — `-V`/`--version` imprime `vantadb-server <CARGO_PKG_VERSION>`; ambos flags entran en `is_known_flag`, el `--help` los documenta y `validate_args` los cubre. Verificado en los dos artefactos: `cargo install --path vantadb-server --features embed-local --force -j 2` (release, 22m21s) y después `vantadb-server --version` → `vantadb-server 0.9.0`.
- **Commit:** `10b10858a`

### FIND-697: `local_model_path` relativo al cwd (Scopegrain `VB-21`)
- **Fecha:** 2026-11-23
- **Objetivo:** Eliminar la dependencia del directorio de lanzamiento: el default `embeddings/models/multilingual-e5-small/onnx` se resolvía solo contra el cwd, así que un server lanzado desde `TaskSystem` reportaba `model_present=false` pese a tener el modelo instalado.
- **Resultado:** ✅ — `config::resolve_local_model_path` resuelve rutas relativas contra cwd → dir del ejecutable → `~/.vantadb`, devuelve la cadena original cuando ninguna existe, y es compartida por el provider (`llm.rs`), `capabilities` (`embedding_health.rs`) y el guard de vectores sintéticos (`executor.rs`). `setup-embeddings.ps1` enlaza `<Db>/embeddings` al `embeddings` del repo (junction, sin duplicar 823 MB) para clientes lanzados por el wizard. Verificado: `cargo test -p vantadb --features embed-local --lib f697 -- --test-threads=2` → 1 passed; `scopegrain vantadb status --probe` → `model_present: true`, `model_dir: C:\Users\Eros\.vantadb\embeddings/models/multilingual-e5-small/onnx`, `status: Ready`.
- **Commit:** `bed6473df`
- **Nota:** el `cargo test --lib llm` de la misma sesión destapó un fallo **prexistente** (`test_strict_embeddings_policy`), reproducido sobre `HEAD` con `git stash` → registrado como `FIND-698`.

### FIND-696: Host de desarrollo al límite de commit (pagefile 8 GB) — `os error 1455` y `rustc-LLVM ERROR: out of memory` en el link release
- **Fecha:** 2026-11-23
- **Objetivo:** Desbloquear el re-link de `vantadb-server` con `embed-local`, que murió por falta de commit virtual (~200 MB libres con pagefile de 8 GB).
- **Resultado:** ✅ — el entorno ya no es bloqueante: `Win32_OperatingSystem` → 11.4 GB físicos / 14 GB commit libres, pagefile **12.7 GB** asignados (uso actual 1.6 GB), sin procesos cargo/rustc. Cuatro builds en serie completaron sin OOM ni `os error 1455`: `cargo install --features embed-local` (release 22m21s), re-install tras liberar el lock del MCP vivo (7m31s con `-j 2`) y dos `cargo test --features embed-local`. El bloqueo residual es de **archivo bloqueado** (FIND-177), no de memoria.
- **Commit:** mitigación `-j 2` en `3d2c7d138` · verificación 2026-11-23

### Probe MCP de perfiles (Scopegrain `VB-3`/`VB-5`/`VB-1`/`VB-8`)
- **Fecha:** 2026-11-23
- **Objetivo:** Certificar en vivo, contra el binario instalado, la allowlist estricta de tools, el schema de `dedup_policy`, el recall `scope:"agent"` y la ausencia de efectos con `VANTADB_DEDUP_POLICY=off`.
- **Resultado:** ✅ 15/15 checks verdes (`.scopegrain/research/2026-11-23-vantadb-profile-probe.mjs`, DB aislada bajo `tests/.tmp`, binario `b2f3b48d…`):
  - `dedup_policy` publicado con enum `off, exact, semantic`; valor inválido → `-32602` en perfil `memory` (no solo `brain`).
  - `VANTADB_MCP_STRICT_PROFILE=true` + `VANTADB_MCP_ALLOW_TOOLS=memory_get,memory_recall` → `memory_put` rechazado con `-32601` y **sin efecto** (`memory_get` → `VANTADB_NOT_FOUND`).
  - `VANTADB_DEDUP_POLICY=off` no rompe escrituras.
  - VB-1 `memory_recall{scope:"agent"}` recupera el registro (`count: 1`, `mode: hybrid`) **cuando el payload de `l1/*` es el envelope `MemoryRecord` completo**; con payload de texto el pool queda vacío → contrato implícito registrado como `FIND-699`.
  - VB-22/VB-16 `sg-formal-v1-*` nunca entra al pool de L1.
- **Evidencia:** `.tasksystem/verification/2026-11-23-mcp-profile-17638652-f9dd-47ce-8355-becd3e0cc182.json` (binario `b2f3b48d…`) y `.tasksystem/verification/2026-11-23-mcp-profile-7cb95250-238e-4211-8c85-1ae538879928.json` (**17/17**, binario final `0dce814c…`).

### FIND-683: `memory_put` omitía `dedup_policy` del `inputSchema` e ignoraba `VANTADB_DEDUP_POLICY` fuera de `brain` (Scopegrain `VB-5`)
- **Fecha:** 2026-11-23
- **Objetivo:** Certificar en vivo que el parámetro está publicado con su enum, que un valor inválido se rechaza en cualquier perfil, y que la variable de entorno **se aplica** (no solo se tolera) fuera del perfil `brain`.
- **Resultado:** ✅ — 4 checks en `.scopegrain/research/2026-11-23-vantadb-profile-probe.mjs` con perfil `memory`: (1) `inputSchema.properties.dedup_policy.enum == ["off","exact","semantic"]`; (2) `dedup_policy:"nonsense"` → `-32602` (no hay bypass por perfil); (3) `VANTADB_DEDUP_POLICY=off` no rompe escrituras; (4) `VANTADB_DEDUP_POLICY=exact` deduplica de verdad: la segunda escritura con payload idéntico devuelve la clave existente (`dup-1`) y `dup-2` no existe (`memory_get` → `-32004 VANTADB_NOT_FOUND`).
- **Evidencia:** `.tasksystem/verification/2026-11-23-mcp-profile-7cb95250-238e-4211-8c85-1ae538879928.json` (17/17 checks, binario `0dce814c…`).
- **Commit:** `01a38f353` (fix) · verificación 2026-11-23

### FIND-698: `test_strict_embeddings_policy` fallaba en `HEAD`: los fallbacks hardcodeados cargaban un modelo real con un `local_model_path` inválido
- **Fecha:** 2026-11-23
- **Objetivo:** Restaurar el contrato de "strict falla cerrado" cuando el path configurado no existe, y quitar el leak de env que hacía fallar el mismo test por otra razón.
- **Resultado:** ✅ — `model_path_candidates` solo añade los tres modelos hardcodeados cuando `local_model_path` está **vacío** (descubrimiento de un config sin definir); con un path no vacío pero inválido no resuelve nada y strict devuelve `Error::Embedding`. Como el estimador (`resolve_model_path`) y el loader (`load_session_inner`) comparten esa función, la estimación de presupuesto y la sesión cargada siguen de acuerdo. El test queda `#[serial_test::serial]` y limpia `VANTADB_MEMORY_LIMIT`/`VANTADB_LOCAL_MODEL` con un nuevo `EnvGuard::unset`.
- **Verificación:** `cargo test -p vantadb --features embed-local --lib llm -- --test-threads=2` → **30 passed, 0 failed** (en `HEAD` era 29/1).
- **Commit:** `afb8cc83f`

### Cierre de cabos del hub `VantaDB Docs` (Studio/Team Sync + 3 páginas restauradas + 10 enlaces)
- **Fecha:** 2026-11-23
- **Objetivo:** Quitar la contradicción hub↔PDR en la tabla de precios y devolver al hub sus enlaces de arquitectura/almacenamiento/reviews.
- **Resultado:** ✅ — (1) `Studio Pro` → etiqueta `roadmap post-v0.10.0` y `Team Sync Cloud` → `roadmap post-1.0`, con nota de estado que cita `docs/dev/Backlog-negocio.md:113` (Studio vive en el worktree `develop-desktop`) y `GTM-05` (solo waitlist); precios intactos. (2) Las 3 páginas borradas (`System Architecture & Internals`, `Storage Engine, Data Model & Guarantees`, `Research & Design Reviews DB`) se **restauraron** (no recrearon) conservando los IDs que el hub ya enlazaba, y se les añadió al final un «Índice a las fuentes del repo» (`docs/dev/architecture/ARCHITECTURE.md` + 62 ADRs, `docs/api/STORAGE_FORMATS.md`, `docs/dev/reviews/` con 38 ficheros). Edición **aditiva**: `replace_content` falló con `validation_error: would delete 5 child page(s)` y no se usó `allow_deleting_content`, así que el contenido original y las 9 subpáginas hijas siguen ahí. (3) Las 10 celdas «Documento en Notion» mal formadas pasaron a `<mention>` limpios (`malformed_left: 0`, `mention_count: 10`).
- **Evidencia:** hub `page_last_edited_at 2026-11-23T18:03:27.685Z`; páginas restauradas `18:02:22.993Z`, `18:02:24.026Z`, `18:02:24.897Z`.
- **Correcciones al brief (verificadas):** son **62** ADRs, no 54; `STORAGE_FORMATS.md` describe el storage root como directorio pero **no lo nombra `.vanta`** (`.vanta` es config global del usuario y extensión de ficheros), así que la página lo dice con ese matiz; `docs/dev/Backlog.md` está en `.gitignore:270` (verificado con `git check-ignore`), pero no se afirmó en la página.

### FIND-703: `capabilities` podía decir `Ready` con el provider fallando cerrado (una sola lista de candidatos para pesos y tokenizer)
- **Fecha:** 2026-11-23
- **Objetivo:** Cerrar la regresión de coherencia que introdujo FIND-698 (hallazgo crítico H06-ARCH-001 de L6): `model_files_present` conservaba los fallbacks hardcodeados incondicionales mientras el loader ya los limitaba, así que el probe de salud y el provider podían discrepar.
- **Resultado:** ✅ — (1) `model_path_candidates` y `tokenizer_candidates` viven en `llm.rs` con la misma puerta (hardcodeados solo si el dir está vacío) y `embedding_health::model_files_present` + `try_load_tokenizer` las **consumen**, así que pesos y tokenizer no pueden venir de modelos distintos (antes un dir con `model.onnx` sin `tokenizer.json` cargaba el tokenizer de otro modelo → vectores basura silenciosos); (2) `executor` usa `provider.is_synthetic()` y se elimina `is_synthetic_local_fallback`, que re-derivaba el veredicto desde otro `Config::default()` por nodo en el hot path de INSERT; (3) `resolve_local_model_path("")` deja de resolver al cwd (era lo que hacía inalcanzable toda rama de descubrimiento).
- **Verificación:** `cargo test -p vantadb --features embed-local --lib llm -- --test-threads=2` → **31 passed** (nuevo `model_path_candidates_gate_the_hardcoded_defaults`); `cargo clippy -p vantadb --features embed-local --all-targets -j 2 -- -D warnings` limpio. Binario reinstalado (release offline, 21m29s, `sha256 2d1d2e87e84928b6405ec11b2e036fe08e8c6bea306c10fdb98b037a6b0c5a2b` — **medición histórica de ese día**: ese artefacto quedó fuera del pin en F0.2, el pin vigente es `cc199689…`, ver §F0.2 al final de este fichero): probe de embeddings `passed-functional-embeddings`, probe de perfiles **17/17**, `scopegrain vantadb status --probe` → `model_present: true`, `status: Ready`.
- **Commit:** `3b6f389a3`
- **Nota:** la deuda estructural de fondo (5 funciones de descubrimiento, 3 raíces, 2 políticas) queda en `FIND-704`, igual que **dos residuos que NO cubre este fix**: (a) `detect_dim` sigue leyendo `embeddings/manifest.json` **relativo al cwd** (`src/llm.rs:440`), así que la dimensión (384 e5 / 768 gemma) depende del directorio de lanzamiento; (b) `vantadb-mcp/src/handlers/tools.rs:4810-4827` conserva una tercera copia de la lista de candidatos en producción.

### Verificación final de la cadena de embeddings (estado certificado)
- **Fecha:** 2026-11-23
- **Resultado:** binario con `embed-local` reinstalado y **pinneado entonces** en los tres sitios que lo referencian (`TaskSystem/.tasksystem/research/…inputs.json`, `TaskSystem/.scopegrain/research/…inputs.json`, `TaskSystem/.scopegrain/vantadb.json` → `binarySha256`): `sha256 54e6e2cb33acfd9dd5dbbb31881175f835e0a73f9eb82ed540dae2b0f4b3a393` (incluye FIND-695, FIND-697 y FIND-698) — **medición histórica del 2026-11-23: ese pin se retiró en F0.2; el pin vigente del binario certificado es `cc199689…` (ver §F0.2)**. `vantadb-server --version` → `vantadb-server 0.9.0`. Probe de embeddings → `passed-functional-embeddings` (`.tasksystem/verification/2026-11-16-native-embedding-b8298a71-…json`); probe de perfiles → **17/17** (`…/2026-11-23-mcp-profile-b5432803-…json`); `scopegrain vantadb status --probe` → `model_present: true`, `status: Ready`.
- **Nota:** el sha **no es reproducible entre builds** (cargo re-linkea el binario; `target/release` y el instalado difieren). El pin debe apuntar siempre al artefacto realmente probado.

### FIND-684: Allowlist estricta de tools MCP, recall `scope:"agent"` sobre `l1/sg-*` e instalación del modelo ONNX local (Scopegrain `VB-3`/`VB-1`/`VB-16`/`VB-21`/`VB-22`)
- **Fecha:** 2026-11-23
- **Objetivo:** Certificar en vivo, con el binario reconstruido, los tres cierres operativos que pide Scopegrain `v0.1.1`: allowlist estricta que rechace antes de efectos, recall de L1 con `scope:"agent"` sobre `l1/sg-*` excluyendo `sg-formal-v1-*`, y modelo ONNX local alcanzable desde cualquier cwd.
- **Resultado:** ✅ — (1) `VANTADB_MCP_STRICT_PROFILE=true` + `VANTADB_MCP_ALLOW_TOOLS=memory_get,memory_recall` → `memory_put` rechazado con `-32601` **sin** registrar nada (`memory_get` → `VANTADB_NOT_FOUND`); (2) `memory_recall{scope:"agent"}` recupera el registro de `l1/sg-probe` (`count: 1`, `mode: hybrid`) y el namespace `sg-formal-v1-*` nunca entra al pool; (3) `model_dir` resuelto contra el data dir (`scopegrain vantadb status --probe` → `model_present: true`, `status: Ready`). Probe: `.scopegrain/research/2026-11-23-vantadb-profile-probe.mjs`, **15/15 checks**; evidencia `.tasksystem/verification/2026-11-23-mcp-profile-17638652-f9dd-47ce-8355-becd3e0cc182.json`. Residuo documentado: el recall exige que el payload de `l1/*` sea el envelope `MemoryRecord` completo (`FIND-699`).
- **Commit:** `01a38f353` (fix) + `bed6473df` (model_dir) · verificación 2026-11-23

### FIND-682: Egress HTTP implícito a Ollama con `VANTADB_EMBEDDING_PROVIDER=local` en un binario sin features de embeddings (Scopegrain `VB-4`)
- **Fecha:** 2026-11-23
- **Objetivo:** Certificar con un binario **compilado sin** `embed-local`/`remote-inference` que `provider=local` no dispara tráfico HTTP a Ollama y que la degradación es explícita, no silenciosa.
- **Resultado:** ✅ — binario sin features (`sha256 a470d1a3…`, `cargo install --path vantadb-server --force -j 2`, 8m14s) + probe con tripwire HTTP (`2026-11-23-vantadb-no-egress-probe.mjs`), **5/5 checks**:
  - `capabilities` → `effective_provider: "none"`, `compiled_features: {embed_local: false, remote_inference: false}` (no enruta a `ollama`).
  - `embed_texts` → `-32603 "strict embeddings mode: binary compiled without embedding features"` (falla cerrado).
  - **0 requests** al endpoint HTTP en las dos llamadas.
  - `memory_put` → `fallback: true` + `warning` explícito y `vector: null` (no persiste vector sintético).
- **Evidencia:** `.tasksystem/verification/2026-11-23-no-egress-901e16fe-7508-4457-8418-6c5a4e8a3a31.json`. El binario con `embed-local` se reinstaló después para dejar la máquina en estado certificado.
- **Commit:** `01a38f353` (fix) · verificación 2026-11-23

### FIND-699: `memory_put` no documentaba ni validaba el contrato del envelope L1 (fallo silencioso de datos)
- **Fecha:** 2026-11-23
- **Objetivo:** Cerrar el fallo silencioso detectado en la certificación VB-1: un `memory_put` con payload de texto plano se guardaba, era buscable y **nunca** aparecía en `memory_recall` con `scope:"agent"`/`"team"`, porque `sdk_to_memory_record` fijaba `agent_id`/`team_id` a `None`.
- **Resultado:** ✅ — (1) `l1_reader.rs` lee `agent_id`/`team_id`/`user_id` del metadata del registro en el fallback (trim, vacío → `None`), resolviéndolos antes del literal del struct porque `meta_map` se mueve a `metadata`; (2) el `inputSchema` de `memory_put` documenta el contrato del envelope L1 para namespaces `l1/*` y el fallback por metadata; (3) el test de regresión dejó de ser vacuo: su fixture era un envelope **incompleto** (sin `id`/`type`/`priority`/`scene_name`/`created_at`/`updated_at`), el parseo fallaba y recall devolvía vacío — por eso el assert negativo "pasaba". Ahora usa un envelope completo y asserta el lado positivo: el envelope se recuerda **y** un payload plano con `metadata.agent_id` también.
- **Verificación:** `cargo test -p vantadb-mcp -- --test-threads=2` → todos los targets ok, 0 fallos (lib 52/52, incluido el `vb4_and_vb16` antes vacuo).
- **Commit:** `61c2be235`

### FIND-700: los 4 jobs de `docs-gates` en rojo al trackearse `docs/`
- **Fecha:** 2026-11-24 (cierre; medido el 2026-11-23)
- **Objetivo:** Devolver a verde `Internal links resolve`, `Frontmatter schema` + `kind agreement`, `No credential in published docs` y `Generated indexes are current`, rojos desde que `b0d740e32` empezó a trackear `docs/` (465 ficheros).
- **Resultado:** ✅ — 42 enlaces rotos → **0** (4 repuntados a la ruta real; 38 retirados porque apuntaban a ficheros que `255db8208` borró a propósito: glosario, `VISION.md`, `verified-numbers.md`, etc., dejando el término como texto); frontmatter 47+24+5 → **0** con `stamp-frontmatter.mjs --fix-kind` (cuerpos intactos, paridad verificada contra `HEAD`); índices regenerados (343→723 entradas, 0 docs perdidos); el único `aws-secret-access-key` era un falso positivo dentro de una URL `file://` (ventana de 40 chars sobre `%20Proyect/…SHOW`), declarado en `scripts/docs/secrets-allowlist.json` con patrón y razón en vez de mutilar la URL.
- **Corrección de premisa:** la fila afirmaba que `check-links.mjs` no estaba cableado en CI. **Es falso**: el job `docs-links` / `Internal links resolve` existe y es **required** (`.github/workflows/docs-gates.yml:196`). El rojo no era un gate ausente sino deuda heredada que quedó visible al trackear `docs/`.
- **Verificación:** los cuatro gates ejecutados por mí el 2026-11-24: `check-links` 0 broken, `check-docs` all clear, `check-secrets` 0 errores (4 warnings informativos, presupuesto 28), `gen-index --check` sin stale.
- **Commit:** `c54165cff`
- **Derivada:** `FIND-707` (3 bugs de `stamp-frontmatter.mjs` con BOM/CRLF que hubo que reparar a mano en 33 docs).

### FIND-706: latencia de tools MCP creciente bajo escritura sostenida (y el `exit 255` de `MCP-42`)
- **Fecha:** 2026-11-23
- **Objetivo:** Reproducir el `exit 255` del writer directo registrado en `MCP-42` (36 m 37 s) antes de declararlo conocido o bloqueante.
- **Resultado:** 🟡 — **no se reproduce como crash**. Repro de 15 min con 3 tool calls por ronda cada 60 s (DB aislada, binario `2d1d2e87…` — repro histórico; pin vigente `cc199689…`, §F0.2): rondas 1-9 OK con latencias 17 / 28 / 28 / 23 / 27 / 48 / 24 / 30 / 63 s; ronda 10 → `MCP error -32001: Request timed out` a los 79 s. El stderr del server muestra **cierre limpio** (`MCP stdio server shut down total=32 errors=0`, `cleaned discovery file on Drop`, `HNSW index persisted`, `Storage flushed`), sin panic ni OOM.
- **Cuantificación (2026-11-24, 10 muestras por población, DB aislada):** `memory_put` con auto-embed **mediana 2421 ms** (2307–3536); con vector provisto **7 ms**; `capabilities` **1 ms**; `initialize` 3065 ms. Es decir: **99,7 % del coste es el embedding ONNX síncrono por escritura**; storage y transporte son irrelevantes. La vía de fix es cachear embedding por texto y reutilizar la sesión (o batch), no tocar el motor. Un cliente que mata el hijo al expirar una petición (>60 s) produce el `255` observado.
- **Registro:** `FIND-706` (abierta)

### F0.2: rebuild del binario certificado y re-pin del sha (release v0.10.0, Fase 0)
- **Fecha:** 2026-11-24
- **Objetivo:** el sha que el release cita como binario certificado (`2d1d2e87…`, instalado y probado por el certify del 2026-11-23 a las 15:41) **ya no corresponde al código del tag**: es anterior a `61c2be235` (`FIND-699`, contrato del envelope L1, 2026-11-23 18:10) y a `d0df55c2b` (`FIND-485`, `__vanta_embed_status`, 2026-11-24). **Corrección de la premisa del plan:** la fila F0.2 dice «anterior a `FIND-699`/`703`/`485»`, pero para `FIND-703` es falso — `3b6f389a3` es de las 15:17 y el binario se re-construyó después, así que ya lo incluía (`review-certify-20261123-154221.md:43,77`; la evidencia `…mcp-profile-383b8e00-….json` es de las 15:41:07). Rebuild, re-probe y re-pin con la premisa corregida → pin nuevo `cc199689…`.
- **Resultado:** ✅ `cargo install --path vantadb-server --features embed-local --force -j 2 --locked` → exit 0, **23m47s**. El `--locked` resolvió (lock con `url 2.5.9`, `Cargo.lock:5613-5614`); no hizo falta el fallback sin `--locked`. Binario nuevo: `sha256 cc199689f3623036c1c37a846b9cce4d659c96e570471a615cfe54ce2fe921e5` (`Get-FileHash -Algorithm SHA256` sobre el instalado; `vantadb-server --version` → `vantadb-server 0.9.0`, la versión del workspace la sube release-plz en F0.4). **Reemplaza a `2d1d2e87…` como pin vigente.**
- **Probes (los tres sobre el binario nuevo).** Embeddings: `node .scopegrain/research/2026-11-16-vantadb-embedding-probe.mjs <root>` → stdout `{status: "passed-functional-embeddings", controlIssues: [], positivePassed: true, error: null}`. Perfiles MCP: `2026-11-23-vantadb-profile-probe.mjs` → stdout `{status: "passed-mcp-profile-contract", checks: 17, failed: []}`: allowlist estricta sin efecto colateral, `dedup_policy` (enum + rechazo `-32602` + `off`/`exact`), recall `scope:"agent"` sobre `l1/sg-*` y exclusión de `sg-formal-v1-*`. `scopegrain vantadb status --probe` → `model_present: true`, `status: Ready`, `vector_search: true`, `persistence: true`, `iql_queries: true`, `runtime_profile: Performance`.
- **Evidencia:** `.scopegrain/verification/2026-11-16-native-embedding-a88b8e8e-e937-4e24-b6a0-be9ed80ac0ed.json` y `.scopegrain/verification/2026-11-23-mcp-profile-b565bb4a-2db2-4d8d-a8cb-e1e330bee2d8.json`; ambos registran el sha nuevo, en campos distintos: `inputs.executable.sha256` (embeddings) y `executableSha256` (perfiles). Re-pin de los dos pines locales de Scopegrain: `.scopegrain/vantadb.json` → `binarySha256` y `.scopegrain/research/2026-11-16-native-embedding-probe-inputs.json` → `executable.sha256`; ambos **verifican** el digest (`verifyFile` en el probe de embeddings, `verifyBinaryDigest` en `vantadb-runtime.mjs`), así que sin re-pin los probes fallan cerrado.
- **Desvíos de entorno (documentados):** (1) el primer `cargo install` compiló bien (22m59s) y el *move* falló con `os error 5` porque los `vantadb-server --mcp` vivos tenían el binario abierto (clase `FIND-177`; `Get-Process` los listó y se detuvieron con `Stop-Process`). Hubo además un intento interrumpido (`f02-install2.log`, sin `Finished`). (2) El CLI global `scopegrain` no resuelve (`MODULE_NOT_FOUND` en `%APPDATA%\npm\node_modules\scopegrain\bin\scopegrain.mjs`); se corrió `node .\bin\scopegrain.mjs` desde el repo hermano. (3) El probe de embeddings lee su spec en `<root>/.tasksystem/research/`, mientras vive en `.scopegrain/research/`: se ejecutó con un root de trabajo en `%TEMP%` que replica `.tasksystem/research/` y la evidencia se copió a `.scopegrain/verification/`. Logs: `%TEMP%\opencode\f02-install*.log`.
- **Estado certificado anterior corregido:** este fichero tenía **cinco** shas en juego (`54e6e2cb…`, `a470d1a3…`, `0dce814c…`, `2d1d2e87…`, `cc199689…`) y la entrada «Verificación final de la cadena de embeddings (estado certificado)» seguía presentando `54e6e2cb…` como el pin, con esta nota de vigencia añadida. Desde acá el pin vigente es, y solo es, `cc199689…`.
- **Nota:** el sha no es reproducible entre builds (ver §«Verificación final de la cadena de embeddings»); vale para el artefacto instalado que corrieron estos probes.
- **Commit:** (este commit)

### FIND-177 — Lock de bins en Windows con MCP vivo (2026-11-24)
- **Qué:** `docs/dev/references/troubleshooting.md` (nuevo): síntoma `Acceso denegado (os error 5)` / `LNK1104` / `failed to remove file ...\target\debug\vantadb-server.exe`; causa (launcher copia ambos bins a `%TEMP%\vanta-mcp-bin`, el lock aparece con MCP arrancado in place); fix por **`Path` del proceso, nunca por nombre** (el cliente `opencode.exe` aparece siempre y matarlo mataría la sesión); workaround `cargo nextest run --profile audit -p vantadb --lib --features server --build-jobs 2` (WIRE-04) con su alcance real.
- **Verificación:** contrato `rg` 6/1 hits · `check-docs` all clear · `check-links` 0 broken · markdownlint 0 issues. `verify.ps1` no corrido (carril Rust ocupado).
- **Commit:** `2dd8501af`.
- **Hallazgo que corrige la fila:** `dev-tools/verify.ps1` **no** cae por este lock (su `nextest` es `-p vantadb`, línea 87); el único step expuesto es `cli-probes` (línea 106). El probe advisory quedó propuesto para `FIND-192` con línea y cambio exactos, sin aplicar.
- **Deuda:** D1 consolidar con `docs/user/troubleshooting.md` (decisión del owner) · D2 probe advisory en `verify.ps1` (para `FIND-192`) · D3 `docs/index.md`+`llms.txt` se regeneran en el cierre de wave · D4 huérfano report-only para `check-docs`.

### Higiene de entorno en waves: cleanup no interactivo de `target/` (`FIND-192`)
- **Fecha:** 2026-11-24
- **Objetivo:** El disco C: llegó a **0 bytes** por `target/` (31 GB liberados a mano durante este mismo pipeline) y el link falló con `LNK1140`. `dev-tools/target-cleanup.ps1` existía pero solo se usaba a mano.
- **Resultado:** ✅ Gate pre-wave no interactivo con cuatro puertas antes de cada `Remove-Item`: umbral (`-MinFreeGB`, default 15 GB) → guarda de procesos `cargo/rustc/clippy-driver/rustdoc` **re-chequeada por ítem** (no una vez) → `Assert-UnderTarget` (`GetFullPath` + prefijo **con** separador: cierra `..`, `target2/`, `target\.`, `target/` mismo) → salto de reparse points. Modos: `-DryRun`, `-AutoSafe`, `-AutoSafe -DryRun`, `-SelfCheck` (PASS 17/17 sobre rutas reales). Exit codes para runner: `0` ok · `2` builds en vuelo · `3` sigue bajo umbral (no arrancar la wave) · `1` rechazo. Umbral, comandos y exit codes documentados en `docs/dev/references/troubleshooting.md`.
- **Causa raíz medida:** los PDB llegan a ~10 GB porque `Cargo.toml:743` (`[profile.bench] debug = 1`) y `:856-857` (`[profile.test] debug = 1`) son deliberados — cada target de test/bench enlazado emite su `.pdb`; apagarlo rompería flamegraphs y backtraces. `incremental` crece por sesión crate×perfil sin GC (12 GB hoy, ~22 GB en el incidente). `--incremental` solo se rompe si se borra **a mitad de build** — por eso la guarda aborta en vez de "limpiar igual".
- **Commit:** `e20fad63f`
- **Actualizado 2026-11-25 (`FIND-746`):** el exit `2` **ya no significa "builds en vuelo"**. Con un build vivo el gate ahora **degrada** y poda `*.pdb` con antigüedad ≥ `-MinAgeMinutes` (default 30) sin tocar `incremental/`; `2` quedó acotado a "degradado que no pudo liberar nada y el disco sigue bajo umbral". Ver §`FIND-746` más abajo y `docs/dev/references/troubleshooting.md`.
- **Deuda:** D1 el probe WIN-LOCK de `FIND-177` sigue sin aplicar en `dev-tools/verify.ps1` · D2 no existe un runner versionado de waves que invoque `-AutoSafe`: umbral y exit codes quedan listos para enchufarlo · D3 el borrado real no se ejecutó (6 `cargo` + 2 `rustc` en vuelo, 4.8 GB libres al momento del cierre).

### Los tests dejaban GB en `%TEMP%` cuando el proceso moría sin `Drop` (`FIND-732`)
- **Fecha:** 2026-11-25
- **Objetivo:** 15,35 GB en un día (1.887 entradas) y **tres** gates abortados por `StorageFull`, el último con **0,52 GB** libres. El síntoma declarado ("los tests limpian bien, no al fallar") era **falso a medias**.
- **Resultado:** ✅ El discriminante no es pasar/fallar sino **`Drop` de no-`Drop`**. `tempfile::TempDir` limpia en `Drop` y su `drop` ignora el error, así que un `panic` (unwind) limpia y un proceso **matado** (nextest `slow-timeout terminate-after=3`, `abort`, SIGKILL) no. Además `FjallBackend::compact()` deja `<TEMP>\.fjall_compaction_tmp_<pid>` por cada compactación interrumpida, porque escribe el rewrite como hermano del data dir y solo lo borra en la compactación siguiente.
  - **Medición:** 22 tests Fjall en verde → **0 residuo** · test con `panic!` → **0** · test con `std::process::abort()` → `.tmpXXXXXX` de **68,1 MB con `lock`**.
  - **Fix:** `dev-tools/temp-collector.ps1` (nuevo) con **dos pruebas de abandono** — lock de Fjall vía `FileShare.None` **e** inactividad de 10 min (la segunda cierra las ventanas vivas: el lock solo no basta y `Remove-Item` borra a través de handles) — más `Assert-UnderTemp`. Invocado de forma **advisory** desde `dev-tools/verify.ps1` (antes de `nextest`) y `.githooks/pre-push`. **Cero tests reescritos.**
- **Verificación:** residuo medido → recolectado (68,1 MB → 0, `-MinAgeMinutes 0`) · barrido por defecto encontró **136 MB reales sin catalogar** (`vanta-cog-ckpt-*`, 69 min idle) · `-SelfCheck` **PASS 19/19** (incluye el caso real `.tmp*`+lock libre, la guarda de inactividad y "`.tmp*` ajeno intacto") · `target-cleanup -SelfCheck` 16/16 · `bash -n pre-push` 0 · `check-docs` exit 0.
- **Commit:** `264940d57`
- **Review P2-01:** `changes-required` → corregido y re-medido en la misma ronda: **F1** el lock era la única prueba de vida (3 ventanas vivas + `Remove-Item` borra a través de handles) → segunda prueba obligatoria de inactividad; **F2** el self-check no ejercitaba `.tmp*`+lock libre (95 % del residuo) → escenario real con 4 asserts; F3-F12 aritmética, exit codes, prefijos `vanta*`/`vantadb*`.
- **Deuda sugerida (fila `FIND-735`):** hay tests con **path fijo** bajo `%TEMP%` (`src/schema.rs:260`, `src/wal.rs:2396,2443,2489`, `src/index/serialize/file.rs:435`): colisionan entre corridas concurrentes y ponen el residuo de compactación directamente en `%TEMP%`. Hoy mitigado por nombre, no arreglado.

### El recolector de temporales también barre ficheros sueltos (`FIND-743`)
- **Fecha:** 2026-11-25
- **Objetivo:** el residuo suelto que dejan los tests (`vanta_test_wal_healing_<rand>.corrupt` 63 B, `vanta_test_sharded_<rand>.shard0.corrupt` 54 B, `vanta-ckpt-<pid>.jsonl*`) era **invisible** para el recolector de `FIND-732`, que iteraba sólo **directorios** bajo `%TEMP%`: no entraba en su inventario ni en su reporte, así que "limpio" significaba a la vez "no hay nada" y "no miré la mitad".
- **Resultado:** ✅ El inventario incluye **ficheros sueltos** con la **misma doble prueba** que los directorios — lock `FileShare.None` sobre el propio fichero **e** inactividad `-MinAgeMinutes` (10 por defecto) — más una **re-verificación de la guarda justo antes de borrar** (entre inventario y borrado una DB puede reabrirse; se vio en vivo un fichero en tránsito saltado como "respetado por duda"). El reporte **separa** `dirs` de `files` con su tamaño y siempre imprime ambas líneas, incluso en cero. `Invoke-Collect` es el único camino de borrado: lo usan el barrido real y el self-check, así que el assert ya no puede pasar sin ejercitarlo.
  - **Sonda verificada empíricamente** (Windows 11 26200, PS 7.6.10): `FileShare.None` falla con el fichero abierto por cualquier otro handle, sea cual sea su share (`None`, `Read`, `ReadWrite`, `ReadWrite+Delete`) **y es cross-process**. Complemento necesario medido: `Remove-Item` **sí** borra a través de un handle que concede `DELETE` (lo que concede `std::fs::File`) → quien protege es la sonda, **no** `Remove-Item`.
  - **El review cazó un assert vacuo**: el del fichero abierto usaba `FileShare.ReadWrite` (sin `DELETE`), así que `Remove-Item` fallaba solo y el assert pasaba **aunque se borrara la guarda** (mutación M1 → 29/29 igual). Corregido con handles que conceden `DELETE` **más un control positivo** que prueba que un `Remove-Item` directo sí borra.
- **Verificación:** `-SelfCheck` **PASS 36/36** (19 previos + 17 nuevos: libre se borra · abierto se respeta · reciente se respeta · `Report_*.txt`/`notas-vanta_*.txt` ajenos intactos · prefijo válido fuera de `%TEMP%` rechazado end-to-end · reapertura de DB tras el inventario respetada · control positivo de que `Remove-Item` atraviesa un handle con `DELETE` · `vanta-ckpt-<pid>` clasificado con su PID) · inventario real: **15 ficheros sueltos (2,0 KB)** invisibles hasta hoy + 1 `.tmp*` de 4,02 MB · barrido real: **15/15 ficheros + 1/1 carpeta, 4,02 MB liberados** · señuelos con control positivo **7/7** · mutaciones (sin guarda / sin sonda / sin inactividad) **FAIL 32-35/36**.
- **Commit:** `3bfd2f14a`
- **Review P2-01:** tier fast (`dev-tools/**`) `changes-required` → corregido en la misma ronda: **F1** el assert del fichero abierto era vacuo; **F2** los directorios se borraban por un camino duplicado sin cubrir la guarda nueva. También F4 (`-Force` en la enumeración → un residuo *Hidden* ahora se ve y se borra), F5 (`Test-PidAlive` para `vanta-ckpt-<pid>`), F6 (dudas separadas de fallos de borrado), F7 (`Assert-UnderTemp` end-to-end en vez de un assert tautológico).
- **Deuda restante:** `FIND-744` sigue **produciendo** el `.shard0.corrupt` — este ticket sólo hace que **se vea y se recoja**.

### El binario declara su commit, para que un artefacto stale no pueda pasar un DoD (`FIND-741`)
- **Fecha:** 2026-11-25
- **Objetivo:** `MCP-47` verificó el L0 de `MCP-44` contra el binario de `target/release` (16:13), **anterior** a `2fb20c728`. El binario respondió `{"ok":true}` **sin** el bloque `memory:{captured,queued}`: la feature no estaba y la respuesta tenía **la forma anterior**. Casi todos los DoD de la ola (`MCP-43`, `MCP-44`, `MCP-47`, `MEM-73`) se verifican contra un binario, así que un artefacto stale **devuelve algo plausible y el test pasa sin probar nada**.
- **Resultado:** ✅ `vantadb-server --version` → `vantadb-server 0.9.0 (commit <sha>, worktree clean|dirty)`. El SHA se **graba en el artefacto** (`build.rs` → `cargo:rustc-env`), no se consulta al ejecutar.
  - **Por qué embebido y no `git rev-parse` en runtime:** la pregunta del DoD es *"¿lo que ejecuté salió de este commit?"*; un `git rev-parse` en runtime contesta *"¿en qué commit estoy parado ahora?"* y **colapsa (a) binario == HEAD con (b) binario de un commit anterior** en la misma salida, así que el test no podría fallar nunca — sería decorativo. `vergen` haría lo mismo con ~10 crates transitorios, sin justificar la dependencia.
  - **Por qué en el crate hoja:** `cargo:rustc-env` pertenece al paquete **emisor**, así que un commit invalida **sólo `vantadb-server`**. En el crate raíz invalidaría el engine de 200k LOC en cada commit y tiraría sccache workspace-wide.
  - **`rerun-if-changed`** sobre `.git/{HEAD,index,logs/HEAD}` (resuelto con `--absolute-git-dir`, funciona en worktrees), fuentes/manifests del engine linkeado, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml`. Sin git → `unknown` (+ escape `VANTADB_BUILD_COMMIT` para CI/Docker, con `cargo:warning` si discrepa).
- **Coste medido:** commit → rebuild **~19 s de `vantadb-server`, cero del engine** (`Compiling vantadb v0.9.0` nunca aparece; medido también por el reviewer de forma independiente) · no-op con árbol quieto **1,55 s** · sin dependencias nuevas, `Cargo.lock` intacto.
- **Verificación:** `vantadb-server/tests/git_commit_declared.rs` lanza el **proceso real** vía `CARGO_BIN_EXE_*` y falla si el sha declarado ≠ `git rev-parse --short HEAD`; sólo se salta si **no hay metadata git** (si `.git` existe y git falla, **falla**). Repro en vivo, sin fabricar el escenario: el binario decía `d246ba9a6` mientras HEAD ya era `3bfd2f14a` → el check falla y **obliga a rebuild**. El artefacto de `target/release` (un día de antigüedad) sigue diciendo sólo `vantadb-server 0.9.0` → no declara commit, no pasa. Review P2-01 APPROVE (7/7 del contrato con comando y salida).
- **Commit:** `e8df96232`
- **Protocolo para los DoD de la ola** (usar el path **exacto**, nunca el de `PATH`):
  ```powershell
  $bin = "./target/debug/vantadb-server.exe"
  & $bin --version                              # (commit <sha>, worktree clean|dirty)
  git rev-parse --short HEAD                    # debe coincidir
  git status --porcelain --untracked-files=no   # debe estar vacío
  ```
  Verificar **sólo** si los dos SHAs coinciden **y** dice `worktree clean`. Si no: `cargo build -p vantadb-server --bin vantadb-server` y repetir.
- **Deuda:** `FIND-98` queda con base — el mismo mecanismo en el paquete raíz detectaría el `vanta-cli` instalado stale, a cambio de un rebuild del engine por commit; el pin `binarySha256` del binario certificado hay que re-derivarlo en la próxima reinstalación.

### `-AutoSafe` degrada a podar `*.pdb` en vez de negarse en bloque (`FIND-746`)
- **Fecha:** 2026-11-25
- **Objetivo:** la guarda de builds en vuelo era **todo o nada**: con cualquier `cargo`/`rustc` vivo no borraba **nada**, incluidos los `*.pdb`, que **no participan** de la cache incremental de rustc. El 2026-11-25 eso terminó en disco a **0 bytes** y **7,6 GB** liberados **a mano** durante `MCP-43`. Negarse a podar lo que se puede probar seguro es peor que podarlo.
- **Resultado:** ✅ `Invoke-DegradedPrune` poda **solo** grupos `Kind='pdb'` con `LastWriteTime <= now - MinAgeMinutes` (default **30**; `<1` con `-AutoSafe` → `REFUSED` exit 1), deja todo `incremental/` intacto y **lo dice** en el output. `Select-PdbItems` centraliza la selección y la usan el gate **y** el self-check, así que romperla rompe el check (M7 → 7 FAILs). `Assert-UnderTarget` sigue siendo la **última puerta** antes de `Remove-Item` (más salto de reparse points y sin `-Recurse`).
  - **Un exit code cambió:** antes `2` = "builds en vuelo, no se tocó nada" incondicional. Ahora el camino degradado da **`0`** si podó algo (o si el disco ya está sobre umbral) y **`2`** solo si **no pudo liberar nada** y el disco sigue bajo umbral. `0`/`1`/`3` del camino sin builds, `-DryRun` y `-Clean` quedan idénticos. **Sin consumidores que se rompan:** `rg 'target-cleanup|AutoSafe' .githooks/ dev-tools/verify.ps1 dev-tools/verify_changed.ps1 justfile` → 0 hits.
- **Medición (2026-11-25, disco en 10,36 GB, 5 builds en vuelo):** `*.pdb` bajo `target/` = **307 ficheros / 7,89 GB**; **podable ahora (≥30 min) = 132 ficheros / 3,60 GB**; `incremental/` (intacto) = 3 dirs · 14,51 GB; total sin builds = 22,38 GB. Primera medición del día (target más chico): 121 ficheros / 2,47 GB.
- **Verificación:** `-SelfCheck` **PASS 40/40** (19 previos + 21 nuevos; incluye build simulado real, junction, señuelo fuera del prefijo y cableado del caller) · **7/7 mutaciones** lo rompen (M1 guarda `Kind`, M2 antigüedad, M3 `exit 2`, M4 `Assert-UnderTarget`, M5 `REFUSED`, M6 reparse, M7 filtro del caller) · `-AutoSafe -DryRun` exit 0 · `-AutoSafe -MinAgeMinutes 0` → exit 1 `REFUSED` · check-docs/check-links/markdownlint 0 · review P2-01 `changes-required` (F1 cobertura del cableado, F2 claim falso) → corregidos + F3-F8.
- **Commit:** `c70370078`
- **Corregido en este cierre:** la entrada de `FIND-192` (`e20fad63f`) documentaba `2 = builds en vuelo`; quedó con una nota de actualización apuntando acá, porque un lector que la leyera sola habría interpretado al revés el exit code.

### FIND-98: el `vanta-cli` instalado declara de qué commit salió, y un check falla si no es `HEAD`
- **Fecha:** 2026-11-26
- **Objetivo:** la fila decía «`vanta-cli` instalado stale (79 vs 87 tools)». El binario instalado (`~/.cargo/bin/vanta-cli.exe`, **2026-09-11**) respondía `vanta-cli 0.9.0` igual que uno fresco: nada en el artefacto permitía responder «¿esto salió de `HEAD`?» — el mismo agujero que `FIND-741` cerró para `vantadb-server`.
- **Premisa corregida (medida):** el «79 vs 87 tools» no mide el CLI. El registro de tools vive en `vantadb-mcp`, que linkea `vantadb-server`; `vanta-cli server --mcp` lo spawnea (`src/cli_handlers/server.rs:314`). Con el server instalado al día, instalado y fuente dan **84 == 84** con `VANTADB_MCP_PROFILE=full` aunque el CLI tenga 2,5 meses. El criterio viejo del plan (`tools/list` diff 0) **pasa con un CLI stale**: se cambió el criterio, no el número. Lo stale era el binario del CLI (diff de superficie: solo la descripción de `encryption`), no el server.
- **Diagnóstico:** copia manual a `~/.cargo/bin` (sin entrada en `~/.cargo/.crates2.json`, sin script de instalación del CLI, sin validación posterior) + `--version` que solo imprime `CARGO_PKG_VERSION`, constante durante todo un ciclo `0.9.x`.
- **Resultado:** `build.rs` (raíz, ya existía como no-op) escribe `OUT_DIR/build_version.txt` = `<version> (commit <sha>)` y `src/bin/vanta-cli.rs` lo embebe con `include_str!`; `--version`/`-V` lo imprimen **conservando el prefijo** `vanta-cli <version>` que parsea `scripts/verify-release.ps1:163`. Nuevo `scripts/check-cli-fresh.ps1`: 1 si el instalado no declara commit, declara `commit unknown` o declara uno que no es `HEAD` (nombra ambos); 0 si coincide; **2 si no puede comprobar** (binario inexistente, que no arranca, o cwd sin repo) — "no pude leerlo" nunca se reporta como "stale".
- **Sin etiqueta `worktree clean|dirty`** (a diferencia de FIND-741): la etiqueta solo es cierta si el script se re-ejecuta en cada cambio del árbol, o sea en cada edición de `src/`, y entonces el label cambia → se recompila. `vantadb-server` puede permitírselo (~1,4k líneas); el engine de 200k no. Se vigilan `.git/HEAD`, `.git/logs/HEAD` y la ref simbólica (`--git-path`, correcto en worktrees). Un build con cambios sin commitear declara el commit base y el script lo avisa con `git status` en vivo.
- **Por qué no `cargo:rustc-env` (el mecanismo de FIND-741):** `rustc-env` pertenece al **paquete**, y este paquete es el engine de 200k LOC: cada commit invalidaría la lib y todos los bindings. El fichero de `OUT_DIR` solo entra en el dep-info del target bin. **Coste medido** (`cargo build -p vantadb --bin vanta-cli -j 2 -v`, descartando corridas con `Blocking waiting for file lock`): simular commit (`touch .git/logs/HEAD`) → **9,5 s**, `rustc src/lib.rs` **0**, `rustc vanta_cli` 1; `git add`/index (no vigilado) → **0,5 s**, cero rustc; no-op → 0,5 s.
- **Hallazgo del reviewer (F4) que cambió el diseño:** con `.git/index` en el rerun set, cada `git add` de **cualquier** carril relinkeaba el CLI (8-12 s) aunque el fichero de identidad no cambiara — medido con mtime idéntico antes/después. `write_if_changed` no lo evitaba (cargo reconstruye los dependientes cuando el script se re-ejecuta), así que se borró y se quitó `index`: 8-12 s → 0,5 s.
- **Contrato (dos corridas literales, HEAD `e1b917fc8`):** antes, con el binario del 2026-09-11 puesto de vuelta → `STALE: the installed vanta-cli does not declare a commit - it predates FIND-98 and cannot be verified.` + `binary`, `reports: vanta-cli 0.9.0`, `HEAD: e1b917fc8`, exit 1. Después de reconstruir e instalar → `OK: installed vanta-cli matches HEAD (e1b917fc8) - vanta-cli 0.9.0 (commit e1b917fc8)`, exit 0 (con aviso de árbol sucio, en vivo).
- **Matriz de códigos (binarios falsos):** sin commit → 1 · `commit unknown` → 1 · sha ajeno al repo → 1 · crash (exit 7, sin salida) → **2** · ruta inexistente → **2**.
- **Test:** `find98_build_identity::version_declares_a_commit_from_this_history` (proceso real `CARGO_BIN_EXE_vanta-cli`; `nextest` excluye `cli_tests` del perfil default, se corre con `cargo test -p vantadb --test cli_tests find98`). Afirma que el commit declarado **existe y es ancestro de `HEAD`**, no igualdad: en este árbol multi-carril `HEAD` avanza entre compilar y correr (la primera corrida GREEN falló con `HEAD = 1de8d9896` vs binario `f097c5f78`, exactamente esa carrera). La igualdad la exige el script, a demanda — el reparto que FIND-741 ya había documentado para `vantadb-server`. **Techo:** el test no puede fallar por un binario stale; el único gate de frescura es el script.
- **Review P2-01:** `vanta-review`, ronda 1, **CHANGES-REQUIRED** (2 high + 4 medium + 4 low, 0 critical, todo verificado en vivo) → los dos high (label de árbol stale; aserción de test vacua) y los medium accionables (exit 2 real; rerun set/`write_if_changed`; mensaje de `commit unknown`) se cerraron con la medición que los motivaba. Quedan como techo/deuda: un build sucio declara el commit base (el artefacto no puede saberlo), nada corre el script en CI/hook (`.github/**` y `dev-tools/**` fuera de alcance; el gate hermano de `e2e-memory-cycle.ps1:25-31` sí está cableado en `nightly.yml:276-284`), y el tarball de release (sin `.git`) necesita `VANTADB_BUILD_COMMIT`.
- **Commit:** `3926de7c6` (código) + `89197adac` (ronda P2-01) + este commit (cierre).
- **Deuda:** la reinstalación se hizo copiando `target/debug/vanta-cli.exe` (perfil dev `opt-level=1`) con respaldo `vanta-cli.exe.bak-find98`, el fix que la propia fila del Backlog sancionaba; no se corrió `cargo install` (release) por tiempo/espacio. El pin `binarySha256` del binario certificado apunta a `vantadb-server`, no al CLI, así que no requiere re-derivarse.

### FIND-754: el check de frescura del CLI corre en el `pre-push` y avisa, sin cambiar el veredicto
- **Fecha:** 2026-11-26
- **Objetivo:** `scripts/check-cli-fresh.ps1` (`FIND-98`) detectaba correctamente un `vanta-cli` stale y **no lo invocaba nada**: el binario instalado seguía respondiendo `vanta-cli 0.9.0 (commit 210bbfaa5)` con `HEAD` en `2f8ea4d76`. Un instrumento correcto que nadie ejecuta es indistinguible de no tenerlo.
- **Decisión (y por qué no bloqueante):** el check va al `pre-push` en modo **advisory**. El CLI stale lo sufre **quien lo usa**, no quien pushea, y el push no es el momento de arreglarlo: un rojo por «no instalé el CLI» frenaría pushes legítimos de la mayoría de los carriles. **No** va al gate por PR porque compara contra un binario instalado en `~/.cargo/bin`, que un runner no tiene. El job de `nightly` que lo haría universal queda **propuesto** con su YAML en la ficha (`nightly.yml` es de otro carril: `FIND-769`).
- **Resultado:** `scripts/check-cli-fresh.ps1` gana `-Advisory` (todas las salidas pasan por `Exit-Check`): mantiene la matriz `0/1/2` en modo default y en advisory sale **0 en todos sus caminos** — calla cuando no hay binario **o cuando ya coincide con `HEAD`**, nombra el stale con los dos commits y el `cargo install … --force`, y dice la razón cuando no puede leerlo. El hook **llama el script tal cual** (no reimplementa la comparación), una vez por push, y **no lo llama con `CI` definido**.
- **Radio de explosión acotado a propósito:** el cambio al hook es **puramente aditivo** (`git diff -U0` → 0 líneas `-`), así que ningún camino de bloqueo existente se toca; `bash -n .githooks/pre-push` → 0. El hook lo usan todos los carriles, así que la prueba es el contrato.
- **Prueba de tres caras (hook real bajo Git Bash, rango `HEAD~1..HEAD`, gates pesados con `SKIP=` para no compilar):**
  - **fresco** (binario que declara `HEAD`): **ninguna línea** del check (silencio deliberado: es el caso sin hallazgo), `pre-push: barrier passed. Push allowed.` **exit=0**.
  - **stale** (el instalado de verdad, declara `210bbfaa5`): `pre-push: CLI freshness ADVISORY (does not block this push):` + `STALE: the installed vanta-cli is behind HEAD.` + `binary declares: 210bbfaa5 (a different commit)` + `repo HEAD: 2f8ea4d76` + `fix: cargo install --path "…" --bin vanta-cli --force` → `barrier passed. Push allowed.` **exit=0** (avisa, **no** bloquea).
  - **sin binario** (`PATH` sin `~/.cargo/bin`): `probe: (no vanta-cli on PATH)` y **ni una línea** del check → `barrier passed. Push allowed.` **exit=0**.
  - **`CI=1`** con el binario stale presente → igual de silencioso, `barrier passed. Push allowed.` **exit=0**.
- **Matriz del script, default vs `-Advisory` (medida con binarios falsos `.cmd` + el real):** fresco 0/0 *(sin salida)* · stale **1**/0 · crash (`exit /b 7`) **2**/0 · ausente **2**/0 *(sin salida)*. Los códigos del modo default son exactamente los de `FIND-98`: el switch no cambia el contrato existente, solo su consumo.
- **Verificación:** `bash -n .githooks/pre-push` → 0 · `pwsh scripts/check-cli-fresh.ps1` (default, binario stale) → **exit 1** (el contrato `FIND-98` sigue midiendo) · las cuatro corridas del hook llegan a `barrier passed` · `check-links` 0 · `check-docs` 0 · **`gen-index --check` → 1, ROJO esperado y no regenerado acá**: `docs/index.md` cuenta 713/261 contra 715/263 (los dos task files nuevos: éste y `FIND-769.md`); `--write` lo corre el orquestador porque ese índice es compartido con el carril de `FIND-769`, y `docs-gates.yml` tiene `docs-index` como check *required* — sin ese paso el push aterriza en rojo.
- **Contrato de la fila cumplido:** `rg -n 'check-cli-fresh' .github/ .githooks/ dev-tools/justfile justfile` → 1 call site (`.githooks/pre-push`), advisory y con el fix en la salida.
- **Review P2-01:** `vanta-review`, ronda 1, **CHANGES-REQUIRED** (0 critical · 1 high · 2 medium · 3 low · 1 nit), todo re-verificado en vivo por el revisor (matriz × 2 modos, 4 caras del hook, `core.hooksPath`, auditoría de los caminos de bloqueo con la ruta que tiene espacio). El **código pasó**; los hallazgos eran del borde del changeset y de la higiene del advisory: (1) [high] `gen-index --check` → **1** mientras el DoD decía «0» → se **mide y se nombra como rojo esperado**, con el motivo; (2) [medium] `Backlog.md` no era `-1` sino `-7` (6 líneas de `FIND-769` ya borradas por su carril en el working tree) → el commit se construye como `HEAD` − sólo mi fila y la supresión ajena queda sin stagear; (3) [medium] el advisory hablaba justo en el caso sin hallazgo (banner `ADVERTENCIA` de árbol sucio en cada push) → el camino fresco es **silencio** en `-Advisory`; (4) [low] «ALWAYS exit 0» era falso ante un error terminante (`git` fuera del `PATH`) → el comentario lo dice y el hook lo absorbe con `|| true`; (5) [low] transcripción recortada → transcripciones literales post-fix; (6) [low] `status: completed` con §Review vacío → este bullet; (7) nits: `Exit-Check 0` final, `|| true` en la tubería de `sed` (EPIPE no puede bloquear), bullet de `RULES.md` movido a su sitio. Sin ronda 2: el revisor dejó por escrito que con los dos cambios requeridos aplicados pasa a APPROVE sin re-revisión de código.
- **Commit:** este commit (cierre de `FIND-754`, local; sin push — política del owner, Regla 7).
- **Deuda:** el job de `nightly` (YAML propuesto: `cargo install --path . --bin vanta-cli` + el check en modo default, donde sí debe fallar) no se aplicó porque `.github/workflows/**` es de otro carril. Mientras no exista, la señal depende de que el carril tenga el binario: es el techo declarado del advisory. La ronda P2-01 sugirió además registrarlo como fila `FIND-*` para que no envejezca como deuda invisible: **no se creó** (este carril toca el Backlog sólo para suprimir su propia fila) — queda para el orquestador.

### FIND-778: el helper de commit valida el resultado y no deja `HEAD` en un commit vacío
- **Fecha:** 2026-11-26
- **Objetivo:** dos commits (`72643d9df`, `290df23d0`) con el árbol vacío canónico `4b825dc…` borraron los **2365** paths mientras `git-commit-isolated.ps1` imprimía `git commit failed (exit 1)`. El worktree quedó intacto y la historia se reparó con `commit-tree` + `update-ref`, pero el defecto de diseño era tratar el exit code como resultado: el `throw` del `catch` saltaba la verificación por contenido, y "el comando falló" se leyó como "no se creó nada". El mecanismo no se reprodujo.
- **Resultado:** la validación salió del `try/catch` y corre igual en éxito y en error: árbol vacío; ≥50 paths borrados que **siguen existiendo en el worktree**; pérdida de más de 200 entradas respecto de `HEAD` (el mayor descenso legítimo medido en 300 commits es **43**, `960e338d0`; la catástrofe fue 2365); o un path sembrado que no aparece **byte a byte** en el commit. Cualquiera de las cuatro restaura la ref con `git update-ref HEAD <old> <new>` (compare-and-swap: si otro carril movió `HEAD`, se niega y sale 5), imprime el SHA del commit malo y **lo conserva** (nunca `--amend` ni borrado), y sale 4. La identidad se prueba por **contenido** (árbol sembrado o blobs de los paths nombrados), nunca por el mensaje: un commit que no es de este carril **no se revierte** y sale 6, y con `git commit` ≠ 0 el script **nunca imprime `OK`** si el commit no es demostrablemente suyo. Excepción: el árbol vacío se restaura **siempre**. El fallo del comando ya no salta la verificación, y un `git commit` que sale 0 sin mover `HEAD` también es una violación.
- **Índice privado:** se aborta si no existe, está vacío o lista 0 ficheros antes del commit (un índice vacío produce exactamente el commit vacío); la limpieza vive en el `finally` y reintenta 10×150 ms porque en Windows un handle abierto hace fallar un `Remove-Item` único **en silencio** — la explicación más probable de los 39 `idx-*` de ~250 KB en `%TEMP%\opencode`. El nombre del índice ahora lleva el PID.
- **Decisión colateral (declarada):** el refresco del índice compartido pasó de `git add` a `git reset -q -- <paths>`: con un borrado, `git add` no puede expresarlo (`exit 128`) y el helper salía 3 tras un `git rm` legítimo, dejando el índice compartido desincronizado. Lo encontró el escenario 5 del test, no una suposición.
- **Test:** `dev-tools/tests/git-commit-isolated.tests.ps1`, **37/37 PASS**, siete escenarios en repos desechables con `TEMP` redirigido (el checkout real no se toca): (1) éxito con árbol vacío fabricado por hook → exit 4 + restauración; (2) **fallo con `HEAD` movido** → exit 4 + restauración + SHA conservado; (3) hook aborta → exit 2; (4) camino feliz → exit 0; (5) `git rm` → exit 0; (6) **commit válido de otro carril** → exit 6, sin `OK` y sin revertirlo; (7) árbol vacío de otro carril → exit 4, restaurado igualmente.
- **Verificación:** repositorio real tras el commit de cierre: `HEAD^{tree}` ≠ `4b825dc…` y `git ls-tree -r --name-only HEAD | Measure-Object -Line` → **2368** (2365 de la línea base + `FIND-757.md` de otro carril + los dos ficheros nuevos de este cierre), sin borrados.
- **Review P2-01:** `vanta-review`, una ronda, **CHANGES-REQUIRED** → cerrada. Dos bloqueantes reales: **C1** el camino de éxito imprimía `OK` y salía 0 cuando el `HEAD` era un commit **válido de otro carril** (regresión frente al `exit 2` antiguo; la guarda 4 comparaba presencia, no contenido) → ahora compara blobs y la identidad gobierna el éxito, con escenario 6; **H1** el árbol vacío se quedaba en `HEAD` cuando el commit era ajeno → ahora se restaura siempre, con escenario 7. Medios: identidad por asunto eliminada (colisiona con mensajes convencionales), techo de 200 entradas declarado; bajos: fallos de medición ahora fallan cerrado y cifras/citas corregidas. Detalle y tabla en `docs/dev/tasks/FIND-778.md` §Review.
- **Commit:** este commit (cierre de `FIND-778`, local; sin push — Regla 7).
- **Deuda:** el mecanismo original sigue sin reproducirse: las guardas fallan cerrado sobre el resultado, no sobre la causa. Si el índice privado desaparece entre `write-tree` y `commit`, el commit vacío se crea; lo que ya no puede pasar es que quede en `HEAD` o que se reporte como éxito. Un borrado legítimo de más de 200 ficheros exige `git commit` normal. Los 39 `idx-*` históricos no se borran retroactivamente.

### El commit se crea con plumbing: matar el proceso ya no puede dejar un árbol vacío (`FIND-780`)
- **Fecha:** 2026-11-26
- **Objetivo:** las guardas de `FIND-778` eran correctas pero estaban **después** del commit y **dentro** del proceso; un pipe cortado (`... | Select-Object -First N`) desenrolla el script, el `finally` borra el índice privado, y el **hijo** `git commit` —que en Windows sobrevive a su padre— lee un índice vacío y commitea `4b825dc…`. Las guardas no corren porque el proceso que las corre ya no existe. **La causa era mía: pipeé el helper.** Tres incidentes: `72643d9df`, `290df23d0`, `5766654ee`.
- **Resultado:** ✅ El commit se crea con **plumbing**: `git write-tree` (después de los hooks, como hace git) → `git commit-tree <tree> -p HEAD -m <msg>` → `git update-ref -m <msg> HEAD <new> <old>` (compare-and-swap). `commit-tree` recibe el árbol como **argumento**: desde ahí ningún paso lee el índice, así que el `finally` que lo borra (que se queda: nada de residuo) ya no puede afectar al commit y un kill deja `HEAD` en el SHA anterior **o** en el commit completo. El árbol vacío ya no se detecta y deshace: **no se crea** (el índice vaciado por un hook se rechaza *antes* de `commit-tree`, exit 4, y el colapso de entradas usa el mismo umbral de `FIND-778`). Las cuatro guardas de `FIND-778` siguen como red para cuando **el hook** mueve `HEAD` por su cuenta — y corren **también cuando el hook pasa el gate**: los rechazos previos se reportan a través del bloque de validación, no con `exit` dentro del `try` (ronda 1 del review).
- **El gate no se pierde (la decisión):** se corre el hook **real** con `git hook run pre-commit`, con `GIT_INDEX_FILE` ya apuntando al índice privado, así que su `git diff --staged` ve exactamente los paths del commit. Se descartó `pre-commit run --config … --files` porque el shim aporta **cinco gates más** que el YAML — `DOC-04` (junk/binarios), guard de `unmerged`, guard de config sin stagear, `verify_staged.ps1` y `actionlint` — y reimplementarlos sería una copia que se desincroniza. Trade-off declarado: `prepare-commit-msg`/`commit-msg`/`post-commit` no corren (no existen hoy en `.githooks`; el helper **avisa si aparecen**, que es lo único que puede hacer un aviso), y la ventana del CAS dura lo que los hooks: si otro carril commitea en medio, `update-ref` falla. **Eso no es una garantía nueva** —git 2.51 ya se niega dentro de `git commit` (`fatal: cannot lock ref 'HEAD': is at … but expected …`, medido con el helper anterior, que también salía 6)—; ahora es explícita, no depende de un detalle interno de git y el mensaje distingue las dos causas (otro carril movió `HEAD` vs. ref bloqueado). Escenario 9.
- **Test:** `dev-tools/tests/git-commit-isolated.tests.ps1`, **82/82 PASS**, quince escenarios. Los ocho nuevos: (8) el gate corre y su `git diff --staged` resuelve al índice privado (probado por el **efecto del propio hook**, no por nuestro log); (9) `HEAD` se mueve durante los hooks → exit 6, commit ajeno intacto, mi edición en el worktree; (10) **kill del proceso que commitea** (`Stop-Process -Force` sobre el helper con el hook en vuelo) → `HEAD` quieto, árbol ≠ vacío, **cero** commits creados; (11) **estático**: el script no contiene ninguna invocación `& git commit` (cierra el híbrido que el kill no ve: `git hook run` puesto *y* `git commit` de vuelta); (12) y (13) **un hook que PASA el gate y mueve `HEAD` a un commit de árbol vacío** → exit 4 y `HEAD` restaurado; (14) los hooks que `git commit` correría y este script no (`prepare-commit-msg`/`commit-msg`/`post-commit`) se **nombran** si existen; (15) un lock tomado en `refs/heads/<rama>` → exit 2 y mensaje "could not lock HEAD" (no "otro carril"). El 10 **falla contra el script anterior**: `[master 711a687] killtest / 60 files changed, 60 deletions(-)` y `HEAD` al árbol vacío (5 FAIL solo en ese escenario, **13 en la corrida**). Reproducible: `$env:FIND780_HELPER=<script viejo>`.
- **Regla `RULES.md` §11:** el `Must not` de pipear **no se borró**: pasó a **"(historia, FIND-780)"** con el mecanismo y los tres SHAs (sigue siendo por qué el helper tiene esta forma, y capturar en variable sigue siendo la costumbre). Lo que cambió es **qué protege**: antes la prosa era la única barrera, ahora hay un `Must` de forma (`write-tree` + `commit-tree` + `update-ref` con gate explícito) y un `Must not` nuevo (volver a `git commit` o no correr el gate). El bloque de ejemplo manual, que enseñaba `git commit -m` sobre índice privado, lleva nota apuntando al plumbing.
- **Verificación:** suite **82/82** exit 0 · contra el script anterior **13 FAILED** (5 en el kill, más los mensajes nuevos y el guard estático; los escenarios 12/13 **pasan** contra el anterior, o sea que el HIGH-1 del review fue una regresión de la primera versión de este fix, no una carencia previa) · commit real con el helper, con el `pre-commit` a la vista (`a31f8eb8a`, primera versión; la ronda 1 del review encontró un bug en ella y este cierre añade el fix) · `HEAD^{tree}` ≠ `4b825dc…`; `git ls-tree -r --name-only HEAD | Measure-Object -Line` → **2374** (2373 al abrir + `docs/dev/tasks/FIND-780.md`; el helper y su test ya estaban trackeados) y el **único** borrado del primer commit es la fila propia del Backlog (`git diff --numstat HEAD~1..HEAD` → `0 1 docs/dev/Backlog.md`).
- **Review P2-01:** `vanta-review`, **dos rondas**. Ronda 1 **REQUEST CHANGES** sobre `a31f8eb8a` (0 críticos, 1 alto, 3 medios, 3 bajos, nits); ronda 2 **APPROVE** con los 10 hallazgos CLOSED, más dos LOW corregidos en este mismo commit. **HIGH-1 (el importante, y era mío):** los dos rechazos nuevos hacían `exit` **dentro** del `try`, así que el bloque de validación de `FIND-778` no corría; con un hook que **pasa** el gate y mueve `HEAD` a un commit de árbol vacío, el helper imprimía "HEAD did not move" y dejaba el árbol vacío en `HEAD` — la regresión exacta que esta tarea existe para impedir. Reproducido por el revisor y por mí (`exit 4`, `EMPTY ON HEAD = True`), corregido llevando el rechazo a `$preCommitRefusal` + `throw` para que decida el bloque, con escenarios 12/13. **M1:** la afirmación "antes revertía con exit 0" era **falsa** (git 2.51 ya hace su propio CAS; el helper anterior también salía 6) → corregida a "misma garantía, ahora explícita". **M2:** el "tripwire" del aviso sólo miraba `pre-commit`, así que no podía detectar el día que se añadiera otro hook de commit → ahora los busca y avisa (escenario 14). **M3:** `status: completed` con el review vacío → `in-review` hasta cerrar. **L1:** escenario 11 estático para el híbrido (el revisor lo hizo morder: 1 FAIL exacto). **L3:** `FIND780_HELPER` resuelto. Nits: diagnóstico falso de `update-ref` (ahora distingue ref bloqueado de `HEAD` movido, escenario 15) y `commit-tree` ≠ `git commit` (declarado). **LOW-A/B de la ronda 2:** una cifra vieja corregida y el mensaje "HEAD did not move" ya no se imprime cuando un hook borra la ref.
- **Commit:** este commit (cierre de `FIND-780`, local; sin push — Regla 7).
- **Deuda:** `prepare-commit-msg`/`commit-msg`/`post-commit` no corren (no existen hoy; el aviso los detecta y añadir uno obliga a tocar el helper). Un `Stop-Process -Force` no ejecuta el `finally`, así que puede dejar un `idx-*` en `%TEMP%\opencode` — igual que antes y no arreglable desde un proceso muerto; y un kill *dentro* de `update-ref` puede dejar un `.git/HEAD.lock` (la ref queda en el SHA viejo: sin pérdida, pero el siguiente `update-ref` falla hasta borrarlo). El "nothing to commit" se decide **antes** de los hooks, así que un hook que stagee ficheros no puede completar un commit que el helper se niega a empezar (el shim actual nunca escribe el índice). `--amend` sigue fuera de alcance; la salida de emergencia ya no es `--no-verify` sino `SKIP=<id>`, que el hook honra.

### La autoría del commit tiene una sola definición, y un detector que la vigila (`FIND-776`)
- **Fecha:** 2026-11-26
- **Objetivo:** la política de commit estaba definida de **cuatro** formas distintas en los ficheros que lee un agente (`.opencode/AGENTS.md:313` y `:641`, `agents/vanta-worker.md:4-7`, `prompts/pipeline-full.md:172`), y el `shell: allow` hacía que **el propio agente decidiera** si cumplía. El agente lee el documento que cargó, así que el comportamiento dependía de por dónde entró.
- **Resultado:** ✅ Definición canónica en `docs/dev/workflow/RULES.md` §11 (el runbook del helper, con su techo declarado) y el resto **cita** en vez de redefinir. `vanta-lead.md:20` y `:16` alineadas: *"git mutating **de publicación** (push/tags/release-plz/publish); commit local de cierre = **ejecutor**"*. Y el contrato que faltaba: check **`commit-authority`** en `dev-tools/check-harness-contract.mjs`, que exige el statement canónico y **falla ante 7 formas contradictorias** — probado re-introduciendo la línea vieja (`caught=1`) y con el árbol limpio (`clean=0`).
- **Verificación:** `node dev-tools/check-harness-contract.mjs` → **7 checks, exit 0** · sonda `P12` del carril del harness: contradicción inyectada → `FAIL commit-authority` · `rg 'prepara\+ejecuta commit|git mutating'` sobre `vanta-lead.md` → la forma vieja **ya no existe**.
- **Commit:** `.opencode/` `b7f97f6` (ronda 3).
- **Nota:** la cerré yo porque el carril del harness tenía `Backlog.md` prohibido; el detector lo verificó él.

### El `actionlint` del `pre-commit` llevaba toda la sesión sin lintar nada (`FIND-784`)
- **Fecha:** 2026-11-26
- **Objetivo:** el hook decía `actionlint ok` en **cada commit**, así que nadie lo miró. Lo destapó un bloqueo real: el WIP de `FIND-766` dejó `.github/workflows/certification.yml` inválido y el hook falló.
- **Resultado:** ✅ La invocación **no podía funcionar**: `actionlint` **sin argumentos lee `stdin`**, con un **directorio** devuelve `Incorrect function` en Windows, y sólo acepta **ficheros**; y `2>/dev/null` tiraba el motivo, así que el mensaje era `actionlint FAILED` a secas. Arreglado a `actionlint .github/workflows/*.yml` **sin ocultar stderr**.
- **Verificación (el contraste que lo prueba):** la versión de `HEAD` de `certification.yml` daba `exit 0` y la del WIP `exit -1`; con el hook arreglado el gate **mide** y el fichero corregido vuelve a `exit 0`. `bash -n .githooks/pre-commit` → 0. Los 20 workflows: `exit 0`.
- **Commit:** `cc93e5d79`
- **Nota:** el gate de CI (`docs-gates`) **sí** linteaba, así que el repo no estaba ciego — pero **local y CI no medían lo mismo** (clase `FIND-762`). Es la séptima variante del patrón del día y la peor situada: en el hook que da por bueno cada commit.
