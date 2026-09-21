# TBH-10: Convert `bench_concurrent.rs` to `criterion_main!`

## Metadata
- **Plan file:** `docs/plans/2026-08-30-testing-bench-harden.md`
- **Creado:** 2026-08-31T11:00
- **last-synced:** 2026-08-31T11:00
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Sub-agente:** vanta-worker
- **Commit:** d4d7c24c on develop (final; prior 9052e63e and 2d8cfd39 amended to fold task file in)
- **Pre-commit hooks:** fmt ok, clippy ok

## Contexto
`benches/bench_concurrent.rs` actualmente usa `fn main()` custom con `Instant::now()`
(harness custom en `Cargo.toml`). NO produce `estimates.json` → no entra en el
pipeline de regresión nocturna (`bench_regression.py`). El resto de los 19 benches
usan `criterion_main!`. La auditoría multi-agente del 2026-08-30 (gap audit
benchmarks) recomienda convertirlo para que genere `estimates.json` y sea
comparable contra baseline. Prioridad MEDIA.

## Archivos clave
| Path | Rol |
|---|---|
| `benches/bench_concurrent.rs` | Refactor: reemplazar `fn main()` con `criterion_main!` |
| `Cargo.toml` líneas 211-213 | `[[bench]] name = "bench_concurrent" harness = false` — **NO TOCAR** (todos los otros 19 benches usan el mismo patrón) |
| `benches/hnsw_pure.rs` | Referencia (patrón criterion estándar) |
| `benches/canonical_p99.rs` | Referencia (patrón criterion estándar con histogramas) |
| `.github/workflows/heavy-bench-nightly-51.yml` línea 61 | Invoca `cargo bench --bench bench_concurrent -- --nocapture` — **NO TOCAR** (sigue funcionando igual) |

## Impacto mapeado (Regla 0)

### Mapeo de referencias
- `grep -r "bench_concurrent"` en todo el workspace:
  - `Cargo.toml:212` — `[[bench]] name = "bench_concurrent"` ✓ (no se cambia)
  - `.github/workflows/heavy-bench-nightly-51.yml:61` — `cargo bench --bench bench_concurrent` ✓ (no se cambia)
  - `benches/bench_concurrent.rs` — el archivo a refactorizar
  - Sin otros call sites

### Archivos que referencian el bench (no se tocan)
1. `Cargo.toml` — la entrada `[[bench]]` sigue igual (nombre del archivo no cambia, `harness = false` sigue siendo correcto: `criterion_main!` expande a un `main()` y funciona con `harness = false`).
2. `.github/workflows/heavy-bench-nightly-51.yml` — el comando `cargo bench --bench bench_concurrent` es agnóstico al formato interno del bench. Sigue produciendo `estimates.json` (en `target/criterion/bench_concurrent/<group>/<bench>/`).
3. Los otros 19 benches — no se tocan.

### Decisión sobre `harness`
- **MANTENER `harness = false`** en `Cargo.toml`. NO cambiar.
- Todos los 19 otros benches del workspace usan `harness = false` + `criterion_main!`. Confirmado:
  - `hnsw_pure`, `canonical_p99`, `hybrid_queries`, `high_density`, `stress_test`,
    `backend_compare`, `hnsw_recall_ef`, `vfile_search`, `incremental_bench`,
    `ivf_bench`, `memory_budget`, `tokenizer_bench`, `batch_existing_check`,
    `param_sweep`, `acorn_filtered_search`, `sparse_hot_path` — todos con `harness = false`.
- Criterion docs (verificadas vía webfetch https://docs.rs/criterion/0.8/criterion/macro.criterion_main.html):
  > "Currently, using Criterion.rs requires disabling the benchmark harness generated
  > automatically by rustc. This can be done like so: `[[bench]] name = "my_bench" harness = false`"
  > "The `criterion_main` macro expands to a `main` function which runs all of the
  > benchmarks in the given groups."

### Comportamiento preservado
- Mide los mismos 2 escenarios: read-only concurrent searches + mixed read-write.
- Mismos thread counts: [1, 4, 8, 16].
- Mismo setup: 10k vectores 128d, AutoTune::set_ef(100), tempdir StorageEngine.
- Mismas métricas reportadas: QPS, p50, p99, speedup, efficiency (read-only); QPS,
  p50, p99, insert rate (mixed).
- **Diferencia cosmética**: las tablas `println!` se preservan dentro de las
  funciones bench (se ejecutan en setup o se loguean entre iteraciones).

## Contrato (verificable mecánicamente)
| Check | Comando | Esperado |
|---|---|---|
| Termina en `criterion_main!` | `Select-String -Path benches/bench_concurrent.rs -Pattern "criterion_main!"` | ≥ 1 match |
| NO tiene `fn main()` | `Select-String -Path benches/bench_concurrent.rs -Pattern "^fn main"` | 0 matches |
| Compila | `cargo check -p vantadb --benches` | exit 0 |
| Build OK | `cargo build -p vantadb --benches` | exit 0 |
| Cargo.toml intacto | `Select-String -Path Cargo.toml -Pattern 'name = "bench_concurrent"\|harness = false'` | match en líneas 211-213 |

## Acceptance Criteria
1. `benches/bench_concurrent.rs` usa `criterion_group!` + `criterion_main!`
2. La lógica de medición actual (Instant::now, threads, latencias, p50/p99, QPS)
   se preserva — solo cambia la integración con criterion
3. `Cargo.toml` línea `[[bench]] name = "bench_concurrent" harness = false` se
   **MANTIENE** sin cambios (decisión: NO TOCAR; alinea con los 19 otros benches)
4. **Ponytail reflex:** mantener la lógica existente. NO refactors innecesarios. NO
   cambios de scope de las mediciones
5. **NO TOCAR** otros benches. **NO TOCAR** el workflow nightly. **NO TOCAR** el
   baseline de nightly (TBH-02 ya creó entries para los nightly benches;
   bench_concurrent no estaba en el baseline intencionalmente porque NO generaba
   estimates.json — ahora SÍ debería)

## Pasos

### Step 1: Discovery
- **Acción:** Leer `bench_concurrent.rs` + `hnsw_pure.rs` + `canonical_p99.rs` +
  `Cargo.toml` (sección `[[bench]]`)
- **Verify:** Confirmar patrón `criterion_main!` en 19 otros benches
- **Estado:** ✅ DONE

### Step 2: Update task state
- **Acción:** Marcar TBH-10 in-progress
- **Estado:** ✅ DONE

### Step 3: Refactor bench_concurrent.rs
- **Acción:** Reemplazar `fn main()` con `criterion_main!(benches);`. Wrap benchmarks
  existentes en `criterion_group!`. Preservar setup (AutoTune, 10k vectores, tempdir)
  y reporte (println! tables) dentro de las bench functions.
- **Verify:** Diff muestra `criterion_main!` y NO `fn main()`
- **Estado:** ✅ DONE

### Step 4: Verify cargo check
- **Acción:** `cargo check -p vantadb --benches`
- **Verify:** exit 0
- **Estado:** ✅ DONE

### Step 5: Verify cargo build
- **Acción:** `cargo build -p vantadb --benches`
- **Verify:** exit 0
- **Estado:** ✅ DONE

### Step 6: Verify cargo fmt
- **Acción:** `cargo fmt --check`
- **Verify:** exit 0
- **Estado:** ✅ DONE

### Step 7: Commit
- **Acción:** `git add benches/bench_concurrent.rs .opencode/skills/campaign-executor/tasks/TBH-10.md` + commit conventional
- **Estado:** ✅ DONE (commit 9052e63e)

### Step 8: Close task
- **Acción:** `campaign_update_task_state` con `completed` + recitation
- **Estado:** ✅ DONE

## Recitation
- **Contract met:** all 5 acceptance criteria satisfied
- **Verification evidence:**
  - `cargo check -p vantadb --benches` → exit 0
  - `cargo build -p vantadb --benches` → exit 0 (all 20 benches built)
  - `cargo fmt --check` → exit 0
  - `cargo bench_concurrent --test` → all 8 bench functions `Success` (read_only/t{1,4,8,16} + mixed_rw/t{1,4,8,16})
  - Pre-commit hooks (fmt + clippy) passed
  - `Select-String -Pattern "criterion_main!" benches/bench_concurrent.rs` → 1 match
  - `Select-String -Pattern "^fn main" benches/bench_concurrent.rs` → 0 matches
  - `Select-String -Pattern "criterion_group!" benches/bench_concurrent.rs` → 1 match
  - Cargo.toml lines 211-213 (bench_concurrent entry) UNCHANGED
- **Diff:** 86 insertions, 49 deletions in benches/bench_concurrent.rs
- **Commit:** d4d7c24c (`refactor(TBH-10): convert bench_concurrent to criterion_main (generates estimates.json for nightly regression)`)
- **Branch:** develop
- **Side effects:** bench_concurrent now generates `target/criterion/bench_concurrent/{read_only,mixed_rw}/t{1,4,8,16}/estimates.json`, making it eligible for `bench_regression.py` (TBH-04)
- **Follow-up (NOT part of TBH-10):** TBH-02 baseline should now include bench_concurrent entries. Out of scope; the orquestador can dispatch this as a small docs task.

## Dependencias
- TBH-01..05, TBH-11..15, TBH-17, TBH-19, TBH-20, TBH-22, TBH-23, TBH-07, TBH-21: predecesoras (✅ completed)

## Notas
- `criterion_main!` macro expande a un `fn main()` que itera sobre los groups y
  llama `Criterion::default().final_summary()`. Por eso `harness = false` +
  `criterion_main!` es el patrón estándar del repo.
- Después de este cambio, `bench_concurrent` generará `target/criterion/bench_concurrent/.../estimates.json`,
  haciéndolo elegible para el pipeline `bench_regression.py` (TBH-04).
- TBH-02 (baseline) NO creó entry para `bench_concurrent` porque no generaba
  `estimates.json`. El siguiente TBH que toque el baseline (TBH-08 si existe, o
  el siguiente) debería considerar agregar `bench_concurrent` al baseline. NO es
  parte de TBH-10.
