# DEF-07: Presupuesto de alcance: core-promise vs labs

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 14, Fase F1)
- **Fuente:** Backlog `P55` fila `DEF-07` (L942) + plan Task 14
- **Esfuerzo:** 🟢 1d · **Prioridad:** 🟠 · **Tipo:** Docs (decisión + doc)
- **Turns estimados:** 5-10
- **Creado:** 2026-09-26 · **last-synced:** 2026-09-26
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0 abiertas (criterio = North Star, decidido por evidencia) · **Pendientes (downhill):** 3 steps

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
| `lastAction` | Task file creado; criterio North Star fijado; destinos (EXPERIMENTAL + SPEC) decididos por evidencia |
| `result` | ⬜ pending (`OK` al completar) |
| `nextAction` | Step 1 — inventario de superficies + criterion test |
| `contract` | §Contrato + §Invariantes (verificación: rg core-promise ×2 + validate-docs-coverage) |
| `nextTask` | DEF-08 (Task 15 del master) |

```
=== RECITATION ===
Objetivo activo: DEF-07 — Presupuesto de alcance: core-promise vs labs
Estado: pending
Última acción: task file creado (2026-09-26)
Resultado: —
Próxima acción: Step 1 — inventario de superficies + criterion test (North Star)
Contrato: ver ## Contrato (tabla + regla de inversión + referencia SPEC)
Invariantes: no reabrir jerarquía DEF-01/VISION · criterio trazable por fila · EN en doc técnico
Deuda: ninguna
Próxima tarea si completa: DEF-08
last-synced: 2026-09-26
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
- **Skills cargadas (SDP):** `campaign-executor` (base task system) · `writing-guidelines` (voz/estilo; doc EN) · `documentation-and-adrs` (decisión de frontera registrada — si el owner la considera estructural, evaluar ADR en Review) · `spec-driven-development` (decisión -> contrato verificable en SPEC) · `doubt-driven-development` (anti "clasificar por gusto": verificación adversarial contra North Star) · `interview-me` (solo si una fila borderline no la resuelve el criterio → Gate P owner)
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
| Pendientes de ejecución (downhill) | 3 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — N/A: 100% docs, sin input externo ni dependencias.
- [x] **PERFORMANCE** — N/A: sin hot path; no introduce claims numéricos.

## Steps

### Step 1: Inventario de superficies + criterion test
- **Archivos:** notas de trabajo en este task file (Notas) — sin editar docs aún
- **Acción:** listar TODAS las superficies (core engine/SDK/CLI, Python SDK, MCP, server, embed-local, vanta-memory, proxy, desktop/Studio, web, WASM/TS/Node, providers/integrations, harness/benchmarks) y pasar cada una por el criterion test: ¿sirve directamente a "agente activo recupera memoria en 7d" (core-promise) o es superficie secundaria (labs)? Anotar la evidencia de cada clasificación (1 línea por superficie).
- **Verify:** inventario completo (0 superficies sin clasificar) con criterio citado por fila.
- **Estado:** ⬜ PENDING

### Step 2: Tabla + regla de inversión en `EXPERIMENTAL_FEATURES.md`
- **Archivos:** `docs/user/operations/EXPERIMENTAL_FEATURES.md`
- **Acción:** sección nueva "Scope Budget — core-promise vs labs" (EN): tabla superficie→categoría→disposición (acelerar/congelar) + regla de inversión (qué consume el presupuesto "1 comando, 0 config"; admisión de nuevas superficies como labs por default; promoción solo con evidencia North Star; próximos triggers). Alinear taxonomía con DEF-02 (misma etiqueta por fila).
- **Verify:** sección presente; todas las filas clasificadas; regla de inversión explícita (admisión + promoción + congelamiento).
- **Estado:** ⬜ PENDING · **Dependencia soft:** DEF-05 (North Star formal)

### Step 3: Referencia desde SPEC + cross-checks + cierre
- **Archivos:** `SPEC.md` (§Adenda/Frontera, L118), `docs/user/operations/EXPERIMENTAL_FEATURES.md`
- **Acción:** extender la mención de SPEC L118 para referenciar la tabla Scope Budget; cross-check de 0 contradicciones con VISION (núcleo + 3 puertas) y DEF-01 (si existe); commit.
- **Verify:** `rg -n "core-promise" SPEC.md docs/user/operations/EXPERIMENTAL_FEATURES.md` → hits en ambos; `pwsh scripts/validate-docs-coverage.ps1` exit 0; diff = 2 archivos.
- **Estado:** ⬜ PENDING

## Dependencias
- **DEF-01 (soft):** jerarquía de producto (si ya está, respetarla; si no, no bloquear — la tabla es compatible con "núcleo + 3 puertas").
- **DEF-05 (soft):** North Star es el criterio; si no está formalizada, usar la formulación de la research P55/DEF-05 ("agentes activos que recuperan memoria en 7d").
- **DEF-02 consume esta clasificación** (soft, dependencia inversa).
- Siguiente: DEF-08.

## Review (GATE — agente distinto, P2-01)
- **Revisor:** [PENDIENTE al ejecutar — `vanta-audit`/`vanta-review`, agente distinto al implementador]
- **Enfoque:** ¿alguna clasificación es arbitraria (no trazable al North Star)? ¿contradice VISION/DEF-01? ¿la regla de inversión tiene consecuencias reales?
- **Cómo se probó:** [a poblar: re-ejecución del criterion test sobre 2-3 filas al azar por el revisor]
- **Checklist anti-hábitos tóxicos:** según plantilla
- **Veredicto:** [PENDIENTE — ✅ approve | ❌ cambios requeridos]

## Notas
- Stop condition: ninguna (master Task 14).
- Fila borderline que el criterio no resuelva → registrar en Notas + Gate P owner (no inventar categoría nueva).
- Doc técnico en inglés; este task file en español (planning).
- Commit `docs:` atómico; push owner-gated.
