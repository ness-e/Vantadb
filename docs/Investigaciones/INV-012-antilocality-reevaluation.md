# INV-012: Anti-Locality Disk Layout — re-evaluación

**Fecha:** 2026-08-03
**Estado:** ✅ COMPLETADA
**Fuente:** docs/Backlog.md línea 202
**Tipo:** Benchmark + recomendación (sin implementación)

## Veredicto: WONTFIX confirmado — mejora ~7%, bajo el 9% original y lejos del 15% requerido

Re-ejecutado `benches/vfile_search.rs` (el benchmark original de DRV-130) en la arquitectura actual (LSM + multi-level storage). result no block.

## Resultados (release, 200 queries/batch)

| Grupo | Actual ms/batch | ms/q | Ratio vs in_memory | Baseline DRV-130 ms/q |
|-------|-----------------|------|--------------------|-----------------------|
| `in_memory` | 126.4 | 0.63 | 1x | 3.9 |
| `with_vfile` | 614.5 | 3.07 | 4.9x | 12.2 |
| `with_vfile_compacted` | 571.5 | 2.86 | 4.5x | 11.1 |

**Mejora locativa actual = (614.5 − 571.5)/614.5 ≈ 7.0%** — inferior al 9% de DRV-130, y por debajo del threshold de re-apertura (15%).

## Análisis

1. **Los cambios de arquitectura (LSM + multi-level) NO alteraron el resultado.** `with_vfile` pesa ~4.9x sobre in-memory; la compactación BFS recupera solo ~1 unidad del ratio.
2. **Causa raíz DRV-130 sigue vigente:** la ruta de acceso del search es greedy (distancia-guía) y diverge del orden BFS tras los primeros nodos; el overhead dominante es call/mmap deref, no page misses.
3. **Limitación del benchmark:** con 10K × 128 × f32 ≈ 5MB, el VantaFile entra en page cache del OS; mmap convierte el "I/O" en memoria. Esto infravalora el valor potencial del layout en SSD frío con datasets grandes.

## Recomendación final

**CONFIRMAR WONTFIX. NO re-abrir.** La mejora del layout BFS es ~7%, consistente — in fact ligeramente inferior — al 9% que originó el cierre. Para considerar re-apertura, se debería validar con dataset 1M+ en SSD frío/cold-cache (medir locality real de page faults), lo cual se documenta como limitación, no como work item.

## Notas de proceso

- `cargo bench --bench vfile_search` no completa el grupo `build_index` (100 samples × ~13s = ~22 min); los 3 grupos de search (relevantes) terminan antes y son válidos. Para futuras corridas: `-- --bench vfile_search` selectivo o reducir sample size.
- Absolutos no comparables entre runs (otra máquina); métrica válida es el ratio `compacted/with_vfile` (apples-to-apples, misma máquina) = 7%.