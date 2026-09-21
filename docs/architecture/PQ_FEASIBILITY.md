# PQ Feasibility — Product Quantization for > RAM Datasets

> **Status:** DECISION DOCUMENTED — **@defer**
> **Owner:** vanta-worker (scoping) / vanta-engine (algorithm)
> **Date:** 2026-08-05
> **Backlog refs:** `NUEVO-16` · `REC-009` (2026-07-31, "viabilidad deferida")
> **Plan ref:** Task 48 of `docs/plans/2026-08-05-backlog-validation-actions.md`
> **Fulfills:** the `docs/research/pq-feasibility.md` deliverable declared by REC-009,
> which was never created. This document supersedes it as the feasibility source.

---

## 1. Where quantization lives today

`src/vector/quantization.rs` exposes **stateless packed quantizers** (no codebooks,
no training): whole vector -> packed bytes + a scalar scale (`max_abs`).

| Function | Bits/dim | Pack unit | Output |
|----------|----------|-----------|--------|
| `rabitq_quantize` / `similarity` | 1  | `u64` (POPCNT) | `Box<[u64]>` |
| `turbo_quant_quantize` / `similarity` | 4 | `u8` nibble | `(Box<[u8]>, f32)` |
| `sq8_quantize` / `similarity` | 8 | `i8` | `(Box<[i8]>, f32)` |

These feed `node::VectorRepresentations` (`Binary`, `Turbo`, `SQ8`, `Full`,
`MmapFull`, `None`). **There is no `PQ` variant today.**

`src/index/scann.rs` is the only quantized index (vs the HNSW graph): per-dimension
scalar SQ8 (`u8` code + per-dim min/max bounds) with a two-stage approximate score
then full f32 re-rank. It carries the explicit gate for this analysis:

```rust
// ponytail
// Simplified SQ8 only - no anisotropic quantization, no PQ, no GPU.
```

`VecIndex` (`src/index/mod.rs`) is the pluggable backend trait (`search`/`add`/
`estimate_memory_bytes`/`len`); `create_index` selects on `IndexType::{Hnsw, Ivf,
Flat, DiskAnn, Scann}`. Note: the `DiskAnn` variant is a **Vamana graph kept
entirely in memory** (`src/index/diskann.rs`) — no disk I/O, no mmap; only the
graph algorithm is DiskANN-inspired. Storage tiering
(`docs/architecture/STORAGE-TIERS.md`) maps
segments to mmapped `vstore_L*.vanta` levels L0-L3 (L0 64MiB, L2 4GiB, L3 32GiB);
cold tiers sit **on disk via mmap**, not RAM-resident.

So "exceeds RAM" is currently handled at the storage tier (disk mmap), not at the
quantization layer — the key overlap with PQ's value proposition.

## 2. What PQ is (theory, verified)

Product Quantization -- Jegou, Douze, Schmid, *Product quantization for nearest
neighbor search*, IEEE TPAMI 2011 (reference paper, hosted on
`corpus-texmex.irisa.fr`). It compresses a D-dim vector by:

1. **Splitting** the vector into M contiguous **subvectors / subspaces**
   (`D = M x dim_per_sub`).
2. Training, per subspace, an independent **k-means codebook** of K centroids
   (commonly K = 256, an 8-bit index per subspace).
3. Storing each vector as its M centroid IDs, so
   **D x 32 bits -> M x log2(K) bits** plus the shared codebook.

At query time, **asymmetric distance (ADC)** precomputes a lookup table of
distances from the query subvectors to each centroid, so scoring reads tables,
not decompressed floats.

Confirmed across sources (emergentmind, grokipedia, inferensys): the defining
trait vs SQ8 is that each subspace is quantized independently; codebooks are
learned by k-means on a training set (the texmex corpus explicitly ships learn
vectors for this). Compression is commonly 32-64x, up to 96x when M approx D/8;
some sources cite >95% for tuned M/nbits.

## 3. Corpus motivating PQ (datasets > RAM)

Band references: `docs/glosario/memory-efficiency.md` (XL "1M-10M -> 12GB",
'Massive' >10M -> >12GB; suggestion mmap + PQ). Dataset numbers from
`corpus-texmex.irisa.fr` and ann-benchmarks.

| Dataset | dims | count | f32 in RAM | PQ code (M=16, 8b) |
|---------|------|-------|------------|--------------------|
| SIFT1M    | 128 | 1,000,000    | ~512 MB  | ~16 MB |
| GIST1M    | 960 | 1,000,000    | ~3.84 GB  | ~16 MB |
| SIFT1B    | 128 | 1,000,000,000 | ~512 GB f32 | ~16 GB |
| GloVe 100-angular | 100  | 1,183,514 | 473 MB | ~16 MB |
| nytimes-256 / fashion-mnist-784 | 256/784 | n/a | suite reference | ~16 MB |

The motivating gap: SIFT1B needs ~92 GB on disk (u8) and would be ~512 GB if
expanded to f32 in RAM; PQ at 16 B/vec drops it to ~16 GB. That is the "96x"
claim in `NUEVO-16`. Everything tracked today (GloVe-100, SIFT1M subsets 10K/100K)
is well under RAM on the target nodes.

## 4. How it would fit (scoping only, not to build)

- Add `VectorRepresentations::PQ` storing packed centroid-IDs, padded to the
  existing `MMAP_ALIGNMENT` (8) mmap-safety convention (src/node.rs).
- Add `pq_quantize` / `pq_similarity` (ADC lookup table) in quantization.rs;
  the k-means codebook trainer is vanta-engine hot-path work.
- Either extend `ScannIndex` or add an `IndexType::Pq` backend reusing `VecIndex`;
  then lift the `// ponytail` `no PQ` note.
- Storage offset / packed-offset semantics unchanged.

## 5. Decision

> **@defer** -- do not implement PQ in this cycle (unchanged from REC-009).

**Rationale**

1. **No demonstrated demand.** Tracked benchmarks (GloVe-100, SIFT1M subsets)
   fit in RAM / are already covered by mmap tiering + SQ8. No workload keeps a
   full vector set > RAM in the baseline of today.
2. **SQ8 + tiered mmap already bounds the problem.** "Exceeds RAM" is caught at
   the storage tier (cold levels on disk via mmap). PQ's only unique claim is a
   denser RAM layout that duplicates what the tier already does.
3. **PQ is vanta-engine domain, not worker.** k-means codebook training + ADC
   lookup tables + sub-vector SIMD kernels are engine/tuner hot-path work. The
   `scann.rs` ponytail "no PQ" is the marker for that scope.
4. **Recall cost.** Published sources put PQ around 0.95 recall vs SQ8
   ~0.985 / f32 ~0.998 (repo glosario memory-efficiency table). Trading recall
   for density is only worth it if the "dataset does not fit RAM" condition is
   actually hit; today it is not.

**Promote to @appr when (any):**

- A validated > RAM workload (e.g. GIST1M / SIFT1B scale, or a GloVe-300 assembly
  > 1-2 GB) enters the API surface as an explicitly required path.
- A public competitive bench requests a PQ/SQ "compression trade-off" column
  against Pinecone PQFS / Milvus IVF-PQ (Pinecone's L3 > 1M uses IVF + PQFS;
  see docs/research/INV-019-pinecone-architecture-competitor.md).
- Arch/engine explicitly asks for the PQ design (then engine owns the trainer
  and worker owns the thin bindings).

## 6. Phases plan (if @appr ever, conditional)

- **P1 — Trainer + quantizer** (vanta-engine): offline k-means codebook on the
  corpus subset; `VectorRepresentation::PQ`; `pq_quantize`/`pq_similarity` ADC.
- **P2 — Index integration**: extend `ScannIndex` or add `IndexType::Pq` reusing
  `VecIndex`; two-stage approximate + re-rank; lift the `no PQ` ponytail note.
  Profile ADC on mmapped PQ store.
- **P3 — Correctness + benches**: roundtrip/recall bounds tests; a SIFT1M +
  GIST1M (and a 1B subset) recall vs latency vs SQ8 comparison.
- **P4 — Serialization + storage/tier parity**: `VantaFile` code layout, LSM
  migration path, `estimate_memory_bytes`, and the 8-byte alignment on mmap.

> This is a conditional plan only: it is executed if and only if a promoter
> condition in Sec 5 (Promote) is met. Do not start P1 without that trigger.

## 7. Non-goals / deferred

- Anisotropic quantization and full IVF-PQ / PQFS fall in the same deferred
  bucket (see scann.rs note). PQ here means the classic Jegou scheme: symmetric
  k-means codebooks at build, asymmetric ADC at query.
- No GPU kernels; CPU SIMD ADC only (matches repo constraint set).
- No new dependency for k-means: there is no k-means in the tree today; a tiny
  in-house centroid trainer is the lazy fit if P1 ever starts.

## 8. Sources cited

- Jegou, Douze, Schmid, *Product quantization for nearest neighbor search*,
  IEEE TPAMI 2011. corpus-texmex.irisa.fr - SIFT1M / GIST1M / SIFT1B.
- ann-benchmarks (erikbern) - glove-100-angular, nytimes-256, fashion-mnist dataset
  listings (ann-benchmarks.com).
- Docs glosario/memory-efficiency.md - tier bands + PQ recall/latency rows.
- Docs/Investigaciones/INV-019-pinecone-architecture-competitor.md - IVF + PQFS.
- Secondary theory confirmations: emergentmind.com, grokipedia, inferensys glossaries.