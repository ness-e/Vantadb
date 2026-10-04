---
title: "`gate-docs-links.yml` — GATE: Docs — Links, Schema, Indexes"
kind: runbook
status: active
description: Cuatro jobs independientes que miden cuatro números que antes no se medían
tags: [vantadb, ci, gate-docs, documentation]
related: [.github/workflows/gate-docs-links.yml, RULES.md, gate-docs-21.md, TRIGGERS.md]
---

# `gate-docs-links.yml` — GATE: Docs — Links, Schema, Indexes

## ¿Qué hace?

Cuatro jobs independientes que miden cuatro números que antes no se medían.

> **Contexto:** el 2026-09-28 el corpus de `docs/` tenía 1725 ficheros, ~110 enlaces markdown
> rotos, **1481 ficheros huérfanos (86%)** y **1250 ficheros sin frontmatter (73%)**. Este
> workflow convierte esos tres números en algo que un PR puede romper y arreglar de forma visible.

| Estado 2026-09-28 | Antes | Después |
|---|---|---|
| Enlaces markdown rotos | ~110 | 109 (presupuesto) |
| Huérfanos | 1481 de 1717 (86%) | **4 de 1489** |
| Sin frontmatter | 1250 de 1717 (73%) | **0** |
| Wikilinks en prosa | 650 | **40** en 33 ficheros (presupuesto) |
| Violaciones de esquema que gatean | — (no había gate) | **0** |

| Estado 2026-10-04 (F1 cerrado) | Valor |
|---|---|
| Enlaces markdown rotos (fuera de frozen) | **0** (de 44; triage P0-P4 en `docs/dev/tasks/DOCS-F1.md`) |
| Wikilinks en prosa | **20** (de 40) — 19 en `tasks/**` congelados + 1 en historial; superficie viva = 0 |
| Errores markdownlint | **0** en la superficie controlada (de 12); el master plan (recitations) queda excluido del ratchet — FIND |
| Mojibake en prosa (`[[bench]]`/`[[test]]`/`[[package]]`) | **0** fuera de `tasks/` |

| Job | Comprueba | Script | Gate |
|---|---|---|---|
| `docs-links` | Enlaces markdown resuelven (0 rotos fuera de frozen); wikilinks ≤ 20 | `check-links.mjs` | sí |
| `docs-schema` | Frontmatter válido, `kind` de acuerdo con la ruta, huérfanos | `check-docs.mjs` | parcial (ver abajo) |
| `docs-index` | Índices generados al día | `gen-index.mjs --check` | sí |
| `docs-lint-regression` | markdownlint no empeora (master plan excluido: FIND) | baseline = 0 | sí, sobre presupuesto |

## ¿Por qué `docs-schema` no gatea los huérfanos todavía?

`check-docs.mjs` tiene dos modos. Por defecto gatea sólo lo que ya está limpio
(frontmatter presente, esquema válido, `kind` coherente, sin wikilinks). Los **nombres
duplicados** (69 grupos) y los **huérfanos** se reportan pero no bloquean, porque un gate
que nace rojo se apaga. Cuando el recuento llegue a 0, se promotes con `--strict`.

Los huérfanos ya bajaron de 1481 a 4 gracias a `gen-index.mjs`; el resto es deuda
deliberada, documentada en `docs/dev/plans/2026-09-28-docs-consolidation.md`.

## ¿Por qué hay presupuestos numéricos en lugar de reglas?

Tres gates usan un **presupuesto** en vez de exigir cero, y los tres por la misma razón: un
gate que nace rojo se apaga. Los tres están fechados y referenciados a una tarea del plan
`docs/dev/plans/2026-09-28-docs-consolidation.md`, así que baja cuando la deuda se drena.

| Presupuesto | Valor 2026-09-28 | Por qué no es cero | Se drena en |
|---|---|---|---|
| Enlaces markdown rotos | 109 | `repair-links.mjs` sólo reparó 1 de 110 automáticamente. No son rutas desviadas: son enlaces a documentos que **no existen en el repo**. 62 de los 109 son de superficie pública (`docs/user` 34 + `docs/api` 28) | F1-T1 |
| Wikilinks en prosa | 40 ocurrencias / 33 ficheros | ~30 son mojibake de una pasada de codificación anterior: `[[bench]]`, `[[test]]`, `[[package]]`, `[[bin]]`. El texto original no existe en el fichero, así que restaurarlo requiere leer `git log`. ~10 son enlaces a ficheros renombrados | F1-T1, F1-T2 |
| Errores markdownlint | 12 | 5 preexistentes + 7 introducidos por la propia migración de wikilinks. `markdownlint --fix` no se usa: son ficheros de prosa escritos a mano | F1-T5 |

**Estado 2026-10-04 (F1 cerrado):** los tres presupuestos bajaron — enlaces
rotos **0** (P0-P2 drenados; P3 excluido del gate, ver abajo), wikilinks **20**
(de 40), markdownlint **0** en la superficie controlada (de 12; el master plan
— recitations machine-appended del orquestador — queda excluido del ratchet,
FIND).
Triage completo de las 55 entradas: `docs/dev/tasks/DOCS-F1.md`.

## ¿Qué queda fuera del gate bloqueante? (P3/P4 — triage F1)

El triage de las 55 entradas rotas (2026-10-04, `docs/dev/tasks/DOCS-F1.md`)
clasificó cada una P0-P4. Quedan **fuera del gate** por política, con el motivo
en el script (`FROZEN_RE`) y en el workflow (`gate-docs-links.yml`):

- **P3 — frozen by design:** `docs/dev/avance/historial/**` (registro histórico:
  snapshots y campañas) y `docs/dev/tasks/**` (work-items congelados; restricción
  del owner: no se mueven ni se editan sin su propio task file). Un enlace roto
  ahí nunca se repara, así que contarlo sería ruido permanente. `archive/**` ya
  estaba fuera de scope por `ARCHIVE_RE`.
- **P4 — externos:** nunca gatean (solo conteo); el `schedule` semanal hace el
  barrido (patrón qdrant/lancedb: `fail: false` + issue de seguimiento).

Cada presupuesto tiene su flag para ajustarlo sin tocar código:
`--max-broken=N`, `--max-wikilinks=N`, y `BASELINE` en el propio workflow.

## ¿Cuándo se ejecuta?

- **Push** a `main` / `develop` con cambios en `docs/**`, `scripts/docs/**` o `.markdownlint-cli2.yaml`
- **Pull request** a `main` / `develop` con los mismos paths
- **Programado:** lunes 07:23 UTC (barrido informativo de todo el repo, nunca bloquea)
- **Workflow dispatch** manual

## Scriptos

Todos en `scripts/docs/`, sin dependencias externas, Node >= 18:

| Script | Para qué |
|---|---|
| `lib.mjs` | Índice de ficheros, frontmatter, derivación de `kind`, validación |
| `check-links.mjs` | Integridad de enlaces internos. Offline, sin red |
| `repair-links.mjs` | Re-resuelve enlaces rotos sólo con coincidencia única |
| `wikilinks-to-md.mjs` | Convierte `[[x]]` → `[x](y.md)`. Idempotente, nunca adivina |
| `fix-link-labels.mjs` | Reparación puntual de etiquetas con barra invertida |
| `stamp-frontmatter.mjs` | Sella `title`/`kind`/`status`/`description`/`tags` derivados |
| `gen-index.mjs` | Genera `docs/index.md`, índice de ADRs, índices de sección y `llms.txt` |
| `check-docs.mjs` | Gate único: esquema, `kind`, duplicados, huérfanos, wikilinks |

**Una sola definición de la sintaxis de enlace.** `MD_LINK`, `WIKI_LINK`, `segment()` y
`proseOf()` viven en `lib.mjs` y los tres scripts que escanean enlaces los importan. Tres
scripts con tres regex para la misma sintaxis son la razón por la que un corpus acaba con
tres totales distintos para ella — que es exactamente lo que pasó durante esta migración:
`check-links` decía 210, `check-docs` decía 93, y el número real en prosa era 40.

## Invariantes

- Los índices generados llevan un banner `<!-- GENERATED ... -->` y **nunca** se editan a mano.
- `wikilinks-to-md.mjs` es idempotente: una segunda ejecución no cambia nada.
- Ningún script adivina un destino de enlace. Si la coincidencia no es única, se reporta y
  se deja el enlace como estaba. Un enlace equivocado es peor que uno roto.
- El frontmatter de Obsidian (`links: "[[...]]"`) queda exento de la migración a propósito:
  ese tipo de propiedad es global en el vault y reescribirla lo corrompe.
