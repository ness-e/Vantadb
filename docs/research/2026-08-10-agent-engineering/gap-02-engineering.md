# GAP-02: Sistema de Tareas Automático vs Buenas Prácticas de Ingeniería

> **Fecha:** 2026-08-10
> **Alcance:** Sistema de tareas automático de VantaDB (`.opencode/`) vs prácticas de ingeniería de software (end-to-end), pensamiento de sistemas y entrega de proyectos.
> **Baselines:** `eng-01-software.md`, `eng-02-systems.md`, `eng-03-project.md`
> **Archivos analizados:** AGENTS.md, VANTADB-OPERATING-MANUAL.md, RULES.md, prompts (iter-loop-tools, pipeline-full, task.md, plan.md, research-agent), commands/pipeline.md, 6 skills de ingeniería, task real DRV-131.

---

## 1. Tabla Maestra — Práctica | Estado actual | GAP | Acción propuesta

| Práctica | Estado actual | GAP | Acción propuesta |
|---|---|---|---|
| Debug sistemático completo | ✅ PARCIAL — RULES.md §10 mapea Bug→`systematic-debugging`; plan.md Paso 0 exige repro root-cause; skill completo (4 fases, Iron Law) | Solo es obligatorio para bugs *reportados*; defs de debug por tests que fallan en VERIFY dependen de que el agente cargue la skill | Añadir a task.md/pipeline-full una evidencia de Phase 1 (repro) como campo del contrato para cualquier fix |
| Análisis de problemas complejos (Cynefin, tradeoffs, pre-mortem, decision matrix) | ❌ NO — no se menciona Cynefin, pre-mortem ni decision matrix en pipeline | El system no clasifica complejidad del problema, solo tipo de tarea | Gate de triaje: tareas 🔴 usan Cynefin + pre-mortem breve antes del task file |
| Investigación técnica | ✅ PARCIAL — `research-agent.md` define rol con digest ≤500 palabras y formato fijo (Hallazgos/Estructura/Riesgos/Referencias) | Solo aplicable al sub-agente de research; no es obligatoria para decisiones de diseño no-obvias | Exigir research para decisiones de arquitectura (fuente: eng-01/eng-03) |
| DoD por niveles con acceptance criteria | ✅ PARCIAL — `task.md` tiene tabla tipo→checks; DRV-131 tiene Acceptance Criteria + NO list + Verification | No hay DoD declarado a nivel *cambio* (un commit) ni a nivel *release* como estándar único | Formalizar `references/definition-of-done.md` como contrato multi-nivel y referenciarlo en task.md |
| Quality gates engineering (mutation/coverage/security/performance) | ✅ PARCIAL — checks mecánicos por tipo en task.md; CI two-tier; verify.ps1 (fmt/clippy/test/deny/audit) | No hay gate de *coverage* mínimo ni mutation testing; security/performance no son fase explícita en el pipeline | P2: agregar `cargo llvm-cov --fail-under` en CI heavy; fase SECURITY/PERF en pipeline-full |
| Refactoring seguro (Fowler: 2 sombreros) | ❌ NO — existe `code-simplification` y `deprecation-and-migration`, pero no se documenta el patrón 2-sombreros (no mezclar refactor con behavior) | Refactors sin pruebas previas de comportamiento | Documentar en code-simplification; contrato: `git diff` sin cambios de lógica en commit de refactor |
| Code review (segunda opinión) | ❌ PARCIAL débil — state REVIEW existe; `code-review-and-quality` (5 ejes); pero es *self-review*: el mismo agente revisa su trabajo | Sin verificador en contexto fresco; `doubt-driven-development` solo para high stakes | Gate opcional: review cross por sub-agente para tareas 🔴 |
| Documentación viva (ADR, design docs, doc-as-code) | ✅ PARCIAL — Regla 5 (ADR + `campaign_memory_write(decisions)`); doc-driven development público | No enforcement automático de "docs con el PR" (Regla 3 es manual) | Añadir check en pipeline-full: si toca API pública → ADR/docs obligatorios en el commit |
| Estimación/riesgo/dependencias en plan/task | ✅ PARCIAL — esfuerzo 🟢🟡🔴, prioridad, gate 🔴 BLOQUEADO por dependencia en plan.md | Estimación discreta sin calibración (sin compare actual vs estimado); sin riesgo (probabilidad × impacto) formal | Log de effort real vs estimado en plan file para calibrar |
| Contradicciones internas en el pipeline | ⚠️ SÍ — ver §5 | Doble skill de debug; conteos de skills inconsistentes | Unificar skill canónica; auditar SKILLS-MANIFEST vs disco |

---

## 2. Prácticas que YA captura el sistema (fortalezas)

1. **Debug sistemático en bugs reportados** — `systematic-debugging` es un skill completo (`.agents/skills/systematic-debugging/SKILL.md`, 296 líneas): Iron Law ("NO FIXES WITHOUT ROOT CAUSE"), 4 fases (Root Cause → Pattern → Hypothesis → Implementation), redis de red flags, tabla de racionalizaciones y la regla de "3+ fixes fallidos → cuestionar la arquitectura" (Phase 4.5). Se referencia en RULES.md §10, plan.md (Paso 0, tareas bug), iter-loop-tools.md, pipeline-full.md, audit.md, unified-review. Es la práctica de debug más fuerte disponible. Complementa a `debugging-and-error-recovery` (`.opencode/skills/`) con bucle STOP→PRESERVE→DIAGNOSE→FIX→GUARD→RESUME y `docs/references/bug-workflow.md` para el flujo de reporte.

2. **Contrato booleano verificable** — invariante central de RULES.md: cada tarea tiene un contrato que se valida con comando mecánico (MCP `campaign_verify_cmd` con expected exit code y timeout) en vez de auto-reporte. Es la práctica más robusta del system: separa la palabra del agente de la evidencia del shell.

3. **Triaje con verificación de realidad (plan.md Paso 0)** — antes de incluir una tarea en el plan se verifica contra el código real con `codegraph_explore`: existe el símbolo/archivo, el gap de comportamiento persiste, qué llama y grafica el blast radius. Gate DO/DEFER/SKIP/BLOQUEADO con justificación escrita en el plan file (evidencia + gate justification). Elimina tareas stale (el ejemplo explícito del prompt: "tarea X menciona código renombrado → SKIP").
   - Tabla de decisión por evidencia: referencias existen + gap real + cambio acotado → ✅ DO; gap ambiguo → 🟡 DEFER; no existe / ya implementado → ❌ SKIP; depende de tarea no lista → 🔴 BLOQUEADO.

4. **Acceptance Criteria + Criterios de NO + Verification** — DRV-131.md (task real COMPLETED) incluye: checklist de aceptación invocable ("IndexType enum existe y HnswConfig tiene index_type field"), bloques de comandos de verificación (`cargo check`, `cargo test -p vantadb -- ivf`, `cargo clippy -D warnings`), criterios de NO (no tocar `flat.rs`, no deps externas de clustering, no PQ, no bindings Python/TS/WASM). Es el patrón de task file correcto y lo que toda tarea debería heredar.

5. **Checks mecánicos por tipo de tarea** — task.md mapea tipo→skill→comando: Rust (`cargo check`, `nextest`, `fmt`, `clippy -D warnings`), Frontend (`npx tsc --noEmit`, `npm run lint`), Python SDK (`pytest -v`), TS SDK (`npx tsc`, `npm test`), Docs (`scripts/validate-docs-coverage`). Con esto el DoD por *tipo de archivo* está cubierto.

6. **CI two-tier + pre-flight local** — Fast Gate (<5 min, determinista, offline: fmt, clippy, unit + integration rápido) vs Heavy Certification (manual/scheduled, hasta 2h: stress_protocol, SIFT, competitive_bench, chaos_integrity, wal_resilience). `verify.ps1` y `verify_changed.ps1` para el dev. Stack de dev tools instalado: nextest, sccache (CI), deny, audit, bloat, outdated, machete, release-plz, git-cliff.

7. **Investigación técnica con formato fijo** — research-agent.md (28 líneas) define el rol de investigación, digest ≤500 palabras, y formato obligatorio (Hallazgos clave / Estructura / Riesgos / Referencias con line numbers). Determina exactamente qué es "investigar bien" sin ambigüedad.

8. **Documentación de decisiones** — Regla 5: ADR en `docs/architecture/adr/` (plantilla `docs/_templates/adr.md`) o memoria del agente (`campaign_memory_write(file="decisions")`). Doc-Driven Development ("docs primero, nunca docs detrás del código"). Esto captura la práctica de eng-03 de memoria de decisiones.

9. **Convenciones de commits y release** — Conventional Commits obligatorios, release-plz automatiza bump/changelog/tag/publish, main solo releases (PR desde develop), 0 commits directos a main, "nunca tocar versión/changelog/tag manualmente". Regla 6: límite de deuda técnica por PR (saldo neto cero o negativo con monedas P2 conocidas).

10. **Skills de ingeniería instaladas** — 32 skills en `.opencode/skills/` cubren todo el lifecycle, con tabla explícita de factory mapping (ADDYOSMANI agent-skills): DEFINE (spec-driven-development, interview-me, idea-refine), PLAN (planning-and-task-breakdown), BUILD (incremental-implementation, TDD, context-engineering, source-driven-development, doubt-driven-development, frontend-ui-engineering, api-and-interface-design), VERIFY (debugging-and-error-recovery, browser-testing-with-devtools), REVIEW (code-review-and-quality, code-simplification, security-and-hardening, performance-optimization), SHIP (git-workflow-and-versioning, ci-cd-and-automation, shipping-and-launch, documentation-and-adrs, deprecation-and-migration, observability-and-instrumentation), META (using-agent-skills). Con tabla anti-racionalización (prohíbe "muy chico para skill", "después los tests").

11. **Ritual de inicio/fin de sesión** — cargar `progreso`, revisar `git status`, `git log`; al finalizar `progreso` migra completadas a `docs/progreso/` y elimina la fila de Backlog.md. Cierra el loop de tracking.

12. **State machine C0 con enforcement** — `config/state-tools.mjs` valida herramientas por estado (PLAN/ACT/VERIFY/COLLATERAL/RESEARCH/EVALUATE/REVIEW/ACCEPT/CLOSE/STALL) y el pipeline ejecuta estados en orden: /pipeline plan|task|run|interactive|execution. Esto es más estricto que el medio del mercado e implementa el "verificador separado" que recomienda eng-03 (incluso si es el mismo agente, el estado obliga a declarar).

---

## 3. Prácticas FALTANTES (gaps totales)

1. **Cynefin-like clasificación de complejidad** — el system clasifica por *tipo de tarea* (bug/feature/refactor/research) pero nunca por *dominio de complejidad* (obvio/complicado/complejo/caótico). Una feature compleja (muchas dependencias, tradeoffs) y una trivial reciben el mismo workflow. eng-02 insiste en que la estrategia de resolución debe depender de la naturaleza del problema, no del etiquetado.
   - Señal de humo: el task file solo declara Esfuerzo 🟢🟡🔴 y Prioridad, no "por qué es complicado".

2. **Pre-mortem / trade-off / decision matrix como paso del pipeline** — no hay paso obligatorio "¿qué puede salir mal? / ¿qué alternativas hay? / ¿con qué criterio elijo A sobre B?". La decisión de DRV-131 (Opción A vs B vs C para IVF) se documenta en el propio task, pero por disciplina del autor, no porque el formato lo exija.
   - Lo más cercano es `campaign_memory_write(decisions)` y la Regla 5 (ADR), que exigen el *resultado* de la decisión, no el *proceso* de elegirla.

3. **Coverage mínimo como gate** — la suite es robusta (DRV-131: 1547 tests lib, 16 tests IVF, 0 clippy warnings) pero no hay umbral de cobertura mecanizado en CI (`--fail-under`), ni mutation testing (cargo-mutants), ni tracking de cobertura a lo largo del tiempo. Un PR puede bajar la cobertura de un módulo caliente sin que nada falle.

4. **Security/Performance como fase explícita en el pipeline** — existen las skills (`security-and-hardening`, `performance-optimization`) y pre-flight `cargo audit` + `deny`, pero:
   - No son paso mandatorio en pipeline-full (dependen de "REVIEW si corresponde").
   - No hay gate de security por cambio (Ej: `cargo audit` solo corre en verify, no por-PR si el cambio toca deps).
   - Performance no se mide contra baseline salvo benchmarks Heavy (Criterion en `benches/`).

5. **Definición-de-hecho multi-nivel** — `references/definition-of-done.md` existe en AGENTS.md como referencia, pero no está referenciada como contrato en task.md, ni aplica como gate separado por nivel (cambio / task / release). El DoD por tipo está cubierto (tabla de task.md) pero no el DoD por *nivel* de entrega.

6. **Métricas de flujo (DORA)** — eng-03 recomienda flow metrics (cycle time, lead time, change failure rate, deployment frequency). El system no las instrumenta: los plan files registran estado y iteraciones pero no fechas de inicio/fin estructuradas, por lo que no hay manera automática de calcular cycle time ni CFR.

7. **Segunda opinión / revisión en contexto fresco** — todo el REVIEW (state REVIEW, code-review-and-quality 5 ejes) lo ejecuta el mismo contexto que implementó. No hay patrón de "verificador" (un segundo sub-proceso con contexto limpio) salvo `doubt-driven-development` solo para high stakes (Regla: adversarial review en contexto fresco). eng-03 marca la revisión independiente como gate de calidad.

8. **Refactoring 2-sombreros explícito (Fowler)** — `code-simplification` y `deprecation-and-migration` existen, pero ninguna documenta la separación "cambio de comportamiento" vs "cambio estructural" en commits distintos con tests al descubierto entre ambos sombreros. Un refactor mezclado con un fix en un mismo commit es el anti-patrón clásico.

9. **Rollback automatizado a nivel feature** — existe `/rollback` y shipping-and-launch con plan de rollback, pero el rollback es por comando manual al ship, no un mecanismo de "green/blue" o "flag" por feature. No hay feature flags en el pipeline (la skill ci-cd recomienda flags).

---

## 4. Prácticas PARCIALES (implementadas pero incompletas)

| Práctica | Qué hay | Qué falta |
|---|---|---|
| Debug sistemático | Skill completo + wiring en bugs (plan.md Paso 0, iter-loop-tools, pipeline-full) | El gate del contrato mide *resultado* (comando pasa), no *proceso* (¿hubo repro + hipótesis antes del fix?). Un bug "arreglado" por prueba y error pasa el contrato igual. La doble skill de debug (ver §5.1) diluye cuál es el canon. "Root-cause first" de systematic-debugging no está en el contract de la verificación |
| DoD / acceptance criteria | Por tarea (DRV-131) y por tipo (task.md) | No declarado como estándar único multi-nivel; la certificación heavy es "manual/scheduled"; no hay vinculación automática entre acceptance criteria checkbox y CI |
| Code review | 5 ejes en skill + state REVIEW | Self-review del mismo agente; sin gate de verificación independiente; el APPROVE lo emite quien implementó |
| Investigación / source-driven | research-agent.md + source-driven-development (DETECT→FETCH→IMPLEMENT→CITE) | No es obligatoria para decisiones de diseño no-obvias; "IF en duda" no es un trigger estricto; CITE se recomienda pero no se verifica la cita |
| Estimación | Esfuerzo 🟢🟡🔴 + Prioridad + gate 🔴 BLOQUEADO por dependencia (plan.md) | Sin calibración (esfuerzo real vs estimado no se registra al cerrar); sin riesgo (probabilidad × impacto); la escala 3 niveles oculta el rango interno de un 🟡 (1h-3d) |
| Documentación viva | ADR + memoria de decisiones + doc-driven | Regla 3 es manual ("recordar" en vez de gate); no hay check en pipeline-full que falle si tocas API pública sin docs; la tabla de skills del manual desincroniza (ver §5.3) |
| Verificación pre-push | docs + `dev-tools/verify.ps1` + `verify_changed.ps1` | Hooks git NO instalados (AGENTS.md lo admite) → la verificación pre-push es manual por disciplina del agente; `--no-verify` queda abierto; riesgo real de regresión silenciosa |
| Refactoring | skills de code-simplification y deprecation | Sin patrón 2-sombreros; no se separa comportamiento de estructura en commits |
| Testing | TDD + nextest + CI two-tier + cargo-fuzz (nightly) | Mutation testing ausente; coverage sin umbral; flaky tests se aislean con Issue pero no hay montior de flakiness rate |

---

## 5. Prácticas MAL implementadas o contradictorias

1. **Doble fuente de verdad para debugging** — `systematic-debugging` (`.agents/skills/`, 296 líneas, 4 fases con Iron Law) y `debugging-and-error-recovery` (`.opencode/skills/`, bucle STOP→PRESERVE→DIAGNOSE→FIX→GUARD→RESUME + Triage Checklist con "Step 1: Reproduce") coexisten sin relación declarada. AGENTS.md: la guía de diseño (diseño/creativo) dice "Corrección de bugs: systematic-debugging → writing-plans", mientras la tabla de lifecycle VERIFY/ingeniería usa `debugging-and-error-recovery`. 21 referencias reparten entre ambos nombres. Consecuencia: un agente puede cargar una y desconocer los pasos de la otra.
   - Costo demostrado: al localizar los prompts, las referencias a `systematic-debugging` llevan a la copia `.agents/` mientras `plan.md` resuelve skills a `.opencode/skills/<nombre>/` — ambigüedad de contexto real para el agente ejecutor.

2. **Path resolution incompleto** — AGENTS.md resuelve `tasks/<ID>.md` → `.opencode/skills/campaign-executor/tasks/<ID>.md` y `skills/X` → `.opencode/skills/X/`. Hay dos casos reales no documentados encontrados en esta sesión:
   - Tasks completadas viven en `tasks/complete/` (DRV-131 está AHÍ, NO en la raíz `tasks/`). La consulta `tasks/DRV-131.md` falla en el primer intento; además hay `tasks/closed/`. La tabla de Path Resolution no documenta estos subdirectorios (solo "grupo dentro de tasks").
   - `systematic-debugging` vive en `.agents/skills/` (carpeta de proyecto), fuera del path que declararía plan.md si se resolviera mecánicamente.

3. **Conteos de skills inconsistentes** — SKILLS-MANIFEST.md dice 104 skills; AGENTS.md dice "82 `.agents/skills/` + 32 `.opencode/skills/` = 114" en una sección y "104" en otra; el disco (esta sesión) tiene 29 dirs en `.opencode/skills/` + sub-skills de ponytail. No hay fuente única autoritativa contando el disco. Dificulta auditoría de cobertura y el "0 falsos positivos" de North Star.

4. **North Star sin métricas mecanizadas** — RULES.md declara "tasa de completado >90%, 0 falsos positivos, 0 regresión silenciosa" como invariantes, pero no hay instrumentación que los mida (los plan files podrían, pero no se registran fechas/estados normalizados). Un invariante no medible es un slogan; siguen siendo aspiracionales hasta que exista un `/status` que los derive de datos.

5. **"Verificación mecánica, nunca auto-reporte" vs verificación delegada al ejecutor** — el invariante de RULES.md es correcto, pero en iter-loop-tools el agente ejecuta `campaign_verify_cmd` sobre su propio resultado; la mayoría de contratos son comandos (bien), pero los de tipo "docs están actualizados" o "sin warnings" terminan siendo auto-informados por un comando sin salida estricta. El estado VERIFY lo ejecuta el mismo agente que ACTuó.

---

## 6. Fallas operativas reales (evidencias de esta sesión)

1. **DRV-131.md no estaba donde Path Resolution dicta** — al resolver `tasks/DRV-131.md` según AGENTS.md falló: la task vive en `tasks/complete/` (archivada, COMPLETED, absorbida por COMP-027 Multiple Index Types). Se encontró vía `Get-ChildItem`. El patrón de archivo (complete/ y closed/) no está documentado en la tabla de resolución → operación con ruta correcta que falla el primer intento.

2. **Glob tool devolvió "No files found" para patrones válidos** — `**/DRV-1*.md` y `.opencode/skills/*/SKILL.md` devolvieron vacío en el entorno (Windows). Se resolvió con bash `Get-ChildItem` + `Test-Path` + Read de directorios. Fricción real del harness de un solo dev; vale la pena documentar un workaround en el manual para que el agente no pierda tiempo.

3. **Hooks git no instalados** — AGENTS.md lo admite explícitamente: "Los hooks git NO están instalados. La verificación previa a push es manual (Regla 1)". El pre-push barrier existe como template (`templates/pre-push.ps1`) pero no está activado. Es el gate más crítico (Regla 1 / Regla 7 pre-push) y el más fácil de saltar con prisa o `--no-verify`. Regla 1 prohibe `--no-verify`, pero sin hook no hay enforcement.

4. **Manual de operación largo y con drift** — VANTADB-OPERATING-MANUAL.md (917 líneas, 14 secciones) y AGENTS.md (530+ líneas) duplican el mismo conocimiento (skills, MCP, verify, flujo de release) con conteos y nombres en ligera desincronización (punto §5.3). El lookup de skills correcto requiere consultar AGENTS.md + el manual + el manifest a la vez; cada fuente es lexicográficamente distinta.

5. **Ruta de script contradictoria** — `verify_changed.ps1` se declara en `dev-tools/`, y la tabla de dev-tools lo referencia, pero la tabla CI/Hooks del propio AGENTS.md también habla de `scripts/validate-docs-coverage` para docs sin confirmar que ambos caminos sean co-consistentes (en scope de sesión no se pudo ejecutar; anotado como área a validar).

---

## 7. Mejoras priorizadas (P0–P3)

### P0 — Urgente (correcto momento: ahora)

| ID | Mejora | Impacto | Esfuerzo | Dónde |
|---|---|---|---|---|
| P0-1 | **Unificar el skill de debugging**: elegir canónico (recomendado: `systematic-debugging` por sus 4 fases + Iron Law; o fusionar ambos) y que el otro delegue o se deprecie. Actualizar las 21 referencias (AGENTS.md, manual, RULES.md, prompts) a UN nombre | Elimina doble fuente de verdad; evita que un fix se haga sin Iron Law | 🟢 ~1-2h | AGENTS.md, RULES.md, prompts/, manual |
| P0-2 | **Documentar los casos no cubiertos de Path Resolution**: `tasks/ID.md` puede estar en `tasks/complete/` o `tasks/closed/`; `skills/X` busca en `.opencode/skills/` Y `.agents/skills/` Y `~/.agents/skills/` | Elimina el fallo operativo §6.1 | 🟢 15 min | AGENTS.md tabla Path Resolution |

### P1 — Alta (próximo sprint de tooling)

| ID | Mejora | Impacto | Esfuerzo | Dónde |
|---|---|---|---|---|
| P1-1 | **Gate de proceso para bug fixes**: para cualquier `fix:`, el contrato del task incluye evidencia de Phase 1 (repro + hipótesis escrita + 1 variable) antes del cambio | Transferir el debug sistemático de "wiring" a "gate de proceso" | 🟡 2-4h | task.md, RULES.md |
| P1-2 | **Instrumentar métricas North Star** (tasa de completado, falsos positivos) leyendo plan files con un script simple; fechas/estados normalizados al crear cada task | Hace medibles los invariantes de RULES.md | 🟡 4h | script + docs |
| P1-3 | **Activar el pre-push barrier** (`templates/pre-push.ps1`) como hook git real, siguiendo instrucción existente del template | Cierra el gate de push que hoy es manual | 🟡 2h | git hooks + template |

### P2 — Media (backlog razonable)

| ID | Mejora | Impacto | Esfuerzo | Dónde |
|---|---|---|---|---|
| P2-1 | **Coverage mínimo mecanizado**: `cargo llvm-cov --fail-under <umbral>` como gate en CI Heavy con umbral inicial prudente + subida gradual | Preventivo de regresión silenciosa; complementa los 0 clippy warnings | 🟡 4-8h | CI_POLICY, verify.ps1 |
| P2-2 | **Security/Performance como fase explícita en pipeline-full** para cambios que tocan trust boundaries o hot paths (no "REVIEW si corresponde") | Gate estructural por cambio | 🟡 2-4h | pipeline-full.md, task.md |
| P2-3 | **Review en segunda instancia** para tareas 🔴/alto riesgo: `doubt-driven-development` como gate obligatorio (revisión adversarial en contexto fresco) | Recupera la segunda opinión del §3.7 | 🟢 1h | task.md, RULES.md |
| P2-4 | **DoD multi-nivel explícito**: referenciar `references/definition-of-done.md` desde task.md y diferenciar DoD por nivel (task / commit / release) en el plan file | Cubre el patch §3.5 | 🟢 1-2h | task.md, plan.md |

### P3 — Baja (mejora continua)

| ID | Mejora | Impacto | Esfuerzo | Dónde |
|---|---|---|---|---|
| P3-1 | **Cynefin + pre-mortem** en el triaje de tareas 🔴/ambiguas: clasificar dominio de complejidad y registrar "top 3 riesgos" en el plan file | Mejora el triaje en problema complejo (eng-02) | 🟢 1-2h | plan.md |
| P3-2 | **Calibración de estimación**: registrar effort real al cerrar la task en el plan file; comparar vs estimado en `/status` | Refina la estimación del harness | 🟢 1h + disciplina | plan.md, status |
| P3-3 | **Refactoring 2-sombreros**: documentar en `code-simplification` + contrato "commit de refactor no cambia comportamiento" en RULES.md | Refactors seguros (Fowler) | 🟢 1-2h | skill, RULES.md |
| P3-4 | **Métricas DORA de flujo** (cycle/lead time, CFR) derivadas de plan files normalizados | Alinea con eng-03 flow metrics | 🟡 4h | script + docs |
| P3-5 | **Auditar conteo de skills** y regenerar SKILLS-MANIFEST desde el disco; unificar cifras en manual/AGENTS.md | Fuente única de verdad | 🟢 1h | SKILLS-MANIFEST, manual |
| P3-6 | **Mutation testing optativo** en CI Heavy (cargo-mutants sobre módulos calientes) | Detecta tests débiles que el coverage no ve | 🟡 8-16h | CI Heavy |

---

## Apéndice A — Método de evaluación y evidencia

El análisis se ejecutó a partir de la definición de cada práctica como un juicio verificable: **"¿el documento declara la práctica Y el pipeline la enforceda mecánicamente?"** La distinción declarado/enforced recorre todo el reporte y separa fortalezas reales de documentos que solo lo dicen.

Evidencia recopilada en esta sesión (order de acción):

1. Lectura de los 3 baselines: `eng-01-software.md` (ciclo de vida end-to-end de feature), `eng-02-systems.md` (pensamiento de sistemas, debugging sistemático, quick-fix vs root-fix, pre-mortem), `eng-03-project.md` (proyecto/entrega: slices delgados, DoD por nivel, gates por nivel, flow metrics).
2. Lectura del sistema: `.opencode/AGENTS.md` (entry points y reglas 0-7), `.opencode/skills/campaign-executor/RULES.md` (North Star + invariantes), los 4 prompts (`iter-loop-tools.md`, `pipeline-full.md`, `task.md`, `plan.md`, `research-agent.md`), `.opencode/commands/pipeline.md`, y los SKILL.md de `spec-driven-development`, `test-driven-development`, `debugging-and-error-recovery`, `code-review-and-quality`, `planning-and-task-breakdown`, `source-driven-development`, `systematic-debugging`.
3. Elección de task real como caso de inspección: `tasks/DRV-131.md` (la resolvían plan.md y task.md como ejemplo de tipo de tarea con skills). El lookup inicial por la ruta del Path Resolution **falló** (ver §6.1) y se localizó en `tasks/complete/DRV-131.md`: IVF index, 836 líneas en `src/index/ivf.rs`, 16 tests, 1547 tests lib. Es la pieza de evidencia "task file bien formado".
4. Ejecución de workarounds por fallas del entorno (glob tool devolvió vacío en patrones válidos → bash `Get-ChildItem` + `Test-Path` + Read de directorios). Documentado como fricción operativa del harness (§6.2).

### Criterio de calificación usado

- ✅ **CAPTURA**: práctica declarada en workflow normativo Y verificable por comando mecánico o field del formato obligatorio.
- ✅ **PARCIAL**: declarada pero con enforcement opcional, o cubierta en un subconjunto de caminos del pipeline.
- ❌ **FALTANTE**: no declarada en ningún documento normativo del pipeline (puede existir como skill instalada pero sin wiring).
- ⚠️ **CONTRADICCIÓN**: dos fuentes del mismo level normativo (AGENTS/manual/prompts) se contradicen entre sí.

## Apéndice B — Cruce explícito con los baselines

### B.1 vs `eng-01-software.md` (ingeniería de software end-to-end)

| Práctica del baseline | Veredicto | Nota de evidencia |
|---|---|---|
| Feature arranca por requisito (no código primero) | ✅ PARCIAL | `spec-driven-development` + task.md fases (spec→plan→tasks→implement) existen; el arranque depende de que el agente respete el orden de pipeline-full |
| Testing como parte del change (no separado) | ✅ CAPTURA | TDD wiring en RULES.md §10 + Contrato booleano + tabla tipo→check en task.md |
| Calidad del código como gate (lint/format/coverage) | ✅ PARCIAL | fmt+clippy+deny en CI/verify; coverage sin umbral (gap §3.3) |
| Documentación mantenida con el code (doc-as-code) | ✅ PARCIAL | doc-driven, ADR, Rule 3; enforcement manual (§4) |
| Feedback loop rápido | ✅ CAPTURA | cargo-watch, verify_changed ~30s, Fast Gate <5 min |
| Refactoring seguro | ❌ | 2-sombreros ausente (§3.8) |
| Release automatizado y semver | ✅ CAPTURA | release-plz + conventional commits + Regla 7 |

### B.2 vs `eng-02-systems.md` (pensamiento de sistemas y debugging)

| Práctica del baseline | Veredicto | Nota de evidencia |
|---|---|---|
| Root-cause antes de fix (quick-fix es deuda) | ✅ CAPTURA (en el skill) | Iron Law de `systematic-debugging` + "Root-cause first" en plan.md Paso 0 |
| Root-cause asumido por el pipeline (no solo el skill) | ❌ | El contrato valida resultado, no método (§4, P1-1) |
| Localizar falla en sistemas multicomponente (evidencia por capas) | ✅ PARCIAL | `systematic-debugging` Phase 1 paso 4 documenta instrumentar por boundary; sin gate |
| Pre-mortem antes de cambio significativo | ❌ | Ausente (§3.2) |
| Cambiar una variable a la vez | ✅ PARCIAL | Skill lo exige (Phase 3); sin enforcement en contrato |
| 3+ fixes fallidos → cuestionar arquitectura | ✅ PARCIAL | Phase 4.5 del skill; sin señal en pipeline-full para escalarlo al agente orquestador |

### B.3 vs `eng-03-project.md` (entrega y proyecto)

| Práctica del baseline | Veredicto | Nota de evidencia |
|---|---|---|
| Shaping del problema antes de planificar | ✅ PARCIAL | plan.md Paso 0 verifica realidad de cada tarea (triaje) — es exactamente shaping del backlog |
| Slices delgados e incrementales | ✅ PARCIAL | `incremental-implementation` skill instalada; no es obligatoria en task.md |
| DoD por nivel (change/task/release) | ✅ PARCIAL | Por tipo (task.md) y por task (AC en DRV-131); multi-nivel sin declarar (§3.5) |
| Gates de calidad por nivel | ✅ PARCIAL | Fast Gate vs Heavy Certification; coverage y perf sin gate (§3.3-3.4) |
| Flow metrics (lead/cycle time) | ❌ | Ausente (§3.6) |
| Gestión de dependencias entre tasks | ✅ PARCIAL | gate 🔴 BLOQUEADO en plan.md; sin grafo de dependencias formal automático |
| Revisión independiente antes de merge | ❌ | Self-review (§3.7) |

## Apéndice C — Trazabilidad de hallazgos (archivo → consecuencia)

| Hallazgo (§) | Archivo de origen | Archivo(s) afectado(s) |
|---|---|---|
| Doble skill de debug (§5.1) | `.agents/skills/systematic-debugging/SKILL.md` + `.opencode/skills/debugging-and-error-recovery/SKILL.md` | AGENTS.md, RULES.md §10, prompts (21 refs) |
| Path resolution incompleta (§5.2, §6.1) | `tasks/complete/DRV-131.md`, `tasks/closed/` | AGENTS.md tabla Path Resolution |
| Conteo de skills (agreement drifts) (§5.3) | SKILLS-MANIFEST.md vs `Get-ChildItem .opencode/skills` (29 dirs) | SKILLS-MANIFEST.md, AGENTS.md |
| North Star no medible (§5.4) | RULES.md (rule de tasa completado) | `/status`, plan files |
| Hooks git sin instalar (§4, §6.3) | `.opencode/skills/unified-review/templates/pre-push.ps1` | git hooks locales |
| Coverage sin umbral (§3.3) | `docs/operations/CI_POLICY.md`, `dev-tools/verify.ps1` | CI Heavy |

## Apéndice D — Veredicto cuantitativo por práctica

Muestra de 10 prácticas evaluadas (de la tabla maestra):

| Práctica | Peso del impacto | Veredicto | Prioridad de cierre |
|---|---|---|---|
| Debug sistemático | Alto | ✅ PARCIAL (skill fuerte, gate débil) | P0-1 / P1-1 |
| Complejidad de problema (Cynefin/pre-mortem) | Medio-alto | ❌ FALTANTE | P3-1 |
| Investigación técnica | Medio | ✅ PARCIAL | — |
| DoD por niveles | Alto | ✅ PARCIAL | P2-4 |
| Quality gates | Alto | ✅ PARCIAL | P2-1 / P2-2 |
| Refactoring seguro | Medio | ❌ FALTANTE | P3-3 |
| Code review independiente | Alto | ❌ FALTANTE | P2-3 |
| Documentación viva | Medio | ✅ PARCIAL | — |
| Estimación/riesgo/dependencias | Medio | ✅ PARCIAL | P3-2 |
| Consistencia interna del pipeline | Alto | ⚠️ CONTRADICCIÓN | P0-1 / P0-2 |

Tendencia general: **el sistema declara más de lo que enforceda**. Patrón repetido en 6 de 10 prácticas: la práctica existe como skill o script instalado, pero el trigger del pipeline que la activa es "si corresponde" (optativo), no un paso obligatorio con contrato. La arquitectura soporta el cierre a costo bajo porque las piezas (skills, scripts, estado C0, gate) ya existen — falta el wiring normativo.

## Resumen ejecutivo

- **Fortaleza central:** el contrato booleano verificable (RULES.md) + triaje con verificación de realidad (plan.md Paso 0) + checks mecánicos por tipo (task.md, con mapeo exacto en DRV-131). Es un sistema de tareas que exige evidencia mecánica, mata tareas stale antes de ejecutarlas y tiene estado C0 validado — por encima de la media de harness de un solo dev.
- **Gap crítico:** el debugging sistemático y la verificación pre-push existen como skills/scripts pero **no como gates de proceso** — el contrato mide resultado, no método. La doble skill de debugging + los fallos de Path Resolution (complete/, doble ubicación de skills) + hooks no instalados son los riesgos operativos reales.
- **Prioridad:** P0 unificar skill de debug y arreglar path resolution; P1 gates de proceso (Phase 1 evidence, pre-push barrier, métricas North Star); P2 coverage, security/performance phase, review adversarial, DoD multi-nivel; P3 complejidad (Cynefin/pre-mortem), calibración, DORA, mutation.
- **Retorno esperado:** pasar de "el sistema exige que el test pase" a "el sistema exige que el método sea correcto" — el paso exacto que piden eng-02 (root-cause) y eng-03 (DoD por nivel).