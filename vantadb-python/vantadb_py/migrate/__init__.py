"""Migration tools: import data from ChromaDB or LanceDB into VantaDB.

Usage:
    python -m vantadb_py.migrate chroma --source ./chroma_db --target ./vantadb_db
    python -m vantadb_py.migrate lancedb --source ./lancedb_dir --target ./vantadb_db
"""

from .chroma import migrate_from_chromadb
from .lancedb import migrate_from_lancedb

__all__ = ["migrate_from_chromadb", "migrate_from_lancedb"]


def main():
    import argparse
    import sys

    parser = argparse.ArgumentParser(description="Migrate vector DB data into VantaDB")
    sub = parser.add_subparsers(dest="command", required=True)

    chroma_p = sub.add_parser("chroma", help="Migrate from ChromaDB")
    chroma_p.add_argument("--source", required=True, help="ChromaDB persist directory or `:memory:`")
    chroma_p.add_argument("--target", required=True, help="VantaDB storage path")
    chroma_p.add_argument("--batch-size", type=int, default=1000, help="Records per batch")

    lancedb_p = sub.add_parser("lancedb", help="Migrate from LanceDB")
    lancedb_p.add_argument("--source", required=True, help="LanceDB dataset URI (directory or s3://...)")
    lancedb_p.add_argument("--target", required=True, help="VantaDB storage path")
    lancedb_p.add_argument("--batch-size", type=int, default=1000, help="Records per batch")

    args = parser.parse_args()

    if args.command == "chroma":
        count = migrate_from_chromadb(args.source, args.target, args.batch_size)
    else:
        count = migrate_from_lancedb(args.source, args.target, args.batch_size)

    print(f"✅ Migrated {count} records into VantaDB at {args.target}")


if __name__ == "__main__":
    main()
