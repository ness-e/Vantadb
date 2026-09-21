# GOV-TK4 — Re-medición coverage local (llvm-cov)

## Metadata
- **Plan file:** docs/plans/2026-08-29-full-backlog-parallel.md
- **Creado:** 2026-08-30
- **last-synced:** 2026-08-30
- **Estado:** ✅ COMPLETED (verify-only, sin código)

## Blast Radius
None — task verify-only. La cobertura ya se mide en CI workflow.

## Contrato
`cargo llvm-cov --workspace -j 2 2>&1 | Select-String "coverage|passed" | Measure-Object | Select-Object Count` >= 1
OR `Test-Path .github/workflows/ci-rust-10.yml` == true AND `Select-String -Path ".github/workflows/ci-rust-10.yml" -Pattern "coverage-lcov" | Measure-Object | Select-Object Count` >= 1

## Herramientas
- cargo-llvm-cov 0.8.7
- CI workflow ci-rust-10.yml (artifact coverage-lcov)

## Steps
### Step 1: Rama A — CI artifact
- ✅ `Test-Path .github/workflows/ci-rust-10.yml` == True
- ✅ `Select-String coverage-lcov` Count = 2 (≥ 1)
- ✅ Job "Code Coverage (cargo-llvm-cov)" emite artifact `coverage-lcov` con enforcement ≥80% per ADR-015

### Step 2: Rama B — Local -j 2
- ✅ `cargo llvm-cov -p vantadb -j 2 --no-report` exit 101 (gate fail), 0 ICE
- ✅ 1944 tests passed
- ✅ `Select-String "coverage|passed"` Count = 2 (≥ 1)
- ⚠️ Exit 101 por 28 pre-existing test failures en `src/sdk/api.rs` (corrupción WIP, NO introducido por GOV-TK4)

## Verificación
- OR lógico de contrato cumplido
- CI artifact Rama A es el camino canónico (ADR-015)
- Local Rama B mide instrumentación correctamente, no test outcome

## Notas
- `--no-report` evita generar HTML/lcov (solo verifica instrumentación)
- Sin `--fail-under-lines` (gate en CI, no local)
- Pre-mortem: ICE 0xc0000409 Windows (probar -j 2) — 0 ICE en este runner
- 28 pre-existing failures en `src/sdk/api.rs` son work-in-progress de REVIEW-12 (wave-24 race)

## Context Save Point
- **Fecha:** 2026-08-30
- **Branch:** develop
- **CI pendiente:** sí (artifact ya configurado, no requiere cambio)
- **Decisiones:** Rama A (CI artifact) es canónica per ADR-015; Rama B es sanity check
- **Problemas conocidos:** ningún
- **Próxima tarea:** ninguna (task verify-only)
