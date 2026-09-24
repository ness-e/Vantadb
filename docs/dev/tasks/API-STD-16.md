# Task API-STD-16 — Re-validación fallo-por-fallo + clases nuevas

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes — pasada fresca 2026-09-24, distinto contexto que la auditoría inicial)
> **Fecha:** 2026-09-24

## 1. Contrato

Tabla ~40 fallos → veredicto (✅ persiste tal cual / ✅ ya-no-existe / FIND-*) + búsqueda clases nuevas.

## 2. Veredictos bindings (15)

| Fallo | Veredicto 2026-09-24 |
|---|---|
| Py `get/delete` nodo `:1563,1577` vs memory `:402,442` | ✅ persiste (+MATIZ nuevo: sub-cliente `db.memory.*` ya es vía correcta) |
| `score→distance` TS `:609,663,742` | ✅ persiste (3 sitios, Gate P: migrar a `score`) |
| `put_batch` columnar Py `:702` | ✅ persiste (Gate P: array-objetos) |
| TS traversals `number[]` `:90-107,1328` | ✅ persiste (+NUEVO: `native.ts:315` napi tampoco acepta >2^53) |
| TS `importRecords` bucle `:900-926` | ✅ persiste (FIND-79 documenta causa) |
| Node mínimo `index.d.ts:243-447` | ✅ persiste |
| Node sin sparse-escritura / sin `exclude_superseded` | ✅ persiste |
| `searchWithMethod` separado `:440` | ✅ persiste |
| Doc `supersede` Python-only `:215` | ✅ deriva persiste (existe WASM `:1445`+TS) |
| Doc Py search pure-ANN `:75` | ✅ deriva persiste (es híbrido `:1243`) |
| `graph_degree` 3 nombres | ✅ persiste |
| Triple nombre node-insert | ✅ persiste |
| Node `node_id: string` `:63-64` | ✅ SANO (modelo a copiar) |
| Py `search_multi` ausente | ✅ persiste |
| Py sin `export_filtered/import_records/audit_deep` | ✅ persiste |

## 3. Veredictos red + MCP + core + IQL + CLI + proxy + memory (25)

| Fallo | Veredicto |
|---|---|
| Verbos URL `router:197-207,218-219,244-247` | ✅ persiste |
| Sin versionar | ✅ persiste MATIZADO: health/metrics tienen gemelos v2 (`:155,233`); sin gemelo: conversation/skill/dashboard |
| `201` impl ×6 vs YAML `200` | ✅ persiste (`:252,263,1213,1322,1433,1521`) |
| `POST /threads/{id}` mensaje | ✅ persiste |
| Paginación mixta + `versions` sin paginar | ✅ persiste (Gate P: cursor) |
| YAML drifts (RecordInput/Lisp/enum/cursor/tag) | ✅ persiste (5) |
| MCP alias doble `:309/:373→:1691,1695` | ✅ persiste |
| MCP doble listado `:218/:703` | ✅ persiste |
| `thread_id:number` `:631` | ✅ persiste MATIZADO: mensaje mitigado AUD-050 (`:1970-1973`), tipo pendiente |
| `bulk_import_stream` bypass `:991` | ✅ by-design documentado |
| `query_iql` crudo `:1641` | ✅ persiste |
| Errores string vs tipados | ✅ persiste |
| `VantaHeader` stutter / `Generic` catch-all / `FilterOp` sin docs / `Write.node_id` sin serde | ✅ persisten (4) |
| IQL 6-vs-7 / 3 lecturas / orden-alt / `==` / int→float / UPPERCASE / sin VERSION | ✅ persisten (7) |
| CLI flags / `--json` parcial / cajas+truncado / concerns | ✅ persisten (4, C2 matizado: `--json` existe `:91,192,222,257,273`) |
| Proxy `/snapshot` sin auth `:816-840` / verbo / camel / self-loop / ttl / rate-unused | ✅ persisten (6; X1 seguridad) |
| Memory D37/D21/MEM-16 | ✅ persisten; V1 degradación = diseño sano |

## 4. Clases nuevas buscadas (fresco, no listadas antes)

- **Panics en frontera:** Py `src/`: **0** `unwrap/expect` ✅ limpio. WASM `:1998-2038`: solo bajo `#[cfg(all(test, wasm32))]` (`:1990`) ✅ permitido.
- **NAPI `u128`:** `native.ts:315` "napi only takes numbers" → **FIND-NEW-01**: tercer sitio >2^53 (para 18-W1).
- **MCP tools count:** `opencode.jsonc:87` dice 87 tools vs `AGENTS.md` 15 → **FIND-NEW-02**: deriva doc (para 17).
- **Qdrant/industria:** confirma decisiones 15 (score, objetos, cursor) — sin contradicciones.

## 5. DoD

- [x] Contrato ✅ (40 veredictos + 2 FIND-NEW) · Task file sync · Recitation: nada grave nuevo; 2 hallazgos menores registrados

## Context Save Point

API-STD-16 DONE. Next: API-STD-17 (docs/updates). FIND-NEW-01/02 → 17/18.
