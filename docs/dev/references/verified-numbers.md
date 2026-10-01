---
title: VantaDB — Verified Numbers Registry
kind: research
status: active
description: Números canónicos del repo (para material público y docs) con el comando exacto que los verifica. Regla del owner 2026-10-01 — ningún número se publica sin estar acá o verificarse con su comando.
tags: [vantadb, references, numbers, metrics]
---

# VantaDB — Verified Numbers Registry

> **Regla (owner, 2026-10-01):** todo número que describa el repo y vaya a material público
> (README, docs, posts, Notion) se toma de esta tabla o se verifica con el comando listado y
> **se registra acá** (valor + fecha + comando). Un número con número y sin comando es una
> aspiración, no un dato. Verificador: vanta-lead (checklist mecánico).

## Tabla canónica

| Número | Valor verificado | Comando de verificación | Verificado |
|---|---|---|---|
| Workflows (archivos `.yml`) | **37** | `(Get-ChildItem .github/workflows -Filter *.yml).Count` | 2026-10-01 |
| ADRs (numerados) | **54** | `(Get-ChildItem docs/dev/architecture/adr -Filter 'ADR-*.md').Count` | 2026-10-01 |
| Archivos totales en `adr/` | 56 (= 54 ADRs + `README.md` + `DECISIONS-NOT-TAKEN.md`) | `(Get-ChildItem docs/dev/architecture/adr -File).Count` | 2026-10-01 |
| Documentos `.md` totales | **1689** (1469 no-archivados) | `node scripts/docs/check-docs.mjs` (línea `documents:`) | 2026-10-01 |
| Task files | **1027** | `(Get-ChildItem docs/dev/tasks -Recurse -Filter *.md).Count` | 2026-10-01 |
| Paquetes npm publicados | **3** (`vantadb`, `vantadb-node`, `vantadb-wasm`) | nombres en `vantadb-ts/package.json`, `vantadb-node/package.json`, `vantadb-wasm/pkg/package.json` | 2026-10-01 |
| MCP tools | **79 listadas / 85 definidas** | `docs/api/MCP.md` (canónico) + `scripts/validate-docs-coverage.ps1` | 2026-10-01 |
| MCP default profile | `agent` (subset; cambio WIRE-02) | `docs/api/MCP.md` §profiles | 2026-10-01 |
| Adapters en `integrations/` | **9** | `(Get-ChildItem integrations -Directory).Count` → 11 (= 9 + `__pycache__` + `vantadb_shared`) | 2026-10-01 |
| Cobertura de proyecto | **68.2%** (ratchet: no bajar) | `cargo llvm-cov --workspace` / nightly job `coverage-budget` | 2026-10-01 |
| `AGENTS.md` | 17 líneas (stub) | `(Get-Content AGENTS.md).Count` | 2026-10-01 |

## Pendientes de definir (NO publicar hasta verificar)

| Número | Estado | Acción |
|---|---|---|
| Suites de prueba ("77") | Sin definición mecánica (¿test binaries? ¿archivos? ¿nextest?) | Definir convención + comando (`cargo nextest list`) y registrar |
| Crates del workspace ("17+") | Sin conteo mecánico | `cargo metadata` o conteo de `Cargo.toml` y registrar |
| Workflows "activos" | **Stale**: el inventario (`docs/dev/workflow/README.md`) dice "28 active"; hay 37 archivos | A4: definir convención (activos vs total) y corregir el inventario |

## Reglas de uso

1. Los números se citan con su contexto exacto (p.ej. "37 archivos de workflow", no "37 workflows" a secas).
2. Si un número cambia (workflow nuevo, ADR nuevo, paquete nuevo), se actualiza acá en el mismo PR que lo cambia.
3. Números de rendimiento (p50, recall, etc.) van con su harness + fecha en `docs/user/benchmarks/`, no acá.
4. Caso testigo: el análisis externo publicó 87/26/91 sin verificar — los reales eran otros. Este registro existe para que no vuelva a pasar.
