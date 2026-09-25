# FIND-141: Desolapar schedules pesados + guard auto-push + retention corpus fuzz

## Metadata
- **Plan file:** docs/dev/plans/2026-09-21-workflows-repair.md
- **Fuente:** plan Wave 2 §FIND-141 (líneas 56-58)
- **Esfuerzo:** 🟡 1d (3 archivos, 4 líneas, verify mecánico)
- **Prioridad:** 🟡
- **Tipo:** CI/CD / DevOps (YAML workflows, cero Rust)
- **Turns estimados:** 8
- **Creado:** 2026-09-22
- **last-synced:** 2026-09-22
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 5 steps (S1-S5)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | GitHub Actions schedulers (cron UTC); ningún módulo Rust llama estos YAML |
| Callees | `.github/workflows/ci-gate.yml` (reusable, solo lectura `checks: read`); `scripts/bench_regression.py`; `benchmarks/criterion_baseline.json`; `fuzz/corpus/*` |
| Implicaciones | contrato público NO cambia; performance N/A (solo horarios/retención); sin migración de datos; tests afectados: ninguno (solo schedule/push/artifact) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/heavy-certification-50.yml` (313L), `.github/workflows/heavy-bench-nightly-51.yml` (334L), `.github/workflows/fuzz-40.yml` (169L), `.opencode/rules/release-ci.md` (42L completo), `.opencode/references/definition-of-done.md` (completo), `SPEC.md` raíz (108L)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `ci-gate.yml` (reusable workflow, ambos heavy + fuzz); `rust-setup` action; `scripts/bench_regression.py` + `verify_datasets.sh` + `download_benchmark_datasets.sh`; `benchmarks/criterion_baseline.json` (auto-push target); `actions/cache` (fuzz corpus) + `actions/upload-artifact` (bench + fuzz)
- **Archivos que referencian a los editados (referencias entrantes):** ningún workflow referencia a estos 3 por `uses:` (verificado: `heavy-cert`, `heavy-bench`, `fuzz-40` no son reusable targets); badges README no citan estos 3 (rustdoc era FIND-137, no aplica)
- **Veredicto impacto:** BAJO — 3 archivos disjuntos, 1 línea (cron) + 1 línea (commit msg) + 2 líneas (retention), cero lógica, cero refactors

## Contrato

"0 solapes heavy-cert vs heavy-bench (crons distintos verificados por grep); `actionlint` 3 files exit 0; `git diff --check` limpio; auto-push con `[skip ci]` sin loop; fuzz con `retention-days` explícito en ambos uploads"

## Spec (SDD — Phase 1b)

> No es feature-add: cero `pub fn`/tools/endpoints/bindings/componentes/capabilities nuevas. Solo horarios, mensaje de commit y retención. Tabla N/A válida.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Qué schedule mover | A: cert Sun 03:00→04:00 (1 trigger/sem, pero colisiona con ocr-nightly daily 04:00 prohibido) / B: bench daily 03:00→02:00 (7 triggers/día, slot 02:00 vacío verificado) | B | ✅ decidido-por-evidencia (grep cron: 6 schedules; `0 4 * * *` ocupado por ocr-nightly.yml:21; `0 2 * * *` libre) |
| 2 | Guard anti-loop auto-push | A: `[skip ci]` en mensaje (estándar GitHub, 1 línea) / B: `if:` con check de actor + mensaje (más código) | A | ✅ decidido-por-evidencia (bench sin trigger `push` — el riesgo es CI general en push a main; `[skip ci]` lo cubre; B es refactor prohibido) |
| 3 | Retention fuzz | A: 14 días (2 runs semanales, artefactos por run_id efímeros) / B: 30 días (consistencia con bench nightly) | A 14 | ✅ decidido (weekly Mon 06:00 → 14 retiene ~2; crashes con 14d ventana triage suficiente; bench 30d necesita tendencia larga, fuzz no) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** resto workflows intactos (FIND-140/146 en paralelo: release-*, arch-metrics, ocr-*, opencode.yml); `src/`, `web/`, `desktop/`, `reparacion.bat`, `.opencode`, `completions/*`, `*.lock`, stash GOV-C4, `docs/dev/Backlog.md`, plan file (solo recitation) intactos; `C:/Users/Eros/.vantadb*` y secretos nunca tocados; `continue-on-error: true` de bench (línea 167, CATEGORY INFORMATIONAL) intacto; concurrency bench (`cancel-in-progress: false`) intacta; cache keys fuzz intactas
- **Comandos de verificación:** `actionlint .github/workflows/heavy-certification-50.yml .github/workflows/heavy-bench-nightly-51.yml .github/workflows/fuzz-40.yml` (exit 0); `git diff --check` (limpio); `git diff --staged \| grep -i "password\|secret\|api_key\|token"` (vacío); grep cron post-edit (crons distintos)
- **Deuda pendiente:** ninguna (colateral adapters-compat Sun 03:00 pre-existente, fuera de scope, solo nota — no bloquea)

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | # FIND-141 encabezado |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | ## Contrato + ## Invariantes + evidencia/artefactos |
| `nextTask` | FIND-146 (Wave 2 paralelo, orquestador) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda

> Solo horarios/mensaje/retención. No se introduce `unsafe`, `clone` hot-path ni deuda nueva. Colateral adapters-compat Sun 03:00 (pre-existente, ver Notas) no se toca → no se registra como deuda nueva del PR.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable cumple + `git diff --check` + `actionlint` 3 files + grep cron sin solape pesado |
| **Commit** | Commit atómico (~6 líneas), `ci: FIND-141 — ...`, `git diff` solo 3 workflows + task file, verificación mecánica (nunca auto-reporte) |
| **Release** | N/A (tarea CI sin release; NO PUSH; justifica en Notas) |

## Herramientas necesarias

- actionlint (verify contrato)
- git diff --check + git status/diff (staging selectivo)
- campaign_verify_cmd (intento; BUG exit -1 conocido → fallback bash directa + mención)

**Skills cargadas (SDP):** `ci-cd-and-automation` (pipeline CI, quality gates, schedules como triggers) + `git-workflow-and-versioning` (conventional `ci:`, commits atómicos, save-point). SDP tool `campaign_discover_skills_v2` phase=BUILD keywords=[github-actions,schedule,cron,fuzz,concurrency,cache] devolvió 8 (campaign-executor, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, source-driven-development, frontend-ui-engineering, api-and-interface-design); se cargan las 2 aplicables a CI-YAML (resto descartadas: TDD/frontend/API son para código, no YAML — ponytail).

## Investigation Notes

- Crones pre-edit (grep `cron:` en `.github/workflows`, 6 hits): `adapters-compat.yml` `0 3 * * 0` (Sun 03:00); `fuzz-40.yml` `0 6 * * 1` (Mon 06:00); `heavy-bench-nightly-51.yml` `0 3 * * *` (daily 03:00); `heavy-certification-50.yml` `0 3 * * 0` (Sun 03:00); `ocr-nightly.yml` `0 4 * * *` (daily 04:00); `sec-codeql-30.yml` `0 0 * * 0` (Sun 00:00). Solape objetivo: bench-daily vs cert-Sun cada domingo 03:00 UTC confirmado.
- Elección documentada (Spec #1): se mueve bench → `0 2 * * *` (slot vacío, sin colisión con ocr-nightly). Alternativa cert→04:00 descartada por colisión con `ocr-nightly` daily 04:00 (archivo prohibido FIND-146). Cron `0 2 * * *` válido 5-campos (min 0, hora 2, diario).
- Push step (bench líneas 259-267): `update-baseline` → `git config` bot → `git add benchmarks/criterion_baseline.json` → `git commit -m "chore(ci): ..."` → `git push`. Sin trigger `push:` en bench (solo schedule/pull_request/workflow_dispatch) pero el push a main dispara CI general (Regla 1) → `[skip ci]` es el guard estándar.
- Cache fuzz (líneas 88-94 + 145-151): `actions/cache@v6.1.0` path `fuzz/corpus/${{ matrix.target }}`, key `fuzz-corpus-${{ matrix.target }}-${{ github.ref_name }}`, restore prefix. Verificada: NO se toca (contrato solo pide verificarla antes de añadir retention a artifacts).
- Uploads fuzz sin retention (líneas 107-115 job `fuzz`, 161-169 job `fuzz-pr`): sin `retention-days` → default 90d. Se añade `retention-days: 14` a ambos.
- Web research: N/A (cron `[skip ci]`/retention son sintaxis estable GitHub Actions; sin ambigüedad de API externa; internet N/A por instrucción salvo docs oficiales — no requerido).
- Gates P/D: no disparado (blast 3 files <10, sin hot path/API pública/símbolos nuevos, contrato claro, no feature-add).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — approach validado, slots verificados, sintaxis conocida |
| Pendientes de ejecución (downhill) | 5 — S1 cron, S2 skip-ci, S3 retention ×2, S4 verify, S5 commit |
| % completado | 10% (discovery + task file) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — N/A justificado: sin trust boundaries (sin input usuario/auth/sesiones/dependencias nuevas/storage/FFI/red). Cambio YAML horarios+mensaje+retención. Secrets-grep en S4 como hygiene. Skill `security-and-hardening` no aplica.
- [x] **PERFORMANCE** — N/A justificado: sin hot paths (no `vector/`/HNSW/engine/loops/serialización). Desolape reduce contención de runners (beneficio incidental, no benchmark). `canonical_p99` N/A. Cargo N/A.

## Steps

### Step 1: Desolapar schedule bench 03:00 → 02:00
- **Archivos:** `.github/workflows/heavy-bench-nightly-51.yml` (línea 5)
- **Acción:** `cron: '0 3 * * *'` → `cron: '0 2 * * *'` (1 línea; cert Sun 03:00 intacto)
- **Verify:** `grep -n "cron:" .github/workflows/heavy-bench-nightly-51.yml .github/workflows/heavy-certification-50.yml` (crons distintos)
- **Estado:** ✅ COMPLETED (bench `0 2 * * *`, cert `0 3 * * 0`)

### Step 2: Guard [skip ci] en auto-push baseline
- **Archivos:** `.github/workflows/heavy-bench-nightly-51.yml` (línea 266)
- **Acción:** `git commit -m "chore(ci): update criterion bench baseline (nightly)"` → `... (nightly) [skip ci]"` (1 línea; sintaxis push intacta)
- **Verify:** `grep -n "skip ci" .github/workflows/heavy-bench-nightly-51.yml` (1 hit) + lectura push step (config/add/commit/push orden intacto)
- **Estado:** ✅ COMPLETED ([skip ci] línea 266, push step intacto)

### Step 3: Retention explícita corpus fuzz (ambos uploads)
- **Archivos:** `.github/workflows/fuzz-40.yml` (bloques 107-115 y 161-169)
- **Acción:** añadir `retention-days: 14` a `Upload fuzz corpus + crashes` (job `fuzz`) y a `Upload fuzz corpus + crashes` (job `fuzz-pr`) — 2 líneas; cache keys intactas
- **Verify:** `grep -n "retention-days" .github/workflows/fuzz-40.yml` (2 hits, ambos 14)
- **Estado:** ✅ COMPLETED (2× retention-days: 14, cache intacta)

### Step 4: Verify triple mecánico
- **Archivos:** (ninguno — solo comandos)
- **Acción:** `actionlint` 3 files + `git diff --check` + secrets-grep + grep cron final. Intentar `campaign_verify_cmd` primero (si BUG exit -1 → bash directa + mención en recitation)
- **Verify:** actionlint exit 0; diff-check vacío; secrets-grep vacío; `0 2 * * *` vs `0 3 * * 0` distintos
- **Estado:** ✅ COMPLETED (actionlint 0 vía campaign_verify_cmd; diff-check 0; crons distintos; skip-ci 1 hit; retention 2×14; secrets 0)

### Step 5: Commit selectivo ci + recitation (NO PUSH)
- **Archivos:** 3 workflows + `docs/dev/tasks/FIND-141.md` (staging explícito, nada más)
- **Acción:** `git add` solo esos 4 + `git commit -m "ci: FIND-141 — desolapar heavy schedules, guard skip-ci baseline, retention fuzz"` + `campaign_update_task_state(completed)` + recitation plan file. NO PUSH. `git status --short` limpio de WIP ajeno antes del add
- **Verify:** `git log --oneline -1` + `git show --stat HEAD` (4 files) + `git status --short` (sin restos)
- **Estado:** ✅ COMPLETED (commit 3150e5fc: 3 files +173/-2; WIP ajeno intacto unstaged; NO PUSH)

## Dependencias
- Bloquea a: Wave 3 FIND-142 (renombres, tras Waves 0-2 verdes)
- Paralelo seguro con: FIND-140 (release-*), FIND-146 (arch-metrics/ocr/opencode) — archivos disjuntos por principio 1-archivo-1-dueño

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-lead post-delegación (yo ejecuto como lead — dominio release/CI propio; revisión = verify mecánico + `git show` del diff; P2-01 batch del orquestador al cierre del plan cubre revisor distinto)
- **Enfoque:** ¿mover bench-daily (no cert-weekly) es correcto? ¿`[skip ci]` basta sin `if:`? ¿14d vs 30d?
- **Cómo se probó:** actionlint + diff-check + greps (evidencia en S4, no auto-reporte)
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
- **Veredicto:** ⬜ pendiente (tras S4)

## Notas

- Colateral pre-existente (NO tocar, NO deuda del PR): `adapters-compat.yml` Sun 03:00 sigue solapando con `heavy-cert` Sun 03:00 los domingos. Fuera de scope (principio 1-archivo-1-dueño; FIND-136 ya tocó ese archivo por caché). Si el orquestador quiere, nace FIND futuro — no se arregla inline (pipeline-full: rápido se arregla, lento → FIND-*; esto es schedule ajeno, ni siquiera rápido sin dueño).
- Rollback: `git revert <hash>` (commit atómico) + re-run workflows; sin migración (revert limpio, DoD shippable ✓).
- Cargo N/A (cero Rust); Internet N/A (sintaxis estable, verificada en-repo).

## Context Save Point

> Último estado verificado: DISCOVERY completo + task file creado. Próximo: S1 (edit cron bench línea 5). Para reanudar: leer este file desde S1 ⬜, respetar `git status` (WIP ajeno intacto), ejecutar S1→S5 en orden. Recitation plan file: PARTIAL con próximo S1.
