# FIND-70: bench en nightly con feature o skip documentado

## Metadata
- **Plan file:** docs/dev/plans/2026-09-15-find-correcciones.md
- **Fuente:** plan file Task 9 (Wave2) + docs/dev/Backlog.md FIND-70
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡
- **Tipo:** CI/CD / DevOps (devops — `campaign_detect_task_type`)
- **Turns estimados:** 5-10
- **Creado:** 2026-09-15
- **last-synced:** 2026-09-15
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 (decisión correr-vs-skip tomada por evidencia: correr)
- **Pendientes (downhill):** 0 steps (2/2 DONE + review approve)
- **Branch:** develop
- **Commit:** `ci: FIND-70 — ...` (tras verify)
- **nextTask:** FIND-87 (Wave3)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | ningún workflow referencia a `heavy-bench-nightly-51.yml` (es entry-point por schedule/dispatch); `analyze` consume sus artefactos vía glob `nightly_*_results.txt` (auto-incluye el nuevo) |
| Callees | `.github/actions/rust-setup`, `scripts/bench_regression.py`, `benches/ingestion_concurrent.rs`, feature `async-ingestion` (tokio sync/macros/rt) |
| Implicaciones | contrato CI no cambia (solo se añade 1 step a job existente); sin cambio de comportamiento público/API/SDK; sin impacto en performance de prod (bench-only); sin migración de datos; tests existentes no afectados (nightly ≠ Fast Gate) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/heavy-bench-nightly-51.yml` (327 líneas, completo), `Cargo.toml:105-135` (features) + `:221-240` (benches incl. `ingestion_concurrent` + `required-features`), `docs/user/operations/BENCHMARKS.md:413-579` (§13 + post-FIND-57 + FIND-61), `.opencode/rules/release-ci.md` (completo), `docs/user/operations/CI_POLICY.md:419-422` (§8 nightly)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** el workflow usa `./.github/actions/rust-setup`, `scripts/bench_regression.py`, benches `hnsw_pure/hybrid_queries/stress_test/bench_concurrent/canonical_p99/memory_budget/incremental_bench/ivf_bench/high_density`; el bench usa `vantadb::ingestion` (feature-gated `async-ingestion`, `src/lib.rs:160`)
- **Archivos que referencian a los editados (referencias entrantes):** `rg "heavy-bench-nightly" .github/ docs/` → solo CI_POLICY §8 (descripción, sin comando que romper); `rg "ingestion_concurrent|async-ingestion" .github/workflows/` → 0 hits (el gap: ningún job lo corre)
- **Veredicto impacto:** bajo — 1 step añadido a job `light-benchmarks` existente + 1 nota docs; `Cargo.toml` PROHIBIDO (solo lectura, intacto); sin `continue-on-error` nuevo (release-ci.md R5); artefacto nuevo auto-recogido por globs existentes

## Contrato

"nightly corre el bench con `--features async-ingestion` (o skip documentado en yml + BENCHMARKS coherente) + `actionlint` OK + Cargo.toml sin cambios (solo lectura)"

## Spec (SDD)

No aplica — no se agregan símbolos/contratos públicos nuevos (Phase 1b: 0 `pub fn`, 0 tools MCP, 0 endpoints, 0 métodos de binding). Cambio CI-only + nota docs.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `Cargo.toml` intacto (prohibido editar); sin `continue-on-error` nuevo sin CATEGORY (release-ci.md R5 / AGENTS.md Regla 2); no tocar `FIND-92/93` (Wave9 batch CI, archivos ajenos); no tocar `.opencode/`, `completions/`, desktop lock (prohibidos); artefactos existentes del nightly intactos (globs `nightly_*_results.txt` + `target/criterion/`)
- **Comandos de verificación:** `actionlint .github/workflows/heavy-bench-nightly-51.yml` (exit 0) + `git diff --check` (limpio) + `rg -n "ingestion_concurrent" .github/workflows/` (≥1 hit) + `git diff --stat` (solo 2 archivos propios)
- **Deuda pendiente:** ninguna (o documentada abajo si el bench con feature no compila → salida skip)

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | FIND-70 — bench en nightly con feature o skip documentado |
| `lastAction` | (se actualiza por step) |
| `result` | PARTIAL (IN PROGRESS, steps pendientes) |
| `nextAction` | Step 1: añadir step + nota docs |
| `contract` | ## Contrato + ## Invariantes de dominio + evidencia |
| `nextTask` | FIND-87 (Wave3) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — cambio CI-only de 1 step + nota docs; no se introduce deuda nueva.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | contrato verificable se cumple (rg hit + actionlint OK) + YAML parse OK |
| **Commit** | commit atómico (~15 líneas), `ci:` conventional, `git diff` solo archivos propios, verificación mecánica |
| **Release** | N/A (CI-only, sin cambio de versión; `verify.ps1` completo no requerido — se corre `verify_changed`-equivalente: actionlint + diff-check) |

## Herramientas necesarias
- actionlint (instalado: WinGet rhysd.actionlint)
- rg (búsqueda), git diff --check
- campaign_verify_cmd (verificación mecánica)

**Skills cargadas (SDP):** ci-cd-and-automation (base type CI/CD + guía de jobs/steps GH Actions) · git-workflow-and-versioning (conventional `ci:`, commit atómico, pre-commit hygiene) · doubt-driven-development (base type, revisión adversarial de correr-vs-skip) · source-driven-development (SDP lifecycle BUILD, sintaxis GH Actions contra docs oficial si hay duda) · base: campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail(full).
SDP nota: `campaign_discover_skills_v2` devolvió además incremental-implementation, test-driven-development, context-engineering, frontend-ui-engineering, api-and-interface-design — descartadas por irrelevantes (sin UI, sin API pública, sin lógica nueva; se registra el descarte en vez de cargarlas a ciegas).

## Investigation Notes

- **Repro del gap:** `rg -n "ingestion_concurrent|async-ingestion" .github/workflows/` → 0 hits en workflows (solo BENCHMARKS.md + Cargo.toml). `Cargo.toml:238-240`: `[[bench]] name="ingestion_concurrent" / required-features=["async-ingestion"]`. Sin la feature, `cargo bench` **salta silenciosamente** ese bench → el gate citado en docs jamás corre en nightly.
- **Docs coherentes con correr:** BENCHMARKS.md `:422-423,491-495,529-530` citan el bench SIEMPRE con `--features async-ingestion` → la salida "correr" no requiere reescritura de comandos, solo 1 nota de cobertura nightly.
- **Prueba local intentada:** `cargo bench -p vantadb --bench ingestion_concurrent --features async-ingestion --no-run -j 2` superó la ventana de 5 min en host Windows frío sin emitir error ni éxito (compile bench-profile release+debuginfo, no conclusivo — sin log versionado). Riesgo residual cubierto por el propio nightly (si rompiera, el nightly lo reporta en rojo; 0 prod afectado). La decisión correr se sostiene sin esa prueba (feature trivial + bench medido en §13/FIND-61).
- **Diseño del fix (ponytail: mínimo):** 1 step `Run ingestion_concurrent benchmarks` en `light-benchmarks` tras `bench_concurrent` (agrupación lógica: benches de concurrencia juntos), comando `cargo bench --bench ingestion_concurrent --features async-ingestion -- --nocapture 2>&1 | tee nightly_ingestion_results.txt` (mismo patrón que los 8 steps vecinos; el `tee` alimenta el glob `nightly_*_results.txt` del upload + `analyze` sin tocar esos jobs). Sin `continue-on-error` (R5). Sin cambios en `analyze`/`critcmp`/`high-density`.
- **Web research:** no requerida (sintaxis GH Actions estándar, mismo patrón que 8 steps vecinos; actionlint como validador mecánico).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — correr-vs-skip resuelto (correr); sintaxis validada por patrón vecino + actionlint |
| Pendientes de ejecución (downhill) | 2 — Step 1 (editar yml + nota docs), Step 2 (verify + commit) |
| % completado | 30% (discovery + task file) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica: sin trust boundaries (sin input de usuario, auth, deps nuevas/bump, storage, FFI, red). Justificación: diff = 1 step YAML + 1 nota docs; 0 dependencias añadidas (`cargo deny`/`audit` no afectados).
- [x] **PERFORMANCE** — no aplica como gate de medición: el cambio no toca hot paths de prod (bench-only, nightly). El bench añadido ES el harness de medición (§13); su coste (~5 min) cabe en `timeout-minutes: 60` existente.

## Steps

### Step 1: Añadir step ingestion_concurrent al nightly + nota BENCHMARKS
- **Archivos:** `.github/workflows/heavy-bench-nightly-51.yml`, `docs/user/operations/BENCHMARKS.md`
- **Acción:** insertar step `Run ingestion_concurrent benchmarks` con `--features async-ingestion` tras el step `bench_concurrent` (:60-61); añadir nota de cobertura nightly en §13 de BENCHMARKS (tras línea de Reproduce, sin tocar comandos/números)
- **Verify:** `rg -n "ingestion_concurrent" .github/workflows/` ≥1 hit + `git diff --stat` solo 2 archivos
- **Estado:** ✅ DONE (3 hits yml:63,67,68; diff 7+4 líneas solo archivos propios)

### Step 2: Verify mecánico + commit atómico
- **Archivos:** (los mismos, sin ediciones nuevas)
- **Acción:** `actionlint` OK + YAML parse OK + `git diff --check` limpio + `git diff Cargo.toml` vacío (prohibido intacto) → `git add` solo los 2 archivos → `git commit -m "ci: FIND-70 — ..."` → `campaign_update_task_state completed` + recitation en plan file
- **Verify:** contrato §Invariantes (4 comandos)
- **Estado:** ✅ DONE (actionlint exit 0 bash directa; `campaign_verify_cmd` exit -1 vacío = bug conocido plan Riesgos; YAML 12 steps incl. ingestion; diff-check limpio en mis files; Cargo.toml vacío)

## Dependencias
- Wave2 con FIND-79/78: disjuntos (archivos TS/README vs workflow/docs-bench) — sin dependencia
- Batch CI Wave9 (FIND-92/93): no tocar esos files
- nextTask: FIND-87 (Wave3)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review (sub-agente distinto, leaf read-only, task ses_f5beac41cffeSD6nO299qs6Bco)
- **Enfoque:** correr-vs-skip correcto (evidencia, no preferencia); step mínimo bien ubicado tras `bench_concurrent`; docs coherente sin tocar números; alternativas descartadas con razón (job separado / `--all-features` global / quitar `required-features` / skip)
- **Cómo se probó:** revisor re-ejecutó: `rg` 3 hits (yml:63,67,68) ✅ · `actionlint` exit 0 ✅ · `git diff --check` limpio ✅ · `git diff --stat -- Cargo.toml` vacío ✅ · YAML parse OK (12 steps) ✅ · globs artefacto L87/153/250 ✅ · sin `continue-on-error` nuevo ✅ · checklist 10 ítems OK
- **Checklist anti-hábitos tóxicos:**
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
- Gate D (question-gates): no dispara — blast radius 2 archivos, contrato mecánico, sin símbolos públicos, sin feature-add (ver Spec).
- Gate P: no dispara — decisión correr-vs-skip resuelta por evidencia dentro del appetite del contrato (skip era salida alternativa explícita del propio contrato).
- `campaign_verify_cmd` tiene bug conocido (exit -1 vacío, plan Riesgos) → evidencia vía bash directa + `campaign_verify_cmd` intentado primero para actionlint.
