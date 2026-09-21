"""Migrate a ChromaDB persistent store into VantaDB.

Runnable standalone:

    python -m vantadb_py.migrate.chroma --source ./chroma_db --dest ./vantadb_db
    python -m vantadb_py.migrate.chroma --source ./chroma_db --dest ./vantadb_db \
        --collection-name my_collection --namespace docs

or programmatically:

    from vantadb_py.migrate import migrate_from_chroma
    count = migrate_from_chroma("./chroma_db", "./vantadb_db")
"""

from __future__ import annotations

import argparse
import sys


def _collection_names(client, collection_name: str | None) -> list[str]:
    """Return the ChromaDB collection names to migrate."""
    if collection_name:
        return [collection_name]
    return [c.name for c in client.list_collections()]


def migrate_from_chroma(
    source_path: str,
    dest_path: str,
    collection_name: str | None = None,
    namespace: str | None = None,
    batch_size: int = 500,
) -> int:
    """Copy every document from a ChromaDB persistent store into VantaDB.

    Each ChromaDB collection becomes a VantaDB namespace (unless ``namespace``
    is given, in which case all collections share it). ChromaDB ids become
    keys, documents become payloads, and metadatas/embeddings are preserved.
    """
    try:
        import chromadb
    except ImportError:
        raise ImportError("chromadb is required for this migration: pip install chromadb")

    from vantadb_py import VantaDB

    client = chromadb.PersistentClient(path=source_path)
    target = VantaDB(dest_path)

    total = 0
    for name in _collection_names(client, collection_name):
        collection = client.get_collection(name)
        ns = namespace or name

        offset = 0
        while True:
            data = collection.get(
                limit=batch_size,
                offset=offset,
                include=["documents", "metadatas", "embeddings"],
            )
            ids = data["ids"]
            if not ids:
                break

            documents = data.get("documents")
            metadatas = data.get("metadatas")
            embeddings = data.get("embeddings")
            if documents is None or len(documents) == 0:
                documents = [None] * len(ids)
            if metadatas is None or len(metadatas) == 0:
                metadatas = [None] * len(ids)
            if embeddings is None or len(embeddings) == 0:
                embeddings = [None] * len(ids)

            for i, key in enumerate(ids):
                vector = list(embeddings[i]) if embeddings[i] is not None else None
                target.put(
                    ns,
                    str(key),
                    documents[i] or "",
                    metadata=metadatas[i],
                    vector=vector,
                )
            total += len(ids)
            offset += len(ids)

    target.flush()
    return total


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="python -m vantadb_py.migrate.chroma",
        description="Migrate a ChromaDB persistent store into VantaDB.",
    )
    parser.add_argument("--source", required=True, help="Path to the ChromaDB persistent store")
    parser.add_argument("--dest", required=True, help="Path where the VantaDB database will be created")
    parser.add_argument("--collection-name", default=None, help="Migrate only this collection (default: all)")
    parser.add_argument("--namespace", default=None, help="Target VantaDB namespace (default: collection name)")
    parser.add_argument("--batch-size", type=int, default=500, help="Records per batch (default: 500)")
    args = parser.parse_args(argv)

    count = migrate_from_chroma(
        args.source,
        args.dest,
        collection_name=args.collection_name,
        namespace=args.namespace,
        batch_size=args.batch_size,
    )
    print(f"Migrated {count} records from ChromaDB into VantaDB at {args.dest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
