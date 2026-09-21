# C0-UNIFY-06: Unificar C0 en forma v2 absorbente (C0-unified.mjs + .md)

## Metadata
- **Plan file:** docs/plans/2026-09-04-durability-release-readiness.md (virtual — tarea fuera de plan, ejecución directa por contrato usuario)
- **Fuente:** Contrato usuario 2026-09-05 (D16A-D: forma C0 v2 absorbente, contenido todo, pareja nueva v2, limpieza 4 sitios)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Tipo:** Mixto (task-system JS/MD + MCP)
- **Turns estimados:** 15-30
- **Creado:** 2026-09-05T12:00
- **last-synced:** 2026-09-05T12:00
- **Estado:** ✅ COMPLETED (sin commit por instrucción usuario)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `.opencode/task-system/mcp/campaign-server.mjs` (import STATE_TOOLS, getAllowedTools, validateAction, BUDGET_LIMITS, classifyWorkflow/loadWorkflow); `.opencode/task-system/config/parity-check.mjs`; prompts `iter-loop-tools.md`, `pipeline-full.md`; `skills/campaign-executor/SKILL.md`; agents `vanta-lead.md`, `vanta-research.md` |
| Callees | `config/state-tools.mjs` (STATE_TOOLS 10 estados); `workflows/*.json` (5 templates: bug-fix/feature-add/refactor/research/nine-second-saloon); `prompts/question-gates.md` (Gates D/V/C); `mcp/campaign-server.mjs` BUDGET_LIMITS + C0_CHECK_CONFIG |
| Implicaciones | Contrato MCP no cambia en modo legacy (aditivo con flag); enforce_state idéntico por defecto; workflows JSON siguen legibles; parity 10/10 debe seguir pasando + extenderse a v2; sin migración de datos; tests `hardening.test.mjs` no tocados |

## Impacto mapeado (Regla 0)

> GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0)

- **Archivos leídos (completos):** `.opencode/task-system/config/state-tools.mjs` (102L: STATE_TOOLS 10 estados PLAN/ACT/VERIFY/COLLATERAL/RESEARCH/EVALUATE/REVIEW/ACCEPT/CLOSE/STALL + getAllowedTools/validateAction/matchPattern); `.opencode/task-system/prompts/iter-loop-tools.md` (399L: Step0 SDP, MODO DISCOVERY/EJECUCIÓN/CIERRE, state machine C0 líneas 117-138 + tabla 144-155 duplicada); `.opencode/task-system/prompts/pipeline-full.md` (280L: Paso 0-0b, flujo PENDING/IN PROGRESS/FAILED, §3 recitation, §7 RESULTADO); `.opencode/task-system/mcp/campaign-server.mjs` (~2400L: BUDGET_LIMITS 216-222, STATE_TOOLS import:11, enforce_state 1980-2047, classifyWorkflow 1681-1695, loadWorkflow 1692, get_workflow 1839-1852, classify_workflow 1738-1761, C0_CHECK_CONFIG 1867-1879); `.opencode/task-system/config/parity-check.mjs` (39L: canónico state-tools.mjs vs iter-loop-tools.md + SKILL.md); `workflows/bug-fix.json`, `feature-add.json`, `refactor.json`, `research.json`, `nine-second-saloon.json` (phase templates con instructions/max_iterations/on + allowed_tools por fase, NO enforcement); `.opencode/skills/campaign-executor/SKILL.md` (429L: diagrama C0 78-95, budget tabla 242-249); `.opencode/task-system/prompts/question-gates.md` (142L: Gates P/D/V/C + registro GATES_EVALUADOS)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `campaign-server.mjs` → `../config/state-tools.mjs` (STATE_TOOLS/getAllowedTools/validateAction); `parity-check.mjs` → `../config/state-tools.mjs`; `iter-loop-tools.md` → referencia `config/state-tools.mjs` como fuente canónica (línea 157-159); `pipeline-full.md` → referencia `campaign-server.mjs` BUDGET_LIMITS + `campaign_get_workflow`; `SKILL.md` → referencia `iter-loop-tools.md` canonical + BUDGET_LIMITS server
- **Archivos que referencian a los editados (referencias entrantes):** `rg state-tools|STATE_TOOLS` → VANTADB-OPERATING-MANUAL.md:21, AGENTS.md? no, `vanta-lead.md`, `vanta-research.md:310`, `RULES.md:253`, `REFERENCE-SYNTHESIS.md` (histórico), `task-system/memory/lessons.md`; `rg campaign_get_workflow|campaign_classify_workflow` → `vanta-lead.md:443`, `iter-loop-tools.md:19/52`, `pipeline-full.md:21`; `rg BUDGET_LIMITS` → `SKILL.md:240-249`, `iter-loop-tools.md:231/397`, `pipeline-full.md:122/164`
- **Veredicto impacto:** medio — cambio aditivo con flag preserva runtime legacy; riesgo principal es divergencia de fuentes (3 definiciones C0) que esta tarea justamente elimina; si C0-unified.mjs diverge en un allowed/denied → enforce_state cambiaría veredicto → mitigado con re-export exacto + tests parity + node --check

## Contrato
"existe pareja nueva `.opencode/task-system/C0-unified.mjs` + `C0-unified.md` v2 absorbente con todo (allowed/denied + transiciones/guardas + instrucciones por tipo + BUDGET_LIMITS + DoD + Gates D/V/C); `state-tools.mjs` y `workflows/*.json` quedan legacy deprecated con pointer; tabla `iter-loop-tools.md:144-155` purgada a pointer; `pipeline-full.md:21-23` fix; `campaign_get_workflow`/`classify_workflow` devuelven perfil unificado; verificado por `node --check` + parity 10/10 + `rg` + `node --test` (sin romper enforce_state legacy)"

## Spec (SDD — Phase 1b detectó símbolos públicos nuevos → feature-add)

> Nueva superficie pública: módulo `C0-unified.mjs` (exports), campos nuevos en respuestas MCP (`unified`, `deprecated`, `canonical`), flag opt-in. Tabla requerida.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Ubicación pareja v2 | A: `.opencode/task-system/C0-unified.mjs` raíz (visible, 1 nivel, tradeoff: nuevo top-level) / B: `config/C0-unified.mjs` (agrupado, tradeoff: se confunde con legacy) | A raíz task-system (contrato dice `.opencode/task-system/C0-unified.mjs`) | ✅ decidido-por-evidencia (ref: contrato usuario: "pareja nueva .opencode/task-system/C0-unified.mjs + C0-unified.md") |
| 2 | Estrategia legacy | A: re-export exacto (cero divergencia, tradeoff: import extra) / B: copia (independiente, tradeoff: deriva futura) | A re-export (Regla 0: NO romper enforce_state) | ✅ decidido-por-evidencia (ref: `config/state-tools.mjs:73-92` getAllowedTools/validateAction deben quedar idénticos) |
| 3 | Flag v2 aditivo | A: `unified:true` param por-tool + `C0_V2=1` env global (opt-in explícito, tradeoff: 2 mecanismos) / B: solo env (tradeoff: tests no pueden opt-in por llamada) | A ambos (param > env > legacy) | ✅ decidido-por-evidencia (ref: contrato "v2 aditivo con flag", `campaign-server.mjs:1839-1761` tools aceptan params extra sin romper schema) |
| 4 | Workflows JSON legacy | A: añadir `_deprecated` + `_canonical` pointer (parseable, tradeoff: campo extra ignorado por loader) / B: borrar archivos (tradeoff: rompe loadWorkflow + links) | A campo pointer (no romper loader) | ✅ decidido-por-evidencia (ref: `campaign-server.mjs:1692-1695` loadWorkflow lee JSON directo; borrar rompería `availableTemplates`) |
| 5 | Parity extendida | A: parity-check lee C0-unified.mjs como canónico + legacy como espejo (tradeoff: 2 imports) / B: solo legacy (tradeoff: v2 puede divergir) | A ambos con comparativa exacta | ✅ decidido-por-evidencia (ref: `config/parity-check.mjs:16-21` ya importa STATE_TOOLS; extender a unified) |

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** enforce_state legacy idéntico por defecto (mismos allowed/denied por estado); loadWorkflow/classifyWorkflow existentes responden igual salvo campos aditivos; parity 10/10 (PLAN/ACT/VERIFY/COLLATERAL/RESEARCH/EVALUATE/REVIEW/ACCEPT/CLOSE/STALL) en todas las fuentes; BUDGET_LIMITS números intactos (10/15/40/5/120)
- **Comandos de verificación:** `node --check .opencode/task-system/C0-unified.mjs` + `node .opencode/task-system/config/parity-check.mjs` + `node --test .opencode/task-system/mcp/*.test.mjs`
- **Deuda pendiente:** ninguna (NO commit por instrucción usuario; push/commit lo hace orquestador)

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado C0-UNIFY-06 |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | PARTIAL (en ejecución) |
| `nextAction` | Próximo step ⬜ PENDING |
| `contract` | ## Contrato + ## Invariantes + evidencia |
| `nextTask` | STABLE-04 (plan activo más reciente) |

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda — la tarea ELIMINA deuda (3 fuentes C0 divergentes → 1 canónica v2 + legacy como espejo). No se introduce `unsafe`, `clone` hot-path ni dual-API permanente (flag es transición documentada con sunset en C0-unified.md).

## Definition of Done (contrato multi-nivel — P2-08)
| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable arriba se cumple + `node --check` OK + parity 10/10 + `rg` sin refs rotas + `node --test` sin regresión |
| **Commit** | NO APLICA por instrucción explícita usuario (NO commitees) — diff queda en worktree para orquestador |
| **Release** | NO APLICA (cambio task-system interno, sin semver Rust) |

## Herramientas necesarias
- node --check, node --test (verify)
- rg/grep (blast radius + verificación)
- codegraph_explore (no aplica a .mjs task-system — análisis manual Read ya hecho)

**Skills cargadas (SDP):** api-and-interface-design (boundaries Contract First, Hyrum, One-Version — justificación: nuevo módulo público + error/format estables) + documentation-and-adrs (justificación: spec C0-unified.md + ADRs de unificación) + base campaign-executor/progreso/ponytail. `campaign_discover_skills_v2` FALLÓ con `Invalid regular expression: nothing to repeat` (bug grepSkillsManifest sin escape — keywords con `.`/`/` del archivosClave) → SDP base-only + manual (registrado per pipeline-full Paso 0b d).

## Investigation Notes
- `campaign_discover_skills_v2` bug reproducible: construye `new RegExp(...${kw}...)` sin escapar kw; cualquier keyword con `.`/`+`/`*`/`?` o vacía rompe. No bloquea tarea (fallback manual). Candidato a FIND-* si orquestador quiere (no se crea acá por scope).
- Gate D evaluado: blast radius 9 archivos clave (<10 límite pero toca API pública MCP + state machine core) + agrega símbolos públicos (C0-unified.mjs exports + campos MCP). Contrato usuario ya aprueba superficie ("pareja nueva v2", "v2 aditivo con flag") → se considera GO implícito, no se pregunta (orquestador ya decidió). Registrado en GATES_EVALUADOS.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 5 |
| % completado | 100% (5/5 steps + verify full) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — No toca trust boundaries (sin input usuario, sin auth, sin deps nuevas, sin storage, sin FFI, sin red). `campaign-server.mjs` cambios son campos aditivos + imports; sin `eval`/`exec` nuevos; `validateOutput`/`checkBlockedEnv` intactos. Justificación: diff solo añade datos, no relaja validación. Skill `security-and-hardening` NO requerida.
- [x] **PERFORMANCE** — No toca hot paths (no vector/engine/search/serialización). C0-unified.mjs es lookup estático O(1) + re-export; overhead despreciable (1 import). Sin bench requerido.

## Steps

### Step 1: Crear C0-unified.mjs v2 absorbente (runtime)
- **Archivos:** `.opencode/task-system/C0-unified.mjs` (nuevo, ~350L)
- **Acción:** Re-export exacto STATE_TOOLS/getAllowedTools/validateAction desde config/state-tools.mjs (cero divergencia) + C0_TRANSITIONS/C0_INVALID/C0_CHECK_CONFIG espejo + TYPE_PROFILES (5 workflows: instrucciones iniciales localizing/spec/audit/scoping/diagnose + fases completas importadas de JSONs) + BUDGET_LIMITS espejo + DoD + Gates D/V/C resumen + helpers getUnifiedProfile()/isUnifiedEnabled()/resolveWorkflowProfile(). Header v2 canónico + nota legacy.
- **Verify:** `node --check .opencode/task-system/C0-unified.mjs`
- **Estado:** ✅ COMPLETED

### Step 2: Crear C0-unified.md v2 (prose spec)
- **Archivos:** `.opencode/task-system/C0-unified.md` (nuevo, ~250L)
- **Acción:** Spec prose absorbente: tabla allowed/denied (generada de STATE_TOOLS), diagrama transiciones + inválidas, instrucciones por tipo (5 perfiles), BUDGET_LIMITS tabla, DoD 3 niveles, Gates D/V/C, guía migración legacy→v2 + sunset, flag `unified:true` / `C0_V2=1`.
- **Verify:** `rg -n "PLAN|ACT|VERIFY|COLLATERAL|RESEARCH|EVALUATE|REVIEW|ACCEPT|CLOSE|STALL" .opencode/task-system/C0-unified.md` (10/10 presentes)
- **Estado:** ✅ COMPLETED

### Step 3: Limpieza 4 sitios (legacy pointers + fixes)
- **Archivos:** `.opencode/task-system/config/state-tools.mjs` (header deprecated pointer, sin tocar runtime); `.opencode/task-system/workflows/*.json` (5 files: campo `_deprecated` + `_canonical` pointer); `.opencode/task-system/prompts/iter-loop-tools.md` (tabla 144-155 purgada a pointer + nota canónica); `.opencode/task-system/prompts/pipeline-full.md` (líneas 21-23 fix: workflow NO define allowed_tools)
- **Acción:** Edits mínimos aditivos; tabla iter-loop reemplazada por pointer que lista estados inline (preserva parity grep) + link a C0-unified.md; pipeline-full reword a "phase template como guía, enforcement siempre C0".
- **Verify:** `rg -n "C0-unified" .opencode/task-system/config/state-tools.mjs .opencode/task-system/prompts/iter-loop-tools.md .opencode/task-system/prompts/pipeline-full.md`
- **Estado:** ✅ COMPLETED

### Step 4: campaign-server aditivo + parity extendida
- **Archivos:** `.opencode/task-system/mcp/campaign-server.mjs` (import unified, helpers, tools get_workflow/classify_workflow con campos aditivos + flag); `.opencode/task-system/config/parity-check.mjs` (chequea C0-unified.mjs como canónico + legacy espejo + C0-unified.md + SKILL.md)
- **Acción:** Importar C0-unified.mjs (try/catch fallback a legacy si falta); `campaign_get_workflow(name, {unified})` devuelve `{...legacy, unified: <perfil>, canonical: C0-unified, deprecated: legacy-pointer}`; `campaign_classify_workflow` igual + `profile`; parity compara STATE_TOOLS legacy vs UNIFIED_TOOLS exacto + 10 estados en .md/SKILL.
- **Verify:** `node --check .opencode/task-system/mcp/campaign-server.mjs && node --check .opencode/task-system/config/parity-check.mjs`
- **Estado:** ✅ COMPLETED

### Step 5: Verify full contrato (parity 10/10 + rg + tests)
- **Archivos:** (ninguno — solo comandos)
- **Acción:** `node .opencode/task-system/config/parity-check.mjs` (10/10 OK) + `rg` refs + `node --test` task-system + `node --check` todos los .mjs tocados. Sin commit (instrucción usuario).
- **Verify:** `node .opencode/task-system/config/parity-check.mjs && node --test .opencode/task-system/mcp/`
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (tarea standalone task-system; no bloquea plan durability-release-readiness)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** self adversarial + parity mecánica (sin sub-agente distinto disponible; deuda de proceso declarada — orquestador puede pedir P2-01 a vanta-audit/vanta-review)
- **Enfoque:** ¿re-export exacto sin drift? ¿flag aditivo sin romper legacy? ¿4 sitios sin refs rotas? Alternativa descartada: copiar listas en unified (drift futuro) → re-export por construcción.
- **Cómo se probó:** node --check ×4 exit 0; parity 10/10 + 5 perfiles + budget exit 0; node --test mcp/ 42/42; enforce parity 10/10 (validateAction legacy≡unified); flag off→legacy exacto, on→perfil full; C0_V2=1 env OK; JSONs 5/5 válidos; rg refs en 4 sitios + server.
- **Hallazgos auto-review (3):**
  1. workflows/*.json legacy path devuelve 2 keys extra (_deprecated/_canonical) aun sin flag — aceptado (pointer exigido por contrato; consumidores leen .definition).
  2. campaign-server.mjs top-level await import — verificado bajo node; bun soporta TLA; fallback null preserva legacy.
  3. Tabla vieja decía ACT denied "(ninguna)" vs state-tools.mjs ["delete"] — divergencia real eliminada por la purga; enforce_state nunca leyó la tabla.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ✅ approve (con deuda de proceso: review por agente distinto pendiente si el orquestador lo exige)

## Notas
- Regla 0 cumplida arriba antes de Step 1 (ninguna edición hecha aún).
- NO commit por instrucción explícita usuario ("Devolvé RESULTADO (NO commitees)") — DoD Commit marcado NO APLICA.

## Context Save Point
- **Fecha:** 2026-09-05
- **Branch:** develop
- **CI pendiente:** no (cambio .mjs/.md task-system, verificado con node --test + parity, sin Rust)
- **Decisiones:** Spec tabla 5 filas (ubicación raíz, re-export, flag dual, JSON pointer, parity dual) — todas implementadas
- **Problemas conocidos:** campaign_discover_skills_v2 regex bug (fuera de scope, reportado en cierre); plan file con cambios ajenos BND-08 (no tocados); review P2-01 por agente distinto pendiente (deuda de proceso)
- **Próxima tarea:** STABLE-04 (plan activo) — o la que indique orquestador
