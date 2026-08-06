# Unified Review — certify — 2026-08-05

**Profile:** vantadb
**Mode:** certify
**Duration:** ~16 min
**Quality Gate:** ❌ FAIL — ABORTED at L1 (critical)
**Ponytail mode:** full

## Executive Summary

La certificación del gate pre-push (modo `certify`, profile `vantadb`) se
**detuvo en la fase crítica L1 (Rust Workspace Check) por fallo ambiental de
disco, no por defecto de código**.

Los gates mecánicos que dependen únicamente del código **pasaron**: `cargo fmt`
limpio, `cargo check --workspace --tests` OK, `cargo clippy --workspace --tests
-D warnings` con **0 warnings**, `cargo deny check` OK (advisories/bans/
licencias/sources), `cargo machete` sin dependencias sin uso. El único punto de
fallo fue `cargo nextest run --profile audit` (H01-CODE-001/002): 2 tests del
módulo `security.rs` abortaron con `IoError: StorageFull, Os 112` — el disco C:
está completamente lleno (474 GB usados, **1.70 GB libres**). Los tests de
seguridad que abren bases de datos temporales no tienen espacio para escribirlas.

El gate crítico exige que *todos* los checks mecánicos pasen; un fallo de
`nextest` (aunque la causa raíz sea disco, no lógica) lo abrirá como **FAIL** y
aborta el resto del pipeline (L2-L6 no se ejecutaron).

L1 reportó además un candidate finding de código (H01-CODE-003), pero al
verificarlo en contexto resultó ser **falso positivo en dos aspectos**:

1. Archivo equivocado: el `unwrap()` apuntado en `cache_warmer.rs` está
   realmente en `src/index/core.rs:191`.
2. Es código de **test** (`#[test] fn concurrent_insert_preserves_hnsw_invariants`,
   dentro de `#[cfg(test)]`) — líneas 178, 191 y 196 del mismo bloque. En tests,
   `unwrap()`/`expect()` son idiomáticos; el ban de la convención aplica a
   producción. **No hay defecto de código que corregir.**

**Qué hacer:** liberar espacio en disco (dashboard de IS, `target/`, temp) y
re-ejecutar `cargo nextest run --profile audit --workspace --build-jobs 2`, y
re-correr el gate. No hay evidencia de fallo de lógica; los gates de código
están verdes.

## Scoreboard

| Phase | Status | Score | Findings (C/H/M/L/I) | Duration |
|-------|--------|-------|----------------------|----------|
| L0 Diff Impact | ✅ | — | 0/0/0/0/0 | 2s |
| L1 Core Language | ❌ | 5/10 | 0/0/2/0/0 | ~15 min |
| L2 Bindings | ⏭️ abort | — | — | — |
| L3 Web | ⏭️ abort | — | — | — |
| L4 CI/CD | ⏭️ abort | — | — | — |
| L5 Docs | ⏭️ abort | — | — | — |
| L6 Architecture | ⏭️ abort | — | — | — |
| **OVERALL** | ❌ | **N/A** | **0/0/2/0/0** | 16 min |

_(No se computa score global: el gate aborta en la primera critical fail.)_

## Findings

### H01-CODE-001 (high) — `tests/security.rs:23`
- **Categoría:** TEST / ERROR (StorageFull)
- `test_iql_invalid_insert_node_id_overflow` panicked: `open: IoError(Storage, StorageFull, Os 112)`. El open de la DB temporal falló por **disco lleno**.
- **Recomendación:** liberar espacio (C: libre = 1.70 GB) y re-correr. No es defecto de lógica.

### H01-CODE-002 (high) — `tests/security.rs:598`
- **Categoría:** TEST / ERROR (StorageFull)
- `test_rapid_open_close_cycle_no_crash` falló igual (I/O StorageFull) — ciclo fuzz de open/close escribiendo a disco sin headroom.
- **Recomendación:** misma causa raíz (disco). Confirmar headroom en temp antes de re-certificar.

### H01-CODE-003 (low) — `src/cache_warmer.rs`
- **Categoría:** CODE
- `storage.flush_pending_hnsw().unwrap()` — un `unwrap()` nuevo en el diff del working tree. No es sobre input de usuario (handle interno de DB), pero viola la convención del proyecto (Regla 4: ban de unwrap/expect en código nuevo).
- **Recomendación:** propagar el error vía `Result` (anyhow) en lugar de `unwrap()`.

### H01-CODE-004 (info) — tooling gap
- `cargo-audit` no está instalado / `cargo audited` no existe → el advisory scan no se ejecutó. `cargo deny` (advisories) sí pasó. No cuenta como gate.

### H01-CODE-005 (info) — tooling gap
- `cargo semver-checks` únicamente no está instalado y el intento de `check` no devolvió output (timeout 600s). El diff no introduce `unsafe` ni nuevas deps ni bumps en Cargo.toml, así que el riesgo de breaking change en API es bajo.

## Commands (rel. al fallo)

```
cargo nextest run --profile audit --workspace --build-jobs 2   # FAIL: 2 tests StorageFull
cargo fmt --all -- --check                                      # PASS
cargo check --workspace --tests -j 2                            # PASS
cargo clippy --workspace --tests -j 2 -- -D warnings            # PASS (0 warnings)
cargo deny check                                                # PASS
cargo machete                                                   # PASS
```

## Recomendaciones (priorizadas)

1. **(blocker, antes de re-certify)** Liberar espacio en disco: mín. 2-3 GB libres en C:. Objetivo: `cargo clean` controlado en `target/` o limpiar temp del OS si no se necesita el cache de build.
2. **(high, this iteration)** Revertir/refactorizar el `unwrap()` en `src/cache_warmer.rs` → `?` + propagación de error.
3. **(medium, backlog)** Instalar `cargo-audit` y `cargo-semver-checks` en el toolchain para que los gates suplementarios corran en futuras certificaciones.
4. **(re-run)** Re-ejecutar `skill unified-review --mode certify --profile vantadb` tras liberar disco.

---
_Generated by unified-review skill. Profile: vantadb. Mode: certify._