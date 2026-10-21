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
| `audit-full-20260808-002617.md` | Audit completo vigente del pipeline |
| `review-full-2026-07-27-0309.md` | Review completo |
| `review-certify-2026-08-05-2025.md` | Certify reciente |
| `review-full-2026-08-05-t1545.md` | Review rápido |
| `archive/` | Audits y reviews superados (pre-unificación) |
