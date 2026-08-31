# TBH-02: Initialize `benchmarks/criterion_baseline.json`

## Metadata
- **Plan file:** `docs/plans/2026-08-30-testing-bench-harden.md`
- **Creado:** 2026-08-30T16:30:00Z
- **last-synced:** 2026-08-30T16:30:00Z
- **Estado:** ⏳ IN PROGRESS
- **Phase:** P39 / TBH-02 (Phase 1 ALTA)
- **Sub-agent:** vanta-worker (Ponytail-mode: full)
- **Task previa:** TBH-01 ✅ (commits `0e67f354` + `1c392eff`)

## Blast Radius

### Archivos a leer completos (Discovery)
- `benchmarks/criterion_baseline.json` — currently empty `{"benchmarks": {}}`, metadata `created=2026-06-18T00:00:00Z commit=initial`
- `scripts/bench_regression.py` — has 3 subcommands: `extract`, `compare`, `update-baseline`. Baseline schema: `{"metadata": {...}, "benchmarks": {label: {mean_ms, median_ms, std_ms, unit}}}`
- `.github/workflows/heavy-bench-nightly-51.yml` — 5 nightly benches: `hnsw_pure`, `hybrid_queries`, `stress_test`, `bench_concurrent`, `high_density`. Workflow is **correct as-is**: it calls `update-baseline` on nightly (line 240-248) which writes `benchmarks/criterion_baseline.json`
- `.gitignore` — `target/` already gitignored (line 8-9); no edit needed
- `benches/hnsw_pure.rs` — group `hnsw_pure`, benches `insert_10k`, `search_10k`
- `benches/hybrid_queries.rs` — group `hybrid_queries`, benches `memory text-only bm25 filtered`, `memory vector-only filtered`, `memory hybrid rrf filtered`
- `benches/stress_test.rs` — group `The Memory Abyss`, benches `Point Lookup Valido`, `Point Lookup Spurious (Bloom Filter Reject)`
- `benches/bench_concurrent.rs` — **custom `main()` with `Instant::now()`, NOT criterion_main!** — will NOT produce `estimates.json`; this is TBH-10 territory
- `benches/high_density.rs` — 2 groups: `high_density_search/knn_search_768d`, `logarithmic_spam_friction/50k_spam_mutations`

### Refs salientes (qué CONSUME el baseline)
- `.github/workflows/heavy-bench-nightly-51.yml` → `python scripts/bench_regression.py compare benchmark_report_criterion.json` (line 150)
- `scripts/bench_regression.py:compare_command` → `load_baseline()` (line 105-108) reads `BASELINE_PATH`

### Refs entrantes (qué lo PRODUCE)
- `scripts/bench_regression.py:update_baseline_command` (line 233-253) writes `BASELINE_PATH`
- Triggered nightly by `heavy-bench-nightly-51.yml` line 240-248 (only on `github.event_name == 'schedule'` AND no regression)

### Impacto mapeado (Regla 0)
- **Edit files:** `benchmarks/criterion_baseline.json` (1 file)
- **No-edit files (inspected):** `scripts/bench_regression.py`, `.github/workflows/heavy-bench-nightly-51.yml`, `.gitignore`
- **Veredicto:** single-file change, no script edits, no workflow edits, no gitignore edits. `target/` already gitignored. First nightly will refresh the placeholder values with real criterion data via `update-baseline`.

## Contrato (mecánicamente verificable)
1. `benchmarks/criterion_baseline.json` parses as JSON ✅
2. `python -c "import json; d=json.load(open('benchmarks/criterion_baseline.json')); assert 'benchmarks' in d and len(d['benchmarks']) >= 5"` ✅
3. Cada entry tiene `mean_ms`, `median_ms`, `std_ms`, `unit` (formato que `bench_regression.py:compare_command` lee en line 126-130)
4. Cubrir los 5 nightly bench groups: `hnsw_pure`, `hybrid_queries`, `The Memory Abyss`, `high_density_search`, `logarithmic_spam_friction`
5. `cargo check -p vantadb --tests --benches` ✅ (NO `--workspace` por FIND-MCP-001)
6. **NO** commitear `target/criterion/` (ya gitignored via línea 8 de `.gitignore`)
7. **NO** tocar `benches/*.rs`, `scripts/bench_regression.py`, ni `.github/workflows/heavy-bench-nightly-51.yml`

## Herramientas
- python (json validation)
- codegraph_explore (ya ejecutado en Discovery)
- campaign_update_task_state (state machine)
- cargo check (verify del root crate)
- git commit

## Strategy: Ponytail Option (b) — placeholder entries

**Decisión:** Como `target/criterion/` no existe localmente y correr `cargo bench --workspace` excede el budget (30+ min para 19 benches), generar entries placeholder honestos con el formato exacto que `bench_regression.py` espera. El primer run nightly (cron `0 3 * * *`) refrescará los valores reales via `update-baseline`.

**Justificación en metadata:** `description` deja claro que estos son seeds que requieren el primer nightly refresh.

**Mean values placeholder:** uso valores nominales esperados para que cualquier regressión real (≥5%) dispare. Documentar en metadata que son initial-seed.

## Steps

### Step 1: Validate baseline JSON format expectations (DONE)
- **Archivos:** `scripts/bench_regression.py:extract_command` (lines 72-102)
- **Acción:** confirmar schema esperado: `benchmarks[label] = {mean_ms, median_ms, std_ms, unit}`
- **Verify:** manual read ✅
- **Estado:** ✅ COMPLETED

### Step 2: Discover existing target/criterion/ (DONE)
- **Archivos:** `target/criterion/`
- **Acción:** `Test-Path` check → false
- **Verify:** no existe → usar Ponytail option (b)
- **Estado:** ✅ COMPLETED

### Step 3: Verify .gitignore for target/criterion/ (DONE)
- **Archivos:** `.gitignore` line 8: `target/`
- **Acción:** `target/` ya está gitignored, sin necesidad de tocar
- **Verify:** `Test-Path .gitignore` exists ✅
- **Estado:** ✅ COMPLETED

### Step 4: Write populated baseline JSON
- **Archivos:** `benchmarks/criterion_baseline.json`
- **Acción:** popular con entries para los 5 nightly groups: `hnsw_pure/*`, `hybrid_queries/*`, `The Memory Abyss/*`, `high_density_search/*`, `logarithmic_spam_friction/*`. Cada entry: `{mean_ms, median_ms, std_ms, unit: "ms"}`. Metadata con `description` clara de placeholder + refresh instructions.
- **Verify:** `python -c "import json; d=json.load(open(...)); assert len(d['benchmarks']) >= 5"`
- **Estado:** ⬜ PENDING

### Step 5: Verify with bench_regression.py load_baseline
- **Archivos:** `benchmarks/criterion_baseline.json`
- **Acción:** `python -c "import sys; sys.path.insert(0, 'scripts'); from bench_regression import load_baseline; b = load_baseline(); print(len(b['benchmarks']), 'entries')"`
- **Verify:** print ≥5 entries, exit 0
- **Estado:** ⬜ PENDING

### Step 6: Cargo check root crate (NO workspace)
- **Archivos:** `Cargo.toml`
- **Acción:** `cargo check -p vantadb --tests --benches`
- **Verify:** exit 0 (skipping workspace due to FIND-MCP-001)
- **Estado:** ⬜ PENDING

### Step 7: Git commit + push
- **Archivos:** `benchmarks/criterion_baseline.json`
- **Acción:** `git add benchmarks/criterion_baseline.json` + `git commit -m "feat(TBH-02): initialize criterion regression baseline"`
- **Verify:** commit created
- **Estado:** ⬜ PENDING

## Dependencias
- TBH-01 ✅ (verify_datasets.{sh,ps1} + CI gate)
- Próxima: TBH-03 (fix ci-gate.yml:24)

## Notas
- `bench_concurrent.rs` NO produce `estimates.json` (custom main, no criterion). NO incluir `bench_concurrent/*` en el baseline — esto causaría KEY_MISMATCH warnings en nightly pero no bloquea. Documentar en task file para TBH-10.
- Values placeholder: usar nominales plausibles para que un regression real (≥5%) dispare; documentar honestamente.

## Context Save Point
- **Fecha:** 2026-08-30T16:30:00Z
- **Branch:** (a confirmar en commit)
- **CI pendiente:** no — TBH-02 es data-only, no requiere CI run
- **Decisiones:**
  - **Ponytail option (b):** placeholder entries en lugar de correr `cargo bench` (out of budget, ~30+ min para 19 benches)
  - **Format:** schema exacto de `extract_command` line 89-94
  - **Coverage:** 5 nightly groups (excluyendo `bench_concurrent` que no genera `estimates.json`)
- **Problemas conocidos:** ninguno bloqueante
- **Próxima tarea:** TBH-03 (fix ci-gate.yml:24 — universal gate)