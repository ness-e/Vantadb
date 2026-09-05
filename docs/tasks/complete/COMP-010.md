# COMP-010: Auto-embedding function abstraction

**Tipo:** Rust core — refactor
**Esfuerzo:** 🟡 1-2 sem → estimado real ~4-6h
**Dependencia:** DRV-123 ✅ (completada)

## Objetivo

Refactorizar el auto-embedding de `LlmClient` concreto (hardcodeado a Ollama) a un trait `EmbeddingProvider` abstracto con 2 implementaciones: Ollama + OpenAI.

## What exists (no tocar)

- `LlmClient` en `src/llm.rs` — tiene `generate_embedding()` (Ollama) y `summarize_context()` (Ollama generate)
- 4 call sites que usan `LlmClient::new().generate_embedding(text)`:
  - `src/executor.rs:237-238` (INSERT node_id)
  - `src/executor.rs:369-370` (INSERT InsertMessage)
  - `src/physical_plan.rs:235-236` (query vector refine)
  - `src/physical_plan.rs:746-747` (query vector refine)
- El módulo está compilado condicionalmente con `#[cfg(feature = "remote-inference")]` en `src/lib.rs:91`
- `summarize_context()` es generación de texto **no** embedding — mantener en `LlmClient`

## Plan de implementación

### Task 1: Trait + OllamaProvider + factory

1. En `src/llm.rs`, definir el trait:
   ```rust
   /// Abstract embedding provider.
   pub trait EmbeddingProvider: Send + Sync {
       fn embed(&self, text: &str) -> Result<Vec<f32>>;
   }
   ```

2. Refactorizar `LlmClient::generate_embedding()` → `OllamaProvider` implementando el trait
   ```rust
   pub struct OllamaProvider { ... }
   impl EmbeddingProvider for OllamaProvider { ... }
   ```

3. Mantener `LlmClient` solo con `summarize_context()` (generación de texto, no embedding)

4. Agregar factory function:
   ```rust
   /// Lee VANTA_EMBEDDING_PROVIDER (ollama|openai) y devuelve el provider.
   pub fn get_embedding_provider() -> Box<dyn EmbeddingProvider> { ... }
   ```

5. Actualizar los 4 call sites:
   ```
   - let llm = crate::llm::LlmClient::new();
   - match llm.generate_embedding(text) {
   + let provider = crate::llm::get_embedding_provider();
   + match provider.embed(text) {
   ```

6. `cargo check -p vantadb --features remote-inference` ✅

### Task 2: OpenAIProvider

1. Agregar `OpenAIProvider` en `src/llm.rs`:
   ```rust
   pub struct OpenAIProvider { ... }
   impl EmbeddingProvider for OpenAIProvider { ... }
   ```
   - URL: `https://api.openai.com/v1/embeddings`
   - Auth: `Authorization: Bearer {VANTA_OPENAI_API_KEY}`
   - Modelo: `text-embedding-3-small` (o `VANTA_OPENAI_MODEL`)
   - Feature: mismo `remote-inference` (reusa `dep:reqwest`)

2. Conectar en `get_embedding_provider()`:
   ```rust
   match env::var("VANTA_EMBEDDING_PROVIDER").as_deref() {
       Ok("openai") => Box::new(OpenAIProvider::new()),
       _ => Box::new(OllamaProvider::new()), // default
   }
   ```

3. `cargo check -p vantadb --features remote-inference` ✅

### Task 3: Verificación

- `cargo check -p vantadb --features remote-inference` ✅
- `cargo test --package vantadb --lib llm` ✅ (si existen tests)
- `cargo test --package vantadb --lib executor` ✅ (7 tests auto-embedding)
- `cargo fmt --check` ✅

## Constraints
- **Ponytail mode full:** mínimo código, nada especulativo
- `summarize_context()` NO refactorizar — es otro concern (text generation)
- No agregar nuevas dependencies externas (OpenAI usa `dep:reqwest` existente)
- No romper API pública existente
