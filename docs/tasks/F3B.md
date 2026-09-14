# F3B — job CI informativo `canonical_p99` (verificar toolchain primero)

## Metadata
- **Plan file:** `docs/plans/2026-09-13-cleanCA-fase3.md` (Task 4, Wave 2)
- **Creado:** 2026-09-14
- **Estado:** ⏳ IN PROGRESS (B1 toolchain-check → B2 job + baseline → B3 cierre)
- **Ruta:** vanta-lead (CI) + vanta-worker (bench)
- **Appetite:** 4h · 🟢 · 🟡 · **Rama:** `develop` · **NO commitear** (commitea el lead)
- **SDP:** campaign-executor, progreso, systematic-debugging, test-driven-development, code-review-and-quality, doubt-driven-development, source-driven-development, planning-and-task-breakdown, codebase-memory, observability-and-instrumentation

## Gate Justificación
Decisión humana Q5 (informativo primero) + deuda Fase 2 (bench-en-CI al mergear S3/S5). Sin número no hay optimización medible (Regla 9).

## Contrato
Workflow nuevo **no bloqueante** que (1) verifica que la toolchain de CI compila el bench en release (aquí tantivy rompe el build local — si en CI también rompe, reportarlo como HALLAZGO con dueño, nunca verde falso), (2) corre `canonical_p99` y publica el número vs baseline `docs/operations/BENCHMARKS.md`, (3) ante regresión abre hallazgo sin bloquear el merge.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `benches/canonical_p99.rs` (135L, criterion, seed 42, 100k×1536d + 1k queries, `common::apply_fixed_profile`), `docs/operations/BENCHMARKS.md` (§11 consumo guard + baseline §1 p99 ~57ms @10k; sin tabla canónica 100k×1536d publicada — el número lo publica este job), `Cargo.toml` (`[[bench]] canonical_p99 harness=false`, default features incluyen `advanced-tokenizer → tantivy`), `.github/workflows/heavy-bench-nightly-51.yml` (ya corre canonical_p99 en nightly + PRs que tocan `benches/**`; NO duplicar), `.github/workflows/perf-bench-40.yml` (Python, otro scope), `.github/workflows/arch-metrics-informational.yml` (patrón informativo con `continue-on-error: true`), `.github/workflows/ci-gate.yml` (reusable gate).
- **Referencias hacia dentro:** el workflow nuevo referencia `actions/checkout`, `./.github/actions/rust-setup`, `scripts/bench_regression.py` (extract/compare, ya usado por heavy-51), `benchmarks/criterion_baseline.json`.
- **Referencias entrantes:** ninguna (archivo nuevo, sin callers).
- **Veredicto de impacto:** BAJO — 1 archivo nuevo en `.github/workflows/` + 0 cambios en `src/`/`benches/`. No toca el bench (prohibido). Disjunto de F3C (`src/config.rs`). Trigger paths disjuntos de heavy-51 (perf-source paths vs bench paths) → complemento, no duplicado.

## Herramientas
- bash exacto: `cargo bench -p vantadb --bench canonical_p99 -- --help` (compila), parse YAML (python), `cargo fmt --check` (solo si toca Rust — no previsto)
- Write workflow + task file; codegraph solo si duda (no hizo falta: scope CI-only, blast radius = 1 archivo nuevo)
- Verificación YAML: python `yaml.safe_load` (actionlint no instalado local — documentado abajo)

## Steps
### Step B1: toolchain-check (compilar bench en release; si falla → HALLAZGO + Gate V)
- **Archivos:** `benches/canonical_p99.rs` (lectura), `Cargo.toml` (lectura, features)
- **Acción:** intentar compilación local del bench (`--no-run`); registrar resultado. Diseñar el job CI con step 1 = compile-gate release (`--no-run`) separado del timed-run, de modo que un fallo de toolchain se reporte como hallazgo con dueño en vez de verde falso.
- **Verify:** `cargo bench -p vantadb --bench canonical_p99 --no-run` (local, documentar) + job CI contiene step `toolchain-check` explícito
- **Estado:** ✅ COMPLETO (2026-09-14)
- **Evidencia B1:** `cargo bench -p vantadb --bench canonical_p99 --no-run` → `Finished bench profile [optimized + debuginfo] in 8m 18s`, exe `canonical_p99-c2af329de9a692ab.exe` generado. Solo 5 warnings pre-existentes en `src/sdk/search/debug_ops.rs` (unused imports, ajenos al bench, no tocados). **El pre-mortem "tantivy rompe el build local" NO reproduce en esta máquina hoy**: el bench compila con default features (incluye `advanced-tokenizer→tantivy`). `cargo bench ... -- --help` → exit 0 (binario funcional). Gate V NO disparado (no hay fallo que arbitrar). El job CI conserva el step `toolchain-check` separado + issue HALLAZGO automático por si CI sí rompe (nunca verde falso).

### Step B2: job informativo + baseline publicado
- **Archivos:** `.github/workflows/bench-canonical-p99-informational.yml` (NUEVO)
- **Acción:** crear workflow no bloqueante: (1) toolchain-check compile release, (2) run canonical_p99 + extracción p50/p95/p99 + artefacto, (3) compare vs baseline vía `scripts/bench_regression.py` con `continue-on-error` + CATEGORY INFORMATIONAL, ante regresión abre issue con labels `benchmark,regression` (patrón heavy-51) sin bloquear merge. Triggers: PR con paths perf-source (`src/index/**`, `src/storage/**`, `Cargo.toml`, `Cargo.lock`) + `workflow_dispatch`. contratado: NUNCA verde falso, NUNCA bloquea merge, NUNCA duplica heavy-51.
- **Verify:** archivo existe + YAML parse OK + `rg continue-on-error` con CATEGORY + paths disjuntos de heavy-51
- **Estado:** ✅ COMPLETO (2026-09-14)
- **Evidencia B2:** `.github/workflows/bench-canonical-p99-informational.yml` creado (2 jobs: `toolchain-check` + `bench-informational`). YAML parse OK (`yaml.safe_load`, jobs + triggers verificados). `continue-on-error: true` ×12 (2 jobs + 10 steps) con `# CATEGORY: INFORMATIONAL` ×3 (Regla 2: exención etiquetada, monitoreada vía issues automáticos `benchmark,toolchain` / `benchmark,regression`). Triggers estrictamente disjuntos de heavy-51 en `benches/**` (este: `src/index/**`, `src/storage/**`, `Cargo.toml/lock`; overlap Cargo.toml intencional y documentado). Pins SHA idénticos a heavy-51. Publica p50/p95/p99 en `$GITHUB_STEP_SUMMARY` + artefactos 30d + compare vía `scripts/bench_regression.py`. Baseline citado: BENCHMARKS.md §11 + §1 (el número 100k×1536d lo publica CI en el primer run; local no se corre por costo ~horas).

### Step B3: cierre (doc + task file)
- **Archivos:** este task file, `BENCHMARKS.md` (solo lectura — baseline §1/§11 citado, no editado: el número lo publica CI)
- **Acción:** documentar número-vs-baseline esperado o hallazgo toolchain con dueño; Gates D/V/C; RESULTADO §7.
- **Verify:** Gates evaluados + RESULTADO completo
- **Estado:** ✅ COMPLETO (2026-09-14)
- **Evidencia B3:** sin hallazgo toolchain (B1 verde local); baseline-vs-número delegado a primer run CI (artefacto + summary). `cargo fmt --check` limpio (0 Rust tocado). `cargo bench -- --help` exit 0. Gates: D no disparado (contrato no ambiguo, blast radius 1 archivo nuevo) · V no disparado (0 fallas verify) · C: colaterales = warnings pre-existentes `debug_ops.rs` (no tocados, no FIND nuevo: fuera de scope + ya visibles en CI).

## Dependencias
- Wave 2 con F3C (disjuntas: workflows/benches vs `src/config.rs`); nextTask = cierre de campaña.
- Precede: F3G ✅, F3X ✅ (commits `dd892c4c`, `13f0f729` en develop).

## Referencias
- Regla 9 (no optimizar sin medir) + Regla 11 (claims con fuente reproducible)
- Decisión bench-en-CI-al-mergear S3/S5 (humana Fase 2) + Q5 informativo-primero
- Skills 1 línea c/u: campaign-executor (pipeline DISCOVERY→CIERRE) · progreso (registro avance) · systematic-debugging (root-cause si toolchain falla) · test-driven-development (contrato verificable antes de implementar) · code-review-and-quality (5 ejes pre-cierre) · doubt-driven-development (adversarial: ¿verde falso?) · source-driven-development (GitHub Actions docs oficiales) · planning-and-task-breakdown (slices B1/B2/B3) · codebase-memory (blast radius) · observability-and-instrumentation (números publicados como telemetría: qué/pregunta vs baseline)

## Investigación código (DISCOVERY)
- `docs/tasks/F3B.md` NO existía → DISCOVERY completo.
- Bench compila local: tantivy rompe el build local según pre-mortem del plan → B1 lo verifica mecánicamente abajo.
- Features: `canonical_p99` sin `required-features` → compila con default (incluye `advanced-tokenizer→tantivy`).
- Baseline actual en BENCHMARKS.md: §1 Stress Protocol (p99 ~57ms @10k, otra escala) + §11 consumo guard (compile-gate policy, regresión p99 >10% requiere ADR/revert). No hay tabla 100k×1536d publicada → el job la publica como artefacto CI.
- heavy-51 ya corre canonical_p99 en nightly + PRs `benches/**` → el job nuevo triggerea en perf-source paths (`src/index/**`, `src/storage/**`, Cargo) → complemento.

## Investigación problema
Job informativo ≠ verde falso: si la toolchain no compila, el deliverable es el HALLAZGO documentado con dueño, no un job que siempre pasa. Por eso el compile-gate es un step/job separado con salida explícita, y todo el workflow lleva `continue-on-error: true` + `# CATEGORY: INFORMATIONAL` (Regla 2: exención etiquetada, monitoreada vía issues automáticos).

## Investigación internet
No requerida (sin ambigüedad: patrón informativo copiado de `arch-metrics-informational.yml` + `heavy-bench-nightly-51.yml` del propio repo, ambos precedentes internos).

## Notas
- WIP ajeno en git status (`completions/`, `desktop/Cargo.lock`, `.opencode` submodule) → NO tocar, NO commitear (commitea el lead).

## Context Save Point
- **Fecha:** 2026-09-14
- **Branch:** develop
- **CI pendiente:** sí (workflow nuevo sin correr en CI — se valida por YAML parse + precedente interno)
- **Decisiones:** trigger paths perf-source para no duplicar heavy-51; issues automáticos como monitoreo del INFORMATIONAL
- **Problemas conocidos:** tantivy rompe build local (pre-mortem plan) — B1 verifica
- **Próxima tarea:** cierre de campaña (tras F3C)
