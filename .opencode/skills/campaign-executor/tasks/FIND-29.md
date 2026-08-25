# FIND-29: Último cast manual `u8*`→`f32*` (from_raw_parts) en layer.rs → align_to canónico

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-core-server-mcp.md (Task 9, Wave 0)
- **Fuente:** FIND-29 (plan file) + recomendación de FIND-28 review
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Tipo:** Rust
- **Turns estimados:** 4
- **Creado:** 2026-08-25
- **Estado:** ✅ COMPLETED (implementación + verify; commit lo ejecuta el lead)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0 (4/4 steps ✅)

## Context Save Point (2026-08-25)

**Implementación completa y verificada. Pendiente: commit del lead (worker NO commitea — regla del plan).**

- Steps 1-3 aplicados: los 2 `from_raw_parts(u8*→f32*)` de layer.rs reemplazados por el mecanismo canónico `align_to::<f32>` (el mismo que usa `VectorRepresentations::as_f32_slice`, vector_data.rs:176 — REVIEW-15), con guard `len != expected → 0.0` (mismo fallback que el guard de bounds). `debug_assert_eq!(align_offset(4), 0)` preservado en ambos sitios. Test `test_layer_align_to_decodes_original_values` añadido (equivalencia de valores con el cast previo).
- Verify: `cargo nextest run -p vantadb -E 'test(index)'` **354/354 ✅** · layer/search **10/10 ✅** (incl. `test_layer_align_to_decodes_original_values`, `test_search_vfile_in_memory_parity`, `test_search_vfile_tombstone_header_excluded`) · `cargo check -p vantadb` ✅ · `rustfmt --check` de los 2 archivos ✅ · 0 `from_raw_parts`/`as *const f32` reales residuales en layer.rs ✅.
- ⚠️ Colaterales (Gate C) — **NO son de mi cambio**: `cargo clippy -p vantadb --all-targets -- -D warnings` falla SOLO por dead-code en `src/sdk/builder.rs:25` (`VantaEmbedded` Clone) y `cargo fmt --check` por drift en un test de supersede — ambos son WIP sin commitear de REVIEW-13 (tarea concurrente de la Wave 0, `git status` lo confirma). Mis archivos (layer.rs, tests.rs) están clippy-clean y fmt-clean. NO toqué code de REVIEW-13 (fuera de blast radius, riesgo de conflicto). El lead debe resolver el dead-code de REVIEW-13 (su propia tarea) antes del clippy full-workspace.
- ⚠️ WIP server: `campaign_update_task_state in-progress` bloqueado (MOD-14/REVIEW-13 en progreso por sub-agentes paralelos). El lead debe resolver WIP al cerrar.
- `as_f32_slice` no es invocable directamente en layer.rs porque el path opera sobre sub-rangos `&[u8]` de `VantaFile` (el engine NO construye `MmapFull` en carga vfile — 0 sitios). Se aplicó su MECANISMO (`align_to` + guard de cobertura), el approach exacto de FIND-28 adaptado al tipo real.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `search_layer` es invocado por `src/index/search/mod.rs` (search_nearest/nearest.rs con `vector_store: Option<&VantaFile>`) |
| Callees | `slice::align_to::<f32>` — mecanismo idéntico al de `VectorRepresentations::as_f32_slice()` (src/node/vector_data.rs:176, REVIEW-15) |
| Implicaciones | Los 2 sitios (entry-point loop y neighbor loop) hacen el mismo cast `from_raw_parts(u8*→f32*)` sobre un sub-rango del mmap del vector store (`vs.mmap_bytes()[vec_start..vec_end]`). Sound hoy (INV-024 M-1: `read_header` rechaza `vector_offset % 4 != 0`, vfile.rs:326; + `debug_assert_eq!(align_offset(4), 0)`), pero el cast crudo es UB si la base no está alineada. Fix: mismo approach que FIND-28 — reinterpretación segura vía `align_to` (el mecanismo exacto de `as_f32_slice`). Capa de datos: NO hay `VectorRepresentations` en scope (el engine no construye `MmapFull` en paths vfile); el sub-rango es `&[u8]` crudo del archivo, así que el helper no es invocable directamente — se aplica su mecanismo inline con la misma semántica de fallback. No cambia comportamiento público, no requiere migración/reindex. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `src/index/search/layer.rs` (393 L, completo — sitios 61-71 y 229-239)
  - `src/index/search/tests.rs` (parity test 581-680 — cubre el path `vector_store=Some`; helpers `build_index_with_vfile`)
  - `src/node/vector_data.rs` (as_f32_slice:157-184 — helper canónico REVIEW-15, mecanismo `align_to`)
  - `src/storage/vfile.rs` (read_header:318-330 — guard INV-024 M-1; VantaFileMap as_slice)
- **Referencias hacia dentro (imports/deps):** layer.rs usa `crate::storage::vfile::VantaFile` (ya en scope). `align_to` es método estándar de `slice` — sin nuevos imports. No se agregan imports nuevos a tests.rs (DiskNodeHeader y VantaFile ya importados).
- **Referencias entrantes:** `search_layer` lo llama `src/index/search/mod.rs` (search_nearest) y nearest.rs. Ningún otro módulo toca los casts internos.
- **Veredicto impacto:** bajo — se reemplazan 2 casts inseguros `from_raw_parts` por el mecanismo canónico `align_to` (el mismo que usa `as_f32_slice`). Comportamiento observable idéntico: mismo `&[f32]` (misma reinterp de los mismos bytes), mismo fallback `0.0` cuando el guard de bounds falla y ahora también si el middle alineado no cubre el rango (que hoy no puede pasar dado INV-024 M-1). Elimina los últimos 2 `from_raw_parts u8*→f32*` del index (deuda negativa, Regla 6).

## Contrato
"`cargo nextest run -p vantadb layer` (o tests index) pasa; `cargo clippy -p vantadb --all-targets -- -D warnings`; `as_f32_slice` aplicado; 0 `from_raw_parts` u8*→f32* residual en layer.rs"

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** los 2 sitios deben producir el MISMO `&[f32]` que el cast manual previo (mismos valores — reinterp de los mismos bytes) y mismo fallback `0.0` cuando el guard de bounds (`vec_end <= mmap len`) falla. No alterar lógica de búsqueda ni el algoritmo. Preservar los `debug_assert_eq!(align_offset(4), 0)`.
- **Comandos de verificación:** `cargo check -p vantadb` (OK); `cargo clippy -p vantadb --all-targets -- -D warnings` (limpio); `cargo fmt --check` (OK); `cargo nextest run -p vantadb` (PASS; mínimo tests layer/search/index).
- **Deuda pendiente:** ninguna (el fix elimina deuda).

## Deuda técnica (Regla 6)

**Saldo neto:** Sin deuda — elimina 2 `unsafe from_raw_parts` crudos (moneda de pago). No introduce deuda nueva.

## Definition of Done

| Nivel | Gate |
|-------|------|
| Task | Contrato verificable pasa (check + clippy + fmt + nextest tests layer/search) |
| Commit | Lo ejecuta el lead (worker NO commitea). Diff atómico: layer.rs + tests.rs |
| Release | N/A (fix interno, sin release) |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — fix de seguridad/memoria (UB). Skill `security-and-hardening` cargada. Threat model: el único "input" es memoria mmap del vector store (dato de archivo no confiable en el peor caso); la mitigación es eliminar el UB garantizando alineación vía `align_to` (mismo patrón auditado REVIEW-15/FIND-28). Sin dependencias nuevas, sin secretos, sin boundaries de red/auth.
- [x] **PERFORMANCE** — toca `search_layer` (hot path de búsqueda), pero el fix NO cambia el algoritmo ni el acceso a memoria: `align_to` sobre un slice ya 4-alineado produce el middle en O(1) sin copia (misma zero-copy que el cast previo). No es optimización sino corrección → no requiere benchmark (Regla 9 no aplica a correcciones; FIND-28 igual). Guard de regresión: `test_search_vfile_in_memory_parity` (ids+scores idénticos disk vs in-memory).

## Steps

### Step 1: layer.rs — reemplazar cast en entry-point loop (L61-97)
- **Archivos:** `src/index/search/layer.rs` (L61-97)
- **Acción:** reemplazar `unsafe { std::slice::from_raw_parts(vec_data.as_ptr() as *const f32, header.vector_len as usize) }` por `unsafe { vec_data.align_to::<f32>() }` (mecanismo de `as_f32_slice`, vector_data.rs:176) con guard `f32_vec.len() != header.vector_len as usize → 0.0` (mismo fallback que el guard de bounds). Mantener `debug_assert_eq!(align_offset(4), 0)`. Actualizar comentario SAFETY.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅

### Step 2: layer.rs — reemplazar cast en neighbor loop (L229-267)
- **Archivos:** `src/index/search/layer.rs` (L229-267)
- **Acción:** idéntico al Step 1 para el segundo sitio (`v_data`/`f32_v`).
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅

### Step 3: tests.rs — test de equivalencia de valores
- **Archivos:** `src/index/search/tests.rs`
- **Acción:** añadir test `test_layer_align_to_decodes_original_values`: vfile con 1 nodo (header + payload bytes), replicar la extracción del path disk (read_header → sub-rango → align_to) y assert que el slice decodificado == los valores f32 originales (ground truth = lo que producía el cast manual sobre los mismos bytes). Los tests de parity existentes (L628, L655) cubren la semántica de search completa.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅

### Step 4: verify full
- **Archivos:** —
- **Acción:** correr verificación completa del contrato: clippy, fmt, nextest (layer/search/index).
- **Verify:** `cargo clippy -p vantadb --all-targets -- -D warnings` · `cargo fmt --check` · `cargo nextest run -p vantadb layer` y suite index
- **Estado:** ✅

## Dependencias
- Ninguna (hermano de FIND-28 ya completado `2d9fa75f` — mismo patrón).

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-audit (verificar luego de implementar; gate de cierre).
- **Enfoque:** el approach (reutilizar mecanismo `align_to` de `as_f32_slice` vs documentar SAFETY del cast) es el correcto y consistente con FIND-28/REVIEW-15.
- **Cómo se probó:** cargo check + clippy + fmt + nextest mecánico.
- **Veredicto:** pendiente.

## Notas
- layer.rs opera sobre sub-rangos de `VantaFile` (`vs.mmap_bytes()`), NO sobre `VectorRepresentations` → `as_f32_slice()` no es invocable directamente (el enum no está en scope en ese path; el engine no construye `MmapFull` en la carga vfile — 0 sitios de construcción en engine). Se aplica el mecanismo exacto del helper (align_to + guard de cobertura) — mismo approach FIND-28, adaptado al tipo de dato real.
- Guard INV-024 M-1 (vfile.rs:326) garantiza `vector_offset % 4 == 0`; mmap base ≥4-alineada (memmap2 page-aligned / shim AlignedBytes) → el middle de `align_to` siempre cubre el rango completo en la práctica; el guard de len es belt-and-braces (mismo rol que `aligned.len() != len → None` del helper).
