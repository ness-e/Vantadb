---
title: "Propuesta de limpieza de artefactos documentales"
type: review
status: pending-owner-approval
date: 2026-08-22
source: "GOV-E1 — docs/plans/2026-08-22-doc-governance-plan.md"
audit_source: "docs/reviews/auditoria-documentacion-2026-08-21.md (V1+II)"
---

# Propuesta de limpieza de artefactos — 2026-08-22

> **Este documento es SOLO una propuesta. Ningún archivo fue borrado ni movido en este PR.**
> Cada ítem requiere aprobación explícita del owner (checkbox) antes de ejecutar la acción recomendada.
>
> **Metodología (Regla 0):** para cada candidato se verificó existencia real (`Test-Path`), estado de
> tracking en git (`git ls-files`), cobertura de `.gitignore`, y referencias entrantes/salientes
> (`rg` global sobre todo el workspace, incluyendo `.opencode/`). Los hallazgos **corrigen** tres
> supuestos de la auditoría V1+II del 2026-08-21; las correcciones están marcadas por ítem.

## Resumen ejecutivo

| # | Candidato | Tracked | Acción propuesta | Riesgo |
|---|-----------|---------|------------------|--------|
| 1 | `docs/book/book/` (~83 archivos build) | ❌ no | Borrado local opcional (ya ignorado) | 🟢 nulo |
| 2 | `docs/examples/__pycache__/` (1 .pyc) | ❌ no | Borrado local opcional (ya ignorado) | 🟢 nulo |
| 3 | `docs/TDAM-VANTADB/` (vacía) | ❌ no | `rmdir` local opcional | 🟢 nulo |
| 4 | `docs/benchmarks/_run_stdout.md` (8 KB) | ✅ sí | `git rm` tras aprobación | 🟢 bajo |
| 5 | `docs/web/DESIGN_RULES.md` (16.5 KB) | ✅ sí | Extraer único → standards/, luego archivar + fix link en master-index | 🟡 medio |
| 6 | `docs/.obsidian/` (18 items) | ❌ no | Mantener local (ya ignorado); decisión owner solo si quiere compartir config | 🟢 nulo |
| 7 | Stubs `book/src/blog` + `case_studies` | ✅ sí | **RESUELTO** por GOV-B1 (commit 8b21733) — sin acción | — |

**Conclusión:** de los 7 candidatos originales, solo el #4 requiere una acción de repo real
(`git rm`), y el #5 requiere edición de contenido. Los ítems 1, 2, 3 y 6 ya están cubiertos
por `.gitignore` y nunca fueron commiteados: son ruido exclusivamente local.

---

## Ítem 1 — `docs/book/book/` (output compilado mdBook)

### Contenido
97 entradas totales, 83 archivos: 51 `.html`, 8 `.css`, 8 `.js`, 11 `.woff2`, 1 `.nojekyll`,
1 `.png`, 1 `.svg`, 2 `.txt`. Es el build output regenerable de `mdbook build docs/book`.

### Corrección a la auditoría
La auditoría V1+II lo describe como "~90 archivos html/css/fonts **commiteado**". Verificación
mecánica: `git ls-files docs/book/book` → **0 archivos trackeados**. El `.gitignore` ya lo cubre
(línea 15: `docs/book/book/`). No hay nada que des-commitear.

### Referencias entrantes (rg global)
Solo menciones textuales/históricas, ninguna dependencia funcional:
- `docs/reviews/auditoria-documentacion-2026-08-21.md` (este mismo hallazgo)
- `docs/master-index.md` (mención)
- Snapshots históricos: `docs/progreso/campanas/engineering-health-waves.md`,
  `docs/avance/historial/snapshot-*.md`, planes archivados

### Referencias salientes
Ninguna (es output generado).

### Veredicto recomendado
✅ **Borrar localmente cuando se desee.** Sin acción git necesaria. Regenerable con
`mdbook build docs/book`. Opción adicional: confirmar que ningún entorno CI espera el path
(verificado: no hay referencias funcionales en workflows ni scripts).

### Riesgo
🟢 Nulo — contenido regenerable, ignorado por git.

- [ ] ☐ Aprobación owner — borrado local de `docs/book/book/`

---

## Ítem 2 — `docs/examples/__pycache__/`

### Contenido
1 archivo: `fnd05_python_context_manager.cpython-311.pyc` — bytecode compilado residual de haber
ejecutado el ejemplo Python in-place.

### Corrección a la auditoría
No trackeado (`git ls-files` = 0). Ya cubierto por `.gitignore` línea 43: `**/__pycache__/`.
No es deuda de repo, solo basura local.

### Referencias entrantes
- `dev-tools/scripts/collect_code.ps1` (excluye `__pycache__` genéricamente — no depende de este dir)
- Mención en auditoría-documentación

### Referencias salientes
Ninguna.

### Veredicto recomendado
✅ **Borrar localmente.** Sin acción git. El `.pyc` se regenera solo si alguien vuelve a correr
el ejemplo desde ese directorio.

### Riesgo
🟢 Nulo.

- [ ] ☐ Aprobación owner — borrado local de `docs/examples/__pycache__/`

---

## Ítem 3 — `docs/TDAM-VANTADB/`

### Contenido
Directorio completamente vacío (0 items con `-Force`, 0 trackeados). Probable residuo de una
campaña TDAM cuyo contenido fue reubicado o eliminado previamente.

### Referencias entrantes
- `docs/master-index.md` (¿lo indexa? verificar al ejecutar GOV-C4-style sweep — hoy solo mención textual)
- Auditoría-documentación

### Referencias salientes
Ninguna (vacío).

### Veredicto recomendado
✅ **`rmdir docs/TDAM-VANTADB` local.** Windows no versiona directorios vacíos, así que tampoco hay
acción git. Si master-index lo lista, remover la fila en la próxima pasada de índice.

### Riesgo
🟢 Nulo — vacío y no versionado.

- [ ] ☐ Aprobación owner — eliminar `docs/TDAM-VANTADB/`

---

## Ítem 4 — `docs/benchmarks/_run_stdout.md`

### Contenido
8,172 bytes. Log crudo de stdout de una corrida de benchmark, incluye un traceback. Único
candidato que **sí está trackeado en git**.

### Referencias entrantes
- `docs/reviews/auditoria-documentacion-2026-08-21.md` (única referencia en todo el workspace)

### Referencias salientes
Ninguna funcional (es un volcado de log).

### Veredicto recomendado
✅ **`git rm docs/benchmarks/_run_stdout.md`** tras aprobación. Es un artefacto efímero commiteado
por accidente; los resultados canónicos de benchmarks viven en `docs/operations/BENCHMARKS.md`
(Regla 9/11). Antes de borrar: confirmar que el traceback no documenta un bug pendiente — si lo
hace, extraerlo a un ticket primero.

### Riesgo
🟢 Bajo — sin inbound refs fuera de la auditoría; historial preservado en git de todos modos.

- [ ] ☐ Aprobación owner — `git rm docs/benchmarks/_run_stdout.md`

---

## Ítem 5 — `docs/web/DESIGN_RULES.md` vs `docs/web/standards/design-rules.md`

### Contenido
Los dos archivos **NO son duplicados literales** (corrección a la auditoría, que dice "raíz duplica
standards/design-rules.md"):

| Archivo | Tamaño | Contenido real |
|---|---|---|
| `docs/web/DESIGN_RULES.md` | 16.5 KB | Tutorial conceptual en español: terminología correcta de visualización de información, guía de rediseño, checklist. Frontmatter `language: es` (viola Doc Language Split) |
| `docs/web/standards/design-rules.md` | 10.5 KB | Reglas técnicas de frontend: arquitectura CSS Tailwind v4, utility classes, animaciones, anti-patrones |

Solapan en tema general (diseño web VantaDB) pero no en contenido: ninguno sustituye al otro.

### Referencias entrantes
- `docs/master-index.md:293` — **link vivo**: `[DESIGN_RULES.md](web/DESIGN_RULES.md)` ← se romperá si se elimina sin actualizar
- Menciones históricas (sin link): bitácora, sesiones 2026-07, plan archivado backlog-pipeline,
  research/investigacion-equipo (nota mojibake)

### Referencias salientes
`DESIGN_RULES.md` referencia recursos externos de diseño (sección "Recursos para Profundizar").
`standards/design-rules.md` referencia código de `web/`.

### Veredicto recomendado
⚠️ **Fusionar selectivamente y archivar**, no borrar directo:
1. Revisar si `DESIGN_RULES.md` tiene material único aprovechable → moverlo a un doc apropiado
   en inglés bajo `docs/web/guides/` (o descartarlo si es redundante con brand-identity.md).
2. `git mv docs/web/DESIGN_RULES.md docs/archive/` (preserva historial).
3. Actualizar `docs/master-index.md:293`: apuntar a `web/standards/design-rules.md` o quitar la fila.
4. Sweep AUD-007 post-movimiento = 0 links rotos.

Nota adicional: el archivo tiene mojibake detectado desde 2026-08-09
(`research/investigacion-equipo-2026-08-09.md:157`) — argumento extra para archivarlo en vez de repararlo.

### Riesgo
🟡 Medio — único ítem con link entrante vivo (master-index) y decisión editorial de qué conservar.
Mitigación: pasos ordenados arriba + verificación mecánica de links.

- [ ] ☐ Aprobación owner — fusionar/archivar `docs/web/DESIGN_RULES.md`

---

## Ítem 6 — `docs/.obsidian/` (config de vault Obsidian)

### Contenido
18 entradas: `app.json`, `appearance.json`, `community-plugins.json`, `core-plugins.json`,
`graph.json`, `templates.json`, `workspace.json`, carpeta `plugins/`. Configuración personal del
vault Obsidian con el que el owner navega `docs/`.

### Corrección a la auditoría / estado real
No trackeado (`git ls-files` = 0) y ya ignorado (`.gitignore:161: .obsidian/`). La pregunta de la
auditoría ("sacar del repo vs mantener") está **ya respondida de facto**: nunca entró al repo.

### Referencias entrantes
- `docs/master-index.md` — exclusión deliberada documentada (correcto)
- `docs/research/human-facing-db-ui/*/RESEARCH.md` — menciones contextuales a Obsidian como herramienta
- Historiales internos

### Referencias salientes
Ninguna funcional hacia el repo.

### Veredicto recomendado
✅ **Mantener tal cual** (local, ignorado). Alternativa si el owner quiere portabilidad del vault:
commitear solo los settings estables (`app.json`, `core-plugins.json`) excluyendo `workspace.json`
(máquina-específico) — pero eso es opt-in, no limpieza.

### Riesgo
🟢 Nulo — no afecta el repo mientras siga ignorado.

- [ ] ☐ Decisión owner — mantener ignorado (recomendado) / commitear settings mínimos

---

## Ítem 7 — Stubs redirect `book/src/blog` y `book/src/case_studies`

### Estado
**RESUELTO — NO es candidato.** Aplicado hoy por GOV-B1 (commit `8b21733`):
- `case_studies/*.md` → stubs `ARCHIVED - see docs/archive/case-studies-unverified/` (los case
  studies ficticios fueron movidos a archive interno con README disclaimer).
- `blog/*.md` → stubs `{{#include ../../../blog/...}}` a las fuentes reales en `docs/blog/`.

Verificado en esta sesión: los 7 archivos stub existen y cumplen esa función; el TOC del book
(SUMMARY.md) sigue resolviendo sin links muertos públicos.

### Acción
Ninguna. Se documenta aquí únicamente para cerrar la lista de 7 de la auditoría.

- [x] ☑ RESUELTO (GOV-B1, commit 8b21733) — sin aprobación requerida

---

## Plan de ejecución (post-aprobación)

1. Ítems 1-3: `Remove-Item` local (sin git).
2. Ítem 4: ticket → `git rm docs/benchmarks/_run_stdout.md` (lead ejecuta).
3. Ítem 5: tarea dedicada (extraer → `git mv` a archive → fix master-index:293 → sweep AUD-007).
4. Ítem 6: sin acción (o ticket menor si owner elige commitear settings).

Cualquier borrado/movimiento real queda **fuera de esta PR** y se ejecuta en tareas separadas
tras las casillas de aprobación correspondientes.
