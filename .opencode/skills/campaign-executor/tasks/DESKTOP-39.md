# DESKTOP-39: Ingest con embedding desde texto — verificar src/llm.rs y decidir

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24T00:00:00
- **Estado:** ✅ COMPLETED (2026-08-24) — WONTFIX-UI, decisión Caso B documentada

## Impacto mapeado (Regla 0)
- Leídos completos: `src/llm.rs` (358L), `desktop/src/components/IngestForm.tsx`, `desktop/src/vanta.ts` (grep embeddings), `Cargo.toml` (feature gate)
- Referencias hacia dentro: IngestForm → `ingest`/`get`/`vantaErrorMessage` de vanta.ts; NativeConnection.put pasa `input.vector = item.embedding` (`native.rs:229`)
- Referencias entrantes: IngestForm usado por WorkspaceShell; ImportDrop NO menciona embeddings (grep desktop/src: solo vanta.ts) → fuera de alcance
- Veredicto: cambio UI-only en 1 archivo; src/ intocable (fuera de dominio worker para este task)

## Decisión documentada — CASO B: embedding requiere modelo externo (WONTFIX-UI)
**Evidencia del código:**
1. `src/llm.rs:1-4` — doc de módulo: *"Embedding generation and LLM runtime behavior remain external or experimental; the core stores and retrieves provided vectors."*
2. `src/llm.rs:26-29` — `EmbeddingProvider::embed(&self, text) -> Result<Vec<f32>>` existe pero es **trait abstracto**, sin implementación local.
3. `src/llm.rs:39-47` — factory `get_embedding_provider()`: solo dos providers → Ollama (HTTP a `VANTA_LLM_URL`, default `http://localhost:11434`, modelo `all-minilm`) u OpenAI.
4. `src/llm.rs:144-145` — `OpenAIProvider::new()` hace `expect("VANTA_OPENAI_API_KEY must be set")` — panic sin API key.
5. `Cargo.toml:107` — módulo completo tras feature opt-in `remote-inference = ["dep:reqwest"]`.
6. Desktop ya soporta vector opcional: `IngestItem.embedding?: number[]` (`desktop/src/vanta.ts:30-35`) — pero ningún componente lo puebla.

**Conclusión:** no existe path de embedding local sin servicio externo. Un botón "Generar vector" fallaría siempre que no haya Ollama corriendo o API key configurada → se documenta el límite honestamente en UI y se cierra como WONTFIX-UI.

**Implementado (Caso B):** nota informativa bajo el botón de IngestForm (`IngestForm.tsx`) explicando que sin vector el registro se guarda como texto, y que la búsqueda semántica requiere proveedor externo vía `VANTA_EMBEDDING_PROVIDER` / `VANTA_LLM_URL` / `VANTA_OPENAI_API_KEY`. Sin botón deshabilitado (no hay config detectable desde la UI sin nuevo comando Tauri — YAGNI).

## Blast Radius
Callers: desktop/src/components/IngestForm.tsx, desktop/src/components/ImportDrop.tsx
Callees: src/llm.rs (embeddings), src/sdk/api.rs (put con vector), desktop/src/vanta.ts
Implicaciones: Botón "generar vector" en IngestForm/ImportDrop si existe path sin LLM externo; si requiere modelo externo, documentar límite y cerrar WONTFIX-UI

## Spec
N/A — feature condicional con contrato mecánico

## Contrato
`cargo check -p vantadb`; decidir y ejecutar según verificación (no inventar claim de embedding local)

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: Verificar src/llm.rs — API pública y modelo requerido ✅
- Verificado: ver sección "Decisión documentada" arriba (evidencia file:line).

### Step 2: Decidir camino según verificación ✅
- **CASO B** — embedding requiere modelo externo (Ollama server o OpenAI API key). Cerrado WONTFIX-UI con decisión documentada.

### Step 3: Implementar según decisión ✅
- Nota informativa en `desktop/src/components/IngestForm.tsx` (tooltip + texto visible). ImportDrop fuera de alcance (no maneja embeddings).
- Verify: `cd desktop && npm run build` + `cd desktop && npm test`

## Dependencias
- src/llm.rs (verificar primero)
- DESKTOP-10 (put bridge) — ya completada

## Notas
- DoD: decidir y ejecutar según verificación (no inventar claim de embedding local)
- Verificar primero qué expone el core — no asumir
- Si requiere modelo externo: documentar límite honestamente