"""Migration tools: import data from ChromaDB or LanceDB into VantaDB.

Each migration script is runnable standalone:

    python -m vantadb_py.migrate.chroma --source ./chroma_db --dest ./vantadb_db
    python -m vantadb_py.migrate.lancedb --source ./lancedb_dir --dest ./vantadb_db

or used programmatically:

    from vantadb_py.migrate import migrate_from_chroma, migrate_from_lancedb

    count = migrate_from_chroma("./chroma_db", "./vantadb_db")
    count = migrate_from_lancedb("./lancedb_dir", "./vantadb_db")
"""

def migrate_from_chroma(*args, **kwargs):
    """Migrate a ChromaDB persistent store into VantaDB. See ``vantadb_py.migrate.chroma``."""
    from .chroma import migrate_from_chroma as fn

    return fn(*args, **kwargs)


def migrate_from_lancedb(*args, **kwargs):
    """Migrate a LanceDB dataset into VantaDB. See ``vantadb_py.migrate.lancedb``."""
    from .lancedb import migrate_from_lancedb as fn

    return fn(*args, **kwargs)


__all__ = ["migrate_from_chroma", "migrate_from_lancedb"]
