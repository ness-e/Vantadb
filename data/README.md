# data/ — Benchmark datasets (ANN benchmarks + GloVe subsets)

> **Local cache.** Both `/data/` and `/datasets/` are gitignored (see `.gitignore:23,75`).
> This README documents what the download scripts in `scripts/` and `dev-tools/scripts/` produce
> and where the files land on disk. No binary files in this repo — these scripts fetch them at
> benchmark time.

## Datasets

| Name | Source URL | License | Approx size | Download command | Destination |
|---|---|---|---|---|---|
| **GloVe-100** (`glove.6B.100d.txt`) | https://nlp.stanford.edu/data/glove.6B.zip | Public Domain (PDDL 1.0) | ~822 MB zip (extracts only 100d txt ~350 MB) | `bash scripts/download_benchmark_datasets.sh` *or* `pwsh scripts/download_benchmark_datasets.ps1` | `data/benchmark/glove.6B.100d.txt` |
| **SIFT-128 Euclidean** subset (HDF5) | http://ann-benchmarks.com/sift-128-euclidean.hdf5 | ann-benchmarks (MIT) | ~500 MB HDF5 → ~50 MB extracted binaries | `python scripts/download_ground_truth.py --datasets sift-128` | `data/benchmark/sift-128/{train.f32,test.f32,test_neighbors.u64,meta.json}` |
| **GloVe-100 Angular** subset (HDF5) | http://ann-benchmarks.com/glove-100-angular.hdf5 | ann-benchmarks (MIT) | ~1 GB HDF5 → ~80 MB extracted binaries | `python scripts/download_ground_truth.py --datasets glove-100` | `data/benchmark/glove-100/{train.f32,test.f32,test_neighbors.u64,meta.json}` |

> **All three at once:** `python scripts/download_ground_truth.py` (default `--datasets sift-128 glove-100`).
> The script caches the HDF5 in `data/benchmark/cache/` to avoid re-downloading.

## Layout after downloads

```
data/
└── benchmark/
    ├── glove.6B.100d.txt          # raw GloVe (only 100d vector file extracted)
    ├── cache/
    │   ├── sift-128-euclidean.hdf5
    │   └── glove-100-angular.hdf5
    ├── sift-128/
    │   ├── train.f32              # 10k × 128d, flat f32 row-major (subset of SIFT-1M)
    │   ├── test.f32               # 200 queries × 128d
    │   ├── test_neighbors.u64     # ground truth: 200 × 10 neighbor IDs
    │   └── meta.json              # dims, counts, metric=euclidean, source path
    └── glove-100/
        ├── train.f32              # 10k × 100d (subset of GloVe-6B)
        ├── test.f32               # 200 queries × 100d
        ├── test_neighbors.u64     # ground truth: 200 × 10 neighbor IDs
        └── meta.json              # metric=angular
```

## Subset sizes (controlled by the download script)

`scripts/download_ground_truth.py` defines `N_TRAIN=10_000`, `N_QUERIES=200`,
`K_GROUND_TRUTH=10`. The script takes the **first** N_TRAIN entries from the full HDF5
(1M SIFT / 400k GloVe) so benchmarks run in seconds, not hours. Full datasets stay on disk
only in their raw HDF5 form inside `cache/`.

## Verifying presence

After downloads, `Test-Path data/benchmark/sift-128/train.f32` and similar should return
`True`. The companion check scripts are:

- `scripts/verify_datasets.sh` / `.ps1` — exit 1 when certification tests would silently skip.
- `cargo test --workspace --features test-bench-datasets` — runs benchmarks that read these files.

## Related

- [`datasets/README.md`](../datasets/README.md) — SIFT-1M raw `.fvecs` (used by competitive_bench and legacy scripts).
- [`embeddings/README.md`](../embeddings/README.md) — local ONNX/HF embedding models (separate cache).
- `scripts/download_benchmark_datasets.sh` / `.ps1` — GloVe-100 downloader.
- `scripts/download_ground_truth.py` — HDF5 → raw f32 binary extractor.