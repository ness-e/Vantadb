# PRV-03: Ollama — Add get/list methods

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Blast Radius
- `providers/ollama/src/python.rs` — archivo único
- `vantadb-ollama` crate — solo usado desde Python vía PyO3

## Contrato
`cargo check -p vantadb-ollama` pasa + `get()` y `list()` expuestos

## Steps

### Step 1: Add `get(namespace, key)` method
- **Archivos:** `providers/ollama/src/python.rs`
- **Acción:** Exponer `VantaEmbedded::get()` — lee un registro por namespace+key. Seguir el patrón de OpenAI/litellm.
- **Verify:** `cargo check -p vantadb-ollama`

### Step 2: Add `list(namespace, limit, cursor)` method
- **Archivos:** `providers/ollama/src/python.rs`
- **Acción:** Exponer `VantaEmbedded::list()` — enumera registros con paginación.
- **Verify:** `cargo check -p vantadb-ollama`

## Dependencias
- Ninguna

## Context Save Point
- **Fecha:** 2026-07-22
- **Branch:** develop
