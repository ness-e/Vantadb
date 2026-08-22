# MEM-46 — Embeddings para records L1 (fundación recall semántico)

Plan: `docs/plans/2026-08-22-vanta-final-cierre.md` Task 4 · Ruta: vanta-worker
**Estado:** COMPLETED (sin commit — regla del orquestador)

## Paso 0 (RESUELTO POR LEAD — no repetir)
Core ya tiene `EmbeddingProvider`:
- `src/llm.rs:26` — `pub trait EmbeddingProvider: Send + Sync { fn embed(&self, text:&str) -> Result<Vec<f32>>; }`
- `src/llm.rs:39` — `get_embedding_provider()` (env `VANTA_EMBEDDING_PROVIDER`: openai | ollama default)
- Ya usado por `src/physical_plan/vector.rs` + `src/executor.rs`

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `vanta-memory/src/core/record/l1_writer.rs` (259L) — target principal
- `vanta-memory/src/core/record/l1_dedup.rs` (config + run_l1_dedup)
- `vanta-memory/src/services/pipeline_worker.rs` (run_l1_inner, dedup_config passthrough — sin cambios de firma)
- `vanta-memory/tests/l1_dedup.rs` (422L) — call sites a actualizar
- `vanta-memory/Cargo.toml`
- `vantadb/src/llm.rs` (trait/factory), `vantadb/src/sdk/types.rs` (VantaMemoryRecord.vector), `vantadb/src/sdk/api.rs:428` (`VantaEmbedded::get`)
- `vantadb/src/lib.rs:99` — **hallazgo clave:** `pub mod llm` está tras feature `remote-inference` (=dep:reqwest); vanta-memory usa `default-features = false`

**Referencias entrantes (callers a actualizar):**
- `write_memory`: tests/l1_dedup.rs ×3, tests/generation_log.rs ×1
- `apply_dedup_batch`: l1_dedup.rs:169 (interno), tests/l1_dedup.rs ×1
- `run_l1_dedup`: firma SIN cambios (pasa hook vía L1DedupConfig) → pipeline_worker intacto

**Referencias salientes:** SDK put/get; tracing; core llm (solo tras feature)

**Veredicto de impacto:** bajo — cambios confinados a vanta-memory (crate propio del worker). Core NO se toca (visibilidad de `llm` ya es pública; solo feature-gated). Default features de vanta-memory quedan lean; tests acceden al trait real vía unificación de features en dev-deps.

## Steps

### Step 1 ✅ — Config/hook plumbing
- `EmbedFn = Arc<dyn Fn(&str) -> Option<Vec<f32>> + Send + Sync>` en l1_writer.rs
- `L1DedupConfig.embed: Option<EmbedFn>` (default None = disabled, P4); derive Copy/Debug reemplazado por Clone + Debug manual (embed.is_some())
- `write_memory` / `apply_dedup_batch` reciben `Option<&EmbedFn>`; put_record recibe vector
- Embed best-effort: fallo → tracing::warn + record SIN vector (nunca bloquea)
- Constructor `core_embedding_hook()` tras feature passthrough `embeddings = ["vantadb/remote-inference"]`
- dev-deps: `vantadb = { features = ["remote-inference"] }` para que los tests vean `vantadb::llm` con comandos default

### Step 2 ✅ — Tests D19 (tests/l1_dedup.rs, fakes locales sin red)
(a) FixedEmbedding → record con vector dim 8 consultable vía db.get · (b) FailingEmbedding → record sin vector, sin panic · (c) None → vector-free idéntico al actual · (d) dimensión consistente ×2 records · e2e run_l1_dedup con config.embed → records con vector.
**Hallazgo:** el SDK lee "sin vector" como `Some([])` (`usable_vector` filtra vacíos/ceros antes de indexar; src/sdk/api.rs:24). Assert helper `assert_no_usable_vector` (None-or-empty).

### Step 3 ✅ — Verify mecánico (todos exit 0)
`cargo check -p vanta-memory` ✅ · `cargo nextest run -p vanta-memory` ✅ **465/465** (460 previos + 5 nuevos) · `cargo fmt --check` ✅ · `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` ✅ · extra: `cargo check -p vanta-memory --features embeddings` ✅

## Context Save Point
Tarea COMPLETA sin commit (regla del orquestador). Nota de entorno: rustc crasheó (0xc0000409/E0463) compilando tests con jobs paralelos altos tras agregar reqwest al grafo — `-j 2` lo resolvió (agotamiento de recursos, no código); si reaparece, usar `-j 2`.

## Contrato
"`cargo check -p vanta-memory` pasa; D19 (a)(b)(c)(d) verdes" (multi-namespace vector search queda para MEM-47 recall).

## Invariantes
- P4: fallo de embedding nunca bloquea ni pierde el record
- Default build lean: reqwest NO entra por defecto en deps normales
- Sin unwrap/expect en código de producción
- No tocar core `vantadb`; no commitear; no editar plan file

## Context Save Point
(ninguno aún)
