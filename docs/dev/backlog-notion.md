---
title: Backlog Notion — tareas sobre páginas VantaDB Docs
kind: research
---

# Backlog Notion — tareas sobre páginas VantaDB Docs

> **Propósito:** registrar todo el trabajo pendiente sobre las páginas Notion (edición, mejora, investigación y sincronización con código). Fuentes: análisis integral 2026-09-18 + **auditoría completa VantaDB Docs 2026-09-30** (hub + 30 hijas + subtree Propuesta; 4 research + scan de placeholders; skill `notion-mcp-master` + 27 fuentes web) → filas N-18..N-32.
> **Regla:** las páginas de lanzamiento (posts, changelogs, FAQ pública) son messaging crítico — cada edición se verifica contra código antes de darse por hecha. Los posts de anuncio siguen pausados hasta el gate Fase A (decisión 2026-09-08, vigente).
> **Convención:** `N-xx` = tarea Notion. `🆕` pendiente · `⏳` en curso · ✅ hecha (se elimina la fila; el registro queda en `docs/dev/avance/`). Formato: esquema canónico de 10 columnas (`.opencode/references/backlog-format.md`, migrado 2026-09-30).

## Actualización de ejemplos (API vieja → `Client`/`search`)

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `N-03` | — | **Reescribir claims de posts (HN/Reddit/Twitter/video/guion) contra API actual** | — | 🟡 1d | 🟡 Media | ⏸️ Bloqueada: gate Fase A | Páginas Plan 00/03. Regla vigente: solo features de la release anunciada. Reescritura cuando se vayan a usar. | Origen: análisis 2026-09-18 · Ver: FASE-A/R-05 (`Backlog-negocio.md`) | Dep: gate Fase A |

## Roadmap y Propuesta (sincronización con código)

> Sin filas pendientes — `N-17` cerrada 2026-09-30 (sync ejecutado + verificado + Sync #2 validación externa; registro en `docs/dev/avance/activo/operaciones.md`).

## Research (brechas del análisis, sin tarea en ningún backlog)

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `N-07` | — | **Meta-memoria operativa** | — | 🟠 2-3d | 🟡 Media | 🆕 Pendiente | P(IK)/abstención selectiva/revalidación proactiva sobre memoria (Nelson-Narens monitoring-control, Kadavath P(IK), Self-Refine/Reflexion como referencia). Candidata a MGR-26 o dentro de MGR-12. | Origen: Dim 8 (sin research específica) · Trackeado: MGR-18 (P49) + abstención en SCH-05 (P53) | — |
| `N-08` | — | **Ranking temporalmente consciente** | — | 🟠 2-3d | 🟡 Media | 🆕 Pendiente | La literatura advierte dilución por post-filtro temporal (R@10 80%→37.5% en reasoning). Diseñar ranking que no traiga contexto irrelevante al filtrar por `valid_at`. | Origen: Dim 5/ámbito 4 · Trackeado: MGR-11 (P49) / WIRE-08 (P56) | — |
| `N-09` | — | **Fórmula de decaimiento de confianza** | — | 🟡 1-2d | 🟡 Media | 🆕 Pendiente | Ninguna fórmula canónica validada para memoria semántica; definir decay + revalidación (entra con MGR-12). | Origen: Dim 6 · Trackeado: MGR-11/12 (P49) / SCH-04 (P53); sin canónico — sigue abierto | — |
| `N-10` | — | **Benchmark entity-memory longitudinal** | — | 🟠 2-3d | 🟡 Media | 🆕 Pendiente | Probar identidad a través de sesiones/cambios de nombre/contradicciones con fusiones reversibles puntuadas (no existe estándar; LongMemEval/LoCoMo como base). | Origen: Dim 7 · Trackeado: base en VER-08 ✅ (2026-09-30); sigue sin estándar externo | — |

> Nota: eval de ingesta (LoCoMo) = EXE-02 y test cascada multiagente = EXE-07, ya en `docs/dev/Backlog.md` — no se duplican aquí.

## Benchmarks y credibilidad (post-investigación 2026-09-24)

> Lo que NINGÚN benchmark público mide hoy (mem0.ai/blog/ai-memory-benchmarks-in-2026) y VantaDB va a medir y publicar (VER-08/VER-09 ✅ — los números ya existen; lo pendiente es publicarlos en la página `Benchmarks`). Cada fila = contenido para la página `Benchmarks` de Notion.

> Sin filas pendientes — `N-12..N-16` cerradas por absorción en `FIND-688` (Ola 3, 2026-11-23; registro en `docs/dev/avance/activo/operaciones.md`). `N-28` (consolidación + chart view) sigue activa en «Organización del hub + artefactos nuevos».

## Re-validación continua

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `N-11` | — | **Re-validar "Análisis de cobertura" de las dims** | — | — | 🟡 Media | 🆕 Pendiente (recurrente) | Re-validar la fecha 2026-09-14 tras cada slice IMPL-MGR que cambie un estado REAL/PARCIAL/PROPUESTA. | Origen: post-investigación 2026-09-24 · Se dispara con cada IMPL-MGR | — |

## Contenido stale — sync repo→Notion (auditoría 2026-09-30)

> Fuente: auditoría completa de VantaDB Docs (hub + 30 páginas + subtree Propuesta; 4 research + scan de placeholders). Regla: append-only, marcas [REAL]/[PROPUESTA], verificación contra código (Regla 11).

> Sin filas pendientes — `N-18..N-24` cerradas por absorción en `FIND-688` (Ola 3, 2026-11-23; registro en `docs/dev/avance/activo/operaciones.md`).

## Organización del hub + artefactos nuevos (auditoría 2026-09-30)

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `N-25` | — | **Reorg hub v2** | — | 🟠 2-3d | 🟡 Media | 🆕 Pendiente (2026-09-30) | (1) crear sección real "Capacidades verificadas" (API&SDKs / Memoria / Confiabilidad / Release) y mover las 8 páginas de "Sin sección"; (2) eliminar duplicación child-blocks + mention-list; (3) subir `Definición oficial` junto al Resumen; (4) separar "Taller del owner" (workbook + Plantilla) de las docs de producto; (5) TOC + descripción del hub + niveles de heading consistentes; (6) triage de `Preguntas sin sección`. Verificación: navegación 2 niveles; 0 páginas en "Sin sección"; 0 duplicados. | Origen: auditoría VantaDB Docs 2026-09-30 | — |
| `N-26` | — | **ADRs → database** | — | 🟠 1-2d | 🟡 Media | 🆕 Pendiente (2026-09-30) | Crear database ADRs (Status Proposed/Accepted/Superseded · Fecha · Área · Owner) + migrar ADRs del repo (ADR-0000..0053 — 54 + `DECISIONS-NOT-TAKEN`) como páginas + views table/board/timeline + botón "Nueva ADR" (template) + lock de estructura. Verificación: cada ADR del repo presente; views funcionan. | Origen: auditoría VantaDB Docs 2026-09-30 | — |
| `N-27` | — | **Changelog → database + release notes** | — | 🟡 1d | 🟡 Media | 🆕 Pendiente (2026-09-30) | Database de versiones (0.5.0→0.8.0) con view feed/list desde `docs/CHANGELOG.md`; release notes por versión; retirar el bloque monolítico duplicado del roadmap. Verificación: 1 entrada por release real. | Origen: auditoría VantaDB Docs 2026-09-30 | — |
| `N-28` | — | **Benchmarks: consolidación + VER-09 + chart view** | — | 🟡 1d | 🟡 Media | 🆕 Pendiente (2026-09-30) | Complementa lo absorbido de `N-12..N-16` (cerradas en `FIND-688`): consolidar la página Benchmarks (protocolo + resultados VER-08), añadir head-to-head VER-09 (mem0/Zep/Letta), chart view para métricas y link canónico a `BENCHMARKS.md` §19. Verificación: reproducible (Regla 11); 1 fuente por número. | Origen: auditoría VantaDB Docs 2026-09-30 · Ver: `FIND-688` (absorbe `N-12..N-16`) | — |
| `N-29` | — | **Páginas nuevas: FAQ + Migration + Glossary + Capacidades** | — | 🟠 1-2d | 🟡 Media | 🆕 Pendiente (2026-09-30) | (1) FAQ user-facing; (2) Migration guide 0.7→0.8 (desde `UPGRADE.md` §0.8.0); (3) Glossary (desde `docs/user/glosario/`); (4) índice "Capacidades" (capacidad→estado→evidencia→página). Verificación: páginas llenas + links cruzados. | Origen: auditoría VantaDB Docs 2026-09-30 | — |

## Configuración del workspace (skill `notion-mcp-master` + web 2026-09-30)

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `N-30` | — | **Wiki mode + verificación con expiración** | — | 🟠 2-3d | 🟢 Nice-to-have | 🆕 Pendiente (2026-09-30) | Convertir el hub en wiki; verificación (con expiry, ej. 90d) en páginas core; owner por página; locks en páginas canónicas + lock database en ADRs. Verificación: verificaciones activas; expiración probada. | Origen: auditoría VantaDB Docs 2026-09-30 · Ver: N-26 (locks ADR) | — |
| `N-31` | — | **Adopción de formatos** | — | 🟡 1d | 🟢 Nice-to-have | 🆕 Pendiente (2026-09-30) | Callouts semánticos (⚠️/🚧/ℹ️/✅), toggle headings (FAQ/changelogs), `<table_of_contents/>`, code blocks con lenguaje+wrap, tablas simples vs databases, synced blocks solo para snippets estables; fase 2: dashboard de "docs por verificar" + forms (requiere plan Business — verificar) + skill "cómo escribimos docs". Verificación: formato aplicado en páginas top; checklist del skill cumplido. | Origen: auditoría VantaDB Docs 2026-09-30 | — |

## Workbook del owner (por diseño — no IA)

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `N-32` | — | **Llenar workbook A1-E2 + Síntesis** | — | 🔴 1-2sem | 🟠 Media | 🆕 Pendiente (owner, 2026-09-30) | (1) 17 páginas de ejercicios (Alcance, Límites, Usuario, Casos, CDA, Core V1, Criterios, Cuándo no, Contrato Operativo, Módulos, Modelo de Datos, Arq. Memoria, Arq. DX, Durabilidad, ADRs, Posicionamiento, Propuesta de Valor) — escritura del owner ("sin ayuda de IA" por diseño); (2) `Plantilla`: corregir partes A-E ausentes + estado de 8 CRITs + confirmar destino `[ARCHIVED]`; (3) Síntesis de Fundamentos (hub) → semilla `VISION.md`. Verificación: páginas sin "Tu respuesta:" pendientes; síntesis escrita. | Origen: auditoría VantaDB Docs 2026-09-30 · Dueño: owner | — |
