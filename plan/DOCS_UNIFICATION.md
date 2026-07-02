# Docs Unification Plan

> Evaluación y plan de unificación de los árboles de documentación.

---

## Resumen de la Evaluación

| Conflicto | Veredicto | Acción |
|-----------|-----------|--------|
| `docs/Backlog.md` vs `web/docs/backlog.md` | **NO unificar** — producto vs web son tracks distintos | Cross-links recíprocos |
| `docs/REPORTE_INVESTIGACION...` vs `web/docs/research/ANALISIS...` | **SÍ unificar** — ~90% duplicado | Archivar REPORTE |
| `docs/CHANGELOG.md` vs `changelog.tsx` | **Consistente** — un minor time lag | Actualizar fecha de v0.2.0 |
| `docs/master-index.md` | **Desactualizado** | Actualizar |

## Plan de Acción

### 1. Archivar REPORTE_INVESTIGACION_Y_DECISIONES.md

El documento `docs/REPORTE_INVESTIGACION_Y_DECISIONES.md` (v2.0, 624L) es ~90%
duplicado de `web/docs/research/ANALISIS_COMPLETO_Y_DECISIONES.md` (v4.0, 1499L).

**Acción**:
- Mover `docs/REPORTE_INVESTIGACION_Y_DECISIONES.md` → `docs/archive/REPORTE_INVESTIGACION_Y_DECISIONES.md`
- Dejar un redirect/note en la ubicación original apuntando al canónico
- El canónico es `web/docs/research/ANALISIS_COMPLETO_Y_DECISIONES.md`

### 2. Cross-links entre Backlogs

**docs/Backlog.md** (producto):
- Agregar una sección o referencia: "Web Site" → `web/docs/backlog.md`

**web/docs/backlog.md** (web):
- Ya referencia `docs/REPORTE_INVESTIGACION...` — actualizar a la nueva ubicación
- Agregar referencia a `docs/Backlog.md`

### 3. Actualizar master-index.md

- Bump versión de v0.1.5 → v0.2.0
- Corregir repo URL de `vantadb/vantadb` → `ness-e/Vantadb`
- Agregar sección "Web Site" con link a `web/docs/README.md`

### 4. CHANGELOG.md

- Mover `[Unreleased]` → `[v0.2.0] - 2026-07-02` para reflejar la realidad

### 5. README.md

- El root `README.md` debería mencionar que el proyecto incluye `web/` como frontend
- El root `README_ES.md` igual
