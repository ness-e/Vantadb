# Task API-STD-06 — INDIVIDUAL (5/11) WASM

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes)
> **Fecha:** 2026-09-24

## 1. Objetivo + contrato

Ficha individual WASM: funcionamiento + uso + código + veredicto.

## 2. Funcionamiento

Vista browser/Node-wasm (`wasm-bindgen`, `pkg/`, `opt-level="s"`): 47 fns (`BINDINGS_NAMESPACES.md:79-132`) memoria(15)/graph(11)/system(21) + OPFS/IDB persistencia (`connect_persistent/connect_idb/save/load`). Base de `vantadb-ts` en browser.

## 3. Uso (ejemplo mínimo)

```js
import init, { VantaDb } from "vantadb-wasm/pkg/";
await init();
const db = new VantaDb();
db.put({ namespace: "docs", key: "a", payload: "hello" });
```

## 4. Código (re-verificado 2026-09-24)

- **W1 Frontera errores SANA:** `to_js_err` sistemático en TODOS los cruces (`lib.rs:541,563,601,625,683,701,756,892,1113,1151,1204…` — 30+ sitios): mapea `Error→JsValue`, nunca panic. Es el patrón que Python/Node deben igualar (bloque usuario §C).
- **W2 Emite `score`:** `set("score")` (`:1262` verificado en auditoría previa) — lado sano del bug TS.
- **W3 Formas canónicas presentes:** `put_batch` array-objetos (`:1157`), `search_multi` (`:1482`), `bulk_import(_bytes)` (`:1555,1564`), `import_records` real del core (`:701,1064,1095` — el que TS evita por FIND-79), `graph_degree` (`:1896`), u128 strings decimales (`:1769`).
- **W4 P2-8 CONFIRMADO (deuda viva):** `collect_all_deduped()` O(n) (`:564-596`; Regla 6 `AGENTS.md` — 🟡 2-4h, moneda de pago).
- **W5 OPFS solo-WASM** (`:585,618,944,973,1006-1035` — persistencia browser, no portable por diseño).

## 5. Veredicto + implicaciones

WASM es el binding más completo y con mejor frontera de errores. Propuestas 15: (a) pagar P2-8 con P2-5; (b) tomar `to_js_err`+code como estándar cross-binding; (c) arreglar `import_records` para shapes `MemoryInput`/u64-string (desbloquea T3 de TS). Blast radius: `vantadb-wasm/*` + `vantadb-ts` + `pkg/`.

## 6. DoD

- [x] Contrato ✅ · Task file sync · Recitation: WASM ficha completa

## Context Save Point

API-STD-06 DONE. Next: API-STD-07 (HTTP+OpenAPI). Deuda: P2-8 viva. WIP: ninguno.
