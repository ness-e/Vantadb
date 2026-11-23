---
title: "Auditoría Integral Consolidada — Campaña de Extracción (Ola 4)"
kind: research
status: active
description: "Resumen consolidado de los cinco ejes de cierre: licencias, integridad del repo, trazabilidad de los 43 backlogs, cobertura 42/42 y estado de la ola de correcciones"
tags: [extraccion, audit, licenses, backlogs]
---

# Auditoría Integral Consolidada — Campaña de Extracción (Ola 4)

Resumen de cierre de la ola de consolidación documental (workers W1–W5). Este
documento **no re-deriva** datos: remite a las fuentes verificadas
[backlogs/LICENSES.md](backlogs/LICENSES.md) (matriz legal),
[agent-memory-2026-11-07/AUDIT-REPO.md](agent-memory-2026-11-07/AUDIT-REPO.md)
(integridad de árbol, índices, `.gitignore`, enlaces y cobertura) y al `git diff`
de la ola (corrección de backlogs y nuevos documentos).

## 1. Licencias del corpus (43 proyectos)

Fuente: [backlogs/LICENSES.md](backlogs/LICENSES.md) — verificación local de los
`LICENSE*`/`NOTICE*` en los clones fijados por SHA, cruzada con las 265 filas
`X-*` frente a la licencia `Apache-2.0` de VantaDB.

| Categoría | Proyectos | Filas `X-*` | Gate |
|---|---:|---:|---|
| Permisivas (`MIT`, `Apache-2.0`, `PostgreSQL`, `BSD-3-Clause` permisiva, `MIT` local Tencent) | 34 | 209 | Compatible; reimplementación propia y `NOTICE`/copyright si hay obra derivada |
| Copyleft fuerte | 2 (OpenViking `AGPL-3.0`, Khoj `AGPL-3.0`+`GPL-3.0`) | 21 | **Incompatible**: solo especificación *clean-room* |
| Source-available / modificada | 3 (SurrealDB `BSL-1.1`, Redis tri-licencia `RSALv2/SSPL-1.0/AGPL-3.0`, Dify `Apache-2.0` modificada) | 14 | **Incompatible**: solo contratos funcionales abstractos |
| Mixta por componente | 4 (Meilisearch `MIT`+`BSL-1.1`, Weaviate `BSD`+`wl/` privativa, Flowise `Apache-2.0`+Commercial, AutoGen `MIT`+`CC-BY-4.0`) | 21 | Permitida solo sobre subárbol permisivo verificado (`0/21` filas en rutas restringidas) |
| `NOASSERTION` en raíz | 0 | 0 | Resueltas todas; 4 submódulos no clonados permanecen vedados |
| **Total** | **43** | **265** | — |

Obligaciones derivadas (detalle en §3–§4 de `LICENSES.md`): bloqueo *clean-room*
para 35 filas (`X-OVK-01..14`, `X-KHO-01..07`, `X-SUR-01..04`, `X-RED-01..05`,
`X-DIF-01..05`), guardia de subárbol para 21 filas mixtas (`*enterprise_edition*`,
`wl/**`, `enterprise/**`, `IdentityManager.ts`, docs `CC-BY-4.0`) y política
general permisiva para las 209 restantes.

## 2. Integridad del repositorio

Fuente: [AUDIT-REPO.md](agent-memory-2026-11-07/AUDIT-REPO.md). En el corte
auditado el árbol `docs/dev/extraccion/` contaba 124 archivos trackeados (115
`.md` + 9 `.json`), con `docs/index.md` y `llms.txt` al 115/115, `0` enlaces
rotos, `0` wikilinks y la excepción `!docs/dev/extraccion/backlogs/` correcta.

Hallazgos resueltos en esta ola:

- **Invariante `tracked-but-ignored`:** los cuatro archivos trackeados pero
  ignorados (`docs/dev/avance/investigaciones.md`,
  `docs/dev/reviews/2026-11-03-forensic-verification-external-claims.md`,
  `metricas/README.md` y `tests/fixtures/v0.9.0/db/data/vector_index.bin`)
  quedaron des-ignorados mediante excepciones puntuales en `.gitignore`
  (`*` sobre el directorio + `!ruta` explícita), sin abrir los directorios
  scratch. `git ls-files -c -i --exclude-standard` devuelve vacío y
  `git check-ignore` confirma que los vecinos siguen ignorados.
- **Sincronización `41+1` → `42/42`:** README, EXPANSION-STATUS, CONSOLIDATED e
  `investigaciones.md` se actualizaron al estado real acreditado en `b76bd1a1e`.

## 3. Trazabilidad de los backlogs

- **43 backlogs** por proyecto bajo [backlogs/](backlogs/) en el formato
  canónico de 10 columnas de `docs/dev/Backlog.md` (convenciones en
  [backlogs/README.md](backlogs/README.md)).
- **265 filas `X-*`** con **265 fichas 1:1** completadas en los lotes A/B/C
  (`ca7400f9d`, `466726e70`, `83218f1e4`, `81623ec33`), cada una con evidencia
  `archivo:líneas` sobre el clon fijado por SHA.
- Correcciones de esta ola: `backlogs/openviking.md` — `X-OVK-11` con
  dependencias justificadas (cliente Studio existente y contratos de lectura del
  backend) y rutas abreviadas expandidas a ruta completa verificable en el clon
  (`web-studio/src/features/workbench/sessions/lib/memory-diff.ts`).

## 4. Cobertura de revisión

Estado vigente: **42/42 informes aceptados con límites** (Meilisearch E3
acreditado 2026-11-20 en `0405f4aa7`/`b76bd1a1e`), según
[REVIEW-COVERAGE.md](agent-memory-2026-11-07/REVIEW-COVERAGE.md) y
[EXPANSION-STATUS.md](agent-memory-2026-11-07/EXPANSION-STATUS.md). La
aceptación es documental: las 265 filas `X-*` permanecen `⬜ Candidata` y
requieren revalidación contra `HEAD` antes de especificar; nada de lo aquí
propuesto está implementado ni aprobado.

## 5. Documentos de la ola

| Documento | Rol |
|---|---|
| [backlogs/LICENSES.md](backlogs/LICENSES.md) | Matriz legal verificada de los 43 proyectos y gate previo a spec |
| [agent-memory-2026-11-07/AUDIT-REPO.md](agent-memory-2026-11-07/AUDIT-REPO.md) | Auditoría de árbol, índices, `.gitignore` y enlaces |
| Este documento | Resumen consolidado de los cinco ejes para cierre de la ola |
