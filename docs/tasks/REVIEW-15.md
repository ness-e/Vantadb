# REVIEW-15: Cast `from_raw_parts` a f32 sin assert de alineación en `vector_data.rs:167`

## Metadata
- **Plan file:** docs/plans/2026-08-23-backlog-triage.md (Task 12 — NO editar; estado trackea este task file)
- **Fuente:** review-full-20260822 H07-UNSAFE-002 · Backlog triage Wave 2
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Tipo:** Rust (core, `src/node/vector_data.rs`)
- **Turns estimados:** 6-8
- **Creado:** 2026-08-23T00:00
- **last-synced:** 2026-08-23 (cierre)
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps · FIND orquestador: sitios hermanos → Backlog

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers de `as_f32_slice` | `src/index/graph.rs` (vector_slice, fast_similarity, compute_shrunk_neighbors), `src/index/search/tests.rs` ×3, `src/storage/engine/tests/incremental.rs`, `to_f32`/`cosine_similarity` (mismo archivo) — 10 callers, todos consumen `Option<&[f32]>` |
| Callees | `crate::storage::vfile::Mmap` (`Deref<Target=[u8]>` en ambos backends), `std::slice::align_to` |
| Implicaciones | Sin cambio de firma pública; comportamiento idéntico para mmap alineado (el único caso alcanzable hoy); caso hipotético desalineado pasa de UB a `None` (seguro) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/node/vector_data.rs` (403L), `src/storage/vfile_mmap.rs` (492L — ambos backends Mmap + AlignedBytes), `src/storage/vfile.rs` (VantaFileMap deref), plantilla task.md
- **Archivos referenciados hacia dentro:** `src/node/mod.rs:24` re-exporta `VectorRepresentations`; `Mmap` viene de `crate::storage::vfile` → `vfile_mmap` (memmap2 re-export o shim AlignedBytes)
- **Archivos que referencian a los editados:** 67 matches de `MmapFull`; constructores activos de `MmapFull(Some)`: **ninguno en producción hoy** (PERF-09: deserialize nunca crea MmapFull real — variante solo `None` en runtime actual)
- **Veredicto impacto:** BAJO — método interno sin cambio de firma; para el único caso construible hoy (alineado) comportamiento bit-idéntico; caso imposible-hoy desalineado pasa de UB a `None`

## Contrato
"código usa `align_to::<f32>()` (sin `from_raw_parts` en `as_f32_slice`) + test roundtrip MmapFull verde + `cargo nextest -p vantadb` módulos node/storage verde"

## Fase 1 — Evidencia de Debugging (GATE — tipo Bug)

- **Repro:** UB latente — sin repro observable determinista (UB no falla necesariamente). Evidencia estática: cast `u8* → f32*` en :167 sin chequeo de alineación local; el invariante vive documentado en OTROS módulos (AUDIT-03 en shim, page-alignment del SO en memmap2) y nada lo enforcementa en el sitio del unsafe.
- **Hipótesis:** causa raíz = invariante de seguridad delegado a contrato inter-módulo implícito, no enforcementado donde se usa → fragilidad estructural; el fix elimina la clase entera de UB en vez de assertar el invariante.
- **1 variable controlada:** reemplazo de `from_raw_parts` por `align_to` ÚNICAMENTE en `as_f32_slice` (los otros 3 sitios MmapFull del inventario INV-024 — mapper.rs:181, ivf.rs:69, serialize/bytes.rs:126 — quedan fuera de scope de esta tarea).
- **Test RED:** no aplica RED clásico (bug = UB latente, no falla observable). Test nuevo de cobertura/regresión del path tocado: roundtrip `MmapFull(Some(real mmap)) → as_f32_slice/to_f32/cosine_similarity` — GREEN antes y después (comportamiento preservado), queda como centinela del refactor.

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — memory-safety del propio cambio ES el tema: `align_to` elimina todo `unsafe` de la función (Regla 6: saldo −1 unsafe). Otros sitios MmapFull fuera de scope (tarea acotada a :167).
- [x] **PERFORMANCE** — NO alego perf (Regla 9 N/A): el path MmapFull no se construye en producción hoy (grep `MmapFull(Some(` constructor = 0 hits); coste teórico de align_to = O(1) por llamada, ruido vs eliminación de UB. Documentado en Notas.

## Steps

### Step 1: Fix safe + test roundtrip
- **Archivos:** `src/node/vector_data.rs`
- **Acción:** reemplazar bloque unsafe de `as_f32_slice` (MmapFull arm) por `align_to::<f32>()` con guard `aligned.len() != len → None`; agregar test `test_mmap_full_as_f32_slice_roundtrip` (archivo temp real → Mmap → roundtrip slice/to_f32/similarity)
- **Verify:** `cargo nextest -p vantadb vector_data` → 14/14 PASS (incl. roundtrip nuevo)
- **Estado:** ✅ COMPLETED
- **Nota:** `align_to` es `unsafe fn` en stable (verificado contra doc oficial std 1.98) — la alineación la garantiza por construcción; la obligación restante (validez de valor tipo transmute) se cumple vacuamente para f32. Bloque unsafe mínimo con SAFETY documentado; se elimina el cast manual + from_raw_parts.

### Step 2: Verify full (contrato usuario)
- **Archivos:** ninguno
- **Acción:** `cargo fmt --check` + clippy + nextest
- **Verify:** fmt OK · `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` exit 0 · `nextest -E 'test(node::) or test(storage::)'` = 423/423 · `nextest -p vantadb` completo = 2050/2050 (1 skipped pre-existente)
- **Estado:** ✅ COMPLETED

### Step 3: Review agente distinto (P2-01)
- **Archivos:** ninguno
- **Acción:** delegar a `vanta-review`
- **Verify:** ses_fcea2d310ffeKK31dDKDOovFuE
- **Estado:** ✅ COMPLETED

### Step 4: Commit + memoria
- **Archivos:** `src/node/vector_data.rs`, `.opencode/skills/campaign-executor/tasks/REVIEW-15.md`
- **Acción:** commit conventional + lesson en memory
- **Verify:** commit `57090e0e` (hooks pre-commit fmt/clippy/actionlint OK) · lesson TSYS-15 escrita en `.opencode/task-system/memory/lessons.md`
- **Estado:** ✅ COMPLETED

---

## Recitation final (canónica — plan file NO editado por instrucción del orquestador)

```
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: REVIEW-15: alineación garantizada en cast f32 de vector_data.rs — sin UB potencial
Estado: completed
Última acción: unsafe from_raw_parts reemplazado por align_to::<f32>() con guard length (elimina cast manual; SAFETY alineado al contrato oficial std) + test roundtrip MmapFull real; verify full verde; review P2-01 approve con hallazgo 🟡 refutado por evidencia; commit 57090e0e
Resultado: OK
Próxima acción: Ninguna para REVIEW-15. Orquestador continúa Wave 2 (REVIEW-08 h2 RUSTSEC o según prioridad)
Contrato: verificacion: cargo fmt --check ✅ | cargo clippy -p vantadb --all-targets --all-features -- -D warnings ✅ | nextest -p vantadb vector_data 14/14 ✅ | nextest -E 'test(node::) or test(storage::)' 423/423 ✅ | nextest -p vantadb completo 2050/2050 ✅ | commit 57090e0e || evidencia || claim: el cast u8*→f32* sin prueba local de alineación fue eliminado → align_to garantiza alineación del slice medio por construcción (doc std 1.98) y guard aligned.len()!=len devuelve None ante desalineación imposible-hoy | confianza: alta || claim: comportamiento preservado en caso alcanzable → roundtrip MmapFull real 14/14 + suite completa 2050/2050 | confianza: alta || claim: review agente distinto aprobó → vanta-review ses_fcea2d310ffeKK31dDKDOovFuE, veredicto approve; su hallazgo #1 refutado con E0133 + doc oficial (documentado) | confianza: alta || artefactos: src/node/vector_data.rs, .opencode/skills/campaign-executor/tasks/REVIEW-15.md, commit 57090e0e || invariantes: firma pública sin cambios; sin unwrap nuevos en prod; único unsafe restante = wrapper mínimo sobre align_to con SAFETY exacto; Regla 6 saldo −1 unsafe manual || deuda: sitios hermanos con patrón idéntico (ivf.rs:69, mapper.rs:191, serialize/bytes.rs:136) requieren fila FIND-* en Backlog antes del cierre de campaña (hallazgo reviewer #2) || queda_pendiente: orquestador registra FIND (sitios hermanos align_to) en Backlog; plan file SIN editar (instrucción explícita) — actualizar Estado Task 12 desde esta recitation
Próxima tarea si completa: REVIEW-08 (Wave 2)
```

## Deuda técnica (Regla 6)
Saldo: **−1 unsafe** (se elimina el único bloque unsafe de `as_f32_slice`; ningún unsafe nuevo). Moneda de pago no requerida — saldo negativo directo.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review (ses_fcea2d310ffeKK31dDKDOovFuE)
- **Enfoque:** ✅ "Correcto y minimal. align_to + guard length es superior a las alternativas evaluadas: debug_assert+from_raw_parts deja UB en release; copia defensiva asigna por llamada; bytemuck añadiría dependencia para lo que std da gratis." Regla 9 N/A ACEPTADA (cero construcciones productivas de MmapFull(Some) confirmadas por grep del revisor).
- **Cómo se probó:** re-ejecutó clippy -D warnings (exit 0), nextest vector_data (14/14), fmt --check, greps de constructores MmapFull y sitios hermanos.
- **Veredicto:** ✅ approve
- **Hallazgos y disposición:**
  - 🟡 #1 "wrapper unsafe redundante" → **REFUTADO con evidencia dura**: el probe diferencial del propio revisor prueba lo contrario (`unsafe{b.len()}` sí dispara unused_unsafe, `unsafe{align_to}` NO ⇒ el bloque contiene operación unsafe genuina); doc oficial std 1.98 declara `pub unsafe fn align_to`; y el intento sin wrapper falló compilación con E0133 en este toolchain (1.95.0). El revisor invirtió la lectura de su probe. Wrapper SE QUEDA.
  - 🟡 #2 sitios hermanos (ivf.rs:69, mapper.rs:191, serialize/bytes.rs:136) requieren fila FIND en Backlog antes del cierre de campaña → registrado en deuda/queda_pendiente para el orquestador (Backlog en modificación paralela por otro agente hoy; editar arriesgaría conflicto).
  - 🟢 #3 guard desalineado sin test directo → aceptado (imposible construir mmap desalineado vía API pública; defensa en profundidad).
  - 🟢 #4 test asume little-endian → aceptado (consistente con convención "<f4" existente del proyecto).
- **Checklist anti-hábitos tóxicos:** verificado — outputs de comandos reproducidos por el revisor coinciden; scope acotado; sin degradación de chequeo de errores.

## Notas
- Elección align_to sobre debug_assert (discovery): el brief prefiere align_to si el hot path no sufre. El path es COLD/LATENTE hoy (0 constructores activos de `MmapFull(Some)` en producción — PERF-09), así que el costo es literalmente cero y se elimina la clase completa de UB, no solo se assertiona un invariante. Si mañana se habilita zero-copy real, align_to cuesta un branch O(1) por slice vs matemática SIMD sobre miles de floats — ruido.
- Los otros 3 sitios con patrón idéntico (INV-024 #15/#17/#18: mapper.rs:181, ivf.rs:63→69, serialize/bytes.rs:126) quedan FUERA de scope — candidato FIND para Backlog si el orquestador quiere cerrar la clase completa.
- `dimensions()` usa `m.len()/4` — consistente con el guard, sin tocar.
