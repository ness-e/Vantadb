# REVIEW-14 — Panics frágiles ante store corrupto (version_history) + unwraps frágiles (explain.rs)

> **Estado:** ⏳ IN PROGRESS · **Appetite:** max 1h · 🟢 · Prioridad 🟢
> **Fuente:** review-full-20260822 H09-CODE-005 · Plan: `docs/plans/2026-08-23-backlog-triage.md` Task 11

## DISCOVERY (hecho, no asumido)

### Site 1 — `src/sdk/version_history.rs:283` ⚠️ MOVIDO/RECLASIFICADO

- El patrón `k[k.len()-8..].try_into().unwrap()` existe SOLO dentro de `#[cfg(test)] mod tests`
  (`fn version_key_roundtrips`). Grep workspace `len()\s*-\s*8|from_be_bytes` en `src/` = 1 match (ese).
- **No hay código de producción que parsee keys del partition `Versions`**: todos los paths
  (`purge_key`/`get_version`/`versions`/`evict_overflow`) construyen keys con `version_key()`/
  `version_prefix()` y decodifican VALUES (snapshots postcard), nunca keys.
- El claim "panic ante store corrupto" era falso para producción hoy; el riesgo real es el patrón
  frágil copiable + falta de validación al leer keys escaneadas.

### Variante de error (pregunta explícita del contrato: Corrupt vs InvalidInput)

- `VantaError` NO tiene variante `Corrupt` (enum 30 variantes, `#[non_exhaustive]`, src/error.rs:91-266).
- Añadir `Corrupt` tocaría la taxonomía pública y arriesga matches exhaustivos en bindings (deuda P2-6,
  `vantadb-python/src/types.rs:365`) → fuera de scope.
- Las keys del partition SOLO se producen vía `version_key()` desde ns/key ya validados — el input de
  usuario jamás llega como raw key ⇒ key corta = **corrupción de store**, NO input inválido.
- Clase corrupción existente en el código: `BackendError` (ver error.rs:578 test "rocksdb corruption").
  **Decisión: `VantaError::backend_error(...)`.** Desviación documentada respecto al texto literal del
  contrato ("Corrupt") — la esencia mecánica (error tipado de corrupción, sin panic) se cumple.

### Site 2 — `src/sdk/search/explain.rs:103/147/183` ✅ CONFIRMADO EN DISCO

- Los 3 unwraps son `request.query_sparse.as_ref().unwrap()` bajo arms donde `has_sparse==true`
  garantiza `Some` no-vacío HOY — pero frágiles: cualquier cambio de flags/mode nuevo → panic.
- Fix por contrato: bindear `Some` en el pattern del match. Semántica preservada incluyendo
  `SearchProfileMode::Keyword` que FUERZA sparse off (nullifica la Option local, no solo el bool).
- **Colateral (fuera de scope → FIND):** mismo patrón frágil en `src/sdk/search/mod.rs`
  (:207/:240/:265/:315/:346/:369) y `src/sdk/search/debug_ops.rs` (:288/:335/:374). Fila FIND en Backlog.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `src/sdk/version_history.rs` (503L), `src/sdk/search/explain.rs` (215L),
  `src/error.rs` (variantes + constructores, L1-459), grep de `BackendPartition::Versions` (12 hits, solo
  version_history.rs los parseea… ninguno), grep `query_sparse` (tipos confirmados: `Option<SparseVector>`).
- **Referencias hacia dentro:** `version_history.rs` usa `StorageEngine::{put_to_partition,
  scan_partition_prefix, get_from_partition, write_backend_batch}`; `explain.rs` usa
  `sparse_memory_search(namespace, &SparseVector, filters, budget)` (firma en `hybrid.rs:15`).
- **Referencias entrantes:** `versions()` expuesto vía `VantaEmbedded::versions` (API pública SDK — solo
  cambia comportamiento ante keys truncadas, imposible en stores sanos). `explain_memory_search` público,
  comportamiento idéntico (reestructuración equivalente verificada arm-por-arm).
- **Veredicto:** blast radius acotado a 2 archivos + tests inline. Sin cambios de API pública, sin hot
  path (scans de history son raros, cap 32), sin FFI. Regla 8 no aplica (sin locks/tokio/dashmap).

## Contrato mecánico

1. Test nuevo: key <8 bytes en el path de version_history devuelve error de corrupción tipado (sin panic).
2. `rg -n "\.unwrap\(\)" src/sdk/search/explain.rs` = 0 matches.
3. Suite `-p vantadb` verde + fmt/clippy -D warnings.

## Steps

- [x] Step 1: `version_from_key()` helper (`Result<u64>`, BackendError si <8B) + validación en `versions()`
      + test nuevo corto→Corrupción + test existente :283 usa el helper (sin unwrap frágil)
- [x] Step 2: explain.rs — bind `Some(query_sparse)` en patterns, eliminar 3 unwraps, grep 0
- [x] Step 3: Verify full (fmt + clippy -D warnings + nextest -p vantadb) + commit + FIND row Backlog

## Evidencia (2026-08-23)

- `cargo fmt --check` → exit 0
- `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` → Finished sin warnings
- `cargo nextest run -p vantadb` → **2051/2051 passed** (1 skipped pre-existente; baseline 2050 +1 test nuevo)
- Test nuevo explícito: `sdk::version_history::tests::short_versions_partition_key_returns_corruption_error_not_panic` PASS
- `rg "\.unwrap\(\)" src/sdk/search/explain.rs` → 0 matches
- FIND-27 registrada en `docs/Backlog.md` (unwraps hermanos mod.rs/debug_ops.rs)

## Context Save Point

- Decisión de variante: `BackendError`, motivo arriba. Si el owner quiere `Corrupt` real → tarea aparte
  de taxonomía de errores (impacta bindings P2-6).
- Sibling sites (mod.rs/debug_ops.rs) van a fila FIND — NO arreglar aquí.
