# HARD-02: Tuning de gates (coverage reporte, review risk-tiered, nightly, release dry-run)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 2, Fase F0)
- **Fuente:** plan file Task 2 (`:74-96`) + **decisión owner 2026-09-26: aplicar (a)+(b)+(c) completo** (coverage→reporte+presupuesto, review risk-tiered, nightly) + R1 (Google eng: coverage lossy; fast <5min vs nightly)
- **Esfuerzo:** 🟡 2d · **Prioridad:** 🔴 · **Tipo:** Mixto (CI + docs + harness `.opencode/`)
- **Ruta:** vanta-lead (+ **Gate H** para partes `.opencode/`)
- **Turns estimados:** 15-25
- **Creado:** 2026-09-26T19:53 · **last-synced:** 2026-09-26T19:53
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 1 abierta (mecanismo/scope del nightly y su notificación) — debe bajar a 0 para ✅
- **Pendientes (downhill):** 7 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `dev-tools/verify.ps1` ← `.githooks/pre-push` + flujo mínimo AGENTS.md + `just verify`; `.config/nextest.toml` ← jobs CI (test/test-windows/coverage) + verify.ps1; `.opencode/task-system/prompts/pipeline-full.md:158` (gate P2-01) ← `/pipeline run`; `CONSTRAINTS.md` ← `constraint-driven-development` + floor-guard |
| Callees | `.github/workflows/ci-rust.yml` (coverage job `:331-403`, ADR-018), `heavy-certification.yml` (semanal `cron: "0 3 * * 0"` `:14-15`), `heavy-bench-nightly.yml` (diario), `dev-tools/floor-guard.ps1`, `dev-tools/gate-common.ps1` (`Get-CoreFeatures`), `docs/dev/workflow/RULES.md` (§1/§2/§4/§6/§7 gobiernan workflows) |
| Implicaciones | Cambia la POLÍTICA de gates (no código de engine). Fast gate local deja de bloquear por coverage → debe compensarse con reporte + presupuesto (CI ADR-018 ≥80% intacto). Nuevo workflow nightly (o schedule diario) → inventario de workflows (`docs/dev/workflow/README.md`/`TRIGGERS.md`) debe actualizarse. Review tiering cambia comportamiento del pipeline (`.opencode/`). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `dev-tools/verify.ps1` (102 L), `.config/nextest.toml` (107 L), `docs/dev/operations/CI_POLICY.md` (480 L), `CONSTRAINTS.md` (161 L), `docs/dev/workflow/RULES.md` (182 L), `.github/workflows/ci-rust.yml` (667 L); parciales: `.opencode/task-system/prompts/pipeline-full.md` (`:120-199`), `heavy-certification.yml` (`:1-80`)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `dev-tools/gate-common.ps1` (`. $PSScriptRoot/gate-common.ps1` en verify.ps1:46); `junit.xml` (profile audit); jobs CI referencian `--profile audit`/`ci-windows` (nextest.toml); `floor-guard.ps1` (CONSTRAINTS.md:23)
- **Archivos que referencian a los editados (referencias entrantes):** `.opencode/AGENTS.md` (flujo mínimo + Pre-Flight), `CONTRIBUTING.md:49-68` (Code Quality/CI gates), `docs/dev/operations/CI_POLICY.md:19-34` (rutas canónicas de scripts), `Justfile` (`just verify`), `.githooks/pre-push`, STABLE-00/ADR-031 (presupuesto `<5min`), plan master `:535-541`
- **Veredicto impacto:** medio-alto — toca el gate que corre en cada push local (verify.ps1) y el pipeline (`pipeline-full.md`); mitigación = floor-guard verde + CONSTRAINTS.md actualizado + CI canónico intacto + Gate H.

## Contrato
"coverage ya no bloquea (reporte + presupuesto por directorio documentado en CI_POLICY) Y review risk-tiered documentado y activo en prompts (solo diffs `docs/api|sdk|parser|storage|wire` → adversarial; resto verify fast) Y workflow nightly de certificación pesada existe Y fast gate medido <5min"

## Spec (SDD — Phase 1b: NO feature-add — no agrega símbolos/contratos públicos; decisiones documentadas)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Alcance del cambio de coverage | A) solo fast gate local (`verify.ps1`) → reporte + presupuesto; CI ADR-018 (≥80%) intacto / B) relajar también el CI | A | ✅ decidido-por-evidencia: `CI_POLICY.md:379-390` ("Nunca bajar el umbral de 80%") + plan `:79` cita solo verify.ps1 (`:49,77-85`) |
| 2 | Mecanismo del presupuesto | A) reporte JSON por directorio + check contra presupuesto (warn no-bloqueante; documentado) / B) solo reporte sin referencia | A | ✅ decidido-por-evidencia: contrato del plan `:82` ("reporte + presupuesto por directorio") |
| 3 | Paths adversariales del tiering | A) lista explícita `docs/api|sdk|parser|storage|wire` (regla mecánica) / B) heurística por tamaño de diff | A | ✅ decidido-por-evidencia: contrato del plan `:82` (lista explícita) |
| 4 | Nightly: archivo nuevo vs extender heavy | A) nuevo `nightly.yml` / B) schedule diario sobre `heavy-certification.yml` (hoy semanal `0 3 * * 0`) — evaluar solape | decidir con evidencia en Step 4 | ⛔ abierta (uphill #1, junto con la notificación) |
| 5 | Notificación de fallo del nightly | A) auto-issue (`gh issue create`, permiso `issues: write`) / B) solo badge | A | ⛔ abierta (uphill #1 — es la incógnita declarada por el plan `:95`) |
| 6 | "release dry-run" (título del plan) | A) evaluar junto al nightly (candidato: `release-plz`/`cargo package --dry-run` sobre crates publicables) / B) diferir | evaluar; si excede appetite → diferir con nota | ⛔ pendiente al ejecutar (el Contrato NO lo exige — ver Notas) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** el **floor** de `CONSTRAINTS.md` no se debilita — `pwsh dev-tools/floor-guard.ps1` exit 0 antes y después; **CI canónico ADR-018 (root crate ≥80%) intacto**; fast gate sigue determinista/offline y **<5min**; toda exclusión nueva de tests requiere tabla + categoría en CI_POLICY (Regla 2, solo vanta-lead); `verify.ps1` sigue siendo el gate del pre-push y no se redefine la jerarquía de gates (AGENTS.md Regla 1); cambios en `.opencode/` → **Gate H (`/harness` verde) ANTES del commit** (y `.opencode/` es repo separado — commit propio).
- **Comandos de verificación:** `pwsh dev-tools/floor-guard.ps1` (exit 0) + `pwsh dev-tools/verify.ps1` (ALL PASS) + `pwsh scripts/validate-docs-coverage.ps1` (para docs) + `/harness` (para `.opencode/`).
- **Deuda pendiente:** FIND-134..147 (CI) y FIND-152 (harness residual) se evalúan/cierran acá si siguen vigentes (plan `:96`).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# HARD-02: …` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | HARD-03 |

```
=== RECITATION ===
Objetivo activo: HARD-02 — tuning de gates (coverage reporte, review risk-tiered, nightly)
Estado: pending
Última acción: task file creado (Plan: master-roadmap Task 2; decisión owner a+b+c)
Resultado: —
Próxima acción: Step 1 — coverage fast gate a reporte+presupuesto (verify.ps1)
Contrato: ver ## Contrato
Invariantes: floor-guard verde; ADR-018 intacto; fast gate <5min; Gate H en .opencode/
Deuda: ninguna al abrir
Próxima tarea si completa: HARD-03
last-synced: 2026-09-26
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda neta — la relajación del gate local se compensa con (1) presupuesto por directorio verificable y documentado, (2) CI ADR-018 intacto, (3) floor-guard verde + registro de la decisión owner en CONSTRAINTS.md. Sin suppressions nuevas.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato (4 condiciones) + `floor-guard.ps1` exit 0 + `verify.ps1` ALL PASS + Gate H verde (partes `.opencode/`) |
| **Commit** | Atómico(s) por bloque (verify.ps1 · prompts `.opencode/` · nightly workflow · docs); `.opencode/` en su repo separado; conventional (`ci:`/`docs:`/`chore:`); sin push (Regla 7) |
| **Release** | N/A (política/CI, no versiona) — justificado en Notas. Si el nightly incorpora "release dry-run", se documenta ahí su relación con release-plz (solo dry-run, jamás publica). |

## Herramientas necesarias

- `pwsh dev-tools/verify.ps1` / `floor-guard.ps1` / `gate-common.ps1`, cargo-llvm-cov (`--json` para presupuesto), actionlint (si disponible; fallback parse YAML), `gh` CLI (workflows/issues), `campaign_verify_cmd`, codegraph_explore
- Leer ANTES de editar: `.opencode/rules/release-ci.md` (CI/release), `.opencode/rules/concurrency-async.md` no aplica; `.opencode/AGENTS.md` (Gate H + Regla 1/2), `.opencode/references/floor-guard.md` (floor)

**Skills cargadas (SDP v3, pre-run 2026-09-27):** `campaign-executor` · `progreso` · `ci-cd-and-automation` · `git-workflow-and-versioning` · `doubt-driven-development` · `incremental-implementation` · `test-driven-development` · `context-engineering` · **PINNED (policy):** `ci-cd-and-automation`, `git-workflow-and-versioning`

## Investigation Notes

- `verify.ps1:49` `$CoverageThreshold = 60` (P2-06) y `:79` step bloqueante `--fail-under-lines 60`; si falta llvm-cov, skippea con warning (`:83-84`). El cambio owner (a) es SOLO acá (fast gate local).
- CI tiene su propio coverage job (`ci-rust.yml:331-403`) con gate ≥80% root (ADR-018, `CI_POLICY.md:379-390`): **no se toca**.
- `heavy-certification.yml` YA corre semanal (`cron: "0 3 * * 0"`, `:14-15`) y `heavy-bench-nightly.yml` corre diario → el "nightly de certificación pesada" puede solaparse; decidir con evidencia (Step 4) si es archivo nuevo o schedule diario extendido, y qué subset corre (candidatos: gates lentos derivados de HARD-01).
- `pipeline-full.md:158` contiene el gate Review P2-01 → ahí (y coherente con `task.md` Fase 5) va el tiering.
- `CONSTRAINTS.md:23` referencia floor-guard (`dev-tools/floor-guard.ps1` existe, `:122` "VantaDB is at 2"); `:36-37` filas de coverage + ratchet 68.2% → actualizar con la decisión owner y `Last reviewed`.
- Reglas de workflows aplicables (RULES.md): §1 triggers, §2 `timeout-minutes` en todos los jobs, §4 permisos mínimos (auto-issue = `issues: write`), §6 `continue-on-error` con `CATEGORY`, §7 `pull_request` con `branches`. Inventario: `docs/dev/workflow/README.md` (27 files) + `TRIGGERS.md`.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 1 — mecanismo/scope del nightly + su notificación (se resuelve en Step 4 con evidencia de solape y permisos) |
| Pendientes de ejecución (downhill) | 7 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — Cambio de permisos CI a declarar: `issues: write` SOLO en el job/step de notificación del nightly (permiso mínimo por RULES.md §4; sin secrets, publish sigue OIDC/tokenless). No toca trust boundaries de producto.
- [x] **PERFORMANCE** — El cambio ES de presupuesto/tiempo: medir fast gate (`Measure-Command { pwsh dev-tools/verify.ps1 }`) y registrar <5min (Step 6); nightly recibe lo que no cabe. Sin hot paths de producto.

## Steps

### Step 1: Coverage fast gate → reporte + presupuesto (verify.ps1)
- **Archivos:** `dev-tools/verify.ps1` (`:49`, `:77-85`), `.config/nextest.toml` (solo si el reporte por directorio requiere profile/filtros — justificar si no se toca)
- **Acción:** reemplazar el step bloqueante (`--fail-under-lines 60`) por: ejecutar llvm-cov + emitir reporte JSON/resumen por directorio y comparar contra el presupuesto (paso 2). Por defecto **warn no-bloqueante**; el fail queda reservado a violaciones explícitas del presupuesto.
- **Verify:** `pwsh dev-tools/verify.ps1` ALL PASS (coverage en modo reporte/presupuesto) + `pwsh dev-tools/floor-guard.ps1` exit 0
- **Estado:** ⬜ PENDING

### Step 2: Presupuesto por directorio con evidencia + registro owner
- **Archivos:** `docs/dev/operations/CI_POLICY.md` (§Coverage `:313-390`), `CONSTRAINTS.md` (`:36-37`, `Last reviewed`)
- **Acción:** medir coverage actual por directorio (`cargo llvm-cov report --json` → agregación); fijar presupuesto por directorio (ratchet: no perder más de X pt rel. a baseline); documentar tabla (directorio | baseline | presupuesto | comando) en CI_POLICY; registrar la decisión owner en CONSTRAINTS.md (explícito: CI ADR-018 intacto; floor-guard sigue verde).
- **Verify:** números reproducibles con el comando citado + `pwsh dev-tools/floor-guard.ps1` exit 0 + `rg` de la tabla en CI_POLICY
- **Estado:** ⬜ PENDING

### Step 3: Review risk-tiered en prompts (Gate H)
- **Archivos:** `.opencode/task-system/prompts/pipeline-full.md` (`:158`), `.opencode/task-system/prompts/task.md` (Fase 5, si hace falta coherencia)
- **Acción:** publicar la tabla de tiering — diffs que tocan `docs/api|sdk|parser|storage|wire` → review adversarial obligatorio (vanta-review/vanta-audit/doubt); resto → verify fast mecánico. Incluir caso de prueba (2 ejemplos path→tier) que el reviewer pueda contrastar.
- **Verify:** `/harness` verde (Gate H) + caso de prueba documentado (ej.: `src/sdk/**` → adversarial; `docs/**` → fast)
- **Estado:** ⬜ PENDING · **GATE H: SÍ** (`.opencode/` — `/harness` ANTES del commit; repo separado)

### Step 4: Nightly de certificación pesada + notificación (resuelve uphill #1)
- **Archivos:** `.github/workflows/` (nuevo `nightly.yml` **o** schedule diario extendiendo `heavy-certification.yml` — decidir con evidencia), `docs/dev/workflow/README.md` + `TRIGGERS.md` (inventario/trigger matrix), `docs/dev/operations/CI_POLICY.md` (sección)
- **Acción:** definir el lane nocturno (subset de certificación pesada + gates lentos derivados de HARD-01 si aplica); notificación en fallo (auto-issue con permiso mínimo `issues: write`) + badge; evaluar solape con el semanal existente y `heavy-bench-nightly.yml` para no duplicar.
- **Verify:** YAML válido (actionlint si disponible; fallback parse) + `timeout-minutes` en todos los jobs (RULES §2) + permisos mínimos declarados (RULES §4) + trigger documentado en `TRIGGERS.md`/CI_POLICY (RULES §7) — verificación remota post-push = owner lane
- **Estado:** ⬜ PENDING

### Step 5: FIND-134..147 + FIND-152 — triage/vigencia
- **Archivos:** `docs/dev/Backlog.md` (filas), `docs/dev/operations/CI_POLICY.md` (si cierra hallazgos), `.opencode/references/` (FIND-152)
- **Acción:** evaluar vigencia uno a uno; cerrar con evidencia (archivo/commit) o rutear; actualizar filas del Backlog (fuente única de hallazgos).
- **Verify:** `rg "FIND-13[4-9]|FIND-14[0-7]|FIND-152" docs/dev/Backlog.md` → cada uno con disposición (cerrado/ruteado) o justificación
- **Estado:** ⬜ PENDING

### Step 6: Fast gate <5min — medición + registro
- **Archivos:** `docs/dev/operations/CI_POLICY.md` (tabla de mediciones, patrón STABLE-08 `:230-248`)
- **Acción:** `Measure-Command { pwsh dev-tools/verify.ps1 }` (warm; cold documentado si aplica); registrar entorno + wall time; si >300s → FIND + análisis (no silencio).
- **Verify:** medición <300s registrada con comando + entorno; >300s → FIND creado
- **Estado:** ⬜ PENDING

### Step 7: Cierre (contrato + Gate H + commit local)
- **Archivos:** — (todo el set)
- **Acción:** correr contrato completo; `verify.ps1`; `/harness` (por `.opencode/`); recitation; commits locales (VantaDB + repo `.opencode/` por separado); learnings vía `campaign_memory_write`.
- **Verify:** Contrato (4 condiciones) + `pwsh dev-tools/verify.ps1` ALL PASS + `/harness` verde
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (F0). Coordina con **HARD-01** (recepción de gates lentos >5min en nightly), **HARD-07** (tiering de política ↔ mecanización del invariante REVIEW→ACCEPT) y **HARD-06** (FINDs de CI/área compartida: FIND-134..147 se cierran acá, FIND-153/154 en HARD-06). Siguiente: HARD-03.

## Review (GATE — agente distinto, P2-01)

> Placeholder. Lo completa un agente DISTINTO al implementador al ejecutar la tarea (vanta-review/vanta-audit; fallback sin subagentes: doubt-driven-development degradado + escalado al owner). Sin review registrado, NO se marca ✅ COMPLETED.

- **Revisor:** ⬜ (designar al ejecutar — distinto del implementador)
- **Enfoque:** ¿el tuning debilita el quality bar o lo compensa con presupuesto verificable? ¿el tiering es mecánico (lista de paths) y no ambiguo? ¿el nightly aporta señal y no rojo crónico?
- **Cómo se probó:** ⬜ (pegar: floor-guard exit 0, medición <5min, caso de prueba del tiering, YAML/actionlint del nightly)
- **Checklist anti-hábitos tóxicos** (contrato de comportamiento — el revisor verifica que el implementador NO haya incurrido en ninguno antes de aprobar):
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
- **Veredicto:** ⬜ pendiente

## Notas
- **Gate H obligatorio** para Steps 3 y cualquier edición en `.opencode/` (repo separado `configOpencode` — ignorado por VantaDB git; el commit del harness es aparte). Si `/harness` rechaza → rediseñar (no forzar).
- Stop conditions del plan: floor-guard rojo → revertir coverage a bloqueante; appetite >3d → solo (b)+(c).
- FIND-134..147 (CI) y FIND-152 (harness residual) se evalúan/cierran acá si siguen vigentes (plan `:96,531`).
- Título menciona "release dry-run"; el Contrato NO lo exige → evaluar como candidato del nightly/job (solo `--dry-run`/`cargo package --dry-run`; NUNCA publica); si excede appetite, diferir con nota.
- `verify.ps1` cambia en local únicamente: el CI canónico (ADR-018) y el presupuesto `<5min` (ADR-031/STABLE-00) quedan como están.
