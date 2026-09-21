# TBH-09 — benches/crash_recovery.rs

## Metadata
- **Plan file:** docs/plans/2026-08-30-testing-bench-harden.md
- **Created:** 2026-08-31T13:05
- **last-synced:** 2026-08-31T13:05
- **Estado:** ✅ COMPLETED
- **Tipo (campaign_detect_task_type):** bench / new-criterion-bench (1 new bench file + Cargo.toml block)
- **Esfuerzo:** 🟡 Medio
- **Prioridad:** MEDIA (Phase 2, gap crítico de benchmarking — segundo gap identificado por auditoría 2026-08-30)

## Impacto mapeado (Regla 0)

### Archivos leídos completos
| Archivo | Líneas | Notas |
|---------|--------|-------|
| `benches/wal_throughput.rs` | 155 | Plantilla exacta a copiar: `criterion_main!` + `iter_custom` + `apply_fixed_profile` + `latency` printing. |
| `benches/common/mod.rs` | 102 | `apply_fixed_profile` (warm 3s + measure 5s + 0.95 CI). |
| `src/bin/crash_helper.rs` | 57 | Usa `StorageEngine::open_with_config(db_path, Some(config))` con `SyncMode::Always`. NO se modifica. |
| `src/storage/engine/init.rs` | 623 | `StorageEngine::open()` (default config) y `open_with_config(path, Some(config))` (custom). Internamente llaman a `recover_state()` que cronometra WAL replay (`wal_replay_ms`). El crate reporta esto a `crate::metrics::record_startup()`. |
| `Cargo.toml` | 684 | Líneas 192-266: 19 `[[bench]] harness = false` (incluye `wal_throughput` recién agregado). Añadir `[[bench]] crash_recovery` después de línea 266. |

### Referencias hacia adentro (outbound references de los archivos tocados)
- `benches/crash_recovery.rs` → usa `vantadb::storage::StorageEngine` y `vantadb::node::UnifiedNode` (públicos per lib.rs).
- `Cargo.toml` → añade `[[bench]] name = "crash_recovery" harness = false`. Patrón exacto de las 19 benches existentes.

### Referencias hacia afuera (inbound — quién lee o importa estos archivos)
- `benches/crash_recovery.rs` → será invocado por `cargo bench -p vantadb --bench crash_recovery` y (futuro) `.github/workflows/heavy-bench-nightly-51.yml` cuando se extienda el matrix.
- `Cargo.toml` → ya consumido por `cargo build/check/bench -p vantadb`.

### Veredicto de impacto
**Mínimo, blast radius = 0 archivos aguas abajo.** Cambios son:
1. Nuevo archivo `benches/crash_recovery.rs` (1 file nuevo).
2. 1 `[[bench]]` block nuevo en `Cargo.toml` (3 líneas agregadas).
3. Cargo.lock se actualiza solo — añadido por `cargo build`, NO editado a mano.

### SDP (Skill Discovery Protocol)
- `campaign-executor` (base, auto-cargada via MCP).
- `incremental-implementation` (regla 6 del agente worker) — slice vertical delgado.
- `test-driven-development` — el bench es el "test" (mide comportamiento; cumple rol de regression detector).
- `code-simplification` (ponytail full) — bench mínimo, sin sobreingeniería.

### Gates
- **P (pre-flight):** no requerido (no toca WAL/storage/vector core — propiedad de vanta-arch/vanta-engine).
- **D (discovery):** disparado — leído WAL API + benches reference + Cargo.toml + crash_helper + engine/init.rs; veredicto arriba.
- **V (verify):** contrato mecánico (5 checks), ver `## Contrato`.
- **C (commit):** conventional commit `feat(TBH-09):`.

## Contexto

TBH-08 midió WAL **write throughput** (qué tan rápido el WAL ingiere). TBH-09 mide el otro extremo: **qué tan rápido el engine se recupera de un crash** (open + WAL replay cronometrado). La auditoría multi-agente del 2026-08-30 identificó esto como segundo gap crítico: **no podemos saber si un cambio al startup degrada performance hasta medirlo**. Sin este bench, cualquier cambio en `recover_state` o `StorageEngine::open` es un salto a ciegas.

El bench cronometra `StorageEngine::open_with_config()` sobre un directorio que contiene un WAL pre-poblado con N records. El sweep es exactamente lo que el plan especifica: `[100, 10k, 100k]`.

## Contrato (verificable mecánicamente)

```
1. benches/crash_recovery.rs existe y compila
2. benches/crash_recovery.rs contiene `criterion_group!` + `criterion_main!` válidos
3. Cargo.toml tiene `[[bench]] name = "crash_recovery" harness = false`
4. cargo check -p vantadb --benches → exit 0
5. cargo build -p vantadb --benches → exit 0
6. cargo fmt --check → exit 0
7. Sweep del bench cubre: corpus sizes [100, 10_000, 100_000]
```

## Spec del bench

| Dimensión | Valores |
|-----------|---------|
| Corpus size (WAL records pre-populados) | 100, 10_000, 100_000 |
| Sync mode durante pre-populate | `SyncMode::Always` (para que el WAL quede realmente durable — refleja "crash" real) |
| Sync mode durante open/recovery | default (Periodic) — es el path real de recovery |
| Setup por iter | (a) tempdir fresh, (b) `StorageEngine::open_with_config()` + insertar N nodos, (c) drop engine (escribe `vanta.wal`), (d) re-`open_with_config()` cronometrado |
| Métrica primary | wall-time de `open_with_config()` (la que ya reporta `metrics::record_startup`) |
| Métrica throughput | `criterion::Throughput::Elements(N)` → open/sec per N |
| Profile | `common::apply_fixed_profile` (warm 3s, measure 5s, 0.95 CI) |
| `sample_size` | 10 |

## Archivos a crear/modificar

| Archivo | Acción |
|---------|--------|
| `benches/crash_recovery.rs` | CREAR |
| `Cargo.toml` (workspace) | AÑADIR 1 `[[bench]]` block al final de la lista (línea ~267, después de `wal_throughput`) |
| `Cargo.lock` | regenerado por `cargo build` |
| `.opencode/skills/campaign-executor/tasks/TBH-09.md` | CREAR (este archivo) |

## Steps

### Step 1: Crear task file ✅
- **Acción:** este archivo. **Estado:** ✅

### Step 2: PLAN — diseñar sweep ✅
- **Acción:** spec arriba (3 corpus sizes; pre-populate con `SyncMode::Always` para que el WAL sea durable; open cronometrado con default config). **Estado:** ✅

### Step 3: ACT — crear `benches/crash_recovery.rs` ⬜ PENDING
- **Acción:** escribir bench mínimo que cumpla contrato.
- **Ponytail reflex:**
  - NO medir checkpoint / transactions / HNSW rebuild por separado (eso vive en benches dedicados o `wal.rs` tests).
  - NO usar `iter` — usar `iter_custom` para que el corpus pre-populado se regenere en cada iter (si lo dejara fuera del loop, los criterios no serían comparables).
  - NO usar `Persistent corpus on disk` (overhead de I/O para corpus setup enmascara la medición de open/recovery; el setup debe ser trivial).
  - **3 corpus sizes exactos** — ni más ni menos (spec del plan).

### Step 4: ACT — añadir `[[bench]]` en Cargo.toml ⬜ PENDING
- **Acción:** Edit tool. 3 líneas (`[[bench]]`, `name`, `harness`) después del `wal_throughput`.
- **Verify:** `grep "name = \"crash_recovery\"" Cargo.toml` → 1 match.

### Step 5: VERIFY ⬜ PENDING
- `cargo check -p vantadb --benches`
- `cargo build -p vantadb --benches`
- `cargo fmt --check`

### Step 6: COMMIT ⬜ PENDING
- `git add benches/crash_recovery.rs Cargo.toml Cargo.lock .opencode/skills/campaign-executor/tasks/TBH-09.md`
- `git commit -m "feat(TBH-09): add crash_recovery bench (open + WAL replay sweep)"`

## Dependencias

- TBH-08 (WAL throughput bench) ✅ COMPLETED — el patrón a copiar vive en `benches/wal_throughput.rs`.
- Ninguna otra dependencia.
- **Pre-existente no relacionado:** `vantadb-mcp/tests/context_tests.rs:70` no compila (FIND-MCP-001). Usar `-p vantadb` evita el bug.

## Notas

- **Ponytail reflex (mantener bench mínimo):**
  - 1 sample = `StorageEngine::open_with_config()` sobre un directorio con WAL pre-poblado.
  - Pre-populate **dentro** del bench (no precomputado on-disk) — aislado, reproducible, sin dependencies externas.
  - `SyncMode::Always` durante pre-populate garantiza fsync real (un crash post-iter puede dejar datos no escritos; con Always todos los datos están en disco).
  - Open durante el bench usa `VantaConfig::default()` (SyncMode::Periodic — path real).
- **Per-rule Regla 9:** este bench ES la medición que faltaba. Cumple Regla 9 cerrando el agujero de medición del startup path.
- **Wall-time sensitivity:** en Windows el `open_with_config` puede tener variabilidad alta (filesystem cache warm-up, antivirus scanning). El `sample_size(10)` + `apply_fixed_profile` mitiga, pero la primera ejecución del bench puede tardar más de lo normal.
- **Self-check (post-create):** la primera ejecución del bench debe poder ejecutarse sin errores. Si falla el bench, la regresión es detectable.

## Context Save Point
- **Fecha:** 2026-08-31
- **Branch:** develop
- **CI pendiente:** no (cambio aditivo, no toca WAL/storage/vector ni core types)
- **Decisiones:**
  - Sweep: 3 corpus sizes [100, 10_000, 100_000] (spec del plan, no más).
  - Pre-populate: `SyncMode::Always` + `insert()` (refleja crash real con WAL durable).
  - Open cronometrado: default config (`SyncMode::Periodic`).
  - `iter_custom` (no `iter`) — recrea tempdir cada iter (necesario para que el corpus sea fresh y los open timings sean comparables).
  - NO usar corpus on-disk persistente (overhead enmascara la medición).
- **Próxima tarea:** handoff al orquestador.