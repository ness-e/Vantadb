# AUDIT-04 — Root-cause crash benchmark Python (0xC0000409)

> **Task:** Task 14 del plan `docs/plans/2026-08-05-backlog-validation-actions.md`
> **Estado:** ✅ COMPLETED
> **Fecha:** 2026-08-05
> **Commit:** `d2c7b0a5` — fix(AUDIT-04): acotar cache_warmer.co_access para evitar OOM en searches
> **Archivo tocado:** `src/cache_warmer.rs` (+97/-1)

## Causa raíz (atribuida con evidencia)

**NO es stack overflow** (daría 0xC00000FD + sin mensaje de allocación). El log real
(`heavy_nocturnal_tests.log:146-149`) muestra:

```
memory allocation of 270352 bytes failed
[FAILED] Python Benchmark Suite (exit code -1073740791)
```

`-1073740791` = `0xC0000409` = `__fastfail` de Rust al abortar por fallo de
asignación (`handle_alloc_error`). El crash es **heap exhaustion por crecimiento
no acotado de `CacheWarmer.co_access`** — un grafo completo de pares O(n²)
(`HashMap<u128, HashMap<u128, u32>>`) alimentado en CADA search y en cada
`get_many`:

- `src/sdk/search/mod.rs:451` (lexical_search), `:556` (prefilter), `:711` (vector_memory_search)
- `src/storage/engine/ops.rs:1429` (get_many registra TODOS los IDs fetched → en lexical_search eso es el set completo de candidatos, ~100+ ids)

El `decay()` cada 1000 eventos solo divide conteos a la mitad (y durante
exactamente 1000 eventos NO corre: `prev % 1000 == 0` con prev=999 no dispara),
así que la tabla crece monotónicamente con pares DISTINTOS. La rama UAF ya fue
descartada por Task 12 (`bff30d38`); `try_numpy_array` copia.

## Evidencia de reproducción (repro `audit04_repro.py`)

| Escenario | Peak Private Bytes | Resultado |
|---|---|---|
| 1K/128d/100q | 122.8 MB | sin crash |
| 2K/64d/200q | 359.7 MB | sin crash |
| **10K/128d/1000q (original) ANTES del fix** | **2514.1 MB** | sin crash en esta máquina (32 GB), pero crecimiento ~2.03 MB/query en hybrid |
| 10K/128d/1000q **DESPUÉS del fix** | **332.9 MB** | sin crash, crecimiento eliminado |

**Experimento decisivo (identical vs distinct queries, 2K/64d):**
- Queries idénticas (mismos hits → pocos pares nuevos): 89.6 → 72 MB (sin crecimiento)
- Queries distintas (pares nuevos por query): 86.5 → 338 MB (+250 MB en el primer batch)

→ El crecimiento es función de los **pares distintos** registrados: exactamente lo
que `co_access` acumula.

## Fix aplicado (acotado, no invasivo)

En `src/cache_warmer.rs`:
- `MAX_CO_ACCESS_PAIRS = 1_000_000` (≈64-90 B/par → ~90 MB techo, vs 2.5 GB antes)
- Campo `pair_count` (AtomicUsize) + `saturated` (AtomicBool, monotónico)
- `record_co_access`: al saturar, deja de INSERTAR pares nuevos; solo refresca
  conteos de pares ya trackeados (preserva prefetch de pares calientes)
- `decay()` reconcilia `pair_count` con la tabla real; `clear()` resetea saturación
- Constructor `with_config_and_cap` (privado) para tests baratos

No toca API pública, no cambia el contrato de `suggest_warm_ids`. Si vanta-arch
quiere reintroducir aprendizaje post-decay, opción: resetear `saturated` cuando
`pair_count < max_pairs / 2` tras decay (histeresis) — no aplicado por simplicidad.

## Verificación

- `cargo test -p vantadb cache_warmer` → 9/9 ok (2 nuevos: `test_pair_cap_saturates_and_stops_learning`, `test_clear_resets_saturation`)
- `cargo clippy -p vantadb --all-targets` → exit 0
- `maturin develop --release` → wheel 0.5.0 instalado
- Benchmark real `vantadb_local_bench.py --size 2000 --dim 64 --queries 200` → **3× exit 0**
- Repro original 10K/128d/1000q post-fix → **exit 0, peak 333 MB** (vs 2.5 GB)

## Notas

- Benchmark NO es el bug: es uso legítimo. El fix es del core.
- El crash original de abril no se reproduce en esta máquina (32 GB RAM) porque
  el proceso llegó a 2.5 GB y sobrevivió; en la máquina original (presión de
  memoria) el allocator falló a los 270 KB. La causa es la misma: crecimiento
  no acotado.
- Probes de integridad: `git status` limpio de artefactos (repro en
  `%TEMP%\opencode\`); plan file NO tocado.
