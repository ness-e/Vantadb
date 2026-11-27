#!/usr/bin/env python3
"""Download ANN benchmark datasets + pre-compute ground truth as raw binaries.

Single entry point for ALL bench datasets (Fase 0 plan bench-estandar).
Idempotent: skips datasets whose output + meta.json already exist. Writes
data/benchmark/manifest.json with file list + sha256 for verification.

Usage:
    python scripts/download_ground_truth.py [--datasets SIFT-128 GLOVE-100]
    python scripts/download_ground_truth.py --check   # verify presence only

Output in data/benchmark/:
  {dataset}/train.f32       — training vectors (flat f32, row-major)
  {dataset}/test.f32        — query vectors (flat f32, row-major)
  {dataset}/test_neighbors.u64 — ground truth neighbor IDs (u64, N_QUERIES × K)
  {dataset}/meta.json       — dims, count, metric, source
  beir-scifact/{corpus.jsonl,queries.jsonl,qrels.tsv,meta.json}
"""

import argparse
import hashlib
import json
import sys
import zipfile
from pathlib import Path

import numpy as np

BENCH_DIR = Path("data/benchmark")

DATASETS = {
    "sift-128": {
        "kind": "hdf5",
        "url": "http://ann-benchmarks.com/sift-128-euclidean.hdf5",
        "metric": "euclidean",
        "n_train": 10_000,
        "n_queries": 200,
    },
    "glove-100": {
        "kind": "glove-txt",
        "local": "data/benchmark/glove.6B.100d.txt",
        "metric": "angular",
        "n_train": 100_000,
        "n_queries": 1_000,
    },
    "glove-300": {
        "kind": "glove-txt",
        "local": "data/benchmark/glove.6B.300d.txt",
        "metric": "angular",
        "n_train": 10_000,
        "n_queries": 200,
    },
    "sift-1m": {
        # FIND-664: the `.fvecs` layout that
        # `tests/certification/{sift_validation,competitive_bench}.rs` read,
        # produced from the ann-benchmarks HDF5. It used to descend from
        # `dev-tools/scripts/download_sift.py` (an FTP pull from
        # corpus-texmex that is dead, that swallowed its own errors and exited
        # 0) — so the two opt-in SIFT jobs in certification.yml could never
        # provision anything and skipped green instead.
        "kind": "hdf5-fvecs",
        "url": "http://ann-benchmarks.com/sift-128-euclidean.hdf5",
        "dir": "datasets/sift",
        "metric": "euclidean",
        "n_train": 1_000_000,
        "n_queries": 10_000,
    },
    "glove-100-ann": {
        "kind": "hdf5",
        "url": "http://ann-benchmarks.com/glove-100-angular.hdf5",
        "metric": "angular",
        "n_train": 1_183_514,
        "n_queries": 10_000,
    },
    "beir-scifact": {
        "kind": "beir-zip",
        "url": "https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets/scifact.zip",
        "metric": "text",
    },
}

N_TRAIN = 10_000  # default subset for quick benchmarks (overridden per dataset)
N_QUERIES = 200
K_GROUND_TRUTH = 10


def sha256_of(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def _count_lines(path: Path) -> int:
    with open(path, encoding="utf-8") as f:
        return sum(1 for _ in f)


def dataset_dir(name: str, info: dict) -> Path:
    """Where a dataset lands: `data/benchmark/<name>/` unless it overrides."""
    return Path(info["dir"]) if "dir" in info else BENCH_DIR / name


def expected_files(name: str, info: dict) -> list[str]:
    if info["kind"] == "beir-zip":
        return ["corpus.jsonl", "queries.jsonl", "qrels.tsv", "meta.json"]
    if info["kind"] == "hdf5-fvecs":
        return ["sift_base.fvecs", "sift_query.fvecs", "sift_groundtruth.ivecs"]
    return ["train.f32", "test.f32", "test_neighbors.u64", "meta.json"]


def missing_files(name: str, info: dict) -> list[str]:
    """Relative names of the expected files that are absent (empty = ready)."""
    out = dataset_dir(name, info)
    return [f for f in expected_files(name, info) if not (out / f).exists()]


def is_ready(name: str, info: dict) -> bool:
    return not missing_files(name, info)


def download_hdf5(url: str, dest: Path) -> Path:
    """Download HDF5 if not cached."""
    import requests  # lazy: --check must work without the network stack installed

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
                print(f"\r    {pct}% ({downloaded // (1 << 20)} MiB)", end="")
    print()
    return cache


def extract_dataset(name: str, hdf5_path: Path, info: dict):
    """Extract HDF5 → raw binaries for Rust benchmarks."""
    import h5py  # lazy: GloVe/BEIR provisioning must not require h5py (FIND-664)

    out = BENCH_DIR / name
    out.mkdir(parents=True, exist_ok=True)
    n_train_cfg = info.get("n_train", N_TRAIN)
    n_queries_cfg = info.get("n_queries", N_QUERIES)
    metric = info["metric"]

    with h5py.File(hdf5_path, "r") as f:
        train_all = f["train"][:]  # (n_train, dims)
        test_all = f["test"][:]  # (n_test, dims)
        neighbors_all = f["neighbors"][:]  # (n_test, K)
        # distances_all = f["distances"][:]

    dims = train_all.shape[1]
    n_train = min(n_train_cfg, train_all.shape[0])
    n_test = min(n_queries_cfg, test_all.shape[0])

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

    print(f"  {name}: {n_train} train × {dims}d = {train.nbytes // (1 << 20)} MiB")
    print(f"           {n_test} queries, ground truth k={K_GROUND_TRUTH}")
    print(f"           metric: {metric}")


def write_vecs(path: Path, rows, dtype) -> None:
    """Write a `.fvecs`/`.ivecs` file: int32 dim prefix, then `dim` values."""
    import struct

    dim = int(rows.shape[1])
    with open(path, "wb") as f:
        for row in rows:
            f.write(struct.pack("<i", dim))
            f.write(row.astype(dtype).tobytes())


def extract_hdf5_fvecs(name: str, hdf5_path: Path, info: dict):
    """HDF5 → the `.fvecs`/`.ivecs` layout `tests/certification/sift_*.rs` read.

    JOIN of the two formats in this repo: `sift_validation` and
    `competitive_bench` read `datasets/sift/sift_{base,query}.fvecs` +
    `sift_groundtruth.ivecs`; the only live source for SIFT-1M is the
    ann-benchmarks HDF5 that every other dataset here already downloads.
    """
    import h5py  # lazy (FIND-664)

    out = dataset_dir(name, info)
    out.mkdir(parents=True, exist_ok=True)
    n_train = info["n_train"]
    n_queries = info["n_queries"]

    with h5py.File(hdf5_path, "r") as f:
        train = f["train"][:n_train].astype(np.float32)
        test = f["test"][:n_queries].astype(np.float32)
        gt = f["neighbors"][:n_queries, : info.get("k", 100)].astype(np.int32)

    if train.shape[0] != n_train or test.shape[0] != n_queries:
        raise ValueError(
            f"{name}: HDF5 has train={train.shape[0]} test={test.shape[0]}, "
            f"expected {n_train}/{n_queries}"
        )

    write_vecs(out / "sift_base.fvecs", train, np.float32)
    write_vecs(out / "sift_query.fvecs", test, np.float32)
    write_vecs(out / "sift_groundtruth.ivecs", gt, np.int32)
    print(
        f"  {name}: {n_train} base x {train.shape[1]}d, {n_queries} queries, "
        f"ground truth k={gt.shape[1]} -> {out}"
    )
    print(
        f"           disk: {(train.nbytes + test.nbytes + gt.nbytes) // (1 << 20)} MiB"
    )


def extract_glove_txt(name: str, info: dict):
    """Convert local GloVe txt (word + floats per line) → standard layout.
    Queries are sampled uniformly from the corpus (documented in meta)."""
    import random

    out = BENCH_DIR / name
    out.mkdir(parents=True, exist_ok=True)
    vecs: list = []
    with open(info["local"], encoding="utf-8") as f:
        for line in f:
            parts = line.rstrip("\n").split(" ")
            if len(parts) > 2:
                vecs.append([float(x) for x in parts[1:]])
            if len(vecs) >= info["n_train"] + info["n_queries"]:
                break
    arr = np.asarray(vecs, dtype=np.float32)
    dims = arr.shape[1]
    rng = random.Random(42)
    idx = list(range(arr.shape[0]))
    rng.shuffle(idx)
    train = arr[idx[: info["n_train"]]]
    test = arr[idx[info["n_train"] : info["n_train"] + info["n_queries"]]]
    train = train / np.maximum(np.linalg.norm(train, axis=1, keepdims=True), 1e-12)
    test = test / np.maximum(np.linalg.norm(test, axis=1, keepdims=True), 1e-12)
    train = train.astype(np.float32)
    test = test.astype(np.float32)
    # Ground truth by exact search (subset is small: n_train × n_queries).
    d = test @ train.T
    test_neighbors = np.argsort(-d, axis=1)[:, :K_GROUND_TRUTH].astype(np.uint64)
    train.tofile(out / "train.f32")
    test.tofile(out / "test.f32")
    test_neighbors.tofile(out / "test_neighbors.u64")
    meta = {
        "dims": int(dims),
        "n_train": train.shape[0],
        "n_test": test.shape[0],
        "k_ground_truth": K_GROUND_TRUTH,
        "metric": info["metric"],
        "queries": "uniform sample of corpus (no held-out split upstream)",
        "source": info["local"],
    }
    with open(out / "meta.json", "w") as f:
        json.dump(meta, f, indent=2)
    print(
        f"  {name}: {train.shape[0]} train × {dims}d (normalized), {test.shape[0]} queries + exact ground truth"
    )


def extract_beir_zip(name: str, info: dict):
    """Download + extract BEIR SciFact (corpus/queries/qrels) for text benches."""
    import requests  # lazy: --check must work without the network stack installed

    out = BENCH_DIR / name
    out.mkdir(parents=True, exist_ok=True)
    cache = BENCH_DIR / "cache" / info["url"].rpartition("/")[2]
    if not cache.exists():
        cache.parent.mkdir(parents=True, exist_ok=True)
        print(f"  Downloading {info['url']} ...")
        r = requests.get(info["url"], stream=True, timeout=300)
        r.raise_for_status()
        with open(cache, "wb") as f:
            f.writelines(r.iter_content(chunk_size=1 << 20))
    else:
        print(f"  Found cached {cache}")
    with zipfile.ZipFile(cache) as z:
        names = z.namelist()

        def pick(*cands):
            for c in cands:
                for n in names:
                    if n.endswith(c):
                        return n
            raise ValueError(f"missing {cands} in {cache.name}")

        corpus = pick("corpus.jsonl")
        queries = pick("queries.jsonl")
        qrels = pick("qrels/test.tsv", "qrels.tsv", "test.tsv")
        for src, dst in [
            (corpus, "corpus.jsonl"),
            (queries, "queries.jsonl"),
            (qrels, "qrels.tsv"),
        ]:
            data = z.read(src)
            if (out / dst).exists():
                (out / dst).unlink()
            # Zip members carry subdirs (scifact/corpus.jsonl); flatten.
            (out / dst).write_bytes(data)
    n_corpus = _count_lines(out / "corpus.jsonl")
    n_queries = _count_lines(out / "queries.jsonl")
    meta = {
        "n_corpus": n_corpus,
        "n_queries": n_queries,
        "metric": "text",
        "source": info["url"],
    }
    with open(out / "meta.json", "w") as f:
        json.dump(meta, f, indent=2)
    print(f"  {name}: {n_corpus} docs, {n_queries} queries")


def write_manifest(names: list):
    manifest = {"datasets": {}}
    for name in names:
        info = DATASETS[name]
        out = dataset_dir(name, info)
        files = {}
        for f in expected_files(name, info):
            p = out / f
            if p.exists():
                files[f] = {"bytes": p.stat().st_size, "sha256": sha256_of(p)}
        manifest["datasets"][name] = {"metric": info["metric"], "files": files}
    with open(BENCH_DIR / "manifest.json", "w") as f:
        json.dump(manifest, f, indent=2)
    print(f"Manifest -> {BENCH_DIR / 'manifest.json'}")


def main():
    parser = argparse.ArgumentParser(
        description="Download ANN benchmark datasets (idempotent)"
    )
    parser.add_argument(
        "--datasets",
        nargs="+",
        default=list(DATASETS.keys()),
        choices=list(DATASETS.keys()),
        help="Datasets to download",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Verify presence only (exit 1 if anything missing, no downloads)",
    )
    args = parser.parse_args()

    if args.check:
        # FIND-664: name the absent FILE, not just the dataset — this is the
        # TBH-01 gate CI runs before a bench, and "MISSING sift-1m" does not
        # tell anyone what to do.
        absent = [(n, missing_files(n, DATASETS[n])) for n in args.datasets]
        absent = [(n, f) for n, f in absent if f]
        if absent:
            for n, files in absent:
                out = dataset_dir(n, DATASETS[n])
                print(
                    f"MISSING {n}: {', '.join(str(out / f) for f in files)}",
                    file=sys.stderr,
                )
            print(
                "Provision with: python scripts/download_ground_truth.py --datasets "
                + " ".join(n for n, _ in absent),
                file=sys.stderr,
            )
            sys.exit(1)
        print(f"All {len(args.datasets)} datasets ready in {BENCH_DIR.resolve()}")
        return

    done = []
    failed = []
    for name in args.datasets:
        info = DATASETS[name]
        print(f"\n{'=' * 60}")
        print(f"Dataset: {name}")
        print(f"{'=' * 60}")
        if is_ready(name, info):
            print("  Already ready — skipped.")
            done.append(name)
            continue
        try:
            kind = info["kind"]
            if kind == "hdf5":
                hdf5_path = download_hdf5(info["url"], BENCH_DIR)
                extract_dataset(name, hdf5_path, info)
            elif kind == "hdf5-fvecs":
                hdf5_path = download_hdf5(info["url"], BENCH_DIR)
                extract_hdf5_fvecs(name, hdf5_path, info)
            elif kind == "glove-txt":
                extract_glove_txt(name, info)
            elif kind == "beir-zip":
                extract_beir_zip(name, info)
            else:
                raise ValueError(f"unknown kind: {kind}")
            done.append(name)
        # Broad by design: one dataset failing must not stop the others, and
        # every failure is now surfaced as exit 1 at the end (FIND-664).
        except Exception as e:  # noqa: BLE001
            print(f"  FAILED: {e}", file=sys.stderr)
            failed.append(name)

    write_manifest(done)
    # FIND-664: a per-dataset failure used to be printed and swallowed, so the
    # caller saw exit 0 and the bench downstream panicked 40 minutes later.
    if failed:
        print(
            f"\n{len(failed)} dataset(s) FAILED: {failed} — exit 1 "
            f"(see the FAILED lines above for the cause)",
            file=sys.stderr,
        )
        sys.exit(1)
    print(f"\nDone. Files in {BENCH_DIR.resolve()}")


if __name__ == "__main__":
    main()
