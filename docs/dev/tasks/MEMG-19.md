---
title: "TASK MEMG-19: Prospectiva + descartes documentados (sensorial/emocional)"
kind: task
description: "Prospectiva/intencional como patrón de uso (working+temporal+procedimental; no almacén) con benchmarks PM-Bench/TriggerBench + descartes sensorial-como-almacén y emocional/motivacional con respaldos y criterio de revisión; no normativo, sin implementación"
---

# TASK MEMG-19: Prospectiva + descartes documentados (sensorial/emocional)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 63, Wave F5 — Frontera & estrategia; research/docs-only)
- **Fuente:** Backlog `MEMG-19` (`docs/dev/Backlog.md:157`) + plan Task 63 (L1809-1828, bloque F0 expandido) + dims hub §Validación (Notion — citada por ID, sin transcripción) + validación externa 2026-09-30 (nota DELTA `docs/dev/Backlog.md:120` + fila `MEMG-19` `:157`)
- **Esfuerzo:** 🟢 1d | **Appetite:** max 1d
- **Prioridad:** 🟢 (plan) / 🟡 P2 (Backlog)
- **Tipo:** Research/docs (docs-only — cero código; cero símbolos públicos)
- **Turns estimados:** 6-10 (una sesión de sub-agente)
- **Creado:** 2026-10-06 | **last-synced:** 2026-10-06
- **Estado:** ⏳ IN PROGRESS (**server key real: `MEMG-19`** — el numérico `63` no resuelve en el plan parser; precedente MEMG-16/`58`; reservada ⏳ por el orquestador)
- **Incógnitas (uphill):** 0 — downhill directo (prospectiva → descartes). Resueltas en DISCOVERY: (a) los descartes ya están decididos (sensorial 2026-09-14; emocional en la validación 2026-09-30) → este run **documenta**, no decide (Gate D no disparado); (b) las 8 fuentes externas resuelven y están fetch-verificadas; (c) las anclas repo del patrón (working/temporal/procedimental) verificadas contra worktree
- **Pendientes (downhill):** 4 steps (1-4)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `MEMG-19`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Lectores de docs (sin código):** master plan Task 63 (estado — **prohibido editar**), Backlog `:157` (fila MEMG-19 — se elimina al cierre, Trigger 1 progreso), `memg-18-multimodalidad.md` §1.1(C)/§3.8 (pointer al descarte sensorial documentado), `memg-14-marco-2.0.md` §2.2/§2.7 (pointer descartes + prospectiva entregada), avance `operaciones.md` (registro de cierre), dims hub §Validación (Notion — citada por ID; sync = lane owner). |
| Callees | Fuentes citadas (sin modificar): anclas de código (`vanta-memory/src/core/abstractions/types.rs`, `vanta-memory/src/utils/task_checkpoint.rs`, `vanta-memory/src/context_engine/mmd.rs`, `vanta-memory/src/services/scheduler.rs`, `vanta-memory/src/utils/timer_scanner.rs`, `vanta-memory/src/core/hooks/auto_recall.rs`, `src/sdk/types/record.rs`), docs del marco (`memg-14`, `memg-18`, `NOTION-SYNC-2026-09-24.md`), URLs externas fetch-verificadas. |
| Implicaciones | **Aditivo docs-only:** `docs/dev/research/memg-19-prospectiva-descartes.md` (nuevo; `kind: research`) + `docs/dev/tasks/MEMG-19.md` (nuevo, este archivo) + ediciones puntuales (pointer) en `memg-18`/`memg-14` + eliminación de fila `Backlog.md:157` + registro `operaciones.md`. `gen-index --write` regenera índices. Sin código, sin wire, sin deps, sin locks. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-06, worktree sobre HEAD `55b9a532` — sin drift tras `c29e2cb9`; MEMG-15 commiteó su trabajo durante el DISCOVERY → Backlog limpio, sin staging ajeno pendiente).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 63 L1809-1828 + vecinos 61/62 + reglas de ejecución L30-38 + regla investigación profunda L2089).
  - `docs/dev/Backlog.md` (:142-171 DELTA P2/P3; fila `:157` MEMG-19; `FIND-288` :431 y `FIND-290` :433 — consumidores pendientes de checkpoints/scoring, dim1).
  - `docs/dev/backlog-futuro.md` (:1-52 — FUT-15 + revisiones).
  - `docs/dev/research/memg-14-marco-2.0.md` (:1-209 — completo; §2.2 ámbitos candidatos, §2.3 ejes, §2.7 no-elevar).
  - `docs/dev/research/memg-18-multimodalidad.md` (:1-161 — completo; §1.1(C) almacén sensorial descartado, §3.8 lo que no toca).
  - `docs/dev/tasks/MEMG-18.md` (:1-200 — completo; formato canónico de task file de la campaña).
  - `docs/dev/strategy/NOTION-SYNC-2026-09-24.md` (:120-144 §13-14 syncs + §0 mapeo de IDs de páginas).
  - `docs/dev/avance/activo/operaciones.md` (tail — formato de registro de cierre F5).
  - `vanta-memory/src/core/abstractions/types.rs` (:25-40 — `MemoryType`: Persona/Episodic/Instruction/WorkFact/WorkTask/WorkMethod/WorkArtifact).
  - `vanta-memory/src/utils/task_checkpoint.rs` (:1-80 — checkpoints de tarea = dim 1 working memory, MEMG-20; consumer pendiente FIND-288).
  - `vanta-memory/src/context_engine/mmd.rs` (:1-30 — MMD = working memory de la tarea actual).
  - `vanta-memory/src/services/scheduler.rs` (:1-60 — pass pull-based + loop opt-in; ADR-0054; timers internos del pipeline).
  - `vanta-memory/src/utils/timer_scanner.rs` (:14-25 — `TimerScanner`/`run_once`).
  - `vanta-memory/src/core/hooks/auto_recall.rs` (greps: :283-296 scopes `RecallScope::{…,Agent,Team}`, :390-423 `perform_auto_recall*`, :653-657 filtro D22 por `agent_id`/`team_id`).
  - `src/sdk/types/record.rs` (:201, :287-297 `valid_at_ms`/`invalid_at_ms`, :402-406 `as_of_ms` — SCH-02/03).
  - `.opencode/task-system/prompts/pipeline-full.md` (contrato de ejecución — leído completo).
- **Archivos referenciados hacia dentro (imports/deps):** n/a (docs; sin imports). El research-doc referenciará hacia afuera con enlaces relativos (documentation-skill §1).
- **Referencias entrantes (grep `MEMG-19|prospectiva|sensorial|emocional` HEAD `55b9a532`→`c29e2cb9` sin cambios en los hits):** `Backlog.md:157` (fila), master plan L1809-1828 + L1814-1824 (**prohibido editar**), `memg-18-multimodalidad.md:38,:121` (refs a `Backlog.md:157`), `memg-14-marco-2.0.md:108` (MEMG-19 entre los que desarrollan candidatos F5), `ROADMAP-v0.7.md:67` (mención informativa), `FND-24` (`docs/dev/tasks/FND-24.md:50,59` + `research/archive/FND-24-icp-jtbd.md` + `validacion/04-marketing-branding-gtm-playbook.md:35` — "emocional" en contexto JTBD/marketing, **distinto**, sin cambio), `SKILLS-MANIFEST.md:402` (skill a11y no relacionada).
- **Veredicto impacto:** **BAJO (aditivo docs-only)** — 1 doc nuevo + 1 task file + 2 pointers + 1 fila Backlog eliminada al cierre + 1 registro avance + índices regenerados. Sin cambios en código, contratos, wire, deps ni locks. Riesgos del pre-mortem mitigados: (1) sin transcripción de Notion — citas por ID de página (patrón NOTION-SYNC §14); (2) fuentes 8/8 fetch-verificadas y fechadas (regla investigación profunda owner 2026-10-06); (3) tono declarado "no normativo / extensión futura" desde el §0 del doc.

## Contrato

"(a) prospectiva/intencional como patrón de uso documentado (working+temporal+procedimental; no almacén) con benchmarks de referencia (PM-Bench/TriggerBench); (b) descartes sensorial-como-almacén y emocional/motivacional con respaldos (arXiv 2602.23944 MemEmo/HLME etc.) + criterio de revisión (emocional: re-evaluar en 6-12m); grep del repo > 0 para ambos." (plan Task 63 L1818).

**Verificación del contrato (cierre):** research-doc existe con (a) prospectiva como patrón de uso (definición + decisión "no almacén" + anclas working/temporal/procedimental + benchmarks de referencia con cifras verificadas) y (b) ambos descartes con respaldos fetch-verificados + criterio de revisión por candidata; `rg -c "sensorial"` y `rg -c "emocional"` > 0 sobre el doc; fuentes fechadas; gates docs 0 (`check-links` + `check-docs` + `gen-index --check`) + `validate-docs-coverage.ps1` 0 gaps.

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): no disparado.** (a) Blast radius ≤ 8 archivos, docs-only, sin hot path/API pública; (b) sin símbolos públicos nuevos (cero código); (c) contrato sancionado por el plan F0 (Task 63, Gate Result ✅ DO) — sin ambigüedad nueva; (d) **los descartes ya están decididos y registrados** (sensorial: 2026-09-14, P49; emocional: validación externa 2026-09-30) — este run documenta el registro, no lo re-litiga. Si la evidencia hubiera contradicho un descarte → BLOQUEO/nota, no decisión unilateral.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Hogar del doc | A) **research-doc nuevo `docs/dev/research/memg-19-prospectiva-descartes.md` (kind=research)** / B) sección en `memg-14-marco-2.0.md` (mezcla F5) / C) `docs/dev/strategy/` (no es hogar de investigaciones) | ✅ **A** — patrón `memg-14-*.md`/`memg-18-*.md`; `kind: research` → ruta canónica (documentation-skill §2.1) |
| 2 | Framing de prospectiva | A) **Patrón de uso (working+temporal+procedimental; no almacén)** / B) dimensión/almacén nuevo (contra: el survey 2602.06052 no lista "prospective" entre sus mecanismos de store; PM-Bench/TriggerBench evalúan *capacidad del agente*, no un store; la industria (Proactive Memory Agent) la implementa como política de intervención) | ✅ **A** — contrato L1818(a) + evidencia multi-fuente (§1 del doc) |
| 3 | Sensorial | A) **Documentar el descarte existente (2026-09-14) con respaldo multi-fuente + criterio de revisión** / B) reabrirlo (contra: sin evidencia nueva; el material sensorial ya entra por ingesta multimodal — MEMG-18) | ✅ **A** — el descarte vale para el *almacén dedicado*; la materialización sigue el trigger de MEMG-18 |
| 4 | Emocional/motivacional | A) **Descarte documentado + criterio de re-evaluación 6-12m** / B) elevarlo ahora (contra: MemEmo — ningún sistema evaluado robusto en las 3 dimensiones; PsychoAgent — retrieval afectivo sin significancia en raters) | ✅ **A** — estado del arte inmaduro; ventana 2027-03-30 → 2027-09-30 + triggers |
| 5 | Verificación de fuentes | A) **Fetch de cada URL citada en DISCOVERY (TSYS-13); solo se citan las resueltas, con fecha** / B) citar y marcar `[a verificar]` | ✅ **A** — red disponible; 8/8 externas resueltas (7 arXiv + Mem0) |
| 6 | Notion | A) **Citar IDs de página (patrón NOTION-SYNC) + `[a verificar — Notion]` donde el detalle no está en repo; sin fetch** / B) fetch Notion (workers no operan Notion — regla 0c-context; precedente `mgr-23-24:11`) | ✅ **A** — sin transcripción (pre-mortem #1 del plan) |
| 7 | Tono | A) **Declarar "no normativo / extensión futura" en §0 y §Límites** / B) tono normativo (contra: el marco exige evidencia de producto para elevar) | ✅ **A** — contrato: "extensiones futuras, no normativas" |

## Invariantes de dominio (handoff — MUST)

1. **El alcance v1.0 no se reabre** (pre-mortem #1 del plan): el contrato es documental, cero código. Los descartes se documentan, no se re-litigan.
2. **La prospectiva NO es almacén:** el doc no propone tipos/estructuras/almacenes nuevos; el patrón mapea a superficies existentes (working D1 + temporal D5 + procedimental D4).
3. **Descartes consistentes con el registro** (sensorial 2026-09-14; emocional 2026-09-30): cualquier reapertura es del owner (Gate D/no unilateral); la re-evaluación emocional ocurre en su ventana (6-12m) con triggers declarados.
4. **Regla 11:** todo claim con fuente reproducible (repo path o URL fetch-verificada + fecha); cero `[a verificar]` sin marcar; cifras (65.1% F1, +8.3pp, 0.933 vs 0.500/0.667) citadas de las fuentes fetch-verificadas.
5. **No tocar:** master plan, `opencode.jsonc`, `docs/pipeline-state.json` (prohibidos). WIP ajeno (MEMG-15) no se stagea; commit con **pathspec**.
6. **Frontera de este run:** cero implementación; cero símbolos públicos; el doc **no es normativo** para código hasta que un caso/trigger lo promueva (fila de implementación).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** ≤0 — sin código, sin `unsafe`, sin deps. El run **elimina** deuda de trazabilidad: la prospectiva y los descartes sensorial/emocional dejan de vivir solo en Notion y quedan auditables desde el repo (respaldos + criterios de revisión + IDs de página citados). Deuda declarada: ninguna nueva.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Research-doc con (a) prospectiva como patrón de uso (definición + decisión + anclas + benchmarks) y (b) descartes sensorial/emocional (respaldos + criterios), verificable 1:1 contra el contrato L1818; fuentes fetch-verificadas y fechadas; grep > 0; gates docs 0; task file completo (Impacto Regla 0 + Review P2-01) |
| **Commit** | Commit atómico conventional `docs(research):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | n/a (docs; sin changelog) |

## Herramientas necesarias

- `webfetch` (verificación TSYS-13) + skill `coordinated-web-search` (router obligatorio) + grep/read (anclas de código) + `campaign_*` (estado/scope/verify)
- Gates docs: `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` + `pwsh scripts/validate-docs-coverage.ps1`
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre) + fork `vanta-review` (P2-01)

**Skills cargadas (SDP v3):** base auto (`campaign-executor` · `progreso` · `ponytail`) · `writing-guidelines` (base type docs) · `documentation-and-adrs` (keyword docs) · `documentation-skill` (obligatoria `docs/**`) · `source-driven-development` (verificación de fuentes) · `spec-driven-development` (lifecycle DEFINE — decisión/spec documental) · `coordinated-web-search` (router obligatorio) · `performance-optimization` (policy pin — declarado N/A en §Fases). Excluidas con justificación: `writing-plans` (contrato sancionado por el plan F0; sin plan multi-paso nuevo), `incremental-implementation`/`test-driven-development`/`context-engineering` (lifecycle BUILD — docs-only, sin lógica ni tests), `observability-and-instrumentation` (sin instrumentación).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — N/A: docs-only, sin trust boundary nuevo, sin input de usuario, sin deps, sin red en producto (los fetches de verificación son de DISCOVERY, solo lectura).
- [ ] **PERFORMANCE** — N/A: sin hot paths, sin código; Regla 9 no dispara (sin claim de optimización). El policy pin `performance-optimization` se declara N/A por este motivo.

## Steps

### Step 1 — DISCOVERY + task file (Regla 0 + Spec + contrato)

- **Archivos:** `docs/dev/tasks/MEMG-19.md` (nuevo, este archivo)
- **Acción:** evidencia anclada (fila Backlog, dims hub §Validación por ID, memg-14/memg-18, anclas de código del patrón); resolución de los descartes registrados + Gate D evaluado; verificación TSYS-13 de las 8 fuentes externas (todas fetch-verificadas 2026-10-06, títulos y cifras registrados en §Notas); formato canónico con Impacto Regla 0 + Spec + DoD.
- **Verify:** task file existe + `campaign_validate_scope` OK (advisory)
- **Evidencia:** ✅ este archivo; fuentes verificadas en §Notas (8/8 con fecha y cifras)
- **Estado:** ✅ COMPLETED

### Step 2 — Research-doc: prospectiva + descartes

- **Archivos:** `docs/dev/research/memg-19-prospectiva-descartes.md` (nuevo)
- **Acción:** redactar el research-doc: §0 resumen (no normativo); §1 prospectiva/intencional como patrón de uso (1.1 definición; 1.2 decisión "no almacén"; 1.3 materialización working+temporal+procedimental con anclas; 1.4 benchmarks PM-Bench/TriggerBench/Proactive Memory Agent; 1.5 límites); §2 descartes (2.1 sensorial-como-almacén; 2.2 emocional/motivacional; 2.3 registro auditable); §3 límites y deuda; §4 fuentes (repo + externas fechadas).
- **Verify:** `node scripts/docs/check-docs.mjs` (frontmatter/kind OK)
- **Evidencia:** ✅ doc creado (`docs/dev/research/memg-19-prospectiva-descartes.md`): §0 resumen + §1 patrón de uso (definición PM-Bench + decisión no-almacén con fundamento 4 puntos + 4 capas ancladas en código + 3 benchmarks con cifras verificadas) + §2 descartes (sensorial: Mem0/survey/MIRIX/MEMG-18 §2.1; emocional: MemEmo/PsychoAgent/survey 2511.20657 §2.2; registro auditable §2.3) + §3 límites + §4 fuentes. `check-docs`: GATING all clear (kind/path OK); `check-links`: 0 broken; grep del contrato: `sensorial`=10 · `emocional`=10 · `prospectiva`=7 (>0 ✅). **Profundización multi-fuente (regla owner 2026-10-06):** 9 fuentes externas fetch-verificadas (8 arXiv + Mem0) — tabla en §Notas.
- **Estado:** ✅ COMPLETED

### Step 3 — Pointers (memg-18 + memg-14) + gates docs

- **Archivos:** `docs/dev/research/memg-18-multimodalidad.md` (§1.1(C) + §3.8), `docs/dev/research/memg-14-marco-2.0.md` (§2.2 + §2.7), `docs/index.md` + `llms.txt` (generados)
- **Acción:** en memg-18, reemplazar la referencia stale `../Backlog.md:157` del descarte sensorial por el pointer al doc nuevo (§1.1 C + §3.8); en memg-14, pointer de descartes en §2.7 + estado "entregado" de MEMG-19 en §2.2; `gen-index --write`.
- **Verify:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` exit 0
- **Evidencia:** ✅ pointers aplicados (memg-18 `:38` y `:121` → `memg-19-prospectiva-descartes.md` §2.1; memg-14 `:108` → "MEMG-18 y MEMG-19 entregados"; memg-14 `:146` bullet "Descartes de dimensión (dims hub §Validación)" → doc §2); `gen-index --write` regeneró `docs/index.md` (Research 69→70 · Task files 1100→1101) + `llms.txt` (459→460 páginas) — delta propio limpio, sin contaminación ajena; `gen-index --check` exit 0; `check-links` 0 broken; `check-docs` GATING all clear; `validate-docs-coverage.ps1` 0 gaps.
- **Estado:** ✅ COMPLETED

### Step 4 — Verify full + OCR + Review P2-01 + commit local + cierre campaign

- **Archivos:** `docs/dev/tasks/MEMG-19.md` (§Review + RESULTADO), `docs/dev/Backlog.md` (eliminar fila MEMG-19), `docs/dev/avance/activo/operaciones.md` (registro de cierre)
- **Acción:** `pwsh scripts/validate-docs-coverage.ps1` · OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) → Critical/High bloquean; Medium → FIND. Clasificar tier HARD-02: paths `docs/**` → **Fast**. Fork `vanta-review` (fresh context) para P2-01. Commit **LOCAL** `docs(research):` con pathspec; cierre `campaign_update_task_state(completed, taskId:"MEMG-19")` con recitation + payload `review`; Trigger 1 progreso (fila Backlog eliminada + registro avance).
- **Verify:** veredicto registrado + `git show --stat HEAD` limitado a archivos propios + `git status` sin WIP ajeno stageado
- **Evidencia:** ✅ **Gates (working tree, server-side vía `campaign_verify_cmd`):** `check-links` ✅ exit 0 (3072 links, 0 broken; retry tras 1 hiccup transitorio de spawn del server) · `check-docs` ✅ GATING all clear · `gen-index --check` ✅ exit 0 · `validate-docs-coverage.ps1` ✅ 0 gaps. **OCR delegation:** `pwsh dev-tools/ocr-review.ps1 -Format json` → 1 archivo code reviewable = **WIP ajeno** (`dev-tools/heavy-test-lock.ps1`, untracked, fuera de pathspec); diff propio docs-only excluido por diseño → 0 hallazgos aplicables. **Review P2-01:** vanta-review tier Fast (sesión `ses_ef06a43f1ffeQ0YZ2vhn21XWlw`) → ✅ **APPROVE** (0 Required; O-1/N-1/N-2 aplicados; N-3 informativo; N-4 cumplido). **Commit 1:** `93846595` (6 archivos propios; pre-commit hook OK) — `docs(research):` LOCAL. **Commit 2 (cierre):** Backlog.md (fila MEMG-19 eliminada) + avance + task file final. **Trigger 1 progreso:** fila `MEMG-19` eliminada de `Backlog.md` + registro en `operaciones.md`. **Campaign:** `completed` taskId `MEMG-19` con payload review.
- **Estado:** ✅ COMPLETED

## Dependencias

- ✅ Evidencia base: dims hub §Validación (Notion, por ID: `Las 8 dimensiones de la memoria` + `Los 10 ámbitos afectados` — IDs abreviados de NOTION-SYNC §14) · validación externa 2026-09-30 (`Backlog.md:120`) · `memg-14-marco-2.0.md` §2.2/§2.7 · `memg-18-multimodalidad.md` §1.1(C)/§3.8 · `FIND-288`/`FIND-290` (consumidores pendientes dim1 — contexto del patrón).
- Consumidores: MEMG-14 (pointer descartes), MEMG-18 (pointer sensorial), cadencia del marco 2.0 (`memg-14` §4) para la re-evaluación emocional.
- nextTask: lo decide el orquestador.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review — P2-01, contexto fresco (no participó de la implementación); tier **Fast** (HARD-02: paths `docs/**`; verify mecánico + veredicto; sin adversarial completo). Sesión reviewer: `ses_ef06a43f1ffeQ0YZ2vhn21XWlw`. Veredicto: ✅ **APPROVE** (0 Required; 1 Optional + 4 NIT).
- **Enfoque:** contrato L1818 1:1 (prospectiva patrón/no-almacén + benchmarks; descartes + respaldos + criterio 6-12m; grep > 0; sin implementación; no normativo); consistencia con el registro (sensorial 2026-09-14; emocional 2026-09-30); Regla 11 (fuentes fetch-verificadas); anclas de código; gates 4/4; scope docs-only.
- **Cómo se probó:** reviewer re-corrió `check-links` (exit 0) · `check-docs` (GATING all clear) · `gen-index --check` (exit 0) · `validate-docs-coverage` (0 gaps); `rg -c` (sensorial=10 · emocional=10 · prospectiva=7 — coincide exacto con el auto-reporte); **fetch en vivo 9/9** (PM-Bench 65.1% F1/v1 2026-07-14/COLM 2026 · TriggerBench v1 2026-06-22 · Proactive +8.3pp/+6.8pp · MemEmo "none of the evaluated systems…" · PsychoAgent 0.933 vs 0.500/0.667 sin significancia · survey TMLR v4 · 2511.20657 v2 · MIRIX · Mem0 quotes exactos); anclas de código 7/7 sin drift; scope check (`git status` → solo docs propios + ajenos no atribuibles).
- **Hallazgos + disposición:**
  1. [OPTIONAL r1] O-1 — anchor stale `Backlog.md:140` en task file (:12/:146) → **aplicado**: citas corregidas a `:120` (nota DELTA) + `:157` (fila MEMG-19).
  2. [NIT r1] N-1 — rango ":140-171 DELTA" empezaba en `:142` → **aplicado** (`:142-171`).
  3. [NIT r1] N-2 — HEAD citado `55b9a532` vs actual `c29e2cb9` (sin drift verificado) → **re-pinneado** (doc §4 + task file: "sin drift tras `c29e2cb9`"; Context Save Point → `c29e2cb9`).
  4. [NIT r1] N-3 — plan L1814 cita `Backlog.md:160` (stale; archivo prohibido, informativo) → **no accionable** por este lane (nota).
  5. [NIT r1] N-4 — forward-looking "fila removida al cierre" → **condición de cierre** cumplida en Step 4 (fila eliminada + registro avance).
- **Veredicto:** ✅ **APPROVE** — contrato 1:1, descartes sin reapertura, fuentes 9/9 re-verificadas en vivo por el reviewer, gates verdes, scope docs-only; listo para commit local (Step 4).

## Context Save Point

- **Última acción:** Steps 1-4 ✅ — DISCOVERY completo (task file + fuentes 9/9 fetch-verificadas + Gate D evaluado); research-doc (prospectiva + descartes); pointers (memg-18/memg-14) + gates docs 4/4; OCR (N/A docs-only); review P2-01 APPROVE (O-1/N-1/N-2 aplicados); commit `93846595`; fila Backlog removida; registro avance; cierre campaign `MEMG-19`.
- **Próximo paso:** ninguno (tarea cerrada). Handoff: **server key real = `MEMG-19`** (el numérico `63` no resuelve en el plan parser — precedente MEMG-16/`58`); push pendiente de instrucción owner (Regla 7).
- **Estado del worktree:** HEAD `93846595` + commit de cierre docs(avance); WIP ajeno (master plan + `opencode.jsonc` + `dev-tools/heavy-test-lock.ps1`) NO se stagea; prohibido tocar `docs/pipeline-state.json`.

## Notas

- **Fuentes verificadas y fechadas (multi-fuente — regla owner 2026-10-06; todas fetch-verificadas el 2026-10-06, TSYS-13):**

  | URL | Fecha (verificada) | Claim que respalda |
  |---|---|---|
  | `arxiv.org/abs/2607.12385` | v1 2026-07-14 (COLM 2026) | PM-Bench: PM = "execute an intention at a specific future cue or state while other activities are ongoing"; mejor agente 65.1% F1; ninguna estrategia domina — la prospectiva es un problema abierto |
  | `arxiv.org/abs/2606.23459` | v1 2026-06-22 | TriggerBench: PM vs RM (RM satura a 100K tokens, PM decae con contexto); precision-recall tradeoff + fragilidad atencional + overfit "always-remind" |
  | `arxiv.org/abs/2607.08716` | v1 2026-07-09 | Proactive Memory Agent: memoria como intervención activa (+8.3pp Terminal-Bench 2.0 / +6.8pp τ²-Bench); intervención selectiva > exposición pasiva/always-on |
  | `arxiv.org/abs/2602.06052` | v4 2026-08-04 (TMLR) | Survey: mecanismos = sensory/working/episodic/semantic/procedural (sin "prospective" como store); memoria = sustrato + mecanismo + sujeto |
  | `arxiv.org/abs/2602.23944` | v1 2026-02-27 | MemEmo/HLME: 3 dimensiones emocionales; **ningún sistema evaluado robusto en las 3** |
  | `arxiv.org/abs/2608.07438` | v2 2026-08-31 (BICA 2026) | PsychoAgent: retrieval afectivo 0.933 vs 0.500/0.667 en conflict-recall pero **diferencias pairwise no significativas** con 5 raters ciegos |
  | `arxiv.org/abs/2511.20657` | v2 2026-05-02 | Survey de emotional intelligence: reconocer/evocar/expresar = affective computing (no store de memoria); desafíos abiertos |
  | `mem0.ai/library/agent-memory/how-memory-shapes-us-a-deep-dive-into-the-types-of-memory` | actualizado 2026-09-03 | Mem0: equivalente IA de memoria sensorial = raw input buffer; "usually isn't retained on purpose"; "Most AI systems don't need a dedicated sensory-memory analog" |

  (Adicional: `arxiv.org/abs/2507.07957` — MIRIX, v1 2025-07-10: multimodal via Resource Memory + 6 tipos estándar, sin store sensorial; re-verificado para el descarte sensorial.)
- **Coordinación MEMG-15 (en vuelo → commiteó `55b9a532` durante el DISCOVERY):** `docs/dev/Backlog.md` quedó limpio (sin staging ajeno); re-verificar `git status` al cierre antes del commit (pathspec).
- **Precedente workers↔Notion:** `mgr-23-24:11` — workers no operan Notion; el doc cita IDs de página y no transcribe (pre-mortem #1 del plan); el canal de sync es el draft de MEMG-14 §5 (lane owner).
- **NOTICED BUT NOT TOUCHING:** `FND-24` + `research/archive/FND-24-icp-jtbd.md` + `validacion/04-marketing-branding-gtm-playbook.md:35` ("emocional" en JTBD/marketing — contexto distinto, sin relación con memoria); `ROADMAP-v0.7.md:67` (mención informativa); `SKILLS-MANIFEST.md:402` (skill a11y); `EXPERIMENTAL_FEATURES.md` (sin cambio — no hay feature nueva).

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: 93846595 (docs research, LOCAL — sin push) + commit de cierre docs(avance)
ARCHIVOS: docs/dev/research/memg-19-prospectiva-descartes.md (nuevo), docs/dev/tasks/MEMG-19.md (nuevo), docs/dev/research/memg-18-multimodalidad.md (pointer), docs/dev/research/memg-14-marco-2.0.md (pointer), docs/index.md + llms.txt (generados), docs/dev/Backlog.md (fila MEMG-19 removida), docs/dev/avance/activo/operaciones.md (registro)
VERIFY_CONTRATO: pasa — research-doc (prospectiva patrón de uso + descartes con respaldos + criterio 6-12m; contrato L1818 1:1) + grep sensorial=10 · emocional=10 (>0) + gates docs 4/4 exit 0 (server-side) + validate-docs-coverage 0 gaps; fuentes 9/9 fetch-verificadas (TSYS-13) + re-verificadas en vivo por el reviewer
BLOQUEO: ninguno
GATES_EVALUADOS: P:no(plan sanciona Task 63; docs-only; sin símbolos) D:no(descartes ya registrados; sin ambigüedad nueva) V:no(sin fallas de verify; 1 hiccup transitorio de spawn resuelto en retry) C:no(WIP ajeno no stageado; pathspec limpio)
SKILLS_CARGADAS: campaign-executor, progreso, ponytail (base) · writing-guidelines · documentation-and-adrs · documentation-skill · source-driven-development · spec-driven-development · coordinated-web-search · performance-optimization (policy pin, N/A declarado)
```
