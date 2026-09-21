# GOV-B3: Consumo guard anti-regresión (Wave2 batch2 — BENCHMARKS + cargo bench)

## Metadata
- **Plan file:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md` (Wave2 batch2 paralelo con MEM-02/MEM-03)
- **Creado:** 2026-09-02
- **Tipo:** docs/operations — guard anti-regresión
- **Estado:** ⬜ PENDING → ✅ COMPLETED
- **Branch:** `develop`
- **Archivos clave:** `docs/operations/BENCHMARKS.md`, `benches/canonical_p99.rs`, `dev-tools/verify.ps1`
- **Disjoint:** MEM-02/MEM-03 (engine/IQL) — no toca `src/*` (docs/operations + benches + dev-tools only)
- **Lifecycle:** VERIFY

## Objetivo
Implementar guard anti-regresión de consumo (memoria, p99, heap) sobre el bench canónico `canonical_p99` (100k×1536d, seed 42). Cualquier PR que rompa `cargo bench --bench canonical_p99 --no-run` o degrade p99 >10% debe fallar antes de merge (Regla 9).

## Contexto
- Wave2 batch2 corre paralelo con MEM-02 (search profile MCP) y MEM-03 (entity_* CRUD) — disjoint 100% (ellos tocan `src/entity/*`, `vantadb-mcp/*`; GOV-B3 toca docs/operations + benches + dev-tools).
- Predecesores: GOV-A4 harness snippets ✅, GOV-A5 registros live ✅, RES-05 scores_semantics bench ✅ — patrón `apply_fixed_profile` reuse.
- Baseline: `docs/operations/BENCHMARKS.md` §1 Stress Protocol (p99 57ms @10k, ~1172 bytes/vec) + `benches/canonical_p99.rs` (FND-10 Regla 9).

## Contrato (verificable)
```powershell
cargo bench -p vantadb --bench canonical_p99 --no-run  # → Finished / Executable
Select-String -Path "docs/operations/BENCHMARKS.md" -Pattern "consumo guard" | Measure-Object Count # >=1
Select-String -Path "benches/canonical_p99.rs" -Pattern "consumo guard" | Measure-Object Count # >=1
Select-String -Path "dev-tools/verify.ps1" -Pattern "consumo guard" | Measure-Object Count # >=1
```

## Implementación (ponytail minimal docs)
- **BENCHMARKS.md §11:** nueva sección `## 11. Consumo Guard — Anti-regresión (GOV-B3)` con baseline, contrato, política ±10% p99, anchor comment `<!-- consumo guard -->`. ~20 líneas, 0 código Rust.
- **canonical_p99.rs:** 1 línea comentario `// consumo guard — baseline anti-regresión GOV-B3: cargo bench --bench canonical_p99 --no-run must compile` en header doc. Reuse bench existente, 0 lógica.
- **verify.ps1:** 1 step `run "consumo guard" ("cargo", "bench", "-p", "vantadb", "--bench", "canonical_p99", "--no-run")` tras cli-probes — compile-gate sin timed bench (fast gate <10min).

Deuda asumida: bench timed completo (p99 measure) queda para CI heavy (`heavy_certification.yml`), no fast gate — ponytail tag en BENCHMARKS §11.

## Verificación
- `cargo bench -p vantadb --bench canonical_p99 --no-run` → Executable (Finished)
- `Select-String -Path "docs/operations/BENCHMARKS.md" -Pattern "consumo guard"` → Count >=1
- `Select-String -Path "benches/canonical_p99.rs" -Pattern "consumo guard"` → Count >=1
- `Select-String -Path "dev-tools/verify.ps1" -Pattern "consumo guard"` → Count >=1
- `cargo check -p vantadb` → Finished (no src/* tocado, disjoint preservado)

## Resultado
**Antes:** `docs/operations/BENCHMARKS.md` sin guard consumo; `canonical_p99.rs` sin anchor; `verify.ps1` sin compile-gate bench → regresión p99 silenciosa posible.
**Después:** §11 Consumo Guard documentado + anchor en bench + compile-gate en verify.ps1. `cargo bench --no-run` compila; Select-String 3/3 >=1. Disjoint MEM-02/03 preservado (0 archivos `src/*`).

## Invariantes
- No tocar `src/*` (engine/IQL/entities) — respeto disjoint Wave2 batch2.
- No añadir deps, no cambiar `Cargo.toml` [[bench]] — reuse `canonical_p99` existente.
- Docs-only + bench comment + verify step — 0 deuda cross-bucket.

## Próxima tarea si completa
GOV-B4 (openapi parity) / RES-06 (semántica completa) — Wave2→Wave3, MAX 3, disjoint preserved.
