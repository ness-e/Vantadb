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
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 1 abierta — jerarquía final (decisión owner, Gate P)
- **Pendientes (downhill):** 4 steps

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
| 1 | Jerarquía de producto | A) 1 núcleo ("SQLite para agentes" = motor de memoria embebido gobernado) + 3 puertas ICP (AI-IDEs MCP · local-LLM/privacidad · frameworks) / B) multi-posicionamiento actual (RAG/edge/agentes en paralelo) | A | ⏳ Gate P owner (Step 2) — propuesta en `Backlog P54` + `SPEC` Adenda #1 |
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
| `nextAction` | Step 1 ⬜ PENDING (evidence pack de conflicto) |
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

**Skills cargadas (SDP):** `SDP: campaign_discover_skills_v2 phase=BUILD → campaign-executor, spec-driven-development, interview-me, documentation-and-adrs, writing-guidelines`
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

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 1 — jerarquía final (decisión owner vía Gate P) |
| Pendientes de ejecución (downhill) | 4 — Steps 1–4 |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — no aplica: doc-only, sin trust boundaries ni dependencias nuevas.
- [ ] **PERFORMANCE** — no aplica: sin código ni hot paths.

## Steps

### Step 1: Evidence pack de conflicto + propuesta de jerarquía
- **Archivos:** `SPEC.md`, `README.md`, `docs/dev/vision/VISION.md`, `docs/dev/research/product-definition-gap-2026-09-24.md`
- **Acción:** inventariar citas file:line de targets en conflicto y redactar wording candidato de la jerarquía (1 núcleo + 3 puertas).
- **Verify:** notas con ≥6 citas file:line; `rg -n "RAG|edge" README.md SPEC.md docs/dev/vision/VISION.md` documentado.
- **Estado:** ⬜ PENDING

### Step 2: Gate P owner (decisión de jerarquía)
- **Archivos:** `docs/dev/tasks/DEF-01.md` (registro de la respuesta)
- **Acción:** `question` al owner: opción A (recomendada) vs B + wording; registrar respuesta textual.
- **Verify:** respuesta registrada en Notas; **sin respuesta → STOP/BLOQUEO** (no editar los docs).
- **Estado:** ⬜ PENDING

### Step 3: Aplicar jerarquía aprobada en los 3 docs
- **Archivos:** `SPEC.md` (adenda), `README.md` (posición/boundary), `docs/dev/vision/VISION.md`; `README_ES.md` solo paridad si cambió el EN
- **Acción:** escribir la jerarquía aprobada; eliminar targets en conflicto; referenciar `docs/dev/strategy/` como fuente (sin duplicar estrategia).
- **Verify:** `rg -n "<frase clave aprobada>" SPEC.md README.md docs/dev/vision/VISION.md` → 3 hits coherentes; `git diff --check` limpio.
- **Estado:** ⬜ PENDING

### Step 4: Gates + cierre
- **Archivos:** `docs/dev/tasks/DEF-01.md` (estado/recitation)
- **Acción:** correr gates, verificar links/anclas, registrar evidencia y commit local `docs:`.
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` exit 0 · markdownlint sin errores · OCR sin Critical/High.
- **Estado:** ⬜ PENDING

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
- **Veredicto:** ⬜ pendiente (✅ approve | ❌ cambios requeridos)

## Notas
- **No aplica Fase 1 — Evidencia de Debugging** (tipo Docs, no Bug).
- **Scope discipline:** no editar `docs/dev/strategy/**` (fuente, no destino), no marketing, no claims de benchmarks (DEF-06), no tocar `README_ES.md` salvo paridad.
- **Release N/A:** docs de producto/planning; el cambio no participa del contrato de release.
- Creado por `vanta-docs` (subagente) desde el master roadmap — estado inicial ⬜ PENDING.
