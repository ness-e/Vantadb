def migrate_from_lancedb(source: str, target: str, batch_size: int = 1000) -> int:
    try:
        import lancedb
    except ImportError:
        raise ImportError("Install lancedb: pip install lancedb")

    db = lancedb.connect(source)
    target_db = __import__("vantadb_py").VantaDB(target)

    total = 0
    for table_name in db.table_names():
        table = db.open_table(table_name)
        n = table.count_rows()
        offset = 0
        while offset < n:
            batch_rows = table.to_lance().to_batches({"batch_size": batch_size, "offset": offset}).read_all().to_pylist()
            if not batch_rows:
                break
            entries = []
            for row in batch_rows:
                doc_id = row.get("id") or row.get("_id") or str(hash(str(row)))
                payload = row.get("text") or row.get("content") or row.get("payload") or ""
                meta = {k: v for k, v in row.items() if k not in ("id", "_id", "text", "content", "payload", "vector", "_vector")}
                vec = row.get("vector") or row.get("_vector")
                entries.append((
                    table_name,
                    str(doc_id),
                    payload,
                    meta if meta else None,
                    vec if vec else None,
                    None,
                ))
            target_db.put_batch(entries)
            total += len(batch_rows)
            offset += batch_size

    target_db.flush()
    return total
