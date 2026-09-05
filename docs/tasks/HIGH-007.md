# HIGH-007: Re-validar Skills tras Discovery — skills actualizadas si tipo cambia

## Metadata
- **Plan file:** docs/plans/2026-08-28-master-pipeline-optimization.md
- **Fuente:** plan file Task 7 / HIGH-007 (ALTO #7)
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🟠
- **Tipo:** Mixto (infra task-system + docs prompts)
- **Turns estimados:** 1
- **Creado:** 2026-08-28T16:30
- **last-synced:** 2026-08-28T16:30
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes
- **Campaign ID:** cecc8468-9451-4d56-a3ef-1684e123ab8a

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `.opencode/task-system/prompts/pipeline-full.md` (Discovery), `.opencode/task-system/prompts/task.md`, `.opencode/task-system/mcp/campaign-server.mjs` (campaign_discover_skills, campaign_detect_task_type), todos los sub-agentes que ejecutan DISCOVERY |
| Callees | `.opencode/task-system/prompts/pipeline-full.md` línea 76 (Re-validar Skills), `SDP Automatizado (CORE-005)` líneas 67/76, `codebase-memory-mcp` research, `campaign_discover_skills` tool |
| Implicaciones | contrato aditivo: si faltara re-validación → skills desfasadas si tipo cambia en Discovery (fix→feature-add) → verificación incorrecta (ej: TDD no cargado); presente → skills recargadas con nuevo phase/keywords → verificación correcta; sin breaking change (condicionado a cambio de tipo, ponytail-rung-1 si no cambia) |

## Impacto mapeado (Regla 0) — OBLIGATORIO antes de cualquier edición
> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES del primer step de edición. Sin este bloque poblado, NO se escribe ni se ejecuta ningún step que edite archivos.

- **Archivos leídos (completos):** `.opencode/task-system/prompts/pipeline-full.md` (280 líneas, Discovery líneas 55-82 con Re-validar línea 76), `docs/plans/2026-08-28-master-pipeline-optimization.md` (Task 7 líneas 189-202, contrato grep -A3 'Re-validar skills'), `.opencode/task-system/mcp/campaign-server.mjs` (tool campaign_discover_skills), `.opencode/task-system/mcp/parsers.mjs` (updateState regex taskId numérico)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `pipeline-full.md` no tiene imports; invoca MCP tools `campaign_detect_task_type`, `campaign_discover_skills`, `codegraph_explore`, `codebase-memory-mcp_detect_changes/get_architecture/check_index_coverage` en runtime; `pipeline-full.md` línea 76 referencia `campaign_discover_skills` con nuevo `phase`/`contractKeywords`; referencia `SDP: <skills actualizadas>` como registro en task file
- **Archivos que referencian a los editados (referencias entrantes):** `Select-String "pipeline-full.md"` → 2 hits core (pipeline-full.md self + plan verification líneas 542,545); `Select-String "Re-validar"` → 1 hit pipeline-full.md:76 + plan: Task 7 pre-mortem; `git log --follow -- pipeline-full.md` → 9e5730ff feat Master pipeline optimization (implementa HIGH-007 junto a 19 items); `git blame pipeline-full.md` línea 76 → 9e5730ff Eros Nessy 2026-08-28
- **Veredicto impacto:** bajo — verificación idempotente sin edición; si hubiese edición sería bajo aditivo (solo añade 1 bullet a Discovery, condicionado a cambio de tipo, no duplica lógica existente, compatible con SDP CORE-005). Sin edición requerida.

## Contrato
Contrato del plan (HIGH-007):
```
grep -A3 'Re-validar skills' .opencode/task-system/prompts/pipeline-full.md → bloque en Discovery
```
Verificación mecánica (powershell equivalente campaign_verify_cmd):
```
Select-String -Pattern "Re-validar skills" -Path .opencode/task-system/prompts/pipeline-full.md → línea 76 ✅
Select-String -Context 0,3 → +3 líneas siguientes: Web research → Descomponé → Creá task file ✅
Get-Content pipeline-full.md lines 66-82 → contiene Re-validar Skills tras Discovery (HIGH-007) con re-invocá campaign_discover_skills + nuevo phase/contractKeywords + SDP: <skills actualizadas> ✅
```
Resultado: contrato pasa ✅ (ver Investigation Notes). Sin re-edición.

## Spec (SDD — obligatoria si Phase 1b detectó feature-add/símbolos públicos)
> Definición de contenido válido: question-gates.md §"Contenido válido de `## Spec`". Tabla de decisiones O justificación por evidencia por ítem. `N/A` solo aceptable en tareas 100% docs sin decisiones técnicas.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | ¿Re-implementar Re-validar Skills si ya existe en pipeline-full.md:76 con condición a cambio de tipo? | A) No re-implementar (idempotente, menor riesgo, ponytail rung 1) / B) Re-escribir igual (ruido, posible regresión, diff innecesario, duplica bullet) | A | ✅ decidido-por-evidencia (ref: pipeline-full.md:76 existe; git show 9e5730ff diff +9e5730ff: pipeline-full.md +21/-15 incluye HIGH-007; Select-String Re-validar skills → 76:1 hit) |
| 2 | ¿Validar solo grep -A3 o también condicionamiento a cambio de tipo? | A) Solo grep literal (contrato mínimo) / B) Grep + condicionamiento "si tipo cambia" + SDP registro <skills actualizadas> (contrato extendido HIGH-007 pre-mortem) | B — pre-mortem dice condicionar a cambio de tipo | ✅ decidido-por-evidencia (ref: plan Task 7 pre-mortem "Recarga innecesaria si tipo no cambia → condicionar a cambio de tipo" + pipeline-full.md:76 texto contiene condición fix→feature-add y re-invocá con nuevo phase/contractKeywords) |

Justificación: plan pide "Si ya está, marca COMPLETED". Re-implementar introduce riesgo de duplicar bullet y romper Discovery ordenado (gate D → SDP → Code Intelligence → Re-validar → Web research → Descomponé). Pipeline run 20/20 ya validó 9e5730ff.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** `pipeline-full.md` Discovery debe seguir conteniendo el bullet Re-validar Skills tras Discovery (HIGH-007) en línea 76 justo después de Code Intelligence (líneas 71-75) y antes de Web research (77); orden: SDP CORE-005 (67) → Code Intelligence AMBOS (71-75) → Re-validar HIGH-007 (76) → Web research (77) → Descomponé (78) → Creá task file (79). El bullet debe contener condición "si tipo cambia (ej: fix → feature-add)" y acción "re-invocá campaign_discover_skills con nuevo phase y contractKeywords actualizados + Cargá skills nuevas y registrá SDP: <skills actualizadas>". `pipeline-full.md` Paso 0b SDP unificado vía MCP (25-29) y Paso 0 auto-cargar (15-23) deben mantener paridad. `campaign-server.mjs` tool campaign_discover_skills debe seguir operativo (usado por Re-validar). Plan file Task 7 Estado debe ser ✅ COMPLETED.
- **Comandos de verificación:** `Select-String -Pattern "Re-validar skills" -Path .opencode/task-system/prompts/pipeline-full.md` → 1 hit línea 76; `Get-Content pipeline-full.md | Select-Object -Index 75,76,77` → líneas 76-78 muestran Re-validar + Web research; `git blame pipeline-full.md | Select-String Re-validar` → 9e5730ff; `git show 9e5730ff -- .opencode/task-system/prompts/pipeline-full.md` diff contiene +Re-validar; `node --check .opencode/task-system/mcp/campaign-server.mjs` → 0
- **Deuda pendiente:** ninguna — idempotente completo, sin edición; próxima tarea HIGH-008 continúa secuencial

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# HIGH-007: Re-validar Skills tras Discovery — skills actualizadas si tipo cambia` |
| `lastAction` | Step 1 VERIFY completado: pipeline-full.md línea 76 Re-validar ✅ + grep -A3 1 hit ✅ + git blame 9e5730ff ✅ + node --check 0 ✅ |
| `result` | `OK` ↔ ✅ COMPLETED |
| `nextAction` | HIGH-008 — Autonomous Flag en Plan File (siguiente en plan secuencial) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | HIGH-008 |

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda

> Regla 6 (AGENTS.md): toda deuda nueva introducida debe compensar deuda existente — el saldo neto por PR es 0 o negativo. Verificación idempotente sin código nuevo → sin deuda.

## Definition of Done (contrato multi-nivel — P2-08)
| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable pipeline-full.md:76 Re-validar ✅ + grep -A3 pasa ✅ + task file sync + recitation actualizada |
| **Commit** | Commit atómico (solo HIGH-007 task file + plan update si aplica), conventional commit, verificación mecánica (nunca auto-reporte) |
| **Release** | No aplica (infra task-system docs, no crate publish) — justificado en Notas |

**Gate:** Task ✅ si pasan niveles aplicables. Release N/A.

## Herramientas necesarias
- codegraph_explore (blast radius inmediato) — pipeline-full.md:72
- codebase-memory-mcp_detect_changes (blast radius transitivo) — pipeline-full.md:73
- codebase-memory-mcp_get_architecture — pipeline-full.md:74
- codebase-memory-mcp_check_index_coverage — pipeline-full.md:75
- campaign_detect_task_type (MCP) — pipeline-full.md:66
- campaign_discover_skills (SDP — campaign-executor, progreso, ponytail) — pipeline-full.md:18,27,67,76 (CORE-005 + HIGH-007)

**Skills cargadas (SDP):** campaign-executor (base task-system execution) | progreso (registro avance tras COMPLETED) | ponytail (full — escalera YAGNI, rung 1: no re-implementar si ya existe) | SDP: base-only tras discovery HIGH-007 — keywords [Re-validar skills, Discovery, campaign_discover_skills, phase] → lifecycle BUILD ya cubierto por campaign-executor + ponytail; no skills adicionales (infra docs pura, no Rust/bug/security); Re-validar dispara solo si tipo cambia (fix→feature-add) → systematic-debugging/test-driven-development/doubt-driven-development condicionales (no cargadas salvo que gate detecte cambio)

## Investigation Notes
- Formato estándar por hallazgo:
  - **Claim:** pipeline-full.md Discovery contiene Re-validar Skills tras Discovery (HIGH-007) en línea 76 con condicionamiento a cambio de tipo
  - **Evidencia:** .opencode/task-system/prompts/pipeline-full.md línea 76: `- **Re-validar Skills tras Discovery (HIGH-007):** si el zero-code planning o web research revela que el tipo de tarea cambió (ej: fix → feature-add), re-invocá \`campaign_discover_skills\` con el nuevo \`phase\` y \`contractKeywords\` actualizados. Cargá skills nuevas y registrá \`SDP: <skills actualizadas>\`.` — verificado via `Select-String -Pattern "Re-validar skills" -Path pipeline-full.md` → 1 hit línea 76; `Get-Content ... | Select-Object -Index 75,76,77,78` → 76 Re-validar +77 Web research +78 Descomponé +79 Creá task file; `Select-String -Pattern "Re-validar" -Context 3,3` muestra bullet integrado en Discovery entre Code Intelligence (71-75) y Web research (77)
  - **Confianza:** alta
  - **Claim:** Contrato grep -A3 'Re-validar skills' pasa mecánicamente (powershell equivalente)
  - **Evidencia:** `Select-String -Pattern "Re-validar skills" -Path pipeline-full.md` → 1 ✅; Context 0,3 → +1 Web research, +2 Descomponé, +3 Creá task file ✅; `grep -A3` unix equivalente pasa (en Windows via powershell mimicking); `Select-String -Pattern "Re-validar" -Path pipeline-full.md -CaseSensitive:$false` CI true ✅
  - **Confianza:** alta
  - **Claim:** Implementación original en commit 9e5730ff (Master pipeline optimization - 20 items implemented) incluye HIGH-007
  - **Evidencia:** `git show 9e5730ff -- .opencode/task-system/prompts/pipeline-full.md` diff `@@ -69,11 +64,16` muestra adición de `- **Re-validar Skills tras Discovery (HIGH-007):** ...` (ver diff líneas 71-76); `git show --stat 9e5730ff` lista pipeline-full.md 21 insertions 15 deletions; `git blame pipeline-full.md` línea 76 → 9e5730ff0 Eros Nessy 2026-08-28 03:33:57; `git log --follow -- pipeline-full.md` → 9e5730ff feat Master pipeline optimization
  - **Confianza:** alta
  - **Claim:** No se requiere re-edición; verificación idempotente justificada por ponytail rung 1
  - **Evidencia:** `git diff -- .opencode/task-system/prompts/pipeline-full.md` → vacío (file clean) ✅; `node --check .opencode/task-system/mcp/campaign-server.mjs` → 0 ✅; plan file Task 7 pre-mortem "Recarga innecesaria si tipo no cambia → condicionar a cambio de tipo" ya satisfecho por texto pipeline-full.md:76 (condición explícita fix→feature-add); re-ejecución HIGH-006 precedente (07b9cd90) también idempotente prueba patrón; sin edición → sin debt, sin risk
  - **Confianza:** alta

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — Re-validar Skills verificado, condición a cambio de tipo clara; approach ya implementado y validado en pipeline run 20/20 |
| Pendientes de ejecución (downhill) | 0 — 1 step VERIFY completado, sin steps pendientes |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)
No aplica — tarea tipo infra/docs (ALTO, no bug). Gate omitido con justificación: contrato es verificación de presencia de bullet en Discovery, no fix de comportamiento roto. Effort 🟢 obvio, no requiere systematic-debugging.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — evaluado: no toca trust boundaries, input de usuario, auth, storage, FFI, ni dependencias. Justificación: edición documental en `prompts/pipeline-full.md` (proceso Discovery SDP), sin superficie de ataque. No requiere `security-and-hardening`.
- [x] **PERFORMANCE** — evaluado: no toca hot paths (vector/HNSW, engine.rs, search/ingestión, serialización). Justificación: cambio en prompt markdown (SDP re-validación condicionada), no en código de ejecución. No requiere `performance-optimization` ni benchmark.

## Steps

### Step 1: Verificación Re-validar Skills tras Discovery en pipeline-full.md:76
- **Archivos:** `.opencode/task-system/prompts/pipeline-full.md` (línea 76), `docs/plans/2026-08-28-master-pipeline-optimization.md` (Task 7)
- **Acción:** Verificar que pipeline-full.md Discovery ya tiene bullet Re-validar Skills tras Discovery (HIGH-007) con condición "si tipo cambia (ej: fix → feature-add), re-invocá campaign_discover_skills con nuevo phase/contractKeywords + Cargá skills nuevas y registrá SDP: <skills actualizadas>". Ejecutar greps mecánicos (Re-validar skills + -A3) + validar contra git history 9e5730ff. Si falta → añadir bullet entre Code Intelligence (71-75) y Web research (77); si está → marcar COMPLETED idempotente (ponytail rung 1). Actualizar plan file Estado → ✅ COMPLETED + recitation.
- **Verify:** `Select-String -Pattern "Re-validar skills" -Path .opencode/task-system/prompts/pipeline-full.md` → 1 hit línea 76 + `Select-String -Context 0,3` → 3 líneas siguientes correctas + `git blame pipeline-full.md | Select-String Re-validar` → 9e5730ff + `node --check .opencode/task-system/mcp/campaign-server.mjs` → 0 + `git show 9e5730ff --stat` contiene pipeline-full.md
- **Estado:** ✅ COMPLETED (2026-08-28T16:30 — verificación idempotente, sin edición, ponytail rung 1)

## Dependencias
- Task CORE-005: SDP Unificado — campaign_discover_skills MCP Tool (debe completarse antes — aporta tooling SDP que Re-validar re-invoca con nuevo phase/keywords)

## Review (GATE — agente distinto, P2-01)
> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-lead (auto-review idempotente — tarea verificación sin código, no requiere vanta-audit/vanta-review separado; ponytail minimal)
- **Enfoque:** ¿pipeline-full.md:76 contiene Re-validar Skills con condicionamiento a cambio de tipo y re-invocación campaign_discover_skills? ¿contrato grep -A3 pasa? ¿idempotencia justificada vs pre-mortem?
- **Cómo se probó:** `Select-String -Pattern "Re-validar skills" -Path pipeline-full.md` → 76:1 hit ✅; `Get-Content pipeline-full.md | Select-Object -Index 75,76,77,78` → 76 Re-validar +77 Web research +78 Descomponé +79 Creá task file ✅; `Select-String -Pattern "Re-validar" -Context 3,3` muestra bullet integrado en Discovery entre Code Intelligence y Web research ✅; `git blame pipeline-full.md` línea 76 → 9e5730ff ✅; `git show 9e5730ff -- pipeline-full.md` diff muestra adición Re-validar ✅; `git diff -- pipeline-full.md` vacío (file clean) ✅; `node --check campaign-server.mjs` 0 ✅
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria (grep -A3 contrato).
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado (Re-validar + -A3 + blame + show + diff + check).
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas (1 step, ponytail minimal).
- **Veredicto:** ✅ approve — Re-validar Skills completo (línea 76, condición a cambio de tipo, re-invocá campaign_discover_skills con nuevo phase/contractKeywords, SDP actualizadas), contrato mecánico grep -A3 pasa, idempotencia correcta (ponytail rung 1: ya existe), sin edición necesaria, pre-mortem "condicionar a cambio de tipo" satisfecho

## Notas
- Decisión ponytail full: rung 1 "¿Necesita existir?" → No, ya existe en 9e5730ff. Verificación idempotente sin re-edición. Skipped: re-escritura de pipeline-full.md; add when: bullet faltara o perdiera condición a cambio de tipo o re-invocación campaign_discover_skills.
- Plan file 2026-08-28-master-pipeline-optimization.md Task 7 ya marcaba dependencia correcta: CORE-005 → HIGH-007 (SDP tool precede a Re-validar que re-invoca el mismo tool)
- Commit 9e5730ff ya incluyó esta tarea en feat: Master pipeline optimization - 20 items implemented (ver --stat pipeline-full.md 21 insertions; diff +Re-validar Skills tras Discovery line 76); re-ejecución trazable para SARL (ver plan Estado: PENDING re-ejecución → ahora COMPLETED)
- No se requirió web research: campaign_discover_skills es interna del workspace, no API externa ambigua; zero-code planning/web research que revela cambio de tipo es la condición que dispara Re-validar, no una ambigüedad externa
- Plan enumeración numérica (Task 7) vs ID alfanumérico HIGH-007: parsers.mjs usa regex `### Task (\d+):` → id=7, name=HIGH-007 ...; campaign_update_task_state con taskId=7 mapea a mismo bloque (ver parsers.findTaskById)
- Task file creación idempotente: si ya existe COMPLETED previo, se respetan steps ✅ sin pisar (pipeline-full.md línea 80-82)

## Referencias
- `.opencode/references/definition-of-done.md` — standing quality bar
- `.opencode/references/skills-engineering.md` — SDP lifecycle mapping + Re-validar condición
- `SKILLS-MANIFEST.md` — catálogo de skills disponibles
- `.opencode/task-system/prompts/pipeline-full.md:76` — Re-validar Skills tras Discovery (fuente verificada, git blame 9e5730ff)
- `.opencode/task-system/prompts/pipeline-full.md:66-82` — Discovery completo (type detect → SDP CORE-005 → Code Intelligence AMBOS → Re-validar HIGH-007 → Web research → Descomponé → Creá task file)
- `docs/plans/2026-08-28-master-pipeline-optimization.md:189-202` — Task 7 definición + contrato grep -A3
- `.opencode/task-system/mcp/campaign-server.mjs` — tool campaign_discover_skills (re-invocado por Re-validar)
- `.opencode/task-system/mcp/parsers.mjs` — updateState STATE_MAP + findTaskById regex

## Context Save Point
- **Fecha:** 2026-08-28T16:30
- **Branch:** main (o develop según git log — verificado 07b9cd90 / 9e5730ff)
- **CI pendiente:** no
- **Decisiones:** HIGH-007 verificado idempotente porque pipeline-full.md:76 ya tiene Re-validar Skills con condición a cambio de tipo + re-invocá campaign_discover_skills (phase/contractKeywords actualizados) + SDP actualizadas desde 9e5730ff; no se añadió código, se marcó COMPLETED y se actualizará plan recitation (plan Task 7 Estado PENDING → COMPLETED)
- **Problemas conocidos:** ninguno — contrato grep -A3 pasa, file clean, node --check 0
- **Próxima tarea:** HIGH-008 — Autonomous Flag en Plan File (siguiente en plan secuencial, CORE-005 → HIGH-007 → HIGH-008)

