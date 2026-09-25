# Task API-STD-03 — INDIVIDUAL (2/11) Python SDK

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes)
> **Fecha:** 2026-09-24

## 1. Objetivo + contrato

Ficha individual Python: funcionamiento + uso + código + veredicto por fallo.

## 2. Funcionamiento

Vista PyO3 (`*.abi3.so`, import nativo): 44 métodos pyclass + `connect()` módulo (`lib.rs:2376`). Dominios: memory (put/get/list/search + TTL/supersede/snippets), graph (insert/get/delete por `id:u128` + traversals + **solo-Python** `graph_page_rank`), wiki (`recover_archived_nodes`), system (IQL, flush/compact/rebuild, import/export, `hardware_profile`). Sub-clientes `db.memory/graph/system/wiki` vía macro `forward_to_db!` (`:545-565` incluye `hardware_profile, bulk_import(_bytes), recover_archived_nodes`).

## 3. Uso (ejemplo mínimo)

```python
import vantadb
db = vantadb.connect("/tmp/vanta.db")
db.memory.put("docs", "k1", "payload", vector=[...])
hits = db.memory.search("docs", query_vector=[...])
```

## 4. Código (re-verificado 2026-09-24 por grep directo, 27 matches)

- **P1 CONFIRMADO + MATIZADO — doble `get/delete`:** memory `get(namespace,key)` (`:402`) y `delete(namespace,key)` (`:442`) en sub-cliente vs flat nodo `get(id:u128)` (`:1563`) y `delete(id,reason)` (`:1577`). El hazard existe pero hay vía correcta (`db.memory.*`); el flat conserva semántica opuesta a TS.
- **P2 `put_batch` columnar CONFIRMADO:** `put_batch` (`:702`) + `put_batch_raw` solo-Python (`:811`) vs array-objetos WASM/TS.
- **P3 `search` híbrido CONFIRMADO:** `search` (`:1243`) + `search_vector` puro (`:1592`) + `search_batch(_requests)` solo-Python (`:1625,1681`).
- **P4 sin `search_multi` CONFIRMADO:** cero matches en todo `lib.rs` (vs WASM/TS/Node que sí).
- **P5 sin `export_namespace_filtered/import_records/audit_deep` CONFIRMADO:** por ausencia en los 27 matches (solo `bulk_import(:1473)/_bytes(:1483)`, audit shallow).
- **P6 solo-Python CONFIRMADO:** `hardware_profile` (`:1822`), `recover_archived_nodes` (`:2123`), `query_structured`, `graph_page_rank` (sin match `fn` directo = delegado por macro `:545-565`).
- **P7 P2-5 dual-API `put_batch`:** deuda viva (Regla 6, ~53 líneas branching legacy).

## 5. Veredicto + implicaciones

7/7 confirmados (P1 matizado: el sub-cliente ya es la vía correcta; falta deprecar/renombrar el flat nodo o documentar). Propuestas 15: firma canónica objeto vs columnar (medir perf antes, Regla 9), añadir `search_multi`, unificar export/import/audit, `u128` node-ids como string en wire JSON, pagar P2-5. Blast radius: `tests/api/python.rs`, `sdk_serialization`, READMEs.

## 6. DoD

- [x] Contrato ✅ · Task file sync · Recitation: Python ficha completa, 7/7 (1 matizado)

## Context Save Point

API-STD-03 DONE. Next: API-STD-04 (TS). Deuda: P2-5 viva. WIP: ninguno.
