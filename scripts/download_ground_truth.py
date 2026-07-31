#!/usr/bin/env python3
"""Download ANN benchmark datasets + pre-compute ground truth as raw binaries.

Usage:
    python scripts/download_ground_truth.py [--datasets SIFT-128 GLOVE-100]

Output in data/benchmark/:
  {dataset}/train.f32       — training vectors (flat f32, row-major)
  {dataset}/test.f32        — query vectors (flat f32, row-major)
  {dataset}/test_neighbors.u64 — ground truth neighbor IDs (u64, N_QUERIES × K)
  {dataset}/meta.json       — dims, count, metric, source
"""

import argparse
import json
import os
import sys
from pathlib import Path

import h5py
import numpy as np
import requests

BENCH_DIR = Path("data/benchmark")

DATASETS = {
    "sift-128": {
        "url": "http://ann-benchmarks.com/sift-128-euclidean.hdf5",
        "metric": "euclidean",
    },
    "glove-100": {
        "url": "http://ann-benchmarks.com/glove-100-angular.hdf5",
        "metric": "angular",
    },
}

N_TRAIN = 10_000  # subset for quick benchmarks
N_QUERIES = 200
K_GROUND_TRUTH = 10


def download_hdf5(url: str, dest: Path) -> Path:
    """Download HDF5 if not cached."""
    cache = BENCH_DIR / "cache" / url.rpartition("/")[2]
    if cache.exists():
        print(f"  Found cached {cache}")
        return cache
    cache.parent.mkdir(parents=True, exist_ok=True)
    print(f"  Downloading {url} ...")
    r = requests.get(url, stream=True, timeout=300)
    r.raise_for_status()
    total = int(r.headers.get("content-length", 0))
    downloaded = 0
    with open(cache, "wb") as f:
        for chunk in r.iter_content(chunk_size=1 << 20):
            f.write(chunk)
            downloaded += len(chunk)
            if total:
                pct = downloaded * 100 // total
                print(f"\r    {pct}% ({downloaded // (1<<20)} MiB)", end="")
    print()
    return cache


def extract_dataset(name: str, hdf5_path: Path, metric: str):
    """Extract HDF5 → raw binaries for Rust benchmarks."""
    out = BENCH_DIR / name
    out.mkdir(parents=True, exist_ok=True)

    with h5py.File(hdf5_path, "r") as f:
        train_all = f["train"][:]  # (n_train, dims)
        test_all = f["test"][:]  # (n_test, dims)
        neighbors_all = f["neighbors"][:]  # (n_test, K)
        # distances_all = f["distances"][:]

    dims = train_all.shape[1]
    n_train = min(N_TRAIN, train_all.shape[0])
    n_test = min(N_QUERIES, test_all.shape[0])

    train = train_all[:n_train].astype(np.float32)
    test = test_all[:n_test].astype(np.float32)
    test_neighbors = neighbors_all[:n_test, :K_GROUND_TRUTH].astype(np.uint64)

    # Write raw f32 vectors
    train.tofile(out / "train.f32")
    test.tofile(out / "test.f32")
    test_neighbors.tofile(out / "test_neighbors.u64")

    meta = {
        "dims": dims,
        "n_train": n_train,
        "n_test": n_test,
        "k_ground_truth": K_GROUND_TRUTH,
        "metric": metric,
        "source": str(hdf5_path),
    }
    with open(out / "meta.json", "w") as f:
        json.dump(meta, f, indent=2)

    print(f"  {name}: {n_train} train × {dims}d = {train.nbytes // (1<<20)} MiB")
    print(f"           {n_test} queries, ground truth k={K_GROUND_TRUTH}")
    print(f"           metric: {metric}")


def main():
    parser = argparse.ArgumentParser(description="Download ANN benchmark datasets")
    parser.add_argument(
        "--datasets",
        nargs="+",
        default=list(DATASETS.keys()),
        choices=list(DATASETS.keys()),
        help="Datasets to download",
    )
    args = parser.parse_args()

    for name in args.datasets:
        info = DATASETS[name]
        print(f"\n{'='*60}")
        print(f"Dataset: {name}")
        print(f"{'='*60}")
        try:
            hdf5_path = download_hdf5(info["url"], BENCH_DIR)
            extract_dataset(name, hdf5_path, info["metric"])
        except Exception as e:
            print(f"  FAILED: {e}", file=sys.stderr)

    print(f"\nDone. Files in {BENCH_DIR.resolve()}")


if __name__ == "__main__":
    main()
