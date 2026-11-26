---
title: "Dossier transversal - candidatos unificados C1-C16"
kind: index
status: active
description: "Síntesis consolidada de la campaña agent-memory 2026-11-07 y su backlog ejecutable de candidatos transversales"
tags: [extraccion, transversal, consolidated]
---

# Dossier transversal — candidatos unificados C1–C16

Este dossier no corresponde a un repositorio upstream: es la capa de **síntesis**
de la campaña `agent-memory-2026-11-07`. Los 44 dossiers de proyecto analizan
cada fuente por separado; aquí se unifican los mecanismos convergentes en 16
candidatos transversales.

| Archivo | Contenido |
|---------|-----------|
| [CONSOLIDATED.md](CONSOLIDATED.md) | Síntesis documental completa: recomendaciones, fichas C1–C16 con seams/FMEA/aceptación, issues transferibles, prerrequisitos en VantaDB, qué no trasladar, benchmarks, matriz de brechas, deuda propia y escritorio/UI |
| [BACKLOG.md](BACKLOG.md) | 16 filas ejecutables (formato canónico de 10 columnas) mapeadas 1:1 desde C1–C16, con transformaciones documentadas de prioridad/severidad |

## Relación con el resto del corpus

- **Documentos de control de la campaña** (método, cobertura de revisión,
  contrato de ampliación, notas de benchmarks, referencia Vanta y evidencia
  cruda): permanecen en [`../agent-memory-2026-11-07/`](../agent-memory-2026-11-07/).
- **Backlogs por proyecto** (`X-*`): cada uno declara en su columna Relaciones
  con qué candidatos `Cn` converge; este backlog es el destino de esas
  convergencias.
- **Gate legal:** no aplica como dossier de clone — la síntesis es obra propia
  de la campaña; las discriminaciones de licencia por mecanismo viven en cada
  ficha de CONSOLIDATED y en [`../LICENSES.md`](../LICENSES.md).

## Advertencia de autoridad

CONSOLIDATED razona sobre snapshots inmutables (motor V `97c9623c8…`, Studio
`0ad38c542…`). Nada aquí certifica el estado actual de HEAD: toda fila del
backlog exige revalidación antes de especificar.
