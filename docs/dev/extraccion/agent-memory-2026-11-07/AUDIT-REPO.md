---
title: "Auditoría de Repositorio e Integridad — Campaña de Extracción"
kind: research
status: active
description: "Estado de commits, índices generados, reglas de gitignore y enlaces del árbol docs/dev/extraccion"
tags: [extraccion, audit]
---

# Auditoría de Repositorio e Integridad — Campaña de Extracción

> **Corte auditado:** `HEAD` en `develop` (`799f4a187`, 2026-11-22; último commit sobre `extraccion/` + índices: `f7eafc3e6`). Auditoría estática de solo lectura sobre Git, índices derivados, `.gitignore`, grafo de enlaces y consistencia documental; sin ejecución de builds, tests ni commits.

## 1. Resumen ejecutivo

| Dimensión | Veredicto | Evidencia verificada |
|---|---|---|
| Árbol Git `docs/dev/extraccion/` | Conforme | 124 archivos trackeados (115 `.md` + 9 `.json`), worktree limpio (`0` modificados/untracked/ignorados bajo `extraccion/`) |
| Índices generados (`docs/index.md`, `llms.txt`) | Conforme | 115/115 documentos `.md` listados 1-a-1 en ambos archivos (`3` `index`, `93` `research`, `19` `review`), `0` rutas rotas locales |
| Regla `.gitignore` para `backlogs/` | Conforme (con deuda colateral fuera del árbol) | `.gitignore:146-147` declara `*logs/` seguido de `!docs/dev/extraccion/backlogs/`; `git check-ignore` devuelve vacío |
| Referencias legacy (`research/agent-memory`, `../../Backlog-OpenViking`) | Conforme (con menciones textuales históricas a `.gitignore:268`) | `0` enlaces rotos a `docs/dev/research/agent-memory` o `../../Backlog-OpenViking`; `0` wikilinks `[[...]]` |
| Enlace a `../../avance/investigaciones.md` | Hallazgo medio (tracked-but-ignored) | Presente en [README.md](README.md) L18 (no en [../README.md](../README.md)); destino force-tracked en Git pero directorio ignorado en `.gitignore:280` |
| Sincronización de estado `41+1` vs `42/42` | Desincronizado (4 archivos) | [REVIEW-COVERAGE.md](REVIEW-COVERAGE.md) declara `42/42` (Meilisearch E3 acreditado 2026-11-20), pero [README.md](README.md), [EXPANSION-STATUS.md](EXPANSION-STATUS.md), [CONSOLIDATED.md](CONSOLIDATED.md) e `investigaciones.md` conservan `41+1` |

## 2. Historial de commits recientes (`extraccion/` e índices)

| Commit | Descripción | Alcance |
|---|---|---|
| `f7eafc3e6` | `docs(index): regenerate llms.txt missed from extraccion gate fix` | Sincroniza `llms.txt` tras el cierre de gates documentales |
| `8b4ff7fbc` | `docs(extraccion): close docs-gates — frontmatter kinds, markdownlint, internal links` | Ajusta frontmatter/lint/links en 43 archivos y regenera `docs/index.md` |
| `b76bd1a1e` | `docs(extraccion): cerrar Meilisearch E3, Milvus X-MIL-07 y cobertura` | Pasa [REVIEW-COVERAGE.md](REVIEW-COVERAGE.md) a `42 con límites` y regenera índices |
| `6b80d2e05` | `docs(extraccion): letta-code acreditado en checkout local 17436c8e4...` | Acredita `letta-code` y servidor V1 archive en [letta.md](letta.md) y su backlog |
| `1a4d91fb7` | `docs(extraccion): acreditar mecanismos del servidor Letta V1 archive...` | Amplía evidencia V1 archive en informe y backlog de Letta |
| `0405f4aa7` | `docs(extraccion): acreditar eje E3 de incidencias de forge en informe Meilisearch` | Incorpora §12.1 en [meilisearch.md](meilisearch.md) y actualiza backlog |
| `ca7400f9d` | `docs(extraccion): fichas completas en backlogs restantes (lotes A, D, E)` | Completa 21 backlogs en `docs/dev/extraccion/backlogs/` |
| `466726e70` | `docs(extraccion): fichas completas en 8 backlogs (...)` | Completa 8 backlogs (incluido `openviking.md`) |
| `83218f1e4` | `docs(extraccion): fichas completas en 7 backlogs (...)` | Completa 7 backlogs |
| `81623ec33` | `docs(extraccion): fichas completas en 7 backlogs (...)` | Completa 7 backlogs |
| `4a1656b6a` | `docs(index): regenerar llms.txt con la carpeta extraccion incluida` | Primera inclusión del árbol `extraccion/` en `llms.txt` |
| `a1812a793` | `docs(extraccion): completar revisiones profundas y backlogs por proyecto` | Crea los 43 backlogs, [../README.md](../README.md) y la negación `!docs/dev/extraccion/backlogs/` en `.gitignore` |
| `5724b70e4` | `docs(extraccion): agrupar investigacion comparativa y backlog OpenViking` | Mueve `docs/dev/research/agent-memory-2026-11-07/` y [../Backlog-OpenViking-VantaDB.md](../Backlog-OpenViking-VantaDB.md) a `docs/dev/extraccion/` |

## 3. Verificación de índices generados (`docs/index.md` y `llms.txt`)

El inventario físico y en Git bajo `docs/dev/extraccion/` suma **115 documentos Markdown**:

- `2` en raíz ([../README.md](../README.md), [../Backlog-OpenViking-VantaDB.md](../Backlog-OpenViking-VantaDB.md)).
- `50` en `agent-memory-2026-11-07/` (`8` documentos de control + `42` informes de proyectos).
- `19` en `agent-memory-2026-11-07/reviews/`.
- `44` en `backlogs/` ([../BACKLOG-FORMAT.md](../BACKLOG-FORMAT.md) + `43` backlogs por proyecto/OpenViking).

| Índice generado | Entradas `extraccion/` | Cobertura | Observaciones |
|---|---:|---|---|
| `docs/index.md` | 115 | 115/115 (100%) | Rutas relativas `./dev/extraccion/...` válidas; `0` enlaces rotos |
| `llms.txt` | 115 | 115/115 (100%) | URLs raw `https://raw.githubusercontent.com/ness-e/Vantadb/main/docs/dev/extraccion/...` generadas por `scripts/docs/gen-index.mjs:316` (resolverán en remoto al fusionar `develop` en `main`) |

> **Nota operativa:** al incorporar este mismo archivo `AUDIT-REPO.md` al árbol (documento #116), debe ejecutarse `node scripts/docs/gen-index.mjs --write` antes de validar `gen-index.mjs --check` o crear un commit.

## 4. Auditoría de `.gitignore` e invariante tracked-ignored

1. **Excepción `backlogs/` correcta:** `.gitignore:146` define el patrón heredado amplio `*logs/` e inmediatamente después `.gitignore:147` declara `!docs/dev/extraccion/backlogs/` (introducido en `a1812a793`). `git check-ignore -v --no-index docs/dev/extraccion/backlogs/README.md` devuelve vacío.
2. **Invariante `tracked-but-ignored` — RESUELTO en la ola de consolidación (2026-11-23):** `.gitignore:9-10,113-114` exige `git ls-files -c -i --exclude-standard -> 0` porque `release-plz` rechaza archivos trackeados-pero-ignorados como worktree sucio. Los **4 archivos** detectados quedaron des-ignorados con excepciones puntuales (regla del directorio convertida a `dir/*` + `!ruta` explícita, sin abrir los directorios scratch):
   - `docs/dev/avance/investigaciones.md` — `docs/dev/avance/*` + `!docs/dev/avance/investigaciones.md`.
   - `docs/dev/reviews/2026-11-03-forensic-verification-external-claims.md` — `docs/dev/reviews/*` + `!docs/dev/reviews/2026-11-03-forensic-verification-external-claims.md`.
   - `metricas/README.md` — `**/metricas/*` + `!metricas/README.md` (la forma `dir/` impedía la re-inclusión).
   - `tests/fixtures/v0.9.0/db/data/vector_index.bin` — `!tests/fixtures/v0.9.0/db/data/vector_index.bin` tras `!tests/fixtures/v0.9.0/db/`.

   Verificación: `git ls-files -c -i --exclude-standard` devuelve vacío y `git check-ignore` confirma que los archivos vecinos (`metricas/foo.tmp`, `docs/dev/avance/otro.md`, `docs/dev/reviews/otro.md`) siguen ignorados.

## 5. Auditoría de enlaces internos y referencias legacy

- **`docs/dev/research/agent-memory` y `../../Backlog-OpenViking`:** **0 coincidencias** en todo `docs/dev/extraccion/`. Todas las referencias al estudio central usan `[../Backlog-OpenViking-VantaDB.md](../Backlog-OpenViking-VantaDB.md)` o `Backlog-OpenViking-VantaDB.md` según el nivel del archivo.
- **Wikilinks `[[...]]`:** **0 coincidencias** en todo `docs/dev/extraccion/`.
- **Enlaces `../../` hacia código y docs del repositorio:** verificados (47 ocurrencias hacia `src/`, `vanta-memory/`, `vantadb-mcp/`, `evals/` y `docs/user/operations/BENCHMARKS.md`), todos con profundidad relativa exacta (`../../../` desde la raíz de `extraccion/` y `../../../../` desde `agent-memory-2026-11-07/`).
- **Caso `../../avance/investigaciones.md`:**
  - **Ubicación real:** [../README.md](../README.md) (`docs/dev/extraccion/README.md`) **no contiene** este enlace. El enlace reside en [README.md](README.md) línea 18 (`../../avance/investigaciones.md#agent-memory-2026-11-07-evaluación-comparativa-de-memoriacontexto-para-agentes`), donde la ruta relativa y el ancla (`investigaciones.md:224`) resuelven sintácticamente.
  - **Riesgo de repositorio:** `docs/dev/avance/` está listado en `.gitignore:280` como scratch local de agente, pero `docs/dev/avance/investigaciones.md` está **force-tracked en el índice Git** (`100644 07516751ea...`, editado en `8b4ff7fbc`). En consecuencia, aunque hoy no produciría 404 en GitHub ni dispara `check-links.mjs` (`buildTargetSet()` lee `git ls-files`), rompe la invariante de `release-plz` (`tracked-but-ignored`) y quedará roto (404) si se sanea el índice con `git rm --cached` sin eliminar el enlace o añadir negación explícita en `.gitignore`.
- **Residuo textual pre-migración:** [README.md](README.md) L120 y nueve recibos bajo `reviews/` mencionan en prosa histórica que `.gitignore:268` ignora `docs/dev/research/` y que la entrega se incorporó con `force-add`. En `HEAD` la línea es `.gitignore:277` y `docs/dev/extraccion/` se trackea de forma nativa sin `force-add`.

## 6. Desincronización de cobertura: `41+1` vs `42/42`

Tras los commits `0405f4aa7` (acreditación del eje E3 de incidencias en [meilisearch.md](meilisearch.md) L45–48 y §12.1) y `b76bd1a1e`, el estado real del corpus pasó a **42/42 informes aceptados con límites**, pero cuatro documentos no se actualizaron:

| Archivo y líneas | Estado declarado actualmente | Estado real acreditado | Acción requerida |
|---|---|---|---|
| [REVIEW-COVERAGE.md](REVIEW-COVERAGE.md) L19–25, L97, L123–125 | **42 aceptados con límites** (Meilisearch E3 acreditado 2026-11-20) | Vigente (fuente actualizada en `b76bd1a1e`) | Mantener; revisar nota colateral de `codebase-memory-mcp` en L89/L130 frente a §11.5 (`7764b490b`) |
| [README.md](README.md) L13, L111, L119 | **41 con límites + 1 limitado** (Meilisearch E3 no acreditado por prohibición de forge) | Desactualizado | Actualizar a 42 aceptados con límites (E3 acreditado 2026-11-20 con alcance residual en §12.1) y corregir nota `.gitignore` de L120 |
| [EXPANSION-STATUS.md](EXPANSION-STATUS.md) L18–21 (y referencias L40, L45) | **41 con límites y 1 limitado** en bloque `Estado vigente` | Desactualizado | Añadir bloque vigente 2026-11-20 (`42 con límites`) reclasificando Tanda 8 (`41+1`) como antecedente |
| [CONSOLIDATED.md](CONSOLIDATED.md) L11, L213 | **41 aceptados con límites** y Meilisearch E3 **NO acreditado** | Desactualizado | Sincronizar cabecera L11 y fila Meilisearch de la tabla §3 (L213) con §12.1 de [meilisearch.md](meilisearch.md) |
| `docs/dev/avance/investigaciones.md` L229 | **41 aceptados con límites** + Meilisearch E3 **NO acreditado** | Desactualizado | Sincronizar cierre documental (y resolver su condición `tracked-but-ignored` en `.gitignore`) |
