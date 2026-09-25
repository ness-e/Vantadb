# TECH-02 — Fix wrapper TS `reindexHnswFromText`

**Estado:** ✅ COMPLETED
**Commit:** `274edcf9` (develop)
**Fecha:** 2026-08-05

## Qué se hizo

`vantadb-ts/src/vantadb.ts:542-548` — el wrapper `reindexHnswFromText` tenía un comentario stale y lanzaba `VantaError("WASM_ERROR", ...)` afirmando que `reindex_hnsw_from_text` no estaba exportado por el pkg WASM. La premisa era falsa: el pkg SÍ lo exporta (`vantadb-wasm/pkg/vantadb_wasm.d.ts:183`, también presente en `vantadb-ts/node_modules/vantadb-wasm/vantadb_wasm.d.ts:183`).

**Fix (1-línea):**
```ts
reindexHnswFromText(namespace: string, pageSize: number = 1000): unknown {
  this._assertOpen();
  return this._wasm("reindexHnswFromText", () => this.inner.reindex_hnsw_from_text(namespace, pageSize));
}
```
Mismo patrón que `rebuildIndex`/`exportAll`/etc. Dif: +1 / -4. Sin rebuild/publish de pkg (como requería la tarea).

## Verification

- ✅ d.ts firma: `reindex_hnsw_from_text(namespace: string, page_size?: number | null): any` — matchea `(namespace, pageSize)`.
- ✅ `npx tsc --noEmit`: solo el error PRE-EXISTENTE `TS2306` en `vantadb-ts/src/vantadb.ts(1,40)` (import de `vantadb-wasm` no resuelve como módulo). Cero errores en el método editado → tipo correcto. Confirmado con `git stash`: el error existe igual en HEAD.
- ⚠️ `npm test` (vitest): FALLA igual en HEAD — pre-existente, ambiental. 5 suites fallan al importar `vantadb-wasm` porque su `package.json` (solo `browser`/`types`, sin `main`/`module`) no resuelve entry en node env de vitest. FUERA DE SCOPE: requiere arreglar packaging del pkg WASM (otra tarea). Contract permitía fallback compile+signature.

## Notas / decisiones

- `VantaError` sigue usado en `_assertOpen` y otros métodos → import no quedó huérfano.
- El default `pageSize = 1000` coincide con `unwrap_or(1000).max(1).min(1000)` del core Rust.
- Tarea pendiente real (no esta tarea): `vantadb-wasm/package.json` necesita `main`/`module` apuntando a `pkg/vantadb_wasm.js` para que vitest resuelva el import — o usar alias en `vitest.config.ts`.
