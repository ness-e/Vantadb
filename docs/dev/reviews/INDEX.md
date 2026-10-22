---
title: "Reviews y auditorias - indice"
kind: index
status: active
description: "Mapa de los informes de revision y auditoria del proyecto. La campana del 2026-10-20 (auditoria exhaustiva del arbol v0.9.0) tiene 24 ficheros: este indice dice cual es cual."
tags: [vantadb, reviews, index, audit, reviews]
---

# Reviews y auditorías — índice

> Pipeline que escribe en esta carpeta: `dev-tools/audit-all.ps1` (`$ReportDir`), prompt
> `audit-full.md` (escribe `audit-<mode>-<timestamp>.md`) y la skill `unified-review`.
> El registro canonico de que fuente esta viva esta en
> [`docs/dev/avance/fuentes-vivas.md`](../avance/fuentes-vivas.md).

## Campaña del 2026-10-20 — auditoría exhaustiva del árbol v0.9.0

Campaña read-only de 11 waves (W0-W10) + 7 subagentes, ejecutada mientras corria el CI del
release v0.9.0. **141 hallazgos** (2 P0, 39 P1, 59 P2, 41 P3) + 30 de las waves.

| Fichero | Qué es |
|---|---|
| [`2026-10-20-auditoria-profunda-v0.9.0.md`](2026-10-20-auditoria-profunda-v0.9.0.md) | **Informe consolidado** (407 líneas): veredicto por área, los 2 P0 y los P1 con su detalle (qué rompe / por qué importa / fix / cómo verificar), cobertura por área y riesgos |
| [`2026-10-20-auditoria-profunda-v0.9.0-ledger.md`](2026-10-20-auditoria-profunda-v0.9.0-ledger.md) | **Ledger de los 141 hallazgos** (165 líneas): una línea por hallazgo con id, severidad, área, ubicación y resumen. Es el registro que consume el backlog |
| [`2026-10-20-release-v0.9.0-postmortem.md`](2026-10-20-release-v0.9.0-postmortem.md) | **Post-mortem del release v0.9.0**: los 7 fallos que lo costaron, causa raíz de cada uno, el arreglo y el checklist de pre-flight |
| [`audit-deep-sintesis-20261020-033416.md`](audit-deep-sintesis-20261020-033416.md) | **Síntesis de las 11 waves**: tablero por wave, top-20 hallazgos, mapa de cobertura, FODA y veredicto global |
| `audit-deep-w0-harness-…` … `audit-deep-w10-verificacion-…` | **12 informes por wave**: W0 harness · W1 core Rust · W2 bindings · W3 server+MCP · W4 embeddings+providers · W5 docs · W6 tests · W7 benchmarks · W8 tooling · W9 cross-cutting · W10 verificación |
| `audit-subagent-s1-ci-release-…` … `audit-subagent-s7-readmes-…` | **7 informes crudos de subagente**: S1 CI/release · S2 core Rust · S3 bindings+embeddings · S4 providers+adapters · S5 MCP+server+skills · S6 árbol de docs · S7 READMEs. Son la evidencia de los hallazgos por área |

**Cómo se cierra un hallazgo:** el id de origen del ledger (`D-01`, `S3-B1`, `W5-12`…) es el que
aparece en la columna *Relaciones* de la fila del backlog
([`docs/dev/Backlog.md`](../Backlog.md), Phase 60). Al completarse, la fila **sale** de la tabla
activa y su registro pasa a [`docs/dev/avance/`](../avance/) (el formato no admite
`✅ Completada` en tablas activas).


## Auditorias de hueco de cobertura - 2026-10-21

Cuatro auditorias de solo lectura que cierran los huecos que la campana del 2026-10-20 declaro
(`FIND-502`, `FIND-504`, `FIND-503` parcial y `FIND-506`). Sin `cargo`/`npm`: lo no ejecutado va
marcado como sospechado en cada informe.

| Fichero | Que es |
|---|---|
| [`audit-grammar-iql-20261021.md`](audit-grammar-iql-20261021.md) | **Gramatica IQL** (`src/parser/`): 14 hallazgos (1 critico) - el `WHERE` entero se pierde si la primera condicion no parsea, el trailing input se descarta en silencio, `~` tiene dos semanticas, recursion sin limite en subqueries |
| [`audit-ffi-bindings-20261021.md`](audit-ffi-bindings-20261021.md) | **Frontera FFI y bindings**: 19 hallazgos (7 High) - la frontera es memoria-segura por construccion, pero 3 stubs fabrican datos (`metricsPrometheus`, `graphShortestPath`, `findNodesByLabel`) y `searchMulti` lanza siempre |
| [`audit-physical-plan-20261021.md`](audit-physical-plan-20261021.md) | **physical_plan / planner / executor**: 20 hallazgos (4 criticos) - la proyeccion `FETCH` anula el filtro `AS OF`, el path vectorial no scopea por entidad, el CBO puede eliminar un predicado y la reserva del governor se fuga si la query falla |
| [`audit-adapters-skills-20261021.md`](audit-adapters-skills-20261021.md) | **Los 9 adaptadores y las skills**: 37 hallazgos (10 High) - 5 ejemplos de la skill reventan, el MCP no expone `exclude_superseded`, los 9 README prometen BM25 y ninguno lo cumple |

## Auditorias anteriores (con su estado actual)

| Fichero | Qué es | Estado de sus hallazgos |
|---|---|---|
| `audit-full-20261017-multiagent.md` | Auditoria multiagente del 2026-10-17 (8 hallazgos `D2-FIND-01..08`) | **Los 8 addressed** (verificado en el codigo el 2026-10-21): el bypass de auth con solo `jwt_secret` (el middleware lo declara, `src/server/middleware.rs`: "FIND-346: If jwt_secret IS configured, enforce Bearer auth even when api_key is None"), la lectura/escritura arbitraria de ficheros por `/api/v2/export` e `/import` (`validate_http_export_import_path` antes de usar la ruta), el `unreachable!()` alcanzable (eliminado), el feature `reqwest/blocking` (eliminado) y el bloqueo de workers Tokio (`spawn_blocking`). Sus ids (`FIND-346/347/355/356/365/366/375/376`) **no** llegaron a ser filas del backlog: la numeracion quedo repartida entre la sesion que los resolvio y la de esta campana. |
| `2026-10-18-full-system-audit.md` | Auditoria de sistema completa del 2026-10-18 | Sus hallazgos son los de la campana del 2026-10-20 (los cita por id) |
| `analisis-externo-2026-10-01.md` | Analisis externo del 2026-10-01 | Sus 2 hallazgos citados (`FIND-221`, `FIND-98`) **estan en el backlog** |

## Notas de wave — cifrado en reposo (ADR-0057)

| Fichero | Qué es |
|---|---|
| [`2026-10-21-w6-encryption-activation.md`](2026-10-21-w6-encryption-activation.md) | **W6** — activacion del cifrado en reposo: un contexto y un registro por proceso, deteccion de modo en el open, rotacion de la master key, el snapshot copia el registro y la documentacion honesta (limites, runbook de claves). Incluye lo que **queda abierto** para W7 (la medicion del p99). |

## Informes del pipeline

| Fichero | Qué es |
|---|---|
| Notion › [«Reviews 2026»](https://app.notion.com/p/401d04459756803ab32bc3af85dcdecd) | Los 45 reports históricos del pipeline (`audit-full-20260808-002617.md`, `review-full-2026-07-27-0309.md`, `review-certify-2026-08-05-2025.md`, `review-full-2026-08-05-t1545.md` y 41 más): ya no están en el repo, se conservan en Notion con el análisis de vigencia de cada uno |
| `archive/` | Audits y reviews superados (pre-unificación) — ya no están en el repo: se conservan en Notion › [«Reviews 2026»](https://app.notion.com/p/401d04459756803ab32bc3af85dcdecd), una página por review con el análisis de vigencia |
