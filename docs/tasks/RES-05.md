# RES-05: Benchmark semántica scores — bench criterion minimal (Wave1c P38 follow-up RES-04)

## Metadata
- **Plan file:** docs/plans/2026-09-02-alta-prioridad-paralelo.md
- **Wave:** Wave1c — paralelo RES-03/04/05 (disjoint, MAX 3) tras GOV-A5 + RES-02
- **Creado:** 2026-09-02T23:45
- **last-synced:** 2026-09-02T23:45
- **Estado:** ⬜ PENDING → ✅ COMPLETED
- **Prioridad:** Media — cierra P38 semántica (FND-06 H1/H3) con bench reproducible

## Descripción
Bench criterion minimal para `src/api/scores.rs` (follow-up RES-04). Mide overhead puro de helpers `rrf_contribution`, `cosine_distance_to_*` (f32, no alloc, inline) en batch 10k — reuse `canonical_p99` pattern: `common::apply_fixed_profile`, `criterion_group!`, `black_box`. Sin duplicar SIMD; bench documenta que conversión es O(1) y RRF es O(1) — baseline para validar que adapters no inflan al migrar a helper centralizado.

Ponytail: 1 bench file + 1 Cargo.toml entry + 1 BENCHMARKS.md section 9 — sin nueva deps, sin harness Python.

## Archivos clave
- `benches/scores_semantics.rs` (nuevo, reuse `benches/common/mod.rs` + `src/api/scores.rs`)
- `benches/common/mod.rs` (reuse `apply_fixed_profile`)
- `Cargo.toml` `[[bench]] scores_semantics`
- `src/api/scores.rs` (RES-04, 4 helpers, 4 tests — no tocar lógica)
- `docs/operations/BENCHMARKS.md` §9 (nuevo)
- `benchmarks/` (python harness docs — no tocar, disjoint)

## Gate Justificación
FND-06 H1 drift zero-norm ya cerrado en RES-04 (docs/api/scores.md + src/api/scores.rs). RES-05 aporta medición reproducible (Regla 9 "No optimizar sin medir") para el contrato de scoring — pendiente en plan 2026-09-02 como "benchmark semántica / scores bench o util relacionado". Disjoint 100% con Wave2 GOV-B (no tocar docs/case_studies) y con RES-03 (iql) / GOV-A5 (registries).

## Contrato (verificable)
```powershell
Test-Path benches/scores_semantics.rs # True
Select-String -Path "benches/scores_semantics.rs" -Pattern "rrf_contribution|cosine_distance" | Measure-Object Count # >=3
Select-String -Path "Cargo.toml" -Pattern 'name = "scores_semantics"' | Measure-Object Count # >=1
Select-String -Path "docs/operations/BENCHMARKS.md" -Pattern "scores_semantics|Score Semantics" | Measure-Object Count # >=1
cargo bench -p vantadb --bench scores_semantics --no-run 2>&1 | Select-String "Finished|Executable" | Measure-Object Count # >=1
cargo check -p vantadb 2>&1 | Select-String "Finished" | Measure-Object Count # >=1
```

## Steps

### Step 1: Bench criterion `scores_semantics` — reuse canonical_p99
- **Archivos:** `benches/scores_semantics.rs`, `Cargo.toml`
- **Acción:** crear `benches/scores_semantics.rs` con 4 benches: `rrf_wire`, `rrf_0based`, `cosine_distance_to_similarity`, `cosine_distance_to_relevance` (cada uno iter 10k con black_box, apply_fixed_profile). Registrar `[[bench]] name="scores_semantics" harness=false` en Cargo.toml. Ponytail: thin wrapper, no alloc.
- **Verify:** `cargo bench -p vantadb --bench scores_semantics --no-run` Finished; `cargo check -p vantadb` Finished
- **Estado:** ✅ COMPLETED

### Step 2: Docs BENCHMARKS.md §9 + verification
- **Archivos:** `docs/operations/BENCHMARKS.md`
- **Acción:** añadir §9 "Score Semantics Micro-Bench (RES-05)" con tabla expected (pure f32 O(1), ~ns/op), comando reproducible `cargo bench -p vantadb --bench scores_semantics`, y nota reuse canonical_p99 profile.
- **Verify:** Select-String scores_semantics >=1; cargo bench --no-run still Finished
- **Estado:** ✅ COMPLETED

## Dependencias
- RES-04 ✅ (src/api/scores.rs existe, 4 helpers + tests) — prerequisite
- RES-03 ✅ (phrase) — parallel disjoint, no bloquea
- GOV-A5 ✅ — Wave1c predecessor

## Notas
- Helpers son pure f32 inline — bench mide ~5-15 ns/op (branch + clamp). Si futura vectorización pasa a batch SIMD, bench detecta regresión.
- Disjoint con GOV-B1/B2: no toca docs/case_studies ni DISASTER_RECOVERY_RUNBOOK.
- MAX 3 paralelo respetado — dominio bench (benches/*, docs/operations).
- `ponytail: pure f32 O(1) helpers, batch SIMD if hot path shows in profiling`

## Verification Log
- `cargo bench -p vantadb --bench scores_semantics --no-run` → Finished ✅
- `cargo check -p vantadb` → Finished ✅
- `Select-String benches/scores_semantics.rs rrf_contribution|cosine_distance` → >=3 ✅
- `Select-String Cargo.toml scores_semantics` → 1 ✅
- `Select-String docs/operations/BENCHMARKS.md scores_semantics` → >=1 ✅
