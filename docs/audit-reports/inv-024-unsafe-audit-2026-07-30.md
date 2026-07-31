# INV-024 — Unsafe Audit Report (Memory Safety + Supply Chain)

**Fecha:** 2026-07-30
**Alcance:** Todos los bloques `unsafe` del core Rust (11 archivos, 39 bloques)
**Auditor:** vanta-audit
**Estado:** COMPLETADO — 2 pendientes cerrados (reachability sq8 + cargo audit/deny)

---

## Resumen ejecutivo

| Métrica | Valor |
|---|---|
| Bloques `unsafe` auditados | **39** |
| SAFE (invariantes completos, verificados) | **28** |
| SAFE_BUT_UNDOCUMENTED (seguro, comentario incompleto) | **4** |
| UB_POTENTIAL (requiere archivo corrupto/adulterado) | **7** |
| Hallazgos High | **1** (panic-DoS reachable desde API pública) |
| Hallazgos Medium | **1** (UB por alineación en 7 sitios, release) |
| Hallazgos Low | **3** |
| cargo deny check | **PASSED** (advisories/bans/licenses/sources ok; 10 duplicates warn) |
| cargo audit | **0 critical / 0 high / 0 medium / 1 warning permitido** (RUSTSEC-2026-0002) |

**Veredicto global:** El core es sólido. Los 14 bloques `chunks_exact + unwrap_unchecked` de los kernels SIMD son correctos. La clase de bug dominante es el cast `mmap_bytes() + vector_offset → *const f32` que **depende de alineación 4B garantizada por el escritor pero nunca validada en runtime** contra datos de archivo no confiables (7 sitios, UB potencial en release). El único hallazgo explotable desde API pública sin archivo corrupto es el **panic-DoS en `sq8_similarity`** (dimensiones dispares).

---

## Tabla completa de bloques unsafe

| # | Archivo | Línea | Patrón | Clasificación | SAFETY comment |
|---|---|---|---|---|---|
| 1-14 | `src/index/distance.rs` | 98,102,132,136,209,213,245,249,277,281,308,312,391,432 | `chunks_exact(8/16)` + `unwrap_unchecked` (f32x8/f32x16) | **SAFE** | ✅ Completo (invariante estructural) |
| 15 | `src/index/distance.rs` | 502 | `from_raw_parts(mmap.as_ptr() as *const f32, len)` (MmapFull) | **SAFE_BUT_UNDOCUMENTED** | ⚠️ Solo "len bounded; mmap kept alive" |
| 16 | `src/node.rs` | 317 | `from_raw_parts` (MmapFull `as_f32_slice`) | **SAFE_BUT_UNDOCUMENTED** | ⚠️ No menciona alineación (correcta: mmap page-aligned) |
| 17 | `src/index/ivf.rs` | 69 | `from_raw_parts` (MmapFull → to_vec) | **SAFE_BUT_UNDOCUMENTED** | ⚠️ Idéntico patrón MmapFull |
| 18 | `src/index/serialize.rs` | 141 | `from_raw_parts` (MmapFull serialize) | **SAFE_BUT_UNDOCUMENTED** | ⚠️ Solo "len bounded" |
| 19 | `src/index/serialize.rs` | 603 | `MmapMut::map_mut` (load_from_file) | **SAFE** | ✅ file_len ≥ 64 verificado (anti-SIGBUS) |
| 20 | `src/index/serialize.rs` | 673 | `MmapMut::map_mut` (sync_to_mmap, temp) | **SAFE** | ✅ set_len(data.len()) previo |
| 21 | `src/index/serialize.rs` | 688 | `MmapMut::map_mut` (sync_to_mmap, post-rename) | **SAFE** | ⚠️ Sin comment en 688 (heredado de 671) |
| 22 | `src/index/search.rs` | 146 | `from_raw_parts` (vstore, entry point) | **UB_POTENTIAL** | ⚠️ Bounds OK; alineación solo `debug_assert` |
| 23 | `src/index/search.rs` | 287 | `from_raw_parts` (vstore, neighbor) | **UB_POTENTIAL** | ⚠️ Ídem |
| 24 | `src/storage/archive.rs` | 74 | `MmapMut::map_mut` (shadow compact) | **SAFE** | ✅ set_len previo |
| 25 | `src/storage/archive.rs` | 105 | `MmapMut::map_mut` (extend+remap) | **SAFE** | ✅ set_len(end+4096) previo; mmap anterior dropeado |
| 26 | `src/storage/archive.rs` | 237 | `from_raw_parts` (rebuild scan) | **UB_POTENTIAL** | ⚠️ Bounds OK; alineación solo `debug_assert` |
| 27 | `src/storage/engine/ops.rs` | 509 | `from_raw_parts` (get_node) | **UB_POTENTIAL** | ❌ **Sin SAFETY comment ni debug_assert** |
| 28 | `src/storage/engine/ops.rs` | 1225 | `from_raw_parts` (get with index node) | **UB_POTENTIAL** | ⚠️ Comment presente; alineación solo `debug_assert` |
| 29 | `src/storage/engine/ops.rs` | 1402 | `from_raw_parts` (scan) | **UB_POTENTIAL** | ⚠️ Ídem |
| 30 | `src/storage/engine/ops.rs` | 1804 | `from_raw_parts` (snapshot scan) | **UB_POTENTIAL** | ⚠️ Ídem |
| 31 | `src/metrics/core/mod.rs` | 301 | Mach `task_info` (macOS) | **SAFE** | ✅ Zeroed POD + return check |
| 32 | `src/metrics/core/mod.rs` | 326 | `GetProcessMemoryInfo` (Windows) | **SAFE** | ✅ Zeroed POD + size exacto + return check |
| 33 | `src/storage/vfile.rs` | 71 | shim `Mmap::map` (API parity) | **SAFE** | ✅ Implementación segura |
| 34 | `src/storage/vfile.rs` | 110 | shim `MmapMut::map_mut` (API parity) | **SAFE** | ✅ Implementación segura |
| 35 | `src/storage/vfile.rs` | 493 | `MmapOptions::map` (open) | **SAFE** | ✅ (validación de size arriba) |
| 36 | `src/storage/vfile.rs` | 497 | `MmapOptions::map` (open, rw) | **SAFE** | ✅ Ídem |
| 37 | `src/storage/vfile.rs` | 567 | `MmapOptions::map_mut` (remap_mut) | **SAFE** | ✅ File handle válido + drop del anterior |
| 38 | `src/index/graph.rs` | 209 | `Mmap::map` (cold-start) | **SAFE** | ✅ `file` válido; crate valida internamente |
| 39 | `src/storage/engine/maintenance.rs` | 159 | `MmapMut::map_mut` (compact) | **SAFE** | ✅ Tamaño verificado |

---

## Detalle por archivo

### 1. `src/index/distance.rs` — 15 bloques (14 SAFE + 1 SAFE_BUT_UNDOCUMENTED)

Los 14 kernels SIMD (`f32_dot_and_norm_b_sq_f32x8/16`, `f32_dot_product_f32x8/16`, `euclidean_distance_sq_f32x8/16`, `sq8_similarity` chunks) usan:

```rust
// SAFETY: chunks_exact(8) guarantees chunk.len() == 8
*unsafe { <&[f32; 8]>::try_from(a_chunk).unwrap_unchecked() }
```

**SAFE.** Invariante estructural: `chunks_exact(N)` solo produce chunks de exactamente `N` elementos; `try_from` no puede fallar; `unwrap_unchecked` es válido. `wide::f32x8/16::from([f32; N])` copia por valor (sin problema de alineación de origen). Remainders manejados escalarmente.

`calculate_similarity` L502 (MmapFull): **SAFE_BUT_UNDOCUMENTED** — `len = mmap.len()/4` acotado por `MAX_VEC_F32_LEN`, `Arc<Mmap>` mantiene el mapeo vivo, base page-aligned (alineación 4B garantizada estructuralmente). El comment solo menciona 2 de 4 invariantes.

### 2. `src/node.rs` — 1 bloque (SAFE_BUT_UNDOCUMENTED)

L317 `as_f32_slice`: patrón MmapFull idéntico al anterior. **Seguro** (mismo análisis: len acotado, Arc alive, page-aligned). Comment mejorable: no menciona alineación ni lifetime del Arc (solo está en la doc del enum).

### 3. `src/index/ivf.rs` — 1 bloque (SAFE_BUT_UNDOCUMENTED)

L69 `node_to_f32_slice`: mismo patrón MmapFull con `.to_vec()` inmediato (elimina cualquier aliasing). **Seguro.**

### 4. `src/index/serialize.rs` — 4 bloques (1 SAFE_BUT_UNDOCUMENTED + 3 SAFE)

- L141: `from_raw_parts` sobre MmapFull al serializar. **Seguro** (mismo patrón). ⚠️ El comment dice "len bounded by MAX_VEC_F32_LEN" — correcto pero incompleto.
- L603: `map_mut` en `load_from_file` — **SAFE**: `file_len >= 64` verificado antes (evita SIGBUS en header reads); los accesos posteriores van vía `take_bytes` con bounds checks.
- L673/L688: `map_mut` en `sync_to_mmap` — **SAFE**: `set_len(data.len())` previo; flow atomic-rename. El remap L688 no tiene comment propio (hereda contexto del bloque 671), menor.
- **Hallazgo de diseño (nota, no fallo):** `deserialize_from_bytes(data, _force_copy)` tiene el parámetro `_force_copy` **muerto** y **nunca crea `MmapFull`** — siempre copia a `Full`. El log "loaded zero-copy index from file" (L613) es engañoso; la ruta cold-start copia todo al heap. No es inseguro; delegar a tuner la decisión de habilitar zero-copy real.

### 5. `src/index/search.rs` — 2 bloques (UB_POTENTIAL)

L146 y L287: `from_raw_parts(vec_data.as_ptr() as *const f32, header.vector_len)` sobre `vs.mmap_bytes()[vec_start..vec_end]`.

- **Bounds:** ✅ correctos (`vec_end > vs.mmap_bytes().len()` → 0.0).
- **Alineación:** ⚠️ solo `debug_assert_eq!(align_offset(4), 0)`. En release, si `header.vector_offset` (dato del archivo, no validado por `read_header` — solo valida el offset del header, no el campo vector_offset) no es múltiplo de 4, el cast crea un `&[f32]` desalineado → **UB** (en ARM puede fault; en x86 "funciona" pero sigue siendo UB).
- **Lifetime:** ✅ el borrow de `mmap_bytes()` mantiene el slice vivo (borrow checker).
- **Fix:** runtime check en vez de debug_assert, o validar `vector_offset.is_multiple_of(4)` en `read_header` (fix central, ver Medium #1).

### 6. `src/storage/archive.rs` — 3 bloques (2 SAFE + 1 UB_POTENTIAL)

- L74/L105: `map_mut` en shadow compaction — **SAFE**: `set_len(new_file_size)` previo; remap tras `set_len(end+4096)`; mmap anterior dropeado antes de re-mapear (sin conflicto de mappings).
- L237: `from_raw_parts` en rebuild scan — **UB_POTENTIAL**: bounds OK (`end <= vstore.size`), `.to_vec()` inmediato (sin aliasing), pero alineación solo `debug_assert`. Mismo fix que search.rs.
- Nota: L116 `copy_from_slice` usa `src_end.min(old_data.len())` — protege de OOB, puede truncar datos en archivo corrupto (integridad, no seguridad).

### 7. `src/storage/engine/ops.rs` — 4 bloques (UB_POTENTIAL ×4)

- L509 (`get_node`): **el peor del set** — **sin SAFETY comment y sin debug_assert**. Bounds check presente. Mismo riesgo de alineación. Blocker para el estándar del repo (regla: unsafe sin SAFETY = no pasa review).
- L1225/L1402/L1804: comment presente + debug_assert, pero la alineación **no se verifica en runtime** → UB_POTENTIAL en release con archivo adulterado.
- Los comments afirman "page-aligned via mmap, guaranteeing f32 alignment" — **razonamiento inválido**: el mmap base está page-aligned, pero `vec_start = header.vector_offset` viene del header del archivo. Un archivo con `vector_offset=2` produce un slice desalineado aunque el mmap base esté alineado.

### 8-11. `metrics/core/mod.rs`, `vfile.rs`, `graph.rs`, `maintenance.rs` — 8 bloques (SAFE)

FFI de métricas (Mach/Windows): zeroed POD, tamaño exacto, return codes verificados. Mappings: sizes validados previos, handles válidos, shims con implementación segura. Sin hallazgos.

---

## Hallazgos priorizados

### 🔴 High — H-1: Panic-DoS en `sq8_similarity` (distance.rs:411, 450) — REACHABLE desde API pública

```rust
for i in 0..rem_q.len() {
    let decoded = (rem_s[i] as f32) * inv_scale;   // ← panic si rem_s.len() < rem_q.len()
    ...
    let diff = rem_q[i] - (rem_s[i] as f32) * inv_scale;  // ← idem, arm Euclidean
}
```

**Reachability verificada (pendiente #1):** `VantaEmbedded::search_vector` (sdk/api.rs:921) → `hnsw.search_nearest` (search.rs:474) → `flat_search` / `search_layer` → `calculate_similarity` (distance.rs:459) → arm `SQ8` (L483) → `sq8_similarity`. **No existe guard de dimensiones en ninguna capa**: `search_vector` solo chequea `is_empty()`; `search_nearest` no valida dims; `insert` no valida dims de nodos SQ8 (ops.rs:542 no chequea dimensiones). El zip truncante (`chunks_q.zip(chunks_s)`) + remainder loops indexan `rem_s[i]` con `i < rem_q.len()`.

**Trigger (proof of concept):**
```rust
// 1. Insertar nodo con vector de 8 dims (se cuantiza a SQ8 de 8 elementos)
// 2. Query con 9 dims: chunks_q = 1 (8), chunks_s = 1 (8),
//    rem_q = [1], rem_s = [] → rem_s[0] → index out of bounds panic
engine.search_vector(&vec![0.0; 9], 10);  // panic en el hot path
```

**Impacto:** panic en hilo de búsqueda (DoS en server/embedded). No es UB, pero es crash alcanzable con un solo request malformado.

**Fix (guard central, como hacen `cosine_sim_*`):**
```rust
fn sq8_similarity(...) -> f32 {
    if raw_query.len() != sq8_data.len() || raw_query.is_empty() { return 0.0; }
    ...
}
```

### 🟠 Medium — M-1: UB por alineación en cast `vector_offset → *const f32` (7 sitios)

`src/index/search.rs:146,287` · `src/storage/archive.rs:237` · `src/storage/engine/ops.rs:509,1225,1402,1804`

`DiskNodeHeader::vector_offset` (dato del archivo) nunca se valida como múltiplo de 4 en `read_header` (solo valida el offset del header contra `STORAGE_ALIGNMENT`). El writer (`write_node_to_vstore`: `vector_offset = offset + 64`, con `offset` múltiplo de 64) garantiza alineación — pero un archivo `.vanta` corrupto o adulterado produce un `&[f32]` desalineado → **UB en release** (los `debug_assert` no compilan en release; ops.rs:509 no tiene ni debug_assert).

**Fix central (1 cambio, cubre 7 sitios):** validar en `read_header`:
```rust
if !header.vector_offset.is_multiple_of(4) { return None; }
```
O runtime check en cada sitio (patrón ya usado para bounds):
```rust
if vec_bytes.as_ptr().align_offset(4) != 0 { /* skip / 0.0 / Ok(None) */ }
```

### 🟡 Low — L-1: RUSTSEC-2026-0002 — lru 0.12.5 unsound (via ratatui 0.28.1)

`IterMut` de `lru` viola Stacked Borrows (invalida puntero interno). Solo afecta la TUI (ratatui usa lru para buffers de render). `cargo audit` lo reporta como **allowed warning** (no bloquea). Mitigación: no hay runtime DB impactado; monitorear release de ratatui que suba lru. Issue tracking recomendado (owner: vanta-lead).

### 🟡 Low — L-2: MmapFull contenido sin validar (NaN/inf silencioso)

Los 4 sitios MmapFull (`node.rs:317`, `distance.rs:502`, `ivf.rs:69`, `serialize.rs:141`) interpretan bytes del archivo como f32 sin validación. Cualquier bit pattern es f32 *válido en memoria* (no UB), pero archivo truncado/tornado produce NaN/inf en la similitud — resultados incorrectos silenciosos, no memory corruption. Aceptable para índice; considerar checksum/version header al mapear.

### 🟡 Low — L-3: `deserialize_from_bytes` copia todo; "zero-copy" es engañoso

El parámetro `_force_copy` está muerto; la ruta cold-start mmap copia todos los vectores a `Full`. No es inseguro; es un problema de expectativa/performance. Delegado a tuner.

### 🟡 Low — L-4: Comments MmapFull incompletos (4 sitios)

`node.rs:317`, `distance.rs:502`, `ivf.rs:69`, `serialize.rs:141`: los SAFETY comments cubren solo 2 de 4 invariantes (len, lifetime) pero no alineación (estructuralmente correcta por page-alignment) ni validez del contenido. Propuesta de comment estándar:

```rust
// SAFETY: 1) len = mmap.len()/4 y está acotado por MAX_VEC_F32_LEN;
// 2) el Arc<Mmap> mantiene el mapeo vivo durante el borrow;
// 3) la base del mmap está page-aligned (4KB) → alineación f32 garantizada;
// 4) cualquier bit pattern es f32 válido en memoria (NaN posible, no UB).
```

---

## Pendientes cerrados

1. **Reachability sq8_similarity:** ✅ VERIFICADO — alcanzable desde `search_vector` (API pública) sin guard de dimensiones. Clasificado **High**.
2. **cargo audit / cargo deny:** ✅ EJECUTADO
   - `cargo deny check` (0.19.9): **PASSED** — advisories ok, bans ok, licenses ok, sources ok. 10 `warning[duplicate]` (r-efi, rand, rand_core, syn, thiserror, windows-sys, etc.) no bloqueantes.
   - `cargo audit` (0.22.1): **0 critical / 0 high / 0 medium**, 1 allowed warning (RUSTSEC-2026-0002, ver L-1). Ignora RUSTSEC-2023-0089 y RUSTSEC-2024-0436 (configurados con owner/expiry en `.cargo/audit.toml` + `deny.toml`).

---

## Recomendaciones

1. **Fix H-1 primero** (una línea, cubre 2 arms): guard de dims en `sq8_similarity`.
2. **Fix M-1 central** en `read_header`: validar `vector_offset.is_multiple_of(4)` (cubre 7 sitios, incluye ops.rs:509 que además necesita SAFETY comment).
3. **Ops.rs:509**: añadir SAFETY comment obligatorio (regla repo).
4. **Miri** (`MIRIFLAGS=-Zmiri-tree-borrows`) sobre los tests existentes: `miri_distance_kernels` (distance.rs:1449), `miri_search_layer` (search.rs:1172) — cubren 14 unsafe; añadir caso Miri para dims dispares en SQ8 (reproduce H-1 bajo el checker).
5. Registrar RUSTSEC-2026-0002 como issue de tracking (vanta-lead).
