# DEF-02: `EXPERIMENTAL_FEATURES.md` regenerado a 0.7.0 + categorías labs

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 10, Fase F1)
- **Fuente:** Backlog `P55` fila `DEF-02` (L937) + plan Task 10
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟠 · **Tipo:** Docs
- **Turns estimados:** 15-30
- **Creado:** 2026-09-26 · **last-synced:** 2026-09-27
- **Estado:** ⏳ IN PROGRESS — steps 4/4 ✅ + contrato ✅; P2-01 fresh review + commit: LEAD
- **Incógnitas (uphill):** 0 abiertas (barrido de verificación, sin decisiones abiertas) · **Pendientes (downhill):** 0 steps

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
| 1 | Versión declarada | A: solo encabezado/frontmatter / B: reemplazar TODAS las refs stale (`v0.1.x` L47/L83) y alinear con workspace | B | ✅ decidido-por-evidencia (research P55 §2: doc stale vs changelog 0.7.0; `Cargo.toml:781`) |
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
| `lastAction` | Regeneración completa: version sweep + evidencia por fila (34 filas) + MCP 85 + labs/core-promise + Regla 11 sweep; gates docs verdes (validate-docs-coverage 0 gaps · markdownlint 0 · rg v0.1.x=0 · dry-run 52 paths/6 features) |
| `result` | 🟡 PARTIAL (steps 4/4 ✅; ACCEPT bloqueado por review gate — degradado registrado) |
| `nextAction` | LEAD: P2-01 fresh review + commit `docs:` (DEF-02); después DEF-03 |
| `contract` | §Contrato + §Invariantes (verificación: validate-docs-coverage exit 0 + markdownlint 0 + rg v0.1.x=0 — todos ✅) |
| `nextTask` | DEF-03 (Task 11 del master) |

```
=== RECITATION ===
Objetivo activo: DEF-02 — EXPERIMENTAL_FEATURES.md regenerado a 0.7.0 + categorías labs
Estado: in-progress (steps 4/4 ✅ + contrato ✅; P2-01 fresh + commit: LEAD)
Última acción: regeneración completa verificada — version sweep, evidencia por fila (34 filas), MCP 85 (47+38 extend), categorías labs/core-promise, Regla 11 sweep, gates docs verdes, dry-run DEF-03 OK, verify.ps1 ALL 10 PASS
Resultado: 🟡 PARTIAL (trabajo ✅; ACCEPT bloqueado por P2-01 fresh — fallback degradado registrado, falta reviewer distinto o waiver owner)
Próxima acción: LEAD — P2-01 fresh review (vanta-review) + commit `docs: … (DEF-02)`; luego DEF-03
Contrato: ✅ validate-docs-coverage exit 0 (0 gaps) + markdownlint 0 + rg v0.1.x=0 + verify.ps1 ALL 10 PASS + 0 claims sin respaldo
Invariantes: include book intacto · 0 claims sin respaldo · filas verificables por DEF-03
Deuda: review P2-01 fresh (no spawneable desde F1a) · findings (a)(b) para routing del orquestador
Próxima tarea si completa: DEF-03
last-synced: 2026-09-27
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
- **Skills cargadas (SDP v3, pre-run 2026-09-27):** `campaign-executor` · `progreso` · `writing-guidelines` · `writing-plans` · `incremental-implementation` · `test-driven-development` · `context-engineering` · `source-driven-development` · **PINNED (policy):** ninguno
  - Descartadas del SDP (no aplican): `frontend-ui-engineering`, `test-driven-development`, `context-engineering`, `api-and-interface-design`

## Investigation Notes
- Fixes inmediatos de 4 claims falsos ya aplicados 2026-09-24 (header del doc L15); falta versionar + verificar cada fila.
- Versiones: workspace `0.7.0` (`Cargo.toml:781`); refs stale `v0.1.x` en L47 ("not stable product claims for v0.1.x") y L83 ("outside the v0.1.x MVP").
- Fila MCP dice "~87 tools" (L52); `docs/api/MCP.md:536` documenta 85 tras API-04 (2026-09-25) → contar real desde `handlers/tools.rs`.
- `vanta-proxy` y `vantadb-wasm` fuera de `default-members` con `CATEGORY: EXPERIMENTAL` (`Cargo.toml:757-769`) → insumo directo para categorías labs.
- Mirror book: include de 1 línea; regenerar solo el principal.
- Regla 11: números del doc (p.ej. "9 models (8 ≤3 GB + Qwen3 exception)") deben citar `embeddings/manifest.json`/BENCHMARKS o reformularse.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 steps |
| % completado | 100% (steps) — gates: P2-01 fresh + commit (LEAD) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — N/A: 100% docs, sin input externo ni dependencias nuevas.
- [x] **PERFORMANCE** — N/A: sin hot path; ningún número nuevo de performance (Regla 11 solo recorta claims).

## Steps

### Step 1: Barrido de versión (v0.1.x → 0.7.0)
- **Archivos:** `docs/user/operations/EXPERIMENTAL_FEATURES.md`
- **Acción:** reemplazar refs stale "v0.1.x" (L47, L83) por "v0.7.0"; actualizar encabezado y `last_reviewed` del frontmatter. NO tocar el include del book.
- **Verify:** ✅ `rg "v0\.1\.x" docs/user/operations/EXPERIMENTAL_FEATURES.md` → 0 hits (exit 1; retry 1: la primera pasada falló porque la nueva nota de revisión reintrodujo el literal "v0.1.x" — corregido a "0.1-era"). Frontmatter `last_reviewed: 2026-09-27`; nota de revisión 2026-09-27 (EN).
- **Estado:** ✅ DONE (2026-09-27)

### Step 2: Verificación por fila — tablas MVP + Optional
- **Archivos:** `docs/user/operations/EXPERIMENTAL_FEATURES.md`
- **Acción:** cada fila (12 Production-facing + 2 Optional) recibe evidencia: feature/target (`Cargo.toml` L149-190), comando o `file:line` (p.ej. embed-local → `Cargo.toml:160` + `src/llm.rs`; server → feature `server` L177-185). Fila MCP pasa al conteo real de `handlers/tools.rs`.
- **Verify:** ✅ 0 filas sin evidencia (script ad-hoc: 34 filas de datos con ≥3 celdas pobladas) Y conteo MCP real: **47 base + 38 extend = 85** (`vantadb-mcp/src/handlers/tools.rs:25`; regex §6 de `validate-docs-coverage.ps1` sobre el bloque base → 47) — "~87" eliminado; citado `docs/api/MCP.md:536`. Corregido `vanta-server` → `vantadb-server` (binario real; `vanta-server` no existe).
- **Estado:** ✅ DONE (2026-09-27)

### Step 3: Verificación por fila — Experimental/Utilities/Deferred + categorías labs
- **Archivos:** `docs/user/operations/EXPERIMENTAL_FEATURES.md`
- **Acción:** evidenciar las filas restantes (8 Experimental + 3 Utilities + 6 Deferred) y etiquetar proxy/desktop/web/memory con categoría explícita (core-promise vs Labs) alineada a DEF-07; corregir cualquier claim falso residual.
- **Verify:** ✅ 0 filas sin evidencia; categorías explícitas: 3× `Category: labs` (proxy/desktop/web) + 1× `Category: core-promise` (vanta-memory, provisional por North Star + default-members; DEF-07 finaliza). Claims falsos corregidos: `vanta-server`→`vantadb-server`; "16 etapas"→lista de etapas verificable (`server.rs`); "testing frontend fuera de CI"→Desktop CI (build + `src-tauri` tests 3 OS; vitest/Playwright sí fuera); RBAC básico existente (`src/rbac.rs`; enterprise/multi-tenancy siguen Deferred); paths de ejemplos muertos (`examples/docker/*`, `langchain_rag.py`) → reales (`docs/dev/archive/docker/`, `examples/python/langchain_ollama_rag.py`); link FUZZING roto → `../../dev/operations/FUZZING.md`; `bootstrap.rs`→`src/server/bootstrap.rs:332`.
- **Estado:** ✅ DONE (2026-09-27) · Cross-check DEF-07: tabla Scope Budget aún no existe (DEF-07 PENDING) → criterio North Star provisional aplicado y anotado en el doc.

### Step 4: Barrido Regla 11 + gate-ready + cierre
- **Archivos:** `docs/user/operations/EXPERIMENTAL_FEATURES.md`
- **Acción:** todo número con fuente (comando/BENCHMARKS/manifest) o quitado/reformulado; dry-run manual de los checks que DEF-03 automatizará (cada fila Production-facing mapea a feature/ruta existente); commit.
- **Verify:** ✅ `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps; `npx markdownlint-cli2 "docs/user/operations/EXPERIMENTAL_FEATURES.md"` → 0 errores; dry-run DEF-03: 52 paths citados + 6 features (`cli/server/embed-local/remote-inference/wal-shipping/advanced-tokenizer`) resuelven (script ad-hoc; se excluyó el path del repo externo); `git diff --stat -- docs/user/operations/EXPERIMENTAL_FEATURES.md` = 1 archivo (60+/57−); `git diff --check` limpio; **`pwsh dev-tools/verify.ps1` → ALL 10 PASS** (fmt/check/clippy/audit/deny/nextest/docs-coverage/cli-probes/consumo-guard/backup — retry 2 tras Gate V autorizado). **Commit: NO ejecutado** (instrucción del orquestador: el LEAD commitea; push owner-gated).
- **Estado:** ✅ DONE (2026-09-27) — commit pendiente del LEAD

## Dependencias
- **DEF-07 (soft):** la categorización core-promise vs labs alimenta Step 3; sin ella, usar criterio North Star provisional (no bloquear).
- **DEF-03 (Task 11) depende de esta tarea** (doc final parseable por el gate).

## Review (GATE — agente distinto, P2-01)
- **Revisor:** ⚠️ NO SPAWNEABLE desde este sub-agente (sin tool de spawn en F1a). Fallback aplicado: `doubt-driven-development` **modo degradado** (mismo contexto — NO sustituye P2-01 fresh). **P2-01 fresh: PENDIENTE (LEAD/owner).**
- **Enfoque:** ¿cada fila es verificable? ¿las categorías no son arbitrarias? ¿hay claims sin fuente?
- **Cómo se probó (degradado, evidencia re-ejecutable):** `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps · `npx markdownlint-cli2 "docs/user/operations/EXPERIMENTAL_FEATURES.md"` → 0 errores · `rg "v0\.1\.x" …` → 0 · row-check ad-hoc (34 filas ≥3 celdas) · MCP base=47 + declared 85 · dry-run DEF-03 (52 paths + 6 features) OK · `git diff --check` limpio · **`dev-tools/verify.ps1` → ALL 10 PASS** (retry 2; lock resuelto con autorización del owner vía Gate V).
- **Hallazgos del pase adversarial (degradado):** 1 corregido en-loop (literal "v0.1.x" reintroducido por la nota → "0.1-era"); 1 corregido en-loop (path externo `docs/user/web/` backtickeado → falsa alarma de path-check → de-tokenizado); riesgo residual aceptado: `vanta-memory` como `core-promise` es provisional (criterio North Star; DEF-07 decide). Nota de entorno: el kill autorizado de procesos para desbloquear `verify.ps1` desconectó el MCP `vantadb` de la sesión (esperado; remediación futura: `CARGO_TARGET_DIR` — lección registrada).
- **Checklist anti-hábitos tóxicos:** ✅ outputs citados son de comandos re-ejecutados (ver §Steps + recitation) · ✅ no se declaró done sin verify · ✅ fallos reportados (lock de `verify.ps1` → Gate V → resuelto con autorización) · ✅ sin claims sin fuente (Regla 11).
- **Veredicto:** ✅ APPROVE (ronda 2 fresca, `ses_f1bd8d0e7ffeRsroAs0UEnwVdx` ≠ autor `ses_f1c16a7f8ffeD8796INlAQVmmF`) — contrato re-ejecutado (docs-coverage 0 gaps · markdownlint 0 · rg v0.1.x 0) · ~15 filas/40 paths spot-checkeados sin fallos · Regla 11 limpio · 2 nits corregidos al commitear · finding (a) book-mirror ruteado al orquestador.

## Notas
- Los 4 claim fixes de 2026-09-24 quedan como baseline; esta tarea completa la regeneración (barrido 2026-09-27).
- No tocar `docs/user/book/**` (include automático).
- Stop condition (master Task 10): >3 filas irreconciliables → FIND + nota, no forzar. **No disparada** (0 filas irreconciliables; 4 claims falsos corregidos inline).
- Commit `docs:` atómico; push owner-gated (política del master §Política de commits/push).
- **Findings (routing → orquestador; NO escritos a Backlog por boundary "Backlog solo vía orquestador"):** (a) book mirror roto pre-existente: `docs/user/book/src/operations/FUZZING.md` incluye `../../../operations/FUZZING.md` (no existe; el archivo vive en `docs/dev/operations/FUZZING.md`) y `mdbook build docs/user/book` falla antes por `../../../TEST_MAP.md` → book sin CI que lo detecte (mdbook v0.5.4 local, build roto); (b) `verify.ps1` (fast-tier) bloqueado 2× por 16 procesos `vanta-cli`/`vantadb-server` en `target\debug` (start 09-26 19:18 → 09-27 13:09). Análisis concurrente de WIRE-10 (lección 2026-09-27): serían ~8 sesiones MCP live del owner — su remediación recomendada es `CARGO_TARGET_DIR=target/session-*` (redirigir el build), no matar. En DEF-02 se terminaron con **autorización explícita del owner vía Gate V** (question) y se re-corrió el gate (ver recitation). Si reaparece el lock en el cierre del LEAD: preferir el redirect de target dir.
- **WIP paralelo en el worktree (NO tocado):** `.github/workflows/perf-bench.yml`, `docs/dev/tasks/DEF-01.md`, `docs/dev/plans/2026-09-26-master-roadmap.md` — wave F1a.
