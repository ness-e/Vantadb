---
title: "ADR-0012: Naming de env vars — `VANTA_DB` vs `VANTADB_STORAGE_PATH`"
kind: adr
status: superseded
description: "Superseded by ADR-0059 — the VANTA_DB / VANTADB_STORAGE_PATH coexistence is gone; both the CLI --db flag and Config::from_env() read VANTADB_STORAGE_PATH, and VANTA_DB is read nowhere in the tree"
tags: [vantadb, adr]
superseded_by: docs/dev/architecture/adr/ADR-0059-storage-path-env-unification.md
---

# ADR-0012: Naming de env vars — `VANTA_DB` vs `VANTADB_STORAGE_PATH`

## Status

Superseded by [ADR-0059](./ADR-0059-storage-path-env-unification.md). The
coexistence this ADR decided no longer exists in the code: the global `--db`
flag and `VantaConfig::from_env()` both read `VANTADB_STORAGE_PATH`, and
`VANTA_DB` is read **nowhere** in the tree (verified 2026-10-20).

The decision text below is left unedited as the historical record. The debt
it named — *"la coexistencia de dos nombres persiste"* — was settled later by
the FIND-89 env-var consolidation (2026-09-14), which folded the `--db` flag's
env into `VANTADB_STORAGE_PATH` with no shims.

- **Status:** Accepted (2026-08-05) — superseded 2026-10-20 by ADR-0059
- **Date:** 2026-08-05
- **Deciders:** vanta-lead, vanta-arch
- **Related:** TECH-01 (fix `--db` MCP child), TECH-04, AUD-010

## Context

Coexisten dos lecturas de env vars para la ubicación de la base de datos:

- `VANTA_DB` — flag CLI de clap (`src/cli.rs:14-16`, `#[arg(short, long, env = "VANTA_DB", default_value = "./db", global = true)]`).
- `VANTADB_STORAGE_PATH` — lectura de configuración (`src/config.rs:408`, fallback `"vantadb_data"`).

El bug TECH-01 (MCP child ignora `--db`) ocurre porque el padre setea `VANTA_DB` en el hijo (`src/cli_handlers/server.rs:244`) pero `VantaConfig::from_env()` lee `VANTADB_STORAGE_PATH`.

## Decision

1. **Sin renombrado.** `VANTA_DB` se mantiene como flag CLI (clap). `VANTADB_STORAGE_PATH` se mantiene como env de configuración. Renombrar cualquiera de los dos rompe contratos existentes (docs, scripts, CI, usuarios) con beneficio cosmético.
2. **El child `vantadb-server` setea AMBOS.** El fix del síntoma (TECH-01) es que `cmd.env("VANTA_DB", db_path)` **añada** `VANTADB_STORAGE_PATH`, no que reemplace el flag CLI.
3. **Documentar** en `docs/user/operations/CONFIGURATION.md`: tabla con ambas vars, cuál gana en cada contexto (CLI flag > env de config), y la relación entre ellas.

## Consequences

- ✅ Fix mínimo de TECH-01 sin breaking change.
- ✅ ADR elimina la ambigüedad para futuros agentes (Regla 5: memoria de decisiones).
- ⚠️ Deuda documentada: la coexistencia de dos nombres persiste. Si en el futuro se quisiera unificar, el ADR queda como base (deprecar `VANTA_DB` con alias `VANTADB_CLI_DB_PATH` y warning).

## Verificación

- `vanta-cli server --mcp --db /x` → lock/persistencia en `/x` (TECH-01 e2e).
- `docs/user/operations/CONFIGURATION.md` documenta ambas vars.
