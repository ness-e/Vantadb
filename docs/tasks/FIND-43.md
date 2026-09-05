# FIND-43: Aplanar builder CacheWarmer (no recursivo)

## Metadata
- **Plan file:** docs/plans/2026-08-29-full-backlog-parallel.md
- **W5-2** (Wave 5, parallel 3 con FIND-38 + MOD-15)
- **Creado:** 2026-08-29T21:00
- **last-synced:** 2026-08-29T21:00
- **Estado:** ✅ COMPLETED (2026-08-29T21:05)
- **SDP:** files="src/cache_warmer.rs" keywords=["CacheWarmer","builder","flatten","recursive"] → base-only (incremental-implementation aplicable pero refactor trivial)

## Blast Radius
- **Interno:** `src/cache_warmer.rs` (único archivo)
- **Entrantes (callers):** `new()` (no callers externos — uso interno), `with_config()` (7 callers, todos tests internos en cache_warmer.rs:283,300,316,329,341,369), `with_config_and_cap()` (5 callers, todos tests internos en cache_warmer.rs:383,413,427,457)
- **Salientes:** ninguno (constructores, no dependencias)
- **API pública:** `CacheWarmer::new` y `CacheWarmer::with_config` son `pub` pero `pub(crate)` (no expuestos fuera del crate)
- **Veredicto:** impacto interno solo, blast radius acotado a tests en mismo archivo

## Contrato (del plan file)

```
Select-String -Path "src/cache_warmer.rs" -Pattern "with_config_and_cap" | Measure-Object | Select-Object Count` >= 1 (no recursivo)
AND
cargo check -p vantadb` exit 0
```

**Contrato mecánico:**
1. La función `with_config_and_cap` debe seguir existiendo (>=1 match) — backward compat con 5 tests
2. `cargo check -p vantadb` debe pasar (exit 0)

## Spec

| Decisión | Elección | Justificación |
|----------|----------|---------------|
| Eliminar `new()` | NO — mantener | Default trait + tests lo usan (`Default for CacheWarmer` línea 271) |
| Eliminar `with_config()` | NO — mantener | 7 tests dependen (283, 300, 316, 329, 341, 369) — backward compat |
| Eliminar `with_config_and_cap()` | NO — mantener | 5 tests dependen (383, 413, 427, 457) — el contrato verifica presencia |
| Aplanar recursión `new → with_config → with_config_and_cap` | SÍ | Cada constructor hace su propio `Self { ... }`, sin delegar |
| Crear helper interno `_new_with_thresholds()` | NO | Ponytail: duplicar 8 líneas de struct init es más corto que abstraer |
| Modificar visibilidad de `with_config_and_cap` | NO — mantener `fn` (privado) | ya está así, solo lo usan tests del mismo módulo |

## Pasos atómicos

### Step 1: Aplanar constructores
- **Archivos:** `src/cache_warmer.rs`
- **Acción:** Reemplazar las 3 funciones constructoras para que cada una haga su propio `Self { ... }` directamente, sin delegar entre sí.
- **Verify:** `cargo check -p vantadb` exit 0; tests pasan
- **Estado:** ✅ COMPLETED (2026-08-29T21:05)

### Step 2: Silenciar dead_code en constructores test-only
- **Archivos:** `src/cache_warmer.rs`
- **Acción:** Añadir `#[cfg_attr(not(test), allow(dead_code))]` a `with_config` y `with_config_and_cap` (solo usados en tests del mismo módulo — antes la cadena recursiva `new → with_config → with_config_and_cap` los hacía reachable transitivamente; al aplanar, los dos últimos quedan test-only).
- **Verify:** `cargo check -p vantadb` exit 0 sin warnings
- **Estado:** ✅ COMPLETED (2026-08-29T21:05)

## Herramientas
- codegraph_explore, Read, Edit

## Notes
- vanta-worker no hace commit — staged para vanta-lead (Regla 6 boundary).
- No tocar `Default for CacheWarmer` (línea 290) — sigue llamando a `Self::new()`.
- **Lecciones aprendidas:** al aplanar cadenas constructoras, los niveles internos quedan test-only si los tests son el único caller — usar `#[cfg_attr(not(test), allow(dead_code))]` evita warnings sin ensuciar el código de producción.
- **Resultado verify (2026-08-29T21:05):**
  - Contract 1: `(Select-String -Path 'src\cache_warmer.rs' -Pattern 'with_config_and_cap' | Measure-Object).Count` = **5** ✅ (>=1)
  - Contract 2: `cargo check -p vantadb` exit code = **0** ✅
  - `cargo check -p vantadb --tests` exit code = **0** ✅
  - `cargo test -p vantadb --lib cache_warmer` → 11/11 passed ✅

## Context Save Point
- **Fecha:** 2026-08-29T21:00
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** flatten via duplicación (8 líneas) sobre abstracción helper (Ponytail: ladder rung 6 — código más corto)
- **Problemas conocidos:** ninguno
- **Próxima tarea:** FIND-43 completa → siguiente task