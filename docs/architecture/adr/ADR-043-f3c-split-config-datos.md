---
title: "ADR-043: S-split-config B+B (datos para firma humana)"
type: adr
status: proposed
tags: [vantadb, architecture, adr]
created: 2026-09-14
last_reviewed: 2026-09-14
---

# ADR-043: S-split-config B+B (datos para firma humana)

> **Regla 5:** Contexto/Decisión/Consecuencias los escribe el HUMANO con sus
> palabras. Esta entrada aporta SOLO datos (implementación F3C-impl).
> Diseño canónico: `docs/tasks/F3C.md`. Plan: `docs/plans/2026-09-13-cleanCA-fase3.md`.

## Contexto

_(lo escribe el humano con sus palabras)_

## Decisión

_(la escribe el humano — Q1=B `RbacCfg` propio 6to dominio; Q2=B 12 ficheros
con lectura directa no consolidados → FIND-89; Q3=A fachada A fuente única +
C7 sin shims, B+B Q4 vigente)_

## Consecuencias

_(las escribe el humano; la IA solo aportó los datos de abajo)_

## Datos aportados por IA (referencia F3C-impl)

- Anclas intactas: `HotReloadConfig`, `from_config`, `apply_to` (8 `update!`),
  `pub struct Config` plano, `parse_env_or` (warn+default, nunca fail-fast),
  `watch_config` + `apply_hot_reload_from_value` confinados en `src/config.rs`.
- Dominios: `StorageCfg` (15 campos) / `ServerCfg` (17) / `LlmCfg` (4+1 cfg) /
  `EvictionCfg` (8) / `PoolCfg` (8) / `RbacCfg` propio (1) + alias compat
  `RbacConfig`; `Config` conserva `Default`, todos los `with_*`, acceso plano
  (`config.port`, `..Default::default()`), `HotReloadConfig` + watcher.
- Vistas sin duplicar estado: `Config::storage_cfg/server_cfg/llm_cfg/
  eviction_cfg/pool_cfg/rbac_cfg` (`From<&Config>`, construidas al vuelo).
- C7 breaking sin shims (solo `src/config.rs`): 7 vars
  `VANTA_LLM_URL`→`VANTADB_LLM_URL`, `VANTA_LLM_MODEL`→`VANTADB_LLM_MODEL`,
  `VANTA_LLM_SUMMARIZE_MODEL`→`VANTADB_LLM_SUMMARIZE_MODEL`,
  `VANTA_LOCAL_MODEL`→`VANTADB_LOCAL_MODEL`, `VANTA_PREFETCH`→`VANTADB_PREFETCH`,
  `VANTA_DISABLE_PREFETCH`→`VANTADB_DISABLE_PREFETCH`,
  `VANTA_BACKEND`→`VANTADB_BACKEND`. Cero `VANTA_*` en `src/config.rs`
  (`rg -n "VANTA_" src/config.rs` vacío: `VANTADB_` no matchea `VANTA_`).
- Fuera de alcance (Q2=B → FIND-89): lecturas directas en 12 ficheros
  (`cli.rs`, `crypto.rs`, `cli_handlers/server.rs`, `llm.rs`, `metadata.rs`,
  `index/graph/prefetch.rs`, `server/bootstrap.rs`, `physical_plan/vector.rs`,
  `server/telemetry.rs`, `error.rs`, `server/errors.rs`,
  `storage/engine/maintenance.rs`) + legacy restantes (`VANTA_DB`,
  `VANTA_EMBEDDING_PROVIDER`, `VANTA_OPENAI_*`, `VANTA_BACKUP_DIR`, mirrors).
- Blast radius: 108 sitios `Config {` (64 `src/` + 44 `tests/`) — compilador
  como árbitro; `cargo check -p vantadb --tests --all-targets` verde.
- Docs: `docs/operations/CONFIGURATION.md` tabla + nota de migración F3C;
  `docs/CHANGELOG.md` NO tocado a mano (Regla 7: lo genera release-plz desde
  `feat!:` + `BREAKING CHANGE:`).
- Tests: 3 nuevos en `src/config.rs` (vistas, Rbac alias, hot-reload) +
  `tests/prefetch_benchmark.rs` migrado a `VANTADB_PREFETCH`.
