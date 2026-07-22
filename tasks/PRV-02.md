# PRV-02: LiteLLM — Fix embed→embedding + cache + get/list

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Blast Radius
- `providers/litellm/src/python.rs` — archivo único
- `vantadb-litellm` crate — solo usado desde Python vía PyO3
- No afecta a otros providers

## Contrato
`cargo check -p vantadb-litellm` pasa + `get()` y `list()` expuestos

## Steps

### Step 1: Fix `embed` → `embedding` function name
- **Archivos:** `providers/litellm/src/python.rs`
- **Acción:** Cambiar `.getattr("embed")` a `.getattr("embedding")` — según docs oficiales de LiteLLM.
- **Verify:** `cargo check -p vantadb-litellm`

### Step 2: Cache litellm module / embedding function
- **Archivos:** `providers/litellm/src/python.rs`
- **Acción:** En lugar de `PyModule::import(py, "litellm")` en cada llamada a `embed()`, cachear la función importada en `self.embed_fn: Option<Py<PyAny>>`.
- **Verify:** `cargo check -p vantadb-litellm`

### Step 3: Add `get(namespace, key)` method
- **Archivos:** `providers/litellm/src/python.rs`
- **Acción:** Exponer `VantaEmbedded::get()`.
- **Verify:** `cargo check -p vantadb-litellm`

### Step 4: Add `list(namespace, limit, cursor)` method
- **Archivos:** `providers/litellm/src/python.rs`
- **Acción:** Exponer `VantaEmbedded::list()`.
- **Verify:** `cargo check -p vantadb-litellm`

## Dependencias
- Ninguna

## Context Save Point
- **Fecha:** 2026-07-22
- **Branch:** develop
- **Decisiones:** Cachear `embedding` function en struct field en vez de re-importar módulo en cada llamada
