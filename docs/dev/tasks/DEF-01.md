---
title: "DEF-01: Decisión única de producto + alineación SPEC/README/VISION"
kind: task
description: "SPEC/README/VISION declaran la MISMA jerarquía (1 núcleo + 3 puertas) sin contradicciones Y Gate P registrado (owner) Y pwsh scripts/validate-docs-coverage.ps1 verde\""
---

# DEF-01: Decisión única de producto + alineación SPEC/README/VISION

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 9)
- **Fuente:** Backlog `DEF-01` (L936) · master Task 9 · R4 (`product-definition-gap-2026-09-24.md`, hallazgo #1)
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🔴
- **Tipo:** Docs (decisión de producto + alineación)
- **Turns estimados:** 8
- **Creado:** 2026-09-26T19:55
- **last-synced:** 2026-09-26T19:55
- **Estado:** ⏳ IN PROGRESS (2026-09-27) — Steps 1–3 ✅ (Gate P: opción c; 4 docs alineados) · Step 4: gates ✅ · cierre pendiente (P2-01: reviewer fresco o waiver)
- **Incógnitas (uphill):** 0 abiertas — jerarquía decidida por owner (opción c, 2026-09-27)
- **Pendientes (downhill):** 1 — Step 4 (solo ACCEPT mecanizado)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | DEF-05 (North Star cuelga de la jerarquía) · DEF-07 (presupuesto core-promise vs labs) · ICP-01..03 (one-pagers) · README_ES (paridad EN↔ES) · lectores externos del README |
| Callees | `docs/dev/strategy/` (ROADMAP.md histórico, GO_TO_MARKET.md, Manual estratégico) · `docs/dev/research/product-definition-gap-2026-09-24.md` · SPEC Adenda 2026-09-24 |
| Implicaciones | Posicionamiento público (README) + definición de producto (SPEC/VISION) · cero código · riesgo de drift si los 3 docs no se editan juntos (Regla 3, mismo PR) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `SPEC.md` (122L) · `README.md` (434L) · `docs/dev/vision/VISION.md` (278L) · `docs/dev/strategy/ROADMAP.md` (485L) · `docs/dev/strategy/VantaDB_Manual_Estrategico_Unificado.md` (header/contexto)
- **Archivos referenciados hacia dentro (imports/dependencias):** `README_ES.md` (espejo ES del README) · `VISION.md` frontmatter `related: GO_TO_MARKET.md, VANTADB-PRO-FEATURES.md` · `SPEC.md` citado por planes MVP (`2026-09-17-mvp-memoria-agentes.md`)
- **Archivos que referencian a los editados (referencias entrantes):** README es entry point público (badges + docs index) · SPEC es fuente de verdad de sub-agentes del MVP · VISION relacionada con GO_TO_MARKET/COMPARISON
- **Veredicto impacto:** **medio** — cambia posicionamiento en 3 docs públicos; se rompe si README y SPEC siguen diciendo targets distintos (R4); README_ES requiere paridad si cambia el EN.

## Contrato
"SPEC/README/VISION declaran la MISMA jerarquía (1 núcleo + 3 puertas) sin contradicciones Y Gate P registrado (owner) Y `pwsh scripts/validate-docs-coverage.ps1` verde"

## Spec (SDD — decisión de producto)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Jerarquía de producto | A) 1 núcleo ("SQLite para agentes" = motor de memoria embebido gobernado) + 3 puertas ICP (AI-IDEs MCP · local-LLM/privacidad · frameworks) / B) multi-posicionamiento actual (RAG/edge/agentes en paralelo) / C) híbrida: A + RAG como capacidad declarada, no target | C | ✅ decidido (owner, 2026-09-27, opción c) — RAG: UNA mención como capacidad del núcleo; "edge"/"structured-knowledge" fuera |
| 2 | Docs a editar | A) los 3 existentes: `SPEC.md` §Adenda + `README.md` + `docs/dev/vision/VISION.md` / B) crear VISION nuevo | A | ✅ decidido-por-evidencia (Backlog `DEF-01` archivos: `SPEC.md`, `README.md`, `docs/dev/vision/VISION.md`) |
| 3 | Lengua | A) SPEC ES (planning) · README/VISION EN (técnico) / B) unificar a un idioma | A | ✅ decidido-por-evidencia (AGENTS.md §Doc Language Split) |
| 4 | Alcance | A) 3 docs + referencias internas (sin marketing) / B) incluir GO_TO_MARKET/Manual | A | ✅ decidido-por-evidencia (master Task 9 pre-mortem F2: "scope creep a marketing") |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) la jerarquía declarada es **idéntica** en los 3 docs (sin targets en conflicto); (2) README/VISION en inglés, SPEC en español (planning); (3) paridad README↔README_ES si cambia contenido EN; (4) Gate P obligatorio ANTES de editar — sin respuesta del owner no se toca nada (BLOQUEO).
- **Comandos de verificación:** `rg -n "<frase clave aprobada>" SPEC.md README.md docs/dev/vision/VISION.md` → 3 hits coherentes · `pwsh scripts/validate-docs-coverage.ps1` → exit 0.
- **Deuda pendiente:** ninguna.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|---|---|
| `activeGoal` | `# DEF-01: Decisión única de producto + alineación SPEC/README/VISION` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | ACCEPT: reviewer fresco (vanta-review → `reviewer_context` + verdict) o waiver owner para degraded — único pendiente de cierre |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia (comandos ejecutados) |
| `nextTask` | DEF-02 |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (task doc-only; no introduce deuda nueva).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable ✅ (3 docs con misma jerarquía + Gate P registrado + validate-docs-coverage verde) |
| **Commit** | 1 commit atómico `docs:` (3 docs + task file), `git diff` limpio, verificación mecánica registrada (nunca auto-reporte) |
| **Release** | N/A justificado (docs de producto/planning; sin artefacto de release) — ver Notas |

## Herramientas necesarias
- `question` (Gate P owner — Step 2; sin respuesta → STOP)
- `pwsh scripts/validate-docs-coverage.ps1` · `npx markdownlint-cli2 README.md SPEC.md docs/dev/vision/VISION.md`
- `pwsh dev-tools/ocr-review.ps1` (input del review P2-01)

**Skills cargadas (SDP v3, pre-run 2026-09-27):** `campaign-executor` · `progreso` · `writing-guidelines` · `writing-plans` · `incremental-implementation` · `test-driven-development` · `context-engineering` · `source-driven-development` · **PINNED (policy):** ninguno
- `campaign-executor` — flujo pipeline/task system (base).
- `spec-driven-development` — tabla Spec + decisiones.
- `interview-me` — estructura de la ronda Gate P con el owner (keyword mapping "spec").
- `documentation-and-adrs` — estándar de docs.
- `writing-guidelines` — prosa técnica (EN) del README/VISION.

## Investigation Notes
- **Conflicto R4 (hallazgo #1):** README.md:36 posiciona "AI agents, local RAG pipelines, and edge applications" vs SPEC.md:7 "memoria persistente funcionando en su agente de código"; VISION.md:20 "AI agents, RAG pipelines, and structured knowledge applications".
- **Jerarquía propuesta (fuente P54 + Adenda):** núcleo único ("motor de memoria embebido gobernado" = "SQLite para agentes") + 3 puertas de entrada (tracks ICP): `ICP-01` AI-IDEs vía MCP · `ICP-02` local-LLM/privacidad · `ICP-03` frameworks. VISION.md:112-120 ya materializa los 3 tracks; Backlog P54 header: "El núcleo ... es único; los tracks son puertas de entrada, no productos separados".
- **SPEC Adenda 2026-09-24:** decisión #1 (3 tracks ICP) y #4 (North Star) ya registradas; DEF-01 completa la jerarquía explícita y elimina contradicciones de targets.
- **Fuentes de estrategia (solo lectura):** `docs/dev/strategy/ROADMAP.md` es histórico (banner 2026-08-17: "documento histórico") — no editar; `VantaDB_Manual_Estrategico_Unificado.md` es negocio — no editar.
- **Precedente de paridad:** C-10 (2026-09-25) cerró paridad README ES/EN — mantener el patrón.

## Evidence Pack (Step 1 — 2026-09-27)

**Verify ejecutado:** `rg -n "RAG|edge" README.md SPEC.md docs/dev/vision/VISION.md` vía `campaign_verify_cmd` (taskId DEF-01) → exit 0 · 25 hits: **README 4 · VISION 21 · SPEC 0**. El multi-target vive en README/VISION; SPEC no declara RAG/edge como target. `docs/SPEC.md` no existe (canónico: `SPEC.md` raíz). `README_ES.md` existe → paridad obligatoria en Step 3.

**Targets declarados en conflicto (citas file:line):**

| # | Fuente | Cita (extracto) | Target declarado |
|---|--------|-----------------|------------------|
| 1 | `README.md:37` | "designed for AI agents, local RAG pipelines, and edge applications" | agents · RAG · edge (3 en paralelo) |
| 2 | `VISION.md:20` | "for AI agents, RAG pipelines, and structured knowledge applications" | agents · RAG · knowledge apps (≠ README) |
| 3 | `SPEC.md:7` | "memoria persistente funcionando en su agente de código (OpenCode, Claude Code, Cursor, Codex)" | agente de código (único) |
| 4 | `SPEC.md:15` | "Dev que usa agentes de código y quiere que recuerden entre sesiones" | idem (coherente con L7) |
| 5 | `VISION.md:91` | "### Secondary: Knowledge Platform Engineer" | ICP histórico ≠ puertas 2026-09-24 |
| 6 | `VISION.md:103` | "### Tertiary: Local Tool Developer" | ICP histórico (≈ ICP-01 parcial) |
| 7 | `VISION.md:112-114` | "Tracks ICP decididos (decisión owner) … el núcleo es único: motor de memoria embebido gobernado; los tracks son puertas de entrada" | dirección ya decidida (contradicha por L20/L91) |
| 8 | `SPEC.md:118` | "la jerarquía de producto en `VISION.md` (DEF-01)" | SPEC delega la jerarquía en VISION |
| 9 | `README.md:233` | "**New** — MCP server for AI agents" | ICP-01 presente sin jerarquía |
| 10 | `docs/dev/research/product-definition-gap-2026-09-24.md:19-22` | "Son targets distintos … Nada en la definición resuelve cuál manda cuando compiten" + "Resuelto parcialmente … Cierre documental → DEF-01" | conflicto R4 documentado |
| 11 | `docs/dev/Backlog.md:926` | "El núcleo (motor de memoria embebido gobernado) es único; los tracks son puertas de entrada, no productos separados" (decisión owner D1 2026-09-24) | fuente de la jerarquía |
| 12 | `README_ES.md:34` | "diseñado para agentes de IA, pipelines locales de RAG y aplicaciones edge" | espejo ES del conflicto |

**Wording candidato (EN — propuesta NO aplicada; sujeta a Gate P):**

- `README.md:37` (opción a):
  > "VantaDB is a local-first, embedded memory engine for AI agents. One core — durable, governed agent memory with native hybrid retrieval (BM25 + HNSW + RRF), no external services — with three entry points: **AI IDEs** (via MCP), **local/private LLM stacks**, and **agent frameworks**."
- `VISION.md:20` (opción a):
  > "**VantaDB** is an embedded, local-first, transactional agent-memory engine. It unifies documents, vectors, graph relationships, and metadata under a single transactional contract — one core with three entry points: AI-IDEs via MCP, local-LLM/private deployments, and agent frameworks (see §Tracks ICP)."
- `SPEC.md` §Adenda (opción a): una línea — jerarquía 1 núcleo + 3 puertas registrada (referencia a `VISION.md §Tracks ICP`, sin duplicar estrategia).
- RAG pasa a **capacidad** del núcleo (hybrid retrieval), no target; "edge" deja de ser target (caso de deploy, no ICP); `VISION.md:91-110` (Secondary/Tertiary) se sustituye por referencia a la tabla L112-120.

## Gate P — opciones para el owner (Step 2, BLOQUEO 2026-09-27)

> Sin decisión registrada (§Spec fila 1: "⏳ Gate P owner"). Regla dura: sin respuesta **no se editan** SPEC/README/VISION.

| Opción | Contenido | Trade-off |
|---|---|---|
| **(a) 1 núcleo + 3 puertas (recomendada)** | Núcleo = motor de memoria embebido gobernado ("SQLite para agentes"); puertas = ICP-01 AI-IDEs vía MCP · ICP-02 local-LLM/privacidad · ICP-03 frameworks. Elimina RAG/edge/structured-knowledge como targets en README/VISION; RAG queda como capacidad. | Cumple contrato ("sin contradicciones"); el pitch pierde "RAG/edge" como keywords de target |
| **(b) Statu quo multi-posicionamiento** | README/VISION quedan como hoy (RAG/edge/agentes en paralelo); solo se añade nota de tracks. | NO cumple el contrato; contradicción R4 persiste → DEF-01 vacía |
| **(c) Híbrido: (a) + RAG como capacidad declarada** | Igual que (a), conservando "RAG" una vez como capacidad del núcleo (hybrid retrieval / RAG-ready), nunca como ICP. | Conserva discoverability SEO; exige disciplina de redacción para no re-leerse como target |

**Recomendación:** **(a)** — alineada a decisión owner 2026-09-24 (Adenda SPEC #1 + Backlog L926) y al research §1 ("puertas de entrada, no productos separados"). **(c)** aceptable si el owner quiere conservar "RAG" en el pitch.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 1 — jerarquía final (decisión owner vía Gate P) |
| Pendientes de ejecución (downhill) | 4 — Steps 1–4 |
| % completado | 25% (1/4 steps) — Step 2 bloqueado por Gate P |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — no aplica: doc-only, sin trust boundaries ni dependencias nuevas.
- [ ] **PERFORMANCE** — no aplica: sin código ni hot paths.

## Steps

### Step 1: Evidence pack de conflicto + propuesta de jerarquía
- **Archivos:** `SPEC.md`, `README.md`, `docs/dev/vision/VISION.md`, `docs/dev/research/product-definition-gap-2026-09-24.md`
- **Acción:** inventariar citas file:line de targets en conflicto y redactar wording candidato de la jerarquía (1 núcleo + 3 puertas).
- **Verify:** notas con ≥6 citas file:line; `rg -n "RAG|edge" README.md SPEC.md docs/dev/vision/VISION.md` documentado.
- **Estado:** ✅ DONE (2026-09-27) — 12 citas en §Evidence Pack; rg vía `campaign_verify_cmd` → exit 0 (25 hits: README 4 · VISION 21 · SPEC 0).

### Step 2: Gate P owner (decisión de jerarquía)
- **Archivos:** `docs/dev/tasks/DEF-01.md` (registro de la respuesta)
- **Acción:** `question` al owner: opción A (recomendada) vs B + wording; registrar respuesta textual.
- **Verify:** respuesta registrada en Notas; **sin respuesta → STOP/BLOQUEO** (no editar los docs).
- **Estado:** ✅ DONE (2026-09-27) — decisión owner registrada: **opción (c) híbrida** (ver §Notas). Respuesta textual del owner: "opción (c) HÍBRIDA = la opción (a) + 'RAG' se conserva UNA vez como capacidad declarada del núcleo (NO como puerta/target) … Eliminar 'edge'/'structured-knowledge' como targets."

### Step 3: Aplicar jerarquía aprobada en los 3 docs
- **Archivos:** `SPEC.md` (adenda), `README.md` (posición/boundary), `docs/dev/vision/VISION.md`; `README_ES.md` solo paridad si cambió el EN
- **Acción:** escribir la jerarquía aprobada; eliminar targets en conflicto; referenciar `docs/dev/strategy/` como fuente (sin duplicar estrategia).
- **Verify:** `rg -n "<frase clave aprobada>" SPEC.md README.md docs/dev/vision/VISION.md` → 3 hits coherentes; `git diff --check` limpio.
- **Estado:** ✅ DONE (2026-09-27) — wording (c) aplicado: `README.md:37` · `SPEC.md:119` + fila 6 Adenda · `VISION.md:20` + L93 (Secondary/Tertiary superseded) + L193 · paridad `README_ES.md:34`. Verify: anclas "three entry points" (README+VISION, exit 0) · "tres puertas de entrada" (SPEC+README_ES, exit 0) · ausencia de targets RAG/edge/structured-knowledge (0 hits, exit 1 esperado) · `git diff --check` exit 0.

### Step 4: Gates + cierre
- **Archivos:** `docs/dev/tasks/DEF-01.md` (estado/recitation)
- **Acción:** correr gates, verificar links/anclas, registrar evidencia y commit local `docs:`.
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` exit 0 · markdownlint sin errores · OCR sin Critical/High.
- **Estado:** 🟡 gates ✅ / ACCEPT ⏸ — `validate-docs-coverage` exit 0 (0 gaps) · `markdownlint-cli2` 3 canónicos: 0 issues, exit 0 (hallazgo MD028 **preexistente** en `README_ES.md:290` — presente en HEAD, no del diff; ver §Notas) · OCR advisory: `.md` excluidos (`unsupported_ext`) → 0 findings aplicables a DEF-01. **Bloqueo de cierre:** P2-01 sin tool de spawn en el subagente → degradado registrado en §Review; ACCEPT requiere `reviewer_context` fresco (vanta-review) o `waiver {owner, ref}`. Commit local: lo ejecuta el LEAD (el subagente no commitea).

## Dependencias
- **Ninguna dura.** Gate P owner (Step 2) es dependencia de decisión — sin respuesta, BLOQUEO (stop condition del master Task 9).
- Downstream: DEF-05 (North Star), DEF-07 (presupuesto), ICP-01..03 (one-pagers).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` (leaf, no implementa) — fallback: `doubt-driven-development` en contexto fresco.
- **Enfoque:** ¿la jerarquía es la MISMA en los 3 docs (cero targets en conflicto)? ¿La decisión registrada es del owner (Gate P), no una propuesta de la IA? ¿Se respetó el scope (sin marketing)?
- **Cómo se probó:** comandos del contrato ejecutados y registrados (no auto-reporte) + diff solo-docs + OCR delegation sin Critical/High.
- **Checklist anti-hábitos tóxicos** (contrato de comportamiento — el revisor verifica que el implementador NO haya incurrido en ninguno antes de aprobar; fuente §12 de `docs/Investigaciones/2026-08-10-agent-engineering/agent-02-task-execution.md`):
  - [ ] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [ ] No saltarse la clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar contra los acceptance criteria.
  - [ ] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [ ] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [ ] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** 🟡 **DEGRADADO** (2026-09-27) — el subagente ejecutor no dispone de tool de spawn (`task`) → no se pudo invocar `vanta-review` fresco. Se ejecutó el checklist mecánico en modo degradado (declarado, nunca fabricado como fresh). **ACCEPT pendiente:** requiere `reviewer_context` de reviewer fresco (vanta-review) o `waiver {owner, ref}` para `mode:'degraded'` — el server bloquea degraded sin waiver (`state-tools.mjs:144-151`).
- **Ronda 2 (fresca, `ses_f1bcc61e8ffeSO1ODdmBMpVIdN` ≠ autora):** ✅ APPROVE — coherencia 3 fuentes verificada (anchors EN/ES), 0 hits targets en conflicto, paridad EN/ES espejo, VISION sin contradicción, gates re-ejecutados (docs-coverage 0 gaps · markdownlint 0 · diff --check 0), MD028 colateral confirmado preexistente.
- **Checklist degradado (verificación mecánica — replays de evidencia, sin auto-aprobación):**
  - [x] Salidas de comandos reales (verify_cmd/shell; exit codes registrados — sin invenciones).
  - [x] Clarificación: Gate P ejecutado y decisión owner registrada (§Notas: opción c) — no se asumió wording.
  - [x] Acceptance criteria: contrato verificado pieza a pieza (misma jerarquía en 3 docs + paridad + anclas + 0 targets en conflicto).
  - [x] Fallos no ignorados: `exit -1` del 1er anchor test (transitorio) → retry documentado; MD028 preexistente reportado, no ocultado.
  - [x] Búsqueda/evidencia: 3 fuentes (research R4, Backlog P54/L926, SPEC Adenda) + rg multi-doc — no saturado en 1 intento.
  - [x] Citas: toda afirmación con file:line (Evidence Pack, 12 citas).
  - [x] Reintentos con diagnóstico (1 retry anchor; causa: contención de spawn, no bucle).
  - [x] Steps conectados al objetivo (Gate P → decisión → aplicación → gates).
  - [x] Paths money/security: N/A (doc-only).
  - [x] Presupuesto: ~27 tool calls de 40; paradas explícitas.

## Notas
- **Gate P (owner, 2026-09-27) — decisión registrada:** opción **(c) híbrida** = 1 núcleo (motor de memoria embebido gobernado, "SQLite para agentes") + 3 puertas ICP (ICP-01 AI-IDEs vía MCP · ICP-02 local-LLM/privacidad · ICP-03 frameworks); **"RAG" se conserva UNA vez como capacidad declarada del núcleo (no como puerta/target)**; se eliminan "edge"/"structured-knowledge" como targets. Fuente: SARL RESUME DEF-01 (respuesta del owner al question del LEAD).
- **No aplica Fase 1 — Evidencia de Debugging** (tipo Docs, no Bug).
- **Hallazgo colateral (FIND candidato — preexistente):** `README_ES.md:290` MD028 (`blank line inside blockquote`, patrón dos `> [!NOTE]` seguidos) existe en HEAD; **no introducido por DEF-01** (único cambio en ese archivo: L34). Fix trivial pero fuera de scope → el LEAD decide fila `FIND-*`.
- **OCR advisory (2026-09-27):** todos los `.md` de DEF-01 excluidos por `unsupported_ext` → 0 findings aplicables; el único archivo reviewable del workspace es `.github/workflows/perf-bench.yml` (otra wave, fuera de scope DEF-01). Sin Critical/High para DEF-01.
- **Deuda de review:** P2-01 en modo degradado (única vía sin tool de spawn) — ver §Review. Sin waiver del owner, el ACCEPT mecanizado queda bloqueado por diseño (HARD-07).
- **Scope discipline:** no editar `docs/dev/strategy/**` (fuente, no destino), no marketing, no claims de benchmarks (DEF-06), no tocar `README_ES.md` salvo paridad.
- **Release N/A:** docs de producto/planning; el cambio no participa del contrato de release.
- Creado por `vanta-docs` (subagente) desde el master roadmap — estado inicial ⬜ PENDING.

## Context Save Point (2026-09-27)

- **Hecho:** Steps 1–3 ✅ — evidence pack (12 citas) · Gate P (decisión owner: opción c) · jerarquía aplicada en `README.md:37` / `SPEC.md:119`+fila 6 Adenda / `VISION.md:20`+L93+L193 / `README_ES.md:34`. Step 4: gates ✅ (`validate-docs-coverage` exit 0 · markdownlint 3 canónicos 0 issues · OCR sin findings aplicables).
- **Bloqueo de cierre (único pendiente):** ACCEPT mecanizado — sin tool de spawn de subagentes, no se pudo invocar `vanta-review`; review DEGRADADO registrado en §Review. Para cerrar: pasar `reviewer_context` fresco + verdict, o `waiver {owner, ref}` para degraded.
- **Al reanudar (LEAD):** (1) spawnear `vanta-review` y reanudar con `reviewer_context` + verdict → `campaign_update_task_state completed` con `review {mode:'fresh', reviewer_context, author_context:'ses_f1c16a7fbffem7Jcum1RxTWhCy', verdict:'approve'}`; o (2) obtener waiver del owner y cerrar con `degraded`+waiver. Luego: commit local `docs:` (LEAD) con los 5 archivos de DEF-01.
- **Invariantes:** no editar `docs/dev/strategy/**`; no marketing; paridad README_ES mantenida; no tocar archivos de WIRE-10/DEF-02 (waves paralelas presentes en el working tree).
- **Presupuesto:** ~27/40 tool calls; sin stalls.
