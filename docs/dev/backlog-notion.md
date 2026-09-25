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
| `N-17` | **Sync 2026-09-24 (10 páginas + higiene)** — aplicar los borradores de `docs/dev/strategy/NOTION-SYNC-2026-09-24.md`: Problema (evidencia autoenvenenamiento/ventana/benchmarks quemados), Propuesta (matriz 2026-09-24 + decisiones owner + Anexo C Memory Contracts), Roadmap (P52–P56 + gates), SDKs, Benchmarks (caveats + planificado), Seguridad, Observabilidad, Gobernanza, Casos de uso (3 tracks ICP), Definición oficial (tagline + North Star); consolidar `VantaDB Docs (1)` y archivar `VantaDB OLD` | Claims verificados contra código (Regla 11) + fecha + links cruzados al repo | ⬜ Pendiente (2026-09-24) |
## Research (brechas del análisis, sin tarea en ningún backlog)

| ID | Tema | Alcance | Origen | Estado |
|----|------|---------|--------|--------|
| `N-07` | Meta-memoria operativa | P(IK)/abstención selectiva/revalidación proactiva sobre memoria (Nelson-Narens monitoring-control, Kadavath P(IK), Self-Refine/Reflexion como referencia). Candidata a MGR-26 o dentro de MGR-12 | Dim 8 (sin research específica) | ⬜ Pendiente (→ trackeado en MGR-18 P49 + abstención en SCH-05 P53) |
| `N-08` | Ranking temporalmente consciente | La literatura advierte dilución por post-filtro temporal (R@10 80%→37.5% en reasoning). Diseñar ranking que no traiga contexto irrelevante al filtrar por `valid_at` | Dim 5/ámbito 4 | ⬜ Pendiente (→ trackeado en MGR-11 P49 / WIRE-08 P56) |
| `N-09` | Fórmula de decaimiento de confianza | Ninguna fórmula canónica validada para memoria semántica; definir decay + revalidación (entra con MGR-12) | Dim 6 | ⬜ Pendiente (→ trackeado en MGR-11/12 P49 / SCH-04 P53; sin canónico — sigue abierto) |
| `N-10` | Benchmark entity-memory longitudinal | Probar identidad a través de sesiones/cambios de nombre/contradicciones con fusiones reversibles puntuadas (no existe estándar; LongMemEval/LoCoMo como base) | Dim 7 | ⬜ Pendiente (→ base en VER-08 P52; sigue sin estándar externo) |

> Nota: eval de ingesta (LoCoMo) = EXE-02 y test cascada multiagente = EXE-07, ya en `docs/dev/Backlog.md` — no se duplican aquí.

## Benchmarks y credibilidad (post-investigación 2026-09-24)

> Lo que NINGÚN benchmark público mide hoy (mem0.ai/blog/ai-memory-benchmarks-in-2026) y VantaDB va a medir y publicar (VER-08/VER-09). Cada fila = contenido para la página `Benchmarks` de Notion.

| ID | Tema | Alcance | Verificación | Estado |
|----|------|---------|--------------|--------|
| `N-12` | Benchmarks propios publicados | Protocolo VantaDB (dataset commiteado, comandos de reproducción) + resultados canonical_p99/LoCoMo/LongMemEval-S/BEAM-subset en la página Benchmarks | Reproducible desde repo (Regla 11) | ⬜ Pendiente (→ VER-08 P52) |
| `N-13` | Write-quality (calidad de escritura) | Medir qué se guarda vs qué se descarta (extracción selectiva) — nadie lo mide | Reporte con métrica definida | ⬜ Pendiente (→ VER-08 P52) |
| `N-14` | Abstención | LongMemEval `_abs`: declinar correctamente ante eventos inexistentes; liga con SCH-05 | Subscore publicado | ⬜ Pendiente (→ VER-08/SCH-05) |
| `N-15` | Aislamiento per-user | Multi-usuario concurrente sin filtraciones entre tenants — ningún benchmark público lo cubre | Test de aislamiento documentado | ⬜ Pendiente (→ VER-08 + tests RBAC existentes) |
| `N-16` | Token-economy | Pares accuracy+tokens por retrieval (no accuracy sola); comparar contra full-context (~25K tokens) | Tabla por sistema con tokens mean | ⬜ Pendiente (→ VER-08/VER-09) |

## Re-validación continua

| ID | Alcance | Estado |
|----|---------|--------|
| `N-11` | Re-validar "Análisis de cobertura" de las dims (fecha 2026-09-14) tras cada slice IMPL-MGR que cambie un estado REAL/PARCIAL/PROPUESTA | ⬜ Pendiente (recurrente) |
