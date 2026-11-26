"""Migrate a ChromaDB persistent store into VantaDB.

Runnable standalone:

    python -m vantadb_py.migrate.chroma --source ./chroma_db --dest ./vantadb_db
    python -m vantadb_py.migrate.chroma --source ./chroma_db --dest ./vantadb_db \\
        --collection-name my_collection --namespace docs

or programmatically:

    from vantadb_py.migrate import migrate_from_chroma
    count = migrate_from_chroma("./chroma_db", "./vantadb_db")

Optional keyword-only arguments:
    namespace_prefix — prepended to each collection-derived namespace.
    on_collection(name, count) — called after each collection finishes, even empty.

Limitations: reads via collection.get (never collection.query, no auto-embedding);
PersistentClient is not a filesystem read-only snapshot — quiesce the source before
migrating; a failed run can leave a partial destination (re-running updates the
same keys; version history may grow).
"""

from __future__ import annotations

import argparse
import sys
from collections.abc import Callable


def _collection_names(client, collection_name: str | None) -> list[str]:
    """Return the ChromaDB collection names to migrate (sorted, deterministic).

    Supports both list_collections() return shapes: objects with ``.name``
    and plain strings.
    """
    if collection_name:
        return [collection_name]
    names = [getattr(c, "name", c) for c in client.list_collections()]
    return sorted(str(n) for n in names)


def _validate_parallel(name: str, ids, documents, metadatas, embeddings) -> None:
    n = len(ids)
    for label, arr in (
        ("documents", documents),
        ("metadatas", metadatas),
        ("embeddings", embeddings),
    ):
        if arr is not None and len(arr) != n:
            raise ValueError(
                f"collection '{name}': {label} length {len(arr)} != ids length {n} "
                "(refusing to truncate parallel arrays)"
            )


def migrate_from_chroma(
    source_path: str,
    dest_path: str,
    collection_name: str | None = None,
    namespace: str | None = None,
    batch_size: int = 500,
    *,
    namespace_prefix: str = "",
    on_collection: Callable[[str, int], None] | None = None,
) -> int:
    """Copy every document from a ChromaDB persistent store into VantaDB.

    Each ChromaDB collection becomes a VantaDB namespace (``namespace`` forces a
    single shared namespace; otherwise ``namespace_prefix + collection.name``).
    ChromaDB ids become keys, documents become payloads, and metadatas/embeddings
    are preserved without silent coercion.
    """
    if batch_size <= 0:
        raise ValueError("batch_size must be a positive integer")

    try:
        import chromadb
    except ImportError:
        raise ImportError(
            "chromadb is required for this migration: pip install chromadb"
        )

    from vantadb import Client

    client = chromadb.PersistentClient(
        path=source_path,
        settings=chromadb.Settings(anonymized_telemetry=False),
    )
    target = Client(dest_path)

    total = 0
    try:
        for name in _collection_names(client, collection_name):
            collection = client.get_collection(name, embedding_function=None)
            ns = namespace or (namespace_prefix + name)

            count = 0
            offset = 0
            while True:
                data = collection.get(
                    limit=batch_size,
                    offset=offset,
                    include=["documents", "metadatas", "embeddings"],
                )
                ids = data.get("ids") or []
                if not ids:
                    break

                documents = data.get("documents")
                metadatas = data.get("metadatas")
                embeddings = data.get("embeddings")
                _validate_parallel(name, ids, documents, metadatas, embeddings)

                records = []
                for i, key in enumerate(ids):
                    doc = documents[i] if documents is not None else None
                    emb = embeddings[i] if embeddings is not None else None
                    vector = [float(x) for x in emb] if emb is not None else None
                    records.append(
                        {
                            "namespace": ns,
                            "key": str(key),
                            "payload": doc if doc is not None else "",
                            "metadata": metadatas[i] if metadatas is not None else None,
                            "vector": vector,
                        }
                    )
                target.put_batch(records)
                count += len(ids)
                offset += len(ids)

            total += count
            if on_collection is not None:
                on_collection(name, count)

        target.flush()
        return total
    finally:
        target.close()


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="python -m vantadb_py.migrate.chroma",
        description="Migrate a ChromaDB persistent store into VantaDB.",
    )
    parser.add_argument(
        "--source", required=True, help="Path to the ChromaDB persistent store"
    )
    parser.add_argument(
        "--dest", required=True, help="Path where the VantaDB database will be created"
    )
    parser.add_argument(
        "--collection-name",
        default=None,
        help="Migrate only this collection (default: all)",
    )
    parser.add_argument(
        "--namespace",
        default=None,
        help="Target VantaDB namespace (default: collection name)",
    )
    parser.add_argument(
        "--batch-size", type=int, default=500, help="Records per batch (default: 500)"
    )
    parser.add_argument(
        "--namespace-prefix", default="", help="Prefix for derived namespaces"
    )
    args = parser.parse_args(argv)

    count = migrate_from_chroma(
        args.source,
        args.dest,
        collection_name=args.collection_name,
        namespace=args.namespace,
        batch_size=args.batch_size,
        namespace_prefix=args.namespace_prefix,
    )
    print(f"Migrated {count} records from ChromaDB into VantaDB at {args.dest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
