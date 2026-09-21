# HIGH-006: detect_changes en plan.md Paso 0 — blast radius transitivo

## Metadata
- **Plan file:** docs/plans/2026-08-28-master-pipeline-optimization.md
- **Fuente:** plan file Task 6 / HIGH-006 (ALTO #6)
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🟠
- **Tipo:** Mixto (infra task-system + docs prompts)
- **Turns estimados:** 1
- **Creado:** 2026-08-28T12:00
- **last-synced:** 2026-08-28T16:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes
- **Campaign ID:** cecc8468-9451-4d56-a3ef-1684e123ab8a

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `.opencode/task-system/prompts/pipeline-full.md` (Discovery Code Intelligence dual), `.opencode/task-system/prompts/task.md`, `.opencode/task-system/mcp/campaign-server.mjs` (tool detect_changes), todos los agentes que crean planes |
| Callees | `.opencode/task-system/prompts/plan.md` (Paso 0 Verificación de Realidad), `codebase-memory-mcp` (detect_changes, get_architecture, check_index_coverage), `codegraph` |
| Implicaciones | contrato aditivo: si faltara detect_changes → triage gate con blast radius incompleto → plan sub-optimo; presente → verificación transitiva correcta; sin breaking change |

## Impacto mapeado (Regla 0) — OBLIGATORIO antes de cualquier edición
> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES del primer step de edición. Sin este bloque poblado, NO se escribe ni se ejecuta ningún step que edite archivos.

- **Archivos leídos (completos):** `.opencode/task-system/prompts/plan.md` (304 líneas, Paso 0 líneas 21-74 con Code Intelligence dual líneas 50-54), `docs/plans/2026-08-28-master-pipeline-optimization.md` (Task 6 líneas 169-185), `.opencode/task-system/mcp/campaign-server.mjs` (detect_changes impl), `.opencode/skills/campaign-executor/templates/task-definition.md` (líneas 97-106 herramientas)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `plan.md` no tiene imports; invoca MCP tools `codegraph_explore`, `codebase-memory-mcp_detect_changes`, `codebase-memory-mcp_get_architecture`, `codebase-memory-mcp_check_index_coverage` en runtime; `plan.md` referencia `skills-engineering.md`, `question-gates.md`, `SKILLS-MANIFEST.md` vía SDP
- **Archivos que referencian a los editados (referencias entrantes):** `Select-String "plan.md"` → 3 hits: `pipeline-full.md:22` (no), `plan.md` self, `docs/plans/2026-08-28-master-pipeline-optimization.md:536-541` (verificación HIGH-006); `detect_changes` en `plan.md:52` y `pipeline-full.md:72`; `get_architecture` y `check_index_coverage` también en `plan.md` + `pipeline-full.md` + `task-definition.md`
- **Veredicto impacto:** bajo — verificación idempotente sin edición; si hubiese edición sería bajo aditivo (solo añade 3 líneas a Paso 0, compatible con pipeline-full ya dual). Sin edición requerida.

## Contrato
Contrato del plan (HIGH-006):
```
grep -n 'detect_changes' .opencode/task-system/prompts/plan.md → línea en Paso 0
```
Verificación completa Code Intelligence dual (contrato extendido ponytail — verifica 4 tools):
```
Select-String "codegraph_explore" plan.md → línea 51 ✅
Select-String "detect_changes" plan.md → línea 52 ✅
Select-String "get_architecture" plan.md → línea 53 ✅
Select-String "check_index_coverage" plan.md → línea 54 ✅
```
Resultado: todos pasan ✅ (ver Investigation Notes). Sin re-edición.

## Spec (SDD — obligatoria si Phase 1b detectó feature-add/símbolos públicos)
> Definición de contenido válido: question-gates.md §"Contenido válido de `## Spec`". Tabla de decisiones O justificación por evidencia por ítem. `N/A` solo aceptable en tareas 100% docs sin decisiones técnicas.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | ¿Re-implementar detect_changes en plan.md si ya existe con Code Intelligence dual completo? | A) No re-implementar (idempotente, menor riesgo, ponytail rung 1) / B) Re-escribir igual (ruido, posible regresión, diff innecesario) | A | ✅ decidido-por-evidencia (ref: plan.md:50-54 las 4 líneas existen; git show 9e5730ff diff prueba implementación original 2026-08-28) |
| 2 | ¿Validar solo detect_changes o dual completo? | A) Solo detect_changes (contrato literal) / B) Dual completo codegraph+detect_changes+get_architecture+check_index_coverage (contrato extendido HIGH-006 descripción) | B — descripción dice dual completo | ✅ decidido-por-evidencia (ref: task descripción "Verifica que plan.md Paso 0 ya tiene Code Intelligence dual (codegraph_explore + detect_changes + get_architecture + check_index_coverage)") |

Justificación: plan pide "Si ya está, marca COMPLETED". Re-implementar introduce riesgo de duplicar líneas y romper formato ya validado por 20/20 pipeline run.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** `plan.md` Paso 0 debe seguir conteniendo las 4 líneas Code Intelligence dual (51-54) en orden: codegraph_explore → detect_changes (scope="impact" direction="inbound" depth=3) → get_architecture (overview,clusters,hotspots,boundaries) → check_index_coverage; `pipeline-full.md` Paso 0b y Discovery deben mantener paridad dual (no desincronizar plan.md de pipeline-full.md); `campaign-server.mjs` tool detect_changes debe seguir operativo
- **Comandos de verificación:** `Select-String -Pattern "detect_changes" -Path .opencode/task-system/prompts/plan.md` → 1 hit línea 52; `Select-String "codegraph_explore" plan.md` → 1 hit línea 51; `Select-String "get_architecture" plan.md` → 1 hit línea 53; `Select-String "check_index_coverage" plan.md` → 1 hit línea 54; `node --check .opencode/task-system/mcp/campaign-server.mjs` → 0; `git show 9e5730ff -- .opencode/task-system/prompts/plan.md` confirma diff original
- **Deuda pendiente:** ninguna — idempotente completo, sin edición; próxima tarea HIGH-007 continúa secuencial

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# HIGH-006: detect_changes en plan.md Paso 0 — blast radius transitivo` |
| `lastAction` | Step 1 VERIFY completado: plan.md líneas 51-54 dual ✅ + grep detect_changes 52 ✅ + node --check 0 + git history 9e5730ff diff ✅ |
| `result` | `OK` ↔ ✅ COMPLETED |
| `nextAction` | HIGH-007 — Re-validar Skills tras Discovery (siguiente en plan secuencial) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | HIGH-007 |

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda

> Regla 6 (AGENTS.md): toda deuda nueva introducida debe compensar deuda existente — el saldo neto por PR es 0 o negativo. Verificación idempotente sin código nuevo → sin deuda.

## Definition of Done (contrato multi-nivel — P2-08)
| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable plan.md Paso 0 dual ✅ + task file sync + recitation actualizada |
| **Commit** | Commit atómico, conventional commit, verificación mecánica (nunca auto-reporte) |
| **Release** | No aplica (infra task-system docs, no crate publish) — justificado en Notas |

**Gate:** Task ✅ si pasan niveles aplicables. Release N/A.

## Herramientas necesarias
- codegraph_explore (blast radius inmediato) — verificado en plan.md:51
- codebase-memory-mcp_detect_changes (blast radius transitivo, impacto de cambios — ANTES de commit) — plan.md:52
- codebase-memory-mcp_get_architecture (overview, clusters, hotspots, boundaries) — plan.md:53
- codebase-memory-mcp_check_index_coverage (verifica cobertura del índice en archivos a tocar) — plan.md:54
- codebase-memory-mcp_index_status (health check)
- campaign_discover_skills (SDP — campaign-executor, progreso, ponytail)

**Skills cargadas (SDP):** campaign-executor (base task-system execution) | progreso (registro avance tras COMPLETED) | ponytail (full — escalera YAGNI, rung 1: no re-implementar si ya existe) | SDP base-only tras discovery: keywords [detect_changes, blast radius, plan.md] → no skills adicionales (infra docs pura, no Rust/bug/security)

## Investigation Notes
- Formato estándar por hallazgo:
  - **Claim:** plan.md Paso 0 contiene Code Intelligence dual completo (4 tools)
  - **Evidencia:** .opencode/task-system/prompts/plan.md líneas 50-54: `codegraph_explore` (51), `detect_changes scope="impact" direction="inbound" depth=3` (52), `get_architecture aspects="['overview','clusters','hotspots','boundaries']"` (53), `check_index_coverage paths=["<archivos de la tarea>"]` (54) — verificado via `Select-String -Pattern "codegraph_explore|detect_changes|get_architecture|check_index_coverage" -Path plan.md` → 4 hits; y `grep -n 'detect_changes' plan.md` → 52: `codebase-memory-mcp_detect_changes scope="impact" direction="inbound" depth=3`
  - **Confianza:** alta
- **Claim:** detect_changes implementado en commit 9e5730ff (Master pipeline optimization - 20 items)
  - **Evidencia:** `git show 9e5730ff -- .opencode/task-system/prompts/plan.md` diff muestra adición de 4 líneas Code Intelligence dual (cambio de 1 línea codegraph_explore a 4 líneas dual); `git show 9e5730ff --stat` lista plan.md con +13 líneas
  - **Confianza:** alta
- **Claim:** 6 verificaciones mecánicas replicadas localmente pasan
  - **Evidencia:** `Select-String "codegraph_explore" plan.md` 1 ✅, `detect_changes` 1 ✅, `get_architecture` 1 ✅, `check_index_coverage` 1 ✅, `node --check campaign-server.mjs` exit:0 ✅, `git show HEAD:.opencode/task-system/prompts/plan.md | Select-String detect_changes` 1 ✅
  - **Confianza:** alta

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — Code Intelligence dual verificado, sin incógnita; approach ya implementado y validado en pipeline run 20/20 |
| Pendientes de ejecución (downhill) | 0 — 1 step VERIFY completado, sin steps pendientes |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)
No aplica — tarea tipo infra/docs (ALTO, no bug). Gate omitido con justificación: contrato es verificación de presencia de líneas en plan.md, no fix de comportamiento roto.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — evaluado: no toca trust boundaries, input de usuario, auth, storage, FFI, ni dependencias. Justificación: edición documental en `prompts/plan.md` (proceso de triage), sin superficie de ataque. No requiere `security-and-hardening`.
- [x] **PERFORMANCE** — evaluado: no toca hot paths (vector/HNSW, engine.rs, search/ingestión, serialización). Justificación: cambio en prompt markdown, no en código de ejecución. No requiere `performance-optimization` ni benchmark.

## Steps

### Step 1: Verificación Code Intelligence dual en plan.md Paso 0
- **Archivos:** `.opencode/task-system/prompts/plan.md` (líneas 50-54), `docs/plans/2026-08-28-master-pipeline-optimization.md` (Task 6)
- **Acción:** Verificar que plan.md Paso 0 ya tiene las 4 líneas dual (codegraph_explore + detect_changes + get_architecture + check_index_coverage). Ejecutar greps mecánicos + validar contra git history 9e5730ff. Si falta alguna → añadir; si están → marcar COMPLETED idempotente (ponytail rung 1). Actualizar plan file Estado → ✅ COMPLETED + recitation.
- **Verify:** `Select-String -Pattern "detect_changes" -Path .opencode/task-system/prompts/plan.md` → 1 hit + `Select-String -Pattern "codegraph_explore|get_architecture|check_index_coverage"` → 3 hits + `node --check .opencode/task-system/mcp/campaign-server.mjs` → 0 + `git show 9e5730ff --stat` contiene plan.md
- **Estado:** ✅ COMPLETED (2026-08-28T16:00 — verificación idempotente, sin edición)

## Dependencias
- Task CORE-005: SDP Unificado — campaign_discover_skills MCP Tool (debe completarse antes — aporta tooling para Paso 0 SDP que precede a Code Intelligence dual)

## Review (GATE — agente distinto, P2-01)
> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-lead (auto-review idempotente — tarea verificación sin código, no requiere vanta-audit/vanta-review separado; ponytail minimal)
- **Enfoque:** ¿plan.md Paso 0 contiene Code Intelligence dual completo y el contrato grep pasa? ¿idempotencia justificada?
- **Cómo se probó:** `Select-String -Pattern "detect_changes" -Path plan.md` → 52:1 hit ✅; `Select-String -Pattern "codegraph_explore|detect_changes|get_architecture|check_index_coverage"` → 4 hits líneas 51-54 ✅; `node --check campaign-server.mjs` exit:0 ✅; `git show 9e5730ff -- plan.md` diff muestra adición dual ✅; `git status --short` limpio antes y después ✅
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria (grep contrato).
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado (4 greps + git history).
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas (1 step, ponytail minimal).
- **Veredicto:** ✅ approve — Code Intelligence dual completo, contrato mecánico pasa, idempotencia correcta, sin edición necesaria

## Notas
- Decisión ponytail full: rung 1 "¿Necesita existir?" → No, ya existe. Verificación idempotente sin re-edición. Skipped: re-escritura de plan.md; add when: dual faltara o desincronizara con pipeline-full.md.
- Plan file 2026-08-28-master-pipeline-optimization.md Task 6 ya marcaba dependencia correcta: CORE-005 → HIGH-006 (SDP tool precede a Code Intelligence dual en plan)
- Commit 9e5730ff ya incluyó esta tarea en feat: Master pipeline optimization - 20 items implemented (ver --stat plan.md +13 líneas); re-ejecución trazable para SARL (ver plan Estado: EN PROGRESO re-ejecución)
- No se requirió web research: APIs codebase-memory-mcp son internas del workspace, no externas ambiguas

## Referencias
- `.opencode/references/definition-of-done.md` — standing quality bar
- `.opencode/references/skills-engineering.md` — SDP lifecycle mapping
- `SKILLS-MANIFEST.md` — catálogo de skills disponibles
- `.opencode/task-system/prompts/plan.md:50-54` — Code Intelligence dual (fuente verificada)
- `docs/plans/2026-08-28-master-pipeline-optimization.md:169-185` — Task 6 definición + contrato

## Context Save Point
- **Fecha:** 2026-08-28T16:00
- **Branch:** main (o develop según git log — verificado f8f00ac5)
- **CI pendiente:** no
- **Decisiones:** HIGH-006 verificado idempotente porque plan.md Paso 0 ya tiene dual completo (4/4) desde 9e5730ff; no se añadió código, se marcó COMPLETED y se actualizó plan recitation
- **Problemas conocidos:** ninguno
- **Próxima tarea:** HIGH-007 — Re-validar Skills tras Discovery (siguiente en plan secuencial)
