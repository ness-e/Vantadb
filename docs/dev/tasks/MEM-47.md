# MEM-47 — Semantic recall end-to-end (swap overlap→vector + fallback D38)

## Contexto
P31 Task 5. Paga la deuda #1: recall/dedup/query rankean por similitud semántica cuando
el record tiene vector (MEM-46, commit e22b496a); fallback keyword-overlap para records
sin vector — nunca se rompe un record legacy. Decisión D38 fija el contrato.

## Contrato de validación
- `cargo check -p vanta-memory` exit 0 ✅
- `cargo nextest run -p vanta-memory` 470/470 exit 0 ✅ (465 previos + 5 nuevos D19)
- `cargo fmt --check` exit 0 ✅ · `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` exit 0 ✅
- Tests D19 (`tests/semantic_recall.rs`): (a) paráfrasis sin keywords comunes matchea por vector ✅;
  (b) record sin vector cae a keyword-overlap (paridad con/sin hook) ✅;
  (c) dedup encuentra candidatos semánticos (y legacy path vacío sin hook) ✅;
  (d) scene_query usa vector cuando hay hook (keyword-only no matchea) ✅;
  (e) RecallScope respeta en modo vector (Session excluye cross-session semántico, Agent lo incluye) ✅;
  (f) suite completa sin regresiones ✅.

## Impacto mapeado (Regla 0)
**Archivos leídos completos:** auto_recall.rs (550L), l1_reader.rs (217L), l1_dedup.rs,
l1_writer.rs (EmbedFn MEM-46), knowledge_handlers.rs, abstractions/types.rs (MemoryRecord),
pipeline_worker.rs:380-460, scene_index.rs:156-200, tests/recall.rs, sdk/api.rs usable_vector.
**Referencias entrantes actualizadas:** perform_auto_recall (hooks/mod re-export, pipeline_worker:426,
tests ×8), recall_candidates (l1_dedup:93, record/mod re-export, tests), scene_query (re-export gateway/mod,
5 tests internos), MemoryRecord constructors (~11 sitios).
**Veredicto:** cambios contenidos en vanta-memory; core `vantadb` intocado; wire contract
RecalledMemory.score y SceneQueryHit.score quedan usize (= keyword overlap), cero churn de wire.

## Diseño implementado (D38)
- **Fusión:** dos pools rankeados + RRF local (k=60, Cormack et al.) en `l1_reader::rrf_merge`;
  pool keyword (records sin vector usable, umbral propio del consumidor) ⊕ pool vector
  (records con vector usable, coseno ≥ `MIN_COSINE_SIMILARITY=0.35`). Rank-based → counts y
  cosenos nunca compiten directo.
- **Query embedding:** hook `Option<&EmbedFn>` por parámetro en los 3 consumidores;
  default None → byte-identical pre-MEM-47 (fallback garantiza comportamiento legacy).
- **Modos:** `RecallMode::effective(embeddings_available)` — Keyword siempre legacy;
  Embedding/Hybrid corren dual-pool solo si hubo query embeddeable + pool con vectores.
- **Consumidores:**
  - auto_recall: `perform_auto_recall(db, params, embed)` + `search_records` dual-pool; effective_mode honesto.
  - dedup: `recall_candidates(..., embed)` + `recall_candidate_matches(..., embed)`; batch_dedup pasa `config.embed`.
  - knowledge_handlers: `scene_query(db, req, embed)` — embedding query-time de ambos lados
    (bloques no llevan vector persistido); None → path legacy exacto.
  - pipeline_worker pasa `self.dedup_config.embed.as_ref()` al recall (hook compartido).
- **MemoryRecord.vector:** campo `#[serde(default, skip_serializing_if)] Option<Vec<f32>>`,
  poblado en read paths desde VantaMemoryRecord.vector (usable_vector filtra vacíos/ceros);
  payload L1 sigue sin vector (el vector vive en el nodo).
- **Tests:** fake embedding determinista (tabla de familias sinónimas → vectores base
  ortogonales + FNV/LCG hash 64-dim zero-centered para textos desconocidos).

## Steps
- ✅ S1: MemoryRecord.vector + read paths + constructor sites (~11)
- ✅ S2: helpers shared (cosine_similarity/rrf_merge/MIN_COSINE_SIMILARITY) + recall_candidates swap + dedup plumbing
- ✅ S3: auto_recall swap + callers (pipeline_worker/tests/e2e)
- ✅ S4: scene_query swap + tests internos
- ✅ S5: tests D19 semantic_recall.rs + verify completo

## Estado
✅ COMPLETED — iteración 2 (verify mecánico 4/4 exit 0)

## Notas / deuda conocida
- scene_query embebe cada bloque vivo por query (# ponytail: O(N) embeds; upgrade path =
  persistir vectors de bloques al escribir + HNSW).
- RecalledMemory.score sigue siendo overlap count; hits puramente semánticos reportan 0 ahí
  (documentado en search_records).
- rustc crashes transitorios 0xc0000409 durante verify (3 veces): limpiar deps/fingerprints de
  vanta-memory y reintentar resuelve; NO perseguir Cargo.toml.
