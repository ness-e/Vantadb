---
title: "Auditoría de Gobernanza Documental — VantaDB"
type: audit
status: final
mode: fresh (sin delta vs last-audit-state.json)
scope: docs/ completo + archivos raíz docs/ + verificación exhaustiva contra código
date: 2026-08-21
method: 2 sub-agentes en paralelo (integridad de referencias + verificación de claims contra código) + análisis estructural del lead
---

# Auditoría de Gobernanza Documental — VantaDB (2026-08-21)

> **Alcance:** `docs/` completo (490 archivos .md, ~17 MB, 30+ carpetas), archivos raíz de `docs/`, y verificación de implementaciones declaradas contra todo el repo (`src/`, `vanta-memory/`, `desktop/`, `vantadb-mcp/`, `vantadb-server/`, bindings, `.github/workflows/`).
> **Método:** auditoría fresca. Fases ejecutadas en orden: Inventario → Maestros → Verificación código → Redundancia → Sincronización → Brechas.
> **Regla:** solo se reporta; ningún archivo fue modificado.

---

## 1. Resumen Ejecutivo

**Estado general:** la capa **doc↔código está excepcionalmente sana** — de 16 claims de implementación verificados contra código real, 14 fueron ✅ confirmados con evidencia exacta (archivo:línea), 2 ⚠️ parciales y 0 ❌ falsos. Cada tarea completada cita commit hash verificable. Esto es muy superior al promedio de proyectos de este tamaño.

El problema está en la capa de **gobernanza documental**: los índices maestros están congelados desde julio, el Backlog contradice sus propios contadores, el CHANGELOG acumula 3 semanas sin cortar release, existen dos sistemas paralelos de progreso y dos hogares de investigación, y las campañas P29/P30/P31 viven solo en plan files sin reflejo en el Backlog (que se declara "single source of truth").

**Hallazgos más graves:**

1. 🔴 **Filtro nextest inefectivo en CI** — `.config/nextest.toml:27` filtra por `not binary(python_sdk_boundary)` pero ese binario no existe (el real es `tests/api/python.rs`) → el test NO queda excluido del perfil default como se pretende. Único hallazgo con impacto de CI real. (TEST_MAP.md:86 + nextest.toml:27)
2. 🔴 **Backlog fuera de sincronía con las campañas activas** — MEM-43 tiene commit `a0bcb112` (feat merged) y estado ✅ COMPLETED en el plan `2026-08-22-vanta-final-cierre.md`, pero figura ❌ Pendiente en Backlog.md:703; P29/P30/P31 no existen como secciones del Backlog pese a que este se declara fuente única de verdad.
3. 🔴 **CHANGELOG [Unreleased] con ~650 líneas acumuladas** desde 0.5.0 (2026-07-31): Vanta Studio completo, REST `/api/v2/*` (~29 paths), crate `vanta-memory`, supersession API pública nueva (ADR-028) — sin corte de release ni evaluación semver. Riesgo directo sobre Regla 7 (release-plz).
4. 🟠 **master-index.md congelado en 2026-07-21**: 2 enlaces rotos, no indexa ~15 carpetas existentes ni 3 docs nuevos de `docs/api/`.
5. 🟠 **Contador de tareas contradictorio**: header del Backlog y ROADMAP declaran "~24 items abiertos"; conteo real de filas ❌ en Backlog.md = **45**.

### Salud documental: **6.5 / 10** *(revisada tras Volumen II; inicial 7)*

| Dimensión | Nota | Justificación |
|---|---|---|
| Exactitud doc↔código | 9 | 14/16 claims verificados con commit hash; falsos positivos casi nulos |
| Frescura de índices maestros | 4 | master-index 07-21, README 07-01, TEST_MAP 07-22, progreso resumen stale |
| Coherencia interna Backlog | 5 | contadores contradictorios, fases fantasma (P29-P31), referencias muertas |
| Taxonomía de carpetas | 5 | 2 sistemas de progreso, 2 de investigación, artefactos de build commiteados |
| Cadencia de releases/changelog | 4 | 3 semanas de Unreleased sin corte |
| Trazabilidad de decisiones | 8 | ADRs 001-029 completos, backlog-futuro re-verificado |

---

## 2. FASE 1 — Inventario

**Totales:** 490 archivos .md (~17 MB). Clasificación:

| Tipo | Carpetas principales | Volumen aprox. |
|---|---|---|
| Plan de campaña | plans/ (14) + plans/archive/ (46) | 60 |
| Investigación | Investigaciones/ (48+) , research/ (13) | 61 |
| Tarea / tracking | Backlog.md, backlog-futuro.md, progreso/, avance/ | ~25 |
| Reporte / auditoría / review | reviews/ (22), reports/ (4) | 26 |
| Referencia | glosario/ (57), api/ (11), architecture/ (+34 ADRs) | ~115 |
| Operación | operations/ (32), workflow/ (15) | 47 |
| Doc de usuario | tutorials/ (8), book/ (2 árboles, ~120 archivos con assets generados), blog/ (7), case_studies/ (2), examples/, FAQ, QUICKSTART | ~140 |
| Estrategia / visión | strategy/ (7), vision/ (1) | 8 |
| Otros | web/ (28), wasm/ (1), discord/ (6), benchmarks/ (7), graphrag/ (1), _templates/ (4), archive/ (2), .obsidian/ (14) | ~63 |
| **Vacía** | **TDAM-VANTADB/** | **0 archivos — carpeta muerta** |

Observaciones de inventario:
- `docs/book/book/` es el **output compilado de mdBook commiteado al repo** (html, css, fonts — ~90 archivos binarios/de build).
- `docs/archive/` tiene solo 2 archivos, pero múltiples documentos citan contenido "archivado" en rutas inexistentes (ver §3).
- `.obsidian/` (config de vault personal) commiteado dentro de docs/.

---

## 3. FASE 2 — Documentos maestros: estado extraído

| Doc | Fecha última revisión declarada | Estado declarado | Problema detectado |
|---|---|---|---|
| `master-index.md` | 2026-07-21 | active | 2 enlaces rotos; no indexa avance/, Investigaciones/, research/, reviews/, reports/, workflow/, web/, benchmarks/, book/, discord/, TDAM-VANTADB; no lista `api/VANTA_MEMORY.md`, `WASM_PERSISTENCE.md`, `WASM_STANDALONE.md` (creados después); cita carpeta `audit-reports/` eliminada |
| `docs/README.md` | 2026-07-01 | active | Enlaces OK, pero omite la mayoría de carpetas nuevas; describe estructura que ya no existe ("MPTS") |
| `Backlog.md` | 2026-08-07 (frontmatter) con syncs hasta 08-18 | active, "~24 items abiertos" | Contador real: **45 filas ❌**. Cita 10 veces `docs/audit-reports/*` (no existe), `docs/REPORTE_EVALUACION_COMPLETO.md` (no existe), `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md` y `PROJECT_FULL_REVIEW_2026-07-13.md` (no existen). P29/P30/P31 referenciadas pero sin sección. MEM-43 stale (ver §4) |
| `backlog-futuro.md` | revisado 2026-08-17 | freeze I+D | ✅ Consistente — re-verificación multi-agente documentada con evidencia de código; coincide con nuestra verificación independiente (RaBitQ existe, DuplicatePreventionFilter sin callers) |
| `CHANGELOG.md` | actualizado 08-20 | Unreleased + [0.5.0] 2026-07-31 | **~650 líneas en Unreleased** (líneas 8–653) sin corte de versión; workspace sigue en 0.5.0 (`Cargo.toml:645`) pese a features públicas mayores |
| `TEST_MAP.md` | 2026-07-22 | reference | 2 binarios de test mal nombrados; 1 filtro CI inefectivo; workflows y justfile targets verificados ✅ (todos existen) |
| `last-audit-state.json` | 2026-08-12 | PASS, 0 critical | ✅ Apunta a reporte existente (`docs/reviews/audit-full-20260812-231204.md` ✓). Línea base ignorada por consigna (auditoría fresca) |
| `QUICKSTART.md` (17/08), `README.md` docs (09/08), `ci-cd-guide.md` (12/08), `FAQ.md`, `chaos-testing.md` | — | — | Sin referencias rotas detectadas en el scan |

---

## 4. FASE 3 — Verificación contra el código (16 claims auditados)

Sub-agente de verificación independiente; veredicto con evidencia archivo:línea.

| # | Claim (fuente) | Clasificación | Evidencia |
|---|---|---|---|
| 1 | SEC-01 fix UAF `__array_interface__` (Backlog P1) | ✅ Implementado y bien documentado | `vantadb-python/src/types.rs:387-416` — copia owned (`PyBytes`), test `test_sdk.py:739` |
| 2 | MCP-01 `ensure_indexes_current` al arrancar MCP server | ✅ | `vantadb-mcp/src/server.rs:38`; tests `mcp_tests.rs:2008-2059` |
| 3 | MCP-03 score→distancia real (1−sim) | ✅ | `vantadb-mcp/src/handlers/tools.rs:610-629` |
| 4 | MCP-15 PrefetchGuard anti-recursión infinita | ✅ | `src/storage/engine/get.rs:24-46,234,246`; test `engine.rs:675` |
| 5 | FUT-01 RaBitQ 1-bit (XOR+POPCNT Hamming) | ✅ | `src/vector/quantization.rs:16-46`; `Binary(Box<[u64]>)` en `src/node/vector_data.rs:83` |
| 6 | FUT-09 premisa: `DuplicatePreventionFilter` sin callers | ✅ (premisa correcta) | Existe en `src/utils/duplicate_prevention.rs:30`; cero llamadas desde storage/sdk |
| 7 | RELEASE-01 gate semver-checks | ✅ (matiz de ubicación) | Está en `ci-rust-10.yml:88-118`, no en `release.yml` como dice la fila |
| 8 | Versión 0.5.0 publicada | ✅ | `Cargo.toml:645-646` — sigue en 0.5.0 |
| 9 | **P26 Studio Fases 0-4 completadas** | ⚠️ **Parcial** | Componentes y deps ✅ (`WorkspaceShell.tsx`, `Mark.tsx`, cmdk, react-querybuilder…), pero **zustand NO está en `desktop/package.json`** — `desktop/src/store/undo.ts:3` admite "zustand lo instala VS-09 en paralelo" y nunca se instaló. La fila VS-08 declara dep zustand: sobrepasa la realidad |
| 10 | ADR-028 supersession (core+Python) | ✅ | `types.rs:206,209,231`; `supersede()` en `api.rs:840-873`; filtro search `search/mod.rs:70-93`; Python `__init__.py:186-187`; ADR existe |
| 11 | REST `/api/v2/*` ~27 endpoints + dashboard | ✅ | `cli_server.rs:215-260`: **29 paths**; dashboard `nest_service("/dashboard")` :1703-1706 |
| 12 | P27 vanta-memory existe; MEM-36 pendiente | ✅ coherente | Crate sustancial (~100 archivos); `client.memory.*` ausente en TS/Python = correctamente marcada pendiente |
| 13 | CORE-01 pendiente (Binary/Turbo no persiste vector) | ✅ bien marcado como pendiente | `ops.rs:62-66`: solo `Full` persiste; comentario en `get.rs:186` |
| 14 | put_batch reconstruye índices derivados/texto | ✅ | `api.rs:384-391` con comentario explícito del fix |
| 15 | 15 tools MCP; P25 gaps aún abiertos | ✅ coherente | `tools.rs:19-166` registra exactamente 15; no existen purge/export/delete_by_filter/page_rank tools |
| 16 | explain shape anidada (T15) | ✅ | `vector_types.rs:77`; test `mcp_tests.rs:2281` |

### Tabla: tareas declaradas completadas que NO lo están (o al revés)

| Ítem | Declaración | Realidad | Severidad | Acción |
|---|---|---|---|---|
| VS-08/VS-09 (zustand) | "✅ Hecho" con dep zustand | Dep ausente de `desktop/package.json`; store undo es vanilla TS funcional | 🟡 Baja (funciona, pero el registro miente sobre el stack) | Corregir fila VS-08 o instalar la dep |
| RELEASE-01 | "job dedicado en release/ci" citing release.yml | Gate real vive en `ci-rust-10.yml:88` | 🟢 Cosmética | Corregir columna Archivos |
| MEM-43 (**caso inverso**) | Backlog.md:703 → "❌ Pendiente" | Commit `a0bcb112` merged + Task 1 del plan `2026-08-22-vanta-final-cierre.md` → "✅ COMPLETED" | 🔴 Alta (Backlog desincronizado) | Marcar ✅ en Backlog con hash |
| DESKTOP-26 premisa | "vitest hoy no configurado" | No verificado en esta auditoría | — No verificado | Confirmar antes de ejecutar |

**Veredicto global Fase 3:** no existe ningún caso grave de "documentado como hecho y NO implementado". El patrón dominante es el inverso (trabajo hecho y mal reflejado en el Backlog), que se detalla en §6.

---

## 5. FASE 4 — Redundancia estructural

| Par sospechoso | Diagnóstico | Propuesta de unificación |
|---|---|---|
| **avance/ (17 archivos)** vs **progreso/** | Dos sistemas de progreso deliberados: `avance/` es "live mirror por dominio" (activo/core-engine.md etc., actualizado 18-20/08) y `progreso/README.md` es el log histórico (**372 KB, monolito ilegible**). Duplican el mismo estado en dos formatos; `avance/COBERTURA.md` duplica datos de coverage de TEST_MAP/reviews | Consolidar: `avance/` se vuelve la **única** vista viva por dominio; `progreso/README.md` se divide por año/mes o se convierte en índice que apunte a BACKLOG_HISTORY.md + ARCHIVO_HISTORICO.md. Migración: mover secciones ≥1 mes de antigüedad a `progreso/historico-YYYY-MM.md` (git mv + link) |
| **Investigaciones/ (48+)** vs **research/ (13)** | `research/` nació 2026-08-18 como formato nuevo estructurado (PLAN/NN-RESEARCH/SYNTHESIS) para TDAM y human-facing-db-ui; `Investigaciones/` sigue recibiendo archivos sueltos (ej. `cargo-check-optimizacion.md`, `DESKTOP-01*.md`, `TIR-*.md`). Dos convenciones conviviendo | Declarar `research/` formato canónico para campañas multi-doc y `Investigaciones/` para investigaciones puntuales — **documentar la regla en docs/README.md**, o migrar los 48 a research/ con subcarpetas. Opción lazy: regla escrita > migración masiva |
| **reviews/ (22)** vs **reports/ (4)** | Funciones realmente distintas (auditorías puntuales vs métricas de pipeline dora/northstar/pipeline-evals) pero nombres indistinguibles. Además, el ex-directorio `audit-reports/` fue disuelto sin actualizar sus 12+ referencias (Backlog ×10, master-index ×1, ROADMAP ×2) | Renombrar conceptualmente en README: reviews/ = auditorías, reports/ = telemetría del pipeline. Crear `docs/audit-reports/REDIRECT.md` de 3 líneas que diga dónde quedó cada reporte archivado, para matar las referencias muertas sin tocar 12 archivos |
| **strategy/ROADMAP.md vs Backlog** | ROADMAP es histórico con banner honesto, pero repite el contador "~24 abiertos" ya falso | Dejar el banner, borrar el número puntual (apuntar al Backlog sin cifra) |
| **book/src vs book/book** | `book/book/` (~90 archivos html/css/fonts) es artefacto de build commiteado | Agregar `docs/book/book/` a .gitignore + build en CI; git rm --cached |
| **TDAM-VANTADB/** | Carpeta vacía | `rmdir` |
| **blog triplicado** | Posts reales en `docs/blog/` (7), stubs redirect en `book/src/blog/`, y master-index afirma que viven en `web/content/blog/` que **no existe** | Corregir la frase de master-index:161; mantener stubs del book como redirects |
| glosario/ (57) | Correcto y bien indexado | Sin acción |
| plans/ vs plans/archive/ | Correcto (activo mínimo, histórico archivado) | Sin acción |

---

## 6. FASE 5 — Sincronización cruzada

### Matriz de sincronización de documentos maestros

| Par | ¿Sincronizados? | Evidencia |
|---|---|---|
| ROADMAP ↔ Backlog | ⚠️ Parcial | Banner honesto en ambos sentidos, pero ambos repiten "~24 abiertos" vs 45 filas ❌ reales |
| Backlog ↔ backlog-futuro | ✅ | Re-verificación 08-17 mutuamente consistente; coincide con nuestro check de código |
| Backlog ↔ plans/ | 🔴 Roto | Campaña activa P31 (`plans/2026-08-22-vanta-final-cierre.md`, in-progress) sin fase en Backlog; P29/P30 cerradas solo en plans/progreso; MEM-43 completado en plan+código pero Pendiente en Backlog:703 |
| plans ↔ CHANGELOG | ⚠️ Parcial | Los commits de campañas llegan a CHANGELOG vía conventional commits ✅, pero no hay release que los agrupe |
| CHANGELOG ↔ releases | 🔴 Roto | [Unreleased] líneas 8-653 (~3 semanas): Vanta Studio F0-F4, REST v2, vanta-memory, supersession API pública — sin tag desde v0.5.0 (31/07). La API pública cambió (nuevos campos/métodos SDK) sin evaluación minor/major |
| Backlog ↔ progreso/avance | ⚠️ | progreso/README "FASE 4 en ejecución (2026-08-19)" — obsoleto: P26 cerró 20/08 y P27→P31 ocurrieron después; avance/activo sí está fresco (20/08) |
| master-index ↔ realidad | 🔴 Roto | Congelado 3 semanas antes de la mayor expansión de docs (P26/P27 crearon research/, avance/activo/, api/VANTA_MEMORY.md…) |

### Hallazgos específicos pedidos por la consigna

- **Ítems "doc old" extraídos y nunca planificados:** ninguno detectado — el proceso EXTRACCION-DOC-OLD (archive/EXTRACCION-DOC-OLD-2026-08-05.md) trazó batch CI-01..07 con cierre documentado; OLD-01 (PGWire) permanece trackeado como roadmap. ✅
- **Tareas completadas que no figuran en CHANGELOG:** ninguna detectada a nivel de entrada individual (los commits convencionales alimentan el Unreleased); el problema es la falta de corte de versión, no entradas faltantes.
- **Planes huérfanos / tareas sin plan:** `plans/2026-08-19-web-design-audit.md` y `plans/2026-08-21-vanta-{context-engine,proxy-knowledge}.md` (budget.json sin .md visible para context-engine/proxy-knowledge) no tienen fila de tracking en Backlog; P31 tiene 8 tasks sin fase Backlog.
- **Fechas/estados contradictorios:**
  - Plan `2026-08-22-vanta-final-cierre.md` creado el 21/08 cita "auditoría final post-P30 … 2026-08-22" (fecha futura respecto de su creación; naming adelantado un día).
  - Backlog.md:703-705 dice "Detectado … 2026-08-21 (post-P30)" pero esas filas ya estaban parcialmente ejecutadas el mismo día (commit a0bcb112) — ventana de stale de <24 h, aceptable pero confirma que el Backlog ya no es el registro en tiempo real, lo es el plan file.
  - `ADR-026-vanta-studio-fase3-rest-dashboard.md` vive en `docs/architecture/` raíz mientras todos los demás ADRs viven en `docs/architecture/adr/` — DESKTOP-27 lo cita como si estuviera en adr/.
  - TEST_MAP.md:86 + `.config/nextest.toml:27`: filtro `binary(python_sdk_boundary)` no matchea nada → exclusión de test inoperante en perfil default (impacto CI).

---

## 7. FASE 6 — Brechas de backlog priorizadas

Tareas que deberían existir en el Backlog según el estado real y no existen:

| Prioridad | Brecha propuesta | Justificación (evidencia) |
|---|---|---|
| 🔴 Crítica | **SYNC-01: Reparar filtro nextest `python_sdk_boundary`→`python`** (`.config/nextest.toml:27` + TEST_MAP.md:86) | Único hallazgo con efecto de CI real hoy |
| 🔴 Crítica | **SYNC-02: Corte de release 0.6.0** — triage del Unreleased (¿algún cambio es breaking? supersession añade campos a `VantaMemoryRecord`), ejecutar release-plz | 650 líneas sin publicar; Regla 7 propia |
| 🟠 Alta | **SYNC-03: Re-sincronizar Backlog ↔ planes** — marcar MEM-43 ✅ (hash a0bcb112), crear sección/fila para P31 (8 tasks) o registrar P29/P30 en BACKLOG_HISTORY con puntero al plan | El Backlog pierde su rol de single source of truth |
| 🟠 Alta | **IDX-01: Regenerar master-index.md** — quitar 2 enlaces rotos (audit-reports:184, PROMPT-MAESTRO-FREEZE:192), indexar las 15 carpetas faltantes y los 3 docs nuevos de api/; corregir frase blog (:161) | Puerta de entrada a toda la doc |
| 🟠 Alta | **IDX-02: Purgar referencias muertas del Backlog** — 10 refs a `docs/audit-reports/*`, `REPORTE_EVALUACION_COMPLETO.md` ×2, 2 reviews inexistentes (líneas 213, 230, 341, 427-431) | Rompen la trazabilidad que el propio Backlog promete |
| 🟡 Media | **GOV-01: Decisión de taxonomía** — regla escrita avance↔progreso e Investigaciones↔research en docs/README.md; split del monolito progreso/README.md (372 KB) | Costo de mantenimiento creciente |
| 🟡 Media | **GOV-02: Mover `ADR-026-*` a `adr/`** y actualizar DESKTOP-27 | Convención rota de ADRs |
| 🟢 Baja | **GOV-03:** gitignore `docs/book/book/` + `rmdir TDAM-VANTADB` + decidir destino de `.obsidian/` | Higiene de repo |
| 🟢 Baja | **GOV-04:** corregir fila VS-08 (zustand) y RELEASE-01 (ubicación del gate) | Precisión del registro |

---

## 8. Plan de acción priorizado

**Crítico (esta semana):**
1. SYNC-01 fix nextest filter + TEST_MAP (30 min).
2. SYNC-02 evaluar breaking changes del Unreleased y cortar 0.6.0 vía release-plz (2-4 h).
3. SYNC-03 sync Backlog↔planes: MEM-43 ✅, fase P31, archivo P29/P30 (1 h).

**Alto (próximas 2 semanas):**
4. IDX-01 regenerar master-index + IDX-02 purga de refs muertas (2-3 h, delegable a vanta-docs).
5. GOV-02 ADR-026 a adr/.

**Medio (próximo mes):**
6. GOV-01 decisión de taxonomía documentada + split de progreso/README.md.
7. Re-auditoría delta programada (reusar last-audit-state.json como mecanismo, actualizándolo tras cada auditoría).

**Bajo (opportunista):**
8. GOV-03 higiene de repo; GOV-04 precisiones de filas.

---

## Anexo — Metodología y límites

> **⚠️ VOLUMEN II (2026-08-22):** este Volumen I cubrió maestros, planes activos, claims recientes y enlaces. Las áreas marcadas abajo como no verificadas fueron cerradas en una segunda pasada profunda — ver **§9 Volumen II** al final del documento.

- Verificación de código realizada por 2 sub-agentes independientes (ses_fd879a81fffeIKTnhazHMz7FW7 enlaces, ses_fd871f506ffeWVlnrKfAtef85m claims) + análisis directo del lead.
- Muestras no cubiertas: los ~210 items ✅ históricos del Backlog migrados a progreso/BACKLOG_HISTORY.md no fueron re-verificados uno a uno (se verificó una muestra estratificada de 16 claims recientes/pivotes, todos con commit citado). Marcar cualquier conclusión extendida al resto como **no verificado**.
- `docs/book/`, `tutorials/` y `web/guides|reference|standards` se inventariaron pero su contenido técnico no fue verificado contra API en esta pasada (contexto secundario de gobernanza).

---
---

# VOLUMEN II — Pasada profunda de las áreas no cubiertas (2026-08-22)

> **Método:** 7 sub-agentes adicionales en paralelo. Se leyeron y verificaron: operations/ (33 docs), api/ (11 docs vs superficie real), architecture/ + 34 ADRs, reviews/ + reports/ (12 archivos con cruce contra Backlog/progreso), Investigaciones/ (58 archivos), progreso/ + avance/ (contra 793 commits de git desde 01/08), carpetas menores (workflow, benchmarks, wasm, discord, web, graphrag, case_studies, examples, _templates), doc de usuario (tutorials ×8, book/src ×90 stubs, glosario muestreado, blog ×7, QUICKSTART, FAQ).
> **Cobertura resultante:** las zonas "no verificadas" del Volumen I quedan cerradas. Único remanente: BACKLOG_HISTORY verificado por muestra de 8 filas (calidad alta, todas con commit citado) — no fila a fila.

## V2.1 — Resumen ejecutivo del Volumen II

La segunda pasada **confirma el patrón** del Volumen I (doc↔código sano, gobernanza débil) pero baja la nota por hallazgos nuevos que afectan a usuarios externos y operadores:

1. 🔴 **`openapi.yaml` define 3 paths de ~29 reales** y el gate `check-api-version` solo valida el campo `version`, nunca los paths — el contrato REST público está desprotegido. (`gate-docs-21.yml:56-81`)
2. 🔴 **`case_studies/`: clientes ficticios presentados como deployments reales** ("EdgeSense" RPi5, "CodexAgent" M2 Max con tablas comparativas y benchmarks sin fuente ni disclaimer). Riesgo reputacional directo para Show HN; contradice el propio estándar "honest results" de PERF-03.
3. 🔴 **`PYTHON_RELEASE_POLICY.md` niega la publicación a PyPI que ya ocurrió** (0.5.0 live 2026-08-01) — política de release operando sobre una premisa falsa.
4. 🟠 **El mirror `avance/` nació roto**: su contrato (`meta.md`) promete cobertura viva por dominio, pero faltan dominios enteros creados después (vanta-proxy, vanta-memory/TDAM, context engine) y quedó congelado el 20/08. `bitacora.md` muerta desde el 27/07 (narrativa) / 11/08 (git).
5. 🟠 **4 valores de coverage contradictorios conviviendo**: gate ≥59% (TEST_MAP/CI_POLICY) vs ≥80% ADR-015/018 vs "80.55% CII Silver" (progreso README:32) vs 81.40% root — sin fuente canónica única.
6. 🟠 **Tutoriales rotos para usuarios reales**: `graph_bfs("doc1","doc3")` con firma incorrecta en los 2 tutoriales de migración (TypeError garantizado); `ef_search` como parámetro inexistente en glosario/hnsw.md.

### Salud documental revisada: **6.5 / 10**

---

## V2.2 — operations/ (33 docs): 19 ✅ · 8 ⚠️ · 2 ❌ · 4 menores

| Doc | Veredicto | Hallazgo | Evidencia |
|---|---|---|---|
| CONFIGURATION.md | ⚠️ | Default `rate_limit_rpm` **100 en doc vs 600 real**; faltan 4 env vars usadas en código | config.rs:299,659 |
| CI_POLICY.md | ⚠️ | "14 workflows" vs 17 reales; contradicción interna de umbral coverage (59% vs 80%) | ci-rust-10.yml:360-378 |
| DURABILITY_GUARANTEES.md | ⚠️ | WAL CRC32C/quarantine ✓, pero cita rutas muertas `src/storage.rs:*` y knob `flush_interval_ms` inexistente | grep flush_interval = 0 hits |
| DEPLOYMENT_GUIDE.md | ⚠️ | Documenta fallback env `PORT` que no existe | L411 vs config.rs:518 |
| DISASTER_RECOVERY_RUNBOOK.md | 🟡 | Flags `doctor --fix` / `restore --dry-run` ausentes de la tabla CLI canónica — posible flag fantasma | L142,233 |
| RELIABILITY_GATE.md | ⚠️ | Failpoints con rutas muertas post-refactor a `src/storage/engine/` | L107-109 |
| GC_TTL.md | 🟡 | Claims de knobs GC sin respaldo (0 hits `VANTA_GC*` en src) | grep |
| **PYTHON_RELEASE_POLICY.md** | ❌ | "does not publish to PyPI" — falso: 0.5.0 publicado en PyPI | progreso/README.md:365 |
| **operations/master-index.md** | ❌ | Lista 26 de 32 archivos — faltan 7 | comparación dir |
| BENCHMARKS, SECURITY, FUZZING, BACKUP_POLICY, GRAFANA_SETUP, PERFORMANCE_GUIDE/TUNING, MEMORY_TELEMETRY, SQLITE_MIGRATION, chaos-testing, pilot-* ×4, etc. | ✅ | Verificados OK (chaos-testing.md es el único con failpoint paths correctos y actuales) | detalle en sesión |

**Env vars en código NO documentadas:** `VANTA_EMBEDDING_PROVIDER`, `VANTA_OPENAI_API_KEY`, `VANTA_OPENAI_MODEL` (src/llm.rs:40,132,147), `VANTADB_REPORTED_VERSION` (src/metadata.rs:22).

## V2.3 — api/ (11 docs): superficie real vs documentada

| Métrica | Real | Documentado | Gap |
|---|---|---|---|
| Endpoints `/api/v2/*` | ~29 (+health/metrics/conversation/skill) | HTTP_API.md: **4** · openapi.yaml: **3** | 26 sin documentar |
| Tools MCP dispatch | ~33 (15 core + 6 skill + 8 code_* + 4 wiki_*) | MCP.md: 21 · skills mcp-protocol.md: "15" | 3 cifras incompatibles entre fuentes |
| Métodos públicos SDK | ~75 total (34 en api.rs) | EMBEDDED_SDK cubre ~60 principales | aceptable |

Otros hallazgos api/:
- **HTTP_API.md:108** usa sintaxis LISP `(memory:get ...)` que MCP.md declara no soportada.
- **MCP.md** no documenta DimensionMismatch ni envelope/isError — eso vive solo en `skills/vantadb-mcp/references/` → dos fuentes de verdad divergentes.
- **TS_SDK.md**: `deleteByFilter()`, `graphDegree()`, `graphIsDag()` exportados y sin documentar (vantadb.ts:488,974,921).
- **IQL.md**: faltan JOIN, subqueries, PROFILE y la advertencia LINK-silencioso.
- **PYTHON_SDK.md es ejemplar** — marca "(not yet exposed)" donde corresponde y todo lo documentado existe.
- GRAPH_RAG.md, VANTA_MEMORY.md, WASM_* ✅ consistentes. Ningún doc cita versiones viejas como actuales.

## V2.4 — architecture/ + ADRs (51 docs): mayormente sanos

- Los 34 ADRs: implementaciones afirmadas verificadas contra código (supersede(), batch_append, CRC32C, HnswConfig, features Cargo.toml) — coherentes. Drift cosmético de números de línea en ADR-024/025.
- **ADR-008 (WASM storage)**: la queja de WASM_STORAGE_REVIEW.md quedó resuelta en código (opfs.rs, idb.rs, worker.rs) — el review es histórico válido.
- Problemas estructurales: `ADR-026-vanta-studio-fase3-rest-dashboard.md` fuera de `adr/`; colisión de numeración "001" (001_unified_config vs ADR-0001); colisión "019" entre series; ARCHITECTURE.md cita `src/node.rs` ×4 cuando el módulo es ahora `src/node/unified.rs`.
- wiki-links rotos en case_studies hacia ADRs con filenames inexistentes.

## V2.5 — reviews/ + reports/: 0 huérfanos duros

- **Todos los findings ≥media de los 12 reportes están ticketeados/resueltos/descartados con evidencia** — incluida la última auditoría (AUD-022..041 resueltos con commit; AUD-042 activo y correctamente marcado BLOQUEADO upstream). El sistema de procesamiento de hallazgos funciona.
- 2 huérfanos blandos: P1-2 SEC-MMAP-UB (defer documentado solo in-situ, sin nota de re-evaluación); follow-ups TIR-02(a)/TIR-04(b)/TIR-08(c) en prosa del header P18, sin filas de tarea.
- **reports/ stale (11 días)** y con **contradicción de métricas**: pipeline-evals.md reporta 0.0% primer-intento + 1 regresión; northstar.md reporta 100% + 0 regresiones del mismo evento.

## V2.6 — Investigaciones/ (58 archivos leídos)

- Mayoría procesada correctamente. Hallazgos:
  - ❌→backlog: **follow-ups TIR-02/04/08 aceptados y nunca ticketeados como filas** (~36 líneas de trabajo total).
  - ⚠️ Sin nota de cierre: ACID_TRANSACTIONS (plan 4a-4d completo sin decisión registrada de implementar o rechazar), MVCC_SNAPSHOT_ISOLATION, VantaDB-28-07, COGNEE_EVALUATION (absorbida por TDAM).
  - ⚠️ Contradicción COMP-010 auto-embedding: historial dice implementado 2026-07-27 (.opencode/tasks/COMP-010.md:84) pero el plan P31 Task 4 declara "NO verificado contra código" — doble-verificación pendiente que el plan ya trackea.
  - 🗑️ Colisión de ID: dos investigaciones distintas comparten ID `INV-019`.
  - 🔗 Referencia muerta sistémica: `cargo-check-optimizacion.md` **no existe** pero Backlog.md:22 lo cita como origen.

## V2.7 — progreso/ + avance/: ~87% de cobertura commits↔registro

Sobre **793 commits desde 01/08**:
- progreso/README.md registra bien hasta el 21/08 (muestreo 15 commits → 13 registrados). **No registrados: MEM-43 (`a0bcb112`), MEM-44 (`785db22c`), plan P31 (`f76502b2`).**
- **avance/activo (mirror) roto desde su creación respecto a su contrato**: faltan dominios vanta-proxy, vanta-memory/TDAM, context engine; Vanta Studio F2-F4 solo parcialmente reflejado; congelado al 20/08.
- bitacora.md muerta (última narrativa 27/07). Duplicación confirmada: mismo evento archivado 3× (README.md:373,385,389). Resumen ejecutivo obsoleto ("FASE 4 en ejecución").
- BACKLOG_HISTORY.md: calidad alta (muestreo 8 filas, todas con commit/evidencia).
- **Coverage contradictorio en 4 documentos** (ver V2.1 punto 5).

## V2.8 — Carpetas menores

| Carpeta | Veredicto | Hallazgo clave |
|---|---|---|
| workflow/ | ✅ | 15/15 nombres coinciden con .github/workflows; faltan docs para ci-examples-12.yml y release.yml |
| benchmarks/ | ✅/⚠️ | Consistente con PERF-01 sellado; `_run_stdout.md` (log crudo con traceback) es artefacto a eliminar |
| wasm/CRASH_MODEL.md | ⚠️ | Dice "serialize ALL records" pero PERF-08 introdujo persistencia diferencial (lib.rs:261-268,749) |
| discord/ | ✅ | Coincide con DISC-01..03 del Backlog |
| web/ | ✅/⚠️ | Guías verificadas OK salvo 1 import roto (routing.md → view-component.tsx inexistente); DESIGN_RULES.md raíz duplica standards/design-rules.md |
| graphrag/README.md | ✅ | FUT-03 sigue vigente confirmado (0 hits leiden/louvain en src/) |
| **case_studies/** | ❌ | Clientes ficticios sin disclaimer (ver V2.1 punto 2) |
| examples/ | ✅ | Imports válidos; `__pycache__/` commiteado a eliminar |
| _templates/ | ✅ | Usados activamente |

## V2.9 — Doc de usuario: seguible al 95%, con rotas puntuales

- **Rotas confirmadas:** `graph_bfs("doc1","doc3")` firma incorrecta en tutorials/03:178 y migration-from-lancedb.md:281 (firma real `(roots, max_depth)`, lib.rs:1612); `ef_search` parámetro fantasma en glosario/hnsw.md:121; FAQ claim "Periodic = fsync cada 5s" contradice wal.rs:338 (default sync every write); URL GitHub inconsistente (FAQ `vantadb/vantadb` vs QUICKSTART/blog `ness-e/Vantadb`).
- **Glosario faltante:** IQL, RaBitQ, TDAM, supersession, vanta-memory, context engine. Métricas de rrf.md/bm25.md sin respaldo en benchmarks sellados.
- **book/src:** son stubs `{{#include}}` de 1 línea → sin riesgo de doble mantenimiento ✓; SUMMARY completo salvo tutorial Vectara y 4 posts de blog.
- QUICKSTART ✅ verificado comando por comando. Blog: claims numéricos honestos y consistentes; pendiente solo el drift cosmético de BLOG-CTA.

---

## V2.10 — Tablas entregables actualizadas

### Documentos obsoletos (consolidado V1+V2, ordenado por severidad)

| Documento | Problema | Severidad | Acción |
|---|---|---|---|
| openapi.yaml | 3 paths de ~29; gate no valida paths | 🔴 | Regenerar spec desde cli_server.rs + extender gate |
| case_studies/ (ambos) | Ficticios sin disclaimer | 🔴 | Disclaimer visible o reescribir como "reference architectures" |
| PYTHON_RELEASE_POLICY.md | Niega publish a PyPI ya ocurrido | 🔴 | Reescribir policy post-0.5.0 |
| HTTP_API.md | 26 endpoints sin documentar + ejemplo LISP muerto | 🔴 | Regenerar |
| MCP.md ↔ skills/references | Cifras de tools divergentes (21 vs 15 vs ~33); DimensionMismatch/envelope solo en skills | 🟠 | Unificar fuente, cross-ref |
| avance/activo/* | Mirror roto: dominios faltantes, congelado 20/08 | 🟠 | Crear vanta-memory.md + vanta-proxy.md; catch-up |
| CONFIGURATION.md | rate_limit_rpm erróneo; 4 env vars faltantes; 4 sin documentar | 🟠 | Sync puntual |
| master-index.md + operations/master-index.md | Rotos e incompletos (7+15 rutas) | 🟠 | Regenerar |
| wasm/CRASH_MODEL.md | No refleja persistencia diferencial PERF-08 | 🟡 | Actualizar §persistencia |
| tutorials 03 + migration-lancedb | graph_bfs rota en runtime | 🟠 | Corregir snippet (usuarios reales tropiezan) |
| glosario/hnsw.md + faltantes | ef_search fantasma; 6 términos ausentes | 🟡 | Corregir + añadir |
| FAQ.md | fsync claim falso; URL inconsistente | 🟡 | Corregir 2 líneas |
| ARCHITECTURE.md | src/node.rs ×4 desactualizado | 🟢 | sed trivial |
| TEST_MAP.md + nextest.toml | Binarios mal nombrados; filtro CI inefectivo | 🔴 (CI) | Fix 30 min (ya en plan V1) |
| CHANGELOG.md | 650+ líneas Unreleased sin corte | 🔴 | Release 0.6.0 (ya en plan V1) |
| reports/{northstar,pipeline-evals}.md | Contradicción mutua de métricas; stale | 🟡 | Regenerar o anotar divergencia |
| benchmarks/_run_stdout.md, examples/__pycache__/, TDAM-VANTADB/, web/DESIGN_RULES.md, book/book/ | Artefactos/duplicados/vacíos | 🟢 | Limpiar |

### Brechas de backlog nuevas (Volumen II)

| Prioridad | Brecha | Justificación |
|---|---|---|
| 🟠 Alta | TIR-02a recovery time en evals/dora.mjs (~30 líneas) | Decisión tomada, nunca ticketeada |
| 🟠 Alta | TIR-04b formalizar tasks/closed/ | ídem |
| 🟡 Media | TIR-08c criterios en research-agent.md (~6 líneas) | ídem |
| 🟡 Media | Decisión ACID 4a-4d (implementar / rechazar con registro) | Diseño completo sin destino |
| 🟡 Media | Resolver doble-verificación COMP-010 (plan P31 Task 4 ya lo trackea — no duplicar, solo ejecutar) | Historial vs plan contradictorios |
| 🟡 Media | Renumerar INV-019 duplicado | Ambigüedad de referencias |
| 🟢 Baja | Notas de cierre en 4 investigaciones obsoletas-sin-marcar | Higiene |
| 🟢 Baja | Docs de workflow para release.yml + ci-examples-12.yml | Completitud |

### Plan de acción V2 (se suma al de V1 §8)

**Crítico (antes de cualquier exposición pública / Show HN):**
1. Disclaimer o reescritura de case_studies/ (30 min) — riesgo reputacional máximo.
2. Corregir snippets rotos de tutoriales + ef_search + FAQ fsync + URL GitHub (1 h).
3. Regenerar openapi.yaml + HTTP_API.md desde cli_server.rs (medio día, delegable a vanta-docs).

**Alto:** catch-up del mirror avance/ (dominios vanta-memory + vanta-proxy + P31); unificar cifra de coverage a valor canónico único (ADR-018); ticketear TIR-02a/04b/08c; reescribir PYTHON_RELEASE_POLICY.md.

**Medio:** unificar MCP.md↔skills/references en una sola fuente con cross-ref; regenerar reports/ (northstar/pipeline-evals) resolviendo su contradicción; notas de cierre en investigaciones obsoletas; limpiar artefactos (_run_stdout, __pycache__, TDAM-VANTADB, DESIGN_RULES.md, book/book).

---

## Anexo V2 — Metodología y límites

- Sub-agentes de esta pasada (sesiones): fd85e5e51ffeOFAt1r7gdLLY2N (operations), fd85e231dffeKLncfhMMpgZHlQ (api), fd85de5ddffeRIkEgt5Q0Qm3Ax (architecture/ADRs), fd85daeeeffeAQPdsld6iz7yfm (reviews/reports), fd855aa37ffeqXdLg1K6f5XsTV (carpetas menores), fd8557649ffedLROwZOxNV4K58 (doc usuario), fd84dc26dffeu2L5TaZ5irrJow (progreso/avance vs git), fd84936afffegj6yM1rTGXRGTd (Investigaciones).
- Todo read-only; ningún archivo modificado salvo este informe.
- Límites restantes: BACKLOG_HISTORY verificado por muestra (8/210 filas); snippets de tutorial no ejecutados (verificación estática de firmas contra código); claims de benchmarks no re-ejecutados (solo consistencia documental).

---

## ADDENDUM (2026-08-22) — Verificación dirigida post-informe

Dos ítems pendientes de §V2 fueron resueltos contra el código:

1. **Tantivy (resuelve la duda de §3.4):** `tantivy` SÍ es dependencia real (`Cargo.toml:86`, opcional, feature `advanced-tokenizer`; lockfile :4130). El hallazgo del sub-agente de glosario ("BM25 sin dependencia tantivy") fue incorrecto. Matiz correcto: tantivy aporta únicamente el tokenizador multilingüe (`src/text_index.rs:26-29`); postings/scoring BM25 son implementación propia. Acción corregida: ajustar `glosario/bm25.md` (negación falsa) y precisar master-index/TEXT_INDEX_DESIGN ("tokenizer Tantivy + índice invertido propio", no "Tantivy-based" a secas).
2. **Flags fantasma del runbook (CONFIRMADOS — severidad elevada 🟡→🔴):** `src/cli.rs:130-144` — `Restore` solo acepta `--input/--force/--rebuild`; **no existe `--dry-run`**. `Doctor` no acepta flags; **`doctor --fix` no existe**. `DISASTER_RECOVERY_RUNBOOK.md:233,266` instruye `restore --dry-run` como verificación DIARIA de backups → el comando fallaría tal cual documentado. Es el doc de operations más peligroso del repo: falla justo cuando se necesita. Acción: reescribir los comandos con la CLI real o añadir los flags al CLI (decisión de producto).

---

# PLAN DE REVISIÓN — Registro de decisiones del owner (2026-08-22)

> Respuestas registradas vía sesión de preguntas. Este registro define el plan de revisión, investigación, análisis y ejecución derivado de esta auditoría.

## D1. Alcance del plan

**Decisión: PLAN COMPLETO.** El plan cubre todo Volumen I + Volumen II, incluyendo severidades 🟡 y 🟢 (glosario, artefactos, ADR drift, regeneración de reports), no solo críticos/altos.

## D2. Modo de ejecución

**Decisión: SOLO PLAN DOCUMENTADO.** Se entrega un plan detallado con tareas, estimaciones y orden. La ejecución la dispara el owner cuando decida. *Excepción registrada en D13.*

## D3. Condicionante temporal

**Decisión: FECHA SHOW HN PRÓXIMA (Sept 2026).** Los fixes reputacionales/públicos son bloqueantes del lanzamiento y van primero: case_studies (→D6), openapi.yaml/HTTP_API.md, tutoriales rotos, QUICKSTART/FAQ, README raíz.

## D4. Comandos fantasma del runbook (`restore --dry-run`, `doctor --fix`)

**Decisión: DOCS YA + CLI EN BACKLOG.** (a) Corregir DISASTER_RECOVERY_RUNBOOK.md a la sintaxis real existente (`restore --input ... [--force|--rebuild]`, verificación alternativa sin dry-run) como fix inmediato; (b) crear tarea de backlog para añadir `--dry-run` a `Restore` y `--fix` a `Doctor` en `cli.rs` (código+tests+semver).

## D5. Release 0.6.0

**Decisión: DIFERIR.** No se corta release en este plan. El Unreleased (~650 líneas) queda acumulando; el triage semver queda documentado como pendiente para cuando el owner decida cortar.

## D6. Case studies ficticios (EdgeSense/CodexAgent)

**Decisión: ELIMINAR.** Ambos documentos salen de docs/case_studies/ antes del lanzamiento público. Opcionalmente pueden preservarse en archive/ como material interno no-público. CLD-04 (case study con pilot real) sigue siendo el camino canónico.

## D7. Coverage canónico

**Decisión: MEDIR Y FIJAR.** Ejecutar `cargo llvm-cov` una vez para obtener la cifra real, y fijarla como valor único canónico en ADR-018 + TEST_MAP.md + CI_POLICY.md + progreso/README.md, eliminando los otros tres valores (59%/80%/80.55%/81.40%).

## D8. Fuente de verdad MCP (re-preguntada tras verificación de código)

**Verificado:** el server expone exactamente **33 tools** (15 core + 6 skill_* + 8 code_* + 4 wiki_*), todas anunciadas en tools/list (`handlers/tools.rs:180-184`). Hoy MCP.md documenta 21 y la skill dice 15.
**Decisión: SKILL COMO FUENTE ÚNICA.** `skills/vantadb-mcp/references/api-reference.md` (+SKILL.md) se expande a las 33 tools y se convierte en fuente canónica; `docs/api/MCP.md` pasa a stub con link hacia la skill. Regla hash-SAME skill↔`.opencode/skills/` se mantiene.

## D9. Consolidación progreso ↔ avance

**Decisión: AVANCE DOMINA.** `docs/avance/activo/*` se vuelve LA vista viva por dominio (con catch-up de dominios faltantes: vanta-memory/TDAM, vanta-proxy, context engine). `progreso/README.md` se reduce a índice + historial archivado por meses; su monolito de 372KB se divide. `bitacora.md` se archiva oficialmente.

## D10. Verificación dinámica

**Decisión: INCLUIDA EN EL PLAN.** Ejecutar código real: snippets de tutoriales/ejemplos, `cargo test` (reconciliar las 3 cifras citadas), `cargo llvm-cov` (alimenta D7), comandos CLI (`vanta-cli doctor --help`, restore real), y consulta de registros crates.io/PyPI/npm.

## D11. Zonas nunca auditadas

**Decisión: AUDITAR TODO.** Entran al plan: README.md + README_ES.md raíz, CONTRIBUTING.md, SECURITY/SUPPORT/CLA raíz, VantaDB_Manual_Estrategico_Unificado.md (164KB), SKILLS-MANIFEST.md (conteo declarado vs instaladas), `.opencode/` (AGENTS/agents/rules/references), integrations/ + providers/ (READMEs y claims), contenido profundo de workflows CI, plans/archive/ (46 archivos).

## D12. Artefactos y duplicados

**Decisión: PROPONER SIN EJECUTAR.** El plan lista la eliminación propuesta (gitignore book/book/, borrar __pycache__/, TDAM-VANTADB/, benchmarks/_run_stdout.md, fusionar DESIGN_RULES.md↔standards/design-rules.md) marcada como **requiere aprobación separada** antes de cualquier borrado.

## D13. Micro-fixes TIR (task-system)

**Decisión: EJECUTAR YA (excepción a D2).** TIR-02a (recovery time en evals/dora.mjs ~30 líneas), TIR-04b (formalizar tasks/closed/: regla de movimiento al escalar + re-procesamiento pending + índice rg "❌ FAILED") y TIR-08c (criterios saturación<20% + broadening/narrowing en research-agent.md ~6 líneas; jitter WONTFIT) se ejecutan directamente dentro del plan — total ~1h, decisión ya tomada en sus investigaciones.

## D14. ACID 4a-4d (rollback coordinado multi-capa)

**Decisión: POST-LAUNCH TICKETEADO.** INV-010 (diseño completo 2PC con WAL árbitro: Prepare record, commit point tras aplicar al store, fases 4a WAL keystone / 4b KV pre-image / 4c VantaFile watermark / 4d índices derivados) se ticketea en Backlog como fase post-Show-HN con prioridad definida. La inconsistencia teórica post-abort queda documentada como deuda conocida de durabilidad hasta entonces.

## Resumen de decisiones → estructura del plan resultante

| # | Decisión | Impacto en el plan |
|---|---|---|
| D3+D6 | Show HN próximo + eliminar case_studies | Wave 0 bloqueante: reputación pública |
| D4 | Runbook docs ya, CLI después | Fix docs inmediato + tarea backlog |
| D5 | Diferir release | Sin corte 0.6.0; triage semver documentado |
| D7+D10 | Medir coverage + verificación dinámica | Fase de medición ejecutable al inicio |
| D8 | Skill fuente única MCP | Reescritura api-reference a 33 tools + stub MCP.md |
| D9 | avance domina | Catch-up dominios + split monolito progreso |
| D11 | Auditar todo lo intocado | Segunda ola de auditoría (raíz, manual, .opencode, integrations) |
| D12 | Artefactos propuestos | Lista de borrado con flag de aprobación |
| D13 | TIR directo | ~1h de ejecución temprana |
| D14 | ACID post-launch | Ticket en backlog, no ejecución |

> **Siguiente paso acordado:** redactar el plan de ejecución detallado (tareas atómicas, waves, estimaciones, agentes asignados) a partir de estas decisiones.
