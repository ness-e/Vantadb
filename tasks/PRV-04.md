# PRV-04: All providers — search(text_query, filters, distance_metric)

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Steps

### Step 1: Openai — agregar text_query, filters, distance_metric params
- **Archivos:** `providers/openai/src/python.rs`
- **Acción:** El método `search()` actual usa `query_embedding: Vec<f32>` solamente. Agregar parámetros opcionales `text_query: Option<String>`, `filters: Option<HashMap<String, String>>`, `distance_metric: Option<String>`.
- **Verify:** `cargo check -p vantadb-openai`

### Step 2: Ollama — igual que openai
- **Archivos:** `providers/ollama/src/python.rs`
- **Verify:** `cargo check -p vantadb-ollama`

### Step 3: Litellm — igual que openai
- **Archivos:** `providers/litellm/src/python.rs`
- **Verify:** `cargo check -p vantadb-litellm`
