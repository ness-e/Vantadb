# NUEVO-16 — Product Quantization (roadmap)

> **Status:** ✅ COMPLETED 2026-08-05
> **Plan:** `docs/plans/2026-08-05-backlog-validation-actions.md` → Task 48
> **Document:** `docs/architecture/PQ_FEASIBILITY.md`

## Result

Scoping técnico + investigación de corpus + doc de viabilidad. **Decisión: @defer**
(mantiene REC-009 2026-07-31). Entregable de REC-009 (`docs/research/pq-feasibility.md`)
nunca se creó — este doc lo reemplaza como fuente de viabilidad.

## Key findings

- `src/vector/quantization.rs`: RaBitQ (1-bit u64), TurboQuant (4-bit nibble), SQ8
  (8-bit i8) — funciones stateless SIN codebooks. Sin variante PQ en
  `VectorRepresentations` (Binary/Turbo/SQ8/Full/MmapFull/None).
- `src/index/scann.rs`: único índice cuantizado (SQ8 per-dim min/max + re-rank f32);
  nota explícita `// no anisotropic quantization, no PQ, no GPU`.
- PQ teórico verificado multi-fuente: split en M subespacios, codebooks k-means
  independientes por subespacio, D×32 bits → M×log2(K) bits, ADC (asymmetric
  distance) con lookup tables (Jégou 2011 TPAMI).
- Corpus objetivo (citado, corpus-texmex + ann-benchmarks): SIFT1M ~512MB f32,
  GIST1M ~3.84GB, SIFT1B ~512GB f32 (92GB u8) → PQ M=16/8b = 16GB; GloVe-100
  473MB. El gap >RAM real es SIFT1B-class.
- Pinecone usa IVF + PQFS en slabs >1M (INV-019) = techo competitivo.
- Encaje: `VectorRepresentations::PQ` + `pq_quantize/pq_similarity` ADC + extender
  ScannIndex o `IndexType::Pq`; trainer k-means = dominio vanta-engine.

## Decisión @defer (rationale)

1. Sin demanda demostrada — benchmarks actuales (GloVe-100, SIFT1M subsets) caben
   en RAM / cubiertos por mmap tiering + SQ8.
2. SQ8 + tiering mmap (L0–L3) ya maneja "excede RAM" a nivel storage.
3. PQ = hot-path de vanta-engine (k-means, ADC, SIMD sub-vector), no de worker.
4. Costo de recall (~0.95 vs SQ8 ~0.985) no se paga si el dataset no excede RAM.

**Promotores para @appr:** workload validado >RAM (GIST1M/SIFT1B/Glove-300 >1-2GB),
bench competitivo vs Pinecone PQFS/Milvus IVF-PQ, o pedido explícito de arch/engine.

## Deliverables

- `docs/architecture/PQ_FEASIBILITY.md` (decisión @defer + plan de fases
  condicional P1–P4 + corpus citado + non-goals).
- Commit: `docs(NUEVO-16): viabilidad Product Quantization (update REC-009)`

## Recitation

- **Objetivo activo:** Ejecutar plan backlog-validation (Task 48 NUEVO-16).
- **Última acción:** Doc de viabilidad PQ publicado (@defer) con corpus citado.
- **Resultado:** ✅
- **Próxima acción:** Task 49 (NUEVO-22 sparse indexed) — o, si se gatilla un
  promotor del Sec 5, re-evaluar PQ con vanta-engine.
