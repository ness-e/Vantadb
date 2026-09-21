"""Migrate a LanceDB dataset into VantaDB.

Runnable standalone:

    python -m vantadb_py.migrate.lancedb --source ./lancedb_dir --dest ./vantadb_db
    python -m vantadb_py.migrate.lancedb --source ./lancedb_dir --dest ./vantadb_db \
        --table-name my_table --namespace docs

or programmatically:

    from vantadb_py.migrate import migrate_from_lancedb
    count = migrate_from_lancedb("./lancedb_dir", "./vantadb_db")
"""

from __future__ import annotations

import argparse
import sys


def _table_names(db, table_name: str | None) -> list[str]:
    """Return the LanceDB table names to migrate."""
    if table_name:
        return [table_name]
    # list_tables() returns a namedtuple with a .tables field; table_names() is
    # deprecated but kept for older lancedb versions.
    tables = db.list_tables()
    if hasattr(tables, "tables"):
        return list(tables.tables)
    return list(tables)


def _pick(row: dict, *keys: str):
    """Return the first present value among keys, or None."""
    for key in keys:
        if row.get(key) is not None:
            return row[key]
    return None


def migrate_from_lancedb(
    source_path: str,
    dest_path: str,
    table_name: str | None = None,
    namespace: str | None = None,
    batch_size: int = 500,
) -> int:
    """Copy every row from a LanceDB dataset into VantaDB.

    Each LanceDB table becomes a VantaDB namespace (unless ``namespace`` is
    given). Columns named ``id``/``_id`` become keys, text-ish columns
    (``text``/``content``/``payload``) become payloads, ``vector``/``_vector``
    become vectors, and every remaining column is stored as metadata.
    """
    try:
        import lancedb
    except ImportError:
        raise ImportError("lancedb is required for this migration: pip install lancedb")

    from vantadb_py import VantaDB

    db = lancedb.connect(source_path)
    target = VantaDB(dest_path)

    total = 0
    for name in _table_names(db, table_name):
        table = db.open_table(name)
        ns = namespace or name

        rows = table.to_arrow().to_pylist()
        for i in range(0, len(rows), batch_size):
            batch = rows[i : i + batch_size]
            for row in batch:
                key = str(_pick(row, "id", "_id") or f"row_{i}")
                payload = _pick(row, "text", "content", "payload") or ""
                vector = _pick(row, "vector", "_vector")
                vector = list(vector) if vector is not None else None
                metadata = {
                    k: v
                    for k, v in row.items()
                    if k not in ("id", "_id", "text", "content", "payload", "vector", "_vector")
                }
                target.put(ns, key, payload, metadata=metadata or None, vector=vector)
            total += len(batch)

    target.flush()
    return total


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="python -m vantadb_py.migrate.lancedb",
        description="Migrate a LanceDB dataset into VantaDB.",
    )
    parser.add_argument("--source", required=True, help="Path to the LanceDB dataset directory")
    parser.add_argument("--dest", required=True, help="Path where the VantaDB database will be created")
    parser.add_argument("--table-name", default=None, help="Migrate only this table (default: all)")
    parser.add_argument("--namespace", default=None, help="Target VantaDB namespace (default: table name)")
    parser.add_argument("--batch-size", type=int, default=500, help="Records per batch (default: 500)")
    args = parser.parse_args(argv)

    count = migrate_from_lancedb(
        args.source,
        args.dest,
        table_name=args.table_name,
        namespace=args.namespace,
        batch_size=args.batch_size,
    )
    print(f"Migrated {count} records from LanceDB into VantaDB at {args.dest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
