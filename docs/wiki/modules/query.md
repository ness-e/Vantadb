---
title: "Módulo: query (`parser`, `planner`, `physical_plan`, `executor`)"
kind: concept
status: active
description: "Pipeline IQL: lexer/grammar, planner, operadores físicos y ejecutor — estructura, versión del lenguaje y fuentes."
---

# Módulo: query (`parser`, `planner`, `physical_plan`, `executor`)

Responsabilidad: el lenguaje de consulta IQL y su ejecución sobre el motor.

## Estructura

`[HECHO]` `src/parser/` = `mod.rs`, `lexer.rs`, `grammar.rs` (nom); la versión del lenguaje es una constante pública ([`IQL_VERSION = 4` en `src/parser/mod.rs:20`](../../../src/parser/mod.rs#L20), re-exportada en `src/lib.rs:200`) — la referencia [IQL](../../api/IQL.md) documenta `AS OF` valid-time, LIMIT/OFFSET y COUNT/SUM/GROUP BY junto a las constantes `IQL_VERSION_MIN_*` del mismo archivo. `[HECHO]` `src/planner.rs` planifica, `src/cost_estimator.rs` (`pub(crate)`) estima, `src/physical_plan/` contiene operadores `scan`, `filter`, `project`, `join`, `aggregate`, `dedup`, `sort`, `offset`, `vector`; `src/executor.rs` ejecuta. `[HECHO]` `src/operator_registry.rs` es un registro de operadores lógicos dispatch-by-name (`src/lib.rs:132-133`); `src/query.rs` es el punto de entrada del módulo.

## Entradas públicas

`[HECHO]` `Embedded::query` (IQL → `QueryResult` re-exportado) expuesta en bindings como `query` y en MCP como tool `query_iql` (BINDINGS_NAMESPACES; excepción Node: sin método IQL, FIND-268). `[HECHO]` Los namespaces de memoria son consultables como tablas IQL sanitizadas (`/`/`-`→`_`, dígito/`.` inicial→`_` prefijo) — regla documentada en IQL.md.

## Tests existentes

`[HECHO]` `tests/logic/parser.rs`, `tests/logic/executor.rs`, `tests/logic/joins.rs`, `tests/logic/governor.rs`, `tests/logic/columnar.rs` (feature `arrow`), `tests/query_result_basic.rs`, `tests/query_result_advanced.rs`, `tests/time_travel.rs` (AS OF, ADR-0046 D3) — registrados en `Cargo.toml`.

## Límites

`[INFERENCIA]` LISP no está soportado (IQL.md lo declara; análisis en `docs/dev/architecture/LISP_ANALYSIS.md`). `[HECHO]` El snapshot `tests/api/public-api.txt` es la autoridad de la superficie `query`/`parser`/`physical_plan` pública.

## Fuentes

`src/parser/` · `src/planner.rs` · `src/physical_plan/` · `src/executor.rs` · `src/cost_estimator.rs` · `src/operator_registry.rs` · [IQL](../../api/IQL.md) · [LISP_ANALYSIS](../../dev/architecture/LISP_ANALYSIS.md)
