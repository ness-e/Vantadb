---
title: WIRE-04 — TTL superficie completa (server HTTP + default por colección + sweeper)
kind: task
description: TTL HTTP verificado con test E2E (put ttlms → expira → get None / purgeexpired ≥1) Y default TTL por
---

# WIRE-04 — TTL superficie completa (server HTTP + default por colección + sweeper)

> **Fase:** F2 · **Plan:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 18) · **Branch:** develop
> **Ruta:** vanta-worker · **Appetite:** max 2d · **Estado:** ⏳ IN PROGRESS
> **Commit esperado (LEAD):** `feat(ttl): collection default + background sweeper (WIRE-04)`

## Contrato (verbatim — ley)

"TTL HTTP verificado con test E2E (put `ttl_ms` → expira → `get` None / `purge_expired` ≥1) Y default TTL por
colección implementado (config + fallback en put, solo writes nuevos) y documentado en `HTTP_API.md` Y sweeper
background invocado que purga memoria/índices con test de expiración física (nodo ausente del índice, no solo
filtrado en lectura)"

## SDP (Paso 0b)

SDP: `campaign-executor` · `progreso` · `documentation-and-adrs` (pinned) · `api-and-interface-design` (pinned) ·
`incremental-implementation` · `test-driven-development` · `source-driven-development` · `rust-write-tests` ·
`observability-and-instrumentation`

## Verificación de realidad (ajustada vs seed)

- ✅ El param HTTP **ya existe**: `records_put`/`records_put_batch` toman `MemoryInput` (`src/server/handlers.rs:248,259`)
  que incluye `ttl_ms` (`src/sdk/types/record.rs:74-77`); `put_one` lo resuelve a `expires_at_ms` (`src/sdk/api/memory.rs:88`);
  `get` filtra expirados vía lazy eviction (`record_from_node`, `src/sdk/serialization/mod.rs:363`).
- ✅ `purge_expired` existe y purga nodo + derived indexes + text index + version history (`src/sdk/api/memory.rs:587-698`),
  expuesto en `DELETE /api/v2/maintenance/expired-records` (`src/server/router.rs:196-199`, `handlers.rs:1149`).
- 🆕 Gaps reales: (1) NO existe default TTL por colección/namespace (`rg default_ttl|collection_ttl|ttl_default` = 0 hits);
  (2) NO existe sweeper background — `purge_expired` es manual y `GcWorker::sweep` (`src/gc.rs:34`) solo se registra para
  threads (`src/agentic/thread.rs:205,252`) sin spawn real (comentario `gc.rs:33` lo promete).
- ⚠️ Dep MGR-09 (Backlog:815) PENDIENTE — informa la parte semántica default/decay; **este slice no la requiere**.
- Fuera de scope (WIP ajeno): `setup-embeddings.ps1`, `src/cli_handlers/server.rs`, `src/lib.rs`, `src/embedding_health.rs`,
  `vantadb-mcp/src/handlers/tools.rs`, `tests/embedding*`, `perf-bench.yml`, bindings Py/TS/Node, `vantadb-ts/` (WIRE-03).

## Impacto mapeado (Regla 0)

**Archivos leídos completos:** `src/gc.rs` · `src/server/state.rs` · `src/server/bootstrap.rs` · `src/server/router.rs`
(150-329) · `src/sdk/types/record.rs` · `src/sdk/api/memory.rs` (37-348, 560-719) · `vantadb-server/tests/helpers/mod.rs` ·
`vantadb-server/tests/server.rs` (1-200) · `docs/api/HTTP_API.md` (215-259, 430-459) · `.opencode/rules/{server-mcp,api-contract}.md`.

**Referencias hacia dentro (quién depende de lo que toco):**

| Símbolo | Referencias entrantes | Efecto |
|---|---|---|
| `Config` (campo nuevo) | `Embedded.config` (`builder.rs:17`, copiado de `engine.config` vía `from_engine`) — server usa `StorageEngine::open_with_config(Some(config))` → el config del server llega al SDK | Añadir campos es aditivo (`..Default::default()` en todos los constructores externos verificados) |
| `MemoryInput` | HTTP `handlers.rs:248/259` · SDK `put/put_batch` · MCP · Python/Node/WASM (serialización) | NO se modifica la struct — el default se resuelve en `put_one`/`put_batch_inner` |
| `put_one` | 1 caller `put` (`memory.rs:161`); `put_batch_inner` es path paralelo (`memory.rs:197`) | Ambos paths deben aplicar el default |
| `spawn_memory_ttl_sweeper` (nuevo, `gc.rs`, `#[cfg(feature="server")]`) | 1 caller nuevo: `server/bootstrap.rs::run` | Sin cambios en `ServerState` (handle vive en scope de `run`) |
| `purge_expired` | `maintenance_purge` (HTTP) · `vantadb-node` · `vantadb-wasm` · tests | REUSO — no se modifica |
| `GcWorker` | `agentic/thread.rs` (threads) | NO se toca; solo se corrige el comentario stale `gc.rs:33` |

**Blast radius:** 8 archivos tocados + 1 test file nuevo (`vantadb-server/tests/ttl_e2e.rs`). 0 breaking: campos nuevos de
`Config` con default, función nueva feature-gated, paths existentes intactos.

**Veredicto:** verde — aditivo, sin cambiar semántica de reads/writes existentes (default solo aplica a writes nuevos sin `ttl_ms`).

## Steps atómicos

| # | Step | Archivos | Verificación | Estado |
|---|------|----------|--------------|--------|
| 1 | **E2E HTTP explícito**: test `ttl_ms` → 201 con `expires_at_ms` → expira → `GET` 404 + nodo aún físico + `purge_expired` ≥1 + nodo ausente | `src/server/ttl_tests.rs` (nuevo, lib test target) + wiring en `src/server/routing.rs` | `cargo nextest -p vantadb --lib --features server -E 'test(ttl_http_explicit)'` ✅ | ✅ |
| 2 | **RED default TTL por namespace**: 5 tests SDK (aplica; otro ns no; `ttl_ms` explícito gana; batch; no-backfill tras reopen) | `src/sdk/api.rs` | RED: 4 fallos por razón correcta (`put_batch must apply…`, `writes after the config change must inherit…`) ✅ | ✅ |
| 3 | **GREEN default TTL**: `Config.memory_default_ttl_ms` (+env `VANTADB_MEMORY_DEFAULT_TTL_MS`) + `effective_ttl_ms` en `put_one`/`put_batch_inner` | `src/config.rs`, `src/sdk/api/memory.rs` | `cargo nextest -p vantadb --lib -E 'test(default_ttl)\|test(put_batch_applies)'` → 4/4 ✅ | ✅ |
| 4 | **RED sweeper**: 2 tests físicos + shutdown/join (feature server) | `src/gc.rs` | RED: E0425 `spawn_memory_ttl_sweeper` not found ✅ | ✅ |
| 5 | **GREEN sweeper**: `spawn_memory_ttl_sweeper` + `MemoryTtlSweeper` (Drop cancela, `shutdown()` join) + comentario stale `GcWorker::sweep` corregido | `src/gc.rs` | `cargo nextest -p vantadb --lib --features server -E 'test(ttl_sweeper)'` → 2/2 ✅ | ✅ |
| 6 | **Config intervalo + spawn en producción**: `Config.ttl_sweep_interval_ms` (default 60s, `0` desactiva) + spawn en `bootstrap::run` (skip read-only) + shutdown tras el run loop | `src/config.rs`, `src/server/bootstrap.rs` | `cargo clippy -p vantadb --lib --features server -- -D warnings` ✅ + E2E sweeper HTTP ✅ | ✅ |
| 7 | **E2E HTTP default + sweeper**: 2 tests HTTP (default por namespace; purga física sin `purge_expired` manual) | `src/server/ttl_tests.rs` | `cargo nextest -p vantadb --lib --features server -E 'test(ttl)'` → 21/21 ✅ | ✅ |
| 8 | **Docs**: `HTTP_API.md` (Records + Namespace default TTL + Maintenance), `CONFIGURATION.md` (2 campos nuevos) y `openapi.yaml` (`ttl_ms.description`) | `docs/api/HTTP_API.md`, `docs/user/operations/CONFIGURATION.md`, `docs/api/openapi.yaml` | `pwsh scripts/validate-docs-coverage.ps1` → **0 gaps** ✅ + `nextest -p vantadb --test openapi_yaml_parity` → 11/11 ✅ | ✅ |
| 9 | **Verify full + snapshot**: gate (`verify.ps1`) + snapshot public-api quirúrgico (solo mis 4 líneas) | `tests/api/public-api.txt` | ver §Verificación | ✅ |

## Verificación (evidencia)

| Check | Comando | Resultado |
|---|---|---|
| fmt | `cargo fmt --all -- --check` (vía `verify.ps1`) | ✅ ok |
| check | `cargo check -p vantadb --no-default-features --features cli,fjall,memmap2,fs2,roaring` | ✅ ok |
| clippy | `cargo clippy -p vantadb … -- -D warnings` (gate) + `cargo clippy -p vantadb --lib --features server -- -D warnings` | ✅ ok |
| audit / deny | `cargo audit` · `cargo deny check` | ✅ ok |
| nextest (gate features) | `cargo nextest run --profile audit -p vantadb --no-default-features --features cli,fjall,memmap2,fs2,roaring -E 'test(/ttl|gc_|purge/)'` | ✅ 22/22 |
| nextest (server) | `cargo nextest run --profile audit -p vantadb --lib --features server -E 'test(ttl)'` | ✅ 21/21 |
| nextest (gate completo) | `cargo nextest run --profile audit -p vantadb … -E <fastGateFilter>` (vía `verify.ps1`) | ✅ post-fix: snapshot regenerado (`VANTADB_PUBLIC_API_UPDATE=1`; incluye `embedding_health` — COMMITEADO en `f73b9c9b`, no WIP — + los 4 campos TTL) + `cargo nextest -p vantadb --test public_api` verde. Fallo previo = snapshot stale contra código commiteado + timeout frío (FIND-178, atribución corregida por el revisor) |
| docs-coverage | `pwsh scripts/validate-docs-coverage.ps1` | ✅ 0 gaps |
| cli-probes / consumo guard / backup doc | `cargo run -p vantadb --bin vanta-cli -- --help` · `cargo bench -p vantadb --bench canonical_p99 --no-run` · doc anchor | ✅ ok |

**Contrato — evidencia por cláusula:**
1. *TTL HTTP E2E*: `ttl_http_explicit_ttl_expires_then_purges_physically` (404 lazy + nodo presente + `purge_expired` ≥1 + ausente) ✅
2. *Default TTL por colección (config + fallback, solo writes nuevos, documentado)*: `default_ttl_applies_when_input_omits_ttl`, `default_ttl_ignores_namespaces_without_configuration`, `explicit_ttl_overrides_namespace_default`, `put_batch_applies_namespace_default_ttl`, `default_ttl_is_not_backfilled_on_reopen` + `ttl_http_namespace_default_applies_to_new_writes`; docs `HTTP_API.md` §Namespace default TTL + `CONFIGURATION.md` ✅
3. *Sweeper background invocado + expiración física*: `ttl_sweeper_physically_removes_expired_records_and_indexes` (nodo + scalar index ausentes), `ttl_sweeper_shutdown_joins_and_stops_further_sweeps`, `ttl_http_sweeper_physically_purges_without_manual_call` (sin `purge_expired` manual; remanente = 0); invocación real en `bootstrap::run` ✅

## Context Save Point

- **Última acción (2026-09-27):** steps 1–9 ✅. Implementado default TTL por namespace + sweeper + E2E HTTP + docs + snapshot public-api (quirúrgico). Ver §Verificación.
- **Pendiente (LEAD, NO worker):** review fresco P2-01 ✅ APPROVE-con-fixes (`ses_f1abbc40cffeXcFqdqDgM80Y60`) → fixes R1 (GC_TTL.md) + R2 (atribución public_api: snapshot stale vs `embedding_health` COMMITEADO `f73b9c9b`, no WIP) aplicados + snapshot regenerado + `public_api` verde → commit local. El gate local no puede compilar `-p vantadb-server` mientras sesiones MCP vivas sostienen el bin (FIND-177).
- **Decisiones de diseño:** (a) default = mapa namespace→ms en `Config` (env `namespace:ms,...`), fallback exclusivo en put (writes nuevos, sin backfill, `ttl_ms` explícito gana); (b) sweeper = loop tokio en `gc.rs` que reutiliza `Embedded::purge_expired` (purga física de nodos + índices derivados/text) sobre `spawn_blocking`, cancelación por `watch` + `Drop`, join por `shutdown()`; (c) `GcWorker` (threads) intacto — solo se corrigió su comentario stale; (d) cero cambios en `ServerState`/`MemoryInput`; (e) E2E HTTP movido al lib test target por FIND-177.

## Notas

- `GcWorker::sweep` sigue siendo caller-driven (threads); el comentario que prometía un `tokio::spawn` inexistente quedó corregido y apunta al sweeper real.
- El sweeper corre su primer tick de inmediato (semántica de `tokio::time::interval`): limpia registros que expiraron mientras el proceso estaba caído.
- Observabilidad: `tracing::info!(purged)` por tick con purgas y `warn` en fallos (reintenta el próximo tick); métrica Prometheus dedicada no agregada (fuera de contrato; candidato a `vanta-tuner`).
- `MemoryInput` no se tocó: el default se resuelve en el SDK (HTTP/SDK/MCP/bindings heredan el comportamiento por el mismo `put`).
- **Dependencia de módulo (para el reviewer):** el sweeper vive en `gc.rs` (plan) y depende de `sdk::Embedded` — dirección adapter→caso-de-uso (Apéndice V.1: permitida); existe un ciclo potencial `sdk → agentic → gc → sdk` (el edge `sdk → agentic` ya existía en `builder.rs`), sin impacto en compilación/feature-gating. Alternativa si el reviewer lo exige: mover `spawn_memory_ttl_sweeper`+`MemoryTtlSweeper` a `src/sdk/api/memory.rs` (junto a `purge_expired`), preservando el gate `feature = "server"`.


## Pre-mortem / Stop conditions

- F1 sweeper sin cancelación → `watch` shutdown token + `Drop` cancela + test de `shutdown()`/join. ✅ mitigado en diseño.
- F2 default retroactivo → aplica solo en writes nuevos; test de reopen verifica no-backfill. ✅ mitigado.
- F3 purga que no limpia HNSW/text → se reutiliza `purge_expired` (ya purga derivados) + test post-purga con `namespace_stats`. ✅
- **Stop:** si el sweeper exige refactor de lifecycle (engine+tokio) >1d → entregar default+E2E y dejar sweep como FIND. Rabbit hole: GC distribuido → NO.
