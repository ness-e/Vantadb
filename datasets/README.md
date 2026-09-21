# datasets/ — Raw SIFT-1M (texmex corpus)

> **Local cache.** `/datasets/` is gitignored (see `.gitignore:75`). This README documents
> the raw SIFT-1M download produced by `dev-tools/scripts/download_sift.py`. No binary files
> in this repo — the script fetches them from IRISA's FTP at runtime.

## Datasets

| Name | Source URL | License | Approx size | Download command | Destination |
|---|---|---|---|---|---|
| **SIFT-1M** (texmex corpus) | ftp://ftp.irisa.fr/local/texmex/corpus/sift.tar.gz | Research use (Jégou et al., INRIA) — non-commercial redistribution discouraged | ~160 MB compressed, ~1 GB uncompressed | `python dev-tools/scripts/download_sift.py` | `datasets/sift.tar.gz` + `datasets/sift_*` |

> **What the tarball contains:** `sift_base.fvecs` (1M × 128 SIFT descriptors, training set),
> `sift_query.fvecs` (10k queries), `sift_groundtruth.ivecs` (100-NN ground truth for each
> query). These are the standard texmex/INRIA files used by ANN literature since ~2010.

## Layout after download

```
datasets/
├── sift.tar.gz                     # ~160 MB (kept unless you delete it manually)
└── sift_*/                         # extracted from the tarball
    ├── sift_base.fvecs             # 1M × 128 float32 (binary .fvecs format)
    ├── sift_query.fvecs            # 10k × 128
    └── sift_groundtruth.ivecs      # 10k × 100 int32 (top-100 exact NN per query)
```

## How it's consumed

The raw `.fvecs` / `.ivecs` formats are read by:

- `benchmarks/competitive_bench.py` — competitive benchmarking vs external ANN libraries
  (faiss, hnswlib, scann, etc.). This is the only consumer in the repo today.
- Legacy `tests/competitive_*` paths (pre-`data/benchmark/` migration, see git history).

The HDF5-based subsets documented in [`data/README.md`](../data/README.md) (`sift-128/` train.f32,
test.f32, test_neighbors.u64) are **derived** from the same SIFT-1M corpus, sliced to 10k train
+ 200 queries + top-10 NN for fast Rust criterion benchmarks.

## Why two directories?

| Directory | Format | Consumer | Use case |
|---|---|---|---|
| `data/benchmark/` | HDF5 → raw f32 binary (10k subset) | Rust criterion benches, certification tests | Fast deterministic benches in CI |
| `datasets/` | Raw .fvecs / .ivecs (full 1M) | Python competitive_bench.py | Head-to-head comparisons vs faiss/scann at realistic scale |

Both are gitignored; pick whichever your benchmark needs. Downloading both is normal for a
full local setup (~1.2 GB total on disk).

## Verifying presence

After download, `Test-Path datasets/sift_base.fvecs` should return `True`. The script
prints "Extraction complete. You can now run the benchmarks!" on success.

## Related

- [`data/README.md`](../data/README.md) — HDF5-derived subsets (sift-128/, glove-100/) used by Rust benches.
- [`embeddings/README.md`](../embeddings/README.md) — local ONNX/HF embedding models (separate cache).
- `dev-tools/scripts/download_sift.py` — this directory's downloader (FTP).
- `scripts/download_ground_truth.py` — produces the `data/benchmark/` subsets from ann-benchmarks HDF5.