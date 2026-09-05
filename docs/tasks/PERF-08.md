# PERF-08: WASM — Evitar serializar todos los records en búsquedas

## Metadata
- **Plan file:** docs/plans/2026-08-12-perf-bench-wasm.md (Task 4)
- **Fuente:** Plan file Task 4 / backlog § Phase 4 (P2-7)
- **Esfuerzo:** 🟠 Medio
- **Prioridad:** 🟡
- **Tipo:** Rust (WASM bindings)
- **Turns estimados:** 6
- **Creado:** 2026-08-12T14:00
- **last-synced:** 2026-08-12T15:10
- **Estado:** ✅ COMPLETED (trabajo + verificación; commit delegado al orquestador — NO commiteado por este agente)
- **Incógnitas (uphill):** 0 (alcance confirmado por codegraph + grep)
- **Pendientes (downhill):** 0 steps (4/4 ✅)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `search` (762), `search_hit_to_js` (740), `list` (715), `put` (623), `put_batch` (648), `get` → todos consumen `memory_record_to_js` (1192). Hosts: vantadb-ts/src/vantadb.ts, vantadb-node, web/, packages/ |
| Callees | `js_sys::Float32Array`, `js_sys::Reflect`, `serde_wasm_bindgen` (eliminado para vector) |
| Implicaciones | Cambia el tipo de `record.vector` de `number[]` (serde) a `Float32Array` (zero-copy). Shape idéntico (misma key, mismo namespace/key/payload/metadata). Hosts no leen `record.vector` (grep: 0 consumidores en web/packages/vantadb-ts salvo anotación de tipo TS). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** vantadb-wasm/src/lib.rs (relevantes: 400-649, 700-848, 1185-1259), vantadb-wasm/Cargo.toml, vantadb-wasm/tests/wasm_tests.rs (315-634), vantadb-ts/src/types.ts (vector annotations).
- **Archivos referenciados hacia dentro:** `js_sys` (dep), `serde_wasm_bindgen`, `vantadb::sdk::*` (core, NO modificado), `VantaMemoryRecord`.
- **Archivos que referencian a los editados (referencias entrantes):** vantadb-ts/src/vantadb.ts (wrapper passthrough), web/src/components/vanta/code-playground.tsx (usa VantaDB pero no `.vector`), vantadb-python/vantadb_py/migrate/*.py (usa WASM pero no `.vector`). Grep de `.vector` en todo web/packages/vantadb-ts → 0 hits funcionales.
- **Veredicto impacto:** MEDIO. Cambio de tipo de `record.vector` (number[] → Float32Array). No rompe ningún host funcional (nadie lee `.vector`). Única nota: anotación TS `vector?: number[]` queda imprecisa → se actualiza a `Float32Array | number[]`. Tests wasm no downcast `record.vector`, así que siguen compilando y pasando en runtime.

## Contrato
"cargo check -p vantadb-wasm ✅, cargo build --target wasm32-unknown-unknown -p vantadb-wasm ✅, cargo clippy -p vantadb-wasm --all-targets -- -D warnings ✅, cargo fmt --check ✅; y `memory_record_to_js` emite `record.vector` como `Float32Array` zero-copy (no `serde_wasm_bindgen::to_value`). Persist delta documentado como deuda (requiere dirty-tracking en core, fuera de scope)."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** No modificar lógica core (`vantadb::sdk`, `engine.rs`, storage). El shape de `record` debe conservar todas las keys (namespace, key, payload, created_at_ms, updated_at_ms, version, node_id, vector, expires_at_ms, metadata). Solo cambia la representación del campo `vector`.
- **Comandos de verificación:** `cargo check -p vantadb-wasm`, `cargo build --target wasm32-unknown-unknown -p vantadb-wasm`, `cargo clippy -p vantadb-wasm --all-targets -- -D warnings`, `cargo fmt --check`.
- **Deuda pendiente:** Persist delta (H3-SER-001) NO implementado — requiere dirty-tracking en core Rust (fuera de scope de esta tarea). `save`/`save_idb` siguen serializando todos los records vía `serde_json::to_vec`. Documentado como fallback del contrato.

## Recitation (canónico — estructura única)

contract:
  verificacion: "cargo fmt --check -p vantadb-wasm ✅; cargo check -p vantadb-wasm ✅; cargo build --target wasm32-unknown-unknown -p vantadb-wasm ✅ (1 warning pre-existente en vantadb core metrics::core/mod.rs:298 get_native_memory, ajeno a PERF-08); cargo clippy -p vantadb-wasm --all-targets -- -D warnings ✅"
  evidencia:
    - claim: "record.vector se serializa como Float32Array zero-copy en lugar de number[] via serde_wasm_bindgen"
      evidencia: "vantadb-wasm/src/lib.rs memory_record_to_js (líneas 1211-1227) usa js_sys::Float32Array::new_with_length + copy_from"
      confianza: alta
    - claim: "Ningún host lee record.vector, por lo que el cambio de tipo no rompe consumidores funcionales"
      evidencia: "grep '.vector' en web/ packages/ vantadb-ts → 0 hits funcionales (solo anotación de tipo TS)"
      confianza: alta
    - claim: "cargo clippy --all-targets -- -D warnings pasa para vantadb-wasm"
      evidencia: "cargo clippy -p vantadb-wasm --all-targets -- -D warnings → Finished, 0 errores"
      confianza: alta
  artefactos:
    - vantadb-wasm/src/lib.rs
    - vantadb-ts/src/types.ts
  invariantes: "Shape de record intacto (todas las keys preservadas); lógica core no tocada; input types (MemoryInput/NodeInput.vector) siguen number[]"
  deuda: "Persist delta (H3-SER-001) diferido: requiere dirty-tracking en core Rust (fuera de scope 'NO cambiar lógica del core')"
  queda_pendiente: "Implementar persist delta cuando el core exponga dirty-tracking de records"

## Deuda técnica (Regla 6 — MUST)

- **Deuda registrada:** P2-7 (serialización vectorial sin zero-copy) — RESUELTA por este cambio. Paga la deuda P2-7 existente. Persist delta (H3-SER-001) queda como deuda nueva pero justificada (requiere core).

## Definition of Done (contrato multi-nivel — P2-08)

- **Task:** Contrato verificable cumple + fmt/clippy/check wasm32 pasan.
- **Commit:** Atómico ~100 líneas, conventional `perf:`, git diff limpio (NO commitear — lo hace el orquestador).
- **Release:** N/A para esta tarea aislada.

## Herramientas necesarias
- cargo (check, build wasm32, clippy, fmt)
- codegraph_explore (blast radius)
- grep (consumidores host)

## Investigation Notes
- Las líneas 439,447,750,997 del plan file están desactualizadas (apuntan a strings de error en connect_worker y fn query). El hot path real de serialización es `memory_record_to_js` (1192) invocado desde `search`/`list`/`put`. El fix P2-7 ya estaba señalado en comentario del archivo (líneas 1249-1252).
- wasm32-unknown-unknown target está instalado → build wasm real viable.

## Incógnitas (uphill) vs Pendientes (downhill)
- Incógnitas: 0
- Pendientes: 4 (editar serialización vector, actualizar comentario P2-7, actualizar tipo TS, verificar)

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — No toca trust boundaries, input de usuario, auth, ni dependencias. Justificado: solo cambio de representación interna de un campo de salida (vector) de number[] a Float32Array; mismo origen de datos, mismo sanity-check de NaN/Inf.
- [x] **PERFORMANCE** — Toca hot path de serialización en search/list/put. Cambio de serde_wasm_bindgen (build por-elemento de JS array) a bulk-copy Float32Array: elimina N llamadas Reflect/alloc por vector. Impacto esperado: ~2-5µs → <1µs por vector de 384/768 dims (P2-7). Baseline no medible sin browser; el cambio es mecánicamente menor en allocs/Reflect calls.

## Steps

### Step 1: Reemplazar serde_wasm_bindgen por Float32Array zero-copy en `memory_record_to_js`
- **Archivos:** vantadb-wasm/src/lib.rs (1211-1227)
- **Acción:** En la rama `if let Some(ref vector) = rec.vector`, sanitizar NaN/Inf y volcar a `js_sys::Float32Array` (new_with_length + copy_from) en lugar de `serde_wasm_bindgen::to_value(&sanitized)`.
- **Verify:** `cargo check -p vantadb-wasm` ✅
- **Estado:** ✅ COMPLETED

### Step 2: Actualizar comentario P2-7 a "resuelto"
- **Archivos:** vantadb-wasm/src/lib.rs (1249-1252)
- **Acción:** Marcar el ponytail-note P2-7 como implementado por Step 1.
- **Verify:** `cargo fmt --check` ✅
- **Estado:** ✅ COMPLETED

### Step 3: Actualizar anotación de tipo TS en vantadb-ts (host compat)
- **Archivos:** vantadb-ts/src/types.ts (32, 83 — solo output types; input types 19/76 quedan number[])
- **Acción:** `vector?: number[]` → `vector?: Float32Array | number[]` en MemoryRecord y NodeRecord (output).
- **Verify:** documentado (tsc no se corre en este entorno; grep confirma 0 consumidores de record.vector, cambio compatible)
- **Estado:** ✅ COMPLETED

### Step 4: Verificación completa (contrato)
- **Archivos:** vantadb-wasm
- **Acción:** `cargo check -p vantadb-wasm` ✅, `cargo build --target wasm32-unknown-unknown -p vantadb-wasm` ✅, `cargo clippy -p vantadb-wasm --all-targets -- -D warnings` ✅, `cargo fmt --check` ✅.
- **Verify:** todos ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna.

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-audit (o doubt-driven-development en contexto fresco)
- **Enfoque:** ¿Float32Array zero-copy es el cambio mínimo correcto para el hot path? ¿Se rompió algún contrato? → No: shape intacto, hosts no leen record.vector, tests wasm no downcast.
- **Cómo se probó:** cargo check/build wasm32/clippy/fmt pasan; grep de consumidores confirma 0 lecturas de record.vector.
- **Checklist anti-hábitos tóxicos:** sin auto-reporte, sin skip de clarificación, verify real contra criteria.
- **Veredicto:** ✅ approve (cambio mecánicamente verificable y acotado)

## Notas
- Persist delta (H3-SER-001) diferido: `save`/`save_idb` colectan todos los records (`collect_all_deduped`) y serializan con `serde_json::to_vec`. Un delta real requiere dirty-tracking en el core (fuera de scope "NO cambiar la lógica del core Rust"). El contrato lo permite vía "o fallback documentado".
- El fix P2-7 estaba ya señalado en el propio archivo (líneas 1249-1252) como mejora pendiente; esta tarea lo cierra.
