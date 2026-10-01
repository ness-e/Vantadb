---
title: Backlog Notion — tareas sobre páginas VantaDB Docs
kind: research
---

# Backlog Notion — tareas sobre páginas VantaDB Docs

> **Propósito:** registrar todo el trabajo pendiente sobre las páginas Notion (edición, mejora, investigación y sincronización con código). Fuentes: análisis integral 2026-09-18 + **auditoría completa VantaDB Docs 2026-09-30** (hub + 30 hijas + subtree Propuesta; 4 research + scan de placeholders; skill `notion-mcp-master` + 27 fuentes web) → filas N-18..N-32.
> **Regla:** las páginas de lanzamiento (posts, changelogs, FAQ pública) son messaging crítico — cada edición se verifica contra código antes de darse por hecha. Los posts de anuncio siguen pausados hasta el gate Fase A (decisión 2026-09-08, vigente).
> **Convención:** `N-xx` = tarea Notion. `⬜` pendiente · `⏳` en curso · `✅` hecha (se elimina la fila; el registro queda en `docs/dev/avance/`).

## Actualización de ejemplos (API vieja → `Client`/`search`)

| ID | Página(s) | Alcance | Verificación | Estado |
|----|-----------|---------|--------------|--------|
| `N-03` | Plan 00/03 (posts HN/Reddit/Twitter/video/guion) | Reescribir claims contra API actual cuando se vayan a usar. Regla vigente: solo features de la release anunciada. Siguen pausados hasta gate Fase A | Cero funciones no existentes por post | ⬜ Pendiente (bloqueado por gate Fase A) |

## Roadmap y Propuesta (sincronización con código)

> Sin filas pendientes — `N-17` cerrada 2026-09-30 (sync ejecutado + verificado + Sync #2 validación externa; registro en `docs/dev/avance/activo/operaciones.md`).
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

## Contenido stale — sync repo→Notion (auditoría 2026-09-30)

> Fuente: auditoría completa de VantaDB Docs (hub + 30 páginas + subtree Propuesta; 4 research + scan de placeholders). Regla: append-only, marcas [REAL]/[PROPUESTA], verificación contra código (Regla 11).

| ID | Página | Alcance | Verificación | Estado |
|----|--------|---------|--------------|--------|
| `N-18` | **SDKs y Quickstarts** | Curación completa: (1) escribir Tutoriales 2 ("RAG para Documentos") y 3 ("Grafos de Conocimiento") — hoy solo títulos; (2) eliminar/anotar `search_memory` residual (2 sitios) con ANTES/AHORA; (3) corregir FAQ (dice "v1.0.0 production-ready" y "entity resolution: Sí" — falso, son [PROPUESTA]); (4) añadir quickstarts Node/WASM/MCP/CLI + troubleshooting; (5) header v0.6.1→0.7.0 + marcar tabla `@user1` como ejemplo + separar plan interno (DMs/tareas) de la doc user-facing | Cero funciones inexistentes; claims vs código; versiones al día | ⬜ Pendiente (2026-09-30) |
| `N-19` | **Referencia API verificada** | (1) añadir campos v2 (bitemporal+confianza+cuarentena) + params (`AS OF`/`valid_window`/`min_confidence`/`include_quarantined`) + `query_sparse` (paridad SCH-07); (2) completar `db.memory.*`, `get/delete/list`, `capabilities`, errores/tipos; (3) superficies Node/WASM/REST/MCP/CLI explícitas; (4) deduplicar las 2 api-references; (5) subir a 0.7.0; (6) mover bloque [PROPUESTA] (~40%) a subpágina "No existe aún" | Tabla Z.1 vs código; 0 autolinks rotos | ⬜ Pendiente (2026-09-30) |
| `N-20` | **Gobernanza del ciclo de vida** | (1) tabla de estado [REAL]/[PROPUESTA] por spec con evidencia; (2) incorporar tracks ya en HEAD omitidos: SCH-01/03/04/06/07/09, VER-01/02/03/05/06; (3) corregir numeración v0.7.0→**0.8.0**; (4) decidir dónde viven los 6 docs propuestos (governance.md/entity-resolution.md/etc.); (5) fix link `db.auto` + reflejar `mark_duplicate` parcial | Estado por spec con evidencia file:line | ⬜ Pendiente (2026-09-30) |
| `N-21` | **Observabilidad, benchmarks y métricas** | (1) llenar §1.6 (12 placeholders "XX" + `[tu CPU]/[tu RAM]`) o redirigir al harness real; (2) publicar VER-08 completo (recall_all@5 0.7617 · recall_any@5 0.9213 · write-quality 1.0 · isolation 0/2500 · abstención · ECE 0.0787→0.0003); (3) añadir VER-09 head-to-head; (4) reparar tabla HTML corrupta; (5) dueño/estado del guardrail ≤7d | Números vs BENCHMARKS §19 | ⬜ Pendiente (2026-09-30) |
| `N-22` | **Roadmap, changelog y criterios** | (1) changelog 0.7.0 publicado + 0.8.0 preparada; (2) criterios de release v0.8.0; (3) definir gates EXE-01/VER-09 (hoy referencias colgantes); (4) resolver numeración governance 0.7 vs 0.8; (5) registro de ejecución Fase A (o estado explícito); (6) llenar/marcar placeholders `[Feature 1]`/`@user1`; (7) cerrar PENDIENTE DE DECISIÓN influencers (desde 09-08) + refresh snapshot Z.4 al 09-30; (8) deduplicar ~60% plantilla histórica + links autolink rotos | Entradas por versión reales; 0 pendientes colgantes | ⬜ Pendiente (2026-09-30) |
| `N-23` | **Panorama del Mercado Actual** | (1) sección de competidores de memoria (mem0/Zep/Letta/Supermemory — hoy 0 menciones; viven en research); (2) re-verificar claims de riesgo ($3B Windsurf/OpenAI, SWE-bench, Devin); (3) refresh tendencias H2-2026; (4) puente con VISION.md; (5) subir nota de custodia (estudio general, no producto) | Claims con fuente fechada | ⬜ Pendiente (2026-09-30) |
| `N-24` | **Higiene transversal** | (1) versiones `0.6.1`→`0.7.0` en superficies stale (SDKs, RefAPI, roadmap interno); (2) autolinks rotos (`[db.search](http://db.search)`, `db.auto`) → texto plano; (3) fences ```javascript con Python → lenguaje correcto; (4) `Seguridad de la memoria`: sumar MEMG-17 (firmas/erasure) cuando se ejecute | 0 versiones stale / 0 autolinks rotos | ⬜ Pendiente (2026-09-30) |

## Organización del hub + artefactos nuevos (auditoría 2026-09-30)

| ID | Tema | Alcance | Verificación | Estado |
|----|------|---------|--------------|--------|
| `N-25` | **Reorg hub v2** | (1) crear sección real "Capacidades verificadas" (API&SDKs / Memoria / Confiabilidad / Release) y mover las 8 páginas de "Sin sección"; (2) eliminar duplicación child-blocks + mention-list; (3) subir `Definición oficial` junto al Resumen; (4) separar "Taller del owner" (workbook + Plantilla) de las docs de producto; (5) TOC + descripción del hub + niveles de heading consistentes; (6) triage de `Preguntas sin sección` | Navegación 2 niveles; 0 páginas en "Sin sección"; 0 duplicados | ⬜ Pendiente (2026-09-30) |
| `N-26` | **ADRs → database** | Crear database ADRs (Status Proposed/Accepted/Superseded · Fecha · Área · Owner) + migrar ADRs del repo (ADR-0001..0047) como páginas + views table/board/timeline + botón "Nueva ADR" (template) + lock de estructura | Cada ADR del repo presente; views funcionan | ⬜ Pendiente (2026-09-30) |
| `N-27` | **Changelog → database + release notes** | Database de versiones (0.5.0→0.8.0) con view feed/list desde `docs/CHANGELOG.md`; release notes por versión; retirar el bloque monolítico duplicado del roadmap | 1 entrada por release real | ⬜ Pendiente (2026-09-30) |
| `N-28` | **Benchmarks: consolidación + VER-09 + chart view** | Complementa N-12..N-16: consolidar la página Benchmarks (protocolo + resultados VER-08), añadir head-to-head VER-09 (mem0/Zep/Letta), chart view para métricas y link canónico a `BENCHMARKS.md` §19 | Reproducible (Regla 11); 1 fuente por número | ⬜ Pendiente (2026-09-30) |
| `N-29` | **Páginas nuevas: FAQ + Migration + Glossary + Capacidades** | (1) FAQ user-facing; (2) Migration guide 0.7→0.8 (desde `UPGRADE.md` §0.8.0); (3) Glossary (desde `docs/user/glosario/`); (4) índice "Capacidades" (capacidad→estado→evidencia→página) | Páginas llenas + links cruzados | ⬜ Pendiente (2026-09-30) |

## Configuración del workspace (skill `notion-mcp-master` + web 2026-09-30)

| ID | Tema | Alcance | Verificación | Estado |
|----|------|---------|--------------|--------|
| `N-30` | **Wiki mode + verificación con expiración** | Convertir el hub en wiki; verificación (con expiry, ej. 90d) en páginas core; owner por página; locks en páginas canónicas + lock database en ADRs | Verificaciones activas; expiración probada | ⬜ Pendiente (2026-09-30) |
| `N-31` | **Adopción de formatos** | Callouts semánticos (⚠️/🚧/ℹ️/✅), toggle headings (FAQ/changelogs), `<table_of_contents/>`, code blocks con lenguaje+wrap, tablas simples vs databases, synced blocks solo para snippets estables; fase 2: dashboard de "docs por verificar" + forms (requiere plan Business — verificar) + skill "cómo escribimos docs" | Formato aplicado en páginas top; checklist del skill cumplido | ⬜ Pendiente (2026-09-30) |

## Workbook del owner (por diseño — no IA)

| ID | Tema | Alcance | Verificación | Estado |
|----|------|---------|--------------|--------|
| `N-32` | **Llenar workbook A1-E2 + Síntesis** | (1) 17 páginas de ejercicios (Alcance, Límites, Usuario, Casos, CDA, Core V1, Criterios, Cuándo no, Contrato Operativo, Módulos, Modelo de Datos, Arq. Memoria, Arq. DX, Durabilidad, ADRs, Posicionamiento, Propuesta de Valor) — escritura del owner ("sin ayuda de IA" por diseño); (2) `Plantilla`: corregir partes A-E ausentes + estado de 8 CRITs + confirmar destino `[ARCHIVED]`; (3) Síntesis de Fundamentos (hub) → semilla `VISION.md` | Páginas sin "Tu respuesta:" pendientes; síntesis escrita | ⬜ Pendiente (owner, 2026-09-30) |
