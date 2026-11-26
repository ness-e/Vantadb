---
title: "Backlogs de extracción — índice y formato"
kind: index
status: active
description: "Formato canónico de backlogs de extracción y matriz legal; los backlogs por proyecto viven en carpetas propias bajo docs/dev/extraccion"
tags: [extraccion, backlog, index]
---

# Backlogs de extracción

Los backlogs canónicos por proyecto viven ahora en una carpeta propia dentro de
`docs/dev/extraccion/<repositorio>/BACKLOG.md`. Esta carpeta conserva el
formato obligatorio y la matriz legal [LICENSES.md](LICENSES.md).

Cada backlog convierte las candidatas del informe de extracción en filas
ejecutables con el **esquema canónico de 10 columnas** de `docs/dev/Backlog.md`.

## Formato obligatorio por archivo

Frontmatter:

```yaml
---
title: "Backlog de extracción — <Proyecto>"
kind: research
status: active
description: "<una línea: qué contiene>"
tags: [extraccion, backlog, <slug>]
---
```

Cabecera:

```markdown
# Backlog de extracción — <Proyecto>

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/<slug>.md`
> (o `docs/dev/extraccion/Backlog-OpenViking-VantaDB.md` para OpenViking).
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.
```

Tabla con las **10 columnas canónicas**, en este orden:

```
| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
```

- **ID:** `X-<PREFIJO>-NN` (prefijo corto único por proyecto, p. ej. `X-M0-01`).
- **Severidad:** 🔴/🟠/🟡/🟢/— según impacto si la brecha se confirma.
- **Hallazgo:** nombre corto del mecanismo propuesto **en voz VantaDB** — describe
  el comportamiento deseado (recibo, frontera, ledger, revisión, capability…),
  no "portar X de Y".
- **Archivo:línea:** seam de VantaDB donde aplicaría (ruta repo, ej.
  `vanta-memory/src/context_engine/engine.rs`) o `spec` si es diseño previo.
- **Esfuerzo:** 🟢 1d / 🟡 2-5d / 🟠 1-2sem / 🔴 >2sem.
- **Prioridad:** 🔴 P0 / 🟠 P1 / 🟡 P2 / 🔵 P3.
- **Estado:** `⬜ Candidata` siempre (nada implementado por esta campaña).
- **Descripción:** Qué · Por qué · Acción mínima · **Trazabilidad:** `informe
  §Candidata X · evidencia <ruta-dentro-del-clon>:<líneas>` — cada fila DEBE
  citar el archivo/módulo/líneas del clone de donde sale el punto.
- **Relaciones:** otras filas del mismo backlog o candidatos C1–C16 de
  `CONSOLIDATED.md` con los que converge.
- **Dependencias:** prerrequisitos en VantaDB declarados en el informe.

## Reglas

1. **Voz VantaDB**: las filas describen comportamiento deseado en VantaDB; no se
   redactan como "copiar/portar el componente tal".
2. **Trazabilidad obligatoria**: cada fila cita su evidencia (`archivo:líneas`
   del clone) en Descripción. La procedencia se conserva aquí y en el informe —
   no se borra.
3. Una fila por candidata/unidad mínima independiente del informe; no fusionar
   ni inventar candidatas nuevas.
4. Nada de promesas runtime: todo es propuesta sujeta a validación contra HEAD.
5. Marcar `—` en campos no aplicables; no dejar celdas vacías.
6. **Gate legal obligatorio ([LICENSES.md](LICENSES.md))**: antes de especificar o implementar cualquier fila `X-*`, consultar [LICENSES.md](LICENSES.md) para verificar si el proyecto origen es permisivo (`MIT`/`Apache-2.0`/`BSD`/`PostgreSQL`) o exige rediseño *clean-room* estricto sin abrir el clon (`AGPL-3.0`, `BSL-1.1`, `SSPL`/`RSALv2` o licencias modificadas).
