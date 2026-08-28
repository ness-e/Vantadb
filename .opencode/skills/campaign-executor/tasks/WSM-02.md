# Task WSM-02 — Manejo cuotas storage browser (QuotaExceededError)

## Estado: ⬜ PENDING

## Archivos clave
- `vantadb-wasm/src/opfs.rs`
- `vantadb-wasm/src/idb.rs`

## Contrato de verificación
- `Select-String -Path "vantadb-wasm/src/opfs.rs" -Pattern "QuotaExceeded|estimate\(\)" | Measure-Object | Select-Object Count` >= 2
- `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` exit 0

## Steps

### Step 1: DISCOVERY - Blast radius y code intelligence
- [ ] CodeGraph explore opfs.rs e idb.rs
- [ ] Detectar referencias entrantes/salientes
- [ ] Verificar coverage del índice
- [ ] Documentar impacto mapeado (Regla 0)

### Step 2: Implementar quota check en OpfsStorage
- [ ] Agregar método `check_quota()` que use `navigator.storage.estimate()`
- [ ] Agregar manejo de `QuotaExceededError` en `write_file` y `append_file`
- [ ] Crear error tipado `QuotaExceeded` con info accionable (usage, quota, % usado)
- [ ] Agregar al menos 2 referencias a "QuotaExceeded" o "estimate()" en opfs.rs

### Step 3: Implementar quota check en IdbStorage
- [ ] Agregar manejo de `QuotaExceededError` en `write_file`
- [ ] Manejar errores de IndexedDB quota exceeded (QuotaExceededError DOMException)

### Step 4: Tests y verificación
- [ ] `cargo check -p vantadb-wasm --target wasm32-unknown-unknown`
- [ ] `cargo fmt --check`
- [ ] `cargo clippy -- -D warnings`

### Step 5: Commit y cierre
- [ ] Commit convencional `feat: WSM-02 — Manejo cuotas storage browser`
- [ ] Ejecutar skill progreso

## Context Save Point
- **Último step completado:** Ninguno
- **Estado actual:** DISCOVERY

## Gate D (Question Gates)
- Blast radius: 2 archivos (opfs.rs, idb.rs) — ≤10 archivos ✓
- No nueva API pública (solo manejo interno de errores) ✓
- Contrato mecánico claro ✓
- No requiere GO del usuario

## Gate V (Verify failures)
- Umbral: 2 fallas mismo error → question al usuario

## Gate C (Colaterales)
- Si git status muestra archivos fuera de opfs.rs/idb.rs → confirmar alcance

---

### Impacto Mapeado (Regla 0) — SECCIÓN REQUERIDA ANTES DE EDITAR

**Archivos leídos completos:**
- `vantadb-wasm/src/opfs.rs` (298 líneas)
- `vantadb-wasm/src/idb.rs` (202 líneas)
- `vantadb-wasm/src/lib.rs` (ver uso de OpfsStorage e IdbStorage)

**Referencias salientes (imports/dependencias):**
- opfs.rs: js_sys, wasm_bindgen, wasm_bindgen_futures
- idb.rs: js_sys, wasm_bindgen, wasm_bindgen_futures

**Referencias entrantes (quién usa estos módulos):**
- lib.rs: `pub use opfs::OpfsStorage`, `pub use opfs::OpfsFile`, `pub use idb::IdbStorage`
- lib.rs: `VantaDB::connect_persistent` usa `OpfsStorage::open`
- lib.rs: `VantaDB::save`/`save_idb`/`load`/`load_idb` usan métodos de OpfsStorage e IdbStorage

**Veredicto de impacto:** BAJO
- Cambios aislados a opfs.rs e idb.rs
- No rompe APIs públicas (solo agrega manejo de errores interno)
- No cambia firmas de funciones existentes
- Solo mejora mensajes de error y agrega checks preventivos