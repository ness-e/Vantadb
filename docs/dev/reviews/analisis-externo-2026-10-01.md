---
title: Análisis externo 2026-10-01 — catálogo 42 ítems (verificado) + decisiones owner
kind: review
status: active
description: "Registro versionado del análisis externo de 42 ítems y su meta-verificación: estado por ítem, correcciones aplicadas, filas creadas (DIST-15..18, WIRE-12..18, WSM-15, VER-10, SRV-10, FIND-221..223, MKT-22, PROV-13) y decisiones del owner."
tags: [vantadb, reviews, audit, backlog, scope]
---

# Análisis externo 2026-10-01 — catálogo 42 ítems (verificado) + decisiones owner

> **Procedencia:** análisis externo (42 ítems) recibido 2026-10-01, meta-verificado por un segundo documento, y procesado por vanta-lead con decisiones del owner en sesión. Este archivo versiona el resultado: qué se trackeó, qué se corrigió, qué se creó y qué se descartó — el resultado ya no vive solo en el chat.

## 1. Veredicto de la verificación

- El catálogo de 42 ítems es **mayormente correcto**: 28 ya estaban trackeados en los backlogs, 7 no tenían fila, 3 filas eran stale (A1/D2/D9) y 4 parciales (B7/F1/E2/D7).
- La meta-revisión (segundo documento) fue la más rigurosa; la verificación independiente de vanta-lead la confirmó salvo dos cifras: **ADRs = 54 numerados** (su "56" = archivos totales en `adr/` = 54 + `README.md` + `DECISIONS-NOT-TAKEN.md`) y **npm = 3** ✓.
- Números canónicos verificados → [`verified-numbers.md`](../references/verified-numbers.md) (registro con comando de verificación por cifra).

## 2. Estado por ítem del catálogo (42)

### Ya trackeados (28 — sin alta)
`FUT-02/03/08/10/14..24` · `PRO-01..06` · `MEMG-01/10/12/15/18` · `MGR-04/11/22/25` · `DEC-02` · `FIND-98/120/121/187/189/196/199` · `STU-01` · `SCH-09` · `DIST-05..08` · `CLD-04` (+ `SRV-10` referenciado en la API pública, fila creada abajo).

### Filas nuevas creadas (7 + 1)
| Ítem del catálogo | ID final | Nota |
|---|---|---|
| `graphrag_search` en bindings | **DIST-15** | el propuesto `DIST-09` colisionaba (icebox archivado) |
| `verify` de certificados vía MCP | **DIST-16** | el propuesto `DIST-10` colisionaba (task file `DIST-10-14`) |
| IQL `LIMIT`/`OFFSET` | **WIRE-12** | — |
| IQL agregaciones (`COUNT`/`SUM`/`GROUP BY`) | **WIRE-13** | — |
| Lock OPFS multi-pestaña | **WSM-15** | el propuesto `WSM-04` ya fue "Errores tipados" (ejecutado) |
| Attestation de escritura | **VER-10** | — |
| Contrato/fix de atomicidad `put_batch` | **FIND-221** | + contrato "atómico por chunk" documentado ya en `PYTHON_SDK.md` |
| Cifrado en reposo del server | **SRV-10** | 8.º hallazgo: sin fila pese a estar referenciado en `HTTP_API.md:701` |

### Stale / parciales (correcciones al catálogo — el repo ya estaba al día)
- **A1** (bitemporal+confianza): implementado en main (SCH-01..07 ✅); "stale" solo en el documento.
- **D2** (timeouts del server): implementado (`src/server/router.rs:242,258`); fuente del doc desactualizada.
- **D9** (notificaciones MCP): arreglado (`vantadb-mcp/src/protocol.rs:24` + tests).
- **B7** (reranker): cerrado — el reranking vive en `integrations/` (MMR sí está en core).
- **F1** (`put_batch`): gap de contrato → FIND-221. **E2** (`verify` MCP): → DIST-16. **D7** (audit compliance): cubierto por PRO-06.

## 3. Hallazgos del meta-documento aplicados

1. **Comentario stale en `vanta-proxy/src/capture.rs:297-299`** ("Hoy FAIL…" con el dual-write ya implementado) → corregido en `4aeb2c00` (test verde verificado por el lead).
2. **Coverage en nightly sin "watcher"** → regla del Build Cop escrita en `CONSTRAINTS.md` (`85f5a0bc`); la alerta automática ya existía (job `notify-failure`, deduped).
3. **Método "4 pasadas"** (los reviews viven como archivos, no en chat) → adoptado; este documento es su primer artefacto.

## 4. Decisiones owner aplicadas (2026-10-01)

- 22 preguntas de estrategia/backlog + 30 por componente de la lista de congelados.
- **Lista de congelados hasta 1.0** → [`EXPERIMENTAL_FEATURES.md` §Freeze List](../../user/operations/EXPERIMENTAL_FEATURES.md) (`77940780`).
- Proxy: **congelado sin publicar hasta 1.0.0** (DIST-18 re-scopeado, `3eebbea3`).
- Scheduler L0→L3: hogar `vantadb-server` ([ADR-0054](../architecture/adr/ADR-0054-scheduler-host-vantadb-server.md)) **+ starter para embedders** (decisión B2).
- **C5 cerrado:** registro de case studies no verificados eliminado (`77940780` / `fc9e1de4`).
- Instalador como selector de módulos → fila **DX-12**; drift de `completions/` → fila **DX-13**.
- Release 0.8.0: sin fecha — GO del owner pendiente (todo lo técnico listo).

## 5. Referencias

- Registro de números: [`verified-numbers.md`](../references/verified-numbers.md)
- Filas creadas: [`Backlog.md`](../Backlog.md) §Alta 2026-10-01 + §Derivadas de ADR-0054
- Cierres: [`backlog-history.md`](../avance/historial/backlog-history.md) §Cierres pre-release 0.8.0
