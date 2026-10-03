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

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `N-12` | — | **Benchmarks propios publicados** | — | 🟡 1d | 🟡 Media | 🆕 Pendiente | Protocolo VantaDB (dataset commiteado, comandos de reproducción) + resultados canonical_p99/LoCoMo/LongMemEval-S/BEAM-subset en la página Benchmarks. Verificación: reproducible desde repo (Regla 11). | Origen: post-investigación 2026-09-24 · Datos: VER-08 ✅ (2026-09-30) · Ver: N-28 (consolidación + chart) | — |
| `N-13` | — | **Write-quality (calidad de escritura)** | — | 🟢 2-3h | 🟡 Media | 🆕 Pendiente | Medir qué se guarda vs qué se descarta (extracción selectiva) — nadie lo mide. Reporte con métrica definida. Verificación: subscore publicado. | Datos: VER-08 ✅ (write-quality 1.0) | — |
| `N-14` | — | **Abstención** | — | 🟢 2-3h | 🟡 Media | 🆕 Pendiente | LongMemEval `_abs`: declinar correctamente ante eventos inexistentes; liga con SCH-05. Verificación: subscore publicado. | Datos: VER-08 ✅ (abstención) · Ver: SCH-05 (P53) | — |
| `N-15` | — | **Aislamiento per-user** | — | 🟢 2-3h | 🟡 Media | 🆕 Pendiente | Multi-usuario concurrente sin filtraciones entre tenants — ningún benchmark público lo cubre. Test de aislamiento documentado. | Datos: VER-08 ✅ (isolation 0/2500) + tests RBAC existentes | — |
| `N-16` | — | **Token-economy** | — | 🟢 2-3h | 🟡 Media | 🆕 Pendiente | Pares accuracy+tokens por retrieval (no accuracy sola); comparar contra full-context (~25K tokens). Tabla por sistema con tokens mean. | Datos: VER-08/VER-09 ✅ (2026-09-30) | — |

## Re-validación continua

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `N-11` | — | **Re-validar "Análisis de cobertura" de las dims** | — | — | 🟡 Media | 🆕 Pendiente (recurrente) | Re-validar la fecha 2026-09-14 tras cada slice IMPL-MGR que cambie un estado REAL/PARCIAL/PROPUESTA. | Origen: post-investigación 2026-09-24 · Se dispara con cada IMPL-MGR | — |

## Contenido stale — sync repo→Notion (auditoría 2026-09-30)

> Fuente: auditoría completa de VantaDB Docs (hub + 30 páginas + subtree Propuesta; 4 research + scan de placeholders). Regla: append-only, marcas [REAL]/[PROPUESTA], verificación contra código (Regla 11).

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `N-18` | 🟡 Media | **SDKs y Quickstarts — curación completa** | — | 🟠 1-2d | 🟠 Media | 🆕 Pendiente (2026-09-30) | (1) escribir Tutoriales 2 ("RAG para Documentos") y 3 ("Grafos de Conocimiento") — hoy solo títulos; (2) eliminar/anotar `search_memory` residual (2 sitios) con ANTES/AHORA; (3) corregir FAQ ("v1.0.0 production-ready" y "entity resolution: Sí" — falso, son [PROPUESTA]); (4) añadir quickstarts Node/WASM/MCP/CLI + troubleshooting; (5) header v0.6.1→0.7.0 + marcar tabla `@user1` como ejemplo + separar plan interno (DMs/tareas) de la doc user-facing. Verificación: cero funciones inexistentes; claims vs código; versiones al día. | Origen: auditoría VantaDB Docs 2026-09-30 | — |
| `N-19` | 🟡 Media | **Referencia API verificada — actualización completa** | — | 🟠 1-2d | 🟠 Media | 🆕 Pendiente (2026-09-30) | (1) añadir campos v2 (bitemporal+confianza+cuarentena) + params (`AS OF`/`valid_window`/`min_confidence`/`include_quarantined`) + `query_sparse` (paridad SCH-07); (2) completar `db.memory.*`, `get/delete/list`, `capabilities`, errores/tipos; (3) superficies Node/WASM/REST/MCP/CLI explícitas; (4) deduplicar las 2 api-references; (5) subir a 0.7.0; (6) mover bloque [PROPUESTA] (~40%) a subpágina "No existe aún". Verificación: tabla Z.1 vs código; 0 autolinks rotos. | Origen: auditoría VantaDB Docs 2026-09-30 | — |
| `N-20` | 🟡 Media | **Gobernanza del ciclo de vida — estados por spec** | — | 🟡 1d | 🟡 Media | 🆕 Pendiente (2026-09-30) | (1) tabla de estado [REAL]/[PROPUESTA] por spec con evidencia; (2) incorporar tracks ya en HEAD omitidos: SCH-01/03/04/06/07/09, VER-01/02/03/05/06; (3) corregir numeración v0.7.0→**0.8.0**; (4) decidir dónde viven los 6 docs propuestos (governance.md/entity-resolution.md/etc.); (5) fix link `db.auto` + reflejar `mark_duplicate` parcial. Verificación: estado por spec con evidencia file:line. | Origen: auditoría VantaDB Docs 2026-09-30 | — |
| `N-21` | 🟡 Media | **Observabilidad, benchmarks y métricas — placeholders y números** | — | 🟡 1d | 🟡 Media | 🆕 Pendiente (2026-09-30) | (1) llenar §1.6 (12 placeholders "XX" + `[tu CPU]/[tu RAM]`) o redirigir al harness real; (2) publicar VER-08 completo (recall_all@5 0.7617 · recall_any@5 0.9213 · write-quality 1.0 · isolation 0/2500 · abstención · ECE 0.0787→0.0003); (3) añadir VER-09 head-to-head; (4) reparar tabla HTML corrupta; (5) dueño/estado del guardrail ≤7d. Verificación: números vs BENCHMARKS §19. | Origen: auditoría VantaDB Docs 2026-09-30 | — |
| `N-22` | 🟡 Media | **Roadmap, changelog y criterios — pendientes colgantes** | — | 🟡 1d | 🟡 Media | 🆕 Pendiente (2026-09-30) | (1) changelog 0.7.0 publicado + 0.8.0 preparada; (2) criterios de release v0.8.0; (3) registrar gates EXE-01/VER-09 (ambos ✅ 2026-09-30 — actualizar referencia); (4) resolver numeración governance 0.7 vs 0.8; (5) registro de ejecución Fase A (o estado explícito); (6) llenar/marcar placeholders `[Feature 1]`/`@user1`; (7) cerrar PENDIENTE DE DECISIÓN influencers (desde 09-08) + refresh snapshot Z.4 al 09-30; (8) deduplicar ~60% plantilla histórica + links autolink rotos. Verificación: entradas por versión reales; 0 pendientes colgantes. | Origen: auditoría VantaDB Docs 2026-09-30 | — |
| `N-23` | 🟡 Media | **Panorama del Mercado Actual — competidores y claims** | — | 🟡 1d | 🟡 Media | 🆕 Pendiente (2026-09-30) | (1) sección de competidores de memoria (mem0/Zep/Letta/Supermemory — hoy 0 menciones; viven en research); (2) re-verificar claims de riesgo ($3B Windsurf/OpenAI, SWE-bench, Devin); (3) refresh tendencias H2-2026; (4) puente con VISION.md; (5) subir nota de custodia (estudio general, no producto). Verificación: claims con fuente fechada. | Origen: auditoría VantaDB Docs 2026-09-30 | — |
| `N-24` | 🟢 Baja | **Higiene transversal — versiones, autolinks, fences** | — | 🟢 2-3h | 🟡 Media | 🆕 Pendiente (2026-09-30) | (1) versiones `0.6.1`→`0.7.0` en superficies stale (SDKs, RefAPI, roadmap interno); (2) autolinks rotos (`[db.search](http://db.search)`, `db.auto`) → texto plano; (3) fences ```javascript con Python → lenguaje correcto; (4) `Seguridad de la memoria`: sumar MEMG-17 (firmas/erasure) cuando se ejecute. Verificación: 0 versiones stale / 0 autolinks rotos. | Origen: auditoría VantaDB Docs 2026-09-30 · Ver: MEMG-17 | — |

## Organización del hub + artefactos nuevos (auditoría 2026-09-30)

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `N-25` | — | **Reorg hub v2** | — | 🟠 2-3d | 🟡 Media | 🆕 Pendiente (2026-09-30) | (1) crear sección real "Capacidades verificadas" (API&SDKs / Memoria / Confiabilidad / Release) y mover las 8 páginas de "Sin sección"; (2) eliminar duplicación child-blocks + mention-list; (3) subir `Definición oficial` junto al Resumen; (4) separar "Taller del owner" (workbook + Plantilla) de las docs de producto; (5) TOC + descripción del hub + niveles de heading consistentes; (6) triage de `Preguntas sin sección`. Verificación: navegación 2 niveles; 0 páginas en "Sin sección"; 0 duplicados. | Origen: auditoría VantaDB Docs 2026-09-30 | — |
| `N-26` | — | **ADRs → database** | — | 🟠 1-2d | 🟡 Media | 🆕 Pendiente (2026-09-30) | Crear database ADRs (Status Proposed/Accepted/Superseded · Fecha · Área · Owner) + migrar ADRs del repo (ADR-0000..0053 — 54 + `DECISIONS-NOT-TAKEN`) como páginas + views table/board/timeline + botón "Nueva ADR" (template) + lock de estructura. Verificación: cada ADR del repo presente; views funcionan. | Origen: auditoría VantaDB Docs 2026-09-30 | — |
| `N-27` | — | **Changelog → database + release notes** | — | 🟡 1d | 🟡 Media | 🆕 Pendiente (2026-09-30) | Database de versiones (0.5.0→0.8.0) con view feed/list desde `docs/CHANGELOG.md`; release notes por versión; retirar el bloque monolítico duplicado del roadmap. Verificación: 1 entrada por release real. | Origen: auditoría VantaDB Docs 2026-09-30 | — |
| `N-28` | — | **Benchmarks: consolidación + VER-09 + chart view** | — | 🟡 1d | 🟡 Media | 🆕 Pendiente (2026-09-30) | Complementa N-12..N-16: consolidar la página Benchmarks (protocolo + resultados VER-08), añadir head-to-head VER-09 (mem0/Zep/Letta), chart view para métricas y link canónico a `BENCHMARKS.md` §19. Verificación: reproducible (Regla 11); 1 fuente por número. | Origen: auditoría VantaDB Docs 2026-09-30 · Ver: N-12..N-16 | — |
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
