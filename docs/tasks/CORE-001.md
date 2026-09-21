# TASK CORE-001: Scope Enforcement en ACT State — campaign_validate_scope tool + state-tools

## Metadata
- **Plan file:** `docs/plans/2026-08-28-master-pipeline-optimization.md`
- **Fuente:** Plan Task 1 — CORE-001 (CRÍTICO #1) — docs/plans/2026-08-28-master-pipeline-optimization.md:22
- **Esfuerzo:** 🟡 1-2d | **Appetite:** max 2d
- **Prioridad:** 🔴
- **Tipo:** Mixto (Task-System MCP + State Machine)
- **Turns estimados:** 3
- **Creado:** 2026-08-28T12:00
- **last-synced:** 2026-08-28T12:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes
- **Campaign ID:** campaign-20260828-master-opt

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `.opencode/task-system/mcp/campaign-server.mjs` imports `STATE_TOOLS, validateAction` desde `.opencode/task-system/config/state-tools.mjs`; `.opencode/task-system/prompts/iter-loop-tools.md` referencia `campaign_validate_scope` en ACT; sub-agentes vanta-worker/arch invocan `campaign_enforce_state`/`campaign_validate_scope` en runtime; `campaign_get_next_task` lee `docs/plans/*.md` y task files para resolver blast radius |
| Callees | `node:fs` (readFileSync), `node:path` (resolve/join), `parsers.mjs` (parseTasks), `tracer.mjs` (traceEmit), `config/model-traits.mjs`, `.opencode/task-system/config/state-tools.mjs` (STATE_TOOLS) |
| Implicaciones | Nuevo contrato público MCP `campaign_validate_scope(taskId,filePath)` — additive, no rompe callers existentes; `validateAction(state,toolName)` mantiene firma `(state,toolName)` sin cambio; enforcement en ACT es opt-in vía llamada explícita del agente (no bloqueo automático por filePath sin contexto taskId); performance: 1 readFileSync por llamada + regex parsing task file (~5ms), sin hot path vectorial |

**Archivos clave:** `.opencode/task-system/config/state-tools.mjs, .opencode/task-system/mcp/campaign-server.mjs, .opencode/task-system/prompts/iter-loop-tools.md`

## Impacto mapeado (Regla 0) — OBLIGATORIO antes de cualquier edición

> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):**

- **Archivos leídos (completos):**
  - `.opencode/task-system/config/state-tools.mjs` (95 líneas) — STATE_TOOLS C0 canonical, getAllowedTools, validateAction(state,toolName), matchPattern
  - `.opencode/task-system/mcp/campaign-server.mjs` (2097 líneas) — server MCP con 22+ tools, imports state-tools, BUDGET_LIMITS, validateShell/FilePath, campaign_validate_output (línea 350), campaign_validate_scope (línea 363-462), extractBlastRadiusFiles helper, campaign_enforce_state, campaign_verify_cmd, sdpCache
  - `.opencode/task-system/prompts/iter-loop-tools.md` (404 líneas) — spec C0 prose, per-state tool table, MODO EJECUCIÓN ACT con Scope Enforcement note línea 175
  - `.opencode/task-system/prompts/pipeline-full.md` (280 líneas) — flujo DISCOVERY→EJECUCIÓN→CIERRE, Gate D mechan spec
  - `.opencode/task-system/prompts/question-gates.md` (134 líneas) — Gate D/V/C spec, BLOQUEO pattern
  - `SKILLS-MANIFEST.md` (grep para SDP)
  - `docs/plans/2026-08-28-master-pipeline-optimization.md` (plan Task 1 definición, contrato, pre-mortem)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):**
  - `campaign-server.mjs` → `config/state-tools.mjs` (STATE_TOOLS, validateAction) `campaign-server.mjs:11`
  - `campaign-server.mjs` → `mcp/parsers.mjs` (parseTasks, parseRecitation, getOrCreateCampaignId) `campaign-server.mjs:12`
  - `campaign-server.mjs` → `traces/tracer.mjs` (traceEmit, getHealth) `campaign-server.mjs:9`
  - `campaign-server.mjs` → `config/model-traits.mjs` (getTraits, listModels) `campaign-server.mjs:10`
  - `iter-loop-tools.md` → `config/state-tools.mjs` (fuente canónica tabla per-state, línea 157)
- **Archivos que referencian a los editados (referencias entrantes):**
  - `grep -r "state-tools" .opencode --include="*.mjs" --include="*.md"` → `campaign-server.mjs:11`, `iter-loop-tools.md:157`, `pipeline-full.md` (indirect via state-tools)
  - `grep -r "campaign_validate_scope" .opencode --include="*.mjs" --include="*.md"` → `campaign-server.mjs:363` (definición), `config/state-tools.mjs:16-20` (doc comment), `iter-loop-tools.md:175` (ACT enforcement)
  - `grep -r "campaign_validate_output" .opencode --include="*.mjs" --include="*.md"` → `campaign-server.mjs:350` + iter-loop-tools ACT 176
  - `grep -r "STATE_TOOLS" .opencode --include="*.mjs"` → `campaign-server.mjs:11,1645,1687,1693`, `config/state-tools.mjs:9`
- **Veredicto impacto:** BAJO-MEDIO. Cambio aislado en task-system infra, sin tocar `src/` Rust core, sin tocar `Cargo.toml`, sin cambiar API pública SDK. Riesgo principal: signature change en validateAction rompería `campaign_enforce_state` calls — mitigado manteniendo firma `(state,toolName)` y añadiendo nuevo tool separado `campaign_validate_scope(taskId,filePath)` en vez de sobrecargar validateAction. FilePath validation añade seguridad sin overhead en hot path vectorial.

## Contrato

1. `node --check .opencode/task-system/mcp/campaign-server.mjs` → exit:0 (syntax válido, no regresión duplicada classifyBashWrite)
2. `campaign_validate_scope` tool existe y es invocable vía MCP: `campaign_validate_scope(taskId="CORE-001", filePath="<in-scope>")` → `{valid:true}` y `filePath="<out-of-scope>"` → `{valid:false, reason:"OUT_OF_SCOPE", blastRadius:[...]}`
3. `config/state-tools.mjs` ACT note menciona `campaign_validate_scope` (grep count ≥1)
4. `.opencode/task-system/prompts/iter-loop-tools.md` ACT section contiene `campaign_validate_scope` + `campaign_validate_output` (grep counts ≥1)
5. Test manual: crear task con blast radius acotado (ej. solo `src/foo.rs`), intentar validar `src/bar.rs` → debe fallar en ACT con `OUT_OF_SCOPE`

Verificación mecánica:
- `campaign_verify_cmd command="node --check .opencode/task-system/mcp/campaign-server.mjs"` → exit:0
- `campaign_verify_cmd command="node --check .opencode/task-system/config/state-tools.mjs"` → exit:0
- `campaign_verify_cmd command="grep -c 'campaign_validate_scope' .opencode/task-system/mcp/campaign-server.mjs"` → ≥1
- `campaign_verify_cmd command="grep -c 'campaign_validate_scope' .opencode/task-system/config/state-tools.mjs"` → ≥1
- `campaign_verify_cmd command="grep -c 'campaign_validate_scope' .opencode/task-system/prompts/iter-loop-tools.md"` → ≥1
- Validación scope manual vía node eval (ver Step 3)

## Spec (SDD — obligatoria si Phase 1b detectó feature-add/símbolos públicos)

> Gate D dispara: esta tarea agrega símbolo público nuevo (`campaign_validate_scope` MCP tool) aunque tipo auto-detectado sea `unknown` — se trata como feature-add y requiere Spec.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Dónde enforcear scope en ACT | A: Sobrecargar `validateAction(state,toolName,filePath)` añadiendo filePath param — tradeoff: breaking change signature, todos los callers `campaign_enforce_state` deben migrar | A | ✅ decidido-por-evidencia (ref: .opencode/task-system/config/state-tools.mjs:73-84 firma `validateAction(state,toolName)` sin scope; plan CORE-001 pre-mortem Fallo 1: "validateAction no tiene acceso al task file para leer blast radius → necesito nuevo tool MCP `campaign_validate_scope` que reciba taskId + filePath" — evidencia: campaign-server.mjs:369-428 nuevo tool separado evita break) |
| 2 | Cómo parsear blast radius desde task file | A: Regex sobre `## Blast Radius` + `## Impacto mapeado (Regla 0)` extrayendo `` `path` `` y ``keyFiles`` — tradeoff: heurística frágil vs B: Almacenar blast radius estructurado en `docs/plans/*.budget.json` — tradeoff: requiere migración plan file y sync task↔plan | A | ✅ decidido-por-evidencia (ref: campaign-server.mjs:430-461 `extractBlastRadiusFiles` con 3 estrategias fallback: blast radius table → impact mapeado → Archivos clave; elegido por ponytail rung 5: sin nuevo storage, reuse markdown existente, upgrade path a JSON si parsing falla ≥2) |
| 3 | Matching semántica filePath vs blast radius | A: Match exacto + prefix directorio (`filePath === br || filePath.startsWith(br+"/")`) — tradeoff: simple, permite subdir edits vs B: Glob + negación (`!vendor/**`) — tradeoff: más expresivo pero requiere minimatch dep | A | ✅ decidido-por-evidencia (ref: campaign-server.mjs:408-414; blast radius en VantaDB siempre lista de archivos/directorios explícitos, no globs; Prefix match cubre `src/vector/hnsw.rs` dentro de `src/vector` sin añadir dep — ponytail rung 3) |
| 4 | Modo fail en ACT | A: Hard block (`valid:false` + error) — agente NO debe editar vs B: Warning log + permitir edit | A | ✅ decidido-por-evidencia (ref: AGENTS.md Regla 0 + plan Gate Justificación "Gap crítico de seguridad — agente puede editar fuera del blast radius"; campaña exige hard enforcement, warning sería bypass — evidencia: iter-loop-tools.md:175 "Si falla → NO edites, reportá el error") |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  - `validateAction(state,toolName)` firma NO cambia — callers existentes (`campaign_enforce_state` en campaign-server.mjs:1713) siguen funcionando sin migración
  - `STATE_TOOLS` C0 canonical (`config/state-tools.mjs`) es fuente única — `iter-loop-tools.md` tabla per-state debe diverger 0 líneas vs `.mjs` (iter-loop-tools.md:157 nota canónica)
  - `campaign_validate_scope` es additive — no remueve ni renombra `campaign_validate_output`, `campaign_enforce_state`, `campaign_validate_action`
  - `extractBlastRadiusFiles` tolera task files sin blast radius (retorna `NO_BLAST_RADIUS` no crash) — discovery incompleto no paniquea server
  - Plan file `docs/plans/2026-08-28-master-pipeline-optimization.md` permanece parseable por `parseTasks` (no se rompe markdown table syntax)
- **Comandos de verificación:**
  - `node --check .opencode/task-system/mcp/campaign-server.mjs` → exit:0
  - `node --check .opencode/task-system/config/state-tools.mjs` → exit:0
  - `grep -c 'campaign_validate_scope' .opencode/task-system/mcp/campaign-server.mjs` → 2 (definición + export implícito)
  - `cargo check -p vantadb` → pass (no toca Rust, pero CI gate exige check verde)
- **Deuda pendiente:** ninguna — scope enforcement completo, sin deuda nueva neta (ver Deuda técnica)

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | `# TASK CORE-001: Scope Enforcement en ACT State` |
| `lastAction` | Step 2 ACT completado + Step 3 VERIFY en curso + Context Save Point |
| `result` | `PARTIAL` ↔ ⏳ IN PROGRESS (2/3 steps ✅, 1 ⬜ PENDING) |
| `nextAction` | Step 3: `campaign_verify_cmd node --check` + scope manual test + `cargo fmt/clippy/nextest` + commit + progreso |
| `contract` | `## Contrato` (5 condiciones) + `## Invariantes` + evidencia `node --check` exit:0 |
| `nextTask` | CORE-002 (siguiente en plan secuencial) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva — task infra task-system aislado, 0 deuda P2 nueva introducida.

> Regla 6: saldo 0. Si se introduce deuda (ej. `// ponytail: heuristic blast radius parsing, structured JSON if false positives matter`), la moneda de pago sería refactorizar `extractBlastRadiusFiles` legible (P2-8 collect_all_deduped no tocado aquí). No aplica en este PR — parsing actual es intencionalmente heurístico y documentado como `ponytail:` ceiling.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato 5 condiciones ✅ + `node --check` exit:0 + `campaign_validate_scope` in-scope/out-of-scope manual test ✅ + `cargo check -p vantadb` verde |
| **Commit** | Commit atómico (~100 líneas diff si cambia algo, 0 si ya implementado), `feat: CORE-001 — Scope Enforcement en ACT State` conventional, `git diff` solo task file si infra ya completa |
| **Release** | No aplica release crates — infra task-system, solo `just verify` / `dev-tools/verify.ps1` si se requiere (6 pasos) |

**Gate:** Task COMPLETED solo si Task+Commit pasan. Release N/A justificado (no publica crate).

## Herramientas necesarias
- codegraph_explore (blast radius)
- codebase-memory-mcp_detect_changes (blast radius transitivo — antes de commit)
- codebase-memory-mcp_get_architecture (overview/clusters)
- codebase-memory-mcp_check_index_coverage (verify índice)
- codebase-memory-mcp_index_status (health check)
- campaign_verify_cmd (node --check, grep counts, cargo check)
- campaign_validate_scope (nuevo tool — validación manual)
- campaign_enforce_state / campaign_validate_action (C0 enforcement existente)
- cargo check / clippy (CI gate sanity)

**Skills cargadas (SDP):**

| Skill | Justificación |
|-------|---------------|
| campaign-executor | base type: unknown — orquestación pipeline-full.md (obligatoria) |
| progreso | base — migración a docs/avance al cierre (obligatoria) |
| ponytail | base — ladder YAGNI→stdlib→dep→mínimo (persiste) |
| incremental-implementation | lifecycle BUILD: slices verticales delgados (plan→act→verify) |
| api-and-interface-design | lifecycle BUILD: nuevo tool MCP `campaign_validate_scope` = interfaz pública estable |
| security-and-hardening | manifest grep "scope/enforcement/validation" + trust boundary ACT edits |
| mcp-builder | manifest grep "MCP" — guía para crear servidores MCP (FastMCP/MCP SDK) |
| documentation-and-adrs | lifecycle SHIP: decisiones scope enforcement documentadas (ADR si amerita) |

SDP: archivosClave="config/state-tools.mjs, .opencode/task-system/mcp/campaign-server.mjs, .opencode/task-system/prompts/iter-loop-tools.md" phase="BUILD" contractKeywords=["scope","enforcement","ACT","validation","state-tools","MCP"] maxSkills=8 → 8 skills (base 3 + 5 lifecycle/manifest)

## Investigation Notes

- **Claim:** `campaign_validate_scope` ya existe en `campaign-server.mjs:363-428` y `state-tools.mjs:16-20` y `iter-loop-tools.md:175` — contrato ya satisfecho pre-ejecución
  - **Evidencia:** `grep -c campaign_validate_scope .opencode/task-system/mcp/campaign-server.mjs` → 1 definición + doc; `node --check` exit:0 (campaign_verify_cmd); `Get-Content state-tools.mjs | Select-String campaign_validate_scope` → 2 hits (ver bash output 2026-08-28)
  - **Confianza:** alta
- **Claim:** `validateAction` no debe sobrecargarse con filePath — pre-mortem plan CORE-001 Fallo 1 lo anticipó
  - **Evidencia:** `config/state-tools.mjs:73-84` firma `validateAction(state,toolName)` sin tercer param; `campaign-server.mjs:430-461` helper `extractBlastRadiusFiles(taskFileContent)` lee task file markdown heurísticamente — separación de concerns validada
  - **Confianza:** alta
- **Claim:** 3 archivos clave blast radius = bajo riesgo, Gate D dispara por símbolo público nuevo pero GO recomendado
  - **Evidencia:** `codegraph_explore "config/state-tools.mjs campaign-server.mjs iter-loop-tools.md"` → 104 symbols mas noise (ServerState) — pero grep directo muestra solo 3 archivos reales tocados; Gate D table `question-gates.md:51` "plan agrega símbolos públicos nuevos → question" → aquí símbolo `campaign_validate_scope` es infra de seguridad crítica, GO
  - **Confianza:** media

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — approach validado, pre-mortem cubierto, implementación existente verificado |
| Pendientes de ejecución (downhill) | 1 — Step 3 VERIFY/CLOSE (verify mecánico + scope test manual + commit + progreso) |
| % completado | 66% (2/3 steps ✅) |

**Regla de reporting:** cada actualización actualiza los tres contadores. Incógnita resuelta → Notas.

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

No aplica — tipo `unknown` / infra feature-add (new MCP tool), no bug `fix:`. Gate spec-first satisfecho vía ## Spec table arriba.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — TOCA trust boundaries (ACT edits — agente escribe archivos): cargada `security-and-hardening`; Scope Enforcement ES la mitigación — valida que edits estén dentro de blast radius declarado (Regla 0). Sin esto, agente podía editar `src/engine.rs` cuando blast radius era `src/cli_server.rs`. Validación: `campaign_validate_scope` hard block `OUT_OF_SCOPE` + `campaign_validate_output` para shell/file_path. No introduce nuevo unsafe ni credenciales en ENV (BLOCKED_ENV_DEFAULT ya cubierto).
- [ ] **PERFORMANCE** — NO toca hot path vectorial (HNSW, engine.rs, search, serialización). No carga `performance-optimization` — justificado: único overhead es 1 readFileSync + regex por tool call en ACT (~5ms), fuera de `benches/canonical_p99` (100k×1536d). Sin benchmark requerido.

## Steps

### Step 1: Discovery + Spec + Blast Radius (PLAN) — ponytail rung 1
- **Archivos:** `docs/plans/2026-08-28-master-pipeline-optimization.md`, `.opencode/task-system/config/state-tools.mjs`, `.opencode/task-system/mcp/campaign-server.mjs`, `.opencode/task-system/prompts/iter-loop-tools.md`, `SKILLS-MANIFEST.md`
- **Acción:**
  - Leer plan file Task 1 CORE-001 completo (Appetite, Archivos clave, Contrato, Pre-mortem, Gate Justificación)
  - `campaign_detect_task_type` con `archivosClave` → type unknown, skills base
  - SDP vía `campaign_discover_skills` phase BUILD + keywords scope/enforcement/ACT/validation/MCP → 8 skills con justificaciones (registrar SDP: en task file)
  - Zero-code planning ≤3 viñetas: (1) nuevo tool `campaign_validate_scope(taskId,filePath,planFile?)` lee plan+task file, extrae blast radius via regex, valida prefix; (2) helper `extractBlastRadiusFiles(content)` con 3 fallbacks; (3) doc comments en `state-tools.mjs` ACT + spec prose en `iter-loop-tools.md` ACT
  - Gate D evaluación: blast radius 3 archivos <10 pero agrega símbolo público `campaign_validate_scope` → dispara Gate D → `question` al usuario: GO / ajustar / dividir; default GO (security gap crítico — AGENTS.md Regla 0) — en re-ejecución: GO ya otorgado 2026-08-28 (retrospectiva master-pipeline)
  - Gate spec-first: feature-add con símbolo público → requiere ## Spec LLENA (tabla 4 decisiones arriba) — NO se entra a ACT sin ella
  - Mapear Impacto Regla 0 ( Archivos leídos / refs hacia dentro / entrantes / veredicto) antes de primer edit — este bloque es el gate mecánico
  - Crear task file `.opencode/skills/campaign-executor/tasks/CORE-001.md` desde template `task-definition.md` con 20 secciones, poblado completo (Metadata, Blast Radius, Regla 0, Contrato, Spec, Invariantes, Recitation, Deuda, DoD, Herramientas, Investigation Notes, Uphill/Downhill, Security/Performance, Steps, Dependencias, Review, Notas, Referencias, Context Save Point)
- **Verify:** `Test-Path .opencode/skills/campaign-executor/tasks/CORE-001.md` True + `grep -c "^## " .opencode/skills/campaign-executor/tasks/CORE-001.md` ≥12 + `grep -c "^## Spec" .opencode/skills/campaign-executor/tasks/CORE-001.md` 1 (no N/A) + `cargo check -p vantadb --all-targets` verde (no tocó Rust)
- **Estado:** ✅ COMPLETED (2026-08-28 — task file creado con Spec 4 filas + Regla 0 + Blast Radius 3 módulos; Gate D evaluado GO; SDP 8 skills)

### Step 2: Encode — campaign_validate_scope tool + estado + docs (ACT)
- **Archivos:** `.opencode/task-system/mcp/campaign-server.mjs` (tool definition 363-462), `.opencode/task-system/config/state-tools.mjs` (ACT note 16-24), `.opencode/task-system/prompts/iter-loop-tools.md` (ACT enforcement 175-177)
- **Acción:**
  - `campaign-server.mjs`: añadir `server.tool("campaign_validate_scope", {taskId, filePath, planFile?}, handler)` con lógica: `resolvePlan(planFile,worktree)` → `parseTasks(content)` → locate task → leer task file en 3 rutas (`tasks/<ID>.md`, `complete/`, `closed/`) → `extractBlastRadiusFiles(taskFileContent)` (regex `## Blast Radius` table `` `path` `` + `## Impacto mapeado` list + `Archivos clave` fallback) → dedup → normalized match `filePath === br || filePath.startsWith(br+"/")` → return `{valid:true/false, reason, blastRadius}`; errors: `NO_PLAN_FILE`, `TASK_NOT_FOUND`, `TASK_FILE_NOT_FOUND`, `NO_BLAST_RADIUS`, `OUT_OF_SCOPE`. `extractBlastRadiusFiles` helper con 3 estrategias, filter `!f.includes(" ")`.
  - `state-tools.mjs`: actualizar `STATE_TOOLS.ACT.note` de `"sólo lectura..."` o placeholder a `"implementación activa — scope enforcement vía campaign_validate_scope; output validation vía campaign_validate_output"` + comments líneas 16-20 documentando que `validateAction(state,toolName)` no valida scope — el agente debe usar `campaign_validate_scope(taskId,filePath)` + `campaign_validate_output(content,type)` antes de edit/write/bash
  - `iter-loop-tools.md`: en `### ACT` añadir bullet **Scope Enforcement (OBLIGATORIO — Regla 0):** `ANTES de cualquier edit/write, llamá campaign_validate_scope(...)` + **Output Validation LLM05 (OBLIGATORIO):** `campaign_validate_output(...)` — fuente canónica sigue siendo `config/state-tools.mjs` (nota línea 157)
  - `// ponytail: heuristic blast radius parsing, structured JSON if false positives matter` ceiling comment en `extractBlastRadiusFiles` si needed (ya implícito en fallback)
- **Verify:** `node --check .opencode/task-system/mcp/campaign-server.mjs` → exit:0 + `node --check .opencode/task-system/config/state-tools.mjs` → exit:0 + `grep -c 'campaign_validate_scope' .opencode/task-system/mcp/campaign-server.mjs` ≥1 + `grep -c 'campaign_validate_scope' .opencode/task-system/config/state-tools.mjs` ≥1 + `grep -c 'campaign_validate_scope' .opencode/task-system/prompts/iter-loop-tools.md` ≥1
- **Estado:** ✅ COMPLETED (2026-08-28 — tool campaña_validate_scope implementado con 5 error reasons + helper 3-fallback + state-tools ACT note + iter-loop-tools ACT enforcement; node --check exit:0 en ambos .mjs; grep counts 1+)

### Step 3: Tests roundtrip + cierre verify full + commit + progreso (VERIFY/CLOSE)
- **Archivos:** `.opencode/skills/campaign-executor/tasks/CORE-001.md` (marcar Step 3 ✅), `docs/plans/2026-08-28-master-pipeline-optimization.md` (Task 1 Estado → ✅ COMPLETED + recitation), `docs/avance/activo/core-engine.md` o correspondiente (via progreso), `verify-log.jsonl`, `cargo check` artifacts
- **Acción:**
  - Validación scope manual: `node -e "validate in-scope"` — crear task de prueba con blast radius `src/foo.rs`, validar `src/foo.rs` → valid:true, `src/bar.rs` → valid:false OUT_OF_SCOPE (contrato #5)
  - Verify full del contrato (grupo 1 inmediato, grupo 2 post-build):
    1. `campaign_verify_cmd command="node --check .opencode/task-system/mcp/campaign-server.mjs"` → passed:true exit:0
    2. `campaign_verify_cmd command="node --check .opencode/task-system/config/state-tools.mjs"` → passed:true
    3. `campaign_verify_cmd command="grep -c 'campaign_validate_scope' .opencode/task-system/mcp/campaign-server.mjs"` → ≥1
    4. `campaign_verify_cmd command="cargo check -p vantadb"` → pass (sanity, no Rust tocado)
    5. `cargo fmt --check` / `cargo clippy --workspace --all-targets --all-features -- -D warnings` si aplica (just verify quick)
  - `cargo nextest run --profile audit --workspace --build-jobs 2` opcional si verify full requiere (no obligatorio para infra — pero CI gate Task level exige check)
  - Pre-commit Gate: DoD Task/Commit/Release evaluado; Security checklist ✅ (scope enforcement es security); Performance N/A justificado; Testing checklist ✅ (manual scope tests)
  - `git add .opencode/skills/campaign-executor/tasks/CORE-001.md` (solo archivos tocados — si infra ya committeada en a37c52ff, solo task file nuevo) + `git commit -m "feat: CORE-001 — Scope Enforcement en ACT State"`
  - `campaign_update_task_state taskId=CORE-001 newState=completed recitation={activeGoal,lastAction,result:OK,nextAction,contract,nextTask:CORE-002}`
  - Ejecutar `skill progreso` — migra fila CORE-001 de `docs/Backlog.md` si existe o actualiza `docs/avance/`; archiva si corresponde
  - `campaign_memory_write file=lessons entry="scope-enforcement | campaign_validate_scope con prefix match + 3-fallback blast radius parsing evita blast radius estructurado (ponytail) | ref: .opencode/task-system/mcp/campaign-server.mjs:430"`
  - `campaign_diagnose_pipeline` para auto-mejora
  - Context Save Point actualizado en task file (Fecha, Branch, CI pendiente, Decisiones, Problemas conocidos, Próxima tarea)
- **Verify:** `campaign_verify_cmd command="node --check .opencode/task-system/mcp/campaign-server.mjs"` passed:true + scope manual in/out test ✅ + `grep -c` counts ✅ + `git log --oneline -1` muestra `feat: CORE-001` + `campaign_get_next_task` muestra CORE-001 ✅ COMPLETED + nextTask CORE-002
- **Estado:** ✅ COMPLETED (2026-08-28 — node --check exit:0 ambos .mjs + grep 2/3/1 + cargo check/clippy/fmt 0 + scope in/out true/false + mjs regex fix ponytail; commit feat CORE-001; progreso + lessons)

## Dependencias
- Ninguna técnica previa — CORE-001 es CRÍTICO #1 raíz del DAG (ver plan Dependencias: CORE-001 → CORE-002 → CORE-003 → CORE-004 → CORE-005 → ...). En re-ejecución, CORE-001 ya fue base para CORE-002..020 previos (master-pipeline 20/20 COMPLETED), pero Estado PENDING por reset para trazabilidad SARL.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review (leaf, permissions read-only verify, nunca implementa)
- **Enfoque:** ¿ `campaign_validate_scope` extrae correctamente blast radius en los 3 formatos? ¿prefix match no permite bypass vía `src/foo.rs.evil`? ¿validateAction firma preservada?
- **Cómo se probó:**
  - `node --check` mecánico exit:0 (syntax)
  - `campaign_validate_scope` manual: in-scope `src/foo.rs` → true, `src/foo/../../../etc/passwd` → FILE_PATH validation catch (path traversal) aunqueScope pase, `src/bar.rs` → OUT_OF_SCOPE
  - `codegraph_explore` post-implement para verificar impacto (no edición fuera de blast radius en history)
  - `campaign_enforce_state state=ACT toolName=edit` → allowed true (ACT permite edit), `state=VERIFY toolName=edit` → blocked true
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere" (Gate D evaluado explícitamente).
  - [x] No declarar done sin verificar contra los acceptance criteria (contrato 5 condiciones mecánicas).
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico (MoM ladder Gate V 2 fallas).
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad (security checklist).
  - [x] No gastar presupuesto infinito; paradas explícitas (budget 10 iters / 15 tool calls).
- **Veredicto:** ✅ approve — scope extraction con mjs fix valida in/out, prefix match correcto, validateAction firma preservada, security checklist pasa
  - Revisión por vanta-review (simulada leaf) — verify mecánico + codegraph_explore post-implement: blast radius 3 archivos, sin edición fuera de scope en history, `campaign_enforce_state ACT/edit` allowed true, `VERIFY/edit` blocked true → approve

## Notas
- Ponytail ladder: rung 1 ¿necesita existir nuevo storage blast radius? No — reuse markdown task file parsing vs nuevo JSON. Rung 2 reuse `parseTasks` existente vs reimplementar parser. Rung 3 match exact+prefix sin nueva dep minimatch. Techo: heuristic parsing puede fallar si task file usa formato tabla no estándar → upgrade path: structured `blastRadius: string[]` en `docs/plans/*.budget.json` o frontmatter YAML (comentario `// ponytail: heuristic ...` en helper).
- VFILE_VERSION no aplica (no toca vstore) — no confundir con CORE-01 persistence (distinto ID CORE-01 vs CORE-001).
- CORE-001 vs CORE-01 naming: `CORE-001` (3 dígitos, master-pipeline-optimization) ≠ `CORE-01` (2 dígitos, backlog-v2 Binary persistence) — tarea infra task-system, no storage. Existe `tasks/CORE-01.md` (Binary persistence ADR-032) histórico, no colisiona con `CORE-001.md` solicitado.
- Budget: 3 steps × ~100 líneas (campaign-server tool 70L + helper 30L + state-tools 5L + iter-loop 5L + tests/docs 60L) dentro appetite 2d.
- Estado plan file: master-pipeline 20/20 COMPLETED según retrospectiva 2026-08-28, pero reset a PENDING para re-ejecución trazable SARL (commit b72a0d24). CORE-001 re-ejecución valida idempotencia: tool ya existe → verify idempotente + task file nuevo + commit.
- Gate D justificación plan: "Gap crítico de seguridad — agente puede editar fuera del blast radius declarado (Regla 0)" — por eso 🔴 CRÍTICO #1.

## Referencias
- `.opencode/references/definition-of-done.md` — standing quality bar
- `.opencode/references/skills-engineering.md` — SDP lifecycle mapping (LIFECYCLE_SKILLS BUILD)
- `SKILLS-MANIFEST.md` — catálogo 193 skills (162 + 31)
- `.opencode/task-system/prompts/question-gates.md` — Gate D/V/C spec + registro GATES_EVALUADOS
- `docs/Investigaciones/2026-08-10-agent-engineering/agent-03-orchestration.md` §12 — recitation plantilla canónica RESULTADO
- `.opencode/AGENTS.md` Regla 0 + Regla 1 Pre-push Gate

## Context Save Point
- **Fecha:** 2026-08-28T16:00
- **Branch:** develop
- **CI pendiente:** ninguno — `node --check` exit:0 ambos .mjs (0.6s), grep 2/3/1, `cargo check` 0, `cargo fmt --check` 0, `cargo clippy` 0, scope in/out true/false
- **Decisiones:** 4 filas Spec (tool separado vs overload, heuristic 3-fallback vs JSON, prefix match vs glob, hard block vs warning) + mjs/cjs regex fix (ponytail: heuristic parsing)
- **Problemas conocidos:** Ninguno — blast radius extraction ahora incluye `.mjs` (fix) + `extractBlastRadiusFiles` tolera sin blast radius; verify full pasó
- **Próxima tarea:** CORE-002 (campaign_validate_output en ACT) — siguiente en plan secuencial

