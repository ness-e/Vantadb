# FND-10: Regla 9 (No optimizar sin medir) + benchmark canónico P99

- **ID:** FND-10
- **Fuente:** docs/Backlog.md:498 (P20b, prio 🔴)
- **Plan:** docs/plans/2026-08-16-wave-p20-tsys.md (Wave 3, Task FND-10)
- **Sub-agente:** vanta-tuner
- **Gate:** 🔴 (Regla 9 + baseline)
- **Contrato:** "regla en AGENTS.md + benchmark canónico P99 ejecutable con baseline registrado"

## Contexto

FND-11 (otra wave, ya commiteada) tomó **Regla 10** (AI Guardian). Las reglas existentes llegan hasta 8. Esta tarea crea **Regla 9** — libre — en `.opencode/AGENTS.md`.

El backlog (Backlog.md:498) la define: "Regla 9 — No optimizar sin medir + benchmark canónico P99 — Investigar benchmarks existentes (PERF-02 baseline criterion, `benches/`). Verificación: establecer benchmark canónico P99 (insert 100k×1536d, buscar) como baseline de no-regresión. Implementación: regla en AGENTS.md (todo cambio de rendimiento exige benchmark before/after). DoD: regla + benchmark canónico ejecutable con baseline registrado."

## DISCOVERY

Benchmarks existentes (formato criterion 0.8, `harness = false`, registro `[[bench]]` en Cargo.toml):

- `benches/hnsw_pure.rs` — CPIndex puro, dim **1536** (misma dim que el contrato), insert+search, pero solo 10k nodes, sin P99 explícito. Es el patrón base.
- `benches/high_density.rs` — StorageEngine completo, 768d, 250k/1M, sin P99.
- `benches/common/mod.rs` — `apply_fixed_profile` (warm-up 3s, measurement 5s, CI 0.95, significance 0.05, noise 0.05) + dataset determinístico.
- PERF-02 (✅ 2026-08-12) — baseline rig criterion: perfiles fijos + critcmp regression gate + dataset sintético persistido (`benches/data/synthetic_dataset.bin`, dim 256 — no cubre el contrato 1536d).
- `docs/operations/BENCHMARKS.md` — métricas certificadas (p50/p95/p99 de SDK Python), sin benchmark canónico Rust insert 100k×1536d + search.
- `docs/operations/PERFORMANCE_TUNING.md` §7 — comandos de benchmark (`cargo bench`), perfil con RUSTFLAGS target-cpu=native.

**Conclusión:** ningún bench existente cubre el contrato (insert 100k × 1536d + search con P99). Se crea `benches/canonical_p99.rs` nuevo. No se modifican benches existentes.

## Steps

### Step 1: Crear `benches/canonical_p99.rs` ✅
- Bench criterion nuevo: insert 100k × 1536d (CPIndex, iter_custom) + search 1000 queries con histograma P50/P95/P99 explícito.
- Registro `[[bench]] name = "canonical_p99", harness = false` en Cargo.toml (tras ivf_bench).
- Patrón copiado de `hnsw_pure.rs` (config HNSW m=16, ef_construction=100, ef_search=50, cosine) + `common::apply_fixed_profile`.

### Step 2: Compilar y correr baseline ✅
- `cargo bench -p vantadb --bench canonical_p99 -- --quick` (baseline **quick**: criterion quick mode, sample reducido — documentado).
- Registrar resultados: insert total/throughput + search p50/p95/p99.

### Step 3: Regla 9 en `.opencode/AGENTS.md` ✅
- Insertar "### Regla 9: No Optimizar sin Medir" entre Regla 8 (L555) y Regla 10 (L569).
- Tabla estilo Regla 1/2: disparador → acción obligatoria. Referencia explícita al benchmark canónico `cargo bench -p vantadb --bench canonical_p99` y al requisito before/after con P99.

### Step 4: Documentar baseline ✅
- Sección en `docs/operations/BENCHMARKS.md` (inglés, fuente de verdad) con el baseline registrado: entorno (CPU/RAM/OS), comando, resultados P99 insert+search, fecha.
- Este task file registra el baseline.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `benches/hnsw_pure.rs`, `benches/high_density.rs`, `benches/common/mod.rs`, `Cargo.toml` (§benches), `docs/operations/BENCHMARKS.md`, `docs/operations/PERFORMANCE_TUNING.md` (§7), `.opencode/AGENTS.md` (Reglas 1-10), plan file, task file PERF-02.md.
- **Referencias hacia dentro:** `benches/canonical_p99.rs` (nuevo, sin referencias entrantes). AGENTS.md — Regla 9: sin referencias entrantes al texto nuevo (reglas se consultan por número).
- **Referencias salientes:** canonical_p99.rs → `vantadb::index::{CPIndex, FilterBitset, HnswConfig}`, `vantadb::node::{VectorRepresentations, DistanceMetric}`, `common::apply_fixed_profile`. AGENTS.md Regla 9 → `benches/canonical_p99.rs`, BENCHMARKS.md.
- **Veredicto:** impacto cero sobre código existente. Solo archivo nuevo en `benches/`, registro `[[bench]]` en Cargo.toml (no toca benches existentes), texto nuevo en AGENTS.md, sección nueva en BENCHMARKS.md. Cargo.toml: el registro bench es aditivo, no modifica entries existentes.

## Archivos protegidos (NO tocar)

- `docs/Backlog.md`, `.opencode/skills/campaign-executor/tasks/AUD-024.md`, `.opencode/task-system/enforcement/verify-log.jsonl`, `completions/_vanta-cli.ps1`, `docs/plans/2026-08-16-wave-p20-tsys.md`, `.opencode/agents/*`.

## Baseline registrado (Step 4)

> Ver `docs/operations/BENCHMARKS.md` § "Canonical P99 Baseline (FND-10)" — se agrega en Step 4.

**Entorno (baseline quick):**
- CPU: 12th Gen Intel Core i5-1235U (10 cores / 12 threads)
- RAM: 31.8 GB total (14.3 GB libres al momento del run)
- OS: Microsoft Windows 11 Pro 10.0.26200
- Comando: `cargo bench -p vantadb --bench canonical_p99 -- --quick`
- Modo: quick (criterion `--quick` — significance-based, sample reducido)
- Fecha: 2026-08-16

**Resultados (se llenan tras el run en Step 2):**
- Insert 100k × 1536d: **322.59 s** (~310 vec/s, build único medido por iter_custom)
- Search p50/p95/p99 (1000 queries, top_k=10): **p50=1.4786ms / p95=2.3708ms / p99=3.0746ms**
- Search batch (1000 queries): **1.58 s**

## Contract verification

- `Select-String .opencode/AGENTS.md -Pattern "Regla 9"` → debe existir y referenciar `canonical_p99`
- `Test-Path benches/canonical_p99.rs` → True
- Baseline en `docs/operations/BENCHMARKS.md` (P99 insert + search con entorno)

## Estado

- **Step 1:** ✅ COMPLETO — `benches/canonical_p99.rs` creado + registro `[[bench]]` en Cargo.toml (harness=false, tras ivf_bench)
- **Step 2:** ✅ COMPLETO — compila (`cargo bench --no-run` OK, 3m14s) y corre baseline (`-- --quick`): insert 322.59s, search p99=3.07ms
- **Step 3:** ✅ COMPLETO — Regla 9 insertada en `.opencode/AGENTS.md` entre Regla 8 y Regla 10, tabla estilo Regla 1/2, referencia explícita a `canonical_p99` y BENCHMARKS.md
- **Step 4:** ✅ COMPLETO — §8 "Canonical P99 Baseline (FND-10)" agregada a `docs/operations/BENCHMARKS.md` con entorno, comando, resultados y nota de uso

<!-- Context Save Point: baseline quick registrado 2026-08-16; sin commit (lead commitea al cerrar wave) -->