# PRV-01: OpenAI — Fix pydantic Embedding bug + add get/list

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Blast Radius
- `providers/openai/src/python.rs` — archivo único
- `vantadb-openai` crate — solo usado desde Python vía PyO3
- No afecta a otros providers ni al core

## Contrato
`cargo check -p vantadb-openai` pasa + `get()` y `list()` expuestos en PyO3

## Steps

### Step 1: Fix `cast::<PyDict>()` bug in embed()
- **Archivos:** `providers/openai/src/python.rs`
- **Acción:** Cambiar `item.cast::<PyDict>()?` a `item.get_item("embedding")?.extract::<Vec<f32>>()` — el objeto pydantic Embedding no es PyDict.
- **Verify:** `cargo check -p vantadb-openai`

### Step 2: Add `get(namespace, key)` method
- **Archivos:** `providers/openai/src/python.rs`
- **Acción:** Exponer `VantaEmbedded::get()` — lee un registro por namespace+key.
- **Verify:** `cargo check -p vantadb-openai`

### Step 3: Add `list(namespace, limit, cursor)` method
- **Archivos:** `providers/openai/src/python.rs`
- **Acción:** Exponer `VantaEmbedded::list()` — enumera registros con paginación.
- **Verify:** `cargo check -p vantadb-openai` + verificar que `list()` acepta namespace, limit, cursor

## Dependencias
- Ninguna

## Context Save Point
- **Fecha:** 2026-07-22
- **Branch:** develop
- **Decisiones:** PyO3 extract::<Vec<f32>> directo funciona porque pydantic Embedding implementa `__getitem__` (Mapping protocol)
