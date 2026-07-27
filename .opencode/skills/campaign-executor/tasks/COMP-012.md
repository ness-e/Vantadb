# COMP-012: RoaringBitmaps for Metadata Indexing

**ID:** COMP-012
**Prioridad:** 🟡 Media-Alta
**Esfuerzo:** ~1 semana
**Estado:** ❌ No implementado
**Dependencias:** Pre-COMP-003
**Sub-agente:** `vanta-worker`

## 📋 Descripción

Reemplazar el `FilterBitset` custom (backed by `Vec<u64>`) con `croaring::Bitmap` (RoaringBitmap). Esto mejora compresión de bitsets sparse, acelera operaciones de intersección vía SIMD, y permite >128 bits.

**No cambiar** el formato on-disk `DiskNodeHeader` (sigue almacenando `bitset: u128`). La conversión `to_u128()`/`from_u128()` se mantiene para compatibilidad binaria.

## 🔍 Contexto Actual

### `FilterBitset` (src/node.rs:18)
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FilterBitset(Vec<u64>);
```
Es un `Vec<u64>` custom con: `set_bit`, `has_bit`, `matches_mask`, `is_empty`, `is_all_set`, `all_set`, `to_u128`, `from_u128`, `to_bytes`, `from_bytes`, `word_count`.

### `DiskNodeHeader` (src/node.rs:735)
```rust
#[repr(C, align(64))]
pub struct DiskNodeHeader {
    pub id: u128,
    pub bitset: u128,   // ← 128-bit limit en disco
    ...
}
```
El formato on-disk está limitado a 128 bits. **No cambiar.** `to_u128()`/`from_u128()` truncan a 128 bits para serialización a disco.

### Uso en Hot Path
- `search_layer()` en `src/index/search.rs` — `matches_mask` se llama por nodo candidato durante HNSW traversal
- `flat_search()` en `src/index/flat.rs`
- `ivf_search()` en `src/index/ivf.rs` — `matches_mask` en scan
- `engine.vector_search()` y `engine.hybrid_search()` en `src/engine.rs`
- `engine.scan_bitset()` en `src/engine.rs`

### Tests existentes (27)
- `src/node.rs`: 17 tests de FilterBitset (test_filter_bitset_*, test_bitset_operations)
- `src/engine.rs`: 7 tests (test_scan_bitset_*, test_vector_search_with_bitset_filter, test_filter_field_*)
- `src/index/ivf.rs`: test_ivf_search_with_bitset_filter

## 🎯 Plan de Implementación

### Paso 1: Agregar dependencia croaring

En `Cargo.toml`:
```toml
croaring = "2.6"
```

No feature-gate — es una dependencia central (reemplaza el Vec<u64> usado siempre).

### Paso 2: Reemplazar FilterBitset inner type

**Mantener la struct `FilterBitset` (nombre público, Serialize/Deserialize, etc.)** pero cambiar inner type:

```rust
use croaring::Bitmap;  // o croaring::Treemap si necesitamos >32 bits

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FilterBitset(/* inner type aquí */);
```

**Mapa de métodos:**

| Método actual | Implementación con croaring |
|---|---|
| `new()` | `Self(Bitmap::new())` |
| `with_capacity(bits)` | `Self(Bitmap::with_container_capacity(bits.div_ceil(1<<16) as u32))` |
| `all_set()` | Mantener sentinel — `Self(Bitmap::from_range(0..=u32::MAX))` es caro. Mejor flag. |
| `is_all_set()` | Mantener check de sentinel |
| `set_bit(pos)` | `self.0.add(pos as u32)` |
| `has_bit(pos)` | `self.0.contains(pos as u32)` |
| `matches_mask(mask)` | `mask.0.is_subset(&self.0)` — todos los bits del mask están en self |
| `is_empty()` | `self.0.is_empty()` |
| `word_count()` | Deprecar o retornar 0 (no aplica a Roaring). Marcar `#[allow(dead_code)]` |
| `to_u128()` | Convertir primeros 128 bits a u128 |
| `from_u128(v)` | Crear Bitmap desde u128 |
| `to_bytes()` | `self.0.serialize::<Portable>()` |
| `from_bytes(data)` | `Bitmap::deserialize::<Portable>(data).map(|b| (Self(b), data.len()))` |

### Paso 3: Mantener el sentinel `ALL_BITSET`

```rust
pub static ALL_BITSET: LazyLock<FilterBitset> = LazyLock::new(FilterBitset::all_set);
```

El `all_set()` actual crea `vec![u64::MAX]`. Con croaring, podemos:
- Opción A: Mantener sentinel con un flag `is_all_set: bool` en FilterBitset
- Opción B: Crear Bitmap con `u32::MAX` elementos

Opción A es más eficiente. Pero `matches_mask` se basa en que `is_all_set` retorne true rápido. Podemos:

```rust
pub fn all_set() -> Self {
    Self(Bitmap::from_range(0..=u32::MAX))  // todos los bits
}

pub fn is_all_set(&self) -> bool {
    self.0.cardinality() == (u32::MAX as u64 + 1)
    // o: self.0.contains_range(0..=u32::MAX)
}
```

Pero esto es caro. Mejor **mantener el sentinel** con un flag interno:

```rust
pub struct FilterBitset {
    inner: Bitmap,
    all_set: bool,  // true → match everything
}
```

Y `matches_mask` chequea `self.all_set || mask.inner.is_subset(&self.inner)`.

### Paso 4: Actualizar to_u128/from_u128

La conversión a u128 actual lee los primeros 2 u64 words. Con croaring:

```rust
pub fn to_u128(&self) -> u128 {
    let mut iter = self.0.iter();
    let lo = iter.next().unwrap_or(0) as u128;
    let hi = if iter.len() > 0 { /* next value */ } else { 0 };
    lo | (hi << 64)
}
```

No, esto no es correcto — to_u128 no es la lista de valores, es la representación de bits. Necesitamos reconstruct los primeros 128 bits:

```rust
pub fn to_u128(&self) -> u128 {
    let mut result: u128 = 0;
    for i in 0..128 {
        if self.0.contains(i as u32) {
            result |= 1u128 << i;
        }
    }
    result
}
```

Y `from_u128`:
```rust
pub fn from_u128(v: u128) -> Self {
    let mut bm = Bitmap::new();
    for i in 0..128 {
        if (v & (1u128 << i)) != 0 {
            bm.add(i as u32);
        }
    }
    Self { inner: bm, all_set: false }
}
```

### Paso 5: Actualizar tests

Los tests existentes deben seguir pasando. Los que verifican `word_count()` pueden necesitar actualización (Roaring no expone word count de la misma forma).

### Paso 6: Compactar con run_optimize

Opcional: llamar `self.0.run_optimize()` después de operaciones bulk para minimizar el tamaño en memoria.

## 📁 Archivos a Modificar

1. **`Cargo.toml`** — Agregar `croaring = "2.6"`
2. **`src/node.rs`** — Reemplazar FilterBitset, actualizar métodos, manterer DiskNodeHeader intacto
3. **`src/index/search.rs`** — Usa `matches_mask` y `is_all_set` — no debería requerir cambios (API igual)
4. **`src/index/flat.rs`** — Ídem
5. **`src/index/ivf.rs`** — Ídem
6. **`src/engine.rs`** — Ídem
7. **`src/storage/ops.rs`** — Usa `to_u128()` / `from_u128()` — compatible
8. **`src/storage/archive.rs`** — Usa `from_u128()` — compatible

## ✅ Criterios de Aceptación

- [ ] `cargo check -p vantadb` pasa sin errores
- [ ] `cargo test -p vantadb` — todos los tests de FilterBitset pasan (17 en src/node.rs)
- [ ] `cargo test -p vantadb` — tests de engine que usan bitset pasan (7 en src/engine.rs)
- [ ] `cargo test -p vantadb` — test_ivf_search_with_bitset_filter pasa
- [ ] `cargo clippy -p vantadb` sin nuevos warnings
- [ ] No se cambia DiskNodeHeader ni formato on-disk
- [ ] to_u128()/from_u128() roundtrip preserva datos para ≤128 bits
- [ ] matches_mask() funciona correctamente con sentinel ALL_BITSET

## ⚠️ Riesgos

1. **croaring requiere C/C++ toolchain** — CRoaring se compila desde C. En Windows requiere MSVC. Verificar que `cargo build` funciona.
2. **Serialize/Deserialize** — FilterBitset actual deriva `Serialize, Deserialize` de serde. croaring::Bitmap no implementa serde directamente. Podemos mantener `to_bytes()`/`from_bytes()` para serialización binaria, y envolver en un Serializer/Deserializer custom, o simplemente eliminar la derivada serde y usar el formato Portable de croaring.
3. **Performance regression** — `matches_mask` en hot path (search_layer). RoaringBitmap::is_subset es O(n) donde n es el número de contenedores — debería ser comparable o mejor que el loop actual de Vec<u64>.

## 🔗 Referencias

- croaring crate: https://docs.rs/croaring/latest/croaring/
- CRoaring C library: https://github.com/RoaringBitmap/CRoaring
- DiskNodeHeader: `src/node.rs:735` (bitset como u128, fijo a 64 bytes)
- FilterBitset actual: `src/node.rs:18-171`
- search_layer: `src/index/search.rs:89` (hot path con matches_mask)
