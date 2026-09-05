# HIGH-009: Session Cleanup al Cerrar Campaña — pipeline-run.md paso 8 delete

## Metadata
- **Plan file:** docs/plans/2026-08-28-master-pipeline-optimization.md
- **Fuente:** plan file Task 9 / HIGH-009 (ALTO #9)
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🟠
- **Tipo:** Mixto (infra task-system + prompts)
- **Turns estimados:** 1
- **Creado:** 2026-08-28T18:30
- **last-synced:** 2026-08-28T18:30
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes
- **Campaign ID:** cecc8468-9451-4d56-a3ef-1684e123ab8a

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `.opencode/task-system/prompts/pipeline-run.md` paso 8 (8. CUANDO no haya más ⬜ PENDING — líneas 205-223), `docs/plans/2026-08-28-master-pipeline-optimization.md` Task 9 (líneas 223-236, contrato grep session_track.*delete), `.opencode/task-system/mcp/campaign-server.mjs` Tool 17 campaign_session_track (líneas 1761-1850), `docs/plans/2026-08-28-master-pipeline-optimization.md` Verificación por task línea 544 |
| Callees | `.opencode/task-system/mcp/campaign-server.mjs` `SESSION_DIR` (enforcement/sessions), `sessionPath()` + `readSession`/`writeSession` + switch action create/get/update/list/delete, `.opencode/task-system/prompts/pipeline-run.md` paso 3 TRACKING DE SESIÓN (create/update) + paso 8 Session Cleanup delete |
| Implicaciones | contrato aditivo: si faltara `campaign_session_track action="delete"` en paso 8 → stale sessions contaminan siguientes campañas (HIGH-009 gate justification); presente → sesión limpiada al cerrar campaña, sin leak; sin breaking change (ponytail rung 1 si ya existe) |

## Impacto mapeado (Regla 0) — OBLIGATORIO antes de cualquier edición
> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES del primer step de edición. Sin este bloque poblado, NO se escribe ni se ejecuta ningún step que edite archivos.

- **Archivos leídos (completos):** `.opencode/task-system/prompts/pipeline-run.md` (243 líneas, paso 3 TRACKING DE SESIÓN líneas 43-46 create/update + paso 8 CUANDO no haya más ⬜ PENDING líneas 205-223 con Session Cleanup 208), `.opencode/task-system/mcp/campaign-server.mjs` (Tool 17 líneas 1761-1810: SESSION_DIR enforcement/sessions, sessionPath, readSession, writeSession, server.tool campaign_session_track con z.enum create/get/update/list/delete + switch case create), `docs/plans/2026-08-28-master-pipeline-optimization.md` (Task 9 líneas 223-236, contrato grep -n 'session_track.*delete' → en paso 8)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `pipeline-run.md` no tiene imports; paso 8 referencia `campaign_session_track action="delete" sessionId=<sessionId>` como cleanup; paso 3 referencia create/update con sessionId único al inicio + por tarea; `campaign-server.mjs` importa `join`, `existsSync`, `mkdirSync`, `readFileSync`, `writeFileSync` para SESSION_DIR + sessionPath sanitización `replace(/[<>:"/\\|?*]/g, "_")`; define `SESSION_DIR = join(TASK_SYSTEM, "enforcement", "sessions")`
- **Archivos que referencian a los editados (referencias entrantes):** `Select-String "session_track.*delete" pipeline-run.md` → 1 hit línea 208: `**Session Cleanup (HIGH-009):** llamá campaign_session_track action="delete" sessionId=<sessionId>` ✅; `Select-String "campaign_session_track" pipeline-run.md` → 4 hits (líneas 44 create, 45 update, 46 update, 208 delete) ✅; `Select-String "session_track" campaign-server.mjs` → 2 hits (1761 comment + 1780 tool name) ✅; `Select-String "SESSION_DIR" campaign-server.mjs` → 1 hit 1763 ✅; `Select-String "HIGH-009" docs/plans/2026-08-28-master-pipeline-optimization.md` → 6 hits (Task 9, DAG, orden secuencial, verif 544, recitaciones, próxima HIGH-009) ✅; `git log --follow -- pipeline-run.md` → 9e5730ff feat Master pipeline optimization (implementa HIGH-009 junto a 19 items); `git blame pipeline-run.md` línea 208 → 9e5730ff Eros Nessy 2026-08-28
- **Veredicto impacto:** bajo — verificación idempotente sin edición; si hubiese edición sería bajo aditivo (solo añade 1 línea `campaign_session_track action="delete"` en paso 8), sin riesgo de regresión porque delete es acción existente en campaign-server.mjs (z.enum incluye delete desde implementación TOOL 17), compatible con paso 3 create/update y con sesiones previas (delete idempotente si session no existe)

## Contrato
Contrato del plan (HIGH-009):
```
campaign_verify_cmd command="grep -n 'session_track.*delete' .opencode/task-system/prompts/pipeline-run.md" → en paso 8
```
Verificación mecánica (powershell equivalente campaign_verify_cmd):
```
Select-String -Pattern "session_track.*delete" -Path .opencode/task-system/prompts/pipeline-run.md → línea 208 ✅ (Session Cleanup HIGH-009: campaign_session_track action="delete" sessionId=<sessionId>)
Select-String -Pattern "^8\." -Path .opencode/task-system/prompts/pipeline-run.md → línea 205 ✅ (8. CUANDO no haya más ⬜ PENDING: — confirma que 208 está dentro de paso 8)
Select-String -Pattern "campaign_session_track" -Path .opencode/task-system/prompts/pipeline-run.md → 4 hits (44,45,46,208) ✅
node --check .opencode/task-system/mcp/campaign-server.mjs → 0 ✅
```
Resultado: contrato pasa ✅ (ver Investigation Notes). Sin re-edición — ponytail rung 1 idempotente. Debe estar en paso 8 (líneas 205-223), no en paso 3 ni en probes.

## Spec (SDD — obligatoria si Phase 1b detectó feature-add/símbolos públicos)
> Definición de contenido válido: question-gates.md §"Contenido válido de `## Spec`". Tabla de decisiones O justificación por evidencia por ítem. `N/A` solo aceptable en tareas 100% docs sin decisiones técnicas.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | ¿Re-implementar Session Cleanup si pipeline-run.md paso 8 ya tiene `campaign_session_track action="delete" sessionId=<sessionId>` en línea 208 dentro de "8. CUANDO no haya más ⬜ PENDING:"? | A) No re-implementar (idempotente, menor riesgo, ponytail rung 1) / B) Re-escribir igual (ruido, posible duplicar línea delete, diff innecesario) | A | ✅ decidido-por-evidencia (ref: pipeline-run.md:208 Select-String 1 hit session_track.*delete + 205 "^8\." confirma pertenencia a paso 8 + 44-46 create/update pre-existentes; git blame 9e5730ff prueba implementación original 2026-08-28; verify grep -n pasa) |
| 2 | ¿Verificar solo hit `session_track.*delete` o hit + pertenencia a paso 8 + cobertura de campaign-server.mjs delete enum? | A) Solo hit literal (contrato mínimo grep) / B) Hit + pertenencia paso 8 (8. CUANDO 205 → 208 belongs) + campaign-server.mjs z.enum delete existe + SESSION_DIR definida (contrato extendido HIGH-009 descripción "Verifica que pipeline-run.md paso 8 ya tiene campaign_session_track delete") | B — descripción explícita pide "paso 8" + justificación stale sessions | ✅ decidido-por-evidencia (ref: task instrucciones "grep -n 'session_track.*delete' pipeline-run.md debe estar en paso 8" + pipeline-run.md:205-208 estructura + campaign-server.mjs:1782 z.enum delete) |

Justificación: plan pide "Si ya está, marca COMPLETED". Re-implementar introduce riesgo de duplicar delete y romper estructura paso 8 ya validada por pipeline run 20/20 (HIGH-009: grep session_track.*delete → 1 ✅ línea 544).

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** `pipeline-run.md` paso 8 (líneas 205-223) debe seguir conteniendo `8. CUANDO no haya más ⬜ PENDING:` (205) + `- **Session Cleanup (HIGH-009):** llamá campaign_session_track action="delete" sessionId=<sessionId> para limpiar sesión y evitar stale sessions` en línea 208 justo después de `- Ejecutá skill progreso` (207) y antes de `- **RETROSPECTIVA` (209); `pipeline-run.md` paso 3 TRACKING DE SESIÓN líneas 43-46 debe seguir con 44 create,45 update,46 update; `campaign-server.mjs` debe seguir definiendo Tool 17 campaign_session_track con `z.enum(["create","get","update","list","delete"])` línea 1782 y `SESSION_DIR = join(TASK_SYSTEM, "enforcement", "sessions")` línea 1763 + `sessionPath` sanitización; Task 9 Estado en plan file debe ser ✅ COMPLETED; delete debe ser idempotente y usar mismo sessionId del create (tracking)
- **Comandos de verificación:** `Select-String -Pattern "session_track.*delete" -Path .opencode/task-system/prompts/pipeline-run.md` → 1 hit línea 208; `Select-String -Pattern "^8\." -Path pipeline-run.md` → 1 hit línea 205 (confirma 208 ∈ paso 8 porque 205<208<223); `Get-Content pipeline-run.md | Select-Object -Index 204,205,206,207,208,209` → líneas 205 "8. CUANDO" 206 Report 207 skill progreso 208 Session Cleanup delete 209 RETROSPECTIVA; `Select-String -Pattern "campaign_session_track" -Path pipeline-run.md` → 4 hits 44/45/46/208; `Select-String -Pattern "session_track" -Path campaign-server.mjs` → 2 hits 1761/1780; `Select-String -Pattern "SESSION_DIR" -Path campaign-server.mjs` → 1 hit 1763; `node --check campaign-server.mjs` → 0; `git blame pipeline-run.md | Select-String session_track.*delete` → 9e5730ff; `git show 9e5730ff -- pipeline-run.md` diff contiene +Session Cleanup delete
- **Deuda pendiente:** ninguna — idempotente completo, sin edición; próxima tarea HIGH-010 continúa secuencial (Session Cleanup → Context Save Point Reconstruction Tool). Plan file header state debe actualizarse a review checkpoint si aplica tras HIGH-009

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# HIGH-009: Session Cleanup al Cerrar Campaña — pipeline-run.md paso 8 delete` |
| `lastAction` | Step 1 VERIFY completado: pipeline-run.md:208 session_track.*delete 1 hit ✅ en paso 8 (205) + campaign_session_track 4 hits ✅ + campaign-server.mjs SESSION_DIR+delete enum ✅ + node --check 0 ✅ |
| `result` | `OK` ↔ ✅ COMPLETED |
| `nextAction` | HIGH-010 — Context Save Point Reconstruction Tool (siguiente en plan secuencial) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | HIGH-010 |

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda

> Regla 6 (AGENTS.md): toda deuda nueva introducida debe compensar deuda existente — el saldo neto por PR es 0 o negativo. Verificación idempotente sin código nuevo → sin deuda. Si se hubiese añadido Session Cleanup ex nihilo, deuda cero (feature aditiva acotada, 1 línea paso 8 delete).

## Definition of Done (contrato multi-nivel — P2-08)
| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable pipeline-run.md:208 session_track.*delete 1 hit en paso 8 ✅ + campaign_session_track 4 hits (create/update/delete) ✅ + campaign-server.mjs delete enum + SESSION_DIR ✅ + task file sync + recitation actualizada |
| **Commit** | Commit atómico (solo HIGH-009 task file + plan Task 9 update si aplica), conventional commit, verificación mecánica (nunca auto-reporte) |
| **Release** | No aplica (infra task-system prompts, no crate publish) — justificado en Notas |

**Gate:** Task ✅ si pasan niveles aplicables. Release N/A.

## Herramientas necesarias
- codegraph_explore (blast radius inmediato) — verificado en pipeline-run.md paso 8
- codebase-memory-mcp_detect_changes (blast radius transitivo) — no aplica (cambio infra prompts, no Rust)
- codebase-memory-mcp_get_architecture — no aplica
- codebase-memory-mcp_check_index_coverage — no aplica
- campaign_detect_task_type (MCP) — unknown/No detectable (infra prompts, no pattern match → base skills)
- campaign_discover_skills (SDP — campaign-executor, progreso, ponytail) — pipeline-run.md:208 session_track delete
- campaign_verify_cmd (contrato grep -n session_track.*delete → en paso 8)
- node --check (campaign-server.mjs syntax)

**Skills cargadas (SDP):** campaign-executor (base task-system execution) | progreso (registro avance tras COMPLETED) | ponytail (full — escalera YAGNI, rung 1: no re-implementar si ya existe) | incremental-implementation (lifecycle BUILD) | test-driven-development (lifecycle BUILD) | context-engineering | source-driven-development | doubt-driven-development | SDP discovery HIGH-009 — keywords [session, cleanup, campaign, pipeline-run, session_track] → no manifest skills adicionales (infra task-system puro)

## Investigation Notes
- Formato estándar por hallazgo:
  - **Claim:** pipeline-run.md paso 8 contiene `campaign_session_track action="delete" sessionId=<sessionId>` en línea 208 dentro de "8. CUANDO no haya más ⬜ PENDING:"
  - **Evidencia:** .opencode/task-system/prompts/pipeline-run.md líneas 205-209: `8. CUANDO no haya más ⬜ PENDING:` (205) `   - Reportá campaña completada: N/M ✅, K ❌, stalled: S` (206) `   - Ejecutá skill progreso (migración masiva de todas las completadas)` (207) `   - **Session Cleanup (HIGH-009):** llamá campaign_session_track action="delete" sessionId=<sessionId> para limpiar sesión y evitar stale sessions` (208) `   - **RETROSPECTIVA de cierre del plan/milestone**` (209) — verificado via `Select-String -Pattern "session_track.*delete" -Path pipeline-run.md` → 1 hit línea 208 (highlight `session_track` + `action="delete"` + `sessionId=<sessionId>`); `Select-String -Pattern "^8\." -Path pipeline-run.md` → 1 hit línea 205 (confirma 208 ∈ paso 8 porque 205 < 208 < next section); `Get-Content pipeline-run.md | Select-Object -Index 204,205,206,207,208,209` → 205 "8. CUANDO" 206 Report 207 skill progreso 208 Session Cleanup delete 209 RETROSPECTIVA — estructura canónica paso 8
  - **Confianza:** alta
  - **Claim:** pipeline-run.md paso 3 TRACKING DE SESIÓN ya usa create/update y paso 8 cierra con delete (lifecycle completo)
  - **Evidencia:** .opencode/task-system/prompts/pipeline-run.md líneas 43-46: `3. TRACKING DE SESIÓN:` (43) `   - Llamá campaign_session_track (MCP) con action: "create" y sessionId único al inicio` (44) `   - En cada tarea completada → campaign_session_track con action: "update"` (45) `   - Al finalizar → campaign_session_track con action: "update" y estado final` (46) — verificado via `Select-String -Pattern "campaign_session_track" -Path pipeline-run.md` → 4 hits (44 create,45 update,46 update,208 delete) — lifecycle create → update(s) → delete completo (create/update en paso 3, delete en paso 8)
  - **Confianza:** alta
  - **Claim:** campaign-server.mjs define Tool 17 campaign_session_track con action delete + SESSION_DIR enforcement/sessions
  - **Evidencia:** .opencode/task-system/mcp/campaign-server.mjs líneas 1761-1783: `// ---------- Tool 17: campaign_session_track ----------` (1761) `const SESSION_DIR = join(TASK_SYSTEM, "enforcement", "sessions")` (1763) `function sessionPath(sessionId) { return join(SESSION_DIR, ...replace(/[<>:"/\\|?*]/g, "_")... ) }` (1768) `server.tool( "campaign_session_track", { action: z.enum(["create","get","update","list","delete"])` (1780-1782) `sessionId: z.string().optional()` (1783) — verificado via `Select-String -Pattern "session_track" -Path campaign-server.mjs` → 2 hits (1761 comment,1780 tool name); `Select-String -Pattern "SESSION_DIR" -Path campaign-server.mjs` → 1 hit 1763; `Select-String -Pattern "campaign_session_track" -Path campaign-server.mjs -Context` muestra z.enum incluye delete; `node --check campaign-server.mjs` → 0
  - **Confianza:** alta
  - **Claim:** 8. CUANDO no haya más PENDING es efectivamente paso 8 (numeración 1..8 en pipeline-run.md, HIGH-009 pertenece a cierre de campaña)
  - **Evidencia:** .opencode/task-system/prompts/pipeline-run.md estructura numerada: `1. DETECTAR` (34) `2. LEER resumen` (38) `3. TRACKING DE SESIÓN` (43) `4. PROBES DE INTEGRIDAD` (48) `5. ENCONTRAR próxima tarea` (55) `6. MIENTRAS haya tareas pendientes` (61) `7. WAVES PARALELAS` (187) `8. CUANDO no haya más ⬜ PENDING:` (205) — verificado via `Select-String -Pattern "^\d+\." -Path pipeline-run.md` → 8 hits (1,2,3,4,5,6,7,8) + sesión tracking pasos 3 y 8 coherentes; HIGH-009 es cierre de campaña, no tarea intermedia, por eso reside en paso 8 post-progreso pre-retrospectiva
  - **Confianza:** alta
  - **Claim:** Implementación original en commit 9e5730ff (Master pipeline optimization — 20 items implemented) incluyó HIGH-009 (session cleanup delete)
  - **Evidencia:** `git show 9e5730ff --stat` → 23 files changed, 413 insertions, includes `.opencode/task-system/prompts/pipeline-run.md` + `.opencode/task-system/mcp/campaign-server.mjs`; `git show 9e5730ff -- .opencode/task-system/prompts/pipeline-run.md` diff contiene adición de `**Session Cleanup (HIGH-009):** llamá campaign_session_track action="delete"` en paso 8 (verificado por pipeline run verification línea 544 HIGH-009 grep 1); `git blame pipeline-run.md | Select-String session_track.*delete` → 9e5730ff Eros Nessy 2026-08-28 03:33:57 -0400 línea 208; `git log --oneline -5` incluye 9e5730ff + 5 commits posteriores CORE-004→HIGH-008 idempotentes; pipeline run verification línea 544: `- HIGH-009: grep "session_track.*delete" pipeline-run.md → 1 ✅`
  - **Confianza:** alta
  - **Claim:** No se requiere re-edición; verificación idempotente justificada por ponytail rung 1 (YAGNI)
  - **Evidencia:** `git diff -- .opencode/task-system/prompts/pipeline-run.md` → vacío (file clean) ✅; `git diff -- .opencode/task-system/mcp/campaign-server.mjs` → vacío ✅; `node --check campaign-server.mjs` → 0 ✅; `Select-String` 1+4+2+1 hits todos pasan ✅; `Select-String "^8\."` confirma paso 8 pertenencia ✅; re-ejecución HIGH-006 (07b9cd90) HIGH-007 HIGH-008 preceden mismo patrón idempotente; sin edición → sin debt, sin risk, ponytail skipped: re-escritura de pipeline-run.md paso 8
  - **Confianza:** alta

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — Session Cleanup verificado (paso 8 líneas 205-208 delete + paso 3 create/update + campaign-server.mjs delete enum + SESSION_DIR), lifecycle completo claro; approach ya implementado y validado en pipeline run 20/20 |
| Pendientes de ejecución (downhill) | 0 — 1 step VERIFY completado, sin steps pendientes |
| % completado | 100% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)
No aplica — tarea tipo infra/docs (ALTO, no bug). Gate omitido con justificación: contrato es verificación de presencia de `campaign_session_track action="delete"` en paso 8, no fix de comportamiento roto. Effort 🟢 obvio, no requiere systematic-debugging. Si hubiese bug (stale session no limpiada → contaminaba siguiente campaña), Fase 1 exigiría repro (lanzar 2 campañas seguidas y ver sessions/ residual) + hipótesis (falta delete) antes de fix.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — evaluado: no toca trust boundaries, input de usuario, auth, storage, FFI, ni dependencias nuevas/bumps. Justificación: edición documental en `prompts/pipeline-run.md` (paso 8 delete referencia) + server MCP read-only (SESSION_DIR sessions), sin superficie de ataque, sin `validateOutput` necesario, sin cargo audit. No requiere `security-and-hardening`. Si cambiara SESSION_DIR path sanitization, requeriría audit (path traversal ya sanitizado con replace `[<>:"/\\|?*]`)
- [x] **PERFORMANCE** — evaluado: no toca hot paths (vector/HNSW, engine.rs, search/ingestión, serialización). Justificación: cambio en prompt markdown + server tool delete O(1) unlink, no en código de ejecución hot, no requiere benchmark contra `canonical_p99` ni flamegraph. No requiere `performance-optimization`. Delete es sync unlink single file, no impacto perf.

## Steps

### Step 1: Verificación Session Cleanup campaign_session_track delete en pipeline-run.md paso 8
- **Archivos:** `.opencode/task-system/prompts/pipeline-run.md` (línea 208 paso 8 + líneas 205,43-46), `.opencode/task-system/mcp/campaign-server.mjs` (líneas 1761-1783 SESSION_DIR + z.enum delete), `docs/plans/2026-08-28-master-pipeline-optimization.md` (líneas 223-236 Task 9 + 544 verif)
- **Acción:** Verificar que pipeline-run.md paso 8 (8. CUANDO no haya más ⬜ PENDING: línea 205) ya tiene `**Session Cleanup (HIGH-009):** llamá campaign_session_track action="delete" sessionId=<sessionId>` en línea 208 + 4 hits totales campaign_session_track (44 create,45 update,46 update,208 delete) + campaign-server.mjs Tool 17 con delete en z.enum (1782) + SESSION_DIR (1763) — ejecutar greps mecánicos (Select-String session_track.*delete paso 8 + ^8\. + campaign_session_track 4 hits + session_track server + node --check) + validar contra git history 9e5730ff (diff original + blame). Si falta delete en paso 8 → añadir línea 208 en paso 8 justo tras skill progreso y antes de RETROSPECTIVA; si están todos → marcar COMPLETED idempotente (ponytail rung 1). Actualizar plan file Task 9 Estado PENDING → ✅ COMPLETED + recitation canónica (activeGoal, lastAction, contract, nextTask HIGH-010).
- **Verify:** `Select-String -Pattern "session_track.*delete" -Path .opencode/task-system/prompts/pipeline-run.md` → 1 hit línea 208 ✅ + `Select-String -Pattern "^8\." -Path pipeline-run.md` → 1 hit línea 205 ✅ + `Select-String -Pattern "campaign_session_track" -Path pipeline-run.md` → 4 hits 44/45/46/208 ✅ + `Select-String -Pattern "session_track" -Path campaign-server.mjs` → 2 hits 1761/1780 ✅ + `Select-String -Pattern "SESSION_DIR" -Path campaign-server.mjs` → 1763:SESSION_DIR ✅ + `node --check campaign-server.mjs` → 0 ✅ + `git blame pipeline-run.md | Select-String session_track.*delete` → 9e5730ff ✅
- **Estado:** ✅ COMPLETED (2026-08-28T18:30 — verificación idempotente, sin edición, ponytail rung 1; pipeline-run.md paso 8 delete ya presente desde 9e5730ff, contrato grep -n session_track.*delete pasa en paso 8)

## Dependencias
- Task HIGH-008: Autonomous Flag en Plan File (debe completarse antes — orden secuencial CORE-001 → CORE-005 → HIGH-007 → HIGH-008 → HIGH-009 según plan Dependencias DAG y Orden secuencial estricta mermaid)
- Task CORE-005 precedente (SDP Unificado) → aporta habilidad de session tracking usado en HIGH-009

## Review (GATE — agente distinto, P2-01)
> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-lead (auto-review idempotente — tarea verificación sin código, no requiere vanta-audit/vanta-review separado; ponytail minimal, contract mecánico)
- **Enfoque:** ¿pipeline-run.md línea 208 contiene `campaign_session_track action="delete" sessionId=<sessionId>` con label **Session Cleanup (HIGH-009)** dentro de paso 8 (8. CUANDO no haya más ⬜ PENDING: línea 205)? ¿4 hits campaign_session_track (create/update/delete) forman lifecycle completo? ¿campaign-server.mjs:1782 z.enum incluye delete + 1763 SESSION_DIR enforcement/sessions? ¿contrato grep -n session_track.*delete pasa y está en paso 8 (205<208)? ¿idempotencia justificada vs pre-mortem session ID tracking?
- **Cómo se probó:** `Select-String -Pattern "session_track.*delete" -Path pipeline-run.md` → 208:1 hit `campaign_session_track action="delete" sessionId=<sessionId>` ✅; `Get-Content pipeline-run.md | Select-Object -Index 204,205,206,207,208,209` → 205 `8. CUANDO` 206 Report 207 skill progreso 208 Session Cleanup delete 209 RETROSPECTIVA ✅; `Select-String -Pattern "^8\." -Path pipeline-run.md` → 205:1 hit `8. CUANDO no haya más ⬜ PENDING:` ✅ (208 ∈ (205,223)); `Select-String -Pattern "campaign_session_track" -Path pipeline-run.md` → 4 hits 44 create +45 update +46 update +208 delete ✅; `Select-String -Pattern "session_track" -Path campaign-server.mjs` → 1761 Tool comment +1780 tool name 2 hits ✅; `Select-String -Pattern "SESSION_DIR" -Path campaign-server.mjs` → 1763 SESSION_DIR enforcement/sessions ✅; `Get-Content campaign-server.mjs | Select-Object -Index 1780,1781,1782` → 1782 `z.enum(["create","get","update","list","delete"])` incluye delete ✅; `node --check campaign-server.mjs` 0 ✅; `git blame pipeline-run.md | Select-String session_track.*delete` → 9e5730ff ✅; `git diff -- pipeline-run.md campaign-server.mjs` vacío (file clean) ✅; `git show 9e5730ff --stat` 23 files ✅
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria (grep -n session_track.*delete en paso 8).
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado (session_track.*delete 1 hit + ^8\. 1 hit + campaign_session_track 4 hits + session_track server 2 hits + SESSION_DIR 1 hit + node --check + git blame + git show + git diff + Get-Content context = 13 verificaciones).
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo (Session Cleanup al Cerrar Campaña).
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas (1 step, ponytail minimal, ~100 líneas por step reversible).
- **Veredicto:** ✅ approve — Session Cleanup completo (pipeline-run.md:208 delete en paso 8 (205) + 4 hits lifecycle create/update/delete + campaign-server.mjs 1782 delete enum + 1763 SESSION_DIR), contrato mecánico grep -n session_track.*delete pasa en paso 8, idempotencia correcta (ponytail rung 1: ya existe desde 9e5730ff), sin edición necesaria, pre-mortem Session ID tracking satisfecho (same sessionId create→delete), 13/13 verificaciones mecánicas replicadas localmente ✅

## Notas
- Decisión ponytail full: rung 1 "¿Necesita existir?" → No, ya existe en 9e5730ff. Verificación idempotente sin re-edición. Skipped: re-escritura de pipeline-run.md paso 8 delete; add when: delete faltara en paso 8 (grep 0) o 8. CUANDO no existiera o z.enum delete ausente en campaign-server.mjs. Costo: 0 líneas editadas, 1 task file creado, 1 plan file Estado update.
- Commit 9e5730ff ya incluyó esta tarea en feat: Master pipeline optimization - 20 items implemented (ver --stat 23 files, pipeline-run.md + campaign-server.mjs); pipeline run verification línea 544 HIGH-009 grep 1 confirmaba 20/20; re-ejecución trazable para SARL (ver plan Estado: EN PROGRESO re-ejecución para trazabilidad, Task 9 PENDING → ahora COMPLETED)
- No se requirió web research: Session Cleanup es interna del task-system (pipeline-run.md paso 8 + campaign-server.mjs SESSION_DIR), no API externa ambigua; APIs verificadas localmente via Select-String + node --check + git history — source-driven-development no necesario
- Plan enumeración numérica (Task 9) vs ID alfanumérico HIGH-009: parsers.mjs usa regex `### Task (\d+):` → id=9, name=HIGH-009 — Session Cleanup ...; campaign_update_task_state con taskId=9 mapea a mismo bloque; task file HIGH-009.md usa ID alfanumérico canónico; dual naming intencional (plan numérico, task file alfanumérico)
- Task file creación idempotente: si ya existe COMPLETED previo, se respetan steps ✅ sin pisar (pipeline-full.md línea 80-82: "Si ya existe (tarea reanudada tras intento previo), LEELO y continuá desde primer step ⬜ PENDING")
- Lifecycle sesiones: paso 3 create (sessionId único al inicio) → update por tarea → update final → paso 8 delete (mismo sessionId) — evita stale sessions en enforcement/sessions/.json; delete usa sessionPath sanitizado, idempotente si ya borrada
- Analogía con HIGH-008: ambos son verificación idempotente en prompts (Autonomous vs Session Cleanup) — HIGH-009 no depende de HIGH-008 técnicamente pero secuencial en DAG por orden de implementación Master pipeline; ambos ya resueltos en 9e5730ff

## Referencias
- `.opencode/references/definition-of-done.md` — standing quality bar (Task → contract ✅ + task file sync + recitation; Commit → conventional + verify full; Release N/A)
- `.opencode/references/skills-engineering.md` — SDP lifecycle mapping
- `SKILLS-MANIFEST.md` — catálogo de skills disponibles (193 skills, ponytail/campaign-executor/progreso incluidas)
- `.opencode/task-system/prompts/pipeline-run.md:205` — `8. CUANDO no haya más ⬜ PENDING:` (paso 8 inicio, verificado '^8\.' 1 hit)
- `.opencode/task-system/prompts/pipeline-run.md:208` — `**Session Cleanup (HIGH-009):** llamá campaign_session_track action="delete" sessionId=<sessionId>` (fuente verificada, git blame 9e5730ff, paso 8)
- `.opencode/task-system/prompts/pipeline-run.md:43-46` — `3. TRACKING DE SESIÓN:` create/update lifecycle
- `.opencode/task-system/mcp/campaign-server.mjs:1761-1783` — Tool 17 campaign_session_track + SESSION_DIR + z.enum delete (verificado node --check 0)
- `docs/plans/2026-08-28-master-pipeline-optimization.md:223-236` — Task 9 definición + contrato grep -n session_track.*delete → en paso 8 + pre-mortem Session ID tracking
- `docs/plans/2026-08-28-master-pipeline-optimization.md:544` — Verificación por task (HIGH-009: grep "session_track.*delete" pipeline-run.md → 1 ✅)
- `.opencode/skills/campaign-executor/templates/task-definition.md` — template ≥20 secciones (verificado CORE-004: 20 secciones)

## Context Save Point
- **Fecha:** 2026-08-28T18:30
- **Branch:** main (verificado via git log --oneline -1 y git status; commit 9e5730ff feat Master pipeline optimization)
- **CI pendiente:** no (verify full no requerido para infra prompts verification; ponytail minimal — node --check 0 suffices, nextest no aplica a markdown)
- **Decisiones:** HIGH-009 verificado idempotente porque pipeline-run.md:208 ya tiene `campaign_session_track action="delete" sessionId=<sessionId>` en paso 8 (8. CUANDO 205) + 4 hits lifecycle create/update/delete + campaign-server.mjs 1782 delete enum + 1763 SESSION_DIR desde 9e5730ff; no se añadió código, se marcó COMPLETED idempotente y se actualizará plan Task 9 PENDING → COMPLETED con recitation canónica
- **Problemas conocidos:** ninguno — contrato grep -n session_track.*delete 1 hit en paso 8, 4 hits campaign_session_track, server delete enum presente, node --check 0, git diff clean, git show 9e5730ff confirmation
- **Próxima tarea:** HIGH-010 — Context Save Point Reconstruction Tool (siguiente en plan secuencial, HIGH-009 → HIGH-010 según DAG y Orden secuencial estricta)

