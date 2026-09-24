# Backlog Notion — tareas sobre páginas VantaDB Docs

> **Propósito:** registrar todo el trabajo pendiente sobre las páginas Notion (edición, mejora, investigación y sincronización con código). Fuente: análisis integral 2026-09-18 (código vs Backlog vs Notion).
> **Regla:** las páginas de lanzamiento (posts, changelogs, FAQ pública) son messaging crítico — cada edición se verifica contra código antes de darse por hecha. Los posts de anuncio siguen pausados hasta el gate Fase A (decisión 2026-09-08, vigente).
> **Convención:** `N-xx` = tarea Notion. `⬜` pendiente · `⏳` en curso · `✅` hecha (se elimina la fila; el registro queda en `docs/dev/avance/`).

## Actualización de ejemplos (API vieja → `Client`/`search`)

| ID | Página(s) | Alcance | Verificación | Estado |
|----|-----------|---------|--------------|--------|
| `N-03` | Plan 00/03 (posts HN/Reddit/Twitter/video/guion) | Reescribir claims contra API actual cuando se vayan a usar. Regla vigente: solo features de la release anunciada. Siguen pausados hasta gate Fase A | Cero funciones no existentes por post | ⬜ Pendiente (bloqueado por gate Fase A) |

## Roadmap y Propuesta (sincronización con código)

| ID | Página(s) | Alcance | Verificación | Estado |
|----|-----------|---------|--------------|--------|
## Research (brechas del análisis, sin tarea en ningún backlog)

| ID | Tema | Alcance | Origen | Estado |
|----|------|---------|--------|--------|
| `N-07` | Meta-memoria operativa | P(IK)/abstención selectiva/revalidación proactiva sobre memoria (Nelson-Narens monitoring-control, Kadavath P(IK), Self-Refine/Reflexion como referencia). Candidata a MGR-26 o dentro de MGR-12 | Dim 8 (sin research específica) | ⬜ Pendiente |
| `N-08` | Ranking temporalmente consciente | La literatura advierte dilución por post-filtro temporal (R@10 80%→37.5% en reasoning). Diseñar ranking que no traiga contexto irrelevante al filtrar por `valid_at` | Dim 5/ámbito 4 | ⬜ Pendiente |
| `N-09` | Fórmula de decaimiento de confianza | Ninguna fórmula canónica validada para memoria semántica; definir decay + revalidación (entra con MGR-12) | Dim 6 | ⬜ Pendiente |
| `N-10` | Benchmark entity-memory longitudinal | Probar identidad a través de sesiones/cambios de nombre/contradicciones con fusiones reversibles puntuadas (no existe estándar; LongMemEval/LoCoMo como base) | Dim 7 | ⬜ Pendiente |

> Nota: eval de ingesta (LoCoMo) = EXE-02 y test cascada multiagente = EXE-07, ya en `docs/dev/Backlog.md` — no se duplican aquí.

## Re-validación continua

| ID | Alcance | Estado |
|----|---------|--------|
| `N-11` | Re-validar "Análisis de cobertura" de las dims (fecha 2026-09-14) tras cada slice IMPL-MGR que cambie un estado REAL/PARCIAL/PROPUESTA | ⬜ Pendiente (recurrente) |
