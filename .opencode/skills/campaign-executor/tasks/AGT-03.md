# AGT-03 — Spot-check refs deuda P2 (Regla 6)

- **Plan:** `docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md` (Task 11)
- **Estado:** ✅ COMPLETED
- **Contrato:** refs P2 verificadas contra código; actualizadas o migradas
- **Rol:** vanta-docs (verificación de docs, NO código)

## Objetivo

Verificar vigencia de las refs `file:line` de la tabla de deuda P2 (Regla 6) en
`.opencode/AGENTS.md` (líneas 474-480). El código evolucionó desde su registro.

## Impacto mapeado (Regla 0)

- **Archivo a editar:** `.opencode/AGENTS.md` — solo la tabla de deuda P2 (Regla 6, líneas 474-480).
- **Leído completo:** sección Regla 6 (líneas 470-481) + referencias a P2 en Regla 4 (432-437).
- **Referencias hacia dentro (a la tabla):** ninguna otra sección apunta a filas P2 específicas
  más que la propia tabla y menciones genéricas en Regla 4 (432-437) que siguen siendo válidas
  (P2-3/P2-8/P2-6/P2-5/P2-7 como "deuda histórica" — texto no `file:line`, no requiere cambio).
- **Referencias salientes (de la tabla):** rutas de código verificadas una a una (ver Verificación).
- **Veredicto de impacto:** edición acotada a la tabla. No borra la tabla (Regla 0) — solo
  actualiza refs y marca filas resueltas. No afecta otras secciones ni código.

## DISCOVERY — verificación por ref (comandos reales)

| ID | Ref registrada | Verificación | Veredicto |
|----|----------------|--------------|-----------|
| P2-1 | `vantadb-wasm/src/opfs.rs:83-87` (`delete()` stub) | `grep "fn delete" vantadb-wasm/src/opfs.rs` → línea 101: `pub async fn delete(&self) -> Result<bool, JsValue>` implementado con `js_call(&self.handle, "remove", ...)` (101-104). No es stub. | ✅ RESUELTO |
| P2-3 | `vantadb-python/src/convert.rs:23-70` (LRU O(n) `min_by_key`) | `grep "min_by_key\|LRU\|lru" convert.rs` → línea 47 `lru::LruCache`, comentario 699-700: "O(1) eviction ... (AUD-039) — the old hand-rolled cache scanned with min_by_key (O(n))". | ✅ RESUELTO |
| P2-5 | `vantadb-python/src/lib.rs` ~312 (dual API `put_batch`) | `grep "fn put_batch" lib.rs` → línea 494. Body 507-560 = branch legacy tuplas (deprecado), 562+ = kwargs. Dual API sigue vigente. | ✅ VIGENTE — ref movida 312→494 |
| P2-6 | `vantadb-python/src/types.rs:365` (match no exhaustivo `VantaError`) | `grep "VantaError\|match " types.rs` → 365 sin match de VantaError. `map_vanta_error` (convert.rs:786-818) tiene catch-all `_ =>` (818) + jerarquía de excepciones MOD-20. | ✅ RESUELTO |
| P2-7 | `src/sdk/serialization/mod.rs:227-294` (serialización completa sin zero-copy) | `read mod.rs:220-299` → región ahora es payload index keys + `sparse_vector_to_field/from_field` (encoding tipado por campo). Comentarios `AUD-023 (P2-7)` en mod.rs:298 y 1620. | ✅ RESUELTO (refactor AUD-023) |
| P2-8 | `vantadb-wasm/src/lib.rs:402-433` (`collect_all_deduped()` O(n)) | `grep "collect_all_deduped" lib.rs` → línea 564 (`fn collect_all_deduped`). Dedup por `node_id` u128 (zero alloc), guard `MAX_RECORDS`; sigue juntando todo en memoria (O(n)). | ✅ VIGENTE — ref movida 402-433→564-596 |

## Decisión (ponytail)

**Mantener la tabla** y actualizarla en `.opencode/AGENTS.md` (no migrar a issues):
- Solo 2 filas quedan vigentes (P2-5, P2-8) — migrar a issues por 2 items sería más trabajo.
- 4 filas se marcan resueltas siguiendo el patrón ya usado por P2-2 (strikethrough + ✅ RESUELTO + ref).
- Edición pequeña y acotada a la tabla; sin drift recurrente que justifique migrar.

## ACT

Actualizada tabla Regla 6 (`.opencode/AGENTS.md:474-480`):
- P2-1, P2-3, P2-6, P2-7 → marcadas ✅ RESUELTO (strikethrough, patrón P2-2).
- P2-5 → ref actualizada a `lib.rs:494` (dual API sigue).
- P2-8 → ref actualizada a `lib.rs:564-596` (O(n) sigue).

## VERIFY

- `rg -n "P2-1|P2-3|P2-5|P2-6|P2-7|P2-8" .opencode/AGENTS.md` → tabla refleja estado nuevo; Regla 4 (432-437) sin `file:line` stale.
- Cada ref vigente apunta al código real:
  - P2-5 → `vantadb-python/src/lib.rs:494` (`fn put_batch`) ✅
  - P2-8 → `vantadb-wasm/src/lib.rs:564` (`fn collect_all_deduped`) ✅
- Sin cambios de código (tarea docs); `cargo check` no aplica.

## Notas

- No se migró a issues: solo 2 items vigentes; mantener tabla es más simple.
- Tarea de docs: NO se commitea (el lead verifica mecánico y commitea por tarea).
