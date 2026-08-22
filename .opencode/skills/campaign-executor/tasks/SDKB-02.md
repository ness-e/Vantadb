# SDKB-02 — Sub-clientes TypeScript

## Estado
- Plan: `docs/plans/2026-08-22-vantadb-bindings-sdk.md` Task 2
- Mapa canon: `docs/api/BINDINGS_NAMESPACES.md` (§ Sub-Client Design v1)
- Decisiones: D42 (solo capa TS), D43 (v1 agrupa métodos ya expuestos; nada nuevo)

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `vantadb-ts/src/vantadb.ts` (1021L) — clase plana única `VantaDB`, 38 métodos públicos, `_wasm()` error boundary, `_assertOpen()`, constructor privado + factories estáticos
- `vantadb-ts/src/__tests__/vanta.test.ts` (803L) — suite vitest existente (backward-compat = pasa sin cambios)
- `docs/api/BINDINGS_NAMESPACES.md` (238L) — mapa canon de dominios
- `vantadb-ts/package.json` — test runner: `vitest run`; build: `tsc`

**Referencias hacia dentro (imports de vantadb.ts):**
- `vantadb-wasm` (pkg WASM), `./errors.js`, `./guards.js`, `./types.js`
- Re-exportado por entry point del paquete (`dist/vantadb.js`)

**Referencias entrantes (quién usa VantaDB):**
- Tests: `__tests__/*.test.ts`, `integration.test.ts`
- Consumers externos vía npm package exports (`.` y `./types`)
- `native.ts` implementa subset sync — NO se toca

**Veredicto de impacto:** cambio ADDITIVO en una sola clase. Getters nuevos (`memory`, `graph`, `wiki`, `system`) no colisionan con los 38 métodos planos existentes. Suite existente intacta. Riesgo principal: this-binding en getters → resuelto con arrow functions que capturan `this` (patrón canon del mapa).

## Steps
1. ✅ Implementar getters sub-cliente en `vantadb.ts` (memory 12 / graph 10 / wiki 0 / system 16) — lazy frozen delegantes con arrows
2. ✅ Tests nuevos `__tests__/subclients.test.ts` (17 tests) — identidad delegación, frozen, this-binding (`bfs.call` destructurado), CLOSED propagación, wiki vacío, supersede deferido documentado
3. ✅ Verify mecánico: `npm test` 246/246 ✅ · `npx tsc --noEmit` ✅

## Deuda colateral descubierta (fuera de scope D42 → Backlog)
1. **pkg WASM roto bajo Node ≥22:** el snippet inline de `idb.rs` es una IIFE sin export pero el binario importa `__vanta_ensure_idb_bridge` desde él → LinkError al instanciar. Fallaba en HEAD para TODA la suite TS. Parche local aplicado a `vantadb-wasm/pkg/snippets/.../inline0.js` (artefacto gitignored) agregando `export function __vanta_ensure_idb_bridge() {}`. Fix real: exportar la función en el snippet Rust + regenerar pkg (dueño: Arch).
2. **Drift types.ts ↔ pkg:** `graph_topological_sort` devuelve array plano; `types.ts` declara `{sorted, has_cycle}`. El test de sub-cliente no depende del shape (delegación pura). Dueño: flat API / Arch.

## Contrato
"`npm test` pasa (suite existente intacta = backward-compat); tests nuevos delegan al método plano idéntico (mismo resultado, misma firma)"

## Stop condition
Si un sub-cliente requiere lógica nueva (no solo delegación) → fuera de v1, documentar.
