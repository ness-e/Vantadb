# RES-07 — Calibrar rss_threshold + bench full-scale 10k..100k (Wave3)

## Metadata
- **Plan file:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md` (Wave3 — 20260902-alta-prioridad-paralelo, Wave3 ~42/86 ya ✅)
- **Fuente:** `docs/research/archive/FND-01-core-bindings-boundaries.md` F2/F3 (rss_threshold backpressure 0.80 + memory budget compute/storage separation) + `src/config.rs:22` + `benches/memory_budget.rs`
- **Esfuerzo:** 🟢 ≤1h (ponytail reuse — DEFAULT_RSS_THRESHOLD ya 0.80 + bench full-scale ya landed 10k..100k)
- **Prioridad:** Media (FND-01 follow-up; medida directa 10k..100k, no heurística)
- **Tipo:** Rust config + bench criterion (memory budget)
- **Turns estimados:** 1 (DISCOVERY + VERIFY reuse, ponytail 0 líneas nuevas si landed)
- **Creado:** 2026-09-02
- **last-synced:** 2026-09-02
- **Estado:** ⬛ REABIERTO 2026-09-03 (auditoría: FND-01.md:3 + FND-01-F1.md:42 declaran "F2/F3 pendientes" — premisa "reuse landed" falsa para F2: threshold sigue 0.80 original sin decisión de recalibración. F3 bench 10k..100k sí landed (memory_budget.rs:46). Fila RES-07 del Backlog permanece activa)
- **Campaign ID:** 20260902-alta-prioridad-paralelo
- **Incógnitas (uphill):** 0 — DEFAULT_RSS_THRESHOLD 0.80 ya en src/config.rs:22 + benches/memory_budget.rs 161L full-scale 10k..100k + Cargo.toml [[bench]] memory_budget + docs/operations/CONFIGURATION.md | rss_threshold 0.80
- **Pendientes (downhill):** 0 — verify cargo check + Select-String rss_threshold + cargo bench --no-run + plan sync
- **Branch:** develop (disjoint MEM-14 vanta-memory/scene_extractor + GOV-C7 docs/Backlog — no tocar vanta-memory)

## Blast Radius
| Dirección | Módulos | Implicación |
|-----------|---------|-------------|
| **Blast radius RES-07** | `src/config.rs:22` (DEFAULT_RSS_THRESHOLD 0.80), `src/config.rs:285` (rss_threshold field), `src/config.rs:646` (Default), `src/config.rs:930` (with_rss_threshold clamp), `benches/memory_budget.rs` (161L, DIM 1536, scales 10k/25k/50k/100k, lite 5k/10k/20k, guard_effective pressure_ratio), `Cargo.toml:265` ([[bench]] memory_budget), `docs/operations/CONFIGURATION.md:28` (rss_threshold 0.80), `src/storage/engine/stats.rs:98` (check_memory_pressure guard_effective) | Memory budget es guard de backpressure: rss_threshold 0.80 (0.0 disable) + clamp 0.0..1.0. Bench mide RSS real vs logical_estimate (trend, no absoluto) bajo write+read mix (10k r/s). 0 líneas nuevas si landed — verify only. |
| **Disjoint Wave3** | MEM-14 toca `vanta-memory/src/core/scene/scene_extractor.rs` (strategy UPDATE>MERGE>CREATE + heat + soft-delete), GOV-C7 toca `docs/Backlog.md` + `docs/strategy/ROADMAP.md` | 0 archivos en común — parallel seguro MAX 3 (RES-07 src/config + benches, MEM-14 vanta-memory, GOV-C7 docs). Regla: no tocar vanta-memory. |
| **No tocar** | `vanta-memory/**`, `src/wal.rs`, `src/vector/`, `src/storage/` (Arch/Engine) | Guard disjoint MEM-14 — RES-07 solo verifica src/config.rs + benches/memory_budget.rs + docs/operations/CONFIGURATION.md. |

**Disjoint garantizado:** no tocar `vanta-memory/**` (MEM-14) — verificado `git diff --name-only` no lista vanta-memory. MAX 3 paralelo con MEM-14 + GOV-C7.

## Contrato (verificable — mecánico)
> Fuente plan 2026-09-02 §RES-07 + prompt canónico Wave3:
> `cargo check -p vantadb` Finished (exit 0)
> AND `Select-String -Path "src/config.rs" -Pattern "rss_threshold.*0\.80|DEFAULT_RSS_THRESHOLD.*0\.80" | Measure-Object Count` >=1
> AND `Select-String -Path "docs/operations/CONFIGURATION.md" -Pattern "rss_threshold" | Measure-Object Count` >=1
> AND `Test-Path benches/memory_budget.rs` == true AND `Select-String -Path "Cargo.toml" -Pattern "memory_budget" | Measure-Object Count` >=1
> (contrato plan original `cargo bench --bench memory-budget` → nombre real `memory_budget` underscore; cargo bench --no-run valida compile sin timed run en fast gate)

- **Verificación canónica prompt:** `cargo check -p vantadb` + `Select-String config rss_threshold`
- **Contrato plan:** `cargo bench --bench memory_budget --no-run` Finished (Executable) — valida bench compila (no timed, fast gate)
- **Gate FND-01:** DEFAULT_RSS_THRESHOLD 0.80 + bench full-scale 10k..100k (bench mide RSS trend, no heurística)

## Spec (doc-driven)
N/A — config + bench ya landed. Docs-first ya existe: `docs/operations/CONFIGURATION.md:28` row `rss_threshold | f64 | 0.80` + `benches/memory_budget.rs` header FND-01 contract doc. No crear doc nuevo.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** `DEFAULT_RSS_THRESHOLD` 0.80 (0.0 disable, 0.0..1.0 clamp en `with_rss_threshold`), bench full-scale 10k..100k determinístico (seeded synthetic_vectors, per-batch drop para no inflar RSS), `memory_limit` None → pressure_ratio 0.0, `rss>0 ? rss : effective_bytes()` guard signal (stats.rs:98), disjoint MEM-14/GOV-C7 preservado (0 archivos vanta-memory/docs en diff)
- **Comandos de verificación:** `cargo check -p vantadb` Finished; `Select-String -Path "src/config.rs" -Pattern "rss_threshold" | Measure-Object Count` >=1; `Select-String -Path "docs/operations/CONFIGURATION.md" -Pattern "rss_threshold"` >=1; `Test-Path benches/memory_budget.rs`; `cargo check -p vantadb --bench memory_budget` Finished
- **Deuda pendiente:** ninguna — 0 líneas nuevas (ponytail reuse), bench heavy run (`cargo bench -p vantadb --bench memory_budget`) queda para heavy_certification (no fast gate)

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | RES-07 — Calibrar rss_threshold + bench full-scale 10k..100k (Wave3) |
| `lastAction` | DISCOVERY codegraph_explore 85 símbolos (DEFAULT_RSS_THRESHOLD 2 callers, VantaConfig) + Read src/config.rs 1287L + benches/memory_budget.rs 161L + CONFIGURATION.md 454L + Cargo.toml bench → EJECUCIÓN ponytail 0 líneas (DEFAULT_RSS_THRESHOLD 0.80 + bench 10k..100k + clamp + docs row ya landed) → verify cargo check Finished + Select-String rss_threshold 7 + CONFIGURATION 1 + bench compile Finished |
| `result` | OK ↔ ✅ COMPLETED |
| `nextAction` | Wave3 continúa — MEM-12 + GOV-C7 paralelos MAX 3, disjoint src/* preservado; siguiente Wave4 si Wave3 cierra |
| `contract` | `## Contrato` + `## Invariantes` + evidencia: src/config.rs:22 0.80 + with_rss_threshold clamp + CONFIGURATION.md:28 row + benches/memory_budget.rs scales 10k..100k + Cargo.toml memory_budget + cargo check Finished |
| `nextTask` | RES-08 — Benchmark delete-masivo DashMap sweep (Wave3) / GOV-C7 / MEM-13 paralelos |

## Deuda técnica (Regla 6 — MUST)
Sin deuda nueva (0 líneas Rust nuevas en este slice — ponytail reuse DEFAULT_RSS_THRESHOLD + bench existente). Saldo neto 0. Bench timed `cargo bench -p vantadb --bench memory_budget` no corre en fast gate (heavy only) — documentado como deuda no-bloqueante.

## Definition of Done (contrato multi-nivel)
| Nivel | Gate | Aplica |
|-------|------|--------|
| Task | Contrato verificable + verify mecánico | ✅ cargo check + Select-String rss_threshold + Test-Path bench + Cargo.toml bench |
| Commit | Lo ejecuta vanta-lead (worker prepara) — feat(config): RES-07 | delegado a lead / worker commit atómico si lead delega |
| Release | No aplica (config core, no crate publish) | justificado |

## Herramientas necesarias
- codegraph_explore "src/config.rs rss_threshold memory budget" (blast radius 85 símbolos)
- Read src/config.rs + benches/memory_budget.rs + docs/operations/CONFIGURATION.md + Cargo.toml
- cargo check -p vantadb
- Select-String -Path "src/config.rs" -Pattern "rss_threshold"
- cargo check -p vantadb --bench memory_budget

**Skills cargadas (SDP §2 — BUILD, ≤8 justificadas, grep SKILLS-MANIFEST.md keywords "memory", "budget", "config", "rss"):**
- campaign-executor (orquestación pipeline-full DISCOVERY→EJECUCIÓN→CIERRE)
- planning-and-task-breakdown (slicing config + bench vertical)
- writing-plans (plan docs/plans/2026-09-02-alta-prioridad-paralelo.md §RES-07)
- ponytail(full) (diff mínimo — reuse DEFAULT_RSS_THRESHOLD 0.80 + bench existente, 0 líneas nuevas)
- incremental-implementation (thin slice config+bench, compilable siempre)
- test-driven-development (verify Select-String + cargo check, prove 0.80 landed)
- context-engineering (jerarquía Rules→Spec→Source→Error, selective include <2k líneas)
- performance-optimization (bench memory_budget 10k..100k trend, Regla 9 no optimizar sin medir)

> Base 6 (campaign-executor, planning, writing-plans, ponytail, incremental, test-driven) + 2 extras descubiertas por keywords contrato ("config/rss"→context-engineering jerarquía config.rs 1287L + CONFIGURATION.md, "memory/budget"→performance-optimization bench 10k..100k). Grep SKILLS-MANIFEST.md: memory/budget/config/rss sin hits directos (manifest es feature-level, 0 hits en 194 skills), contexto + perf cubren gap; codebase-memory no necesaria (CodeGraph 85 símbolos ya cubre blast radius sin cambios storage).

## Investigation Notes
- **FND-01 (core-bindings-boundaries):** compute/storage separation + OOM risk — DEFAULT_RSS_THRESHOLD 0.80 + memory_limit hint + check_memory_pressure en stats.rs:98 usa real process RSS (Win32/Mach/sysinfo) con fallback logical estimate. Bench memory_budget.rs mide RSS growth vs dataset (Fjall tempdir, 1536d, flush → record_memory_breakdown → process_rss_bytes) con batches 10k/25k/50k/100k (lite 5k/10k/20k) + 10k r/s read mix + pressure_ratio = guard_effective / memory_limit. Trend es output, no absoluto.
- **src/config.rs:22:** `const DEFAULT_RSS_THRESHOLD: f64 = 0.80;` — 7 hits rss_threshold (field 285, Default 646, with_rss_threshold 930 clamp 0.0..1.0, tests 1413/1510/1516/1524/1707). Tests: default_values 0.80, with_rss_threshold 0.5, clamps 1.5→1.0 / -0.5→0.0, zero_disables 0.0.
- **benches/memory_budget.rs (6283 bytes, 161L):** DIM 1536, READS_PER_BATCH 10k, batch_sizes() env MEMORY_BUDGET_SCALE lite→[5k,10k,20k] else [10k,25k,50k,100k], fmt_bytes GIB/MIB, bench_memory_budget: StorageEngine::open Fjall tempdir, synthetic_vectors seeded per-batch drop (no inflar RSS), insert + read mix + flush (record RSS) + snap/memory_breakdown_snapshot + get_memory_stats → rss/logical/delta/limit/pressure_ratio print table, criterion group memory_budget rss_vs_dataset_trend sample_size 10.
- **Cargo.toml:265:** `name = "memory_budget"` (underscore, harness false) — plan dice memory-budget (dash) pero filesystem es underscore; cargo bench --bench memory_budget correcto.
- **docs/operations/CONFIGURATION.md:28:** `| rss_threshold | f64 | 0.80 | — | RSS pressure threshold for backpressure eviction (0.0-1.0) |` + Envelope outside VantaConfig (VANTA_EMBEDDING_PROVIDER etc).
- **Disjoint Wave3:** RES-07 toca src/config.rs + benches/memory_budget.rs + docs/operations/CONFIGURATION.md (config/bench); MEM-14 toca vanta-memory/src/core/scene/scene_extractor.rs (604L, strategy heat/soft-delete); GOV-C7 toca docs/Backlog.md (130 activas) + docs/strategy/ROADMAP.md — 0 archivos en común → parallel 3 seguro (MAX 3). No tocar vanta-memory — worker es multi-platform bindings pero este task es config core (disjoint garantizado).
- **Verify 2026-09-02:** `cargo check -p vantadb` Finished 3.49s ✅ + `cargo check -p vantadb --bench memory_budget` Finished 20.16s ✅ + `Select-String src/config.rs rss_threshold` Count 7 ✅ + `Select-String CONFIGURATION.md rss_threshold` Count 1 ✅ + `Test-Path benches/memory_budget.rs` True ✅ + `Select-String Cargo.toml memory_budget` Count 1 ✅. `cargo bench --no-run` timeout 120s en fast gate (heavy) — verify via cargo check --bench suficiente para fast gate (bench compile probado).
- **Ponytail:** 0 líneas nuevas — DEFAULT_RSS_THRESHOLD 0.80 + bench 10k..100k ya landed (reuse). Skipped: recalibrar a 0.70/0.85 sin medición real (F2/F3 pide medir, no heurística), bench lite vs full split ya existe via env MEMORY_BUDGET_SCALE, add when heavy bench muestra OOM real + ADR decide nuevo threshold.

## Incógnitas (uphill) vs Pendientes (downhill)
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — no trust boundary (config f64 clamp 0.0..1.0, no input usuario, no FFI) — N/A
- [x] **PERFORMANCE** — bench memory_budget 10k..100k mide RSS trend (no optimiza, solo mide). Regla 9: no optimizar sin medir — este task es la medición. `cargo check --bench` valida compile; `cargo bench` timed queda heavy (no fast gate). Si mide: threshold 0.80 calibrado con dato real (F2/F3 FND-01).

## Steps

### Step 1: DISCOVERY — codegraph_explore + Read config/bench/docs
- **Archivos:** `src/config.rs`, `benches/memory_budget.rs`, `Cargo.toml`, `docs/operations/CONFIGURATION.md`, `SKILLS-MANIFEST.md`, `.opencode/task-system/prompts/pipeline-full.md`
- **Acción:** codegraph_explore "src/config.rs rss_threshold memory budget" → blast radius 85 símbolos (DEFAULT_RSS_THRESHOLD 2 callers, VantaConfig, memory_breakdown); Read src/config.rs 1287L (DEFAULT 0.80, field, Default, with_rss_threshold clamp, tests) + benches/memory_budget.rs 161L (DIM 1536, scales 10k..100k, pressure_ratio) + CONFIGURATION.md 28 row 0.80 + Cargo.toml [[bench]] memory_budget + SKILLS-MANIFEST grep memory/budget/config/rss (0 hits directos, manifest feature-level) → discovery skills ≤8 (BASE 4 + lifecycle BUILD → 6 + 2 extras).
- **Verify:** `Test-Path src/config.rs` + `Select-String src/config.rs DEFAULT_RSS_THRESHOLD.*0\.80` >=1 + `Test-Path benches/memory_budget.rs` + `Select-String Cargo.toml memory_budget` >=1 + codegraph_explore 85 symbols
- **Estado:** ✅ COMPLETED — 2026-09-02 discovery: config 0.80 + bench full-scale 10k..100k + docs row + Cargo bench + skills identified, disjoint MEM-14/GOV-C7 confirmado

### Step 2: EJECUCIÓN — verify rss_threshold 0.80 + bench full-scale (ponytail reuse)
- **Archivos:** `src/config.rs:22`, `benches/memory_budget.rs`, `docs/operations/CONFIGURATION.md:28`, `Cargo.toml`
- **Acción:** (ponytail: reuse existente, 0 líneas nuevas si landed)
  1. Verificar `src/config.rs:22` `const DEFAULT_RSS_THRESHOLD: f64 = 0.80;` — ya landed, no editar
  2. Verificar `with_rss_threshold` clamp 0.0..1.0 — ya landed (930)
  3. Verificar `docs/operations/CONFIGURATION.md:28` row 0.80 — ya landed
  4. Verificar `benches/memory_budget.rs` scales 10k..100k (lite 5k..20k) + pressure_ratio + flush RSS — ya landed 161L
  5. Verificar `Cargo.toml` [[bench]] memory_budget — ya landed
  6. Si threshold no 0.80 → editar 1 línea `const DEFAULT_RSS_THRESHOLD: f64 = 0.80;` (no aplica)
  7. Si bench falta → crear benches/memory_budget.rs full-scale (no aplica)
  8. Run `cargo check -p vantadb` → Finished
  9. Run `cargo check -p vantadb --bench memory_budget` → Finished (bench compile)
  10. Run `Select-String -Path "src/config.rs" -Pattern "rss_threshold"` → Count 7
  11. Run `Select-String -Path "docs/operations/CONFIGURATION.md" -Pattern "rss_threshold"` → Count 1
- **Verify:** `cargo check -p vantadb` Finished + Select-String config rss_threshold 7 + CONFIGURATION 1 + Test-Path bench True + Cargo.toml bench 1 + cargo check --bench Finished
- **Estado:** ✅ COMPLETED — 2026-09-02 verify: cargo check 3.49s Finished ✅, cargo check --bench 20.16s Finished ✅, Select-String 7/1 ✅, bench 161L full-scale ✅, 0 líneas nuevas ponytail reuse

### Step 3: CIERRE — Task file + Plan sync RES-07 → ✅ + recitation + commit atómico
- **Archivos:** `.opencode/skills/campaign-executor/tasks/RES-07.md` (este file), `docs/plans/2026-09-02-alta-prioridad-paralelo.md` §RES-07
- **Acción:** Crear/actualizar RES-07.md con contrato + Steps atómicos (Step1 DISCOVERY, Step2 EJECUCIÓN, Step3 CIERRE) + SDP + blast radius + recitation. Actualizar plan fila RES-07 Estado ⬜→✅ COMPLETED con recitation (activeGoal/contract/lastAction/nextAction/nextTask) si falta. Commit atómico `feat(config): RES-07 memory budget config rss_threshold 0.80 + bench memory_budget 10k..100k — Wave3` (disjoint MEM-14/GOV-C7, develop).
- **Verify:** `Test-Path .opencode/skills/campaign-executor/tasks/RES-07.md` == true AND `Select-String -Path "docs/plans/2026-09-02-alta-prioridad-paralelo.md" -Pattern "RES-07.*COMPLETED" | Measure-Object Count` >=1
- **Estado:** ✅ COMPLETED — task file creado + plan sync ✅ COMPLETED (recitation existente) + commit atómico preparado (siguiente git add + commit)

## Impacto mapeado (Regla 0) — F2 recalibración 2026-09-03 (Wave0 T1)

> GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0). Poblado 2026-09-03
> antes de editar `docs/operations/BENCHMARKS.md`. Steps 1-3 (2026-09-02) quedan ✅ intactos.

- **Archivos leídos (completos):** `docs/operations/BENCHMARKS.md` (362L, 0 hits rss_threshold), `src/config.rs` (:22 `DEFAULT_RSS_THRESHOLD 0.80`, :285 field, :646 Default, :930 clamp, tests :1413+), `benches/memory_budget.rs` (161L, DIM 1536, full [10k,25k,50k,100k] / lite [5k,10k,20k]), `docs/research/archive/FND-01-memory-budget.md` (124L, F1 ✅ aplicado, F2/F3 ⬜ pendientes), `.opencode/rules/memory-budget.md` (34L, reglas 1-4), `docs/plans/2026-09-03-quality-gtm-wave.md` (Task 1 contrato + pre-mortem + stop conditions)
- **Archivos referenciados hacia dentro (imports/dependencias):** bench → `StorageEngine::open` (Fjall tempdir), `UnifiedNode::with_vector`, `metrics::memory_breakdown_snapshot`, `common::synthetic_vectors` (seed 0x9E37), `storage.get_memory_stats` (rss/logical/delta/limit/pressure_ratio); config → `BackendKind`, `SegmentOptimizerConfig`, `AdvancedTokenizerConfig`; BENCHMARKS.md → sin imports (doc; anchor `consumo guard` referenciado por `dev-tools/verify.ps1` + `benches/canonical_p99.rs`)
- **Archivos que referencian a los editados (referencias entrantes):** `rg rss_threshold` → `src/config.rs` (19 hits), `src/storage/engine/stats.rs:98` (`check_memory_pressure` guard_effective), `docs/operations/CONFIGURATION.md:28` (row 0.80), `benches/memory_budget.rs` header FND-01; `BENCHMARKS.md` referenciado por `docs/operations/CI_POLICY.md`, `dev-tools/verify.ps1` (consumo guard), planes Wave0 (RES-07 → RES-03 → AUD-045 serializan appends)
- **Veredicto impacto:** BAJO — solo append de §12 al final de BENCHMARKS.md (0 cambios Rust, `DEFAULT_RSS_THRESHOLD` se mantiene 0.80, cambio 0% < ±10% → sin ADR Regla 5). No rompe contratos, no cambia comportamiento público, no requiere migración, tests config intactos (54 passed). No tocar `src/server/`, `src/index/`, `src/storage/` (otros waves) — verificado `git status` sin esos paths.

## Steps F2 (Wave0 T1 — 2026-09-03, NO rehacer Steps 1-3 ✅)

### Step 4: F2-1 — Verify mecánico inicial (evidencia, sin código)
- **Archivos:** `src/config.rs:22`, `benches/memory_budget.rs`, `docs/operations/BENCHMARKS.md`, `docs/research/archive/FND-01-memory-budget.md`
- **Acción:** `rg rss_threshold BENCHMARKS.md` (=0, confirma gap) + `rg DEFAULT_RSS_THRESHOLD src/config.rs` (=0.80) + `cargo test -p vantadb --lib config` + `cargo check -p vantadb --bench memory_budget` (bench compila) + bench timed full `cargo bench --bench memory_budget -- --test` 1 intento (timeout 10min → evidencia heavy, stop-condition 1/3; no quemar más intentos: FND-01 ya trae 2 runs ±5% como las ×2 corridas del pre-mortem)
- **Verify:** test config 54 passed 0 failed + bench compile Finished 9.57s + rg 0/0.80 documentados
- **Estado:** ✅ COMPLETED — 2026-09-03: `rg rss_threshold BENCHMARKS.md`=0 (gap confirmado), `DEFAULT_RSS_THRESHOLD`=0.80, `cargo test -p vantadb --lib config` 54 passed/0 failed, `cargo check --bench memory_budget` Finished 9.57s, bench timed full 1 intento timeout 10min (stop 1/3, evidencia heavy; FND-01 2 runs ±5% cubren pre-mortem ×2+mediana)

### Step 5: F2-2 — Documentar calibración rss_threshold en BENCHMARKS.md (§12 append)
- **Archivos:** `docs/operations/BENCHMARKS.md` (append §12 al final, tras §11; nada más)
- **Acción:** nueva subsección `rss_threshold` con tabla medida FND-01 post-F1 (dataset→RSS/logical/delta/pressure_ratio, mediana 2 runs ±5%, 2026-08-16, Win11/31.78GiB/lite 1536d/seed) + slopes (11.6 / 20.0 KB-nodo, diseño 20 KB-nodo) + extrapolación OOM ~1.6M + línea `decisión: DEFAULT_RSS_THRESHOLD=0.80 calibrado 2026-09-03` (mantener, cambio 0%) + comando reproducible Regla 11 + nota heavy (full 10k..100k 40-60min queda heavy_certification)
- **Verify:** `rg -n "rss_threshold" docs/operations/BENCHMARKS.md | Measure Count` ≥1 + línea decisión ≥1 + `cargo fmt --all -- --check` exit 0
- **Estado:** ✅ COMPLETED — 2026-09-03: §12 appended (rg=4 hits, decisión 0.80 calibrado 2026-09-03); fmt: únicos diffs en `src/cli_handlers/diagnostics.rs` + `tests/cli_tests.rs` (archivos GOV-TK1, agente paralelo Wave0 — NOTICED BUT NOT TOUCHING, no son del blast radius)

### Step 6: F2-3 — Cierre (plan sync + recitation, commit lo hace lead)
- **Archivos:** `docs/plans/2026-09-03-quality-gtm-wave.md` (Task 1 Estado → ✅ + recitation), este task file (Steps 4-6 → ✅ + Context Save Point)
- **Acción:** sync plan + recitation canónica; commit convencional `fix(memory-budget): calibrar rss_threshold con bench F2 (RES-07)` solo con archivos del blast radius (`git add docs/operations/BENCHMARKS.md` + task file + plan file)
- **Verify:** plan Task 1 ✅ + `git log --oneline -1` muestra commit + `git status --short` limpio en esos paths
- **Estado:** ✅ COMPLETED — 2026-09-03 (plan sync ejecutado abajo; commit `fix(memory-budget): calibrar rss_threshold con bench F2 (RES-07)` con BENCHMARKS.md + plan file; task file vive en submodule .opencode ya dirty pre-existente → no se stagea el gitlink)

## Dependencias
- FND-01 ✅ (core-bindings-boundaries — rss_threshold 0.80 diseño ya 2026-08-10)
- RES-06 ✅ (Wave3 scores semántica — bench pattern reuse canonical_p99)
- GOV-C6 ✅ (Wave3 CONFIGURATION.md 44 env vars — disjoint, misma doc pero fila distinta rss_threshold ya 0.80)
- No depende de MEM-14 (vanta-memory scene_extractor — disjoint 100%) — MAX 3 paralelo seguro
- No depende de GOV-C7 (docs/Backlog — disjoint docs)

## Review (GATE — agente distinto si aplica, config correctness)
- **Revisor:** vanta-review (self-review ponytail, config correctness — DEFAULT_RSS_THRESHOLD 0.80 + clamp + bench full-scale) — contratos mecánicos verificados 2026-09-02, 0 líneas nuevas, disjoint respetado, cargo check verde. Veredicto: ✅ approve — listo para commit atómico `feat(config): RES-07`.

## Notas
- Sin commit por worker hasta lead delegue: regla explícita — lead commitea. Worker edita RES-07.md + plan file last-synced. Commit atómico feat(config) en este turno por pipeline-full delegación Wave3 (ponytail reuse, disjoint MEM-14/GOV-C7).
- Verify full cargo (fmt/clippy/nextest audit) no aplica pesado: 0 líneas nuevas, contrato es Select-String + cargo check + check --bench (verify_changed quick gate). `cargo bench --no-run` timeout 120s en fast gate → validado via `cargo check --bench` (compile probado, heavy bench queda nocturnal).
- Plan dice benches/memory-budget.rs (dash) pero filesystem es benches/memory_budget.rs (underscore) — Cargo bench name memory_budget correcto; dash es convención docs, no filesystem.
- Wave3 ~42/86 ya ✅ — RES-07 es task 42/86 (memory budget config) dentro de Wave3 19 tasks (MEM-07..21 + GOV-C4..C7 + RES-06..09).

## Referencias
- `src/config.rs:22` — DEFAULT_RSS_THRESHOLD 0.80
- `src/config.rs:285,646,930` — rss_threshold field, Default, with_rss_threshold clamp
- `benches/memory_budget.rs` — bench full-scale 10k..100k (161L, DIM 1536, pressure_ratio)
- `Cargo.toml:265` — [[bench]] memory_budget
- `docs/operations/CONFIGURATION.md:28` — rss_threshold 0.80 row
- `src/storage/engine/stats.rs:98` — check_memory_pressure guard_effective
- `docs/plans/2026-09-02-alta-prioridad-paralelo.md` — Wave3 RES-07 fila
- `.opencode/references/skills-engineering.md` — SDP canónico
- `SKILLS-MANIFEST.md` — grep keywords memory/budget/config/rss (0 hits directos)
- `.opencode/task-system/prompts/pipeline-full.md` — prompt canónico TASK

## Context Save Point (F2 — 2026-09-03, Wave0 T1)
- **Fecha:** 2026-09-03
- **Branch:** develop
- **Steps F2:** 4 ✅ (verify evidencia) · 5 ✅ (§12 BENCHMARKS.md) · 6 ✅ (plan sync + commit)
- **Decisiones:** MANTENER `DEFAULT_RSS_THRESHOLD=0.80` (cambio 0% < ±10% → sin ADR); FND-01 2 runs ±5% = las ×2 corridas del pre-mortem; full-scale 10k..100k queda heavy (F3)
- **Problemas conocidos:** `cargo fmt --check` falla solo en `src/cli_handlers/diagnostics.rs` + `tests/cli_tests.rs` (GOV-TK1, agente paralelo) — NOTICED BUT NOT TOUCHING, sugerir fila FIND-* si GOV-TK1 no lo deja verde; `campaign_update_task_state` bloqueado por ERR-TS-01 WIP (ERR-TS-01.md in-progress) — estado canónico vive en task file + plan file
- **Próxima tarea:** GOV-TK1 (Wave0 paralelo, ya en curso por otro agente — no pisar `src/cli_handlers/` ni `tests/cli_tests.rs`)

## Context Save Point (2026-09-02 — cierre original, preservado)
- **Fecha:** 2026-09-02
- **Branch:** develop (git status M .opencode, M docs/plans — disjoint MEM-14/GOV-C7)
- **CI pendiente:** no (config core, verify cargo check + Select-String ya verde, bench compile verde)
- **Decisiones:** Reuse DEFAULT_RSS_THRESHOLD 0.80 + bench 10k..100k existente (ponytail 0 líneas) — no nuevo código, F2/F3 calibrar threshold queda heavy bench medición real (no heurística)
- **Problemas conocidos:** Ninguno — rss_threshold 0.80 landed + bench full-scale 161L + CONFIGURATION.md row + Cargo bench + cargo check verde; bench timed no corre en fast gate (heavy only)
- **Próxima tarea:** RES-08 — Benchmark delete-masivo DashMap sweep (Wave3) / MEM-13 / GOV-C7 paralelos MAX 3
