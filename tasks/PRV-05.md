# PRV-05: All providers — timeout + counter fix + list_namespaces

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Steps

### Step 1: Openai — timeout configurable + counter UUID + list_namespaces
- **Archivos:** `providers/openai/src/python.rs`
- **Acción:** 
  - Agregar `timeout: Option<f64>` como parámetro de constructor y pasarlo al cliente OpenAI
  - Cambiar counter `AtomicU64` a UUID/timestamp para evitar colisiones multi-instancia
  - Agregar `list_namespaces()` que expone `VantaEmbedded::list_namespaces()`
- **Verify:** `cargo check -p vantadb-openai`

### Step 2: Ollama — timeout + counter + list_namespaces
- **Archivos:** `providers/ollama/src/python.rs`
- **Acción:** Mismos cambios
- **Verify:** `cargo check -p vantadb-ollama`

### Step 3: Litellm — timeout + counter + list_namespaces
- **Archivos:** `providers/litellm/src/python.rs`
- **Acción:** Mismos cambios
- **Verify:** `cargo check -p vantadb-litellm`
