---
title: "Zep Cloud / Konig - extracción docs-only"
kind: index
status: active
description: "Índice del dossier Zep Cloud/Konig: informe de análisis y backlog de candidatas para VantaDB"
tags: [extraccion, zep, konig, backlog]
---

# Zep Cloud / Konig — extracción docs-only

Esta carpeta contiene el estudio complementario de Zep Cloud y Konig. No es una extracción de código: Zep Cloud/Konig es cerrado. La evidencia primaria es documentación pública de `help.getzep.com/v4`, páginas públicas de producto y el contraste contra VantaDB HEAD/candidatos existentes.

| Archivo | Contenido |
|---|---|
| `ZEP-CLOUD-KONIG.md` | Informe completo: retrieval, Context Block, governance, seguridad, Agent Skills, admission, Konig, benchmarks, límites y matriz contra VantaDB |
| `BACKLOG.md` | Candidatas `X-ZEP-*` para investigación/especificación futura; ninguna está implementada ni aprobada |

## Estado

- **Tipo de evidencia:** docs-only.
- **Fecha de consulta:** 2026-11-20.
- **Alcance:** Zep Cloud v4, Konig, Context Graphs, Context Block, governance, Agent Skills y metodología de evaluación.
- **No alcance:** código del producto, ejecución de servicios, benchmarks propios, verificación de claims comerciales ni implementación en VantaDB.

## Fuentes principales

- Docs v4: <https://help.getzep.com/v4/llms.txt>
- Retrieval: <https://help.getzep.com/v4/searching-the-graph>
- Context Block: <https://help.getzep.com/v4/retrieving-context>
- Templates: <https://help.getzep.com/v4/context-templates>
- Smart Query: <https://help.getzep.com/v4/querying-a-graph>
- Governance: <https://help.getzep.com/v4/governance>
- ABAC: <https://help.getzep.com/v4/policy-based-access-control>
- Seguridad de memoria: <https://help.getzep.com/v4/memory-security>
- Agent Skills: <https://help.getzep.com/v4/agent-skills>
- Admission: <https://help.getzep.com/v4/agent-skills/learning-and-admission>
- Konig: <https://www.getzep.com/platform/agent-knowledge-graph/>

## Relación con material previo

- Graphiti ya fue auditado como proyecto open-source: `../agent-memory-2026-11-07/graphiti.md`
- Backlog previo: `../backlogs/graphiti.md`
- Consolidado general: `../agent-memory-2026-11-07/CONSOLIDATED.md`
- Importador Zep existente: `../../api/MEMORY_INTERCHANGE_FORMAT.md`
