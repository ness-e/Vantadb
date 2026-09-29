---
title: "DEF-07: Presupuesto de alcance: core-promise vs labs"
kind: task
description: "tabla categorizada (core-promise vs labs) por superficie Y regla de inversión documentada Y referencia desde SPEC — rg -n \"core-promise\" SPEC.md docs/user/operations/EXPERIMENTALFEATURES.md con contenido en ambos Y pwsh..."
---

# DEF-07: Presupuesto de alcance: core-promise vs labs

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 14, Fase F1)
- **Fuente:** Backlog `P55` fila `DEF-07` (L942) + plan Task 14
- **Esfuerzo:** 🟢 1d · **Prioridad:** 🟠 · **Tipo:** Docs (decisión + doc)
- **Turns estimados:** 5-10
- **Creado:** 2026-09-26 · **last-synced:** 2026-09-27
- **Estado:** ✅ COMPLETED (2026-09-27 — review fresco P2-01 ✅ + fixes; commit LEAD)
- **Incógnitas (uphill):** 0 abiertas (criterio = North Star, decidido por evidencia) · **Pendientes (downhill):** 0 (review fresco ✅; commit LEAD)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/user/operations/EXPERIMENTAL_FEATURES.md` (recibe la tabla Scope Budget), `SPEC.md` §Adenda/Frontera (L118 ya nombra "core-promise vs labs" — destino natural de la referencia), **DEF-02** (consume las categorías para las filas labs), one-pagers ICP F5 (definen qué se acelera), `docs/dev/vision/VISION.md` (jerarquía — no puede contradecirla) |
| Callees | `docs/dev/research/product-definition-gap-2026-09-24.md` §3 (fuente del hallazgo), DEF-05 North Star (criterio de clasificación), DEF-01 jerarquía (soft), `docs/dev/strategy/ROADMAP-v0.7.md` (vigente), `GO_TO_MARKET.md` (tracks ICP) |
| Implicaciones | Solo docs. Riesgos: clasificación por gusto → mitiga criterio North Star por fila; contradicción con VISION/DEF-01 (núcleo único + 3 puertas) → invariante explícito; no toca roadmap comercial (solo referencia). Sin impacto en código/CI. |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `docs/dev/research/product-definition-gap-2026-09-24.md` (90 L), `docs/dev/strategy/ROADMAP-v0.7.md` (56 L), `SPEC.md` (122 L), `docs/user/operations/EXPERIMENTAL_FEATURES.md` (109 L)
- **Archivos referenciados hacia dentro:** VISION.md (leído ≥80 L: One-Line Positioning L24/L26, ICP L53+), `ROADMAP.md` (histórico — nota 2026-08-17 L19, NO usar como prioridad), `GO_TO_MARKET.md` (tracks)
- **Archivos que referencian a los editados (grep `core-promise`):** `SPEC.md:118` (ya declara que la frontera "core-promise vs labs" vivirá en `EXPERIMENTAL_FEATURES.md` regenerado por DEF-02/03), master plan Task 14, Backlog filas DEF-02/DEF-07
- **Veredicto impacto:** bajo — doc-only; encaja en el hueco ya declarado por SPEC L118; sin consumidores mecánicos hasta que DEF-03 lo parsee.

## Contrato
"tabla categorizada (core-promise vs labs) por superficie Y regla de inversión documentada Y referencia desde SPEC — `rg -n "core-promise" SPEC.md docs/user/operations/EXPERIMENTAL_FEATURES.md` con contenido en ambos Y `pwsh scripts/validate-docs-coverage.ps1` exit 0"

## Spec (SDD — decisiones de producto/doc)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Criterio de clasificación | A: North Star (¿sirve a que un agente recupere memoria en 7d?) / B: shipped vs no shipped / C: gusto | A | ✅ decidido-por-evidencia (research §3 + pre-mortem Task 14: "criterio = North Star"; DEF-05) |
| 2 | Set de categorías | A: 2 (core-promise / labs) / B: 3+ (agrega frozen/deprecated) | A + columna "disposición" (acelerar / congelar) en la regla de inversión | ✅ decidido-por-evidencia (research §3; filas labs ya existen en EXPERIMENTAL L57-60) |
| 3 | Admisión de superficies nuevas | A: entra directo si "suena útil" / B: entra labs por default; promoción a core-promise requiere evidencia North Star | B | ✅ decidido-por-evidencia (pre-mortem Task 14: "sin consecuencias → regla de inversión explícita") |
| 4 | Dónde vive | A: sólo strategy/ / B: sección en `EXPERIMENTAL_FEATURES.md` + referencia desde SPEC (SPEC L118 ya lo espera ahí) | B | ✅ decidido-por-evidencia (DoD Backlog DEF-07 + DoD master Task 14) |

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** no reabrir la jerarquía "núcleo único + 3 puertas ICP" (decisión owner 2026-09-24, VISION/DEF-01); no prometer superficies no shippeadas; cada fila con criterio trazable (no clasificación ad-hoc); el doc técnico va en inglés (regla de idioma).
- **Comandos de verificación:** `rg -n "core-promise" SPEC.md docs/user/operations/EXPERIMENTAL_FEATURES.md` (hits en ambos) · `pwsh scripts/validate-docs-coverage.ps1` exit 0 · lectura cruzada VISION §One-Line/ICP sin contradicción.
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← valor en este task file |
|------------------------|----------------------------|
| `activeGoal` | DEF-07 — Presupuesto de alcance: core-promise vs labs |
| `lastAction` | Steps 1-3 ✅: inventario 13/13 (criterion North Star); sección "Scope Budget — core-promise vs labs" + regla de inversión en EXPERIMENTAL_FEATURES.md; ref en SPEC §Frontera; verify full verde |
| `result` | OK (review fresco ✅ APPROVE; commit LEAD) |
| `nextAction` | LEAD: commit `docs: scope budget core-promise vs labs (DEF-07)`; cerrar wave F1c |
| `contract` | §Contrato + §Invariantes — verificado: `rg -n "core-promise"` ×2 ✅ · `validate-docs-coverage` exit 0 ✅ · `validate-frontier` exit 0 ✅ · `markdownlint-cli2` 2 docs ✅ · diff docs = SPEC + EXPERIMENTAL |
| `nextTask` | DEF-08 (Task 15 del master) |

```
=== RECITATION ===
Objetivo activo: DEF-07 — Presupuesto de alcance: core-promise vs labs
Estado: completed (review fresco ✅; commit LEAD)
Última acción: Steps 1-3 completos — inventario 13/13; sección "Scope Budget — core-promise vs labs" + regla admisión→promoción→freeze en EXPERIMENTAL_FEATURES.md; SPEC §Frontera referencia la sección
Resultado: ✅ (verify full verde + review fresco APPROVE)
Próxima acción: LEAD — review fresco P2-01 + commit `docs: scope budget core-promise vs labs (DEF-07)`
Contrato: `rg -n "core-promise" SPEC.md docs/user/operations/EXPERIMENTAL_FEATURES.md` ✅ ambos · `pwsh scripts/validate-docs-coverage.ps1` exit 0 ✅ · `pwsh scripts/validate-frontier.ps1` exit 0 ✅ · `npx markdownlint-cli2` 2 docs ✅
Invariantes: no reabrir jerarquía DEF-01/VISION · criterio trazable por fila · EN en doc técnico
Deuda: ninguna
Próxima tarea si completa: DEF-08
last-synced: 2026-09-27
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda (decisión + doc; cierra el hallazgo #3 de la research P55).

## Definition of Done (contrato multi-nivel — P2-08)
- **Task:** contrato ✅ + tabla completa (todas las superficies) + regla de inversión con admisión/promoción
- **Commit:** atómico `docs:` + `(DEF-07)`; diff: `EXPERIMENTAL_FEATURES.md` + `SPEC.md` (solo la referencia)
- **Release:** N/A aplicable — docs sin versionado

## Herramientas necesarias
- `rg`/grep, `pwsh` (validate-docs-coverage), lectura cruzada de strategy/
- **Skills cargadas (SDP v3, pre-run 2026-09-27):** `campaign-executor` · `progreso` · `writing-guidelines` · `writing-plans` · `incremental-implementation` · `test-driven-development` · `context-engineering` · `source-driven-development` · **PINNED (policy):** ninguno
  - Descartadas del SDP (no aplican): `frontend-ui-engineering`, `test-driven-development`, `context-engineering`, `api-and-interface-design`

## Investigation Notes
- Terminología "core-promise vs labs" nace en research P55 §3 ("el alcance real desbordó el MVP declarado") + plan Task 14; "Labs" ya aparece en `EXPERIMENTAL_FEATURES.md` L57-60 (proxy / Vanta Studio / web / vanta-memory).
- `SPEC.md:118` (Adenda 2026-09-24) delega explícitamente la frontera a `EXPERIMENTAL_FEATURES.md` (DEF-02/03) → DEF-07 documenta ahí y SPEC solo referencia.
- VISION: núcleo único + 3 puertas (L24-26, ICP L53+) — la clasificación debe ser compatible, no competir.
- `ROADMAP.md` es histórico (nota L19: fases Sem 1-16 ya ejecutadas) → NO usarlo como fuente de prioridad; vigente: `ROADMAP-v0.7.md`.
- Superficies a cubrir (mínimo): core (engine/SDK/CLI), Python SDK, MCP, server, `embed-local`, vanta-memory, vanta-proxy, desktop/Studio, web (repo externo), WASM/TS/Node, providers/integrations.
- North Star (DEF-05, decisión owner): "agentes activos que recuperan una memoria con éxito en ventana de 7 días".

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 steps (review/commit = LEAD) |
| % completado | 100% (3/3 steps) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — N/A: 100% docs, sin input externo ni dependencias.
- [x] **PERFORMANCE** — N/A: sin hot path; no introduce claims numéricos.

## Steps

### Step 1: Inventario de superficies + criterion test
- **Archivos:** notas de trabajo en este task file (Notas) — sin editar docs aún
- **Acción:** listar TODAS las superficies (core engine/SDK/CLI, Python SDK, MCP, server, embed-local, vanta-memory, proxy, desktop/Studio, web, WASM/TS/Node, providers/integrations, harness/benchmarks) y pasar cada una por el criterion test: ¿sirve directamente a "agente activo recupera memoria en 7d" (core-promise) o es superficie secundaria (labs)? Anotar la evidencia de cada clasificación (1 línea por superficie).
- **Verify:** inventario completo (0 superficies sin clasificar) con criterio citado por fila.
- **Estado:** ✅ COMPLETE (2026-09-27) — inventario 13/13 clasificadas en §Notas (script check: filas=13, clasificadas=13)

### Step 2: Tabla + regla de inversión en `EXPERIMENTAL_FEATURES.md`
- **Archivos:** `docs/user/operations/EXPERIMENTAL_FEATURES.md`
- **Acción:** sección nueva "Scope Budget — core-promise vs labs" (EN): tabla superficie→categoría→disposición (acelerar/congelar) + regla de inversión (qué consume el presupuesto "1 comando, 0 config"; admisión de nuevas superficies como labs por default; promoción solo con evidencia North Star; próximos triggers). Alinear taxonomía con DEF-02 (misma etiqueta por fila).
- **Verify:** sección presente; todas las filas clasificadas; regla de inversión explícita (admisión + promoción + congelamiento).
- **Estado:** ✅ COMPLETE (2026-09-27) — sección "Scope Budget — core-promise vs labs" (13/13 filas + regla admisión→promoción→freeze + watchlist); `pwsh scripts/validate-frontier.ps1` exit 0 (campaign_verify_cmd passed)

### Step 3: Referencia desde SPEC + cross-checks + cierre
- **Archivos:** `SPEC.md` (§Adenda/Frontera, L118), `docs/user/operations/EXPERIMENTAL_FEATURES.md`
- **Acción:** extender la mención de SPEC L118 para referenciar la tabla Scope Budget; cross-check de 0 contradicciones con VISION (núcleo + 3 puertas) y DEF-01 (si existe); commit.
- **Verify:** `rg -n "core-promise" SPEC.md docs/user/operations/EXPERIMENTAL_FEATURES.md` → hits en ambos; `pwsh scripts/validate-docs-coverage.ps1` exit 0; diff = 2 archivos.
- **Estado:** ✅ COMPLETE (2026-09-27) — SPEC §Frontera referencia §Scope Budget; rg hits en ambos ✅; coverage exit 0 ✅ (campaign_verify_cmd); markdownlint 2 docs ✅; cross-check VISION/DEF-01 sin contradicción. Commit: pendiente (LEAD cierra wave; NO se commiteó acá).

## Dependencias
- **DEF-01 (soft):** jerarquía de producto (si ya está, respetarla; si no, no bloquear — la tabla es compatible con "núcleo + 3 puertas").
- **DEF-05 (soft):** North Star es el criterio; si no está formalizada, usar la formulación de la research P55/DEF-05 ("agentes activos que recuperan memoria en 7d").
- **DEF-02 consume esta clasificación** (soft, dependencia inversa).
- Siguiente: DEF-08.

## Review (GATE — agente distinto, P2-01)
- **Revisor:** `vanta-review` fresco (sesión `ses_f1b51d51dffeeRzSJuKJlD8TrM`; ≠ autor) — **✅ APPROVE** (2026-09-27)
- **Enfoque:** criterion test re-ejecutado en filas 7/9/13 + fronteras Studio/server; coherencia VISION/DEF-01/DEF-02; consecuencias de la regla de inversión; gates mecánicos.
- **Cómo se probó:** `rg -n "core-promise"` → SPEC:149 + EXPERIMENTAL:17,19,21,23,25,27,48,98 ✅ · `validate-docs-coverage` → 0 gaps, exit 0 ✅ · `validate-frontier` → 12 rows all resolved, exit 0 ✅ · `markdownlint-cli2` → 0 issues ✅. Evidencia citada verificada por el revisor: `bootstrap.rs:332` (`conversation_trigger: None`), default-members/ADR-0031, gate en CI (`gate-docs.yml:98`).
- **Checklist anti-hábitos tóxicos:** cumplido (0 clasificaciones arbitrarias; evidencia chequeable por fila).
- **Fixes post-review (2026-09-27):** Optional (cláusula "for Cargo features and workspace members" en la regla de admisión) + Nit 2 (excepción "guardrails/evidence layer" en el criterion) aplicados en `EXPERIMENTAL_FEATURES.md`; Nit 1 (`evals/`) aceptado como tooling interno.
- **Veredicto:** ✅ **APPROVE** — 0 Critical / 0 Required / 1 Optional / 2 Nits (fixes aplicados); ACCEPT habilitado (payload review fresh para HARD-07).

## Notas
- Stop condition: ninguna (master Task 14).
- Fila borderline que el criterio no resuelva → registrar en Notas + Gate P owner (no inventar categoría nueva).
- Doc técnico en inglés; este task file en español (planning).
- Commit `docs:` atómico; push owner-gated.

### Step 1 — Inventario de superficies + criterion test (2026-09-27)

**Criterio aplicado (North Star DEF-05):** ¿la superficie sirve *directamente* a "agentes activos que recuperan una memoria con éxito en ventana de 7 días" — o a una de las 3 puertas ICP del núcleo único (DEF-01)? Sí → `core-promise`; no → `labs`. Evidencia citada por fila: 13/13 clasificadas, 0 pendientes.

| # | Superficie | Clasificación | Evidencia de clasificación (criterio aplicado) |
|---|---|---|---|
| 1 | Core engine + Rust SDK + CLI (+ instaladores) | core-promise | put/search + WAL **son** el path de recall; instalador = "1 comando" (SPEC F1/F2) → sirve directo al North Star |
| 2 | Python SDK (`vantadb-py`) | core-promise | QUICKSTART path (EXPERIMENTAL §MVP); puerta ICP-03 (adapters PyPI, VISION §Update) → sirve directo |
| 3 | MCP (`vantadb-mcp`) | core-promise | puerta ICP-01 (VISION §Update: AI-IDEs vía MCP); North Star "medible en proxy/MCP" (DEF-05) → sirve directo |
| 4 | `vanta-memory` (L0→L3) | core-promise | ciclo de vida de memoria = sustancia del North Star (captura→recall→consolidación); SPEC F5 → sirve directo (parcial: host del scheduler, WIRE-01) |
| 5 | `embed-local` | core-promise | la promesa "0 config" exige embeddings locales reales (SPEC F1/F6) → sirve directo |
| 6 | Adaptadores/importers (`integrations/`) | core-promise | puerta ICP-03: adapters PyPI + importers Mem0/Zep (VISION §Update; MKT-18f) → sirve directo |
| 7 | Harness/benchmarks (`benches/`, `benchmarks/`, VER-08/09) | core-promise | capa de evidencia de la promesa + guardrails NS (Regla 11; gate p99; SPEC §Guardrails) → sirve directo (evidencia) |
| 8 | `vanta-proxy` (LLM gateway) | labs | DEF-02 "Category: labs"; research §3 (gateway = overrun). Carve-out declarado: loop de memoria = instrumentación NS (DEF-05) + auto-recall MVP; path redacción → ICP-02; el gateway como producto no sirve directo |
| 9 | `vantadb-server` (HTTP wrapper) | labs | wrapper opcional "local dev / network exposure" (EXPERIMENTAL §Optional); research §3 (JWT/rate-limit = overrun); ningún gate ICP lo requiere; único rol core-adyacente = host del scheduler (`src/server/bootstrap.rs:332`, WIRE-01) → no sirve directo hoy |
| 10 | Vanta Studio (desktop) | labs | DEF-02 "Category: labs"; research §3 (GUI = overrun). El recall del agente no pasa por GUI. Trigger de promoción: viewer mínimo ICP-01 (F5) |
| 11 | Web console (repo externo) | labs | DEF-02 labs; sitio separado `ness-e/Vantadb-web`; sin superficie en este árbol → no sirve directo |
| 12 | WASM/TS/Node (`vantadb-wasm`, `vantadb-ts`, `vantadb-node`) | labs | research §3 (overrun); ICP-03 es Python/PyPI (VISION); wasm fuera de `default-members` (Cargo.toml, ADR-0031) → no sirve directo a los clientes target (4 IDEs) |
| 13 | Providers LLM (`remote-inference`: Ollama/OpenAI/litellm) | labs | EXPERIMENTAL §Experimental: "external optional integration, not core dependency" (alternativa a `embed-local`) → no sirve directo |

**Fronteras (resueltas por el criterio; triggers documentados en la regla — no son filas sin clasificar):**
- **desktop/Studio (fila 10):** el Studio completo = labs; si el Gate P prefiere clasificar ya el "viewer desktop" de ICP-01 como core-promise, es un cambio de 1 fila (trigger escrito en la regla).
- **`vantadb-server` (fila 9):** labs; su único rol core-adyacente (host del scheduler vanta-memory) se promueve por la regla si WIRE-01 rutea por ahí.

## Context Save Point
- **Fecha:** 2026-09-27 · **last-synced:** 2026-09-27
- **Branch:** develop
- **CI pendiente:** no bloqueante (gate-docs.yml correrá lint + frontier al push; nada commiteado — LEAD cierra wave)
- **Decisiones:** clasificación 100% por North Star por fila; proxy/Studio/web = labs (DEF-02 confirmado); `vanta-memory` = core-promise (provisional finalizado); server/WASM/TS/Node/providers = labs; carve-outs con trigger: proxy (loop de memoria = instrumentación NS) · Studio (viewer mínimo ICP-01); harness/benchmarks = core-promise (capa de evidencia)
- **Problemas conocidos:** ninguno; 2 fronteras con trigger documentado (§Notas Step 1 + watchlist del doc)
- **Próxima tarea:** DEF-08 (Task 15)
