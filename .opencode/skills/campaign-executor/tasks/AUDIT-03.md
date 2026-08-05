# AUDIT-03: Miri guard sobre el CORE Rust (7 bloques UB_POTENTIAL de INV-024)

## Metadata
- **Plan file:** `docs/plans/2026-08-05-backlog-validation-actions.md`
- **Creado:** 2026-08-05
- **last-synced:** 2026-08-05
- **Estado:** ✅ DONE (2026-08-05)
- **Severidad:** 🔴 Bloqueante
- **Esfuerzo:** 🟢 2-4h

## Objetivo
Ejecutar Miri sobre el core Rust (`cargo +nightly miri test -p vantadb`) con
`MIRIFLAGS=-Zmiri-tree-borrows` para cubrir los 7 bloques UB_POTENTIAL de
INV-024. Ya existe `tests/miri_unsafe.rs`. Premisa re-escalada 2026-08-05:
`vantadb-python` = 0 dev-deps/0 tests Rust/cdylib → Miri NO cubre FFI
CPython/NumPy (esa boundary la cubre AUDIT-04 con repro Python + ASAN).
**Ejecutar DESPUÉS de AUDIT-01** (ya completado: `bff30d38` — getter PyBytes
own w/o raw pointer UB).

## Blast Radius
- **Archivo:** `tests/miri_unsafe.rs` (+ UB candidates: `unsafe` blocks en
  `src/` mapeados en INV-024).
- **Precondición:** AUDIT-01 completado (✅).
- **Riesgo:** Miri no disponible en Windows con toolchain nightly local →
  documentar fallo de herramienta + fallback (runner Linux de CI / WSL) o
  alcance revisionado.

## Contrato
`cargo +nightly miri test -p vantadb` (o `.github/workflows` job )
pasa / produce inventario de UB restantes verificado; 0 panics UB sin fix
documentado.

## Herramientas
- bash (nightly Miri), codegraph (ubicar `unsafe` blocks), web (validar
  Miri flags actuales/docs oficiales miri tree-borrows)

## Steps
### Step 1: Preparar + inventario UB
- **Archivo:** `tests/miri_unsafe.rs` + grep `unsafe` en `src/` (INV-024 list)
- **Acción:** confirmar toolchain nightly + Miri disponible (Windows / WSL /
  CI). Inventariar los 7 bloques UB de INV-024 vs código actual.
- **Verify:** `cargo +nightly --version && rustup component list | grep miri`
- **Estado:** ⬜ PENDING

### Step 2: Ejecutar Miri
- **Acción:** `cargo +nightly miri test -p vantadb` con `MIRIFLAGS=-Zmiri-tree-borrows`.
  Candidato también miri test con target tests/miri_unsafe.rs específico.
  En Windows si Miri no opera: intentar `-p vantadb` vía WSL o documentar que
  se ejecuta en CI Linux job `miri`. Si ninguna vía corre → fallback: revisión
  manual de cada `unsafe` con `// SAFETY:` (saber si el bloque se supervisa).
- **Verify:** salida miri (0 errors / reporte descubierto)
- **Estado:** ⬜ PENDING

### Step 3: Fix o documentación de hallazgos + colaterales
- **Acción:** TL1UBlocas UB genuinos correctos; pseudocontratos sin SAFETY doc. 
  Si no aplicable on box: dejar workflow `ci-miri.yml` job + whitepaper the
  inventories findings.
- **Verify:** `cargo fmt --check && cargo clippy -D warnings`, commit
  `fix(AUDIT-03): ...` u `docs(AUDIT-03)`.
- **Estado:** ⬜ PENDING

## Dependencias
- AUDIT-01 ✅ completado. Paralelo independiente de AUDREP-01/04 (no comparte
  archivo — corre en `tests/miri_unsafe.rs` y `src/` unsafe audit, mientras los
  otros tocan `src/storage/archive.rs`).

## Notas
- `tests/miri_unsafe.rs` ya existe — expandir/afirmar cobertura, no reescribir
  desde cero.
- Regla: verificar contra internet si hay duda sobre flags `-Zmiri-tree-borrows`
  (tag vigente 2026).
- Si Miri no puede correr (Win), NO declarar 0 UB — documentar honestamente en
  el task file + CI job para Linux upfront.

## Context Save Point
- **Fecha:** 2026-08-05
- **Branch:** develop
- **CI pendiente:** sí
- **Decisiones:** destación delante de parallela; Windows constrain measured.
- **Problemas conocidos:** Miri en Windows 2026-08-05 state — doc man GONZ.

## Resultado (2026-08-05)
- **Miri correr en Windows:** ✅ `cargo +nightly miri test -p vantadb --test miri_unsafe
  --no-default-features` con `MIRIFLAGS=-Zmiri-tree-borrows -Zmiri-disable-isolation`:
  **10/10 passed** (9 raw-pointer + 1 engine real path).
- **Hallazgos corregidos (3):**
  1. `stats.rs check_memory_pressure` — `limit == 0` (hardware detection 0 / Miri)
     hacía que TODOS los inserts fallaran con ResourceLimit; 0 = unlimited (bug real).
  2. `vfile.rs` — backing InMemory era `Vec<u8>` (align-1): `base + vector_offset`
     podía quedar misalignado → UB real en `from_raw_parts::<f32>` release. Nuevo
     `AlignedBytes` (4-aligned, Drop propio, Send+Sync) para InMemory + shim
     no-memmap2.
  3. FFI telemetría (QueryWorkingSetEx/mincore/get_native_memory) no implementada
     por Miri → shims `#[cfg(miri)]` en `get_resident_bytes_impl` y `_get_rss_virt`
     (telemetry-only, SAFE FFI).
- **Guard central ya existente:** `read_header` rechaza `vector_offset % 4 != 0`
  (cubre los 7 sites de INV-024) + test de regresión
  `test_vfile_read_header_rejects_misaligned_vector_offset`.
- **Cobertura:** paths reales de engine get/insert/search bajo Miri
  (`miri_engine_in_memory_hnsw_vector_paths`). Variantes mmap NO son
  Miri-runnable (syscall mmap unsupported) → cubiertas por guard central + SAFETY
  comments. No se añadió job CI Linux (Windows corrió Miri sin bloqueo).
- **Gates:** `cargo fmt --check` limpio en archivos tocados, `cargo clippy -D warnings`
  limpio (default + no-default), 341 tests storage + 31 vfile + 33 metrics ✅.
- **Nota worktree:** `benchmarks/graphrag_bench.rs` (untracked, user) no compila
  (`Box<dyn Error>` no Send) — pre-existente, fuera de scope AUDIT-03.