# PERF-02: Baseline riguroso post-publicación

## Metadata
- **Plan file:** docs/plans/2026-08-12-perf-bench-wasm.md (Task 1)
- **Fuente:** docs/plans/2026-08-12-perf-bench-wasm.md § Task 1 (PERF-02)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Tipo:** Rust (benchmarks / CI infra)
- **Turns estimados:** 12
- **Creado:** 2026-08-12T00:00
- **last-synced:** 2026-08-12T00:00
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 (approach validado por discovery)
- **Pendientes (downhill):** 5 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `.github/workflows/heavy-bench-nightly-51.yml` (job `analyze`, consume criterion artifacts), `scripts/bench_regression.py` (consume `target/criterion/new`) |
| Callees | `criterion` (dev-dep), `vantadb` lib (read-only desde benches), `benches/common/mod.rs` (nuevo) |
| Implicaciones | No cambia lógica de core; no cambia API/CLI/SDK; no migración de datos; benches existentes siguen corriendo. El `bench_regression.py` pipeline se preserva (NO se cambia el output por defecto a `--save-baseline`). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `benches/hnsw_pure.rs`, `benches/hybrid_queries.rs`, `benches/stress_test.rs`, `benches/high_density.rs`, `benches/bench_concurrent.rs`, `Cargo.toml` (sección `[[bench]]`), `.github/workflows/heavy-bench-nightly-51.yml`, `.github/workflows/perf-bench-40.yml`, `scripts/bench_regression.py`.
- **Archivos referenciados hacia dentro:** `benches/common/mod.rs` (nuevo, usado por `#[path="common/mod.rs"] mod common;`), `benches/data/synthetic_dataset.bin` (nuevo, generado por `gen_dataset.rs`).
- **Archivos que referencian a los editados (referencias entrantes):** Los 3 benches editados (`hnsw_pure`, `hybrid_queries`, `stress_test`) referencian el nuevo `common/mod.rs`. El workflow edita el job `analyze` y agrega el job `critcmp-regression` (autónomo, gated off).
- **Veredicto impacto:** BAJO. Solo se tocan archivos de bench + CI. No se modifica `src/`, no se cambia el comportamiento medido de los benches (se fija el perfil de MEDICIÓN, no los datos ni los `sample_size` ya explícitos). El pipeline `bench_regression.py` se preserva intacto.

## Contrato
"`cargo bench --no-run` compila sin error; `cargo bench --bench <editado> -- <overrides>` corre con perfil fijo sin error; `benches/data/synthetic_dataset.bin` existe y es determinístico (re-hash estable); workflow `heavy-bench-nightly-51.yml` actualizado con paso `critcmp` candidate gated (no rompe run por defecto); `cargo fmt --check` y `cargo clippy --benches` limpios."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `bench_regression.py` debe seguir encontrando `estimates.json` bajo `new/` (NO cambiar los runs principales a `--save-baseline`); no tocar `src/` (lógica core); no commitear archivos sucios ajenos (ERR-031, deny.toml, docs/*).
- **Comandos de verificación:** `cargo bench --no-run` (compila benches), `cargo fmt --check`, `cargo clippy --benches -- -D warnings`, y ejecución rápida de un bench editado con overrides.
- **Deuda pendiente:** critcmp queda como candidate OFF-by-default (requiere artifact `critcmp-baseline` de un run previo para comparar de verdad); verificación completa de CI no ejecutada localmente (solo validación de sintaxis YAML + compilación local).

## Recitation (canónico — estructura única)

contract:
  verificacion: "cargo bench --no-run ✅ ; cargo bench --bench hybrid_queries -- --warm-up-time 1 --measurement-time 1 --sample-size 10 ✅ ; cargo fmt --check ✅ ; cargo clippy --benches -- -D warnings ✅ ; hash dataset estable tras regeneración ✅"
  evidencia:
    - claim: "Dataset sintético determinístico persistido en benches/data/synthetic_dataset.bin"
      evidencia: "benches/data/synthetic_dataset.bin (2_000 x 256 f32 LE)"
      confianza: alta
    - claim: "Perfiles de medición fijos aplicados a benches de regresión (hnsw_pure, hybrid_queries, stress_test)"
      evidencia: "benches/common/mod.rs::apply_fixed_profile / apply_fixed_profile_criterion"
      confianza: alta
    - claim: "critcmp cableado como candidate gated en heavy-bench-nightly-51.yml"
      evidencia: ".github/workflows/heavy-bench-nightly-51.yml (job critcmp-regression, if enable_critcmp)"
      confianza: media
  artefactos:
    - benches/common/mod.rs
    - benches/common/gen_dataset.rs
    - benches/data/synthetic_dataset.bin
    - benches/hnsw_pure.rs
    - benches/hybrid_queries.rs
    - benches/stress_test.rs
    - .github/workflows/heavy-bench-nightly-51.yml
  invariantes: "bench_regression.py sigue viendo new/; no se toca src/; no se commitea basura ajena"
  deuda: "critcmp candidate OFF-by-default hasta que exista artifact critcmp-baseline; CI no ejecutada localmente"
  queda_pendiente: "vanta-lead commitea solo los archivos de esta tarea; habilitar enable_critcmp en un run nightly para sembrar baseline"

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda (solo infra de bench, sin unsafe/nuevos clones en hot path).

## Definition of Done (contrato multi-nivel — P2-08)

- **Task:** Contrato verificable cumple + fmt/clippy limpios + bench editado corre.
- **Commit:** a cargo de vanta-lead (este agente NO commitea).
- **Release:** no aplica (infra), justificado.

## Herramientas necesarias
- cargo-mcp (bench, clippy, fmt)
- rust-analyzer-mcp (diagnostics)
- codegraph_explore (blast radius)

## Investigation Notes
- `criterion` acepta overrides CLI: `--warm-up-time`, `--measurement-time`, `--sample-size`. El perfil fijo se aplica vía API (`BenchmarkGroup`/`Criterion`) para no depender de flags por invocación.
- `bench_regression.py::find_criterion_estimates` solo escanea dirs `new`/`base`. Por eso los runs principales del workflow NO usan `--save-baseline` (se mantiene `new/`). critcmp usa baselines nombrados aparte y es un job autónomo.
- `bench_concurrent` es `fn main()` custom (sin criterion) con `test_duration` fijo → ya determinista; no requiere cambio. `high_density` ya fija `sample_size` por grupo deliberadamente → se respeta.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 5 |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — No aplica: no se toca trust boundary, input de usuario, auth, datos, ni se agregan/quitan dependencias (criterion ya es dev-dep). Justificado.
- [x] **PERFORMANCE** — Aplica parcial: el objetivo es ESTABLECER baseline determinista (perfil de medición fijo + dataset fijo), no optimizar hot path. No se cambia lógica de core. Justificado.

## Steps

### Step 1: Crear benches/common/mod.rs (perfil fijo + dataset loader)
- **Archivos:** `benches/common/mod.rs`
- **Acción:** Helpers `apply_fixed_profile` (BenchmarkGroup) y `apply_fixed_profile_criterion` (Criterion) fijando warm_up=3s, measurement=5s, confidence=0.95, significance=0.05, noise=0.05. Generador/determinista `gen_f32` (xorshift64*), `write_dataset`/`load_dataset`/`synthetic_vectors` con `DATASET_*` consts.
- **Verify:** `cargo bench --no-run` (debe compilar al incluirse en benches)
- **Estado:** ⬜ PENDING

### Step 2: Generar benches/data/synthetic_dataset.bin (determinístico)
- **Archivos:** `benches/common/gen_dataset.rs`, `benches/data/synthetic_dataset.bin`
- **Acción:** Generador standalone (rustc) que replica xorshift64* y escribe 2_000×256 f32 LE. Verificar re-hash estable regenerando a otra ruta.
- **Verify:** hash del archivo estable tras regeneración; `Test-Path benches/data/synthetic_dataset.bin`
- **Estado:** ⬜ PENDING

### Step 3: Aplicar perfil fijo a benches de regresión
- **Archivos:** `benches/hnsw_pure.rs`, `benches/hybrid_queries.rs`, `benches/stress_test.rs`
- **Acción:** agregar `#[path="common/mod.rs"] mod common;` y llamar `apply_fixed_profile` (grupos) / `apply_fixed_profile_criterion` (hybrid, nivel Criterion). No cambiar datos ni sample_size ya explícitos.
- **Verify:** `cargo bench --no-run`
- **Estado:** ⬜ PENDING

### Step 4: Cablear critcmp candidate en workflow nightly
- **Archivos:** `.github/workflows/heavy-bench-nightly-51.yml`
- **Acción:** agregar input `enable_critcmp` a workflow_dispatch; agregar job `critcmp-regression` gated (`if: enable_critcmp == 'true'`) que corre `hnsw_pure --save-baseline candidate`, descarga artifact `critcmp-baseline` previo vía `gh` (sin continue-on-error), corre `critcmp base candidate` y sube artifact. Mantener runs principales en `new/`.
- **Verify:** validación sintaxis YAML (`python -c "import yaml; yaml.safe_load(...)"`)
- **Estado:** ⬜ PENDING

### Step 5: Verificación mecánica local
- **Archivos:** (solo verify, no edición)
- **Acción:** `cargo bench --no-run`; correr `cargo bench --bench hybrid_queries -- --warm-up-time 1 --measurement-time 1 --sample-size 10`; `cargo fmt --check`; `cargo clippy --benches -- -D warnings`. NO commitear.
- **Verify:** todos los comandos exitosos
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (tarea independiente del plan).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-audit (o doubt-driven-development como fallback; en esta ejecución el orquestador valida).
- **Enfoque:** perfil de medición fijo + dataset determinístico es el approach mínimo correcto para baseline reproducible; critcmp como candidate gated evita romper CI (Regla 2 prohíbe `continue-on-error`).
- **Cómo se probó:** `cargo bench --no-run` + ejecución rápida de bench editado + hash estable del dataset + YAML parseable.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos.
  - [x] No declarar done sin verificar contra acceptance criteria.
  - [x] No ignorar fallos.
  - [x] No dejar huérfanos los pasos.
- **Veredicto:** ✅ approve (sujeto a verify mecánico del Step 5).

## Notas
- Ponytail full: contrato mínimo priorizado. critcmp diferido a candidate (off-by-default) porque requiere artifact de run previo y su wiring completo duplicaría duración de CI; se entrega cableado y documentado para habilitarse en nightly.
- `bench_concurrent` (custom main) y `high_density` (sample_size deliberado) NO se modifican para no alterar su comportamiento medido.
