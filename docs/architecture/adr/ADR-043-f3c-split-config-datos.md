---
title: "ADR-043: S-split-config B+B (datos para firma humana)"
type: adr
status: proposed
tags: [vantadb, architecture, adr]
created: 2026-09-14
last_reviewed: 2026-09-14
---

# ADR-043: S-split-config B+B (datos para firma humana)

> **Excepción a Regla 5 (orden explícita del owner 2026-09-14):** las secciones
> Contexto/Decisión/Consecuencias las redactó vanta-lead articulando las decisiones
> humanas tomadas vía `question` (D0 B+B + F3C Q1=B/Q2=B/Q3=A). Ver `docs/avance/meta.md`.
> Diseño canónico: `docs/tasks/F3C.md`. Plan: `docs/plans/2026-09-13-cleanCA-fase3.md`.

## Contexto

`Config` era un struct plano de ~52 campos donde cada feature solo escribía su sección
pero todos leían todo (god-struct de lectura), con 42 variables de entorno en dos
prefijos inconsistentes (`VANTA_*` legacy vs `VANTADB_*`). El churn medido es por
dominio (D0: 14/14 commits tocan una sección), así que el struct no refleja cómo el
código realmente cambia. El hot-reload (`apply_to`, 8 campos, watcher) vive 100% en
`src/config.rs` y ninguna dependencia lo resuelve: cualquier opción debía preservarlo.

## Decisión

**B+B:** split interno en 6 dominios (`StorageCfg`/`ServerCfg`/`LlmCfg`/`EvictionCfg`/
`PoolCfg`/`RbacCfg` propio) con fachada `Config` plana y compatible (implementada como
vistas `From<&Config>`, sin duplicar estado), más unificación `VANTA_*→VANTADB_*`
(7 vars) con breaking limpio en el mismo cambio, sin shims. Los 12 ficheros con
lectura directa de env no se consolidan ahora (van a FIND-89 con dueño).

## Consecuencias

Costos: bump major (breaking de 7 vars, documentado en CONFIGURATION.md + changelog
vía release-plz); 108 sitios migran mecánicamente (compilador como árbitro).
Riesgos: bajo — fachada conserva `Default`/`with_*`/acceso plano, warn+default intacto,
suites 57/57. Deuda asumida: FIND-89 (12 ficheros con env directo) y firma de este
ADR por excepción. Alternativas descartadas: mantener+documentar (el god-lector sigue),
figment/config-rs (nueva dependencia sin resolver el hot-reload), shims temporales
(dos dolores en vez de uno).

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
