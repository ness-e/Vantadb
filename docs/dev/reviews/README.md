---
title: Reviews archivados (2026-08-26) — migrados a Notion
kind: index
status: archived
description: Los 45 reports archivados se conservan en Notion (página "Reviews 2026"), con el análisis de vigencia de cada uno
tags: [vantadb, review]
---

# Reviews archivados (2026-08-26) — migrados a Notion

> Los reports que vivían en `docs/dev/reviews/archive/` **ya no están en el repo**.
> Se conservan íntegros en Notion, con un análisis de vigencia added por review.

## Dónde están

**Notion › «Reviews 2026»** (dentro de «VantaDB Docs»):
<https://app.notion.com/p/401d04459756803ab32bc3af85dcdecd>

Una página por review, con este contenido:

1. El callout de metadatos (título, tipo, estado, tags, origen).
2. El cuerpo original del report (espejo literal del markdown).
3. Un **toggle «Auditoría de vigencia»** con el análisis a 2026-10-22:
   - qué analiza el review y qué define;
   - verificación de cada hallazgo contra el código actual, con evidencia `archivo:línea`;
   - **los puntos que siguen vigentes y no se han aplicado**, priorizados P1..P4 con
     impacto y esfuerzo;
   - la higiene documental pendiente del propio review.

## Por qué se movieron

Sus contenidos accionables ya estaban consolidados en
[`PLAN-ACCION-UNIFICADO.md`](PLAN-ACCION-UNIFICADO.md) (permanece en el repo como
documento activo de la serie) y los hallazgos vivos siguiendo el flujo habitual
(`docs/dev/Backlog.md` → `docs/dev/avance/`). Mantener los reports en el repo solo
duplicaba estado: los conteos y rutas de julio-agosto 2026 ya no describen el
proyecto, y los referenciaba un índice que quedó congelado.

**El historial no se pierde:** los 45 ficheros siguen en el historial de git
(eliminados en el commit que cerró esta limpieza) y su contenido íntegro está en
Notion.

## Índices que sustituyen a este

- [`INDEX.md`](INDEX.md) — los reviews vivos (campañas de 2026-10-20/21).
- [`docs/dev/reports/INDEX.md`](../reports/INDEX.md) — registro maestro de reportes;
  las filas de los 45 archivados apuntan a su página de Notion.
- [`docs/dev/avance/`](../avance/) — registro canónico de lo completado.
