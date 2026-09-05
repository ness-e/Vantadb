# FIND-28: Casts `u8*`→`f32*` sin chequeo de alineación (UB) en 3 sitios

## Metadata
- **Plan file:** docs/plans/2026-08-24-batch-review-mod-find.md
- **Fuente:** FIND-28 (plan file, wave 1)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟠
- **Tipo:** Rust
- **Turns estimados:** 5
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24
- **Estado:** ✅ COMPLETED (implementación + verify + review; commit lo ejecuta el lead)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0 (4/4 steps ✅)

## Context Save Point (2026-08-24)

**Implementación completa y verificada. Pendiente: commit del lead (worker NO commitea).**

- Steps 1-3 aplicados: los 3 casts crudos reemplazados por `as_f32_slice()` + eliminada const muerta `MAX_VEC_F32_LEN` (graph.rs:20).
- Verify: `cargo check -p vantadb` ✅ · `cargo clippy -p vantadb --all-targets -- -D warnings` ✅ · `rustfmt --check` de los 4 archivos ✅ · `cargo nextest run -p vantadb` 2055/2055 ✅
- Review P2-01: vanta-audit **✅ APPROVE** (evidencia punto por punto; sin casts crudos residuales en los archivos tocados; semántica None/0.0/Err preservada; eliminación de const segura). Recomendación no bloqueante → FIND-29 creado en Backlog.
- ⚠️ WIP: `campaign_update_task_state in-progress` bloqueado por el server (DESKTOP-24/28, REVIEW-06 con estado in-progress stale). El lead debe resolver WIP antes de marcar FIND-28 ✅.
- ⚠️ Colateral: `cargo fmt --check` del workspace falla SOLO en `vantadb-python/src/lib.rs` (drift pre-existente de MOD-19, fuera de blast radius). No tocado. El lead debe correr fmt al cerrar MOD-19.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/index/ivf.rs` (IvfIndex::build), `src/index/distance/mapper.rs` (calculate_similarity — hot path de búsqueda), `src/index/serialize/bytes.rs` (serialize de nodos) |
| Callees | `VectorRepresentations::as_f32_slice()` (src/node/vector_data.rs:157) — helper seguro ya existente (REVIEW-15) |
| Implicaciones | Los 3 sitios hacen el mismo cast inseguro `mmap.as_ptr() as *const f32` vía `from_raw_parts` sin verificar alineación. Ambos backends mmap garantizan base 4-alineada (memmap2: page-aligned; shim: `AlignedBytes` align 4 — AUDIT-03), pero el cast crudo es UB si la alineación no se cumple. Fix: reutilizar `as_f32_slice()` que ya usa `align_to::<f32>()` y retorna `None` si el middle alineado no cubre el rango. No cambia comportamiento público, no requiere migración/reindex. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `src/index/ivf.rs` (L59-78: `node_to_f32_slice`)
  - `src/index/distance/mapper.rs` (L150-203: arm `MmapFull` de `calculate_similarity`)
  - `src/index/serialize/bytes.rs` (L110-156: arm `MmapFull` de serialización)
  - `src/node/vector_data.rs` (L157-184: `as_f32_slice` — helper seguro a reutilizar)
- **Referencias hacia dentro (imports/deps):** los 3 archivos usan `VectorRepresentations` (tipo ya en scope vía `crate::node`). `as_f32_slice` es `pub fn` sobre el enum → accesible en los 3 módulos sin nuevos imports.
- **Referencias entrantes:** `node_to_f32_slice` solo lo usa `IvfIndex::build` (ivf.rs:90). `calculate_similarity` lo llaman los kernels de distancia y search. `serialize` lo llama el persist de nodos.
- **Veredicto impacto:** bajo — se reemplaza un cast inseguro por el helper seguro equivalente; comportamiento observable idéntico (mismo slice, misma semántica de None/0.0/Err en los casos límite ya chequeados). Elimina 3 bloques `unsafe` (deuda negativa, Regla 6).

## Contrato
"`cargo check -p vantadb` + `cargo clippy -p vantadb --all-targets -- -D warnings` + `cargo fmt --check` limpios, y `cargo nextest run -p vantadb` (tests de ivf/distance/serialize) pasan; los 3 casts crudos reemplazados por `as_f32_slice()` (que aplica `align_to`)."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** los 3 sitios deben producir el MISMO slice `&[f32]` que antes (mismos valores, misma semántica de casos límite: ivf→None, mapper→0.0, bytes→Err InvalidData). No alterar lógica de negocio ni el algoritmo.
- **Comandos de verificación:** `cargo check -p vantadb` (esperado: OK); `cargo clippy -p vantadb --all-targets -- -D warnings` (esperado: limpio); `cargo fmt --check` (esperado: OK); `cargo nextest run -p vantadb` (esperado: PASS).
- **Deuda pendiente:** ninguna (el fix elimina deuda).

## Deuda técnica (Regla 6)

**Saldo neto:** Sin deuda — elimina 3 `unsafe` existentes (moneda de pago por la deuda P2-2/raw-pointer-UB ya resuelta en AUDIT-01 y consolidada aquí). No introduce deuda nueva.

## Definition of Done

| Nivel | Gate |
|-------|------|
| Task | Contrato verificable pasa (check + clippy + fmt + nextest) |
| Commit | Lo ejecuta el lead (worker NO commitea). Diff atómico, ~30 líneas |
| Release | N/A (fix interno, sin release) |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — fix de seguridad/memoria (UB). Skill `security-and-hardening` cargada. Threat model: el único "input" es memoria mmap; la mitigación es eliminar el UB garantizando alineación vía `align_to`. Sin dependencias nuevas, sin secretos, sin boundaries de red/auth.
- [x] **PERFORMANCE** — toca `calculate_similarity` (hot path de búsqueda), pero el fix NO cambia el algoritmo: `as_f32_slice()` para mmap ya era el path usado; el costo es idéntico (mismo slice zero-copy). No requiere benchmark (no es optimización, es corrección). Si en el futuro el copy de `node_to_f32_slice` fuera hot, evaluar; hoy ivf.build es offline.

## Steps

### Step 1: ivf.rs — reemplazar cast inseguro
- **Archivos:** `src/index/ivf.rs` (L60-74)
- **Acción:** `node_to_f32_slice` → `vector.as_f32_slice().map(|s| s.to_vec())` (reutiliza helper REVIEW-15; elimina el `unsafe` y el chequeo manual de len que ya hace el helper).
- **Verify:** `cargo check -p vantadb` ✅
- **Estado:** ✅

### Step 2: distance/mapper.rs — reemplazar cast inseguro
- **Archivos:** `src/index/distance/mapper.rs` (L181-191)
- **Acción:** en el arm `MmapFull`, reemplazar el `from_raw_parts` por `node_vec.as_f32_slice()` (None → `return 0.0`, igual semántica que el chequeo len actual). Eliminado `use crate::index::MAX_VEC_F32_LEN` (quedó muerto).
- **Verify:** `cargo check -p vantadb` ✅
- **Estado:** ✅

### Step 3: serialize/bytes.rs — reemplazar cast inseguro
- **Archivos:** `src/index/serialize/bytes.rs` (L126-142)
- **Acción:** en el arm `MmapFull`, reemplazar el `from_raw_parts` por `node.vec_data.as_f32_slice()` (None → Err InvalidData, igual semántica; colapsa los 2 errores en 1 — ningún test aserte el mensaje). Eliminada const muerta `MAX_VEC_F32_LEN` (graph.rs:20) — quedó sin usuarios crate-wide (vector_data.rs tiene copia local).
- **Verify:** `cargo check -p vantadb` ✅
- **Estado:** ✅

### Step 4: verify full
- **Archivos:** —
- **Acción:** correr verificación completa del contrato.
- **Verify:** `cargo clippy -p vantadb --all-targets -- -D warnings` ✅ · `rustfmt --check` (4 archivos) ✅ · `cargo nextest run -p vantadb` 2055/2055 ✅ · Review P2-01 vanta-audit ✅ APPROVE
- **Estado:** ✅

## Dependencias
- Ninguna.

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-audit (verificar luego de implementar; gate de cierre).
- **Enfoque:** el approach (reutilizar helper `align_to` existente vs documentar SAFETY) es el correcto y consistente con REVIEW-15.
- **Cómo se probó:** cargo check + clippy + fmt + nextest mecánico.
- **Veredicto:** pendiente.

## Notas
- Paths reales: el plan decía `src/storage/engine/mapper.rs:191` y `src/sdk/serialization/bytes.rs:136`, pero los archivos reales son `src/index/distance/mapper.rs:191` y `src/index/serialize/bytes.rs:136` (el plan tenía rutas aproximadas; verificado por glob+grep). El sitio `src/index/ivf.rs:69` coincide.
- `as_f32_slice()` ya usa `align_to::<f32>()` y retorna None si el middle alineado no cubre el rango (REVIEW-15, vector_data.rs:157-184) — es el patrón canónico de este repo.
