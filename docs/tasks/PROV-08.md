# TASK PROV-08: READMEs ×3 completos — tabla 7 métodos, quickstart, requisito pip

## Metadata
- **Plan file:** `docs/plans/2026-08-25-research-providers-quickwins.md`
- **Fuente:** Backlog P45 PROV-08 (INV-providers-01 H-10) — READMEs hoy 5 líneas ("Methods: embed, search, store" cuando hay 7); agregar quickstart + requisito pip + tabla métodos
- **Esfuerzo:** 🟢 ~45min (mecánico docs)
- **Prioridad:** Baja (quickwin Wave 1, mecánico)
- **Tipo:** docs / documentation
- **Turns estimados:** 2
- **Creado:** 2026-08-26T19:00
- **last-synced:** 2026-08-26T19:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | Ninguno — READMEs son docs standalone, no importados por Rust/Python runtime |
| Callees | `providers/openai/src/python.rs`, `providers/ollama/src/python.rs`, `providers/litellm/src/python.rs` (fuente de verdad: 7 pymethods cada uno) |
| Implicaciones | 0 WAL/vector/storage, 0 concurrencia, 0 FFI, 0 unsafe, 0 hot path. Solo markdown. Reversible (overwrite 3 files). No rompe contratos runtime. |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos, antes de editar):**
  - `providers/openai/README.md` (43 líneas HEAD 2754c783) — ya contiene tabla 7 métodos + pip install openai + quickstart
  - `providers/ollama/README.md` (43 líneas HEAD) — tabla 7 + pip install ollama + quickstart
  - `providers/litellm/README.md` (43 líneas HEAD) — tabla 7 + pip install litellm + quickstart
  - `providers/openai/src/python.rs` (349L) — 7 pymethods: embed, search, store, delete, get, list, list_namespaces
  - `providers/ollama/src/python.rs` (358L) — mismos 7
  - `providers/litellm/src/python.rs` (356L) — mismos 7
  - `providers/*/vantadb_*.pyi` (31L c/u) — confirma 7 métodos tipados
  - `docs/plans/2026-08-25-research-providers-quickwins.md` (plan Wave 1 Task 5)
  - `docs/Backlog.md` P45 PROV-08 fila
- **Archivos referenciados hacia dentro (imports/includes):** READMEs no tienen imports; referencian `maturin develop --release` y `pip install` externals
- **Archivos que referencian a los editados (referencias entrantes):**
  - `docs/plans/2026-08-25-research-providers-quickwins.md` Wave 1 Task 5 → gating quickwins
  - `docs/Backlog.md` PROV-08 fila que se elimina via `skill progreso` al completar
  - `docs/reviews/research-providers-20260825.md` H-10 (origen hallazgo)
  - Ningún código importa README (docs only)
- **Veredicto de impacto:** BAJO — 3 archivos markdown, overwrite idempotente. Ya implementado en HEAD (commit 2754c783 2026-08-26). Verify-only en esta corrida.

## Contrato
"README menciona todos los métodos y el requisito pip del SDK proveedor (tabla 7 métodos, quickstart, pip install openai/ollama/litellm)"
- **Cita contrato plan:** `docs/plans/2026-08-25-research-providers-quickwins.md:20` — Task 5 | PROV-08 | READMEs ×3 completos: tabla 7 métodos, quickstart, requisito pip del SDK proveedor | README menciona todos los métodos y el requisito
- **Criterio mecánico:** cada `providers/*/README.md` debe contener: (1) tabla con 7 filas `embed, search, store, get, list, delete, list_namespaces`, (2) bloque `pip install openai|ollama|litellm`, (3) sección `## Quickstart` con `from vantadb_* import VantaDB*`

## Spec
N/A — fix mecánico docs, sin símbolos públicos nuevos (`pub fn`/endpoint/binding). No es feature-add. Contrato mecánico suficiente: presencia de strings verificables (grep). Gate P/D no dispara.

## Invariantes de dominio (handoff - MUST)
- **Invariantes a preservar:** los 7 métodos documentados deben coincidir con `#[pymethods]` reales en `python.rs` + `.pyi`; pip requirement debe nombrar el SDK correcto por provider (openai→openai, ollama→ollama, litellm→litellm); quickstart debe ser copiable y usar `VantaDBOpenAI/VantaDBOllama/VantaDBLiteLLM` con firma real
- **Comandos de verificación:** `Select-String -Path providers/*/README.md -Pattern "pip install (openai|ollama|litellm)"` 3 matches; `Select-String -Pattern "^\| \`(embed|search|store|get|list|delete|list_namespaces)\`"` 7 filas por file; `Get-Content providers/*/README.md | Select-String "Quickstart"` 3 matches
- **Deuda pendiente:** ninguna — docs fix cerrado, 0 deuda (Regla 6 saldo 0)

## Recitation (canónico)
- **activeGoal:** PROV-08 — READMEs ×3 completos (tabla 7 métodos + quickstart + pip)
- **lastAction:** discovery + verify — 3 READMEs auditados: 7/7 métodos, pip install openai/ollama/litellm, quickstart present (commit 2754c783 ya contiene fix)
- **result:** OK
- **nextAction:** cierre verify-only sin commit (per instrucción no commit) + recitation sync plan file
- **contract:** ver ## Contrato; deuda ninguna; invariantes ver arriba
- **nextTask:** PROV-07 (según plan Wave 1 orden: PROV-08 era Task 5/5 wave1; Wave2 inicia PROV-02/09 — próximo pendiente según plan run es PROV-07 si no completado, sino Wave2)

## Deuda técnica (Regla 6 - MUST)
**Saldo neto:** 0 — fix docs ya compensado en 2754c783 (42 líneas añadidas por README, reemplazan "Methods: embed, search, store" incompleto). Esta iteración es verify-only, sin deuda nueva.

## Definition of Done (P2-08)
| Nivel | Gate |
|-------|------|
| **Task** | cada README contiene 7 métodos + pip install correcto + quickstart (grep 3×7 + 3 pip + 3 quickstart) ✅ |
| **Commit** | No requerido en esta corrida (instrucción "no commit"); contenido ya en HEAD 2754c783 `fix(providers): quickwins INV` |
| **Release** | No aplica (docs, no artefacto publicable) |

## Herramientas necesarias
- Read (READMEs x3, python.rs x3, .pyi x3, plan, backlog)
- Grep/Select-String (métodos, pip, quickstart)
- Bash (cargo check opcional — providers ya compilan, no tocado por docs)
- codegraph_explore (blast radius docs — aislado)

**Skills cargadas (SDP):**
- `source-driven-development` — verificar SDK pip correctos contra docs oficiales openai/ollama/litellm (STACK: Python 3.11+, pip packages `openai`/`ollama`/`litellm` — maturin extension)
- `documentation-and-adrs` — README completeness, API table, quickstart structure (Lifecycle SHIP)
- `ponytail` (full) — ladder: existe > stdlib > dep > mínimo código; README overwrite idempotente 42L, sin abstracción, sin config
- `campaign-executor`, `progreso` (base)
- **SDP discovery:** Lifecycle BUILD/SHIP + grep `SKILLS-MANIFEST.md` por `README|provider|pip|quickstart|documentation` → hits: `documentation-and-adrs` (7/10 KEEP), `ci-cd-and-automation` pipeline pero N/A docs, `coordinated-web-search` providers genérico descartado (no necesita web research para docs ya en HEAD), `markdown-documentation` empty dir (no skill). Ningún candidato nuevo más allá de base + source-driven-development + documentation-and-adrs. **SDP: sin candidatos adicionales** (keywords: README, provider, pip, quickstart). **SKILLS_CARGADAS: campaign-executor, progreso, ponytail, source-driven-development, documentation-and-adrs**

## Investigation Notes
- **STACK DETECTED:** `pyo3 0.29` (Cargo.toml providers), `maturin` (README build), Python SDKs vía pip: `openai` (https://pypi.org/project/openai/), `ollama` (https://pypi.org/project/ollama/), `litellm` (https://pypi.org/project/litellm/) — cada README nombra el correcto.
- **Source hierarchy (SDD):** verificación docs no requirió fetch nuevo (pip names son canónicos PyPI); si se requiriera cita oficial, sería `https://github.com/openai/openai-python`, `https://github.com/ollama/ollama-python`, `https://docs.litellm.ai/docs/` — los README ya usan `pip install <name>` exacto, consistente con pypi registry. Patrón `maturin develop --release` sigue https://www.maturin.rs/ (citado en README).
- **Evidencia código:** `git show 2754c783 --stat` — 3 READMEs +42L cada uno (de `**Methods:** embed, search, store` a tabla 7 + Install + Quickstart). Diff auditado línea por línea arriba.
- **Verify mecánico:** `Select-String` 7/7 métodos por file + `pip install openai|ollama|litellm` 3/3 + `Quickstart` 3/3 — todos ✅ en HEAD y worktree (sin diff).
- **Ponytail:** skipped: templating engine README, autogenerador desde python.rs, config extra. Add when READMEs necesiten sync automático (ej: CI que gen desde pymethods). Hoy overwrite manual 42L es mínimo.

## Incógnitas vs Pendientes
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 |
| Pendientes ejecución | 2 → Step 1 audit README (completado), Step 2 verify final + task file sync |
| % completado | 100% (verify-only) |

## Fases explícitas - SECURITY | PERFORMANCE
- [x] **SECURITY** — evaluado: docs only, no trust boundary, no FFI, no input usuario, no deps nuevas → checklist security-and-hardening N/A justificado
- [x] **PERFORMANCE** — evaluado: no toca hot path (vector/engine/search) → benchmark canónico N/A (Regla 9 no aplica)

## Steps
### Step 1: Discovery — auditar READMEs vs contrato (7 métodos + pip + quickstart) ✅ COMPLETED
- **Archivos:** `providers/openai/README.md`, `providers/ollama/README.md`, `providers/litellm/README.md`, `providers/*/src/python.rs` (fuente verdad 7 pymethods), `providers/*/*.pyi`
- **Acción:** Leer completos los 3 READMEs + extraer pymethods reales (`fn embed/search/store/delete/get/list/list_namespaces`) y comparar. Verificar presencia de tabla 7 filas, `pip install openai|ollama|litellm`, `## Quickstart` + `from vantadb_* import`. Documentar match/mismatch.
- **Verify:** grep mecánico 7/7 por file + pip 3/3 + quickstart 3/3 → todos ✅ (HEAD 2754c783 ya fix). Si mismatch → overwrite README (42L) idempotente.
- **Estado:** ✅ COMPLETED (evidence: verify block 7 métodos + 3 pip + 3 quickstart ✅)

### Step 2: Verify final + sync task file y recitation (sin commit) ✅ COMPLETED
- **Archivos:** `providers/*/README.md` (verify), `.opencode/skills/campaign-executor/tasks/PROV-08.md` (este file), `docs/plans/2026-08-25-research-providers-quickwins.md` (recitation)
- **Acción:** Re-correr verify grep final (7 métodos + pip + quickstart) + `git diff HEAD -- providers/*/README.md` debe ser vacío (ya en HEAD). Actualizar este task file estado COMPLETED + sync recitation en plan file si aplica. No commit per instrucción.
- **Verify:** `Select-String` re-confirm 7+3+3 ✅ + `git diff` vacío ✅ + task file `Estado: ✅ COMPLETED` — ejecutado 2026-08-26: Get-Content 3 READMEs (7/7 métodos + pip install openai/ollama/litellm + Quickstart) + git diff vacío
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (PROV-08 independiente Wave 1; comparte Wave con PROV-01/06/03/07; ya implementado permite cierre aislado)

## Review (GATE - agente distinto, P2-01)
- **Revisor:** documentation-and-adrs contexto fresco (docs completeness) + source-driven-development (pip names)
- **Enfoque:** ¿tabla cubre los 7 métodos exactos del pymethods? ¿pip requirement correcto por provider? ¿quickstart copiable con firma real?
- **Cómo se probó:** grep mecánico + lectura directa README + contraste contra `python.rs` signatures y `.pyi`
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos (grep output capturado)
  - [x] No saltarse clarificación (contrato mecánico claro)
  - [x] No declarar done sin verify mecánico
  - [x] No ignorar fallos parciales
  - [x] No single-search — codegraph (blast radius) + grep + git diff
  - [x] No copiar sin citar — cita commit 2754c783 + pypi
  - [x] No reintentar en bucle sin diagnóstico
  - [x] No dejar huérfanos pasos (2 steps atómicos)
  - [x] No degradar chequeo errores
  - [x] No gastar presupuesto infinito (verify-only)
- **Veredicto:** ⏳ pendiente (verify Step 2)

## Notas
- Task ya implementado en HEAD antes de esta corrida (pipeline detecta fix pre-existente, como PROV-06/01/03). Steps son verify-only. Cierre sin commit per instrucción explícita ("no commit").
