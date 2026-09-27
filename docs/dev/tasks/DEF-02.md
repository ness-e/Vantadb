# DEF-02: `EXPERIMENTAL_FEATURES.md` regenerado a 0.7.0 + categorías labs

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 10, Fase F1)
- **Fuente:** Backlog `P55` fila `DEF-02` (L937) + plan Task 10
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟠 · **Tipo:** Docs
- **Turns estimados:** 15-30
- **Creado:** 2026-09-26 · **last-synced:** 2026-09-26
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0 abiertas (barrido de verificación, sin decisiones abiertas) · **Pendientes (downhill):** 4 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/user/book/src/operations/EXPERIMENTAL_FEATURES.md` (include `{{#include}}` — 1 línea, no se toca), `README.md` (Product Boundary), `docs/dev/FASE-A.md` (§2 Docs: "Límites declarados"), `SPEC.md` §Adenda/Frontera, `docs/user/QUICKSTART.md` (Current Boundary), **DEF-03** (el gate parsea este doc), N-17 (sync Notion) |
| Callees | `Cargo.toml` (features L149-190, workspace members L741-763, `default-members` L757-763, versión L772 `0.7.0`), `vantadb-mcp/src/handlers/tools.rs` (conteo real de tools), `embeddings/manifest.json` (modelos), `src/llm.rs` (`embed-local`/FIND-100), `src/server/router.rs` (IQL vivo), carpetas `desktop/`, `vanta-proxy/`, `vanta-memory/` |
| Implicaciones | Solo docs (0 código). El doc es **insumo duro de DEF-03**: cada fila "Production-facing" debe mapear a feature/ruta verificable. Riesgo: declarar una fila que el futuro gate no pueda chequear → doc y script deben evolucionar juntos. El espejo del book es un include automático: regenerar solo el archivo principal no lo rompe. |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `docs/user/operations/EXPERIMENTAL_FEATURES.md` (109 L), `docs/user/book/src/operations/EXPERIMENTAL_FEATURES.md` (1 L, include), `Cargo.toml` (features/workspace), `docs/dev/research/product-definition-gap-2026-09-24.md` (§2, fuente P55)
- **Archivos referenciados hacia dentro (imports/includes):** el book incluye el archivo principal vía `{{#include ../../../operations/EXPERIMENTAL_FEATURES.md}}`
- **Archivos que referencian a los editados (grep `EXPERIMENTAL_FEATURES`):** `README.md`, `docs/dev/FASE-A.md:70`, `SPEC.md:118`, `docs/user/QUICKSTART.md:266`, `docs/api/VERSIONING.md` (lista de superficies)
- **Veredicto impacto:** bajo — doc único + include; si cambia la estructura de filas, el único consumidor mecánico es DEF-03 (aún no ejecutada, diseñada después contra el formato final).

## Contrato
"cada fila verificada contra código/feature real (evidencia `file:line` o comando) Y versión 0.7.0 declarada Y categorías labs explícitas Y 0 claims sin respaldo (Regla 11) — `pwsh scripts/validate-docs-coverage.ps1` exit 0 (= 0 gaps) Y `npx markdownlint-cli2 "docs/user/operations/EXPERIMENTAL_FEATURES.md"` 0 errores Y `rg "v0\.1\.x" docs/user/operations/EXPERIMENTAL_FEATURES.md` = 0 hits"

## Spec (SDD — no feature-add: decisiones de doc/frontera)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Versión declarada | A: solo encabezado/frontmatter / B: reemplazar TODAS las refs stale (`v0.1.x` L47/L83) y alinear con workspace | B | ✅ decidido-por-evidencia (research P55 §2: doc stale vs changelog 0.7.0; `Cargo.toml:772`) |
| 2 | Categorías labs | A: sección "Labs" separada / B: etiqueta explícita por fila (columna Boundary) con la misma taxonomía de DEF-07 | B | ⏳ depende DEF-07 (soft) — sin ella, criterio North Star provisional |
| 3 | Evidencia por fila | A: `file:line` / B: comando / C: ambos según tipo (número→comando+fuente; feature→feature/target o `file:line`) | C | ✅ decidido-por-evidencia (Regla 11 exige comando para números) |
| 4 | Fila MCP "~87 tools" | A: dejar "~87" / B: conteo real contra `handlers/tools.rs` | B | ✅ decidido-por-evidencia (`docs/api/MCP.md:536`: 85 post-API-04) |

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** el book sigue apuntando al archivo principal (1 sola fuente); ningún claim numérico sin fuente (Regla 11); ninguna fila promete algo que DEF-03 no pueda verificar contra Cargo/rutas.
- **Comandos de verificación:** `pwsh scripts/validate-docs-coverage.ps1` (0 gaps) · `npx markdownlint-cli2 "docs/user/operations/EXPERIMENTAL_FEATURES.md"` · `rg "v0\.1\.x" docs/user/operations/EXPERIMENTAL_FEATURES.md` (0) · conteo MCP con el mismo regex del script (`"name":\s*"` sobre el bloque `handle_tools_list` de `vantadb-mcp/src/handlers/tools.rs`)
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← valor en este task file |
|------------------------|----------------------------|
| `activeGoal` | DEF-02 — EXPERIMENTAL_FEATURES.md regenerado a 0.7.0 + categorías labs |
| `lastAction` | Task file creado; fixes inmediatos 2026-09-24 ya aplicados; barrido por fila definido |
| `result` | ⬜ pending (`OK` al completar) |
| `nextAction` | Step 1 — barrido de versión v0.1.x → 0.7.0 |
| `contract` | §Contrato + §Invariantes (verificación: validate-docs-coverage + markdownlint + rg) |
| `nextTask` | DEF-03 (Task 11 del master) |

```
=== RECITATION ===
Objetivo activo: DEF-02 — EXPERIMENTAL_FEATURES.md regenerado a 0.7.0 + categorías labs
Estado: pending
Última acción: task file creado (2026-09-26)
Resultado: —
Próxima acción: Step 1 — barrido de versión v0.1.x → 0.7.0
Contrato: ver ## Contrato (validate-docs-coverage 0 gaps + markdownlint + rg v0.1.x=0)
Invariantes: include book intacto · 0 claims sin respaldo · filas verificables por DEF-03
Deuda: ninguna
Próxima tarea si completa: DEF-03
last-synced: 2026-09-26
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda (barrido de verificación; cero código nuevo).

## Definition of Done (contrato multi-nivel — P2-08)
- **Task:** contrato ✅ + markdownlint + `validate-docs-coverage` 0 gaps + 0 claims sin respaldo
- **Commit:** atómico `docs:` + `(DEF-02)`, diff limitado a `docs/user/operations/EXPERIMENTAL_FEATURES.md`
- **Release:** N/A aplicable — tarea 100% docs; el bump lo decide release-plz al cerrar el plan (no tocar versión/CHANGELOG a mano)

## Herramientas necesarias
- `pwsh` (validate-docs-coverage), `npx markdownlint-cli2`, `rg`/grep, `codegraph_explore` (conteo/paridad MCP)
- **Skills cargadas (SDP):** `campaign-executor` (base task system) · `writing-guidelines` (voz/estilo del doc regenerado) · `documentation-and-adrs` (frontera y decisiones documentadas) · `source-driven-development` (cada fila contra código/fuente real, no memoria) · `incremental-implementation` (barrido por tabla en diffs controlados) · `doubt-driven-development` (claims numéricos → verificación adversarial Regla 11)
  - Descartadas del SDP (no aplican): `frontend-ui-engineering`, `test-driven-development`, `context-engineering`, `api-and-interface-design`

## Investigation Notes
- Fixes inmediatos de 4 claims falsos ya aplicados 2026-09-24 (header del doc L15); falta versionar + verificar cada fila.
- Versiones: workspace `0.7.0` (`Cargo.toml:772`); refs stale `v0.1.x` en L47 ("not stable product claims for v0.1.x") y L83 ("outside the v0.1.x MVP").
- Fila MCP dice "~87 tools" (L52); `docs/api/MCP.md:536` documenta 85 tras API-04 (2026-09-25) → contar real desde `handlers/tools.rs`.
- `vanta-proxy` y `vantadb-wasm` fuera de `default-members` con `CATEGORY: EXPERIMENTAL` (`Cargo.toml:757-769`) → insumo directo para categorías labs.
- Mirror book: include de 1 línea; regenerar solo el principal.
- Regla 11: números del doc (p.ej. "9 models (8 ≤3 GB + Qwen3 exception)") deben citar `embeddings/manifest.json`/BENCHMARKS o reformularse.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 4 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — N/A: 100% docs, sin input externo ni dependencias nuevas.
- [x] **PERFORMANCE** — N/A: sin hot path; ningún número nuevo de performance (Regla 11 solo recorta claims).

## Steps

### Step 1: Barrido de versión (v0.1.x → 0.7.0)
- **Archivos:** `docs/user/operations/EXPERIMENTAL_FEATURES.md`
- **Acción:** reemplazar refs stale "v0.1.x" (L47, L83) por "v0.7.0"; actualizar encabezado y `last_reviewed` del frontmatter. NO tocar el include del book.
- **Verify:** `rg "v0\.1\.x" docs/user/operations/EXPERIMENTAL_FEATURES.md` → 0 hits.
- **Estado:** ⬜ PENDING

### Step 2: Verificación por fila — tablas MVP + Optional
- **Archivos:** `docs/user/operations/EXPERIMENTAL_FEATURES.md`
- **Acción:** cada fila (13 Production-facing + 2 Optional) recibe evidencia: feature/target (`Cargo.toml` L149-190), comando o `file:line` (p.ej. embed-local → `Cargo.toml:160` + `src/llm.rs`; server → feature `server` L177-185). Fila MCP pasa al conteo real de `handlers/tools.rs`.
- **Verify:** 0 filas sin evidencia; conteo MCP coincide con lo declarado (mismo regex que `validate-docs-coverage.ps1` §6).
- **Estado:** ⬜ PENDING

### Step 3: Verificación por fila — Experimental/Utilities/Deferred + categorías labs
- **Archivos:** `docs/user/operations/EXPERIMENTAL_FEATURES.md`
- **Acción:** evidenciar las filas restantes (8 Experimental + 3 Utilities + 6 Deferred) y etiquetar proxy/desktop/web/memory con categoría explícita (core-promise vs Labs) alineada a DEF-07; corregir cualquier claim falso residual.
- **Verify:** 0 filas sin evidencia; etiqueta presente en cada fila labs; cross-check con la tabla de DEF-07 (si existe) o criterio North Star provisional.
- **Estado:** ⬜ PENDING · **Dependencia soft:** DEF-07

### Step 4: Barrido Regla 11 + gate-ready + cierre
- **Archivos:** `docs/user/operations/EXPERIMENTAL_FEATURES.md`
- **Acción:** todo número con fuente (comando/BENCHMARKS/manifest) o quitado/reformulado; dry-run manual de los checks que DEF-03 automatizará (cada fila Production-facing mapea a feature/ruta existente); commit.
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps; `npx markdownlint-cli2 "docs/user/operations/EXPERIMENTAL_FEATURES.md"` → 0 errores; `git diff --stat` = 1 archivo.
- **Estado:** ⬜ PENDING

## Dependencias
- **DEF-07 (soft):** la categorización core-promise vs labs alimenta Step 3; sin ella, usar criterio North Star provisional (no bloquear).
- **DEF-03 (Task 11) depende de esta tarea** (doc final parseable por el gate).

## Review (GATE — agente distinto, P2-01)
- **Revisor:** [PENDIENTE al ejecutar — `vanta-audit`/`vanta-review`, agente distinto al implementador]
- **Enfoque:** ¿cada fila es verificable? ¿las categorías no son arbitrarias? ¿hay claims sin fuente?
- **Cómo se probó:** [a poblar: comandos re-ejecutados por el revisor — no auto-reporte]
- **Checklist anti-hábitos tóxicos:** según plantilla (no inventar outputs, no declarar done sin verificar, no ignorar fallos, etc.)
- **Veredicto:** [PENDIENTE — ✅ approve | ❌ cambios requeridos]

## Notas
- Los 4 claim fixes de 2026-09-24 quedan como baseline; esta tarea completa la regeneración.
- No tocar `docs/user/book/**` (include automático).
- Stop condition (master Task 10): >3 filas irreconciliables → FIND + nota, no forzar.
- Commit `docs:` atómico; push owner-gated (política del master §Política de commits/push).
