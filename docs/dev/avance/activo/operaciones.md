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
- **Objetivo:** el checklist piloto (doc de venta enterprise) tenía un paso clone a la org `vantadb` que no existe (FIND-17/ADR-030: owner real `ness-e/*`).
- **Resultado:** ✅ ambas URLs verificadas live 404 (`github.com/vantadb/vantadb-examples` y `github.com/ness-e/vantadb-examples`); ningún repo existe en ninguna org → rama TODO-humano del contrato (no crear repos desde el agente). `docs/dev/operations/pilot-onboarding-checklist.md:51` ahora TODO explícito en inglés; `rg "vantadb/vantadb-examples" docs/user/operations/ docs/api/` = 0. Fila Backlog eliminada (con ella muere la cita a la ruta vieja `pilot-onboarding-checklist.md` — la canónica es `docs/user/operations/...`).
- **Archivos tocados:** `docs/dev/operations/pilot-onboarding-checklist.md`, `docs/dev/Backlog.md` (fila removida), `docs/dev/plans/2026-09-03-quality-gtm-wave.md` (Task 3 → ✅)
- **Deuda abierta:** crear el repo `vantadb-examples` (o apuntar a ejemplos reales) es acción humana externa.

### MKT-18i: Compose demo multi-servicio VantaDB + Ollama (re-escalado: sin AnythingLLM)
- **Fecha:** 2026-09-03
- **Objetivo:** demo copypaste local-first (r/LocalLLaMA): `docker compose up -d` orquesta VantaDB + Ollama + AnythingLLM.
- **Resultado:** ✅ compose raíz con 2 servicios, tags explícitos: `ollama/ollama:0.33.2` (pin verificado vía Docker Hub API: digest de `latest` == `0.33.2`, multi-arch) + `build: .` con `image: vantadb/server:0.5.0` (== workspace/Dockerfile APP_VERSION). Cabecera: quickstart, RAM mínima ~4 GB, nota GPU CPU-default (docs.ollama.com/docker), volumen de modelos persistente. **Re-escalado por stop condition:** AnythingLLM NO soporta VantaDB como vector backend — evidencia: `server/.env.example` master (github.com/Mintplex-Labs/anything-llm): `VECTOR_DB` ∈ lancedb/chroma/chromacloud/pinecone/astra/pgvector/weaviate/qdrant/milvus/zilliz; sin glue inventado. Enlaces: `docs/user/operations/DEPLOYMENT_GUIDE.md` §3 + `docs/user/tutorials/02-local-rag-pipeline.md` (README no tiene bloque docker — verificado).
- **Contrato verificado:** `rg -ci "ollama|anythingllm|anything-llm" docker-compose.yml` = 14 (≥2 ✅); sin docker CLI en host → `docker compose config -q` reemplazado por parse PyYAML equivalente OK + assert de tags explícitos (sin `:latest`); run-time `up -d` diferido (sin daemon). `docker-compose.dev.yml` intacto.
- **Archivos tocados:** `docker-compose.yml`, `docs/user/operations/DEPLOYMENT_GUIDE.md`, `docs/user/tutorials/02-local-rag-pipeline.md`, `docs/dev/Backlog.md` (fila re-escalada), plan Task 6 → ✅.
- **Commit:** `abb6594c` (develop).
- **Deuda abierta:** upstream feature request a AnythingLLM (backend VantaDB) = acción humana; run-time verificación `up -d` pendiente en host con daemon; si SRV-07 publica imagen oficial, swap `build: .` → `image:` publicada (nota inline en el compose).


### SRV-07: Dockerfile unprivileged + wiring release (quality-gtm wave1)
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
- **Contrato verificado:** `Test-Path vantadb-server/Dockerfile`=False ✅; `rg "COPY vantadb/" vantadb-server/`=0 ✅; `rg -l "vantadb-server/Dockerfile"` ex-archives = solo Backlog (fila eliminada en este cierre) + esta deuda histórica SRV-07 (registro fechado, exenta) ✅; PyYAML `safe_load` OK ×2 con `build={context: .., dockerfile: Dockerfile}` ✅; `rg "release-binary|target:"` en composes=0 ✅; `rg "vantadb-server/Dockerfile|--target"` en hardening+GUIDE=0 ✅. Sin daemon docker en host → build/smoke diferidos al job `docker-image` de CI (precedente SRV-07/MKT-18i, NUNCA fakes). actionlint N/A (0 workflows), cargo fmt/clippy/nextest N/A (0 Rust).
- **Archivos tocados:** `vantadb-server/Dockerfile` (baja trackeada ya registrada en `0a54a545`, 112 deletions; la copia del worktree — resurrección no-trackeada — eliminada en esta sesión vía `git rm`; estado final consistente worktree+index+HEAD), `vantadb-server/docker-compose.yml` (2 builds → raíz, hardening runtime intacto), `vantadb-server/docker-compose.prod.yml` (build → raíz, sin `target release-binary`/`args` muertos; env prod + resources intactos), `docs/user/operations/hardening.md` (§5 reescrito sobre imagen canónica + nota de deprecación; tabla comparativa corregida UID-mecanismo), `docs/user/operations/DEPLOYMENT_GUIDE.md` (1 línea: puntero vivo al archivo eliminado), `docs/dev/Backlog.md` (fila FIND-56 eliminada).
- **Deuda abierta:** verificación end-to-end en CI al primer dispatch/tag (job `docker-image`: build raíz + smoke `--user 10001:10001` + export como asset).

- **Origen:** Backlog FIND-56 (L219) + deuda SRV-07 (2) de esta misma página.

### GOV-TK3 (docs): drift yaml-real x3 - Resultado: doc-fix x3 (codigo verificado correcto); parity 5/5, parser 117/117, docs-coverage 0 gaps. Commit b3be4176 (2026-09-05).

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
- **Objetivo:** DISCOVERY arch + MVP auth (OIDC discovery DEFER vía ADR-039).
- **Resultado:** ✅ server 58/58 + auth 3/3 + rotation/rbac 13/13 + server 42/42 + clippy/fmt 0; race git-add paralela revertida.
- **Commit:** a0a3087f (+ADR-039)

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
