# Auditoría de Documentación — `docs/progreso/README.md` (líneas 1101–2200)

- **Archivo:** `docs/progreso/README.md` (3320 líneas totales)
- **Rango auditado:** 1101–2200 (solo lectura, sin edición)
- **Fecha de auditoría:** 2026-08-03
- **Método:** Lectura en bloques (1101–1400, 1401–1700, 1701–2000, 2001–2200) vía Read con offset/limit.

---

## 1. Estructura

| Línea | Elemento | Tipo | Nota |
|-------|----------|------|------|
| 1101–1340 | Continuación de bloque anterior al rango (checklists CI + entradas detalle DISC-02/03, TSK-47/49, ROAD-06, TSK-45, TSK-106b, TSK-71, Fix WASM, TSK-112, TSK-118, CLI-EPIC, TSK-111/119/86/87/88) | H3 sueltos | Sin H2 contenedor visible en el rango; son entradas de detalle sin cabecera de sección propia |
| 1341 | `--` | Separador huérfano | Sin contexto, flota entre bloques |
| 1343 | `## Tasks Completed (Migrated from Backlog)` | **H2 (EN)** | Sección de tareas completadas migradas del backlog |
| 1435 | `### July 2026 — Code Audit (2nd pass)` | H3 | Tabla bajo el H2 de 1343 |
| 1489 | `### DISC Discoveries Completed` | H3 | Tabla bajo el H2 de 1343 |
| 1503 | `## Completed Task History` | **H2 (EN)** | Historial con entradas H3 fechadas |
| 1505 | `### [2026-06-22] Fix Heavy Certification Workflow Failures` | H3 | |
| 1522 | `### [2026-06-22] Batch CI/CD Fixes + StorageEngine Locking (TSK-134/135/138/140/126/128/129)` | H3 | |
| 1544 | `### [2026-06-22] jemalloc Instrumentation + CI/CD Swap (TSK-130/137)` | H3 | |
| 1569 | `## Tareas Completadas (Migradas desde Backlog)` | **H2 (ES)** | **Sección equivalente al H2 de 1343** — duplicación estructural |
| 1571+ | `### INV-007 … CODE-027` | H3 | Entradas del bloque "Tareas Completadas" (ES) |
| 2079–2084 | Bloque MEM-01 | H3 huérfano | **Sin heading `###` propio** — cuelga del bloque MCP-IDE (sus campos Fecha/Objetivo/Checklist se fusionan en el bloque anterior) |

### Hallazgos de estructura

| Línea | Problema | Severidad |
|-------|----------|-----------|
| 1343 vs 1569 | Dos H2 con el mismo propósito: `Tasks Completed (Migrated from Backlog)` (EN) y `Tareas Completadas (Migradas desde Backlog)` (ES) — contenido solapado/duplicado a nivel de sección | **Alta** |
| 2079–2084 | MEM-01 sin encabezado `###`; sus `- **Fecha:**`/`- **Objetivo:**` aparecen pegados al bloque MCP-IDE (línea 2079 sigue a `- **Ids:** MCP-IDE` de 2078) | **Alta** |
| 1101–1340 | Entradas de detalle sin H2 contenedor en el rango (dependen de cabeceras fuera del rango) — navegación por TOC rota | Media |
| 1341 | Separador `--` huérfano | Baja |

---

## 2. Formato

| Línea | Problema | Severidad |
|-------|----------|-----------|
| 1347–1419 | Tabla "Tasks Completed" **sin fila de cabecera** (`| ID | Tarea | Prioridad | Estado |`) — arranca directo con filas de datos; la tabla hermana (1437) sí tiene cabecera | **Alta** |
| 1378–1396 | Filas AUD-05…AUD-21 con la resolución incrustada en la columna Tarea (`| AUD-05 | Reparar broken links… | → ✅ 18 links… | 🔴 | ✅ |`) → 5–6 celdas vs 4 del resto de la tabla; esquema roto | **Alta** |
| 1437 vs 1347 | Dos tablas del mismo concepto con esquemas distintos (una con cabecera, otra sin) | Media |
| 1369–1370 | Fecha pegada al pipe: `…2026-06-20|` sin espacio antes de `\|` (DISC-02, DISC-03) | Baja |
| 1476 / 1501 | Fila AUD-WORK con texto largo (resumen completo) dentro de una celda — mismo problema de celda desbordada | Media |
| 1135, 1189, 1255, 1261 | Mezcla ES/EN dentro de entradas: `sq8_quantize()` y `sq8_similarity()` (1135), `✅ sin errores` (1189), `Checklist completado:` (1255), `conteo de registros` (1261) | Media |
| 1101–1340, 1350 | Secciones EN con frases ES ("pendiente reverificación formal", "Checklist Real", "Causa raíz") — idioma inconsistente entre y dentro de bloques | Media |
| 1146–1252 vs 1255+ | Formato de fecha en 3 variantes: en el título `(2026-06-21)` (TSK-45, TSK-106b…), campo `**Fecha:** 2026-07-14` (DRV-*), e inline `✅ Done 2026-06-18` (tabla 1353) | Media |
| 2079–2084 | MEM-01: bloque sin heading + `Fecha:` duplicada/desplazada dentro del bloque MCP-IDE | Alta |
| 1571+ | Entradas "Tareas Completadas" sin campo Prioridad y con commit opcional (presente en COMP-028/OLD-21/TSK-107b, ausente en INV-*/REC-*/WEB-*/MCP-*) | Media |

---

## 3. Duplicados (IDs repetidos en el rango)

### IDs con dos entradas H3/tabla de contenido distinto (mismo ID, dos registros)

| ID | Líneas | Nota |
|----|--------|------|
| **DRV-109** | **1809 y 1886** | Mismo ID, dos entradas H3 completas (ver §4 — contradicción) |
| AUD-05 | 1378 y 1443 | Mismo ID, tareas distintas |
| AUD-06 | 1379 y 1444 | Ídem |
| AUD-07 | 1380 y 1445 | Ídem |
| AUD-08 | 1383 y 1446 | Ídem |
| AUD-09 | 1384 y 1447 | Ídem |
| AUD-10 | 1385 y 1448 | Ídem |
| AUD-11 | 1386 y 1449 | Ídem |
| AUD-12 | 1387 y 1450 | Ídem |
| AUD-13 | 1388 y 1451 | Ídem |
| AUD-14 | 1389 y 1452 | Ídem |
| AUD-15 | 1390 y 1453 | Ídem |
| AUD-16 | 1391 y 1454 | Ídem |
| AUD-17 | 1392 y 1455 | Ídem |
| AUD-18 | 1393 y 1456 | Ídem |
| TSK-111 | 1268 y 1426 | H3 "NUNCA IMPLEMENTADO" + fila tabla ❌ |
| TSK-119 | 1281 y 1457 | H3 "NUNCA FUE SDK, ELIMINADO" + fila tabla ❌ |
| TSK-86 | 1293 y 1458 | H3 "NUNCA IMPLEMENTADO" + fila tabla ❌ |
| TSK-87 | 1305 y 1459 | H3 "NUNCA FUE SDK, ELIMINADO" + fila tabla ❌ |
| TSK-88 | 1317 y 1460 | H3 "NUNCA IMPLEMENTADO" + fila tabla ❌ |
| TSK-45 | 1146 y 1420 | H3 detalle + fila tabla |
| TSK-106b | 1164 y 1421 | H3 detalle + fila tabla |
| TSK-71 | 1177 y 1422 | H3 detalle + fila tabla |
| TSK-112 | 1214 y 1423 | H3 detalle + fila tabla |
| TSK-118 | 1243 y 1425 | H3 detalle + fila tabla |
| CLI-EPIC | 1252 y 1463 | H3 detalle + fila tabla |
| TSK-47 | 1133 (detalle) y 1357 (fila) | Representación duplicada del mismo trabajo ✅ |
| TSK-49 | 1139 (detalle) y 1358 (fila) | Ídem |
| DISC-02 | 1132 (detalle) y 1369 (fila) | Ídem |
| DISC-03 | 1131 (detalle) y 1370 (fila) | Ídem |
| ROAD-06 | 1144 (detalle) y 1373 (fila) | Ídem |
| DISC-01 | 1469 y 1493 | Tabla "July 2026" + tabla "DISC Discoveries" |
| DISC-04 | 1470 y 1494 | Ídem |
| DISC-06 | 1471 y 1495 | Ídem |
| DISC-07 | 1472 y 1496 | Ídem |
| DISC-08 | 1473 y 1497 | Ídem |
| DISC-09 | 1474 y 1498 | Ídem |
| DISC-10 | 1475 y 1499 | Ídem |
| **AUD-WORK** | **1381, 1476 y 1501** | **3 ocurrencias** (tabla migrada + tabla julio + tabla DISC) |

### IDs buscados por la auditoría

| ID | Resultado |
|----|-----------|
| DRV-109 | **Sí duplicado** — 1809 y 1886 |
| AUD-01…AUD-04 | Una sola ocurrencia (tabla julio, 1439–1442) — no duplicados |
| AUD-05…AUD-18 | **14 IDs duplicados** (1378–1396 vs 1439–1456) |
| AUD-19…AUD-44 | Una sola ocurrencia (tabla migrada, 1394–1419) — no duplicados |
| DOC-19, DOC-20 | **No encontrados en el rango** (0 ocurrencias) |
| WEB-03 | Una sola ocurrencia en 1349 — no duplicado en el rango |

---

## 4. Contradicciones (mismo ID, contenido/estado distinto)

| ID | Líneas | Problema | Severidad |
|----|--------|----------|-----------|
| AUD-05…AUD-18 | 1378–1396 vs 1439–1456 | **Colisión de IDs entre dos auditorías distintas.** La tabla "July 2026 — Code Audit (2nd pass)" reutiliza AUD-01…AUD-18 para hallazgos de la 2ª pasada (ej. AUD-05 = `.ok()` UTF-8, AUD-06 = N+1 scan_nodes), mientras la tabla migrada usa AUD-05…AUD-44 para la 1ª pasada (ej. AUD-05 = broken links, AUD-06 = DURABILITY_GUARANTEES). **14 IDs con descripciones completamente diferentes bajo el mismo identificador.** | **Alta** |
| DRV-109 | 1809 vs 1886 | Copia 1 (1809): "already correct from the start, no changes needed" — **no-op, sin commit**. Copia 2 (1886): "using same `py.detach()` pattern as DRV-102" — **sí requirió cambio**, commit `74fdc23`. Afirmaciones mutuamente excluyentes. | **Alta** |
| WEB-13 vs INV-013 | 1943–1950 vs 1613–1618 | WEB-13 (✅) afirma "JSON-LD structured data across all 25 route files" y checklist "JSON-LD structured data (WebSite, Organization schemas)"; INV-013 (✅) audita **"JSON-LD AUSENTE… cero `<script type="application/ld+json">`"** y concluye que Next.js 16 Metadata API no genera JSON-LD. Dos entradas completadas que se contradicen. | **Alta** |
| REV-001 vs NUEVO-05 | 1725–1730 vs 1960–1968 | REV-001 (07-14) "Remove `-Zsanitizer=thread` flag incompatible"; NUEVO-05 (07-10) "TSan job … with nightly + `-Z sanitizer=thread`". Uno añade y otro elimina la misma bandera, ambos listados ✅ sin nota de revert/relación. | Media |
| DEVOPS-13 | 1816–1821 | "No-op — no `.github/workflows/` files exist in this repository". **Afirmación falsa**: el resto del documento referencia decenas de workflows (`heavy_certification.yml`, `python_wheels.yml`, `release.yml`, `nightly_bench.yml`, `rust_ci.yml`). | **Alta** |
| TSK-111, TSK-119, TSK-86, TSK-87, TSK-88 | 1426, 1457–1460 | Filas con estado **❌** dentro de la sección 1343–1345 cuyo encabezado dice "These tasks reached **100% completion**" — el título de la sección contradice las filas que contiene. | Media |
| DISC-05 | 1350 | Estado `✅ (pendiente reverificación formal)` — marcada completada pero con verificación pendiente; estado ambiguo. | Media |
| SEC-01/SEC-02 | 1952–1958 | Afirman que bincode ya estaba migrado "via **AUD-03**", pero en la tabla de julio (1441) AUD-03 = `from_raw_parts` sin bounds check. Referencia a un ID colisionado, ambigua. | Baja |

---

## 5. Entradas no-tarea (autopsias, investigaciones sin código, no-ops, SKIPs)

| Línea | ID | Tipo de no-tarea | Severidad |
|-------|----|------------------|-----------|
| 1268 | TSK-111 | ❌ NUNCA IMPLEMENTADO (documentado, nunca codificado) | — |
| 1281 | TSK-119 | ❌ NUNCA FUE SDK, ELIMINADO (autopsia) | — |
| 1293 | TSK-86 | ❌ NUNCA IMPLEMENTADO (autopsia) | — |
| 1305 | TSK-87 | ❌ NUNCA FUE SDK, ELIMINADO (autopsia) | — |
| 1317 | TSK-88 | ❌ NUNCA IMPLEMENTADO (autopsia) | — |
| 1809 | DRV-109 (1ª copia) | No-op: "no changes needed" | — |
| 1816 | DEVOPS-13 | No-op falso: "no workflows files exist" | — |
| 1952 | SEC-01/SEC-02 | Verificación-only: "both already resolved" | — |
| 2005 | MKT-14 | SKIP por gate: "feature ya implementada… Sin code changes" | — |
| 2041 | INV-019 | ❌ SKIP: "ya implementada" | — |
| 2030 | INV-006 | Plan sin implementación (solo plan 205 líneas) | — |
| 1571 | INV-007 | Investigación + diseño, 0 cambios de código | — |
| 1578 | INV-008 | Diseño (gate: parcialmente implementado), 0 cambios | — |
| 1585 | INV-009 | Diseño, 0 cambios | — |
| 1592 | INV-010 | Diseño ACID rollback, 0 cambios | — |
| 1599 | INV-011 | Auditoría: "Separación YA limpia — sin cambios requeridos" | — |
| 1606 | INV-012 | Benchmark + recomendación: "WONTFIX CONFIRMADO" | — |
| 1613 | INV-013 | Auditoría JSON-LD, 0 cambios | — |
| 1620 | INV-014 | Auditoría light-mode, 0 cambios | — |
| 1627 | INV-015 | Auditoría touch targets, 0 cambios | — |
| 1634 | INV-016 | Auditoría motion tokens, 0 cambios | — |
| 2184 | CODE-022 | Verificación-only: "ya fue eliminado en commit previo" | — |
| 1409 | AUD-34 | Tarea meta: actualizar conteo de commits en estos mismos docs de progreso | Baja |

**Nota:** Las entradas de investigación (INV-007…016) y SKIPs están marcadas ✅ dentro de "Tareas Completadas", inflando el conteo de completadas reales del proyecto.

---

## 6. Calidad (faltantes y sobrantes)

### Faltantes

| Línea(s) | Qué falta | Severidad |
|----------|-----------|-----------|
| 1180, 1217, 1250, 1266 | Commits ausentes: TSK-71 "*(pending — no commit yet)*", TSK-112 "*(pending)*", TSK-118, CLI-EPIC | Media |
| 1907–1941, 1958, 1968, 1979, 1991, 2003, 2015, 2028, 2039, 2054, 2066, 2095, 2106, 2118, 2141, 2161, 2173, 2182, 2191, 2200 | Entradas WEB-15/16, WEB-09…13, SEC-01/02, NUEVO-05/06, MCP-02, DX-03, DOC-09, WEB-01, WEB-08, WEB-14, DOC-11, CODE-022/027 **sin commits** (a diferencia de COMP-028/OLD-21/TSK-107b que sí los citan) | Media |
| 1571–2200 | Campo "Fuente" inconsistente: presente en INV-*/COMP-*/OLD-21/TSK-107b/GH-*/REC-*/REV-*/DRV-*, ausente en WEB-*/NUEVO-*/MKT-*/MCP-*/DX-*/DOC-*/CODE-* | Media |
| 1347–1419 | Fila de cabecera de tabla ausente en la primera tabla migrada | Alta |
| 2079–2084 | Heading `### MEM-01` ausente | Alta |
| — | Cross-reference entre secciones duplicadas (1343/1569) y entre entradas duplicadas | Media |
| 1133 vs 1357, 1139 vs 1358, 1144 vs 1373, 1146 vs 1420, 1164 vs 1421, 1177 vs 1422, 1214 vs 1423, 1243 vs 1425, 1252 vs 1463 | Fechas duplicadas o ausentes en una de las dos representaciones del mismo task | Baja |

### Sobrantes

| Línea(s) | Qué sobra | Severidad |
|----------|-----------|-----------|
| 1343 vs 1569 | Dos H2 con idéntico propósito (EN + ES) — duplicación de sección | **Alta** |
| 1347–1499 | 30+ IDs repetidos entre tablas/secciones (ver §3) | **Alta** |
| 1101–1339 | Bloques de detalle duplicados por filas de tabla equivalentes (TSK-45/71/106b/112/118, DISC-02/03, ROAD-06, CLI-EPIC, TSK-111/119/86/87/88) | Media |
| 1571–2200 | Entradas no-tarea (SKIP/no-op/verificación) listadas como "Completadas" (INV-007…016, MKT-14, INV-019, DEVOPS-13, CODE-022, SEC-01/02, DRV-109 1ª copia) | Media |
| 1341 | Separador `--` huérfano | Baja |

---

## Resumen ejecutivo

- **Estructura:** El rango contiene 3 H2 (`## Tasks Completed (Migrated from Backlog)` en 1343, `## Completed Task History` en 1503, `## Tareas Completadas (Migradas desde Backlog)` en 1569) + 3 H3 de tabla (1435, 1489) + ~60 entradas H3. El H2 de 1569 (ES) duplica el propósito del de 1343 (EN).
- **Formato:** Tablas con esquemas inconsistentes (una sin cabecera), celdas desbordadas con resoluciones incrustadas (1378–1396), 3 formatos de fecha distintos, mezcla ES/EN dentro de bloques, y el bloque MEM-01 (2079–2084) huérfano sin heading.
- **Duplicados:** 40+ IDs repetidos. Críticos: **DRV-109** (1809 y 1886), **AUD-05…AUD-18** (14 IDs en dos tablas), **AUD-WORK** (3 ocurrencias), DISC-01/04/06/07/08/09/10 (2 c/u). DOC-19/DOC-20 no existen en el rango; WEB-03 aparece 1 sola vez.
- **Contradicciones:** La colisión AUD-05…18 (misma ID, tareas distintas), DRV-109 (no-op vs implementado), WEB-13 vs INV-013 (JSON-LD implementado vs ausente), REV-001 vs NUEVO-05 (TSan eliminado vs añadido), DEVOPS-13 (niega workflows que el propio doc cita).
- **No-tareas:** ~24 entradas sin código real (autopsias NUNCA IMPLEMENTADO, investigaciones INV, SKIPs, no-ops, verificación-only) marcadas ✅.
- **Calidad:** ~30 entradas sin commit, campo "Fuente" inconsistente, falta cross-referencing; sobran secciones duplicadas y dobles representaciones del mismo task.
