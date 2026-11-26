#!/usr/bin/env python3
"""ChromaDB -> VantaDB migration CLI on top of the SDK migrator.

Example:
    python scripts/migrate_from_chroma.py --chroma-path ./chroma_data \
        --vanta-path ./vanta_data --collection all --batch-size 500

--help works without vantadb or chromadb installed (optional imports are
deferred until after argument parsing).
"""

from __future__ import annotations

import argparse
import importlib.util
import sys
import time
from collections.abc import Callable
from pathlib import Path


def _parse() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="migrate_from_chroma.py",
        description="Migrate a ChromaDB persistent store into VantaDB.",
    )
    parser.add_argument(
        "--chroma-path",
        default="./chroma_data",
        help="Path to the ChromaDB persistent store (default: ./chroma_data)",
    )
    parser.add_argument(
        "--vanta-path",
        default="./vanta_data",
        help="Destination VantaDB database path (default: ./vanta_data)",
    )
    parser.add_argument(
        "--collection",
        default="all",
        help="Collection to migrate or 'all' (default: all)",
    )
    parser.add_argument(
        "--namespace-prefix",
        default="chroma/",
        help="Prefix for derived namespaces (default: 'chroma/')",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=500,
        help="Records per batch; must be positive (default: 500)",
    )
    return parser.parse_args()


def _resolve(path: str) -> Path:
    return Path(path).expanduser().resolve()


def _load_migrator() -> Callable | None:
    """Load migrate_from_chroma from THIS checkout (not the installed wheel).

    The published 0.9 wheel lacks the keyword-only extension, so the helper
    is imported by file path. Only stdlib plus `from vantadb import Client`
    (lazy, inside the migration call) is required.
    """
    if importlib.util.find_spec("vantadb") is None:
        print(
            "VantaDB SDK is not installed. Run: pip install vantadb-py",
            file=sys.stderr,
        )
        return None
    helper = (
        Path(__file__).resolve().parents[1]
        / "vantadb-python"
        / "vantadb_py"
        / "migrate"
        / "chroma.py"
    )
    if not helper.is_file():
        print(
            "error: migration helper not found in this checkout. "
            "Run this script from a complete VantaDB checkout.",
            file=sys.stderr,
        )
        return None
    spec = importlib.util.spec_from_file_location("_vanta_chroma_migrator", helper)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.migrate_from_chroma


def main() -> int:
    args = _parse()

    if args.batch_size <= 0:
        print("error: --batch-size must be a positive integer.", file=sys.stderr)
        return 1

    src = _resolve(args.chroma_path)
    dst = _resolve(args.vanta_path)

    if src == dst or dst in src.parents or src in dst.parents:
        print(
            "error: source and destination cannot be the same or nested paths.",
            file=sys.stderr,
        )
        return 1

    if not src.is_dir() or not (src / "chroma.sqlite3").is_file():
        print(
            f"error: '{src}' does not exist or has no chroma.sqlite3 "
            "(is this the right ChromaDB store path?)",
            file=sys.stderr,
        )
        return 1

    if importlib.util.find_spec("chromadb") is None:
        print("ChromaDB is not installed. Run: pip install chromadb", file=sys.stderr)
        return 1

    migrator = _load_migrator()
    if migrator is None:
        return 1

    stats = {"collections": 0, "records": 0}

    def _on_collection(name: str, count: int) -> None:
        stats["collections"] += 1
        stats["records"] += count

    from chromadb.errors import ChromaError

    t0 = time.perf_counter()
    try:
        migrator(
            str(src),
            str(dst),
            collection_name=None if args.collection == "all" else args.collection,
            batch_size=args.batch_size,
            namespace_prefix=args.namespace_prefix,
            on_collection=_on_collection,
        )
    except (ChromaError, RuntimeError, OSError, ValueError) as exc:
        print(
            f"error: migration failed after {stats['records']} record(s) "
            f"processed from completed collections: {exc}. "
            f"The destination at '{dst}' may contain a partial import; "
            "re-running updates the same keys.",
            file=sys.stderr,
        )
        return 1
    elapsed = time.perf_counter() - t0

    rate = stats["records"] / elapsed if elapsed > 0 else 0.0
    print(
        f"Migration complete: {stats['collections']} collection(s), "
        f"{stats['records']} record(s) in {elapsed:.2f}s ({rate:.0f} docs/s)."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
