# REVIEW-04: Refactor 3 god modules — node.rs, config.rs, vfile.rs

## Metadata
- **Plan file:** ninguno activo (Backlog directo)
- **Fuente:** docs/Backlog.md:367
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Tipo:** Rust
- **Turns estimados:** 15-30
- **Creado:** 2026-08-12T00:00
- **last-synced:** 2026-08-12T00:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | UnifiedNode: 88 callers (src/python.rs, src/agentic/thread.rs, src/gc.rs, src/ingestion.rs +27, tests storage/engine/tests/*). FieldValue: 32 callers (src/scalar_index.rs, cli_handlers/diagnostics.rs, cli_handlers/crud.rs +16). VantaFile: storage engine, archive. Config: cli, cli_server, sdk |
| Callees | node.rs → storage/vfile.rs (Mmap), zerocopy, croaring, serde. vfile.rs → binary_header, crypto (Cipher/EncryptionStream feature encryption), node::DiskNodeHeader. config.rs → backend, storage::engine::SegmentOptimizerConfig, tokenizer (feature advanced-tokenizer) |
| Implicaciones | Re-exports públicos en lib.rs:157-160 NO cambian (Edge, FieldValue, NodeFlags, RelFields, SparseVector, UnifiedNode, VectorRepresentations, DistanceMetric). `crate::node::*` paths internos (storage/ops.rs usa `crate::node::RelFields`/`crate::node::Edge`) deben seguir resolviendo. API pública semver intacta — refactor interno puro, zero behavior change |
| Riesgo | medio — 88+32 callers requieren que los re-exports de node.rs queden idénticos; vfile.rs tiene 22 `unsafe` que deben moverse con sus `// SAFETY:` intactos |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/node.rs` (2078L — secciones mapeadas vía grep/section headers + codegraph), `src/config.rs` (1595L — header 1-40), `src/storage/vfile.rs` (1309L — header 1-40), `src/lib.rs` (re-exports 157-160)
- **Archivos referenciados hacia dentro:** node.rs → `crate::storage::vfile::Mmap`, zerocopy, croaring, serde, web_time. vfile.rs → `crate::binary_header::VantaHeader`, `crate::node::DiskNodeHeader`, `crate::crypto::{Cipher, EncryptionStream}` (feature), `crate::storage::engine::STORAGE_ALIGNMENT` (feature darwin). config.rs → `crate::backend::BackendKind`, `crate::storage::engine::SegmentOptimizerConfig`, `crate::tokenizer::AdvancedTokenizerConfig` (feature)
- **Archivos que referencian a los editados:** `src/lib.rs` (pub mod node, pub use node::*), `src/storage/mod.rs` (pub mod vfile), `src/storage/ops.rs` (`crate::node::RelFields`/`crate::node::Edge`), `src/columnar.rs` (`nodes_to_record_batch` usa UnifiedNode), cli_server.rs (NodeDTO From<&UnifiedNode>), + todos los callers de UnifiedNode/FieldValue listados arriba
- **Veredicto impacto:** medio — el refactor solo mueve items entre archivos/módulos; si los re-exports y module paths (`crate::node`, `crate::storage::vfile`) quedan idénticos, nada externo cambia. Clave: NO renombrar items públicos, NO cambiar visibilidad pub→pub(crate)

## Contrato
"`cargo check -p vantadb` pasa, `cargo nextest run --profile audit -p vantadb --build-jobs 2` pasa, `src/lib.rs` re-exports idénticos (grep de `pub use node::` anterior/posterior), `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` sin warnings nuevos, y cero cambios de comportamiento/API (duff diff de lógica = refactor puro)"

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. Re-exports públicos `lib.rs:157-160` (Edge, FieldValue, NodeFlags, RelFields, SparseVector, UnifiedNode, VectorRepresentations, DistanceMetric) presentes y con la MISMA firma/semántica
  2. Todo path interno `crate::node::*` y `crate::storage::vfile::*` sigue resolviendo (grep de callers antes/después)
  3. Los 22 `unsafe` de vfile.rs se movieron CON su bloque `// SAFETY:` intacto (core-engine.md R-4)
  4. Cero `unwrap()`/`expect()` nuevos (core-engine.md R-3)
  5. `unsafe` total del workspace no aumenta (Regla 6 — saldo neto deuda 0/negativo)
  6. `config.rs` NO se parte (ver Notas — ponytail assessment existente dice "cohesive, leave as-is")
- **Comandos de verificación:**
  - `cargo check -p vantadb` → exit 0
  - `cargo nextest run --profile audit -p vantadb --build-jobs 2` → exit 0
  - `cargo fmt --check` → exit 0
  - `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` → exit 0
  - `rg -c "unsafe" src/storage/vfile.rs` → no mayor que antes del refactor (los unsafe se mueven, no se multiplican)
- **Deuda pendiente:** ninguna si el refactor preserva invariantes; si aparece deuda nueva de movimiento, documentar el pago

## Recitation (canónico — estructura única)

- **activeGoal:** REVIEW-04 — Refactor 3 god modules (node.rs, config.rs, vfile.rs) sin cambio de API ni comportamiento
- **lastAction:** Definición de tarea: blast radius mapeado, steps atómicos creados, delegación a vanta-worker
- **result:** ⬜ PENDING
- **nextAction:** Ejecutar Step 1 — crear `src/node/` como directorio y mover secciones
- **contract:** verificación = `cargo check -p vantadb` + `cargo nextest run --profile audit -p vantadb --build-jobs 2` + clippy -D warnings + re-exports idénticos
- **nextTask:** REVIEW-05 (god files restantes: serialize.rs, distance.rs, physical_plan.rs)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — refactor de movimiento no introduce deuda nueva; paga la deuda de "god module node.rs"

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato del task file se cumple (check + nextest + clippy + re-exports idénticos) |
| **Commit** | 1-2 commits atómicos por archivo (node.rs, vfile.rs), conventional commit `refactor:`, diff de lógica ZERO (solo movimiento) |
| **Release** | No aplica release (refactor interno sin cambios API) — justificado: cambio interno, semver no cambia |

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, test)
- rust-analyzer-mcp (diagnostics, goto def para mover items con precisión)
- codegraph_explore (verificar que callers sigan resolviendo)

## Investigation Notes
- Backlog 2026-08-09 reportaba node.rs 1554→1882L; hoy está en 2078L (creció aún más) — el problema de god module empeoró, refactor sigue justificado
- config.rs 1313→1595L pero el HEAD del archivo YA tiene assessment ponytail: "1287L but cohesive - enums, structs, Default (env parsing), builder methods, hot-reload watcher are all interdependent. Splitting would add indirection without reducing real complexity. Leave as-is."
  → **Decisión: config.rs se EXCLUYE del split activo.** Solo validar que el assessment sigue vigente y registrar en Notas. No pierde campaña: node.rs (2078L) y vfile.rs (1309L) son los que aportan
- vfile.rs: mmap shim (1-~135) + resident_bytes (240-245) + AlignedBytes (403-461) + VantaFileMap (490) + VantaFile (532-883) + tests (890+). Split natural: `mmap.rs` (shim + AlignedBytes + resident_bytes) separado de VantaFile. VantaFile queda ~350L → deja de ser god module
- node.rs: split natural por secciones — FilterBitset→`bitset.rs`; VectorData (VectorRepresentations/SparseVector)→`vector_data.rs`; LabelIntern→`label.rs`; Edge→`edge.rs`; FieldValue→`field.rs`; NodeFlags+NodeTier→`flags.rs`; DiskNodeHeader→`disk.rs`; UnifiedNode→`unified.rs`; tests→`tests/` o módulos de test por archivo

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 1 — decidir si vfile.rs mmap shim va a subdir separado o queda // ─── al archivo (impacto mínimo, decidir en ejecución) |
| Pendientes de ejecución (downhill) | 6 steps |
| % completado | 0% |

## Steps

### Step 1: Preparar estructura de directorios + inventario exacto
- **Archivos:** `src/node/` (nuevo dir), `src/node.rs` (existe)
- **Acción:** Convertir `src/node.rs` en `src/node/mod.rs` (git mv). Inventariar items exportados (grep `pub` top-level) y mantener lista de re-exports en mod.rs. NO mover código todavía
- **Verify:** `cargo check -p vantadb` (funciona igual con mod.rs)
- **Estado:** ⬜ PENDING

### Step 2: Mover Edge + FieldValue + NodeFlags + NodeTier + DiskNodeHeader a submódulos
- **Archivos:** `src/node/edge.rs`, `src/node/field.rs`, `src/node/flags.rs`, `src/node/disk.rs`, `src/node/mod.rs`
- **Acción:** Mover cada sección (Edge 702-875, FieldValue 876-1005, NodeFlags 1006-1072, NodeTier 1073-1100, DiskNodeHeader 1101-1159) a su archivo, con `pub use` re-exports en mod.rs. Mantener items `pub` idénticos (firma exacta, docs intactas)
- **Verify:** `cargo check -p vantadb` + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings`
- **Estado:** ⬜ PENDING

### Step 3: Mover UnifiedNode + VectorData + LabelIntern + FilterBitset a submódulos
- **Archivos:** `src/node/unified.rs`, `src/node/vector_data.rs`, `src/node/label.rs`, `src/node/bitset.rs`, `src/node/mod.rs`
- **Acción:** Mover UnifiedNode (1160-1350), VectorData (458-645), LabelIntern (646-701), FilterBitset (1-457) a sus archivos con re-exports. OJO: FilterBitset tiene cfg(feature="roaring")/no-roaring dual — mover AMBAS variantes
- **Verify:** `cargo check -p vantadb` + `cargo nextest run --profile audit -p vantadb --build-jobs 2`
- **Estado:** ⬜ PENDING

### Step 4: Mover tests de node.rs a módulos de test por archivo
- **Archivos:** `src/node/**/*.rs` (tests modules)
- **Acción:** tests (1351-2078) se distribuyen a `#[cfg(test)] mod tests` dentro de cada submódulo según el item que testean. Cero tests eliminados/reducidos
- **Verify:** `cargo nextest run --profile audit -p vantadb --build-jobs 2` (misma suite, misma cantidad de tests)
- **Estado:** ⬜ PENDING

### Step 5: Split vfile.rs — separar mmap shim + AlignedBytes
- **Archivos:** `src/storage/vfile.rs`, `src/storage/vfile/` opcional
- **Acción:** Evaluar en ejecución: si mmap_shim + AlignedBytes + resident_bytes (~490L de 1309L) se separan a `src/storage/vfile_mmap.rs` (o subdir vfile/) con re-export `pub(crate) use`, VantaFile queda ~820L con tests. Preservar los 22 `unsafe` con sus `// SAFETY:` (core-engine R-4)
- **Verify:** `cargo check -p vantadb` + `cargo nextest run --profile audit -p vantadb --build-jobs 2` + `rg -c "unsafe"` no mayor
- **Estado:** ⬜ PENDING

### Step 6: Verificación full + re-exports + distribución
- **Archivos:** todo el refactor
- **Acción:** `cargo fmt --check`, `cargo clippy -p vantadb --all-targets --all-features -- -D warnings`, `cargo nextest run --profile audit -p vantadb --build-jobs 2`, comparar `grep pub use node::` en lib.rs (idéntico al previo), `rg -c unsafe` no mayor, y confirmar zero diff de lógica (git diff --stat: solo movimientos). Review por agente distinto (P2-01 gate)
- **Verify:** todos los comandos exit 0
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (standalone desde Backlog)

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-audit (leaf — solo revisa, nunca implementa)
- **Enfoque:** ¿el refactor es movimiento puro o cambió semántica? ¿los re-exports quedaron idénticos? ¿los unsafe se movieron con SAFETY?
- **Cómo se probó:** `git diff --stat` + comandos de verificación reales (check/nextest/clippy) ejecutados, no auto-reporte
- **Checklist anti-hábitos tóxicos:** (se llena en review)
- **Veredicto:** ⏳ pendiente

## Notas
- **config.rs NO se parte** — el assessment ponytail en el header del archivo ("cohesive... leave as-is") es la decisión registrada. Este refactor cubre node.rs + vfile.rs; config.rs queda documentado como no-split con justificación. La fila de backlog decía "3 god modules" pero el análisis actual reduce a 2 con justificación técnica (invariante ya registrado arriba)
- vfile.rs unsafe: 22 usos concentrados (core-engine.md R-4), confirmado en reglas de área
- Features a respetar: `roaring` (FilterBitset dual), `encryption` (vfile cipher), `advanced-tokenizer` (config), `memmap2` (shim) — el refactor NO debe cambiar feature-gating
- api-contract.md R-7: NO gatear campos pub de structs públicos con cfg(feature) — el refactor no debe introducir cfg nuevos en campos pub

## Context Save Point
- **Fecha:** 2026-08-12T00:00
- **Branch:** develop (verify branch actual)
- **CI pendiente:** sí — verify.ps1 antes de push
- **Decisiones:** config.rs excluido del split (ponytail assessment existente); node.rs y vfile.rs se parten
- **Problemas conocidos:** ninguno
- **Próxima tarea:** REVIEW-05