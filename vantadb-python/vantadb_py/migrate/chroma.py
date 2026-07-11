def migrate_from_chromadb(source: str, target: str, batch_size: int = 1000) -> int:
    try:
        import chromadb
    except ImportError:
        raise ImportError("Install chromadb: pip install chromadb")

    client = chromadb.PersistentClient(path=source) if source != ":memory:" else chromadb.Client()
    target_db = __import__("vantadb_py").VantaDB(target)

    total = 0
    for collection in client.list_collections():
        offset = 0
        while True:
            results = collection.get(limit=batch_size, offset=offset, include=["metadatas", "documents", "embeddings"])
            if not results["ids"]:
                break
            entries = []
            for i, doc_id in enumerate(results["ids"]):
                entries.append((
                    collection.name,
                    doc_id,
                    results["documents"][i] or "",
                    results["metadatas"][i] if results["metadatas"] else None,
                    results["embeddings"][i] if results["embeddings"] else None,
                    None,
                ))
            target_db.put_batch(entries)
            total += len(results["ids"])
            offset += batch_size

    target_db.flush()
    return total
